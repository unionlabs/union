package blockchain

import (
	"context"
	"crypto/sha256"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	payloadattribute "github.com/prysmaticlabs/prysm/v4/consensus-types/payload-attribute"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

const blobCommitmentVersionKZG uint8 = 0x01

var defaultLatestValidHash = bytesutil.PadTo([]byte{0xff}, 32)

// notifyForkchoiceUpdate signals execution engine the fork choice updates. Execution engine should:
// 1. Re-organizes the execution payload chain and corresponding state to make head_block_hash the head.
// 2. Applies finality to the execution state: it irreversibly persists the chain of all execution payloads and corresponding state, up to and including finalized_block_hash.
func (s *Service) notifyForkchoiceUpdate(ctx context.Context, arg *fcuConfig) (*enginev1.PayloadIDBytes, error) {
	ctx, span := trace.StartSpan(ctx, "blockChain.notifyForkchoiceUpdate")
	defer span.End()

	if arg.headBlock.IsNil() {
		log.Error("Head block is nil")
		return nil, nil
	}
	headBlk := arg.headBlock.Block()
	if headBlk == nil || headBlk.IsNil() || headBlk.Body().IsNil() {
		log.Error("Head block is nil")
		return nil, nil
	}
	// Must not call fork choice updated until the transition conditions are met on the Pow network.
	isExecutionBlk, err := blocks.IsExecutionBlock(headBlk.Body())
	if err != nil {
		log.WithError(err).Error("Could not determine if head block is execution block")
		return nil, nil
	}
	if !isExecutionBlk {
		return nil, nil
	}
	headPayload, err := headBlk.Body().Execution()
	if err != nil {
		log.WithError(err).Error("Could not get execution payload for head block")
		return nil, nil
	}
	finalizedHash := s.cfg.ForkChoiceStore.FinalizedPayloadBlockHash()
	justifiedHash := s.cfg.ForkChoiceStore.UnrealizedJustifiedPayloadBlockHash()
	fcs := &enginev1.ForkchoiceState{
		HeadBlockHash:      headPayload.BlockHash(),
		SafeBlockHash:      justifiedHash[:],
		FinalizedBlockHash: finalizedHash[:],
	}
	if arg.attributes == nil {
		arg.attributes = payloadattribute.EmptyWithVersion(headBlk.Version())
	}
	payloadID, lastValidHash, err := s.cfg.ExecutionEngineCaller.ForkchoiceUpdated(ctx, fcs, arg.attributes)
	if err != nil {
		switch err {
		case execution.ErrAcceptedSyncingPayloadStatus:
			forkchoiceUpdatedOptimisticNodeCount.Inc()
			log.WithFields(logrus.Fields{
				"headSlot":                  headBlk.Slot(),
				"headPayloadBlockHash":      fmt.Sprintf("%#x", bytesutil.Trunc(headPayload.BlockHash())),
				"finalizedPayloadBlockHash": fmt.Sprintf("%#x", bytesutil.Trunc(finalizedHash[:])),
			}).Info("Called fork choice updated with optimistic block")
			return payloadID, nil
		case execution.ErrInvalidPayloadStatus:
			forkchoiceUpdatedInvalidNodeCount.Inc()
			headRoot := arg.headRoot
			if len(lastValidHash) == 0 {
				lastValidHash = defaultLatestValidHash
			}
			invalidRoots, err := s.cfg.ForkChoiceStore.SetOptimisticToInvalid(ctx, headRoot, headBlk.ParentRoot(), bytesutil.ToBytes32(lastValidHash))
			if err != nil {
				log.WithError(err).Error("Could not set head root to invalid")
				return nil, nil
			}
			if err := s.removeInvalidBlockAndState(ctx, invalidRoots); err != nil {
				log.WithError(err).Error("Could not remove invalid block and state")
				return nil, nil
			}

			r, err := s.cfg.ForkChoiceStore.Head(ctx)
			if err != nil {
				log.WithFields(logrus.Fields{
					"slot":                 headBlk.Slot(),
					"blockRoot":            fmt.Sprintf("%#x", bytesutil.Trunc(headRoot[:])),
					"invalidChildrenCount": len(invalidRoots),
				}).Warn("Pruned invalid blocks, could not update head root")
				return nil, invalidBlock{error: ErrInvalidPayload, root: arg.headRoot, invalidAncestorRoots: invalidRoots}
			}
			b, err := s.getBlock(ctx, r)
			if err != nil {
				log.WithError(err).Error("Could not get head block")
				return nil, nil
			}
			st, err := s.cfg.StateGen.StateByRoot(ctx, r)
			if err != nil {
				log.WithError(err).Error("Could not get head state")
				return nil, nil
			}
			pid, err := s.notifyForkchoiceUpdate(ctx, &fcuConfig{
				headState:  st,
				headRoot:   r,
				headBlock:  b,
				attributes: arg.attributes,
			})
			if err != nil {
				return nil, err // Returning err because it's recursive here.
			}

			if err := s.saveHead(ctx, r, b, st); err != nil {
				log.WithError(err).Error("could not save head after pruning invalid blocks")
			}

			log.WithFields(logrus.Fields{
				"slot":                 headBlk.Slot(),
				"blockRoot":            fmt.Sprintf("%#x", bytesutil.Trunc(headRoot[:])),
				"invalidChildrenCount": len(invalidRoots),
				"newHeadRoot":          fmt.Sprintf("%#x", bytesutil.Trunc(r[:])),
			}).Warn("Pruned invalid blocks")
			return pid, invalidBlock{error: ErrInvalidPayload, root: arg.headRoot, invalidAncestorRoots: invalidRoots}

		default:
			log.WithError(err).Error(ErrUndefinedExecutionEngineError)
			return nil, nil
		}
	}
	forkchoiceUpdatedValidNodeCount.Inc()
	if err := s.cfg.ForkChoiceStore.SetOptimisticToValid(ctx, arg.headRoot); err != nil {
		log.WithError(err).Error("Could not set head root to valid")
		return nil, nil
	}
	// If the forkchoice update call has an attribute, update the payload ID cache.
	hasAttr := arg.attributes != nil && !arg.attributes.IsEmpty()
	nextSlot := s.CurrentSlot() + 1
	if hasAttr && payloadID != nil {
		var pId [8]byte
		copy(pId[:], payloadID[:])
		log.WithFields(logrus.Fields{
			"blockRoot": fmt.Sprintf("%#x", bytesutil.Trunc(arg.headRoot[:])),
			"headSlot":  headBlk.Slot(),
			"payloadID": fmt.Sprintf("%#x", bytesutil.Trunc(payloadID[:])),
		}).Info("Forkchoice updated with payload attributes for proposal")
		s.cfg.PayloadIDCache.Set(nextSlot, arg.headRoot, pId)
	} else if hasAttr && payloadID == nil && !features.Get().PrepareAllPayloads {
		log.WithFields(logrus.Fields{
			"blockHash": fmt.Sprintf("%#x", headPayload.BlockHash()),
			"slot":      headBlk.Slot(),
		}).Error("Received nil payload ID on VALID engine response")
	}
	return payloadID, nil
}

// getPayloadHash returns the payload hash given the block root.
// if the block is before bellatrix fork epoch, it returns the zero hash.
func (s *Service) getPayloadHash(ctx context.Context, root []byte) ([32]byte, error) {
	blk, err := s.getBlock(ctx, s.ensureRootNotZeros(bytesutil.ToBytes32(root)))
	if err != nil {
		return [32]byte{}, err
	}
	if blocks.IsPreBellatrixVersion(blk.Block().Version()) {
		return params.BeaconConfig().ZeroHash, nil
	}
	payload, err := blk.Block().Body().Execution()
	if err != nil {
		return [32]byte{}, errors.Wrap(err, "could not get execution payload")
	}
	return bytesutil.ToBytes32(payload.BlockHash()), nil
}

// notifyNewPayload signals execution engine on a new payload.
// It returns true if the EL has returned VALID for the block
func (s *Service) notifyNewPayload(ctx context.Context, preStateVersion int,
	preStateHeader interfaces.ExecutionData, blk interfaces.ReadOnlySignedBeaconBlock) (bool, error) {
	ctx, span := trace.StartSpan(ctx, "blockChain.notifyNewPayload")
	defer span.End()

	// Execution payload is only supported in Bellatrix and beyond. Pre
	// merge blocks are never optimistic
	if blk == nil {
		return false, errors.New("signed beacon block can't be nil")
	}
	if preStateVersion < version.Bellatrix {
		return true, nil
	}
	if err := consensusblocks.BeaconBlockIsNil(blk); err != nil {
		return false, err
	}
	body := blk.Block().Body()
	enabled, err := blocks.IsExecutionEnabledUsingHeader(preStateHeader, body)
	if err != nil {
		return false, errors.Wrap(invalidBlock{error: err}, "could not determine if execution is enabled")
	}
	if !enabled {
		return true, nil
	}
	payload, err := body.Execution()
	if err != nil {
		return false, errors.Wrap(invalidBlock{error: err}, "could not get execution payload")
	}

	var lastValidHash []byte
	if blk.Version() >= version.Deneb {
		var versionedHashes []common.Hash
		versionedHashes, err = kzgCommitmentsToVersionedHashes(blk.Block().Body())
		if err != nil {
			return false, errors.Wrap(err, "could not get versioned hashes to feed the engine")
		}
		pr := common.Hash(blk.Block().ParentRoot())
		lastValidHash, err = s.cfg.ExecutionEngineCaller.NewPayload(ctx, payload, versionedHashes, &pr)
	} else {
		lastValidHash, err = s.cfg.ExecutionEngineCaller.NewPayload(ctx, payload, []common.Hash{}, &common.Hash{} /*empty version hashes and root before Deneb*/)
	}
	switch err {
	case nil:
		newPayloadValidNodeCount.Inc()
		return true, nil
	case execution.ErrAcceptedSyncingPayloadStatus:
		newPayloadOptimisticNodeCount.Inc()
		log.WithFields(logrus.Fields{
			"slot":             blk.Block().Slot(),
			"payloadBlockHash": fmt.Sprintf("%#x", bytesutil.Trunc(payload.BlockHash())),
		}).Info("Called new payload with optimistic block")
		return false, nil
	case execution.ErrInvalidPayloadStatus:
		lvh := bytesutil.ToBytes32(lastValidHash)
		return false, invalidBlock{
			error:         ErrInvalidPayload,
			lastValidHash: lvh,
		}
	default:
		return false, errors.WithMessage(ErrUndefinedExecutionEngineError, err.Error())
	}
}

// reportInvalidBlock deals with the event that an invalid block was detected by the execution layer
func (s *Service) pruneInvalidBlock(ctx context.Context, root, parentRoot, lvh [32]byte) error {
	newPayloadInvalidNodeCount.Inc()
	invalidRoots, err := s.SetOptimisticToInvalid(ctx, root, parentRoot, lvh)
	if err != nil {
		return err
	}
	if err := s.removeInvalidBlockAndState(ctx, invalidRoots); err != nil {
		return err
	}
	log.WithFields(logrus.Fields{
		"blockRoot":            fmt.Sprintf("%#x", root),
		"invalidChildrenCount": len(invalidRoots),
	}).Warn("Pruned invalid blocks")
	return invalidBlock{
		invalidAncestorRoots: invalidRoots,
		error:                ErrInvalidPayload,
		lastValidHash:        lvh,
	}
}

// getPayloadAttributes returns the payload attributes for the given state and slot.
// The attribute is required to initiate a payload build process in the context of an `engine_forkchoiceUpdated` call.
func (s *Service) getPayloadAttribute(ctx context.Context, st state.BeaconState, slot primitives.Slot, headRoot []byte) payloadattribute.Attributer {
	emptyAttri := payloadattribute.EmptyWithVersion(st.Version())

	// If it is an epoch boundary then process slots to get the right
	// shuffling before checking if the proposer is tracked. Otherwise
	// perform this check before. This is cheap as the NSC has already been updated.
	var val cache.TrackedValidator
	var ok bool
	e := slots.ToEpoch(slot)
	stateEpoch := slots.ToEpoch(st.Slot())
	if e == stateEpoch {
		val, ok = s.trackedProposer(st, slot)
		if !ok {
			return emptyAttri
		}
	}
	st = st.Copy()
	if slot > st.Slot() {
		var err error
		st, err = transition.ProcessSlotsUsingNextSlotCache(ctx, st, headRoot, slot)
		if err != nil {
			log.WithError(err).Error("Could not process slots to get payload attribute")
			return emptyAttri
		}
	}
	if e > stateEpoch {
		emptyAttri := payloadattribute.EmptyWithVersion(st.Version())
		val, ok = s.trackedProposer(st, slot)
		if !ok {
			return emptyAttri
		}
	}
	// Get previous randao.
	prevRando, err := helpers.RandaoMix(st, time.CurrentEpoch(st))
	if err != nil {
		log.WithError(err).Error("Could not get randao mix to get payload attribute")
		return emptyAttri
	}

	// Get timestamp.
	t, err := slots.ToTime(uint64(s.genesisTime.Unix()), slot)
	if err != nil {
		log.WithError(err).Error("Could not get timestamp to get payload attribute")
		return emptyAttri
	}

	var attr payloadattribute.Attributer
	switch st.Version() {
	case version.Deneb:
		withdrawals, err := st.ExpectedWithdrawals()
		if err != nil {
			log.WithError(err).Error("Could not get expected withdrawals to get payload attribute")
			return emptyAttri
		}
		attr, err = payloadattribute.New(&enginev1.PayloadAttributesV3{
			Timestamp:             uint64(t.Unix()),
			PrevRandao:            prevRando,
			SuggestedFeeRecipient: val.FeeRecipient[:],
			Withdrawals:           withdrawals,
			ParentBeaconBlockRoot: headRoot,
		})
		if err != nil {
			log.WithError(err).Error("Could not get payload attribute")
			return emptyAttri
		}
	case version.Capella:
		withdrawals, err := st.ExpectedWithdrawals()
		if err != nil {
			log.WithError(err).Error("Could not get expected withdrawals to get payload attribute")
			return emptyAttri
		}
		attr, err = payloadattribute.New(&enginev1.PayloadAttributesV2{
			Timestamp:             uint64(t.Unix()),
			PrevRandao:            prevRando,
			SuggestedFeeRecipient: val.FeeRecipient[:],
			Withdrawals:           withdrawals,
		})
		if err != nil {
			log.WithError(err).Error("Could not get payload attribute")
			return emptyAttri
		}
	case version.Bellatrix:
		attr, err = payloadattribute.New(&enginev1.PayloadAttributes{
			Timestamp:             uint64(t.Unix()),
			PrevRandao:            prevRando,
			SuggestedFeeRecipient: val.FeeRecipient[:],
		})
		if err != nil {
			log.WithError(err).Error("Could not get payload attribute")
			return emptyAttri
		}
	default:
		log.WithField("version", st.Version()).Error("Could not get payload attribute due to unknown state version")
		return emptyAttri
	}

	return attr
}

// removeInvalidBlockAndState removes the invalid block, blob and its corresponding state from the cache and DB.
func (s *Service) removeInvalidBlockAndState(ctx context.Context, blkRoots [][32]byte) error {
	for _, root := range blkRoots {
		if err := s.cfg.StateGen.DeleteStateFromCaches(ctx, root); err != nil {
			return err
		}
		// Delete block also deletes the state as well.
		if err := s.cfg.BeaconDB.DeleteBlock(ctx, root); err != nil {
			// TODO(10487): If a caller requests to delete a root that's justified and finalized. We should gracefully shutdown.
			// This is an irreparable condition, it would me a justified or finalized block has become invalid.
			return err
		}
		if err := s.blobStorage.Remove(root); err != nil {
			// Blobs may not exist for some blocks, leading to deletion failures. Log such errors at debug level.
			log.WithError(err).Debug("Could not remove blob from blob storage")
		}
	}
	return nil
}

func kzgCommitmentsToVersionedHashes(body interfaces.ReadOnlyBeaconBlockBody) ([]common.Hash, error) {
	commitments, err := body.BlobKzgCommitments()
	if err != nil {
		return nil, errors.Wrap(invalidBlock{error: err}, "could not get blob kzg commitments")
	}

	versionedHashes := make([]common.Hash, len(commitments))
	for i, commitment := range commitments {
		versionedHashes[i] = ConvertKzgCommitmentToVersionedHash(commitment)
	}
	return versionedHashes, nil
}

func ConvertKzgCommitmentToVersionedHash(commitment []byte) common.Hash {
	versionedHash := sha256.Sum256(commitment)
	versionedHash[0] = blobCommitmentVersionKZG
	return versionedHash
}
