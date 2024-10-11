package blockchain

import (
	"context"
	"fmt"
	"time"

	"github.com/pkg/errors"
	"go.opencensus.io/trace"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/das"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// A custom slot deadline for processing state slots in our cache.
const slotDeadline = 5 * time.Second

// A custom deadline for deposit trie insertion.
const depositDeadline = 20 * time.Second

// This defines size of the upper bound for initial sync block cache.
var initialSyncBlockCacheSize = uint64(2 * params.BeaconConfig().SlotsPerEpoch)

// postBlockProcessConfig is a structure that contains the data needed to
// process the beacon block after validating the state transition function
type postBlockProcessConfig struct {
	ctx            context.Context
	signed         interfaces.ReadOnlySignedBeaconBlock
	blockRoot      [32]byte
	headRoot       [32]byte
	postState      state.BeaconState
	isValidPayload bool
}

// postBlockProcess is called when a gossip block is received. This function performs
// several duties most importantly informing the engine if head was updated,
// saving the new head information to the blockchain package and
// handling attestations, slashings and similar included in the block.
func (s *Service) postBlockProcess(cfg *postBlockProcessConfig) error {
	ctx, span := trace.StartSpan(cfg.ctx, "blockChain.onBlock")
	defer span.End()
	cfg.ctx = ctx
	if err := consensusblocks.BeaconBlockIsNil(cfg.signed); err != nil {
		return invalidBlock{error: err}
	}
	startTime := time.Now()
	fcuArgs := &fcuConfig{}

	if s.inRegularSync() {
		defer s.handleSecondFCUCall(cfg, fcuArgs)
	}
	defer s.sendLightClientFeeds(cfg)
	defer s.sendStateFeedOnBlock(cfg)
	defer reportProcessingTime(startTime)
	defer reportAttestationInclusion(cfg.signed.Block())

	err := s.cfg.ForkChoiceStore.InsertNode(ctx, cfg.postState, cfg.blockRoot)
	if err != nil {
		return errors.Wrapf(err, "could not insert block %d to fork choice store", cfg.signed.Block().Slot())
	}
	if err := s.handleBlockAttestations(ctx, cfg.signed.Block(), cfg.postState); err != nil {
		return errors.Wrap(err, "could not handle block's attestations")
	}

	s.InsertSlashingsToForkChoiceStore(ctx, cfg.signed.Block().Body().AttesterSlashings())
	if cfg.isValidPayload {
		if err := s.cfg.ForkChoiceStore.SetOptimisticToValid(ctx, cfg.blockRoot); err != nil {
			return errors.Wrap(err, "could not set optimistic block to valid")
		}
	}
	start := time.Now()
	cfg.headRoot, err = s.cfg.ForkChoiceStore.Head(ctx)
	if err != nil {
		log.WithError(err).Warn("Could not update head")
	}
	newBlockHeadElapsedTime.Observe(float64(time.Since(start).Milliseconds()))
	if cfg.headRoot != cfg.blockRoot {
		s.logNonCanonicalBlockReceived(cfg.blockRoot, cfg.headRoot)
		return nil
	}
	if err := s.getFCUArgs(cfg, fcuArgs); err != nil {
		log.WithError(err).Error("Could not get forkchoice update argument")
		return nil
	}
	if err := s.sendFCU(cfg, fcuArgs); err != nil {
		return errors.Wrap(err, "could not send FCU to engine")
	}

	return nil
}

func getStateVersionAndPayload(st state.BeaconState) (int, interfaces.ExecutionData, error) {
	if st == nil {
		return 0, nil, errors.New("nil state")
	}
	var preStateHeader interfaces.ExecutionData
	var err error
	preStateVersion := st.Version()
	switch preStateVersion {
	case version.Phase0, version.Altair:
	default:
		preStateHeader, err = st.LatestExecutionPayloadHeader()
		if err != nil {
			return 0, nil, err
		}
	}
	return preStateVersion, preStateHeader, nil
}

func (s *Service) onBlockBatch(ctx context.Context, blks []consensusblocks.ROBlock, avs das.AvailabilityStore) error {
	ctx, span := trace.StartSpan(ctx, "blockChain.onBlockBatch")
	defer span.End()

	if len(blks) == 0 {
		return errors.New("no blocks provided")
	}

	if err := consensusblocks.BeaconBlockIsNil(blks[0]); err != nil {
		return invalidBlock{error: err}
	}
	b := blks[0].Block()

	// Retrieve incoming block's pre state.
	if err := s.verifyBlkPreState(ctx, b); err != nil {
		return err
	}
	preState, err := s.cfg.StateGen.StateByRootInitialSync(ctx, b.ParentRoot())
	if err != nil {
		return err
	}
	if preState == nil || preState.IsNil() {
		return fmt.Errorf("nil pre state for slot %d", b.Slot())
	}

	// Fill in missing blocks
	if err := s.fillInForkChoiceMissingBlocks(ctx, blks[0].Block(), preState.CurrentJustifiedCheckpoint(), preState.FinalizedCheckpoint()); err != nil {
		return errors.Wrap(err, "could not fill in missing blocks to forkchoice")
	}

	jCheckpoints := make([]*ethpb.Checkpoint, len(blks))
	fCheckpoints := make([]*ethpb.Checkpoint, len(blks))
	sigSet := bls.NewSet()
	type versionAndHeader struct {
		version int
		header  interfaces.ExecutionData
	}
	preVersionAndHeaders := make([]*versionAndHeader, len(blks))
	postVersionAndHeaders := make([]*versionAndHeader, len(blks))
	var set *bls.SignatureBatch
	boundaries := make(map[[32]byte]state.BeaconState)
	for i, b := range blks {
		v, h, err := getStateVersionAndPayload(preState)
		if err != nil {
			return err
		}
		preVersionAndHeaders[i] = &versionAndHeader{
			version: v,
			header:  h,
		}

		set, preState, err = transition.ExecuteStateTransitionNoVerifyAnySig(ctx, preState, b)
		if err != nil {
			return invalidBlock{error: err}
		}
		// Save potential boundary states.
		if slots.IsEpochStart(preState.Slot()) {
			boundaries[b.Root()] = preState.Copy()
		}
		jCheckpoints[i] = preState.CurrentJustifiedCheckpoint()
		fCheckpoints[i] = preState.FinalizedCheckpoint()

		v, h, err = getStateVersionAndPayload(preState)
		if err != nil {
			return err
		}
		postVersionAndHeaders[i] = &versionAndHeader{
			version: v,
			header:  h,
		}
		sigSet.Join(set)
	}

	var verify bool
	if features.Get().EnableVerboseSigVerification {
		verify, err = sigSet.VerifyVerbosely()
	} else {
		verify, err = sigSet.Verify()
	}
	if err != nil {
		return invalidBlock{error: err}
	}
	if !verify {
		return errors.New("batch block signature verification failed")
	}

	// blocks have been verified, save them and call the engine
	pendingNodes := make([]*forkchoicetypes.BlockAndCheckpoints, len(blks))
	var isValidPayload bool
	for i, b := range blks {
		root := b.Root()
		isValidPayload, err = s.notifyNewPayload(ctx,
			postVersionAndHeaders[i].version,
			postVersionAndHeaders[i].header, b)
		if err != nil {
			return s.handleInvalidExecutionError(ctx, err, root, b.Block().ParentRoot())
		}
		if isValidPayload {
			if err := s.validateMergeTransitionBlock(ctx, preVersionAndHeaders[i].version,
				preVersionAndHeaders[i].header, b); err != nil {
				return err
			}
		}
		if err := avs.IsDataAvailable(ctx, s.CurrentSlot(), b); err != nil {
			return errors.Wrapf(err, "could not validate blob data availability at slot %d", b.Block().Slot())
		}
		args := &forkchoicetypes.BlockAndCheckpoints{Block: b.Block(),
			JustifiedCheckpoint: jCheckpoints[i],
			FinalizedCheckpoint: fCheckpoints[i]}
		pendingNodes[len(blks)-i-1] = args
		if err := s.saveInitSyncBlock(ctx, root, b); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		if err := s.cfg.BeaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{
			Slot: b.Block().Slot(),
			Root: root[:],
		}); err != nil {
			tracing.AnnotateError(span, err)
			return err
		}
		if i > 0 && jCheckpoints[i].Epoch > jCheckpoints[i-1].Epoch {
			if err := s.cfg.BeaconDB.SaveJustifiedCheckpoint(ctx, jCheckpoints[i]); err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
		}
		if i > 0 && fCheckpoints[i].Epoch > fCheckpoints[i-1].Epoch {
			if err := s.updateFinalized(ctx, fCheckpoints[i]); err != nil {
				tracing.AnnotateError(span, err)
				return err
			}
		}
	}
	// Save boundary states that will be useful for forkchoice
	for r, st := range boundaries {
		if err := s.cfg.StateGen.SaveState(ctx, r, st); err != nil {
			return err
		}
	}
	lastB := blks[len(blks)-1]
	lastBR := lastB.Root()
	// Also saves the last post state which to be used as pre state for the next batch.
	if err := s.cfg.StateGen.SaveState(ctx, lastBR, preState); err != nil {
		return err
	}
	// Insert all nodes but the last one to forkchoice
	if err := s.cfg.ForkChoiceStore.InsertChain(ctx, pendingNodes); err != nil {
		return errors.Wrap(err, "could not insert batch to forkchoice")
	}
	// Insert the last block to forkchoice
	if err := s.cfg.ForkChoiceStore.InsertNode(ctx, preState, lastBR); err != nil {
		return errors.Wrap(err, "could not insert last block in batch to forkchoice")
	}
	// Set their optimistic status
	if isValidPayload {
		if err := s.cfg.ForkChoiceStore.SetOptimisticToValid(ctx, lastBR); err != nil {
			return errors.Wrap(err, "could not set optimistic block to valid")
		}
	}
	arg := &fcuConfig{
		headState: preState,
		headRoot:  lastBR,
		headBlock: lastB,
	}
	if _, err := s.notifyForkchoiceUpdate(ctx, arg); err != nil {
		return err
	}
	return s.saveHeadNoDB(ctx, lastB, lastBR, preState, !isValidPayload)
}

