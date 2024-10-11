package metrics

import (
	"math"
	"runtime/metrics"
	"sort"
	"sync/atomic"
)

func getOrRegisterRuntimeHistogram(name string, scale float64, r Registry) *runtimeHistogram {
	if r == nil {
		r = DefaultRegistry
	}
	constructor := func() Histogram { return newRuntimeHistogram(scale) }
	return r.GetOrRegister(name, constructor).(*runtimeHistogram)
}

// runtimeHistogram wraps a runtime/metrics histogram.
type runtimeHistogram struct {
	v           atomic.Value // v is a pointer to a metrics.Float64Histogram
	scaleFactor float64
}

func newRuntimeHistogram(scale float64) *runtimeHistogram {
	h := &runtimeHistogram{scaleFactor: scale}
	h.update(new(metrics.Float64Histogram))
	return h
}

func RuntimeHistogramFromData(scale float64, hist *metrics.Float64Histogram) *runtimeHistogram {
	h := &runtimeHistogram{scaleFactor: scale}
	h.update(hist)
	return h
}

func (h *runtimeHistogram) update(mh *metrics.Float64Histogram) {
	if mh == nil {
		// The update value can be nil if the current Go version doesn't support a
		// requested metric. It's just easier to handle nil here than putting
		// conditionals everywhere.
		return
	}

	s := metrics.Float64Histogram{
		Counts:  make([]uint64, len(mh.Counts)),
		Buckets: make([]float64, len(mh.Buckets)),
	}
	copy(s.Counts, mh.Counts)
	for i, b := range mh.Buckets {
		s.Buckets[i] = b * h.scaleFactor
	}
	h.v.Store(&s)
}

func (h *runtimeHistogram) Clear() {
	panic("runtimeHistogram does not support Clear")
}
func (h *runtimeHistogram) Update(int64) {
	panic("runtimeHistogram does not support Update")
}

// Snapshot returns a non-changing copy of the histogram.
func (h *runtimeHistogram) Snapshot() HistogramSnapshot {
	hist := h.v.Load().(*metrics.Float64Histogram)
	return newRuntimeHistogramSnapshot(hist)
}

type runtimeHistogramSnapshot struct {
	internal   *metrics.Float64Histogram
	calculated bool
	// The following fields are (lazily) calculated based on 'internal'
	mean     float64
	count    int64
	min      int64 // min is the lowest sample value.
	max      int64 // max is the highest sample value.
	variance float64
}

func newRuntimeHistogramSnapshot(h *metrics.Float64Histogram) *runtimeHistogramSnapshot {
	return &runtimeHistogramSnapshot{
		internal: h,
	}
}

// calc calculates the values for the snapshot. This method is not threadsafe.
func (h *runtimeHistogramSnapshot) calc() {
	h.calculated = true
	var (
		count int64   // number of samples
		sum   float64 // approx sum of all sample values
		min   int64
		max   float64
	)
	if len(h.internal.Counts) == 0 {
		return
	}
	for i, c := range h.internal.Counts {
		if c == 0 {
			continue
		}
		if count == 0 { // Set min only first loop iteration
			min = int64(math.Floor(h.internal.Buckets[i]))
		}
		count += int64(c)
		sum += h.midpoint(i) * float64(c)
		// Set max on every iteration
		edge := h.internal.Buckets[i+1]
		if math.IsInf(edge, 1) {
			edge = h.internal.Buckets[i]
		}
		if edge > max {
			max = edge
		}
	}
	h.min = min
	h.max = int64(max)
	h.mean = sum / float64(count)
	h.count = count
}

// Count returns the sample count.
func (h *runtimeHistogramSnapshot) Count() int64 {
	if !h.calculated {
		h.calc()
	}
	return h.count
}

// Size returns the size of the sample at the time the snapshot was taken.
func (h *runtimeHistogramSnapshot) Size() int {
	return len(h.internal.Counts)
}

// Mean returns an approximation of the mean.
func (h *runtimeHistogramSnapshot) Mean() float64 {
	if !h.calculated {
		h.calc()
	}
	return h.mean
}

func (h *runtimeHistogramSnapshot) midpoint(bucket int) float64 {
	high := h.internal.Buckets[bucket+1]
	low := h.internal.Buckets[bucket]
	if math.IsInf(high, 1) {
		// The edge of the highest bucket can be +Inf, and it's supposed to mean that this
		// bucket contains all remaining samples > low. We can't get the middle of an
		// infinite range, so just return the lower bound of this bucket instead.
		return low
	}
	if math.IsInf(low, -1) {
		// Similarly, we can get -Inf in the left edge of the lowest bucket,
		// and it means the bucket contains all remaining values < high.
		return high
	}
	return (low + high) / 2
}

