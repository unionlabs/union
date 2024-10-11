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
	"crypto/ecdsa"
	"errors"
	"fmt"
	"math/big"
	"math/rand"
	"net"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/forkid"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/les/flowcontrol"
	"github.com/ethereum/go-ethereum/les/utils"
	vfc "github.com/ethereum/go-ethereum/les/vflux/client"
	vfs "github.com/ethereum/go-ethereum/les/vflux/server"
	"github.com/ethereum/go-ethereum/light"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

var (
	errClosed            = errors.New("peer set is closed")
	errAlreadyRegistered = errors.New("peer is already registered")
	errNotRegistered     = errors.New("peer is not registered")
)

const (
	maxRequestErrors  = 20 // number of invalid requests tolerated (makes the protocol less brittle but still avoids spam)
	maxResponseErrors = 50 // number of invalid responses tolerated (makes the protocol less brittle but still avoids spam)

	allowedUpdateBytes = 100000                // initial/maximum allowed update size
	allowedUpdateRate  = time.Millisecond * 10 // time constant for recharging one byte of allowance

	freezeTimeBase    = time.Millisecond * 700 // fixed component of client freeze time
	freezeTimeRandom  = time.Millisecond * 600 // random component of client freeze time
	freezeCheckPeriod = time.Millisecond * 100 // buffer value recheck period after initial freeze time has elapsed

	// If the total encoded size of a sent transaction batch is over txSizeCostLimit
	// per transaction then the request cost is calculated as proportional to the
	// encoded size instead of the transaction count
	txSizeCostLimit = 0x4000

	// handshakeTimeout is the timeout LES handshake will be treated as failed.
	handshakeTimeout = 5 * time.Second
)

const (
	announceTypeNone = iota
	announceTypeSimple
	announceTypeSigned
)

type keyValueEntry struct {
	Key   string
	Value rlp.RawValue
}

type keyValueList []keyValueEntry
type keyValueMap map[string]rlp.RawValue

func (l keyValueList) add(key string, val interface{}) keyValueList {
	var entry keyValueEntry
	entry.Key = key
	if val == nil {
		val = uint64(0)
	}
	enc, err := rlp.EncodeToBytes(val)
	if err == nil {
		entry.Value = enc
	}
	return append(l, entry)
}

func (l keyValueList) decode() (keyValueMap, uint64) {
	m := make(keyValueMap)
	var size uint64
	for _, entry := range l {
		m[entry.Key] = entry.Value
		size += uint64(len(entry.Key)) + uint64(len(entry.Value)) + 8
	}
	return m, size
}

func (m keyValueMap) get(key string, val interface{}) error {
	enc, ok := m[key]
	if !ok {
		return errResp(ErrMissingKey, "%s", key)
	}
	if val == nil {
		return nil
	}
	return rlp.DecodeBytes(enc, val)
}

// peerCommons contains fields needed by both server peer and client peer.
type peerCommons struct {
	*p2p.Peer
	rw p2p.MsgReadWriter

	id           string      // Peer identity.
	version      int         // Protocol version negotiated.
	network      uint64      // Network ID being on.
	frozen       atomic.Bool // Flag whether the peer is frozen.
	announceType uint64      // New block announcement type.
	serving      atomic.Bool // The status indicates the peer is served.
	headInfo     blockInfo   // Last announced block information.

	// Background task queue for caching peer tasks and executing in order.
	sendQueue *utils.ExecQueue

	// Flow control agreement.
	fcParams flowcontrol.ServerParams // The config for token bucket.
	fcCosts  requestCostTable         // The Maximum request cost table.

	closeCh chan struct{}
	lock    sync.RWMutex // Lock used to protect all thread-sensitive fields.
}

// isFrozen returns true if the client is frozen or the server has put our
// client in frozen state
func (p *peerCommons) isFrozen() bool {
	return p.frozen.Load()
}

// canQueue returns an indicator whether the peer can queue an operation.
func (p *peerCommons) canQueue() bool {
	return p.sendQueue.CanQueue() && !p.isFrozen()
}

// queueSend caches a peer operation in the background task queue.
// Please ensure to check `canQueue` before call this function
func (p *peerCommons) queueSend(f func()) bool {
	return p.sendQueue.Queue(f)
}

// String implements fmt.Stringer.
func (p *peerCommons) String() string {
	return fmt.Sprintf("Peer %s [%s]", p.id, fmt.Sprintf("les/%d", p.version))
}

// PeerInfo represents a short summary of the `eth` sub-protocol metadata known
// about a connected peer.
type PeerInfo struct {
	Version    int      `json:"version"`    // Ethereum protocol version negotiated
	Difficulty *big.Int `json:"difficulty"` // Total difficulty of the peer's blockchain
	Head       string   `json:"head"`       // SHA3 hash of the peer's best owned block
}

// Info gathers and returns a collection of metadata known about a peer.
func (p *peerCommons) Info() *PeerInfo {
	return &PeerInfo{
		Version:    p.version,
		Difficulty: p.Td(),
		Head:       fmt.Sprintf("%x", p.Head()),
	}
}

// Head retrieves a copy of the current head (most recent) hash of the peer.
func (p *peerCommons) Head() (hash common.Hash) {
	p.lock.RLock()
	defer p.lock.RUnlock()

	return p.headInfo.Hash
}

// Td retrieves the current total difficulty of a peer.
func (p *peerCommons) Td() *big.Int {
	p.lock.RLock()
	defer p.lock.RUnlock()

	return new(big.Int).Set(p.headInfo.Td)
}

// HeadAndTd retrieves the current head hash and total difficulty of a peer.
func (p *peerCommons) HeadAndTd() (hash common.Hash, td *big.Int) {
	p.lock.RLock()
	defer p.lock.RUnlock()

	return p.headInfo.Hash, new(big.Int).Set(p.headInfo.Td)
}

