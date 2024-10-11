/*
Package subtest provides a TestContext to subtests which handles cleanup, and
provides a testing.TB, and context.Context.

This package was inspired by github.com/frankban/quicktest.

# DEPRECATED

With the addition of T.Cleanup() in go1.14 this package provides very
little value. A context.Context can be managed by tests that need it with
little enough boilerplate that it doesn't make sense to wrap testing.T in a
TestContext.
*/
package subtest // import "gotest.tools/v3/x/subtest"

import (
	"context"
	"testing"

	"gotest.tools/v3/internal/cleanup"
)

type testcase struct {
	testing.TB
	ctx          context.Context
	cleanupFuncs []cleanupFunc
}

type cleanupFunc func()

func (tc *testcase) Ctx() context.Context {
	if tc.ctx == nil {
		var cancel func()
		tc.ctx, cancel = context.WithCancel(context.Background())
		cleanup.Cleanup(tc, cancel)
	}
	return tc.ctx
}

// cleanup runs all cleanup functions. Functions are run in the opposite order
// in which they were added. Cleanup is called automatically before Run exits.
func (tc *testcase) cleanup() {
	for _, f := range tc.cleanupFuncs {
		// Defer all cleanup functions so they all run even if one calls
		// t.FailNow() or panics. Deferring them also runs them in reverse order.
		defer f()
	}
	tc.cleanupFuncs = nil
}

func (tc *testcase) AddCleanup(f func()) {
	tc.cleanupFuncs = append(tc.cleanupFuncs, f)
}

func (tc *testcase) Parallel() {
	tp, ok := tc.TB.(parallel)
	if !ok {
		panic("Parallel called with a testing.B")
	}
	tp.Parallel()
}

type parallel interface {
	Parallel()
}

// Run a subtest. When subtest exits, every cleanup function added with
// TestContext.AddCleanup will be run.
func Run(t *testing.T, name string, subtest func(t TestContext)) bool {
	return t.Run(name, func(t *testing.T) {
		tc := &testcase{TB: t}
		defer tc.cleanup()
		subtest(tc)
	})
}

// TestContext provides a testing.TB and a context.Context for a test case.
type TestContext interface {
	testing.TB
	// AddCleanup function which will be run when before Run returns.
	//
	// Deprecated: Go 1.14+ now includes a testing.TB.Cleanup(func()) which
	// should be used instead. AddCleanup will be removed in a future release.
	AddCleanup(f func())
	// Ctx returns a context for the test case. Multiple calls from the same subtest
	// will return the same context. The context is cancelled when Run
	// returns.
	Ctx() context.Context
	// Parallel calls t.Parallel on the testing.TB. Panics if testing.TB does
	// not implement Parallel.
	Parallel()
}

var _ TestContext = &testcase{}
