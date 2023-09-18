package rootmulti

import (
	"bytes"
	"fmt"
	"testing"
	"time"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	codecTypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/store/cachemulti"
	"github.com/cosmos/cosmos-sdk/store/iavl"
	sdkmaps "github.com/cosmos/cosmos-sdk/store/internal/maps"
	"github.com/cosmos/cosmos-sdk/store/listenkv"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/store/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

func TestStoreType(t *testing.T) {
	db := dbm.NewMemDB()
	store := NewStore(db, log.NewNopLogger())
	store.MountStoreWithDB(types.NewKVStoreKey("store1"), types.StoreTypeIAVL, db)
}

func TestGetCommitKVStore(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningDefault))
	err := ms.LoadLatestVersion()
	require.Nil(t, err)

	key := ms.keysByName["store1"]

	store1 := ms.GetCommitKVStore(key)
	require.NotNil(t, store1)
	require.IsType(t, &iavl.Store{}, store1)

	store2 := ms.GetCommitStore(key)
	require.NotNil(t, store2)
	require.IsType(t, &iavl.Store{}, store2)
}

func TestStoreMount(t *testing.T) {
	db := dbm.NewMemDB()
	store := NewStore(db, log.NewNopLogger())

	key1 := types.NewKVStoreKey("store1")
	key2 := types.NewKVStoreKey("store2")
	dup1 := types.NewKVStoreKey("store1")

	require.NotPanics(t, func() { store.MountStoreWithDB(key1, types.StoreTypeIAVL, db) })
	require.NotPanics(t, func() { store.MountStoreWithDB(key2, types.StoreTypeIAVL, db) })

	require.Panics(t, func() { store.MountStoreWithDB(key1, types.StoreTypeIAVL, db) })
	require.Panics(t, func() { store.MountStoreWithDB(nil, types.StoreTypeIAVL, db) })
	require.Panics(t, func() { store.MountStoreWithDB(dup1, types.StoreTypeIAVL, db) })
}

func TestCacheMultiStore(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	cacheMulti := ms.CacheMultiStore()
	require.IsType(t, cachemulti.Store{}, cacheMulti)
}

func TestCacheMultiStoreWithVersion(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := ms.LoadLatestVersion()
	require.Nil(t, err)

	commitID := types.CommitID{}
	checkStore(t, ms, commitID, commitID)

	k, v := []byte("wind"), []byte("blows")

	store1 := ms.GetStoreByName("store1").(types.KVStore)
	store1.Set(k, v)

	cID := ms.Commit()
	require.Equal(t, int64(1), cID.Version)

	// require no failure when given an invalid or pruned version
	_, err = ms.CacheMultiStoreWithVersion(cID.Version + 1)
	require.Error(t, err)

	// require a valid version can be cache-loaded
	cms, err := ms.CacheMultiStoreWithVersion(cID.Version)
	require.NoError(t, err)

	// require a valid key lookup yields the correct value
	kvStore := cms.GetKVStore(ms.keysByName["store1"])
	require.NotNil(t, kvStore)
	require.Equal(t, kvStore.Get(k), v)

	// add new module stores (store4 and store5) to multi stores and commit
	ms.MountStoreWithDB(types.NewKVStoreKey("store4"), types.StoreTypeIAVL, nil)
	ms.MountStoreWithDB(types.NewKVStoreKey("store5"), types.StoreTypeIAVL, nil)
	err = ms.LoadLatestVersionAndUpgrade(&types.StoreUpgrades{Added: []string{"store4", "store5"}})
	require.NoError(t, err)
	ms.Commit()

	// cache multistore of version before adding store4 should works
	_, err = ms.CacheMultiStoreWithVersion(1)
	require.NoError(t, err)

	// require we cannot commit (write) to a cache-versioned multi-store
	require.Panics(t, func() {
		kvStore.Set(k, []byte("newValue"))
		cms.Write()
	})
}

func TestHashStableWithEmptyCommit(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := ms.LoadLatestVersion()
	require.Nil(t, err)

	commitID := types.CommitID{}
	checkStore(t, ms, commitID, commitID)

	k, v := []byte("wind"), []byte("blows")

	store1 := ms.GetStoreByName("store1").(types.KVStore)
	store1.Set(k, v)

	cID := ms.Commit()
	require.Equal(t, int64(1), cID.Version)
	hash := cID.Hash

	// make an empty commit, it should update version, but not affect hash
	cID = ms.Commit()
	require.Equal(t, int64(2), cID.Version)
	require.Equal(t, hash, cID.Hash)
}

