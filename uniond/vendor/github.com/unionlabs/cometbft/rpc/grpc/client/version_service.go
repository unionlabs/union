package client

import (
	"context"

	"github.com/cosmos/gogoproto/grpc"

	pbsvc "github.com/cometbft/cometbft/api/cometbft/services/version/v1"
)

// Version provides version information about a particular CometBFT node.
type Version struct {
	Node  string // The semantic version of the node software (i.e. the version of CometBFT).
	ABCI  string // The version of the ABCI protocol used by the node.
	P2P   uint64 // The version of the P2P protocol used by the node.
	Block uint64 // The version of the block protocol used by the node.
}

// VersionServiceClient provides version information about a CometBFT node.
type VersionServiceClient interface {
	GetVersion(ctx context.Context) (*Version, error)
}

type versionServiceClient struct {
	client pbsvc.VersionServiceClient
}

func newVersionServiceClient(conn grpc.ClientConn) VersionServiceClient {
	return &versionServiceClient{
		client: pbsvc.NewVersionServiceClient(conn),
	}
}

// GetVersion implements VersionServiceClient.
func (c *versionServiceClient) GetVersion(ctx context.Context) (*Version, error) {
	res, err := c.client.GetVersion(ctx, &pbsvc.GetVersionRequest{})
	if err != nil {
		return nil, err
	}
	return &Version{
		Node:  res.Node,
		ABCI:  res.Abci,
		P2P:   res.P2P,
		Block: res.Block,
	}, nil
}

type disabledVersionServiceClient struct{}

func newDisabledVersionServiceClient() VersionServiceClient {
	return &disabledVersionServiceClient{}
}

// GetVersion implements VersionServiceClient.
func (*disabledVersionServiceClient) GetVersion(context.Context) (*Version, error) {
	panic("version service client is disabled")
}
