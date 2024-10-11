package mock

import (
	"context"

	"github.com/cometbft/cometbft/light/provider"
	"github.com/cometbft/cometbft/types"
)

type deadMock struct {
	chainID string
}

// NewDeadMock creates a mock provider that always errors.
func NewDeadMock(chainID string) provider.Provider {
	return &deadMock{chainID: chainID}
}

func (p *deadMock) ChainID() string { return p.chainID }

func (*deadMock) String() string { return "deadMock" }

func (*deadMock) LightBlock(context.Context, int64) (*types.LightBlock, error) {
	return nil, provider.ErrNoResponse
}

func (*deadMock) ReportEvidence(context.Context, types.Evidence) error {
	return provider.ErrNoResponse
}
