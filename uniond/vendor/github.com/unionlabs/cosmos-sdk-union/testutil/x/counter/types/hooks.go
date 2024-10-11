package types

import (
	"context"
	"errors"
)

var _ CounterHooks = MultiCounterHooks{}

// MultiCounterHooks is a slice of hooks to be called in sequence.
type MultiCounterHooks []CounterHooks

// NewMultiCounterHooks returns a MultiCounterHooks from a list of CounterHooks
func NewMultiCounterHooks(hooks ...CounterHooks) MultiCounterHooks {
	return hooks
}

// AfterIncreaseCount calls AfterIncreaseCount on all hooks and collects the errors if any.
func (ch MultiCounterHooks) AfterIncreaseCount(ctx context.Context, newCount int64) error {
	var errs error
	for i := range ch {
		errs = errors.Join(errs, ch[i].AfterIncreaseCount(ctx, newCount))
	}

	return errs
}
