package types

import (
	sdk "github.com/cosmos/cosmos-sdk/types"

	paramtypes "cosmossdk.io/x/params/types"
)

// ParamSubspace defines the expected Subspace interface for module parameters.
type ParamSubspace interface {
	GetParamSet(ctx sdk.Context, ps paramtypes.ParamSet)
}
