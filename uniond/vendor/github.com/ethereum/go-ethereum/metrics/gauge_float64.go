package metrics

import (
	"math"
	"sync/atomic"
)

type GaugeFloat64Snapshot interface {
	Value() float64
}

// GaugeFloat64 hold a float64 value that can be set arbitrarily.
type GaugeFloat64 interface {
	Snapshot() GaugeFloat64Snapshot
	Update(float64)
}

// GetOrRegisterGaugeFloat64 returns an existing GaugeFloat64 or constructs and registers a
// new StandardGaugeFloat64.
func GetOrRegisterGaugeFloat64(name string, r Registry) GaugeFloat64 {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, NewGaugeFloat64()).(GaugeFloat64)
}

// NewGaugeFloat64 constructs a new StandardGaugeFloat64.
func NewGaugeFloat64() GaugeFloat64 {
	if !Enabled {
		return NilGaugeFloat64{}
	}
	return &StandardGaugeFloat64{}
}

// NewRegisteredGaugeFloat64 constructs and registers a new StandardGaugeFloat64.
func NewRegisteredGaugeFloat64(name string, r Registry) GaugeFloat64 {
	c := NewGaugeFloat64()
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// gaugeFloat64Snapshot is a read-only copy of another GaugeFloat64.
type gaugeFloat64Snapshot float64

// Value returns the value at the time the snapshot was taken.
func (g gaugeFloat64Snapshot) Value() float64 { return float64(g) }

// NilGauge is a no-op Gauge.
type NilGaugeFloat64 struct{}

func (NilGaugeFloat64) Snapshot() GaugeFloat64Snapshot { return NilGaugeFloat64{} }
func (NilGaugeFloat64) Update(v float64)               {}
func (NilGaugeFloat64) Value() float64                 { return 0.0 }

// StandardGaugeFloat64 is the standard implementation of a GaugeFloat64 and uses
// atomic to manage a single float64 value.
type StandardGaugeFloat64 struct {
	floatBits atomic.Uint64
}

// Snapshot returns a read-only copy of the gauge.
func (g *StandardGaugeFloat64) Snapshot() GaugeFloat64Snapshot {
	v := math.Float64frombits(g.floatBits.Load())
	return gaugeFloat64Snapshot(v)
}

// Update updates the gauge's value.
func (g *StandardGaugeFloat64) Update(v float64) {
	g.floatBits.Store(math.Float64bits(v))
}
