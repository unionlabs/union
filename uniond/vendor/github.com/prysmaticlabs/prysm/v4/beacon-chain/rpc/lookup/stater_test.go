package lookup

import (
	"context"
	"strconv"
	"strings"
	"testing"
	"time"

	statenative "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"

	"github.com/ethereum/go-ethereum/common/hexutil"
	chainMock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	mockstategen "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen/mock"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestGetState(t *testing.T) {
	ctx := context.Background()

	headSlot := primitives.Slot(123)
	fillSlot := func(state *ethpb.BeaconState) error {
		state.Slot = headSlot
		return nil
	}
	newBeaconState, err := util.NewBeaconState(util.FillRootsNaturalOpt, fillSlot)
	require.NoError(t, err)
	stateRoot, err := newBeaconState.HashTreeRoot(ctx)
	require.NoError(t, err)

	t.Run("head", func(t *testing.T) {
		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
		}

		s, err := p.State(ctx, []byte("head"))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.DeepEqual(t, stateRoot, sRoot)
	})

	t.Run("genesis", func(t *testing.T) {
		params.SetupTestConfigCleanup(t)
		cfg := params.BeaconConfig().Copy()
		cfg.ConfigName = "test"
		params.OverrideBeaconConfig(cfg)

		db := testDB.SetupDB(t)
		b := util.NewBeaconBlock()
		b.Block.StateRoot = bytesutil.PadTo([]byte("foo"), 32)
		util.SaveBlock(t, ctx, db, b)
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)

		bs, err := util.NewBeaconState(func(state *ethpb.BeaconState) error {
			state.BlockRoots[0] = r[:]
			return nil
		})
		require.NoError(t, err)
		newStateRoot, err := bs.HashTreeRoot(ctx)
		require.NoError(t, err)

		require.NoError(t, db.SaveStateSummary(ctx, &ethpb.StateSummary{Root: r[:]}))
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, r))
		require.NoError(t, db.SaveState(ctx, bs, r))

		cc := &mockstategen.CanonicalChecker{Is: true}
		cs := &mockstategen.CurrentSlotter{Slot: bs.Slot() + 1}
		ch := stategen.NewCanonicalHistory(db, cc, cs)
		currentSlot := primitives.Slot(0)
		p := BeaconDbStater{
			BeaconDB:           db,
			ReplayerBuilder:    ch,
			GenesisTimeFetcher: &chainMock.ChainService{Slot: &currentSlot},
			ChainInfoFetcher:   &chainMock.ChainService{State: bs},
		}

		s, err := p.State(ctx, []byte("genesis"))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.DeepEqual(t, newStateRoot, sRoot)
	})

	t.Run("finalized", func(t *testing.T) {
		stateGen := mockstategen.NewService()
		replayer := mockstategen.NewReplayerBuilder()
		replayer.SetMockStateForSlot(newBeaconState, params.BeaconConfig().SlotsPerEpoch*10)
		stateGen.StatesByRoot[stateRoot] = newBeaconState

		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{
				FinalizedCheckPoint: &ethpb.Checkpoint{
					Root:  stateRoot[:],
					Epoch: 10,
				},
			},
			StateGenService: stateGen,
			ReplayerBuilder: replayer,
		}

		s, err := p.State(ctx, []byte("finalized"))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.Equal(t, stateRoot, sRoot)
	})

	t.Run("justified", func(t *testing.T) {
		stateGen := mockstategen.NewService()
		replayer := mockstategen.NewReplayerBuilder()
		replayer.SetMockStateForSlot(newBeaconState, params.BeaconConfig().SlotsPerEpoch*10)
		stateGen.StatesByRoot[stateRoot] = newBeaconState

		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{
				CurrentJustifiedCheckPoint: &ethpb.Checkpoint{
					Root:  stateRoot[:],
					Epoch: 10,
				},
			},
			StateGenService: stateGen,
			ReplayerBuilder: replayer,
		}

		s, err := p.State(ctx, []byte("justified"))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.DeepEqual(t, stateRoot, sRoot)
	})

	t.Run("hex", func(t *testing.T) {
		hex := "0x" + strings.Repeat("0", 63) + "1"
		root, err := hexutil.Decode(hex)
		require.NoError(t, err)
		stateGen := mockstategen.NewService()
		stateGen.StatesByRoot[bytesutil.ToBytes32(root)] = newBeaconState

		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
			StateGenService:  stateGen,
		}

		s, err := p.State(ctx, []byte(hex))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.DeepEqual(t, stateRoot, sRoot)
	})

	t.Run("root", func(t *testing.T) {
		stateId, err := hexutil.Decode("0x" + strings.Repeat("0", 63) + "1")
		require.NoError(t, err)
		stateGen := mockstategen.NewService()
		stateGen.StatesByRoot[bytesutil.ToBytes32(stateId)] = newBeaconState

		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
			StateGenService:  stateGen,
		}

		s, err := p.State(ctx, stateId)
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.DeepEqual(t, stateRoot, sRoot)
	})

	t.Run("root not found", func(t *testing.T) {
		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
		}
		stateId, err := hexutil.Decode("0x" + strings.Repeat("f", 64))
		require.NoError(t, err)
		_, err = p.State(ctx, stateId)
		require.ErrorContains(t, "state not found in the last 8192 state roots", err)
	})

	t.Run("slot", func(t *testing.T) {
		p := BeaconDbStater{
			GenesisTimeFetcher: &chainMock.ChainService{Slot: &headSlot},
			ChainInfoFetcher: &chainMock.ChainService{
				CanonicalRoots: map[[32]byte]bool{
					bytesutil.ToBytes32(newBeaconState.LatestBlockHeader().ParentRoot): true,
				},
				State: newBeaconState,
			},
			ReplayerBuilder: mockstategen.NewReplayerBuilder(mockstategen.WithMockState(newBeaconState)),
		}

		s, err := p.State(ctx, []byte(strconv.FormatUint(uint64(headSlot), 10)))
		require.NoError(t, err)
		sRoot, err := s.HashTreeRoot(ctx)
		require.NoError(t, err)
		assert.Equal(t, stateRoot, sRoot)
	})

	t.Run("invalid_state", func(t *testing.T) {
		p := BeaconDbStater{}
		_, err := p.State(ctx, []byte("foo"))
		require.ErrorContains(t, "could not parse state ID", err)
	})
}

