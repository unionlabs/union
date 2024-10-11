package blockchain

import (
	"context"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	gethtypes "github.com/ethereum/go-ethereum/core/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	mockExecution "github.com/prysmaticlabs/prysm/v4/beacon-chain/execution/testing"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	bstate "github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func Test_NotifyForkchoiceUpdate_GetPayloadAttrErrorCanContinue(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	altairBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockAltair())
	altairBlkRoot, err := altairBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	bellatrixBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockBellatrix())
	bellatrixBlkRoot, err := bellatrixBlk.Block().HashTreeRoot()
	require.NoError(t, err)

	st, _ := util.DeterministicGenesisState(t, 10)
	service.head = &head{
		state: st,
	}

	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{}, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 1, altairBlkRoot, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 2, bellatrixBlkRoot, altairBlkRoot, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	sb := &ethpb.SignedBeaconBlockBellatrix{
		Block: &ethpb.BeaconBlockBellatrix{
			Body: &ethpb.BeaconBlockBodyBellatrix{
				ExecutionPayload: &v1.ExecutionPayload{},
			},
		},
	}
	b, err := consensusblocks.NewSignedBeaconBlock(sb)
	require.NoError(t, err)

	pid := &v1.PayloadIDBytes{1}
	service.cfg.ExecutionEngineCaller = &mockExecution.EngineClient{PayloadIDBytes: pid}
	st, _ = util.DeterministicGenesisState(t, 1)
	require.NoError(t, beaconDB.SaveState(ctx, st, bellatrixBlkRoot))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, bellatrixBlkRoot))

	// Intentionally generate a bad state such that `hash_tree_root` fails during `process_slot`
	s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{})
	require.NoError(t, err)
	arg := &fcuConfig{
		headState: s,
		headRoot:  [32]byte{},
		headBlock: b,
	}

	service.cfg.PayloadIDCache.Set(1, [32]byte{}, [8]byte{})
	got, err := service.notifyForkchoiceUpdate(ctx, arg)
	require.NoError(t, err)
	require.DeepEqual(t, got, pid) // We still get a payload ID even though the state is bad. This means it returns until the end.
}

