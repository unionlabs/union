package initialsync

import (
	"context"
	"fmt"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2pTypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	prysmsync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/verify"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	blocks2 "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/math"
	p2ppb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

const (

	// maxPendingRequests limits how many concurrent fetch request one can initiate.
	maxPendingRequests = 64
	// peersPercentagePerRequest caps percentage of peers to be used in a request.
	peersPercentagePerRequest = 0.75
	// handshakePollingInterval is a polling interval for checking the number of received handshakes.
	handshakePollingInterval = 5 * time.Second
	// peerLocksPollingInterval is a polling interval for checking if there are stale peer locks.
	peerLocksPollingInterval = 5 * time.Minute
	// peerLockMaxAge is maximum time before stale lock is purged.
	peerLockMaxAge = 60 * time.Minute
	// nonSkippedSlotsFullSearchEpochs how many epochs to check in full, before resorting to random
	// sampling of slots once per epoch
	nonSkippedSlotsFullSearchEpochs = 10
	// peerFilterCapacityWeight defines how peer's capacity affects peer's score. Provided as
	// percentage, i.e. 0.3 means capacity will determine 30% of peer's score.
	peerFilterCapacityWeight = 0.2
	// backtrackingMaxHops how many hops (during search for common ancestor in backtracking) to do
	// before giving up.
	backtrackingMaxHops = 128
)

var (
	errNoPeersAvailable      = errors.New("no peers available, waiting for reconnect")
	errFetcherCtxIsDone      = errors.New("fetcher's context is done, reinitialize")
	errSlotIsTooHigh         = errors.New("slot is higher than the finalized slot")
	errBlockAlreadyProcessed = errors.New("block is already processed")
	errParentDoesNotExist    = errors.New("beacon node doesn't have a parent in db with root")
	errNoPeersWithAltBlocks  = errors.New("no peers with alternative blocks found")
)

// Period to calculate expected limit for a single peer.
var blockLimiterPeriod = 30 * time.Second

// blocksFetcherConfig is a config to setup the block fetcher.
type blocksFetcherConfig struct {
	clock                    *startup.Clock
	ctxMap                   prysmsync.ContextByteVersions
	chain                    blockchainService
	p2p                      p2p.P2P
	db                       db.ReadOnlyDatabase
	peerFilterCapacityWeight float64
	mode                     syncMode
}

// blocksFetcher is a service to fetch chain data from peers.
// On an incoming requests, requested block range is evenly divided
// among available peers (for fair network load distribution).
type blocksFetcher struct {
	sync.Mutex
	ctx             context.Context
	cancel          context.CancelFunc
	rand            *rand.Rand
	chain           blockchainService
	clock           *startup.Clock
	ctxMap          prysmsync.ContextByteVersions
	p2p             p2p.P2P
	db              db.ReadOnlyDatabase
	blocksPerPeriod uint64
	rateLimiter     *leakybucket.Collector
	peerLocks       map[peer.ID]*peerLock
	fetchRequests   chan *fetchRequestParams
	fetchResponses  chan *fetchRequestResponse
	capacityWeight  float64       // how remaining capacity affects peer selection
	mode            syncMode      // allows to use fetcher in different sync scenarios
	quit            chan struct{} // termination notifier
}

// peerLock restricts fetcher actions on per peer basis. Currently, used for rate limiting.
type peerLock struct {
	sync.Mutex
	accessed time.Time
}

// fetchRequestParams holds parameters necessary to schedule a fetch request.
type fetchRequestParams struct {
	ctx   context.Context // if provided, it is used instead of global fetcher's context
	start primitives.Slot // starting slot
	count uint64          // how many slots to receive (fetcher may return fewer slots)
}

// fetchRequestResponse is a combined type to hold results of both successful executions and errors.
// Valid usage pattern will be to check whether result's `err` is nil, before using `blocks`.
type fetchRequestResponse struct {
	pid   peer.ID
	start primitives.Slot
	count uint64
	bwb   []blocks2.BlockWithROBlobs
	err   error
}

// newBlocksFetcher creates ready to use fetcher.
func newBlocksFetcher(ctx context.Context, cfg *blocksFetcherConfig) *blocksFetcher {
	blocksPerPeriod := flags.Get().BlockBatchLimit
	allowedBlocksBurst := flags.Get().BlockBatchLimitBurstFactor * flags.Get().BlockBatchLimit
	// Allow fetcher to go almost to the full burst capacity (less a single batch).
	rateLimiter := leakybucket.NewCollector(
		float64(blocksPerPeriod), int64(allowedBlocksBurst-blocksPerPeriod),
		blockLimiterPeriod, false /* deleteEmptyBuckets */)

	capacityWeight := cfg.peerFilterCapacityWeight
	if capacityWeight >= 1 {
		capacityWeight = peerFilterCapacityWeight
	}

	ctx, cancel := context.WithCancel(ctx)
	return &blocksFetcher{
		ctx:             ctx,
		cancel:          cancel,
		rand:            rand.NewGenerator(),
		chain:           cfg.chain,
		clock:           cfg.clock,
		ctxMap:          cfg.ctxMap,
		p2p:             cfg.p2p,
		db:              cfg.db,
		blocksPerPeriod: uint64(blocksPerPeriod),
		rateLimiter:     rateLimiter,
		peerLocks:       make(map[peer.ID]*peerLock),
		fetchRequests:   make(chan *fetchRequestParams, maxPendingRequests),
		fetchResponses:  make(chan *fetchRequestResponse, maxPendingRequests),
		capacityWeight:  capacityWeight,
		mode:            cfg.mode,
		quit:            make(chan struct{}),
	}
}

// start boots up the fetcher, which starts listening for incoming fetch requests.
func (f *blocksFetcher) start() error {
	select {
	case <-f.ctx.Done():
		return errFetcherCtxIsDone
	default:
		go f.loop()
		return nil
	}
}

// stop terminates all fetcher operations.
func (f *blocksFetcher) stop() {
	defer func() {
		if f.rateLimiter != nil {
			f.rateLimiter.Free()
			f.rateLimiter = nil
		}
	}()
	f.cancel()
	<-f.quit // make sure that loop() is done
}

// requestResponses exposes a channel into which fetcher pushes generated request responses.
func (f *blocksFetcher) requestResponses() <-chan *fetchRequestResponse {
	return f.fetchResponses
}

// loop is a main fetcher loop, listens for incoming requests/cancellations, forwards outgoing responses.
func (f *blocksFetcher) loop() {
	defer close(f.quit)

	// Wait for all loop's goroutines to finish, and safely release resources.
	wg := &sync.WaitGroup{}
	defer func() {
		wg.Wait()
		close(f.fetchResponses)
	}()

	// Periodically remove stale peer locks.
	go func() {
		ticker := time.NewTicker(peerLocksPollingInterval)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				f.removeStalePeerLocks(peerLockMaxAge)
			case <-f.ctx.Done():
				return
			}
		}
	}()

	// Main loop.
	for {
		// Make sure there is are available peers before processing requests.
		if _, err := f.waitForMinimumPeers(f.ctx); err != nil {
			log.Error(err)
		}

		select {
		case <-f.ctx.Done():
			log.Debug("Context closed, exiting goroutine (blocks fetcher)")
			return
		case req := <-f.fetchRequests:
			wg.Add(1)
			go func() {
				defer wg.Done()
				select {
				case <-f.ctx.Done():
				case f.fetchResponses <- f.handleRequest(req.ctx, req.start, req.count):
				}
			}()
		}
	}
}

