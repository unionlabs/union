package iavl

import (
	"bytes"
	"fmt"
	"math/rand"
	"os"
	"os/exec"
	"path"
	"testing"

	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
	iavlrand "github.com/cosmos/iavl/internal/rand"
)

const (
	dbType = "goleveldb"
)

func createLegacyTree(t *testing.T, dbDir string, version int) (string, error) {
	relateDir := path.Join(t.TempDir(), dbDir)
	if _, err := os.Stat(relateDir); err == nil {
		err := os.RemoveAll(relateDir)
		if err != nil {
			t.Errorf("%+v\n", err)
		}
	}

	cmd := exec.Command("sh", "-c", fmt.Sprintf("./cmd/legacydump/legacydump %s %s random %d %d", dbType, relateDir, version, version/2)) //nolint:gosec
	var out bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &out
	cmd.Stderr = &stderr
	err := cmd.Run()
	if err != nil || stderr.Len() > 0 {
		t.Log(fmt.Sprint(err) + ": " + stderr.String())
		if err == nil {
			err = fmt.Errorf("stderr: %s", stderr.String())
		}
	}
	t.Log("Result: " + out.String())

	return relateDir, err
}

func TestLazySet(t *testing.T) {
	legacyVersion := 1000
	dbDir := fmt.Sprintf("legacy-%s-%d", dbType, legacyVersion)
	relateDir, err := createLegacyTree(t, dbDir, legacyVersion)
	require.NoError(t, err)

	db, err := dbm.NewGoLevelDB("test", relateDir)
	require.NoError(t, err)

	defer func() {
		if err := db.Close(); err != nil {
			t.Errorf("DB close error: %v\n", err)
		}
		if err := os.RemoveAll(relateDir); err != nil {
			t.Errorf("%+v\n", err)
		}
	}()

	tree := NewMutableTree(db, 1000, false, NewNopLogger())

	// Load the latest legacy version
	_, err = tree.LoadVersion(int64(legacyVersion))
	require.NoError(t, err)

	// Commit new versions
	postVersions := 1000
	for i := 0; i < postVersions; i++ {
		leafCount := rand.Intn(50)
		for j := 0; j < leafCount; j++ {
			_, err = tree.Set([]byte(fmt.Sprintf("key-%d-%d", i, j)), []byte(fmt.Sprintf("value-%d-%d", i, j)))
			require.NoError(t, err)
		}
		_, _, err = tree.SaveVersion()
		require.NoError(t, err)
	}

	tree = NewMutableTree(db, 1000, false, NewNopLogger())

	// Verify that the latest legacy version can still be loaded
	_, err = tree.LoadVersion(int64(legacyVersion))
	require.NoError(t, err)
}

func TestLegacyReferenceNode(t *testing.T) {
	legacyVersion := 20
	dbDir := fmt.Sprintf("./legacy-%s-%d", dbType, legacyVersion)
	relateDir, err := createLegacyTree(t, dbDir, legacyVersion)
	require.NoError(t, err)

	db, err := dbm.NewGoLevelDB("test", relateDir)
	require.NoError(t, err)

	defer func() {
		if err := db.Close(); err != nil {
			t.Errorf("DB close error: %v\n", err)
		}
		if err := os.RemoveAll(relateDir); err != nil {
			t.Errorf("%+v\n", err)
		}
	}()

	tree := NewMutableTree(db, 1000, false, NewNopLogger())

	// Load the latest legacy version
	_, err = tree.LoadVersion(int64(legacyVersion))
	require.NoError(t, err)
	legacyLatestVersion := tree.root.nodeKey.version

	// Commit new versions without updates
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)

	// Load the previous version
	newTree := NewMutableTree(db, 1000, false, NewNopLogger())
	_, err = newTree.LoadVersion(version - 1)
	require.NoError(t, err)
	// Check if the reference node is refactored
	require.Equal(t, newTree.root.nodeKey.nonce, uint32(0))
	require.Equal(t, newTree.root.nodeKey.version, legacyLatestVersion)
}

func TestDeleteVersions(t *testing.T) {
	legacyVersion := 100
	dbDir := fmt.Sprintf("./legacy-%s-%d", dbType, legacyVersion)
	relateDir, err := createLegacyTree(t, dbDir, legacyVersion)
	require.NoError(t, err)

	db, err := dbm.NewGoLevelDB("test", relateDir)
	require.NoError(t, err)

	defer func() {
		if err := db.Close(); err != nil {
			t.Errorf("DB close error: %v\n", err)
		}
		if err := os.RemoveAll(relateDir); err != nil {
			t.Errorf("%+v\n", err)
		}
	}()

	tree := NewMutableTree(db, 1000, false, NewNopLogger())

	// Load the latest legacy version
	_, err = tree.LoadVersion(int64(legacyVersion))
	require.NoError(t, err)

	// Commit new versions
	postVersions := 100
	for i := 0; i < postVersions; i++ {
		leafCount := rand.Intn(10)
		for j := 0; j < leafCount; j++ {
			_, err = tree.Set([]byte(fmt.Sprintf("key-%d-%d", i, j)), []byte(fmt.Sprintf("value-%d-%d", i, j)))
			require.NoError(t, err)
		}
		_, _, err = tree.SaveVersion()
		require.NoError(t, err)
	}

	// Check the available versions
	versions := tree.AvailableVersions()
	targetVersion := 0
	for i := len(versions) - 1; i >= 0; i-- {
		if versions[i] < legacyVersion {
			targetVersion = versions[i]
			break
		}
	}

	// Test LoadVersionForOverwriting for the legacy version
	err = tree.LoadVersionForOverwriting(int64(targetVersion))
	require.NoError(t, err)
	latestVersion, err := tree.ndb.getLatestVersion()
	require.NoError(t, err)
	require.Equal(t, int64(targetVersion), latestVersion)
	legacyLatestVersion, err := tree.ndb.getLegacyLatestVersion()
	require.NoError(t, err)
	require.Equal(t, int64(targetVersion), legacyLatestVersion)

	// Test DeleteVersionsTo for the legacy version
	for i := 0; i < postVersions; i++ {
		leafCount := rand.Intn(20)
		for j := 0; j < leafCount; j++ {
			_, err = tree.Set([]byte(fmt.Sprintf("key-%d-%d", i, j)), []byte(fmt.Sprintf("value-%d-%d", i, j)))
			require.NoError(t, err)
		}
		_, _, err = tree.SaveVersion()
		require.NoError(t, err)
	}
	// Check if the legacy versions are deleted at once
	versions = tree.AvailableVersions()
	err = tree.DeleteVersionsTo(legacyLatestVersion - 1)
	require.NoError(t, err)
	pVersions := tree.AvailableVersions()
	require.Equal(t, len(versions), len(pVersions))
	toVersion := legacyLatestVersion + int64(postVersions)/2
	err = tree.DeleteVersionsTo(toVersion)
	require.NoError(t, err)
	pVersions = tree.AvailableVersions()
	require.Equal(t, postVersions/2, len(pVersions))
}