func TestMultistoreCommitLoad(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	store := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := store.LoadLatestVersion()
	require.Nil(t, err)

	// New store has empty last commit.
	commitID := types.CommitID{}
	checkStore(t, store, commitID, commitID)

	// Make sure we can get stores by name.
	s1 := store.GetStoreByName("store1")
	require.NotNil(t, s1)
	s3 := store.GetStoreByName("store3")
	require.NotNil(t, s3)
	s77 := store.GetStoreByName("store77")
	require.Nil(t, s77)

	// Make a few commits and check them.
	nCommits := int64(3)
	for i := int64(0); i < nCommits; i++ {
		commitID = store.Commit()
		expectedCommitID := getExpectedCommitID(store, i+1)
		checkStore(t, store, expectedCommitID, commitID)
	}

	// Load the latest multistore again and check version.
	store = newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err = store.LoadLatestVersion()
	require.Nil(t, err)
	commitID = getExpectedCommitID(store, nCommits)
	checkStore(t, store, commitID, commitID)

	// Commit and check version.
	commitID = store.Commit()
	expectedCommitID := getExpectedCommitID(store, nCommits+1)
	checkStore(t, store, expectedCommitID, commitID)

	// Load an older multistore and check version.
	ver := nCommits - 1
	store = newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err = store.LoadVersion(ver)
	require.Nil(t, err)
	commitID = getExpectedCommitID(store, ver)
	checkStore(t, store, commitID, commitID)
}

func TestMultistoreLoadWithUpgrade(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	store := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := store.LoadLatestVersion()
	require.Nil(t, err)

	// write some data in all stores
	k1, v1 := []byte("first"), []byte("store")
	s1, _ := store.GetStoreByName("store1").(types.KVStore)
	require.NotNil(t, s1)
	s1.Set(k1, v1)

	k2, v2 := []byte("second"), []byte("restore")
	s2, _ := store.GetStoreByName("store2").(types.KVStore)
	require.NotNil(t, s2)
	s2.Set(k2, v2)

	k3, v3 := []byte("third"), []byte("dropped")
	s3, _ := store.GetStoreByName("store3").(types.KVStore)
	require.NotNil(t, s3)
	s3.Set(k3, v3)

	s4, _ := store.GetStoreByName("store4").(types.KVStore)
	require.Nil(t, s4)

	// do one commit
	commitID := store.Commit()
	expectedCommitID := getExpectedCommitID(store, 1)
	checkStore(t, store, expectedCommitID, commitID)

	ci, err := store.GetCommitInfo(1)
	require.NoError(t, err)
	require.Equal(t, int64(1), ci.Version)
	require.Equal(t, 3, len(ci.StoreInfos))
	checkContains(t, ci.StoreInfos, []string{"store1", "store2", "store3"})

	// Load without changes and make sure it is sensible
	store = newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	err = store.LoadLatestVersion()
	require.Nil(t, err)
	commitID = getExpectedCommitID(store, 1)
	checkStore(t, store, commitID, commitID)

	// let's query data to see it was saved properly
	s2, _ = store.GetStoreByName("store2").(types.KVStore)
	require.NotNil(t, s2)
	require.Equal(t, v2, s2.Get(k2))

	// now, let's load with upgrades...
	restore, upgrades := newMultiStoreWithModifiedMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err = restore.LoadLatestVersionAndUpgrade(upgrades)
	require.Nil(t, err)

	// s1 was not changed
	s1, _ = restore.GetStoreByName("store1").(types.KVStore)
	require.NotNil(t, s1)
	require.Equal(t, v1, s1.Get(k1))

	// store3 is mounted, but data deleted are gone
	s3, _ = restore.GetStoreByName("store3").(types.KVStore)
	require.NotNil(t, s3)
	require.Nil(t, s3.Get(k3)) // data was deleted

	// store4 is mounted, with empty data
	s4, _ = restore.GetStoreByName("store4").(types.KVStore)
	require.NotNil(t, s4)

	iterator := s4.Iterator(nil, nil)

	values := 0
	for ; iterator.Valid(); iterator.Next() {
		values += 1
	}
	require.Zero(t, values)

	require.NoError(t, iterator.Close())

	// write something inside store4
	k4, v4 := []byte("fourth"), []byte("created")
	s4.Set(k4, v4)

	// store2 is no longer mounted
	st2 := restore.GetStoreByName("store2")
	require.Nil(t, st2)

	// restore2 has the old data
	rs2, _ := restore.GetStoreByName("restore2").(types.KVStore)
	require.NotNil(t, rs2)
	require.Equal(t, v2, rs2.Get(k2))

	// store this migrated data, and load it again without migrations
	migratedID := restore.Commit()
	require.Equal(t, migratedID.Version, int64(2))

	reload, _ := newMultiStoreWithModifiedMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	// unmount store3 since store3 was deleted
	unmountStore(reload, "store3")

	rs3, _ := reload.GetStoreByName("store3").(types.KVStore)
	require.Nil(t, rs3)

	err = reload.LoadLatestVersion()
	require.Nil(t, err)
	require.Equal(t, migratedID, reload.LastCommitID())

	// query this new store
	rl1, _ := reload.GetStoreByName("store1").(types.KVStore)
	require.NotNil(t, rl1)
	require.Equal(t, v1, rl1.Get(k1))

	rl2, _ := reload.GetStoreByName("restore2").(types.KVStore)
	require.NotNil(t, rl2)
	require.Equal(t, v2, rl2.Get(k2))

	rl4, _ := reload.GetStoreByName("store4").(types.KVStore)
	require.NotNil(t, rl4)
	require.Equal(t, v4, rl4.Get(k4))

	// check commitInfo in storage
	ci, err = reload.GetCommitInfo(2)
	require.NoError(t, err)
	require.Equal(t, int64(2), ci.Version)
	require.Equal(t, 3, len(ci.StoreInfos), ci.StoreInfos)
	checkContains(t, ci.StoreInfos, []string{"store1", "restore2", "store4"})
}

