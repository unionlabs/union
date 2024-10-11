// Copyright 2018 The Prometheus Authors
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

package prometheus

import (
	"runtime"
	"testing"
	"time"

	dto "github.com/prometheus/client_model/go"
)

func TestGoCollectorGoroutines(t *testing.T) {
	var (
		c               = NewGoCollector()
		metricCh        = make(chan Metric)
		waitCh          = make(chan struct{})
		endGoroutineCh  = make(chan struct{})
		endCollectionCh = make(chan struct{})
		old             = -1
	)
	defer func() {
		close(endGoroutineCh)
		// Drain the collect channel to prevent goroutine leak.
		for {
			select {
			case <-metricCh:
			case <-endCollectionCh:
				return
			}
		}
	}()

	go func() {
		c.Collect(metricCh)
		for i := 1; i <= 10; i++ {
			// Start 10 goroutines to be sure we'll detect an
			// increase even if unrelated goroutines happen to
			// terminate during this test.
			go func(c <-chan struct{}) {
				<-c
			}(endGoroutineCh)
		}
		<-waitCh
		c.Collect(metricCh)
		close(endCollectionCh)
	}()

	for {
		select {
		case m := <-metricCh:
			// m can be Gauge or Counter,
			// currently just test the go_goroutines Gauge
			// and ignore others.
			if m.Desc().fqName != "go_goroutines" {
				continue
			}
			pb := &dto.Metric{}
			m.Write(pb)
			if pb.GetGauge() == nil {
				continue
			}

			if old == -1 {
				old = int(pb.GetGauge().GetValue())
				close(waitCh)
				continue
			}

			if diff := old - int(pb.GetGauge().GetValue()); diff > -1 {
				t.Errorf("want at least one new goroutine, got %d fewer", diff)
			}
		case <-time.After(1 * time.Second):
			t.Fatalf("expected collect timed out")
		}
		break
	}
}

func TestGoCollectorGC(t *testing.T) {
	var (
		c               = NewGoCollector()
		metricCh        = make(chan Metric)
		waitCh          = make(chan struct{})
		endCollectionCh = make(chan struct{})
		oldGC           uint64
		oldPause        float64
	)

	go func() {
		c.Collect(metricCh)
		// force GC
		runtime.GC()
		<-waitCh
		c.Collect(metricCh)
		close(endCollectionCh)
	}()

	defer func() {
		// Drain the collect channel to prevent goroutine leak.
		for {
			select {
			case <-metricCh:
			case <-endCollectionCh:
				return
			}
		}
	}()

	first := true
	for {
		select {
		case metric := <-metricCh:
			pb := &dto.Metric{}
			metric.Write(pb)
			if pb.GetSummary() == nil {
				continue
			}
			if len(pb.GetSummary().Quantile) != 5 {
				t.Errorf("expected 4 buckets, got %d", len(pb.GetSummary().Quantile))
			}
			for idx, want := range []float64{0.0, 0.25, 0.5, 0.75, 1.0} {
				if *pb.GetSummary().Quantile[idx].Quantile != want {
					t.Errorf("bucket #%d is off, got %f, want %f", idx, *pb.GetSummary().Quantile[idx].Quantile, want)
				}
			}
			if first {
				first = false
				oldGC = *pb.GetSummary().SampleCount
				oldPause = *pb.GetSummary().SampleSum
				close(waitCh)
				continue
			}
			if diff := *pb.GetSummary().SampleCount - oldGC; diff < 1 {
				t.Errorf("want at least 1 new garbage collection run, got %d", diff)
			}
			if diff := *pb.GetSummary().SampleSum - oldPause; diff <= 0 {
				t.Errorf("want an increase in pause time, got a change of %f", diff)
			}
		case <-time.After(1 * time.Second):
			t.Fatalf("expected collect timed out")
		}
		break
	}
}

func BenchmarkGoCollector(b *testing.B) {
	c := NewGoCollector().(*goCollector)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		ch := make(chan Metric, 8)
		go func() {
			// Drain all metrics received until the
			// channel is closed.
			for range ch {
			}
		}()
		c.Collect(ch)
		close(ch)
	}
}