func Test_NotifyForkchoiceUpdate(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	altairBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockAltair())
	altairBlkRoot, err := altairBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	bellatrixBlk := util.SaveBlock(t, ctx, beaconDB, util.NewBeaconBlockBellatrix())
	bellatrixBlkRoot, err := bellatrixBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	st, _ := util.DeterministicGenesisState(t, 10)
	service.head = &head{
		state: st,
	}

	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{}, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 1, altairBlkRoot, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 2, bellatrixBlkRoot, altairBlkRoot, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	tests := []struct {
		name             string
		blk              interfaces.ReadOnlySignedBeaconBlock
		headRoot         [32]byte
		finalizedRoot    [32]byte
		justifiedRoot    [32]byte
		newForkchoiceErr error
		errString        string
	}{
		{
			name: "phase0 block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: &ethpb.BeaconBlock{Body: &ethpb.BeaconBlockBody{}}})
				require.NoError(t, err)
				return b
			}(),
		},
		{
			name: "altair block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockAltair{Block: &ethpb.BeaconBlockAltair{Body: &ethpb.BeaconBlockBodyAltair{}}})
				require.NoError(t, err)
				return b
			}(),
		},
		{
			name: "not execution block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: &ethpb.BeaconBlockBellatrix{
					Body: &ethpb.BeaconBlockBodyBellatrix{
						ExecutionPayload: &v1.ExecutionPayload{
							ParentHash:    make([]byte, fieldparams.RootLength),
							FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
							StateRoot:     make([]byte, fieldparams.RootLength),
							ReceiptsRoot:  make([]byte, fieldparams.RootLength),
							LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
							PrevRandao:    make([]byte, fieldparams.RootLength),
							ExtraData:     make([]byte, 0),
							BaseFeePerGas: make([]byte, fieldparams.RootLength),
							BlockHash:     make([]byte, fieldparams.RootLength),
							Transactions:  make([][]byte, 0),
						},
					},
				}})
				require.NoError(t, err)
				return b
			}(),
		},
		{
			name: "happy case: finalized root is altair block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: &ethpb.BeaconBlockBellatrix{
					Body: &ethpb.BeaconBlockBodyBellatrix{
						ExecutionPayload: &v1.ExecutionPayload{},
					},
				}})
				require.NoError(t, err)
				return b
			}(),
			finalizedRoot: altairBlkRoot,
			justifiedRoot: altairBlkRoot,
		},
		{
			name: "happy case: finalized root is bellatrix block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: &ethpb.BeaconBlockBellatrix{
					Body: &ethpb.BeaconBlockBodyBellatrix{
						ExecutionPayload: &v1.ExecutionPayload{},
					},
				}})
				require.NoError(t, err)
				return b
			}(),
			finalizedRoot: bellatrixBlkRoot,
			justifiedRoot: bellatrixBlkRoot,
		},
		{
			name: "forkchoice updated with optimistic block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: &ethpb.BeaconBlockBellatrix{
					Body: &ethpb.BeaconBlockBodyBellatrix{
						ExecutionPayload: &v1.ExecutionPayload{},
					},
				}})
				require.NoError(t, err)
				return b
			}(),
			newForkchoiceErr: execution.ErrAcceptedSyncingPayloadStatus,
			finalizedRoot:    bellatrixBlkRoot,
			justifiedRoot:    bellatrixBlkRoot,
		},
		{
			name: "forkchoice updated with invalid block",
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				b, err := consensusblocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: &ethpb.BeaconBlockBellatrix{
					Body: &ethpb.BeaconBlockBodyBellatrix{
						ExecutionPayload: &v1.ExecutionPayload{},
					},
				}})
				require.NoError(t, err)
				return b
			}(),
			newForkchoiceErr: execution.ErrInvalidPayloadStatus,
			finalizedRoot:    bellatrixBlkRoot,
			justifiedRoot:    bellatrixBlkRoot,
			headRoot:         [32]byte{'a'},
			errString:        ErrInvalidPayload.Error(),
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			service.cfg.ExecutionEngineCaller = &mockExecution.EngineClient{ErrForkchoiceUpdated: tt.newForkchoiceErr}
			st, _ := util.DeterministicGenesisState(t, 1)
			require.NoError(t, beaconDB.SaveState(ctx, st, tt.finalizedRoot))
			require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, tt.finalizedRoot))
			arg := &fcuConfig{
				headState: st,
				headRoot:  tt.headRoot,
				headBlock: tt.blk,
			}
			_, err = service.notifyForkchoiceUpdate(ctx, arg)
			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, err)
				if tt.errString == ErrInvalidPayload.Error() {
					require.Equal(t, true, IsInvalidBlock(err))
					require.Equal(t, tt.headRoot, InvalidBlockRoot(err)) // Head root should be invalid. Not block root!
				}
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func Test_NotifyForkchoiceUpdate_NIlLVH(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	// Prepare blocks
	ba := util.NewBeaconBlockBellatrix()
	ba.Block.Body.ExecutionPayload.BlockNumber = 1
	wba := util.SaveBlock(t, ctx, beaconDB, ba)
	bra, err := wba.Block().HashTreeRoot()
	require.NoError(t, err)

	bb := util.NewBeaconBlockBellatrix()
	bb.Block.Body.ExecutionPayload.BlockNumber = 2
	wbb := util.SaveBlock(t, ctx, beaconDB, bb)
	brb, err := wbb.Block().HashTreeRoot()
	require.NoError(t, err)

	bc := util.NewBeaconBlockBellatrix()
	pc := [32]byte{'C'}
	bc.Block.Body.ExecutionPayload.BlockHash = pc[:]
	bc.Block.Body.ExecutionPayload.BlockNumber = 3
	wbc := util.SaveBlock(t, ctx, beaconDB, bc)
	brc, err := wbc.Block().HashTreeRoot()
	require.NoError(t, err)

	bd := util.NewBeaconBlockBellatrix()
	pd := [32]byte{'D'}
	bd.Block.Body.ExecutionPayload.BlockHash = pd[:]
	bd.Block.Body.ExecutionPayload.BlockNumber = 4
	bd.Block.ParentRoot = brc[:]
	wbd := util.SaveBlock(t, ctx, beaconDB, bd)
	brd, err := wbd.Block().HashTreeRoot()
	require.NoError(t, err)

	fcs.SetBalancesByRooter(func(context.Context, [32]byte) ([]uint64, error) { return []uint64{50, 100, 200}, nil })
	require.NoError(t, fcs.UpdateJustifiedCheckpoint(ctx, &forkchoicetypes.Checkpoint{}))
	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 1, bra, [32]byte{}, [32]byte{'A'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 2, brb, bra, [32]byte{'B'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 3, brc, brb, [32]byte{'C'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 4, brd, brc, [32]byte{'D'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	// Prepare Engine Mock to return invalid LVH =  nil
	service.cfg.ExecutionEngineCaller = &mockExecution.EngineClient{ErrForkchoiceUpdated: execution.ErrInvalidPayloadStatus, OverrideValidHash: [32]byte{'C'}}
	st, _ := util.DeterministicGenesisState(t, 1)
	service.head = &head{
		state: st,
		block: wba,
	}

	require.NoError(t, beaconDB.SaveState(ctx, st, bra))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, bra))
	a := &fcuConfig{
		headState: st,
		headBlock: wbd,
		headRoot:  brd,
	}
	_, err = service.notifyForkchoiceUpdate(ctx, a)
	require.Equal(t, true, IsInvalidBlock(err))
	require.Equal(t, brd, InvalidBlockRoot(err))
	require.Equal(t, brd, InvalidAncestorRoots(err)[0])
	require.Equal(t, 1, len(InvalidAncestorRoots(err)))
}

//
//
//  A <- B <- C <- D
//       \
//         ---------- E <- F
//                     \
//                       ------ G
// D is the current head, attestations for F and G come late, both are invalid.
// We switch recursively to F then G and finally to D.
//
// We test:
// 1. forkchoice removes blocks F and G from the forkchoice implementation
// 2. forkchoice removes the weights of these blocks
// 3. the blockchain package calls fcu to obtain heads G -> F -> D.

func Test_NotifyForkchoiceUpdateRecursive_DoublyLinkedTree(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	// Prepare blocks
	ba := util.NewBeaconBlockBellatrix()
	ba.Block.Body.ExecutionPayload.BlockNumber = 1
	wba := util.SaveBlock(t, ctx, beaconDB, ba)
	bra, err := wba.Block().HashTreeRoot()
	require.NoError(t, err)

	bb := util.NewBeaconBlockBellatrix()
	bb.Block.Body.ExecutionPayload.BlockNumber = 2
	wbb := util.SaveBlock(t, ctx, beaconDB, bb)
	brb, err := wbb.Block().HashTreeRoot()
	require.NoError(t, err)

	bc := util.NewBeaconBlockBellatrix()
	bc.Block.Body.ExecutionPayload.BlockNumber = 3
	wbc := util.SaveBlock(t, ctx, beaconDB, bc)
	brc, err := wbc.Block().HashTreeRoot()
	require.NoError(t, err)

	bd := util.NewBeaconBlockBellatrix()
	pd := [32]byte{'D'}
	bd.Block.Body.ExecutionPayload.BlockHash = pd[:]
	bd.Block.Body.ExecutionPayload.BlockNumber = 4
	wbd := util.SaveBlock(t, ctx, beaconDB, bd)
	brd, err := wbd.Block().HashTreeRoot()
	require.NoError(t, err)

	be := util.NewBeaconBlockBellatrix()
	pe := [32]byte{'E'}
	be.Block.Body.ExecutionPayload.BlockHash = pe[:]
	be.Block.Body.ExecutionPayload.BlockNumber = 5
	wbe := util.SaveBlock(t, ctx, beaconDB, be)
	bre, err := wbe.Block().HashTreeRoot()
	require.NoError(t, err)

	bf := util.NewBeaconBlockBellatrix()
	pf := [32]byte{'F'}
	bf.Block.Body.ExecutionPayload.BlockHash = pf[:]
	bf.Block.Body.ExecutionPayload.BlockNumber = 6
	bf.Block.ParentRoot = bre[:]
	wbf := util.SaveBlock(t, ctx, beaconDB, bf)
	brf, err := wbf.Block().HashTreeRoot()
	require.NoError(t, err)

	bg := util.NewBeaconBlockBellatrix()
	bg.Block.Body.ExecutionPayload.BlockNumber = 7
	pg := [32]byte{'G'}
	bg.Block.Body.ExecutionPayload.BlockHash = pg[:]
	bg.Block.ParentRoot = bre[:]
	wbg := util.SaveBlock(t, ctx, beaconDB, bg)
	brg, err := wbg.Block().HashTreeRoot()
	require.NoError(t, err)

	fcs.SetBalancesByRooter(func(context.Context, [32]byte) ([]uint64, error) { return []uint64{50, 100, 200}, nil })
	require.NoError(t, fcs.UpdateJustifiedCheckpoint(ctx, &forkchoicetypes.Checkpoint{}))
	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 1, bra, [32]byte{}, [32]byte{'A'}, ojc, ofc)
	require.NoError(t, err)

	bState, _ := util.DeterministicGenesisState(t, 10)
	require.NoError(t, beaconDB.SaveState(ctx, bState, bra))
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 2, brb, bra, [32]byte{'B'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 3, brc, brb, [32]byte{'C'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 4, brd, brc, [32]byte{'D'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 5, bre, brb, [32]byte{'E'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 6, brf, bre, [32]byte{'F'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 7, brg, bre, [32]byte{'G'}, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	// Insert Attestations to D, F and G so that they have higher weight than D
	// Ensure G is head
	fcs.ProcessAttestation(ctx, []uint64{0}, brd, 1)
	fcs.ProcessAttestation(ctx, []uint64{1}, brf, 1)
	fcs.ProcessAttestation(ctx, []uint64{2}, brg, 1)
	fcs.SetBalancesByRooter(service.cfg.StateGen.ActiveNonSlashedBalancesByRoot)
	jc := &forkchoicetypes.Checkpoint{Epoch: 0, Root: bra}
	require.NoError(t, fcs.UpdateJustifiedCheckpoint(ctx, jc))
	fcs.SetBalancesByRooter(func(context.Context, [32]byte) ([]uint64, error) { return []uint64{50, 100, 200}, nil })
	require.NoError(t, fcs.UpdateJustifiedCheckpoint(ctx, &forkchoicetypes.Checkpoint{}))
	headRoot, err := fcs.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, brg, headRoot)

	// Prepare Engine Mock to return invalid unless head is D, LVH =  E
	service.cfg.ExecutionEngineCaller = &mockExecution.EngineClient{ErrForkchoiceUpdated: execution.ErrInvalidPayloadStatus, ForkChoiceUpdatedResp: pe[:], OverrideValidHash: [32]byte{'D'}}
	st, _ := util.DeterministicGenesisState(t, 1)
	service.head = &head{
		state: st,
		block: wba,
	}

	require.NoError(t, beaconDB.SaveState(ctx, st, bra))
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, bra))
	a := &fcuConfig{
		headState: st,
		headBlock: wbg,
		headRoot:  brg,
	}
	_, err = service.notifyForkchoiceUpdate(ctx, a)
	require.Equal(t, true, IsInvalidBlock(err))
	require.Equal(t, brf, InvalidBlockRoot(err))

	// Ensure Head is D
	headRoot, err = fcs.Head(ctx)
	require.NoError(t, err)
	require.Equal(t, brd, headRoot)

	// Ensure F and G where removed but their parent E wasn't
	require.Equal(t, false, fcs.HasNode(brf))
	require.Equal(t, false, fcs.HasNode(brg))
	require.Equal(t, true, fcs.HasNode(bre))
}

func Test_NotifyNewPayload(t *testing.T) {
	cfg := params.BeaconConfig()
	cfg.TerminalTotalDifficulty = "2"
	params.OverrideBeaconConfig(cfg)
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx, fcs := tr.ctx, tr.fcs

	phase0State, _ := util.DeterministicGenesisState(t, 1)
	altairState, _ := util.DeterministicGenesisStateAltair(t, 1)
	bellatrixState, _ := util.DeterministicGenesisStateBellatrix(t, 2)
	a := &ethpb.SignedBeaconBlockAltair{
		Block: &ethpb.BeaconBlockAltair{
			Body: &ethpb.BeaconBlockBodyAltair{},
		},
	}
	altairBlk, err := consensusblocks.NewSignedBeaconBlock(a)
	require.NoError(t, err)
	blk := &ethpb.SignedBeaconBlockBellatrix{
		Block: &ethpb.BeaconBlockBellatrix{
			Slot: 1,
			Body: &ethpb.BeaconBlockBodyBellatrix{
				ExecutionPayload: &v1.ExecutionPayload{
					BlockNumber:   1,
					ParentHash:    make([]byte, fieldparams.RootLength),
					FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
					StateRoot:     make([]byte, fieldparams.RootLength),
					ReceiptsRoot:  make([]byte, fieldparams.RootLength),
					LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
					PrevRandao:    make([]byte, fieldparams.RootLength),
					ExtraData:     make([]byte, 0),
					BaseFeePerGas: make([]byte, fieldparams.RootLength),
					BlockHash:     make([]byte, fieldparams.RootLength),
					Transactions:  make([][]byte, 0),
				},
			},
		},
	}
	bellatrixBlk, err := consensusblocks.NewSignedBeaconBlock(util.HydrateSignedBeaconBlockBellatrix(blk))
	require.NoError(t, err)
	st := params.BeaconConfig().SlotsPerEpoch.Mul(uint64(epochsSinceFinalitySaveHotStateDB))
	service.genesisTime = time.Now().Add(time.Duration(-1*int64(st)*int64(params.BeaconConfig().SecondsPerSlot)) * time.Second)
	r, err := bellatrixBlk.Block().HashTreeRoot()
	require.NoError(t, err)
	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	state, blkRoot, err := prepareForkchoiceState(ctx, 0, [32]byte{}, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	state, blkRoot, err = prepareForkchoiceState(ctx, 1, r, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))

	tests := []struct {
		postState      bstate.BeaconState
		invalidBlock   bool
		isValidPayload bool
		blk            interfaces.ReadOnlySignedBeaconBlock
		newPayloadErr  error
		errString      string
		name           string
	}{
		{
			name:           "phase 0 post state",
			postState:      phase0State,
			blk:            altairBlk, // same as phase 0 for this test
			isValidPayload: true,
		},
		{
			name:           "altair post state",
			postState:      altairState,
			blk:            altairBlk,
			isValidPayload: true,
		},
		{
			name:           "nil beacon block",
			postState:      bellatrixState,
			errString:      "signed beacon block can't be nil",
			isValidPayload: false,
		},
		{
			name:           "new payload with optimistic block",
			postState:      bellatrixState,
			blk:            bellatrixBlk,
			newPayloadErr:  execution.ErrAcceptedSyncingPayloadStatus,
			isValidPayload: false,
		},
		{
			name:           "new payload with invalid block",
			postState:      bellatrixState,
			blk:            bellatrixBlk,
			newPayloadErr:  execution.ErrInvalidPayloadStatus,
			errString:      ErrInvalidPayload.Error(),
			isValidPayload: false,
			invalidBlock:   true,
		},
		{
			name:           "altair pre state, altair block",
			postState:      bellatrixState,
			blk:            altairBlk,
			isValidPayload: true,
		},
		{
			name:      "altair pre state, happy case",
			postState: bellatrixState,
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				blk := &ethpb.SignedBeaconBlockBellatrix{
					Block: &ethpb.BeaconBlockBellatrix{
						Body: &ethpb.BeaconBlockBodyBellatrix{
							ExecutionPayload: &v1.ExecutionPayload{
								ParentHash: bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
							},
						},
					},
				}
				b, err := consensusblocks.NewSignedBeaconBlock(blk)
				require.NoError(t, err)
				return b
			}(),
			isValidPayload: true,
		},
		{
			name:      "not at merge transition",
			postState: bellatrixState,
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				blk := &ethpb.SignedBeaconBlockBellatrix{
					Block: &ethpb.BeaconBlockBellatrix{
						Body: &ethpb.BeaconBlockBodyBellatrix{
							ExecutionPayload: &v1.ExecutionPayload{
								ParentHash:    make([]byte, fieldparams.RootLength),
								FeeRecipient:  make([]byte, fieldparams.FeeRecipientLength),
								StateRoot:     make([]byte, fieldparams.RootLength),
								ReceiptsRoot:  make([]byte, fieldparams.RootLength),
								LogsBloom:     make([]byte, fieldparams.LogsBloomLength),
								PrevRandao:    make([]byte, fieldparams.RootLength),
								ExtraData:     make([]byte, 0),
								BaseFeePerGas: make([]byte, fieldparams.RootLength),
								BlockHash:     make([]byte, fieldparams.RootLength),
								Transactions:  make([][]byte, 0),
							},
						},
					},
				}
				b, err := consensusblocks.NewSignedBeaconBlock(blk)
				require.NoError(t, err)
				return b
			}(),
			isValidPayload: true,
		},
		{
			name:      "happy case",
			postState: bellatrixState,
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				blk := &ethpb.SignedBeaconBlockBellatrix{
					Block: &ethpb.BeaconBlockBellatrix{
						Body: &ethpb.BeaconBlockBodyBellatrix{
							ExecutionPayload: &v1.ExecutionPayload{
								ParentHash: bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
							},
						},
					},
				}
				b, err := consensusblocks.NewSignedBeaconBlock(blk)
				require.NoError(t, err)
				return b
			}(),
			isValidPayload: true,
		},
		{
			name:      "undefined error from ee",
			postState: bellatrixState,
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				blk := &ethpb.SignedBeaconBlockBellatrix{
					Block: &ethpb.BeaconBlockBellatrix{
						Body: &ethpb.BeaconBlockBodyBellatrix{
							ExecutionPayload: &v1.ExecutionPayload{
								ParentHash: bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
							},
						},
					},
				}
				b, err := consensusblocks.NewSignedBeaconBlock(blk)
				require.NoError(t, err)
				return b
			}(),
			newPayloadErr: ErrUndefinedExecutionEngineError,
			errString:     ErrUndefinedExecutionEngineError.Error(),
		},
		{
			name:      "invalid block hash error from ee",
			postState: bellatrixState,
			blk: func() interfaces.ReadOnlySignedBeaconBlock {
				blk := &ethpb.SignedBeaconBlockBellatrix{
					Block: &ethpb.BeaconBlockBellatrix{
						Body: &ethpb.BeaconBlockBodyBellatrix{
							ExecutionPayload: &v1.ExecutionPayload{
								ParentHash: bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
							},
						},
					},
				}
				b, err := consensusblocks.NewSignedBeaconBlock(blk)
				require.NoError(t, err)
				return b
			}(),
			newPayloadErr: ErrInvalidBlockHashPayloadStatus,
			errString:     ErrInvalidBlockHashPayloadStatus.Error(),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			e := &mockExecution.EngineClient{ErrNewPayload: tt.newPayloadErr, BlockByHashMap: map[[32]byte]*v1.ExecutionBlock{}}
			e.BlockByHashMap[[32]byte{'a'}] = &v1.ExecutionBlock{
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("b")),
				},
				TotalDifficulty: "0x2",
			}
			e.BlockByHashMap[[32]byte{'b'}] = &v1.ExecutionBlock{
				Header: gethtypes.Header{
					ParentHash: common.BytesToHash([]byte("3")),
				},
				TotalDifficulty: "0x1",
			}
			service.cfg.ExecutionEngineCaller = e
			root := [32]byte{'a'}
			state, blkRoot, err := prepareForkchoiceState(ctx, 0, root, [32]byte{}, params.BeaconConfig().ZeroHash, ojc, ofc)
			require.NoError(t, err)
			require.NoError(t, service.cfg.ForkChoiceStore.InsertNode(ctx, state, blkRoot))
			postVersion, postHeader, err := getStateVersionAndPayload(tt.postState)
			require.NoError(t, err)
			isValidPayload, err := service.notifyNewPayload(ctx, postVersion, postHeader, tt.blk)
			if tt.errString != "" {
				require.ErrorContains(t, tt.errString, err)
				if tt.invalidBlock {
					require.Equal(t, true, IsInvalidBlock(err))
				}
			} else {
				require.NoError(t, err)
				require.Equal(t, tt.isValidPayload, isValidPayload)
				require.Equal(t, false, IsInvalidBlock(err))
			}
		})
	}
}

