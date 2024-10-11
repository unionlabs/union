package shared

import (
	"fmt"
	"strconv"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/container/slice"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/math"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var errNilValue = errors.New("nil value")

func ValidatorFromConsensus(v *eth.Validator) *Validator {
	return &Validator{
		PublicKey:                  hexutil.Encode(v.PublicKey),
		WithdrawalCredentials:      hexutil.Encode(v.WithdrawalCredentials),
		EffectiveBalance:           fmt.Sprintf("%d", v.EffectiveBalance),
		Slashed:                    v.Slashed,
		ActivationEligibilityEpoch: fmt.Sprintf("%d", v.ActivationEligibilityEpoch),
		ActivationEpoch:            fmt.Sprintf("%d", v.ActivationEpoch),
		ExitEpoch:                  fmt.Sprintf("%d", v.ExitEpoch),
		WithdrawableEpoch:          fmt.Sprintf("%d", v.WithdrawableEpoch),
	}
}

func PendingAttestationFromConsensus(a *eth.PendingAttestation) *PendingAttestation {
	return &PendingAttestation{
		AggregationBits: hexutil.Encode(a.AggregationBits),
		Data:            AttDataFromConsensus(a.Data),
		InclusionDelay:  fmt.Sprintf("%d", a.InclusionDelay),
		ProposerIndex:   fmt.Sprintf("%d", a.ProposerIndex),
	}
}

func HistoricalSummaryFromConsensus(s *eth.HistoricalSummary) *HistoricalSummary {
	return &HistoricalSummary{
		BlockSummaryRoot: hexutil.Encode(s.BlockSummaryRoot),
		StateSummaryRoot: hexutil.Encode(s.StateSummaryRoot),
	}
}

func (s *SignedBLSToExecutionChange) ToConsensus() (*eth.SignedBLSToExecutionChange, error) {
	change, err := s.Message.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Message")
	}
	sig, err := bytesutil.DecodeHexWithLength(s.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}
	return &eth.SignedBLSToExecutionChange{
		Message:   change,
		Signature: sig,
	}, nil
}

func (b *BLSToExecutionChange) ToConsensus() (*eth.BLSToExecutionChange, error) {
	index, err := strconv.ParseUint(b.ValidatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "ValidatorIndex")
	}
	pubkey, err := bytesutil.DecodeHexWithLength(b.FromBLSPubkey, fieldparams.BLSPubkeyLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "FromBLSPubkey")
	}
	executionAddress, err := bytesutil.DecodeHexWithLength(b.ToExecutionAddress, common.AddressLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "ToExecutionAddress")
	}
	return &eth.BLSToExecutionChange{
		ValidatorIndex:     primitives.ValidatorIndex(index),
		FromBlsPubkey:      pubkey,
		ToExecutionAddress: executionAddress,
	}, nil
}

func BLSChangeFromConsensus(ch *eth.BLSToExecutionChange) *BLSToExecutionChange {
	return &BLSToExecutionChange{
		ValidatorIndex:     fmt.Sprintf("%d", ch.ValidatorIndex),
		FromBLSPubkey:      hexutil.Encode(ch.FromBlsPubkey),
		ToExecutionAddress: hexutil.Encode(ch.ToExecutionAddress),
	}
}

func SignedBLSChangeFromConsensus(ch *eth.SignedBLSToExecutionChange) *SignedBLSToExecutionChange {
	return &SignedBLSToExecutionChange{
		Message:   BLSChangeFromConsensus(ch.Message),
		Signature: hexutil.Encode(ch.Signature),
	}
}

func SignedBLSChangesToConsensus(src []*SignedBLSToExecutionChange) ([]*eth.SignedBLSToExecutionChange, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 16)
	if err != nil {
		return nil, err
	}
	changes := make([]*eth.SignedBLSToExecutionChange, len(src))
	for i, ch := range src {
		changes[i], err = ch.ToConsensus()
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d]", i))
		}
	}
	return changes, nil
}

func SignedBLSChangesFromConsensus(src []*eth.SignedBLSToExecutionChange) []*SignedBLSToExecutionChange {
	changes := make([]*SignedBLSToExecutionChange, len(src))
	for i, ch := range src {
		changes[i] = SignedBLSChangeFromConsensus(ch)
	}
	return changes
}

