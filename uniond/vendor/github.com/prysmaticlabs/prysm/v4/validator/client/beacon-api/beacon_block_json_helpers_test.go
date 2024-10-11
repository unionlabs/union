package beacon_api

import (
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestBeaconBlockJsonHelpers_JsonifyTransactions(t *testing.T) {
	input := [][]byte{{1}, {2}, {3}, {4}}

	expectedResult := []string{
		hexutil.Encode([]byte{1}),
		hexutil.Encode([]byte{2}),
		hexutil.Encode([]byte{3}),
		hexutil.Encode([]byte{4}),
	}

	result := jsonifyTransactions(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyBlsToExecutionChanges(t *testing.T) {
	input := []*ethpb.SignedBLSToExecutionChange{
		{
			Message: &ethpb.BLSToExecutionChange{
				ValidatorIndex:     1,
				FromBlsPubkey:      []byte{2},
				ToExecutionAddress: []byte{3},
			},
			Signature: []byte{7},
		},
		{
			Message: &ethpb.BLSToExecutionChange{
				ValidatorIndex:     4,
				FromBlsPubkey:      []byte{5},
				ToExecutionAddress: []byte{6},
			},
			Signature: []byte{8},
		},
	}

	expectedResult := []*shared.SignedBLSToExecutionChange{
		{
			Message: &shared.BLSToExecutionChange{
				ValidatorIndex:     "1",
				FromBLSPubkey:      hexutil.Encode([]byte{2}),
				ToExecutionAddress: hexutil.Encode([]byte{3}),
			},
			Signature: hexutil.Encode([]byte{7}),
		},
		{
			Message: &shared.BLSToExecutionChange{
				ValidatorIndex:     "4",
				FromBLSPubkey:      hexutil.Encode([]byte{5}),
				ToExecutionAddress: hexutil.Encode([]byte{6}),
			},
			Signature: hexutil.Encode([]byte{8}),
		},
	}

	assert.DeepEqual(t, expectedResult, shared.SignedBLSChangesFromConsensus(input))
}

func TestBeaconBlockJsonHelpers_JsonifyEth1Data(t *testing.T) {
	input := &ethpb.Eth1Data{
		DepositRoot:  []byte{1},
		DepositCount: 2,
		BlockHash:    []byte{3},
	}

	expectedResult := &shared.Eth1Data{
		DepositRoot:  hexutil.Encode([]byte{1}),
		DepositCount: "2",
		BlockHash:    hexutil.Encode([]byte{3}),
	}

	result := jsonifyEth1Data(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyAttestations(t *testing.T) {
	input := []*ethpb.Attestation{
		{
			AggregationBits: []byte{1},
			Data: &ethpb.AttestationData{
				Slot:            2,
				CommitteeIndex:  3,
				BeaconBlockRoot: []byte{4},
				Source: &ethpb.Checkpoint{
					Epoch: 5,
					Root:  []byte{6},
				},
				Target: &ethpb.Checkpoint{
					Epoch: 7,
					Root:  []byte{8},
				},
			},
			Signature: []byte{9},
		},
		{
			AggregationBits: []byte{10},
			Data: &ethpb.AttestationData{
				Slot:            11,
				CommitteeIndex:  12,
				BeaconBlockRoot: []byte{13},
				Source: &ethpb.Checkpoint{
					Epoch: 14,
					Root:  []byte{15},
				},
				Target: &ethpb.Checkpoint{
					Epoch: 16,
					Root:  []byte{17},
				},
			},
			Signature: []byte{18},
		},
	}

	expectedResult := []*shared.Attestation{
		{
			AggregationBits: hexutil.Encode([]byte{1}),
			Data: &shared.AttestationData{
				Slot:            "2",
				CommitteeIndex:  "3",
				BeaconBlockRoot: hexutil.Encode([]byte{4}),
				Source: &shared.Checkpoint{
					Epoch: "5",
					Root:  hexutil.Encode([]byte{6}),
				},
				Target: &shared.Checkpoint{
					Epoch: "7",
					Root:  hexutil.Encode([]byte{8}),
				},
			},
			Signature: hexutil.Encode([]byte{9}),
		},
		{
			AggregationBits: hexutil.Encode([]byte{10}),
			Data: &shared.AttestationData{
				Slot:            "11",
				CommitteeIndex:  "12",
				BeaconBlockRoot: hexutil.Encode([]byte{13}),
				Source: &shared.Checkpoint{
					Epoch: "14",
					Root:  hexutil.Encode([]byte{15}),
				},
				Target: &shared.Checkpoint{
					Epoch: "16",
					Root:  hexutil.Encode([]byte{17}),
				},
			},
			Signature: hexutil.Encode([]byte{18}),
		},
	}

	result := jsonifyAttestations(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyAttesterSlashings(t *testing.T) {
	input := []*ethpb.AttesterSlashing{
		{
			Attestation_1: &ethpb.IndexedAttestation{
				AttestingIndices: []uint64{1, 2},
				Data: &ethpb.AttestationData{
					Slot:            3,
					CommitteeIndex:  4,
					BeaconBlockRoot: []byte{5},
					Source: &ethpb.Checkpoint{
						Epoch: 6,
						Root:  []byte{7},
					},
					Target: &ethpb.Checkpoint{
						Epoch: 8,
						Root:  []byte{9},
					},
				},
				Signature: []byte{10},
			},
			Attestation_2: &ethpb.IndexedAttestation{
				AttestingIndices: []uint64{11, 12},
				Data: &ethpb.AttestationData{
					Slot:            13,
					CommitteeIndex:  14,
					BeaconBlockRoot: []byte{15},
					Source: &ethpb.Checkpoint{
						Epoch: 16,
						Root:  []byte{17},
					},
					Target: &ethpb.Checkpoint{
						Epoch: 18,
						Root:  []byte{19},
					},
				},
				Signature: []byte{20},
			},
		},
		{
			Attestation_1: &ethpb.IndexedAttestation{
				AttestingIndices: []uint64{21, 22},
				Data: &ethpb.AttestationData{
					Slot:            23,
					CommitteeIndex:  24,
					BeaconBlockRoot: []byte{25},
					Source: &ethpb.Checkpoint{
						Epoch: 26,
						Root:  []byte{27},
					},
					Target: &ethpb.Checkpoint{
						Epoch: 28,
						Root:  []byte{29},
					},
				},
				Signature: []byte{30},
			},
			Attestation_2: &ethpb.IndexedAttestation{
				AttestingIndices: []uint64{31, 32},
				Data: &ethpb.AttestationData{
					Slot:            33,
					CommitteeIndex:  34,
					BeaconBlockRoot: []byte{35},
					Source: &ethpb.Checkpoint{
						Epoch: 36,
						Root:  []byte{37},
					},
					Target: &ethpb.Checkpoint{
						Epoch: 38,
						Root:  []byte{39},
					},
				},
				Signature: []byte{40},
			},
		},
	}

	expectedResult := []*shared.AttesterSlashing{
		{
			Attestation1: &shared.IndexedAttestation{
				AttestingIndices: []string{"1", "2"},
				Data: &shared.AttestationData{
					Slot:            "3",
					CommitteeIndex:  "4",
					BeaconBlockRoot: hexutil.Encode([]byte{5}),
					Source: &shared.Checkpoint{
						Epoch: "6",
						Root:  hexutil.Encode([]byte{7}),
					},
					Target: &shared.Checkpoint{
						Epoch: "8",
						Root:  hexutil.Encode([]byte{9}),
					},
				},
				Signature: hexutil.Encode([]byte{10}),
			},
			Attestation2: &shared.IndexedAttestation{
				AttestingIndices: []string{"11", "12"},
				Data: &shared.AttestationData{
					Slot:            "13",
					CommitteeIndex:  "14",
					BeaconBlockRoot: hexutil.Encode([]byte{15}),
					Source: &shared.Checkpoint{
						Epoch: "16",
						Root:  hexutil.Encode([]byte{17}),
					},
					Target: &shared.Checkpoint{
						Epoch: "18",
						Root:  hexutil.Encode([]byte{19}),
					},
				},
				Signature: hexutil.Encode([]byte{20}),
			},
		},
		{
			Attestation1: &shared.IndexedAttestation{
				AttestingIndices: []string{"21", "22"},
				Data: &shared.AttestationData{
					Slot:            "23",
					CommitteeIndex:  "24",
					BeaconBlockRoot: hexutil.Encode([]byte{25}),
					Source: &shared.Checkpoint{
						Epoch: "26",
						Root:  hexutil.Encode([]byte{27}),
					},
					Target: &shared.Checkpoint{
						Epoch: "28",
						Root:  hexutil.Encode([]byte{29}),
					},
				},
				Signature: hexutil.Encode([]byte{30}),
			},
			Attestation2: &shared.IndexedAttestation{
				AttestingIndices: []string{"31", "32"},
				Data: &shared.AttestationData{
					Slot:            "33",
					CommitteeIndex:  "34",
					BeaconBlockRoot: hexutil.Encode([]byte{35}),
					Source: &shared.Checkpoint{
						Epoch: "36",
						Root:  hexutil.Encode([]byte{37}),
					},
					Target: &shared.Checkpoint{
						Epoch: "38",
						Root:  hexutil.Encode([]byte{39}),
					},
				},
				Signature: hexutil.Encode([]byte{40}),
			},
		},
	}

	result := jsonifyAttesterSlashings(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyDeposits(t *testing.T) {
	input := []*ethpb.Deposit{
		{
			Proof: [][]byte{{1}, {2}},
			Data: &ethpb.Deposit_Data{
				PublicKey:             []byte{3},
				WithdrawalCredentials: []byte{4},
				Amount:                5,
				Signature:             []byte{6},
			},
		},
		{
			Proof: [][]byte{
				{7},
				{8},
			},
			Data: &ethpb.Deposit_Data{
				PublicKey:             []byte{9},
				WithdrawalCredentials: []byte{10},
				Amount:                11,
				Signature:             []byte{12},
			},
		},
	}

	expectedResult := []*shared.Deposit{
		{
			Proof: []string{
				hexutil.Encode([]byte{1}),
				hexutil.Encode([]byte{2}),
			},
			Data: &shared.DepositData{
				Pubkey:                hexutil.Encode([]byte{3}),
				WithdrawalCredentials: hexutil.Encode([]byte{4}),
				Amount:                "5",
				Signature:             hexutil.Encode([]byte{6}),
			},
		},
		{
			Proof: []string{
				hexutil.Encode([]byte{7}),
				hexutil.Encode([]byte{8}),
			},
			Data: &shared.DepositData{
				Pubkey:                hexutil.Encode([]byte{9}),
				WithdrawalCredentials: hexutil.Encode([]byte{10}),
				Amount:                "11",
				Signature:             hexutil.Encode([]byte{12}),
			},
		},
	}

	result := jsonifyDeposits(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyProposerSlashings(t *testing.T) {
	input := []*ethpb.ProposerSlashing{
		{
			Header_1: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot:          1,
					ProposerIndex: 2,
					ParentRoot:    []byte{3},
					StateRoot:     []byte{4},
					BodyRoot:      []byte{5},
				},
				Signature: []byte{6},
			},
			Header_2: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot:          7,
					ProposerIndex: 8,
					ParentRoot:    []byte{9},
					StateRoot:     []byte{10},
					BodyRoot:      []byte{11},
				},
				Signature: []byte{12},
			},
		},
		{
			Header_1: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot:          13,
					ProposerIndex: 14,
					ParentRoot:    []byte{15},
					StateRoot:     []byte{16},
					BodyRoot:      []byte{17},
				},
				Signature: []byte{18},
			},
			Header_2: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot:          19,
					ProposerIndex: 20,
					ParentRoot:    []byte{21},
					StateRoot:     []byte{22},
					BodyRoot:      []byte{23},
				},
				Signature: []byte{24},
			},
		},
	}

	expectedResult := []*shared.ProposerSlashing{
		{
			SignedHeader1: &shared.SignedBeaconBlockHeader{
				Message: &shared.BeaconBlockHeader{
					Slot:          "1",
					ProposerIndex: "2",
					ParentRoot:    hexutil.Encode([]byte{3}),
					StateRoot:     hexutil.Encode([]byte{4}),
					BodyRoot:      hexutil.Encode([]byte{5}),
				},
				Signature: hexutil.Encode([]byte{6}),
			},
			SignedHeader2: &shared.SignedBeaconBlockHeader{
				Message: &shared.BeaconBlockHeader{
					Slot:          "7",
					ProposerIndex: "8",
					ParentRoot:    hexutil.Encode([]byte{9}),
					StateRoot:     hexutil.Encode([]byte{10}),
					BodyRoot:      hexutil.Encode([]byte{11}),
				},
				Signature: hexutil.Encode([]byte{12}),
			},
		},
		{
			SignedHeader1: &shared.SignedBeaconBlockHeader{
				Message: &shared.BeaconBlockHeader{
					Slot:          "13",
					ProposerIndex: "14",
					ParentRoot:    hexutil.Encode([]byte{15}),
					StateRoot:     hexutil.Encode([]byte{16}),
					BodyRoot:      hexutil.Encode([]byte{17}),
				},
				Signature: hexutil.Encode([]byte{18}),
			},
			SignedHeader2: &shared.SignedBeaconBlockHeader{
				Message: &shared.BeaconBlockHeader{
					Slot:          "19",
					ProposerIndex: "20",
					ParentRoot:    hexutil.Encode([]byte{21}),
					StateRoot:     hexutil.Encode([]byte{22}),
					BodyRoot:      hexutil.Encode([]byte{23}),
				},
				Signature: hexutil.Encode([]byte{24}),
			},
		},
	}

	result := jsonifyProposerSlashings(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifySignedVoluntaryExits(t *testing.T) {
	input := []*ethpb.SignedVoluntaryExit{
		{
			Exit: &ethpb.VoluntaryExit{
				Epoch:          1,
				ValidatorIndex: 2,
			},
			Signature: []byte{3},
		},
		{
			Exit: &ethpb.VoluntaryExit{
				Epoch:          4,
				ValidatorIndex: 5,
			},
			Signature: []byte{6},
		},
	}

	expectedResult := []*shared.SignedVoluntaryExit{
		{
			Message: &shared.VoluntaryExit{
				Epoch:          "1",
				ValidatorIndex: "2",
			},
			Signature: hexutil.Encode([]byte{3}),
		},
		{
			Message: &shared.VoluntaryExit{
				Epoch:          "4",
				ValidatorIndex: "5",
			},
			Signature: hexutil.Encode([]byte{6}),
		},
	}

	result := JsonifySignedVoluntaryExits(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifySignedBeaconBlockHeader(t *testing.T) {
	input := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          1,
			ProposerIndex: 2,
			ParentRoot:    []byte{3},
			StateRoot:     []byte{4},
			BodyRoot:      []byte{5},
		},
		Signature: []byte{6},
	}

	expectedResult := &shared.SignedBeaconBlockHeader{
		Message: &shared.BeaconBlockHeader{
			Slot:          "1",
			ProposerIndex: "2",
			ParentRoot:    hexutil.Encode([]byte{3}),
			StateRoot:     hexutil.Encode([]byte{4}),
			BodyRoot:      hexutil.Encode([]byte{5}),
		},
		Signature: hexutil.Encode([]byte{6}),
	}

	result := jsonifySignedBeaconBlockHeader(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyIndexedAttestation(t *testing.T) {
	input := &ethpb.IndexedAttestation{
		AttestingIndices: []uint64{1, 2},
		Data: &ethpb.AttestationData{
			Slot:            3,
			CommitteeIndex:  4,
			BeaconBlockRoot: []byte{5},
			Source: &ethpb.Checkpoint{
				Epoch: 6,
				Root:  []byte{7},
			},
			Target: &ethpb.Checkpoint{
				Epoch: 8,
				Root:  []byte{9},
			},
		},
		Signature: []byte{10},
	}

	expectedResult := &shared.IndexedAttestation{
		AttestingIndices: []string{"1", "2"},
		Data: &shared.AttestationData{
			Slot:            "3",
			CommitteeIndex:  "4",
			BeaconBlockRoot: hexutil.Encode([]byte{5}),
			Source: &shared.Checkpoint{
				Epoch: "6",
				Root:  hexutil.Encode([]byte{7}),
			},
			Target: &shared.Checkpoint{
				Epoch: "8",
				Root:  hexutil.Encode([]byte{9}),
			},
		},
		Signature: hexutil.Encode([]byte{10}),
	}

	result := jsonifyIndexedAttestation(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyAttestationData(t *testing.T) {
	input := &ethpb.AttestationData{
		Slot:            1,
		CommitteeIndex:  2,
		BeaconBlockRoot: []byte{3},
		Source: &ethpb.Checkpoint{
			Epoch: 4,
			Root:  []byte{5},
		},
		Target: &ethpb.Checkpoint{
			Epoch: 6,
			Root:  []byte{7},
		},
	}

	expectedResult := &shared.AttestationData{
		Slot:            "1",
		CommitteeIndex:  "2",
		BeaconBlockRoot: hexutil.Encode([]byte{3}),
		Source: &shared.Checkpoint{
			Epoch: "4",
			Root:  hexutil.Encode([]byte{5}),
		},
		Target: &shared.Checkpoint{
			Epoch: "6",
			Root:  hexutil.Encode([]byte{7}),
		},
	}

	result := jsonifyAttestationData(input)
	assert.DeepEqual(t, expectedResult, result)
}

func TestBeaconBlockJsonHelpers_JsonifyWithdrawals(t *testing.T) {
	input := []*enginev1.Withdrawal{
		{
			Index:          1,
			ValidatorIndex: 2,
			Address:        []byte{3},
			Amount:         4,
		},
		{
			Index:          5,
			ValidatorIndex: 6,
			Address:        []byte{7},
			Amount:         8,
		},
	}

	expectedResult := []*shared.Withdrawal{
		{
			WithdrawalIndex:  "1",
			ValidatorIndex:   "2",
			ExecutionAddress: hexutil.Encode([]byte{3}),
			Amount:           "4",
		},
		{
			WithdrawalIndex:  "5",
			ValidatorIndex:   "6",
			ExecutionAddress: hexutil.Encode([]byte{7}),
			Amount:           "8",
		},
	}

	result := jsonifyWithdrawals(input)
	assert.DeepEqual(t, expectedResult, result)
}