func Test_NotifyNewPayload_SetOptimisticToValid(t *testing.T) {
	cfg := params.BeaconConfig()
	cfg.TerminalTotalDifficulty = "2"
	params.OverrideBeaconConfig(cfg)

	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	bellatrixState, _ := util.DeterministicGenesisStateBellatrix(t, 2)
	blk := &ethpb.SignedBeaconBlockBellatrix{
		Block: &ethpb.BeaconBlockBellatrix{
			Body: &ethpb.BeaconBlockBodyBellatrix{
				ExecutionPayload: &v1.ExecutionPayload{
					ParentHash: bytesutil.PadTo([]byte{'a'}, fieldparams.RootLength),
				},
			},
		},
	}
	bellatrixBlk, err := consensusblocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	e := &mockExecution.EngineClient{BlockByHashMap: map[[32]byte]*v1.ExecutionBlock{}}
	e.BlockByHashMap[[32]byte{'a'}] = &v1.ExecutionBlock{
		Header: gethtypes.Header{
			ParentHash: common.BytesToHash([]byte("b")),
		},
		TotalDifficulty: "0x2",
	}
	e.BlockByHashMap[[32]byte{'b'}] = &v1.ExecutionBlock{
		Header: gethtypes.Header{
			ParentHash: common.BytesToHash([]byte("3")),
		},
		TotalDifficulty: "0x1",
	}
	service.cfg.ExecutionEngineCaller = e
	postVersion, postHeader, err := getStateVersionAndPayload(bellatrixState)
	require.NoError(t, err)
	validated, err := service.notifyNewPayload(ctx, postVersion, postHeader, bellatrixBlk)
	require.NoError(t, err)
	require.Equal(t, true, validated)
}

