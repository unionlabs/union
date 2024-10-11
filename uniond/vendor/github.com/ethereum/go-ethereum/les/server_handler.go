// Copyright 2019 The go-ethereum Authors
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
	"errors"
	"fmt"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/flowcontrol"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/trie"
)

const (
	softResponseLimit = 2 * 1024 * 1024 // Target maximum size of returned blocks, headers or node data.
	estHeaderRlpSize  = 500             // Approximate size of an RLP encoded block header

	MaxHeaderFetch           = 192 // Amount of block headers to be fetched per retrieval request
	MaxBodyFetch             = 32  // Amount of block bodies to be fetched per retrieval request
	MaxReceiptFetch          = 128 // Amount of transaction receipts to allow fetching per request
	MaxCodeFetch             = 64  // Amount of contract codes to allow fetching per request
	MaxProofsFetch           = 64  // Amount of merkle proofs to be fetched per retrieval request
	MaxHelperTrieProofsFetch = 64  // Amount of helper tries to be fetched per retrieval request
	MaxTxSend                = 64  // Amount of transactions to be send per request
	MaxTxStatus              = 256 // Amount of transactions to queried per request
)

var (
	errTooManyInvalidRequest = errors.New("too many invalid requests made")
)

// serverHandler is responsible for serving light client and process
// all incoming light requests.
type serverHandler struct {
	forkFilter forkid.Filter
	blockchain *core.BlockChain
	chainDb    ethdb.Database
	txpool     *txpool.TxPool
	server     *LesServer

	closeCh chan struct{}  // Channel used to exit all background routines of handler.
	wg      sync.WaitGroup // WaitGroup used to track all background routines of handler.
	synced  func() bool    // Callback function used to determine whether local node is synced.

	// Testing fields
	addTxsSync bool
}

func newServerHandler(server *LesServer, blockchain *core.BlockChain, chainDb ethdb.Database, txpool *txpool.TxPool, synced func() bool) *serverHandler {
	handler := &serverHandler{
		forkFilter: forkid.NewFilter(blockchain),
		server:     server,
		blockchain: blockchain,
		chainDb:    chainDb,
		txpool:     txpool,
		closeCh:    make(chan struct{}),
		synced:     synced,
	}
	return handler
}

// start starts the server handler.
func (h *serverHandler) start() {
	h.wg.Add(1)
	go h.broadcastLoop()
}

// stop stops the server handler.
func (h *serverHandler) stop() {
	close(h.closeCh)
	h.wg.Wait()
}

// runPeer is the p2p protocol run function for the given version.
func (h *serverHandler) runPeer(version uint, p *p2p.Peer, rw p2p.MsgReadWriter) error {
	peer := newClientPeer(int(version), h.server.config.NetworkId, p, newMeteredMsgWriter(rw, int(version)))
	defer peer.close()
	h.wg.Add(1)
	defer h.wg.Done()
	return h.handle(peer)
}

func (h *serverHandler) handle(p *clientPeer) error {
	p.Log().Debug("Light Ethereum peer connected", "name", p.Name())

	// Execute the LES handshake
	var (
		head   = h.blockchain.CurrentHeader()
		hash   = head.Hash()
		number = head.Number.Uint64()
		td     = h.blockchain.GetTd(hash, number)
		forkID = forkid.NewID(h.blockchain.Config(), h.blockchain.Genesis(), number, head.Time)
	)
	if err := p.Handshake(td, hash, number, h.blockchain.Genesis().Hash(), forkID, h.forkFilter, h.server); err != nil {
		p.Log().Debug("Light Ethereum handshake failed", "err", err)
		return err
	}
	// Connected to another server, no messages expected, just wait for disconnection
	if p.server {
		if err := h.server.serverset.register(p); err != nil {
			return err
		}
		_, err := p.rw.ReadMsg()
		h.server.serverset.unregister(p)
		return err
	}
	// Setup flow control mechanism for the peer
	p.fcClient = flowcontrol.NewClientNode(h.server.fcManager, p.fcParams)
	defer p.fcClient.Disconnect()

	// Reject light clients if server is not synced. Put this checking here, so
	// that "non-synced" les-server peers are still allowed to keep the connection.
	if !h.synced() {
		p.Log().Debug("Light server not synced, rejecting peer")
		return p2p.DiscRequested
	}

	// Register the peer into the peerset and clientpool
	if err := h.server.peers.register(p); err != nil {
		return err
	}
	if p.balance = h.server.clientPool.Register(p); p.balance == nil {
		h.server.peers.unregister(p.ID())
		p.Log().Debug("Client pool already closed")
		return p2p.DiscRequested
	}
	p.connectedAt = mclock.Now()

	var wg sync.WaitGroup // Wait group used to track all in-flight task routines.
	defer func() {
		wg.Wait() // Ensure all background task routines have exited.
		h.server.clientPool.Unregister(p)
		h.server.peers.unregister(p.ID())
		p.balance = nil
		connectionTimer.Update(time.Duration(mclock.Now() - p.connectedAt))
	}()

	// Mark the peer as being served.
	p.serving.Store(true)
	defer p.serving.Store(false)

	// Spawn a main loop to handle all incoming messages.
	for {
		select {
		case err := <-p.errCh:
			p.Log().Debug("Failed to send light ethereum response", "err", err)
			return err
		default:
		}
		if err := h.handleMsg(p, &wg); err != nil {
			p.Log().Debug("Light Ethereum message handling failed", "err", err)
			return err
		}
	}
}

