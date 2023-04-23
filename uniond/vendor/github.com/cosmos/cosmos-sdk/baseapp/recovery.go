package baseapp

import (
	"fmt"
	"runtime/debug"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// RecoveryHandler handles recovery() object.
// Return a non-nil error if recoveryObj was processed.
// Return nil if recoveryObj was not processed.
type RecoveryHandler func(recoveryObj interface{}) error

// recoveryMiddleware is wrapper for RecoveryHandler to create chained recovery handling.
// returns (recoveryMiddleware, nil) if recoveryObj was not processed and should be passed to the next middleware in chain.
// returns (nil, error) if recoveryObj was processed and middleware chain processing should be stopped.
type recoveryMiddleware func(recoveryObj interface{}) (recoveryMiddleware, error)

// processRecovery processes recoveryMiddleware chain for recovery() object.
// Chain processing stops on non-nil error or when chain is processed.
func processRecovery(recoveryObj interface{}, middleware recoveryMiddleware) error {
	if middleware == nil {
		return nil
	}

	next, err := middleware(recoveryObj)
	if err != nil {
		return err
	}

	return processRecovery(recoveryObj, next)
}

// newRecoveryMiddleware creates a RecoveryHandler middleware.
func newRecoveryMiddleware(handler RecoveryHandler, next recoveryMiddleware) recoveryMiddleware {
	return func(recoveryObj interface{}) (recoveryMiddleware, error) {
		if err := handler(recoveryObj); err != nil {
			return nil, err
		}

		return next, nil
	}
}

// newOutOfGasRecoveryMiddleware creates a standard OutOfGas recovery middleware for app.runTx method.
func newOutOfGasRecoveryMiddleware(gasWanted uint64, ctx sdk.Context, next recoveryMiddleware) recoveryMiddleware {
	handler := func(recoveryObj interface{}) error {
		err, ok := recoveryObj.(sdk.ErrorOutOfGas)
		if !ok {
			return nil
		}

		return sdkerrors.Wrap(
			sdkerrors.ErrOutOfGas, fmt.Sprintf(
				"out of gas in location: %v; gasWanted: %d, gasUsed: %d",
				err.Descriptor, gasWanted, ctx.GasMeter().GasConsumed(),
			),
		)
	}

	return newRecoveryMiddleware(handler, next)
}

// newDefaultRecoveryMiddleware creates a default (last in chain) recovery middleware for app.runTx method.
func newDefaultRecoveryMiddleware() recoveryMiddleware {
	handler := func(recoveryObj interface{}) error {
		return sdkerrors.Wrap(
			sdkerrors.ErrPanic, fmt.Sprintf(
				"recovered: %v\nstack:\n%v", recoveryObj, string(debug.Stack()),
			),
		)
	}

	return newRecoveryMiddleware(handler, nil)
}
