package iavl

import (
	"bytes"
	"errors"
	"fmt"
	"runtime"
	"sort"
	"strconv"
	"sync"
	"testing"

	"github.com/cosmos/iavl/fastnode"

	"github.com/cosmos/iavl/internal/encoding"
	iavlrand "github.com/cosmos/iavl/internal/rand"
	"github.com/cosmos/iavl/mock"
	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

var (
	tKey1 = []byte("k1")
	tVal1 = []byte("v1")

	tKey2 = []byte("k2")
	tVal2 = []byte("v2")
	// FIXME: enlarge maxIterator to 100000
	maxIterator = 100
)

func setupMutableTree(skipFastStorageUpgrade bool) *MutableTree {
	memDB := dbm.NewMemDB()
	tree := NewMutableTree(memDB, 0, skipFastStorageUpgrade, NewNopLogger())
	return tree
}

// TestIterateConcurrency throws "fatal error: concurrent map writes" when fast node is enabled
func TestIterateConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}
	tree := setupMutableTree(true)
	wg := new(sync.WaitGroup)
	for i := 0; i < 100; i++ {
		for j := 0; j < maxIterator; j++ {
			wg.Add(1)
			go func(i, j int) {
				defer wg.Done()
				_, err := tree.Set([]byte(fmt.Sprintf("%d%d", i, j)), iavlrand.RandBytes(1))
				require.NoError(t, err)
			}(i, j)
		}
		tree.Iterate(func(_, _ []byte) bool { //nolint:errcheck
			return false
		})
	}
	wg.Wait()
}

// TestConcurrency throws "fatal error: concurrent map iteration and map write" and
// also sometimes "fatal error: concurrent map writes" when fast node is enabled
func TestIteratorConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}
	tree := setupMutableTree(true)
	_, err := tree.LoadVersion(0)
	require.NoError(t, err)
	// So much slower
	wg := new(sync.WaitGroup)
	for i := 0; i < 100; i++ {
		for j := 0; j < maxIterator; j++ {
			wg.Add(1)
			go func(i, j int) {
				defer wg.Done()
				_, err := tree.Set([]byte(fmt.Sprintf("%d%d", i, j)), iavlrand.RandBytes(1))
				require.NoError(t, err)
			}(i, j)
		}
		itr, _ := tree.Iterator(nil, nil, true)
		for ; itr.Valid(); itr.Next() { //nolint:revive
		} // do nothing
	}
	wg.Wait()
}

// TestNewIteratorConcurrency throws "fatal error: concurrent map writes" when fast node is enabled
func TestNewIteratorConcurrency(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}
	tree := setupMutableTree(true)
	for i := 0; i < 100; i++ {
		wg := new(sync.WaitGroup)
		it := NewIterator(nil, nil, true, tree.ImmutableTree)
		for j := 0; j < maxIterator; j++ {
			wg.Add(1)
			go func(i, j int) {
				defer wg.Done()
				_, err := tree.Set([]byte(fmt.Sprintf("%d%d", i, j)), iavlrand.RandBytes(1))
				require.NoError(t, err)
			}(i, j)
		}
		for ; it.Valid(); it.Next() { //nolint:revive
		} // do nothing
		wg.Wait()
	}
}

func TestDeleteVersionsTo(t *testing.T) {
	tree := setupMutableTree(false)

	_, err := tree.set([]byte("k1"), []byte("Fred"))
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	require.NoError(t, tree.DeleteVersionsTo(version))

	proof, err := tree.GetVersionedProof([]byte("k1"), version)
	require.EqualError(t, err, ErrVersionDoesNotExist.Error())
	require.Nil(t, proof)

	proof, err = tree.GetVersionedProof([]byte("k1"), version+1)
	require.Nil(t, err)
	require.Equal(t, 0, bytes.Compare([]byte("Fred"), proof.GetExist().Value))
}

func TestDeleteVersionsFrom(t *testing.T) {
	tree := setupMutableTree(false)

	_, err := tree.set([]byte("k1"), []byte("Wilma"))
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	require.NoError(t, tree.DeleteVersionsFrom(version+1))

	proof, err := tree.GetVersionedProof([]byte("k1"), version)
	require.Nil(t, err)
	require.Equal(t, 0, bytes.Compare([]byte("Wilma"), proof.GetExist().Value))

	proof, err = tree.GetVersionedProof([]byte("k1"), version+1)
	require.EqualError(t, err, ErrVersionDoesNotExist.Error())
	require.Nil(t, proof)

	proof, err = tree.GetVersionedProof([]byte("k1"), version+2)
	require.EqualError(t, err, ErrVersionDoesNotExist.Error())
	require.Nil(t, proof)
}

func TestGetRemove(t *testing.T) {
	require := require.New(t)
	tree := setupMutableTree(false)
	testGet := func(exists bool) {
		v, err := tree.Get(tKey1)
		require.NoError(err)
		if exists {
			require.Equal(tVal1, v, "key should exist")
		} else {
			require.Nil(v, "key should not exist")
		}
	}

	testGet(false)

	ok, err := tree.Set(tKey1, tVal1)
	require.NoError(err)
	require.False(ok, "new key set: nothing to update")

	// add second key to avoid tree.root removal
	ok, err = tree.Set(tKey2, tVal2)
	require.NoError(err)
	require.False(ok, "new key set: nothing to update")

	testGet(true)

	// Save to tree.ImmutableTree
	_, version, err := tree.SaveVersion()
	require.NoError(err)
	require.Equal(int64(1), version)

	testGet(true)

	v, ok, err := tree.Remove(tKey1)
	require.NoError(err)
	require.True(ok, "key should be removed")
	require.Equal(tVal1, v, "key should exist")

	testGet(false)
}