func TestParsePath(t *testing.T) {
	_, _, err := parsePath("foo")
	require.Error(t, err)

	store, subpath, err := parsePath("/foo")
	require.NoError(t, err)
	require.Equal(t, store, "foo")
	require.Equal(t, subpath, "")

	store, subpath, err = parsePath("/fizz/bang/baz")
	require.NoError(t, err)
	require.Equal(t, store, "fizz")
	require.Equal(t, subpath, "/bang/baz")

	substore, subsubpath, err := parsePath(subpath)
	require.NoError(t, err)
	require.Equal(t, substore, "bang")
	require.Equal(t, subsubpath, "/baz")
}

func TestMultiStoreRestart(t *testing.T) {
	db := dbm.NewMemDB()
	pruning := pruningtypes.NewCustomPruningOptions(2, 1)
	multi := newMultiStoreWithMounts(db, pruning)
	err := multi.LoadLatestVersion()
	require.Nil(t, err)

	initCid := multi.LastCommitID()

	k, v := "wind", "blows"
	k2, v2 := "water", "flows"
	k3, v3 := "fire", "burns"

	for i := 1; i < 3; i++ {
		// Set and commit data in one store.
		store1 := multi.GetStoreByName("store1").(types.KVStore)
		store1.Set([]byte(k), []byte(fmt.Sprintf("%s:%d", v, i)))

		// ... and another.
		store2 := multi.GetStoreByName("store2").(types.KVStore)
		store2.Set([]byte(k2), []byte(fmt.Sprintf("%s:%d", v2, i)))

		// ... and another.
		store3 := multi.GetStoreByName("store3").(types.KVStore)
		store3.Set([]byte(k3), []byte(fmt.Sprintf("%s:%d", v3, i)))

		multi.Commit()

		cinfo, err := multi.GetCommitInfo(int64(i))
		require.NoError(t, err)
		require.Equal(t, int64(i), cinfo.Version)
	}

	// Set and commit data in one store.
	store1 := multi.GetStoreByName("store1").(types.KVStore)
	store1.Set([]byte(k), []byte(fmt.Sprintf("%s:%d", v, 3)))

	// ... and another.
	store2 := multi.GetStoreByName("store2").(types.KVStore)
	store2.Set([]byte(k2), []byte(fmt.Sprintf("%s:%d", v2, 3)))

	multi.Commit()

	flushedCinfo, err := multi.GetCommitInfo(3)
	require.Nil(t, err)
	require.NotEqual(t, initCid, flushedCinfo, "CID is different after flush to disk")

	// ... and another.
	store3 := multi.GetStoreByName("store3").(types.KVStore)
	store3.Set([]byte(k3), []byte(fmt.Sprintf("%s:%d", v3, 3)))

	multi.Commit()

	postFlushCinfo, err := multi.GetCommitInfo(4)
	require.NoError(t, err)
	require.Equal(t, int64(4), postFlushCinfo.Version, "Commit changed after in-memory commit")

	multi = newMultiStoreWithMounts(db, pruning)
	err = multi.LoadLatestVersion()
	require.Nil(t, err)

	reloadedCid := multi.LastCommitID()
	require.Equal(t, int64(4), reloadedCid.Version, "Reloaded CID is not the same as last flushed CID")

	// Check that store1 and store2 retained date from 3rd commit
	store1 = multi.GetStoreByName("store1").(types.KVStore)
	val := store1.Get([]byte(k))
	require.Equal(t, []byte(fmt.Sprintf("%s:%d", v, 3)), val, "Reloaded value not the same as last flushed value")

	store2 = multi.GetStoreByName("store2").(types.KVStore)
	val2 := store2.Get([]byte(k2))
	require.Equal(t, []byte(fmt.Sprintf("%s:%d", v2, 3)), val2, "Reloaded value not the same as last flushed value")

	// Check that store3 still has data from last commit even though update happened on 2nd commit
	store3 = multi.GetStoreByName("store3").(types.KVStore)
	val3 := store3.Get([]byte(k3))
	require.Equal(t, []byte(fmt.Sprintf("%s:%d", v3, 3)), val3, "Reloaded value not the same as last flushed value")
}