// sendReceiveHandshake exchanges handshake packet with remote peer and returns any error
// if failed to send or receive packet.
func (p *peerCommons) sendReceiveHandshake(sendList keyValueList) (keyValueList, error) {
	var (
		errc     = make(chan error, 2)
		recvList keyValueList
	)
	// Send out own handshake in a new thread
	go func() {
		errc <- p2p.Send(p.rw, StatusMsg, &sendList)
	}()
	go func() {
		// In the mean time retrieve the remote status message
		msg, err := p.rw.ReadMsg()
		if err != nil {
			errc <- err
			return
		}
		if msg.Code != StatusMsg {
			errc <- errResp(ErrNoStatusMsg, "first msg has code %x (!= %x)", msg.Code, StatusMsg)
			return
		}
		if msg.Size > ProtocolMaxMsgSize {
			errc <- errResp(ErrMsgTooLarge, "%v > %v", msg.Size, ProtocolMaxMsgSize)
			return
		}
		// Decode the handshake
		if err := msg.Decode(&recvList); err != nil {
			errc <- errResp(ErrDecode, "msg %v: %v", msg, err)
			return
		}
		errc <- nil
	}()
	timeout := time.NewTimer(handshakeTimeout)
	defer timeout.Stop()
	for i := 0; i < 2; i++ {
		select {
		case err := <-errc:
			if err != nil {
				return nil, err
			}
		case <-timeout.C:
			return nil, p2p.DiscReadTimeout
		}
	}
	return recvList, nil
}

// handshake executes the les protocol handshake, negotiating version number,
// network IDs, difficulties, head and genesis blocks. Besides the basic handshake
// fields, server and client can exchange and resolve some specified fields through
// two callback functions.
func (p *peerCommons) handshake(td *big.Int, head common.Hash, headNum uint64, genesis common.Hash, forkID forkid.ID, forkFilter forkid.Filter, sendCallback func(*keyValueList), recvCallback func(keyValueMap) error) error {
	p.lock.Lock()
	defer p.lock.Unlock()

	var send keyValueList

	// Add some basic handshake fields
	send = send.add("protocolVersion", uint64(p.version))
	send = send.add("networkId", p.network)
	// Note: the head info announced at handshake is only used in case of server peers
	// but dummy values are still announced by clients for compatibility with older servers
	send = send.add("headTd", td)
	send = send.add("headHash", head)
	send = send.add("headNum", headNum)
	send = send.add("genesisHash", genesis)

	// If the protocol version is beyond les4, then pass the forkID
	// as well. Check http://eips.ethereum.org/EIPS/eip-2124 for more
	// spec detail.
	if p.version >= lpv4 {
		send = send.add("forkID", forkID)
	}
	// Add client-specified or server-specified fields
	if sendCallback != nil {
		sendCallback(&send)
	}
	// Exchange the handshake packet and resolve the received one.
	recvList, err := p.sendReceiveHandshake(send)
	if err != nil {
		return err
	}
	recv, size := recvList.decode()
	if size > allowedUpdateBytes {
		return errResp(ErrRequestRejected, "")
	}
	var rGenesis common.Hash
	var rVersion, rNetwork uint64
	if err := recv.get("protocolVersion", &rVersion); err != nil {
		return err
	}
	if err := recv.get("networkId", &rNetwork); err != nil {
		return err
	}
	if err := recv.get("genesisHash", &rGenesis); err != nil {
		return err
	}
	if rGenesis != genesis {
		return errResp(ErrGenesisBlockMismatch, "%x (!= %x)", rGenesis[:8], genesis[:8])
	}
	if rNetwork != p.network {
		return errResp(ErrNetworkIdMismatch, "%d (!= %d)", rNetwork, p.network)
	}
	if int(rVersion) != p.version {
		return errResp(ErrProtocolVersionMismatch, "%d (!= %d)", rVersion, p.version)
	}
	// Check forkID if the protocol version is beyond the les4
	if p.version >= lpv4 {
		var forkID forkid.ID
		if err := recv.get("forkID", &forkID); err != nil {
			return err
		}
		if err := forkFilter(forkID); err != nil {
			return errResp(ErrForkIDRejected, "%v", err)
		}
	}
	if recvCallback != nil {
		return recvCallback(recv)
	}
	return nil
}

// close closes the channel and notifies all background routines to exit.
func (p *peerCommons) close() {
	close(p.closeCh)
	p.sendQueue.Quit()
}

// serverPeer represents each node to which the client is connected.
// The node here refers to the les server.
type serverPeer struct {
	peerCommons

	// Status fields
	trusted                 bool   // The flag whether the server is selected as trusted server.
	onlyAnnounce            bool   // The flag whether the server sends announcement only.
	chainSince, chainRecent uint64 // The range of chain server peer can serve.
	stateSince, stateRecent uint64 // The range of state server peer can serve.
	txHistory               uint64 // The length of available tx history, 0 means all, 1 means disabled

	fcServer         *flowcontrol.ServerNode // Client side mirror token bucket.
	vtLock           sync.Mutex
	nodeValueTracker *vfc.NodeValueTracker
	sentReqs         map[uint64]sentReqEntry

	// Statistics
	errCount    utils.LinearExpiredValue // Counter the invalid responses server has replied
	updateCount uint64
	updateTime  mclock.AbsTime

	// Test callback hooks
	hasBlockHook func(common.Hash, uint64, bool) bool // Used to determine whether the server has the specified block.
}

func newServerPeer(version int, network uint64, trusted bool, p *p2p.Peer, rw p2p.MsgReadWriter) *serverPeer {
	return &serverPeer{
		peerCommons: peerCommons{
			Peer:      p,
			rw:        rw,
			id:        p.ID().String(),
			version:   version,
			network:   network,
			sendQueue: utils.NewExecQueue(100),
			closeCh:   make(chan struct{}),
		},
		trusted:  trusted,
		errCount: utils.LinearExpiredValue{Rate: mclock.AbsTime(time.Hour)},
	}
}

