package beacon

import (
	"context"
	"fmt"
	"sort"
	"strconv"
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/prysmaticlabs/go-bitfield"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	attaggregation "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/attestations"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/mock"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/emptypb"
)

func TestServer_ListAttestations_NoResults(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	st, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot: 0,
	})
	require.NoError(t, err)
	bs := &Server{
		BeaconDB: db,
		HeadFetcher: &chainMock.ChainService{
			State: st,
		},
	}
	wanted := &ethpb.ListAttestationsResponse{
		Attestations:  make([]*ethpb.Attestation, 0),
		TotalSize:     int32(0),
		NextPageToken: strconv.Itoa(0),
	}
	res, err := bs.ListAttestations(ctx, &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{GenesisEpoch: true},
	})
	require.NoError(t, err)
	if !proto.Equal(wanted, res) {
		t.Errorf("Wanted %v, received %v", wanted, res)
	}
}

func TestServer_ListAttestations_Genesis(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	st, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot: 0,
	})
	require.NoError(t, err)
	bs := &Server{
		BeaconDB: db,
		HeadFetcher: &chainMock.ChainService{
			State: st,
		},
	}

	att := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(0),
		Data: &ethpb.AttestationData{
			Slot:           2,
			CommitteeIndex: 1,
		},
	})

	parentRoot := [32]byte{1, 2, 3}
	signedBlock := util.NewBeaconBlock()
	signedBlock.Block.ParentRoot = bytesutil.PadTo(parentRoot[:], 32)
	signedBlock.Block.Body.Attestations = []*ethpb.Attestation{att}
	root, err := signedBlock.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, db, signedBlock)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, root))
	wanted := &ethpb.ListAttestationsResponse{
		Attestations:  []*ethpb.Attestation{att},
		NextPageToken: "",
		TotalSize:     1,
	}

	res, err := bs.ListAttestations(ctx, &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
			GenesisEpoch: true,
		},
	})
	require.NoError(t, err)
	require.DeepSSZEqual(t, wanted, res)
}

func TestServer_ListAttestations_NoPagination(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	count := primitives.Slot(8)
	atts := make([]*ethpb.Attestation, 0, count)
	for i := primitives.Slot(0); i < count; i++ {
		blockExample := util.NewBeaconBlock()
		blockExample.Block.Body.Attestations = []*ethpb.Attestation{
			{
				Signature: make([]byte, fieldparams.BLSSignatureLength),
				Data: &ethpb.AttestationData{
					Target:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte("root"), 32)},
					Source:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte("root"), 32)},
					BeaconBlockRoot: bytesutil.PadTo([]byte("root"), 32),
					Slot:            i,
				},
				AggregationBits: bitfield.Bitlist{0b11},
			},
		}
		util.SaveBlock(t, ctx, db, blockExample)
		atts = append(atts, blockExample.Block.Body.Attestations...)
	}

	bs := &Server{
		BeaconDB: db,
	}

	received, err := bs.ListAttestations(ctx, &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
			GenesisEpoch: true,
		},
	})
	require.NoError(t, err)
	require.DeepEqual(t, atts, received.Attestations, "Incorrect attestations response")
}

