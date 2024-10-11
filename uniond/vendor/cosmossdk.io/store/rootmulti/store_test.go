package rootmulti

import (
	"bytes"
	"crypto/sha256"
	"fmt"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	corestore "cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/errors"
	"cosmossdk.io/log"
	"cosmossdk.io/store/cachemulti"
	"cosmossdk.io/store/iavl"
	sdkmaps "cosmossdk.io/store/internal/maps"
	"cosmossdk.io/store/metrics"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/types"
)

func TestStoreType(t *testing.T) {
	db := coretesting.NewMemDB()
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	store.MountStoreWithDB(types.NewKVStoreKey("store1"), types.StoreTypeIAVL, db)
}

func TestGetCommitKVStore(t *testing.T) {
	db := coretesting.NewMemDB()
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
	db := coretesting.NewMemDB()
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())

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
	db := coretesting.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	cacheMulti := ms.CacheMultiStore()
	require.IsType(t, cachemulti.Store{}, cacheMulti)
}

func TestCacheMultiStoreWithVersion(t *testing.T) {
	db := coretesting.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := ms.LoadLatestVersion()
	require.Nil(t, err)

	emptyHash := sha256.Sum256([]byte{})
	appHash := emptyHash[:]
	commitID := types.CommitID{Hash: appHash}
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
	key4, key5 := types.NewKVStoreKey("store4"), types.NewKVStoreKey("store5")
	ms.MountStoreWithDB(key4, types.StoreTypeIAVL, nil)
	ms.MountStoreWithDB(key5, types.StoreTypeIAVL, nil)
	err = ms.LoadLatestVersionAndUpgrade(&types.StoreUpgrades{Added: []string{"store4", "store5"}})
	require.NoError(t, err)
	ms.Commit()

	// cache multistore of version before adding store4 should works
	cms2, err := ms.CacheMultiStoreWithVersion(1)
	require.NoError(t, err)

	require.Empty(t, cms2.GetKVStore(key4).Get([]byte("key")))

	// require we cannot commit (write) to a cache-versioned multi-store
	require.Panics(t, func() {
		kvStore.Set(k, []byte("newValue"))
		cms.Write()
	})
}

func TestHashStableWithEmptyCommit(t *testing.T) {
	db := coretesting.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := ms.LoadLatestVersion()
	require.Nil(t, err)

	emptyHash := sha256.Sum256([]byte{})
	appHash := emptyHash[:]
	commitID := types.CommitID{Hash: appHash}
	checkStore(t, ms, commitID, commitID)

	k, v := []byte("wind"), []byte("blows")

	store1 := ms.GetStoreByName("store1").(types.KVStore)
	store1.Set(k, v)

	workingHash := ms.WorkingHash()
	cID := ms.Commit()
	require.Equal(t, int64(1), cID.Version)
	hash := cID.Hash
	require.Equal(t, workingHash, hash)

	// make an empty commit, it should update version, but not affect hash
	workingHash = ms.WorkingHash()
	cID = ms.Commit()
	require.Equal(t, workingHash, cID.Hash)
	require.Equal(t, int64(2), cID.Version)
	require.Equal(t, hash, cID.Hash)
}

