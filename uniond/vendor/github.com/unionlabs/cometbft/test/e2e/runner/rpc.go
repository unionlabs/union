package main

import (
	"context"
	"errors"
	"fmt"
	"time"

	rpchttp "github.com/cometbft/cometbft/rpc/client/http"
	rpctypes "github.com/cometbft/cometbft/rpc/core/types"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/types"
)

// waitForHeight waits for the network to reach a certain height (or above),
// returning the highest height seen. Errors if the network is not making
// progress at all.
func waitForHeight(ctx context.Context, testnet *e2e.Testnet, height int64) (*types.Block, *types.BlockID, error) {
	var (
		err          error
		maxResult    *rpctypes.ResultBlock
		clients      = map[string]*rpchttp.HTTP{}
		lastIncrease = time.Now()
	)

	timer := time.NewTimer(0)
	defer timer.Stop()
	for {
		select {
		case <-ctx.Done():
			return nil, nil, ctx.Err()
		case <-timer.C:
			for _, node := range testnet.Nodes {
				if node.Stateless() {
					continue
				}
				client, ok := clients[node.Name]
				if !ok {
					client, err = node.Client()
					if err != nil {
						continue
					}
					clients[node.Name] = client
				}

				subctx, cancel := context.WithTimeout(ctx, 1*time.Second)
				defer cancel()
				result, err := client.Block(subctx, nil)
				if errors.Is(err, context.DeadlineExceeded) || errors.Is(err, context.Canceled) {
					return nil, nil, ctx.Err()
				}
				if err != nil {
					continue
				}
				if result.Block != nil && (maxResult == nil || result.Block.Height > maxResult.Block.Height) {
					maxResult = result
					lastIncrease = time.Now()
				}
				if maxResult != nil && maxResult.Block.Height >= height {
					return maxResult.Block, &maxResult.BlockID, nil
				}
			}

			if len(clients) == 0 {
				return nil, nil, errors.New("unable to connect to any network nodes")
			}
			if time.Since(lastIncrease) >= 20*time.Second {
				if maxResult == nil {
					return nil, nil, errors.New("chain stalled at unknown height")
				}
				return nil, nil, fmt.Errorf("chain stalled at height %v", maxResult.Block.Height)
			}
			timer.Reset(1 * time.Second)
		}
	}
}

// waitForNode waits for a node to become available and catch up to the given block height.
func waitForNode(ctx context.Context, node *e2e.Node, height int64, timeout time.Duration) (*rpctypes.ResultStatus, error) {
	client, err := node.Client()
	if err != nil {
		return nil, err
	}

	timer := time.NewTimer(0)
	defer timer.Stop()
	var curHeight int64
	lastChanged := time.Now()
	for {
		select {
		case <-ctx.Done():
			return nil, ctx.Err()
		case <-timer.C:
			status, err := client.Status(ctx)
			switch {
			case time.Since(lastChanged) > timeout:
				return nil, fmt.Errorf("timed out waiting for %v to reach height %v", node.Name, height)
			case err != nil:
			case status.SyncInfo.LatestBlockHeight >= height && (height == 0 || !status.SyncInfo.CatchingUp):
				return status, nil
			case curHeight < status.SyncInfo.LatestBlockHeight:
				curHeight = status.SyncInfo.LatestBlockHeight
				lastChanged = time.Now()
			}

			timer.Reset(300 * time.Millisecond)
		}
	}
}

// waitForAllNodes waits for all nodes to become available and catch up to the given block height.
func waitForAllNodes(ctx context.Context, testnet *e2e.Testnet, height int64, timeout time.Duration) (int64, error) {
	var lastHeight int64

	deadline := time.Now().Add(timeout)

	for _, node := range testnet.Nodes {
		if node.Mode == e2e.ModeSeed {
			continue
		}

		status, err := waitForNode(ctx, node, height, time.Until(deadline))
		if err != nil {
			return 0, err
		}

		if status.SyncInfo.LatestBlockHeight > lastHeight {
			lastHeight = status.SyncInfo.LatestBlockHeight
		}
	}

	return lastHeight, nil
}
