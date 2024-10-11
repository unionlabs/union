package stategen

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	consensusblocks "github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestMigrateToCold_CanSaveFinalizedInfo(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	b := util.NewBeaconBlock()
	b.Block.Slot = 1
	br, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b)
	require.NoError(t, service.epochBoundaryStateCache.put(br, beaconState))
	require.NoError(t, service.MigrateToCold(ctx, br))

	wanted := &finalizedInfo{state: beaconState, root: br, slot: 1}
	assert.DeepEqual(t, wanted.root, service.finalizedInfo.root)
	assert.Equal(t, wanted.slot, service.finalizedInfo.slot)
	expectedHTR, err := wanted.state.HashTreeRoot(ctx)
	require.NoError(t, err)
	actualHTR, err := service.finalizedInfo.state.HashTreeRoot(ctx)
	require.NoError(t, err)
	assert.DeepEqual(t, expectedHTR, actualHTR)
}

func TestMigrateToCold_HappyPath(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	stateSlot := primitives.Slot(1)
	require.NoError(t, beaconState.SetSlot(stateSlot))
	b := util.NewBeaconBlock()
	b.Block.Slot = 2
	fRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b)
	require.NoError(t, service.epochBoundaryStateCache.put(fRoot, beaconState))
	require.NoError(t, service.MigrateToCold(ctx, fRoot))

	gotState, err := service.beaconDB.State(ctx, fRoot)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, beaconState.ToProtoUnsafe(), gotState.ToProtoUnsafe(), "Did not save state")
	gotRoot := service.beaconDB.ArchivedPointRoot(ctx, stateSlot/service.slotsPerArchivedPoint)
	assert.Equal(t, fRoot, gotRoot, "Did not save archived root")
	lastIndex, err := service.beaconDB.LastArchivedSlot(ctx)
	require.NoError(t, err)
	assert.Equal(t, primitives.Slot(1), lastIndex, "Did not save last archived index")

	require.LogsContain(t, hook, "Saved state in DB")
}

func TestMigrateToCold_RegeneratePath(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, pks := util.DeterministicGenesisState(t, 32)
	genesisStateRoot, err := beaconState.HashTreeRoot(ctx)
	require.NoError(t, err)
	genesis := blocks.NewGenesisBlock(genesisStateRoot[:])
	util.SaveBlock(t, ctx, beaconDB, genesis)
	gRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.NoError(t, beaconDB.SaveState(ctx, beaconState, gRoot))
	assert.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, gRoot))

	b1, err := util.GenerateFullBlock(beaconState, pks, util.DefaultBlockGenConfig(), 1)
	require.NoError(t, err)
	r1, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b1)
	require.NoError(t, service.beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Slot: 1, Root: r1[:]}))

	b4, err := util.GenerateFullBlock(beaconState, pks, util.DefaultBlockGenConfig(), 4)
	require.NoError(t, err)
	r4, err := b4.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b4)
	require.NoError(t, service.beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Slot: 4, Root: r4[:]}))
	service.finalizedInfo = &finalizedInfo{
		slot:  0,
		root:  genesisStateRoot,
		state: beaconState,
	}

	require.NoError(t, service.MigrateToCold(ctx, r4))

	s1, err := service.beaconDB.State(ctx, r1)
	require.NoError(t, err)
	assert.Equal(t, s1.Slot(), primitives.Slot(1), "Did not save state")
	gotRoot := service.beaconDB.ArchivedPointRoot(ctx, 1/service.slotsPerArchivedPoint)
	assert.Equal(t, r1, gotRoot, "Did not save archived root")
	lastIndex, err := service.beaconDB.LastArchivedSlot(ctx)
	require.NoError(t, err)
	assert.Equal(t, primitives.Slot(1), lastIndex, "Did not save last archived index")

	require.LogsContain(t, hook, "Saved state in DB")
}

func TestMigrateToCold_StateExistsInDB(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	stateSlot := primitives.Slot(1)
	require.NoError(t, beaconState.SetSlot(stateSlot))
	b := util.NewBeaconBlock()
	b.Block.Slot = 2
	fRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b)
	require.NoError(t, service.epochBoundaryStateCache.put(fRoot, beaconState))
	require.NoError(t, service.beaconDB.SaveState(ctx, beaconState, fRoot))

	service.saveHotStateDB.blockRootsOfSavedStates = [][32]byte{{1}, {2}, {3}, {4}, fRoot}
	require.NoError(t, service.MigrateToCold(ctx, fRoot))
	assert.DeepEqual(t, [][32]byte{{1}, {2}, {3}, {4}}, service.saveHotStateDB.blockRootsOfSavedStates)
	assert.LogsDoNotContain(t, hook, "Saved state in DB")
}

