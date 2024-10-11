package metrics

import (
	"sync"
	"time"
)

// Initial slice capacity for the values stored in a ResettingTimer
const InitialResettingTimerSliceCap = 10

type ResettingTimerSnapshot interface {
	Count() int
	Mean() float64
	Max() int64
	Min() int64
	Percentiles([]float64) []float64
}

// ResettingTimer is used for storing aggregated values for timers, which are reset on every flush interval.
type ResettingTimer interface {
	Snapshot() ResettingTimerSnapshot
	Time(func())
	Update(time.Duration)
	UpdateSince(time.Time)
}

// GetOrRegisterResettingTimer returns an existing ResettingTimer or constructs and registers a
// new StandardResettingTimer.
func GetOrRegisterResettingTimer(name string, r Registry) ResettingTimer {
	if nil == r {
		r = DefaultRegistry
	}
	return r.GetOrRegister(name, NewResettingTimer).(ResettingTimer)
}

// NewRegisteredResettingTimer constructs and registers a new StandardResettingTimer.
func NewRegisteredResettingTimer(name string, r Registry) ResettingTimer {
	c := NewResettingTimer()
	if nil == r {
		r = DefaultRegistry
	}
	r.Register(name, c)
	return c
}

// NewResettingTimer constructs a new StandardResettingTimer
func NewResettingTimer() ResettingTimer {
	if !Enabled {
		return NilResettingTimer{}
	}
	return &StandardResettingTimer{
		values: make([]int64, 0, InitialResettingTimerSliceCap),
	}
}

// NilResettingTimer is a no-op ResettingTimer.
type NilResettingTimer struct{}

func (NilResettingTimer) Values() []int64                    { return nil }
func (n NilResettingTimer) Snapshot() ResettingTimerSnapshot { return n }
func (NilResettingTimer) Time(f func())                      { f() }
func (NilResettingTimer) Update(time.Duration)               {}
func (NilResettingTimer) Percentiles([]float64) []float64    { return nil }
func (NilResettingTimer) Mean() float64                      { return 0.0 }
func (NilResettingTimer) Max() int64                         { return 0 }
func (NilResettingTimer) Min() int64                         { return 0 }
func (NilResettingTimer) UpdateSince(time.Time)              {}
func (NilResettingTimer) Count() int                         { return 0 }

// StandardResettingTimer is the standard implementation of a ResettingTimer.
// and Meter.
type StandardResettingTimer struct {
	values []int64
	sum    int64 // sum is a running count of the total sum, used later to calculate mean

	mutex sync.Mutex
}

// Snapshot resets the timer and returns a read-only copy of its contents.
func (t *StandardResettingTimer) Snapshot() ResettingTimerSnapshot {
	t.mutex.Lock()
	defer t.mutex.Unlock()
	snapshot := &resettingTimerSnapshot{}
	if len(t.values) > 0 {
		snapshot.mean = float64(t.sum) / float64(len(t.values))
		snapshot.values = t.values
		t.values = make([]int64, 0, InitialResettingTimerSliceCap)
	}
	t.sum = 0
	return snapshot
}

// Record the duration of the execution of the given function.
func (t *StandardResettingTimer) Time(f func()) {
	ts := time.Now()
	f()
	t.Update(time.Since(ts))
}

// Record the duration of an event.
func (t *StandardResettingTimer) Update(d time.Duration) {
	t.mutex.Lock()
	defer t.mutex.Unlock()
	t.values = append(t.values, int64(d))
	t.sum += int64(d)
}

// Record the duration of an event that started at a time and ends now.
func (t *StandardResettingTimer) UpdateSince(ts time.Time) {
	t.Update(time.Since(ts))
}

// resettingTimerSnapshot is a point-in-time copy of another ResettingTimer.
type resettingTimerSnapshot struct {
	values              []int64
	mean                float64
	max                 int64
	min                 int64
	thresholdBoundaries []float64
	calculated          bool
}

// Count return the length of the values from snapshot.
func (t *resettingTimerSnapshot) Count() int {
	return len(t.values)
}

// Percentiles returns the boundaries for the input percentiles.
// note: this method is not thread safe
func (t *resettingTimerSnapshot) Percentiles(percentiles []float64) []float64 {
	t.calc(percentiles)
	return t.thresholdBoundaries
}

// Mean returns the mean of the snapshotted values
// note: this method is not thread safe
func (t *resettingTimerSnapshot) Mean() float64 {
	if !t.calculated {
		t.calc(nil)
	}

	return t.mean
}

// Max returns the max of the snapshotted values
// note: this method is not thread safe
func (t *resettingTimerSnapshot) Max() int64 {
	if !t.calculated {
		t.calc(nil)
	}
	return t.max
}

// Min returns the min of the snapshotted values
// note: this method is not thread safe
func (t *resettingTimerSnapshot) Min() int64 {
	if !t.calculated {
		t.calc(nil)
	}
	return t.min
}

func (t *resettingTimerSnapshot) calc(percentiles []float64) {
	scores := CalculatePercentiles(t.values, percentiles)
	t.thresholdBoundaries = scores
	if len(t.values) == 0 {
		return
	}
	t.min = t.values[0]
	t.max = t.values[len(t.values)-1]
}
