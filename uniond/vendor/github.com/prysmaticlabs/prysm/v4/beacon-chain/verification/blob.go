package verification

import (
	"context"

	"github.com/pkg/errors"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/runtime/logging"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
)

const (
	RequireBlobIndexInBounds Requirement = iota
	RequireNotFromFutureSlot
	RequireSlotAboveFinalized
	RequireValidProposerSignature
	RequireSidecarParentSeen
	RequireSidecarParentValid
	RequireSidecarParentSlotLower
	RequireSidecarDescendsFromFinalized
	RequireSidecarInclusionProven
	RequireSidecarKzgProofVerified
	RequireSidecarProposerExpected
)

// GossipSidecarRequirements defines the set of requirements that BlobSidecars received on gossip
// must satisfy in order to upgrade an ROBlob to a VerifiedROBlob.
var GossipSidecarRequirements = []Requirement{
	RequireBlobIndexInBounds,
	RequireNotFromFutureSlot,
	RequireSlotAboveFinalized,
	RequireValidProposerSignature,
	RequireSidecarParentSeen,
	RequireSidecarParentValid,
	RequireSidecarParentSlotLower,
	RequireSidecarDescendsFromFinalized,
	RequireSidecarInclusionProven,
	RequireSidecarKzgProofVerified,
	RequireSidecarProposerExpected,
}

// InitsyncSidecarRequirements is the list of verification requirements to be used by the init-sync service
// for batch-mode syncing. Because we only perform batch verification as part of the IsDataAvailable method
// for blobs after the block has been verified, and the blobs to be verified are keyed in the cache by the
// block root, it is safe to skip the following verifications.
// RequireSidecarProposerExpected
// RequireNotFromFutureSlot,
// RequireSlotAboveFinalized,
// RequireSidecarParentSeen,
// RequireSidecarParentValid,
// RequireSidecarParentSlotLower,
// RequireSidecarDescendsFromFinalized,
var InitsyncSidecarRequirements = []Requirement{
	RequireValidProposerSignature,
	RequireSidecarKzgProofVerified,
	RequireBlobIndexInBounds,
	RequireSidecarInclusionProven,
}

var (
	ErrBlobInvalid = errors.New("blob failed verification")
	// ErrBlobIndexInvalid means RequireBlobIndexInBounds failed.
	ErrBlobIndexInvalid = errors.Wrap(ErrBlobInvalid, "incorrect blob sidecar index")
	// ErrFromFutureSlot means RequireSlotNotTooEarly failed.
	ErrFromFutureSlot = errors.Wrap(ErrBlobInvalid, "slot is too far in the future")
	// ErrSlotNotAfterFinalized means RequireSlotAboveFinalized failed.
	ErrSlotNotAfterFinalized = errors.Wrap(ErrBlobInvalid, "slot <= finalized checkpoint")
	// ErrInvalidProposerSignature means RequireValidProposerSignature failed.
	ErrInvalidProposerSignature = errors.Wrap(ErrBlobInvalid, "proposer signature could not be verified")
	// ErrSidecarParentNotSeen means RequireSidecarParentSeen failed.
	ErrSidecarParentNotSeen = errors.Wrap(ErrBlobInvalid, "parent root has not been seen")
	// ErrSidecarParentInvalid means RequireSidecarParentValid failed.
	ErrSidecarParentInvalid = errors.Wrap(ErrBlobInvalid, "parent block is not valid")
	// ErrSlotNotAfterParent means RequireSidecarParentSlotLower failed.
	ErrSlotNotAfterParent = errors.Wrap(ErrBlobInvalid, "slot <= slot")
	// ErrSidecarNotFinalizedDescendent means RequireSidecarDescendsFromFinalized failed.
	ErrSidecarNotFinalizedDescendent = errors.Wrap(ErrBlobInvalid, "blob parent is not descended from the finalized block")
	// ErrSidecarInclusionProofInvalid means RequireSidecarInclusionProven failed.
	ErrSidecarInclusionProofInvalid = errors.Wrap(ErrBlobInvalid, "sidecar inclusion proof verification failed")
	// ErrSidecarKzgProofInvalid means RequireSidecarKzgProofVerified failed.
	ErrSidecarKzgProofInvalid = errors.Wrap(ErrBlobInvalid, "sidecar kzg commitment proof verification failed")
	// ErrSidecarUnexpectedProposer means RequireSidecarProposerExpected failed.
	ErrSidecarUnexpectedProposer = errors.Wrap(ErrBlobInvalid, "sidecar was not proposed by the expected proposer_index")
)

