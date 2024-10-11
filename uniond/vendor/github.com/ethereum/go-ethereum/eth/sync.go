// Copyright 2015 The go-ethereum Authors
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

package eth

import (
	"errors"
	"math/big"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/eth/downloader"
	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/log"
)

const (
	forceSyncCycle      = 10 * time.Second // Time interval to force syncs, even if few peers are available
	defaultMinSyncPeers = 5                // Amount of peers desired to start syncing
)

// syncTransactions starts sending all currently pending transactions to the given peer.
func (h *handler) syncTransactions(p *eth.Peer) {
	var hashes []common.Hash
	for _, batch := range h.txpool.Pending(false) {
		for _, tx := range batch {
			hashes = append(hashes, tx.Hash)
		}
	}
	if len(hashes) == 0 {
		return
	}
	p.AsyncSendPooledTransactionHashes(hashes)
}

// chainSyncer coordinates blockchain sync components.
type chainSyncer struct {
	handler     *handler
	force       *time.Timer
	forced      bool // true when force timer fired
	warned      time.Time
	peerEventCh chan struct{}
	doneCh      chan error // non-nil when sync is running
}

// chainSyncOp is a scheduled sync operation.
type chainSyncOp struct {
	mode downloader.SyncMode
	peer *eth.Peer
	td   *big.Int
	head common.Hash
}

// newChainSyncer creates a chainSyncer.
func newChainSyncer(handler *handler) *chainSyncer {
	return &chainSyncer{
		handler:     handler,
		peerEventCh: make(chan struct{}),
	}
}

// handlePeerEvent notifies the syncer about a change in the peer set.
// This is called for new peers and every time a peer announces a new
// chain head.
func (cs *chainSyncer) handlePeerEvent() bool {
	select {
	case cs.peerEventCh <- struct{}{}:
		return true
	case <-cs.handler.quitSync:
		return false
	}
}

// loop runs in its own goroutine and launches the sync when necessary.
func (cs *chainSyncer) loop() {
	defer cs.handler.wg.Done()

	cs.handler.blockFetcher.Start()
	cs.handler.txFetcher.Start()
	defer cs.handler.blockFetcher.Stop()
	defer cs.handler.txFetcher.Stop()
	defer cs.handler.downloader.Terminate()

	// The force timer lowers the peer count threshold down to one when it fires.
	// This ensures we'll always start sync even if there aren't enough peers.
	cs.force = time.NewTimer(forceSyncCycle)
	defer cs.force.Stop()

	for {
		if op := cs.nextSyncOp(); op != nil {
			cs.startSync(op)
		}
		select {
		case <-cs.peerEventCh:
			// Peer information changed, recheck.
		case err := <-cs.doneCh:
			cs.doneCh = nil
			cs.force.Reset(forceSyncCycle)
			cs.forced = false

			// If we've reached the merge transition but no beacon client is available, or
			// it has not yet switched us over, keep warning the user that their infra is
			// potentially flaky.
			if errors.Is(err, downloader.ErrMergeTransition) && time.Since(cs.warned) > 10*time.Second {
				log.Warn("Local chain is post-merge, waiting for beacon client sync switch-over...")
				cs.warned = time.Now()
			}
		case <-cs.force.C:
			cs.forced = true

		case <-cs.handler.quitSync:
			// Disable all insertion on the blockchain. This needs to happen before
			// terminating the downloader because the downloader waits for blockchain
			// inserts, and these can take a long time to finish.
			cs.handler.chain.StopInsert()
			cs.handler.downloader.Terminate()
			if cs.doneCh != nil {
				<-cs.doneCh
			}
			return
		}
	}
}

// nextSyncOp determines whether sync is required at this time.
func (cs *chainSyncer) nextSyncOp() *chainSyncOp {
	if cs.doneCh != nil {
		return nil // Sync already running
	}
	// If a beacon client once took over control, disable the entire legacy sync
	// path from here on end. Note, there is a slight "race" between reaching TTD
	// and the beacon client taking over. The downloader will enforce that nothing
	// above the first TTD will be delivered to the chain for import.
	//
	// An alternative would be to check the local chain for exceeding the TTD and
	// avoid triggering a sync in that case, but that could also miss sibling or
	// other family TTD block being accepted.
	if cs.handler.chain.Config().TerminalTotalDifficultyPassed || cs.handler.merger.TDDReached() {
		return nil
	}
	// Ensure we're at minimum peer count.
	minPeers := defaultMinSyncPeers
	if cs.forced {
		minPeers = 1
	} else if minPeers > cs.handler.maxPeers {
		minPeers = cs.handler.maxPeers
	}
	if cs.handler.peers.len() < minPeers {
		return nil
	}
	// We have enough peers, pick the one with the highest TD, but avoid going
	// over the terminal total difficulty. Above that we expect the consensus
	// clients to direct the chain head to sync to.
	peer := cs.handler.peers.peerWithHighestTD()
	if peer == nil {
		return nil
	}
	mode, ourTD := cs.modeAndLocalHead()
	op := peerToSyncOp(mode, peer)
	if op.td.Cmp(ourTD) <= 0 {
		// We seem to be in sync according to the legacy rules. In the merge
		// world, it can also mean we're stuck on the merge block, waiting for
		// a beacon client. In the latter case, notify the user.
		if ttd := cs.handler.chain.Config().TerminalTotalDifficulty; ttd != nil && ourTD.Cmp(ttd) >= 0 && time.Since(cs.warned) > 10*time.Second {
			log.Warn("Local chain is post-merge, waiting for beacon client sync switch-over...")
			cs.warned = time.Now()
		}
		return nil // We're in sync
	}
	return op
}