// rejectUpdate returns true if a parameter update has to be rejected because
// the size and/or rate of updates exceed the capacity limitation
func (p *serverPeer) rejectUpdate(size uint64) bool {
	now := mclock.Now()
	if p.updateCount == 0 {
		p.updateTime = now
	} else {
		dt := now - p.updateTime
		p.updateTime = now

		r := uint64(dt / mclock.AbsTime(allowedUpdateRate))
		if p.updateCount > r {
			p.updateCount -= r
		} else {
			p.updateCount = 0
		}
	}
	p.updateCount += size
	return p.updateCount > allowedUpdateBytes
}

// freeze processes Stop messages from the given server and set the status as
// frozen.
func (p *serverPeer) freeze() {
	if p.frozen.CompareAndSwap(false, true) {
		p.sendQueue.Clear()
	}
}

// unfreeze processes Resume messages from the given server and set the status
// as unfrozen.
func (p *serverPeer) unfreeze() {
	p.frozen.Store(false)
}

// sendRequest send a request to the server based on the given message type
// and content.
func sendRequest(w p2p.MsgWriter, msgcode, reqID uint64, data interface{}) error {
	type req struct {
		ReqID uint64
		Data  interface{}
	}
	return p2p.Send(w, msgcode, &req{reqID, data})
}

func (p *serverPeer) sendRequest(msgcode, reqID uint64, data interface{}, amount int) error {
	p.sentRequest(reqID, uint32(msgcode), uint32(amount))
	return sendRequest(p.rw, msgcode, reqID, data)
}

// requestHeadersByHash fetches a batch of blocks' headers corresponding to the
// specified header query, based on the hash of an origin block.
func (p *serverPeer) requestHeadersByHash(reqID uint64, origin common.Hash, amount int, skip int, reverse bool) error {
	p.Log().Debug("Fetching batch of headers", "count", amount, "fromhash", origin, "skip", skip, "reverse", reverse)
	return p.sendRequest(GetBlockHeadersMsg, reqID, &GetBlockHeadersData{Origin: hashOrNumber{Hash: origin}, Amount: uint64(amount), Skip: uint64(skip), Reverse: reverse}, amount)
}

// requestHeadersByNumber fetches a batch of blocks' headers corresponding to the
// specified header query, based on the number of an origin block.
func (p *serverPeer) requestHeadersByNumber(reqID, origin uint64, amount int, skip int, reverse bool) error {
	p.Log().Debug("Fetching batch of headers", "count", amount, "fromnum", origin, "skip", skip, "reverse", reverse)
	return p.sendRequest(GetBlockHeadersMsg, reqID, &GetBlockHeadersData{Origin: hashOrNumber{Number: origin}, Amount: uint64(amount), Skip: uint64(skip), Reverse: reverse}, amount)
}

// requestBodies fetches a batch of blocks' bodies corresponding to the hashes
// specified.
func (p *serverPeer) requestBodies(reqID uint64, hashes []common.Hash) error {
	p.Log().Debug("Fetching batch of block bodies", "count", len(hashes))
	return p.sendRequest(GetBlockBodiesMsg, reqID, hashes, len(hashes))
}

// requestCode fetches a batch of arbitrary data from a node's known state
// data, corresponding to the specified hashes.
func (p *serverPeer) requestCode(reqID uint64, reqs []CodeReq) error {
	p.Log().Debug("Fetching batch of codes", "count", len(reqs))
	return p.sendRequest(GetCodeMsg, reqID, reqs, len(reqs))
}

// requestReceipts fetches a batch of transaction receipts from a remote node.
func (p *serverPeer) requestReceipts(reqID uint64, hashes []common.Hash) error {
	p.Log().Debug("Fetching batch of receipts", "count", len(hashes))
	return p.sendRequest(GetReceiptsMsg, reqID, hashes, len(hashes))
}

// requestProofs fetches a batch of merkle proofs from a remote node.
func (p *serverPeer) requestProofs(reqID uint64, reqs []ProofReq) error {
	p.Log().Debug("Fetching batch of proofs", "count", len(reqs))
	return p.sendRequest(GetProofsV2Msg, reqID, reqs, len(reqs))
}

// requestHelperTrieProofs fetches a batch of HelperTrie merkle proofs from a remote node.
func (p *serverPeer) requestHelperTrieProofs(reqID uint64, reqs []HelperTrieReq) error {
	p.Log().Debug("Fetching batch of HelperTrie proofs", "count", len(reqs))
	return p.sendRequest(GetHelperTrieProofsMsg, reqID, reqs, len(reqs))
}

// requestTxStatus fetches a batch of transaction status records from a remote node.
func (p *serverPeer) requestTxStatus(reqID uint64, txHashes []common.Hash) error {
	p.Log().Debug("Requesting transaction status", "count", len(txHashes))
	return p.sendRequest(GetTxStatusMsg, reqID, txHashes, len(txHashes))
}

// sendTxs creates a reply with a batch of transactions to be added to the remote transaction pool.
func (p *serverPeer) sendTxs(reqID uint64, amount int, txs rlp.RawValue) error {
	p.Log().Debug("Sending batch of transactions", "amount", amount, "size", len(txs))
	sizeFactor := (len(txs) + txSizeCostLimit/2) / txSizeCostLimit
	if sizeFactor > amount {
		amount = sizeFactor
	}
	return p.sendRequest(SendTxV2Msg, reqID, txs, amount)
}

// waitBefore implements distPeer interface
func (p *serverPeer) waitBefore(maxCost uint64) (time.Duration, float64) {
	return p.fcServer.CanSend(maxCost)
}