func (s *Fork) ToConsensus() (*eth.Fork, error) {
	previousVersion, err := bytesutil.DecodeHexWithLength(s.PreviousVersion, 4)
	if err != nil {
		return nil, server.NewDecodeError(err, "PreviousVersion")
	}
	currentVersion, err := bytesutil.DecodeHexWithLength(s.CurrentVersion, 4)
	if err != nil {
		return nil, server.NewDecodeError(err, "CurrentVersion")
	}
	epoch, err := strconv.ParseUint(s.Epoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Epoch")
	}
	return &eth.Fork{
		PreviousVersion: previousVersion,
		CurrentVersion:  currentVersion,
		Epoch:           primitives.Epoch(epoch),
	}, nil
}

func ForkFromConsensus(f *eth.Fork) *Fork {
	return &Fork{
		PreviousVersion: hexutil.Encode(f.PreviousVersion),
		CurrentVersion:  hexutil.Encode(f.CurrentVersion),
		Epoch:           fmt.Sprintf("%d", f.Epoch),
	}
}

func (s *SignedValidatorRegistration) ToConsensus() (*eth.SignedValidatorRegistrationV1, error) {
	msg, err := s.Message.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Message")
	}
	sig, err := bytesutil.DecodeHexWithLength(s.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}
	return &eth.SignedValidatorRegistrationV1{
		Message:   msg,
		Signature: sig,
	}, nil
}

func (s *ValidatorRegistration) ToConsensus() (*eth.ValidatorRegistrationV1, error) {
	feeRecipient, err := bytesutil.DecodeHexWithLength(s.FeeRecipient, fieldparams.FeeRecipientLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "FeeRecipient")
	}
	pubKey, err := bytesutil.DecodeHexWithLength(s.Pubkey, fieldparams.BLSPubkeyLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Pubkey")
	}
	gasLimit, err := strconv.ParseUint(s.GasLimit, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "GasLimit")
	}
	timestamp, err := strconv.ParseUint(s.Timestamp, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Timestamp")
	}
	return &eth.ValidatorRegistrationV1{
		FeeRecipient: feeRecipient,
		GasLimit:     gasLimit,
		Timestamp:    timestamp,
		Pubkey:       pubKey,
	}, nil
}

func ValidatorRegistrationFromConsensus(vr *eth.ValidatorRegistrationV1) *ValidatorRegistration {
	return &ValidatorRegistration{
		FeeRecipient: hexutil.Encode(vr.FeeRecipient),
		GasLimit:     fmt.Sprintf("%d", vr.GasLimit),
		Timestamp:    fmt.Sprintf("%d", vr.Timestamp),
		Pubkey:       hexutil.Encode(vr.Pubkey),
	}
}

func SignedValidatorRegistrationFromConsensus(vr *eth.SignedValidatorRegistrationV1) *SignedValidatorRegistration {
	return &SignedValidatorRegistration{
		Message:   ValidatorRegistrationFromConsensus(vr.Message),
		Signature: hexutil.Encode(vr.Signature),
	}
}

func (s *SignedContributionAndProof) ToConsensus() (*eth.SignedContributionAndProof, error) {
	msg, err := s.Message.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Message")
	}
	sig, err := bytesutil.DecodeHexWithLength(s.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.SignedContributionAndProof{
		Message:   msg,
		Signature: sig,
	}, nil
}

func SignedContributionAndProofFromConsensus(c *eth.SignedContributionAndProof) *SignedContributionAndProof {
	contribution := ContributionAndProofFromConsensus(c.Message)
	return &SignedContributionAndProof{
		Message:   contribution,
		Signature: hexutil.Encode(c.Signature),
	}
}

func (c *ContributionAndProof) ToConsensus() (*eth.ContributionAndProof, error) {
	contribution, err := c.Contribution.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Contribution")
	}
	aggregatorIndex, err := strconv.ParseUint(c.AggregatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "AggregatorIndex")
	}
	selectionProof, err := bytesutil.DecodeHexWithLength(c.SelectionProof, 96)
	if err != nil {
		return nil, server.NewDecodeError(err, "SelectionProof")
	}

	return &eth.ContributionAndProof{
		AggregatorIndex: primitives.ValidatorIndex(aggregatorIndex),
		Contribution:    contribution,
		SelectionProof:  selectionProof,
	}, nil
}

func ContributionAndProofFromConsensus(c *eth.ContributionAndProof) *ContributionAndProof {
	contribution := SyncCommitteeContributionFromConsensus(c.Contribution)
	return &ContributionAndProof{
		AggregatorIndex: fmt.Sprintf("%d", c.AggregatorIndex),
		Contribution:    contribution,
		SelectionProof:  hexutil.Encode(c.SelectionProof),
	}
}