func TestServer_ListAttestations_FiltersCorrectly(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	someRoot := [32]byte{1, 2, 3}
	sourceRoot := [32]byte{4, 5, 6}
	sourceEpoch := primitives.Epoch(5)
	targetRoot := [32]byte{7, 8, 9}
	targetEpoch := primitives.Epoch(7)

	unwrappedBlocks := []*ethpb.SignedBeaconBlock{
		util.HydrateSignedBeaconBlock(
			&ethpb.SignedBeaconBlock{
				Block: &ethpb.BeaconBlock{
					Slot: 4,
					Body: &ethpb.BeaconBlockBody{
						Attestations: []*ethpb.Attestation{
							{
								Data: &ethpb.AttestationData{
									BeaconBlockRoot: someRoot[:],
									Source: &ethpb.Checkpoint{
										Root:  sourceRoot[:],
										Epoch: sourceEpoch,
									},
									Target: &ethpb.Checkpoint{
										Root:  targetRoot[:],
										Epoch: targetEpoch,
									},
									Slot: 3,
								},
								AggregationBits: bitfield.Bitlist{0b11},
								Signature:       bytesutil.PadTo([]byte("sig"), fieldparams.BLSSignatureLength),
							},
						},
					},
				},
			}),
		util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{
			Block: &ethpb.BeaconBlock{
				Slot: 5 + params.BeaconConfig().SlotsPerEpoch,
				Body: &ethpb.BeaconBlockBody{
					Attestations: []*ethpb.Attestation{
						{
							Data: &ethpb.AttestationData{
								BeaconBlockRoot: someRoot[:],
								Source: &ethpb.Checkpoint{
									Root:  sourceRoot[:],
									Epoch: sourceEpoch,
								},
								Target: &ethpb.Checkpoint{
									Root:  targetRoot[:],
									Epoch: targetEpoch,
								},
								Slot: 4 + params.BeaconConfig().SlotsPerEpoch,
							},
							AggregationBits: bitfield.Bitlist{0b11},
							Signature:       bytesutil.PadTo([]byte("sig"), fieldparams.BLSSignatureLength),
						},
					},
				},
			},
		}),
		util.HydrateSignedBeaconBlock(
			&ethpb.SignedBeaconBlock{
				Block: &ethpb.BeaconBlock{
					Slot: 5,
					Body: &ethpb.BeaconBlockBody{
						Attestations: []*ethpb.Attestation{
							{
								Data: &ethpb.AttestationData{
									BeaconBlockRoot: someRoot[:],
									Source: &ethpb.Checkpoint{
										Root:  sourceRoot[:],
										Epoch: sourceEpoch,
									},
									Target: &ethpb.Checkpoint{
										Root:  targetRoot[:],
										Epoch: targetEpoch,
									},
									Slot: 4,
								},
								AggregationBits: bitfield.Bitlist{0b11},
								Signature:       bytesutil.PadTo([]byte("sig"), fieldparams.BLSSignatureLength),
							},
						},
					},
				},
			}),
	}

	var blocks []interfaces.ReadOnlySignedBeaconBlock
	for _, b := range unwrappedBlocks {
		wsb, err := consensusblocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		blocks = append(blocks, wsb)
	}

	require.NoError(t, db.SaveBlocks(ctx, blocks))

	bs := &Server{
		BeaconDB: db,
	}

	received, err := bs.ListAttestations(ctx, &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_Epoch{Epoch: 1},
	})
	require.NoError(t, err)
	assert.Equal(t, 1, len(received.Attestations))
	received, err = bs.ListAttestations(ctx, &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{GenesisEpoch: true},
	})
	require.NoError(t, err)
	assert.Equal(t, 2, len(received.Attestations))
}

func TestServer_ListAttestations_Pagination_CustomPageParameters(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	count := params.BeaconConfig().SlotsPerEpoch * 4
	atts := make([]*ethpb.Attestation, 0, count)
	for i := primitives.Slot(0); i < params.BeaconConfig().SlotsPerEpoch; i++ {
		for s := primitives.CommitteeIndex(0); s < 4; s++ {
			blockExample := util.NewBeaconBlock()
			blockExample.Block.Slot = i
			blockExample.Block.Body.Attestations = []*ethpb.Attestation{
				util.HydrateAttestation(&ethpb.Attestation{
					Data: &ethpb.AttestationData{
						CommitteeIndex: s,
						Slot:           i,
					},
					AggregationBits: bitfield.Bitlist{0b11},
				}),
			}
			util.SaveBlock(t, ctx, db, blockExample)
			atts = append(atts, blockExample.Block.Body.Attestations...)
		}
	}
	sort.Sort(sortableAttestations(atts))

	bs := &Server{
		BeaconDB: db,
	}

	tests := []struct {
		name string
		req  *ethpb.ListAttestationsRequest
		res  *ethpb.ListAttestationsResponse
	}{
		{
			name: "1st of 3 pages",
			req: &ethpb.ListAttestationsRequest{
				QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
					GenesisEpoch: true,
				},
				PageToken: strconv.Itoa(1),
				PageSize:  3,
			},
			res: &ethpb.ListAttestationsResponse{
				Attestations: []*ethpb.Attestation{
					atts[3],
					atts[4],
					atts[5],
				},
				NextPageToken: strconv.Itoa(2),
				TotalSize:     int32(count),
			},
		},
		{
			name: "10 of size 1",
			req: &ethpb.ListAttestationsRequest{
				QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
					GenesisEpoch: true,
				},
				PageToken: strconv.Itoa(10),
				PageSize:  1,
			},
			res: &ethpb.ListAttestationsResponse{
				Attestations: []*ethpb.Attestation{
					atts[10],
				},
				NextPageToken: strconv.Itoa(11),
				TotalSize:     int32(count),
			},
		},
		{
			name: "2 of size 8",
			req: &ethpb.ListAttestationsRequest{
				QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
					GenesisEpoch: true,
				},
				PageToken: strconv.Itoa(2),
				PageSize:  8,
			},
			res: &ethpb.ListAttestationsResponse{
				Attestations: []*ethpb.Attestation{
					atts[16],
					atts[17],
					atts[18],
					atts[19],
					atts[20],
					atts[21],
					atts[22],
					atts[23],
				},
				NextPageToken: strconv.Itoa(3),
				TotalSize:     int32(count)},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			res, err := bs.ListAttestations(ctx, test.req)
			require.NoError(t, err)
			require.DeepSSZEqual(t, res, test.res)
		})
	}
}

