package metrics

import (
	"math"
	"math/rand"
	"sync"
	"time"

	"golang.org/x/exp/slices"
)

const rescaleThreshold = time.Hour

type SampleSnapshot interface {
	Count() int64
	Max() int64
	Mean() float64
	Min() int64
	Percentile(float64) float64
	Percentiles([]float64) []float64
	Size() int
	StdDev() float64
	Sum() int64
	Variance() float64
}

// Samples maintain a statistically-significant selection of values from
// a stream.
type Sample interface {
	Snapshot() SampleSnapshot
	Clear()
	Update(int64)
}

// ExpDecaySample is an exponentially-decaying sample using a forward-decaying
// priority reservoir.  See Cormode et al's "Forward Decay: A Practical Time
// Decay Model for Streaming Systems".
//
// <http://dimacs.rutgers.edu/~graham/pubs/papers/fwddecay.pdf>
type ExpDecaySample struct {
	alpha         float64
	count         int64
	mutex         sync.Mutex
	reservoirSize int
	t0, t1        time.Time
	values        *expDecaySampleHeap
	rand          *rand.Rand
}

// NewExpDecaySample constructs a new exponentially-decaying sample with the
// given reservoir size and alpha.
func NewExpDecaySample(reservoirSize int, alpha float64) Sample {
	if !Enabled {
		return NilSample{}
	}
	s := &ExpDecaySample{
		alpha:         alpha,
		reservoirSize: reservoirSize,
		t0:            time.Now(),
		values:        newExpDecaySampleHeap(reservoirSize),
	}
	s.t1 = s.t0.Add(rescaleThreshold)
	return s
}

// SetRand sets the random source (useful in tests)
func (s *ExpDecaySample) SetRand(prng *rand.Rand) Sample {
	s.rand = prng
	return s
}

// Clear clears all samples.
func (s *ExpDecaySample) Clear() {
	s.mutex.Lock()
	defer s.mutex.Unlock()
	s.count = 0
	s.t0 = time.Now()
	s.t1 = s.t0.Add(rescaleThreshold)
	s.values.Clear()
}

// Snapshot returns a read-only copy of the sample.
func (s *ExpDecaySample) Snapshot() SampleSnapshot {
	s.mutex.Lock()
	defer s.mutex.Unlock()
	var (
		samples       = s.values.Values()
		values        = make([]int64, len(samples))
		max     int64 = math.MinInt64
		min     int64 = math.MaxInt64
		sum     int64
	)
	for i, item := range samples {
		v := item.v
		values[i] = v
		sum += v
		if v > max {
			max = v
		}
		if v < min {
			min = v
		}
	}
	return newSampleSnapshotPrecalculated(s.count, values, min, max, sum)
}

// Update samples a new value.
func (s *ExpDecaySample) Update(v int64) {
	s.update(time.Now(), v)
}

// update samples a new value at a particular timestamp.  This is a method all
// its own to facilitate testing.
func (s *ExpDecaySample) update(t time.Time, v int64) {
	s.mutex.Lock()
	defer s.mutex.Unlock()
	s.count++
	if s.values.Size() == s.reservoirSize {
		s.values.Pop()
	}
	var f64 float64
	if s.rand != nil {
		f64 = s.rand.Float64()
	} else {
		f64 = rand.Float64()
	}
	s.values.Push(expDecaySample{
		k: math.Exp(t.Sub(s.t0).Seconds()*s.alpha) / f64,
		v: v,
	})
	if t.After(s.t1) {
		values := s.values.Values()
		t0 := s.t0
		s.values.Clear()
		s.t0 = t
		s.t1 = s.t0.Add(rescaleThreshold)
		for _, v := range values {
			v.k = v.k * math.Exp(-s.alpha*s.t0.Sub(t0).Seconds())
			s.values.Push(v)
		}
	}
}

// NilSample is a no-op Sample.
type NilSample struct{}

func (NilSample) Clear()                   {}
func (NilSample) Snapshot() SampleSnapshot { return (*emptySnapshot)(nil) }
func (NilSample) Update(v int64)           {}

// SamplePercentiles returns an arbitrary percentile of the slice of int64.
func SamplePercentile(values []int64, p float64) float64 {
	return CalculatePercentiles(values, []float64{p})[0]
}