func peerToSyncOp(mode downloader.SyncMode, p *eth.Peer) *chainSyncOp {
	peerHead, peerTD := p.Head()
	return &chainSyncOp{mode: mode, peer: p, td: peerTD, head: peerHead}
}

func (cs *chainSyncer) modeAndLocalHead() (downloader.SyncMode, *big.Int) {
	// If we're in snap sync mode, return that directly
	if cs.handler.snapSync.Load() {
		block := cs.handler.chain.CurrentSnapBlock()
		td := cs.handler.chain.GetTd(block.Hash(), block.Number.Uint64())
		return downloader.SnapSync, td
	}
	// We are probably in full sync, but we might have rewound to before the
	// snap sync pivot, check if we should re-enable snap sync.
	head := cs.handler.chain.CurrentBlock()
	if pivot := rawdb.ReadLastPivotNumber(cs.handler.database); pivot != nil {
		if head.Number.Uint64() < *pivot {
			block := cs.handler.chain.CurrentSnapBlock()
			td := cs.handler.chain.GetTd(block.Hash(), block.Number.Uint64())
			return downloader.SnapSync, td
		}
	}
	// We are in a full sync, but the associated head state is missing. To complete
	// the head state, forcefully rerun the snap sync. Note it doesn't mean the
	// persistent state is corrupted, just mismatch with the head block.
	if !cs.handler.chain.HasState(head.Root) {
		block := cs.handler.chain.CurrentSnapBlock()
		td := cs.handler.chain.GetTd(block.Hash(), block.Number.Uint64())
		log.Info("Reenabled snap sync as chain is stateless")
		return downloader.SnapSync, td
	}
	// Nope, we're really full syncing
	td := cs.handler.chain.GetTd(head.Hash(), head.Number.Uint64())
	return downloader.FullSync, td
}

// startSync launches doSync in a new goroutine.
func (cs *chainSyncer) startSync(op *chainSyncOp) {
	cs.doneCh = make(chan error, 1)
	go func() { cs.doneCh <- cs.handler.doSync(op) }()
}

// doSync synchronizes the local blockchain with a remote peer.
func (h *handler) doSync(op *chainSyncOp) error {
	if op.mode == downloader.SnapSync {
		// Before launch the snap sync, we have to ensure user uses the same
		// txlookup limit.
		// The main concern here is: during the snap sync Geth won't index the
		// block(generate tx indices) before the HEAD-limit. But if user changes
		// the limit in the next snap sync(e.g. user kill Geth manually and
		// restart) then it will be hard for Geth to figure out the oldest block
		// has been indexed. So here for the user-experience wise, it's non-optimal
		// that user can't change limit during the snap sync. If changed, Geth
		// will just blindly use the original one.
		limit := h.chain.TxLookupLimit()
		if stored := rawdb.ReadFastTxLookupLimit(h.database); stored == nil {
			rawdb.WriteFastTxLookupLimit(h.database, limit)
		} else if *stored != limit {
			h.chain.SetTxLookupLimit(*stored)
			log.Warn("Update txLookup limit", "provided", limit, "updated", *stored)
		}
	}
	// Run the sync cycle, and disable snap sync if we're past the pivot block
	err := h.downloader.LegacySync(op.peer.ID(), op.head, op.td, h.chain.Config().TerminalTotalDifficulty, op.mode)
	if err != nil {
		return err
	}
	h.enableSyncedFeatures()

	head := h.chain.CurrentBlock()
	if head.Number.Uint64() > 0 {
		// We've completed a sync cycle, notify all peers of new state. This path is
		// essential in star-topology networks where a gateway node needs to notify
		// all its out-of-date peers of the availability of a new block. This failure
		// scenario will most often crop up in private and hackathon networks with
		// degenerate connectivity, but it should be healthy for the mainnet too to
		// more reliably update peers or the local TD state.
		if block := h.chain.GetBlock(head.Hash(), head.Number.Uint64()); block != nil {
			h.BroadcastBlock(block, false)
		}
	}
	return nil
}
