package mint

import (
	"context"

	epochstypes "cosmossdk.io/x/epochs/types"
)

var _ epochstypes.EpochHooks = AppModule{}

// BeforeEpochStart calls the mint function.
func (am AppModule) BeforeEpochStart(ctx context.Context, epochIdentifier string, epochNumber int64) error {
	minter, err := am.keeper.Minter.Get(ctx)
	if err != nil {
		return err
	}

	oldMinter := minter

	err = am.mintFn(ctx, am.keeper.Environment, &minter, epochIdentifier, epochNumber)
	if err != nil {
		return err
	}

	if minter.IsEqual(oldMinter) {
		return nil
	}

	return am.keeper.Minter.Set(ctx, minter)
}

// AfterEpochEnd is a noop
func (am AppModule) AfterEpochEnd(_ context.Context, _ string, _ int64) error {
	return nil
}
