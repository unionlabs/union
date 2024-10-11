package iface

import (
	"context"

	"github.com/golang/protobuf/ptypes/empty"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

type NodeClient interface {
	GetSyncStatus(ctx context.Context, in *empty.Empty) (*ethpb.SyncStatus, error)
	GetGenesis(ctx context.Context, in *empty.Empty) (*ethpb.Genesis, error)
	GetVersion(ctx context.Context, in *empty.Empty) (*ethpb.Version, error)
	ListPeers(ctx context.Context, in *empty.Empty) (*ethpb.Peers, error)
	IsHealthy(ctx context.Context) bool
}
