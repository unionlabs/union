// Copyright 2015 The Prometheus Authors
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
	"math"
	"math/rand"
	"reflect"
	"runtime"
	"sort"
	"sync"
	"sync/atomic"
	"testing"
	"testing/quick"
	"time"

	"github.com/prometheus/client_golang/prometheus/internal"

	dto "github.com/prometheus/client_model/go"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func benchmarkHistogramObserve(w int, b *testing.B) {
	b.StopTimer()

	wg := new(sync.WaitGroup)
	wg.Add(w)

	g := new(sync.WaitGroup)
	g.Add(1)

	s := NewHistogram(HistogramOpts{})

	for i := 0; i < w; i++ {
		go func() {
			g.Wait()

			for i := 0; i < b.N; i++ {
				s.Observe(float64(i))
			}

			wg.Done()
		}()
	}

	b.StartTimer()
	g.Done()
	wg.Wait()
}

func BenchmarkHistogramObserve1(b *testing.B) {
	benchmarkHistogramObserve(1, b)
}

func BenchmarkHistogramObserve2(b *testing.B) {
	benchmarkHistogramObserve(2, b)
}

func BenchmarkHistogramObserve4(b *testing.B) {
	benchmarkHistogramObserve(4, b)
}

func BenchmarkHistogramObserve8(b *testing.B) {
	benchmarkHistogramObserve(8, b)
}

func benchmarkHistogramWrite(w int, b *testing.B) {
	b.StopTimer()

	wg := new(sync.WaitGroup)
	wg.Add(w)

	g := new(sync.WaitGroup)
	g.Add(1)

	s := NewHistogram(HistogramOpts{})

	for i := 0; i < 1000000; i++ {
		s.Observe(float64(i))
	}

	for j := 0; j < w; j++ {
		outs := make([]dto.Metric, b.N)

		go func(o []dto.Metric) {
			g.Wait()

			for i := 0; i < b.N; i++ {
				s.Write(&o[i])
			}

			wg.Done()
		}(outs)
	}

	b.StartTimer()
	g.Done()
	wg.Wait()
}

func BenchmarkHistogramWrite1(b *testing.B) {
	benchmarkHistogramWrite(1, b)
}

func BenchmarkHistogramWrite2(b *testing.B) {
	benchmarkHistogramWrite(2, b)
}

func BenchmarkHistogramWrite4(b *testing.B) {
	benchmarkHistogramWrite(4, b)
}

func BenchmarkHistogramWrite8(b *testing.B) {
	benchmarkHistogramWrite(8, b)
}

func TestHistogramNonMonotonicBuckets(t *testing.T) {
	testCases := map[string][]float64{
		"not strictly monotonic":  {1, 2, 2, 3},
		"not monotonic at all":    {1, 2, 4, 3, 5},
		"have +Inf in the middle": {1, 2, math.Inf(+1), 3},
	}
	for name, buckets := range testCases {
		func() {
			defer func() {
				if r := recover(); r == nil {
					t.Errorf("Buckets %v are %s but NewHistogram did not panic.", buckets, name)
				}
			}()
			_ = NewHistogram(HistogramOpts{
				Name:    "test_histogram",
				Help:    "helpless",
				Buckets: buckets,
			})
		}()
	}
}

// Intentionally adding +Inf here to test if that case is handled correctly.
// Also, getCumulativeCounts depends on it.
var testBuckets = []float64{-2, -1, -0.5, 0, 0.5, 1, 2, math.Inf(+1)}

func TestHistogramConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping test in short mode.")
	}

	rand.New(rand.NewSource(42))

	it := func(n uint32) bool {
		mutations := int(n%1e4 + 1e4)
		concLevel := int(n%5 + 1)
		total := mutations * concLevel

		var start, end sync.WaitGroup
		start.Add(1)
		end.Add(concLevel)

		his := NewHistogram(HistogramOpts{
			Name:    "test_histogram",
			Help:    "helpless",
			Buckets: testBuckets,
		})

		allVars := make([]float64, total)
		var sampleSum float64
		for i := 0; i < concLevel; i++ {
			vals := make([]float64, mutations)
			for j := 0; j < mutations; j++ {
				v := rand.NormFloat64()
				vals[j] = v
				allVars[i*mutations+j] = v
				sampleSum += v
			}

			go func(vals []float64) {
				start.Wait()
				for _, v := range vals {
					if n%2 == 0 {
						his.Observe(v)
					} else {
						his.(ExemplarObserver).ObserveWithExemplar(v, Labels{"foo": "bar"})
					}
				}
				end.Done()
			}(vals)
		}
		sort.Float64s(allVars)
		start.Done()
		end.Wait()

		m := &dto.Metric{}
		his.Write(m)
		if got, want := int(*m.Histogram.SampleCount), total; got != want {
			t.Errorf("got sample count %d, want %d", got, want)
		}
		if got, want := *m.Histogram.SampleSum, sampleSum; math.Abs((got-want)/want) > 0.001 {
			t.Errorf("got sample sum %f, want %f", got, want)
		}

		wantCounts := getCumulativeCounts(allVars)
		wantBuckets := len(testBuckets)
		if !math.IsInf(m.Histogram.Bucket[len(m.Histogram.Bucket)-1].GetUpperBound(), +1) {
			wantBuckets--
		}

		if got := len(m.Histogram.Bucket); got != wantBuckets {
			t.Errorf("got %d buckets in protobuf, want %d", got, wantBuckets)
		}
		for i, wantBound := range testBuckets {
			if i == len(testBuckets)-1 {
				break // No +Inf bucket in protobuf.
			}
			if gotBound := *m.Histogram.Bucket[i].UpperBound; gotBound != wantBound {
				t.Errorf("got bound %f, want %f", gotBound, wantBound)
			}
			if gotCount, wantCount := *m.Histogram.Bucket[i].CumulativeCount, wantCounts[i]; gotCount != wantCount {
				t.Errorf("got count %d, want %d", gotCount, wantCount)
			}
		}
		return true
	}

	if err := quick.Check(it, nil); err != nil {
		t.Error(err)
	}
}

func TestHistogramVecConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping test in short mode.")
	}

	rand.New(rand.NewSource(42))

	it := func(n uint32) bool {
		mutations := int(n%1e4 + 1e4)
		concLevel := int(n%7 + 1)
		vecLength := int(n%3 + 1)

		var start, end sync.WaitGroup
		start.Add(1)
		end.Add(concLevel)

		his := NewHistogramVec(
			HistogramOpts{
				Name:    "test_histogram",
				Help:    "helpless",
				Buckets: []float64{-2, -1, -0.5, 0, 0.5, 1, 2, math.Inf(+1)},
			},
			[]string{"label"},
		)

		allVars := make([][]float64, vecLength)
		sampleSums := make([]float64, vecLength)
		for i := 0; i < concLevel; i++ {
			vals := make([]float64, mutations)
			picks := make([]int, mutations)
			for j := 0; j < mutations; j++ {
				v := rand.NormFloat64()
				vals[j] = v
				pick := rand.Intn(vecLength)
				picks[j] = pick
				allVars[pick] = append(allVars[pick], v)
				sampleSums[pick] += v
			}

			go func(vals []float64) {
				start.Wait()
				for i, v := range vals {
					his.WithLabelValues(string('A' + rune(picks[i]))).Observe(v)
				}
				end.Done()
			}(vals)
		}
		for _, vars := range allVars {
			sort.Float64s(vars)
		}
		start.Done()
		end.Wait()

		for i := 0; i < vecLength; i++ {
			m := &dto.Metric{}
			s := his.WithLabelValues(string('A' + rune(i)))
			s.(Histogram).Write(m)

			if got, want := len(m.Histogram.Bucket), len(testBuckets)-1; got != want {
				t.Errorf("got %d buckets in protobuf, want %d", got, want)
			}
			if got, want := int(*m.Histogram.SampleCount), len(allVars[i]); got != want {
				t.Errorf("got sample count %d, want %d", got, want)
			}
			if got, want := *m.Histogram.SampleSum, sampleSums[i]; math.Abs((got-want)/want) > 0.001 {
				t.Errorf("got sample sum %f, want %f", got, want)
			}

			wantCounts := getCumulativeCounts(allVars[i])

			for j, wantBound := range testBuckets {
				if j == len(testBuckets)-1 {
					break // No +Inf bucket in protobuf.
				}
				if gotBound := *m.Histogram.Bucket[j].UpperBound; gotBound != wantBound {
					t.Errorf("got bound %f, want %f", gotBound, wantBound)
				}
				if gotCount, wantCount := *m.Histogram.Bucket[j].CumulativeCount, wantCounts[j]; gotCount != wantCount {
					t.Errorf("got count %d, want %d", gotCount, wantCount)
				}
			}
		}
		return true
	}

	if err := quick.Check(it, nil); err != nil {
		t.Error(err)
	}
}

func getCumulativeCounts(vars []float64) []uint64 {
	counts := make([]uint64, len(testBuckets))
	for _, v := range vars {
		for i := len(testBuckets) - 1; i >= 0; i-- {
			if v > testBuckets[i] {
				break
			}
			counts[i]++
		}
	}
	return counts
}