func (s *SyncCommitteeContribution) ToConsensus() (*eth.SyncCommitteeContribution, error) {
	slot, err := strconv.ParseUint(s.Slot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Slot")
	}
	bbRoot, err := bytesutil.DecodeHexWithLength(s.BeaconBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "BeaconBlockRoot")
	}
	subcommitteeIndex, err := strconv.ParseUint(s.SubcommitteeIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "SubcommitteeIndex")
	}
	aggBits, err := hexutil.Decode(s.AggregationBits)
	if err != nil {
		return nil, server.NewDecodeError(err, "AggregationBits")
	}
	sig, err := bytesutil.DecodeHexWithLength(s.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.SyncCommitteeContribution{
		Slot:              primitives.Slot(slot),
		BlockRoot:         bbRoot,
		SubcommitteeIndex: subcommitteeIndex,
		AggregationBits:   aggBits,
		Signature:         sig,
	}, nil
}

func SyncCommitteeContributionFromConsensus(c *eth.SyncCommitteeContribution) *SyncCommitteeContribution {
	return &SyncCommitteeContribution{
		Slot:              fmt.Sprintf("%d", c.Slot),
		BeaconBlockRoot:   hexutil.Encode(c.BlockRoot),
		SubcommitteeIndex: fmt.Sprintf("%d", c.SubcommitteeIndex),
		AggregationBits:   hexutil.Encode(c.AggregationBits),
		Signature:         hexutil.Encode(c.Signature),
	}
}

func (s *SignedAggregateAttestationAndProof) ToConsensus() (*eth.SignedAggregateAttestationAndProof, error) {
	msg, err := s.Message.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Message")
	}
	sig, err := bytesutil.DecodeHexWithLength(s.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.SignedAggregateAttestationAndProof{
		Message:   msg,
		Signature: sig,
	}, nil
}

func (a *AggregateAttestationAndProof) ToConsensus() (*eth.AggregateAttestationAndProof, error) {
	aggIndex, err := strconv.ParseUint(a.AggregatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "AggregatorIndex")
	}
	agg, err := a.Aggregate.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Aggregate")
	}
	proof, err := bytesutil.DecodeHexWithLength(a.SelectionProof, 96)
	if err != nil {
		return nil, server.NewDecodeError(err, "SelectionProof")
	}
	return &eth.AggregateAttestationAndProof{
		AggregatorIndex: primitives.ValidatorIndex(aggIndex),
		Aggregate:       agg,
		SelectionProof:  proof,
	}, nil
}

func (a *Attestation) ToConsensus() (*eth.Attestation, error) {
	aggBits, err := hexutil.Decode(a.AggregationBits)
	if err != nil {
		return nil, server.NewDecodeError(err, "AggregationBits")
	}
	data, err := a.Data.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Data")
	}
	sig, err := bytesutil.DecodeHexWithLength(a.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.Attestation{
		AggregationBits: aggBits,
		Data:            data,
		Signature:       sig,
	}, nil
}

func AttFromConsensus(a *eth.Attestation) *Attestation {
	return &Attestation{
		AggregationBits: hexutil.Encode(a.AggregationBits),
		Data:            AttDataFromConsensus(a.Data),
		Signature:       hexutil.Encode(a.Signature),
	}
}

func (a *AttestationData) ToConsensus() (*eth.AttestationData, error) {
	slot, err := strconv.ParseUint(a.Slot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Slot")
	}
	committeeIndex, err := strconv.ParseUint(a.CommitteeIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "CommitteeIndex")
	}
	bbRoot, err := bytesutil.DecodeHexWithLength(a.BeaconBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "BeaconBlockRoot")
	}
	source, err := a.Source.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Source")
	}
	target, err := a.Target.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Target")
	}

	return &eth.AttestationData{
		Slot:            primitives.Slot(slot),
		CommitteeIndex:  primitives.CommitteeIndex(committeeIndex),
		BeaconBlockRoot: bbRoot,
		Source:          source,
		Target:          target,
	}, nil
}

func AttDataFromConsensus(a *eth.AttestationData) *AttestationData {
	return &AttestationData{
		Slot:            fmt.Sprintf("%d", a.Slot),
		CommitteeIndex:  fmt.Sprintf("%d", a.CommitteeIndex),
		BeaconBlockRoot: hexutil.Encode(a.BeaconBlockRoot),
		Source:          CheckpointFromConsensus(a.Source),
		Target:          CheckpointFromConsensus(a.Target),
	}
}

