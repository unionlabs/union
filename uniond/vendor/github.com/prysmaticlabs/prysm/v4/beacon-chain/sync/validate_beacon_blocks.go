package sync

import (
	"context"
	"fmt"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

var (
	ErrOptimisticParent    = errors.New("parent of the block is optimistic")
	errRejectCommitmentLen = errors.New("[REJECT] The length of KZG commitments is less than or equal to the limitation defined in Consensus Layer")
)

// validateBeaconBlockPubSub checks that the incoming block has a valid BLS signature.
// Blocks that have already been seen are ignored. If the BLS signature is any valid signature,
// this method rebroadcasts the message.
func (s *Service) validateBeaconBlockPubSub(ctx context.Context, pid peer.ID, msg *pubsub.Message) (pubsub.ValidationResult, error) {
	receivedTime := prysmTime.Now()
	// Validation runs on publish (not just subscriptions), so we should approve any message from
	// ourselves.
	if pid == s.cfg.p2p.PeerID() {
		return pubsub.ValidationAccept, nil
	}

	// We should not attempt to process blocks until fully synced, but propagation is OK.
	if s.cfg.initialSync.Syncing() {
		return pubsub.ValidationIgnore, nil
	}

	ctx, span := trace.StartSpan(ctx, "sync.validateBeaconBlockPubSub")
	defer span.End()

	m, err := s.decodePubsubMessage(msg)
	if err != nil {
		tracing.AnnotateError(span, err)
		return pubsub.ValidationReject, errors.Wrap(err, "Could not decode message")
	}

	s.validateBlockLock.Lock()
	defer s.validateBlockLock.Unlock()

	blk, ok := m.(interfaces.ReadOnlySignedBeaconBlock)
	if !ok {
		return pubsub.ValidationReject, errors.New("msg is not ethpb.ReadOnlySignedBeaconBlock")
	}

	if blk.IsNil() || blk.Block().IsNil() {
		return pubsub.ValidationReject, errors.New("block.Block is nil")
	}

	// Broadcast the block on a feed to notify other services in the beacon node
	// of a received block (even if it does not process correctly through a state transition).
	s.cfg.blockNotifier.BlockFeed().Send(&feed.Event{
		Type: blockfeed.ReceivedBlock,
		Data: &blockfeed.ReceivedBlockData{
			SignedBlock: blk,
		},
	})

	if features.Get().EnableSlasher {
		// Feed the block header to slasher if enabled. This action
		// is done in the background to avoid adding more load to this critical code path.
		go func() {
			blockHeader, err := interfaces.SignedBeaconBlockHeaderFromBlockInterface(blk)
			if err != nil {
				log.WithError(err).WithField("blockSlot", blk.Block().Slot()).Warn("Could not extract block header")
				return
			}
			s.cfg.slasherBlockHeadersFeed.Send(blockHeader)
		}()
	}

	if err := validateDenebBeaconBlock(blk.Block()); err != nil {
		return pubsub.ValidationReject, err
	}

	// Verify the block is the first block received for the proposer for the slot.
	if s.hasSeenBlockIndexSlot(blk.Block().Slot(), blk.Block().ProposerIndex()) {
		return pubsub.ValidationIgnore, nil
	}

	blockRoot, err := blk.Block().HashTreeRoot()
	if err != nil {
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Ignored block")
		return pubsub.ValidationIgnore, nil
	}
	if s.cfg.beaconDB.HasBlock(ctx, blockRoot) {
		return pubsub.ValidationIgnore, nil
	}
	// Check if parent is a bad block and then reject the block.
	if s.hasBadBlock(blk.Block().ParentRoot()) {
		s.setBadBlock(ctx, blockRoot)
		err := fmt.Errorf("received block with root %#x that has an invalid parent %#x", blockRoot, blk.Block().ParentRoot())
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Received block with an invalid parent")
		return pubsub.ValidationReject, err
	}

	s.pendingQueueLock.RLock()
	if s.seenPendingBlocks[blockRoot] {
		s.pendingQueueLock.RUnlock()
		return pubsub.ValidationIgnore, nil
	}
	s.pendingQueueLock.RUnlock()

	// Be lenient in handling early blocks. Instead of discarding blocks arriving later than
	// MAXIMUM_GOSSIP_CLOCK_DISPARITY in future, we tolerate blocks arriving at max two slots
	// earlier (SECONDS_PER_SLOT * 2 seconds). Queue such blocks and process them at the right slot.
	genesisTime := uint64(s.cfg.clock.GenesisTime().Unix())
	if err := slots.VerifyTime(genesisTime, blk.Block().Slot(), earlyBlockProcessingTolerance); err != nil {
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Ignored block: could not verify slot time")
		return pubsub.ValidationIgnore, nil
	}

	// Add metrics for block arrival time subtracts slot start time.
	if err := captureArrivalTimeMetric(genesisTime, blk.Block().Slot()); err != nil {
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Ignored block: could not capture arrival time metric")
		return pubsub.ValidationIgnore, nil
	}

	cp := s.cfg.chain.FinalizedCheckpt()
	startSlot, err := slots.EpochStart(cp.Epoch)
	if err != nil {
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Ignored block: could not calculate epoch start slot")
		return pubsub.ValidationIgnore, nil
	}
	if startSlot >= blk.Block().Slot() {
		err := fmt.Errorf("finalized slot %d greater or equal to block slot %d", startSlot, blk.Block().Slot())
		log.WithFields(getBlockFields(blk)).Debug(err)
		return pubsub.ValidationIgnore, err
	}

	// Process the block if the clock jitter is less than MAXIMUM_GOSSIP_CLOCK_DISPARITY.
	// Otherwise queue it for processing in the right slot.
	if isBlockQueueable(genesisTime, blk.Block().Slot(), receivedTime) {
		if res, err := s.verifyPendingBlockSignature(ctx, blk, blockRoot); err != nil {
			log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not verify block signature")
			return res, err
		}
		s.pendingQueueLock.Lock()
		if err := s.insertBlockToPendingQueue(blk.Block().Slot(), blk, blockRoot); err != nil {
			s.pendingQueueLock.Unlock()
			log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not insert block to pending queue")
			return pubsub.ValidationIgnore, err
		}
		s.pendingQueueLock.Unlock()
		err := fmt.Errorf("early block, with current slot %d < block slot %d", s.cfg.clock.CurrentSlot(), blk.Block().Slot())
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not process early block")
		return pubsub.ValidationIgnore, err
	}

	// Handle block when the parent is unknown.
	if !s.cfg.chain.HasBlock(ctx, blk.Block().ParentRoot()) {
		if res, err := s.verifyPendingBlockSignature(ctx, blk, blockRoot); err != nil {
			log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not verify block signature")
			return res, err
		}
		s.pendingQueueLock.Lock()
		if err := s.insertBlockToPendingQueue(blk.Block().Slot(), blk, blockRoot); err != nil {
			s.pendingQueueLock.Unlock()
			log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not insert block to pending queue")
			return pubsub.ValidationIgnore, err
		}
		s.pendingQueueLock.Unlock()
		err := errors.Errorf("unknown parent for block with slot %d and parent root %#x", blk.Block().Slot(), blk.Block().ParentRoot())
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not identify parent for block")
		return pubsub.ValidationIgnore, err
	}

	err = s.validateBeaconBlock(ctx, blk, blockRoot)
	if err != nil {
		// If the parent is optimistic, process the block as usual
		// This also does not penalize a peer which sends optimistic blocks
		if !errors.Is(ErrOptimisticParent, err) {
			log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not validate beacon block")
			return pubsub.ValidationReject, err
		}
	}

	// Record attribute of valid block.
	span.AddAttributes(trace.Int64Attribute("slotInEpoch", int64(blk.Block().Slot()%params.BeaconConfig().SlotsPerEpoch)))
	blkPb, err := blk.Proto()
	if err != nil {
		log.WithError(err).WithFields(getBlockFields(blk)).Debug("Could not convert beacon block to protobuf type")
		return pubsub.ValidationIgnore, err
	}
	msg.ValidatorData = blkPb // Used in downstream subscriber

	// Log the arrival time of the accepted block
	graffiti := blk.Block().Body().Graffiti()
	startTime, err := slots.ToTime(genesisTime, blk.Block().Slot())
	logFields := logrus.Fields{
		"blockSlot":     blk.Block().Slot(),
		"proposerIndex": blk.Block().ProposerIndex(),
		"graffiti":      string(graffiti[:]),
	}
	if err != nil {
		log.WithError(err).WithFields(logFields).Warn("Received block, could not report timing information.")
		return pubsub.ValidationAccept, nil
	}
	sinceSlotStartTime := receivedTime.Sub(startTime)
	validationTime := prysmTime.Now().Sub(receivedTime)
	logFields["sinceSlotStartTime"] = sinceSlotStartTime
	logFields["validationTime"] = validationTime
	log.WithFields(logFields).Debug("Received block")

	blockArrivalGossipSummary.Observe(float64(sinceSlotStartTime))
	blockVerificationGossipSummary.Observe(float64(validationTime))
	return pubsub.ValidationAccept, nil
}

func (s *Service) validateBeaconBlock(ctx context.Context, blk interfaces.ReadOnlySignedBeaconBlock, blockRoot [32]byte) error {
	ctx, span := trace.StartSpan(ctx, "sync.validateBeaconBlock")
	defer span.End()

	if err := validateDenebBeaconBlock(blk.Block()); err != nil {
		s.setBadBlock(ctx, blockRoot)
		return err
	}

	parentState, err := s.validatePhase0Block(ctx, blk, blockRoot)
	if err != nil {
		return err
	}

	if err = s.validateBellatrixBeaconBlock(ctx, parentState, blk.Block()); err != nil {
		if errors.Is(err, ErrOptimisticParent) {
			return err
		}
		// for other kinds of errors, set this block as a bad block.
		s.setBadBlock(ctx, blockRoot)
		return err
	}
	return nil
}

// Validates beacon block according to phase 0 validity conditions.
// - Checks that the parent is in our forkchoice tree.
// - Validates that the proposer signature is valid.
// - Validates that the proposer index is valid.
func (s *Service) validatePhase0Block(ctx context.Context, blk interfaces.ReadOnlySignedBeaconBlock, blockRoot [32]byte) (state.BeaconState, error) {
	if !s.cfg.chain.InForkchoice(blk.Block().ParentRoot()) {
		s.setBadBlock(ctx, blockRoot)
		return nil, blockchain.ErrNotDescendantOfFinalized
	}

	parentState, err := s.cfg.stateGen.StateByRoot(ctx, blk.Block().ParentRoot())
	if err != nil {
		return nil, err
	}

	if err := blocks.VerifyBlockSignatureUsingCurrentFork(parentState, blk); err != nil {
		return nil, err
	}
	// In the event the block is more than an epoch ahead from its
	// parent state, we have to advance the state forward.
	parentRoot := blk.Block().ParentRoot()
	parentState, err = transition.ProcessSlotsUsingNextSlotCache(ctx, parentState, parentRoot[:], blk.Block().Slot())
	if err != nil {
		return nil, err
	}
	idx, err := helpers.BeaconProposerIndex(ctx, parentState)
	if err != nil {
		return nil, err
	}
	if blk.Block().ProposerIndex() != idx {
		s.setBadBlock(ctx, blockRoot)
		return nil, errors.New("incorrect proposer index")
	}
	return parentState, nil
}

func validateDenebBeaconBlock(blk interfaces.ReadOnlyBeaconBlock) error {
	if blk.Version() < version.Deneb {
		return nil
	}
	commits, err := blk.Body().BlobKzgCommitments()
	if err != nil {
		return errors.New("unable to read commitments from deneb block")
	}
	// [REJECT] The length of KZG commitments is less than or equal to the limitation defined in Consensus Layer
	// -- i.e. validate that len(body.signed_beacon_block.message.blob_kzg_commitments) <= MAX_BLOBS_PER_BLOCK
	if len(commits) > fieldparams.MaxBlobsPerBlock {
		return errors.Wrapf(errRejectCommitmentLen, "%d > %d", len(commits), fieldparams.MaxBlobsPerBlock)
	}
	return nil
}

// validateBellatrixBeaconBlock validates the block for the Bellatrix fork.
// spec code:
//
//	If the execution is enabled for the block -- i.e. is_execution_enabled(state, block.body) then validate the following:
//	   [REJECT] The block's execution payload timestamp is correct with respect to the slot --
//	   i.e. execution_payload.timestamp == compute_timestamp_at_slot(state, block.slot).
//
//	   If execution_payload verification of block's parent by an execution node is not complete:
//	      [REJECT] The block's parent (defined by block.parent_root) passes all validation (excluding execution
//	       node verification of the block.body.execution_payload).
//	   otherwise:
//	      [IGNORE] The block's parent (defined by block.parent_root) passes all validation (including execution
//	       node verification of the block.body.execution_payload).
func (s *Service) validateBellatrixBeaconBlock(ctx context.Context, parentState state.BeaconState, blk interfaces.ReadOnlyBeaconBlock) error {
	// Error if block and state are not the same version
	if parentState.Version() != blk.Version() {
		return errors.New("block and state are not the same version")
	}

	body := blk.Body()
	executionEnabled, err := blocks.IsExecutionEnabled(parentState, body)
	if err != nil {
		return err
	}
	if !executionEnabled {
		return nil
	}

	t, err := slots.ToTime(parentState.GenesisTime(), blk.Slot())
	if err != nil {
		return err
	}
	payload, err := body.Execution()
	if err != nil {
		return err
	}
	if payload.IsNil() {
		return errors.New("execution payload is nil")
	}
	if payload.Timestamp() != uint64(t.Unix()) {
		return errors.New("incorrect timestamp")
	}

	isParentOptimistic, err := s.cfg.chain.IsOptimisticForRoot(ctx, blk.ParentRoot())
	if err != nil {
		return err
	}
	if isParentOptimistic {
		return ErrOptimisticParent
	}
	return nil
}

// Verifies the signature of the pending block with respect to the current head state.
func (s *Service) verifyPendingBlockSignature(ctx context.Context, blk interfaces.ReadOnlySignedBeaconBlock, blkRoot [32]byte) (pubsub.ValidationResult, error) {
	roState, err := s.cfg.chain.HeadStateReadOnly(ctx)
	if err != nil {
		return pubsub.ValidationIgnore, err
	}
	// Ignore block in the event of non-existent proposer.
	_, err = roState.ValidatorAtIndex(blk.Block().ProposerIndex())
	if err != nil {
		return pubsub.ValidationIgnore, err
	}
	if err := blocks.VerifyBlockSignatureUsingCurrentFork(roState, blk); err != nil {
		s.setBadBlock(ctx, blkRoot)
		return pubsub.ValidationReject, err
	}
	return pubsub.ValidationAccept, nil
}

// Returns true if the block is not the first block proposed for the proposer for the slot.
func (s *Service) hasSeenBlockIndexSlot(slot primitives.Slot, proposerIdx primitives.ValidatorIndex) bool {
	s.seenBlockLock.RLock()
	defer s.seenBlockLock.RUnlock()
	b := append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(proposerIdx))...)
	_, seen := s.seenBlockCache.Get(string(b))
	return seen
}

