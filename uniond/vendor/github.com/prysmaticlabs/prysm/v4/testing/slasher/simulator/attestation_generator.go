package simulator

import (
	"context"
	"math"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

func (s *Simulator) generateAttestationsForSlot(
	ctx context.Context, slot primitives.Slot,
) ([]*ethpb.IndexedAttestation, []*ethpb.AttesterSlashing, error) {
	attestations := make([]*ethpb.IndexedAttestation, 0)
	slashings := make([]*ethpb.AttesterSlashing, 0)
	currentEpoch := slots.ToEpoch(slot)

	committeesPerSlot := helpers.SlotCommitteeCount(s.srvConfig.Params.NumValidators)
	valsPerCommittee := s.srvConfig.Params.NumValidators /
		(committeesPerSlot * uint64(s.srvConfig.Params.SlotsPerEpoch))
	valsPerSlot := committeesPerSlot * valsPerCommittee

	if currentEpoch < 2 {
		return nil, nil, nil
	}
	sourceEpoch := currentEpoch - 1

	var slashedIndices []uint64
	startIdx := valsPerSlot * uint64(slot%s.srvConfig.Params.SlotsPerEpoch)
	endIdx := startIdx + valsPerCommittee
	for c := primitives.CommitteeIndex(0); uint64(c) < committeesPerSlot; c++ {
		attData := &ethpb.AttestationData{
			Slot:            slot,
			CommitteeIndex:  c,
			BeaconBlockRoot: bytesutil.PadTo([]byte("block"), 32),
			Source: &ethpb.Checkpoint{
				Epoch: sourceEpoch,
				Root:  bytesutil.PadTo([]byte("source"), 32),
			},
			Target: &ethpb.Checkpoint{
				Epoch: currentEpoch,
				Root:  bytesutil.PadTo([]byte("target"), 32),
			},
		}

		valsPerAttestation := uint64(math.Floor(s.srvConfig.Params.AggregationPercent * float64(valsPerCommittee)))
		for i := startIdx; i < endIdx; i += valsPerAttestation {
			attEndIdx := i + valsPerAttestation
			if attEndIdx >= endIdx {
				attEndIdx = endIdx
			}
			indices := make([]uint64, 0, valsPerAttestation)
			for idx := i; idx < attEndIdx; idx++ {
				indices = append(indices, idx)
			}
			att := &ethpb.IndexedAttestation{
				AttestingIndices: indices,
				Data:             attData,
				Signature:        params.BeaconConfig().EmptySignature[:],
			}
			beaconState, err := s.srvConfig.AttestationStateFetcher.AttestationTargetState(ctx, att.Data.Target)
			if err != nil {
				return nil, nil, err
			}

			// Sign the attestation with a valid signature.
			aggSig, err := s.aggregateSigForAttestation(beaconState, att)
			if err != nil {
				return nil, nil, err
			}
			att.Signature = aggSig.Marshal()

			attestations = append(attestations, att)
			if rand.NewGenerator().Float64() < s.srvConfig.Params.AttesterSlashingProbab {
				slashableAtt := makeSlashableFromAtt(att, []uint64{indices[0]})
				aggSig, err := s.aggregateSigForAttestation(beaconState, slashableAtt)
				if err != nil {
					return nil, nil, err
				}
				slashableAtt.Signature = aggSig.Marshal()
				slashedIndices = append(slashedIndices, slashableAtt.AttestingIndices...)
				slashings = append(slashings, &ethpb.AttesterSlashing{
					Attestation_1: att,
					Attestation_2: slashableAtt,
				})
				attestations = append(attestations, slashableAtt)
			}
		}
		startIdx += valsPerCommittee
		endIdx += valsPerCommittee
	}
	if len(slashedIndices) > 0 {
		log.WithFields(logrus.Fields{
			"amount":  len(slashedIndices),
			"indices": slashedIndices,
		}).Infof("Slashable attestation made")
	}
	return attestations, slashings, nil
}

func (s *Simulator) aggregateSigForAttestation(
	beaconState state.ReadOnlyBeaconState, att *ethpb.IndexedAttestation,
) (bls.Signature, error) {
	domain, err := signing.Domain(
		beaconState.Fork(),
		att.Data.Target.Epoch,
		params.BeaconConfig().DomainBeaconAttester,
		beaconState.GenesisValidatorsRoot(),
	)
	if err != nil {
		return nil, err
	}
	signingRoot, err := signing.ComputeSigningRoot(att.Data, domain)
	if err != nil {
		return nil, err
	}
	sigs := make([]bls.Signature, len(att.AttestingIndices))
	for i, validatorIndex := range att.AttestingIndices {
		privKey := s.srvConfig.PrivateKeysByValidatorIndex[primitives.ValidatorIndex(validatorIndex)]
		sigs[i] = privKey.Sign(signingRoot[:])
	}
	return bls.AggregateSignatures(sigs), nil
}

func makeSlashableFromAtt(att *ethpb.IndexedAttestation, indices []uint64) *ethpb.IndexedAttestation {
	if att.Data.Source.Epoch <= 2 {
		return makeDoubleVoteFromAtt(att, indices)
	}
	attData := &ethpb.AttestationData{
		Slot:            att.Data.Slot,
		CommitteeIndex:  att.Data.CommitteeIndex,
		BeaconBlockRoot: att.Data.BeaconBlockRoot,
		Source: &ethpb.Checkpoint{
			Epoch: att.Data.Source.Epoch - 3,
			Root:  att.Data.Source.Root,
		},
		Target: &ethpb.Checkpoint{
			Epoch: att.Data.Target.Epoch,
			Root:  att.Data.Target.Root,
		},
	}
	return &ethpb.IndexedAttestation{
		AttestingIndices: indices,
		Data:             attData,
		Signature:        params.BeaconConfig().EmptySignature[:],
	}
}

func makeDoubleVoteFromAtt(att *ethpb.IndexedAttestation, indices []uint64) *ethpb.IndexedAttestation {
	attData := &ethpb.AttestationData{
		Slot:            att.Data.Slot,
		CommitteeIndex:  att.Data.CommitteeIndex,
		BeaconBlockRoot: bytesutil.PadTo([]byte("slash me"), 32),
		Source: &ethpb.Checkpoint{
			Epoch: att.Data.Source.Epoch,
			Root:  att.Data.Source.Root,
		},
		Target: &ethpb.Checkpoint{
			Epoch: att.Data.Target.Epoch,
			Root:  att.Data.Target.Root,
		},
	}
	return &ethpb.IndexedAttestation{
		AttestingIndices: indices,
		Data:             attData,
		Signature:        params.BeaconConfig().EmptySignature[:],
	}
}