func (c *Checkpoint) ToConsensus() (*eth.Checkpoint, error) {
	epoch, err := strconv.ParseUint(c.Epoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Epoch")
	}
	root, err := bytesutil.DecodeHexWithLength(c.Root, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Root")
	}

	return &eth.Checkpoint{
		Epoch: primitives.Epoch(epoch),
		Root:  root,
	}, nil
}

func CheckpointFromConsensus(c *eth.Checkpoint) *Checkpoint {
	return &Checkpoint{
		Epoch: fmt.Sprintf("%d", c.Epoch),
		Root:  hexutil.Encode(c.Root),
	}
}

func (s *SyncCommitteeSubscription) ToConsensus() (*validator.SyncCommitteeSubscription, error) {
	index, err := strconv.ParseUint(s.ValidatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "ValidatorIndex")
	}
	scIndices := make([]uint64, len(s.SyncCommitteeIndices))
	for i, ix := range s.SyncCommitteeIndices {
		scIndices[i], err = strconv.ParseUint(ix, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("SyncCommitteeIndices[%d]", i))
		}
	}
	epoch, err := strconv.ParseUint(s.UntilEpoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "UntilEpoch")
	}

	return &validator.SyncCommitteeSubscription{
		ValidatorIndex:       primitives.ValidatorIndex(index),
		SyncCommitteeIndices: scIndices,
		UntilEpoch:           primitives.Epoch(epoch),
	}, nil
}

func (b *BeaconCommitteeSubscription) ToConsensus() (*validator.BeaconCommitteeSubscription, error) {
	valIndex, err := strconv.ParseUint(b.ValidatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "ValidatorIndex")
	}
	committeeIndex, err := strconv.ParseUint(b.CommitteeIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "CommitteeIndex")
	}
	committeesAtSlot, err := strconv.ParseUint(b.CommitteesAtSlot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "CommitteesAtSlot")
	}
	slot, err := strconv.ParseUint(b.Slot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Slot")
	}

	return &validator.BeaconCommitteeSubscription{
		ValidatorIndex:   primitives.ValidatorIndex(valIndex),
		CommitteeIndex:   primitives.CommitteeIndex(committeeIndex),
		CommitteesAtSlot: committeesAtSlot,
		Slot:             primitives.Slot(slot),
		IsAggregator:     b.IsAggregator,
	}, nil
}

func (e *SignedVoluntaryExit) ToConsensus() (*eth.SignedVoluntaryExit, error) {
	sig, err := bytesutil.DecodeHexWithLength(e.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}
	exit, err := e.Message.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Message")
	}

	return &eth.SignedVoluntaryExit{
		Exit:      exit,
		Signature: sig,
	}, nil
}

func SignedExitFromConsensus(e *eth.SignedVoluntaryExit) *SignedVoluntaryExit {
	return &SignedVoluntaryExit{
		Message:   ExitFromConsensus(e.Exit),
		Signature: hexutil.Encode(e.Signature),
	}
}

func (e *VoluntaryExit) ToConsensus() (*eth.VoluntaryExit, error) {
	epoch, err := strconv.ParseUint(e.Epoch, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Epoch")
	}
	valIndex, err := strconv.ParseUint(e.ValidatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "ValidatorIndex")
	}

	return &eth.VoluntaryExit{
		Epoch:          primitives.Epoch(epoch),
		ValidatorIndex: primitives.ValidatorIndex(valIndex),
	}, nil
}

func ExitFromConsensus(e *eth.VoluntaryExit) *VoluntaryExit {
	return &VoluntaryExit{
		Epoch:          fmt.Sprintf("%d", e.Epoch),
		ValidatorIndex: fmt.Sprintf("%d", e.ValidatorIndex),
	}
}

func (m *SyncCommitteeMessage) ToConsensus() (*eth.SyncCommitteeMessage, error) {
	slot, err := strconv.ParseUint(m.Slot, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "Slot")
	}
	root, err := bytesutil.DecodeHexWithLength(m.BeaconBlockRoot, fieldparams.RootLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "BeaconBlockRoot")
	}
	valIndex, err := strconv.ParseUint(m.ValidatorIndex, 10, 64)
	if err != nil {
		return nil, server.NewDecodeError(err, "ValidatorIndex")
	}
	sig, err := bytesutil.DecodeHexWithLength(m.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.SyncCommitteeMessage{
		Slot:           primitives.Slot(slot),
		BlockRoot:      root,
		ValidatorIndex: primitives.ValidatorIndex(valIndex),
		Signature:      sig,
	}, nil
}

