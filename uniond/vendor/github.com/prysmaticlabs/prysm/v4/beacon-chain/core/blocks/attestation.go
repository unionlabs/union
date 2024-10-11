package blocks

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"go.opencensus.io/trace"
)

// ProcessAttestationsNoVerifySignature applies processing operations to a block's inner attestation
// records. The only difference would be that the attestation signature would not be verified.
func ProcessAttestationsNoVerifySignature(
	ctx context.Context,
	beaconState state.BeaconState,
	b interfaces.ReadOnlyBeaconBlock,
) (state.BeaconState, error) {
	if b == nil || b.IsNil() {
		return nil, blocks.ErrNilBeaconBlock
	}
	body := b.Body()
	var err error
	for idx, att := range body.Attestations() {
		beaconState, err = ProcessAttestationNoVerifySignature(ctx, beaconState, att)
		if err != nil {
			return nil, errors.Wrapf(err, "could not verify attestation at index %d in block", idx)
		}
	}
	return beaconState, nil
}

// VerifyAttestationNoVerifySignature verifies the attestation without verifying the attestation signature. This is
// used before processing attestation with the beacon state.
func VerifyAttestationNoVerifySignature(
	ctx context.Context,
	beaconState state.ReadOnlyBeaconState,
	att *ethpb.Attestation,
) error {
	ctx, span := trace.StartSpan(ctx, "core.VerifyAttestationNoVerifySignature")
	defer span.End()

	if err := helpers.ValidateNilAttestation(att); err != nil {
		return err
	}
	currEpoch := time.CurrentEpoch(beaconState)
	prevEpoch := time.PrevEpoch(beaconState)
	data := att.Data
	if data.Target.Epoch != prevEpoch && data.Target.Epoch != currEpoch {
		return fmt.Errorf(
			"expected target epoch (%d) to be the previous epoch (%d) or the current epoch (%d)",
			data.Target.Epoch,
			prevEpoch,
			currEpoch,
		)
	}

	if data.Target.Epoch == currEpoch {
		if !beaconState.MatchCurrentJustifiedCheckpoint(data.Source) {
			return errors.New("source check point not equal to current justified checkpoint")
		}
	} else {
		if !beaconState.MatchPreviousJustifiedCheckpoint(data.Source) {
			return errors.New("source check point not equal to previous justified checkpoint")
		}
	}

	if err := helpers.ValidateSlotTargetEpoch(att.Data); err != nil {
		return err
	}

	s := att.Data.Slot
	minInclusionCheck := s+params.BeaconConfig().MinAttestationInclusionDelay <= beaconState.Slot()
	if !minInclusionCheck {
		return fmt.Errorf(
			"attestation slot %d + inclusion delay %d > state slot %d",
			s,
			params.BeaconConfig().MinAttestationInclusionDelay,
			beaconState.Slot(),
		)
	}

	if beaconState.Version() < version.Deneb {
		epochInclusionCheck := beaconState.Slot() <= s+params.BeaconConfig().SlotsPerEpoch
		if !epochInclusionCheck {
			return fmt.Errorf(
				"state slot %d > attestation slot %d + SLOTS_PER_EPOCH %d",
				beaconState.Slot(),
				s,
				params.BeaconConfig().SlotsPerEpoch,
			)
		}
	}
	activeValidatorCount, err := helpers.ActiveValidatorCount(ctx, beaconState, att.Data.Target.Epoch)
	if err != nil {
		return err
	}
	c := helpers.SlotCommitteeCount(activeValidatorCount)
	if uint64(att.Data.CommitteeIndex) >= c {
		return fmt.Errorf("committee index %d >= committee count %d", att.Data.CommitteeIndex, c)
	}

	if err := helpers.VerifyAttestationBitfieldLengths(ctx, beaconState, att); err != nil {
		return errors.Wrap(err, "could not verify attestation bitfields")
	}

	// Verify attesting indices are correct.
	committee, err := helpers.BeaconCommitteeFromState(ctx, beaconState, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return err
	}
	indexedAtt, err := attestation.ConvertToIndexed(ctx, att, committee)
	if err != nil {
		return err
	}

	return attestation.IsValidAttestationIndices(ctx, indexedAtt)
}

