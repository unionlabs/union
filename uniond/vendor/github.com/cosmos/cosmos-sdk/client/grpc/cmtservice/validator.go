package cmtservice

import (
	"context"

	coretypes "github.com/cometbft/cometbft/rpc/core/types"

	"github.com/cosmos/cosmos-sdk/client"
)

func getValidators(ctx context.Context, clientCtx client.Context, height *int64, page, limit int) (*coretypes.ResultValidators, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}
	return node.Validators(ctx, height, &page, &limit)
}