func TestMultiStoreQuery(t *testing.T) {
	db := dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := multi.LoadLatestVersion()
	require.Nil(t, err)

	k, v := []byte("wind"), []byte("blows")
	k2, v2 := []byte("water"), []byte("flows")
	// v3 := []byte("is cold")

	cid := multi.Commit()

	// Make sure we can get by name.
	garbage := multi.GetStoreByName("bad-name")
	require.Nil(t, garbage)

	// Set and commit data in one store.
	store1 := multi.GetStoreByName("store1").(types.KVStore)
	store1.Set(k, v)

	// ... and another.
	store2 := multi.GetStoreByName("store2").(types.KVStore)
	store2.Set(k2, v2)

	// Commit the multistore.
	cid = multi.Commit()
	ver := cid.Version

	// Reload multistore from database
	multi = newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err = multi.LoadLatestVersion()
	require.Nil(t, err)

	// Test bad path.
	query := abci.RequestQuery{Path: "/key", Data: k, Height: ver}
	qres := multi.Query(query)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), qres.Code)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), qres.Codespace)

	query.Path = "h897fy32890rf63296r92"
	qres = multi.Query(query)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), qres.Code)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), qres.Codespace)

	// Test invalid store name.
	query.Path = "/garbage/key"
	qres = multi.Query(query)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.ABCICode(), qres.Code)
	require.EqualValues(t, sdkerrors.ErrUnknownRequest.Codespace(), qres.Codespace)

	// Test valid query with data.
	query.Path = "/store1/key"
	qres = multi.Query(query)
	require.EqualValues(t, 0, qres.Code)
	require.Equal(t, v, qres.Value)

	// Test valid but empty query.
	query.Path = "/store2/key"
	query.Prove = true
	qres = multi.Query(query)
	require.EqualValues(t, 0, qres.Code)
	require.Nil(t, qres.Value)

	// Test store2 data.
	query.Data = k2
	qres = multi.Query(query)
	require.EqualValues(t, 0, qres.Code)
	require.Equal(t, v2, qres.Value)
}

func TestMultiStore_Pruning(t *testing.T) {
	testCases := []struct {
		name        string
		numVersions int64
		po          pruningtypes.PruningOptions
		deleted     []int64
		saved       []int64
	}{
		{"prune nothing", 10, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing), nil, []int64{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}},
		{"prune everything", 12, pruningtypes.NewPruningOptions(pruningtypes.PruningEverything), []int64{1, 2, 3, 4, 5, 6, 7}, []int64{8, 9, 10, 11, 12}},
		{"prune some; no batch", 10, pruningtypes.NewCustomPruningOptions(2, 1), []int64{1, 2, 3, 4, 6, 5, 7}, []int64{8, 9, 10}},
		{"prune some; small batch", 10, pruningtypes.NewCustomPruningOptions(2, 3), []int64{1, 2, 3, 4, 5, 6}, []int64{7, 8, 9, 10}},
		{"prune some; large batch", 10, pruningtypes.NewCustomPruningOptions(2, 11), nil, []int64{1, 2, 3, 4, 5, 6, 7, 8, 9, 10}},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			db := dbm.NewMemDB()
			ms := newMultiStoreWithMounts(db, tc.po)
			require.NoError(t, ms.LoadLatestVersion())

			for i := int64(0); i < tc.numVersions; i++ {
				ms.Commit()
			}

			for _, v := range tc.saved {
				_, err := ms.CacheMultiStoreWithVersion(v)
				require.NoError(t, err, "expected no error when loading height: %d", v)
			}

			for _, v := range tc.deleted {
				_, err := ms.CacheMultiStoreWithVersion(v)
				require.Error(t, err, "expected error when loading height: %d", v)
			}
		})
	}
}

