package cmtservice

import (
	"context"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"
)

// GetNodeStatus returns the status of the node.
func GetNodeStatus(ctx context.Context, rpc CometRPC) (*coretypes.ResultStatus, error) {
	return rpc.Status(ctx)
}
