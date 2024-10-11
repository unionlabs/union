// Copyright 2021 The go-ethereum Authors
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
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/log"
)

// bodyQueue implements typedQueue and is a type adapter between the generic
// concurrent fetcher and the downloader.
type bodyQueue Downloader

// waker returns a notification channel that gets pinged in case more body
// fetches have been queued up, so the fetcher might assign it to idle peers.
func (q *bodyQueue) waker() chan bool {
	return q.queue.blockWakeCh
}

// pending returns the number of bodies that are currently queued for fetching
// by the concurrent downloader.
func (q *bodyQueue) pending() int {
	return q.queue.PendingBodies()
}

// capacity is responsible for calculating how many bodies a particular peer is
// estimated to be able to retrieve within the allotted round trip time.
func (q *bodyQueue) capacity(peer *peerConnection, rtt time.Duration) int {
	return peer.BodyCapacity(rtt)
}

// updateCapacity is responsible for updating how many bodies a particular peer
// is estimated to be able to retrieve in a unit time.
func (q *bodyQueue) updateCapacity(peer *peerConnection, items int, span time.Duration) {
	peer.UpdateBodyRate(items, span)
}

// reserve is responsible for allocating a requested number of pending bodies
// from the download queue to the specified peer.
func (q *bodyQueue) reserve(peer *peerConnection, items int) (*fetchRequest, bool, bool) {
	return q.queue.ReserveBodies(peer, items)
}

// unreserve is responsible for removing the current body retrieval allocation
// assigned to a specific peer and placing it back into the pool to allow
// reassigning to some other peer.
func (q *bodyQueue) unreserve(peer string) int {
	fails := q.queue.ExpireBodies(peer)
	if fails > 2 {
		log.Trace("Body delivery timed out", "peer", peer)
	} else {
		log.Debug("Body delivery stalling", "peer", peer)
	}
	return fails
}

// request is responsible for converting a generic fetch request into a body
// one and sending it to the remote peer for fulfillment.
func (q *bodyQueue) request(peer *peerConnection, req *fetchRequest, resCh chan *eth.Response) (*eth.Request, error) {
	peer.log.Trace("Requesting new batch of bodies", "count", len(req.Headers), "from", req.Headers[0].Number)
	if q.bodyFetchHook != nil {
		q.bodyFetchHook(req.Headers)
	}

	hashes := make([]common.Hash, 0, len(req.Headers))
	for _, header := range req.Headers {
		hashes = append(hashes, header.Hash())
	}
	return peer.peer.RequestBodies(hashes, resCh)
}

// deliver is responsible for taking a generic response packet from the concurrent
// fetcher, unpacking the body data and delivering it to the downloader's queue.
func (q *bodyQueue) deliver(peer *peerConnection, packet *eth.Response) (int, error) {
	txs, uncles, withdrawals := packet.Res.(*eth.BlockBodiesResponse).Unpack()
	hashsets := packet.Meta.([][]common.Hash) // {txs hashes, uncle hashes, withdrawal hashes}

	accepted, err := q.queue.DeliverBodies(peer.id, txs, hashsets[0], uncles, hashsets[1], withdrawals, hashsets[2])
	switch {
	case err == nil && len(txs) == 0:
		peer.log.Trace("Requested bodies delivered")
	case err == nil:
		peer.log.Trace("Delivered new batch of bodies", "count", len(txs), "accepted", accepted)
	default:
		peer.log.Debug("Failed to deliver retrieved bodies", "err", err)
	}
	return accepted, err
}