func (s *Service) updateEpochBoundaryCaches(ctx context.Context, st state.BeaconState) error {
	e := coreTime.CurrentEpoch(st)
	if err := helpers.UpdateCommitteeCache(ctx, st, e); err != nil {
		return errors.Wrap(err, "could not update committee cache")
	}
	if err := helpers.UpdateProposerIndicesInCache(ctx, st, e); err != nil {
		return errors.Wrap(err, "could not update proposer index cache")
	}
	go func() {
		// Use a custom deadline here, since this method runs asynchronously.
		// We ignore the parent method's context and instead create a new one
		// with a custom deadline, therefore using the background context instead.
		slotCtx, cancel := context.WithTimeout(context.Background(), slotDeadline)
		defer cancel()
		if err := helpers.UpdateCommitteeCache(slotCtx, st, e+1); err != nil {
			log.WithError(err).Warn("Could not update committee cache")
		}
	}()
	// The latest block header is from the previous epoch
	r, err := st.LatestBlockHeader().HashTreeRoot()
	if err != nil {
		log.WithError(err).Error("could not update proposer index state-root map")
		return nil
	}
	// The proposer indices cache takes the target root for the previous
	// epoch as key
	if e > 0 {
		e = e - 1
	}
	target, err := s.cfg.ForkChoiceStore.TargetRootForEpoch(r, e)
	if err != nil {
		log.WithError(err).Error("could not update proposer index state-root map")
		return nil
	}
	err = helpers.UpdateCachedCheckpointToStateRoot(st, &forkchoicetypes.Checkpoint{Epoch: e, Root: target})
	if err != nil {
		log.WithError(err).Error("could not update proposer index state-root map")
	}
	return nil
}

