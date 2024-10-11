// Package ratemap implements a goroutine-safe map of string to float64. It can
// be embedded in implementations whose metrics support fixed sample rates, so
// that an additional parameter doesn't have to be tracked through the e.g.
// lv.Space object.
package ratemap

import "sync"

// RateMap is a simple goroutine-safe map of string to float64.
type RateMap struct {
	mtx sync.RWMutex
	m   map[string]float64
}

// New returns a new RateMap.
func New() *RateMap {
	return &RateMap{
		m: map[string]float64{},
	}
}

// Set writes the given name/rate pair to the map.
// Set is safe for concurrent access by multiple goroutines.
func (m *RateMap) Set(name string, rate float64) {
	m.mtx.Lock()
	defer m.mtx.Unlock()
	m.m[name] = rate
}

// Get retrieves the rate for the given name, or 1.0 if none is set.
// Get is safe for concurrent access by multiple goroutines.
func (m *RateMap) Get(name string) float64 {
	m.mtx.RLock()
	defer m.mtx.RUnlock()
	f, ok := m.m[name]
	if !ok {
		f = 1.0
	}
	return f
}