func TestMultiStore_Pruning_SameHeightsTwice(t *testing.T) {
	const (
		numVersions int64  = 10
		keepRecent  uint64 = 2
		interval    uint64 = 10
	)

	expectedHeights := []int64{}
	for i := int64(1); i < numVersions-int64(keepRecent); i++ {
		expectedHeights = append(expectedHeights, i)
	}

	db := dbm.NewMemDB()

	ms := newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(keepRecent, interval))
	require.NoError(t, ms.LoadLatestVersion())

	var lastCommitInfo types.CommitID
	for i := int64(0); i < numVersions; i++ {
		lastCommitInfo = ms.Commit()
	}

	require.Equal(t, numVersions, lastCommitInfo.Version)

	for v := int64(1); v < numVersions-int64(keepRecent); v++ {
		err := ms.LoadVersion(v)
		require.Error(t, err, "expected error when loading pruned height: %d", v)
	}

	for v := int64(numVersions - int64(keepRecent)); v < numVersions; v++ {
		err := ms.LoadVersion(v)
		require.NoError(t, err, "expected no error when loading height: %d", v)
	}

	// Get latest
	err := ms.LoadVersion(numVersions - 1)
	require.NoError(t, err)

	// Ensure already pruned heights were loaded
	heights, err := ms.pruningManager.GetFlushAndResetPruningHeights()
	require.NoError(t, err)
	require.Equal(t, expectedHeights, heights)

	require.NoError(t, ms.pruningManager.LoadPruningHeights(db))

	// Test pruning the same heights again
	lastCommitInfo = ms.Commit()
	require.Equal(t, numVersions, lastCommitInfo.Version)

	// Ensure that can commit one more height with no panic
	lastCommitInfo = ms.Commit()
	require.Equal(t, numVersions+1, lastCommitInfo.Version)
}

func TestMultiStore_PruningRestart(t *testing.T) {
	db := dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(2, 11))
	ms.SetSnapshotInterval(3)
	require.NoError(t, ms.LoadLatestVersion())

	// Commit enough to build up heights to prune, where on the next block we should
	// batch delete.
	for i := int64(0); i < 10; i++ {
		ms.Commit()
	}

	pruneHeights := []int64{1, 2, 4, 5, 7}

	// ensure we've persisted the current batch of heights to prune to the store's DB
	err := ms.pruningManager.LoadPruningHeights(ms.db)
	require.NoError(t, err)

	actualHeightsToPrune, err := ms.pruningManager.GetFlushAndResetPruningHeights()
	require.NoError(t, err)
	require.Equal(t, len(pruneHeights), len(actualHeightsToPrune))
	require.Equal(t, pruneHeights, actualHeightsToPrune)

	// "restart"
	ms = newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(2, 11))
	ms.SetSnapshotInterval(3)
	err = ms.LoadLatestVersion()
	require.NoError(t, err)

	actualHeightsToPrune, err = ms.pruningManager.GetFlushAndResetPruningHeights()
	require.NoError(t, err)
	require.Equal(t, pruneHeights, actualHeightsToPrune)

	// commit one more block and ensure the heights have been pruned
	ms.Commit()

	actualHeightsToPrune, err = ms.pruningManager.GetFlushAndResetPruningHeights()
	require.NoError(t, err)
	require.Empty(t, actualHeightsToPrune)

	for _, v := range pruneHeights {
		_, err := ms.CacheMultiStoreWithVersion(v)
		require.NoError(t, err, "expected error when loading height: %d", v)
	}
}

// TestUnevenStoresHeightCheck tests if loading root store correctly errors when
// there's any module store with the wrong height
func TestUnevenStoresHeightCheck(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	store := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := store.LoadLatestVersion()
	require.Nil(t, err)

	// commit to increment store's height
	store.Commit()

	// mount store4 to root store
	store.MountStoreWithDB(types.NewKVStoreKey("store4"), types.StoreTypeIAVL, nil)

	// load the stores without upgrades
	err = store.LoadLatestVersion()
	require.Error(t, err)

	// now, let's load with upgrades...
	upgrades := &types.StoreUpgrades{
		Added: []string{"store4"},
	}
	err = store.LoadLatestVersionAndUpgrade(upgrades)
	require.Nil(t, err)
}

