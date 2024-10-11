// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"math"
	"os"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/pebble/internal/rate"
	"golang.org/x/exp/rand"
)

const (
	// If measureLatency is true, the simulator outputs p50, p95, and p99
	// latencies after all writes are completed. In this mode, all writes
	// are immediately queued. If this is disabled, writes come in continually
	// at different rates.
	measureLatency = false

	writeAmount = 2000 << 20 // 2 GB

	// Max rate for all compactions. This is intentionally set low enough that
	// user writes will have to be delayed.
	maxCompactionRate = 100 << 20 // 100 MB/s
	minCompactionRate = 20 << 20  // 20 MB/s

	memtableSize        = 64 << 20 // 64 MB
	maxMemtableCount    = 5
	drainDelayThreshold = 1.05 * memtableSize
	maxFlushRate        = 30 << 20 // 30 MB/s
	minFlushRate        = 4 << 20  // 4 MB/s

	l0CompactionThreshold = 1

	levelRatio = 10
	numLevels  = 7

	compactionDebtSlowdownThreshold = 2 * memtableSize
)

type compactionPacer struct {
	level      atomic.Int64
	maxDrainer *rate.Limiter
	minDrainer *rate.Limiter
}

func newCompactionPacer() *compactionPacer {
	p := &compactionPacer{
		maxDrainer: rate.NewLimiter(maxCompactionRate, maxCompactionRate),
		minDrainer: rate.NewLimiter(minCompactionRate, minCompactionRate),
	}
	return p
}

func (p *compactionPacer) fill(n int64) {
	p.level.Add(n)
}

func (p *compactionPacer) drain(n int64, delay bool) bool {
	p.maxDrainer.Wait(float64(n))

	if delay {
		p.minDrainer.Wait(float64(n))
	}
	level := p.level.Add(-n)
	return level <= compactionDebtSlowdownThreshold
}

type flushPacer struct {
	level           atomic.Int64
	drainDelayLevel float64
	fillCond        sync.Cond
	// minDrainer is the drainer which sets the minimum speed of draining.
	minDrainer *rate.Limiter
	// maxDrainer is the drainer which sets the maximum speed of draining.
	maxDrainer *rate.Limiter
}

func newFlushPacer(mu *sync.Mutex) *flushPacer {
	p := &flushPacer{
		drainDelayLevel: drainDelayThreshold,
		minDrainer:      rate.NewLimiter(minFlushRate, minFlushRate),
		maxDrainer:      rate.NewLimiter(maxFlushRate, maxFlushRate),
	}
	p.fillCond.L = mu
	return p
}

func (p *flushPacer) fill(n int64) {
	p.level.Add(n)
	p.fillCond.Signal()
}

func (p *flushPacer) drain(n int64, delay bool) bool {
	p.maxDrainer.Wait(float64(n))

	if delay {
		p.minDrainer.Wait(float64(n))
	}
	level := p.level.Add(-n)
	p.fillCond.Signal()
	return float64(level) <= p.drainDelayLevel
}

// DB models a Pebble DB.
type DB struct {
	mu         sync.Mutex
	flushPacer *flushPacer
	flushCond  sync.Cond
	memtables  []*int64
	fill       atomic.Int64
	drain      atomic.Int64

	compactionMu    sync.Mutex
	compactionPacer *compactionPacer
	// L0 is represented as an array of integers whereas every other level
	// is represented as a single integer.
	L0 []*int64
	// Non-L0 sstables. sstables[0] == L1.
	sstables            []atomic.Int64
	maxSSTableSizes     []int64
	compactionFlushCond sync.Cond
	prevCompactionDebt  float64
}

func newDB() *DB {
	db := &DB{}
	db.flushPacer = newFlushPacer(&db.mu)
	db.flushCond.L = &db.mu
	db.memtables = append(db.memtables, new(int64))

	db.compactionFlushCond.L = &db.compactionMu
	db.L0 = append(db.L0, new(int64))
	db.compactionPacer = newCompactionPacer()

	db.maxSSTableSizes = make([]int64, numLevels-1)
	db.sstables = make([]atomic.Int64, numLevels-1)
	base := int64(levelRatio)
	for i := uint64(0); i < numLevels-2; i++ {
		// Each level is 10 times larger than the one above it.
		db.maxSSTableSizes[i] = memtableSize * l0CompactionThreshold * base
		base *= levelRatio

		// Begin with each level full.
		newLevel := db.maxSSTableSizes[i]

		db.sstables[i].Store(newLevel)
	}
	db.sstables[numLevels-2].Store(0)
	db.maxSSTableSizes[numLevels-2] = math.MaxInt64

	go db.drainMemtable()
	go db.drainCompaction()

	return db
}

