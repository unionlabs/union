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

package downloader

import (
	"fmt"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/log"
)

// beaconBackfiller is the chain and state backfilling that can be commenced once
// the skeleton syncer has successfully reverse downloaded all the headers up to
// the genesis block or an existing header in the database. Its operation is fully
// directed by the skeleton sync's head/tail events.
type beaconBackfiller struct {
	downloader *Downloader   // Downloader to direct via this callback implementation
	syncMode   SyncMode      // Sync mode to use for backfilling the skeleton chains
	success    func()        // Callback to run on successful sync cycle completion
	filling    bool          // Flag whether the downloader is backfilling or not
	filled     *types.Header // Last header filled by the last terminated sync loop
	started    chan struct{} // Notification channel whether the downloader inited
	lock       sync.Mutex    // Mutex protecting the sync lock
}

// newBeaconBackfiller is a helper method to create the backfiller.
func newBeaconBackfiller(dl *Downloader, success func()) backfiller {
	return &beaconBackfiller{
		downloader: dl,
		success:    success,
	}
}

// suspend cancels any background downloader threads and returns the last header
// that has been successfully backfilled.
func (b *beaconBackfiller) suspend() *types.Header {
	// If no filling is running, don't waste cycles
	b.lock.Lock()
	filling := b.filling
	filled := b.filled
	started := b.started
	b.lock.Unlock()

	if !filling {
		return filled // Return the filled header on the previous sync completion
	}
	// A previous filling should be running, though it may happen that it hasn't
	// yet started (being done on a new goroutine). Many concurrent beacon head
	// announcements can lead to sync start/stop thrashing. In that case we need
	// to wait for initialization before we can safely cancel it. It is safe to
	// read this channel multiple times, it gets closed on startup.
	<-started

	// Now that we're sure the downloader successfully started up, we can cancel
	// it safely without running the risk of data races.
	b.downloader.Cancel()

	// Sync cycle was just terminated, retrieve and return the last filled header.
	// Can't use `filled` as that contains a stale value from before cancellation.
	return b.downloader.blockchain.CurrentSnapBlock()
}

// resume starts the downloader threads for backfilling state and chain data.
func (b *beaconBackfiller) resume() {
	b.lock.Lock()
	if b.filling {
		// If a previous filling cycle is still running, just ignore this start
		// request. // TODO(karalabe): We should make this channel driven
		b.lock.Unlock()
		return
	}
	b.filling = true
	b.filled = nil
	b.started = make(chan struct{})
	mode := b.syncMode
	b.lock.Unlock()

	// Start the backfilling on its own thread since the downloader does not have
	// its own lifecycle runloop.
	go func() {
		// Set the backfiller to non-filling when download completes
		defer func() {
			b.lock.Lock()
			b.filling = false
			b.filled = b.downloader.blockchain.CurrentSnapBlock()
			b.lock.Unlock()
		}()
		// If the downloader fails, report an error as in beacon chain mode there
		// should be no errors as long as the chain we're syncing to is valid.
		if err := b.downloader.synchronise("", common.Hash{}, nil, nil, mode, true, b.started); err != nil {
			log.Error("Beacon backfilling failed", "err", err)
			return
		}
		// Synchronization succeeded. Since this happens async, notify the outer
		// context to disable snap syncing and enable transaction propagation.
		if b.success != nil {
			b.success()
		}
	}()
}

// setMode updates the sync mode from the current one to the requested one. If
// there's an active sync in progress, it will be cancelled and restarted.
func (b *beaconBackfiller) setMode(mode SyncMode) {
	// Update the old sync mode and track if it was changed
	b.lock.Lock()
	updated := b.syncMode != mode
	filling := b.filling
	b.syncMode = mode
	b.lock.Unlock()

	// If the sync mode was changed mid-sync, restart. This should never ever
	// really happen, we just handle it to detect programming errors.
	if !updated || !filling {
		return
	}
	log.Error("Downloader sync mode changed mid-run", "old", mode.String(), "new", mode.String())
	b.suspend()
	b.resume()
}

// SetBadBlockCallback sets the callback to run when a bad block is hit by the
// block processor. This method is not thread safe and should be set only once
// on startup before system events are fired.
func (d *Downloader) SetBadBlockCallback(onBadBlock badBlockFn) {
	d.badBlock = onBadBlock
}

// BeaconSync is the post-merge version of the chain synchronization, where the
// chain is not downloaded from genesis onward, rather from trusted head announces
// backwards.
//
// Internally backfilling and state sync is done the same way, but the header
// retrieval and scheduling is replaced.
func (d *Downloader) BeaconSync(mode SyncMode, head *types.Header, final *types.Header) error {
	return d.beaconSync(mode, head, final, true)
}