func TestTraverse(t *testing.T) {
	tree := setupMutableTree(false)

	for i := 0; i < 6; i++ {
		_, err := tree.set([]byte(fmt.Sprintf("k%d", i)), []byte(fmt.Sprintf("v%d", i)))
		require.NoError(t, err)
	}

	require.Equal(t, 11, tree.nodeSize(), "Size of tree unexpected")
}

func TestMutableTree_DeleteVersionsTo(t *testing.T) {
	tree := setupMutableTree(false)

	type entry struct {
		key   []byte
		value []byte
	}

	versionEntries := make(map[int64][]entry)

	// create 10 tree versions, each with 1000 random key/value entries
	for i := 0; i < 10; i++ {
		entries := make([]entry, 1000)

		for j := 0; j < 1000; j++ {
			k := iavlrand.RandBytes(10)
			v := iavlrand.RandBytes(10)

			entries[j] = entry{k, v}
			_, err := tree.Set(k, v)
			require.NoError(t, err)
		}

		_, v, err := tree.SaveVersion()
		require.NoError(t, err)

		versionEntries[v] = entries
	}

	// delete even versions
	versionToDelete := int64(8)
	require.NoError(t, tree.DeleteVersionsTo(versionToDelete))

	// ensure even versions have been deleted
	for v := int64(1); v <= versionToDelete; v++ {
		_, err := tree.LoadVersion(v)
		require.Error(t, err)
	}

	// ensure odd number versions exist and we can query for all set entries
	for _, v := range []int64{9, 10} {
		_, err := tree.LoadVersion(v)
		require.NoError(t, err)

		for _, e := range versionEntries[v] {
			val, err := tree.Get(e.key)
			require.NoError(t, err)
			if !bytes.Equal(e.value, val) {
				t.Log(val)
			}
			// require.Equal(t, e.value, val)
		}
	}
}

func TestMutableTree_LoadVersion_Empty(t *testing.T) {
	tree := setupMutableTree(false)

	version, err := tree.LoadVersion(0)
	require.NoError(t, err)
	assert.EqualValues(t, 0, version)

	version, err = tree.LoadVersion(-1)
	require.NoError(t, err)
	assert.EqualValues(t, 0, version)

	_, err = tree.LoadVersion(3)
	require.Error(t, err)
}

func TestMutableTree_InitialVersion(t *testing.T) {
	memDB := dbm.NewMemDB()
	tree := NewMutableTree(memDB, 0, false, NewNopLogger(), InitialVersionOption(9))

	_, err := tree.Set([]byte("a"), []byte{0x01})
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)
	assert.EqualValues(t, 9, version)

	_, err = tree.Set([]byte("b"), []byte{0x02})
	require.NoError(t, err)
	_, version, err = tree.SaveVersion()
	require.NoError(t, err)
	assert.EqualValues(t, 10, version)

	// Reloading the tree with the same initial version is fine
	tree = NewMutableTree(memDB, 0, false, NewNopLogger(), InitialVersionOption(9))
	version, err = tree.Load()
	require.NoError(t, err)
	assert.EqualValues(t, 10, version)

	// Reloading the tree with an initial version beyond the lowest should error
	tree = NewMutableTree(memDB, 0, false, NewNopLogger(), InitialVersionOption(10))
	_, err = tree.Load()
	require.Error(t, err)

	// Reloading the tree with a lower initial version is fine, and new versions can be produced
	tree = NewMutableTree(memDB, 0, false, NewNopLogger(), InitialVersionOption(3))
	version, err = tree.Load()
	require.NoError(t, err)
	assert.EqualValues(t, 10, version)

	_, err = tree.Set([]byte("c"), []byte{0x03})
	require.NoError(t, err)
	_, version, err = tree.SaveVersion()
	require.NoError(t, err)
	assert.EqualValues(t, 11, version)
}

func TestMutableTree_SetInitialVersion(t *testing.T) {
	tree := setupMutableTree(false)
	tree.SetInitialVersion(9)

	_, err := tree.Set([]byte("a"), []byte{0x01})
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)
	assert.EqualValues(t, 9, version)
}

func BenchmarkMutableTree_Set(b *testing.B) {
	db := dbm.NewMemDB()
	t := NewMutableTree(db, 100000, false, NewNopLogger())
	for i := 0; i < 1000000; i++ {
		_, err := t.Set(iavlrand.RandBytes(10), []byte{})
		require.NoError(b, err)
	}
	b.ReportAllocs()
	runtime.GC()

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_, err := t.Set(iavlrand.RandBytes(10), []byte{})
		require.NoError(b, err)
	}
}

func prepareTree(t *testing.T) *MutableTree {
	mdb := dbm.NewMemDB()
	tree := NewMutableTree(mdb, 1000, false, NewNopLogger())
	for i := 0; i < 100; i++ {
		_, err := tree.Set([]byte{byte(i)}, []byte("a"))
		require.NoError(t, err)
	}
	_, ver, err := tree.SaveVersion()
	require.True(t, ver == 1)
	require.NoError(t, err)
	for i := 0; i < 100; i++ {
		_, err = tree.Set([]byte{byte(i)}, []byte("b"))
		require.NoError(t, err)
	}
	_, ver, err = tree.SaveVersion()
	require.True(t, ver == 2)
	require.NoError(t, err)

	newTree := NewMutableTree(mdb, 1000, false, NewNopLogger())

	return newTree
}

func TestMutableTree_Version(t *testing.T) {
	tree := prepareTree(t)
	require.True(t, tree.VersionExists(1))
	require.True(t, tree.VersionExists(2))
	require.False(t, tree.VersionExists(3))

	v, err := tree.GetLatestVersion()
	require.NoError(t, err)
	require.Equal(t, int64(2), v)
}

