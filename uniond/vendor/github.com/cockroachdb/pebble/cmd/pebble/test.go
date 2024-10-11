// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"io"
	"log"
	"os"
	"os/signal"
	"runtime"
	"runtime/pprof"
	"sort"
	"sync"
	"syscall"
	"time"

	"github.com/HdrHistogram/hdrhistogram-go"
	"github.com/cockroachdb/pebble"
)

const (
	minLatency = 10 * time.Microsecond
	maxLatency = 10 * time.Second
)

func startCPUProfile() func() {
	runtime.SetMutexProfileFraction(1000)

	done := startRecording("cpu.%04d.prof", pprof.StartCPUProfile, pprof.StopCPUProfile)
	return func() {
		done()
		if p := pprof.Lookup("heap"); p != nil {
			f, err := os.Create("heap.prof")
			if err != nil {
				log.Fatal(err)
			}
			if err := p.WriteTo(f, 0); err != nil {
				log.Fatal(err)
			}
			f.Close()
		}
		if p := pprof.Lookup("mutex"); p != nil {
			f, err := os.Create("mutex.prof")
			if err != nil {
				log.Fatal(err)
			}
			if err := p.WriteTo(f, 0); err != nil {
				log.Fatal(err)
			}
			f.Close()
		}
	}
}

func startRecording(fmtStr string, startFunc func(io.Writer) error, stopFunc func()) func() {
	doneCh := make(chan struct{})
	var doneWG sync.WaitGroup
	doneWG.Add(1)

	go func() {
		defer doneWG.Done()

		start := time.Now()
		t := time.NewTicker(10 * time.Second)
		defer t.Stop()

		var current *os.File
		defer func() {
			if current != nil {
				stopFunc()
				current.Close()
			}
		}()

		for {
			if current != nil {
				stopFunc()
				current.Close()
				current = nil
			}
			path := fmt.Sprintf(fmtStr, int(time.Since(start).Seconds()+0.5))
			f, err := os.Create(path)
			if err != nil {
				log.Fatalf("unable to create cpu profile: %s", err)
				return
			}
			if err := startFunc(f); err != nil {
				log.Fatalf("unable to start cpu profile: %v", err)
				f.Close()
				return
			}
			current = f

			select {
			case <-doneCh:
				return
			case <-t.C:
			}
		}
	}()

	return func() {
		close(doneCh)
		doneWG.Wait()
	}
}

func newHistogram() *hdrhistogram.Histogram {
	return hdrhistogram.New(minLatency.Nanoseconds(), maxLatency.Nanoseconds(), 1)
}

type namedHistogram struct {
	name string
	mu   struct {
		sync.Mutex
		current *hdrhistogram.Histogram
	}
}

func newNamedHistogram(name string) *namedHistogram {
	w := &namedHistogram{name: name}
	w.mu.current = newHistogram()
	return w
}

func (w *namedHistogram) Record(elapsed time.Duration) {
	if elapsed < minLatency {
		elapsed = minLatency
	} else if elapsed > maxLatency {
		elapsed = maxLatency
	}

	w.mu.Lock()
	err := w.mu.current.RecordValue(elapsed.Nanoseconds())
	w.mu.Unlock()

	if err != nil {
		// Note that a histogram only drops recorded values that are out of range,
		// but we clamp the latency value to the configured range to prevent such
		// drops. This code path should never happen.
		panic(fmt.Sprintf(`%s: recording value: %s`, w.name, err))
	}
}

func (w *namedHistogram) tick(fn func(h *hdrhistogram.Histogram)) {
	w.mu.Lock()
	defer w.mu.Unlock()
	h := w.mu.current
	w.mu.current = newHistogram()
	fn(h)
}

type histogramTick struct {
	// Name is the name given to the histograms represented by this tick.
	Name string
	// Hist is the merged result of the represented histograms for this tick.
	// Hist.TotalCount() is the number of operations that occurred for this tick.
	Hist *hdrhistogram.Histogram
	// Cumulative is the merged result of the represented histograms for all
	// time. Cumulative.TotalCount() is the total number of operations that have
	// occurred over all time.
	Cumulative *hdrhistogram.Histogram
	// Elapsed is the amount of time since the last tick.
	Elapsed time.Duration
	// Now is the time at which the tick was gathered. It covers the period
	// [Now-Elapsed,Now).
	Now time.Time
}

type histogramRegistry struct {
	mu struct {
		sync.Mutex
		registered []*namedHistogram
	}

	start      time.Time
	cumulative map[string]*hdrhistogram.Histogram
	prevTick   map[string]time.Time
}

func newHistogramRegistry() *histogramRegistry {
	return &histogramRegistry{
		start:      time.Now(),
		cumulative: make(map[string]*hdrhistogram.Histogram),
		prevTick:   make(map[string]time.Time),
	}
}

