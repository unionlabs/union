package startup

import (
	"context"

	"github.com/pkg/errors"
)

var errClockSet = errors.New("refusing to change clock after it is set")

// ClockSynchronizer provides a synchronization mechanism for services that rely on the genesis time and validator root
// being known before getting to work.
type ClockSynchronizer struct {
	ready chan struct{}
	c     *Clock
}

// ClockWaiter specifies the WaitForClock method. ClockSynchronizer works in a 1:N pattern, with 1 thread calling
// SetClock, and the others blocking on a call to WaitForClock until the expected *Clock value is set.
type ClockWaiter interface {
	WaitForClock(context.Context) (*Clock, error)
}

// ClockSetter specifies the SetClock method. ClockSynchronizer works in a 1:N pattern, so in a given graph of services,
// only one service should be given the ClockSetter, and all others relying on the service's activation should use
// ClockWaiter.
type ClockSetter interface {
	SetClock(c *Clock) error
}

// SetClock sets the Clock value `c` and unblocks all threads waiting for `c` via WaitForClock.
// Calling SetClock more than once will return an error, as calling this function is meant to be a signal
// that the system is ready to start.
func (w *ClockSynchronizer) SetClock(c *Clock) error {
	if w.c != nil {
		return errors.Wrapf(errClockSet, "when SetClock called, Clock already set to time=%d", w.c.GenesisTime().Unix())
	}
	w.c = c
	close(w.ready)
	return nil
}

// WaitForClock will block the caller until the *Clock value is available. If the provided context is canceled (eg via
// a deadline set upstream), the function will return the error given by ctx.Err().
func (w *ClockSynchronizer) WaitForClock(ctx context.Context) (*Clock, error) {
	select {
	case <-w.ready:
		return w.c, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	}
}

// NewClockSynchronizer initializes a single instance of ClockSynchronizer that must be used by all ClockWaiters that
// need to be synchronized to a ClockSetter (ie blockchain service).
func NewClockSynchronizer() *ClockSynchronizer {
	return &ClockSynchronizer{
		ready: make(chan struct{}),
	}
}