// beforeHandle will do a series of prechecks before handling message.
func (h *serverHandler) beforeHandle(p *clientPeer, reqID, responseCount uint64, msg p2p.Msg, reqCnt uint64, maxCount uint64) (*servingTask, uint64) {
	// Ensure that the request sent by client peer is valid
	inSizeCost := h.server.costTracker.realCost(0, msg.Size, 0)
	if reqCnt == 0 || reqCnt > maxCount {
		p.fcClient.OneTimeCost(inSizeCost)
		return nil, 0
	}
	// Ensure that the client peer complies with the flow control
	// rules agreed by both sides.
	if p.isFrozen() {
		p.fcClient.OneTimeCost(inSizeCost)
		return nil, 0
	}
	maxCost := p.fcCosts.getMaxCost(msg.Code, reqCnt)
	accepted, bufShort, priority := p.fcClient.AcceptRequest(reqID, responseCount, maxCost)
	if !accepted {
		p.freeze()
		p.Log().Error("Request came too early", "remaining", common.PrettyDuration(time.Duration(bufShort*1000000/p.fcParams.MinRecharge)))
		p.fcClient.OneTimeCost(inSizeCost)
		return nil, 0
	}
	// Create a multi-stage task, estimate the time it takes for the task to
	// execute, and cache it in the request service queue.
	factor := h.server.costTracker.globalFactor()
	if factor < 0.001 {
		factor = 1
		p.Log().Error("Invalid global cost factor", "factor", factor)
	}
	maxTime := uint64(float64(maxCost) / factor)
	task := h.server.servingQueue.newTask(p, maxTime, priority)
	if !task.start() {
		p.fcClient.RequestProcessed(reqID, responseCount, maxCost, inSizeCost)
		return nil, 0
	}
	return task, maxCost
}

// Afterhandle will perform a series of operations after message handling,
// such as updating flow control data, sending reply, etc.
func (h *serverHandler) afterHandle(p *clientPeer, reqID, responseCount uint64, msg p2p.Msg, maxCost uint64, reqCnt uint64, task *servingTask, reply *reply) {
	if reply != nil {
		task.done()
	}
	p.responseLock.Lock()
	defer p.responseLock.Unlock()

	// Short circuit if the client is already frozen.
	if p.isFrozen() {
		realCost := h.server.costTracker.realCost(task.servingTime, msg.Size, 0)
		p.fcClient.RequestProcessed(reqID, responseCount, maxCost, realCost)
		return
	}
	// Positive correction buffer value with real cost.
	var replySize uint32
	if reply != nil {
		replySize = reply.size()
	}
	var realCost uint64
	if h.server.costTracker.testing {
		realCost = maxCost // Assign a fake cost for testing purpose
	} else {
		realCost = h.server.costTracker.realCost(task.servingTime, msg.Size, replySize)
		if realCost > maxCost {
			realCost = maxCost
		}
	}
	bv := p.fcClient.RequestProcessed(reqID, responseCount, maxCost, realCost)
	if reply != nil {
		// Feed cost tracker request serving statistic.
		h.server.costTracker.updateStats(msg.Code, reqCnt, task.servingTime, realCost)
		// Reduce priority "balance" for the specific peer.
		p.balance.RequestServed(realCost)
		p.queueSend(func() {
			if err := reply.send(bv); err != nil {
				select {
				case p.errCh <- err:
				default:
				}
			}
		})
	}
}

