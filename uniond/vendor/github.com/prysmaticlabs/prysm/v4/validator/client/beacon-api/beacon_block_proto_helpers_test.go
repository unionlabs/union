package beacon_api

import (
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestBeaconBlockProtoHelpers_ConvertProposerSlashingsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.ProposerSlashing
		expectedResult       []*ethpb.ProposerSlashing
		expectedErrorMessage string
	}{
		{
			name:                 "nil proposer slashing",
			expectedErrorMessage: "proposer slashing at index `0` is nil",
			generateInput: func() []*shared.ProposerSlashing {
				return []*shared.ProposerSlashing{
					nil,
				}
			},
		},
		{
			name:                 "bad header 1",
			expectedErrorMessage: "failed to get proposer header 1",
			generateInput: func() []*shared.ProposerSlashing {
				input := generateProposerSlashings()
				input[0].SignedHeader1 = nil
				return input
			},
		},
		{
			name:                 "bad header 2",
			expectedErrorMessage: "failed to get proposer header 2",
			generateInput: func() []*shared.ProposerSlashing {
				input := generateProposerSlashings()
				input[0].SignedHeader2 = nil
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateProposerSlashings,
			expectedResult: []*ethpb.ProposerSlashing{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertProposerSlashingsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertProposerSlashingSignedHeaderToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() *shared.SignedBeaconBlockHeader
		expectedResult       *ethpb.SignedBeaconBlockHeader
		expectedErrorMessage string
	}{
		{
			name:                 "nil signed header",
			expectedErrorMessage: "signed header is nil",
			generateInput:        func() *shared.SignedBeaconBlockHeader { return nil },
		},
		{
			name:                 "nil header",
			expectedErrorMessage: "header is nil",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message = nil
				return input
			},
		},
		{
			name:                 "bad slot",
			expectedErrorMessage: "failed to parse header slot `foo`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message.Slot = "foo"
				return input
			},
		},
		{
			name:                 "bad proposer index",
			expectedErrorMessage: "failed to parse header proposer index `bar`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message.ProposerIndex = "bar"
				return input
			},
		},
		{
			name:                 "bad parent root",
			expectedErrorMessage: "failed to decode header parent root `foo`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message.ParentRoot = "foo"
				return input
			},
		},
		{
			name:                 "bad state root",
			expectedErrorMessage: "failed to decode header state root `bar`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message.StateRoot = "bar"
				return input
			},
		},
		{
			name:                 "bad body root",
			expectedErrorMessage: "failed to decode header body root `foo`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Message.BodyRoot = "foo"
				return input
			},
		},
		{
			name:                 "bad parent root",
			expectedErrorMessage: "failed to decode signature `bar`",
			generateInput: func() *shared.SignedBeaconBlockHeader {
				input := generateSignedBeaconBlockHeader()
				input.Signature = "bar"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateSignedBeaconBlockHeader,
			expectedResult: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot:          1,
					ProposerIndex: 2,
					ParentRoot:    []byte{3},
					StateRoot:     []byte{4},
					BodyRoot:      []byte{5},
				},
				Signature: []byte{6},
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertProposerSlashingSignedHeaderToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertAttesterSlashingsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.AttesterSlashing
		expectedResult       []*ethpb.AttesterSlashing
		expectedErrorMessage string
	}{
		{
			name:                 "nil attester slashing",
			expectedErrorMessage: "attester slashing at index `0` is nil",
			generateInput: func() []*shared.AttesterSlashing {
				return []*shared.AttesterSlashing{
					nil,
				}
			},
		},
		{
			name:                 "bad attestation 1",
			expectedErrorMessage: "failed to get attestation 1",
			generateInput: func() []*shared.AttesterSlashing {
				return []*shared.AttesterSlashing{
					{
						Attestation1: nil,
						Attestation2: nil,
					},
				}
			},
		},
		{
			name:                 "bad attestation 2",
			expectedErrorMessage: "failed to get attestation 2",
			generateInput: func() []*shared.AttesterSlashing {
				input := generateAttesterSlashings()
				input[0].Attestation2 = nil
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateAttesterSlashings,
			expectedResult: []*ethpb.AttesterSlashing{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertAttesterSlashingsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertAttestationToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() *shared.IndexedAttestation
		expectedResult       *ethpb.IndexedAttestation
		expectedErrorMessage string
	}{
		{
			name:                 "nil indexed attestation",
			expectedErrorMessage: "indexed attestation is nil",
			generateInput:        func() *shared.IndexedAttestation { return nil },
		},
		{
			name:                 "bad attesting index",
			expectedErrorMessage: "failed to parse attesting index `foo`",
			generateInput: func() *shared.IndexedAttestation {
				input := generateIndexedAttestation()
				input.AttestingIndices[0] = "foo"
				return input
			},
		},
		{
			name:                 "bad signature",
			expectedErrorMessage: "failed to decode attestation signature `bar`",
			generateInput: func() *shared.IndexedAttestation {
				input := generateIndexedAttestation()
				input.Signature = "bar"
				return input
			},
		},
		{
			name:                 "bad data",
			expectedErrorMessage: "failed to get attestation data",
			generateInput: func() *shared.IndexedAttestation {
				input := generateIndexedAttestation()
				input.Data = nil
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateIndexedAttestation,
			expectedResult: &ethpb.IndexedAttestation{
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
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertIndexedAttestationToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertCheckpointToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() *shared.Checkpoint
		expectedResult       *ethpb.Checkpoint
		expectedErrorMessage string
	}{
		{
			name:                 "nil checkpoint",
			expectedErrorMessage: "checkpoint is nil",
			generateInput:        func() *shared.Checkpoint { return nil },
		},
		{
			name:                 "bad epoch",
			expectedErrorMessage: "failed to parse checkpoint epoch `foo`",
			generateInput: func() *shared.Checkpoint {
				input := generateCheckpoint()
				input.Epoch = "foo"
				return input
			},
		},
		{
			name:                 "bad root",
			expectedErrorMessage: "failed to decode checkpoint root `bar`",
			generateInput: func() *shared.Checkpoint {
				input := generateCheckpoint()
				input.Root = "bar"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateCheckpoint,
			expectedResult: &ethpb.Checkpoint{
				Epoch: 1,
				Root:  []byte{2},
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertCheckpointToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertAttestationsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.Attestation
		expectedResult       []*ethpb.Attestation
		expectedErrorMessage string
	}{
		{
			name:                 "nil attestation",
			expectedErrorMessage: "attestation at index `0` is nil",
			generateInput: func() []*shared.Attestation {
				return []*shared.Attestation{
					nil,
				}
			},
		},
		{
			name:                 "bad aggregation bits",
			expectedErrorMessage: "failed to decode aggregation bits `foo`",
			generateInput: func() []*shared.Attestation {
				input := generateAttestations()
				input[0].AggregationBits = "foo"
				return input
			},
		},
		{
			name:                 "bad data",
			expectedErrorMessage: "failed to get attestation data",
			generateInput: func() []*shared.Attestation {
				input := generateAttestations()
				input[0].Data = nil
				return input
			},
		},
		{
			name:                 "bad signature",
			expectedErrorMessage: "failed to decode attestation signature `bar`",
			generateInput: func() []*shared.Attestation {
				input := generateAttestations()
				input[0].Signature = "bar"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateAttestations,
			expectedResult: []*ethpb.Attestation{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertAttestationsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertAttestationDataToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() *shared.AttestationData
		expectedResult       *ethpb.AttestationData
		expectedErrorMessage string
	}{
		{
			name:                 "nil attestation data",
			expectedErrorMessage: "attestation data is nil",
			generateInput:        func() *shared.AttestationData { return nil },
		},
		{
			name:                 "bad slot",
			expectedErrorMessage: "failed to parse attestation slot `foo`",
			generateInput: func() *shared.AttestationData {
				input := generateAttestationData()
				input.Slot = "foo"
				return input
			},
		},
		{
			name:                 "bad committee index",
			expectedErrorMessage: "failed to parse attestation committee index `bar`",
			generateInput: func() *shared.AttestationData {
				input := generateAttestationData()
				input.CommitteeIndex = "bar"
				return input
			},
		},
		{
			name:                 "bad beacon block root",
			expectedErrorMessage: "failed to decode attestation beacon block root `foo`",
			generateInput: func() *shared.AttestationData {
				input := generateAttestationData()
				input.BeaconBlockRoot = "foo"
				return input
			},
		},
		{
			name:                 "bad source checkpoint",
			expectedErrorMessage: "failed to get attestation source checkpoint",
			generateInput: func() *shared.AttestationData {
				input := generateAttestationData()
				input.Source = nil
				return input
			},
		},
		{
			name:                 "bad target checkpoint",
			expectedErrorMessage: "failed to get attestation target checkpoint",
			generateInput: func() *shared.AttestationData {
				input := generateAttestationData()
				input.Target = nil
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateAttestationData,
			expectedResult: &ethpb.AttestationData{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertAttestationDataToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertDepositsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.Deposit
		expectedResult       []*ethpb.Deposit
		expectedErrorMessage string
	}{
		{
			name:                 "nil deposit",
			expectedErrorMessage: "deposit at index `0` is nil",
			generateInput: func() []*shared.Deposit {
				return []*shared.Deposit{
					nil,
				}
			},
		},
		{
			name:                 "bad proof",
			expectedErrorMessage: "failed to decode deposit proof `foo`",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Proof[0] = "foo"
				return input
			},
		},
		{
			name:                 "nil data",
			expectedErrorMessage: "deposit data at index `0` is nil",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Data = nil
				return input
			},
		},
		{
			name:                 "bad public key",
			expectedErrorMessage: "failed to decode deposit public key `bar`",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Data.Pubkey = "bar"
				return input
			},
		},
		{
			name:                 "bad withdrawal credentials",
			expectedErrorMessage: "failed to decode deposit withdrawal credentials `foo`",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Data.WithdrawalCredentials = "foo"
				return input
			},
		},
		{
			name:                 "bad amount",
			expectedErrorMessage: "failed to parse deposit amount `bar`",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Data.Amount = "bar"
				return input
			},
		},
		{
			name:                 "bad signature",
			expectedErrorMessage: "failed to decode signature `foo`",
			generateInput: func() []*shared.Deposit {
				input := generateDeposits()
				input[0].Data.Signature = "foo"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateDeposits,
			expectedResult: []*ethpb.Deposit{
				{
					Proof: [][]byte{
						{1},
						{2},
					},
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertDepositsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertVoluntaryExitsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.SignedVoluntaryExit
		expectedResult       []*ethpb.SignedVoluntaryExit
		expectedErrorMessage string
	}{
		{
			name:                 "nil voluntary exit",
			expectedErrorMessage: "signed voluntary exit at index `0` is nil",
			generateInput: func() []*shared.SignedVoluntaryExit {
				return []*shared.SignedVoluntaryExit{
					nil,
				}
			},
		},
		{
			name:                 "nil data",
			expectedErrorMessage: "voluntary exit at index `0` is nil",
			generateInput: func() []*shared.SignedVoluntaryExit {
				input := generateSignedVoluntaryExits()
				input[0].Message = nil
				return input
			},
		},
		{
			name:                 "bad epoch",
			expectedErrorMessage: "failed to parse voluntary exit epoch `foo`",
			generateInput: func() []*shared.SignedVoluntaryExit {
				input := generateSignedVoluntaryExits()
				input[0].Message.Epoch = "foo"
				return input
			},
		},
		{
			name:                 "bad validator index",
			expectedErrorMessage: "failed to parse voluntary exit validator index `bar`",
			generateInput: func() []*shared.SignedVoluntaryExit {
				input := generateSignedVoluntaryExits()
				input[0].Message.ValidatorIndex = "bar"
				return input
			},
		},
		{
			name:                 "bad signature",
			expectedErrorMessage: "failed to decode signature `foo`",
			generateInput: func() []*shared.SignedVoluntaryExit {
				input := generateSignedVoluntaryExits()
				input[0].Signature = "foo"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateSignedVoluntaryExits,
			expectedResult: []*ethpb.SignedVoluntaryExit{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertVoluntaryExitsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertTransactionsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []string
		expectedResult       [][]byte
		expectedErrorMessage string
	}{
		{
			name:                 "bad transaction",
			expectedErrorMessage: "failed to decode transaction `foo`",
			generateInput: func() []string {
				return []string{
					"foo",
				}
			},
		},
		{
			name: "valid",
			generateInput: func() []string {
				return []string{
					hexutil.Encode([]byte{1}),
					hexutil.Encode([]byte{2}),
				}
			},
			expectedResult: [][]byte{
				{1},
				{2},
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertTransactionsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertWithdrawalsToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.Withdrawal
		expectedResult       []*enginev1.Withdrawal
		expectedErrorMessage string
	}{
		{
			name:                 "nil withdrawal",
			expectedErrorMessage: "withdrawal at index `0` is nil",
			generateInput: func() []*shared.Withdrawal {
				input := generateWithdrawals()
				input[0] = nil
				return input
			},
		},
		{
			name:                 "bad withdrawal index",
			expectedErrorMessage: "failed to parse withdrawal index `foo`",
			generateInput: func() []*shared.Withdrawal {
				input := generateWithdrawals()
				input[0].WithdrawalIndex = "foo"
				return input
			},
		},
		{
			name:                 "bad validator index",
			expectedErrorMessage: "failed to parse validator index `bar`",
			generateInput: func() []*shared.Withdrawal {
				input := generateWithdrawals()
				input[0].ValidatorIndex = "bar"
				return input
			},
		},
		{
			name:                 "bad execution address",
			expectedErrorMessage: "failed to decode execution address `foo`",
			generateInput: func() []*shared.Withdrawal {
				input := generateWithdrawals()
				input[0].ExecutionAddress = "foo"
				return input
			},
		},
		{
			name:                 "bad amount",
			expectedErrorMessage: "failed to parse withdrawal amount `bar`",
			generateInput: func() []*shared.Withdrawal {
				input := generateWithdrawals()
				input[0].Amount = "bar"
				return input
			},
		},
		{
			name:          "valid",
			generateInput: generateWithdrawals,
			expectedResult: []*enginev1.Withdrawal{
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
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertWithdrawalsToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func TestBeaconBlockProtoHelpers_ConvertBlsToExecutionChangesToProto(t *testing.T) {
	testCases := []struct {
		name                 string
		generateInput        func() []*shared.SignedBLSToExecutionChange
		expectedResult       []*ethpb.SignedBLSToExecutionChange
		expectedErrorMessage string
	}{
		{
			name:                 "nil bls to execution change",
			expectedErrorMessage: "bls to execution change at index `0` is nil",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0] = nil
				return input
			},
		},
		{
			name:                 "nil bls to execution change message",
			expectedErrorMessage: "bls to execution change message at index `0` is nil",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0].Message = nil
				return input
			},
		},
		{
			name:                 "bad validator index",
			expectedErrorMessage: "failed to decode validator index `foo`",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0].Message.ValidatorIndex = "foo"
				return input
			},
		},
		{
			name:                 "bad from bls pubkey",
			expectedErrorMessage: "failed to decode bls pubkey `bar`",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0].Message.FromBLSPubkey = "bar"
				return input
			},
		},
		{
			name:                 "bad to execution address",
			expectedErrorMessage: "failed to decode execution address `foo`",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0].Message.ToExecutionAddress = "foo"
				return input
			},
		},
		{
			name:                 "bad signature",
			expectedErrorMessage: "failed to decode signature `bar`",
			generateInput: func() []*shared.SignedBLSToExecutionChange {
				input := generateBlsToExecutionChanges()
				input[0].Signature = "bar"
				return input
			},
		},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			result, err := convertBlsToExecutionChangesToProto(testCase.generateInput())

			if testCase.expectedResult != nil {
				require.NoError(t, err)
				assert.DeepEqual(t, testCase.expectedResult, result)
			} else if testCase.expectedErrorMessage != "" {
				assert.ErrorContains(t, testCase.expectedErrorMessage, err)
			}
		})
	}
}

func generateProposerSlashings() []*shared.ProposerSlashing {
	return []*shared.ProposerSlashing{
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
}

func generateSignedBeaconBlockHeader() *shared.SignedBeaconBlockHeader {
	return &shared.SignedBeaconBlockHeader{
		Message: &shared.BeaconBlockHeader{
			Slot:          "1",
			ProposerIndex: "2",
			ParentRoot:    hexutil.Encode([]byte{3}),
			StateRoot:     hexutil.Encode([]byte{4}),
			BodyRoot:      hexutil.Encode([]byte{5}),
		},
		Signature: hexutil.Encode([]byte{6}),
	}
}

func generateAttesterSlashings() []*shared.AttesterSlashing {
	return []*shared.AttesterSlashing{
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
}

func generateIndexedAttestation() *shared.IndexedAttestation {
	return &shared.IndexedAttestation{
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
}

func generateCheckpoint() *shared.Checkpoint {
	return &shared.Checkpoint{
		Epoch: "1",
		Root:  hexutil.Encode([]byte{2}),
	}
}

func generateAttestations() []*shared.Attestation {
	return []*shared.Attestation{
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
}

func generateAttestationData() *shared.AttestationData {
	return &shared.AttestationData{
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
}

func generateDeposits() []*shared.Deposit {
	return []*shared.Deposit{
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
}

func generateSignedVoluntaryExits() []*shared.SignedVoluntaryExit {
	return []*shared.SignedVoluntaryExit{
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
}

func generateWithdrawals() []*shared.Withdrawal {
	return []*shared.Withdrawal{
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
}

func generateBlsToExecutionChanges() []*shared.SignedBLSToExecutionChange {
	return []*shared.SignedBLSToExecutionChange{
		{
			Message: &shared.BLSToExecutionChange{
				ValidatorIndex:     "1",
				FromBLSPubkey:      hexutil.Encode([]byte{2}),
				ToExecutionAddress: hexutil.Encode([]byte{3}),
			},
			Signature: hexutil.Encode([]byte{4}),
		},
		{
			Message: &shared.BLSToExecutionChange{
				ValidatorIndex:     "5",
				FromBLSPubkey:      hexutil.Encode([]byte{6}),
				ToExecutionAddress: hexutil.Encode([]byte{7}),
			},
			Signature: hexutil.Encode([]byte{8}),
		},
	}
}
