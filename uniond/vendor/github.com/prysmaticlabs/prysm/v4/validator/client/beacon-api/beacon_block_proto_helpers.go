package beacon_api

import (
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func convertProposerSlashingsToProto(jsonProposerSlashings []*shared.ProposerSlashing) ([]*ethpb.ProposerSlashing, error) {
	proposerSlashings := make([]*ethpb.ProposerSlashing, len(jsonProposerSlashings))

	for index, jsonProposerSlashing := range jsonProposerSlashings {
		if jsonProposerSlashing == nil {
			return nil, errors.Errorf("proposer slashing at index `%d` is nil", index)
		}

		header1, err := convertProposerSlashingSignedHeaderToProto(jsonProposerSlashing.SignedHeader1)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get proposer header 1")
		}

		header2, err := convertProposerSlashingSignedHeaderToProto(jsonProposerSlashing.SignedHeader2)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get proposer header 2")
		}

		proposerSlashings[index] = &ethpb.ProposerSlashing{
			Header_1: header1,
			Header_2: header2,
		}
	}

	return proposerSlashings, nil
}

func convertProposerSlashingSignedHeaderToProto(signedHeader *shared.SignedBeaconBlockHeader) (*ethpb.SignedBeaconBlockHeader, error) {
	if signedHeader == nil {
		return nil, errors.New("signed header is nil")
	}

	if signedHeader.Message == nil {
		return nil, errors.New("header is nil")
	}

	slot, err := strconv.ParseUint(signedHeader.Message.Slot, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse header slot `%s`", signedHeader.Message.Slot)
	}

	proposerIndex, err := strconv.ParseUint(signedHeader.Message.ProposerIndex, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse header proposer index `%s`", signedHeader.Message.ProposerIndex)
	}

	parentRoot, err := hexutil.Decode(signedHeader.Message.ParentRoot)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode header parent root `%s`", signedHeader.Message.ParentRoot)
	}

	stateRoot, err := hexutil.Decode(signedHeader.Message.StateRoot)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode header state root `%s`", signedHeader.Message.StateRoot)
	}

	bodyRoot, err := hexutil.Decode(signedHeader.Message.BodyRoot)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode header body root `%s`", signedHeader.Message.BodyRoot)
	}

	signature, err := hexutil.Decode(signedHeader.Signature)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode signature `%s`", signedHeader.Signature)
	}

	return &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          primitives.Slot(slot),
			ProposerIndex: primitives.ValidatorIndex(proposerIndex),
			ParentRoot:    parentRoot,
			StateRoot:     stateRoot,
			BodyRoot:      bodyRoot,
		},
		Signature: signature,
	}, nil
}

func convertAttesterSlashingsToProto(jsonAttesterSlashings []*shared.AttesterSlashing) ([]*ethpb.AttesterSlashing, error) {
	attesterSlashings := make([]*ethpb.AttesterSlashing, len(jsonAttesterSlashings))

	for index, jsonAttesterSlashing := range jsonAttesterSlashings {
		if jsonAttesterSlashing == nil {
			return nil, errors.Errorf("attester slashing at index `%d` is nil", index)
		}

		attestation1, err := convertIndexedAttestationToProto(jsonAttesterSlashing.Attestation1)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get attestation 1")
		}

		attestation2, err := convertIndexedAttestationToProto(jsonAttesterSlashing.Attestation2)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get attestation 2")
		}

		attesterSlashings[index] = &ethpb.AttesterSlashing{
			Attestation_1: attestation1,
			Attestation_2: attestation2,
		}
	}

	return attesterSlashings, nil
}

func convertIndexedAttestationToProto(jsonAttestation *shared.IndexedAttestation) (*ethpb.IndexedAttestation, error) {
	if jsonAttestation == nil {
		return nil, errors.New("indexed attestation is nil")
	}

	attestingIndices := make([]uint64, len(jsonAttestation.AttestingIndices))

	for index, jsonAttestingIndex := range jsonAttestation.AttestingIndices {
		attestingIndex, err := strconv.ParseUint(jsonAttestingIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse attesting index `%s`", jsonAttestingIndex)
		}

		attestingIndices[index] = attestingIndex
	}

	signature, err := hexutil.Decode(jsonAttestation.Signature)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode attestation signature `%s`", jsonAttestation.Signature)
	}

	attestationData, err := convertAttestationDataToProto(jsonAttestation.Data)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attestation data")
	}

	return &ethpb.IndexedAttestation{
		AttestingIndices: attestingIndices,
		Data:             attestationData,
		Signature:        signature,
	}, nil
}

func convertCheckpointToProto(jsonCheckpoint *shared.Checkpoint) (*ethpb.Checkpoint, error) {
	if jsonCheckpoint == nil {
		return nil, errors.New("checkpoint is nil")
	}

	epoch, err := strconv.ParseUint(jsonCheckpoint.Epoch, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse checkpoint epoch `%s`", jsonCheckpoint.Epoch)
	}

	root, err := hexutil.Decode(jsonCheckpoint.Root)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode checkpoint root `%s`", jsonCheckpoint.Root)
	}

	return &ethpb.Checkpoint{
		Epoch: primitives.Epoch(epoch),
		Root:  root,
	}, nil
}