func (w *histogramRegistry) Register(name string) *namedHistogram {
	hist := newNamedHistogram(name)

	w.mu.Lock()
	w.mu.registered = append(w.mu.registered, hist)
	w.mu.Unlock()

	return hist
}

func (w *histogramRegistry) Tick(fn func(histogramTick)) {
	w.mu.Lock()
	registered := append([]*namedHistogram(nil), w.mu.registered...)
	w.mu.Unlock()

	merged := make(map[string]*hdrhistogram.Histogram)
	var names []string
	for _, hist := range registered {
		hist.tick(func(h *hdrhistogram.Histogram) {
			if p, ok := merged[hist.name]; ok {
				p.Merge(h)
			} else {
				merged[hist.name] = h
				names = append(names, hist.name)
			}
		})
	}

	now := time.Now()
	sort.Strings(names)
	for _, name := range names {
		mergedHist := merged[name]
		if _, ok := w.cumulative[name]; !ok {
			w.cumulative[name] = newHistogram()
		}
		w.cumulative[name].Merge(mergedHist)

		prevTick, ok := w.prevTick[name]
		if !ok {
			prevTick = w.start
		}
		w.prevTick[name] = now
		fn(histogramTick{
			Name:       name,
			Hist:       merged[name],
			Cumulative: w.cumulative[name],
			Elapsed:    now.Sub(prevTick),
			Now:        now,
		})
	}
}

type testWithoutDB struct {
	init func(wg *sync.WaitGroup)
	tick func(elapsed time.Duration, i int)
	done func(wg *sync.WaitGroup, elapsed time.Duration)
}

func runTestWithoutDB(t testWithoutDB) {
	var wg sync.WaitGroup
	t.init(&wg)

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	done := make(chan os.Signal, 3)
	workersDone := make(chan struct{})
	signal.Notify(done, os.Interrupt)

	go func() {
		wg.Wait()
		close(workersDone)
	}()

	if duration > 0 {
		go func() {
			time.Sleep(duration)
			done <- syscall.Signal(0)
		}()
	}

	stopProf := startCPUProfile()
	defer stopProf()

	start := time.Now()
	for i := 0; ; i++ {
		select {
		case <-ticker.C:
			if workersDone != nil {
				t.tick(time.Since(start), i)
			}

		case <-workersDone:
			workersDone = nil
			t.done(&wg, time.Since(start))
			return

		case sig := <-done:
			fmt.Println("operating system is killing the op.", sig)
			if workersDone != nil {
				t.done(&wg, time.Since(start))
			}
			return
		}
	}
}

type test struct {
	init func(db DB, wg *sync.WaitGroup)
	tick func(elapsed time.Duration, i int)
	done func(elapsed time.Duration)
}

func runTest(dir string, t test) {
	// Check if the directory exists.
	if wipe {
		fmt.Printf("wiping %s\n", dir)
		if err := os.RemoveAll(dir); err != nil {
			log.Fatal(err)
		}
	}

	fmt.Printf("dir %s\nconcurrency %d\n", dir, concurrency)

	db := newPebbleDB(dir)
	var wg sync.WaitGroup
	t.init(db, &wg)

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	done := make(chan os.Signal, 3)
	workersDone := make(chan struct{})
	signal.Notify(done, os.Interrupt)

	go func() {
		wg.Wait()
		close(workersDone)
	}()

	if maxSize > 0 {
		go func() {
			for {
				time.Sleep(10 * time.Second)
				if db.Metrics().DiskSpaceUsage() > maxSize*1e6 {
					fmt.Println("max size reached")
					done <- syscall.Signal(0)
				}
			}
		}()
	}
	if duration > 0 {
		go func() {
			time.Sleep(duration)
			done <- syscall.Signal(0)
		}()
	}

	stopProf := startCPUProfile()
	defer stopProf()

	backgroundCompactions := func(p *pebble.Metrics) bool {
		// The last level never gets selected as an input level for compaction,
		// only as an output level, so ignore it for the purposes of determining if
		// background compactions are still needed.
		for i := range p.Levels[:len(p.Levels)-1] {
			if p.Levels[i].Score > 1 {
				return true
			}
		}
		return false
	}

	start := time.Now()
	for i := 0; ; i++ {
		select {
		case <-ticker.C:
			if workersDone != nil {
				t.tick(time.Since(start), i)
				if verbose && (i%10) == 9 {
					fmt.Printf("%s", db.Metrics())
				}
			} else if waitCompactions {
				p := db.Metrics()
				fmt.Printf("%s", p)
				if !backgroundCompactions(p) {
					return
				}
			}

		case <-workersDone:
			workersDone = nil
			t.done(time.Since(start))
			p := db.Metrics()
			fmt.Printf("%s", p)
			if !waitCompactions || !backgroundCompactions(p) {
				return
			}
			fmt.Printf("waiting for background compactions\n")

		case <-done:
			if workersDone != nil {
				t.done(time.Since(start))
			}
			fmt.Printf("%s", db.Metrics())
			return
		}
	}
}