// scheduleRequest adds request to incoming queue.
func (f *blocksFetcher) scheduleRequest(ctx context.Context, start primitives.Slot, count uint64) error {
	if ctx.Err() != nil {
		return ctx.Err()
	}

	request := &fetchRequestParams{
		ctx:   ctx,
		start: start,
		count: count,
	}
	select {
	case <-f.ctx.Done():
		return errFetcherCtxIsDone
	case f.fetchRequests <- request:
	}
	return nil
}

// handleRequest parses fetch request and forwards it to response builder.
func (f *blocksFetcher) handleRequest(ctx context.Context, start primitives.Slot, count uint64) *fetchRequestResponse {
	ctx, span := trace.StartSpan(ctx, "initialsync.handleRequest")
	defer span.End()

	response := &fetchRequestResponse{
		start: start,
		count: count,
		bwb:   []blocks2.BlockWithROBlobs{},
		err:   nil,
	}

	if ctx.Err() != nil {
		response.err = ctx.Err()
		return response
	}

	_, targetEpoch, peers := f.calculateHeadAndTargetEpochs()
	if len(peers) == 0 {
		response.err = errNoPeersAvailable
		return response
	}

	// Short circuit start far exceeding the highest finalized epoch in some infinite loop.
	if f.mode == modeStopOnFinalizedEpoch {
		highestFinalizedSlot := params.BeaconConfig().SlotsPerEpoch.Mul(uint64(targetEpoch + 1))
		if start > highestFinalizedSlot {
			response.err = fmt.Errorf("%w, slot: %d, highest finalized slot: %d",
				errSlotIsTooHigh, start, highestFinalizedSlot)
			return response
		}
	}

	response.bwb, response.pid, response.err = f.fetchBlocksFromPeer(ctx, start, count, peers)
	if response.err == nil {
		bwb, err := f.fetchBlobsFromPeer(ctx, response.bwb, response.pid)
		if err != nil {
			response.err = err
		}
		response.bwb = bwb
	}
	return response
}