// CalculatePercentiles returns a slice of arbitrary percentiles of the slice of
// int64. This method returns interpolated results, so e.g if there are only two
// values, [0, 10], a 50% percentile will land between them.
//
// Note: As a side-effect, this method will also sort the slice of values.
// Note2: The input format for percentiles is NOT percent! To express 50%, use 0.5, not 50.
func CalculatePercentiles(values []int64, ps []float64) []float64 {
	scores := make([]float64, len(ps))
	size := len(values)
	if size == 0 {
		return scores
	}
	slices.Sort(values)
	for i, p := range ps {
		pos := p * float64(size+1)

		if pos < 1.0 {
			scores[i] = float64(values[0])
		} else if pos >= float64(size) {
			scores[i] = float64(values[size-1])
		} else {
			lower := float64(values[int(pos)-1])
			upper := float64(values[int(pos)])
			scores[i] = lower + (pos-math.Floor(pos))*(upper-lower)
		}
	}
	return scores
}

// sampleSnapshot is a read-only copy of another Sample.
type sampleSnapshot struct {
	count  int64
	values []int64

	max      int64
	min      int64
	mean     float64
	sum      int64
	variance float64
}

// newSampleSnapshotPrecalculated creates a read-only sampleSnapShot, using
// precalculated sums to avoid iterating the values
func newSampleSnapshotPrecalculated(count int64, values []int64, min, max, sum int64) *sampleSnapshot {
	if len(values) == 0 {
		return &sampleSnapshot{
			count:  count,
			values: values,
		}
	}
	return &sampleSnapshot{
		count:  count,
		values: values,
		max:    max,
		min:    min,
		mean:   float64(sum) / float64(len(values)),
		sum:    sum,
	}
}

// newSampleSnapshot creates a read-only sampleSnapShot, and calculates some
// numbers.
func newSampleSnapshot(count int64, values []int64) *sampleSnapshot {
	var (
		max int64 = math.MinInt64
		min int64 = math.MaxInt64
		sum int64
	)
	for _, v := range values {
		sum += v
		if v > max {
			max = v
		}
		if v < min {
			min = v
		}
	}
	return newSampleSnapshotPrecalculated(count, values, min, max, sum)
}

// Count returns the count of inputs at the time the snapshot was taken.
func (s *sampleSnapshot) Count() int64 { return s.count }

// Max returns the maximal value at the time the snapshot was taken.
func (s *sampleSnapshot) Max() int64 { return s.max }

// Mean returns the mean value at the time the snapshot was taken.
func (s *sampleSnapshot) Mean() float64 { return s.mean }

// Min returns the minimal value at the time the snapshot was taken.
func (s *sampleSnapshot) Min() int64 { return s.min }

// Percentile returns an arbitrary percentile of values at the time the
// snapshot was taken.
func (s *sampleSnapshot) Percentile(p float64) float64 {
	return SamplePercentile(s.values, p)
}

// Percentiles returns a slice of arbitrary percentiles of values at the time
// the snapshot was taken.
func (s *sampleSnapshot) Percentiles(ps []float64) []float64 {
	return CalculatePercentiles(s.values, ps)
}

// Size returns the size of the sample at the time the snapshot was taken.
func (s *sampleSnapshot) Size() int { return len(s.values) }

// Snapshot returns the snapshot.
func (s *sampleSnapshot) Snapshot() SampleSnapshot { return s }

// StdDev returns the standard deviation of values at the time the snapshot was
// taken.
func (s *sampleSnapshot) StdDev() float64 {
	if s.variance == 0.0 {
		s.variance = SampleVariance(s.mean, s.values)
	}
	return math.Sqrt(s.variance)
}

// Sum returns the sum of values at the time the snapshot was taken.
func (s *sampleSnapshot) Sum() int64 { return s.sum }

// Values returns a copy of the values in the sample.
func (s *sampleSnapshot) Values() []int64 {
	values := make([]int64, len(s.values))
	copy(values, s.values)
	return values
}

// Variance returns the variance of values at the time the snapshot was taken.
func (s *sampleSnapshot) Variance() float64 {
	if s.variance == 0.0 {
		s.variance = SampleVariance(s.mean, s.values)
	}
	return s.variance
}