func TestBuckets(t *testing.T) {
	got := LinearBuckets(-15, 5, 6)
	want := []float64{-15, -10, -5, 0, 5, 10}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("linear buckets: got %v, want %v", got, want)
	}

	got = ExponentialBuckets(100, 1.2, 3)
	want = []float64{100, 120, 144}
	if !reflect.DeepEqual(got, want) {
		t.Errorf("exponential buckets: got %v, want %v", got, want)
	}

	got = ExponentialBucketsRange(1, 100, 10)
	want = []float64{
		1.0, 1.6681, 2.7825, 4.6415, 7.7426, 12.9154, 21.5443,
		35.9381, 59.9484, 100.0000,
	}
	const epsilon = 0.0001
	if !internal.AlmostEqualFloat64s(got, want, epsilon) {
		t.Errorf("exponential buckets range: got %v, want %v (epsilon %f)", got, want, epsilon)
	}
}

func TestHistogramAtomicObserve(t *testing.T) {
	var (
		quit = make(chan struct{})
		his  = NewHistogram(HistogramOpts{
			Buckets: []float64{0.5, 10, 20},
		})
	)

	defer func() { close(quit) }()

	observe := func() {
		for {
			select {
			case <-quit:
				return
			default:
				his.Observe(1)
			}
		}
	}

	go observe()
	go observe()
	go observe()

	for i := 0; i < 100; i++ {
		m := &dto.Metric{}
		if err := his.Write(m); err != nil {
			t.Fatal("unexpected error writing histogram:", err)
		}
		h := m.GetHistogram()
		if h.GetSampleCount() != uint64(h.GetSampleSum()) ||
			h.GetSampleCount() != h.GetBucket()[1].GetCumulativeCount() ||
			h.GetSampleCount() != h.GetBucket()[2].GetCumulativeCount() {
			t.Fatalf(
				"inconsistent counts in histogram: count=%d sum=%f buckets=[%d, %d]",
				h.GetSampleCount(), h.GetSampleSum(),
				h.GetBucket()[1].GetCumulativeCount(), h.GetBucket()[2].GetCumulativeCount(),
			)
		}
		runtime.Gosched()
	}
}

func TestHistogramExemplar(t *testing.T) {
	now := time.Now()

	histogram := NewHistogram(HistogramOpts{
		Name:    "test",
		Help:    "test help",
		Buckets: []float64{1, 2, 3, 4},
		now:     func() time.Time { return now },
	}).(*histogram)

	ts := timestamppb.New(now)
	if err := ts.CheckValid(); err != nil {
		t.Fatal(err)
	}
	expectedExemplars := []*dto.Exemplar{
		nil,
		{
			Label: []*dto.LabelPair{
				{Name: proto.String("id"), Value: proto.String("2")},
			},
			Value:     proto.Float64(1.6),
			Timestamp: ts,
		},
		nil,
		{
			Label: []*dto.LabelPair{
				{Name: proto.String("id"), Value: proto.String("3")},
			},
			Value:     proto.Float64(4),
			Timestamp: ts,
		},
		{
			Label: []*dto.LabelPair{
				{Name: proto.String("id"), Value: proto.String("4")},
			},
			Value:     proto.Float64(4.5),
			Timestamp: ts,
		},
	}

	histogram.ObserveWithExemplar(1.5, Labels{"id": "1"})
	histogram.ObserveWithExemplar(1.6, Labels{"id": "2"}) // To replace exemplar in bucket 0.
	histogram.ObserveWithExemplar(4, Labels{"id": "3"})
	histogram.ObserveWithExemplar(4.5, Labels{"id": "4"}) // Should go to +Inf bucket.

	for i, ex := range histogram.exemplars {
		var got, expected string
		if val := ex.Load(); val != nil {
			got = val.(*dto.Exemplar).String()
		}
		if expectedExemplars[i] != nil {
			expected = expectedExemplars[i].String()
		}
		if got != expected {
			t.Errorf("expected exemplar %s, got %s.", expected, got)
		}
	}
}

