package v1

import (
	"fmt"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// MapForkInfo maps the eth2.ForkInfo proto to the Web3Signer spec.
func MapForkInfo(slot primitives.Slot, genesisValidatorsRoot []byte) (*ForkInfo, error) {
	fork, err := forks.Fork(slots.ToEpoch(slot))
	if err != nil {
		return nil, errors.Wrap(err, "could not get fork info")
	}
	forkData := &Fork{
		PreviousVersion: fork.PreviousVersion,
		CurrentVersion:  fork.CurrentVersion,
		Epoch:           fmt.Sprint(fork.Epoch),
	}
	return &ForkInfo{
		Fork:                  forkData,
		GenesisValidatorsRoot: genesisValidatorsRoot,
	}, nil
}

// MapAggregateAndProof maps the eth2.AggregateAndProof proto to the Web3Signer spec.
func MapAggregateAndProof(from *ethpb.AggregateAttestationAndProof) (*AggregateAndProof, error) {
	if from == nil {
		return nil, fmt.Errorf("AggregateAttestationAndProof is nil")
	}
	aggregate, err := MapAttestation(from.Aggregate)
	if err != nil {
		return nil, err
	}
	return &AggregateAndProof{
		AggregatorIndex: fmt.Sprint(from.AggregatorIndex),
		Aggregate:       aggregate,
		SelectionProof:  from.SelectionProof,
	}, nil
}

// MapAttestation maps the eth2.Attestation proto to the Web3Signer spec.
func MapAttestation(attestation *ethpb.Attestation) (*Attestation, error) {
	if attestation == nil {
		return nil, fmt.Errorf("attestation is nil")
	}
	if attestation.AggregationBits == nil {
		return nil, fmt.Errorf("aggregation bits in attestation is nil")
	}
	data, err := MapAttestationData(attestation.Data)
	if err != nil {
		return nil, err
	}
	return &Attestation{
		AggregationBits: []byte(attestation.AggregationBits),
		Data:            data,
		Signature:       attestation.Signature,
	}, nil
}

// MapAttestationData maps the eth2.AttestationData proto to the Web3Signer spec.
func MapAttestationData(data *ethpb.AttestationData) (*AttestationData, error) {
	if data == nil {
		return nil, fmt.Errorf("attestation data is nil")
	}
	source, err := MapCheckPoint(data.Source)
	if err != nil {
		return nil, errors.Wrap(err, "could not map source for attestation data")
	}
	target, err := MapCheckPoint(data.Target)
	if err != nil {
		return nil, errors.Wrap(err, "could not map target for attestation data")
	}
	return &AttestationData{
		Slot:            fmt.Sprint(data.Slot),
		Index:           fmt.Sprint(data.CommitteeIndex),
		BeaconBlockRoot: data.BeaconBlockRoot,
		Source:          source,
		Target:          target,
	}, nil
}

// MapCheckPoint maps the eth2.Checkpoint proto to the Web3Signer spec.
func MapCheckPoint(checkpoint *ethpb.Checkpoint) (*Checkpoint, error) {
	if checkpoint == nil {
		return nil, fmt.Errorf("checkpoint is nil")
	}
	return &Checkpoint{
		Epoch: fmt.Sprint(checkpoint.Epoch),
		Root:  hexutil.Encode(checkpoint.Root),
	}, nil
}

// MapBeaconBlockBody maps the eth2.BeaconBlockBody proto to the Web3Signer spec.
func MapBeaconBlockBody(body *ethpb.BeaconBlockBody) (*BeaconBlockBody, error) {
	if body == nil {
		return nil, fmt.Errorf("beacon block body is nil")
	}
	if body.Eth1Data == nil {
		return nil, fmt.Errorf("eth1 data in Beacon Block Body is nil")
	}
	block := &BeaconBlockBody{
		RandaoReveal: body.RandaoReveal,
		Eth1Data: &Eth1Data{
			DepositRoot:  body.Eth1Data.DepositRoot,
			DepositCount: fmt.Sprint(body.Eth1Data.DepositCount),
			BlockHash:    body.Eth1Data.BlockHash,
		},
		Graffiti:          body.Graffiti,
		ProposerSlashings: make([]*ProposerSlashing, len(body.ProposerSlashings)),
		AttesterSlashings: make([]*AttesterSlashing, len(body.AttesterSlashings)),
		Attestations:      make([]*Attestation, len(body.Attestations)),
		Deposits:          make([]*Deposit, len(body.Deposits)),
		VoluntaryExits:    make([]*SignedVoluntaryExit, len(body.VoluntaryExits)),
	}
	for i, slashing := range body.ProposerSlashings {
		slashing, err := MapProposerSlashing(slashing)
		if err != nil {
			return nil, fmt.Errorf("could not map proposer slashing at index %v: %v", i, err)
		}
		block.ProposerSlashings[i] = slashing
	}
	for i, slashing := range body.AttesterSlashings {
		slashing, err := MapAttesterSlashing(slashing)
		if err != nil {
			return nil, fmt.Errorf("could not map attester slashing at index %v: %v", i, err)
		}
		block.AttesterSlashings[i] = slashing
	}
	for i, attestation := range body.Attestations {
		attestation, err := MapAttestation(attestation)
		if err != nil {
			return nil, fmt.Errorf("could not map attestation at index %v: %v", i, err)
		}
		block.Attestations[i] = attestation
	}
	for i, Deposit := range body.Deposits {
		deposit, err := MapDeposit(Deposit)
		if err != nil {
			return nil, fmt.Errorf("could not map deposit at index %v: %v", i, err)
		}
		block.Deposits[i] = deposit
	}
	for i, signedVoluntaryExit := range body.VoluntaryExits {
		signedVoluntaryExit, err := MapSignedVoluntaryExit(signedVoluntaryExit)
		if err != nil {
			return nil, fmt.Errorf("could not map signed voluntary exit at index %v: %v", i, err)
		}
		block.VoluntaryExits[i] = signedVoluntaryExit
	}
	return block, nil
}

// MapProposerSlashing maps the eth2.ProposerSlashing proto to the Web3Signer spec.
func MapProposerSlashing(slashing *ethpb.ProposerSlashing) (*ProposerSlashing, error) {
	if slashing == nil {
		return nil, fmt.Errorf("proposer slashing is nil")
	}
	signedHeader1, err := MapSignedBeaconBlockHeader(slashing.Header_1)
	if err != nil {
		return nil, errors.Wrap(err, "could not map signed header 1")
	}
	signedHeader2, err := MapSignedBeaconBlockHeader(slashing.Header_2)
	if err != nil {
		return nil, errors.Wrap(err, "could not map signed header 2")
	}
	return &ProposerSlashing{
		Signedheader1: signedHeader1,
		Signedheader2: signedHeader2,
	}, nil
}

// MapSignedBeaconBlockHeader maps the eth2.AttesterSlashing proto to the Web3Signer spec.
func MapSignedBeaconBlockHeader(signedHeader *ethpb.SignedBeaconBlockHeader) (*SignedBeaconBlockHeader, error) {
	if signedHeader == nil {
		return nil, fmt.Errorf("signed beacon block header is nil")
	}
	if signedHeader.Header == nil {
		return nil, fmt.Errorf("signed beacon block header message is nil")
	}
	return &SignedBeaconBlockHeader{
		Message: &BeaconBlockHeader{
			Slot:          fmt.Sprint(signedHeader.Header.Slot),
			ProposerIndex: fmt.Sprint(signedHeader.Header.ProposerIndex),
			ParentRoot:    signedHeader.Header.ParentRoot,
			StateRoot:     signedHeader.Header.StateRoot,
			BodyRoot:      signedHeader.Header.BodyRoot,
		},
		Signature: signedHeader.Signature,
	}, nil
}

// MapAttesterSlashing maps the eth2.AttesterSlashing proto to the Web3Signer spec.
func MapAttesterSlashing(slashing *ethpb.AttesterSlashing) (*AttesterSlashing, error) {
	if slashing == nil {
		return nil, fmt.Errorf("attester slashing is nil")
	}
	attestation1, err := MapIndexedAttestation(slashing.Attestation_1)
	if err != nil {
		return nil, errors.Wrap(err, "could not map attestation 1")
	}
	attestation2, err := MapIndexedAttestation(slashing.Attestation_2)
	if err != nil {
		return nil, errors.Wrap(err, "could not map attestation 2")
	}
	return &AttesterSlashing{
		Attestation1: attestation1,
		Attestation2: attestation2,
	}, nil
}

// MapIndexedAttestation maps the eth2.IndexedAttestation proto to the Web3Signer spec.
func MapIndexedAttestation(attestation *ethpb.IndexedAttestation) (*IndexedAttestation, error) {
	if attestation == nil {
		return nil, fmt.Errorf("indexed attestation is nil")
	}
	attestingIndices := make([]string, len(attestation.AttestingIndices))
	for i, indices := range attestation.AttestingIndices {
		attestingIndices[i] = fmt.Sprint(indices)
	}
	attestationData, err := MapAttestationData(attestation.Data)
	if err != nil {
		return nil, errors.Wrap(err, "could not map attestation data to IndexedAttestation")
	}
	return &IndexedAttestation{
		AttestingIndices: attestingIndices,
		Data:             attestationData,
		Signature:        attestation.Signature,
	}, nil
}

// MapDeposit maps the eth2.Deposit proto to the Web3Signer spec.
func MapDeposit(deposit *ethpb.Deposit) (*Deposit, error) {
	if deposit == nil {
		return nil, fmt.Errorf("deposit is nil")
	}
	proof := make([]string, len(deposit.Proof))
	for i, p := range deposit.Proof {
		proof[i] = hexutil.Encode(p)
	}
	return &Deposit{
		Proof: proof,
		Data: &DepositData{
			PublicKey:             deposit.Data.PublicKey,
			WithdrawalCredentials: deposit.Data.WithdrawalCredentials,
			Amount:                fmt.Sprint(deposit.Data.Amount),
			Signature:             deposit.Data.Signature,
		},
	}, nil
}

// MapSignedVoluntaryExit maps the eth2.SignedVoluntaryExit proto to the Web3Signer spec.
func MapSignedVoluntaryExit(signedVoluntaryExit *ethpb.SignedVoluntaryExit) (*SignedVoluntaryExit, error) {
	if signedVoluntaryExit == nil {
		return nil, fmt.Errorf("signed voluntary exit is nil")
	}
	if signedVoluntaryExit.Exit == nil {
		return nil, fmt.Errorf("exit in signed voluntary exit is nil")
	}
	return &SignedVoluntaryExit{
		Message: &VoluntaryExit{
			Epoch:          fmt.Sprint(signedVoluntaryExit.Exit.Epoch),
			ValidatorIndex: fmt.Sprint(signedVoluntaryExit.Exit.ValidatorIndex),
		},
		Signature: signedVoluntaryExit.Signature,
	}, nil
}

// MapBeaconBlockAltair maps the eth2.BeaconBlockAltair proto to the Web3Signer spec.
func MapBeaconBlockAltair(block *ethpb.BeaconBlockAltair) (*BeaconBlockAltair, error) {
	if block == nil {
		return nil, fmt.Errorf("beacon block altair is nil")
	}
	body, err := MapBeaconBlockBodyAltair(block.Body)
	if err != nil {
		return nil, errors.Wrap(err, "could not map beacon block body for altair")
	}
	return &BeaconBlockAltair{
		Slot:          fmt.Sprint(block.Slot),
		ProposerIndex: fmt.Sprint(block.ProposerIndex),
		ParentRoot:    block.ParentRoot,
		StateRoot:     block.StateRoot,
		Body:          body,
	}, nil
}

// MapBeaconBlockBodyAltair maps the eth2.BeaconBlockBodyAltair proto to the Web3Signer spec.
func MapBeaconBlockBodyAltair(body *ethpb.BeaconBlockBodyAltair) (*BeaconBlockBodyAltair, error) {
	if body == nil {
		return nil, fmt.Errorf("beacon block body altair is nil")
	}
	if body.SyncAggregate == nil {
		return nil, fmt.Errorf("sync aggregate in beacon block body altair is nil")
	}
	if body.SyncAggregate.SyncCommitteeBits == nil {
		return nil, fmt.Errorf("sync committee bits in sync aggregate in beacon block body altair is nil")
	}

	block := &BeaconBlockBodyAltair{
		RandaoReveal: body.RandaoReveal,
		Eth1Data: &Eth1Data{
			DepositRoot:  body.Eth1Data.DepositRoot,
			DepositCount: fmt.Sprint(body.Eth1Data.DepositCount),
			BlockHash:    body.Eth1Data.BlockHash,
		},
		Graffiti:          body.Graffiti,
		ProposerSlashings: make([]*ProposerSlashing, len(body.ProposerSlashings)),
		AttesterSlashings: make([]*AttesterSlashing, len(body.AttesterSlashings)),
		Attestations:      make([]*Attestation, len(body.Attestations)),
		Deposits:          make([]*Deposit, len(body.Deposits)),
		VoluntaryExits:    make([]*SignedVoluntaryExit, len(body.VoluntaryExits)),
		SyncAggregate: &SyncAggregate{
			SyncCommitteeBits:      []byte(body.SyncAggregate.SyncCommitteeBits),
			SyncCommitteeSignature: body.SyncAggregate.SyncCommitteeSignature,
		},
	}
	for i, slashing := range body.ProposerSlashings {
		proposer, err := MapProposerSlashing(slashing)
		if err != nil {
			return nil, fmt.Errorf("could not map proposer slashing at index %v: %v", i, err)
		}
		block.ProposerSlashings[i] = proposer
	}
	for i, slashing := range body.AttesterSlashings {
		attesterSlashing, err := MapAttesterSlashing(slashing)
		if err != nil {
			return nil, fmt.Errorf("could not map attester slashing at index %v: %v", i, err)
		}
		block.AttesterSlashings[i] = attesterSlashing
	}
	for i, attestation := range body.Attestations {
		attestation, err := MapAttestation(attestation)
		if err != nil {
			return nil, fmt.Errorf("could not map attestation at index %v: %v", i, err)
		}
		block.Attestations[i] = attestation
	}
	for i, deposit := range body.Deposits {
		deposit, err := MapDeposit(deposit)
		if err != nil {
			return nil, fmt.Errorf("could not map deposit at index %v: %v", i, err)
		}
		block.Deposits[i] = deposit
	}
	for i, exit := range body.VoluntaryExits {
		exit, err := MapSignedVoluntaryExit(exit)
		if err != nil {
			return nil, fmt.Errorf("could not map signed voluntary exit at index %v: %v", i, err)
		}
		block.VoluntaryExits[i] = exit
	}
	return block, nil
}

// MapSyncAggregatorSelectionData maps the eth2.SyncAggregatorSelectionData proto to the Web3Signer spec.
func MapSyncAggregatorSelectionData(data *ethpb.SyncAggregatorSelectionData) (*SyncAggregatorSelectionData, error) {
	if data == nil {
		return nil, fmt.Errorf("sync aggregator selection data is nil")
	}
	return &SyncAggregatorSelectionData{
		Slot:              fmt.Sprint(data.Slot),
		SubcommitteeIndex: fmt.Sprint(data.SubcommitteeIndex),
	}, nil
}

// MapContributionAndProof maps the eth2.ContributionAndProof proto to the Web3Signer spec.
func MapContributionAndProof(contribution *ethpb.ContributionAndProof) (*ContributionAndProof, error) {
	if contribution == nil {
		return nil, fmt.Errorf("contribution and proof is nil")
	}
	if contribution.Contribution == nil {
		return nil, fmt.Errorf("contribution in ContributionAndProof is nil")
	}
	if contribution.Contribution.AggregationBits == nil {
		return nil, fmt.Errorf("aggregation bits in ContributionAndProof is nil")
	}
	return &ContributionAndProof{
		AggregatorIndex: fmt.Sprint(contribution.AggregatorIndex),
		SelectionProof:  contribution.SelectionProof,
		Contribution: &SyncCommitteeContribution{
			Slot:              fmt.Sprint(contribution.Contribution.Slot),
			BeaconBlockRoot:   contribution.Contribution.BlockRoot,
			SubcommitteeIndex: fmt.Sprint(contribution.Contribution.SubcommitteeIndex),
			AggregationBits:   []byte(contribution.Contribution.AggregationBits),
			Signature:         contribution.Contribution.Signature,
		},
	}, nil
}
