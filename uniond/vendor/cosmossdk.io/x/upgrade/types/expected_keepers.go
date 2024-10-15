package types

import "context"

type ConsensusKeeper interface {
	AppVersion(ctx context.Context) (uint64, error)
}
