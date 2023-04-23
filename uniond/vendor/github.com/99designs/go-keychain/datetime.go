// +build darwin ios

package keychain

/*
#cgo LDFLAGS: -framework CoreFoundation

#include <CoreFoundation/CoreFoundation.h>
*/
import "C"
import (
	"math"
	"time"
)

const nsPerSec = 1000 * 1000 * 1000

// absoluteTimeIntervalSince1970() returns the number of seconds from
// the Unix epoch (1970-01-01T00:00:00+00:00) to the Core Foundation
// absolute reference date (2001-01-01T00:00:00+00:00). It should be
// exactly 978307200.
func absoluteTimeIntervalSince1970() int64 {
	return int64(C.kCFAbsoluteTimeIntervalSince1970)
}

func unixToAbsoluteTime(s int64, ns int64) C.CFAbsoluteTime {
	// Subtract as int64s first before converting to floating
	// point to minimize precision loss (assuming the given time
	// isn't much earlier than the Core Foundation absolute
	// reference date).
	abs := s - absoluteTimeIntervalSince1970()
	return C.CFAbsoluteTime(abs) + C.CFTimeInterval(ns)/nsPerSec
}

func absoluteTimeToUnix(abs C.CFAbsoluteTime) (int64, int64) {
	int, frac := math.Modf(float64(abs))
	return int64(int) + absoluteTimeIntervalSince1970(), int64(frac * nsPerSec)
}

// TimeToCFDate will convert the given time.Time to a CFDateRef, which
// must be released with Release(ref).
func TimeToCFDate(t time.Time) C.CFDateRef {
	s := t.Unix()
	ns := int64(t.Nanosecond())
	abs := unixToAbsoluteTime(s, ns)
	return C.CFDateCreate(C.kCFAllocatorDefault, abs)
}

// CFDateToTime will convert the given CFDateRef to a time.Time.
func CFDateToTime(d C.CFDateRef) time.Time {
	abs := C.CFDateGetAbsoluteTime(d)
	s, ns := absoluteTimeToUnix(abs)
	return time.Unix(s, ns)
}

// Wrappers around C functions for testing.

func cfDateToAbsoluteTime(d C.CFDateRef) C.CFAbsoluteTime {
	return C.CFDateGetAbsoluteTime(d)
}

func absoluteTimeToCFDate(abs C.CFAbsoluteTime) C.CFDateRef {
	return C.CFDateCreate(C.kCFAllocatorDefault, abs)
}

func releaseCFDate(d C.CFDateRef) {
	Release(C.CFTypeRef(d))
}