// fetchBlocksFromPeer fetches blocks from a single randomly selected peer.
func (f *blocksFetcher) fetchBlocksFromPeer(
	ctx context.Context,
	start primitives.Slot, count uint64,
	peers []peer.ID,
) ([]blocks2.BlockWithROBlobs, peer.ID, error) {
	ctx, span := trace.StartSpan(ctx, "initialsync.fetchBlocksFromPeer")
	defer span.End()

	peers = f.filterPeers(ctx, peers, peersPercentagePerRequest)
	req := &p2ppb.BeaconBlocksByRangeRequest{
		StartSlot: start,
		Count:     count,
		Step:      1,
	}
	for i := 0; i < len(peers); i++ {
		p := peers[i]
		blocks, err := f.requestBlocks(ctx, req, p)
		if err != nil {
			log.WithField("peer", p).WithError(err).Debug("Could not request blocks by range from peer")
			continue
		}
		f.p2p.Peers().Scorers().BlockProviderScorer().Touch(p)
		robs, err := sortedBlockWithVerifiedBlobSlice(blocks)
		if err != nil {
			log.WithField("peer", p).WithError(err).Debug("invalid BeaconBlocksByRange response")
			continue
		}
		return robs, p, err
	}
	return nil, "", errNoPeersAvailable
}

func sortedBlockWithVerifiedBlobSlice(blocks []interfaces.ReadOnlySignedBeaconBlock) ([]blocks2.BlockWithROBlobs, error) {
	rb := make([]blocks2.BlockWithROBlobs, len(blocks))
	for i, b := range blocks {
		ro, err := blocks2.NewROBlock(b)
		if err != nil {
			return nil, err
		}
		rb[i] = blocks2.BlockWithROBlobs{Block: ro}
	}
	sort.Sort(blocks2.BlockWithROBlobsSlice(rb))
	return rb, nil
}

func blobRequest(bwb []blocks2.BlockWithROBlobs, blobWindowStart primitives.Slot) *p2ppb.BlobSidecarsByRangeRequest {
	if len(bwb) == 0 {
		return nil
	}
	lowest := lowestSlotNeedsBlob(blobWindowStart, bwb)
	if lowest == nil {
		return nil
	}
	highest := bwb[len(bwb)-1].Block.Block().Slot()
	return &p2ppb.BlobSidecarsByRangeRequest{
		StartSlot: *lowest,
		Count:     uint64(highest.SubSlot(*lowest)) + 1,
	}
}

func lowestSlotNeedsBlob(retentionStart primitives.Slot, bwb []blocks2.BlockWithROBlobs) *primitives.Slot {
	if len(bwb) == 0 {
		return nil
	}
	// Short-circuit if the highest block is before the deneb start epoch or retention period start.
	// This assumes blocks are sorted by sortedBlockWithVerifiedBlobSlice.
	// bwb is sorted by slot, so if the last element is outside the retention window, no blobs are needed.
	if bwb[len(bwb)-1].Block.Block().Slot() < retentionStart {
		return nil
	}
	for _, b := range bwb {
		slot := b.Block.Block().Slot()
		if slot < retentionStart {
			continue
		}
		commits, err := b.Block.Block().Body().BlobKzgCommitments()
		if err != nil || len(commits) == 0 {
			continue
		}
		return &slot
	}
	return nil
}

func sortBlobs(blobs []blocks.ROBlob) []blocks.ROBlob {
	sort.Slice(blobs, func(i, j int) bool {
		if blobs[i].Slot() == blobs[j].Slot() {
			return blobs[i].Index < blobs[j].Index
		}
		return blobs[i].Slot() < blobs[j].Slot()
	})

	return blobs
}

var errBlobVerification = errors.New("peer unable to serve aligned BlobSidecarsByRange and BeaconBlockSidecarsByRange responses")
var errMissingBlobsForBlockCommitments = errors.Wrap(errBlobVerification, "blobs unavailable for processing block with kzg commitments")