// drainCompaction simulates background compactions.
func (db *DB) drainCompaction() {
	rng := rand.New(rand.NewSource(1))

	for {
		db.compactionMu.Lock()

		for len(db.L0) <= l0CompactionThreshold {
			db.compactionFlushCond.Wait()
		}
		l0Table := db.L0[0]
		db.compactionMu.Unlock()

		var delay bool
		for i, size := int64(0), int64(0); i < *l0Table; i += size {
			size = 10000 + rng.Int63n(500)
			if size > (*l0Table - i) {
				size = *l0Table - i
			}
			delay = db.compactionPacer.drain(size, delay)
		}

		db.compactionMu.Lock()
		db.L0 = db.L0[1:]
		db.compactionMu.Unlock()

		singleTableSize := int64(memtableSize)
		tablesToCompact := 0
		for i := range db.sstables {
			newSSTableSize := db.sstables[i].Add(singleTableSize)
			if newSSTableSize > db.maxSSTableSizes[i] {
				db.sstables[i].Add(-singleTableSize)
				tablesToCompact++
			} else {
				// Lower levels do not need compaction if level above it did not
				// need compaction.
				break
			}
		}

		totalCompactionBytes := int64(tablesToCompact * memtableSize)

		for t := 0; t < tablesToCompact; t++ {
			db.compactionPacer.fill(memtableSize)
			for i, size := int64(0), int64(0); i < memtableSize; i += size {
				size = 10000 + rng.Int63n(500)
				if size > (totalCompactionBytes - i) {
					size = totalCompactionBytes - i
				}
				delay = db.compactionPacer.drain(size, delay)
			}

			db.delayMemtableDrain()
		}
	}
}

// fillCompaction fills L0 sstables.
func (db *DB) fillCompaction(size int64) {
	db.compactionMu.Lock()

	db.compactionPacer.fill(size)

	last := db.L0[len(db.L0)-1]
	if *last+size > memtableSize {
		last = new(int64)
		db.L0 = append(db.L0, last)
		db.compactionFlushCond.Signal()
	}
	*last += size

	db.compactionMu.Unlock()
}

// drainMemtable simulates memtable flushing.
func (db *DB) drainMemtable() {
	rng := rand.New(rand.NewSource(2))

	for {
		db.mu.Lock()
		for len(db.memtables) <= 1 {
			db.flushCond.Wait()
		}
		memtable := db.memtables[0]
		db.mu.Unlock()

		var delay bool
		for i, size := int64(0), int64(0); i < *memtable; i += size {
			size = 1000 + rng.Int63n(50)
			if size > (*memtable - i) {
				size = *memtable - i
			}
			delay = db.flushPacer.drain(size, delay)
			db.drain.Add(size)

			db.fillCompaction(size)
		}

		db.delayMemtableDrain()

		db.mu.Lock()
		db.memtables = db.memtables[1:]
		db.mu.Unlock()
	}
}

// delayMemtableDrain applies memtable drain delays depending on compaction debt.
func (db *DB) delayMemtableDrain() {
	totalCompactionBytes := db.compactionPacer.level.Load()
	compactionDebt := math.Max(float64(totalCompactionBytes)-l0CompactionThreshold*memtableSize, 0.0)

	db.mu.Lock()
	if compactionDebt > compactionDebtSlowdownThreshold {
		// Compaction debt is above the threshold and the debt is growing. Throttle memtable flushing.
		drainLimit := maxFlushRate * float64(compactionDebtSlowdownThreshold/compactionDebt)
		if drainLimit > 0 && drainLimit <= maxFlushRate {
			db.flushPacer.maxDrainer.SetRate(drainLimit)
		}
	} else {
		// Continuously speed up memtable flushing to make sure that slowdown signal did not
		// decrease the memtable flush rate by too much.
		drainLimit := db.flushPacer.maxDrainer.Rate() * 1.05
		if drainLimit > 0 && drainLimit <= maxFlushRate {
			db.flushPacer.maxDrainer.SetRate(drainLimit)
		}
	}

	db.prevCompactionDebt = compactionDebt
	db.mu.Unlock()
}