type ROBlobVerifier struct {
	*sharedResources
	results              *results
	blob                 blocks.ROBlob
	parent               state.BeaconState
	verifyBlobCommitment roblobCommitmentVerifier
}

type roblobCommitmentVerifier func(...blocks.ROBlob) error

var _ BlobVerifier = &ROBlobVerifier{}

// VerifiedROBlob "upgrades" the wrapped ROBlob to a VerifiedROBlob.
// If any of the verifications ran against the blob failed, or some required verifications
// were not run, an error will be returned.
func (bv *ROBlobVerifier) VerifiedROBlob() (blocks.VerifiedROBlob, error) {
	if bv.results.allSatisfied() {
		return blocks.NewVerifiedROBlob(bv.blob), nil
	}
	return blocks.VerifiedROBlob{}, bv.results.errors(ErrBlobInvalid)
}

// SatisfyRequirement allows the caller to assert that a requirement has been satisfied.
// This gives us a way to tick the box for a requirement where the usual method would be impractical.
// For example, when batch syncing, forkchoice is only updated at the end of the batch. So the checks that use
// forkchoice, like descends from finalized or parent seen, would necessarily fail. Allowing the caller to
// assert the requirement has been satisfied ensures we have an easy way to audit which piece of code is satisfying
// a requireent outside of this package.
func (bv *ROBlobVerifier) SatisfyRequirement(req Requirement) {
	bv.recordResult(req, nil)
}

func (bv *ROBlobVerifier) recordResult(req Requirement, err *error) {
	if err == nil || *err == nil {
		bv.results.record(req, nil)
		return
	}
	bv.results.record(req, *err)
}

// BlobIndexInBounds represents the follow spec verification:
// [REJECT] The sidecar's index is consistent with MAX_BLOBS_PER_BLOCK -- i.e. blob_sidecar.index < MAX_BLOBS_PER_BLOCK.
func (bv *ROBlobVerifier) BlobIndexInBounds() (err error) {
	defer bv.recordResult(RequireBlobIndexInBounds, &err)
	if bv.blob.Index >= fieldparams.MaxBlobsPerBlock {
		log.WithFields(logging.BlobFields(bv.blob)).Debug("Sidecar index >= MAX_BLOBS_PER_BLOCK")
		return ErrBlobIndexInvalid
	}
	return nil
}

// NotFromFutureSlot represents the spec verification:
// [IGNORE] The sidecar is not from a future slot (with a MAXIMUM_GOSSIP_CLOCK_DISPARITY allowance)
// -- i.e. validate that block_header.slot <= current_slot
func (bv *ROBlobVerifier) NotFromFutureSlot() (err error) {
	defer bv.recordResult(RequireNotFromFutureSlot, &err)
	if bv.clock.CurrentSlot() == bv.blob.Slot() {
		return nil
	}
	// earliestStart represents the time the slot starts, lowered by MAXIMUM_GOSSIP_CLOCK_DISPARITY.
	// We lower the time by MAXIMUM_GOSSIP_CLOCK_DISPARITY in case system time is running slightly behind real time.
	earliestStart := bv.clock.SlotStart(bv.blob.Slot()).Add(-1 * params.BeaconConfig().MaximumGossipClockDisparityDuration())
	// If the system time is still before earliestStart, we consider the blob from a future slot and return an error.
	if bv.clock.Now().Before(earliestStart) {
		return ErrFromFutureSlot
	}
	return nil
}