// getRequestCost returns an estimated request cost according to the flow control
// rules negotiated between the server and the client.
func (p *serverPeer) getRequestCost(msgcode uint64, amount int) uint64 {
	p.lock.RLock()
	defer p.lock.RUnlock()

	costs := p.fcCosts[msgcode]
	if costs == nil {
		return 0
	}
	cost := costs.baseCost + costs.reqCost*uint64(amount)
	if cost > p.fcParams.BufLimit {
		cost = p.fcParams.BufLimit
	}
	return cost
}

// getTxRelayCost returns an estimated relay cost according to the flow control
// rules negotiated between the server and the client.
func (p *serverPeer) getTxRelayCost(amount, size int) uint64 {
	p.lock.RLock()
	defer p.lock.RUnlock()

	costs := p.fcCosts[SendTxV2Msg]
	if costs == nil {
		return 0
	}
	cost := costs.baseCost + costs.reqCost*uint64(amount)
	sizeCost := costs.baseCost + costs.reqCost*uint64(size)/txSizeCostLimit
	if sizeCost > cost {
		cost = sizeCost
	}
	if cost > p.fcParams.BufLimit {
		cost = p.fcParams.BufLimit
	}
	return cost
}

// HasBlock checks if the peer has a given block
func (p *serverPeer) HasBlock(hash common.Hash, number uint64, hasState bool) bool {
	p.lock.RLock()
	defer p.lock.RUnlock()

	if p.hasBlockHook != nil {
		return p.hasBlockHook(hash, number, hasState)
	}
	head := p.headInfo.Number
	var since, recent uint64
	if hasState {
		since = p.stateSince
		recent = p.stateRecent
	} else {
		since = p.chainSince
		recent = p.chainRecent
	}
	return head >= number && number >= since && (recent == 0 || number+recent+4 > head)
}

// updateFlowControl updates the flow control parameters belonging to the server
// node if the announced key/value set contains relevant fields
func (p *serverPeer) updateFlowControl(update keyValueMap) {
	p.lock.Lock()
	defer p.lock.Unlock()

	// If any of the flow control params is nil, refuse to update.
	var params flowcontrol.ServerParams
	if update.get("flowControl/BL", &params.BufLimit) == nil && update.get("flowControl/MRR", &params.MinRecharge) == nil {
		// todo can light client set a minimal acceptable flow control params?
		p.fcParams = params
		p.fcServer.UpdateParams(params)
	}
	var MRC RequestCostList
	if update.get("flowControl/MRC", &MRC) == nil {
		costUpdate := MRC.decode(ProtocolLengths[uint(p.version)])
		for code, cost := range costUpdate {
			p.fcCosts[code] = cost
		}
	}
}

// updateHead updates the head information based on the announcement from
// the peer.
func (p *serverPeer) updateHead(hash common.Hash, number uint64, td *big.Int) {
	p.lock.Lock()
	defer p.lock.Unlock()

	p.headInfo = blockInfo{Hash: hash, Number: number, Td: td}
}

// Handshake executes the les protocol handshake, negotiating version number,
// network IDs and genesis blocks.
func (p *serverPeer) Handshake(genesis common.Hash, forkid forkid.ID, forkFilter forkid.Filter) error {
	// Note: there is no need to share local head with a server but older servers still
	// require these fields so we announce zero values.
	return p.handshake(common.Big0, common.Hash{}, 0, genesis, forkid, forkFilter, func(lists *keyValueList) {
		// Add some client-specific handshake fields
		//
		// Enable signed announcement randomly even the server is not trusted.
		p.announceType = announceTypeSimple
		if p.trusted {
			p.announceType = announceTypeSigned
		}
		*lists = (*lists).add("announceType", p.announceType)
	}, func(recv keyValueMap) error {
		var (
			rHash common.Hash
			rNum  uint64
			rTd   *big.Int
		)
		if err := recv.get("headTd", &rTd); err != nil {
			return err
		}
		if err := recv.get("headHash", &rHash); err != nil {
			return err
		}
		if err := recv.get("headNum", &rNum); err != nil {
			return err
		}
		p.headInfo = blockInfo{Hash: rHash, Number: rNum, Td: rTd}
		if recv.get("serveChainSince", &p.chainSince) != nil {
			p.onlyAnnounce = true
		}
		if recv.get("serveRecentChain", &p.chainRecent) != nil {
			p.chainRecent = 0
		}
		if recv.get("serveStateSince", &p.stateSince) != nil {
			p.onlyAnnounce = true
		}
		if recv.get("serveRecentState", &p.stateRecent) != nil {
			p.stateRecent = 0
		}
		if recv.get("txRelay", nil) != nil {
			p.onlyAnnounce = true
		}
		if p.version >= lpv4 {
			var recentTx uint
			if err := recv.get("recentTxLookup", &recentTx); err != nil {
				return err
			}
			p.txHistory = uint64(recentTx)
		} else {
			// The weak assumption is held here that legacy les server(les2,3)
			// has unlimited transaction history. The les serving in these legacy
			// versions is disabled if the transaction is unindexed.
			p.txHistory = txIndexUnlimited
		}
		if p.onlyAnnounce && !p.trusted {
			return errResp(ErrUselessPeer, "peer cannot serve requests")
		}
		// Parse flow control handshake packet.
		var sParams flowcontrol.ServerParams
		if err := recv.get("flowControl/BL", &sParams.BufLimit); err != nil {
			return err
		}
		if err := recv.get("flowControl/MRR", &sParams.MinRecharge); err != nil {
			return err
		}
		var MRC RequestCostList
		if err := recv.get("flowControl/MRC", &MRC); err != nil {
			return err
		}
		p.fcParams = sParams
		p.fcServer = flowcontrol.NewServerNode(sParams, &mclock.System{})
		p.fcCosts = MRC.decode(ProtocolLengths[uint(p.version)])

		if !p.onlyAnnounce {
			for msgCode := range reqAvgTimeCost {
				if p.fcCosts[msgCode] == nil {
					return errResp(ErrUselessPeer, "peer does not support message %d", msgCode)
				}
			}
		}
		return nil
	})
}