func TestServer_ListAttestations_Pagination_OutOfRange(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()
	util.NewBeaconBlock()
	count := primitives.Slot(1)
	atts := make([]*ethpb.Attestation, 0, count)
	for i := primitives.Slot(0); i < count; i++ {
		blockExample := util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{
			Block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					Attestations: []*ethpb.Attestation{
						{
							Data: &ethpb.AttestationData{
								BeaconBlockRoot: bytesutil.PadTo([]byte("root"), fieldparams.RootLength),
								Source:          &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
								Target:          &ethpb.Checkpoint{Root: make([]byte, fieldparams.RootLength)},
								Slot:            i,
							},
							AggregationBits: bitfield.Bitlist{0b11},
							Signature:       make([]byte, fieldparams.BLSSignatureLength),
						},
					},
				},
			},
		})
		util.SaveBlock(t, ctx, db, blockExample)
		atts = append(atts, blockExample.Block.Body.Attestations...)
	}

	bs := &Server{
		BeaconDB: db,
	}

	req := &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_Epoch{
			Epoch: 0,
		},
		PageToken: strconv.Itoa(1),
		PageSize:  100,
	}
	wanted := fmt.Sprintf("page start %d >= list %d", req.PageSize, len(atts))
	_, err := bs.ListAttestations(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestServer_ListAttestations_Pagination_ExceedsMaxPageSize(t *testing.T) {
	ctx := context.Background()
	bs := &Server{}
	exceedsMax := int32(cmd.Get().MaxRPCPageSize + 1)

	wanted := fmt.Sprintf("Requested page size %d can not be greater than max size %d", exceedsMax, cmd.Get().MaxRPCPageSize)
	req := &ethpb.ListAttestationsRequest{PageToken: strconv.Itoa(0), PageSize: exceedsMax}
	_, err := bs.ListAttestations(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestServer_ListAttestations_Pagination_DefaultPageSize(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()

	count := primitives.Slot(params.BeaconConfig().DefaultPageSize)
	atts := make([]*ethpb.Attestation, 0, count)
	for i := primitives.Slot(0); i < count; i++ {
		blockExample := util.NewBeaconBlock()
		blockExample.Block.Body.Attestations = []*ethpb.Attestation{
			{
				Data: &ethpb.AttestationData{
					BeaconBlockRoot: bytesutil.PadTo([]byte("root"), 32),
					Target:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte("root"), 32)},
					Source:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte("root"), 32)},
					Slot:            i,
				},
				Signature:       bytesutil.PadTo([]byte("root"), fieldparams.BLSSignatureLength),
				AggregationBits: bitfield.Bitlist{0b11},
			},
		}
		util.SaveBlock(t, ctx, db, blockExample)
		atts = append(atts, blockExample.Block.Body.Attestations...)
	}

	bs := &Server{
		BeaconDB: db,
	}

	req := &ethpb.ListAttestationsRequest{
		QueryFilter: &ethpb.ListAttestationsRequest_GenesisEpoch{
			GenesisEpoch: true,
		},
	}
	res, err := bs.ListAttestations(ctx, req)
	require.NoError(t, err)

	i := 0
	j := params.BeaconConfig().DefaultPageSize
	assert.DeepEqual(t, atts[i:j], res.Attestations, "Incorrect attestations response")
}

