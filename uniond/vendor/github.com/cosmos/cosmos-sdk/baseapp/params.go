package baseapp

import (
	"context"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
)

const InitialAppVersion uint64 = 0

// ParamStore defines the interface the parameter store used by the BaseApp must
// fulfill.
type ParamStore interface {
	Get(ctx context.Context) (cmtproto.ConsensusParams, error)
	Has(ctx context.Context) (bool, error)
	Set(ctx context.Context, cp cmtproto.ConsensusParams) error
}