// ProcessAttestationNoVerifySignature processes the attestation without verifying the attestation signature. This
// method is used to validate attestations whose signatures have already been verified.
func ProcessAttestationNoVerifySignature(
	ctx context.Context,
	beaconState state.BeaconState,
	att *ethpb.Attestation,
) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "core.ProcessAttestationNoVerifySignature")
	defer span.End()

	if err := VerifyAttestationNoVerifySignature(ctx, beaconState, att); err != nil {
		return nil, err
	}

	currEpoch := time.CurrentEpoch(beaconState)
	data := att.Data
	s := att.Data.Slot
	proposerIndex, err := helpers.BeaconProposerIndex(ctx, beaconState)
	if err != nil {
		return nil, err
	}
	pendingAtt := &ethpb.PendingAttestation{
		Data:            data,
		AggregationBits: att.AggregationBits,
		InclusionDelay:  beaconState.Slot() - s,
		ProposerIndex:   proposerIndex,
	}

	if data.Target.Epoch == currEpoch {
		if err := beaconState.AppendCurrentEpochAttestations(pendingAtt); err != nil {
			return nil, err
		}
	} else {
		if err := beaconState.AppendPreviousEpochAttestations(pendingAtt); err != nil {
			return nil, err
		}
	}

	return beaconState, nil
}

// VerifyAttestationSignature converts and attestation into an indexed attestation and verifies
// the signature in that attestation.
func VerifyAttestationSignature(ctx context.Context, beaconState state.ReadOnlyBeaconState, att *ethpb.Attestation) error {
	if err := helpers.ValidateNilAttestation(att); err != nil {
		return err
	}
	committee, err := helpers.BeaconCommitteeFromState(ctx, beaconState, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return err
	}
	indexedAtt, err := attestation.ConvertToIndexed(ctx, att, committee)
	if err != nil {
		return err
	}
	return VerifyIndexedAttestation(ctx, beaconState, indexedAtt)
}

// VerifyIndexedAttestation determines the validity of an indexed attestation.
//
// Spec pseudocode definition:
//
//	def is_valid_indexed_attestation(state: BeaconState, indexed_attestation: IndexedAttestation) -> bool:
//	  """
//	  Check if ``indexed_attestation`` is not empty, has sorted and unique indices and has a valid aggregate signature.
//	  """
//	  # Verify indices are sorted and unique
//	  indices = indexed_attestation.attesting_indices
//	  if len(indices) == 0 or not indices == sorted(set(indices)):
//	      return False
//	  # Verify aggregate signature
//	  pubkeys = [state.validators[i].pubkey for i in indices]
//	  domain = get_domain(state, DOMAIN_BEACON_ATTESTER, indexed_attestation.data.target.epoch)
//	  signing_root = compute_signing_root(indexed_attestation.data, domain)
//	  return bls.FastAggregateVerify(pubkeys, signing_root, indexed_attestation.signature)
func VerifyIndexedAttestation(ctx context.Context, beaconState state.ReadOnlyBeaconState, indexedAtt *ethpb.IndexedAttestation) error {
	ctx, span := trace.StartSpan(ctx, "core.VerifyIndexedAttestation")
	defer span.End()

	if err := attestation.IsValidAttestationIndices(ctx, indexedAtt); err != nil {
		return err
	}
	domain, err := signing.Domain(
		beaconState.Fork(),
		indexedAtt.Data.Target.Epoch,
		params.BeaconConfig().DomainBeaconAttester,
		beaconState.GenesisValidatorsRoot(),
	)
	if err != nil {
		return err
	}
	indices := indexedAtt.AttestingIndices
	var pubkeys []bls.PublicKey
	for i := 0; i < len(indices); i++ {
		pubkeyAtIdx := beaconState.PubkeyAtIndex(primitives.ValidatorIndex(indices[i]))
		pk, err := bls.PublicKeyFromBytes(pubkeyAtIdx[:])
		if err != nil {
			return errors.Wrap(err, "could not deserialize validator public key")
		}
		pubkeys = append(pubkeys, pk)
	}
	return attestation.VerifyIndexedAttestationSig(ctx, indexedAtt, pubkeys, domain)
}