func TestServer_mapAttestationToTargetRoot(t *testing.T) {
	count := primitives.Slot(100)
	atts := make([]*ethpb.Attestation, count)
	targetRoot1 := bytesutil.ToBytes32([]byte("root1"))
	targetRoot2 := bytesutil.ToBytes32([]byte("root2"))

	for i := primitives.Slot(0); i < count; i++ {
		var targetRoot [32]byte
		if i%2 == 0 {
			targetRoot = targetRoot1
		} else {
			targetRoot = targetRoot2
		}
		atts[i] = &ethpb.Attestation{
			Data: &ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Root: targetRoot[:],
				},
			},
			AggregationBits: bitfield.Bitlist{0b11},
		}

	}
	mappedAtts := mapAttestationsByTargetRoot(atts)
	wantedMapLen := 2
	wantedMapNumberOfElements := 50
	assert.Equal(t, wantedMapLen, len(mappedAtts), "Unexpected mapped attestations length")
	assert.Equal(t, wantedMapNumberOfElements, len(mappedAtts[targetRoot1]), "Unexpected number of attestations per block root")
	assert.Equal(t, wantedMapNumberOfElements, len(mappedAtts[targetRoot2]), "Unexpected number of attestations per block root")
}

func TestServer_ListIndexedAttestations_GenesisEpoch(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.BeaconConfig())
	db := dbTest.SetupDB(t)
	helpers.ClearCache()
	ctx := context.Background()
	targetRoot1 := bytesutil.ToBytes32([]byte("root"))
	targetRoot2 := bytesutil.ToBytes32([]byte("root2"))

	count := params.BeaconConfig().SlotsPerEpoch
	atts := make([]*ethpb.Attestation, 0, count)
	atts2 := make([]*ethpb.Attestation, 0, count)

	for i := primitives.Slot(0); i < count; i++ {
		var targetRoot [32]byte
		if i%2 == 0 {
			targetRoot = targetRoot1
		} else {
			targetRoot = targetRoot2
		}
		blockExample := util.NewBeaconBlock()
		blockExample.Block.Body.Attestations = []*ethpb.Attestation{
			{
				Signature: make([]byte, fieldparams.BLSSignatureLength),
				Data: &ethpb.AttestationData{
					BeaconBlockRoot: make([]byte, fieldparams.RootLength),
					Target: &ethpb.Checkpoint{
						Root: targetRoot[:],
					},
					Source: &ethpb.Checkpoint{
						Root: make([]byte, fieldparams.RootLength),
					},
					Slot:           i,
					CommitteeIndex: 0,
				},
				AggregationBits: bitfield.NewBitlist(128 / uint64(params.BeaconConfig().SlotsPerEpoch)),
			},
		}
		util.SaveBlock(t, ctx, db, blockExample)
		if i%2 == 0 {
			atts = append(atts, blockExample.Block.Body.Attestations...)
		} else {
			atts2 = append(atts2, blockExample.Block.Body.Attestations...)
		}

	}

	// We setup 512 validators so that committee size matches the length of attestations' aggregation bits.
	numValidators := uint64(512)
	state, _ := util.DeterministicGenesisState(t, numValidators)

	// Next up we convert the test attestations to indexed form:
	indexedAtts := make([]*ethpb.IndexedAttestation, len(atts)+len(atts2))
	for i := 0; i < len(atts); i++ {
		att := atts[i]
		committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
		require.NoError(t, err)
		idxAtt, err := attestation.ConvertToIndexed(ctx, atts[i], committee)
		require.NoError(t, err, "Could not convert attestation to indexed")
		indexedAtts[i] = idxAtt
	}
	for i := 0; i < len(atts2); i++ {
		att := atts2[i]
		committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
		require.NoError(t, err)
		idxAtt, err := attestation.ConvertToIndexed(ctx, atts2[i], committee)
		require.NoError(t, err, "Could not convert attestation to indexed")
		indexedAtts[i+len(atts)] = idxAtt
	}

	bs := &Server{
		BeaconDB:           db,
		GenesisTimeFetcher: &chainMock.ChainService{State: state},
		HeadFetcher:        &chainMock.ChainService{State: state},
		StateGen:           stategen.New(db, doublylinkedtree.New()),
	}
	err := db.SaveStateSummary(ctx, &ethpb.StateSummary{
		Root: targetRoot1[:],
		Slot: 1,
	})
	require.NoError(t, err)

	err = db.SaveStateSummary(ctx, &ethpb.StateSummary{
		Root: targetRoot2[:],
		Slot: 2,
	})
	require.NoError(t, err)

	require.NoError(t, db.SaveState(ctx, state, bytesutil.ToBytes32(targetRoot1[:])))
	require.NoError(t, state.SetSlot(state.Slot()+1))
	require.NoError(t, db.SaveState(ctx, state, bytesutil.ToBytes32(targetRoot2[:])))
	res, err := bs.ListIndexedAttestations(ctx, &ethpb.ListIndexedAttestationsRequest{
		QueryFilter: &ethpb.ListIndexedAttestationsRequest_GenesisEpoch{
			GenesisEpoch: true,
		},
	})
	require.NoError(t, err)
	assert.Equal(t, len(indexedAtts), len(res.IndexedAttestations), "Incorrect indexted attestations length")
	sort.Slice(indexedAtts, func(i, j int) bool {
		return indexedAtts[i].Data.Slot < indexedAtts[j].Data.Slot
	})
	sort.Slice(res.IndexedAttestations, func(i, j int) bool {
		return res.IndexedAttestations[i].Data.Slot < res.IndexedAttestations[j].Data.Slot
	})

	assert.DeepEqual(t, indexedAtts, res.IndexedAttestations, "Incorrect list indexed attestations response")
}

