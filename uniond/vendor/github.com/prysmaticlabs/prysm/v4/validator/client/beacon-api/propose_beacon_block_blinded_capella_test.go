package beacon_api

import (
	"bytes"
	"context"
	"encoding/json"
	"testing"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/mock"
	test_helpers "github.com/prysmaticlabs/prysm/v4/validator/client/beacon-api/test-helpers"
)

func TestProposeBeaconBlock_BlindedCapella(t *testing.T) {
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	jsonRestHandler := mock.NewMockJsonRestHandler(ctrl)

	blindedCapellaBlock := generateSignedBlindedCapellaBlock()

	genericSignedBlock := &ethpb.GenericSignedBeaconBlock{}
	genericSignedBlock.Block = blindedCapellaBlock

	jsonBlindedCapellaBlock := &shared.SignedBlindedBeaconBlockCapella{
		Signature: hexutil.Encode(blindedCapellaBlock.BlindedCapella.Signature),
		Message: &shared.BlindedBeaconBlockCapella{
			ParentRoot:    hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.ParentRoot),
			ProposerIndex: uint64ToString(blindedCapellaBlock.BlindedCapella.Block.ProposerIndex),
			Slot:          uint64ToString(blindedCapellaBlock.BlindedCapella.Block.Slot),
			StateRoot:     hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.StateRoot),
			Body: &shared.BlindedBeaconBlockBodyCapella{
				Attestations:      jsonifyAttestations(blindedCapellaBlock.BlindedCapella.Block.Body.Attestations),
				AttesterSlashings: jsonifyAttesterSlashings(blindedCapellaBlock.BlindedCapella.Block.Body.AttesterSlashings),
				Deposits:          jsonifyDeposits(blindedCapellaBlock.BlindedCapella.Block.Body.Deposits),
				Eth1Data:          jsonifyEth1Data(blindedCapellaBlock.BlindedCapella.Block.Body.Eth1Data),
				Graffiti:          hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.Graffiti),
				ProposerSlashings: jsonifyProposerSlashings(blindedCapellaBlock.BlindedCapella.Block.Body.ProposerSlashings),
				RandaoReveal:      hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.RandaoReveal),
				VoluntaryExits:    JsonifySignedVoluntaryExits(blindedCapellaBlock.BlindedCapella.Block.Body.VoluntaryExits),
				SyncAggregate: &shared.SyncAggregate{
					SyncCommitteeBits:      hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.SyncAggregate.SyncCommitteeBits),
					SyncCommitteeSignature: hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.SyncAggregate.SyncCommitteeSignature),
				},
				ExecutionPayloadHeader: &shared.ExecutionPayloadHeaderCapella{
					BaseFeePerGas:    bytesutil.LittleEndianBytesToBigInt(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.BaseFeePerGas).String(),
					BlockHash:        hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.BlockHash),
					BlockNumber:      uint64ToString(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.BlockNumber),
					ExtraData:        hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.ExtraData),
					FeeRecipient:     hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.FeeRecipient),
					GasLimit:         uint64ToString(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.GasLimit),
					GasUsed:          uint64ToString(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.GasUsed),
					LogsBloom:        hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.LogsBloom),
					ParentHash:       hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.ParentHash),
					PrevRandao:       hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.PrevRandao),
					ReceiptsRoot:     hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.ReceiptsRoot),
					StateRoot:        hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.StateRoot),
					Timestamp:        uint64ToString(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.Timestamp),
					TransactionsRoot: hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.TransactionsRoot),
					WithdrawalsRoot:  hexutil.Encode(blindedCapellaBlock.BlindedCapella.Block.Body.ExecutionPayloadHeader.WithdrawalsRoot),
				},
				BLSToExecutionChanges: jsonifyBlsToExecutionChanges(blindedCapellaBlock.BlindedCapella.Block.Body.BlsToExecutionChanges),
			},
		},
	}

	marshalledBlock, err := json.Marshal(jsonBlindedCapellaBlock)
	require.NoError(t, err)

	ctx := context.Background()

	// Make sure that what we send in the POST body is the marshalled version of the protobuf block
	headers := map[string]string{"Eth-Consensus-Version": "capella"}
	jsonRestHandler.EXPECT().Post(
		ctx,
		"/eth/v1/beacon/blinded_blocks",
		headers,
		bytes.NewBuffer(marshalledBlock),
		nil,
	)

	validatorClient := &beaconApiValidatorClient{jsonRestHandler: jsonRestHandler}
	proposeResponse, err := validatorClient.proposeBeaconBlock(ctx, genericSignedBlock)
	assert.NoError(t, err)
	require.NotNil(t, proposeResponse)

	expectedBlockRoot, err := blindedCapellaBlock.BlindedCapella.Block.HashTreeRoot()
	require.NoError(t, err)

	// Make sure that the block root is set
	assert.DeepEqual(t, expectedBlockRoot[:], proposeResponse.BlockRoot)
}

