// Copyright 2023 The go-ethereum Authors
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
	"errors"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/log"
)

// BeaconDevSync is a development helper to test synchronization by providing
// a block hash instead of header to run the beacon sync against.
//
// The method will reach out to the network to retrieve the header of the sync
// target instead of receiving it from the consensus node.
//
// Note, this must not be used in live code. If the forkchcoice endpoint where
// to use this instead of giving us the payload first, then essentially nobody
// in the network would have the block yet that we'd attempt to retrieve.
func (d *Downloader) BeaconDevSync(mode SyncMode, hash common.Hash, stop chan struct{}) error {
	// Be very loud that this code should not be used in a live node
	log.Warn("----------------------------------")
	log.Warn("Beacon syncing with hash as target", "hash", hash)
	log.Warn("This is unhealthy for a live node!")
	log.Warn("----------------------------------")

	log.Info("Waiting for peers to retrieve sync target")
	for {
		// If the node is going down, unblock
		select {
		case <-stop:
			return errors.New("stop requested")
		default:
		}
		// Pick a random peer to sync from and keep retrying if none are yet
		// available due to fresh startup
		d.peers.lock.RLock()
		var peer *peerConnection
		for _, peer = range d.peers.peers {
			break
		}
		d.peers.lock.RUnlock()

		if peer == nil {
			time.Sleep(time.Second)
			continue
		}
		// Found a peer, attempt to retrieve the header whilst blocking and
		// retry if it fails for whatever reason
		log.Info("Attempting to retrieve sync target", "peer", peer.id)
		headers, metas, err := d.fetchHeadersByHash(peer, hash, 1, 0, false)
		if err != nil || len(headers) != 1 {
			log.Warn("Failed to fetch sync target", "headers", len(headers), "err", err)
			time.Sleep(time.Second)
			continue
		}
		// Head header retrieved, if the hash matches, start the actual sync
		if metas[0] != hash {
			log.Error("Received invalid sync target", "want", hash, "have", metas[0])
			time.Sleep(time.Second)
			continue
		}
		return d.BeaconSync(mode, headers[0], headers[0])
	}
}
