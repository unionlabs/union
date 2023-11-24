package exported

import (
	sdk "github.com/cosmos/cosmos-sdk/types"
	paramtypes "github.com/cosmos/cosmos-sdk/x/params/types"
)

// GenesisBalance defines a genesis balance interface that allows for account
// address and balance retrieval.
type GenesisBalance interface {
	GetAddress() string
	GetCoins() sdk.Coins
}

type (
	ParamSet = paramtypes.ParamSet

	// Subspace defines an interface that implements the legacy x/params Subspace
	// type.
	//
	// NOTE: This is used solely for migration of x/params managed parameters.
	Subspace interface {
		GetParamSet(ctx sdk.Context, ps ParamSet)
		Get(ctx sdk.Context, key []byte, ptr any)
	}
)
