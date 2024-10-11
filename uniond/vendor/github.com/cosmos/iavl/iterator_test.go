package iavl

import (
	"math/rand"
	"sort"
	"sync"
	"testing"

	corestore "cosmossdk.io/core/store"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

func TestIterator_NewIterator_NilTree_Failure(t *testing.T) {
	start, end := []byte{'a'}, []byte{'c'}
	ascending := true

	performTest := func(t *testing.T, itr corestore.Iterator) {
		require.NotNil(t, itr)
		require.False(t, itr.Valid())
		actualsStart, actualEnd := itr.Domain()
		require.Equal(t, start, actualsStart)
		require.Equal(t, end, actualEnd)
		require.Error(t, itr.Error())
	}

	t.Run("Iterator", func(t *testing.T) {
		itr := NewIterator(start, end, ascending, nil)
		performTest(t, itr)
		require.ErrorIs(t, errIteratorNilTreeGiven, itr.Error())
	})

	t.Run("Fast Iterator", func(t *testing.T) {
		itr := NewFastIterator(start, end, ascending, nil)
		performTest(t, itr)
		require.ErrorIs(t, errFastIteratorNilNdbGiven, itr.Error())
	})

	t.Run("Unsaved Fast Iterator", func(t *testing.T) {
		itr := NewUnsavedFastIterator(start, end, ascending, nil, &sync.Map{}, &sync.Map{})
		performTest(t, itr)
		require.ErrorIs(t, errFastIteratorNilNdbGiven, itr.Error())
	})
}

func TestUnsavedFastIterator_NewIterator_NilAdditions_Failure(t *testing.T) {
	start, end := []byte{'a'}, []byte{'c'}
	ascending := true

	performTest := func(t *testing.T, itr corestore.Iterator) {
		require.NotNil(t, itr)
		require.False(t, itr.Valid())
		actualsStart, actualEnd := itr.Domain()
		require.Equal(t, start, actualsStart)
		require.Equal(t, end, actualEnd)
		require.Error(t, itr.Error())
	}

	t.Run("Nil additions given", func(t *testing.T) {
		tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
		itr := NewUnsavedFastIterator(start, end, ascending, tree.ndb, nil, tree.unsavedFastNodeRemovals)
		performTest(t, itr)
		require.ErrorIs(t, errUnsavedFastIteratorNilAdditionsGiven, itr.Error())
	})

	t.Run("Nil removals given", func(t *testing.T) {
		tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
		itr := NewUnsavedFastIterator(start, end, ascending, tree.ndb, tree.unsavedFastNodeAdditions, nil)
		performTest(t, itr)
		require.ErrorIs(t, errUnsavedFastIteratorNilRemovalsGiven, itr.Error())
	})

	t.Run("All nil", func(t *testing.T) {
		itr := NewUnsavedFastIterator(start, end, ascending, nil, nil, nil)
		performTest(t, itr)
		require.ErrorIs(t, errFastIteratorNilNdbGiven, itr.Error())
	})

	t.Run("Additions and removals are nil", func(t *testing.T) {
		tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
		itr := NewUnsavedFastIterator(start, end, ascending, tree.ndb, nil, nil)
		performTest(t, itr)
		require.ErrorIs(t, errUnsavedFastIteratorNilAdditionsGiven, itr.Error())
	})
}

func TestIterator_Empty_Invalid(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   []byte("a"),
		endIterate:     []byte("a"),
		ascending:      true,
	}

	performTest := func(t *testing.T, itr corestore.Iterator, mirror [][]string) {
		require.Equal(t, 0, len(mirror))
		require.False(t, itr.Valid())
	}

	t.Run("Iterator", func(t *testing.T) {
		itr, mirror := setupIteratorAndMirror(t, config)
		performTest(t, itr, mirror)
	})

	t.Run("Fast Iterator", func(t *testing.T) {
		itr, mirror := setupFastIteratorAndMirror(t, config)
		performTest(t, itr, mirror)
	})

	t.Run("Unsaved Fast Iterator", func(t *testing.T) {
		itr, mirror := setupUnsavedFastIterator(t, config)
		performTest(t, itr, mirror)
	})
}

