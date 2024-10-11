// Package branch contains the core branch service interface.
package branch

import (
	"context"
	"errors"
)

// ErrGasLimitExceeded is returned when the gas limit is exceeded in a
// Service.ExecuteWithGasLimit call.
var ErrGasLimitExceeded = errors.New("branch: gas limit exceeded")

// Service is the branch service interface. It can be used to execute
// code paths in an isolated execution context that can be reverted.
// A revert typically means a rollback on events and state changes.
type Service interface {
	// Execute executes the given function in an isolated context. If the
	// `f` function returns an error, the execution is considered failed,
	// and every change made affecting the execution context is rolled back.
	// If the function returns nil, the execution is considered successful, and
	// committed.
	// The context.Context passed to the `f` function is a child of the context
	// passed to the Execute function, and is what should be used with other
	// core services in order to ensure the execution remains isolated.
	Execute(ctx context.Context, f func(ctx context.Context) error) error
	// ExecuteWithGasLimit executes the given function `f` in an isolated context,
	// with the provided gas limit, this is advanced usage and is used to disallow
	// an execution path to consume an indefinite amount of gas.
	// If the execution fails or succeeds the gas limit is still applied to the
	// parent context, the function returns a gasUsed value which is the amount
	// of gas used by the execution path. If the execution path exceeds the gas
	// ErrGasLimitExceeded is returned.
	ExecuteWithGasLimit(ctx context.Context, gasLimit uint64, f func(ctx context.Context) error) (gasUsed uint64, err error)
}
