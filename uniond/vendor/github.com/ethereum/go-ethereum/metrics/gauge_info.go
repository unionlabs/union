package metrics

import (
	"encoding/json"
	"sync"
)

type GaugeInfoSnapshot interface {
	Value() GaugeInfoValue
}

// GaugeInfos hold a GaugeInfoValue value that can be set arbitrarily.
type GaugeInfo interface {
	Update(GaugeInfoValue)
	Snapshot() GaugeInfoSnapshot
}

// GaugeInfoValue is a mapping of keys to values
type GaugeInfoValue map[string]string

func (val GaugeInfoValue) String() string {
	data, _ := json.Marshal(val)
	return string(data)
}

// GetOrRegisterGaugeInfo returns an existing GaugeInfo or constructs and registers a
// new StandardGaugeInfo.
func GetOrRegisterGaugeInfo(name string, r Registry) GaugeInfo {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, NewGaugeInfo()).(GaugeInfo)
}

// NewGaugeInfo constructs a new StandardGaugeInfo.
func NewGaugeInfo() GaugeInfo {
	if !Enabled {
		return NilGaugeInfo{}
	}
	return &StandardGaugeInfo{
		value: GaugeInfoValue{},
	}
}

// NewRegisteredGaugeInfo constructs and registers a new StandardGaugeInfo.
func NewRegisteredGaugeInfo(name string, r Registry) GaugeInfo {
	c := NewGaugeInfo()
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// gaugeInfoSnapshot is a read-only copy of another GaugeInfo.
type gaugeInfoSnapshot GaugeInfoValue

// Value returns the value at the time the snapshot was taken.
func (g gaugeInfoSnapshot) Value() GaugeInfoValue { return GaugeInfoValue(g) }

type NilGaugeInfo struct{}

func (NilGaugeInfo) Snapshot() GaugeInfoSnapshot { return NilGaugeInfo{} }
func (NilGaugeInfo) Update(v GaugeInfoValue)     {}
func (NilGaugeInfo) Value() GaugeInfoValue       { return GaugeInfoValue{} }

// StandardGaugeInfo is the standard implementation of a GaugeInfo and uses
// sync.Mutex to manage a single string value.
type StandardGaugeInfo struct {
	mutex sync.Mutex
	value GaugeInfoValue
}

// Snapshot returns a read-only copy of the gauge.
func (g *StandardGaugeInfo) Snapshot() GaugeInfoSnapshot {
	return gaugeInfoSnapshot(g.value)
}

// Update updates the gauge's value.
func (g *StandardGaugeInfo) Update(v GaugeInfoValue) {
	g.mutex.Lock()
	defer g.mutex.Unlock()
	g.value = v
}
