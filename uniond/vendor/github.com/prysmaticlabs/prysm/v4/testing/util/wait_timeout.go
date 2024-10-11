package util

import (
	"context"
	"sync"
	"testing"
	"time"
)

// WaitTimeout will wait for a WaitGroup to resolve within a timeout interval.
// Returns true if the waitgroup exceeded the timeout.
func WaitTimeout(wg *sync.WaitGroup, timeout time.Duration) bool {
	ch := make(chan struct{})
	go func() {
		defer close(ch)
		wg.Wait()
	}()
	select {
	case <-ch:
		return false
	case <-time.After(timeout):
		return true
	}
}

// Waiter offers an alternate ux for building tests that want to ensure contexts are used in certain ways.
type Waiter struct {
	c chan struct{}
}

// NewWaiter internally create the chan that Waiter relies on.
func NewWaiter() *Waiter {
	return &Waiter{
		c: make(chan struct{}),
	}
}

// Done is used with RequireDoneAfter and RequireDoneBefore to make assertions
// that certain test code is reached before a timeout or context cancellation.
func (w *Waiter) Done() {
	close(w.c)
}

// RequireDoneAfter forces the test to fail if the timeout is reached before Done is called.
func (w *Waiter) RequireDoneAfter(t *testing.T, timeout time.Duration) {
	select {
	case <-w.c:
		return
	case <-time.After(timeout):
		t.Fatalf("timeout after %s", timeout)
	}
}

// RequireDoneBeforeCancel forces the test to fail if the context is cancelled before Done is called.
func (w *Waiter) RequireDoneBeforeCancel(ctx context.Context, t *testing.T) {
	select {
	case <-w.c:
		return
	case <-ctx.Done():
		t.Fatalf("context canceled before Done with error=%s", ctx.Err())
	}
}
