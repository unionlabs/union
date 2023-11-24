package baseapp

import (
	"context"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
)

// ParamStore defines the interface the parameter store used by the BaseApp must
// fulfill.
type ParamStore interface {
	Get(ctx context.Context) (cmtproto.ConsensusParams, error)
	Has(ctx context.Context) (bool, error)
	Set(ctx context.Context, cp cmtproto.ConsensusParams) error
}