func TestServer_ListIndexedAttestations_OldEpoch(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.BeaconConfig())
	db := dbTest.SetupDB(t)
	helpers.ClearCache()
	ctx := context.Background()

	blockRoot := bytesutil.ToBytes32([]byte("root"))
	count := params.BeaconConfig().SlotsPerEpoch
	atts := make([]*ethpb.Attestation, 0, count)
	epoch := primitives.Epoch(50)
	startSlot, err := slots.EpochStart(epoch)
	require.NoError(t, err)

	for i := startSlot; i < count; i++ {
		blockExample := &ethpb.SignedBeaconBlock{
			Block: &ethpb.BeaconBlock{
				Body: &ethpb.BeaconBlockBody{
					Attestations: []*ethpb.Attestation{
						{
							Data: &ethpb.AttestationData{
								BeaconBlockRoot: blockRoot[:],
								Slot:            i,
								CommitteeIndex:  0,
								Target: &ethpb.Checkpoint{
									Epoch: epoch,
									Root:  make([]byte, fieldparams.RootLength),
								},
							},
							AggregationBits: bitfield.Bitlist{0b11},
						},
					},
				},
			},
		}
		util.SaveBlock(t, ctx, db, blockExample)
		atts = append(atts, blockExample.Block.Body.Attestations...)
	}

	// We setup 128 validators.
	numValidators := uint64(128)
	state, _ := util.DeterministicGenesisState(t, numValidators)

	randaoMixes := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(randaoMixes); i++ {
		randaoMixes[i] = make([]byte, fieldparams.RootLength)
	}
	require.NoError(t, state.SetRandaoMixes(randaoMixes))
	require.NoError(t, state.SetSlot(startSlot))

	// Next up we convert the test attestations to indexed form:
	indexedAtts := make([]*ethpb.IndexedAttestation, len(atts))
	for i := 0; i < len(atts); i++ {
		att := atts[i]
		committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
		require.NoError(t, err)
		idxAtt, err := attestation.ConvertToIndexed(ctx, atts[i], committee)
		require.NoError(t, err, "Could not convert attestation to indexed")
		indexedAtts[i] = idxAtt
	}

	bs := &Server{
		BeaconDB: db,
		GenesisTimeFetcher: &chainMock.ChainService{
			Genesis: time.Now(),
		},
		StateGen: stategen.New(db, doublylinkedtree.New()),
	}
	err = db.SaveStateSummary(ctx, &ethpb.StateSummary{
		Root: blockRoot[:],
		Slot: params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch)),
	})
	require.NoError(t, err)
	require.NoError(t, db.SaveState(ctx, state, bytesutil.ToBytes32([]byte("root"))))
	res, err := bs.ListIndexedAttestations(ctx, &ethpb.ListIndexedAttestationsRequest{
		QueryFilter: &ethpb.ListIndexedAttestationsRequest_Epoch{
			Epoch: epoch,
		},
	})
	require.NoError(t, err)
	require.DeepEqual(t, indexedAtts, res.IndexedAttestations, "Incorrect list indexed attestations response")
}

