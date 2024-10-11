// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"log"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/ackseq"
	"github.com/cockroachdb/pebble/internal/randvar"
	"github.com/cockroachdb/pebble/internal/rate"
	"github.com/spf13/cobra"
	"golang.org/x/exp/rand"
)

const (
	ycsbInsert = iota
	ycsbRead
	ycsbScan
	ycsbReverseScan
	ycsbUpdate
	ycsbNumOps
)

var ycsbConfig struct {
	batch            *randvar.Flag
	keys             string
	initialKeys      int
	prepopulatedKeys int
	numOps           uint64
	scans            *randvar.Flag
	values           *randvar.BytesFlag
	workload         string
}

var ycsbCmd = &cobra.Command{
	Use:   "ycsb <dir>",
	Short: "run customizable YCSB benchmark",
	Long: `
Run a customizable YCSB workload. The workload is specified by the --workload
flag which can take either one of the standard workload mixes (A-F), or
customizable workload fixes specified as a command separated list of op=weight
pairs. For example, --workload=read=50,update=50 performs a workload composed
of 50% reads and 50% updates. This is identical to the standard workload A.

The --batch, --scans, and --values flags take the specification for a random
variable: [<type>:]<min>[-<max>]. The <type> parameter must be one of "uniform"
or "zipf". If <type> is omitted, a uniform distribution is used. If <max> is
omitted it is set to the same value as <min>. The specification "1000" results
in a constant 1000. The specification "10-100" results in a uniformly random
variable in the range [10,100). The specification "zipf(10,100)" results in a
zipf distribution with a minimum value of 10 and a maximum value of 100.

The --batch flag controls the size of batches used for insert and update
operations. The --scans flag controls the number of iterations performed by a
scan operation. Read operations always read a single key.

The --values flag provides for an optional "/<target-compression-ratio>"
suffix. The default target compression ratio is 1.0 (i.e. incompressible random
data). A value of 2 will cause random data to be generated that should compress
to 50% of its uncompressed size.

Standard workloads:

  A:  50% reads   /  50% updates
  B:  95% reads   /   5% updates
  C: 100% reads
  D:  95% reads   /   5% inserts
  E:  95% scans   /   5% inserts
  F: 100% inserts
`,
	Args: cobra.ExactArgs(1),
	RunE: runYcsb,
}

func init() {
	initYCSB(ycsbCmd)
}

func initYCSB(cmd *cobra.Command) {
	ycsbConfig.batch = randvar.NewFlag("1")
	cmd.Flags().Var(
		ycsbConfig.batch, "batch",
		"batch size distribution [{zipf,uniform}:]min[-max]")
	cmd.Flags().StringVar(
		&ycsbConfig.keys, "keys", "zipf", "latest, uniform, or zipf")
	cmd.Flags().IntVar(
		&ycsbConfig.initialKeys, "initial-keys", 10000,
		"initial number of keys to insert before beginning workload")
	cmd.Flags().IntVar(
		&ycsbConfig.prepopulatedKeys, "prepopulated-keys", 0,
		"number of keys that were previously inserted into the database")
	cmd.Flags().Uint64VarP(
		&ycsbConfig.numOps, "num-ops", "n", 0,
		"maximum number of operations (0 means unlimited)")
	ycsbConfig.scans = randvar.NewFlag("zipf:1-1000")
	cmd.Flags().Var(
		ycsbConfig.scans, "scans",
		"scan length distribution [{zipf,uniform}:]min[-max]")
	cmd.Flags().StringVar(
		&ycsbConfig.workload, "workload", "B",
		"workload type (A-F) or spec (read=X,update=Y,...)")
	ycsbConfig.values = randvar.NewBytesFlag("1000")
	cmd.Flags().Var(
		ycsbConfig.values, "values",
		"value size distribution [{zipf,uniform}:]min[-max][/<target-compression>]")
}

type ycsbWeights []float64

