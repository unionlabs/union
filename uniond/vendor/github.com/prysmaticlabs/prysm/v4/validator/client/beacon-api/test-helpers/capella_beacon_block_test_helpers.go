package test_helpers

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func GenerateProtoCapellaBeaconBlock() *ethpb.BeaconBlockCapella {
	return &ethpb.BeaconBlockCapella{
		Slot:          1,
		ProposerIndex: 2,
		ParentRoot:    FillByteSlice(32, 3),
		StateRoot:     FillByteSlice(32, 4),
		Body: &ethpb.BeaconBlockBodyCapella{
			RandaoReveal: FillByteSlice(96, 5),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot:  FillByteSlice(32, 6),
				DepositCount: 7,
				BlockHash:    FillByteSlice(32, 8),
			},
			Graffiti: FillByteSlice(32, 9),
			ProposerSlashings: []*ethpb.ProposerSlashing{
				{
					Header_1: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          10,
							ProposerIndex: 11,
							ParentRoot:    FillByteSlice(32, 12),
							StateRoot:     FillByteSlice(32, 13),
							BodyRoot:      FillByteSlice(32, 14),
						},
						Signature: FillByteSlice(96, 15),
					},
					Header_2: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          16,
							ProposerIndex: 17,
							ParentRoot:    FillByteSlice(32, 18),
							StateRoot:     FillByteSlice(32, 19),
							BodyRoot:      FillByteSlice(32, 20),
						},
						Signature: FillByteSlice(96, 21),
					},
				},
				{
					Header_1: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          22,
							ProposerIndex: 23,
							ParentRoot:    FillByteSlice(32, 24),
							StateRoot:     FillByteSlice(32, 25),
							BodyRoot:      FillByteSlice(32, 26),
						},
						Signature: FillByteSlice(96, 27),
					},
					Header_2: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          28,
							ProposerIndex: 29,
							ParentRoot:    FillByteSlice(32, 30),
							StateRoot:     FillByteSlice(32, 31),
							BodyRoot:      FillByteSlice(32, 32),
						},
						Signature: FillByteSlice(96, 33),
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
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 39,
								Root:  FillByteSlice(32, 40),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 41,
								Root:  FillByteSlice(32, 42),
							},
						},
						Signature: FillByteSlice(96, 43),
					},
					Attestation_2: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{44, 45},
						Data: &ethpb.AttestationData{
							Slot:            46,
							CommitteeIndex:  47,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 49,
								Root:  FillByteSlice(32, 50),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 51,
								Root:  FillByteSlice(32, 52),
							},
						},
						Signature: FillByteSlice(96, 53),
					},
				},
				{
					Attestation_1: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{54, 55},
						Data: &ethpb.AttestationData{
							Slot:            56,
							CommitteeIndex:  57,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 59,
								Root:  FillByteSlice(32, 60),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 61,
								Root:  FillByteSlice(32, 62),
							},
						},
						Signature: FillByteSlice(96, 63),
					},
					Attestation_2: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{64, 65},
						Data: &ethpb.AttestationData{
							Slot:            66,
							CommitteeIndex:  67,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 69,
								Root:  FillByteSlice(32, 70),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 71,
								Root:  FillByteSlice(32, 72),
							},
						},
						Signature: FillByteSlice(96, 73),
					},
				},
			},
			Attestations: []*ethpb.Attestation{
				{
					AggregationBits: FillByteSlice(4, 74),
					Data: &ethpb.AttestationData{
						Slot:            75,
						CommitteeIndex:  76,
						BeaconBlockRoot: FillByteSlice(32, 38),
						Source: &ethpb.Checkpoint{
							Epoch: 78,
							Root:  FillByteSlice(32, 79),
						},
						Target: &ethpb.Checkpoint{
							Epoch: 80,
							Root:  FillByteSlice(32, 81),
						},
					},
					Signature: FillByteSlice(96, 82),
				},
				{
					AggregationBits: FillByteSlice(4, 83),
					Data: &ethpb.AttestationData{
						Slot:            84,
						CommitteeIndex:  85,
						BeaconBlockRoot: FillByteSlice(32, 38),
						Source: &ethpb.Checkpoint{
							Epoch: 87,
							Root:  FillByteSlice(32, 88),
						},
						Target: &ethpb.Checkpoint{
							Epoch: 89,
							Root:  FillByteSlice(32, 90),
						},
					},
					Signature: FillByteSlice(96, 91),
				},
			},
			Deposits: []*ethpb.Deposit{
				{
					Proof: FillByteArraySlice(33, FillByteSlice(32, 92)),
					Data: &ethpb.Deposit_Data{
						PublicKey:             FillByteSlice(48, 94),
						WithdrawalCredentials: FillByteSlice(32, 95),
						Amount:                96,
						Signature:             FillByteSlice(96, 97),
					},
				},
				{
					Proof: FillByteArraySlice(33, FillByteSlice(32, 98)),
					Data: &ethpb.Deposit_Data{
						PublicKey:             FillByteSlice(48, 100),
						WithdrawalCredentials: FillByteSlice(32, 101),
						Amount:                102,
						Signature:             FillByteSlice(96, 103),
					},
				},
			},
			VoluntaryExits: []*ethpb.SignedVoluntaryExit{
				{
					Exit: &ethpb.VoluntaryExit{
						Epoch:          104,
						ValidatorIndex: 105,
					},
					Signature: FillByteSlice(96, 106),
				},
				{
					Exit: &ethpb.VoluntaryExit{
						Epoch:          107,
						ValidatorIndex: 108,
					},
					Signature: FillByteSlice(96, 109),
				},
			},
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      FillByteSlice(64, 110),
				SyncCommitteeSignature: FillByteSlice(96, 111),
			},
			ExecutionPayload: &enginev1.ExecutionPayloadCapella{
				ParentHash:    FillByteSlice(32, 112),
				FeeRecipient:  FillByteSlice(20, 113),
				StateRoot:     FillByteSlice(32, 114),
				ReceiptsRoot:  FillByteSlice(32, 115),
				LogsBloom:     FillByteSlice(256, 116),
				PrevRandao:    FillByteSlice(32, 117),
				BlockNumber:   118,
				GasLimit:      119,
				GasUsed:       120,
				Timestamp:     121,
				ExtraData:     FillByteSlice(32, 122),
				BaseFeePerGas: FillByteSlice(32, 123),
				BlockHash:     FillByteSlice(32, 124),
				Transactions: [][]byte{
					FillByteSlice(32, 125),
					FillByteSlice(32, 126),
				},
				Withdrawals: []*enginev1.Withdrawal{
					{
						Index:          127,
						ValidatorIndex: 128,
						Address:        FillByteSlice(20, 129),
						Amount:         130,
					},
					{
						Index:          131,
						ValidatorIndex: 132,
						Address:        FillByteSlice(20, 133),
						Amount:         134,
					},
				},
			},
			BlsToExecutionChanges: []*ethpb.SignedBLSToExecutionChange{
				{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     135,
						FromBlsPubkey:      FillByteSlice(48, 136),
						ToExecutionAddress: FillByteSlice(20, 137),
					},
					Signature: FillByteSlice(96, 138),
				},
				{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     139,
						FromBlsPubkey:      FillByteSlice(48, 140),
						ToExecutionAddress: FillByteSlice(20, 141),
					},
					Signature: FillByteSlice(96, 142),
				},
			},
		},
	}
}

