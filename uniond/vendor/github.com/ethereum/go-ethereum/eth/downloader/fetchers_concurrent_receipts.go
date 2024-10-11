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

// receiptQueue implements typedQueue and is a type adapter between the generic
// concurrent fetcher and the downloader.
type receiptQueue Downloader

// waker returns a notification channel that gets pinged in case more receipt
// fetches have been queued up, so the fetcher might assign it to idle peers.
func (q *receiptQueue) waker() chan bool {
	return q.queue.receiptWakeCh
}

// pending returns the number of receipt that are currently queued for fetching
// by the concurrent downloader.
func (q *receiptQueue) pending() int {
	return q.queue.PendingReceipts()
}

// capacity is responsible for calculating how many receipts a particular peer is
// estimated to be able to retrieve within the allotted round trip time.
func (q *receiptQueue) capacity(peer *peerConnection, rtt time.Duration) int {
	return peer.ReceiptCapacity(rtt)
}

// updateCapacity is responsible for updating how many receipts a particular peer
// is estimated to be able to retrieve in a unit time.
func (q *receiptQueue) updateCapacity(peer *peerConnection, items int, span time.Duration) {
	peer.UpdateReceiptRate(items, span)
}

// reserve is responsible for allocating a requested number of pending receipts
// from the download queue to the specified peer.
func (q *receiptQueue) reserve(peer *peerConnection, items int) (*fetchRequest, bool, bool) {
	return q.queue.ReserveReceipts(peer, items)
}

// unreserve is responsible for removing the current receipt retrieval allocation
// assigned to a specific peer and placing it back into the pool to allow
// reassigning to some other peer.
func (q *receiptQueue) unreserve(peer string) int {
	fails := q.queue.ExpireReceipts(peer)
	if fails > 2 {
		log.Trace("Receipt delivery timed out", "peer", peer)
	} else {
		log.Debug("Receipt delivery stalling", "peer", peer)
	}
	return fails
}

// request is responsible for converting a generic fetch request into a receipt
// one and sending it to the remote peer for fulfillment.
func (q *receiptQueue) request(peer *peerConnection, req *fetchRequest, resCh chan *eth.Response) (*eth.Request, error) {
	peer.log.Trace("Requesting new batch of receipts", "count", len(req.Headers), "from", req.Headers[0].Number)
	if q.receiptFetchHook != nil {
		q.receiptFetchHook(req.Headers)
	}
	hashes := make([]common.Hash, 0, len(req.Headers))
	for _, header := range req.Headers {
		hashes = append(hashes, header.Hash())
	}
	return peer.peer.RequestReceipts(hashes, resCh)
}

// deliver is responsible for taking a generic response packet from the concurrent
// fetcher, unpacking the receipt data and delivering it to the downloader's queue.
func (q *receiptQueue) deliver(peer *peerConnection, packet *eth.Response) (int, error) {
	receipts := *packet.Res.(*eth.ReceiptsResponse)
	hashes := packet.Meta.([]common.Hash) // {receipt hashes}

	accepted, err := q.queue.DeliverReceipts(peer.id, receipts, hashes)
	switch {
	case err == nil && len(receipts) == 0:
		peer.log.Trace("Requested receipts delivered")
	case err == nil:
		peer.log.Trace("Delivered new batch of receipts", "count", len(receipts), "accepted", accepted)
	default:
		peer.log.Debug("Failed to deliver retrieved receipts", "err", err)
	}
	return accepted, err
}