// BeaconExtend is an optimistic version of BeaconSync, where an attempt is made
// to extend the current beacon chain with a new header, but in case of a mismatch,
// the old sync will not be terminated and reorged, rather the new head is dropped.
//
// This is useful if a beacon client is feeding us large chunks of payloads to run,
// but is not setting the head after each.
func (d *Downloader) BeaconExtend(mode SyncMode, head *types.Header) error {
	return d.beaconSync(mode, head, nil, false)
}

// beaconSync is the post-merge version of the chain synchronization, where the
// chain is not downloaded from genesis onward, rather from trusted head announces
// backwards.
//
// Internally backfilling and state sync is done the same way, but the header
// retrieval and scheduling is replaced.
func (d *Downloader) beaconSync(mode SyncMode, head *types.Header, final *types.Header, force bool) error {
	// When the downloader starts a sync cycle, it needs to be aware of the sync
	// mode to use (full, snap). To keep the skeleton chain oblivious, inject the
	// mode into the backfiller directly.
	//
	// Super crazy dangerous type cast. Should be fine (TM), we're only using a
	// different backfiller implementation for skeleton tests.
	d.skeleton.filler.(*beaconBackfiller).setMode(mode)

	// Signal the skeleton sync to switch to a new head, however it wants
	if err := d.skeleton.Sync(head, final, force); err != nil {
		return err
	}
	return nil
}

// findBeaconAncestor tries to locate the common ancestor link of the local chain
// and the beacon chain just requested. In the general case when our node was in
// sync and on the correct chain, checking the top N links should already get us
// a match. In the rare scenario when we ended up on a long reorganisation (i.e.
// none of the head links match), we do a binary search to find the ancestor.
func (d *Downloader) findBeaconAncestor() (uint64, error) {
	// Figure out the current local head position
	var chainHead *types.Header

	switch d.getMode() {
	case FullSync:
		chainHead = d.blockchain.CurrentBlock()
	case SnapSync:
		chainHead = d.blockchain.CurrentSnapBlock()
	default:
		chainHead = d.lightchain.CurrentHeader()
	}
	number := chainHead.Number.Uint64()

	// Retrieve the skeleton bounds and ensure they are linked to the local chain
	beaconHead, beaconTail, _, err := d.skeleton.Bounds()
	if err != nil {
		// This is a programming error. The chain backfiller was called with an
		// invalid beacon sync state. Ideally we would panic here, but erroring
		// gives us at least a remote chance to recover. It's still a big fault!
		log.Error("Failed to retrieve beacon bounds", "err", err)
		return 0, err
	}
	var linked bool
	switch d.getMode() {
	case FullSync:
		linked = d.blockchain.HasBlock(beaconTail.ParentHash, beaconTail.Number.Uint64()-1)
	case SnapSync:
		linked = d.blockchain.HasFastBlock(beaconTail.ParentHash, beaconTail.Number.Uint64()-1)
	default:
		linked = d.blockchain.HasHeader(beaconTail.ParentHash, beaconTail.Number.Uint64()-1)
	}
	if !linked {
		// This is a programming error. The chain backfiller was called with a
		// tail that's not linked to the local chain. Whilst this should never
		// happen, there might be some weirdnesses if beacon sync backfilling
		// races with the user (or beacon client) calling setHead. Whilst panic
		// would be the ideal thing to do, it is safer long term to attempt a
		// recovery and fix any noticed issue after the fact.
		log.Error("Beacon sync linkup unavailable", "number", beaconTail.Number.Uint64()-1, "hash", beaconTail.ParentHash)
		return 0, fmt.Errorf("beacon linkup unavailable locally: %d [%x]", beaconTail.Number.Uint64()-1, beaconTail.ParentHash)
	}
	// Binary search to find the ancestor
	start, end := beaconTail.Number.Uint64()-1, number
	if number := beaconHead.Number.Uint64(); end > number {
		// This shouldn't really happen in a healthy network, but if the consensus
		// clients feeds us a shorter chain as the canonical, we should not attempt
		// to access non-existent skeleton items.
		log.Warn("Beacon head lower than local chain", "beacon", number, "local", end)
		end = number
	}
	for start+1 < end {
		// Split our chain interval in two, and request the hash to cross check
		check := (start + end) / 2

		h := d.skeleton.Header(check)
		n := h.Number.Uint64()

		var known bool
		switch d.getMode() {
		case FullSync:
			known = d.blockchain.HasBlock(h.Hash(), n)
		case SnapSync:
			known = d.blockchain.HasFastBlock(h.Hash(), n)
		default:
			known = d.lightchain.HasHeader(h.Hash(), n)
		}
		if !known {
			end = check
			continue
		}
		start = check
	}
	return start, nil
}