func SyncCommitteeFromConsensus(sc *eth.SyncCommittee) *SyncCommittee {
	var sPubKeys []string
	for _, p := range sc.Pubkeys {
		sPubKeys = append(sPubKeys, hexutil.Encode(p))
	}

	return &SyncCommittee{
		Pubkeys:         sPubKeys,
		AggregatePubkey: hexutil.Encode(sc.AggregatePubkey),
	}
}

func (sc *SyncCommittee) ToConsensus() (*eth.SyncCommittee, error) {
	var pubKeys [][]byte
	for _, p := range sc.Pubkeys {
		pubKey, err := bytesutil.DecodeHexWithLength(p, fieldparams.BLSPubkeyLength)
		if err != nil {
			return nil, server.NewDecodeError(err, "Pubkeys")
		}
		pubKeys = append(pubKeys, pubKey)
	}
	aggPubKey, err := bytesutil.DecodeHexWithLength(sc.AggregatePubkey, fieldparams.BLSPubkeyLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "AggregatePubkey")
	}
	return &eth.SyncCommittee{
		Pubkeys:         pubKeys,
		AggregatePubkey: aggPubKey,
	}, nil
}

func Eth1DataFromConsensus(e1d *eth.Eth1Data) *Eth1Data {
	return &Eth1Data{
		DepositRoot:  hexutil.Encode(e1d.DepositRoot),
		DepositCount: fmt.Sprintf("%d", e1d.DepositCount),
		BlockHash:    hexutil.Encode(e1d.BlockHash),
	}
}

func (s *ProposerSlashing) ToConsensus() (*eth.ProposerSlashing, error) {
	h1, err := s.SignedHeader1.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "SignedHeader1")
	}
	h2, err := s.SignedHeader2.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "SignedHeader2")
	}

	return &eth.ProposerSlashing{
		Header_1: h1,
		Header_2: h2,
	}, nil
}

func (s *AttesterSlashing) ToConsensus() (*eth.AttesterSlashing, error) {
	att1, err := s.Attestation1.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Attestation1")
	}
	att2, err := s.Attestation2.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Attestation2")
	}
	return &eth.AttesterSlashing{Attestation_1: att1, Attestation_2: att2}, nil
}

func (a *IndexedAttestation) ToConsensus() (*eth.IndexedAttestation, error) {
	indices := make([]uint64, len(a.AttestingIndices))
	var err error
	for i, ix := range a.AttestingIndices {
		indices[i], err = strconv.ParseUint(ix, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("AttestingIndices[%d]", i))
		}
	}
	data, err := a.Data.ToConsensus()
	if err != nil {
		return nil, server.NewDecodeError(err, "Data")
	}
	sig, err := bytesutil.DecodeHexWithLength(a.Signature, fieldparams.BLSSignatureLength)
	if err != nil {
		return nil, server.NewDecodeError(err, "Signature")
	}

	return &eth.IndexedAttestation{
		AttestingIndices: indices,
		Data:             data,
		Signature:        sig,
	}, nil
}

func WithdrawalsFromConsensus(ws []*enginev1.Withdrawal) []*Withdrawal {
	result := make([]*Withdrawal, len(ws))
	for i, w := range ws {
		result[i] = WithdrawalFromConsensus(w)
	}
	return result
}

func WithdrawalFromConsensus(w *enginev1.Withdrawal) *Withdrawal {
	return &Withdrawal{
		WithdrawalIndex:  fmt.Sprintf("%d", w.Index),
		ValidatorIndex:   fmt.Sprintf("%d", w.ValidatorIndex),
		ExecutionAddress: hexutil.Encode(w.Address),
		Amount:           fmt.Sprintf("%d", w.Amount),
	}
}