func TestServer_AttestationPool_Pagination_ExceedsMaxPageSize(t *testing.T) {
	ctx := context.Background()
	bs := &Server{}
	exceedsMax := int32(cmd.Get().MaxRPCPageSize + 1)

	wanted := fmt.Sprintf("Requested page size %d can not be greater than max size %d", exceedsMax, cmd.Get().MaxRPCPageSize)
	req := &ethpb.AttestationPoolRequest{PageToken: strconv.Itoa(0), PageSize: exceedsMax}
	_, err := bs.AttestationPool(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestServer_AttestationPool_Pagination_OutOfRange(t *testing.T) {
	ctx := context.Background()
	bs := &Server{
		AttestationsPool: attestations.NewPool(),
	}

	atts := []*ethpb.Attestation{
		{
			Data: &ethpb.AttestationData{
				Slot:            1,
				BeaconBlockRoot: bytesutil.PadTo([]byte{1}, 32),
				Source:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{1}, 32)},
				Target:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{1}, 32)},
			},
			AggregationBits: bitfield.Bitlist{0b1101},
			Signature:       bytesutil.PadTo([]byte{1}, fieldparams.BLSSignatureLength),
		},
		{
			Data: &ethpb.AttestationData{
				Slot:            2,
				BeaconBlockRoot: bytesutil.PadTo([]byte{2}, 32),
				Source:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{2}, 32)},
				Target:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{2}, 32)},
			},
			AggregationBits: bitfield.Bitlist{0b1101},
			Signature:       bytesutil.PadTo([]byte{2}, fieldparams.BLSSignatureLength),
		},
		{
			Data: &ethpb.AttestationData{
				Slot:            3,
				BeaconBlockRoot: bytesutil.PadTo([]byte{3}, 32),
				Source:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{3}, 32)},
				Target:          &ethpb.Checkpoint{Root: bytesutil.PadTo([]byte{3}, 32)},
			},
			AggregationBits: bitfield.Bitlist{0b1101},
			Signature:       bytesutil.PadTo([]byte{3}, fieldparams.BLSSignatureLength),
		},
	}
	require.NoError(t, bs.AttestationsPool.SaveAggregatedAttestations(atts))

	req := &ethpb.AttestationPoolRequest{
		PageToken: strconv.Itoa(1),
		PageSize:  100,
	}
	wanted := fmt.Sprintf("page start %d >= list %d", req.PageSize, len(atts))
	_, err := bs.AttestationPool(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestServer_AttestationPool_Pagination_DefaultPageSize(t *testing.T) {
	ctx := context.Background()
	bs := &Server{
		AttestationsPool: attestations.NewPool(),
	}

	atts := make([]*ethpb.Attestation, params.BeaconConfig().DefaultPageSize+1)
	for i := 0; i < len(atts); i++ {
		att := util.NewAttestation()
		att.Data.Slot = primitives.Slot(i)
		atts[i] = att
	}
	require.NoError(t, bs.AttestationsPool.SaveAggregatedAttestations(atts))

	req := &ethpb.AttestationPoolRequest{}
	res, err := bs.AttestationPool(ctx, req)
	require.NoError(t, err)
	assert.Equal(t, params.BeaconConfig().DefaultPageSize, len(res.Attestations), "Unexpected number of attestations")
	assert.Equal(t, params.BeaconConfig().DefaultPageSize+1, int(res.TotalSize), "Unexpected total size")
}

func TestServer_AttestationPool_Pagination_CustomPageSize(t *testing.T) {
	ctx := context.Background()
	bs := &Server{
		AttestationsPool: attestations.NewPool(),
	}

	numAtts := 100
	atts := make([]*ethpb.Attestation, numAtts)
	for i := 0; i < len(atts); i++ {
		att := util.NewAttestation()
		att.Data.Slot = primitives.Slot(i)
		atts[i] = att
	}
	require.NoError(t, bs.AttestationsPool.SaveAggregatedAttestations(atts))
	tests := []struct {
		req *ethpb.AttestationPoolRequest
		res *ethpb.AttestationPoolResponse
	}{
		{
			req: &ethpb.AttestationPoolRequest{
				PageToken: strconv.Itoa(1),
				PageSize:  3,
			},
			res: &ethpb.AttestationPoolResponse{
				NextPageToken: "2",
				TotalSize:     int32(numAtts),
			},
		},
		{
			req: &ethpb.AttestationPoolRequest{
				PageToken: strconv.Itoa(3),
				PageSize:  30,
			},
			res: &ethpb.AttestationPoolResponse{
				NextPageToken: "",
				TotalSize:     int32(numAtts),
			},
		},
		{
			req: &ethpb.AttestationPoolRequest{
				PageToken: strconv.Itoa(0),
				PageSize:  int32(numAtts),
			},
			res: &ethpb.AttestationPoolResponse{
				NextPageToken: "",
				TotalSize:     int32(numAtts),
			},
		},
	}
	for _, tt := range tests {
		res, err := bs.AttestationPool(ctx, tt.req)
		require.NoError(t, err)
		assert.Equal(t, tt.res.TotalSize, res.TotalSize, "Unexpected total size")
		assert.Equal(t, tt.res.NextPageToken, res.NextPageToken, "Unexpected next page token")
	}
}

func TestServer_StreamIndexedAttestations_ContextCanceled(t *testing.T) {
	ctx := context.Background()
	ctx, cancel := context.WithCancel(ctx)
	chainService := &chainMock.ChainService{}
	server := &Server{
		Ctx:                 ctx,
		AttestationNotifier: chainService.OperationNotifier(),
		GenesisTimeFetcher: &chainMock.ChainService{
			Genesis: time.Now(),
		},
	}

	exitRoutine := make(chan bool)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mockStream := mock.NewMockBeaconChain_StreamIndexedAttestationsServer(ctrl)
	mockStream.EXPECT().Context().Return(ctx).AnyTimes()
	go func(tt *testing.T) {
		err := server.StreamIndexedAttestations(&emptypb.Empty{}, mockStream)
		assert.ErrorContains(t, "Context canceled", err)
		<-exitRoutine
	}(t)
	cancel()
	exitRoutine <- true
}

func TestServer_StreamIndexedAttestations_OK(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.BeaconConfig())
	db := dbTest.SetupDB(t)
	exitRoutine := make(chan bool)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	ctx := context.Background()

	numValidators := 64
	headState, privKeys := util.DeterministicGenesisState(t, uint64(numValidators))
	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, db, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, gRoot))
	require.NoError(t, db.SaveState(ctx, headState, gRoot))

	activeIndices, err := helpers.ActiveValidatorIndices(ctx, headState, 0)
	require.NoError(t, err)
	epoch := primitives.Epoch(0)
	attesterSeed, err := helpers.Seed(headState, epoch, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	committees, err := computeCommittees(context.Background(), params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epoch)), activeIndices, attesterSeed)
	require.NoError(t, err)

	count := params.BeaconConfig().SlotsPerEpoch
	// We generate attestations for each validator per slot per epoch.
	atts := make(map[[32]byte][]*ethpb.Attestation)
	for i := primitives.Slot(0); i < count; i++ {
		comms := committees[i].Committees
		for j := 0; j < numValidators; j++ {
			var indexInCommittee uint64
			var committeeIndex primitives.CommitteeIndex
			var committeeLength int
			var found bool
			for comIndex, item := range comms {
				for n, idx := range item.ValidatorIndices {
					if primitives.ValidatorIndex(j) == idx {
						indexInCommittee = uint64(n)
						committeeIndex = primitives.CommitteeIndex(comIndex)
						committeeLength = len(item.ValidatorIndices)
						found = true
						break
					}
				}
			}
			if !found {
				continue
			}
			attExample := &ethpb.Attestation{
				Data: &ethpb.AttestationData{
					BeaconBlockRoot: bytesutil.PadTo([]byte("root"), 32),
					Slot:            i,
					Source: &ethpb.Checkpoint{
						Epoch: 0,
						Root:  gRoot[:],
					},
					Target: &ethpb.Checkpoint{
						Epoch: 0,
						Root:  gRoot[:],
					},
				},
			}
			domain, err := signing.Domain(headState.Fork(), 0, params.BeaconConfig().DomainBeaconAttester, headState.GenesisValidatorsRoot())
			require.NoError(t, err)
			encoded, err := signing.ComputeSigningRoot(attExample.Data, domain)
			require.NoError(t, err)
			sig := privKeys[j].Sign(encoded[:])
			attExample.Signature = sig.Marshal()
			attExample.Data.CommitteeIndex = committeeIndex
			aggregationBitfield := bitfield.NewBitlist(uint64(committeeLength))
			aggregationBitfield.SetBitAt(indexInCommittee, true)
			attExample.AggregationBits = aggregationBitfield
			atts[encoded] = append(atts[encoded], attExample)
		}
	}

	chainService := &chainMock.ChainService{}
	server := &Server{
		BeaconDB: db,
		Ctx:      context.Background(),
		HeadFetcher: &chainMock.ChainService{
			State: headState,
		},
		GenesisTimeFetcher: &chainMock.ChainService{
			Genesis: time.Now(),
		},
		AttestationNotifier:         chainService.OperationNotifier(),
		CollectedAttestationsBuffer: make(chan []*ethpb.Attestation, 1),
		StateGen:                    stategen.New(db, doublylinkedtree.New()),
	}

	for dataRoot, sameDataAtts := range atts {
		aggAtts, err := attaggregation.Aggregate(sameDataAtts)
		require.NoError(t, err)
		atts[dataRoot] = aggAtts
	}

	// Next up we convert the test attestations to indexed form.
	attsByTarget := make(map[[32]byte][]*ethpb.Attestation)
	for _, dataRootAtts := range atts {
		targetRoot := bytesutil.ToBytes32(dataRootAtts[0].Data.Target.Root)
		attsByTarget[targetRoot] = append(attsByTarget[targetRoot], dataRootAtts...)
	}

	allAtts := make([]*ethpb.Attestation, 0)
	indexedAtts := make(map[[32]byte][]*ethpb.IndexedAttestation)
	for dataRoot, aggAtts := range attsByTarget {
		allAtts = append(allAtts, aggAtts...)
		for _, att := range aggAtts {
			committee := committees[att.Data.Slot].Committees[att.Data.CommitteeIndex]
			idxAtt, err := attestation.ConvertToIndexed(ctx, att, committee.ValidatorIndices)
			require.NoError(t, err)
			indexedAtts[dataRoot] = append(indexedAtts[dataRoot], idxAtt)
		}
	}

	attsSent := 0
	mockStream := mock.NewMockBeaconChain_StreamIndexedAttestationsServer(ctrl)
	for _, atts := range indexedAtts {
		for _, att := range atts {
			if attsSent == len(allAtts)-1 {
				mockStream.EXPECT().Send(att).Do(func(arg0 interface{}) {
					exitRoutine <- true
				})
				t.Log("cancelled")
			} else {
				mockStream.EXPECT().Send(att)
				attsSent++
			}
		}
	}
	mockStream.EXPECT().Context().Return(ctx).AnyTimes()

	go func(tt *testing.T) {
		assert.NoError(tt, server.StreamIndexedAttestations(&emptypb.Empty{}, mockStream), "Could not call RPC method")
	}(t)

	server.CollectedAttestationsBuffer <- allAtts
	<-exitRoutine
}