func GenerateProtoBlindedCapellaBeaconBlock() *ethpb.BlindedBeaconBlockCapella {
	return &ethpb.BlindedBeaconBlockCapella{
		Slot:          1,
		ProposerIndex: 2,
		ParentRoot:    FillByteSlice(32, 3),
		StateRoot:     FillByteSlice(32, 4),
		Body: &ethpb.BlindedBeaconBlockBodyCapella{
			RandaoReveal: FillByteSlice(96, 5),
			Eth1Data: &ethpb.Eth1Data{
				DepositRoot:  FillByteSlice(32, 6),
				DepositCount: 7,
				BlockHash:    FillByteSlice(32, 8),
			},
			Graffiti: FillByteSlice(32, 9),
			ProposerSlashings: []*ethpb.ProposerSlashing{
				{
					Header_1: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          10,
							ProposerIndex: 11,
							ParentRoot:    FillByteSlice(32, 12),
							StateRoot:     FillByteSlice(32, 13),
							BodyRoot:      FillByteSlice(32, 14),
						},
						Signature: FillByteSlice(96, 15),
					},
					Header_2: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          16,
							ProposerIndex: 17,
							ParentRoot:    FillByteSlice(32, 18),
							StateRoot:     FillByteSlice(32, 19),
							BodyRoot:      FillByteSlice(32, 20),
						},
						Signature: FillByteSlice(96, 21),
					},
				},
				{
					Header_1: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          22,
							ProposerIndex: 23,
							ParentRoot:    FillByteSlice(32, 24),
							StateRoot:     FillByteSlice(32, 25),
							BodyRoot:      FillByteSlice(32, 26),
						},
						Signature: FillByteSlice(96, 27),
					},
					Header_2: &ethpb.SignedBeaconBlockHeader{
						Header: &ethpb.BeaconBlockHeader{
							Slot:          28,
							ProposerIndex: 29,
							ParentRoot:    FillByteSlice(32, 30),
							StateRoot:     FillByteSlice(32, 31),
							BodyRoot:      FillByteSlice(32, 32),
						},
						Signature: FillByteSlice(96, 33),
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
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 39,
								Root:  FillByteSlice(32, 40),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 41,
								Root:  FillByteSlice(32, 42),
							},
						},
						Signature: FillByteSlice(96, 43),
					},
					Attestation_2: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{44, 45},
						Data: &ethpb.AttestationData{
							Slot:            46,
							CommitteeIndex:  47,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 49,
								Root:  FillByteSlice(32, 50),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 51,
								Root:  FillByteSlice(32, 52),
							},
						},
						Signature: FillByteSlice(96, 53),
					},
				},
				{
					Attestation_1: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{54, 55},
						Data: &ethpb.AttestationData{
							Slot:            56,
							CommitteeIndex:  57,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 59,
								Root:  FillByteSlice(32, 60),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 61,
								Root:  FillByteSlice(32, 62),
							},
						},
						Signature: FillByteSlice(96, 63),
					},
					Attestation_2: &ethpb.IndexedAttestation{
						AttestingIndices: []uint64{64, 65},
						Data: &ethpb.AttestationData{
							Slot:            66,
							CommitteeIndex:  67,
							BeaconBlockRoot: FillByteSlice(32, 38),
							Source: &ethpb.Checkpoint{
								Epoch: 69,
								Root:  FillByteSlice(32, 70),
							},
							Target: &ethpb.Checkpoint{
								Epoch: 71,
								Root:  FillByteSlice(32, 72),
							},
						},
						Signature: FillByteSlice(96, 73),
					},
				},
			},
			Attestations: []*ethpb.Attestation{
				{
					AggregationBits: FillByteSlice(4, 74),
					Data: &ethpb.AttestationData{
						Slot:            75,
						CommitteeIndex:  76,
						BeaconBlockRoot: FillByteSlice(32, 38),
						Source: &ethpb.Checkpoint{
							Epoch: 78,
							Root:  FillByteSlice(32, 79),
						},
						Target: &ethpb.Checkpoint{
							Epoch: 80,
							Root:  FillByteSlice(32, 81),
						},
					},
					Signature: FillByteSlice(96, 82),
				},
				{
					AggregationBits: FillByteSlice(4, 83),
					Data: &ethpb.AttestationData{
						Slot:            84,
						CommitteeIndex:  85,
						BeaconBlockRoot: FillByteSlice(32, 38),
						Source: &ethpb.Checkpoint{
							Epoch: 87,
							Root:  FillByteSlice(32, 88),
						},
						Target: &ethpb.Checkpoint{
							Epoch: 89,
							Root:  FillByteSlice(32, 90),
						},
					},
					Signature: FillByteSlice(96, 91),
				},
			},
			Deposits: []*ethpb.Deposit{
				{
					Proof: FillByteArraySlice(33, FillByteSlice(32, 92)),
					Data: &ethpb.Deposit_Data{
						PublicKey:             FillByteSlice(48, 94),
						WithdrawalCredentials: FillByteSlice(32, 95),
						Amount:                96,
						Signature:             FillByteSlice(96, 97),
					},
				},
				{
					Proof: FillByteArraySlice(33, FillByteSlice(32, 98)),
					Data: &ethpb.Deposit_Data{
						PublicKey:             FillByteSlice(48, 100),
						WithdrawalCredentials: FillByteSlice(32, 101),
						Amount:                102,
						Signature:             FillByteSlice(96, 103),
					},
				},
			},
			VoluntaryExits: []*ethpb.SignedVoluntaryExit{
				{
					Exit: &ethpb.VoluntaryExit{
						Epoch:          104,
						ValidatorIndex: 105,
					},
					Signature: FillByteSlice(96, 106),
				},
				{
					Exit: &ethpb.VoluntaryExit{
						Epoch:          107,
						ValidatorIndex: 108,
					},
					Signature: FillByteSlice(96, 109),
				},
			},
			SyncAggregate: &ethpb.SyncAggregate{
				SyncCommitteeBits:      FillByteSlice(64, 110),
				SyncCommitteeSignature: FillByteSlice(96, 111),
			},
			ExecutionPayloadHeader: &enginev1.ExecutionPayloadHeaderCapella{
				ParentHash:       FillByteSlice(32, 112),
				FeeRecipient:     FillByteSlice(20, 113),
				StateRoot:        FillByteSlice(32, 114),
				ReceiptsRoot:     FillByteSlice(32, 115),
				LogsBloom:        FillByteSlice(256, 116),
				PrevRandao:       FillByteSlice(32, 117),
				BlockNumber:      118,
				GasLimit:         119,
				GasUsed:          120,
				Timestamp:        121,
				ExtraData:        FillByteSlice(32, 122),
				BaseFeePerGas:    FillByteSlice(32, 123),
				BlockHash:        FillByteSlice(32, 124),
				TransactionsRoot: FillByteSlice(32, 125),
				WithdrawalsRoot:  FillByteSlice(32, 126),
			},
			BlsToExecutionChanges: []*ethpb.SignedBLSToExecutionChange{
				{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     135,
						FromBlsPubkey:      FillByteSlice(48, 136),
						ToExecutionAddress: FillByteSlice(20, 137),
					},
					Signature: FillByteSlice(96, 138),
				},
				{
					Message: &ethpb.BLSToExecutionChange{
						ValidatorIndex:     139,
						FromBlsPubkey:      FillByteSlice(48, 140),
						ToExecutionAddress: FillByteSlice(20, 141),
					},
					Signature: FillByteSlice(96, 142),
				},
			},
		},
	}
}