// Epoch boundary tasks: it copies the headState and updates the epoch boundary
// caches.
func (s *Service) handleEpochBoundary(ctx context.Context, slot primitives.Slot, headState state.BeaconState, blockRoot []byte) error {
	ctx, span := trace.StartSpan(ctx, "blockChain.handleEpochBoundary")
	defer span.End()
	// return early if we are advancing to a past epoch
	if slot < headState.Slot() {
		return nil
	}
	if !slots.IsEpochEnd(slot) {
		return nil
	}
	copied := headState.Copy()
	copied, err := transition.ProcessSlotsUsingNextSlotCache(ctx, copied, blockRoot, slot+1)
	if err != nil {
		return err
	}
	return s.updateEpochBoundaryCaches(ctx, copied)
}

// This feeds in the attestations included in the block to fork choice store. It's allows fork choice store
// to gain information on the most current chain.
func (s *Service) handleBlockAttestations(ctx context.Context, blk interfaces.ReadOnlyBeaconBlock, st state.BeaconState) error {
	// Feed in block's attestations to fork choice store.
	for _, a := range blk.Body().Attestations() {
		committee, err := helpers.BeaconCommitteeFromState(ctx, st, a.Data.Slot, a.Data.CommitteeIndex)
		if err != nil {
			return err
		}
		indices, err := attestation.AttestingIndices(a.AggregationBits, committee)
		if err != nil {
			return err
		}
		r := bytesutil.ToBytes32(a.Data.BeaconBlockRoot)
		if s.cfg.ForkChoiceStore.HasNode(r) {
			s.cfg.ForkChoiceStore.ProcessAttestation(ctx, indices, r, a.Data.Target.Epoch)
		} else if err := s.cfg.AttPool.SaveBlockAttestation(a); err != nil {
			return err
		}
	}
	return nil
}