// handleMsg is invoked whenever an inbound message is received from a remote
// peer. The remote connection is torn down upon returning any error.
func (h *serverHandler) handleMsg(p *clientPeer, wg *sync.WaitGroup) error {
	// Read the next message from the remote peer, and ensure it's fully consumed
	msg, err := p.rw.ReadMsg()
	if err != nil {
		return err
	}
	p.Log().Trace("Light Ethereum message arrived", "code", msg.Code, "bytes", msg.Size)

	// Discard large message which exceeds the limitation.
	if msg.Size > ProtocolMaxMsgSize {
		clientErrorMeter.Mark(1)
		return errResp(ErrMsgTooLarge, "%v > %v", msg.Size, ProtocolMaxMsgSize)
	}
	defer msg.Discard()

	// Lookup the request handler table, ensure it's supported
	// message type by the protocol.
	req, ok := Les3[msg.Code]
	if !ok {
		p.Log().Trace("Received invalid message", "code", msg.Code)
		clientErrorMeter.Mark(1)
		return errResp(ErrInvalidMsgCode, "%v", msg.Code)
	}
	p.Log().Trace("Received " + req.Name)

	// Decode the p2p message, resolve the concrete handler for it.
	serve, reqID, reqCnt, err := req.Handle(msg)
	if err != nil {
		clientErrorMeter.Mark(1)
		return errResp(ErrDecode, "%v: %v", msg, err)
	}
	if metrics.EnabledExpensive {
		req.InPacketsMeter.Mark(1)
		req.InTrafficMeter.Mark(int64(msg.Size))
	}
	p.responseCount++
	responseCount := p.responseCount

	// First check this client message complies all rules before
	// handling it and return a processor if all checks are passed.
	task, maxCost := h.beforeHandle(p, reqID, responseCount, msg, reqCnt, req.MaxCount)
	if task == nil {
		return nil
	}
	wg.Add(1)
	go func() {
		defer wg.Done()

		reply := serve(h, p, task.waitOrStop)
		h.afterHandle(p, reqID, responseCount, msg, maxCost, reqCnt, task, reply)

		if metrics.EnabledExpensive {
			size := uint32(0)
			if reply != nil {
				size = reply.size()
			}
			req.OutPacketsMeter.Mark(1)
			req.OutTrafficMeter.Mark(int64(size))
			req.ServingTimeMeter.Update(time.Duration(task.servingTime))
		}
	}()
	// If the client has made too much invalid request(e.g. request a non-existent data),
	// reject them to prevent SPAM attack.
	if p.getInvalid() > maxRequestErrors {
		clientErrorMeter.Mark(1)
		return errTooManyInvalidRequest
	}
	return nil
}

// BlockChain implements serverBackend
func (h *serverHandler) BlockChain() *core.BlockChain {
	return h.blockchain
}

// TxPool implements serverBackend
func (h *serverHandler) TxPool() *txpool.TxPool {
	return h.txpool
}

// ArchiveMode implements serverBackend
func (h *serverHandler) ArchiveMode() bool {
	return h.server.archiveMode
}

// AddTxsSync implements serverBackend
func (h *serverHandler) AddTxsSync() bool {
	return h.addTxsSync
}

// getAccount retrieves an account from the state based on root.
func getAccount(triedb *trie.Database, root common.Hash, addr common.Address) (types.StateAccount, error) {
	trie, err := trie.NewStateTrie(trie.StateTrieID(root), triedb)
	if err != nil {
		return types.StateAccount{}, err
	}
	acc, err := trie.GetAccount(addr)
	if err != nil {
		return types.StateAccount{}, err
	}
	if acc == nil {
		return types.StateAccount{}, fmt.Errorf("account %#x is not present", addr)
	}
	return *acc, nil
}

// GetHelperTrie returns the post-processed trie root for the given trie ID and section index
func (h *serverHandler) GetHelperTrie(typ uint, index uint64) *trie.Trie {
	var (
		root   common.Hash
		prefix string
	)
	switch typ {
	case htCanonical:
		sectionHead := rawdb.ReadCanonicalHash(h.chainDb, (index+1)*h.server.iConfig.ChtSize-1)
		root, prefix = light.GetChtRoot(h.chainDb, index, sectionHead), string(rawdb.ChtTablePrefix)
	case htBloomBits:
		sectionHead := rawdb.ReadCanonicalHash(h.chainDb, (index+1)*h.server.iConfig.BloomTrieSize-1)
		root, prefix = light.GetBloomTrieRoot(h.chainDb, index, sectionHead), string(rawdb.BloomTrieTablePrefix)
	}
	if root == (common.Hash{}) {
		return nil
	}
	triedb := trie.NewDatabase(rawdb.NewTable(h.chainDb, prefix), trie.HashDefaults)
	trie, _ := trie.New(trie.TrieID(root), triedb)
	return trie
}

// broadcastLoop broadcasts new block information to all connected light
// clients. According to the agreement between client and server, server should
// only broadcast new announcement if the total difficulty is higher than the
// last one. Besides server will add the signature if client requires.
func (h *serverHandler) broadcastLoop() {
	defer h.wg.Done()

	headCh := make(chan core.ChainHeadEvent, 10)
	headSub := h.blockchain.SubscribeChainHeadEvent(headCh)
	defer headSub.Unsubscribe()

	var (
		lastHead = h.blockchain.CurrentHeader()
		lastTd   = common.Big0
	)
	for {
		select {
		case ev := <-headCh:
			header := ev.Block.Header()
			hash, number := header.Hash(), header.Number.Uint64()
			td := h.blockchain.GetTd(hash, number)
			if td == nil || td.Cmp(lastTd) <= 0 {
				continue
			}
			var reorg uint64
			if lastHead != nil {
				// If a setHead has been performed, the common ancestor can be nil.
				if ancestor := rawdb.FindCommonAncestor(h.chainDb, header, lastHead); ancestor != nil {
					reorg = lastHead.Number.Uint64() - ancestor.Number.Uint64()
				}
			}
			lastHead, lastTd = header, td
			log.Debug("Announcing block to peers", "number", number, "hash", hash, "td", td, "reorg", reorg)
			h.server.peers.broadcast(announceData{Hash: hash, Number: number, Td: td, ReorgDepth: reorg})
		case <-h.closeCh:
			return
		}
	}
}
