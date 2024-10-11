package metrics

type HistogramSnapshot interface {
	SampleSnapshot
}

// Histograms calculate distribution statistics from a series of int64 values.
type Histogram interface {
	Clear()
	Update(int64)
	Snapshot() HistogramSnapshot
}

// GetOrRegisterHistogram returns an existing Histogram or constructs and
// registers a new StandardHistogram.
func GetOrRegisterHistogram(name string, r Registry, s Sample) Histogram {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, func() Histogram { return NewHistogram(s) }).(Histogram)
}

// GetOrRegisterHistogramLazy returns an existing Histogram or constructs and
// registers a new StandardHistogram.
func GetOrRegisterHistogramLazy(name string, r Registry, s func() Sample) Histogram {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, func() Histogram { return NewHistogram(s()) }).(Histogram)
}

// NewHistogram constructs a new StandardHistogram from a Sample.
func NewHistogram(s Sample) Histogram {
	if !Enabled {
		return NilHistogram{}
	}
	return &StandardHistogram{sample: s}
}

// NewRegisteredHistogram constructs and registers a new StandardHistogram from
// a Sample.
func NewRegisteredHistogram(name string, r Registry, s Sample) Histogram {
	c := NewHistogram(s)
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// NilHistogram is a no-op Histogram.
type NilHistogram struct{}

func (NilHistogram) Clear()                      {}
func (NilHistogram) Snapshot() HistogramSnapshot { return (*emptySnapshot)(nil) }
func (NilHistogram) Update(v int64)              {}

// StandardHistogram is the standard implementation of a Histogram and uses a
// Sample to bound its memory use.
type StandardHistogram struct {
	sample Sample
}

// Clear clears the histogram and its sample.
func (h *StandardHistogram) Clear() { h.sample.Clear() }

// Snapshot returns a read-only copy of the histogram.
func (h *StandardHistogram) Snapshot() HistogramSnapshot {
	return h.sample.Snapshot()
}

// Update samples a new value.
func (h *StandardHistogram) Update(v int64) { h.sample.Update(v) }