func ProposerSlashingsToConsensus(src []*ProposerSlashing) ([]*eth.ProposerSlashing, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 16)
	if err != nil {
		return nil, err
	}
	proposerSlashings := make([]*eth.ProposerSlashing, len(src))
	for i, s := range src {
		if s == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d]", i))
		}
		if s.SignedHeader1 == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].SignedHeader1", i))
		}
		if s.SignedHeader1.Message == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].SignedHeader1.Message", i))
		}
		if s.SignedHeader2 == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].SignedHeader2", i))
		}
		if s.SignedHeader2.Message == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].SignedHeader2.Message", i))
		}

		h1Sig, err := bytesutil.DecodeHexWithLength(s.SignedHeader1.Signature, fieldparams.BLSSignatureLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Signature", i))
		}
		h1Slot, err := strconv.ParseUint(s.SignedHeader1.Message.Slot, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Message.Slot", i))
		}
		h1ProposerIndex, err := strconv.ParseUint(s.SignedHeader1.Message.ProposerIndex, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Message.ProposerIndex", i))
		}
		h1ParentRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader1.Message.ParentRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Message.ParentRoot", i))
		}
		h1StateRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader1.Message.StateRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Message.StateRoot", i))
		}
		h1BodyRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader1.Message.BodyRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader1.Message.BodyRoot", i))
		}
		h2Sig, err := bytesutil.DecodeHexWithLength(s.SignedHeader2.Signature, fieldparams.BLSSignatureLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Signature", i))
		}
		h2Slot, err := strconv.ParseUint(s.SignedHeader2.Message.Slot, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Message.Slot", i))
		}
		h2ProposerIndex, err := strconv.ParseUint(s.SignedHeader2.Message.ProposerIndex, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Message.ProposerIndex", i))
		}
		h2ParentRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader2.Message.ParentRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Message.ParentRoot", i))
		}
		h2StateRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader2.Message.StateRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Message.StateRoot", i))
		}
		h2BodyRoot, err := bytesutil.DecodeHexWithLength(s.SignedHeader2.Message.BodyRoot, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].SignedHeader2.Message.BodyRoot", i))
		}
		proposerSlashings[i] = &eth.ProposerSlashing{
			Header_1: &eth.SignedBeaconBlockHeader{
				Header: &eth.BeaconBlockHeader{
					Slot:          primitives.Slot(h1Slot),
					ProposerIndex: primitives.ValidatorIndex(h1ProposerIndex),
					ParentRoot:    h1ParentRoot,
					StateRoot:     h1StateRoot,
					BodyRoot:      h1BodyRoot,
				},
				Signature: h1Sig,
			},
			Header_2: &eth.SignedBeaconBlockHeader{
				Header: &eth.BeaconBlockHeader{
					Slot:          primitives.Slot(h2Slot),
					ProposerIndex: primitives.ValidatorIndex(h2ProposerIndex),
					ParentRoot:    h2ParentRoot,
					StateRoot:     h2StateRoot,
					BodyRoot:      h2BodyRoot,
				},
				Signature: h2Sig,
			},
		}
	}
	return proposerSlashings, nil
}

func ProposerSlashingsFromConsensus(src []*eth.ProposerSlashing) []*ProposerSlashing {
	proposerSlashings := make([]*ProposerSlashing, len(src))
	for i, s := range src {
		proposerSlashings[i] = ProposerSlashingFromConsensus(s)
	}
	return proposerSlashings
}

func ProposerSlashingFromConsensus(src *eth.ProposerSlashing) *ProposerSlashing {
	return &ProposerSlashing{
		SignedHeader1: &SignedBeaconBlockHeader{
			Message: &BeaconBlockHeader{
				Slot:          fmt.Sprintf("%d", src.Header_1.Header.Slot),
				ProposerIndex: fmt.Sprintf("%d", src.Header_1.Header.ProposerIndex),
				ParentRoot:    hexutil.Encode(src.Header_1.Header.ParentRoot),
				StateRoot:     hexutil.Encode(src.Header_1.Header.StateRoot),
				BodyRoot:      hexutil.Encode(src.Header_1.Header.BodyRoot),
			},
			Signature: hexutil.Encode(src.Header_1.Signature),
		},
		SignedHeader2: &SignedBeaconBlockHeader{
			Message: &BeaconBlockHeader{
				Slot:          fmt.Sprintf("%d", src.Header_2.Header.Slot),
				ProposerIndex: fmt.Sprintf("%d", src.Header_2.Header.ProposerIndex),
				ParentRoot:    hexutil.Encode(src.Header_2.Header.ParentRoot),
				StateRoot:     hexutil.Encode(src.Header_2.Header.StateRoot),
				BodyRoot:      hexutil.Encode(src.Header_2.Header.BodyRoot),
			},
			Signature: hexutil.Encode(src.Header_2.Signature),
		},
	}
}