func checkGetVersioned(t *testing.T, tree *MutableTree, version int64, key, value []byte) {
	val, err := tree.GetVersioned(key, version)
	require.NoError(t, err)
	require.True(t, bytes.Equal(val, value))
}

func TestMutableTree_GetVersioned(t *testing.T) {
	tree := prepareTree(t)
	ver, err := tree.LoadVersion(1)
	require.True(t, ver == 2)
	require.NoError(t, err)
	// check key of unloaded version
	checkGetVersioned(t, tree, 1, []byte{1}, []byte("a"))
	checkGetVersioned(t, tree, 2, []byte{1}, []byte("b"))
	checkGetVersioned(t, tree, 3, []byte{1}, nil)

	tree = prepareTree(t)
	ver, err = tree.LoadVersion(2)
	require.True(t, ver == 2)
	require.NoError(t, err)
	checkGetVersioned(t, tree, 1, []byte{1}, []byte("a"))
	checkGetVersioned(t, tree, 2, []byte{1}, []byte("b"))
	checkGetVersioned(t, tree, 3, []byte{1}, nil)
}

func TestMutableTree_DeleteVersion(t *testing.T) {
	tree := prepareTree(t)
	ver, err := tree.LoadVersion(2)
	require.True(t, ver == 2)
	require.NoError(t, err)

	require.NoError(t, tree.DeleteVersionsTo(1))

	require.False(t, tree.VersionExists(1))
	require.True(t, tree.VersionExists(2))
	require.False(t, tree.VersionExists(3))

	// cannot delete latest version
	require.Error(t, tree.DeleteVersionsTo(2))
}

func TestMutableTree_LazyLoadVersionWithEmptyTree(t *testing.T) {
	mdb := dbm.NewMemDB()
	tree := NewMutableTree(mdb, 1000, false, NewNopLogger())
	_, v1, err := tree.SaveVersion()
	require.NoError(t, err)

	newTree1 := NewMutableTree(mdb, 1000, false, NewNopLogger())
	v2, err := newTree1.LoadVersion(1)
	require.NoError(t, err)
	require.True(t, v1 == v2)

	newTree2 := NewMutableTree(mdb, 1000, false, NewNopLogger())
	v2, err = newTree1.LoadVersion(1)
	require.NoError(t, err)
	require.True(t, v1 == v2)

	require.True(t, newTree1.root == newTree2.root)
}

func TestMutableTree_SetSimple(t *testing.T) {
	mdb := dbm.NewMemDB()
	tree := NewMutableTree(mdb, 0, false, NewNopLogger())

	const testKey1 = "a"
	const testVal1 = "test"

	isUpdated, err := tree.Set([]byte(testKey1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, isUpdated)

	fastValue, err := tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err := tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)

	require.Equal(t, []byte(testVal1), fastValue)
	require.Equal(t, []byte(testVal1), regularValue)

	fastNodeAdditions := tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 1, len(fastNodeAdditions))

	fastNodeAddition := fastNodeAdditions[testKey1]
	require.Equal(t, []byte(testKey1), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal1), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())
}

func TestMutableTree_SetTwoKeys(t *testing.T) {
	tree := setupMutableTree(false)

	const testKey1 = "a"
	const testVal1 = "test"

	const testKey2 = "b"
	const testVal2 = "test2"

	isUpdated, err := tree.Set([]byte(testKey1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, isUpdated)

	isUpdated, err = tree.Set([]byte(testKey2), []byte(testVal2))
	require.NoError(t, err)
	require.False(t, isUpdated)

	fastValue, err := tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err := tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal1), fastValue)
	require.Equal(t, []byte(testVal1), regularValue)

	fastValue2, err := tree.Get([]byte(testKey2))
	require.NoError(t, err)
	_, regularValue2, err := tree.GetWithIndex([]byte(testKey2))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal2), fastValue2)
	require.Equal(t, []byte(testVal2), regularValue2)

	fastNodeAdditions := tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 2, len(fastNodeAdditions))

	fastNodeAddition := fastNodeAdditions[testKey1]
	require.Equal(t, []byte(testKey1), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal1), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())

	fastNodeAddition = fastNodeAdditions[testKey2]
	require.Equal(t, []byte(testKey2), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal2), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())
}

func TestMutableTree_SetOverwrite(t *testing.T) {
	tree := setupMutableTree(false)
	const testKey1 = "a"
	const testVal1 = "test"
	const testVal2 = "test2"

	isUpdated, err := tree.Set([]byte(testKey1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, isUpdated)

	isUpdated, err = tree.Set([]byte(testKey1), []byte(testVal2))
	require.NoError(t, err)
	require.True(t, isUpdated)

	fastValue, err := tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err := tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal2), fastValue)
	require.Equal(t, []byte(testVal2), regularValue)

	fastNodeAdditions := tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 1, len(fastNodeAdditions))

	fastNodeAddition := fastNodeAdditions[testKey1]
	require.Equal(t, []byte(testKey1), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal2), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())
}