// InsertSlashingsToForkChoiceStore inserts attester slashing indices to fork choice store.
// To call this function, it's caller's responsibility to ensure the slashing object is valid.
// This function requires a write lock on forkchoice.
func (s *Service) InsertSlashingsToForkChoiceStore(ctx context.Context, slashings []*ethpb.AttesterSlashing) {
	for _, slashing := range slashings {
		indices := blocks.SlashableAttesterIndices(slashing)
		for _, index := range indices {
			s.cfg.ForkChoiceStore.InsertSlashedIndex(ctx, primitives.ValidatorIndex(index))
		}
	}
}

// This saves post state info to DB or cache. This also saves post state info to fork choice store.
// Post state info consists of processed block and state. Do not call this method unless the block and state are verified.
func (s *Service) savePostStateInfo(ctx context.Context, r [32]byte, b interfaces.ReadOnlySignedBeaconBlock, st state.BeaconState) error {
	ctx, span := trace.StartSpan(ctx, "blockChain.savePostStateInfo")
	defer span.End()
	if err := s.cfg.BeaconDB.SaveBlock(ctx, b); err != nil {
		return errors.Wrapf(err, "could not save block from slot %d", b.Block().Slot())
	}
	if err := s.cfg.StateGen.SaveState(ctx, r, st); err != nil {
		return errors.Wrap(err, "could not save state")
	}
	return nil
}

// This removes the attestations in block `b` from the attestation mem pool.
func (s *Service) pruneAttsFromPool(headBlock interfaces.ReadOnlySignedBeaconBlock) error {
	atts := headBlock.Block().Body().Attestations()
	for _, att := range atts {
		if helpers.IsAggregated(att) {
			if err := s.cfg.AttPool.DeleteAggregatedAttestation(att); err != nil {
				return err
			}
		} else {
			if err := s.cfg.AttPool.DeleteUnaggregatedAttestation(att); err != nil {
				return err
			}
		}
	}
	return nil
}

