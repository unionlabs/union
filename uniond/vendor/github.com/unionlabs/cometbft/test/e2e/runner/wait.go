package main

import (
	"context"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
)

// Wait waits for a number of blocks to be produced, and for all nodes to catch
// up with it.
func Wait(ctx context.Context, testnet *e2e.Testnet, blocks int64) error {
	block, _, err := waitForHeight(ctx, testnet, 0)
	if err != nil {
		return err
	}
	return WaitUntil(ctx, testnet, block.Height+blocks)
}

// WaitUntil waits until a given height has been reached.
func WaitUntil(ctx context.Context, testnet *e2e.Testnet, height int64) error {
	logger.Info("wait until", "msg", log.NewLazySprintf("Waiting for all nodes to reach height %v...", height))
	_, err := waitForAllNodes(ctx, testnet, height, waitingTime(len(testnet.Nodes), height))
	if err != nil {
		return err
	}
	return nil
}

// waitingTime estimates how long it should take for a node to reach the height.
// More nodes in a network implies we may expect a slower network and may have to wait longer.
func waitingTime(nodes int, height int64) time.Duration {
	return time.Duration(20+(int64(nodes)*height)) * time.Second
}