func TestMutableTree_SetRemoveSet(t *testing.T) {
	tree := setupMutableTree(false)
	const testKey1 = "a"
	const testVal1 = "test"

	// Set 1
	isUpdated, err := tree.Set([]byte(testKey1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, isUpdated)

	fastValue, err := tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err := tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal1), fastValue)
	require.Equal(t, []byte(testVal1), regularValue)

	fastNodeAdditions := tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 1, len(fastNodeAdditions))

	fastNodeAddition := fastNodeAdditions[testKey1]
	require.Equal(t, []byte(testKey1), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal1), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())

	// Remove
	removedVal, isRemoved, err := tree.Remove([]byte(testKey1))
	require.NoError(t, err)
	require.NotNil(t, removedVal)
	require.True(t, isRemoved)

	fastNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 0, len(fastNodeAdditions))

	fastNodeRemovals := tree.getUnsavedFastNodeRemovals()
	require.Equal(t, 1, len(fastNodeRemovals))

	fastValue, err = tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err = tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)
	require.Nil(t, fastValue)
	require.Nil(t, regularValue)

	// Set 2
	isUpdated, err = tree.Set([]byte(testKey1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, isUpdated)

	fastValue, err = tree.Get([]byte(testKey1))
	require.NoError(t, err)
	_, regularValue, err = tree.GetWithIndex([]byte(testKey1))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal1), fastValue)
	require.Equal(t, []byte(testVal1), regularValue)

	fastNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, 1, len(fastNodeAdditions))

	fastNodeAddition = fastNodeAdditions[testKey1]
	require.Equal(t, []byte(testKey1), fastNodeAddition.GetKey())
	require.Equal(t, []byte(testVal1), fastNodeAddition.GetValue())
	require.Equal(t, int64(1), fastNodeAddition.GetVersionLastUpdatedAt())

	fastNodeRemovals = tree.getUnsavedFastNodeRemovals()
	require.Equal(t, 0, len(fastNodeRemovals))
}

func TestMutableTree_FastNodeIntegration(t *testing.T) {
	mdb := dbm.NewMemDB()
	tree := NewMutableTree(mdb, 1000, false, NewNopLogger())

	const key1 = "a"
	const key2 = "b"
	const key3 = "c"

	const testVal1 = "test"
	const testVal2 = "test2"

	// Set key1
	res, err := tree.Set([]byte(key1), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, res)

	unsavedNodeAdditions := tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 1)

	// Set key2
	res, err = tree.Set([]byte(key2), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, res)

	unsavedNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 2)

	// Set key3
	res, err = tree.Set([]byte(key3), []byte(testVal1))
	require.NoError(t, err)
	require.False(t, res)

	unsavedNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 3)

	// Set key3 with new value
	res, err = tree.Set([]byte(key3), []byte(testVal2))
	require.NoError(t, err)
	require.True(t, res)

	unsavedNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 3)

	// Remove key2
	removedVal, isRemoved, err := tree.Remove([]byte(key2))
	require.NoError(t, err)
	require.True(t, isRemoved)
	require.Equal(t, []byte(testVal1), removedVal)

	unsavedNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 2)

	unsavedNodeRemovals := tree.getUnsavedFastNodeRemovals()
	require.Equal(t, len(unsavedNodeRemovals), 1)

	// Save
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	unsavedNodeAdditions = tree.getUnsavedFastNodeAdditions()
	require.Equal(t, len(unsavedNodeAdditions), 0)

	unsavedNodeRemovals = tree.getUnsavedFastNodeRemovals()
	require.Equal(t, len(unsavedNodeRemovals), 0)

	// Load
	t2 := NewMutableTree(mdb, 0, false, NewNopLogger())

	_, err = t2.Load()
	require.NoError(t, err)

	// Get and GetFast
	fastValue, err := t2.Get([]byte(key1))
	require.NoError(t, err)
	_, regularValue, err := tree.GetWithIndex([]byte(key1))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal1), fastValue)
	require.Equal(t, []byte(testVal1), regularValue)

	fastValue, err = t2.Get([]byte(key2))
	require.NoError(t, err)
	_, regularValue, err = t2.GetWithIndex([]byte(key2))
	require.NoError(t, err)
	require.Nil(t, fastValue)
	require.Nil(t, regularValue)

	fastValue, err = t2.Get([]byte(key3))
	require.NoError(t, err)
	_, regularValue, err = tree.GetWithIndex([]byte(key3))
	require.NoError(t, err)
	require.Equal(t, []byte(testVal2), fastValue)
	require.Equal(t, []byte(testVal2), regularValue)
}

func TestIterate_MutableTree_Unsaved(t *testing.T) {
	tree, mirror := getRandomizedTreeAndMirror(t)
	assertMutableMirrorIterate(t, tree, mirror)
}

func TestIterate_MutableTree_Saved(t *testing.T) {
	tree, mirror := getRandomizedTreeAndMirror(t)

	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	assertMutableMirrorIterate(t, tree, mirror)
}

func TestIterate_MutableTree_Unsaved_NextVersion(t *testing.T) {
	tree, mirror := getRandomizedTreeAndMirror(t)

	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	assertMutableMirrorIterate(t, tree, mirror)

	randomizeTreeAndMirror(t, tree, mirror)

	assertMutableMirrorIterate(t, tree, mirror)
}

func TestIterator_MutableTree_Invalid(t *testing.T) {
	tree := getTestTree(0)

	itr, err := tree.Iterator([]byte("a"), []byte("b"), true)
	require.NoError(t, err)
	require.NotNil(t, itr)
	require.False(t, itr.Valid())
}