// fetchBeaconHeaders feeds skeleton headers to the downloader queue for scheduling
// until sync errors or is finished.
func (d *Downloader) fetchBeaconHeaders(from uint64) error {
	var head *types.Header
	_, tail, _, err := d.skeleton.Bounds()
	if err != nil {
		return err
	}
	// A part of headers are not in the skeleton space, try to resolve
	// them from the local chain. Note the range should be very short
	// and it should only happen when there are less than 64 post-merge
	// blocks in the network.
	var localHeaders []*types.Header
	if from < tail.Number.Uint64() {
		count := tail.Number.Uint64() - from
		if count > uint64(fsMinFullBlocks) {
			return fmt.Errorf("invalid origin (%d) of beacon sync (%d)", from, tail.Number)
		}
		localHeaders = d.readHeaderRange(tail, int(count))
		log.Warn("Retrieved beacon headers from local", "from", from, "count", count)
	}
	for {
		// Some beacon headers might have appeared since the last cycle, make
		// sure we're always syncing to all available ones
		head, _, _, err = d.skeleton.Bounds()
		if err != nil {
			return err
		}
		// If the pivot became stale (older than 2*64-8 (bit of wiggle room)),
		// move it ahead to HEAD-64
		d.pivotLock.Lock()
		if d.pivotHeader != nil {
			if head.Number.Uint64() > d.pivotHeader.Number.Uint64()+2*uint64(fsMinFullBlocks)-8 {
				// Retrieve the next pivot header, either from skeleton chain
				// or the filled chain
				number := head.Number.Uint64() - uint64(fsMinFullBlocks)

				log.Warn("Pivot seemingly stale, moving", "old", d.pivotHeader.Number, "new", number)
				if d.pivotHeader = d.skeleton.Header(number); d.pivotHeader == nil {
					if number < tail.Number.Uint64() {
						dist := tail.Number.Uint64() - number
						if len(localHeaders) >= int(dist) {
							d.pivotHeader = localHeaders[dist-1]
							log.Warn("Retrieved pivot header from local", "number", d.pivotHeader.Number, "hash", d.pivotHeader.Hash(), "latest", head.Number, "oldest", tail.Number)
						}
					}
				}
				// Print an error log and return directly in case the pivot header
				// is still not found. It means the skeleton chain is not linked
				// correctly with local chain.
				if d.pivotHeader == nil {
					log.Error("Pivot header is not found", "number", number)
					d.pivotLock.Unlock()
					return errNoPivotHeader
				}
				// Write out the pivot into the database so a rollback beyond
				// it will reenable snap sync and update the state root that
				// the state syncer will be downloading
				rawdb.WriteLastPivotNumber(d.stateDB, d.pivotHeader.Number.Uint64())
			}
		}
		d.pivotLock.Unlock()

		// Retrieve a batch of headers and feed it to the header processor
		var (
			headers = make([]*types.Header, 0, maxHeadersProcess)
			hashes  = make([]common.Hash, 0, maxHeadersProcess)
		)
		for i := 0; i < maxHeadersProcess && from <= head.Number.Uint64(); i++ {
			header := d.skeleton.Header(from)

			// The header is not found in skeleton space, try to find it in local chain.
			if header == nil && from < tail.Number.Uint64() {
				dist := tail.Number.Uint64() - from
				if len(localHeaders) >= int(dist) {
					header = localHeaders[dist-1]
				}
			}
			// The header is still missing, the beacon sync is corrupted and bail out
			// the error here.
			if header == nil {
				return fmt.Errorf("missing beacon header %d", from)
			}
			headers = append(headers, header)
			hashes = append(hashes, headers[i].Hash())
			from++
		}
		if len(headers) > 0 {
			log.Trace("Scheduling new beacon headers", "count", len(headers), "from", from-uint64(len(headers)))
			select {
			case d.headerProcCh <- &headerTask{
				headers: headers,
				hashes:  hashes,
			}:
			case <-d.cancelCh:
				return errCanceled
			}
		}
		// If we still have headers to import, loop and keep pushing them
		if from <= head.Number.Uint64() {
			continue
		}
		// If the pivot block is committed, signal header sync termination
		if d.committed.Load() {
			select {
			case d.headerProcCh <- nil:
				return nil
			case <-d.cancelCh:
				return errCanceled
			}
		}
		// State sync still going, wait a bit for new headers and retry
		log.Trace("Pivot not yet committed, waiting...")
		select {
		case <-time.After(fsHeaderContCheck):
		case <-d.cancelCh:
			return errCanceled
		}
	}
}