func TestMigrateToCold_ParallelCalls(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, pks := util.DeterministicGenesisState(t, 32)
	genState := beaconState.Copy()
	genesisStateRoot, err := beaconState.HashTreeRoot(ctx)
	require.NoError(t, err)
	genesis := blocks.NewGenesisBlock(genesisStateRoot[:])
	util.SaveBlock(t, ctx, beaconDB, genesis)
	gRoot, err := genesis.Block.HashTreeRoot()
	require.NoError(t, err)
	assert.NoError(t, beaconDB.SaveState(ctx, beaconState, gRoot))
	assert.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, gRoot))

	b1, err := util.GenerateFullBlock(beaconState, pks, util.DefaultBlockGenConfig(), 1)
	require.NoError(t, err)
	wB1, err := consensusblocks.NewSignedBeaconBlock(b1)
	require.NoError(t, err)
	beaconState, err = executeStateTransitionStateGen(ctx, beaconState, wB1)
	assert.NoError(t, err)
	r1, err := b1.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b1)
	require.NoError(t, service.beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Slot: 1, Root: r1[:]}))

	b4, err := util.GenerateFullBlock(beaconState, pks, util.DefaultBlockGenConfig(), 4)
	require.NoError(t, err)
	wB4, err := consensusblocks.NewSignedBeaconBlock(b4)
	require.NoError(t, err)
	beaconState, err = executeStateTransitionStateGen(ctx, beaconState, wB4)
	assert.NoError(t, err)
	r4, err := b4.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b4)
	require.NoError(t, service.beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Slot: 4, Root: r4[:]}))

	b7, err := util.GenerateFullBlock(beaconState, pks, util.DefaultBlockGenConfig(), 7)
	require.NoError(t, err)
	wB7, err := consensusblocks.NewSignedBeaconBlock(b7)
	require.NoError(t, err)
	_, err = executeStateTransitionStateGen(ctx, beaconState, wB7)
	assert.NoError(t, err)
	r7, err := b7.Block.HashTreeRoot()
	require.NoError(t, err)
	util.SaveBlock(t, ctx, service.beaconDB, b7)
	require.NoError(t, service.beaconDB.SaveStateSummary(ctx, &ethpb.StateSummary{Slot: 7, Root: r7[:]}))

	service.finalizedInfo = &finalizedInfo{
		slot:  0,
		root:  genesisStateRoot,
		state: genState,
	}
	service.saveHotStateDB.blockRootsOfSavedStates = [][32]byte{r1, r4, r7}

	// Run the migration routines concurrently for 2 different finalized roots.
	go func() {
		require.NoError(t, service.MigrateToCold(ctx, r4))
	}()

	require.NoError(t, service.MigrateToCold(ctx, r7))

	s1, err := service.beaconDB.State(ctx, r1)
	require.NoError(t, err)
	assert.Equal(t, s1.Slot(), primitives.Slot(1), "Did not save state")
	s4, err := service.beaconDB.State(ctx, r4)
	require.NoError(t, err)
	assert.Equal(t, s4.Slot(), primitives.Slot(4), "Did not save state")

	gotRoot := service.beaconDB.ArchivedPointRoot(ctx, 1/service.slotsPerArchivedPoint)
	assert.Equal(t, r1, gotRoot, "Did not save archived root")
	gotRoot = service.beaconDB.ArchivedPointRoot(ctx, 4)
	assert.Equal(t, r4, gotRoot, "Did not save archived root")
	lastIndex, err := service.beaconDB.LastArchivedSlot(ctx)
	require.NoError(t, err)
	assert.Equal(t, primitives.Slot(4), lastIndex, "Did not save last archived index")
	assert.DeepEqual(t, [][32]byte{r7}, service.saveHotStateDB.blockRootsOfSavedStates, "Did not remove all saved hot state roots")
	require.LogsContain(t, hook, "Saved state in DB")
}
