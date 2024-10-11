// Copyright 2021 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build !go1.17
// +build !go1.17

package prometheus

import (
	"runtime"
	"testing"
	"time"

	dto "github.com/prometheus/client_model/go"
)

func TestGoCollectorMemStats(t *testing.T) {
	var (
		c   = NewGoCollector().(*goCollector)
		got uint64
	)

	checkCollect := func(want uint64) {
		metricCh := make(chan Metric)
		endCh := make(chan struct{})

		go func() {
			c.Collect(metricCh)
			close(endCh)
		}()
	Collect:
		for {
			select {
			case metric := <-metricCh:
				if metric.Desc().fqName != "go_memstats_alloc_bytes" {
					continue Collect
				}
				pb := &dto.Metric{}
				metric.Write(pb)
				got = uint64(pb.GetGauge().GetValue())
			case <-endCh:
				break Collect
			}
		}
		if want != got {
			t.Errorf("unexpected value of go_memstats_alloc_bytes, want %d, got %d", want, got)
		}
	}

	// Speed up the timing to make the test faster.
	c.msMaxWait = 5 * time.Millisecond
	c.msMaxAge = 50 * time.Millisecond

	// Scenario 1: msRead responds slowly, no previous memstats available,
	// msRead is executed anyway.
	c.msRead = func(ms *runtime.MemStats) {
		time.Sleep(20 * time.Millisecond)
		ms.Alloc = 1
	}
	checkCollect(1)
	// Now msLast is set.
	c.msMtx.Lock()
	if want, got := uint64(1), c.msLast.Alloc; want != got {
		t.Errorf("unexpected of msLast.Alloc, want %d, got %d", want, got)
	}
	c.msMtx.Unlock()

	// Scenario 2: msRead responds fast, previous memstats available, new
	// value collected.
	c.msRead = func(ms *runtime.MemStats) {
		ms.Alloc = 2
	}
	checkCollect(2)
	// msLast is set, too.
	c.msMtx.Lock()
	if want, got := uint64(2), c.msLast.Alloc; want != got {
		t.Errorf("unexpected of msLast.Alloc, want %d, got %d", want, got)
	}
	c.msMtx.Unlock()

	// Scenario 3: msRead responds slowly, previous memstats available, old
	// value collected.
	c.msRead = func(ms *runtime.MemStats) {
		time.Sleep(20 * time.Millisecond)
		ms.Alloc = 3
	}
	checkCollect(2)
	// After waiting, new value is still set in msLast.
	time.Sleep(80 * time.Millisecond)
	c.msMtx.Lock()
	if want, got := uint64(3), c.msLast.Alloc; want != got {
		t.Errorf("unexpected of msLast.Alloc, want %d, got %d", want, got)
	}
	c.msMtx.Unlock()

	// Scenario 4: msRead responds slowly, previous memstats is too old, new
	// value collected.
	c.msRead = func(ms *runtime.MemStats) {
		time.Sleep(20 * time.Millisecond)
		ms.Alloc = 4
	}
	checkCollect(4)
	c.msMtx.Lock()
	if want, got := uint64(4), c.msLast.Alloc; want != got {
		t.Errorf("unexpected of msLast.Alloc, want %d, got %d", want, got)
	}
	c.msMtx.Unlock()
}