func Test_reportInvalidBlock(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.MainnetConfig())
	service, tr := minimalTestService(t)
	ctx, _, fcs := tr.ctx, tr.db, tr.fcs
	jcp := &ethpb.Checkpoint{}
	st, root, err := prepareForkchoiceState(ctx, 0, [32]byte{'A'}, [32]byte{}, [32]byte{'a'}, jcp, jcp)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 1, [32]byte{'B'}, [32]byte{'A'}, [32]byte{'b'}, jcp, jcp)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))
	st, root, err = prepareForkchoiceState(ctx, 2, [32]byte{'C'}, [32]byte{'B'}, [32]byte{'c'}, jcp, jcp)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))

	st, root, err = prepareForkchoiceState(ctx, 3, [32]byte{'D'}, [32]byte{'C'}, [32]byte{'d'}, jcp, jcp)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, st, root))

	require.NoError(t, fcs.SetOptimisticToValid(ctx, [32]byte{'A'}))
	err = service.pruneInvalidBlock(ctx, [32]byte{'D'}, [32]byte{'C'}, [32]byte{'a'})
	require.Equal(t, IsInvalidBlock(err), true)
	require.Equal(t, InvalidBlockLVH(err), [32]byte{'a'})
	invalidRoots := InvalidAncestorRoots(err)
	require.Equal(t, 3, len(invalidRoots))
	require.Equal(t, [32]byte{'D'}, invalidRoots[0])
	require.Equal(t, [32]byte{'C'}, invalidRoots[1])
	require.Equal(t, [32]byte{'B'}, invalidRoots[2])
}

