package iavl

import (
	crand "crypto/rand"
	"fmt"
	"testing"

	"github.com/cosmos/cosmos-sdk/store/cachekv"

	dbm "github.com/cometbft/cometbft-db"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cosmos/iavl"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/types/kv"
)

var (
	cacheSize = 100
	treeData  = map[string]string{
		"hello": "goodbye",
		"aloha": "shalom",
	}
	nMoreData = 0
)

func randBytes(numBytes int) []byte {
	b := make([]byte, numBytes)
	_, _ = crand.Read(b)
	return b
}

// make a tree with data from above and save it
func newAlohaTree(t *testing.T, db dbm.DB) (*iavl.MutableTree, types.CommitID) {
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	for k, v := range treeData {
		tree.Set([]byte(k), []byte(v))
	}

	for i := 0; i < nMoreData; i++ {
		key := randBytes(12)
		value := randBytes(50)
		tree.Set(key, value)
	}

	hash, ver, err := tree.SaveVersion()
	require.Nil(t, err)

	return tree, types.CommitID{Version: ver, Hash: hash}
}

func TestLoadStore(t *testing.T) {
	db := dbm.NewMemDB()
	tree, _ := newAlohaTree(t, db)
	store := UnsafeNewStore(tree)

	// Create non-pruned height H
	updated, err := tree.Set([]byte("hello"), []byte("hallo"))
	require.NoError(t, err)
	require.True(t, updated)
	hash, verH, err := tree.SaveVersion()
	cIDH := types.CommitID{Version: verH, Hash: hash}
	require.Nil(t, err)

	// Create pruned height Hp
	updated, err = tree.Set([]byte("hello"), []byte("hola"))
	require.NoError(t, err)
	require.True(t, updated)
	hash, verHp, err := tree.SaveVersion()
	cIDHp := types.CommitID{Version: verHp, Hash: hash}
	require.Nil(t, err)

	// TODO: Prune this height

	// Create current height Hc
	updated, err = tree.Set([]byte("hello"), []byte("ciao"))
	require.NoError(t, err)
	require.True(t, updated)
	hash, verHc, err := tree.SaveVersion()
	cIDHc := types.CommitID{Version: verHc, Hash: hash}
	require.Nil(t, err)

	// Querying an existing store at some previous non-pruned height H
	hStore, err := store.GetImmutable(verH)
	require.NoError(t, err)
	require.Equal(t, string(hStore.Get([]byte("hello"))), "hallo")

	// Querying an existing store at some previous pruned height Hp
	hpStore, err := store.GetImmutable(verHp)
	require.NoError(t, err)
	require.Equal(t, string(hpStore.Get([]byte("hello"))), "hola")

	// Querying an existing store at current height Hc
	hcStore, err := store.GetImmutable(verHc)
	require.NoError(t, err)
	require.Equal(t, string(hcStore.Get([]byte("hello"))), "ciao")

	// Querying a new store at some previous non-pruned height H
	newHStore, err := LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), cIDH, false, DefaultIAVLCacheSize, false)
	require.NoError(t, err)
	require.Equal(t, string(newHStore.Get([]byte("hello"))), "hallo")

	// Querying a new store at some previous pruned height Hp
	newHpStore, err := LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), cIDHp, false, DefaultIAVLCacheSize, false)
	require.NoError(t, err)
	require.Equal(t, string(newHpStore.Get([]byte("hello"))), "hola")

	// Querying a new store at current height H
	newHcStore, err := LoadStore(db, log.NewNopLogger(), types.NewKVStoreKey("test"), cIDHc, false, DefaultIAVLCacheSize, false)
	require.NoError(t, err)
	require.Equal(t, string(newHcStore.Get([]byte("hello"))), "ciao")
}

