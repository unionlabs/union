//go:build !clock_skew
// +build !clock_skew

package time

import (
	"time"
)

// Now returns the current time in UTC with no monotonic component.
func Now() time.Time {
	return Canonical(time.Now())
}