func Test_GetPayloadAttribute(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	attr := service.getPayloadAttribute(ctx, st, 0, []byte{})
	require.Equal(t, true, attr.IsEmpty())

	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, Index: 0})
	// Cache hit, advance state, no fee recipient
	slot := primitives.Slot(1)
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, params.BeaconConfig().EthBurnAddressHex, common.BytesToAddress(attr.SuggestedFeeRecipient()).String())

	// Cache hit, advance state, has fee recipient
	suggestedAddr := common.HexToAddress("123")
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, FeeRecipient: primitives.ExecutionAddress(suggestedAddr), Index: 0})
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, suggestedAddr, common.BytesToAddress(attr.SuggestedFeeRecipient()))
}

func Test_GetPayloadAttribute_PrepareAllPayloads(t *testing.T) {
	resetCfg := features.InitWithReset(&features.Flags{
		PrepareAllPayloads: true,
	})
	defer resetCfg()

	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	attr := service.getPayloadAttribute(ctx, st, 0, []byte{})
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, params.BeaconConfig().EthBurnAddressHex, common.BytesToAddress(attr.SuggestedFeeRecipient()).String())
}

func Test_GetPayloadAttributeV2(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	st, _ := util.DeterministicGenesisStateCapella(t, 1)
	attr := service.getPayloadAttribute(ctx, st, 0, []byte{})
	require.Equal(t, true, attr.IsEmpty())

	// Cache hit, advance state, no fee recipient
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, Index: 0})
	slot := primitives.Slot(1)
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, params.BeaconConfig().EthBurnAddressHex, common.BytesToAddress(attr.SuggestedFeeRecipient()).String())
	a, err := attr.Withdrawals()
	require.NoError(t, err)
	require.Equal(t, 0, len(a))

	// Cache hit, advance state, has fee recipient
	suggestedAddr := common.HexToAddress("123")
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, FeeRecipient: primitives.ExecutionAddress(suggestedAddr), Index: 0})
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, suggestedAddr, common.BytesToAddress(attr.SuggestedFeeRecipient()))
	a, err = attr.Withdrawals()
	require.NoError(t, err)
	require.Equal(t, 0, len(a))
}