func TestGetImmutable(t *testing.T) {
	db := dbm.NewMemDB()
	tree, cID := newAlohaTree(t, db)
	store := UnsafeNewStore(tree)

	updated, err := tree.Set([]byte("hello"), []byte("adios"))
	require.NoError(t, err)
	require.True(t, updated)
	hash, ver, err := tree.SaveVersion()
	cID = types.CommitID{Version: ver, Hash: hash}
	require.Nil(t, err)

	_, err = store.GetImmutable(cID.Version + 1)
	require.Error(t, err)

	newStore, err := store.GetImmutable(cID.Version - 1)
	require.NoError(t, err)
	require.Equal(t, newStore.Get([]byte("hello")), []byte("goodbye"))

	newStore, err = store.GetImmutable(cID.Version)
	require.NoError(t, err)
	require.Equal(t, newStore.Get([]byte("hello")), []byte("adios"))

	res := newStore.Query(abci.RequestQuery{Data: []byte("hello"), Height: cID.Version, Path: "/key", Prove: true})
	require.Equal(t, res.Value, []byte("adios"))
	require.NotNil(t, res.ProofOps)

	require.Panics(t, func() { newStore.Set(nil, nil) })
	require.Panics(t, func() { newStore.Delete(nil) })
	require.Panics(t, func() { newStore.Commit() })
}

func TestTestGetImmutableIterator(t *testing.T) {
	db := dbm.NewMemDB()
	tree, cID := newAlohaTree(t, db)
	store := UnsafeNewStore(tree)

	newStore, err := store.GetImmutable(cID.Version)
	require.NoError(t, err)

	iter := newStore.Iterator([]byte("aloha"), []byte("hellz"))
	expected := []string{"aloha", "hello"}
	var i int

	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}

	require.Equal(t, len(expected), i)
}

func TestIAVLStoreGetSetHasDelete(t *testing.T) {
	db := dbm.NewMemDB()
	tree, _ := newAlohaTree(t, db)
	iavlStore := UnsafeNewStore(tree)

	key := "hello"

	exists := iavlStore.Has([]byte(key))
	require.True(t, exists)

	value := iavlStore.Get([]byte(key))
	require.EqualValues(t, value, treeData[key])

	value2 := "notgoodbye"
	iavlStore.Set([]byte(key), []byte(value2))

	value = iavlStore.Get([]byte(key))
	require.EqualValues(t, value, value2)

	iavlStore.Delete([]byte(key))

	exists = iavlStore.Has([]byte(key))
	require.False(t, exists)
}

func TestIAVLStoreNoNilSet(t *testing.T) {
	db := dbm.NewMemDB()
	tree, _ := newAlohaTree(t, db)
	iavlStore := UnsafeNewStore(tree)

	require.Panics(t, func() { iavlStore.Set(nil, []byte("value")) }, "setting a nil key should panic")
	require.Panics(t, func() { iavlStore.Set([]byte(""), []byte("value")) }, "setting an empty key should panic")

	require.Panics(t, func() { iavlStore.Set([]byte("key"), nil) }, "setting a nil value should panic")
}

func TestIAVLIterator(t *testing.T) {
	db := dbm.NewMemDB()
	tree, _ := newAlohaTree(t, db)
	iavlStore := UnsafeNewStore(tree)
	iter := iavlStore.Iterator([]byte("aloha"), []byte("hellz"))
	expected := []string{"aloha", "hello"}
	var i int

	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)

	iter = iavlStore.Iterator([]byte("golang"), []byte("rocks"))
	expected = []string{"hello"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)

	iter = iavlStore.Iterator(nil, []byte("golang"))
	expected = []string{"aloha"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)

	iter = iavlStore.Iterator(nil, []byte("shalom"))
	expected = []string{"aloha", "hello"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)

	iter = iavlStore.Iterator(nil, nil)
	expected = []string{"aloha", "hello"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)

	iter = iavlStore.Iterator([]byte("golang"), nil)
	expected = []string{"hello"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, treeData[expectedKey])
		i++
	}
	require.Equal(t, len(expected), i)
}