// validateMergeTransitionBlock validates the merge transition block.
func (s *Service) validateMergeTransitionBlock(ctx context.Context, stateVersion int, stateHeader interfaces.ExecutionData, blk interfaces.ReadOnlySignedBeaconBlock) error {
	// Skip validation if block is older than Bellatrix.
	if blocks.IsPreBellatrixVersion(blk.Block().Version()) {
		return nil
	}

	// Skip validation if block has an empty payload.
	payload, err := blk.Block().Body().Execution()
	if err != nil {
		return invalidBlock{error: err}
	}
	isEmpty, err := consensusblocks.IsEmptyExecutionData(payload)
	if err != nil {
		return err
	}
	if isEmpty {
		return nil
	}

	// Handle case where pre-state is Altair but block contains payload.
	// To reach here, the block must have contained a valid payload.
	if blocks.IsPreBellatrixVersion(stateVersion) {
		return s.validateMergeBlock(ctx, blk)
	}

	// Skip validation if the block is not a merge transition block.
	// To reach here. The payload must be non-empty. If the state header is empty then it's at transition.
	empty, err := consensusblocks.IsEmptyExecutionData(stateHeader)
	if err != nil {
		return err
	}
	if !empty {
		return nil
	}
	return s.validateMergeBlock(ctx, blk)
}

// This routine checks if there is a cached proposer payload ID available for the next slot proposer.
// If there is not, it will call forkchoice updated with the correct payload attribute then cache the payload ID.
func (s *Service) runLateBlockTasks() {
	if err := s.waitForSync(); err != nil {
		log.WithError(err).Error("failed to wait for initial sync")
		return
	}

	attThreshold := params.BeaconConfig().SecondsPerSlot / 3
	ticker := slots.NewSlotTickerWithOffset(s.genesisTime, time.Duration(attThreshold)*time.Second, params.BeaconConfig().SecondsPerSlot)
	for {
		select {
		case <-ticker.C():
			s.lateBlockTasks(s.ctx)
		case <-s.ctx.Done():
			log.Debug("Context closed, exiting routine")
			return
		}
	}
}

// missingIndices uses the expected commitments from the block to determine
// which BlobSidecar indices would need to be in the database for DA success.
// It returns a map where each key represents a missing BlobSidecar index.
// An empty map means we have all indices; a non-empty map can be used to compare incoming
// BlobSidecars against the set of known missing sidecars.
func missingIndices(bs *filesystem.BlobStorage, root [32]byte, expected [][]byte) (map[uint64]struct{}, error) {
	if len(expected) == 0 {
		return nil, nil
	}
	if len(expected) > fieldparams.MaxBlobsPerBlock {
		return nil, errMaxBlobsExceeded
	}
	indices, err := bs.Indices(root)
	if err != nil {
		return nil, err
	}
	missing := make(map[uint64]struct{}, len(expected))
	for i := range expected {
		ui := uint64(i)
		if len(expected[i]) > 0 {
			if !indices[i] {
				missing[ui] = struct{}{}
			}
		}
	}
	return missing, nil
}

// isDataAvailable blocks until all BlobSidecars committed to in the block are available,
// or an error or context cancellation occurs. A nil result means that the data availability check is successful.
// The function will first check the database to see if all sidecars have been persisted. If any
// sidecars are missing, it will then read from the blobNotifier channel for the given root until the channel is
// closed, the context hits cancellation/timeout, or notifications have been received for all the missing sidecars.
func (s *Service) isDataAvailable(ctx context.Context, root [32]byte, signed interfaces.ReadOnlySignedBeaconBlock) error {
	if signed.Version() < version.Deneb {
		return nil
	}

	block := signed.Block()
	if block == nil {
		return errors.New("invalid nil beacon block")
	}
	// We are only required to check within MIN_EPOCHS_FOR_BLOB_SIDECARS_REQUESTS
	if !params.WithinDAPeriod(slots.ToEpoch(block.Slot()), slots.ToEpoch(s.CurrentSlot())) {
		return nil
	}

	body := block.Body()
	if body == nil {
		return errors.New("invalid nil beacon block body")
	}
	kzgCommitments, err := body.BlobKzgCommitments()
	if err != nil {
		return errors.Wrap(err, "could not get KZG commitments")
	}
	// expected is the number of kzg commitments observed in the block.
	expected := len(kzgCommitments)
	if expected == 0 {
		return nil
	}
	// get a map of BlobSidecar indices that are not currently available.
	missing, err := missingIndices(s.blobStorage, root, kzgCommitments)
	if err != nil {
		return err
	}
	// If there are no missing indices, all BlobSidecars are available.
	if len(missing) == 0 {
		return nil
	}

	// The gossip handler for blobs writes the index of each verified blob referencing the given
	// root to the channel returned by blobNotifiers.forRoot.
	nc := s.blobNotifiers.forRoot(root)
	for {
		select {
		case idx := <-nc:
			// Delete each index seen in the notification channel.
			delete(missing, idx)
			// Read from the channel until there are no more missing sidecars.
			if len(missing) > 0 {
				continue
			}
			// Once all sidecars have been observed, clean up the notification channel.
			s.blobNotifiers.delete(root)
			return nil
		case <-ctx.Done():
			return errors.Wrap(ctx.Err(), "context deadline waiting for blob sidecars")
		}
	}
}