func TestMultistoreCommitLoad(t *testing.T) {
	db := coretesting.NewMemDB()
	store := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := store.LoadLatestVersion()
	require.Nil(t, err)

	emptyHash := sha256.Sum256([]byte{})
	appHash := emptyHash[:]
	// New store has empty last commit.
	commitID := types.CommitID{Hash: appHash}
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
		workingHash := store.WorkingHash()
		commitID = store.Commit()
		require.Equal(t, workingHash, commitID.Hash)
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
	workingHash := store.WorkingHash()
	commitID = store.Commit()
	require.Equal(t, workingHash, commitID.Hash)
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
	db := coretesting.NewMemDB()
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
	workingHash := store.WorkingHash()
	commitID := store.Commit()
	require.Equal(t, workingHash, commitID.Hash)
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
		values++
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
	db := coretesting.NewMemDB()
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
	db := coretesting.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := multi.LoadLatestVersion()
	require.Nil(t, err)

	k, v := []byte("wind"), []byte("blows")
	k2, v2 := []byte("water"), []byte("flows")
	// v3 := []byte("is cold")

	// Commit the multistore.
	_ = multi.Commit()

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
	cid := multi.Commit()
	ver := cid.Version

	// Reload multistore from database
	multi = newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err = multi.LoadLatestVersion()
	require.Nil(t, err)

	// Test bad path.
	query := types.RequestQuery{Path: "/key", Data: k, Height: ver}
	_, err = multi.Query(&query)
	codespace, code, _ := errors.ABCIInfo(err, false)
	require.EqualValues(t, types.ErrUnknownRequest.ABCICode(), code)
	require.EqualValues(t, types.ErrUnknownRequest.Codespace(), codespace)

	query.Path = "h897fy32890rf63296r92"
	_, err = multi.Query(&query)
	codespace, code, _ = errors.ABCIInfo(err, false)
	require.EqualValues(t, types.ErrUnknownRequest.ABCICode(), code)
	require.EqualValues(t, types.ErrUnknownRequest.Codespace(), codespace)

	// Test invalid store name.
	query.Path = "/garbage/key"
	_, err = multi.Query(&query)
	codespace, code, _ = errors.ABCIInfo(err, false)
	require.EqualValues(t, types.ErrUnknownRequest.ABCICode(), code)
	require.EqualValues(t, types.ErrUnknownRequest.Codespace(), codespace)

	// Test valid query with data.
	query.Path = "/store1/key"
	qres, err := multi.Query(&query)
	require.NoError(t, err)
	require.Equal(t, v, qres.Value)

	// Test valid but empty query.
	query.Path = "/store2/key"
	query.Prove = true
	qres, err = multi.Query(&query)
	require.NoError(t, err)
	require.Nil(t, qres.Value)

	// Test store2 data.
	// Since we are using the request as a reference, the path will be modified.
	query.Data = k2
	query.Path = "/store2/key"
	qres, err = multi.Query(&query)
	require.NoError(t, err)
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
			db := coretesting.NewMemDB()
			ms := newMultiStoreWithMounts(db, tc.po)
			require.NoError(t, ms.LoadLatestVersion())

			for i := int64(0); i < tc.numVersions; i++ {
				ms.Commit()
			}

			for _, v := range tc.deleted {
				// Ensure async pruning is done
				checkErr := func() bool {
					_, err := ms.CacheMultiStoreWithVersion(v)
					return err != nil
				}
				require.Eventually(t, checkErr, 1*time.Second, 10*time.Millisecond, "expected error when loading height: %d", v)
			}

			for _, v := range tc.saved {
				_, err := ms.CacheMultiStoreWithVersion(v)
				require.NoError(t, err, "expected no error when loading height: %d", v)
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

	db := coretesting.NewMemDB()

	ms := newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(keepRecent, interval))
	require.NoError(t, ms.LoadLatestVersion())

	var lastCommitInfo types.CommitID
	for i := int64(0); i < numVersions; i++ {
		lastCommitInfo = ms.Commit()
	}

	require.Equal(t, numVersions, lastCommitInfo.Version)

	// Get latest
	err := ms.LoadVersion(numVersions - 1)
	require.NoError(t, err)

	// Ensure already pruned snapshot heights were loaded
	require.NoError(t, ms.pruningManager.LoadSnapshotHeights(db))

	// Test pruning the same heights again
	lastCommitInfo = ms.Commit()
	require.Equal(t, numVersions, lastCommitInfo.Version)

	// Ensure that can commit one more height with no panic
	lastCommitInfo = ms.Commit()
	require.Equal(t, numVersions+1, lastCommitInfo.Version)

	isPruned := func() bool {
		ls := ms.Commit() // to flush the batch with the pruned heights
		for v := int64(1); v < numVersions-int64(keepRecent); v++ {
			if err := ms.LoadVersion(v); err == nil {
				require.NoError(t, ms.LoadVersion(ls.Version)) // load latest
				return false
			}
		}
		return true
	}
	require.Eventually(t, isPruned, 1000*time.Second, 10*time.Millisecond, "expected error when loading pruned heights")
}