func TestNativeHistogram(t *testing.T) {
	now := time.Now()

	scenarios := []struct {
		name             string
		observations     []float64 // With simulated interval of 1m.
		factor           float64
		zeroThreshold    float64
		maxBuckets       uint32
		minResetDuration time.Duration
		maxZeroThreshold float64
		want             *dto.Histogram
	}{
		{
			name:         "no sparse buckets",
			observations: []float64{1, 2, 3},
			factor:       1,
			want: &dto.Histogram{
				SampleCount: proto.Uint64(3),
				SampleSum:   proto.Float64(6),
				Bucket: []*dto.Bucket{
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.005)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.01)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.025)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.05)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.1)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.25)},
					{CumulativeCount: proto.Uint64(0), UpperBound: proto.Float64(0.5)},
					{CumulativeCount: proto.Uint64(1), UpperBound: proto.Float64(1)},
					{CumulativeCount: proto.Uint64(2), UpperBound: proto.Float64(2.5)},
					{CumulativeCount: proto.Uint64(3), UpperBound: proto.Float64(5)},
					{CumulativeCount: proto.Uint64(3), UpperBound: proto.Float64(10)},
				},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:   "no observations",
			factor: 1.1,
			want: &dto.Histogram{
				SampleCount:      proto.Uint64(0),
				SampleSum:        proto.Float64(0),
				Schema:           proto.Int32(3),
				ZeroThreshold:    proto.Float64(2.938735877055719e-39),
				ZeroCount:        proto.Uint64(0),
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:          "no observations and zero threshold of zero resulting in no-op span",
			factor:        1.1,
			zeroThreshold: NativeHistogramZeroThresholdZero,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(0),
				SampleSum:     proto.Float64(0),
				Schema:        proto.Int32(3),
				ZeroThreshold: proto.Float64(0),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(0)},
				},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "factor 1.1 results in schema 3",
			observations: []float64{0, 1, 2, 3},
			factor:       1.1,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(4),
				SampleSum:     proto.Float64(6),
				Schema:        proto.Int32(3),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(1)},
					{Offset: proto.Int32(7), Length: proto.Uint32(1)},
					{Offset: proto.Int32(4), Length: proto.Uint32(1)},
				},
				PositiveDelta:    []int64{1, 0, 0},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "factor 1.2 results in schema 2",
			observations: []float64{0, 1, 1.2, 1.4, 1.8, 2},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(6),
				SampleSum:     proto.Float64(7.4),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name: "factor 4 results in schema -1",
			observations: []float64{
				0.0156251, 0.0625, // Bucket -2: (0.015625, 0.0625)
				0.1, 0.25, // Bucket -1: (0.0625, 0.25]
				0.5, 1, // Bucket 0: (0.25, 1]
				1.5, 2, 3, 3.5, // Bucket 1: (1, 4]
				5, 6, 7, // Bucket 2: (4, 16]
				33.33, // Bucket 3: (16, 64]
			},
			factor: 4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(14),
				SampleSum:     proto.Float64(63.2581251),
				Schema:        proto.Int32(-1),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(-2), Length: proto.Uint32(6)},
				},
				PositiveDelta:    []int64{2, 0, 0, 2, -1, -2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name: "factor 17 results in schema -2",
			observations: []float64{
				0.0156251, 0.0625, // Bucket -1: (0.015625, 0.0625]
				0.1, 0.25, 0.5, 1, // Bucket 0: (0.0625, 1]
				1.5, 2, 3, 3.5, 5, 6, 7, // Bucket 1: (1, 16]
				33.33, // Bucket 2: (16, 256]
			},
			factor: 17,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(14),
				SampleSum:     proto.Float64(63.2581251),
				Schema:        proto.Int32(-2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(-1), Length: proto.Uint32(4)},
				},
				PositiveDelta:    []int64{2, 2, 3, -6},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "negative buckets",
			observations: []float64{0, -1, -1.2, -1.4, -1.8, -2},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(6),
				SampleSum:     proto.Float64(-7.4),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				NegativeDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "negative and positive buckets",
			observations: []float64{0, -1, -1.2, -1.4, -1.8, -2, 1, 1.2, 1.4, 1.8, 2},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(11),
				SampleSum:     proto.Float64(0),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				NegativeDelta: []int64{1, -1, 2, -2, 2},
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:          "wide zero bucket",
			observations:  []float64{0, -1, -1.2, -1.4, -1.8, -2, 1, 1.2, 1.4, 1.8, 2},
			factor:        1.2,
			zeroThreshold: 1.4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(11),
				SampleSum:     proto.Float64(0),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(1.4),
				ZeroCount:     proto.Uint64(7),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(4), Length: proto.Uint32(1)},
				},
				NegativeDelta: []int64{2},
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(4), Length: proto.Uint32(1)},
				},
				PositiveDelta:    []int64{2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "NaN observation",
			observations: []float64{0, 1, 1.2, 1.4, 1.8, 2, math.NaN()},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(7),
				SampleSum:     proto.Float64(math.NaN()),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "+Inf observation",
			observations: []float64{0, 1, 1.2, 1.4, 1.8, 2, math.Inf(+1)},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(7),
				SampleSum:     proto.Float64(math.Inf(+1)),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
					{Offset: proto.Int32(4092), Length: proto.Uint32(1)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2, -1},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "-Inf observation",
			observations: []float64{0, 1, 1.2, 1.4, 1.8, 2, math.Inf(-1)},
			factor:       1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(7),
				SampleSum:     proto.Float64(math.Inf(-1)),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(4097), Length: proto.Uint32(1)},
				},
				NegativeDelta: []int64{1},
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "limited buckets but nothing triggered",
			observations: []float64{0, 1, 1.2, 1.4, 1.8, 2},
			factor:       1.2,
			maxBuckets:   4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(6),
				SampleSum:     proto.Float64(7.4),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "buckets limited by halving resolution",
			observations: []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 3},
			factor:       1.2,
			maxBuckets:   4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(8),
				SampleSum:     proto.Float64(11.5),
				Schema:        proto.Int32(1),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				PositiveDelta:    []int64{1, 2, -1, -2, 1},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by widening the zero bucket",
			observations:     []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 3},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(8),
				SampleSum:     proto.Float64(11.5),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(1),
				ZeroCount:     proto.Uint64(2),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(1), Length: proto.Uint32(7)},
				},
				PositiveDelta:    []int64{1, 1, -2, 2, -2, 0, 1},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by widening the zero bucket twice",
			observations:     []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 3, 4},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(9),
				SampleSum:     proto.Float64(15.5),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(1.189207115002721),
				ZeroCount:     proto.Uint64(3),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(2), Length: proto.Uint32(7)},
				},
				PositiveDelta:    []int64{2, -2, 2, -2, 0, 1, 0},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by reset",
			observations:     []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 3, 4},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			minResetDuration: 5 * time.Minute,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(2),
				SampleSum:     proto.Float64(7),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(7), Length: proto.Uint32(2)},
				},
				PositiveDelta:    []int64{1, 0},
				CreatedTimestamp: timestamppb.New(now.Add(8 * time.Minute)), // We expect reset to happen after 8 observations.
			},
		},
		{
			name:         "limited buckets but nothing triggered, negative observations",
			observations: []float64{0, -1, -1.2, -1.4, -1.8, -2},
			factor:       1.2,
			maxBuckets:   4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(6),
				SampleSum:     proto.Float64(-7.4),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				NegativeDelta:    []int64{1, -1, 2, -2, 2},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:         "buckets limited by halving resolution, negative observations",
			observations: []float64{0, -1, -1.1, -1.2, -1.4, -1.8, -2, -3},
			factor:       1.2,
			maxBuckets:   4,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(8),
				SampleSum:     proto.Float64(-11.5),
				Schema:        proto.Int32(1),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(1),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(0), Length: proto.Uint32(5)},
				},
				NegativeDelta:    []int64{1, 2, -1, -2, 1},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by widening the zero bucket, negative observations",
			observations:     []float64{0, -1, -1.1, -1.2, -1.4, -1.8, -2, -3},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(8),
				SampleSum:     proto.Float64(-11.5),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(1),
				ZeroCount:     proto.Uint64(2),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(1), Length: proto.Uint32(7)},
				},
				NegativeDelta:    []int64{1, 1, -2, 2, -2, 0, 1},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by widening the zero bucket twice, negative observations",
			observations:     []float64{0, -1, -1.1, -1.2, -1.4, -1.8, -2, -3, -4},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(9),
				SampleSum:     proto.Float64(-15.5),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(1.189207115002721),
				ZeroCount:     proto.Uint64(3),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(2), Length: proto.Uint32(7)},
				},
				NegativeDelta:    []int64{2, -2, 2, -2, 0, 1, 0},
				CreatedTimestamp: timestamppb.New(now),
			},
		},
		{
			name:             "buckets limited by reset, negative observations",
			observations:     []float64{0, -1, -1.1, -1.2, -1.4, -1.8, -2, -3, -4},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			minResetDuration: 5 * time.Minute,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(2),
				SampleSum:     proto.Float64(-7),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				NegativeSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(7), Length: proto.Uint32(2)},
				},
				NegativeDelta:    []int64{1, 0},
				CreatedTimestamp: timestamppb.New(now.Add(8 * time.Minute)), // We expect reset to happen after 8 observations.
			},
		},
		{
			name:             "buckets limited by halving resolution, then reset",
			observations:     []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 5, 5.1, 3, 4},
			factor:           1.2,
			maxBuckets:       4,
			minResetDuration: 9 * time.Minute,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(3),
				SampleSum:     proto.Float64(12.1),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(7), Length: proto.Uint32(4)},
				},
				PositiveDelta:    []int64{1, 0, -1, 1},
				CreatedTimestamp: timestamppb.New(now.Add(9 * time.Minute)), // We expect reset to happen after 8 minutes.
			},
		},
		{
			name:             "buckets limited by widening the zero bucket, then reset",
			observations:     []float64{0, 1, 1.1, 1.2, 1.4, 1.8, 2, 5, 5.1, 3, 4},
			factor:           1.2,
			maxBuckets:       4,
			maxZeroThreshold: 1.2,
			minResetDuration: 9 * time.Minute,
			want: &dto.Histogram{
				SampleCount:   proto.Uint64(3),
				SampleSum:     proto.Float64(12.1),
				Schema:        proto.Int32(2),
				ZeroThreshold: proto.Float64(2.938735877055719e-39),
				ZeroCount:     proto.Uint64(0),
				PositiveSpan: []*dto.BucketSpan{
					{Offset: proto.Int32(7), Length: proto.Uint32(4)},
				},
				PositiveDelta:    []int64{1, 0, -1, 1},
				CreatedTimestamp: timestamppb.New(now.Add(9 * time.Minute)), // We expect reset to happen after 8 minutes.
			},
		},
	}

	for _, s := range scenarios {
		t.Run(s.name, func(t *testing.T) {
			var (
				ts         = now
				funcToCall func()
				whenToCall time.Duration
			)

			his := NewHistogram(HistogramOpts{
				Name:                            "name",
				Help:                            "help",
				NativeHistogramBucketFactor:     s.factor,
				NativeHistogramZeroThreshold:    s.zeroThreshold,
				NativeHistogramMaxBucketNumber:  s.maxBuckets,
				NativeHistogramMinResetDuration: s.minResetDuration,
				NativeHistogramMaxZeroThreshold: s.maxZeroThreshold,
				now:                             func() time.Time { return ts },
				afterFunc: func(d time.Duration, f func()) *time.Timer {
					funcToCall = f
					whenToCall = d
					return nil
				},
			})

			ts = ts.Add(time.Minute)
			for _, o := range s.observations {
				his.Observe(o)
				ts = ts.Add(time.Minute)
				whenToCall -= time.Minute
				if funcToCall != nil && whenToCall <= 0 {
					funcToCall()
					funcToCall = nil
				}
			}
			m := &dto.Metric{}
			if err := his.Write(m); err != nil {
				t.Fatal("unexpected error writing metric", err)
			}
			got := m.Histogram
			if !proto.Equal(s.want, got) {
				t.Errorf("want histogram %q, got %q", s.want, got)
			}
		})
	}
}

func TestNativeHistogramConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("Skipping test in short mode.")
	}

	rand.New(rand.NewSource(42))

	it := func(n uint32) bool {
		ts := time.Now().Add(30 * time.Second).Unix()

		mutations := int(n%1e4 + 1e4)
		concLevel := int(n%5 + 1)
		total := mutations * concLevel

		var start, end sync.WaitGroup
		start.Add(1)
		end.Add(concLevel)

		his := NewHistogram(HistogramOpts{
			Name:                            "test_native_histogram",
			Help:                            "This help is sparse.",
			NativeHistogramBucketFactor:     1.05,
			NativeHistogramZeroThreshold:    0.0000001,
			NativeHistogramMaxBucketNumber:  50,
			NativeHistogramMinResetDuration: time.Hour, // Comment out to test for totals below.
			NativeHistogramMaxZeroThreshold: 0.001,
			now: func() time.Time {
				return time.Unix(atomic.LoadInt64(&ts), 0)
			},
		})

		allVars := make([]float64, total)
		var sampleSum float64
		for i := 0; i < concLevel; i++ {
			vals := make([]float64, mutations)
			for j := 0; j < mutations; j++ {
				v := rand.NormFloat64()
				vals[j] = v
				allVars[i*mutations+j] = v
				sampleSum += v
			}

			go func(vals []float64) {
				start.Wait()
				for i, v := range vals {
					// An observation every 1 to 10 seconds.
					atomic.AddInt64(&ts, rand.Int63n(10)+1)
					if i%2 == 0 {
						his.Observe(v)
					} else {
						his.(ExemplarObserver).ObserveWithExemplar(v, Labels{"foo": "bar"})
					}
				}
				end.Done()
			}(vals)
		}
		sort.Float64s(allVars)
		start.Done()
		end.Wait()

		m := &dto.Metric{}
		his.Write(m)

		// Uncomment these tests for totals only if you have disabled histogram resets above.
		//
		// if got, want := int(*m.Histogram.SampleCount), total; got != want {
		// 	t.Errorf("got sample count %d, want %d", got, want)
		// }
		// if got, want := *m.Histogram.SampleSum, sampleSum; math.Abs((got-want)/want) > 0.001 {
		// 	t.Errorf("got sample sum %f, want %f", got, want)
		// }

		sumBuckets := int(m.Histogram.GetZeroCount())
		current := 0
		for _, delta := range m.Histogram.GetNegativeDelta() {
			current += int(delta)
			if current < 0 {
				t.Fatalf("negative bucket population negative: %d", current)
			}
			sumBuckets += current
		}
		current = 0
		for _, delta := range m.Histogram.GetPositiveDelta() {
			current += int(delta)
			if current < 0 {
				t.Fatalf("positive bucket population negative: %d", current)
			}
			sumBuckets += current
		}
		if got, want := sumBuckets, int(*m.Histogram.SampleCount); got != want {
			t.Errorf("got bucket population sum %d, want %d", got, want)
		}

		return true
	}

	if err := quick.Check(it, nil); err != nil {
		t.Error(err)
	}
}

