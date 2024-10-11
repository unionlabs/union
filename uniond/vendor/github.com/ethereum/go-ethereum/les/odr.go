// Copyright 2016 The go-ethereum Authors
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

package les

import (
	"context"
	"math/rand"
	"sort"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/light"
)

// LesOdr implements light.OdrBackend
type LesOdr struct {
	db                                         ethdb.Database
	indexerConfig                              *light.IndexerConfig
	chtIndexer, bloomTrieIndexer, bloomIndexer *core.ChainIndexer
	peers                                      *serverPeerSet
	retriever                                  *retrieveManager
	stop                                       chan struct{}
}

func NewLesOdr(db ethdb.Database, config *light.IndexerConfig, peers *serverPeerSet, retriever *retrieveManager) *LesOdr {
	return &LesOdr{
		db:            db,
		indexerConfig: config,
		peers:         peers,
		retriever:     retriever,
		stop:          make(chan struct{}),
	}
}

// Stop cancels all pending retrievals
func (odr *LesOdr) Stop() {
	close(odr.stop)
}

// Database returns the backing database
func (odr *LesOdr) Database() ethdb.Database {
	return odr.db
}

// SetIndexers adds the necessary chain indexers to the ODR backend
func (odr *LesOdr) SetIndexers(chtIndexer, bloomTrieIndexer, bloomIndexer *core.ChainIndexer) {
	odr.chtIndexer = chtIndexer
	odr.bloomTrieIndexer = bloomTrieIndexer
	odr.bloomIndexer = bloomIndexer
}

// ChtIndexer returns the CHT chain indexer
func (odr *LesOdr) ChtIndexer() *core.ChainIndexer {
	return odr.chtIndexer
}

// BloomTrieIndexer returns the bloom trie chain indexer
func (odr *LesOdr) BloomTrieIndexer() *core.ChainIndexer {
	return odr.bloomTrieIndexer
}

// BloomIndexer returns the bloombits chain indexer
func (odr *LesOdr) BloomIndexer() *core.ChainIndexer {
	return odr.bloomIndexer
}

// IndexerConfig returns the indexer config.
func (odr *LesOdr) IndexerConfig() *light.IndexerConfig {
	return odr.indexerConfig
}

const (
	MsgBlockHeaders = iota
	MsgBlockBodies
	MsgCode
	MsgReceipts
	MsgProofsV2
	MsgHelperTrieProofs
	MsgTxStatus
)

// Msg encodes a LES message that delivers reply data for a request
type Msg struct {
	MsgType int
	ReqID   uint64
	Obj     interface{}
}

// peerByTxHistory is a heap.Interface implementation which can sort
// the peerset by transaction history.
type peerByTxHistory []*serverPeer

func (h peerByTxHistory) Len() int { return len(h) }
func (h peerByTxHistory) Less(i, j int) bool {
	if h[i].txHistory == txIndexUnlimited {
		return false
	}
	if h[j].txHistory == txIndexUnlimited {
		return true
	}
	return h[i].txHistory < h[j].txHistory
}
func (h peerByTxHistory) Swap(i, j int) { h[i], h[j] = h[j], h[i] }

const (
	maxTxStatusRetry      = 3 // The maximum retries will be made for tx status request.
	maxTxStatusCandidates = 5 // The maximum les servers the tx status requests will be sent to.
)

// RetrieveTxStatus retrieves the transaction status from the LES network.
// There is no guarantee in the LES protocol that the mined transaction will
// be retrieved back for sure because of different reasons(the transaction
// is unindexed, the malicious server doesn't reply it deliberately, etc).
// Therefore, unretrieved transactions(UNKNOWN) will receive a certain number
// of retries, thus giving a weak guarantee.
func (odr *LesOdr) RetrieveTxStatus(ctx context.Context, req *light.TxStatusRequest) error {
	// Sort according to the transaction history supported by the peer and
	// select the peers with longest history.
	var (
		retries int
		peers   []*serverPeer
		missing = len(req.Hashes)
		result  = make([]light.TxStatus, len(req.Hashes))
		canSend = make(map[string]bool)
	)
	for _, peer := range odr.peers.allPeers() {
		if peer.txHistory == txIndexDisabled {
			continue
		}
		peers = append(peers, peer)
	}
	sort.Sort(sort.Reverse(peerByTxHistory(peers)))
	for i := 0; i < maxTxStatusCandidates && i < len(peers); i++ {
		canSend[peers[i].id] = true
	}
	// Send out the request and assemble the result.
	for {
		if retries >= maxTxStatusRetry || len(canSend) == 0 {
			break
		}
		var (
			// Deep copy the request, so that the partial result won't be mixed.
			req     = &TxStatusRequest{Hashes: req.Hashes}
			id      = rand.Uint64()
			distreq = &distReq{
				getCost: func(dp distPeer) uint64 { return req.GetCost(dp.(*serverPeer)) },
				canSend: func(dp distPeer) bool { return canSend[dp.(*serverPeer).id] },
				request: func(dp distPeer) func() {
					p := dp.(*serverPeer)
					p.fcServer.QueuedRequest(id, req.GetCost(p))
					delete(canSend, p.id)
					return func() { req.Request(id, p) }
				},
			}
		)
		if err := odr.retriever.retrieve(ctx, id, distreq, func(p distPeer, msg *Msg) error { return req.Validate(odr.db, msg) }, odr.stop); err != nil {
			return err
		}
		// Collect the response and assemble them to the final result.
		// All the response is not verifiable, so always pick the first
		// one we get.
		for index, status := range req.Status {
			if result[index].Status != txpool.TxStatusUnknown {
				continue
			}
			if status.Status == txpool.TxStatusUnknown {
				continue
			}
			result[index], missing = status, missing-1
		}
		// Abort the procedure if all the status are retrieved
		if missing == 0 {
			break
		}
		retries += 1
	}
	req.Status = result
	return nil
}

// Retrieve tries to fetch an object from the LES network. It's a common API
// for most of the LES requests except for the TxStatusRequest which needs
// the additional retry mechanism.
// If the network retrieval was successful, it stores the object in local db.
func (odr *LesOdr) Retrieve(ctx context.Context, req light.OdrRequest) (err error) {
	lreq := LesRequest(req)

	reqID := rand.Uint64()
	rq := &distReq{
		getCost: func(dp distPeer) uint64 {
			return lreq.GetCost(dp.(*serverPeer))
		},
		canSend: func(dp distPeer) bool {
			p := dp.(*serverPeer)
			if !p.onlyAnnounce {
				return lreq.CanSend(p)
			}
			return false
		},
		request: func(dp distPeer) func() {
			p := dp.(*serverPeer)
			cost := lreq.GetCost(p)
			p.fcServer.QueuedRequest(reqID, cost)
			return func() { lreq.Request(reqID, p) }
		},
	}

	defer func(sent mclock.AbsTime) {
		if err != nil {
			return
		}
		requestRTT.Update(time.Duration(mclock.Now() - sent))
	}(mclock.Now())

	if err := odr.retriever.retrieve(ctx, reqID, rq, func(p distPeer, msg *Msg) error { return lreq.Validate(odr.db, msg) }, odr.stop); err != nil {
		return err
	}
	req.StoreResult(odr.db)
	return nil
}
