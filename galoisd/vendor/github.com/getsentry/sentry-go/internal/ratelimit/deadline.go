package ratelimit

import "time"

// A Deadline is a time instant when a rate limit expires.
type Deadline time.Time

// After reports whether the deadline d is after other.
func (d Deadline) After(other Deadline) bool {
	return time.Time(d).After(time.Time(other))
}

// Equal reports whether d and e represent the same deadline.
func (d Deadline) Equal(e Deadline) bool {
	return time.Time(d).Equal(time.Time(e))
}

// String returns the deadline formatted for debugging.
func (d Deadline) String() string {
	// Like time.Time.String, but without the monotonic clock reading.
	return time.Time(d).Round(0).String()
}