func TestGetLe(t *testing.T) {
	scenarios := []struct {
		key    int
		schema int32
		want   float64
	}{
		{
			key:    -1,
			schema: -1,
			want:   0.25,
		},
		{
			key:    0,
			schema: -1,
			want:   1,
		},
		{
			key:    1,
			schema: -1,
			want:   4,
		},
		{
			key:    512,
			schema: -1,
			want:   math.MaxFloat64,
		},
		{
			key:    513,
			schema: -1,
			want:   math.Inf(+1),
		},
		{
			key:    -1,
			schema: 0,
			want:   0.5,
		},
		{
			key:    0,
			schema: 0,
			want:   1,
		},
		{
			key:    1,
			schema: 0,
			want:   2,
		},
		{
			key:    1024,
			schema: 0,
			want:   math.MaxFloat64,
		},
		{
			key:    1025,
			schema: 0,
			want:   math.Inf(+1),
		},
		{
			key:    -1,
			schema: 2,
			want:   0.8408964152537144,
		},
		{
			key:    0,
			schema: 2,
			want:   1,
		},
		{
			key:    1,
			schema: 2,
			want:   1.189207115002721,
		},
		{
			key:    4096,
			schema: 2,
			want:   math.MaxFloat64,
		},
		{
			key:    4097,
			schema: 2,
			want:   math.Inf(+1),
		},
	}

	for i, s := range scenarios {
		got := getLe(s.key, s.schema)
		if s.want != got {
			t.Errorf("%d. key %d, schema %d, want upper bound of %g, got %g", i, s.key, s.schema, s.want, got)
		}
	}
}

