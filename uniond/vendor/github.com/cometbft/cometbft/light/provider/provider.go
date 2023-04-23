package provider

import (
	"context"

	"github.com/cometbft/cometbft/types"
)

// Provider provides information for the light client to sync (verification
// happens in the client).
type Provider interface {
	// ChainID returns the blockchain ID.
	ChainID() string

	// LightBlock returns the LightBlock that corresponds to the given
	// height.
	//
	// 0 - the latest.
	// height must be >= 0.
	//
	// If the provider fails to fetch the LightBlock due to the IO or other
	// issues, an error will be returned.
	// If there's no LightBlock for the given height, ErrLightBlockNotFound
	// error is returned.
	LightBlock(ctx context.Context, height int64) (*types.LightBlock, error)

	// ReportEvidence reports an evidence of misbehavior.
	ReportEvidence(context.Context, types.Evidence) error
}
