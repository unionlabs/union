package cmtservice

import (
	"context"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"

	"github.com/cosmos/cosmos-sdk/client"
)

// GetNodeStatus returns the status of the node.
func GetNodeStatus(ctx context.Context, clientCtx client.Context) (*coretypes.ResultStatus, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return &coretypes.ResultStatus{}, err
	}
	return node.Status(ctx)
}