func TestMultiStore_PruningRestart(t *testing.T) {
	db := coretesting.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(2, 11))
	require.NoError(t, ms.LoadLatestVersion())

	// Commit enough to build up heights to prune, where on the next block we should
	// batch delete.
	for i := int64(0); i < 10; i++ {
		ms.Commit()
	}

	actualHeightToPrune := ms.pruningManager.GetPruningHeight(ms.LatestVersion())
	require.Equal(t, int64(0), actualHeightToPrune)

	// "restart"
	ms = newMultiStoreWithMounts(db, pruningtypes.NewCustomPruningOptions(2, 11))
	err := ms.LoadLatestVersion()
	require.NoError(t, err)

	actualHeightToPrune = ms.pruningManager.GetPruningHeight(ms.LatestVersion())
	require.Equal(t, int64(0), actualHeightToPrune)

	// commit one more block and ensure the heights have been pruned
	ms.Commit()

	actualHeightToPrune = ms.pruningManager.GetPruningHeight(ms.LatestVersion())
	require.Equal(t, int64(8), actualHeightToPrune)

	// Ensure async pruning is done
	isPruned := func() bool {
		ms.Commit() // to flush the batch with the pruned heights
		for v := int64(1); v <= actualHeightToPrune; v++ {
			if _, err := ms.CacheMultiStoreWithVersion(v); err == nil {
				return false
			}
		}
		return true
	}

	require.Eventually(t, isPruned, 1*time.Second, 10*time.Millisecond, "expected error when loading pruned heights")
}

var _ types.PausablePruner = &pauseableCommitKVStoreStub{}

type pauseableCommitKVStoreStub struct {
	types.CommitKVStore
	pauseCalled []bool
}

func (p *pauseableCommitKVStoreStub) PausePruning(b bool) {
	p.pauseCalled = append(p.pauseCalled, b)
}

func TestPausePruningOnCommit(t *testing.T) {
	store := NewStore(coretesting.NewMemDB(), log.NewNopLogger(), metrics.NewNoOpMetrics())
	store.SetPruning(pruningtypes.NewCustomPruningOptions(2, 11))
	store.MountStoreWithDB(testStoreKey1, types.StoreTypeIAVL, nil)
	require.NoError(t, store.LoadLatestVersion())

	myStub := &pauseableCommitKVStoreStub{CommitKVStore: store.stores[testStoreKey1]}
	store.stores[testStoreKey1] = myStub
	// when
	store.Commit()
	// then
	require.Equal(t, []bool{true, false}, myStub.pauseCalled)
}

// TestUnevenStoresHeightCheck tests if loading root store correctly errors when
// there's any module store with the wrong height
func TestUnevenStoresHeightCheck(t *testing.T) {
	db := coretesting.NewMemDB()
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
	db := coretesting.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	require.NoError(t, multi.LoadLatestVersion())

	err := multi.SetInitialVersion(5)
	require.NoError(t, err)
	require.Equal(t, int64(5), multi.initialVersion)

	multi.Commit()
	require.Equal(t, int64(5), multi.LastCommitID().Version)

	ckvs := multi.GetCommitKVStore(multi.keysByName["store1"])
	iavlStore, ok := ckvs.(*iavl.Store)
	require.True(t, ok)
	require.True(t, iavlStore.VersionExists(5))
}

func TestAddListenersAndListeningEnabled(t *testing.T) {
	db := coretesting.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	testKey := types.NewKVStoreKey("listening_test_key")
	enabled := multi.ListeningEnabled(testKey)
	require.False(t, enabled)

	wrongTestKey := types.NewKVStoreKey("wrong_listening_test_key")
	multi.AddListeners([]types.StoreKey{testKey})
	enabled = multi.ListeningEnabled(wrongTestKey)
	require.False(t, enabled)

	enabled = multi.ListeningEnabled(testKey)
	require.True(t, enabled)
}

