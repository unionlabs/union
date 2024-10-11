package rpc

import (
	"context"
	"encoding/hex"
	"fmt"
	"time"

	cmt "github.com/cometbft/cometbft/api/cometbft/types/v1"
	coretypes "github.com/cometbft/cometbft/rpc/core/types"

	"github.com/cosmos/cosmos-sdk/client"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

// GetChainHeight returns the current blockchain height.
func GetChainHeight(clientCtx client.Context) (int64, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return -1, err
	}

	status, err := node.Status(context.Background())
	if err != nil {
		return -1, err
	}

	height := status.SyncInfo.LatestBlockHeight
	return height, nil
}

// QueryBlocks performs a search for blocks based on BeginBlock and EndBlock
// events via the CometBFT RPC. A custom query may be passed as described below:
//
// To tell which events you want, you need to provide a query. query is a
// string, which has a form: "condition AND condition ..." (no OR at the
// 	moment). condition has a form: "key operation operand". key is a string with
// 	a restricted set of possible symbols ( \t\n\r\\()"'=>< are not allowed).
// 	operation can be "=", "<", "<=", ">", ">=", "CONTAINS" AND "EXISTS". operand
// 	can be a string (escaped with single quotes), number, date or time.

//	Examples:
//		  tm.event = 'NewBlock'               # new blocks
//		  tm.event = 'CompleteProposal'       # node got a complete proposal
//		  tm.event = 'Tx' AND tx.hash = 'XYZ' # single transaction
//		  tm.event = 'Tx' AND tx.height = 5   # all txs of the fifth block
//		  tx.height = 5                       # all txs of the fifth block
//
// For more information, see the /subscribe CometBFT RPC endpoint documentation
func QueryBlocks(clientCtx client.Context, page, limit int, query, orderBy string) (*sdk.SearchBlocksResult, error) {
	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}

	resBlocks, err := node.BlockSearch(context.Background(), query, &page, &limit, orderBy)
	if err != nil {
		return nil, err
	}

	blocks, err := formatBlockResults(resBlocks.Blocks)
	if err != nil {
		return nil, err
	}

	result := sdk.NewSearchBlocksResult(int64(resBlocks.TotalCount), int64(len(blocks)), int64(page), int64(limit), blocks)

	return result, nil
}

// GetBlockByHeight get block by height
func GetBlockByHeight(clientCtx client.Context, height *int64) (*cmt.Block, error) {
	// get the node
	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}

	// header -> BlockchainInfo
	// header, tx -> Block
	// results -> BlockResults
	resBlock, err := node.Block(context.Background(), height)
	if err != nil {
		return nil, err
	}

	out := sdk.NewResponseResultBlock(resBlock, resBlock.Block.Time.Format(time.RFC3339))
	if out == nil {
		return nil, fmt.Errorf("unable to create response block from comet result block: %v", resBlock)
	}

	return out, nil
}

func GetBlockByHash(clientCtx client.Context, hashHexString string) (*cmt.Block, error) {
	hash, err := hex.DecodeString(hashHexString)
	if err != nil {
		return nil, err
	}

	// get the node
	node, err := clientCtx.GetNode()
	if err != nil {
		return nil, err
	}

	resBlock, err := node.BlockByHash(context.Background(), hash)

	if err != nil {
		return nil, err
	} else if resBlock.Block == nil {
		return nil, fmt.Errorf("block not found with hash: %s", hashHexString)
	}

	out := sdk.NewResponseResultBlock(resBlock, resBlock.Block.Time.Format(time.RFC3339))
	if out == nil {
		return nil, fmt.Errorf("unable to create response block from comet result block: %v", resBlock)
	}

	return out, nil
}

// formatBlockResults parses the indexed blocks into a slice of BlockResponse objects.
func formatBlockResults(resBlocks []*coretypes.ResultBlock) ([]*cmt.Block, error) {
	out := make([]*cmt.Block, len(resBlocks))
	for i := range resBlocks {
		out[i] = sdk.NewResponseResultBlock(resBlocks[i], resBlocks[i].Block.Time.Format(time.RFC3339))
		if out[i] == nil {
			return nil, fmt.Errorf("unable to create response block from comet result block: %v", resBlocks[i])
		}
	}

	return out, nil
}