func (w ycsbWeights) get(i int) float64 {
	if i >= len(w) {
		return 0
	}
	return w[i]
}

var ycsbWorkloads = map[string]ycsbWeights{
	"A": {
		ycsbRead:   0.5,
		ycsbUpdate: 0.5,
	},
	"B": {
		ycsbRead:   0.95,
		ycsbUpdate: 0.05,
	},
	"C": {
		ycsbRead: 1.0,
	},
	"D": {
		ycsbInsert: 0.05,
		ycsbRead:   0.95,
		// TODO(peter): default to skewed-latest distribution.
	},
	"E": {
		ycsbInsert: 0.05,
		ycsbScan:   0.95,
	},
	"F": {
		ycsbInsert: 1.0,
		// TODO(peter): the real workload is read-modify-write.
	},
}

func ycsbParseWorkload(w string) (ycsbWeights, error) {
	if weights := ycsbWorkloads[w]; weights != nil {
		return weights, nil
	}
	iWeights := make([]int, ycsbNumOps)
	for _, p := range strings.Split(w, ",") {
		parts := strings.Split(p, "=")
		if len(parts) != 2 {
			return nil, errors.Errorf("malformed weights: %s", errors.Safe(w))
		}
		weight, err := strconv.Atoi(parts[1])
		if err != nil {
			return nil, err
		}
		switch parts[0] {
		case "insert":
			iWeights[ycsbInsert] = weight
		case "read":
			iWeights[ycsbRead] = weight
		case "scan":
			iWeights[ycsbScan] = weight
		case "rscan":
			iWeights[ycsbReverseScan] = weight
		case "update":
			iWeights[ycsbUpdate] = weight
		}
	}

	var sum int
	for _, w := range iWeights {
		sum += w
	}
	if sum == 0 {
		return nil, errors.Errorf("zero weight specified: %s", errors.Safe(w))
	}

	weights := make(ycsbWeights, ycsbNumOps)
	for i := range weights {
		weights[i] = float64(iWeights[i]) / float64(sum)
	}
	return weights, nil
}

func ycsbParseKeyDist(d string) (randvar.Dynamic, error) {
	totalKeys := uint64(ycsbConfig.initialKeys + ycsbConfig.prepopulatedKeys)
	switch strings.ToLower(d) {
	case "latest":
		return randvar.NewDefaultSkewedLatest()
	case "uniform":
		return randvar.NewUniform(1, totalKeys), nil
	case "zipf":
		return randvar.NewZipf(1, totalKeys, 0.99)
	default:
		return nil, errors.Errorf("unknown distribution: %s", errors.Safe(d))
	}
}

func runYcsb(cmd *cobra.Command, args []string) error {
	if wipe && ycsbConfig.prepopulatedKeys > 0 {
		return errors.New("--wipe and --prepopulated-keys both specified which is nonsensical")
	}

	weights, err := ycsbParseWorkload(ycsbConfig.workload)
	if err != nil {
		return err
	}

	keyDist, err := ycsbParseKeyDist(ycsbConfig.keys)
	if err != nil {
		return err
	}

	batchDist := ycsbConfig.batch
	scanDist := ycsbConfig.scans
	if err != nil {
		return err
	}

	valueDist := ycsbConfig.values
	y := newYcsb(weights, keyDist, batchDist, scanDist, valueDist)
	runTest(args[0], test{
		init: y.init,
		tick: y.tick,
		done: y.done,
	})
	return nil
}

type ycsbBuf struct {
	rng      *rand.Rand
	keyBuf   []byte
	valueBuf []byte
	keyNums  []uint64
}

type ycsb struct {
	db           DB
	writeOpts    *pebble.WriteOptions
	weights      ycsbWeights
	reg          *histogramRegistry
	keyDist      randvar.Dynamic
	batchDist    randvar.Static
	scanDist     randvar.Static
	valueDist    *randvar.BytesFlag
	readAmpCount atomic.Uint64
	readAmpSum   atomic.Uint64
	keyNum       *ackseq.S
	numOps       atomic.Uint64
	limiter      *rate.Limiter
	opsMap       map[string]int
}