func TestIAVLReverseIterator(t *testing.T) {
	db := dbm.NewMemDB()

	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	iavlStore := UnsafeNewStore(tree)

	iavlStore.Set([]byte{0x00}, []byte("0"))
	iavlStore.Set([]byte{0x00, 0x00}, []byte("0 0"))
	iavlStore.Set([]byte{0x00, 0x01}, []byte("0 1"))
	iavlStore.Set([]byte{0x00, 0x02}, []byte("0 2"))
	iavlStore.Set([]byte{0x01}, []byte("1"))

	testReverseIterator := func(t *testing.T, start []byte, end []byte, expected []string) {
		iter := iavlStore.ReverseIterator(start, end)
		var i int
		for i = 0; iter.Valid(); iter.Next() {
			expectedValue := expected[i]
			value := iter.Value()
			require.EqualValues(t, string(value), expectedValue)
			i++
		}
		require.Equal(t, len(expected), i)
	}

	testReverseIterator(t, nil, nil, []string{"1", "0 2", "0 1", "0 0", "0"})
	testReverseIterator(t, []byte{0x00}, nil, []string{"1", "0 2", "0 1", "0 0", "0"})
	testReverseIterator(t, []byte{0x00}, []byte{0x00, 0x01}, []string{"0 0", "0"})
	testReverseIterator(t, []byte{0x00}, []byte{0x01}, []string{"0 2", "0 1", "0 0", "0"})
	testReverseIterator(t, []byte{0x00, 0x01}, []byte{0x01}, []string{"0 2", "0 1"})
	testReverseIterator(t, nil, []byte{0x01}, []string{"0 2", "0 1", "0 0", "0"})
}

func TestIAVLPrefixIterator(t *testing.T) {
	db := dbm.NewMemDB()
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	iavlStore := UnsafeNewStore(tree)

	iavlStore.Set([]byte("test1"), []byte("test1"))
	iavlStore.Set([]byte("test2"), []byte("test2"))
	iavlStore.Set([]byte("test3"), []byte("test3"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(0)}, []byte("test4"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(1)}, []byte("test4"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(255)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(0)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(1)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(255)}, []byte("test4"))

	var i int

	iter := types.KVStorePrefixIterator(iavlStore, []byte("test"))
	expected := []string{"test1", "test2", "test3"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, expectedKey)
		i++
	}
	iter.Close()
	require.Equal(t, len(expected), i)

	iter = types.KVStorePrefixIterator(iavlStore, []byte{byte(55), byte(255), byte(255)})
	expected2 := [][]byte{
		{byte(55), byte(255), byte(255), byte(0)},
		{byte(55), byte(255), byte(255), byte(1)},
		{byte(55), byte(255), byte(255), byte(255)},
	}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected2[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, []byte("test4"))
		i++
	}
	iter.Close()
	require.Equal(t, len(expected), i)

	iter = types.KVStorePrefixIterator(iavlStore, []byte{byte(255), byte(255)})
	expected2 = [][]byte{
		{byte(255), byte(255), byte(0)},
		{byte(255), byte(255), byte(1)},
		{byte(255), byte(255), byte(255)},
	}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected2[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, []byte("test4"))
		i++
	}
	iter.Close()
	require.Equal(t, len(expected), i)
}