func TestGetStateRoot(t *testing.T) {
	ctx := context.Background()

	headSlot := primitives.Slot(123)
	fillSlot := func(state *ethpb.BeaconState) error {
		state.Slot = headSlot
		return nil
	}
	newBeaconState, err := util.NewBeaconState(util.FillRootsNaturalOpt, fillSlot)
	require.NoError(t, err)
	stateRoot, err := newBeaconState.HashTreeRoot(ctx)
	require.NoError(t, err)

	t.Run("head", func(t *testing.T) {
		b := util.NewBeaconBlock()
		b.Block.StateRoot = stateRoot[:]
		wsb, err := blocks.NewSignedBeaconBlock(b)
		require.NoError(t, err)
		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{
				State: newBeaconState,
				Block: wsb,
			},
		}

		s, err := p.StateRoot(ctx, []byte("head"))
		require.NoError(t, err)
		assert.DeepEqual(t, stateRoot[:], s)
	})

	t.Run("genesis", func(t *testing.T) {
		db := testDB.SetupDB(t)
		b := util.NewBeaconBlock()
		util.SaveBlock(t, ctx, db, b)
		r, err := b.Block.HashTreeRoot()
		require.NoError(t, err)

		bs, err := util.NewBeaconState(func(state *ethpb.BeaconState) error {
			state.BlockRoots[0] = r[:]
			return nil
		})
		require.NoError(t, err)

		require.NoError(t, db.SaveStateSummary(ctx, &ethpb.StateSummary{Root: r[:]}))
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, r))
		require.NoError(t, db.SaveState(ctx, bs, r))

		p := BeaconDbStater{
			BeaconDB: db,
		}

		s, err := p.StateRoot(ctx, []byte("genesis"))
		require.NoError(t, err)
		genesisBlock, err := db.GenesisBlock(ctx)
		require.NoError(t, err)
		sr := genesisBlock.Block().StateRoot()
		assert.DeepEqual(t, sr[:], s)
	})

	t.Run("finalized", func(t *testing.T) {
		db := testDB.SetupDB(t)
		genesis := bytesutil.ToBytes32([]byte("genesis"))
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesis))
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = genesis[:]
		blk.Block.Slot = 40
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		cp := &ethpb.Checkpoint{
			Epoch: 5,
			Root:  root[:],
		}
		// a valid chain is required to save finalized checkpoint.
		util.SaveBlock(t, ctx, db, blk)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(1))
		// a state is required to save checkpoint
		require.NoError(t, db.SaveState(ctx, st, root))
		require.NoError(t, db.SaveFinalizedCheckpoint(ctx, cp))

		p := BeaconDbStater{
			BeaconDB: db,
		}

		s, err := p.StateRoot(ctx, []byte("finalized"))
		require.NoError(t, err)
		assert.DeepEqual(t, blk.Block.StateRoot, s)
	})

	t.Run("justified", func(t *testing.T) {
		db := testDB.SetupDB(t)
		genesis := bytesutil.ToBytes32([]byte("genesis"))
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesis))
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = genesis[:]
		blk.Block.Slot = 40
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		cp := &ethpb.Checkpoint{
			Epoch: 5,
			Root:  root[:],
		}
		// a valid chain is required to save finalized checkpoint.
		util.SaveBlock(t, ctx, db, blk)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(1))
		// a state is required to save checkpoint
		require.NoError(t, db.SaveState(ctx, st, root))
		require.NoError(t, db.SaveJustifiedCheckpoint(ctx, cp))

		p := BeaconDbStater{
			BeaconDB: db,
		}

		s, err := p.StateRoot(ctx, []byte("justified"))
		require.NoError(t, err)
		assert.DeepEqual(t, blk.Block.StateRoot, s)
	})

	t.Run("hex_root", func(t *testing.T) {
		stateId, err := hexutil.Decode("0x" + strings.Repeat("0", 63) + "1")
		require.NoError(t, err)

		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
		}

		s, err := p.StateRoot(ctx, stateId)
		require.NoError(t, err)
		assert.DeepEqual(t, stateId, s)
	})

	t.Run("hex_root_not_found", func(t *testing.T) {
		p := BeaconDbStater{
			ChainInfoFetcher: &chainMock.ChainService{State: newBeaconState},
		}
		stateId, err := hexutil.Decode("0x" + strings.Repeat("f", 64))
		require.NoError(t, err)
		_, err = p.StateRoot(ctx, stateId)
		require.ErrorContains(t, "state root not found in the last 8192 state roots", err)
	})

	t.Run("slot", func(t *testing.T) {
		db := testDB.SetupDB(t)
		genesis := bytesutil.ToBytes32([]byte("genesis"))
		require.NoError(t, db.SaveGenesisBlockRoot(ctx, genesis))
		blk := util.NewBeaconBlock()
		blk.Block.ParentRoot = genesis[:]
		blk.Block.Slot = 40
		root, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		util.SaveBlock(t, ctx, db, blk)
		st, err := util.NewBeaconState()
		require.NoError(t, err)
		require.NoError(t, st.SetSlot(1))
		// a state is required to save checkpoint
		require.NoError(t, db.SaveState(ctx, st, root))

		slot := primitives.Slot(40)
		p := BeaconDbStater{
			GenesisTimeFetcher: &chainMock.ChainService{Slot: &slot},
			BeaconDB:           db,
		}

		s, err := p.StateRoot(ctx, []byte(strconv.FormatUint(uint64(slot), 10)))
		require.NoError(t, err)
		assert.DeepEqual(t, blk.Block.StateRoot, s)
	})

	t.Run("slot_too_big", func(t *testing.T) {
		p := BeaconDbStater{
			GenesisTimeFetcher: &chainMock.ChainService{
				Genesis: time.Now(),
			},
		}
		_, err := p.StateRoot(ctx, []byte(strconv.FormatUint(1, 10)))
		assert.ErrorContains(t, "slot cannot be in the future", err)
	})

	t.Run("invalid_state", func(t *testing.T) {
		p := BeaconDbStater{}
		_, err := p.StateRoot(ctx, []byte("foo"))
		require.ErrorContains(t, "could not parse state ID", err)
	})
}