func AttesterSlashingsToConsensus(src []*AttesterSlashing) ([]*eth.AttesterSlashing, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 2)
	if err != nil {
		return nil, err
	}

	attesterSlashings := make([]*eth.AttesterSlashing, len(src))
	for i, s := range src {
		if s == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d]", i))
		}
		if s.Attestation1 == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].Attestation1", i))
		}
		if s.Attestation2 == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].Attestation2", i))
		}

		a1Sig, err := bytesutil.DecodeHexWithLength(s.Attestation1.Signature, fieldparams.BLSSignatureLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation1.Signature", i))
		}
		err = slice.VerifyMaxLength(s.Attestation1.AttestingIndices, 2048)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation1.AttestingIndices", i))
		}
		a1AttestingIndices := make([]uint64, len(s.Attestation1.AttestingIndices))
		for j, ix := range s.Attestation1.AttestingIndices {
			attestingIndex, err := strconv.ParseUint(ix, 10, 64)
			if err != nil {
				return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation1.AttestingIndices[%d]", i, j))
			}
			a1AttestingIndices[j] = attestingIndex
		}
		a1Data, err := s.Attestation1.Data.ToConsensus()
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation1.Data", i))
		}
		a2Sig, err := bytesutil.DecodeHexWithLength(s.Attestation2.Signature, fieldparams.BLSSignatureLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation2.Signature", i))
		}
		err = slice.VerifyMaxLength(s.Attestation2.AttestingIndices, 2048)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation2.AttestingIndices", i))
		}
		a2AttestingIndices := make([]uint64, len(s.Attestation2.AttestingIndices))
		for j, ix := range s.Attestation2.AttestingIndices {
			attestingIndex, err := strconv.ParseUint(ix, 10, 64)
			if err != nil {
				return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation2.AttestingIndices[%d]", i, j))
			}
			a2AttestingIndices[j] = attestingIndex
		}
		a2Data, err := s.Attestation2.Data.ToConsensus()
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Attestation2.Data", i))
		}
		attesterSlashings[i] = &eth.AttesterSlashing{
			Attestation_1: &eth.IndexedAttestation{
				AttestingIndices: a1AttestingIndices,
				Data:             a1Data,
				Signature:        a1Sig,
			},
			Attestation_2: &eth.IndexedAttestation{
				AttestingIndices: a2AttestingIndices,
				Data:             a2Data,
				Signature:        a2Sig,
			},
		}
	}
	return attesterSlashings, nil
}

func AttesterSlashingsFromConsensus(src []*eth.AttesterSlashing) []*AttesterSlashing {
	attesterSlashings := make([]*AttesterSlashing, len(src))
	for i, s := range src {
		attesterSlashings[i] = AttesterSlashingFromConsensus(s)
	}
	return attesterSlashings
}

func AttesterSlashingFromConsensus(src *eth.AttesterSlashing) *AttesterSlashing {
	a1AttestingIndices := make([]string, len(src.Attestation_1.AttestingIndices))
	for j, ix := range src.Attestation_1.AttestingIndices {
		a1AttestingIndices[j] = fmt.Sprintf("%d", ix)
	}
	a2AttestingIndices := make([]string, len(src.Attestation_2.AttestingIndices))
	for j, ix := range src.Attestation_2.AttestingIndices {
		a2AttestingIndices[j] = fmt.Sprintf("%d", ix)
	}
	return &AttesterSlashing{
		Attestation1: &IndexedAttestation{
			AttestingIndices: a1AttestingIndices,
			Data: &AttestationData{
				Slot:            fmt.Sprintf("%d", src.Attestation_1.Data.Slot),
				CommitteeIndex:  fmt.Sprintf("%d", src.Attestation_1.Data.CommitteeIndex),
				BeaconBlockRoot: hexutil.Encode(src.Attestation_1.Data.BeaconBlockRoot),
				Source: &Checkpoint{
					Epoch: fmt.Sprintf("%d", src.Attestation_1.Data.Source.Epoch),
					Root:  hexutil.Encode(src.Attestation_1.Data.Source.Root),
				},
				Target: &Checkpoint{
					Epoch: fmt.Sprintf("%d", src.Attestation_1.Data.Target.Epoch),
					Root:  hexutil.Encode(src.Attestation_1.Data.Target.Root),
				},
			},
			Signature: hexutil.Encode(src.Attestation_1.Signature),
		},
		Attestation2: &IndexedAttestation{
			AttestingIndices: a2AttestingIndices,
			Data: &AttestationData{
				Slot:            fmt.Sprintf("%d", src.Attestation_2.Data.Slot),
				CommitteeIndex:  fmt.Sprintf("%d", src.Attestation_2.Data.CommitteeIndex),
				BeaconBlockRoot: hexutil.Encode(src.Attestation_2.Data.BeaconBlockRoot),
				Source: &Checkpoint{
					Epoch: fmt.Sprintf("%d", src.Attestation_2.Data.Source.Epoch),
					Root:  hexutil.Encode(src.Attestation_2.Data.Source.Root),
				},
				Target: &Checkpoint{
					Epoch: fmt.Sprintf("%d", src.Attestation_2.Data.Target.Epoch),
					Root:  hexutil.Encode(src.Attestation_2.Data.Target.Root),
				},
			},
			Signature: hexutil.Encode(src.Attestation_2.Signature),
		},
	}
}