func TestHistogramCreatedTimestamp(t *testing.T) {
	now := time.Now()

	histogram := NewHistogram(HistogramOpts{
		Name:    "test",
		Help:    "test help",
		Buckets: []float64{1, 2, 3, 4},
		now:     func() time.Time { return now },
	})

	var metric dto.Metric
	if err := histogram.Write(&metric); err != nil {
		t.Fatal(err)
	}

	if metric.Histogram.CreatedTimestamp.AsTime().Unix() != now.Unix() {
		t.Errorf("expected created timestamp %d, got %d", now.Unix(), metric.Histogram.CreatedTimestamp.AsTime().Unix())
	}
}

func TestHistogramVecCreatedTimestamp(t *testing.T) {
	now := time.Now()

	histogramVec := NewHistogramVec(HistogramOpts{
		Name:    "test",
		Help:    "test help",
		Buckets: []float64{1, 2, 3, 4},
		now:     func() time.Time { return now },
	}, []string{"label"})
	histogram := histogramVec.WithLabelValues("value").(Histogram)

	var metric dto.Metric
	if err := histogram.Write(&metric); err != nil {
		t.Fatal(err)
	}

	if metric.Histogram.CreatedTimestamp.AsTime().Unix() != now.Unix() {
		t.Errorf("expected created timestamp %d, got %d", now.Unix(), metric.Histogram.CreatedTimestamp.AsTime().Unix())
	}
}

func TestHistogramVecCreatedTimestampWithDeletes(t *testing.T) {
	now := time.Now()

	histogramVec := NewHistogramVec(HistogramOpts{
		Name:    "test",
		Help:    "test help",
		Buckets: []float64{1, 2, 3, 4},
		now:     func() time.Time { return now },
	}, []string{"label"})

	// First use of "With" should populate CT.
	histogramVec.WithLabelValues("1")
	expected := map[string]time.Time{"1": now}

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, histogramVec.MetricVec, dto.MetricType_HISTOGRAM, expected)

	// Two more labels at different times.
	histogramVec.WithLabelValues("2")
	expected["2"] = now

	now = now.Add(1 * time.Hour)

	histogramVec.WithLabelValues("3")
	expected["3"] = now

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, histogramVec.MetricVec, dto.MetricType_HISTOGRAM, expected)

	// Recreate metric instance should reset created timestamp to now.
	histogramVec.DeleteLabelValues("1")
	histogramVec.WithLabelValues("1")
	expected["1"] = now

	now = now.Add(1 * time.Hour)
	expectCTsForMetricVecValues(t, histogramVec.MetricVec, dto.MetricType_HISTOGRAM, expected)
}

func TestNewConstHistogramWithCreatedTimestamp(t *testing.T) {
	metricDesc := NewDesc(
		"sample_value",
		"sample value",
		nil,
		nil,
	)
	buckets := map[float64]uint64{25: 100, 50: 200}
	createdTs := time.Unix(1719670764, 123)

	h, err := NewConstHistogramWithCreatedTimestamp(metricDesc, 100, 200, buckets, createdTs)
	if err != nil {
		t.Fatal(err)
	}

	var metric dto.Metric
	if err := h.Write(&metric); err != nil {
		t.Fatal(err)
	}

	if metric.Histogram.CreatedTimestamp.AsTime().UnixMicro() != createdTs.UnixMicro() {
		t.Errorf("Expected created timestamp %v, got %v", createdTs, &metric.Histogram.CreatedTimestamp)
	}
}