// lateBlockTasks  is called 4 seconds into the slot and performs tasks
// related to late blocks. It emits a MissedSlot state feed event.
// It calls FCU and sets the right attributes if we are proposing next slot
// it also updates the next slot cache and the proposer index cache to deal with skipped slots.
func (s *Service) lateBlockTasks(ctx context.Context) {
	currentSlot := s.CurrentSlot()
	if s.CurrentSlot() == s.HeadSlot() {
		return
	}
	s.cfg.ForkChoiceStore.RLock()
	defer s.cfg.ForkChoiceStore.RUnlock()
	// return early if we are in init sync
	if !s.inRegularSync() {
		return
	}
	s.cfg.StateNotifier.StateFeed().Send(&feed.Event{
		Type: statefeed.MissedSlot,
	})
	s.headLock.RLock()
	headRoot := s.headRoot()
	headState := s.headState(ctx)
	s.headLock.RUnlock()
	lastRoot, lastState := transition.LastCachedState()
	if lastState == nil {
		lastRoot, lastState = headRoot[:], headState
	}
	// Copy all the field tries in our cached state in the event of late
	// blocks.
	lastState.CopyAllTries()
	if err := transition.UpdateNextSlotCache(ctx, lastRoot, lastState); err != nil {
		log.WithError(err).Debug("could not update next slot state cache")
	}
	if err := s.handleEpochBoundary(ctx, currentSlot, headState, headRoot[:]); err != nil {
		log.WithError(err).Error("lateBlockTasks: could not update epoch boundary caches")
	}
	// return early if we already started building a block for the current
	// head root
	_, has := s.cfg.PayloadIDCache.PayloadID(s.CurrentSlot()+1, headRoot)
	if has {
		return
	}

	attribute := s.getPayloadAttribute(ctx, headState, s.CurrentSlot()+1, headRoot[:])
	// return early if we are not proposing next slot
	if attribute.IsEmpty() {
		return
	}

	s.headLock.RLock()
	headBlock, err := s.headBlock()
	if err != nil {
		s.headLock.RUnlock()
		log.WithError(err).Debug("could not perform late block tasks: failed to retrieve head block")
		return
	}
	s.headLock.RUnlock()

	fcuArgs := &fcuConfig{
		headState:  headState,
		headRoot:   headRoot,
		headBlock:  headBlock,
		attributes: attribute,
	}
	_, err = s.notifyForkchoiceUpdate(ctx, fcuArgs)
	if err != nil {
		log.WithError(err).Debug("could not perform late block tasks: failed to update forkchoice with engine")
	}
}

// waitForSync blocks until the node is synced to the head.
func (s *Service) waitForSync() error {
	select {
	case <-s.syncComplete:
		return nil
	case <-s.ctx.Done():
		return errors.New("context closed, exiting goroutine")
	}
}

func (s *Service) handleInvalidExecutionError(ctx context.Context, err error, blockRoot [32]byte, parentRoot [32]byte) error {
	if IsInvalidBlock(err) && InvalidBlockLVH(err) != [32]byte{} {
		return s.pruneInvalidBlock(ctx, blockRoot, parentRoot, InvalidBlockLVH(err))
	}
	return err
}