func convertAttestationToProto(jsonAttestation *shared.Attestation) (*ethpb.Attestation, error) {
	if jsonAttestation == nil {
		return nil, errors.New("json attestation is nil")
	}

	aggregationBits, err := hexutil.Decode(jsonAttestation.AggregationBits)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode aggregation bits `%s`", jsonAttestation.AggregationBits)
	}

	attestationData, err := convertAttestationDataToProto(jsonAttestation.Data)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attestation data")
	}

	signature, err := hexutil.Decode(jsonAttestation.Signature)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode attestation signature `%s`", jsonAttestation.Signature)
	}

	return &ethpb.Attestation{
		AggregationBits: aggregationBits,
		Data:            attestationData,
		Signature:       signature,
	}, nil
}

func convertAttestationsToProto(jsonAttestations []*shared.Attestation) ([]*ethpb.Attestation, error) {
	var attestations []*ethpb.Attestation
	for index, jsonAttestation := range jsonAttestations {
		if jsonAttestation == nil {
			return nil, errors.Errorf("attestation at index `%d` is nil", index)
		}

		attestation, err := convertAttestationToProto(jsonAttestation)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to convert json attestation to proto at index %d", index)
		}

		attestations = append(attestations, attestation)
	}

	return attestations, nil
}

func convertAttestationDataToProto(jsonAttestationData *shared.AttestationData) (*ethpb.AttestationData, error) {
	if jsonAttestationData == nil {
		return nil, errors.New("attestation data is nil")
	}

	slot, err := strconv.ParseUint(jsonAttestationData.Slot, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse attestation slot `%s`", jsonAttestationData.Slot)
	}

	committeeIndex, err := strconv.ParseUint(jsonAttestationData.CommitteeIndex, 10, 64)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to parse attestation committee index `%s`", jsonAttestationData.CommitteeIndex)
	}

	beaconBlockRoot, err := hexutil.Decode(jsonAttestationData.BeaconBlockRoot)
	if err != nil {
		return nil, errors.Wrapf(err, "failed to decode attestation beacon block root `%s`", jsonAttestationData.BeaconBlockRoot)
	}

	sourceCheckpoint, err := convertCheckpointToProto(jsonAttestationData.Source)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attestation source checkpoint")
	}

	targetCheckpoint, err := convertCheckpointToProto(jsonAttestationData.Target)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get attestation target checkpoint")
	}

	return &ethpb.AttestationData{
		Slot:            primitives.Slot(slot),
		CommitteeIndex:  primitives.CommitteeIndex(committeeIndex),
		BeaconBlockRoot: beaconBlockRoot,
		Source:          sourceCheckpoint,
		Target:          targetCheckpoint,
	}, nil
}

func convertDepositsToProto(jsonDeposits []*shared.Deposit) ([]*ethpb.Deposit, error) {
	deposits := make([]*ethpb.Deposit, len(jsonDeposits))

	for depositIndex, jsonDeposit := range jsonDeposits {
		if jsonDeposit == nil {
			return nil, errors.Errorf("deposit at index `%d` is nil", depositIndex)
		}

		proofs := make([][]byte, len(jsonDeposit.Proof))
		for proofIndex, jsonProof := range jsonDeposit.Proof {
			proof, err := hexutil.Decode(jsonProof)
			if err != nil {
				return nil, errors.Wrapf(err, "failed to decode deposit proof `%s`", jsonProof)
			}

			proofs[proofIndex] = proof
		}

		if jsonDeposit.Data == nil {
			return nil, errors.Errorf("deposit data at index `%d` is nil", depositIndex)
		}

		pubkey, err := hexutil.Decode(jsonDeposit.Data.Pubkey)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode deposit public key `%s`", jsonDeposit.Data.Pubkey)
		}

		withdrawalCredentials, err := hexutil.Decode(jsonDeposit.Data.WithdrawalCredentials)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode deposit withdrawal credentials `%s`", jsonDeposit.Data.WithdrawalCredentials)
		}

		amount, err := strconv.ParseUint(jsonDeposit.Data.Amount, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse deposit amount `%s`", jsonDeposit.Data.Amount)
		}

		signature, err := hexutil.Decode(jsonDeposit.Data.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode signature `%s`", jsonDeposit.Data.Signature)
		}

		deposits[depositIndex] = &ethpb.Deposit{
			Proof: proofs,
			Data: &ethpb.Deposit_Data{
				PublicKey:             pubkey,
				WithdrawalCredentials: withdrawalCredentials,
				Amount:                amount,
				Signature:             signature,
			},
		}
	}

	return deposits, nil
}

