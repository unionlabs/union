package beacon_api

import (
	"strconv"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func jsonifyTransactions(transactions [][]byte) []string {
	jsonTransactions := make([]string, len(transactions))
	for index, transaction := range transactions {
		jsonTransaction := hexutil.Encode(transaction)
		jsonTransactions[index] = jsonTransaction
	}
	return jsonTransactions
}

func jsonifyBlsToExecutionChanges(blsToExecutionChanges []*ethpb.SignedBLSToExecutionChange) []*shared.SignedBLSToExecutionChange {
	jsonBlsToExecutionChanges := make([]*shared.SignedBLSToExecutionChange, len(blsToExecutionChanges))
	for index, signedBlsToExecutionChange := range blsToExecutionChanges {
		blsToExecutionChangeJson := &shared.BLSToExecutionChange{
			ValidatorIndex:     uint64ToString(signedBlsToExecutionChange.Message.ValidatorIndex),
			FromBLSPubkey:      hexutil.Encode(signedBlsToExecutionChange.Message.FromBlsPubkey),
			ToExecutionAddress: hexutil.Encode(signedBlsToExecutionChange.Message.ToExecutionAddress),
		}
		signedJson := &shared.SignedBLSToExecutionChange{
			Message:   blsToExecutionChangeJson,
			Signature: hexutil.Encode(signedBlsToExecutionChange.Signature),
		}
		jsonBlsToExecutionChanges[index] = signedJson
	}
	return jsonBlsToExecutionChanges
}

func jsonifyEth1Data(eth1Data *ethpb.Eth1Data) *shared.Eth1Data {
	return &shared.Eth1Data{
		BlockHash:    hexutil.Encode(eth1Data.BlockHash),
		DepositCount: uint64ToString(eth1Data.DepositCount),
		DepositRoot:  hexutil.Encode(eth1Data.DepositRoot),
	}
}

func jsonifyAttestations(attestations []*ethpb.Attestation) []*shared.Attestation {
	jsonAttestations := make([]*shared.Attestation, len(attestations))
	for index, attestation := range attestations {
		jsonAttestations[index] = jsonifyAttestation(attestation)
	}
	return jsonAttestations
}

func jsonifyAttesterSlashings(attesterSlashings []*ethpb.AttesterSlashing) []*shared.AttesterSlashing {
	jsonAttesterSlashings := make([]*shared.AttesterSlashing, len(attesterSlashings))
	for index, attesterSlashing := range attesterSlashings {
		jsonAttesterSlashing := &shared.AttesterSlashing{
			Attestation1: jsonifyIndexedAttestation(attesterSlashing.Attestation_1),
			Attestation2: jsonifyIndexedAttestation(attesterSlashing.Attestation_2),
		}
		jsonAttesterSlashings[index] = jsonAttesterSlashing
	}
	return jsonAttesterSlashings
}

func jsonifyDeposits(deposits []*ethpb.Deposit) []*shared.Deposit {
	jsonDeposits := make([]*shared.Deposit, len(deposits))
	for depositIndex, deposit := range deposits {
		proofs := make([]string, len(deposit.Proof))
		for proofIndex, proof := range deposit.Proof {
			proofs[proofIndex] = hexutil.Encode(proof)
		}

		jsonDeposit := &shared.Deposit{
			Data: &shared.DepositData{
				Amount:                uint64ToString(deposit.Data.Amount),
				Pubkey:                hexutil.Encode(deposit.Data.PublicKey),
				Signature:             hexutil.Encode(deposit.Data.Signature),
				WithdrawalCredentials: hexutil.Encode(deposit.Data.WithdrawalCredentials),
			},
			Proof: proofs,
		}
		jsonDeposits[depositIndex] = jsonDeposit
	}
	return jsonDeposits
}

func jsonifyProposerSlashings(proposerSlashings []*ethpb.ProposerSlashing) []*shared.ProposerSlashing {
	jsonProposerSlashings := make([]*shared.ProposerSlashing, len(proposerSlashings))
	for index, proposerSlashing := range proposerSlashings {
		jsonProposerSlashing := &shared.ProposerSlashing{
			SignedHeader1: jsonifySignedBeaconBlockHeader(proposerSlashing.Header_1),
			SignedHeader2: jsonifySignedBeaconBlockHeader(proposerSlashing.Header_2),
		}
		jsonProposerSlashings[index] = jsonProposerSlashing
	}
	return jsonProposerSlashings
}

// JsonifySignedVoluntaryExits converts an array of voluntary exit structs to a JSON hex string compatible format.
func JsonifySignedVoluntaryExits(voluntaryExits []*ethpb.SignedVoluntaryExit) []*shared.SignedVoluntaryExit {
	jsonSignedVoluntaryExits := make([]*shared.SignedVoluntaryExit, len(voluntaryExits))
	for index, signedVoluntaryExit := range voluntaryExits {
		jsonSignedVoluntaryExit := &shared.SignedVoluntaryExit{
			Message: &shared.VoluntaryExit{
				Epoch:          uint64ToString(signedVoluntaryExit.Exit.Epoch),
				ValidatorIndex: uint64ToString(signedVoluntaryExit.Exit.ValidatorIndex),
			},
			Signature: hexutil.Encode(signedVoluntaryExit.Signature),
		}
		jsonSignedVoluntaryExits[index] = jsonSignedVoluntaryExit
	}
	return jsonSignedVoluntaryExits
}

func jsonifySignedBeaconBlockHeader(signedBeaconBlockHeader *ethpb.SignedBeaconBlockHeader) *shared.SignedBeaconBlockHeader {
	return &shared.SignedBeaconBlockHeader{
		Message: &shared.BeaconBlockHeader{
			BodyRoot:      hexutil.Encode(signedBeaconBlockHeader.Header.BodyRoot),
			ParentRoot:    hexutil.Encode(signedBeaconBlockHeader.Header.ParentRoot),
			ProposerIndex: uint64ToString(signedBeaconBlockHeader.Header.ProposerIndex),
			Slot:          uint64ToString(signedBeaconBlockHeader.Header.Slot),
			StateRoot:     hexutil.Encode(signedBeaconBlockHeader.Header.StateRoot),
		},
		Signature: hexutil.Encode(signedBeaconBlockHeader.Signature),
	}
}

func jsonifyIndexedAttestation(indexedAttestation *ethpb.IndexedAttestation) *shared.IndexedAttestation {
	attestingIndices := make([]string, len(indexedAttestation.AttestingIndices))
	for index, attestingIndex := range indexedAttestation.AttestingIndices {
		attestingIndex := uint64ToString(attestingIndex)
		attestingIndices[index] = attestingIndex
	}

	return &shared.IndexedAttestation{
		AttestingIndices: attestingIndices,
		Data:             jsonifyAttestationData(indexedAttestation.Data),
		Signature:        hexutil.Encode(indexedAttestation.Signature),
	}
}

func jsonifyAttestationData(attestationData *ethpb.AttestationData) *shared.AttestationData {
	return &shared.AttestationData{
		BeaconBlockRoot: hexutil.Encode(attestationData.BeaconBlockRoot),
		CommitteeIndex:  uint64ToString(attestationData.CommitteeIndex),
		Slot:            uint64ToString(attestationData.Slot),
		Source: &shared.Checkpoint{
			Epoch: uint64ToString(attestationData.Source.Epoch),
			Root:  hexutil.Encode(attestationData.Source.Root),
		},
		Target: &shared.Checkpoint{
			Epoch: uint64ToString(attestationData.Target.Epoch),
			Root:  hexutil.Encode(attestationData.Target.Root),
		},
	}
}

func jsonifyAttestation(attestation *ethpb.Attestation) *shared.Attestation {
	return &shared.Attestation{
		AggregationBits: hexutil.Encode(attestation.AggregationBits),
		Data:            jsonifyAttestationData(attestation.Data),
		Signature:       hexutil.Encode(attestation.Signature),
	}
}

func jsonifySignedAggregateAndProof(signedAggregateAndProof *ethpb.SignedAggregateAttestationAndProof) *shared.SignedAggregateAttestationAndProof {
	return &shared.SignedAggregateAttestationAndProof{
		Message: &shared.AggregateAttestationAndProof{
			AggregatorIndex: uint64ToString(signedAggregateAndProof.Message.AggregatorIndex),
			Aggregate:       jsonifyAttestation(signedAggregateAndProof.Message.Aggregate),
			SelectionProof:  hexutil.Encode(signedAggregateAndProof.Message.SelectionProof),
		},
		Signature: hexutil.Encode(signedAggregateAndProof.Signature),
	}
}

func jsonifyWithdrawals(withdrawals []*enginev1.Withdrawal) []*shared.Withdrawal {
	jsonWithdrawals := make([]*shared.Withdrawal, len(withdrawals))
	for index, withdrawal := range withdrawals {
		jsonWithdrawals[index] = &shared.Withdrawal{
			WithdrawalIndex:  strconv.FormatUint(withdrawal.Index, 10),
			ValidatorIndex:   strconv.FormatUint(uint64(withdrawal.ValidatorIndex), 10),
			ExecutionAddress: hexutil.Encode(withdrawal.Address),
			Amount:           strconv.FormatUint(withdrawal.Amount, 10),
		}
	}
	return jsonWithdrawals
}