func newYcsb(
	weights ycsbWeights,
	keyDist randvar.Dynamic,
	batchDist, scanDist randvar.Static,
	valueDist *randvar.BytesFlag,
) *ycsb {
	y := &ycsb{
		reg:       newHistogramRegistry(),
		weights:   weights,
		keyDist:   keyDist,
		batchDist: batchDist,
		scanDist:  scanDist,
		valueDist: valueDist,
		opsMap:    make(map[string]int),
	}
	y.writeOpts = pebble.Sync
	if disableWAL {
		y.writeOpts = pebble.NoSync
	}

	ops := map[string]int{
		"insert": ycsbInsert,
		"read":   ycsbRead,
		"rscan":  ycsbReverseScan,
		"scan":   ycsbScan,
		"update": ycsbUpdate,
	}
	for name, op := range ops {
		w := y.weights.get(op)
		if w == 0 {
			continue
		}
		wstr := fmt.Sprint(int(100 * w))
		fill := strings.Repeat("_", 3-len(wstr))
		if fill == "" {
			fill = "_"
		}
		fullName := fmt.Sprintf("%s%s%s", name, fill, wstr)
		y.opsMap[fullName] = op
	}
	return y
}

func (y *ycsb) init(db DB, wg *sync.WaitGroup) {
	y.db = db

	if ycsbConfig.initialKeys > 0 {
		buf := &ycsbBuf{rng: randvar.NewRand()}

		b := db.NewBatch()
		size := 0
		start := time.Now()
		last := start
		for i := 1; i <= ycsbConfig.initialKeys; i++ {
			if now := time.Now(); now.Sub(last) >= time.Second {
				fmt.Printf("%5s inserted %d keys (%0.1f%%)\n",
					time.Duration(now.Sub(start).Seconds()+0.5)*time.Second,
					i-1, 100*float64(i-1)/float64(ycsbConfig.initialKeys))
				last = now
			}
			if size >= 1<<20 {
				if err := b.Commit(y.writeOpts); err != nil {
					log.Fatal(err)
				}
				b = db.NewBatch()
				size = 0
			}
			key := y.makeKey(uint64(i+ycsbConfig.prepopulatedKeys), buf)
			value := y.randBytes(buf)
			if err := b.Set(key, value, nil); err != nil {
				log.Fatal(err)
			}
			size += len(key) + len(value)
		}
		if err := b.Commit(y.writeOpts); err != nil {
			log.Fatal(err)
		}
		_ = b.Close()
		fmt.Printf("inserted keys [%d-%d)\n",
			1+ycsbConfig.prepopulatedKeys,
			1+ycsbConfig.prepopulatedKeys+ycsbConfig.initialKeys)
	}
	y.keyNum = ackseq.New(uint64(ycsbConfig.initialKeys + ycsbConfig.prepopulatedKeys))

	y.limiter = maxOpsPerSec.newRateLimiter()

	wg.Add(concurrency)

	// If this workload doesn't produce reads, sample the worst case read-amp
	// from Metrics() periodically.
	if y.weights.get(ycsbRead) == 0 && y.weights.get(ycsbScan) == 0 && y.weights.get(ycsbReverseScan) == 0 {
		wg.Add(1)
		go y.sampleReadAmp(db, wg)
	}

	for i := 0; i < concurrency; i++ {
		go y.run(db, wg)
	}
}

