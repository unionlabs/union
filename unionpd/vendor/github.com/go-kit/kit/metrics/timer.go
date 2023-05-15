package metrics

import "time"

// Timer acts as a stopwatch, sending observations to a wrapped histogram.
// It's a bit of helpful syntax sugar for h.Observe(time.Since(x)).
type Timer struct {
	h Histogram
	t time.Time
	u time.Duration
}

// NewTimer wraps the given histogram and records the current time.
func NewTimer(h Histogram) *Timer {
	return &Timer{
		h: h,
		t: time.Now(),
		u: time.Second,
	}
}

// ObserveDuration captures the number of seconds since the timer was
// constructed, and forwards that observation to the histogram.
func (t *Timer) ObserveDuration() {
	d := float64(time.Since(t.t).Nanoseconds()) / float64(t.u)
	if d < 0 {
		d = 0
	}
	t.h.Observe(d)
}

// Unit sets the unit of the float64 emitted by the timer.
// By default, the timer emits seconds.
func (t *Timer) Unit(u time.Duration) {
	t.u = u
}