// SlotAboveFinalized represents the spec verification:
// [IGNORE] The sidecar is from a slot greater than the latest finalized slot
// -- i.e. validate that block_header.slot > compute_start_slot_at_epoch(state.finalized_checkpoint.epoch)
func (bv *ROBlobVerifier) SlotAboveFinalized() (err error) {
	defer bv.recordResult(RequireSlotAboveFinalized, &err)
	fcp := bv.fc.FinalizedCheckpoint()
	fSlot, err := slots.EpochStart(fcp.Epoch)
	if err != nil {
		return errors.Wrapf(ErrSlotNotAfterFinalized, "error computing epoch start slot for finalized checkpoint (%d) %s", fcp.Epoch, err.Error())
	}
	if bv.blob.Slot() <= fSlot {
		return ErrSlotNotAfterFinalized
	}
	return nil
}

// ValidProposerSignature represents the spec verification:
// [REJECT] The proposer signature of blob_sidecar.signed_block_header,
// is valid with respect to the block_header.proposer_index pubkey.
func (bv *ROBlobVerifier) ValidProposerSignature(ctx context.Context) (err error) {
	defer bv.recordResult(RequireValidProposerSignature, &err)
	sd := blobToSignatureData(bv.blob)
	// First check if there is a cached verification that can be reused.
	seen, err := bv.sc.SignatureVerified(sd)
	if seen {
		if err != nil {
			log.WithFields(logging.BlobFields(bv.blob)).WithError(err).Debug("reusing failed proposer signature validation from cache")
			return ErrInvalidProposerSignature
		}
		return nil
	}

	// Retrieve the parent state to fallback to full verification.
	parent, err := bv.parentState(ctx)
	if err != nil {
		log.WithFields(logging.BlobFields(bv.blob)).WithError(err).Debug("could not replay parent state for blob signature verification")
		return ErrInvalidProposerSignature
	}
	// Full verification, which will subsequently be cached for anything sharing the signature cache.
	if err = bv.sc.VerifySignature(sd, parent); err != nil {
		log.WithFields(logging.BlobFields(bv.blob)).WithError(err).Debug("signature verification failed")
		return ErrInvalidProposerSignature
	}
	return nil
}

// SidecarParentSeen represents the spec verification:
// [IGNORE] The sidecar's block's parent (defined by block_header.parent_root) has been seen
// (via both gossip and non-gossip sources) (a client MAY queue sidecars for processing once the parent block is retrieved).
func (bv *ROBlobVerifier) SidecarParentSeen(parentSeen func([32]byte) bool) (err error) {
	defer bv.recordResult(RequireSidecarParentSeen, &err)
	if parentSeen != nil && parentSeen(bv.blob.ParentRoot()) {
		return nil
	}
	if bv.fc.HasNode(bv.blob.ParentRoot()) {
		return nil
	}
	return ErrSidecarParentNotSeen
}

// SidecarParentValid represents the spec verification:
// [REJECT] The sidecar's block's parent (defined by block_header.parent_root) passes validation.
func (bv *ROBlobVerifier) SidecarParentValid(badParent func([32]byte) bool) (err error) {
	defer bv.recordResult(RequireSidecarParentValid, &err)
	if badParent != nil && badParent(bv.blob.ParentRoot()) {
		return ErrSidecarParentInvalid
	}
	return nil
}

// SidecarParentSlotLower represents the spec verification:
// [REJECT] The sidecar is from a higher slot than the sidecar's block's parent (defined by block_header.parent_root).
func (bv *ROBlobVerifier) SidecarParentSlotLower() (err error) {
	defer bv.recordResult(RequireSidecarParentSlotLower, &err)
	parentSlot, err := bv.fc.Slot(bv.blob.ParentRoot())
	if err != nil {
		return errors.Wrap(ErrSlotNotAfterParent, "parent root not in forkchoice")
	}
	if parentSlot >= bv.blob.Slot() {
		return ErrSlotNotAfterParent
	}
	return nil
}

// SidecarDescendsFromFinalized represents the spec verification:
// [REJECT] The current finalized_checkpoint is an ancestor of the sidecar's block
// -- i.e. get_checkpoint_block(store, block_header.parent_root, store.finalized_checkpoint.epoch) == store.finalized_checkpoint.root.
func (bv *ROBlobVerifier) SidecarDescendsFromFinalized() (err error) {
	defer bv.recordResult(RequireSidecarDescendsFromFinalized, &err)
	if !bv.fc.HasNode(bv.blob.ParentRoot()) {
		return ErrSidecarNotFinalizedDescendent
	}
	return nil
}