// setValueTracker sets the value tracker references for connected servers. Note that the
// references should be removed upon disconnection by setValueTracker(nil, nil).
func (p *serverPeer) setValueTracker(nvt *vfc.NodeValueTracker) {
	p.vtLock.Lock()
	p.nodeValueTracker = nvt
	if nvt != nil {
		p.sentReqs = make(map[uint64]sentReqEntry)
	} else {
		p.sentReqs = nil
	}
	p.vtLock.Unlock()
}

// updateVtParams updates the server's price table in the value tracker.
func (p *serverPeer) updateVtParams() {
	p.vtLock.Lock()
	defer p.vtLock.Unlock()

	if p.nodeValueTracker == nil {
		return
	}
	reqCosts := make([]uint64, len(requestList))
	for code, costs := range p.fcCosts {
		if m, ok := requestMapping[uint32(code)]; ok {
			reqCosts[m.first] = costs.baseCost + costs.reqCost
			if m.rest != -1 {
				reqCosts[m.rest] = costs.reqCost
			}
		}
	}
	p.nodeValueTracker.UpdateCosts(reqCosts)
}

// sentReqEntry remembers sent requests and their sending times
type sentReqEntry struct {
	reqType, amount uint32
	at              mclock.AbsTime
}

// sentRequest marks a request sent at the current moment to this server.
func (p *serverPeer) sentRequest(id uint64, reqType, amount uint32) {
	p.vtLock.Lock()
	if p.sentReqs != nil {
		p.sentReqs[id] = sentReqEntry{reqType, amount, mclock.Now()}
	}
	p.vtLock.Unlock()
}

// answeredRequest marks a request answered at the current moment by this server.
func (p *serverPeer) answeredRequest(id uint64) {
	p.vtLock.Lock()
	if p.sentReqs == nil {
		p.vtLock.Unlock()
		return
	}
	e, ok := p.sentReqs[id]
	delete(p.sentReqs, id)
	nvt := p.nodeValueTracker
	p.vtLock.Unlock()
	if !ok {
		return
	}
	var (
		vtReqs   [2]vfc.ServedRequest
		reqCount int
	)
	m := requestMapping[e.reqType]
	if m.rest == -1 || e.amount <= 1 {
		reqCount = 1
		vtReqs[0] = vfc.ServedRequest{ReqType: uint32(m.first), Amount: e.amount}
	} else {
		reqCount = 2
		vtReqs[0] = vfc.ServedRequest{ReqType: uint32(m.first), Amount: 1}
		vtReqs[1] = vfc.ServedRequest{ReqType: uint32(m.rest), Amount: e.amount - 1}
	}
	dt := time.Duration(mclock.Now() - e.at)
	nvt.Served(vtReqs[:reqCount], dt)
}

// clientPeer represents each node to which the les server is connected.
// The node here refers to the light client.
type clientPeer struct {
	peerCommons

	// responseLock ensures that responses are queued in the same order as
	// RequestProcessed is called
	responseLock  sync.Mutex
	responseCount uint64 // Counter to generate an unique id for request processing.

	balance vfs.ConnectedBalance

	// invalidLock is used for protecting invalidCount.
	invalidLock  sync.RWMutex
	invalidCount utils.LinearExpiredValue // Counter the invalid request the client peer has made.

	capacity uint64
	// lastAnnounce is the last broadcast created by the server; may be newer than the last head
	// sent to the specific client (stored in headInfo) if capacity is zero. In this case the
	// latest head is sent when the client gains non-zero capacity.
	lastAnnounce announceData

	connectedAt mclock.AbsTime
	server      bool
	errCh       chan error
	fcClient    *flowcontrol.ClientNode // Server side mirror token bucket.
}

func newClientPeer(version int, network uint64, p *p2p.Peer, rw p2p.MsgReadWriter) *clientPeer {
	return &clientPeer{
		peerCommons: peerCommons{
			Peer:      p,
			rw:        rw,
			id:        p.ID().String(),
			version:   version,
			network:   network,
			sendQueue: utils.NewExecQueue(100),
			closeCh:   make(chan struct{}),
		},
		invalidCount: utils.LinearExpiredValue{Rate: mclock.AbsTime(time.Hour)},
		errCh:        make(chan error, 1),
	}
}

// FreeClientId returns a string identifier for the peer. Multiple peers with
// the same identifier can not be connected in free mode simultaneously.
func (p *clientPeer) FreeClientId() string {
	if addr, ok := p.RemoteAddr().(*net.TCPAddr); ok {
		if addr.IP.IsLoopback() {
			// using peer id instead of loopback ip address allows multiple free
			// connections from local machine to own server
			return p.id
		} else {
			return addr.IP.String()
		}
	}
	return p.id
}

// sendStop notifies the client about being in frozen state
func (p *clientPeer) sendStop() error {
	return p2p.Send(p.rw, StopMsg, struct{}{})
}

// sendResume notifies the client about getting out of frozen state
func (p *clientPeer) sendResume(bv uint64) error {
	return p2p.Send(p.rw, ResumeMsg, bv)
}