func AttsToConsensus(src []*Attestation) ([]*eth.Attestation, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 128)
	if err != nil {
		return nil, err
	}

	atts := make([]*eth.Attestation, len(src))
	for i, a := range src {
		atts[i], err = a.ToConsensus()
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d]", i))
		}
	}
	return atts, nil
}

func AttsFromConsensus(src []*eth.Attestation) []*Attestation {
	atts := make([]*Attestation, len(src))
	for i, a := range src {
		atts[i] = AttFromConsensus(a)
	}
	return atts
}

func DepositsToConsensus(src []*Deposit) ([]*eth.Deposit, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 16)
	if err != nil {
		return nil, err
	}

	deposits := make([]*eth.Deposit, len(src))
	for i, d := range src {
		if d.Data == nil {
			return nil, server.NewDecodeError(errNilValue, fmt.Sprintf("[%d].Data", i))
		}

		err = slice.VerifyMaxLength(d.Proof, 33)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Proof", i))
		}
		proof := make([][]byte, len(d.Proof))
		for j, p := range d.Proof {
			var err error
			proof[j], err = bytesutil.DecodeHexWithLength(p, fieldparams.RootLength)
			if err != nil {
				return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Proof[%d]", i, j))
			}
		}
		pubkey, err := bytesutil.DecodeHexWithLength(d.Data.Pubkey, fieldparams.BLSPubkeyLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Pubkey", i))
		}
		withdrawalCreds, err := bytesutil.DecodeHexWithLength(d.Data.WithdrawalCredentials, fieldparams.RootLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].WithdrawalCredentials", i))
		}
		amount, err := strconv.ParseUint(d.Data.Amount, 10, 64)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Amount", i))
		}
		sig, err := bytesutil.DecodeHexWithLength(d.Data.Signature, fieldparams.BLSSignatureLength)
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d].Signature", i))
		}
		deposits[i] = &eth.Deposit{
			Proof: proof,
			Data: &eth.Deposit_Data{
				PublicKey:             pubkey,
				WithdrawalCredentials: withdrawalCreds,
				Amount:                amount,
				Signature:             sig,
			},
		}
	}
	return deposits, nil
}

func DepositsFromConsensus(src []*eth.Deposit) []*Deposit {
	deposits := make([]*Deposit, len(src))
	for i, d := range src {
		proof := make([]string, len(d.Proof))
		for j, p := range d.Proof {
			proof[j] = hexutil.Encode(p)
		}
		deposits[i] = &Deposit{
			Proof: proof,
			Data: &DepositData{
				Pubkey:                hexutil.Encode(d.Data.PublicKey),
				WithdrawalCredentials: hexutil.Encode(d.Data.WithdrawalCredentials),
				Amount:                fmt.Sprintf("%d", d.Data.Amount),
				Signature:             hexutil.Encode(d.Data.Signature),
			},
		}
	}
	return deposits
}

func SignedExitsToConsensus(src []*SignedVoluntaryExit) ([]*eth.SignedVoluntaryExit, error) {
	if src == nil {
		return nil, errNilValue
	}
	err := slice.VerifyMaxLength(src, 16)
	if err != nil {
		return nil, err
	}

	exits := make([]*eth.SignedVoluntaryExit, len(src))
	for i, e := range src {
		exits[i], err = e.ToConsensus()
		if err != nil {
			return nil, server.NewDecodeError(err, fmt.Sprintf("[%d]", i))
		}
	}
	return exits, nil
}

func SignedExitsFromConsensus(src []*eth.SignedVoluntaryExit) []*SignedVoluntaryExit {
	exits := make([]*SignedVoluntaryExit, len(src))
	for i, e := range src {
		exits[i] = &SignedVoluntaryExit{
			Message: &VoluntaryExit{
				Epoch:          fmt.Sprintf("%d", e.Exit.Epoch),
				ValidatorIndex: fmt.Sprintf("%d", e.Exit.ValidatorIndex),
			},
			Signature: hexutil.Encode(e.Signature),
		}
	}
	return exits
}

func sszBytesToUint256String(b []byte) (string, error) {
	bi := bytesutil.LittleEndianBytesToBigInt(b)
	if !math.IsValidUint256(bi) {
		return "", fmt.Errorf("%s is not a valid Uint256", bi.String())
	}
	return bi.String(), nil
}
