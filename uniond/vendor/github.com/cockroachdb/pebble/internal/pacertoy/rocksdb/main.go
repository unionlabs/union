// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"math"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/pebble/internal/rate"
	"golang.org/x/exp/rand"
)

const (
	// Max rate for all compactions. This is intentionally set low enough that
	// user writes will have to be delayed.
	maxCompactionRate = 80 << 20 // 80 MB/s

	memtableSize          = 64 << 20 // 64 MB
	memtableStopThreshold = 2 * memtableSize
	maxWriteRate          = 30 << 20 // 30 MB/s
	startingWriteRate     = 30 << 20 // 30 MB/s

	l0SlowdownThreshold   = 4
	l0CompactionThreshold = 1

	levelRatio = 10
	numLevels  = 7

	// Slowdown threshold is set at the compaction debt incurred by the largest
	// possible compaction.
	compactionDebtSlowdownThreshold = memtableSize * (numLevels - 2)
)

type compactionPacer struct {
	level   atomic.Int64
	drainer *rate.Limiter
}

func newCompactionPacer() *compactionPacer {
	p := &compactionPacer{
		drainer: rate.NewLimiter(maxCompactionRate, maxCompactionRate),
	}
	return p
}

func (p *compactionPacer) fill(n int64) {
	p.level.Add(n)
}

func (p *compactionPacer) drain(n int64) {
	p.drainer.Wait(float64(n))

	p.level.Add(-n)
}

type flushPacer struct {
	level                 atomic.Int64
	memtableStopThreshold float64
	fillCond              sync.Cond
}

func newFlushPacer(mu *sync.Mutex) *flushPacer {
	p := &flushPacer{
		memtableStopThreshold: memtableStopThreshold,
	}
	p.fillCond.L = mu
	return p
}

func (p *flushPacer) fill(n int64) {
	for float64(p.level.Load()) >= p.memtableStopThreshold {
		p.fillCond.Wait()
	}
	p.level.Add(n)
	p.fillCond.Signal()
}

func (p *flushPacer) drain(n int64) {
	p.level.Add(-n)
}

// DB models a RocksDB DB.
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
	previouslyInDebt    bool

	writeLimiter *rate.Limiter
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

	db.writeLimiter = rate.NewLimiter(startingWriteRate, startingWriteRate)

	go db.drainMemtable()
	go db.drainCompaction()

	return db
}

// drainCompaction simulates background compactions.
func (db *DB) drainCompaction() {
	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

	for {
		db.compactionMu.Lock()

		for len(db.L0) <= l0CompactionThreshold {
			db.compactionFlushCond.Wait()
		}
		l0Table := db.L0[0]
		db.compactionMu.Unlock()

		for i, size := int64(0), int64(0); i < *l0Table; i += size {
			size = 10000 + rng.Int63n(500)
			if size > (*l0Table - i) {
				size = *l0Table - i
			}
			db.compactionPacer.drain(size)
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
		db.compactionPacer.fill(totalCompactionBytes)

		for t := 0; t < tablesToCompact; t++ {
			for i, size := int64(0), int64(0); i < memtableSize; i += size {
				size = 10000 + rng.Int63n(500)
				if size > (totalCompactionBytes - i) {
					size = totalCompactionBytes - i
				}
				db.compactionPacer.drain(size)
			}

			db.delayUserWrites()
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
	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

	for {
		db.mu.Lock()
		for len(db.memtables) <= 1 {
			db.flushCond.Wait()
		}
		memtable := db.memtables[0]
		db.mu.Unlock()

		for i, size := int64(0), int64(0); i < *memtable; i += size {
			size = 1000 + rng.Int63n(50)
			if size > (*memtable - i) {
				size = *memtable - i
			}
			db.flushPacer.drain(size)
			db.drain.Add(size)

			db.fillCompaction(size)
		}

		db.delayUserWrites()

		db.mu.Lock()
		db.memtables = db.memtables[1:]
		db.mu.Unlock()
	}
}

// delayUserWrites applies write delays depending on compaction debt.
func (db *DB) delayUserWrites() {
	totalCompactionBytes := db.compactionPacer.level.Load()
	compactionDebt := math.Max(float64(totalCompactionBytes)-l0CompactionThreshold*memtableSize, 0.0)

	db.mu.Lock()
	if len(db.L0) > l0SlowdownThreshold || compactionDebt > compactionDebtSlowdownThreshold {
		db.previouslyInDebt = true
		if compactionDebt > db.prevCompactionDebt {
			// Debt is growing.
			drainLimit := db.writeLimiter.Rate() * 0.8
			if drainLimit > 0 {
				db.writeLimiter.SetRate(drainLimit)
			}
		} else {
			// Debt is shrinking.
			drainLimit := db.writeLimiter.Rate() * 1 / 0.8
			if drainLimit <= maxWriteRate {
				db.writeLimiter.SetRate(drainLimit)
			}
		}
	} else if db.previouslyInDebt {
		// If compaction was previously delayed and has recovered, RocksDB
		// "rewards" the rate by double the slowdown ratio.

		// From RocksDB:
		// If the DB recovers from delay conditions, we reward with reducing
		// double the slowdown ratio. This is to balance the long term slowdown
		// increase signal.
		drainLimit := db.writeLimiter.Rate() * 1.4
		if drainLimit <= maxWriteRate {
			db.writeLimiter.SetRate(drainLimit)
		}
		db.previouslyInDebt = false
	}

	db.prevCompactionDebt = compactionDebt
	db.mu.Unlock()
}

// fillMemtable simulates memtable filling.
func (db *DB) fillMemtable(size int64) {
	db.mu.Lock()

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
func simulateWrite(db *DB) {
	limiter := rate.NewLimiter(10<<20, 10<<20) // 10 MB/s
	fmt.Printf("filling at 10 MB/sec\n")

	setRate := func(mb int) {
		fmt.Printf("filling at %d MB/sec\n", mb)
		limiter.SetRate(float64(mb << 20))
	}

	go func() {
		rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
		for {
			secs := 5 + rng.Intn(5)
			time.Sleep(time.Duration(secs) * time.Second)
			mb := 11 + rng.Intn(20)
			setRate(mb)
		}
	}()

	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))

	for {
		size := 1000 + rng.Int63n(50)
		limiter.Wait(float64(size))
		db.writeLimiter.Wait(float64(size))
		db.fillMemtable(size)
	}
}

func main() {
	db := newDB()

	go simulateWrite(db)

	tick := time.NewTicker(time.Second)
	start := time.Now()
	lastNow := start
	var lastFill, lastDrain int64

	for i := 0; ; i++ {
		<-tick.C
		if (i % 20) == 0 {
			fmt.Printf("_elapsed___memtbs____dirty_____fill____drain____cdebt__l0count___max-w-rate\n")
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
		maxWriteRate := db.writeLimiter.Rate()

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
			maxWriteRate/(1024.0*1024.0))

		lastNow = now
		lastFill = fill
		lastDrain = drain
	}
}