// freeze temporarily puts the client in a frozen state which means all unprocessed
// and subsequent requests are dropped. Unfreezing happens automatically after a short
// time if the client's buffer value is at least in the slightly positive region.
// The client is also notified about being frozen/unfrozen with a Stop/Resume message.
func (p *clientPeer) freeze() {
	if p.version < lpv3 {
		// if Stop/Resume is not supported then just drop the peer after setting
		// its frozen status permanently
		p.frozen.Store(true)
		p.Peer.Disconnect(p2p.DiscUselessPeer)
		return
	}
	if !p.frozen.Swap(true) {
		go func() {
			p.sendStop()
			time.Sleep(freezeTimeBase + time.Duration(rand.Int63n(int64(freezeTimeRandom))))
			for {
				bufValue, bufLimit := p.fcClient.BufferStatus()
				if bufLimit == 0 {
					return
				}
				if bufValue <= bufLimit/8 {
					time.Sleep(freezeCheckPeriod)
					continue
				}
				p.frozen.Store(false)
				p.sendResume(bufValue)
				return
			}
		}()
	}
}

// reply struct represents a reply with the actual data already RLP encoded and
// only the bv (buffer value) missing. This allows the serving mechanism to
// calculate the bv value which depends on the data size before sending the reply.
type reply struct {
	w              p2p.MsgWriter
	msgcode, reqID uint64
	data           rlp.RawValue
}

// send sends the reply with the calculated buffer value
func (r *reply) send(bv uint64) error {
	type resp struct {
		ReqID, BV uint64
		Data      rlp.RawValue
	}
	return p2p.Send(r.w, r.msgcode, &resp{r.reqID, bv, r.data})
}

// size returns the RLP encoded size of the message data
func (r *reply) size() uint32 {
	return uint32(len(r.data))
}

// replyBlockHeaders creates a reply with a batch of block headers
func (p *clientPeer) replyBlockHeaders(reqID uint64, headers []*types.Header) *reply {
	data, _ := rlp.EncodeToBytes(headers)
	return &reply{p.rw, BlockHeadersMsg, reqID, data}
}

// replyBlockBodiesRLP creates a reply with a batch of block contents from
// an already RLP encoded format.
func (p *clientPeer) replyBlockBodiesRLP(reqID uint64, bodies []rlp.RawValue) *reply {
	data, _ := rlp.EncodeToBytes(bodies)
	return &reply{p.rw, BlockBodiesMsg, reqID, data}
}

// replyCode creates a reply with a batch of arbitrary internal data, corresponding to the
// hashes requested.
func (p *clientPeer) replyCode(reqID uint64, codes [][]byte) *reply {
	data, _ := rlp.EncodeToBytes(codes)
	return &reply{p.rw, CodeMsg, reqID, data}
}

// replyReceiptsRLP creates a reply with a batch of transaction receipts, corresponding to the
// ones requested from an already RLP encoded format.
func (p *clientPeer) replyReceiptsRLP(reqID uint64, receipts []rlp.RawValue) *reply {
	data, _ := rlp.EncodeToBytes(receipts)
	return &reply{p.rw, ReceiptsMsg, reqID, data}
}

// replyProofsV2 creates a reply with a batch of merkle proofs, corresponding to the ones requested.
func (p *clientPeer) replyProofsV2(reqID uint64, proofs trienode.ProofList) *reply {
	data, _ := rlp.EncodeToBytes(proofs)
	return &reply{p.rw, ProofsV2Msg, reqID, data}
}

// replyHelperTrieProofs creates a reply with a batch of HelperTrie proofs, corresponding to the ones requested.
func (p *clientPeer) replyHelperTrieProofs(reqID uint64, resp HelperTrieResps) *reply {
	data, _ := rlp.EncodeToBytes(resp)
	return &reply{p.rw, HelperTrieProofsMsg, reqID, data}
}

// replyTxStatus creates a reply with a batch of transaction status records, corresponding to the ones requested.
func (p *clientPeer) replyTxStatus(reqID uint64, stats []light.TxStatus) *reply {
	data, _ := rlp.EncodeToBytes(stats)
	return &reply{p.rw, TxStatusMsg, reqID, data}
}

// sendAnnounce announces the availability of a number of blocks through
// a hash notification.
func (p *clientPeer) sendAnnounce(request announceData) error {
	return p2p.Send(p.rw, AnnounceMsg, request)
}

// InactiveAllowance implements vfs.clientPeer
func (p *clientPeer) InactiveAllowance() time.Duration {
	return 0 // will return more than zero for les/5 clients
}

// getCapacity returns the current capacity of the peer
func (p *clientPeer) getCapacity() uint64 {
	p.lock.RLock()
	defer p.lock.RUnlock()

	return p.capacity
}

// UpdateCapacity updates the request serving capacity assigned to a given client
// and also sends an announcement about the updated flow control parameters.
// Note: UpdateCapacity implements vfs.clientPeer and should not block. The requested
// parameter is true if the callback was initiated by ClientPool.SetCapacity on the given peer.
func (p *clientPeer) UpdateCapacity(newCap uint64, requested bool) {
	p.lock.Lock()
	defer p.lock.Unlock()

	if newCap != p.fcParams.MinRecharge {
		p.fcParams = flowcontrol.ServerParams{MinRecharge: newCap, BufLimit: newCap * bufLimitRatio}
		p.fcClient.UpdateParams(p.fcParams)
		var kvList keyValueList
		kvList = kvList.add("flowControl/MRR", newCap)
		kvList = kvList.add("flowControl/BL", newCap*bufLimitRatio)
		p.queueSend(func() { p.sendAnnounce(announceData{Update: kvList}) })
	}

	if p.capacity == 0 && newCap != 0 {
		p.sendLastAnnounce()
	}
	p.capacity = newCap
}

// announceOrStore sends the given head announcement to the client if the client is
// active (capacity != 0) and the same announcement hasn't been sent before. If the
// client is inactive the announcement is stored and sent later if the client is
// activated again.
func (p *clientPeer) announceOrStore(announce announceData) {
	p.lock.Lock()
	defer p.lock.Unlock()

	p.lastAnnounce = announce
	if p.capacity != 0 {
		p.sendLastAnnounce()
	}
}