func verifyAndPopulateBlobs(bwb []blocks2.BlockWithROBlobs, blobs []blocks.ROBlob, blobWindowStart primitives.Slot) ([]blocks2.BlockWithROBlobs, error) {
	// Assumes bwb has already been sorted by sortedBlockWithVerifiedBlobSlice.
	blobs = sortBlobs(blobs)
	blobi := 0
	// Loop over all blocks, and each time a commitment is observed, advance the index into the blob slice.
	// The assumption is that the blob slice contains a value for every commitment in the blocks it is based on,
	// correctly ordered by slot and blob index.
	for i, bb := range bwb {
		block := bb.Block.Block()
		if block.Slot() < blobWindowStart {
			continue
		}
		commits, err := block.Body().BlobKzgCommitments()
		if err != nil {
			if errors.Is(err, consensus_types.ErrUnsupportedField) {
				log.
					WithField("block_slot", block.Slot()).
					WithField("retention_start", blobWindowStart).
					Warn("block with slot within blob retention period has version which does not support commitments")
				continue
			}
			return nil, err
		}
		bb.Blobs = make([]blocks.ROBlob, len(commits))
		for ci := range commits {
			// There are more expected commitments in this block, but we've run out of blobs from the response
			// (out-of-bound error guard).
			if blobi == len(blobs) {
				return nil, missingCommitError(bb.Block.Root(), bb.Block.Block().Slot(), commits[ci:])
			}
			bl := blobs[blobi]
			if err := verify.BlobAlignsWithBlock(bl, bb.Block); err != nil {
				return nil, err
			}
			bb.Blobs[ci] = bl
			blobi += 1
		}
		bwb[i] = bb
	}
	return bwb, nil
}

func missingCommitError(root [32]byte, slot primitives.Slot, missing [][]byte) error {
	missStr := make([]string, 0, len(missing))
	for k := range missing {
		missStr = append(missStr, fmt.Sprintf("%#x", k))
	}
	return errors.Wrapf(errMissingBlobsForBlockCommitments,
		"block root %#x at slot %d missing %d commitments %s", root, slot, len(missing), strings.Join(missStr, ","))
}

// fetchBlobsFromPeer fetches blocks from a single randomly selected peer.
func (f *blocksFetcher) fetchBlobsFromPeer(ctx context.Context, bwb []blocks2.BlockWithROBlobs, pid peer.ID) ([]blocks2.BlockWithROBlobs, error) {
	ctx, span := trace.StartSpan(ctx, "initialsync.fetchBlobsFromPeer")
	defer span.End()
	if slots.ToEpoch(f.clock.CurrentSlot()) < params.BeaconConfig().DenebForkEpoch {
		return bwb, nil
	}
	blobWindowStart, err := prysmsync.BlobsByRangeMinStartSlot(f.clock.CurrentSlot())
	if err != nil {
		return nil, err
	}
	// Construct request message based on observed interval of blocks in need of blobs.
	req := blobRequest(bwb, blobWindowStart)
	if req == nil {
		return bwb, nil
	}
	// Request blobs from the same peer that gave us the blob batch.
	blobs, err := f.requestBlobs(ctx, req, pid)
	if err != nil {
		return nil, errors.Wrap(err, "could not request blobs by range")
	}
	f.p2p.Peers().Scorers().BlockProviderScorer().Touch(pid)
	return verifyAndPopulateBlobs(bwb, blobs, blobWindowStart)
}

// requestBlocks is a wrapper for handling BeaconBlocksByRangeRequest requests/streams.
func (f *blocksFetcher) requestBlocks(
	ctx context.Context,
	req *p2ppb.BeaconBlocksByRangeRequest,
	pid peer.ID,
) ([]interfaces.ReadOnlySignedBeaconBlock, error) {
	if ctx.Err() != nil {
		return nil, ctx.Err()
	}
	l := f.peerLock(pid)
	l.Lock()
	log.WithFields(logrus.Fields{
		"peer":     pid,
		"start":    req.StartSlot,
		"count":    req.Count,
		"step":     req.Step,
		"capacity": f.rateLimiter.Remaining(pid.String()),
		"score":    f.p2p.Peers().Scorers().BlockProviderScorer().FormatScorePretty(pid),
	}).Debug("Requesting blocks")
	if f.rateLimiter.Remaining(pid.String()) < int64(req.Count) {
		if err := f.waitForBandwidth(pid, req.Count); err != nil {
			l.Unlock()
			return nil, err
		}
	}
	f.rateLimiter.Add(pid.String(), int64(req.Count))
	l.Unlock()
	return prysmsync.SendBeaconBlocksByRangeRequest(ctx, f.chain, f.p2p, pid, req, nil)
}