func TestIterator_Basic_Ranged_Ascending_Success(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   []byte("e"),
		endIterate:     []byte("w"),
		ascending:      true,
	}
	iteratorSuccessTest(t, config)
}

func TestIterator_Basic_Ranged_Descending_Success(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   []byte("e"),
		endIterate:     []byte("w"),
		ascending:      false,
	}
	iteratorSuccessTest(t, config)
}

func TestIterator_Basic_Full_Ascending_Success(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   nil,
		endIterate:     nil,
		ascending:      true,
	}

	iteratorSuccessTest(t, config)
}

func TestIterator_Basic_Full_Descending_Success(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   nil,
		endIterate:     nil,
		ascending:      false,
	}
	iteratorSuccessTest(t, config)
}

func TestIterator_WithDelete_Full_Ascending_Success(t *testing.T) {
	config := &iteratorTestConfig{
		startByteToSet: 'a',
		endByteToSet:   'z',
		startIterate:   nil,
		endIterate:     nil,
		ascending:      false,
	}

	tree, mirror := getRandomizedTreeAndMirror(t)

	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	randomizeTreeAndMirror(t, tree, mirror)

	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	err = tree.DeleteVersionsTo(1)
	require.NoError(t, err)

	latestVersion, err := tree.ndb.getLatestVersion()
	require.NoError(t, err)
	immutableTree, err := tree.GetImmutable(latestVersion)
	require.NoError(t, err)

	// sort mirror for assertion
	sortedMirror := make([][]string, 0, len(mirror))
	for k, v := range mirror {
		sortedMirror = append(sortedMirror, []string{k, v})
	}

	sort.Slice(sortedMirror, func(i, j int) bool {
		return sortedMirror[i][0] > sortedMirror[j][0]
	})

	t.Run("Iterator", func(t *testing.T) {
		itr := NewIterator(config.startIterate, config.endIterate, config.ascending, immutableTree)
		require.True(t, itr.Valid())
		assertIterator(t, itr, sortedMirror, config.ascending)
	})

	t.Run("Fast Iterator", func(t *testing.T) {
		itr := NewFastIterator(config.startIterate, config.endIterate, config.ascending, immutableTree.ndb)
		require.True(t, itr.Valid())
		assertIterator(t, itr, sortedMirror, config.ascending)
	})

	t.Run("Unsaved Fast Iterator", func(t *testing.T) {
		itr := NewUnsavedFastIterator(config.startIterate, config.endIterate, config.ascending, immutableTree.ndb, tree.unsavedFastNodeAdditions, tree.unsavedFastNodeRemovals)
		require.True(t, itr.Valid())
		assertIterator(t, itr, sortedMirror, config.ascending)
	})
}

func iteratorSuccessTest(t *testing.T, config *iteratorTestConfig) {
	performTest := func(t *testing.T, itr corestore.Iterator, mirror [][]string) {
		actualStart, actualEnd := itr.Domain()
		require.Equal(t, config.startIterate, actualStart)
		require.Equal(t, config.endIterate, actualEnd)

		require.NoError(t, itr.Error())

		assertIterator(t, itr, mirror, config.ascending)
	}

	t.Run("Iterator", func(t *testing.T) {
		itr, mirror := setupIteratorAndMirror(t, config)
		require.True(t, itr.Valid())
		performTest(t, itr, mirror)
	})

	t.Run("Fast Iterator", func(t *testing.T) {
		itr, mirror := setupFastIteratorAndMirror(t, config)
		require.True(t, itr.Valid())
		performTest(t, itr, mirror)
	})

	t.Run("Unsaved Fast Iterator", func(t *testing.T) {
		itr, mirror := setupUnsavedFastIterator(t, config)
		require.True(t, itr.Valid())
		performTest(t, itr, mirror)
	})
}

func setupIteratorAndMirror(t *testing.T, config *iteratorTestConfig) (corestore.Iterator, [][]string) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	mirror := setupMirrorForIterator(t, config, tree)
	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	latestVersion, err := tree.ndb.getLatestVersion()
	require.NoError(t, err)
	immutableTree, err := tree.GetImmutable(latestVersion)
	require.NoError(t, err)

	itr := NewIterator(config.startIterate, config.endIterate, config.ascending, immutableTree)
	return itr, mirror
}