// announce sends the given head announcement to the client if it hasn't been sent before
func (p *clientPeer) sendLastAnnounce() {
	if p.lastAnnounce.Td == nil {
		return
	}
	if p.headInfo.Td == nil || p.lastAnnounce.Td.Cmp(p.headInfo.Td) > 0 {
		if !p.queueSend(func() { p.sendAnnounce(p.lastAnnounce) }) {
			p.Log().Debug("Dropped announcement because queue is full", "number", p.lastAnnounce.Number, "hash", p.lastAnnounce.Hash)
		} else {
			p.Log().Debug("Sent announcement", "number", p.lastAnnounce.Number, "hash", p.lastAnnounce.Hash)
		}
		p.headInfo = blockInfo{Hash: p.lastAnnounce.Hash, Number: p.lastAnnounce.Number, Td: p.lastAnnounce.Td}
	}
}

// Handshake executes the les protocol handshake, negotiating version number,
// network IDs, difficulties, head and genesis blocks.
func (p *clientPeer) Handshake(td *big.Int, head common.Hash, headNum uint64, genesis common.Hash, forkID forkid.ID, forkFilter forkid.Filter, server *LesServer) error {
	recentTx := server.handler.blockchain.TxLookupLimit()
	if recentTx != txIndexUnlimited {
		if recentTx < blockSafetyMargin {
			recentTx = txIndexDisabled
		} else {
			recentTx -= blockSafetyMargin - txIndexRecentOffset
		}
	}
	if recentTx != txIndexUnlimited && p.version < lpv4 {
		return errors.New("Cannot serve old clients without a complete tx index")
	}
	// Note: clientPeer.headInfo should contain the last head announced to the client by us.
	// The values announced in the handshake are dummy values for compatibility reasons and should be ignored.
	p.headInfo = blockInfo{Hash: head, Number: headNum, Td: td}
	return p.handshake(td, head, headNum, genesis, forkID, forkFilter, func(lists *keyValueList) {
		// Add some information which services server can offer.
		*lists = (*lists).add("serveHeaders", nil)
		*lists = (*lists).add("serveChainSince", uint64(0))
		*lists = (*lists).add("serveStateSince", uint64(0))

		// If local ethereum node is running in archive mode, advertise ourselves we have
		// all version state data. Otherwise only recent state is available.
		stateRecent := uint64(core.TriesInMemory - blockSafetyMargin)
		if server.archiveMode {
			stateRecent = 0
		}
		*lists = (*lists).add("serveRecentState", stateRecent)
		*lists = (*lists).add("txRelay", nil)
		if p.version >= lpv4 {
			*lists = (*lists).add("recentTxLookup", recentTx)
		}
		*lists = (*lists).add("flowControl/BL", server.defParams.BufLimit)
		*lists = (*lists).add("flowControl/MRR", server.defParams.MinRecharge)

		var costList RequestCostList
		if server.costTracker.testCostList != nil {
			costList = server.costTracker.testCostList
		} else {
			costList = server.costTracker.makeCostList(server.costTracker.globalFactor())
		}
		*lists = (*lists).add("flowControl/MRC", costList)
		p.fcCosts = costList.decode(ProtocolLengths[uint(p.version)])
		p.fcParams = server.defParams
	}, func(recv keyValueMap) error {
		p.server = recv.get("flowControl/MRR", nil) == nil
		if p.server {
			p.announceType = announceTypeNone // connected to another server, send no messages
		} else {
			if recv.get("announceType", &p.announceType) != nil {
				// set default announceType on server side
				p.announceType = announceTypeSimple
			}
		}
		return nil
	})
}

func (p *clientPeer) bumpInvalid() {
	p.invalidLock.Lock()
	p.invalidCount.Add(1, mclock.Now())
	p.invalidLock.Unlock()
}

func (p *clientPeer) getInvalid() uint64 {
	p.invalidLock.RLock()
	defer p.invalidLock.RUnlock()
	return p.invalidCount.Value(mclock.Now())
}

// Disconnect implements vfs.clientPeer
func (p *clientPeer) Disconnect() {
	p.Peer.Disconnect(p2p.DiscRequested)
}

// serverPeerSubscriber is an interface to notify services about added or
// removed server peers
type serverPeerSubscriber interface {
	registerPeer(*serverPeer)
	unregisterPeer(*serverPeer)
}

// serverPeerSet represents the set of active server peers currently
// participating in the Light Ethereum sub-protocol.
type serverPeerSet struct {
	peers map[string]*serverPeer
	// subscribers is a batch of subscribers and peerset will notify
	// these subscribers when the peerset changes(new server peer is
	// added or removed)
	subscribers []serverPeerSubscriber
	closed      bool
	lock        sync.RWMutex
}

// newServerPeerSet creates a new peer set to track the active server peers.
func newServerPeerSet() *serverPeerSet {
	return &serverPeerSet{peers: make(map[string]*serverPeer)}
}

// subscribe adds a service to be notified about added or removed
// peers and also register all active peers into the given service.
func (ps *serverPeerSet) subscribe(sub serverPeerSubscriber) {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	ps.subscribers = append(ps.subscribers, sub)
	for _, p := range ps.peers {
		sub.registerPeer(p)
	}
}

// register adds a new server peer into the set, or returns an error if the
// peer is already known.
func (ps *serverPeerSet) register(peer *serverPeer) error {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	if ps.closed {
		return errClosed
	}
	if _, exist := ps.peers[peer.id]; exist {
		return errAlreadyRegistered
	}
	ps.peers[peer.id] = peer
	for _, sub := range ps.subscribers {
		sub.registerPeer(peer)
	}
	return nil
}

