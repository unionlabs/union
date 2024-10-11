package client

import (
	"context"
	"fmt"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	emptypb "github.com/golang/protobuf/ptypes/empty"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// SubmitSyncCommitteeMessage submits the sync committee message to the beacon chain.
func (v *validator) SubmitSyncCommitteeMessage(ctx context.Context, slot primitives.Slot, pubKey [fieldparams.BLSPubkeyLength]byte) {
	ctx, span := trace.StartSpan(ctx, "validator.SubmitSyncCommitteeMessage")
	defer span.End()
	span.AddAttributes(trace.StringAttribute("validator", fmt.Sprintf("%#x", pubKey)))

	v.waitOneThirdOrValidBlock(ctx, slot)

	res, err := v.validatorClient.GetSyncMessageBlockRoot(ctx, &emptypb.Empty{})
	if err != nil {
		log.WithError(err).Error("Could not request sync message block root to sign")
		tracing.AnnotateError(span, err)
		return
	}

	duty, err := v.duty(pubKey)
	if err != nil {
		log.WithError(err).Error("Could not fetch validator assignment")
		return
	}

	d, err := v.domainData(ctx, slots.ToEpoch(slot), params.BeaconConfig().DomainSyncCommittee[:])
	if err != nil {
		log.WithError(err).Error("Could not get sync committee domain data")
		return
	}
	sszRoot := primitives.SSZBytes(res.Root)
	r, err := signing.ComputeSigningRoot(&sszRoot, d.SignatureDomain)
	if err != nil {
		log.WithError(err).Error("Could not get sync committee message signing root")
		return
	}

	sig, err := v.keyManager.Sign(ctx, &validatorpb.SignRequest{
		PublicKey:       pubKey[:],
		SigningRoot:     r[:],
		SignatureDomain: d.SignatureDomain,
		Object: &validatorpb.SignRequest_SyncMessageBlockRoot{
			SyncMessageBlockRoot: res.Root,
		},
		SigningSlot: slot,
	})
	if err != nil {
		log.WithError(err).Error("Could not sign sync committee message")
		return
	}

	msg := &ethpb.SyncCommitteeMessage{
		Slot:           slot,
		BlockRoot:      res.Root,
		ValidatorIndex: duty.ValidatorIndex,
		Signature:      sig.Marshal(),
	}
	if _, err := v.validatorClient.SubmitSyncMessage(ctx, msg); err != nil {
		log.WithError(err).Error("Could not submit sync committee message")
		return
	}

	msgSlot := msg.Slot
	slotTime := time.Unix(int64(v.genesisTime+uint64(msgSlot)*params.BeaconConfig().SecondsPerSlot), 0)
	log.WithFields(logrus.Fields{
		"slot":               msg.Slot,
		"slotStartTime":      slotTime,
		"timeSinceSlotStart": time.Since(slotTime),
		"blockRoot":          fmt.Sprintf("%#x", bytesutil.Trunc(msg.BlockRoot)),
		"validatorIndex":     msg.ValidatorIndex,
	}).Info("Submitted new sync message")
	atomic.AddUint64(&v.syncCommitteeStats.totalMessagesSubmitted, 1)
}

// SubmitSignedContributionAndProof submits the signed sync committee contribution and proof to the beacon chain.
func (v *validator) SubmitSignedContributionAndProof(ctx context.Context, slot primitives.Slot, pubKey [fieldparams.BLSPubkeyLength]byte) {
	ctx, span := trace.StartSpan(ctx, "validator.SubmitSignedContributionAndProof")
	defer span.End()
	span.AddAttributes(trace.StringAttribute("validator", fmt.Sprintf("%#x", pubKey)))

	duty, err := v.duty(pubKey)
	if err != nil {
		log.WithError(err).Error("Could not fetch validator assignment")
		return
	}

	indexRes, err := v.validatorClient.GetSyncSubcommitteeIndex(ctx, &ethpb.SyncSubcommitteeIndexRequest{
		PublicKey: pubKey[:],
		Slot:      slot,
	})
	if err != nil {
		log.WithError(err).Error("Could not get sync subcommittee index")
		return
	}
	if len(indexRes.Indices) == 0 {
		log.Debug("Empty subcommittee index list, do nothing")
		return
	}

	selectionProofs, err := v.selectionProofs(ctx, slot, pubKey, indexRes)
	if err != nil {
		log.WithError(err).Error("Could not get selection proofs")
		return
	}

	v.waitToSlotTwoThirds(ctx, slot)

	for i, comIdx := range indexRes.Indices {
		isAggregator, err := altair.IsSyncCommitteeAggregator(selectionProofs[i])
		if err != nil {
			log.WithError(err).Error("Could check in aggregator")
			return
		}
		if !isAggregator {
			continue
		}
		subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
		subnet := uint64(comIdx) / subCommitteeSize
		contribution, err := v.validatorClient.GetSyncCommitteeContribution(ctx, &ethpb.SyncCommitteeContributionRequest{
			Slot:      slot,
			PublicKey: pubKey[:],
			SubnetId:  subnet,
		})
		if err != nil {
			log.WithError(err).Error("Could not get sync committee contribution")
			return
		}
		if contribution.AggregationBits.Count() == 0 {
			log.WithFields(logrus.Fields{
				"slot":   slot,
				"pubkey": hexutil.Encode(pubKey[:]),
				"subnet": subnet,
			}).Warn("Sync contribution for validator has no bits set.")
			continue
		}

		contributionAndProof := &ethpb.ContributionAndProof{
			AggregatorIndex: duty.ValidatorIndex,
			Contribution:    contribution,
			SelectionProof:  selectionProofs[i],
		}
		sig, err := v.signContributionAndProof(ctx, pubKey, contributionAndProof, slot)
		if err != nil {
			log.WithError(err).Error("Could not sign contribution and proof")
			return
		}

		if _, err := v.validatorClient.SubmitSignedContributionAndProof(ctx, &ethpb.SignedContributionAndProof{
			Message:   contributionAndProof,
			Signature: sig,
		}); err != nil {
			log.WithError(err).Error("Could not submit signed contribution and proof")
			return
		}

		contributionSlot := contributionAndProof.Contribution.Slot
		slotTime := time.Unix(int64(v.genesisTime+uint64(contributionSlot)*params.BeaconConfig().SecondsPerSlot), 0)
		log.WithFields(logrus.Fields{
			"slot":               contributionAndProof.Contribution.Slot,
			"slotStartTime":      slotTime,
			"timeSinceSlotStart": time.Since(slotTime),
			"blockRoot":          fmt.Sprintf("%#x", bytesutil.Trunc(contributionAndProof.Contribution.BlockRoot)),
			"subcommitteeIndex":  contributionAndProof.Contribution.SubcommitteeIndex,
			"aggregatorIndex":    contributionAndProof.AggregatorIndex,
			"bitsCount":          contributionAndProof.Contribution.AggregationBits.Count(),
		}).Info("Submitted new sync contribution and proof")
	}
}

// Signs and returns selection proofs per validator for slot and pub key.
func (v *validator) selectionProofs(ctx context.Context, slot primitives.Slot, pubKey [fieldparams.BLSPubkeyLength]byte, indexRes *ethpb.SyncSubcommitteeIndexResponse) ([][]byte, error) {
	selectionProofs := make([][]byte, len(indexRes.Indices))
	cfg := params.BeaconConfig()
	size := cfg.SyncCommitteeSize
	subCount := cfg.SyncCommitteeSubnetCount
	for i, index := range indexRes.Indices {
		subSize := size / subCount
		subnet := uint64(index) / subSize
		selectionProof, err := v.signSyncSelectionData(ctx, pubKey, subnet, slot)
		if err != nil {
			return nil, err
		}
		selectionProofs[i] = selectionProof
	}
	return selectionProofs, nil
}

// Signs input slot with domain sync committee selection proof. This is used to create the signature for sync committee selection.
func (v *validator) signSyncSelectionData(ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, index uint64, slot primitives.Slot) (signature []byte, err error) {
	domain, err := v.domainData(ctx, slots.ToEpoch(slot), params.BeaconConfig().DomainSyncCommitteeSelectionProof[:])
	if err != nil {
		return nil, err
	}
	data := &ethpb.SyncAggregatorSelectionData{
		Slot:              slot,
		SubcommitteeIndex: index,
	}
	root, err := signing.ComputeSigningRoot(data, domain.SignatureDomain)
	if err != nil {
		return nil, err
	}
	sig, err := v.keyManager.Sign(ctx, &validatorpb.SignRequest{
		PublicKey:       pubKey[:],
		SigningRoot:     root[:],
		SignatureDomain: domain.SignatureDomain,
		Object:          &validatorpb.SignRequest_SyncAggregatorSelectionData{SyncAggregatorSelectionData: data},
		SigningSlot:     slot,
	})
	if err != nil {
		return nil, err
	}
	return sig.Marshal(), nil
}

// This returns the signature of validator signing over sync committee contribution and proof object.
func (v *validator) signContributionAndProof(ctx context.Context, pubKey [fieldparams.BLSPubkeyLength]byte, c *ethpb.ContributionAndProof, slot primitives.Slot) ([]byte, error) {
	d, err := v.domainData(ctx, slots.ToEpoch(c.Contribution.Slot), params.BeaconConfig().DomainContributionAndProof[:])
	if err != nil {
		return nil, err
	}
	root, err := signing.ComputeSigningRoot(c, d.SignatureDomain)
	if err != nil {
		return nil, err
	}
	sig, err := v.keyManager.Sign(ctx, &validatorpb.SignRequest{
		PublicKey:       pubKey[:],
		SigningRoot:     root[:],
		SignatureDomain: d.SignatureDomain,
		Object:          &validatorpb.SignRequest_ContributionAndProof{ContributionAndProof: c},
		SigningSlot:     slot,
	})
	if err != nil {
		return nil, err
	}
	return sig.Marshal(), nil
}