// fillMemtable simulates memtable filling.
func (db *DB) fillMemtable(size int64) {
	db.mu.Lock()

	for len(db.memtables) > maxMemtableCount {
		db.flushPacer.fillCond.Wait()
	}
	db.flushPacer.fill(size)
	db.fill.Add(size)

	last := db.memtables[len(db.memtables)-1]
	if *last+size > memtableSize {
		last = new(int64)
		db.memtables = append(db.memtables, last)
		db.flushCond.Signal()
	}
	*last += size

	db.mu.Unlock()
}

// simulateWrite simulates user writes.
func simulateWrite(db *DB, measureLatencyMode bool) {
	limiter := rate.NewLimiter(10<<20, 10<<20) // 10 MB/s
	fmt.Printf("filling at 10 MB/sec\n")

	setRate := func(mb int) {
		fmt.Printf("filling at %d MB/sec\n", mb)
		limiter.SetRate(float64(mb << 20))
	}

	if !measureLatencyMode {
		go func() {
			rng := rand.New(rand.NewSource(3))
			for {
				secs := 5 + rng.Intn(5)
				time.Sleep(time.Duration(secs) * time.Second)
				mb := 10 + rng.Intn(20)
				setRate(mb)
			}
		}()
	}

	rng := rand.New(rand.NewSource(uint64(4)))

	totalWrites := int64(0)
	percentiles := []int64{50, 95, 99}
	percentileIndex := 0
	percentileTimes := make([]time.Time, 0)

	startTime := time.Now()
	for totalWrites <= writeAmount {
		size := 1000 + rng.Int63n(50)
		if !measureLatencyMode {
			limiter.Wait(float64(size))
		}
		db.fillMemtable(size)

		// Calculate latency percentiles
		totalWrites += size
		if percentileIndex < len(percentiles) && totalWrites > (percentiles[percentileIndex]*writeAmount/100) {
			percentileTimes = append(percentileTimes, time.Now())
			percentileIndex++
		}
	}

	time.Sleep(time.Second * 10)
	// Latency should only be measured when `limiter.WaitN` is removed.
	if measureLatencyMode {
		fmt.Printf("_____p50______p95______p99\n")
		fmt.Printf("%8s %8s %8s\n",
			time.Duration(percentileTimes[0].Sub(startTime).Seconds())*time.Second,
			time.Duration(percentileTimes[1].Sub(startTime).Seconds())*time.Second,
			time.Duration(percentileTimes[2].Sub(startTime).Seconds())*time.Second)
	}

	os.Exit(0)
}

func main() {
	db := newDB()

	go simulateWrite(db, measureLatency)

	tick := time.NewTicker(time.Second)
	start := time.Now()
	lastNow := start
	var lastFill, lastDrain int64

	for i := 0; ; i++ {
		<-tick.C
		if (i % 20) == 0 {
			fmt.Printf("_elapsed___memtbs____dirty_____fill____drain____cdebt__l0count___max-f-rate\n")
		}

		db.mu.Lock()
		memtableCount := len(db.memtables)
		db.mu.Unlock()
		dirty := db.flushPacer.level.Load()
		fill := db.fill.Load()
		drain := db.drain.Load()

		db.compactionMu.Lock()
		compactionL0 := len(db.L0)
		db.compactionMu.Unlock()
		totalCompactionBytes := db.compactionPacer.level.Load()
		compactionDebt := math.Max(float64(totalCompactionBytes)-l0CompactionThreshold*memtableSize, 0.0)
		maxFlushRate := db.flushPacer.maxDrainer.Rate()

		now := time.Now()
		elapsed := now.Sub(lastNow).Seconds()
		fmt.Printf("%8s %8d %8.1f %8.1f %8.1f %8.1f %8d %12.1f\n",
			time.Duration(now.Sub(start).Seconds()+0.5)*time.Second,
			memtableCount,
			float64(dirty)/(1024.0*1024.0),
			float64(fill-lastFill)/(1024.0*1024.0*elapsed),
			float64(drain-lastDrain)/(1024.0*1024.0*elapsed),
			compactionDebt/(1024.0*1024.0),
			compactionL0,
			maxFlushRate/(1024.0*1024.0))

		lastNow = now
		lastFill = fill
		lastDrain = drain
	}
}
