package transaction

import "context"

// ExecMode defines the execution mode
type ExecMode uint8

// All possible execution modes.
// For backwards compatibility and easier casting, the exec mode values must be the same as in cosmos/cosmos-sdk/types package.
const (
	ExecModeCheck ExecMode = iota
	ExecModeReCheck
	ExecModeSimulate
	_
	_
	_
	_
	ExecModeFinalize
)

// Service creates a transaction service.
type Service interface {
	// ExecMode returns the current execution mode.
	ExecMode(ctx context.Context) ExecMode
}