func TestIAVLReversePrefixIterator(t *testing.T) {
	db := dbm.NewMemDB()
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	iavlStore := UnsafeNewStore(tree)

	iavlStore.Set([]byte("test1"), []byte("test1"))
	iavlStore.Set([]byte("test2"), []byte("test2"))
	iavlStore.Set([]byte("test3"), []byte("test3"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(0)}, []byte("test4"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(1)}, []byte("test4"))
	iavlStore.Set([]byte{byte(55), byte(255), byte(255), byte(255)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(0)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(1)}, []byte("test4"))
	iavlStore.Set([]byte{byte(255), byte(255), byte(255)}, []byte("test4"))

	var i int

	iter := types.KVStoreReversePrefixIterator(iavlStore, []byte("test"))
	expected := []string{"test3", "test2", "test1"}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, expectedKey)
		i++
	}
	require.Equal(t, len(expected), i)

	iter = types.KVStoreReversePrefixIterator(iavlStore, []byte{byte(55), byte(255), byte(255)})
	expected2 := [][]byte{
		{byte(55), byte(255), byte(255), byte(255)},
		{byte(55), byte(255), byte(255), byte(1)},
		{byte(55), byte(255), byte(255), byte(0)},
	}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected2[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, []byte("test4"))
		i++
	}
	require.Equal(t, len(expected), i)

	iter = types.KVStoreReversePrefixIterator(iavlStore, []byte{byte(255), byte(255)})
	expected2 = [][]byte{
		{byte(255), byte(255), byte(255)},
		{byte(255), byte(255), byte(1)},
		{byte(255), byte(255), byte(0)},
	}
	for i = 0; iter.Valid(); iter.Next() {
		expectedKey := expected2[i]
		key, value := iter.Key(), iter.Value()
		require.EqualValues(t, key, expectedKey)
		require.EqualValues(t, value, []byte("test4"))
		i++
	}
	require.Equal(t, len(expected), i)
}

func nextVersion(iavl *Store) {
	key := []byte(fmt.Sprintf("Key for tree: %d", iavl.LastCommitID().Version))
	value := []byte(fmt.Sprintf("Value for tree: %d", iavl.LastCommitID().Version))
	iavl.Set(key, value)
	iavl.Commit()
}

func TestIAVLNoPrune(t *testing.T) {
	db := dbm.NewMemDB()
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	iavlStore := UnsafeNewStore(tree)
	nextVersion(iavlStore)

	for i := 1; i < 100; i++ {
		for j := 1; j <= i; j++ {
			require.True(t, iavlStore.VersionExists(int64(j)),
				"Missing version %d with latest version %d. Should be storing all versions",
				j, i)
		}

		nextVersion(iavlStore)
	}
}

func TestIAVLStoreQuery(t *testing.T) {
	db := dbm.NewMemDB()
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(t, err)

	iavlStore := UnsafeNewStore(tree)

	k1, v1 := []byte("key1"), []byte("val1")
	k2, v2 := []byte("key2"), []byte("val2")
	v3 := []byte("val3")

	ksub := []byte("key")
	KVs0 := kv.Pairs{}
	KVs1 := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: k1, Value: v1},
			{Key: k2, Value: v2},
		},
	}
	KVs2 := kv.Pairs{
		Pairs: []kv.Pair{
			{Key: k1, Value: v3},
			{Key: k2, Value: v2},
		},
	}

	valExpSubEmpty, err := KVs0.Marshal()
	require.NoError(t, err)

	valExpSub1, err := KVs1.Marshal()
	require.NoError(t, err)

	valExpSub2, err := KVs2.Marshal()
	require.NoError(t, err)

	cid := iavlStore.Commit()
	ver := cid.Version
	query := abci.RequestQuery{Path: "/key", Data: k1, Height: ver}
	querySub := abci.RequestQuery{Path: "/subspace", Data: ksub, Height: ver}

	// query subspace before anything set
	qres := iavlStore.Query(querySub)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, valExpSubEmpty, qres.Value)

	// set data
	iavlStore.Set(k1, v1)
	iavlStore.Set(k2, v2)

	// set data without commit, doesn't show up
	qres = iavlStore.Query(query)
	require.Equal(t, uint32(0), qres.Code)
	require.Nil(t, qres.Value)

	// commit it, but still don't see on old version
	cid = iavlStore.Commit()
	qres = iavlStore.Query(query)
	require.Equal(t, uint32(0), qres.Code)
	require.Nil(t, qres.Value)

	// but yes on the new version
	query.Height = cid.Version
	qres = iavlStore.Query(query)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, v1, qres.Value)

	// and for the subspace
	qres = iavlStore.Query(querySub)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, valExpSub1, qres.Value)

	// modify
	iavlStore.Set(k1, v3)
	cid = iavlStore.Commit()

	// query will return old values, as height is fixed
	qres = iavlStore.Query(query)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, v1, qres.Value)

	// update to latest in the query and we are happy
	query.Height = cid.Version
	qres = iavlStore.Query(query)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, v3, qres.Value)
	query2 := abci.RequestQuery{Path: "/key", Data: k2, Height: cid.Version}

	qres = iavlStore.Query(query2)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, v2, qres.Value)
	// and for the subspace
	qres = iavlStore.Query(querySub)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, valExpSub2, qres.Value)

	// default (height 0) will show latest -1
	query0 := abci.RequestQuery{Path: "/key", Data: k1}
	qres = iavlStore.Query(query0)
	require.Equal(t, uint32(0), qres.Code)
	require.Equal(t, v1, qres.Value)
}

