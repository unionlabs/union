package stategen

import (
	"context"
	"testing"

	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestSaveState_HotStateCanBeSaved(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	// This goes to hot section, verify it can save on epoch boundary.
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))

	r := [32]byte{'a'}
	require.NoError(t, service.SaveState(ctx, r, beaconState))

	// Should save both state and state summary.
	_, ok, err := service.epochBoundaryStateCache.getByBlockRoot(r)
	require.NoError(t, err)
	assert.Equal(t, true, ok, "Should have saved the state")
	assert.Equal(t, true, service.beaconDB.HasStateSummary(ctx, r), "Should have saved the state summary")
}

func TestSaveState_HotStateCached(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	service := New(beaconDB, doublylinkedtree.New())
	service.slotsPerArchivedPoint = 1
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))

	// Cache the state prior.
	r := [32]byte{'a'}
	service.hotStateCache.put(r, beaconState)
	require.NoError(t, service.SaveState(ctx, r, beaconState))

	// Should not save the state and state summary.
	assert.Equal(t, false, service.beaconDB.HasState(ctx, r), "Should not have saved the state")
	assert.Equal(t, false, service.beaconDB.HasStateSummary(ctx, r), "Should have saved the state summary")
	require.LogsDoNotContain(t, hook, "Saved full state on epoch boundary")
}

func TestState_ForceCheckpoint_SavesStateToDatabase(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)

	svc := New(beaconDB, doublylinkedtree.New())
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))

	r := [32]byte{'a'}
	svc.hotStateCache.put(r, beaconState)

	require.Equal(t, false, beaconDB.HasState(ctx, r), "Database has state stored already")
	assert.NoError(t, svc.ForceCheckpoint(ctx, r[:]))
	assert.Equal(t, true, beaconDB.HasState(ctx, r), "Did not save checkpoint to database")

	// Should not panic with genesis finalized root.
	assert.NoError(t, svc.ForceCheckpoint(ctx, params.BeaconConfig().ZeroHash[:]))
}

func TestSaveState_Alreadyhas(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))
	r := [32]byte{'A'}

	// Pre cache the hot state.
	service.hotStateCache.put(r, beaconState)
	require.NoError(t, service.saveStateByRoot(ctx, r, beaconState))

	// Should not save the state and state summary.
	assert.Equal(t, false, service.beaconDB.HasState(ctx, r), "Should not have saved the state")
	assert.Equal(t, false, service.beaconDB.HasStateSummary(ctx, r), "Should have saved the state summary")
	require.LogsDoNotContain(t, hook, "Saved full state on epoch boundary")
}

func TestSaveState_CanSaveOnEpochBoundary(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch))
	r := [32]byte{'A'}

	require.NoError(t, service.saveStateByRoot(ctx, r, beaconState))

	// Should save both state and state summary.
	_, ok, err := service.epochBoundaryStateCache.getByBlockRoot(r)
	require.NoError(t, err)
	require.Equal(t, true, ok, "Did not save epoch boundary state")
	assert.Equal(t, true, service.beaconDB.HasStateSummary(ctx, r), "Should have saved the state summary")
	// Should have not been saved in DB.
	require.Equal(t, false, beaconDB.HasState(ctx, r))
}

func TestSaveState_NoSaveNotEpochBoundary(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch-1))
	r := [32]byte{'A'}
	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, beaconDB, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, gRoot))
	require.NoError(t, service.SaveState(ctx, r, beaconState))

	// Should only save state summary.
	assert.Equal(t, false, service.beaconDB.HasState(ctx, r), "Should not have saved the state")
	assert.Equal(t, true, service.beaconDB.HasStateSummary(ctx, r), "Should have saved the state summary")
	require.LogsDoNotContain(t, hook, "Saved full state on epoch boundary")
	// Should have not been saved in DB.
	require.Equal(t, false, beaconDB.HasState(ctx, r))
}

func TestSaveState_RecoverForEpochBoundary(t *testing.T) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(params.BeaconConfig().SlotsPerEpoch-1))
	r := [32]byte{'A'}
	boundaryRoot := [32]byte{'B'}
	require.NoError(t, beaconState.UpdateBlockRootAtIndex(0, boundaryRoot))

	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, beaconDB, b)
	gRoot, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	require.NoError(t, beaconDB.SaveGenesisBlockRoot(ctx, gRoot))
	// Save boundary state to the hot state cache.
	boundaryState, _ := util.DeterministicGenesisState(t, 32)
	service.hotStateCache.put(boundaryRoot, boundaryState)
	require.NoError(t, service.SaveState(ctx, r, beaconState))

	rInfo, ok, err := service.epochBoundaryStateCache.getByBlockRoot(boundaryRoot)
	assert.NoError(t, err)
	assert.Equal(t, true, ok, "state does not exist in cache")
	assert.Equal(t, rInfo.root, boundaryRoot, "incorrect root of root state info")
	assert.Equal(t, rInfo.state.Slot(), primitives.Slot(0), "incorrect slot of state")
}

func TestSaveState_CanSaveHotStateToDB(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())
	service.EnableSaveHotStateToDB(ctx)
	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(defaultHotStateDBInterval))

	r := [32]byte{'A'}
	require.NoError(t, service.saveStateByRoot(ctx, r, beaconState))

	require.LogsContain(t, hook, "Saving hot state to DB")
	// Should have saved in DB.
	require.Equal(t, true, beaconDB.HasState(ctx, r))
}

func TestEnableSaveHotStateToDB_Enabled(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())

	service.EnableSaveHotStateToDB(ctx)
	require.LogsContain(t, hook, "Entering mode to save hot states in DB")
	require.Equal(t, true, service.saveHotStateDB.enabled)
}

func TestEnableSaveHotStateToDB_AlreadyEnabled(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())
	service.saveHotStateDB.enabled = true
	service.EnableSaveHotStateToDB(ctx)
	require.LogsDoNotContain(t, hook, "Entering mode to save hot states in DB")
	require.Equal(t, true, service.saveHotStateDB.enabled)
}

func TestEnableSaveHotStateToDB_Disabled(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())
	service.saveHotStateDB.enabled = true
	b := util.NewBeaconBlock()
	util.SaveBlock(t, ctx, beaconDB, b)
	r, err := b.Block.HashTreeRoot()
	require.NoError(t, err)
	service.saveHotStateDB.blockRootsOfSavedStates = [][32]byte{r}
	require.NoError(t, service.DisableSaveHotStateToDB(ctx))
	require.LogsContain(t, hook, "Exiting mode to save hot states in DB")
	require.Equal(t, false, service.saveHotStateDB.enabled)
	require.Equal(t, 0, len(service.saveHotStateDB.blockRootsOfSavedStates))
}

func TestEnableSaveHotStateToDB_AlreadyDisabled(t *testing.T) {
	hook := logTest.NewGlobal()
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	service := New(beaconDB, doublylinkedtree.New())
	require.NoError(t, service.DisableSaveHotStateToDB(ctx))
	require.LogsDoNotContain(t, hook, "Exiting mode to save hot states in DB")
	require.Equal(t, false, service.saveHotStateDB.enabled)
}
