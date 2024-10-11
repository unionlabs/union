// +build darwin,!ios

package keychain

import (
	"testing"
	"time"
)

// Test time is 2018-09-13T06:08:49+00:00
const (
	// Number of seconds between the test time and the Unix epoch
	// (1970-01-01T00:00:00+00:00).
	testTimeUnixSeconds = 1536818929
	// Number of seconds between the test time and
	// Core Foundation's absolute reference date
	// (2001-01-01T00:00:00+00:00). See
	// https://developer.apple.com/documentation/corefoundation/cfabsolutetime?language=objc
	testTimeAbsoluteTimeSeconds = 558511729
)

func TestUnixToAbsoluteTime(t *testing.T) {
	var testNano int64 = 123456789
	abs := unixToAbsoluteTime(testTimeUnixSeconds, testNano)
	const expectedAbs = testTimeAbsoluteTimeSeconds + 0.123456789
	if abs != expectedAbs {
		t.Fatalf("expected %f, got %f", expectedAbs, abs)
	}
}

func TestAbsoluteTimeToUnix(t *testing.T) {
	const abs = testTimeAbsoluteTimeSeconds + 0.123456789
	s, ns := absoluteTimeToUnix(abs)
	if s != testTimeUnixSeconds {
		t.Fatalf("expected %d, got %d", testTimeUnixSeconds, s)
	}
	// Some precision loss from floating point.
	const expectedNano = 123456835
	if ns != expectedNano {
		t.Fatalf("expected %d, got %d", expectedNano, ns)
	}
}

func TestTimeToCFDate(t *testing.T) {
	var testNano int64 = 123456789
	tm := time.Unix(testTimeUnixSeconds, testNano)
	d := TimeToCFDate(tm)
	defer releaseCFDate(d)

	abs := cfDateToAbsoluteTime(d)
	const expectedAbs = testTimeAbsoluteTimeSeconds + 0.123456789
	if abs != expectedAbs {
		t.Fatalf("expected %f, got %f", expectedAbs, abs)
	}
}

func TestCFDateToTime(t *testing.T) {
	const abs = testTimeAbsoluteTimeSeconds + 0.123456789
	d := absoluteTimeToCFDate(abs)
	defer releaseCFDate(d)

	tm := CFDateToTime(d)
	// Some precision loss from floating point.
	const expectedNano = testTimeUnixSeconds*nsPerSec + 123456835
	nano := tm.UnixNano()
	if nano != expectedNano {
		t.Fatalf("expected %d, got %d", expectedNano, nano)
	}
}