func BenchmarkIAVLIteratorNext(b *testing.B) {
	b.ReportAllocs()
	db := dbm.NewMemDB()
	treeSize := 1000
	tree, err := iavl.NewMutableTree(db, cacheSize, false)
	require.NoError(b, err)

	for i := 0; i < treeSize; i++ {
		key := randBytes(4)
		value := randBytes(50)
		tree.Set(key, value)
	}

	iavlStore := UnsafeNewStore(tree)
	iterators := make([]types.Iterator, b.N/treeSize)

	for i := 0; i < len(iterators); i++ {
		iterators[i] = iavlStore.Iterator([]byte{0}, []byte{255, 255, 255, 255, 255})
	}

	b.ResetTimer()
	for i := 0; i < len(iterators); i++ {
		iter := iterators[i]
		for j := 0; j < treeSize; j++ {
			iter.Next()
		}
	}
}

func TestSetInitialVersion(t *testing.T) {
	testCases := []struct {
		name     string
		storeFn  func(db *dbm.MemDB) *Store
		expPanic bool
	}{
		{
			"works with a mutable tree",
			func(db *dbm.MemDB) *Store {
				tree, err := iavl.NewMutableTree(db, cacheSize, false)
				require.NoError(t, err)
				store := UnsafeNewStore(tree)

				return store
			}, false,
		},
		{
			"throws error on immutable tree",
			func(db *dbm.MemDB) *Store {
				tree, err := iavl.NewMutableTree(db, cacheSize, false)
				require.NoError(t, err)
				store := UnsafeNewStore(tree)
				_, version, err := store.tree.SaveVersion()
				require.NoError(t, err)
				require.Equal(t, int64(1), version)
				store, err = store.GetImmutable(1)
				require.NoError(t, err)

				return store
			}, true,
		},
	}

	for _, tc := range testCases {
		tc := tc

		t.Run(tc.name, func(t *testing.T) {
			db := dbm.NewMemDB()
			store := tc.storeFn(db)

			if tc.expPanic {
				require.Panics(t, func() { store.SetInitialVersion(5) })
			} else {
				store.SetInitialVersion(5)
				cid := store.Commit()
				require.Equal(t, int64(5), cid.GetVersion())
			}
		})
	}
}

func TestCacheWraps(t *testing.T) {
	db := dbm.NewMemDB()
	tree, _ := newAlohaTree(t, db)
	store := UnsafeNewStore(tree)

	cacheWrapper := store.CacheWrap()
	require.IsType(t, &cachekv.Store{}, cacheWrapper)

	cacheWrappedWithTrace := store.CacheWrapWithTrace(nil, nil)
	require.IsType(t, &cachekv.Store{}, cacheWrappedWithTrace)
}