func TestCacheWraps(t *testing.T) {
	db := coretesting.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))

	cacheWrapper := multi.CacheWrap()
	require.IsType(t, cachemulti.Store{}, cacheWrapper)

	cacheWrappedWithTrace := multi.CacheWrapWithTrace(nil, nil)
	require.IsType(t, cachemulti.Store{}, cacheWrappedWithTrace)
}

func TestTraceConcurrency(t *testing.T) {
	db := coretesting.NewMemDB()
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
	db := coretesting.NewMemDB()
	multi := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	err := multi.LoadLatestVersion()
	require.Nil(t, err)

	emptyHash := sha256.Sum256([]byte{})
	appHash := emptyHash[:]
	commitID := types.CommitID{Hash: appHash}
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

func newMultiStoreWithMounts(db corestore.KVStoreWithBatch, pruningOpts pruningtypes.PruningOptions) *Store {
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	store.SetPruning(pruningOpts)

	store.MountStoreWithDB(testStoreKey1, types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(testStoreKey2, types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(testStoreKey3, types.StoreTypeIAVL, nil)

	return store
}

func newMultiStoreWithModifiedMounts(db corestore.KVStoreWithBatch, pruningOpts pruningtypes.PruningOptions) (*Store, *types.StoreUpgrades) {
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
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
	t.Helper()
	require.Equal(t, expect, got)
	require.Equal(t, expect, store.LastCommitID())
}

func checkContains(tb testing.TB, info []types.StoreInfo, wanted []string) {
	tb.Helper()

	for _, want := range wanted {
		checkHas(tb, info, want)
	}
}

func checkHas(tb testing.TB, info []types.StoreInfo, want string) {
	tb.Helper()
	for _, i := range info {
		if i.Name == want {
			return
		}
	}
	tb.Fatalf("storeInfo doesn't contain %s", want)
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

func (tl *MockListener) OnWrite(storeKey types.StoreKey, key, value []byte, delete bool) error {
	tl.stateCache = append(tl.stateCache, types.StoreKVPair{
		StoreKey: storeKey.Name(),
		Key:      key,
		Value:    value,
		Delete:   delete,
	})
	return nil
}

func TestStateListeners(t *testing.T) {
	db := coretesting.NewMemDB()
	ms := newMultiStoreWithMounts(db, pruningtypes.NewPruningOptions(pruningtypes.PruningNothing))
	require.Empty(t, ms.listeners)

	ms.AddListeners([]types.StoreKey{testStoreKey1})
	require.Equal(t, 1, len(ms.listeners))

	require.NoError(t, ms.LoadLatestVersion())
	cacheMulti := ms.CacheMultiStore()

	store := cacheMulti.GetKVStore(testStoreKey1)
	store.Set([]byte{1}, []byte{1})
	require.Empty(t, ms.PopStateCache())

	// writes are observed when cache store commit.
	cacheMulti.Write()
	require.Equal(t, 1, len(ms.PopStateCache()))

	// test no listening on unobserved store
	store = cacheMulti.GetKVStore(testStoreKey2)
	store.Set([]byte{1}, []byte{1})
	require.Empty(t, ms.PopStateCache())

	// writes are not observed when cache store commit
	cacheMulti.Write()
	require.Empty(t, ms.PopStateCache())
}

type commitKVStoreStub struct {
	types.CommitKVStore
	Committed int
}

func (stub *commitKVStoreStub) Commit() types.CommitID {
	commitID := stub.CommitKVStore.Commit()
	stub.Committed++
	return commitID
}

func prepareStoreMap() (map[types.StoreKey]types.CommitKVStore, error) {
	db := coretesting.NewMemDB()
	store := NewStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	store.MountStoreWithDB(types.NewKVStoreKey("iavl1"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewKVStoreKey("iavl2"), types.StoreTypeIAVL, nil)
	store.MountStoreWithDB(types.NewTransientStoreKey("trans1"), types.StoreTypeTransient, nil)
	if err := store.LoadLatestVersion(); err != nil {
		return nil, err
	}
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
	}, nil
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
			storeMap, err := prepareStoreMap()
			require.NoError(t, err)
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