func GenerateJsonCapellaBeaconBlock() *shared.BeaconBlockCapella {
	return &shared.BeaconBlockCapella{
		Slot:          "1",
		ProposerIndex: "2",
		ParentRoot:    FillEncodedByteSlice(32, 3),
		StateRoot:     FillEncodedByteSlice(32, 4),
		Body: &shared.BeaconBlockBodyCapella{
			RandaoReveal: FillEncodedByteSlice(96, 5),
			Eth1Data: &shared.Eth1Data{
				DepositRoot:  FillEncodedByteSlice(32, 6),
				DepositCount: "7",
				BlockHash:    FillEncodedByteSlice(32, 8),
			},
			Graffiti: FillEncodedByteSlice(32, 9),
			ProposerSlashings: []*shared.ProposerSlashing{
				{
					SignedHeader1: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "10",
							ProposerIndex: "11",
							ParentRoot:    FillEncodedByteSlice(32, 12),
							StateRoot:     FillEncodedByteSlice(32, 13),
							BodyRoot:      FillEncodedByteSlice(32, 14),
						},
						Signature: FillEncodedByteSlice(96, 15),
					},
					SignedHeader2: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "16",
							ProposerIndex: "17",
							ParentRoot:    FillEncodedByteSlice(32, 18),
							StateRoot:     FillEncodedByteSlice(32, 19),
							BodyRoot:      FillEncodedByteSlice(32, 20),
						},
						Signature: FillEncodedByteSlice(96, 21),
					},
				},
				{
					SignedHeader1: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "22",
							ProposerIndex: "23",
							ParentRoot:    FillEncodedByteSlice(32, 24),
							StateRoot:     FillEncodedByteSlice(32, 25),
							BodyRoot:      FillEncodedByteSlice(32, 26),
						},
						Signature: FillEncodedByteSlice(96, 27),
					},
					SignedHeader2: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "28",
							ProposerIndex: "29",
							ParentRoot:    FillEncodedByteSlice(32, 30),
							StateRoot:     FillEncodedByteSlice(32, 31),
							BodyRoot:      FillEncodedByteSlice(32, 32),
						},
						Signature: FillEncodedByteSlice(96, 33),
					},
				},
			},
			AttesterSlashings: []*shared.AttesterSlashing{
				{
					Attestation1: &shared.IndexedAttestation{
						AttestingIndices: []string{"34", "35"},
						Data: &shared.AttestationData{
							Slot:            "36",
							CommitteeIndex:  "37",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "39",
								Root:  FillEncodedByteSlice(32, 40),
							},
							Target: &shared.Checkpoint{
								Epoch: "41",
								Root:  FillEncodedByteSlice(32, 42),
							},
						},
						Signature: FillEncodedByteSlice(96, 43),
					},
					Attestation2: &shared.IndexedAttestation{
						AttestingIndices: []string{"44", "45"},
						Data: &shared.AttestationData{
							Slot:            "46",
							CommitteeIndex:  "47",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "49",
								Root:  FillEncodedByteSlice(32, 50),
							},
							Target: &shared.Checkpoint{
								Epoch: "51",
								Root:  FillEncodedByteSlice(32, 52),
							},
						},
						Signature: FillEncodedByteSlice(96, 53),
					},
				},
				{
					Attestation1: &shared.IndexedAttestation{
						AttestingIndices: []string{"54", "55"},
						Data: &shared.AttestationData{
							Slot:            "56",
							CommitteeIndex:  "57",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "59",
								Root:  FillEncodedByteSlice(32, 60),
							},
							Target: &shared.Checkpoint{
								Epoch: "61",
								Root:  FillEncodedByteSlice(32, 62),
							},
						},
						Signature: FillEncodedByteSlice(96, 63),
					},
					Attestation2: &shared.IndexedAttestation{
						AttestingIndices: []string{"64", "65"},
						Data: &shared.AttestationData{
							Slot:            "66",
							CommitteeIndex:  "67",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "69",
								Root:  FillEncodedByteSlice(32, 70),
							},
							Target: &shared.Checkpoint{
								Epoch: "71",
								Root:  FillEncodedByteSlice(32, 72),
							},
						},
						Signature: FillEncodedByteSlice(96, 73),
					},
				},
			},
			Attestations: []*shared.Attestation{
				{
					AggregationBits: FillEncodedByteSlice(4, 74),
					Data: &shared.AttestationData{
						Slot:            "75",
						CommitteeIndex:  "76",
						BeaconBlockRoot: FillEncodedByteSlice(32, 38),
						Source: &shared.Checkpoint{
							Epoch: "78",
							Root:  FillEncodedByteSlice(32, 79),
						},
						Target: &shared.Checkpoint{
							Epoch: "80",
							Root:  FillEncodedByteSlice(32, 81),
						},
					},
					Signature: FillEncodedByteSlice(96, 82),
				},
				{
					AggregationBits: FillEncodedByteSlice(4, 83),
					Data: &shared.AttestationData{
						Slot:            "84",
						CommitteeIndex:  "85",
						BeaconBlockRoot: FillEncodedByteSlice(32, 38),
						Source: &shared.Checkpoint{
							Epoch: "87",
							Root:  FillEncodedByteSlice(32, 88),
						},
						Target: &shared.Checkpoint{
							Epoch: "89",
							Root:  FillEncodedByteSlice(32, 90),
						},
					},
					Signature: FillEncodedByteSlice(96, 91),
				},
			},
			Deposits: []*shared.Deposit{
				{
					Proof: FillEncodedByteArraySlice(33, FillEncodedByteSlice(32, 92)),
					Data: &shared.DepositData{
						Pubkey:                FillEncodedByteSlice(48, 94),
						WithdrawalCredentials: FillEncodedByteSlice(32, 95),
						Amount:                "96",
						Signature:             FillEncodedByteSlice(96, 97),
					},
				},
				{
					Proof: FillEncodedByteArraySlice(33, FillEncodedByteSlice(32, 98)),
					Data: &shared.DepositData{
						Pubkey:                FillEncodedByteSlice(48, 100),
						WithdrawalCredentials: FillEncodedByteSlice(32, 101),
						Amount:                "102",
						Signature:             FillEncodedByteSlice(96, 103),
					},
				},
			},
			VoluntaryExits: []*shared.SignedVoluntaryExit{
				{
					Message: &shared.VoluntaryExit{
						Epoch:          "104",
						ValidatorIndex: "105",
					},
					Signature: FillEncodedByteSlice(96, 106),
				},
				{
					Message: &shared.VoluntaryExit{
						Epoch:          "107",
						ValidatorIndex: "108",
					},
					Signature: FillEncodedByteSlice(96, 109),
				},
			},
			SyncAggregate: &shared.SyncAggregate{
				SyncCommitteeBits:      FillEncodedByteSlice(64, 110),
				SyncCommitteeSignature: FillEncodedByteSlice(96, 111),
			},
			ExecutionPayload: &shared.ExecutionPayloadCapella{
				ParentHash:    FillEncodedByteSlice(32, 112),
				FeeRecipient:  FillEncodedByteSlice(20, 113),
				StateRoot:     FillEncodedByteSlice(32, 114),
				ReceiptsRoot:  FillEncodedByteSlice(32, 115),
				LogsBloom:     FillEncodedByteSlice(256, 116),
				PrevRandao:    FillEncodedByteSlice(32, 117),
				BlockNumber:   "118",
				GasLimit:      "119",
				GasUsed:       "120",
				Timestamp:     "121",
				ExtraData:     FillEncodedByteSlice(32, 122),
				BaseFeePerGas: bytesutil.LittleEndianBytesToBigInt(FillByteSlice(32, 123)).String(),
				BlockHash:     FillEncodedByteSlice(32, 124),
				Transactions: []string{
					FillEncodedByteSlice(32, 125),
					FillEncodedByteSlice(32, 126),
				},
				Withdrawals: []*shared.Withdrawal{
					{
						WithdrawalIndex:  "127",
						ValidatorIndex:   "128",
						ExecutionAddress: FillEncodedByteSlice(20, 129),
						Amount:           "130",
					},
					{
						WithdrawalIndex:  "131",
						ValidatorIndex:   "132",
						ExecutionAddress: FillEncodedByteSlice(20, 133),
						Amount:           "134",
					},
				},
			},
			BLSToExecutionChanges: []*shared.SignedBLSToExecutionChange{
				{
					Message: &shared.BLSToExecutionChange{
						ValidatorIndex:     "135",
						FromBLSPubkey:      FillEncodedByteSlice(48, 136),
						ToExecutionAddress: FillEncodedByteSlice(20, 137),
					},
					Signature: FillEncodedByteSlice(96, 138),
				},
				{
					Message: &shared.BLSToExecutionChange{
						ValidatorIndex:     "139",
						FromBLSPubkey:      FillEncodedByteSlice(48, 140),
						ToExecutionAddress: FillEncodedByteSlice(20, 141),
					},
					Signature: FillEncodedByteSlice(96, 142),
				},
			},
		},
	}
}

