package test

import (
	"flag"
	"testing"
	"time"
)

// Custom test command line flags.
var (
	long   = flag.Bool("long", false, "enable long running tests")
	stress = flag.Bool("stress", false, "enable stress tests (implies -long)")
)

// timeallowed returns how long a single test is allowed to take.
func timeallowed() time.Duration {
	switch {
	case testing.Short():
		return time.Second / 10
	case *long:
		return 30 * time.Second
	case *stress:
		return 2 * time.Minute
	default:
		return time.Second
	}
}

// Long reports whether long tests are enabled.
func Long() bool {
	return *long || *stress
}

// Stress reports whether stress tests are enabled.
func Stress() bool {
	return *stress
}

// RequireLong marks this test as a long test. Test will be skipped if long
// tests are not enabled.
func RequireLong(t *testing.T) {
	if !Long() {
		t.Skipf("long test: use -long or -stress to enable")
	}
}

// RequireStress marks this test as a stress test. Test will be skipped if stress
// tests are not enabled.
func RequireStress(t *testing.T) {
	if !Stress() {
		t.Skipf("stress test: use -stress to enable")
	}
}

// Repeat the given trial function. The duration is controlled by custom
// command-line flags. The trial function returns whether it wants to continue
// testing.
//
//	-short		run for less time than usual
//	-long		allow more time
//	-stress		run for an extremely long time
func Repeat(t *testing.T, trial func(t *testing.T) bool) {
	start := time.Now()
	d := timeallowed()
	n := 1
	for time.Since(start) < d && trial(t) {
		n++
	}
	t.Logf("%d trials in %s", n, time.Since(start))
}

// Trials returns a function that repeats f.
func Trials(f func(t *testing.T) bool) func(t *testing.T) {
	return func(t *testing.T) {
		Repeat(t, f)
	}
}