func Test_GetPayloadAttributeDeneb(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	st, _ := util.DeterministicGenesisStateDeneb(t, 1)
	attr := service.getPayloadAttribute(ctx, st, 0, []byte{})
	require.Equal(t, true, attr.IsEmpty())

	// Cache hit, advance state, no fee recipient
	slot := primitives.Slot(1)
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, Index: 0})
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, params.BeaconConfig().EthBurnAddressHex, common.BytesToAddress(attr.SuggestedFeeRecipient()).String())
	a, err := attr.Withdrawals()
	require.NoError(t, err)
	require.Equal(t, 0, len(a))

	// Cache hit, advance state, has fee recipient
	suggestedAddr := common.HexToAddress("123")
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, FeeRecipient: primitives.ExecutionAddress(suggestedAddr), Index: 0})
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	attr = service.getPayloadAttribute(ctx, st, slot, params.BeaconConfig().ZeroHash[:])
	require.Equal(t, false, attr.IsEmpty())
	require.Equal(t, suggestedAddr, common.BytesToAddress(attr.SuggestedFeeRecipient()))
	a, err = attr.Withdrawals()
	require.NoError(t, err)
	require.Equal(t, 0, len(a))

	attrV3, err := attr.PbV3()
	require.NoError(t, err)
	hr := service.headRoot()
	require.Equal(t, hr, [32]byte(attrV3.ParentBeaconBlockRoot))

}