func setupFastIteratorAndMirror(t *testing.T, config *iteratorTestConfig) (corestore.Iterator, [][]string) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	mirror := setupMirrorForIterator(t, config, tree)
	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	itr := NewFastIterator(config.startIterate, config.endIterate, config.ascending, tree.ndb)
	return itr, mirror
}

func setupUnsavedFastIterator(t *testing.T, config *iteratorTestConfig) (corestore.Iterator, [][]string) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	// For unsaved fast iterator, we would like to test the state where
	// there are saved fast nodes as well as some unsaved additions and removals.
	// So, we split the byte range in half where the first half is saved and the second half is unsaved.
	breakpointByte := (config.endByteToSet + config.startByteToSet) / 2

	firstHalfConfig := *config
	firstHalfConfig.endByteToSet = breakpointByte // exclusive

	secondHalfConfig := *config
	secondHalfConfig.startByteToSet = breakpointByte

	// First half of the mirror
	mirror := setupMirrorForIterator(t, &firstHalfConfig, tree)
	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	// No unsaved additions or removals should be present after saving
	require.Equal(t, 0, syncMapCount(tree.unsavedFastNodeAdditions))
	require.Equal(t, 0, syncMapCount(tree.unsavedFastNodeRemovals))

	// Ensure that there are unsaved additions and removals present
	secondHalfMirror := setupMirrorForIterator(t, &secondHalfConfig, tree)

	require.True(t, syncMapCount(tree.unsavedFastNodeAdditions) >= len(secondHalfMirror))
	require.Equal(t, 0, syncMapCount(tree.unsavedFastNodeRemovals))

	// Merge the two halves
	if config.ascending {
		mirror = append(mirror, secondHalfMirror...)
	} else {
		mirror = append(secondHalfMirror, mirror...)
	}

	if len(mirror) > 0 {
		// Remove random keys
		for i := 0; i < len(mirror)/4; i++ {
			randIndex := rand.Intn(len(mirror))
			keyToRemove := mirror[randIndex][0]

			_, removed, err := tree.Remove([]byte(keyToRemove))
			require.NoError(t, err)
			require.True(t, removed)

			mirror = append(mirror[:randIndex], mirror[randIndex+1:]...)
		}
	}

	itr := NewUnsavedFastIterator(config.startIterate, config.endIterate, config.ascending, tree.ndb, tree.unsavedFastNodeAdditions, tree.unsavedFastNodeRemovals)
	return itr, mirror
}

func TestNodeIterator_Success(t *testing.T) {
	tree, mirror := getRandomizedTreeAndMirror(t)

	_, _, err := tree.SaveVersion()
	require.NoError(t, err)

	randomizeTreeAndMirror(t, tree, mirror)

	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	// check if the iterating count is same with the entire node count of the tree
	itr, err := NewNodeIterator(tree.root.GetKey(), tree.ndb)
	require.NoError(t, err)
	nodeCount := 0
	for ; itr.Valid(); itr.Next(false) {
		nodeCount++
	}
	require.Equal(t, int64(nodeCount), tree.Size()*2-1)

	// check if the skipped node count is right
	itr, err = NewNodeIterator(tree.root.GetKey(), tree.ndb)
	require.NoError(t, err)
	updateCount := 0
	skipCount := 0
	for itr.Valid() {
		node := itr.GetNode()
		updateCount++
		if node.nodeKey.version < tree.Version() {
			skipCount += int(node.size*2 - 2) // the size of the subtree without the root
		}
		itr.Next(node.nodeKey.version < tree.Version())
	}
	require.Equal(t, nodeCount, updateCount+skipCount)
}

func TestNodeIterator_WithEmptyRoot(t *testing.T) {
	itr, err := NewNodeIterator(nil, newNodeDB(dbm.NewMemDB(), 0, DefaultOptions(), NewNopLogger()))
	require.NoError(t, err)
	require.False(t, itr.Valid())

	itr, err = NewNodeIterator([]byte{}, newNodeDB(dbm.NewMemDB(), 0, DefaultOptions(), NewNopLogger()))
	require.NoError(t, err)
	require.False(t, itr.Valid())
}

func syncMapCount(m *sync.Map) int {
	count := 0
	m.Range(func(_, _ interface{}) bool {
		count++
		return true
	})
	return count
}