func TestUpgradeStorageToFast_LatestVersion_Success(t *testing.T) {
	// Setup
	db := dbm.NewMemDB()
	tree := NewMutableTree(db, 1000, false, NewNopLogger())

	// Default version when storage key does not exist in the db
	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	mirror := make(map[string]string)
	// Fill with some data
	randomizeTreeAndMirror(t, tree, mirror)

	// Enable fast storage
	isUpgradeable, err := tree.IsUpgradeable()
	require.True(t, isUpgradeable)
	require.NoError(t, err)
	enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.True(t, enabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
}

func TestUpgradeStorageToFast_AlreadyUpgraded_Success(t *testing.T) {
	// Setup
	db := dbm.NewMemDB()
	tree := NewMutableTree(db, 1000, false, NewNopLogger())

	// Default version when storage key does not exist in the db
	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	mirror := make(map[string]string)
	// Fill with some data
	randomizeTreeAndMirror(t, tree, mirror)

	// Enable fast storage
	isUpgradeable, err := tree.IsUpgradeable()
	require.True(t, isUpgradeable)
	require.NoError(t, err)
	enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.True(t, enabled)
	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	// Test enabling fast storage when already enabled
	enabled, err = tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.False(t, enabled)
	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
}

func TestUpgradeStorageToFast_DbErrorConstructor_Failure(t *testing.T) {
	ctrl := gomock.NewController(t)
	dbMock := mock.NewMockDB(ctrl)
	rIterMock := mock.NewMockIterator(ctrl)

	// rIterMock is used to get the latest version from disk. We are mocking that rIterMock returns latestTreeVersion from disk
	rIterMock.EXPECT().Valid().Return(true).Times(1)
	rIterMock.EXPECT().Key().Return(nodeKeyFormat.Key(GetRootKey(1)))
	rIterMock.EXPECT().Close().Return(nil).Times(1)

	expectedError := errors.New("some db error")

	dbMock.EXPECT().Get(gomock.Any()).Return(nil, expectedError).Times(1)
	dbMock.EXPECT().NewBatchWithSize(gomock.Any()).Return(nil).Times(1)
	dbMock.EXPECT().ReverseIterator(gomock.Any(), gomock.Any()).Return(rIterMock, nil).Times(1)

	tree := NewMutableTree(dbMock, 0, false, NewNopLogger())
	require.NotNil(t, tree)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
}

func TestUpgradeStorageToFast_DbErrorEnableFastStorage_Failure(t *testing.T) {
	ctrl := gomock.NewController(t)
	dbMock := mock.NewMockDB(ctrl)
	rIterMock := mock.NewMockIterator(ctrl)

	// rIterMock is used to get the latest version from disk. We are mocking that rIterMock returns latestTreeVersion from disk
	rIterMock.EXPECT().Valid().Return(true).Times(1)
	rIterMock.EXPECT().Key().Return(nodeKeyFormat.Key(GetRootKey(1)))
	rIterMock.EXPECT().Close().Return(nil).Times(1)

	expectedError := errors.New("some db error")

	batchMock := mock.NewMockBatch(ctrl)

	dbMock.EXPECT().Get(gomock.Any()).Return(nil, nil).Times(1)
	dbMock.EXPECT().NewBatchWithSize(gomock.Any()).Return(batchMock).Times(1)
	dbMock.EXPECT().ReverseIterator(gomock.Any(), gomock.Any()).Return(rIterMock, nil).Times(1)

	iterMock := mock.NewMockIterator(ctrl)
	dbMock.EXPECT().Iterator(gomock.Any(), gomock.Any()).Return(iterMock, nil)
	iterMock.EXPECT().Error()
	iterMock.EXPECT().Valid().Times(2)
	iterMock.EXPECT().Close()

	batchMock.EXPECT().Set(gomock.Any(), gomock.Any()).Return(expectedError).Times(1)
	batchMock.EXPECT().GetByteSize().Return(100, nil).Times(1)

	tree := NewMutableTree(dbMock, 0, false, NewNopLogger())
	require.NotNil(t, tree)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
	require.ErrorIs(t, err, expectedError)
	require.False(t, enabled)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
}

func TestFastStorageReUpgradeProtection_NoForceUpgrade_Success(t *testing.T) {
	ctrl := gomock.NewController(t)
	dbMock := mock.NewMockDB(ctrl)
	rIterMock := mock.NewMockIterator(ctrl)

	// We are trying to test downgrade and re-upgrade protection
	// We need to set up a state where latest fast storage version is equal to latest tree version
	const latestFastStorageVersionOnDisk = 1
	const latestTreeVersion = latestFastStorageVersionOnDisk

	// Setup fake reverse iterator db to traverse root versions, called by ndb's getLatestVersion
	expectedStorageVersion := []byte(fastStorageVersionValue + fastStorageVersionDelimiter + strconv.Itoa(latestFastStorageVersionOnDisk))

	// rIterMock is used to get the latest version from disk. We are mocking that rIterMock returns latestTreeVersion from disk
	rIterMock.EXPECT().Valid().Return(true).Times(1)
	rIterMock.EXPECT().Key().Return(nodeKeyFormat.Key(GetRootKey(1)))
	rIterMock.EXPECT().Close().Return(nil).Times(1)

	batchMock := mock.NewMockBatch(ctrl)

	dbMock.EXPECT().Get(gomock.Any()).Return(expectedStorageVersion, nil).Times(1)
	dbMock.EXPECT().NewBatchWithSize(gomock.Any()).Return(batchMock).Times(1)
	dbMock.EXPECT().ReverseIterator(gomock.Any(), gomock.Any()).Return(rIterMock, nil).Times(1) // called to get latest version

	tree := NewMutableTree(dbMock, 0, false, NewNopLogger())
	require.NotNil(t, tree)

	// Pretend that we called Load and have the latest state in the tree
	tree.version = latestTreeVersion
	latestVersion, err := tree.ndb.getLatestVersion()
	require.NoError(t, err)
	require.Equal(t, latestVersion, int64(latestTreeVersion))

	// Ensure that the right branch of enableFastStorageAndCommitIfNotEnabled will be triggered
	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
	shouldForce, err := tree.ndb.shouldForceFastStorageUpgrade()
	require.False(t, shouldForce)
	require.NoError(t, err)

	enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.False(t, enabled)
}

func TestFastStorageReUpgradeProtection_ForceUpgradeFirstTime_NoForceSecondTime_Success(t *testing.T) {
	ctrl := gomock.NewController(t)
	dbMock := mock.NewMockDB(ctrl)
	batchMock := mock.NewMockBatch(ctrl)
	iterMock := mock.NewMockIterator(ctrl)
	rIterMock := mock.NewMockIterator(ctrl)

	// We are trying to test downgrade and re-upgrade protection
	// We need to set up a state where latest fast storage version is of a lower version
	// than tree version
	const latestFastStorageVersionOnDisk = 1
	const latestTreeVersion = latestFastStorageVersionOnDisk + 1

	// Setup db for iterator and reverse iterator mocks
	expectedStorageVersion := []byte(fastStorageVersionValue + fastStorageVersionDelimiter + strconv.Itoa(latestFastStorageVersionOnDisk))

	// Setup fake reverse iterator db to traverse root versions, called by ndb's getLatestVersion
	// rItr, err := db.ReverseIterator(rootKeyFormat.Key(1), rootKeyFormat.Key(latestTreeVersion + 1))
	// require.NoError(t, err)

	// dbMock represents the underlying database under the hood of nodeDB
	dbMock.EXPECT().Get(gomock.Any()).Return(expectedStorageVersion, nil).Times(1)

	dbMock.EXPECT().NewBatchWithSize(gomock.Any()).Return(batchMock).Times(2)
	dbMock.EXPECT().ReverseIterator(gomock.Any(), gomock.Any()).Return(rIterMock, nil).Times(1) // called to get latest version
	startFormat := fastKeyFormat.Key()
	endFormat := fastKeyFormat.Key()
	endFormat[0]++
	dbMock.EXPECT().Iterator(startFormat, endFormat).Return(iterMock, nil).Times(1)

	// rIterMock is used to get the latest version from disk. We are mocking that rIterMock returns latestTreeVersion from disk
	rIterMock.EXPECT().Valid().Return(true).Times(1)
	rIterMock.EXPECT().Key().Return(nodeKeyFormat.Key(GetRootKey(latestTreeVersion)))
	rIterMock.EXPECT().Close().Return(nil).Times(1)

	fastNodeKeyToDelete := []byte("some_key")

	// batchMock represents a structure that receives all the updates related to
	// upgrade and then commits them all in the end.
	updatedExpectedStorageVersion := make([]byte, len(expectedStorageVersion))
	copy(updatedExpectedStorageVersion, expectedStorageVersion)
	updatedExpectedStorageVersion[len(updatedExpectedStorageVersion)-1]++
	batchMock.EXPECT().GetByteSize().Return(100, nil).Times(2)
	batchMock.EXPECT().Delete(fastKeyFormat.Key(fastNodeKeyToDelete)).Return(nil).Times(1)
	batchMock.EXPECT().Set(metadataKeyFormat.Key([]byte(storageVersionKey)), updatedExpectedStorageVersion).Return(nil).Times(1)
	batchMock.EXPECT().Write().Return(nil).Times(1)
	batchMock.EXPECT().Close().Return(nil).Times(1)

	// iterMock is used to mock the underlying db iterator behing fast iterator
	// Here, we want to mock the behavior of deleting fast nodes from disk when
	// force upgrade is detected.
	iterMock.EXPECT().Valid().Return(true).Times(1)
	iterMock.EXPECT().Error().Return(nil).Times(1)
	iterMock.EXPECT().Key().Return(fastKeyFormat.Key(fastNodeKeyToDelete)).Times(1)
	// encode value
	var buf bytes.Buffer
	testValue := "test_value"
	buf.Grow(encoding.EncodeVarintSize(int64(latestFastStorageVersionOnDisk)) + encoding.EncodeBytesSize([]byte(testValue)))
	err := encoding.EncodeVarint(&buf, int64(latestFastStorageVersionOnDisk))
	require.NoError(t, err)
	err = encoding.EncodeBytes(&buf, []byte(testValue))
	require.NoError(t, err)
	iterMock.EXPECT().Value().Return(buf.Bytes()).Times(1) // this is encoded as version 1 with value "2"
	iterMock.EXPECT().Valid().Return(true).Times(1)
	// Call Next at the end of loop iteration
	iterMock.EXPECT().Next().Return().Times(1)
	iterMock.EXPECT().Error().Return(nil).Times(1)
	iterMock.EXPECT().Valid().Return(false).Times(1)
	// Call Valid after first iteraton
	iterMock.EXPECT().Valid().Return(false).Times(1)
	iterMock.EXPECT().Close().Return(nil).Times(1)

	tree := NewMutableTree(dbMock, 0, false, NewNopLogger())
	require.NotNil(t, tree)

	// Pretend that we called Load and have the latest state in the tree
	tree.version = latestTreeVersion
	latestVersion, err := tree.ndb.getLatestVersion()
	require.NoError(t, err)
	require.Equal(t, latestVersion, int64(latestTreeVersion))

	// Ensure that the right branch of enableFastStorageAndCommitIfNotEnabled will be triggered
	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
	shouldForce, err := tree.ndb.shouldForceFastStorageUpgrade()
	require.True(t, shouldForce)
	require.NoError(t, err)

	// Actual method under test
	enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.True(t, enabled)

	// Test that second time we call this, force upgrade does not happen
	enabled, err = tree.enableFastStorageAndCommitIfNotEnabled()
	require.NoError(t, err)
	require.False(t, enabled)
}

func TestUpgradeStorageToFast_Integration_Upgraded_FastIterator_Success(t *testing.T) {
	// Setup
	tree, mirror := setupTreeAndMirror(t, 100, false)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err := tree.IsUpgradeable()
	require.True(t, isUpgradeable)
	require.NoError(t, err)

	// Should auto enable in save version
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	sut := NewMutableTree(tree.ndb.db, 1000, false, NewNopLogger())

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = sut.IsUpgradeable()
	require.False(t, isUpgradeable) // upgraded in save version
	require.NoError(t, err)

	// Load version - should auto enable fast storage
	version, err := sut.Load()
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)

	require.Equal(t, int64(1), version)

	// Test that upgraded mutable tree iterates as expected
	t.Run("Mutable tree", func(t *testing.T) {
		i := 0
		sut.Iterate(func(k, v []byte) bool { //nolint:errcheck
			require.Equal(t, []byte(mirror[i][0]), k)
			require.Equal(t, []byte(mirror[i][1]), v)
			i++
			return false
		})
	})

	// Test that upgraded immutable tree iterates as expected
	t.Run("Immutable tree", func(t *testing.T) {
		immutableTree, err := sut.GetImmutable(sut.version)
		require.NoError(t, err)

		i := 0
		immutableTree.Iterate(func(k, v []byte) bool { //nolint:errcheck
			require.Equal(t, []byte(mirror[i][0]), k)
			require.Equal(t, []byte(mirror[i][1]), v)
			i++
			return false
		})
	})
}