// SidecarInclusionProven represents the spec verification:
// [REJECT] The sidecar's inclusion proof is valid as verified by verify_blob_sidecar_inclusion_proof(blob_sidecar).
func (bv *ROBlobVerifier) SidecarInclusionProven() (err error) {
	defer bv.recordResult(RequireSidecarInclusionProven, &err)
	if err = blocks.VerifyKZGInclusionProof(bv.blob); err != nil {
		log.WithError(err).WithFields(logging.BlobFields(bv.blob)).Debug("sidecar inclusion proof verification failed")
		return ErrSidecarInclusionProofInvalid
	}
	return nil
}

// SidecarKzgProofVerified represents the spec verification:
// [REJECT] The sidecar's blob is valid as verified by
// verify_blob_kzg_proof(blob_sidecar.blob, blob_sidecar.kzg_commitment, blob_sidecar.kzg_proof).
func (bv *ROBlobVerifier) SidecarKzgProofVerified() (err error) {
	defer bv.recordResult(RequireSidecarKzgProofVerified, &err)
	if err = bv.verifyBlobCommitment(bv.blob); err != nil {
		log.WithError(err).WithFields(logging.BlobFields(bv.blob)).Debug("kzg commitment proof verification failed")
		return ErrSidecarKzgProofInvalid
	}
	return nil
}

// SidecarProposerExpected represents the spec verification:
// [REJECT] The sidecar is proposed by the expected proposer_index for the block's slot
// in the context of the current shuffling (defined by block_header.parent_root/block_header.slot).
// If the proposer_index cannot immediately be verified against the expected shuffling, the sidecar MAY be queued
// for later processing while proposers for the block's branch are calculated -- in such a case do not REJECT, instead IGNORE this message.
func (bv *ROBlobVerifier) SidecarProposerExpected(ctx context.Context) (err error) {
	defer bv.recordResult(RequireSidecarProposerExpected, &err)
	e := slots.ToEpoch(bv.blob.Slot())
	if e > 0 {
		e = e - 1
	}
	r, err := bv.fc.TargetRootForEpoch(bv.blob.ParentRoot(), e)
	if err != nil {
		return ErrSidecarUnexpectedProposer
	}
	c := &forkchoicetypes.Checkpoint{Root: r, Epoch: e}
	idx, cached := bv.pc.Proposer(c, bv.blob.Slot())
	if !cached {
		pst, err := bv.parentState(ctx)
		if err != nil {
			log.WithError(err).WithFields(logging.BlobFields(bv.blob)).Debug("state replay to parent_root failed")
			return ErrSidecarUnexpectedProposer
		}
		idx, err = bv.pc.ComputeProposer(ctx, bv.blob.ParentRoot(), bv.blob.Slot(), pst)
		if err != nil {
			log.WithError(err).WithFields(logging.BlobFields(bv.blob)).Debug("error computing proposer index from parent state")
			return ErrSidecarUnexpectedProposer
		}
	}
	if idx != bv.blob.ProposerIndex() {
		log.WithError(ErrSidecarUnexpectedProposer).
			WithFields(logging.BlobFields(bv.blob)).WithField("expected_proposer", idx).
			Debug("unexpected blob proposer")
		return ErrSidecarUnexpectedProposer
	}
	return nil
}

func (bv *ROBlobVerifier) parentState(ctx context.Context) (state.BeaconState, error) {
	if bv.parent != nil {
		return bv.parent, nil
	}
	st, err := bv.sr.StateByRoot(ctx, bv.blob.ParentRoot())
	if err != nil {
		return nil, err
	}
	bv.parent = st
	return bv.parent, nil
}

func blobToSignatureData(b blocks.ROBlob) SignatureData {
	return SignatureData{
		Root:      b.BlockRoot(),
		Parent:    b.ParentRoot(),
		Signature: bytesutil.ToBytes96(b.SignedBlockHeader.Signature),
		Proposer:  b.ProposerIndex(),
		Slot:      b.Slot(),
	}
}
