package debug

import (
	"context"
	"math"
	"testing"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockstategen "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen/mock"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func addDefaultReplayerBuilder(s *Server, h stategen.HistoryAccessor) {
	cc := &mockstategen.CanonicalChecker{Is: true}
	cs := &mockstategen.CurrentSlotter{Slot: math.MaxUint64 - 1}
	s.ReplayerBuilder = stategen.NewCanonicalHistory(h, cc, cs)
}

func TestServer_GetBeaconState(t *testing.T) {
	db := dbTest.SetupDB(t)
	ctx := context.Background()
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	slot := primitives.Slot(100)
	require.NoError(t, st.SetSlot(slot))
	b := util.NewBeaconBlock()
	b.Block.Slot = slot
	util.SaveBlock(t, ctx, db, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	gen := stategen.New(db, doublylinkedtree.New())
	require.NoError(t, gen.SaveState(ctx, gRoot, st))
	require.NoError(t, db.SaveState(ctx, st, gRoot))
	bs := &Server{
		StateGen:           gen,
		GenesisTimeFetcher: &mock.ChainService{},
	}
	addDefaultReplayerBuilder(bs, db)
	_, err = bs.GetBeaconState(ctx, &pbrpc.BeaconStateRequest{})
	assert.ErrorContains(t, "Need to specify either a block root or slot to request state", err)
	req := &pbrpc.BeaconStateRequest{
		QueryFilter: &pbrpc.BeaconStateRequest_BlockRoot{
			BlockRoot: gRoot[:],
		},
	}
	res, err := bs.GetBeaconState(ctx, req)
	require.NoError(t, err)
	wanted, err := st.MarshalSSZ()
	require.NoError(t, err)
	assert.DeepEqual(t, wanted, res.Encoded)

	req = &pbrpc.BeaconStateRequest{
		QueryFilter: &pbrpc.BeaconStateRequest_Slot{
			Slot: st.Slot(),
		},
	}
	wanted, err = st.MarshalSSZ()
	require.NoError(t, err)
	res, err = bs.GetBeaconState(ctx, req)
	require.NoError(t, err)
	resState := &pbrpc.BeaconState{}
	err = resState.UnmarshalSSZ(res.Encoded)
	require.NoError(t, err)
	assert.Equal(t, resState.Slot, st.Slot())
	assert.DeepEqual(t, wanted, res.Encoded)

	// request a slot after the state
	// note that if the current slot were <= slot+1, this would fail
	// but the mock stategen.CurrentSlotter gives a current slot far in the future
	// so this acts like requesting a state at a skipped slot
	req = &pbrpc.BeaconStateRequest{
		QueryFilter: &pbrpc.BeaconStateRequest_Slot{
			Slot: slot + 1,
		},
	}
	// since we are requesting a state at a skipped slot, use the same method as stategen
	// to advance to the pre-state for the subsequent slot
	state, err := stategen.ReplayProcessSlots(ctx, st, slot+1)
	require.NoError(t, err)
	wanted, err = state.MarshalSSZ()
	require.NoError(t, err)
	res, err = bs.GetBeaconState(ctx, req)
	require.NoError(t, err)
	resState = &pbrpc.BeaconState{}
	err = resState.UnmarshalSSZ(res.Encoded)
	require.NoError(t, err)
	assert.Equal(t, resState.Slot, state.Slot())
	assert.DeepEqual(t, wanted, res.Encoded)
}

func TestServer_GetBeaconState_RequestFutureSlot(t *testing.T) {
	ds := &Server{GenesisTimeFetcher: &mock.ChainService{}}
	req := &pbrpc.BeaconStateRequest{
		QueryFilter: &pbrpc.BeaconStateRequest_Slot{
			Slot: ds.GenesisTimeFetcher.CurrentSlot() + 1,
		},
	}
	wanted := "Cannot retrieve information about a slot in the future"
	_, err := ds.GetBeaconState(context.Background(), req)
	assert.ErrorContains(t, wanted, err)
}