func TestUpgradeStorageToFast_Integration_Upgraded_GetFast_Success(t *testing.T) {
	// Setup
	tree, mirror := setupTreeAndMirror(t, 100, false)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err := tree.IsUpgradeable()
	require.True(t, isUpgradeable)
	require.NoError(t, err)

	// Should auto enable in save version
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	sut := NewMutableTree(tree.ndb.db, 1000, false, NewNopLogger())

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = sut.IsUpgradeable()
	require.False(t, isUpgradeable) // upgraded in save version
	require.NoError(t, err)

	// LazyLoadVersion - should auto enable fast storage
	version, err := sut.LoadVersion(1)
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.True(t, isFastCacheEnabled)

	require.Equal(t, int64(1), version)

	t.Run("Mutable tree", func(t *testing.T) {
		for _, kv := range mirror {
			v, err := sut.Get([]byte(kv[0]))
			require.NoError(t, err)
			require.Equal(t, []byte(kv[1]), v)
		}
	})

	t.Run("Immutable tree", func(t *testing.T) {
		immutableTree, err := sut.GetImmutable(sut.version)
		require.NoError(t, err)

		for _, kv := range mirror {
			v, err := immutableTree.Get([]byte(kv[0]))
			require.NoError(t, err)
			require.Equal(t, []byte(kv[1]), v)
		}
	})
}