func TestNativeHistogramExemplar(t *testing.T) {
	// Test the histogram with positive NativeHistogramExemplarTTL and NativeHistogramMaxExemplars
	h := NewHistogram(HistogramOpts{
		Name:                        "test",
		Help:                        "test help",
		Buckets:                     []float64{1, 2, 3, 4},
		NativeHistogramBucketFactor: 1.1,
		NativeHistogramMaxExemplars: 3,
		NativeHistogramExemplarTTL:  10 * time.Second,
	}).(*histogram)

	tcs := []struct {
		name           string
		addFunc        func(*histogram)
		expectedValues []float64
	}{
		{
			name: "add exemplars to the limit",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(1, Labels{"id": "1"})
				h.ObserveWithExemplar(3, Labels{"id": "1"})
				h.ObserveWithExemplar(5, Labels{"id": "1"})
			},
			expectedValues: []float64{1, 3, 5},
		},
		{
			name: "remove exemplar in closest pair, the removed index equals to inserted index",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(4, Labels{"id": "1"})
			},
			expectedValues: []float64{1, 3, 4},
		},
		{
			name: "remove exemplar in closest pair, the removed index is bigger than inserted index",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(0, Labels{"id": "1"})
			},
			expectedValues: []float64{0, 1, 4},
		},
		{
			name: "remove exemplar with oldest timestamp, the removed index is smaller than inserted index",
			addFunc: func(h *histogram) {
				h.now = func() time.Time { return time.Now().Add(time.Second * 11) }
				h.ObserveWithExemplar(6, Labels{"id": "1"})
			},
			expectedValues: []float64{0, 4, 6},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			tc.addFunc(h)
			compareNativeExemplarValues(t, h.nativeExemplars.exemplars, tc.expectedValues)
		})
	}

	// Test the histogram with negative NativeHistogramExemplarTTL
	h = NewHistogram(HistogramOpts{
		Name:                        "test",
		Help:                        "test help",
		Buckets:                     []float64{1, 2, 3, 4},
		NativeHistogramBucketFactor: 1.1,
		NativeHistogramMaxExemplars: 3,
		NativeHistogramExemplarTTL:  -1 * time.Second,
	}).(*histogram)

	tcs = []struct {
		name           string
		addFunc        func(*histogram)
		expectedValues []float64
	}{
		{
			name: "add exemplars to the limit",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(1, Labels{"id": "1"})
				h.ObserveWithExemplar(3, Labels{"id": "1"})
				h.ObserveWithExemplar(5, Labels{"id": "1"})
			},
			expectedValues: []float64{1, 3, 5},
		},
		{
			name: "remove exemplar with oldest timestamp, the removed index is smaller than inserted index",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(4, Labels{"id": "1"})
			},
			expectedValues: []float64{3, 4, 5},
		},
		{
			name: "remove exemplar with oldest timestamp, the removed index equals to inserted index",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(0, Labels{"id": "1"})
			},
			expectedValues: []float64{0, 4, 5},
		},
		{
			name: "remove exemplar with oldest timestamp, the removed index is bigger than inserted index",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(3, Labels{"id": "1"})
			},
			expectedValues: []float64{0, 3, 4},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			tc.addFunc(h)
			compareNativeExemplarValues(t, h.nativeExemplars.exemplars, tc.expectedValues)
		})
	}

	// Test the histogram with negative NativeHistogramMaxExemplars
	h = NewHistogram(HistogramOpts{
		Name:                        "test",
		Help:                        "test help",
		Buckets:                     []float64{1, 2, 3, 4},
		NativeHistogramBucketFactor: 1.1,
		NativeHistogramMaxExemplars: -1,
		NativeHistogramExemplarTTL:  -1 * time.Second,
	}).(*histogram)

	tcs = []struct {
		name           string
		addFunc        func(*histogram)
		expectedValues []float64
	}{
		{
			name: "add exemplars to the limit, but no effect",
			addFunc: func(h *histogram) {
				h.ObserveWithExemplar(1, Labels{"id": "1"})
				h.ObserveWithExemplar(3, Labels{"id": "1"})
				h.ObserveWithExemplar(5, Labels{"id": "1"})
			},
			expectedValues: []float64{},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			tc.addFunc(h)
			compareNativeExemplarValues(t, h.nativeExemplars.exemplars, tc.expectedValues)
		})
	}
}

func compareNativeExemplarValues(t *testing.T, exps []*dto.Exemplar, values []float64) {
	if len(exps) != len(values) {
		t.Errorf("the count of exemplars is not %d", len(values))
	}
	for i, e := range exps {
		if e.GetValue() != values[i] {
			t.Errorf("the %dth exemplar value %v is not as expected: %v", i, e.GetValue(), values[i])
		}
	}
}
