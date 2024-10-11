/*
Package cleanup handles migration to and support for the Go 1.14+
testing.TB.Cleanup() function.
*/
package cleanup

import (
	"os"
	"strings"
)

type cleanupT interface {
	Cleanup(f func())
}

// implemented by gotest.tools/x/subtest.TestContext
type addCleanupT interface {
	AddCleanup(f func())
}

type logT interface {
	Log(...interface{})
}

type helperT interface {
	Helper()
}

var noCleanup = strings.ToLower(os.Getenv("TEST_NOCLEANUP")) == "true"

// Cleanup registers f as a cleanup function on t if any mechanisms are available.
//
// Skips registering f if TEST_NOCLEANUP is set to true.
func Cleanup(t logT, f func()) {
	if ht, ok := t.(helperT); ok {
		ht.Helper()
	}
	if noCleanup {
		t.Log("skipping cleanup because TEST_NOCLEANUP was enabled.")
		return
	}
	if ct, ok := t.(cleanupT); ok {
		ct.Cleanup(f)
		return
	}
	if tc, ok := t.(addCleanupT); ok {
		tc.AddCleanup(f)
	}
}