func TestPruning(t *testing.T) {
	legacyVersion := 100
	dbDir := fmt.Sprintf("./legacy-%s-%d", dbType, legacyVersion)
	relateDir, err := createLegacyTree(t, dbDir, legacyVersion)
	require.NoError(t, err)

	db, err := dbm.NewGoLevelDB("test", relateDir)
	require.NoError(t, err)

	defer func() {
		if err := db.Close(); err != nil {
			t.Errorf("DB close error: %v\n", err)
		}
		if err := os.RemoveAll(relateDir); err != nil {
			t.Errorf("%+v\n", err)
		}
	}()

	// Load the latest version
	tree := NewMutableTree(db, 1000, false, NewNopLogger())
	_, err = tree.Load()
	require.NoError(t, err)

	// Save 10 versions without updates
	for i := 0; i < 10; i++ {
		_, _, err = tree.SaveVersion()
		require.NoError(t, err)
	}

	// Save 990 versions
	leavesCount := 10
	toVersion := int64(990)
	pruningInterval := int64(20)
	for i := int64(0); i < toVersion; i++ {
		for j := 0; j < leavesCount; j++ {
			_, err := tree.Set([]byte(fmt.Sprintf("key%d", j)), []byte(fmt.Sprintf("value%d", j)))
			require.NoError(t, err)
		}
		_, v, err := tree.SaveVersion()
		require.NoError(t, err)
		if v%pruningInterval == 0 {
			err = tree.DeleteVersionsTo(v - pruningInterval/2)
			require.NoError(t, err)
		}
	}

	// Reload the tree
	tree = NewMutableTree(db, 0, false, NewNopLogger())
	versions := tree.AvailableVersions()
	require.Equal(t, versions[0], int(toVersion)+legacyVersion+1)
	for _, v := range versions {
		_, err := tree.LoadVersion(int64(v))
		require.NoError(t, err)
	}
	// Check if the legacy nodes are pruned
	_, err = tree.Load()
	require.NoError(t, err)
	itr, err := NewNodeIterator(tree.root.GetKey(), tree.ndb)
	require.NoError(t, err)
	legacyNodes := make(map[string]*Node)
	for ; itr.Valid(); itr.Next(false) {
		node := itr.GetNode()
		if node.nodeKey.nonce == 0 {
			legacyNodes[string(node.hash)] = node
		}
	}

	lNodes, err := tree.ndb.legacyNodes()
	require.NoError(t, err)
	require.Len(t, lNodes, len(legacyNodes))
	for _, node := range lNodes {
		_, ok := legacyNodes[string(node.hash)]
		require.True(t, ok)
	}
}

func TestRandomSet(t *testing.T) {
	legacyVersion := 50
	dbDir := fmt.Sprintf("./legacy-%s-%d", dbType, legacyVersion)
	relateDir, err := createLegacyTree(t, dbDir, legacyVersion)
	require.NoError(t, err)

	db, err := dbm.NewGoLevelDB("test", relateDir)
	require.NoError(t, err)

	defer func() {
		if err := db.Close(); err != nil {
			t.Errorf("DB close error: %v\n", err)
		}
		if err := os.RemoveAll(relateDir); err != nil {
			t.Errorf("%+v\n", err)
		}
	}()

	tree := NewMutableTree(db, 10000, false, NewNopLogger())

	// Load the latest legacy version
	_, err = tree.LoadVersion(int64(legacyVersion))
	require.NoError(t, err)

	// Commit new versions
	postVersions := 1000
	emptyVersions := 10
	for i := 0; i < emptyVersions; i++ {
		_, _, err := tree.SaveVersion()
		require.NoError(t, err)
	}
	for i := 0; i < postVersions-emptyVersions; i++ {
		leafCount := rand.Intn(50)
		for j := 0; j < leafCount; j++ {
			key := iavlrand.RandBytes(10)
			value := iavlrand.RandBytes(10)
			_, err = tree.Set(key, value)
			require.NoError(t, err)
		}
		_, _, err = tree.SaveVersion()
		require.NoError(t, err)
	}

	err = tree.DeleteVersionsTo(int64(legacyVersion + postVersions - 1))
	require.NoError(t, err)
}