func Test_UpdateLastValidatedCheckpoint(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.MainnetConfig())
	service, tr := minimalTestService(t)
	ctx, beaconDB, fcs := tr.ctx, tr.db, tr.fcs

	var genesisStateRoot [32]byte
	genesisBlk := blocks.NewGenesisBlock(genesisStateRoot[:])
	util.SaveBlock(t, ctx, beaconDB, genesisBlk)
	genesisRoot, err := genesisBlk.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, genesisRoot))
	ojc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	ofc := &ethpb.Checkpoint{Root: params.BeaconConfig().ZeroHash[:]}
	fjc := &forkchoicetypes.Checkpoint{Epoch: 0, Root: params.BeaconConfig().ZeroHash}
	require.NoError(t, fcs.UpdateJustifiedCheckpoint(ctx, fjc))
	require.NoError(t, fcs.UpdateFinalizedCheckpoint(fjc))
	state, blkRoot, err := prepareForkchoiceState(ctx, 0, genesisRoot, params.BeaconConfig().ZeroHash, params.BeaconConfig().ZeroHash, ojc, ofc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	fcs.SetOriginRoot(genesisRoot)
	genesisSummary := &ethpb.StateSummary{
		Root: genesisStateRoot[:],
		Slot: 0,
	}
	require.NoError(t, beaconDB.SaveStateSummary(ctx, genesisSummary))

	// Get last validated checkpoint
	origCheckpoint, err := service.cfg.BeaconDB.LastValidatedCheckpoint(ctx)
	require.NoError(t, err)
	require.NoError(t, beaconDB.SaveLastValidatedCheckpoint(ctx, origCheckpoint))

	// Optimistic finalized checkpoint
	blk := util.NewBeaconBlock()
	blk.Block.Slot = 320
	blk.Block.ParentRoot = genesisRoot[:]
	util.SaveBlock(t, ctx, beaconDB, blk)
	opRoot, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)

	opCheckpoint := &ethpb.Checkpoint{
		Root:  opRoot[:],
		Epoch: 10,
	}
	opStateSummary := &ethpb.StateSummary{
		Root: opRoot[:],
		Slot: 320,
	}
	require.NoError(t, beaconDB.SaveStateSummary(ctx, opStateSummary))
	tenjc := &ethpb.Checkpoint{Epoch: 10, Root: genesisRoot[:]}
	tenfc := &ethpb.Checkpoint{Epoch: 10, Root: genesisRoot[:]}
	state, blkRoot, err = prepareForkchoiceState(ctx, 320, opRoot, genesisRoot, params.BeaconConfig().ZeroHash, tenjc, tenfc)
	require.NoError(t, err)
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	assert.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, opRoot))
	require.NoError(t, service.updateFinalized(ctx, opCheckpoint))
	cp, err := service.cfg.BeaconDB.LastValidatedCheckpoint(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, origCheckpoint.Root, cp.Root)
	require.Equal(t, origCheckpoint.Epoch, cp.Epoch)

	// Validated finalized checkpoint
	blk = util.NewBeaconBlock()
	blk.Block.Slot = 640
	blk.Block.ParentRoot = opRoot[:]
	util.SaveBlock(t, ctx, beaconDB, blk)
	validRoot, err := blk.Block.HashTreeRoot()
	require.NoError(t, err)

	validCheckpoint := &ethpb.Checkpoint{
		Root:  validRoot[:],
		Epoch: 20,
	}
	validSummary := &ethpb.StateSummary{
		Root: validRoot[:],
		Slot: 640,
	}
	require.NoError(t, beaconDB.SaveStateSummary(ctx, validSummary))
	twentyjc := &ethpb.Checkpoint{Epoch: 20, Root: validRoot[:]}
	twentyfc := &ethpb.Checkpoint{Epoch: 20, Root: validRoot[:]}
	state, blkRoot, err = prepareForkchoiceState(ctx, 640, validRoot, genesisRoot, params.BeaconConfig().ZeroHash, twentyjc, twentyfc)
	require.NoError(t, err)
	fcs.SetBalancesByRooter(func(_ context.Context, _ [32]byte) ([]uint64, error) { return []uint64{}, nil })
	require.NoError(t, fcs.InsertNode(ctx, state, blkRoot))
	require.NoError(t, fcs.SetOptimisticToValid(ctx, validRoot))
	assert.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, validRoot))
	require.NoError(t, service.updateFinalized(ctx, validCheckpoint))
	cp, err = service.cfg.BeaconDB.LastValidatedCheckpoint(ctx)
	require.NoError(t, err)

	optimistic, err := service.IsOptimisticForRoot(ctx, validRoot)
	require.NoError(t, err)
	require.Equal(t, false, optimistic)
	require.DeepEqual(t, validCheckpoint.Root, cp.Root)
	require.Equal(t, validCheckpoint.Epoch, cp.Epoch)

	// Checkpoint with a lower epoch
	oldCp, err := service.cfg.BeaconDB.FinalizedCheckpoint(ctx)
	require.NoError(t, err)
	invalidCp := &ethpb.Checkpoint{
		Epoch: oldCp.Epoch - 1,
	}
	// Nothing should happen as we no-op on an invalid checkpoint.
	require.NoError(t, service.updateFinalized(ctx, invalidCp))
	got, err := service.cfg.BeaconDB.FinalizedCheckpoint(ctx)
	require.NoError(t, err)
	require.DeepEqual(t, oldCp, got)
}