func (y *ycsb) run(db DB, wg *sync.WaitGroup) {
	defer wg.Done()

	var latency [ycsbNumOps]*namedHistogram
	for name, op := range y.opsMap {
		latency[op] = y.reg.Register(name)
	}

	buf := &ycsbBuf{rng: randvar.NewRand()}

	ops := randvar.NewWeighted(nil, y.weights...)
	for {
		wait(y.limiter)

		start := time.Now()

		op := ops.Int()
		switch op {
		case ycsbInsert:
			y.insert(db, buf)
		case ycsbRead:
			y.read(db, buf)
		case ycsbScan:
			y.scan(db, buf, false /* reverse */)
		case ycsbReverseScan:
			y.scan(db, buf, true /* reverse */)
		case ycsbUpdate:
			y.update(db, buf)
		default:
			panic("not reached")
		}

		latency[op].Record(time.Since(start))
		if ycsbConfig.numOps > 0 && y.numOps.Add(1) >= ycsbConfig.numOps {
			break
		}
	}
}

func (y *ycsb) sampleReadAmp(db DB, wg *sync.WaitGroup) {
	defer wg.Done()

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()
	for range ticker.C {
		m := db.Metrics()
		y.readAmpCount.Add(1)
		y.readAmpSum.Add(uint64(m.ReadAmp()))
		if ycsbConfig.numOps > 0 && y.numOps.Load() >= ycsbConfig.numOps {
			break
		}
	}
}

func (y *ycsb) hashKey(key uint64) uint64 {
	// Inlined version of fnv.New64 + Write.
	const offset64 = 14695981039346656037
	const prime64 = 1099511628211

	h := uint64(offset64)
	for i := 0; i < 8; i++ {
		h *= prime64
		h ^= uint64(key & 0xff)
		key >>= 8
	}
	return h
}

func (y *ycsb) makeKey(keyNum uint64, buf *ycsbBuf) []byte {
	const size = 24 + 10
	if cap(buf.keyBuf) < size {
		buf.keyBuf = make([]byte, size)
	}
	key := buf.keyBuf[:4]
	copy(key, "user")
	key = strconv.AppendUint(key, y.hashKey(keyNum), 10)
	// Use the MVCC encoding for keys. This appends a timestamp with
	// walltime=1. That knowledge is utilized by rocksDB.Scan.
	key = append(key, '\x00', '\x00', '\x00', '\x00', '\x00',
		'\x00', '\x00', '\x00', '\x01', '\x09')
	buf.keyBuf = key
	return key
}

func (y *ycsb) nextReadKey(buf *ycsbBuf) []byte {
	// NB: the range of values returned by keyDist is tied to the range returned
	// by keyNum.Base. See how these are both incremented by ycsb.insert().
	keyNum := y.keyDist.Uint64(buf.rng)
	return y.makeKey(keyNum, buf)
}

func (y *ycsb) randBytes(buf *ycsbBuf) []byte {
	buf.valueBuf = y.valueDist.Bytes(buf.rng, buf.valueBuf)
	return buf.valueBuf
}

func (y *ycsb) insert(db DB, buf *ycsbBuf) {
	count := y.batchDist.Uint64(buf.rng)
	if cap(buf.keyNums) < int(count) {
		buf.keyNums = make([]uint64, count)
	}
	keyNums := buf.keyNums[:count]

	b := db.NewBatch()
	for i := range keyNums {
		keyNums[i] = y.keyNum.Next()
		_ = b.Set(y.makeKey(keyNums[i], buf), y.randBytes(buf), nil)
	}
	if err := b.Commit(y.writeOpts); err != nil {
		log.Fatal(err)
	}
	_ = b.Close()

	for i := range keyNums {
		delta, err := y.keyNum.Ack(keyNums[i])
		if err != nil {
			log.Fatal(err)
		}
		if delta > 0 {
			y.keyDist.IncMax(delta)
		}
	}
}

func (y *ycsb) read(db DB, buf *ycsbBuf) {
	key := y.nextReadKey(buf)
	iter := db.NewIter(nil)
	iter.SeekGE(key)
	if iter.Valid() {
		_ = iter.Key()
		_ = iter.Value()
	}

	type metrics interface {
		Metrics() pebble.IteratorMetrics
	}
	if m, ok := iter.(metrics); ok {
		y.readAmpCount.Add(1)
		y.readAmpSum.Add(uint64(m.Metrics().ReadAmp))
	}

	if err := iter.Close(); err != nil {
		log.Fatal(err)
	}
}

