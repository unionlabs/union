package beacon

import (
	"context"
	"encoding/binary"
	"math"
	"testing"
	"time"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	mockstategen "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen/mock"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	blocktest "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks/testing"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/protobuf/proto"
	"gopkg.in/d4l3k/messagediff.v1"
)

func TestServer_ListBeaconCommittees_CurrentEpoch(t *testing.T) {
	db := dbTest.SetupDB(t)
	helpers.ClearCache()

	numValidators := 128
	ctx := context.Background()
	headState := setupActiveValidators(t, numValidators)

	offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
	m := &mock.ChainService{
		Genesis: prysmTime.Now().Add(time.Duration(-1*offset) * time.Second),
	}
	bs := &Server{
		HeadFetcher:        m,
		GenesisTimeFetcher: m,
		StateGen:           stategen.New(db, doublylinkedtree.New()),
	}
	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, db, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, gRoot))
	require.NoError(t, db.SaveState(ctx, headState, gRoot))

	bs.ReplayerBuilder = mockstategen.NewReplayerBuilder(mockstategen.WithMockState(headState))

	activeIndices, err := helpers.ActiveValidatorIndices(ctx, headState, 0)
	require.NoError(t, err)
	attesterSeed, err := helpers.Seed(headState, 0, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	committees, err := computeCommittees(context.Background(), 0, activeIndices, attesterSeed)
	require.NoError(t, err)

	wanted := &ethpb.BeaconCommittees{
		Epoch:                0,
		Committees:           committees.SlotToUint64(),
		ActiveValidatorCount: uint64(numValidators),
	}
	res, err := bs.ListBeaconCommittees(context.Background(), &ethpb.ListCommitteesRequest{
		QueryFilter: &ethpb.ListCommitteesRequest_Genesis{Genesis: true},
	})
	require.NoError(t, err)
	if !proto.Equal(res, wanted) {
		t.Errorf("Expected %v, received %v", wanted, res)
	}
}

func addDefaultReplayerBuilder(s *Server, h stategen.HistoryAccessor) {
	cc := &mockstategen.CanonicalChecker{Is: true, Err: nil}
	cs := &mockstategen.CurrentSlotter{Slot: math.MaxUint64 - 1}
	s.ReplayerBuilder = stategen.NewCanonicalHistory(h, cc, cs)
}

func TestServer_ListBeaconCommittees_PreviousEpoch(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	params.OverrideBeaconConfig(params.BeaconConfig())
	ctx := context.Background()

	db := dbTest.SetupDB(t)
	helpers.ClearCache()

	numValidators := 128
	headState := setupActiveValidators(t, numValidators)

	mixes := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(mixes); i++ {
		mixes[i] = make([]byte, fieldparams.RootLength)
	}
	require.NoError(t, headState.SetRandaoMixes(mixes))
	require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch))

	b, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
	require.NoError(t, err)
	b, err = blocktest.SetBlockSlot(b, headState.Slot())
	require.NoError(t, err)
	require.NoError(t, db.SaveBlock(ctx, b))
	gRoot, err := b.Block().HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveState(ctx, headState, gRoot))

	offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
	m := &mock.ChainService{
		State:   headState,
		Genesis: prysmTime.Now().Add(time.Duration(-1*offset) * time.Second),
	}
	bs := &Server{
		HeadFetcher:        m,
		GenesisTimeFetcher: m,
		StateGen:           stategen.New(db, doublylinkedtree.New()),
	}
	addDefaultReplayerBuilder(bs, db)

	activeIndices, err := helpers.ActiveValidatorIndices(ctx, headState, 1)
	require.NoError(t, err)
	attesterSeed, err := helpers.Seed(headState, 1, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	startSlot, err := slots.EpochStart(1)
	require.NoError(t, err)
	wanted, err := computeCommittees(context.Background(), startSlot, activeIndices, attesterSeed)
	require.NoError(t, err)

	tests := []struct {
		req *ethpb.ListCommitteesRequest
		res *ethpb.BeaconCommittees
	}{
		{
			req: &ethpb.ListCommitteesRequest{
				QueryFilter: &ethpb.ListCommitteesRequest_Epoch{Epoch: 1},
			},
			res: &ethpb.BeaconCommittees{
				Epoch:                1,
				Committees:           wanted.SlotToUint64(),
				ActiveValidatorCount: uint64(numValidators),
			},
		},
	}
	helpers.ClearCache()
	for i, test := range tests {
		res, err := bs.ListBeaconCommittees(context.Background(), test.req)
		require.NoError(t, err)
		if !proto.Equal(res, test.res) {
			diff, _ := messagediff.PrettyDiff(res, test.res)
			t.Errorf("%d/ Diff between responses %s", i, diff)
		}
	}
}

func TestRetrieveCommitteesForRoot(t *testing.T) {

	db := dbTest.SetupDB(t)
	helpers.ClearCache()
	ctx := context.Background()

	numValidators := 128
	headState := setupActiveValidators(t, numValidators)

	offset := int64(headState.Slot().Mul(params.BeaconConfig().SecondsPerSlot))
	m := &mock.ChainService{
		Genesis: prysmTime.Now().Add(time.Duration(-1*offset) * time.Second),
	}
	bs := &Server{
		HeadFetcher:        m,
		GenesisTimeFetcher: m,
		StateGen:           stategen.New(db, doublylinkedtree.New()),
	}
	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, db, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, db.SaveGenesisBlockRoot(ctx, gRoot))
	require.NoError(t, db.SaveState(ctx, headState, gRoot))
	stateSummary := &ethpb.StateSummary{
		Slot: 0,
		Root: gRoot[:],
	}
	require.NoError(t, db.SaveStateSummary(ctx, stateSummary))

	// Store the genesis seed.
	seed, err := helpers.Seed(headState, 0, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)
	require.NoError(t, headState.SetSlot(params.BeaconConfig().SlotsPerEpoch*10))

	activeIndices, err := helpers.ActiveValidatorIndices(ctx, headState, 0)
	require.NoError(t, err)

	wanted, err := computeCommittees(context.Background(), 0, activeIndices, seed)
	require.NoError(t, err)
	committees, activeIndices, err := bs.retrieveCommitteesForRoot(context.Background(), gRoot[:])
	require.NoError(t, err)

	wantedRes := &ethpb.BeaconCommittees{
		Epoch:                0,
		Committees:           wanted.SlotToUint64(),
		ActiveValidatorCount: uint64(numValidators),
	}
	receivedRes := &ethpb.BeaconCommittees{
		Epoch:                0,
		Committees:           committees.SlotToUint64(),
		ActiveValidatorCount: uint64(len(activeIndices)),
	}
	assert.DeepEqual(t, wantedRes, receivedRes)
}

func setupActiveValidators(t *testing.T, count int) state.BeaconState {
	balances := make([]uint64, count)
	validators := make([]*ethpb.Validator, 0, count)
	for i := 0; i < count; i++ {
		pubKey := make([]byte, params.BeaconConfig().BLSPubkeyLength)
		binary.LittleEndian.PutUint64(pubKey, uint64(i))
		balances[i] = uint64(i)
		validators = append(validators, &ethpb.Validator{
			PublicKey:             pubKey,
			ActivationEpoch:       0,
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			WithdrawalCredentials: make([]byte, 32),
		})
	}
	s, err := util.NewBeaconState()
	require.NoError(t, err)
	if err := s.SetValidators(validators); err != nil {
		t.Error(err)
		return nil
	}
	if err := s.SetBalances(balances); err != nil {
		t.Error(err)
		return nil
	}
	return s
}