func TestService_removeInvalidBlockAndState(t *testing.T) {
	service, tr := minimalTestService(t)
	ctx := tr.ctx

	// Deleting unknown block should not error.
	require.NoError(t, service.removeInvalidBlockAndState(ctx, [][32]byte{{'a'}, {'b'}, {'c'}}))

	// Happy case
	b1 := util.NewBeaconBlock()
	b1.Block.Slot = 1
	blk1 := util.SaveBlock(t, ctx, service.cfg.BeaconDB, b1)
	r1, err := blk1.Block().HashTreeRoot()
	require.NoError(t, err)
	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)
	require.NoError(t, service.cfg.BeaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{
		Slot: 1,
		Root: r1[:],
	}))
	require.NoError(t, service.cfg.BeaconDB.SaveState(ctx, st, r1))

	b2 := util.NewBeaconBlock()
	b2.Block.Slot = 2
	blk2 := util.SaveBlock(t, ctx, service.cfg.BeaconDB, b2)
	r2, err := blk2.Block().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, service.cfg.BeaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{
		Slot: 2,
		Root: r2[:],
	}))
	require.NoError(t, service.cfg.BeaconDB.SaveState(ctx, st, r2))

	require.NoError(t, service.removeInvalidBlockAndState(ctx, [][32]byte{r1, r2}))

	require.Equal(t, false, service.hasBlock(ctx, r1))
	require.Equal(t, false, service.hasBlock(ctx, r2))
	require.Equal(t, false, service.cfg.BeaconDB.HasStateSummary(ctx, r1))
	require.Equal(t, false, service.cfg.BeaconDB.HasStateSummary(ctx, r2))
	has, err := service.cfg.StateGen.HasState(ctx, r1)
	require.NoError(t, err)
	require.Equal(t, false, has)
	has, err = service.cfg.StateGen.HasState(ctx, r2)
	require.NoError(t, err)
	require.Equal(t, false, has)
}

func TestService_getPayloadHash(t *testing.T) {
	service, tr := minimalTestService(t)
	ctx := tr.ctx

	_, err := service.getPayloadHash(ctx, []byte{})
	require.ErrorIs(t, errBlockNotFoundInCacheOrDB, err)

	b := util.NewBeaconBlock()
	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err := consensusblocks.NewSignedBeaconBlock(b)
	require.NoError(t, err)
	require.NoError(t, service.saveInitSyncBlock(ctx, r, wsb))

	h, err := service.getPayloadHash(ctx, r[:])
	require.NoError(t, err)
	require.DeepEqual(t, params.BeaconConfig().ZeroHash, h)

	bb := util.NewBeaconBlockBellatrix()
	h = [32]byte{'a'}
	bb.Block.Body.ExecutionPayload.BlockHash = h[:]
	r, err = b.Block.HashTreeRoot()
	require.NoError(t, err)
	wsb, err = consensusblocks.NewSignedBeaconBlock(bb)
	require.NoError(t, err)
	require.NoError(t, service.saveInitSyncBlock(ctx, r, wsb))

	h, err = service.getPayloadHash(ctx, r[:])
	require.NoError(t, err)
	require.DeepEqual(t, [32]byte{'a'}, h)
}

func TestKZGCommitmentToVersionedHashes(t *testing.T) {
	kzg1 := make([]byte, 96)
	kzg1[10] = 'a'
	kzg2 := make([]byte, 96)
	kzg2[1] = 'b'
	commitments := [][]byte{kzg1, kzg2}

	blk := &ethpb.SignedBeaconBlockDeneb{
		Block: &ethpb.BeaconBlockDeneb{
			Body: &ethpb.BeaconBlockBodyDeneb{
				BlobKzgCommitments: commitments,
			},
		},
	}
	b, err := consensusblocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	vhs, err := kzgCommitmentsToVersionedHashes(b.Block().Body())
	require.NoError(t, err)
	vh0 := "0x01cf2315c97658a7ed54ada181765e23b3fadb5150fab39509f631c0b9af4566"
	vh1 := "0x01e27ce28e527eb07196b31af0f5fa1882ace701a682022ab779f816ac39d47e"
	require.Equal(t, 2, len(vhs))
	require.Equal(t, vhs[0].String(), vh0)
	require.Equal(t, vhs[1].String(), vh1)
}

func TestComputePayloadAttribute(t *testing.T) {
	service, tr := minimalTestService(t, WithPayloadIDCache(cache.NewPayloadIDCache()))
	ctx := tr.ctx

	st, _ := util.DeterministicGenesisStateBellatrix(t, 1)

	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, Index: 0})
	// Cache hit, advance state, no fee recipient
	slot := primitives.Slot(1)
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	cfg := &postBlockProcessConfig{
		ctx:       ctx,
		blockRoot: [32]byte{'a'},
	}
	fcu := &fcuConfig{
		headState:     st,
		proposingSlot: slot,
		headRoot:      [32]byte{},
	}
	require.NoError(t, service.computePayloadAttributes(cfg, fcu))
	require.Equal(t, false, fcu.attributes.IsEmpty())
	require.Equal(t, params.BeaconConfig().EthBurnAddressHex, common.BytesToAddress(fcu.attributes.SuggestedFeeRecipient()).String())

	// Cache hit, advance state, has fee recipient
	suggestedAddr := common.HexToAddress("123")
	service.cfg.TrackedValidatorsCache.Set(cache.TrackedValidator{Active: true, FeeRecipient: primitives.ExecutionAddress(suggestedAddr), Index: 0})
	service.cfg.PayloadIDCache.Set(slot, [32]byte{}, [8]byte{})
	require.NoError(t, service.computePayloadAttributes(cfg, fcu))
	require.Equal(t, false, fcu.attributes.IsEmpty())
	require.Equal(t, suggestedAddr, common.BytesToAddress(fcu.attributes.SuggestedFeeRecipient()))
}