// Set block proposer index and slot as seen for incoming blocks.
func (s *Service) setSeenBlockIndexSlot(slot primitives.Slot, proposerIdx primitives.ValidatorIndex) {
	s.seenBlockLock.Lock()
	defer s.seenBlockLock.Unlock()
	b := append(bytesutil.Bytes32(uint64(slot)), bytesutil.Bytes32(uint64(proposerIdx))...)
	s.seenBlockCache.Add(string(b), true)
}

// Returns true if the block is marked as a bad block.
func (s *Service) hasBadBlock(root [32]byte) bool {
	s.badBlockLock.RLock()
	defer s.badBlockLock.RUnlock()
	_, seen := s.badBlockCache.Get(string(root[:]))
	return seen
}

// Set bad block in the cache.
func (s *Service) setBadBlock(ctx context.Context, root [32]byte) {
	s.badBlockLock.Lock()
	defer s.badBlockLock.Unlock()
	if ctx.Err() != nil { // Do not mark block as bad if it was due to context error.
		return
	}
	log.WithField("root", fmt.Sprintf("%#x", root)).Debug("Inserting in invalid block cache")
	s.badBlockCache.Add(string(root[:]), true)
}

// This captures metrics for block arrival time by subtracts slot start time.
func captureArrivalTimeMetric(genesisTime uint64, currentSlot primitives.Slot) error {
	startTime, err := slots.ToTime(genesisTime, currentSlot)
	if err != nil {
		return err
	}
	ms := prysmTime.Now().Sub(startTime) / time.Millisecond
	arrivalBlockPropagationHistogram.Observe(float64(ms))
	arrivalBlockPropagationGauge.Set(float64(ms))

	return nil
}

// isBlockQueueable checks if the slot_time in the block is greater than
// current_time +  MAXIMUM_GOSSIP_CLOCK_DISPARITY. in short, this function
// returns true if the corresponding block should be queued and false if
// the block should be processed immediately.
func isBlockQueueable(genesisTime uint64, slot primitives.Slot, receivedTime time.Time) bool {
	slotTime, err := slots.ToTime(genesisTime, slot)
	if err != nil {
		return false
	}

	currentTimeWithDisparity := receivedTime.Add(params.BeaconConfig().MaximumGossipClockDisparityDuration())
	return currentTimeWithDisparity.Unix() < slotTime.Unix()
}

func getBlockFields(b interfaces.ReadOnlySignedBeaconBlock) logrus.Fields {
	if consensusblocks.BeaconBlockIsNil(b) != nil {
		return logrus.Fields{}
	}
	graffiti := b.Block().Body().Graffiti()
	return logrus.Fields{
		"slot":          b.Block().Slot(),
		"proposerIndex": b.Block().ProposerIndex(),
		"graffiti":      string(graffiti[:]),
		"version":       b.Block().Version(),
	}
}