// StdDev approximates the standard deviation of the histogram.
func (h *runtimeHistogramSnapshot) StdDev() float64 {
	return math.Sqrt(h.Variance())
}

// Variance approximates the variance of the histogram.
func (h *runtimeHistogramSnapshot) Variance() float64 {
	if len(h.internal.Counts) == 0 {
		return 0
	}
	if !h.calculated {
		h.calc()
	}
	if h.count <= 1 {
		// There is no variance when there are zero or one items.
		return 0
	}
	// Variance is not calculated in 'calc', because it requires a second iteration.
	// Therefore we calculate it lazily in this method, triggered either by
	// a direct call to Variance or via StdDev.
	if h.variance != 0.0 {
		return h.variance
	}
	var sum float64

	for i, c := range h.internal.Counts {
		midpoint := h.midpoint(i)
		d := midpoint - h.mean
		sum += float64(c) * (d * d)
	}
	h.variance = sum / float64(h.count-1)
	return h.variance
}

// Percentile computes the p'th percentile value.
func (h *runtimeHistogramSnapshot) Percentile(p float64) float64 {
	threshold := float64(h.Count()) * p
	values := [1]float64{threshold}
	h.computePercentiles(values[:])
	return values[0]
}

// Percentiles computes all requested percentile values.
func (h *runtimeHistogramSnapshot) Percentiles(ps []float64) []float64 {
	// Compute threshold values. We need these to be sorted
	// for the percentile computation, but restore the original
	// order later, so keep the indexes as well.
	count := float64(h.Count())
	thresholds := make([]float64, len(ps))
	indexes := make([]int, len(ps))
	for i, percentile := range ps {
		thresholds[i] = count * math.Max(0, math.Min(1.0, percentile))
		indexes[i] = i
	}
	sort.Sort(floatsAscendingKeepingIndex{thresholds, indexes})

	// Now compute. The result is stored back into the thresholds slice.
	h.computePercentiles(thresholds)

	// Put the result back into the requested order.
	sort.Sort(floatsByIndex{thresholds, indexes})
	return thresholds
}

func (h *runtimeHistogramSnapshot) computePercentiles(thresh []float64) {
	var totalCount float64
	for i, count := range h.internal.Counts {
		totalCount += float64(count)

		for len(thresh) > 0 && thresh[0] < totalCount {
			thresh[0] = h.internal.Buckets[i]
			thresh = thresh[1:]
		}
		if len(thresh) == 0 {
			return
		}
	}
}

// Note: runtime/metrics.Float64Histogram is a collection of float64s, but the methods
// below need to return int64 to satisfy the interface. The histogram provided by runtime
// also doesn't keep track of individual samples, so results are approximated.

// Max returns the highest sample value.
func (h *runtimeHistogramSnapshot) Max() int64 {
	if !h.calculated {
		h.calc()
	}
	return h.max
}

// Min returns the lowest sample value.
func (h *runtimeHistogramSnapshot) Min() int64 {
	if !h.calculated {
		h.calc()
	}
	return h.min
}

// Sum returns the sum of all sample values.
func (h *runtimeHistogramSnapshot) Sum() int64 {
	var sum float64
	for i := range h.internal.Counts {
		sum += h.internal.Buckets[i] * float64(h.internal.Counts[i])
	}
	return int64(math.Ceil(sum))
}

type floatsAscendingKeepingIndex struct {
	values  []float64
	indexes []int
}

func (s floatsAscendingKeepingIndex) Len() int {
	return len(s.values)
}

func (s floatsAscendingKeepingIndex) Less(i, j int) bool {
	return s.values[i] < s.values[j]
}

func (s floatsAscendingKeepingIndex) Swap(i, j int) {
	s.values[i], s.values[j] = s.values[j], s.values[i]
	s.indexes[i], s.indexes[j] = s.indexes[j], s.indexes[i]
}

type floatsByIndex struct {
	values  []float64
	indexes []int
}

func (s floatsByIndex) Len() int {
	return len(s.values)
}

func (s floatsByIndex) Less(i, j int) bool {
	return s.indexes[i] < s.indexes[j]
}

func (s floatsByIndex) Swap(i, j int) {
	s.values[i], s.values[j] = s.values[j], s.values[i]
	s.indexes[i], s.indexes[j] = s.indexes[j], s.indexes[i]
}
