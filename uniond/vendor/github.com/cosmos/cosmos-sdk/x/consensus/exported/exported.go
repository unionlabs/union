package exported

import (
	"context"

	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type (
	// ParamStore defines an interface that implements the legacy x/params Subspace
	// type.
	//
	// NOTE: This is used solely for migration of x/params managed parameters.
	ParamStore interface {
		Get(ctx sdk.Context, key []byte, ptr interface{})
	}

	// ConsensusParamSetter defines the interface fulfilled by BaseApp's
	// ParamStore which allows setting its appVersion field.
	ConsensusParamSetter interface {
		Get(ctx context.Context) (cmtproto.ConsensusParams, error)
		Has(ctx context.Context) (bool, error)
		Set(ctx context.Context, cp cmtproto.ConsensusParams) error
	}
)