func TestUpgradeStorageToFast_Success(t *testing.T) {
	commitGap := 1000

	type fields struct {
		nodeCount int
	}
	tests := []struct {
		name   string
		fields fields
	}{
		{"less than commit gap", fields{nodeCount: 100}},
		{"equal to commit gap", fields{nodeCount: commitGap}},
		{"great than commit gap", fields{nodeCount: commitGap + 100}},
		{"two times commit gap", fields{nodeCount: commitGap * 2}},
		{"two times plus commit gap", fields{nodeCount: commitGap*2 + 1}},
	}

	for _, tt := range tests {
		tree, mirror := setupTreeAndMirror(t, tt.fields.nodeCount, false)
		enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
		require.Nil(t, err)
		require.True(t, enabled)
		t.Run(tt.name, func(t *testing.T) {
			i := 0
			iter := NewFastIterator(nil, nil, true, tree.ndb)
			for ; iter.Valid(); iter.Next() {
				require.Equal(t, []byte(mirror[i][0]), iter.Key())
				require.Equal(t, []byte(mirror[i][1]), iter.Value())
				i++
			}
			require.Equal(t, len(mirror), i)
		})
	}
}

func TestUpgradeStorageToFast_Delete_Stale_Success(t *testing.T) {
	// we delete fast node, in case of deadlock. we should limit the stale count lower than chBufferSize(64)
	commitGap := 5

	valStale := "val_stale"
	addStaleKey := func(ndb *nodeDB, staleCount int) {
		keyPrefix := "key_prefix"
		b := ndb.db.NewBatch()
		for i := 0; i < staleCount; i++ {
			key := fmt.Sprintf("%s_%d", keyPrefix, i)

			node := fastnode.NewNode([]byte(key), []byte(valStale), 100)
			var buf bytes.Buffer
			buf.Grow(node.EncodedSize())
			err := node.WriteBytes(&buf)
			require.NoError(t, err)
			err = b.Set(ndb.fastNodeKey([]byte(key)), buf.Bytes())
			require.NoError(t, err)
		}
		require.NoError(t, b.Write())
	}
	type fields struct {
		nodeCount  int
		staleCount int
	}

	tests := []struct {
		name   string
		fields fields
	}{
		{"stale less than commit gap", fields{nodeCount: 100, staleCount: 4}},
		{"stale equal to commit gap", fields{nodeCount: commitGap, staleCount: commitGap}},
		{"stale great than commit gap", fields{nodeCount: commitGap + 100, staleCount: commitGap*2 - 1}},
		{"stale twice commit gap", fields{nodeCount: commitGap + 100, staleCount: commitGap * 2}},
		{"stale great than twice commit gap", fields{nodeCount: commitGap, staleCount: commitGap*2 + 1}},
	}

	for _, tt := range tests {
		tree, mirror := setupTreeAndMirror(t, tt.fields.nodeCount, false)
		addStaleKey(tree.ndb, tt.fields.staleCount)
		enabled, err := tree.enableFastStorageAndCommitIfNotEnabled()
		require.Nil(t, err)
		require.True(t, enabled)
		t.Run(tt.name, func(t *testing.T) {
			i := 0
			iter := NewFastIterator(nil, nil, true, tree.ndb)
			for ; iter.Valid(); iter.Next() {
				require.Equal(t, []byte(mirror[i][0]), iter.Key())
				require.Equal(t, []byte(mirror[i][1]), iter.Value())
				i++
			}
			require.Equal(t, len(mirror), i)
		})
	}
}

