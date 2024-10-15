package runtime

import (
	"context"
	"fmt"
	"os"

	"cosmossdk.io/core/branch"
	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var _ branch.Service = BranchService{}

type BranchService struct{}

func (b BranchService) Execute(ctx context.Context, f func(ctx context.Context) error) error {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	branchedCtx, commit := sdkCtx.CacheContext()
	err := f(branchedCtx)
	if err != nil {
		return err
	}
	commit()
	return nil
}

func (b BranchService) ExecuteWithGasLimit(ctx context.Context, gasLimit uint64, f func(ctx context.Context) error) (gasUsed uint64, err error) {
	sdkCtx := sdk.UnwrapSDKContext(ctx)
	branchedCtx, commit := sdkCtx.CacheContext()
	// create a new gas meter
	limitedGasMeter := storetypes.NewGasMeter(gasLimit)
	// apply gas meter with limit to branched context
	branchedCtx = branchedCtx.WithGasMeter(limitedGasMeter)
	err = catchOutOfGas(branchedCtx, f)
	// even before checking the error, we want to get the gas used
	// and apply it to the original context.
	gasUsed = limitedGasMeter.GasConsumed()
	sdkCtx.GasMeter().ConsumeGas(gasUsed, "branch")
	// in case of errors, do not commit the branched context
	// return gas used and the error
	if err != nil {
		return gasUsed, err
	}
	// if no error, commit the branched context
	// and return gas used and no error
	commit()
	return gasUsed, nil
}

// catchOutOfGas is a helper function to catch out of gas panics and return them as errors.
func catchOutOfGas(ctx sdk.Context, f func(ctx context.Context) error) (err error) {
	defer func() {
		if r := recover(); r != nil {
			// we immediately check if it's an out of error gas.
			// if it is not we panic again to propagate it up.
			if _, ok := r.(storetypes.ErrorOutOfGas); !ok {
				_, _ = fmt.Fprintf(os.Stderr, "recovered: %#v", r) // log to stderr
				panic(r)
			}
			err = sdkerrors.ErrOutOfGas
		}
	}()
	return f(ctx)
}
