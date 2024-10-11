package ante

import (
	"fmt"

	"cosmossdk.io/core/appmodule"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// GasTx defines a Tx with a GetGas() method which is needed to use SetUpContextDecorator
type GasTx interface {
	sdk.Tx
	GetGas() uint64
}

// SetUpContextDecorator sets the GasMeter in the Context and wraps the next AnteHandler with a defer clause
// to recover from any downstream OutOfGas panics in the AnteHandler chain to return an error with information
// on gas provided and gas used.
// CONTRACT: Must be first decorator in the chain
// CONTRACT: Tx must implement GasTx interface
type SetUpContextDecorator struct {
	env             appmodule.Environment
	consensusKeeper ConsensusKeeper
}

func NewSetUpContextDecorator(env appmodule.Environment, consensusKeeper ConsensusKeeper) SetUpContextDecorator {
	return SetUpContextDecorator{
		env:             env,
		consensusKeeper: consensusKeeper,
	}
}

func (sud SetUpContextDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, _ bool, next sdk.AnteHandler) (newCtx sdk.Context, err error) {
	// all transactions must implement GasTx
	gasTx, ok := tx.(GasTx)
	if !ok {
		// Set a gas meter with limit 0 as to prevent an infinite gas meter attack
		// during runTx.
		newCtx = SetGasMeter(ctx, 0)
		return newCtx, errorsmod.Wrap(sdkerrors.ErrTxDecode, "Tx must be GasTx")
	}

	newCtx = SetGasMeter(ctx, gasTx.GetGas())

	maxGas, _, err := sud.consensusKeeper.BlockParams(ctx)
	if err != nil {
		return newCtx, err
	}

	if maxGas > 0 && gasTx.GetGas() > maxGas {
		return newCtx, errorsmod.Wrapf(sdkerrors.ErrInvalidGasLimit, "tx gas limit %d exceeds block max gas %d", gasTx.GetGas(), maxGas)
	}

	// Decorator will catch an OutOfGasPanic caused in the next antehandler
	// AnteHandlers must have their own defer/recover in order for the BaseApp
	// to know how much gas was used! This is because the GasMeter is created in
	// the AnteHandler, but if it panics the context won't be set properly in
	// runTx's recover call.
	defer func() {
		if r := recover(); r != nil {
			switch rType := r.(type) {
			case storetypes.ErrorOutOfGas:
				log := fmt.Sprintf(
					"out of gas in location: %v; gasWanted: %d, gasUsed: %d",
					rType.Descriptor, gasTx.GetGas(), newCtx.GasMeter().GasConsumed())

				err = errorsmod.Wrap(sdkerrors.ErrOutOfGas, log)
			default:
				panic(r)
			}
		}
	}()

	return next(newCtx, tx, false)
}

// SetGasMeter returns a new context with a gas meter set from a given context.
func SetGasMeter(ctx sdk.Context, gasLimit uint64) sdk.Context {
	// In various cases such as simulation and during the genesis block, we do not
	// meter any gas utilization.
	if ctx.ExecMode() == sdk.ExecModeSimulate || ctx.BlockHeight() == 0 { // NOTE: using environment here breaks the API of SetGasMeter, an alternative must be found for server/v2. ref: https://github.com/cosmos/cosmos-sdk/issues/19640
		return ctx.WithGasMeter(storetypes.NewInfiniteGasMeter())
	}

	return ctx.WithGasMeter(storetypes.NewGasMeter(gasLimit))
}