func setupTreeAndMirror(t *testing.T, numEntries int, skipFastStorageUpgrade bool) (*MutableTree, [][]string) {
	db := dbm.NewMemDB()

	tree := NewMutableTree(db, 0, skipFastStorageUpgrade, NewNopLogger())

	keyPrefix, valPrefix := "key", "val"

	mirror := make([][]string, 0, numEntries)
	for i := 0; i < numEntries; i++ {
		key := fmt.Sprintf("%s_%d", keyPrefix, i)
		val := fmt.Sprintf("%s_%d", valPrefix, i)
		mirror = append(mirror, []string{key, val})
		updated, err := tree.Set([]byte(key), []byte(val))
		require.False(t, updated)
		require.NoError(t, err)
	}

	// Delete fast nodes from database to mimic a version with no upgrade
	for i := 0; i < numEntries; i++ {
		key := fmt.Sprintf("%s_%d", keyPrefix, i)
		require.NoError(t, db.Delete(fastKeyFormat.Key([]byte(key))))
	}

	sort.Slice(mirror, func(i, j int) bool {
		return mirror[i][0] < mirror[j][0]
	})
	return tree, mirror
}

func TestNoFastStorageUpgrade_Integration_SaveVersion_Load_Get_Success(t *testing.T) {
	// Setup
	tree, mirror := setupTreeAndMirror(t, 100, true)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err := tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	// Should Not auto enable in save version
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	sut := NewMutableTree(tree.ndb.db, 1000, true, NewNopLogger())

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = sut.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	// LazyLoadVersion - should not auto enable fast storage
	version, err := sut.LoadVersion(1)
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	// Load - should not auto enable fast storage
	version, err = sut.Load()
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	// LoadVersion - should not auto enable fast storage
	version, err = sut.LoadVersion(1)
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	// LoadVersionForOverwriting - should not auto enable fast storage
	err = sut.LoadVersionForOverwriting(1)
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	t.Run("Mutable tree", func(t *testing.T) {
		for _, kv := range mirror {
			v, err := sut.Get([]byte(kv[0]))
			require.NoError(t, err)
			require.Equal(t, []byte(kv[1]), v)
		}
	})

	t.Run("Immutable tree", func(t *testing.T) {
		immutableTree, err := sut.GetImmutable(sut.version)
		require.NoError(t, err)

		for _, kv := range mirror {
			v, err := immutableTree.Get([]byte(kv[0]))
			require.NoError(t, err)
			require.Equal(t, []byte(kv[1]), v)
		}
	})
}

func TestNoFastStorageUpgrade_Integration_SaveVersion_Load_Iterate_Success(t *testing.T) {
	// Setup
	tree, mirror := setupTreeAndMirror(t, 100, true)

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err := tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	// Should Not auto enable in save version
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = tree.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	sut := NewMutableTree(tree.ndb.db, 1000, true, NewNopLogger())

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)
	isUpgradeable, err = sut.IsUpgradeable()
	require.False(t, isUpgradeable)
	require.NoError(t, err)

	// Load - should not auto enable fast storage
	version, err := sut.Load()
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = sut.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	// Load - should not auto enable fast storage
	version, err = sut.Load()
	require.NoError(t, err)
	require.Equal(t, int64(1), version)

	isFastCacheEnabled, err = tree.IsFastCacheEnabled()
	require.NoError(t, err)
	require.False(t, isFastCacheEnabled)

	// Test that the mutable tree iterates as expected
	t.Run("Mutable tree", func(t *testing.T) {
		i := 0
		sut.Iterate(func(k, v []byte) bool { //nolint: errcheck
			require.Equal(t, []byte(mirror[i][0]), k)
			require.Equal(t, []byte(mirror[i][1]), v)
			i++
			return false
		})
	})

	// Test that the immutable tree iterates as expected
	t.Run("Immutable tree", func(t *testing.T) {
		immutableTree, err := sut.GetImmutable(sut.version)
		require.NoError(t, err)

		i := 0
		immutableTree.Iterate(func(k, v []byte) bool { //nolint: errcheck
			require.Equal(t, []byte(mirror[i][0]), k)
			require.Equal(t, []byte(mirror[i][1]), v)
			i++
			return false
		})
	})
}

// TestMutableTree_InitialVersion_FirstVersion demonstrate the un-intuitive behavior,
// when InitialVersion is set the nodes created in the first version are not assigned with expected version number.
func TestMutableTree_InitialVersion_FirstVersion(t *testing.T) {
	db := dbm.NewMemDB()

	initialVersion := int64(1000)
	tree := NewMutableTree(db, 0, true, NewNopLogger(), InitialVersionOption(uint64(initialVersion)))

	_, err := tree.Set([]byte("hello"), []byte("world"))
	require.NoError(t, err)

	_, version, err := tree.SaveVersion()
	require.NoError(t, err)
	require.Equal(t, initialVersion, version)
	rootKey := GetRootKey(version)
	// the nodes created at the first version are not assigned with the `InitialVersion`
	node, err := tree.ndb.GetNode(rootKey)
	require.NoError(t, err)
	require.Equal(t, initialVersion, node.nodeKey.version)

	_, err = tree.Set([]byte("hello"), []byte("world1"))
	require.NoError(t, err)

	_, version, err = tree.SaveVersion()
	require.NoError(t, err)
	require.Equal(t, initialVersion+1, version)
	rootKey = GetRootKey(version)
	// the following versions behaves normally
	node, err = tree.ndb.GetNode(rootKey)
	require.NoError(t, err)
	require.Equal(t, initialVersion+1, node.nodeKey.version)
}

func TestMutableTreeClose(t *testing.T) {
	db := dbm.NewMemDB()
	tree := NewMutableTree(db, 0, true, NewNopLogger())

	_, err := tree.Set([]byte("hello"), []byte("world"))
	require.NoError(t, err)

	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	require.NoError(t, tree.Close())
}
