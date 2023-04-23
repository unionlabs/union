package baseapp

import (
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// ParamStore defines the interface the parameter store used by the BaseApp must
// fulfill.
type ParamStore interface {
	Get(ctx sdk.Context) (*tmproto.ConsensusParams, error)
	Has(ctx sdk.Context) bool
	Set(ctx sdk.Context, cp *tmproto.ConsensusParams)
}
