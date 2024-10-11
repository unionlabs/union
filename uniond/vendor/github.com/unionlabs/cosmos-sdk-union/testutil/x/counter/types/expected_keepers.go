package types

import "context"

type CounterHooks interface {
	AfterIncreaseCount(ctx context.Context, newCount int64) error
}

type CounterHooksWrapper struct{ CounterHooks }

// IsOnePerModuleType implements the depinject.OnePerModuleType interface.
func (CounterHooksWrapper) IsOnePerModuleType() {}
