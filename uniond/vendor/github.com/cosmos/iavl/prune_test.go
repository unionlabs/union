package iavl

import (
	"fmt"
	"testing"
	"time"

	dbm "github.com/cosmos/iavl/db"
	"github.com/stretchr/testify/require"
)

func TestAsyncPruning(t *testing.T) {
	db, err := dbm.NewGoLevelDB("test", t.TempDir())
	require.NoError(t, err)
	defer db.Close()

	tree := NewMutableTree(db, 0, false, NewNopLogger(), AsyncPruningOption(true), FlushThresholdOption(1000))

	toVersion := 10000
	keyCount := 10
	pruneInterval := int64(100)
	keepRecent := int64(300)
	for i := 0; i < toVersion; i++ {
		for j := 0; j < keyCount; j++ {
			_, err := tree.Set([]byte(fmt.Sprintf("key-%d-%d", i, j)), []byte(fmt.Sprintf("value-%d-%d", i, j)))
			require.NoError(t, err)
		}

		tree.SetCommitting()
		_, v, err := tree.SaveVersion()
		require.NoError(t, err)
		tree.UnsetCommitting()

		if v%pruneInterval == 0 && v > keepRecent {
			ti := time.Now()
			require.NoError(t, tree.DeleteVersionsTo(v-keepRecent))
			t.Logf("Pruning %d versions took %v\n", keepRecent, time.Since(ti))
		}
	}

	// wait for async pruning to finish
	for i := 0; i < 100; i++ {
		tree.SetCommitting()
		_, _, err := tree.SaveVersion()
		require.NoError(t, err)
		tree.UnsetCommitting()

		firstVersion, err := tree.ndb.getFirstVersion()
		require.NoError(t, err)
		t.Logf("Iteration: %d First version: %d\n", i, firstVersion)
		if firstVersion == int64(toVersion)-keepRecent+1 {
			break
		}
		// simulate the consensus process
		time.Sleep(500 * time.Millisecond)
	}

	// Reload the tree
	tree = NewMutableTree(db, 0, false, NewNopLogger())
	_, err = tree.LoadVersion(int64(toVersion) - keepRecent)
	require.Error(t, err)
	versions := tree.AvailableVersions()
	require.Equal(t, versions[0], toVersion-int(keepRecent)+1)
	for _, v := range versions {
		_, err := tree.LoadVersion(int64(v))
		require.NoError(t, err)
	}
}