func TestNewStateNotFoundError(t *testing.T) {
	e := NewStateNotFoundError(100)
	assert.Equal(t, "state not found in the last 100 state roots", e.message)
}

func TestStateBySlot_FutureSlot(t *testing.T) {
	slot := primitives.Slot(100)
	p := BeaconDbStater{GenesisTimeFetcher: &chainMock.ChainService{Slot: &slot}}
	_, err := p.StateBySlot(context.Background(), 101)
	assert.ErrorContains(t, "requested slot is in the future", err)
}

func TestStateBySlot_AfterHeadSlot(t *testing.T) {
	headSt, err := statenative.InitializeFromProtoPhase0(&ethpb.BeaconState{Slot: 100})
	require.NoError(t, err)
	slotSt, err := statenative.InitializeFromProtoPhase0(&ethpb.BeaconState{Slot: 101})
	require.NoError(t, err)
	currentSlot := primitives.Slot(102)
	mock := &chainMock.ChainService{State: headSt, Slot: &currentSlot}
	mockReplayer := mockstategen.NewReplayerBuilder()
	mockReplayer.SetMockStateForSlot(slotSt, 101)
	p := BeaconDbStater{ChainInfoFetcher: mock, GenesisTimeFetcher: mock, ReplayerBuilder: mockReplayer}
	st, err := p.StateBySlot(context.Background(), 101)
	require.NoError(t, err)
	assert.Equal(t, primitives.Slot(101), st.Slot())
}
