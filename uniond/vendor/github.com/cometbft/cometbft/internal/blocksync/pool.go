package blocksync

import (
	"errors"
	"fmt"
	"math"
	"sort"
	"time"

	flow "github.com/cometbft/cometbft/internal/flowrate"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

/*
eg, L = latency = 0.1s
	P = num peers = 10
	FN = num full nodes
	BS = 1kB block size
	CB = 1 Mbit/s = 128 kB/s
	CB/P = 12.8 kB
	B/S = CB/P/BS = 12.8 blocks/s

	12.8 * 0.1 = 1.28 blocks on conn
*/

const (
	maxPendingRequestsPerPeer = 20
	requestRetrySeconds       = 30

	// Minimum recv rate to ensure we're receiving blocks from a peer fast
	// enough. If a peer is not sending us data at least that rate, we consider
	// them to have timed out, and we disconnect.
	//
	// Based on the experiments with [Osmosis](https://osmosis.zone/), the
	// minimum rate could be as high as 500 KB/s. However, we're setting it to
	// 128 KB/s for now to be conservative.
	minRecvRate = 128 * 1024 // 128 KB/s

	// peerConnWait is the time that must have elapsed since the pool routine
	// was created before we start making requests. This is to give the peer
	// routine time to connect to peers.
	peerConnWait = 3 * time.Second

	// If we're within minBlocksForSingleRequest blocks of the pool's height, we
	// send 2 parallel requests to 2 peers for the same block. If we're further
	// away, we send a single request.
	minBlocksForSingleRequest = 50
)

var (
	requestInterval = 10 * time.Millisecond // timeout between requests
	peerTimeout     = 15 * time.Second      // not const so we can override with tests
)

/*
	Peers self report their heights when we join the block pool.
	Starting from our latest pool.height, we request blocks
	in sequence from peers that reported higher heights than ours.
	Every so often we ask peers what height they're on so we can keep going.

	Requests are continuously made for blocks of higher heights until
	the limit is reached. If most of the requests have no available peers, and we
	are not at peer limits, we can probably switch to consensus reactor.
*/

// BlockPool keeps track of the block sync peers, block requests and block responses.
type BlockPool struct {
	service.BaseService
	startTime   time.Time
	startHeight int64

	mtx cmtsync.Mutex
	// block requests
	requesters map[int64]*bpRequester
	height     int64 // the lowest key in requesters.
	// peers
	peers         map[p2p.ID]*bpPeer
	bannedPeers   map[p2p.ID]time.Time
	sortedPeers   []*bpPeer // sorted by curRate, highest first
	maxPeerHeight int64     // the biggest reported height

	requestsCh chan<- BlockRequest
	errorsCh   chan<- peerError
}

// NewBlockPool returns a new BlockPool with the height equal to start. Block
// requests and errors will be sent to requestsCh and errorsCh accordingly.
func NewBlockPool(start int64, requestsCh chan<- BlockRequest, errorsCh chan<- peerError) *BlockPool {
	bp := &BlockPool{
		peers:       make(map[p2p.ID]*bpPeer),
		bannedPeers: make(map[p2p.ID]time.Time),
		requesters:  make(map[int64]*bpRequester),
		height:      start,
		startHeight: start,

		requestsCh: requestsCh,
		errorsCh:   errorsCh,
	}
	bp.BaseService = *service.NewBaseService(nil, "BlockPool", bp)
	return bp
}

// OnStart implements service.Service by spawning requesters routine and recording
// pool's start time.
func (pool *BlockPool) OnStart() error {
	pool.startTime = time.Now()
	go pool.makeRequestersRoutine()
	return nil
}

func (pool *BlockPool) makeRequestersRoutine() {
	for {
		if !pool.IsRunning() {
			return
		}

		// Check if we are within peerConnWait seconds of start time
		// This gives us some time to connect to peers before starting a wave of requests
		if time.Since(pool.startTime) < peerConnWait {
			// Calculate the duration to sleep until peerConnWait seconds have passed since pool.startTime
			sleepDuration := peerConnWait - time.Since(pool.startTime)
			time.Sleep(sleepDuration)
		}

		pool.mtx.Lock()
		var (
			maxRequestersCreated = len(pool.requesters) >= len(pool.peers)*maxPendingRequestsPerPeer

			nextHeight           = pool.height + int64(len(pool.requesters))
			maxPeerHeightReached = nextHeight > pool.maxPeerHeight
		)
		pool.mtx.Unlock()

		switch {
		case maxRequestersCreated: // If we have enough requesters, wait for them to finish.
			time.Sleep(requestInterval)
			pool.removeTimedoutPeers()
		case maxPeerHeightReached: // If we're caught up, wait for a bit so reactor could finish or a higher height is reported.
			time.Sleep(requestInterval)
		default:
			pool.makeNextRequester(nextHeight)
			// Sleep for a bit to make the requests more ordered.
			time.Sleep(requestInterval)
		}
	}
}

func (pool *BlockPool) removeTimedoutPeers() {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	for _, peer := range pool.peers {
		if !peer.didTimeout && peer.numPending > 0 {
			curRate := peer.recvMonitor.Status().CurRate
			// curRate can be 0 on start
			if curRate != 0 && curRate < minRecvRate {
				err := errors.New("peer is not sending us data fast enough")
				pool.sendError(err, peer.id)
				pool.Logger.Error("SendTimeout", "peer", peer.id,
					"reason", err,
					"curRate", fmt.Sprintf("%d KB/s", curRate/1024),
					"minRate", fmt.Sprintf("%d KB/s", minRecvRate/1024))
				peer.didTimeout = true
			}

			peer.curRate = curRate
		}

		if peer.didTimeout {
			pool.removePeer(peer.id)
		}
	}

	for peerID := range pool.bannedPeers {
		if !pool.isPeerBanned(peerID) {
			delete(pool.bannedPeers, peerID)
		}
	}

	pool.sortPeers()
}

// IsCaughtUp returns true if this node is caught up, false - otherwise.
// TODO: relax conditions, prevent abuse.
func (pool *BlockPool) IsCaughtUp() (isCaughtUp bool, height, maxPeerHeight int64) {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	// Need at least 1 peer to be considered caught up.
	if len(pool.peers) == 0 {
		pool.Logger.Debug("Blockpool has no peers")
		return false, pool.height, pool.maxPeerHeight
	}

	// Some conditions to determine if we're caught up.
	// Ensures we've either received a block or waited some amount of time,
	// and that we're synced to the highest known height.
	// Note we use maxPeerHeight - 1 because to sync block H requires block H+1
	// to verify the LastCommit.
	receivedBlockOrTimedOut := pool.height > 0 || time.Since(pool.startTime) > 5*time.Second
	ourChainIsLongestAmongPeers := pool.maxPeerHeight == 0 || pool.height >= (pool.maxPeerHeight-1)
	isCaughtUp = receivedBlockOrTimedOut && ourChainIsLongestAmongPeers
	return isCaughtUp, pool.height, pool.maxPeerHeight
}

// PeekTwoBlocks returns blocks at pool.height and pool.height+1. We need to
// see the second block's Commit to validate the first block. So we peek two
// blocks at a time. We return an extended commit, containing vote extensions
// and their associated signatures, as this is critical to consensus in ABCI++
// as we switch from block sync to consensus mode.
//
// The caller will verify the commit.
func (pool *BlockPool) PeekTwoBlocks() (first, second *types.Block, firstExtCommit *types.ExtendedCommit) {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	if r := pool.requesters[pool.height]; r != nil {
		first = r.getBlock()
		firstExtCommit = r.getExtendedCommit()
	}
	if r := pool.requesters[pool.height+1]; r != nil {
		second = r.getBlock()
	}
	return first, second, firstExtCommit
}

// PopRequest removes the requester at pool.height and increments pool.height.
func (pool *BlockPool) PopRequest() {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	r := pool.requesters[pool.height]
	if r == nil {
		panic(fmt.Sprintf("Expected requester to pop, got nothing at height %v", pool.height))
	}

	if err := r.Stop(); err != nil {
		pool.Logger.Error("Error stopping requester", "err", err)
	}
	delete(pool.requesters, pool.height)
	pool.height++

	// Notify the next minBlocksForSingleRequest requesters about new height, so
	// they can potentially request a block from the second peer.
	for i := int64(0); i < minBlocksForSingleRequest && i < int64(len(pool.requesters)); i++ {
		pool.requesters[pool.height+i].newHeight(pool.height)
	}
}

// RemovePeerAndRedoAllPeerRequests retries the request at the given height and
// all the requests made to the same peer. The peer is removed from the pool.
// Returns the ID of the removed peer.
func (pool *BlockPool) RemovePeerAndRedoAllPeerRequests(height int64) p2p.ID {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	request := pool.requesters[height]
	peerID := request.gotBlockFromPeerID()
	// RemovePeer will redo all requesters associated with this peer.
	pool.removePeer(peerID)
	pool.banPeer(peerID)
	return peerID
}

// RedoRequestFrom retries the request at the given height. It does not remove the
// peer.
func (pool *BlockPool) RedoRequestFrom(height int64, peerID p2p.ID) {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	if requester, ok := pool.requesters[height]; ok { // If we requested this block
		if requester.didRequestFrom(peerID) { // From this specific peer
			requester.redo(peerID)
		}
	}
}

// Deprecated: use RemovePeerAndRedoAllPeerRequests instead.
func (pool *BlockPool) RedoRequest(height int64) p2p.ID {
	return pool.RemovePeerAndRedoAllPeerRequests(height)
}

// AddBlock validates that the block comes from the peer it was expected from
// and calls the requester to store it.
//
// This requires an extended commit at the same height as the supplied block -
// the block contains the last commit, but we need the latest commit in case we
// need to switch over from block sync to consensus at this height. If the
// height of the extended commit and the height of the block do not match, we
// do not add the block and return an error.
func (pool *BlockPool) AddBlock(peerID p2p.ID, block *types.Block, extCommit *types.ExtendedCommit, blockSize int) error {
	if extCommit != nil && block.Height != extCommit.Height {
		err := fmt.Errorf("block height %d != extCommit height %d", block.Height, extCommit.Height)
		// Peer sent us an invalid block => remove it.
		pool.sendError(err, peerID)
		return err
	}

	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	requester := pool.requesters[block.Height]
	if requester == nil {
		// Because we're issuing 2nd requests for closer blocks, it's possible to
		// receive a block we've already processed from a second peer. Hence, we
		// can't punish it. But if the peer sent us a block we clearly didn't
		// request, we disconnect.
		if block.Height > pool.height || block.Height < pool.startHeight {
			err := fmt.Errorf("peer sent us block #%d we didn't expect (current height: %d, start height: %d)",
				block.Height, pool.height, pool.startHeight)
			pool.sendError(err, peerID)
			return err
		}

		return fmt.Errorf("got an already committed block #%d (possibly from the slow peer %s)", block.Height, peerID)
	}

	if !requester.setBlock(block, extCommit, peerID) {
		err := fmt.Errorf("requested block #%d from %v, not %s", block.Height, requester.requestedFrom(), peerID)
		pool.sendError(err, peerID)
		return err
	}

	peer := pool.peers[peerID]
	if peer != nil {
		peer.decrPending(blockSize)
	}

	return nil
}

// Height returns the pool's height.
func (pool *BlockPool) Height() int64 {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()
	return pool.height
}

// MaxPeerHeight returns the highest reported height.
func (pool *BlockPool) MaxPeerHeight() int64 {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()
	return pool.maxPeerHeight
}

// SetPeerRange sets the peer's alleged blockchain base and height.
func (pool *BlockPool) SetPeerRange(peerID p2p.ID, base int64, height int64) {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	peer := pool.peers[peerID]
	if peer != nil {
		peer.base = base
		peer.height = height
	} else {
		if pool.isPeerBanned(peerID) {
			pool.Logger.Debug("Ignoring banned peer", peerID)
			return
		}
		peer = newBPPeer(pool, peerID, base, height)
		peer.setLogger(pool.Logger.With("peer", peerID))
		pool.peers[peerID] = peer
		// no need to sort because curRate is 0 at start.
		// just add to the beginning so it's picked first by pickIncrAvailablePeer.
		pool.sortedPeers = append([]*bpPeer{peer}, pool.sortedPeers...)
	}

	if height > pool.maxPeerHeight {
		pool.maxPeerHeight = height
	}
}

// RemovePeer removes the peer with peerID from the pool. If there's no peer
// with peerID, function is a no-op.
func (pool *BlockPool) RemovePeer(peerID p2p.ID) {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	pool.removePeer(peerID)
}

func (pool *BlockPool) removePeer(peerID p2p.ID) {
	for _, requester := range pool.requesters {
		if requester.didRequestFrom(peerID) {
			requester.redo(peerID)
		}
	}

	peer, ok := pool.peers[peerID]
	if ok {
		if peer.timeout != nil {
			peer.timeout.Stop()
		}

		delete(pool.peers, peerID)
		for i, p := range pool.sortedPeers {
			if p.id == peerID {
				pool.sortedPeers = append(pool.sortedPeers[:i], pool.sortedPeers[i+1:]...)
				break
			}
		}

		// Find a new peer with the biggest height and update maxPeerHeight if the
		// peer's height was the biggest.
		if peer.height == pool.maxPeerHeight {
			pool.updateMaxPeerHeight()
		}
	}
}

// If no peers are left, maxPeerHeight is set to 0.
func (pool *BlockPool) updateMaxPeerHeight() {
	var max int64
	for _, peer := range pool.peers {
		if peer.height > max {
			max = peer.height
		}
	}
	pool.maxPeerHeight = max
}

func (pool *BlockPool) isPeerBanned(peerID p2p.ID) bool {
	return cmttime.Since(pool.bannedPeers[peerID]) < time.Second*60
}

func (pool *BlockPool) banPeer(peerID p2p.ID) {
	pool.Logger.Debug("Banning peer", peerID)
	pool.bannedPeers[peerID] = cmttime.Now()
}

// Pick an available peer with the given height available.
// If no peers are available, returns nil.
func (pool *BlockPool) pickIncrAvailablePeer(height int64, excludePeerID p2p.ID) *bpPeer {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	for _, peer := range pool.sortedPeers {
		if peer.id == excludePeerID {
			continue
		}
		if peer.didTimeout {
			pool.removePeer(peer.id)
			continue
		}
		if peer.numPending >= maxPendingRequestsPerPeer {
			continue
		}
		if height < peer.base || height > peer.height {
			continue
		}
		peer.incrPending()
		return peer
	}

	return nil
}

// Sort peers by curRate, highest first.
//
// CONTRACT: pool.mtx must be locked.
func (pool *BlockPool) sortPeers() {
	sort.Slice(pool.sortedPeers, func(i, j int) bool {
		return pool.sortedPeers[i].curRate > pool.sortedPeers[j].curRate
	})
}

func (pool *BlockPool) makeNextRequester(nextHeight int64) {
	pool.mtx.Lock()
	request := newBPRequester(pool, nextHeight)
	pool.requesters[nextHeight] = request
	pool.mtx.Unlock()

	if err := request.Start(); err != nil {
		request.Logger.Error("Error starting request", "err", err)
	}
}

// thread-safe.
func (pool *BlockPool) sendRequest(height int64, peerID p2p.ID) {
	if !pool.IsRunning() {
		return
	}
	pool.requestsCh <- BlockRequest{height, peerID}
}

// thread-safe.
func (pool *BlockPool) sendError(err error, peerID p2p.ID) {
	if !pool.IsRunning() {
		return
	}
	pool.errorsCh <- peerError{err, peerID}
}

// for debugging purposes
//
//nolint:unused
func (pool *BlockPool) debug() string {
	pool.mtx.Lock()
	defer pool.mtx.Unlock()

	str := ""
	nextHeight := pool.height + int64(len(pool.requesters))
	for h := pool.height; h < nextHeight; h++ {
		if pool.requesters[h] == nil {
			str += fmt.Sprintf("H(%v):X ", h)
		} else {
			str += fmt.Sprintf("H(%v):", h)
			str += fmt.Sprintf("B?(%v) ", pool.requesters[h].block != nil)
			str += fmt.Sprintf("C?(%v) ", pool.requesters[h].extCommit != nil)
		}
	}
	return str
}

// -------------------------------------

type bpPeer struct {
	didTimeout  bool
	curRate     int64
	numPending  int32
	height      int64
	base        int64
	pool        *BlockPool
	id          p2p.ID
	recvMonitor *flow.Monitor

	timeout *time.Timer

	logger log.Logger
}

func newBPPeer(pool *BlockPool, peerID p2p.ID, base int64, height int64) *bpPeer {
	peer := &bpPeer{
		pool:       pool,
		id:         peerID,
		base:       base,
		height:     height,
		numPending: 0,
		logger:     log.NewNopLogger(),
	}
	return peer
}

func (peer *bpPeer) setLogger(l log.Logger) {
	peer.logger = l
}

func (peer *bpPeer) resetMonitor() {
	peer.recvMonitor = flow.New(time.Second, time.Second*40)
	initialValue := float64(minRecvRate) * math.E
	peer.recvMonitor.SetREMA(initialValue)
}

func (peer *bpPeer) resetTimeout() {
	if peer.timeout == nil {
		peer.timeout = time.AfterFunc(peerTimeout, peer.onTimeout)
	} else {
		peer.timeout.Reset(peerTimeout)
	}
}

func (peer *bpPeer) incrPending() {
	if peer.numPending == 0 {
		peer.resetMonitor()
		peer.resetTimeout()
	}
	peer.numPending++
}

func (peer *bpPeer) decrPending(recvSize int) {
	peer.numPending--
	if peer.numPending == 0 {
		peer.timeout.Stop()
	} else {
		peer.recvMonitor.Update(recvSize)
		peer.resetTimeout()
	}
}

func (peer *bpPeer) onTimeout() {
	peer.pool.mtx.Lock()
	defer peer.pool.mtx.Unlock()

	err := errors.New("peer did not send us anything")
	peer.pool.sendError(err, peer.id)
	peer.logger.Error("SendTimeout", "reason", err, "timeout", peerTimeout)
	peer.didTimeout = true
}

// -------------------------------------

// bpRequester requests a block from a peer.
//
// If the height is within minBlocksForSingleRequest blocks of the pool's
// height, it will send an additional request to another peer. This is to avoid
// a situation where blocksync is stuck because of a single slow peer. Note
// that it's okay to send a single request when the requested height is far
// from the pool's height. If the peer is slow, it will timeout and be replaced
// with another peer.
type bpRequester struct {
	service.BaseService

	pool        *BlockPool
	height      int64
	gotBlockCh  chan struct{}
	redoCh      chan p2p.ID // redo may got multiple messages, add peerId to identify repeat
	newHeightCh chan int64

	mtx          cmtsync.Mutex
	peerID       p2p.ID
	secondPeerID p2p.ID // alternative peer to request from (if close to pool's height)
	gotBlockFrom p2p.ID
	block        *types.Block
	extCommit    *types.ExtendedCommit
}

func newBPRequester(pool *BlockPool, height int64) *bpRequester {
	bpr := &bpRequester{
		pool:        pool,
		height:      height,
		gotBlockCh:  make(chan struct{}, 1),
		redoCh:      make(chan p2p.ID, 1),
		newHeightCh: make(chan int64, 1),

		peerID:       "",
		secondPeerID: "",
		block:        nil,
	}
	bpr.BaseService = *service.NewBaseService(nil, "bpRequester", bpr)
	return bpr
}

func (bpr *bpRequester) OnStart() error {
	go bpr.requestRoutine()
	return nil
}

// Returns true if the peer(s) match and block doesn't already exist.
func (bpr *bpRequester) setBlock(block *types.Block, extCommit *types.ExtendedCommit, peerID p2p.ID) bool {
	bpr.mtx.Lock()
	if bpr.peerID != peerID && bpr.secondPeerID != peerID {
		bpr.mtx.Unlock()
		return false
	}
	if bpr.block != nil {
		bpr.mtx.Unlock()
		return true // getting a block from both peers is not an error
	}

	bpr.block = block
	bpr.extCommit = extCommit
	bpr.gotBlockFrom = peerID
	bpr.mtx.Unlock()

	select {
	case bpr.gotBlockCh <- struct{}{}:
	default:
	}
	return true
}

func (bpr *bpRequester) getBlock() *types.Block {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()
	return bpr.block
}

func (bpr *bpRequester) getExtendedCommit() *types.ExtendedCommit {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()
	return bpr.extCommit
}

// Returns the IDs of peers we've requested a block from.
func (bpr *bpRequester) requestedFrom() []p2p.ID {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()
	peerIDs := make([]p2p.ID, 0, 2)
	if bpr.peerID != "" {
		peerIDs = append(peerIDs, bpr.peerID)
	}
	if bpr.secondPeerID != "" {
		peerIDs = append(peerIDs, bpr.secondPeerID)
	}
	return peerIDs
}

// Returns true if we've requested a block from the given peer.
func (bpr *bpRequester) didRequestFrom(peerID p2p.ID) bool {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()
	return bpr.peerID == peerID || bpr.secondPeerID == peerID
}

// Returns the ID of the peer who sent us the block.
func (bpr *bpRequester) gotBlockFromPeerID() p2p.ID {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()
	return bpr.gotBlockFrom
}

// Removes the block (IF we got it from the given peer) and resets the peer.
func (bpr *bpRequester) reset(peerID p2p.ID) (removedBlock bool) {
	bpr.mtx.Lock()
	defer bpr.mtx.Unlock()

	// Only remove the block if we got it from that peer.
	if bpr.gotBlockFrom == peerID {
		bpr.block = nil
		bpr.extCommit = nil
		bpr.gotBlockFrom = ""
		removedBlock = true
	}

	if bpr.peerID == peerID {
		bpr.peerID = ""
	} else {
		bpr.secondPeerID = ""
	}

	return removedBlock
}

// Tells bpRequester to pick another peer and try again.
// NOTE: Nonblocking, and does nothing if another redo
// was already requested.
func (bpr *bpRequester) redo(peerID p2p.ID) {
	select {
	case bpr.redoCh <- peerID:
	default:
	}
}

func (bpr *bpRequester) pickPeerAndSendRequest() {
	bpr.mtx.Lock()
	secondPeerID := bpr.secondPeerID
	bpr.mtx.Unlock()

	var peer *bpPeer
PICK_PEER_LOOP:
	for {
		if !bpr.IsRunning() || !bpr.pool.IsRunning() {
			return
		}
		peer = bpr.pool.pickIncrAvailablePeer(bpr.height, secondPeerID)
		if peer == nil {
			bpr.Logger.Debug("No peers currently available; will retry shortly", "height", bpr.height)
			time.Sleep(requestInterval)
			continue PICK_PEER_LOOP
		}
		break PICK_PEER_LOOP
	}
	bpr.mtx.Lock()
	bpr.peerID = peer.id
	bpr.mtx.Unlock()

	bpr.pool.sendRequest(bpr.height, peer.id)
}

// Picks a second peer and sends a request to it. If the second peer is already
// set, does nothing.
func (bpr *bpRequester) pickSecondPeerAndSendRequest() (picked bool) {
	bpr.mtx.Lock()
	if bpr.secondPeerID != "" {
		bpr.mtx.Unlock()
		return false
	}
	peerID := bpr.peerID
	bpr.mtx.Unlock()

	secondPeer := bpr.pool.pickIncrAvailablePeer(bpr.height, peerID)
	if secondPeer != nil {
		bpr.mtx.Lock()
		bpr.secondPeerID = secondPeer.id
		bpr.mtx.Unlock()

		bpr.pool.sendRequest(bpr.height, secondPeer.id)
		return true
	}

	return false
}

// Informs the requester of a new pool's height.
func (bpr *bpRequester) newHeight(height int64) {
	select {
	case bpr.newHeightCh <- height:
	default:
	}
}

// Responsible for making more requests as necessary
// Returns only when a block is found (e.g. AddBlock() is called).
func (bpr *bpRequester) requestRoutine() {
	gotBlock := false

OUTER_LOOP:
	for {
		bpr.pickPeerAndSendRequest()

		poolHeight := bpr.pool.Height()
		if bpr.height-poolHeight < minBlocksForSingleRequest {
			bpr.pickSecondPeerAndSendRequest()
		}

		retryTimer := time.NewTimer(requestRetrySeconds * time.Second)
		defer retryTimer.Stop()

		for {
			select {
			case <-bpr.pool.Quit():
				if err := bpr.Stop(); err != nil {
					bpr.Logger.Error("Error stopped requester", "err", err)
				}
				return
			case <-bpr.Quit():
				return
			case <-retryTimer.C:
				if !gotBlock {
					bpr.Logger.Debug("Retrying block request(s) after timeout", "height", bpr.height, "peer", bpr.peerID, "secondPeerID", bpr.secondPeerID)
					bpr.reset(bpr.peerID)
					bpr.reset(bpr.secondPeerID)
					continue OUTER_LOOP
				}
			case peerID := <-bpr.redoCh:
				if bpr.didRequestFrom(peerID) {
					removedBlock := bpr.reset(peerID)
					if removedBlock {
						gotBlock = false
					}
				}
				// If both peers returned NoBlockResponse or bad block, reschedule both
				// requests. If not, wait for the other peer.
				if len(bpr.requestedFrom()) == 0 {
					retryTimer.Stop()
					continue OUTER_LOOP
				}
			case newHeight := <-bpr.newHeightCh:
				if !gotBlock && bpr.height-newHeight < minBlocksForSingleRequest {
					// The operation is a noop if the second peer is already set. The cost is checking a mutex.
					//
					// If the second peer was just set, reset the retryTimer to give the
					// second peer a chance to respond.
					if picked := bpr.pickSecondPeerAndSendRequest(); picked {
						if !retryTimer.Stop() { //nolint:revive // suppress max-control-nesting linter
							<-retryTimer.C
						}
						retryTimer.Reset(requestRetrySeconds * time.Second)
					}
				}
			case <-bpr.gotBlockCh:
				gotBlock = true
				// We got a block!
				// Continue the for-loop and wait til Quit.
			}
		}
	}
}

// BlockRequest stores a block request identified by the block Height and the PeerID responsible for
// delivering the block.
type BlockRequest struct {
	Height int64
	PeerID p2p.ID
}
