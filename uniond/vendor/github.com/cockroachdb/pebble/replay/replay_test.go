package replay

import (
	"bytes"
	"context"
	"encoding/hex"
	"fmt"
	"io"
	"math/rand"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/datatest"
	"github.com/cockroachdb/pebble/internal/humanize"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/rangekey"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func runReplayTest(t *testing.T, path string) {
	fs := vfs.NewMem()
	var ctx context.Context
	var r Runner
	var ct *datatest.CompactionTracker
	datadriven.RunTest(t, path, func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "cat":
			var buf bytes.Buffer
			for _, arg := range td.CmdArgs {
				f, err := fs.Open(arg.String())
				if err != nil {
					fmt.Fprintf(&buf, "%s: %s\n", arg, err)
					continue
				}
				io.Copy(&buf, f)
				require.NoError(t, f.Close())
			}
			return buf.String()
		case "corpus":
			for _, arg := range td.CmdArgs {
				t.Run(fmt.Sprintf("corpus/%s", arg.String()), func(t *testing.T) {
					collectCorpus(t, fs, arg.String())
				})
			}
			return ""
		case "list-files":
			return runListFiles(t, fs, td)
		case "replay":
			name := td.CmdArgs[0].String()
			pacerVariant := td.CmdArgs[1].String()
			var pacer Pacer
			if pacerVariant == "reference" {
				pacer = PaceByReferenceReadAmp{}
			} else if pacerVariant == "fixed" {
				i, err := strconv.Atoi(td.CmdArgs[2].String())
				require.NoError(t, err)
				pacer = PaceByFixedReadAmp(i)
			} else {
				pacer = Unpaced{}
			}

			// Convert the testdata/replay:235 datadriven command position into
			// a run directory suffixed with the line number: eg, 'run-235'
			lineOffset := strings.LastIndexByte(td.Pos, ':')
			require.Positive(t, lineOffset)
			runDir := fmt.Sprintf("run-%s", td.Pos[lineOffset+1:])
			if err := fs.MkdirAll(runDir, os.ModePerm); err != nil {
				return err.Error()
			}

			checkpointDir := fs.PathJoin(name, "checkpoint")
			ok, err := vfs.Clone(fs, fs, checkpointDir, runDir)
			if err != nil {
				return err.Error()
			} else if !ok {
				return fmt.Sprintf("%q does not exist", checkpointDir)
			}

			opts := &pebble.Options{
				FS:                        fs,
				Comparer:                  testkeys.Comparer,
				FormatMajorVersion:        pebble.FormatRangeKeys,
				L0CompactionFileThreshold: 1,
			}
			setDefaultExperimentalOpts(opts)
			ct = datatest.NewCompactionTracker(opts)

			r = Runner{
				RunDir:       runDir,
				WorkloadFS:   fs,
				WorkloadPath: name,
				Pacer:        pacer,
				Opts:         opts,
			}
			ctx = context.Background()
			if err := r.Run(ctx); err != nil {
				return err.Error()
			}
			return ""
		case "scan-keys":
			var buf bytes.Buffer
			it, _ := r.d.NewIter(nil)
			defer it.Close()
			for valid := it.First(); valid; valid = it.Next() {
				fmt.Fprintf(&buf, "%s: %s\n", it.Key(), it.Value())
			}
			if err := it.Error(); err != nil {
				fmt.Fprintln(&buf, err.Error())
			}
			return buf.String()
		case "tree":
			return fs.String()
		case "wait-for-compactions":
			var target int
			if len(td.CmdArgs) == 1 {
				i, err := strconv.Atoi(td.CmdArgs[0].String())
				require.NoError(t, err)
				target = i
			}
			ct.WaitForInflightCompactionsToEqual(target)
			return ""
		case "wait":
			m, err := r.Wait()
			if err != nil {
				return err.Error()
			}
			return fmt.Sprintf("replayed %s in writes", humanize.Bytes.Uint64(m.WriteBytes))
		case "close":
			if err := r.Close(); err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func setDefaultExperimentalOpts(opts *pebble.Options) {
	opts.Experimental.TableCacheShards = 2
}

func TestReplay(t *testing.T) {
	runReplayTest(t, "testdata/replay")
}

func TestReplayPaced(t *testing.T) {
	runReplayTest(t, "testdata/replay_paced")
}

func TestLoadFlushedSSTableKeys(t *testing.T) {
	var buf bytes.Buffer
	var diskFileNums []base.DiskFileNum
	opts := &pebble.Options{
		DisableAutomaticCompactions: true,
		EventListener: &pebble.EventListener{
			FlushEnd: func(info pebble.FlushInfo) {
				for _, tbl := range info.Output {
					diskFileNums = append(diskFileNums, tbl.FileNum.DiskFileNum())
				}
			},
		},
		FS:                 vfs.NewMem(),
		Comparer:           testkeys.Comparer,
		FormatMajorVersion: pebble.FormatRangeKeys,
	}
	d, err := pebble.Open("", opts)
	require.NoError(t, err)
	defer d.Close()

	var flushBufs flushBuffers
	datadriven.RunTest(t, "testdata/flushed_sstable_keys", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "commit":
			b := d.NewIndexedBatch()
			if err := datatest.DefineBatch(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}

			b := d.NewBatch()
			err := loadFlushedSSTableKeys(b, opts.FS, "", diskFileNums, opts.MakeReaderOptions(), &flushBufs)
			if err != nil {
				b.Close()
				return err.Error()
			}

			br, _ := pebble.ReadBatch(b.Repr())
			kind, ukey, v, ok, err := br.Next()
			for ; ok; kind, ukey, v, ok, err = br.Next() {
				fmt.Fprintf(&buf, "%s.%s", ukey, kind)
				switch kind {
				case base.InternalKeyKindRangeDelete,
					base.InternalKeyKindRangeKeyDelete:
					fmt.Fprintf(&buf, "-%s", v)
				case base.InternalKeyKindSet,
					base.InternalKeyKindMerge:
					fmt.Fprintf(&buf, ": %s", v)
				case base.InternalKeyKindRangeKeySet, base.InternalKeyKindRangeKeyUnset:
					s, err := rangekey.Decode(base.MakeInternalKey(ukey, 0, kind), v, nil)
					if err != nil {
						return err.Error()
					}
					if kind == base.InternalKeyKindRangeKeySet {
						fmt.Fprintf(&buf, "-%s: %s → %s", s.End, s.Keys[0].Suffix, s.Keys[0].Value)
					} else {
						fmt.Fprintf(&buf, "-%s: %s", s.End, s.Keys[0].Suffix)
					}
				case base.InternalKeyKindDelete, base.InternalKeyKindSingleDelete:
				default:
					fmt.Fprintf(&buf, ": %x", v)
				}
				fmt.Fprintln(&buf)
			}
			if err != nil {
				fmt.Fprintf(&buf, "err: %s\n", err)
			}

			s := buf.String()
			buf.Reset()
			require.NoError(t, b.Close())

			diskFileNums = diskFileNums[:0]
			return s
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func collectCorpus(t *testing.T, fs *vfs.MemFS, name string) {
	require.NoError(t, fs.RemoveAll("build"))
	require.NoError(t, fs.MkdirAll("build", os.ModePerm))

	var d *pebble.DB
	var wc *WorkloadCollector
	defer func() {
		if d != nil {
			require.NoError(t, d.Close())
		}
	}()
	datadriven.RunTest(t, filepath.Join("testdata", "corpus", name), func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "commit":
			b := d.NewBatch()
			if err := datatest.DefineBatch(td, b); err != nil {
				return err.Error()
			}
			if err := b.Commit(nil); err != nil {
				return err.Error()
			}
			return ""
		case "flush":
			require.NoError(t, d.Flush())
			return ""
		case "list-files":
			if d != nil {
				d.TestOnlyWaitForCleaning()
			}
			return runListFiles(t, fs, td)
		case "open":
			wc = NewWorkloadCollector("build")
			opts := &pebble.Options{
				Comparer:                    testkeys.Comparer,
				DisableAutomaticCompactions: true,
				FormatMajorVersion:          pebble.FormatRangeKeys,
				FS:                          fs,
				MaxManifestFileSize:         96,
			}
			setDefaultExperimentalOpts(opts)
			wc.Attach(opts)
			var err error
			d, err = pebble.Open("build", opts)
			require.NoError(t, err)
			return ""
		case "close":
			err := d.Close()
			require.NoError(t, err)
			d = nil
			return ""
		case "start":
			require.NoError(t, fs.MkdirAll(name, os.ModePerm))
			require.NotNil(t, wc)
			wc.Start(fs, name)
			require.NoError(t, d.Checkpoint(fs.PathJoin(name, "checkpoint"), pebble.WithFlushedWAL()))
			return "started"
		case "stat":
			var buf bytes.Buffer
			for _, arg := range td.CmdArgs {
				fi, err := fs.Stat(arg.String())
				if err != nil {
					fmt.Fprintf(&buf, "%s: %s\n", arg.String(), err)
					continue
				}
				fmt.Fprintf(&buf, "%s:\n", arg.String())
				fmt.Fprintf(&buf, "  size: %d\n", fi.Size())
			}
			return buf.String()
		case "stop":
			wc.mu.Lock()
			for wc.mu.tablesEnqueued != wc.mu.tablesCopied {
				wc.mu.copyCond.Wait()
			}
			wc.mu.Unlock()
			wc.Stop()
			return "stopped"
		case "tree":
			return fs.String()
		case "make-file":
			dir := td.CmdArgs[0].String()
			require.NoError(t, fs.MkdirAll(dir, os.ModePerm))
			fT := td.CmdArgs[1].String()
			filePath := fs.PathJoin(dir, td.CmdArgs[2].String())

			if fT != "file" {
				fileNumInt, err := strconv.Atoi(td.CmdArgs[2].String())
				require.NoError(t, err)
				fileNum := base.FileNum(fileNumInt)
				switch fT {
				case "table":
					filePath = base.MakeFilepath(fs, dir, base.FileTypeTable, fileNum.DiskFileNum())
				case "log":
					filePath = base.MakeFilepath(fs, dir, base.FileTypeLog, fileNum.DiskFileNum())
				case "manifest":
					filePath = base.MakeFilepath(fs, dir, base.FileTypeManifest, fileNum.DiskFileNum())
				}
			}
			f, err := fs.Create(filePath)
			require.NoError(t, err)
			b, err := hex.DecodeString(strings.ReplaceAll(td.Input, "\n", ""))
			require.NoError(t, err)
			_, err = f.Write(b)
			require.NoError(t, err)
			return "created"
		case "find-workload-files":
			var buf bytes.Buffer
			dir := td.CmdArgs[0].String()
			m, s, err := findWorkloadFiles(dir, fs)

			fmt.Fprintln(&buf, "manifests")
			sort.Strings(m)
			for _, elem := range m {
				fmt.Fprintf(&buf, "  %s\n", elem)
			}
			var res []string
			for key := range s {
				res = append(res, key.String())
			}
			sort.Strings(res)

			fmt.Fprintln(&buf, "sstables")
			for _, elem := range res {
				fmt.Fprintf(&buf, "  %s\n", elem)
			}
			fmt.Fprintln(&buf, "error")
			if err != nil {
				fmt.Fprintf(&buf, "  %s\n", err.Error())
			}
			return buf.String()
		case "find-manifest-start":
			var buf bytes.Buffer
			dir := td.CmdArgs[0].String()
			m, _, err := findWorkloadFiles(dir, fs)
			sort.Strings(m)
			require.NoError(t, err)
			i, o, err := findManifestStart(dir, fs, m)
			errString := "nil"
			if err != nil {
				errString = err.Error()
			}
			fmt.Fprintf(&buf, "index: %d, offset: %d, error: %s\n", i, o, errString)
			return buf.String()
		case "delete-all":
			err := fs.RemoveAll(td.CmdArgs[0].String())
			if err != nil {
				return err.Error()
			}
			return ""
		default:
			return fmt.Sprintf("unrecognized command %q", td.Cmd)
		}
	})
}

func TestCollectCorpus(t *testing.T) {
	fs := vfs.NewMem()
	datadriven.Walk(t, "testdata/corpus", func(t *testing.T, path string) {
		collectCorpus(t, fs, filepath.Base(path))
		fs = vfs.NewMem()
	})
}

func runListFiles(t *testing.T, fs vfs.FS, td *datadriven.TestData) string {
	var buf bytes.Buffer
	for _, arg := range td.CmdArgs {
		listFiles(t, fs, &buf, arg.String())
	}
	return buf.String()
}

func TestBenchmarkString(t *testing.T) {
	m := Metrics{
		Final:               &pebble.Metrics{},
		EstimatedDebt:       SampledMetric{samples: []sample{{value: 5 << 25}}},
		PaceDuration:        time.Second / 4,
		QuiesceDuration:     time.Second / 2,
		ReadAmp:             SampledMetric{samples: []sample{{value: 10}}},
		TombstoneCount:      SampledMetric{samples: []sample{{value: 295}}},
		TotalSize:           SampledMetric{samples: []sample{{value: 5 << 30}}},
		TotalWriteAmp:       5.6,
		WorkloadDuration:    time.Second,
		WriteBytes:          30 * (1 << 20),
		WriteStalls:         map[string]int{"memtable": 1, "L0": 2},
		WriteStallsDuration: map[string]time.Duration{"memtable": time.Minute, "L0": time.Hour},
	}
	m.Ingest.BytesIntoL0 = 5 << 20
	m.Ingest.BytesWeightedByLevel = 9 << 20

	var buf bytes.Buffer
	require.NoError(t, m.WriteBenchmarkString("tpcc", &buf))
	require.Equal(t, strings.TrimSpace(`
BenchmarkBenchmarkReplay/tpcc/CompactionCounts 1 0 compactions 0 default 0 delete 0 elision 0 move 0 read 0 rewrite 0 multilevel
BenchmarkBenchmarkReplay/tpcc/DatabaseSize/mean 1 5.36870912e+09 bytes
BenchmarkBenchmarkReplay/tpcc/DatabaseSize/max 1 5.36870912e+09 bytes
BenchmarkBenchmarkReplay/tpcc/DurationWorkload 1 1 sec/op
BenchmarkBenchmarkReplay/tpcc/DurationQuiescing 1 0.5 sec/op
BenchmarkBenchmarkReplay/tpcc/DurationPaceDelay 1 0.25 sec/op
BenchmarkBenchmarkReplay/tpcc/EstimatedDebt/mean 1 1.6777216e+08 bytes
BenchmarkBenchmarkReplay/tpcc/EstimatedDebt/max 1 1.6777216e+08 bytes
BenchmarkBenchmarkReplay/tpcc/FlushUtilization 1 0 util
BenchmarkBenchmarkReplay/tpcc/IngestedIntoL0 1 5.24288e+06 bytes
BenchmarkBenchmarkReplay/tpcc/IngestWeightedByLevel 1 9.437184e+06 bytes
BenchmarkBenchmarkReplay/tpcc/ReadAmp/mean 1 10 files
BenchmarkBenchmarkReplay/tpcc/ReadAmp/max 1 10 files
BenchmarkBenchmarkReplay/tpcc/TombstoneCount/mean 1 295 tombstones
BenchmarkBenchmarkReplay/tpcc/TombstoneCount/max 1 295 tombstones
BenchmarkBenchmarkReplay/tpcc/Throughput 1 2.097152e+07 B/s
BenchmarkBenchmarkReplay/tpcc/WriteAmp 1 5.6 wamp
BenchmarkBenchmarkReplay/tpcc/WriteStall/L0 1 2 stalls 3600 stallsec/op
BenchmarkBenchmarkReplay/tpcc/WriteStall/memtable 1 1 stalls 60 stallsec/op`),
		strings.TrimSpace(buf.String()))
}

func listFiles(t *testing.T, fs vfs.FS, w io.Writer, name string) {
	ls, err := fs.List(name)
	if err != nil {
		fmt.Fprintf(w, "%s: %s\n", name, err)
		return
	}
	sort.Strings(ls)
	fmt.Fprintf(w, "%s:\n", name)
	for _, dirent := range ls {
		fmt.Fprintf(w, "  %s\n", dirent)
	}
}

// TestCompactionsQuiesce replays a workload that produces a nontrivial number of
// compactions several times. It's intended to exercise Waits termination, which
// is dependent on compactions quiescing.
func TestCompactionsQuiesce(t *testing.T) {
	const replayCount = 1
	workloadFS := getHeavyWorkload(t)
	fs := vfs.NewMem()
	var done [replayCount]atomic.Bool
	for i := 0; i < replayCount; i++ {
		func(i int) {
			runDir := fmt.Sprintf("run%d", i)
			require.NoError(t, fs.MkdirAll(runDir, os.ModePerm))
			r := Runner{
				RunDir:       runDir,
				WorkloadFS:   workloadFS,
				WorkloadPath: "workload",
				Pacer:        Unpaced{},
				Opts: &pebble.Options{
					Comparer:           testkeys.Comparer,
					FS:                 fs,
					FormatMajorVersion: pebble.FormatNewest,
					LBaseMaxBytes:      1,
				},
			}
			r.Opts.Experimental.LevelMultiplier = 2
			require.NoError(t, r.Run(context.Background()))
			defer r.Close()

			var m Metrics
			var err error
			go func() {
				m, err = r.Wait()
				done[i].Store(true)
			}()

			wait := 30 * time.Second
			if invariants.Enabled {
				wait = time.Minute
			}

			// The above call to [Wait] should eventually return. [Wait] blocks
			// until the workload has replayed AND compactions have quiesced. A
			// bug in either could prevent [Wait] from ever returning.
			require.Eventually(t, func() bool { return done[i].Load() },
				wait, time.Millisecond, "(*replay.Runner).Wait didn't terminate")
			require.NoError(t, err)
			// Require at least 5 compactions.
			require.Greater(t, m.Final.Compact.Count, int64(5))
			require.Equal(t, int64(0), m.Final.Compact.NumInProgress)
			for l := 0; l < len(m.Final.Levels)-1; l++ {
				require.Less(t, m.Final.Levels[l].Score, 1.0)
			}
		}(i)
	}
}

// getHeavyWorkload returns a FS containing a workload in the `workload`
// directory that flushes enough randomly generated keys that replaying it
// should generate a non-trivial number of compactions.
func getHeavyWorkload(t *testing.T) vfs.FS {
	heavyWorkload.Once.Do(func() {
		t.Run("buildHeavyWorkload", func(t *testing.T) {
			heavyWorkload.fs = buildHeavyWorkload(t)
		})
	})
	return heavyWorkload.fs
}

var heavyWorkload struct {
	sync.Once
	fs vfs.FS
}

func buildHeavyWorkload(t *testing.T) vfs.FS {
	o := &pebble.Options{
		Comparer:           testkeys.Comparer,
		FS:                 vfs.NewMem(),
		FormatMajorVersion: pebble.FormatNewest,
	}
	wc := NewWorkloadCollector("")
	wc.Attach(o)
	d, err := pebble.Open("", o)
	require.NoError(t, err)

	destFS := vfs.NewMem()
	require.NoError(t, destFS.MkdirAll("workload", os.ModePerm))
	wc.Start(destFS, "workload")

	ks := testkeys.Alpha(5)
	var bufKey = make([]byte, ks.MaxLen())
	var bufVal [512]byte
	rng := rand.New(rand.NewSource(time.Now().UnixNano()))
	for i := 0; i < 100; i++ {
		b := d.NewBatch()
		for j := 0; j < 1000; j++ {
			rng.Read(bufVal[:])
			n := testkeys.WriteKey(bufKey[:], ks, rng.Int63n(ks.Count()))
			require.NoError(t, b.Set(bufKey[:n], bufVal[:], pebble.NoSync))
		}
		require.NoError(t, b.Commit(pebble.NoSync))
		require.NoError(t, d.Flush())
	}
	wc.WaitAndStop()

	defer d.Close()
	return destFS
}