func TestSetInitialVersion(t *testing.T) {
	db := dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	require.NoError(t, multi.LoadLatestVersion())

	multi.SetInitialVersion(5)
	require.Equal(t, int64(5), multi.initialVersion)

	multi.Commit()
	require.Equal(t, int64(5), multi.LastCommitID().Version)

	ckvs := multi.GetCommitKVStore(multi.keysByName["store1"])
	iavlStore, ok := ckvs.(*iavl.Store)
	require.True(t, ok)
	require.True(t, iavlStore.VersionExists(5))
}

func TestAddListenersAndListeningEnabled(t *testing.T) {
	db := dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	testKey := types.NewKVStoreKey("listening_test_key")
	enabled := multi.ListeningEnabled(testKey)
	require.False(t, enabled)

	multi.AddListeners(testKey, []types.WriteListener{})
	enabled = multi.ListeningEnabled(testKey)
	require.False(t, enabled)

	mockListener := types.NewStoreKVPairWriteListener(nil, nil)
	multi.AddListeners(testKey, []types.WriteListener{mockListener})
	wrongTestKey := types.NewKVStoreKey("wrong_listening_test_key")
	enabled = multi.ListeningEnabled(wrongTestKey)
	require.False(t, enabled)

	enabled = multi.ListeningEnabled(testKey)
	require.True(t, enabled)
}

var (
	interfaceRegistry = codecTypes.NewInterfaceRegistry()
	testMarshaller    = codec.NewProtoCodec(interfaceRegistry)
	testKey1          = []byte{1, 2, 3, 4, 5}
	testValue1        = []byte{5, 4, 3, 2, 1}
	testKey2          = []byte{2, 3, 4, 5, 6}
	testValue2        = []byte{6, 5, 4, 3, 2}
)

func TestGetListenWrappedKVStore(t *testing.T) {
	buf := new(bytes.Buffer)
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	ms.LoadLatestVersion()
	mockListeners := []types.WriteListener{types.NewStoreKVPairWriteListener(buf, testMarshaller)}
	ms.AddListeners(testStoreKey1, mockListeners)
	ms.AddListeners(testStoreKey2, mockListeners)

	listenWrappedStore1 := ms.GetKVStore(testStoreKey1)
	require.IsType(t, &listenkv.Store{}, listenWrappedStore1)

	listenWrappedStore1.Set(testKey1, testValue1)
	expectedOutputKVPairSet1, err := testMarshaller.MarshalLengthPrefixed(&types.StoreKVPair{
		Key:      testKey1,
		Value:    testValue1,
		StoreKey: testStoreKey1.Name(),
		Delete:   false,
	})
	require.Nil(t, err)
	kvPairSet1Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, expectedOutputKVPairSet1, kvPairSet1Bytes)

	listenWrappedStore1.Delete(testKey1)
	expectedOutputKVPairDelete1, err := testMarshaller.MarshalLengthPrefixed(&types.StoreKVPair{
		Key:      testKey1,
		Value:    nil,
		StoreKey: testStoreKey1.Name(),
		Delete:   true,
	})
	require.Nil(t, err)
	kvPairDelete1Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, expectedOutputKVPairDelete1, kvPairDelete1Bytes)

	listenWrappedStore2 := ms.GetKVStore(testStoreKey2)
	require.IsType(t, &listenkv.Store{}, listenWrappedStore2)

	listenWrappedStore2.Set(testKey2, testValue2)
	expectedOutputKVPairSet2, err := testMarshaller.MarshalLengthPrefixed(&types.StoreKVPair{
		Key:      testKey2,
		Value:    testValue2,
		StoreKey: testStoreKey2.Name(),
		Delete:   false,
	})
	require.NoError(t, err)
	kvPairSet2Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, expectedOutputKVPairSet2, kvPairSet2Bytes)

	listenWrappedStore2.Delete(testKey2)
	expectedOutputKVPairDelete2, err := testMarshaller.MarshalLengthPrefixed(&types.StoreKVPair{
		Key:      testKey2,
		Value:    nil,
		StoreKey: testStoreKey2.Name(),
		Delete:   true,
	})
	require.NoError(t, err)
	kvPairDelete2Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, expectedOutputKVPairDelete2, kvPairDelete2Bytes)

	unwrappedStore := ms.GetKVStore(testStoreKey3)
	require.IsType(t, &iavl.Store{}, unwrappedStore)

	unwrappedStore.Set(testKey2, testValue2)
	kvPairSet3Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, []byte{}, kvPairSet3Bytes)

	unwrappedStore.Delete(testKey2)
	kvPairDelete3Bytes := buf.Bytes()
	buf.Reset()
	require.Equal(t, []byte{}, kvPairDelete3Bytes)
}