func convertVoluntaryExitsToProto(jsonVoluntaryExits []*shared.SignedVoluntaryExit) ([]*ethpb.SignedVoluntaryExit, error) {
	attestingIndices := make([]*ethpb.SignedVoluntaryExit, len(jsonVoluntaryExits))

	for index, jsonVoluntaryExit := range jsonVoluntaryExits {
		if jsonVoluntaryExit == nil {
			return nil, errors.Errorf("signed voluntary exit at index `%d` is nil", index)
		}

		if jsonVoluntaryExit.Message == nil {
			return nil, errors.Errorf("voluntary exit at index `%d` is nil", index)
		}

		epoch, err := strconv.ParseUint(jsonVoluntaryExit.Message.Epoch, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse voluntary exit epoch `%s`", jsonVoluntaryExit.Message.Epoch)
		}

		validatorIndex, err := strconv.ParseUint(jsonVoluntaryExit.Message.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse voluntary exit validator index `%s`", jsonVoluntaryExit.Message.ValidatorIndex)
		}

		signature, err := hexutil.Decode(jsonVoluntaryExit.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode signature `%s`", jsonVoluntaryExit.Signature)
		}

		attestingIndices[index] = &ethpb.SignedVoluntaryExit{
			Exit: &ethpb.VoluntaryExit{
				Epoch:          primitives.Epoch(epoch),
				ValidatorIndex: primitives.ValidatorIndex(validatorIndex),
			},
			Signature: signature,
		}
	}

	return attestingIndices, nil
}

func convertTransactionsToProto(jsonTransactions []string) ([][]byte, error) {
	transactions := make([][]byte, len(jsonTransactions))

	for index, jsonTransaction := range jsonTransactions {
		transaction, err := hexutil.Decode(jsonTransaction)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode transaction `%s`", jsonTransaction)
		}

		transactions[index] = transaction
	}

	return transactions, nil
}

func convertWithdrawalsToProto(jsonWithdrawals []*shared.Withdrawal) ([]*enginev1.Withdrawal, error) {
	withdrawals := make([]*enginev1.Withdrawal, len(jsonWithdrawals))

	for index, jsonWithdrawal := range jsonWithdrawals {
		if jsonWithdrawal == nil {
			return nil, errors.Errorf("withdrawal at index `%d` is nil", index)
		}

		withdrawalIndex, err := strconv.ParseUint(jsonWithdrawal.WithdrawalIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse withdrawal index `%s`", jsonWithdrawal.WithdrawalIndex)
		}

		validatorIndex, err := strconv.ParseUint(jsonWithdrawal.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse validator index `%s`", jsonWithdrawal.ValidatorIndex)
		}

		executionAddress, err := hexutil.Decode(jsonWithdrawal.ExecutionAddress)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode execution address `%s`", jsonWithdrawal.ExecutionAddress)
		}

		amount, err := strconv.ParseUint(jsonWithdrawal.Amount, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to parse withdrawal amount `%s`", jsonWithdrawal.Amount)
		}

		withdrawals[index] = &enginev1.Withdrawal{
			Index:          withdrawalIndex,
			ValidatorIndex: primitives.ValidatorIndex(validatorIndex),
			Address:        executionAddress,
			Amount:         amount,
		}
	}

	return withdrawals, nil
}

func convertBlsToExecutionChangesToProto(jsonSignedBlsToExecutionChanges []*shared.SignedBLSToExecutionChange) ([]*ethpb.SignedBLSToExecutionChange, error) {
	signedBlsToExecutionChanges := make([]*ethpb.SignedBLSToExecutionChange, len(jsonSignedBlsToExecutionChanges))

	for index, jsonBlsToExecutionChange := range jsonSignedBlsToExecutionChanges {
		if jsonBlsToExecutionChange == nil {
			return nil, errors.Errorf("bls to execution change at index `%d` is nil", index)
		}

		if jsonBlsToExecutionChange.Message == nil {
			return nil, errors.Errorf("bls to execution change message at index `%d` is nil", index)
		}

		validatorIndex, err := strconv.ParseUint(jsonBlsToExecutionChange.Message.ValidatorIndex, 10, 64)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode validator index `%s`", jsonBlsToExecutionChange.Message.ValidatorIndex)
		}

		fromBlsPubkey, err := hexutil.Decode(jsonBlsToExecutionChange.Message.FromBLSPubkey)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode bls pubkey `%s`", jsonBlsToExecutionChange.Message.FromBLSPubkey)
		}

		toExecutionAddress, err := hexutil.Decode(jsonBlsToExecutionChange.Message.ToExecutionAddress)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode execution address `%s`", jsonBlsToExecutionChange.Message.ToExecutionAddress)
		}

		signature, err := hexutil.Decode(jsonBlsToExecutionChange.Signature)
		if err != nil {
			return nil, errors.Wrapf(err, "failed to decode signature `%s`", jsonBlsToExecutionChange.Signature)
		}

		signedBlsToExecutionChanges[index] = &ethpb.SignedBLSToExecutionChange{
			Message: &ethpb.BLSToExecutionChange{
				ValidatorIndex:     primitives.ValidatorIndex(validatorIndex),
				FromBlsPubkey:      fromBlsPubkey,
				ToExecutionAddress: toExecutionAddress,
			},
			Signature: signature,
		}
	}

	return signedBlsToExecutionChanges, nil
}
