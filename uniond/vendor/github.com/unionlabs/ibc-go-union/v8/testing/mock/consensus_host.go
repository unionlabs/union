package mock

import (
	"context"

	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	"github.com/cosmos/ibc-go/v8/modules/core/exported"
)

var _ clienttypes.ConsensusHost = (*ConsensusHost)(nil)

type ConsensusHost struct {
	GetSelfConsensusStateFn func(ctx context.Context, height exported.Height) (exported.ConsensusState, error)
	ValidateSelfClientFn    func(ctx context.Context, clientState exported.ClientState) error
}

func (cv *ConsensusHost) GetSelfConsensusState(ctx context.Context, height exported.Height) (exported.ConsensusState, error) {
	if cv.GetSelfConsensusStateFn == nil {
		return nil, nil
	}

	return cv.GetSelfConsensusStateFn(ctx, height)
}

func (cv *ConsensusHost) ValidateSelfClient(ctx context.Context, clientState exported.ClientState) error {
	if cv.ValidateSelfClientFn == nil {
		return nil
	}

	return cv.ValidateSelfClientFn(ctx, clientState)
}