func TestCacheWraps(t *testing.T) {
	db := dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	cacheWrapper := multi.CacheWrap()
	require.IsType(t, cachemulti.Store{}, cacheWrapper)

	cacheWrappedWithTrace := multi.CacheWrapWithTrace(nil, nil)
	require.IsType(t, cachemulti.Store{}, cacheWrappedWithTrace)
}

func TestTraceConcurrency(t *testing.T) {
	db := dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := multi.LoadLatestVersion()
	require.NoError(t, err)

	b := &bytes.Buffer{}
	key := multi.keysByName["store1"]
	tc := types.TraceContext(map[string]interface{}{"blockHeight": 64})

	multi.SetTracer(b)
	multi.SetTracingContext(tc)

	cms := multi.CacheMultiStore()
	store1 := cms.GetKVStore(key)
	cw := store1.CacheWrapWithTrace(b, tc)
	_ = cw
	require.NotNil(t, store1)

	stop := make(chan struct{})
	stopW := make(chan struct{})

	go func(stop chan struct{}) {
		for {
			select {
			case <-stop:
				return
			default:
				store1.Set([]byte{1}, []byte{1})
				cms.Write()
			}
		}
	}(stop)

	go func(stop chan struct{}) {
		for {
			select {
			case <-stop:
				return
			default:
				multi.SetTracingContext(tc)
			}
		}
	}(stopW)

	time.Sleep(3 * time.Second)
	stop <- struct{}{}
	stopW <- struct{}{}
}

func TestCommitOrdered(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := multi.LoadLatestVersion()
	require.Nil(t, err)

	commitID := types.CommitID{}
	checkStore(t, multi, commitID, commitID)

	k, v := []byte("wind"), []byte("blows")
	k2, v2 := []byte("water"), []byte("flows")
	k3, v3 := []byte("fire"), []byte("burns")

	store1 := multi.GetStoreByName("store1").(types.KVStore)
	store1.Set(k, v)

	store2 := multi.GetStoreByName("store2").(types.KVStore)
	store2.Set(k2, v2)

	store3 := multi.GetStoreByName("store3").(types.KVStore)
	store3.Set(k3, v3)

	typeID := multi.Commit()
	require.Equal(t, int64(1), typeID.Version)

	ci, err := multi.GetCommitInfo(1)
	require.NoError(t, err)
	require.Equal(t, int64(1), ci.Version)
	require.Equal(t, 3, len(ci.StoreInfos))
	for i, s := range ci.StoreInfos {
		require.Equal(t, s.Name, fmt.Sprintf("store%d", i+1))
	}
}

//-----------------------------------------------------------------------
// utils

var (
	testStoreKey1 = types.NewKVStoreKey("store1")
	testStoreKey2 = types.NewKVStoreKey("store2")
	testStoreKey3 = types.NewKVStoreKey("store3")
)