func (y *ycsb) scan(db DB, buf *ycsbBuf, reverse bool) {
	count := y.scanDist.Uint64(buf.rng)
	key := y.nextReadKey(buf)
	iter := db.NewIter(nil)
	if err := db.Scan(iter, key, int64(count), reverse); err != nil {
		log.Fatal(err)
	}

	type metrics interface {
		Metrics() pebble.IteratorMetrics
	}
	if m, ok := iter.(metrics); ok {
		y.readAmpCount.Add(1)
		y.readAmpSum.Add(uint64(m.Metrics().ReadAmp))
	}

	if err := iter.Close(); err != nil {
		log.Fatal(err)
	}
}

func (y *ycsb) update(db DB, buf *ycsbBuf) {
	count := int(y.batchDist.Uint64(buf.rng))
	b := db.NewBatch()
	for i := 0; i < count; i++ {
		_ = b.Set(y.nextReadKey(buf), y.randBytes(buf), nil)
	}
	if err := b.Commit(y.writeOpts); err != nil {
		log.Fatal(err)
	}
	_ = b.Close()
}

func (y *ycsb) tick(elapsed time.Duration, i int) {
	if i%20 == 0 {
		fmt.Println("____optype__elapsed__ops/sec(inst)___ops/sec(cum)__p50(ms)__p95(ms)__p99(ms)_pMax(ms)")
	}
	y.reg.Tick(func(tick histogramTick) {
		h := tick.Hist

		fmt.Printf("%10s %8s %14.1f %14.1f %8.1f %8.1f %8.1f %8.1f\n",
			tick.Name,
			time.Duration(elapsed.Seconds()+0.5)*time.Second,
			float64(h.TotalCount())/tick.Elapsed.Seconds(),
			float64(tick.Cumulative.TotalCount())/elapsed.Seconds(),
			time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(100)).Seconds()*1000,
		)
	})
}

func (y *ycsb) done(elapsed time.Duration) {
	fmt.Println("\n____optype__elapsed_____ops(total)___ops/sec(cum)__avg(ms)__p50(ms)__p95(ms)__p99(ms)_pMax(ms)")

	resultTick := histogramTick{}
	y.reg.Tick(func(tick histogramTick) {
		h := tick.Cumulative
		if resultTick.Cumulative == nil {
			resultTick.Now = tick.Now
			resultTick.Cumulative = h
		} else {
			resultTick.Cumulative.Merge(h)
		}

		fmt.Printf("%10s %7.1fs %14d %14.1f %8.1f %8.1f %8.1f %8.1f %8.1f\n",
			tick.Name, elapsed.Seconds(), h.TotalCount(),
			float64(h.TotalCount())/elapsed.Seconds(),
			time.Duration(h.Mean()).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(50)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(95)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(99)).Seconds()*1000,
			time.Duration(h.ValueAtQuantile(100)).Seconds()*1000)
	})
	fmt.Println()

	resultHist := resultTick.Cumulative
	m := y.db.Metrics()
	total := m.Total()

	readAmpCount := y.readAmpCount.Load()
	readAmpSum := y.readAmpSum.Load()
	if readAmpCount == 0 {
		readAmpSum = 0
		readAmpCount = 1
	}

	fmt.Printf("Benchmarkycsb/%s/values=%s %d  %0.1f ops/sec  %d read  %d write  %.2f r-amp  %0.2f w-amp\n\n",
		ycsbConfig.workload, ycsbConfig.values,
		resultHist.TotalCount(),
		float64(resultHist.TotalCount())/elapsed.Seconds(),
		total.BytesRead,
		total.BytesFlushed+total.BytesCompacted,
		float64(readAmpSum)/float64(readAmpCount),
		total.WriteAmp(),
	)
}