func GenerateJsonBlindedCapellaBeaconBlock() *shared.BlindedBeaconBlockCapella {
	return &shared.BlindedBeaconBlockCapella{
		Slot:          "1",
		ProposerIndex: "2",
		ParentRoot:    FillEncodedByteSlice(32, 3),
		StateRoot:     FillEncodedByteSlice(32, 4),
		Body: &shared.BlindedBeaconBlockBodyCapella{
			RandaoReveal: FillEncodedByteSlice(96, 5),
			Eth1Data: &shared.Eth1Data{
				DepositRoot:  FillEncodedByteSlice(32, 6),
				DepositCount: "7",
				BlockHash:    FillEncodedByteSlice(32, 8),
			},
			Graffiti: FillEncodedByteSlice(32, 9),
			ProposerSlashings: []*shared.ProposerSlashing{
				{
					SignedHeader1: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "10",
							ProposerIndex: "11",
							ParentRoot:    FillEncodedByteSlice(32, 12),
							StateRoot:     FillEncodedByteSlice(32, 13),
							BodyRoot:      FillEncodedByteSlice(32, 14),
						},
						Signature: FillEncodedByteSlice(96, 15),
					},
					SignedHeader2: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "16",
							ProposerIndex: "17",
							ParentRoot:    FillEncodedByteSlice(32, 18),
							StateRoot:     FillEncodedByteSlice(32, 19),
							BodyRoot:      FillEncodedByteSlice(32, 20),
						},
						Signature: FillEncodedByteSlice(96, 21),
					},
				},
				{
					SignedHeader1: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "22",
							ProposerIndex: "23",
							ParentRoot:    FillEncodedByteSlice(32, 24),
							StateRoot:     FillEncodedByteSlice(32, 25),
							BodyRoot:      FillEncodedByteSlice(32, 26),
						},
						Signature: FillEncodedByteSlice(96, 27),
					},
					SignedHeader2: &shared.SignedBeaconBlockHeader{
						Message: &shared.BeaconBlockHeader{
							Slot:          "28",
							ProposerIndex: "29",
							ParentRoot:    FillEncodedByteSlice(32, 30),
							StateRoot:     FillEncodedByteSlice(32, 31),
							BodyRoot:      FillEncodedByteSlice(32, 32),
						},
						Signature: FillEncodedByteSlice(96, 33),
					},
				},
			},
			AttesterSlashings: []*shared.AttesterSlashing{
				{
					Attestation1: &shared.IndexedAttestation{
						AttestingIndices: []string{"34", "35"},
						Data: &shared.AttestationData{
							Slot:            "36",
							CommitteeIndex:  "37",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "39",
								Root:  FillEncodedByteSlice(32, 40),
							},
							Target: &shared.Checkpoint{
								Epoch: "41",
								Root:  FillEncodedByteSlice(32, 42),
							},
						},
						Signature: FillEncodedByteSlice(96, 43),
					},
					Attestation2: &shared.IndexedAttestation{
						AttestingIndices: []string{"44", "45"},
						Data: &shared.AttestationData{
							Slot:            "46",
							CommitteeIndex:  "47",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "49",
								Root:  FillEncodedByteSlice(32, 50),
							},
							Target: &shared.Checkpoint{
								Epoch: "51",
								Root:  FillEncodedByteSlice(32, 52),
							},
						},
						Signature: FillEncodedByteSlice(96, 53),
					},
				},
				{
					Attestation1: &shared.IndexedAttestation{
						AttestingIndices: []string{"54", "55"},
						Data: &shared.AttestationData{
							Slot:            "56",
							CommitteeIndex:  "57",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "59",
								Root:  FillEncodedByteSlice(32, 60),
							},
							Target: &shared.Checkpoint{
								Epoch: "61",
								Root:  FillEncodedByteSlice(32, 62),
							},
						},
						Signature: FillEncodedByteSlice(96, 63),
					},
					Attestation2: &shared.IndexedAttestation{
						AttestingIndices: []string{"64", "65"},
						Data: &shared.AttestationData{
							Slot:            "66",
							CommitteeIndex:  "67",
							BeaconBlockRoot: FillEncodedByteSlice(32, 38),
							Source: &shared.Checkpoint{
								Epoch: "69",
								Root:  FillEncodedByteSlice(32, 70),
							},
							Target: &shared.Checkpoint{
								Epoch: "71",
								Root:  FillEncodedByteSlice(32, 72),
							},
						},
						Signature: FillEncodedByteSlice(96, 73),
					},
				},
			},
			Attestations: []*shared.Attestation{
				{
					AggregationBits: FillEncodedByteSlice(4, 74),
					Data: &shared.AttestationData{
						Slot:            "75",
						CommitteeIndex:  "76",
						BeaconBlockRoot: FillEncodedByteSlice(32, 38),
						Source: &shared.Checkpoint{
							Epoch: "78",
							Root:  FillEncodedByteSlice(32, 79),
						},
						Target: &shared.Checkpoint{
							Epoch: "80",
							Root:  FillEncodedByteSlice(32, 81),
						},
					},
					Signature: FillEncodedByteSlice(96, 82),
				},
				{
					AggregationBits: FillEncodedByteSlice(4, 83),
					Data: &shared.AttestationData{
						Slot:            "84",
						CommitteeIndex:  "85",
						BeaconBlockRoot: FillEncodedByteSlice(32, 38),
						Source: &shared.Checkpoint{
							Epoch: "87",
							Root:  FillEncodedByteSlice(32, 88),
						},
						Target: &shared.Checkpoint{
							Epoch: "89",
							Root:  FillEncodedByteSlice(32, 90),
						},
					},
					Signature: FillEncodedByteSlice(96, 91),
				},
			},
			Deposits: []*shared.Deposit{
				{
					Proof: FillEncodedByteArraySlice(33, FillEncodedByteSlice(32, 92)),
					Data: &shared.DepositData{
						Pubkey:                FillEncodedByteSlice(48, 94),
						WithdrawalCredentials: FillEncodedByteSlice(32, 95),
						Amount:                "96",
						Signature:             FillEncodedByteSlice(96, 97),
					},
				},
				{
					Proof: FillEncodedByteArraySlice(33, FillEncodedByteSlice(32, 98)),
					Data: &shared.DepositData{
						Pubkey:                FillEncodedByteSlice(48, 100),
						WithdrawalCredentials: FillEncodedByteSlice(32, 101),
						Amount:                "102",
						Signature:             FillEncodedByteSlice(96, 103),
					},
				},
			},
			VoluntaryExits: []*shared.SignedVoluntaryExit{
				{
					Message: &shared.VoluntaryExit{
						Epoch:          "104",
						ValidatorIndex: "105",
					},
					Signature: FillEncodedByteSlice(96, 106),
				},
				{
					Message: &shared.VoluntaryExit{
						Epoch:          "107",
						ValidatorIndex: "108",
					},
					Signature: FillEncodedByteSlice(96, 109),
				},
			},
			SyncAggregate: &shared.SyncAggregate{
				SyncCommitteeBits:      FillEncodedByteSlice(64, 110),
				SyncCommitteeSignature: FillEncodedByteSlice(96, 111),
			},
			ExecutionPayloadHeader: &shared.ExecutionPayloadHeaderCapella{
				ParentHash:       FillEncodedByteSlice(32, 112),
				FeeRecipient:     FillEncodedByteSlice(20, 113),
				StateRoot:        FillEncodedByteSlice(32, 114),
				ReceiptsRoot:     FillEncodedByteSlice(32, 115),
				LogsBloom:        FillEncodedByteSlice(256, 116),
				PrevRandao:       FillEncodedByteSlice(32, 117),
				BlockNumber:      "118",
				GasLimit:         "119",
				GasUsed:          "120",
				Timestamp:        "121",
				ExtraData:        FillEncodedByteSlice(32, 122),
				BaseFeePerGas:    bytesutil.LittleEndianBytesToBigInt(FillByteSlice(32, 123)).String(),
				BlockHash:        FillEncodedByteSlice(32, 124),
				TransactionsRoot: FillEncodedByteSlice(32, 125),
				WithdrawalsRoot:  FillEncodedByteSlice(32, 126),
			},
			BLSToExecutionChanges: []*shared.SignedBLSToExecutionChange{
				{
					Message: &shared.BLSToExecutionChange{
						ValidatorIndex:     "135",
						FromBLSPubkey:      FillEncodedByteSlice(48, 136),
						ToExecutionAddress: FillEncodedByteSlice(20, 137),
					},
					Signature: FillEncodedByteSlice(96, 138),
				},
				{
					Message: &shared.BLSToExecutionChange{
						ValidatorIndex:     "139",
						FromBLSPubkey:      FillEncodedByteSlice(48, 140),
						ToExecutionAddress: FillEncodedByteSlice(20, 141),
					},
					Signature: FillEncodedByteSlice(96, 142),
				},
			},
		},
	}
}