func newMultiStoreWithMounts(db dbm.DB, pruningOpts pruningtypes.PruningOptions) *Store {
	store := NewStore(db, log.NewNopLogger())
	store.SetPruning(pruningOpts)

	store.MountStoreWithDB(testStoreKey1, types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(testStoreKey2, types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(testStoreKey3, types.StoreTypeIAVL, nil)

	return store
}

func newMultiStoreWithModifiedMounts(db dbm.DB, pruningOpts pruningtypes.PruningOptions) (*Store, *types.StoreUpgrades) {
	store := NewStore(db, log.NewNopLogger())
	store.SetPruning(pruningOpts)

	store.MountStoreWithDB(types.NewKVStoreKey("store1"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("restore2"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("store3"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("store4"), types.StoreTypeIAVL, nil)

	upgrades := &types.StoreUpgrades{
		Added: []string{"store4"},
		Renamed: []types.StoreRename{{
			OldKey: "store2",
			NewKey: "restore2",
		}},
		Deleted: []string{"store3"},
	}

	return store, upgrades
}

func unmountStore(rootStore *Store, storeKeyName string) {
	sk := rootStore.keysByName[storeKeyName]
	delete(rootStore.stores, sk)
	delete(rootStore.storesParams, sk)
	delete(rootStore.keysByName, storeKeyName)
}

func checkStore(t *testing.T, store *Store, expect, got types.CommitID) {
	require.Equal(t, expect, got)
	require.Equal(t, expect, store.LastCommitID())
}

func checkContains(t testing.TB, info []types.StoreInfo, wanted []string) {
	t.Helper()

	for _, want := range wanted {
		checkHas(t, info, want)
	}
}

func checkHas(t testing.TB, info []types.StoreInfo, want string) {
	t.Helper()
	for _, i := range info {
		if i.Name == want {
			return
		}
	}
	t.Fatalf("storeInfo doesn't contain %s", want)
}

func getExpectedCommitID(store *Store, ver int64) types.CommitID {
	return types.CommitID{
		Version: ver,
		Hash:    hashStores(store.stores),
	}
}

func hashStores(stores map[types.StoreKey]types.CommitKVStore) []byte {
	m := make(map[string][]byte, len(stores))
	for key, store := range stores {
		name := key.Name()
		m[name] = types.StoreInfo{
			Name:     name,
			CommitId: store.LastCommitID(),
		}.GetHash()
	}
	return sdkmaps.HashFromMap(m)
}

type MockListener struct {
	stateCache []types.StoreKVPair
}

func (tl *MockListener) OnWrite(storeKey types.StoreKey, key []byte, value []byte, delete bool) error {
	tl.stateCache = append(tl.stateCache, types.StoreKVPair{
		StoreKey: storeKey.Name(),
		Key:      key,
		Value:    value,
		Delete:   delete,
	})
	return nil
}

func TestStateListeners(t *testing.T) {
	var db dbm.DB = dbm.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	listener := &MockListener{}
	ms.AddListeners(testStoreKey1, []types.WriteListener{listener})

	require.NoError(t, ms.LoadLatestVersion())
	cacheMulti := ms.CacheMultiStore()

	store1 := cacheMulti.GetKVStore(testStoreKey1)
	store1.Set([]byte{1}, []byte{1})
	require.Empty(t, listener.stateCache)

	// writes are observed when cache store commit.
	cacheMulti.Write()
	require.Equal(t, 1, len(listener.stateCache))

	// test nested cache store
	listener.stateCache = []types.StoreKVPair{}
	nested := cacheMulti.CacheMultiStore()

	store1 = nested.GetKVStore(testStoreKey1)
	store1.Set([]byte{1}, []byte{1})
	require.Empty(t, listener.stateCache)

	// writes are not observed when nested cache store commit
	nested.Write()
	require.Empty(t, listener.stateCache)

	// writes are observed when inner cache store commit
	cacheMulti.Write()
	require.Equal(t, 1, len(listener.stateCache))
}

type commitKVStoreStub struct {
	types.CommitKVStore
	Committed int
}

func (stub *commitKVStoreStub) Commit() types.CommitID {
	commitID := stub.CommitKVStore.Commit()
	stub.Committed += 1
	return commitID
}

func prepareStoreMap() map[types.StoreKey]types.CommitKVStore {
	var db dbm.DB = dbm.NewMemDB()
	store := NewStore(db, log.NewNopLogger())
	store.MountStoreWithDB(types.NewKVStoreKey("iavl1"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("iavl2"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewTransientStoreKey("trans1"), types.StoreTypeTransient, nil)
	store.LoadLatestVersion()
	return map[types.StoreKey]types.CommitKVStore{
		testStoreKey1: &commitKVStoreStub{
			CommitKVStore: store.GetStoreByName("iavl1").(types.CommitKVStore),
		},
		testStoreKey2: &commitKVStoreStub{
			CommitKVStore: store.GetStoreByName("iavl2").(types.CommitKVStore),
		},
		testStoreKey3: &commitKVStoreStub{
			CommitKVStore: store.GetStoreByName("trans1").(types.CommitKVStore),
		},
	}
}

func TestCommitStores(t *testing.T) {
	testCases := []struct {
		name          string
		committed     int
		exptectCommit int
	}{
		{
			"when upgrade not get interrupted",
			0,
			1,
		},
		{
			"when upgrade get interrupted once",
			1,
			0,
		},
		{
			"when upgrade get interrupted twice",
			2,
			0,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			storeMap := prepareStoreMap()
			store := storeMap[testStoreKey1].(*commitKVStoreStub)
			for i := tc.committed; i > 0; i-- {
				store.Commit()
			}
			store.Committed = 0
			var version int64 = 1
			removalMap := map[types.StoreKey]bool{}
			res := commitStores(version, storeMap, removalMap)
			for _, s := range res.StoreInfos {
				require.Equal(t, version, s.CommitId.Version)
			}
			require.Equal(t, version, res.Version)
			require.Equal(t, tc.exptectCommit, store.Committed)
		})
	}
}
