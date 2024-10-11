package metrics

import (
	"sync/atomic"
)

type CounterSnapshot interface {
	Count() int64
}

// Counters hold an int64 value that can be incremented and decremented.
type Counter interface {
	Clear()
	Dec(int64)
	Inc(int64)
	Snapshot() CounterSnapshot
}

// GetOrRegisterCounter returns an existing Counter or constructs and registers
// a new StandardCounter.
func GetOrRegisterCounter(name string, r Registry) Counter {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, NewCounter).(Counter)
}

// GetOrRegisterCounterForced returns an existing Counter or constructs and registers a
// new Counter no matter the global switch is enabled or not.
// Be sure to unregister the counter from the registry once it is of no use to
// allow for garbage collection.
func GetOrRegisterCounterForced(name string, r Registry) Counter {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, NewCounterForced).(Counter)
}

// NewCounter constructs a new StandardCounter.
func NewCounter() Counter {
	if !Enabled {
		return NilCounter{}
	}
	return new(StandardCounter)
}

// NewCounterForced constructs a new StandardCounter and returns it no matter if
// the global switch is enabled or not.
func NewCounterForced() Counter {
	return new(StandardCounter)
}

// NewRegisteredCounter constructs and registers a new StandardCounter.
func NewRegisteredCounter(name string, r Registry) Counter {
	c := NewCounter()
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// NewRegisteredCounterForced constructs and registers a new StandardCounter
// and launches a goroutine no matter the global switch is enabled or not.
// Be sure to unregister the counter from the registry once it is of no use to
// allow for garbage collection.
func NewRegisteredCounterForced(name string, r Registry) Counter {
	c := NewCounterForced()
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// counterSnapshot is a read-only copy of another Counter.
type counterSnapshot int64

// Count returns the count at the time the snapshot was taken.
func (c counterSnapshot) Count() int64 { return int64(c) }

// NilCounter is a no-op Counter.
type NilCounter struct{}

func (NilCounter) Clear()                    {}
func (NilCounter) Dec(i int64)               {}
func (NilCounter) Inc(i int64)               {}
func (NilCounter) Snapshot() CounterSnapshot { return (*emptySnapshot)(nil) }

// StandardCounter is the standard implementation of a Counter and uses the
// sync/atomic package to manage a single int64 value.
type StandardCounter atomic.Int64

// Clear sets the counter to zero.
func (c *StandardCounter) Clear() {
	(*atomic.Int64)(c).Store(0)
}

// Dec decrements the counter by the given amount.
func (c *StandardCounter) Dec(i int64) {
	(*atomic.Int64)(c).Add(-i)
}

// Inc increments the counter by the given amount.
func (c *StandardCounter) Inc(i int64) {
	(*atomic.Int64)(c).Add(i)
}

// Snapshot returns a read-only copy of the counter.
func (c *StandardCounter) Snapshot() CounterSnapshot {
	return counterSnapshot((*atomic.Int64)(c).Load())
}
