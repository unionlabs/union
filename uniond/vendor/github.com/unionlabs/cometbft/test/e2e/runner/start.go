package main

import (
	"context"
	"errors"
	"sort"
	"time"

	"github.com/cometbft/cometbft/libs/log"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/infra"
)

func Start(ctx context.Context, testnet *e2e.Testnet, p infra.Provider) error {
	if len(testnet.Nodes) == 0 {
		return errors.New("no nodes in testnet")
	}

	// Nodes are already sorted by name. Sort them by name then startAt,
	// which gives the overall order startAt, mode, name.
	nodeQueue := testnet.Nodes
	sort.SliceStable(nodeQueue, func(i, j int) bool {
		a, b := nodeQueue[i], nodeQueue[j]
		switch {
		case a.Mode == b.Mode:
			return false
		case a.Mode == e2e.ModeSeed:
			return true
		case a.Mode == e2e.ModeValidator && b.Mode == e2e.ModeFull:
			return true
		}
		return false
	})

	sort.SliceStable(nodeQueue, func(i, j int) bool {
		return nodeQueue[i].StartAt < nodeQueue[j].StartAt
	})

	if nodeQueue[0].StartAt > 0 {
		return errors.New("no initial nodes in testnet")
	}

	// Start initial nodes (StartAt: 0)
	logger.Info("Starting initial network nodes...")
	nodesAtZero := make([]*e2e.Node, 0)
	for len(nodeQueue) > 0 && nodeQueue[0].StartAt == 0 {
		nodesAtZero = append(nodesAtZero, nodeQueue[0])
		nodeQueue = nodeQueue[1:]
	}
	err := p.StartNodes(context.Background(), nodesAtZero...)
	if err != nil {
		return err
	}
	for _, node := range nodesAtZero {
		if _, err := waitForNode(ctx, node, 0, 15*time.Second); err != nil {
			return err
		}
		if node.PrometheusProxyPort > 0 {
			logger.Info("start", "msg",
				log.NewLazySprintf("Node %v up on http://%s:%v; with Prometheus on http://%s:%v/metrics",
					node.Name, node.ExternalIP, node.RPCProxyPort, node.ExternalIP, node.PrometheusProxyPort),
			)
		} else {
			logger.Info("start", "msg", log.NewLazySprintf("Node %v up on http://%s:%v",
				node.Name, node.ExternalIP, node.RPCProxyPort))
		}
		if node.ZoneIsSet() {
			logger.Info("setting latency", "zone", node.Zone)
			if err := p.SetLatency(ctx, node); err != nil {
				return err
			}
		}
	}

	networkHeight := testnet.InitialHeight

	// Wait for initial height
	logger.Info("Waiting for initial height",
		"height", networkHeight,
		"nodes", len(testnet.Nodes)-len(nodeQueue),
		"pending", len(nodeQueue))

	block, blockID, err := waitForHeight(ctx, testnet, networkHeight)
	if err != nil {
		return err
	}

	// Update any state sync nodes with a trusted height and hash
	for _, node := range nodeQueue {
		if node.StateSync || node.Mode == e2e.ModeLight {
			err = UpdateConfigStateSync(node, block.Height, blockID.Hash.Bytes())
			if err != nil {
				return err
			}
		}
	}

	for _, node := range nodeQueue {
		if node.StartAt > networkHeight {
			// if we're starting a node that's ahead of
			// the last known height of the network, then
			// we should make sure that the rest of the
			// network has reached at least the height
			// that this node will start at before we
			// start the node.

			networkHeight = node.StartAt

			logger.Info("Waiting for network to advance before starting catch up node",
				"node", node.Name,
				"height", networkHeight)

			if _, _, err := waitForHeight(ctx, testnet, networkHeight); err != nil {
				return err
			}
		}

		logger.Info("Starting catch up node", "node", node.Name, "height", node.StartAt)

		err := p.StartNodes(context.Background(), node)
		if err != nil {
			return err
		}
		status, err := waitForNode(ctx, node, node.StartAt, 3*time.Minute)
		if err != nil {
			return err
		}
		if node.PrometheusProxyPort > 0 {
			logger.Info("start", "msg", log.NewLazySprintf("Node %v up on http://%s:%v at height %v; with Prometheus on http://%s:%v/metrics",
				node.Name, node.ExternalIP, node.RPCProxyPort, status.SyncInfo.LatestBlockHeight, node.ExternalIP, node.PrometheusProxyPort))
		} else {
			logger.Info("start", "msg", log.NewLazySprintf("Node %v up on http://%s:%v at height %v",
				node.Name, node.ExternalIP, node.RPCProxyPort, status.SyncInfo.LatestBlockHeight))
		}
		if node.ZoneIsSet() {
			logger.Info("setting latency", "zone", node.Zone)
			if err := p.SetLatency(ctx, node); err != nil {
				return err
			}
		}
	}

	return nil
}