// SampleVariance returns the variance of the slice of int64.
func SampleVariance(mean float64, values []int64) float64 {
	if len(values) == 0 {
		return 0.0
	}
	var sum float64
	for _, v := range values {
		d := float64(v) - mean
		sum += d * d
	}
	return sum / float64(len(values))
}

// A uniform sample using Vitter's Algorithm R.
//
// <http://www.cs.umd.edu/~samir/498/vitter.pdf>
type UniformSample struct {
	count         int64
	mutex         sync.Mutex
	reservoirSize int
	values        []int64
	rand          *rand.Rand
}

// NewUniformSample constructs a new uniform sample with the given reservoir
// size.
func NewUniformSample(reservoirSize int) Sample {
	if !Enabled {
		return NilSample{}
	}
	return &UniformSample{
		reservoirSize: reservoirSize,
		values:        make([]int64, 0, reservoirSize),
	}
}

// SetRand sets the random source (useful in tests)
func (s *UniformSample) SetRand(prng *rand.Rand) Sample {
	s.rand = prng
	return s
}

// Clear clears all samples.
func (s *UniformSample) Clear() {
	s.mutex.Lock()
	defer s.mutex.Unlock()
	s.count = 0
	s.values = make([]int64, 0, s.reservoirSize)
}

// Snapshot returns a read-only copy of the sample.
func (s *UniformSample) Snapshot() SampleSnapshot {
	s.mutex.Lock()
	values := make([]int64, len(s.values))
	copy(values, s.values)
	count := s.count
	s.mutex.Unlock()
	return newSampleSnapshot(count, values)
}

// Update samples a new value.
func (s *UniformSample) Update(v int64) {
	s.mutex.Lock()
	defer s.mutex.Unlock()
	s.count++
	if len(s.values) < s.reservoirSize {
		s.values = append(s.values, v)
	} else {
		var r int64
		if s.rand != nil {
			r = s.rand.Int63n(s.count)
		} else {
			r = rand.Int63n(s.count)
		}
		if r < int64(len(s.values)) {
			s.values[int(r)] = v
		}
	}
}

// expDecaySample represents an individual sample in a heap.
type expDecaySample struct {
	k float64
	v int64
}

func newExpDecaySampleHeap(reservoirSize int) *expDecaySampleHeap {
	return &expDecaySampleHeap{make([]expDecaySample, 0, reservoirSize)}
}

// expDecaySampleHeap is a min-heap of expDecaySamples.
// The internal implementation is copied from the standard library's container/heap
type expDecaySampleHeap struct {
	s []expDecaySample
}

func (h *expDecaySampleHeap) Clear() {
	h.s = h.s[:0]
}

func (h *expDecaySampleHeap) Push(s expDecaySample) {
	n := len(h.s)
	h.s = h.s[0 : n+1]
	h.s[n] = s
	h.up(n)
}

func (h *expDecaySampleHeap) Pop() expDecaySample {
	n := len(h.s) - 1
	h.s[0], h.s[n] = h.s[n], h.s[0]
	h.down(0, n)

	n = len(h.s)
	s := h.s[n-1]
	h.s = h.s[0 : n-1]
	return s
}

func (h *expDecaySampleHeap) Size() int {
	return len(h.s)
}

func (h *expDecaySampleHeap) Values() []expDecaySample {
	return h.s
}

func (h *expDecaySampleHeap) up(j int) {
	for {
		i := (j - 1) / 2 // parent
		if i == j || !(h.s[j].k < h.s[i].k) {
			break
		}
		h.s[i], h.s[j] = h.s[j], h.s[i]
		j = i
	}
}

func (h *expDecaySampleHeap) down(i, n int) {
	for {
		j1 := 2*i + 1
		if j1 >= n || j1 < 0 { // j1 < 0 after int overflow
			break
		}
		j := j1 // left child
		if j2 := j1 + 1; j2 < n && !(h.s[j1].k < h.s[j2].k) {
			j = j2 // = 2*i + 2  // right child
		}
		if !(h.s[j].k < h.s[i].k) {
			break
		}
		h.s[i], h.s[j] = h.s[j], h.s[i]
		i = j
	}
}