func generateSignedBlindedCapellaBlock() *ethpb.GenericSignedBeaconBlock_BlindedCapella {
	return &ethpb.GenericSignedBeaconBlock_BlindedCapella{
		BlindedCapella: &ethpb.SignedBlindedBeaconBlockCapella{
			Block: &ethpb.BlindedBeaconBlockCapella{
				Slot:          1,
				ProposerIndex: 2,
				ParentRoot:    test_helpers.FillByteSlice(32, 3),
				StateRoot:     test_helpers.FillByteSlice(32, 4),
				Body: &ethpb.BlindedBeaconBlockBodyCapella{
					RandaoReveal: test_helpers.FillByteSlice(96, 5),
					Eth1Data: &ethpb.Eth1Data{
						DepositRoot:  test_helpers.FillByteSlice(32, 6),
						DepositCount: 7,
						BlockHash:    test_helpers.FillByteSlice(32, 8),
					},
					Graffiti: test_helpers.FillByteSlice(32, 9),
					ProposerSlashings: []*ethpb.ProposerSlashing{
						{
							Header_1: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									Slot:          10,
									ProposerIndex: 11,
									ParentRoot:    test_helpers.FillByteSlice(32, 12),
									StateRoot:     test_helpers.FillByteSlice(32, 13),
									BodyRoot:      test_helpers.FillByteSlice(32, 14),
								},
								Signature: test_helpers.FillByteSlice(96, 15),
							},
							Header_2: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									Slot:          16,
									ProposerIndex: 17,
									ParentRoot:    test_helpers.FillByteSlice(32, 18),
									StateRoot:     test_helpers.FillByteSlice(32, 19),
									BodyRoot:      test_helpers.FillByteSlice(32, 20),
								},
								Signature: test_helpers.FillByteSlice(96, 21),
							},
						},
						{
							Header_1: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									Slot:          22,
									ProposerIndex: 23,
									ParentRoot:    test_helpers.FillByteSlice(32, 24),
									StateRoot:     test_helpers.FillByteSlice(32, 25),
									BodyRoot:      test_helpers.FillByteSlice(32, 26),
								},
								Signature: test_helpers.FillByteSlice(96, 27),
							},
							Header_2: &ethpb.SignedBeaconBlockHeader{
								Header: &ethpb.BeaconBlockHeader{
									Slot:          28,
									ProposerIndex: 29,
									ParentRoot:    test_helpers.FillByteSlice(32, 30),
									StateRoot:     test_helpers.FillByteSlice(32, 31),
									BodyRoot:      test_helpers.FillByteSlice(32, 32),
								},
								Signature: test_helpers.FillByteSlice(96, 33),
							},
						},
					},
					AttesterSlashings: []*ethpb.AttesterSlashing{
						{
							Attestation_1: &ethpb.IndexedAttestation{
								AttestingIndices: []uint64{34, 35},
								Data: &ethpb.AttestationData{
									Slot:            36,
									CommitteeIndex:  37,
									BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
									Source: &ethpb.Checkpoint{
										Epoch: 39,
										Root:  test_helpers.FillByteSlice(32, 40),
									},
									Target: &ethpb.Checkpoint{
										Epoch: 41,
										Root:  test_helpers.FillByteSlice(32, 42),
									},
								},
								Signature: test_helpers.FillByteSlice(96, 43),
							},
							Attestation_2: &ethpb.IndexedAttestation{
								AttestingIndices: []uint64{44, 45},
								Data: &ethpb.AttestationData{
									Slot:            46,
									CommitteeIndex:  47,
									BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
									Source: &ethpb.Checkpoint{
										Epoch: 49,
										Root:  test_helpers.FillByteSlice(32, 50),
									},
									Target: &ethpb.Checkpoint{
										Epoch: 51,
										Root:  test_helpers.FillByteSlice(32, 52),
									},
								},
								Signature: test_helpers.FillByteSlice(96, 53),
							},
						},
						{
							Attestation_1: &ethpb.IndexedAttestation{
								AttestingIndices: []uint64{54, 55},
								Data: &ethpb.AttestationData{
									Slot:            56,
									CommitteeIndex:  57,
									BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
									Source: &ethpb.Checkpoint{
										Epoch: 59,
										Root:  test_helpers.FillByteSlice(32, 60),
									},
									Target: &ethpb.Checkpoint{
										Epoch: 61,
										Root:  test_helpers.FillByteSlice(32, 62),
									},
								},
								Signature: test_helpers.FillByteSlice(96, 63),
							},
							Attestation_2: &ethpb.IndexedAttestation{
								AttestingIndices: []uint64{64, 65},
								Data: &ethpb.AttestationData{
									Slot:            66,
									CommitteeIndex:  67,
									BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
									Source: &ethpb.Checkpoint{
										Epoch: 69,
										Root:  test_helpers.FillByteSlice(32, 70),
									},
									Target: &ethpb.Checkpoint{
										Epoch: 71,
										Root:  test_helpers.FillByteSlice(32, 72),
									},
								},
								Signature: test_helpers.FillByteSlice(96, 73),
							},
						},
					},
					Attestations: []*ethpb.Attestation{
						{
							AggregationBits: test_helpers.FillByteSlice(4, 74),
							Data: &ethpb.AttestationData{
								Slot:            75,
								CommitteeIndex:  76,
								BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
								Source: &ethpb.Checkpoint{
									Epoch: 78,
									Root:  test_helpers.FillByteSlice(32, 79),
								},
								Target: &ethpb.Checkpoint{
									Epoch: 80,
									Root:  test_helpers.FillByteSlice(32, 81),
								},
							},
							Signature: test_helpers.FillByteSlice(96, 82),
						},
						{
							AggregationBits: test_helpers.FillByteSlice(4, 83),
							Data: &ethpb.AttestationData{
								Slot:            84,
								CommitteeIndex:  85,
								BeaconBlockRoot: test_helpers.FillByteSlice(32, 38),
								Source: &ethpb.Checkpoint{
									Epoch: 87,
									Root:  test_helpers.FillByteSlice(32, 88),
								},
								Target: &ethpb.Checkpoint{
									Epoch: 89,
									Root:  test_helpers.FillByteSlice(32, 90),
								},
							},
							Signature: test_helpers.FillByteSlice(96, 91),
						},
					},
					Deposits: []*ethpb.Deposit{
						{
							Proof: test_helpers.FillByteArraySlice(33, test_helpers.FillByteSlice(32, 92)),
							Data: &ethpb.Deposit_Data{
								PublicKey:             test_helpers.FillByteSlice(48, 94),
								WithdrawalCredentials: test_helpers.FillByteSlice(32, 95),
								Amount:                96,
								Signature:             test_helpers.FillByteSlice(96, 97),
							},
						},
						{
							Proof: test_helpers.FillByteArraySlice(33, test_helpers.FillByteSlice(32, 98)),
							Data: &ethpb.Deposit_Data{
								PublicKey:             test_helpers.FillByteSlice(48, 100),
								WithdrawalCredentials: test_helpers.FillByteSlice(32, 101),
								Amount:                102,
								Signature:             test_helpers.FillByteSlice(96, 103),
							},
						},
					},
					VoluntaryExits: []*ethpb.SignedVoluntaryExit{
						{
							Exit: &ethpb.VoluntaryExit{
								Epoch:          104,
								ValidatorIndex: 105,
							},
							Signature: test_helpers.FillByteSlice(96, 106),
						},
						{
							Exit: &ethpb.VoluntaryExit{
								Epoch:          107,
								ValidatorIndex: 108,
							},
							Signature: test_helpers.FillByteSlice(96, 109),
						},
					},
					SyncAggregate: &ethpb.SyncAggregate{
						SyncCommitteeBits:      test_helpers.FillByteSlice(64, 110),
						SyncCommitteeSignature: test_helpers.FillByteSlice(96, 111),
					},
					ExecutionPayloadHeader: &enginev1.ExecutionPayloadHeaderCapella{
						ParentHash:       test_helpers.FillByteSlice(32, 112),
						FeeRecipient:     test_helpers.FillByteSlice(20, 113),
						StateRoot:        test_helpers.FillByteSlice(32, 114),
						ReceiptsRoot:     test_helpers.FillByteSlice(32, 115),
						LogsBloom:        test_helpers.FillByteSlice(256, 116),
						PrevRandao:       test_helpers.FillByteSlice(32, 117),
						BlockNumber:      118,
						GasLimit:         119,
						GasUsed:          120,
						Timestamp:        121,
						ExtraData:        test_helpers.FillByteSlice(32, 122),
						BaseFeePerGas:    test_helpers.FillByteSlice(32, 123),
						BlockHash:        test_helpers.FillByteSlice(32, 124),
						TransactionsRoot: test_helpers.FillByteSlice(32, 125),
						WithdrawalsRoot:  test_helpers.FillByteSlice(32, 126),
					},
					BlsToExecutionChanges: []*ethpb.SignedBLSToExecutionChange{
						{
							Message: &ethpb.BLSToExecutionChange{
								ValidatorIndex:     127,
								FromBlsPubkey:      test_helpers.FillByteSlice(48, 128),
								ToExecutionAddress: test_helpers.FillByteSlice(20, 129),
							},
							Signature: test_helpers.FillByteSlice(96, 130),
						},
						{
							Message: &ethpb.BLSToExecutionChange{
								ValidatorIndex:     131,
								FromBlsPubkey:      test_helpers.FillByteSlice(48, 132),
								ToExecutionAddress: test_helpers.FillByteSlice(20, 133),
							},
							Signature: test_helpers.FillByteSlice(96, 134),
						},
					},
				},
			},
			Signature: test_helpers.FillByteSlice(96, 135),
		},
	}
}