// unregister removes a remote peer from the active set, disabling any further
// actions to/from that particular entity. It also initiates disconnection at
// the networking layer.
func (ps *serverPeerSet) unregister(id string) error {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	p, ok := ps.peers[id]
	if !ok {
		return errNotRegistered
	}
	delete(ps.peers, id)
	for _, sub := range ps.subscribers {
		sub.unregisterPeer(p)
	}
	p.Peer.Disconnect(p2p.DiscRequested)
	return nil
}

// ids returns a list of all registered peer IDs
func (ps *serverPeerSet) ids() []string {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	var ids []string
	for id := range ps.peers {
		ids = append(ids, id)
	}
	return ids
}

// peer retrieves the registered peer with the given id.
func (ps *serverPeerSet) peer(id string) *serverPeer {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	return ps.peers[id]
}

// len returns if the current number of peers in the set.
func (ps *serverPeerSet) len() int {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	return len(ps.peers)
}

// allServerPeers returns all server peers in a list.
func (ps *serverPeerSet) allPeers() []*serverPeer {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	list := make([]*serverPeer, 0, len(ps.peers))
	for _, p := range ps.peers {
		list = append(list, p)
	}
	return list
}

// close disconnects all peers. No new peers can be registered
// after close has returned.
func (ps *serverPeerSet) close() {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	for _, p := range ps.peers {
		p.Disconnect(p2p.DiscQuitting)
	}
	ps.closed = true
}

// clientPeerSet represents the set of active client peers currently
// participating in the Light Ethereum sub-protocol.
type clientPeerSet struct {
	peers  map[enode.ID]*clientPeer
	lock   sync.RWMutex
	closed bool

	privateKey                   *ecdsa.PrivateKey
	lastAnnounce, signedAnnounce announceData
}

// newClientPeerSet creates a new peer set to track the client peers.
func newClientPeerSet() *clientPeerSet {
	return &clientPeerSet{peers: make(map[enode.ID]*clientPeer)}
}

// register adds a new peer into the peer set, or returns an error if the
// peer is already known.
func (ps *clientPeerSet) register(peer *clientPeer) error {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	if ps.closed {
		return errClosed
	}
	if _, exist := ps.peers[peer.ID()]; exist {
		return errAlreadyRegistered
	}
	ps.peers[peer.ID()] = peer
	ps.announceOrStore(peer)
	return nil
}

// unregister removes a remote peer from the peer set, disabling any further
// actions to/from that particular entity. It also initiates disconnection
// at the networking layer.
func (ps *clientPeerSet) unregister(id enode.ID) error {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	p, ok := ps.peers[id]
	if !ok {
		return errNotRegistered
	}
	delete(ps.peers, id)
	p.Peer.Disconnect(p2p.DiscRequested)
	return nil
}

// ids returns a list of all registered peer IDs
func (ps *clientPeerSet) ids() []enode.ID {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	var ids []enode.ID
	for id := range ps.peers {
		ids = append(ids, id)
	}
	return ids
}

// peer retrieves the registered peer with the given id.
func (ps *clientPeerSet) peer(id enode.ID) *clientPeer {
	ps.lock.RLock()
	defer ps.lock.RUnlock()

	return ps.peers[id]
}

// setSignerKey sets the signer key for signed announcements. Should be called before
// starting the protocol handler.
func (ps *clientPeerSet) setSignerKey(privateKey *ecdsa.PrivateKey) {
	ps.privateKey = privateKey
}

// broadcast sends the given announcements to all active peers
func (ps *clientPeerSet) broadcast(announce announceData) {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	ps.lastAnnounce = announce
	for _, peer := range ps.peers {
		ps.announceOrStore(peer)
	}
}

// announceOrStore sends the requested type of announcement to the given peer or stores
// it for later if the peer is inactive (capacity == 0).
func (ps *clientPeerSet) announceOrStore(p *clientPeer) {
	if ps.lastAnnounce.Td == nil {
		return
	}
	switch p.announceType {
	case announceTypeSimple:
		p.announceOrStore(ps.lastAnnounce)
	case announceTypeSigned:
		if ps.signedAnnounce.Hash != ps.lastAnnounce.Hash {
			ps.signedAnnounce = ps.lastAnnounce
			ps.signedAnnounce.sign(ps.privateKey)
		}
		p.announceOrStore(ps.signedAnnounce)
	}
}

// close disconnects all peers. No new peers can be registered
// after close has returned.
func (ps *clientPeerSet) close() {
	ps.lock.Lock()
	defer ps.lock.Unlock()

	for _, p := range ps.peers {
		p.Peer.Disconnect(p2p.DiscQuitting)
	}
	ps.closed = true
}

// serverSet is a special set which contains all connected les servers.
// Les servers will also be discovered by discovery protocol because they
// also run the LES protocol. We can't drop them although they are useless
// for us(server) but for other protocols(e.g. ETH) upon the devp2p they
// may be useful.
type serverSet struct {
	lock   sync.Mutex
	set    map[string]*clientPeer
	closed bool
}

func newServerSet() *serverSet {
	return &serverSet{set: make(map[string]*clientPeer)}
}

func (s *serverSet) register(peer *clientPeer) error {
	s.lock.Lock()
	defer s.lock.Unlock()

	if s.closed {
		return errClosed
	}
	if _, exist := s.set[peer.id]; exist {
		return errAlreadyRegistered
	}
	s.set[peer.id] = peer
	return nil
}

func (s *serverSet) unregister(peer *clientPeer) error {
	s.lock.Lock()
	defer s.lock.Unlock()

	if s.closed {
		return errClosed
	}
	if _, exist := s.set[peer.id]; !exist {
		return errNotRegistered
	}
	delete(s.set, peer.id)
	peer.Peer.Disconnect(p2p.DiscQuitting)
	return nil
}

func (s *serverSet) close() {
	s.lock.Lock()
	defer s.lock.Unlock()

	for _, p := range s.set {
		p.Peer.Disconnect(p2p.DiscQuitting)
	}
	s.closed = true
}
