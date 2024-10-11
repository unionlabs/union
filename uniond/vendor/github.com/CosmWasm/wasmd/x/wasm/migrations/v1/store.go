package v1

import (
	"context"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

// AddToSecondIndexFn creates a secondary index entry for the creator fo the contract
type AddToSecondIndexFn func(ctx context.Context, creatorAddress sdk.AccAddress, position *types.AbsoluteTxPosition, contractAddress sdk.AccAddress) error

// Keeper abstract keeper
type wasmKeeper interface {
	IterateContractInfo(ctx context.Context, cb func(sdk.AccAddress, types.ContractInfo) bool)
}

// Migrator is a struct for handling in-place store migrations.
type Migrator struct {
	keeper             wasmKeeper
	addToSecondIndexFn AddToSecondIndexFn
}

// NewMigrator returns a new Migrator.
func NewMigrator(k wasmKeeper, fn AddToSecondIndexFn) Migrator {
	return Migrator{keeper: k, addToSecondIndexFn: fn}
}

// Migrate1to2 migrates from version 1 to 2.
func (m Migrator) Migrate1to2(ctx sdk.Context) error {
	m.keeper.IterateContractInfo(ctx, func(contractAddr sdk.AccAddress, contractInfo types.ContractInfo) bool {
		creator := sdk.MustAccAddressFromBech32(contractInfo.Creator)
		err := m.addToSecondIndexFn(ctx, creator, contractInfo.Created, contractAddr)
		if err != nil {
			panic(err)
		}
		return false
	})
	return nil
}
