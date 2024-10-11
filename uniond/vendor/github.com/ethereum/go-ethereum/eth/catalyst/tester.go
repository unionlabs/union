// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package catalyst

import (
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/eth"
	"github.com/ethereum/go-ethereum/eth/downloader"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
)

// FullSyncTester is an auxiliary service that allows Geth to perform full sync
// alone without consensus-layer attached. Users must specify a valid block hash
// as the sync target.
//
// This tester can be applied to different networks, no matter it's pre-merge or
// post-merge, but only for full-sync.
type FullSyncTester struct {
	stack   *node.Node
	backend *eth.Ethereum
	target  common.Hash
	closed  chan struct{}
	wg      sync.WaitGroup
}

// RegisterFullSyncTester registers the full-sync tester service into the node
// stack for launching and stopping the service controlled by node.
func RegisterFullSyncTester(stack *node.Node, backend *eth.Ethereum, target common.Hash) (*FullSyncTester, error) {
	cl := &FullSyncTester{
		stack:   stack,
		backend: backend,
		target:  target,
		closed:  make(chan struct{}),
	}
	stack.RegisterLifecycle(cl)
	return cl, nil
}

// Start launches the beacon sync with provided sync target.
func (tester *FullSyncTester) Start() error {
	tester.wg.Add(1)
	go func() {
		defer tester.wg.Done()

		// Trigger beacon sync with the provided block hash as trusted
		// chain head.
		err := tester.backend.Downloader().BeaconDevSync(downloader.FullSync, tester.target, tester.closed)
		if err != nil {
			log.Info("Failed to trigger beacon sync", "err", err)
		}

		ticker := time.NewTicker(time.Second * 5)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				// Stop in case the target block is already stored locally.
				if block := tester.backend.BlockChain().GetBlockByHash(tester.target); block != nil {
					log.Info("Full-sync target reached", "number", block.NumberU64(), "hash", block.Hash())
					go tester.stack.Close() // async since we need to close ourselves
					return
				}

			case <-tester.closed:
				return
			}
		}
	}()
	return nil
}

// Stop stops the full-sync tester to stop all background activities.
// This function can only be called for one time.
func (tester *FullSyncTester) Stop() error {
	close(tester.closed)
	tester.wg.Wait()
	return nil
}