func TestServer_StreamAttestations_ContextCanceled(t *testing.T) {
	ctx := context.Background()

	ctx, cancel := context.WithCancel(ctx)
	chainService := &chainMock.ChainService{}
	server := &Server{
		Ctx:                 ctx,
		AttestationNotifier: chainService.OperationNotifier(),
	}

	exitRoutine := make(chan bool)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	mockStream := mock.NewMockBeaconChain_StreamAttestationsServer(ctrl)
	mockStream.EXPECT().Context().Return(ctx)
	go func(tt *testing.T) {
		err := server.StreamAttestations(
			&emptypb.Empty{},
			mockStream,
		)
		assert.ErrorContains(tt, "Context canceled", err)
		<-exitRoutine
	}(t)
	cancel()
	exitRoutine <- true
}

func TestServer_StreamAttestations_OnSlotTick(t *testing.T) {
	exitRoutine := make(chan bool)
	ctrl := gomock.NewController(t)
	defer ctrl.Finish()
	ctx := context.Background()
	chainService := &chainMock.ChainService{}
	server := &Server{
		Ctx:                 ctx,
		AttestationNotifier: chainService.OperationNotifier(),
	}

	atts := []*ethpb.Attestation{
		util.HydrateAttestation(&ethpb.Attestation{Data: &ethpb.AttestationData{Slot: 1}, AggregationBits: bitfield.Bitlist{0b1101}}),
		util.HydrateAttestation(&ethpb.Attestation{Data: &ethpb.AttestationData{Slot: 2}, AggregationBits: bitfield.Bitlist{0b1101}}),
		util.HydrateAttestation(&ethpb.Attestation{Data: &ethpb.AttestationData{Slot: 3}, AggregationBits: bitfield.Bitlist{0b1101}}),
	}

	mockStream := mock.NewMockBeaconChain_StreamAttestationsServer(ctrl)
	mockStream.EXPECT().Send(atts[0])
	mockStream.EXPECT().Send(atts[1])
	mockStream.EXPECT().Send(atts[2]).Do(func(arg0 interface{}) {
		exitRoutine <- true
	})
	mockStream.EXPECT().Context().Return(ctx).AnyTimes()

	go func(tt *testing.T) {
		assert.NoError(tt, server.StreamAttestations(&emptypb.Empty{}, mockStream), "Could not call RPC method")
	}(t)
	for i := 0; i < len(atts); i++ {
		// Send in a loop to ensure it is delivered (busy wait for the service to subscribe to the state feed).
		for sent := 0; sent == 0; {
			sent = server.AttestationNotifier.OperationFeed().Send(&feed.Event{
				Type: operation.UnaggregatedAttReceived,
				Data: &operation.UnAggregatedAttReceivedData{Attestation: atts[i]},
			})
		}
	}
	<-exitRoutine
}