func (f *blocksFetcher) requestBlobs(ctx context.Context, req *p2ppb.BlobSidecarsByRangeRequest, pid peer.ID) ([]blocks.ROBlob, error) {
	if ctx.Err() != nil {
		return nil, ctx.Err()
	}
	l := f.peerLock(pid)
	l.Lock()
	log.WithFields(logrus.Fields{
		"peer":     pid,
		"start":    req.StartSlot,
		"count":    req.Count,
		"capacity": f.rateLimiter.Remaining(pid.String()),
		"score":    f.p2p.Peers().Scorers().BlockProviderScorer().FormatScorePretty(pid),
	}).Debug("Requesting blobs")
	// We're intentionally abusing the block rate limit here, treating blob requests as if they were block requests.
	// Since blob requests take more bandwidth than blocks, we should improve how we account for the different kinds
	// of requests, more in proportion to the cost of serving them.
	if f.rateLimiter.Remaining(pid.String()) < int64(req.Count) {
		if err := f.waitForBandwidth(pid, req.Count); err != nil {
			l.Unlock()
			return nil, err
		}
	}
	f.rateLimiter.Add(pid.String(), int64(req.Count))
	l.Unlock()
	return prysmsync.SendBlobsByRangeRequest(ctx, f.clock, f.p2p, pid, f.ctxMap, req)
}

// requestBlocksByRoot is a wrapper for handling BeaconBlockByRootsReq requests/streams.
func (f *blocksFetcher) requestBlocksByRoot(
	ctx context.Context,
	req *p2pTypes.BeaconBlockByRootsReq,
	pid peer.ID,
) ([]interfaces.ReadOnlySignedBeaconBlock, error) {
	if ctx.Err() != nil {
		return nil, ctx.Err()
	}
	l := f.peerLock(pid)
	l.Lock()
	log.WithFields(logrus.Fields{
		"peer":     pid,
		"numRoots": len(*req),
		"capacity": f.rateLimiter.Remaining(pid.String()),
		"score":    f.p2p.Peers().Scorers().BlockProviderScorer().FormatScorePretty(pid),
	}).Debug("Requesting blocks (by roots)")
	if f.rateLimiter.Remaining(pid.String()) < int64(len(*req)) {
		if err := f.waitForBandwidth(pid, uint64(len(*req))); err != nil {
			l.Unlock()
			return nil, err
		}
	}
	f.rateLimiter.Add(pid.String(), int64(len(*req)))
	l.Unlock()

	return prysmsync.SendBeaconBlocksByRootRequest(ctx, f.chain, f.p2p, pid, req, nil)
}

// waitForBandwidth blocks up until peer's bandwidth is restored.
func (f *blocksFetcher) waitForBandwidth(pid peer.ID, count uint64) error {
	log.WithField("peer", pid).Debug("Slowing down for rate limit")
	rem := f.rateLimiter.Remaining(pid.String())
	if uint64(rem) >= count {
		// Exit early if we have sufficient capacity
		return nil
	}
	intCount, err := math.Int(count)
	if err != nil {
		return err
	}
	toWait := timeToWait(int64(intCount), rem, f.rateLimiter.Capacity(), f.rateLimiter.TillEmpty(pid.String()))
	timer := time.NewTimer(toWait)
	defer timer.Stop()
	select {
	case <-f.ctx.Done():
		return errFetcherCtxIsDone
	case <-timer.C:
		// Peer has gathered enough capacity to be polled again.
	}
	return nil
}

// Determine how long it will take for us to have the required number of blocks allowed by our rate limiter.
// We do this by calculating the duration till the rate limiter can request these blocks without exceeding
// the provided bandwidth limits per peer.
func timeToWait(wanted, rem, capacity int64, timeTillEmpty time.Duration) time.Duration {
	// Defensive check if we have more than enough blocks
	// to request from the peer.
	if rem >= wanted {
		return 0
	}
	// Handle edge case where capacity is equal to the remaining amount
	// of blocks. This also handles the impossible case in where remaining blocks
	// exceed the limiter's capacity.
	if capacity <= rem {
		return 0
	}
	blocksNeeded := wanted - rem
	currentNumBlks := capacity - rem
	expectedTime := int64(timeTillEmpty) * blocksNeeded / currentNumBlks
	return time.Duration(expectedTime)
}
