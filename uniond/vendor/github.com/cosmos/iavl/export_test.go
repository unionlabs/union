package iavl

import (
	"math"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

// setupExportTreeBasic sets up a basic tree with a handful of
// create/update/delete operations over a few versions.
func setupExportTreeBasic(t require.TestingT) *ImmutableTree {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	_, err := tree.Set([]byte("x"), []byte{255})
	require.NoError(t, err)
	_, err = tree.Set([]byte("z"), []byte{255})
	require.NoError(t, err)
	_, err = tree.Set([]byte("a"), []byte{1})
	require.NoError(t, err)
	_, err = tree.Set([]byte("b"), []byte{2})
	require.NoError(t, err)
	_, err = tree.Set([]byte("c"), []byte{3})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	_, _, err = tree.Remove([]byte("x"))
	require.NoError(t, err)
	_, _, err = tree.Remove([]byte("b"))
	require.NoError(t, err)
	_, err = tree.Set([]byte("c"), []byte{255})
	require.NoError(t, err)
	_, err = tree.Set([]byte("d"), []byte{4})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	_, err = tree.Set([]byte("b"), []byte{2})
	require.NoError(t, err)
	_, err = tree.Set([]byte("c"), []byte{3})
	require.NoError(t, err)
	_, err = tree.Set([]byte("e"), []byte{5})
	require.NoError(t, err)
	_, _, err = tree.Remove([]byte("z"))
	require.NoError(t, err)
	_, err = tree.Set([]byte("abc"), []byte{6})
	require.NoError(t, err)
	_, version, err := tree.SaveVersion()
	require.NoError(t, err)

	itree, err := tree.GetImmutable(version)
	require.NoError(t, err)
	return itree
}

// setupExportTreeRandom sets up a randomly generated tree.
// nolint: dupl
func setupExportTreeRandom(t *testing.T) *ImmutableTree {
	const (
		randSeed  = 49872768940 // For deterministic tests
		keySize   = 16
		valueSize = 16

		versions    = 8    // number of versions to generate
		versionOps  = 1024 // number of operations (create/update/delete) per version
		updateRatio = 0.4  // ratio of updates out of all operations
		deleteRatio = 0.2  // ratio of deletes out of all operations
	)

	r := rand.New(rand.NewSource(randSeed))
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	var version int64
	keys := make([][]byte, 0, versionOps)
	for i := 0; i < versions; i++ {
		for j := 0; j < versionOps; j++ {
			key := make([]byte, keySize)
			value := make([]byte, valueSize)

			// The performance of this is likely to be terrible, but that's fine for small tests
			switch {
			case len(keys) > 0 && r.Float64() <= deleteRatio:
				index := r.Intn(len(keys))
				key = keys[index]
				keys = append(keys[:index], keys[index+1:]...)
				_, removed, err := tree.Remove(key)
				require.NoError(t, err)
				require.True(t, removed)

			case len(keys) > 0 && r.Float64() <= updateRatio:
				key = keys[r.Intn(len(keys))]
				r.Read(value)
				updated, err := tree.Set(key, value)
				require.NoError(t, err)
				require.True(t, updated)

			default:
				r.Read(key)
				r.Read(value)
				// If we get an update, set again
				for updated, err := tree.Set(key, value); updated && err == nil; {
					key = make([]byte, keySize)
					r.Read(key)
				}
				keys = append(keys, key)
			}
		}
		var err error
		_, version, err = tree.SaveVersion()
		require.NoError(t, err)
	}

	require.EqualValues(t, versions, tree.Version())
	require.GreaterOrEqual(t, tree.Size(), int64(math.Trunc(versions*versionOps*(1-updateRatio-deleteRatio))/2))

	itree, err := tree.GetImmutable(version)
	require.NoError(t, err)
	return itree
}

// setupExportTreeSized sets up a single-version tree with a given number
// of randomly generated key/value pairs, useful for benchmarking.
func setupExportTreeSized(t require.TestingT, treeSize int) *ImmutableTree {
	const (
		randSeed  = 49872768940 // For deterministic tests
		keySize   = 16
		valueSize = 16
	)

	r := rand.New(rand.NewSource(randSeed))
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	for i := 0; i < treeSize; i++ {
		key := make([]byte, keySize)
		value := make([]byte, valueSize)
		r.Read(key)
		r.Read(value)
		updated, err := tree.Set(key, value)
		require.NoError(t, err)

		if updated {
			i--
		}
	}

	_, version, err := tree.SaveVersion()
	require.NoError(t, err)

	itree, err := tree.GetImmutable(version)
	require.NoError(t, err)

	return itree
}

func TestExporter(t *testing.T) {
	tree := setupExportTreeBasic(t)

	expect := []*ExportNode{
		{Key: []byte("a"), Value: []byte{1}, Version: 1, Height: 0},
		{Key: []byte("abc"), Value: []byte{6}, Version: 3, Height: 0},
		{Key: []byte("abc"), Value: nil, Version: 3, Height: 1},
		{Key: []byte("b"), Value: []byte{2}, Version: 3, Height: 0},
		{Key: []byte("c"), Value: []byte{3}, Version: 3, Height: 0},
		{Key: []byte("c"), Value: nil, Version: 3, Height: 1},
		{Key: []byte("b"), Value: nil, Version: 3, Height: 2},
		{Key: []byte("d"), Value: []byte{4}, Version: 2, Height: 0},
		{Key: []byte("e"), Value: []byte{5}, Version: 3, Height: 0},
		{Key: []byte("e"), Value: nil, Version: 3, Height: 1},
		{Key: []byte("d"), Value: nil, Version: 3, Height: 3},
	}

	actual := make([]*ExportNode, 0, len(expect))
	exporter, err := tree.Export()
	require.NoError(t, err)
	defer exporter.Close()
	for {
		node, err := exporter.Next()
		if err == ErrorExportDone {
			break
		}
		require.NoError(t, err)
		actual = append(actual, node)
	}

	assert.Equal(t, expect, actual)
}

func TestExporterCompress(t *testing.T) {
	tree := setupExportTreeBasic(t)

	expect := []*ExportNode{
		{Key: []byte{0, 'a'}, Value: []byte{1}, Version: 1, Height: 0},
		{Key: []byte{1, 'b', 'c'}, Value: []byte{6}, Version: 3, Height: 0},
		{Key: nil, Value: nil, Version: 0, Height: 1},
		{Key: []byte{0, 'b'}, Value: []byte{2}, Version: 3, Height: 0},
		{Key: []byte{0, 'c'}, Value: []byte{3}, Version: 3, Height: 0},
		{Key: nil, Value: nil, Version: 0, Height: 1},
		{Key: nil, Value: nil, Version: 0, Height: 2},
		{Key: []byte{0, 'd'}, Value: []byte{4}, Version: 2, Height: 0},
		{Key: []byte{0, 'e'}, Value: []byte{5}, Version: 3, Height: 0},
		{Key: nil, Value: nil, Version: 0, Height: 1},
		{Key: nil, Value: nil, Version: 0, Height: 3},
	}

	actual := make([]*ExportNode, 0, len(expect))
	innerExporter, err := tree.Export()
	require.NoError(t, err)
	defer innerExporter.Close()

	exporter := NewCompressExporter(innerExporter)
	for {
		node, err := exporter.Next()
		if err == ErrorExportDone {
			break
		}
		require.NoError(t, err)
		actual = append(actual, node)
	}

	assert.Equal(t, expect, actual)
}

func TestExporter_Import(t *testing.T) {
	testcases := map[string]*ImmutableTree{
		"empty tree": NewImmutableTree(dbm.NewMemDB(), 0, false, NewNopLogger()),
		"basic tree": setupExportTreeBasic(t),
	}
	if !testing.Short() {
		testcases["sized tree"] = setupExportTreeSized(t, 4096)
		testcases["random tree"] = setupExportTreeRandom(t)
	}

	for desc, tree := range testcases {
		tree := tree
		for _, compress := range []bool{false, true} {
			if compress {
				desc += "-compress"
			}
			compress := compress
			t.Run(desc, func(t *testing.T) {
				t.Parallel()

				innerExporter, err := tree.Export()
				require.NoError(t, err)
				defer innerExporter.Close()

				exporter := NodeExporter(innerExporter)
				if compress {
					exporter = NewCompressExporter(innerExporter)
				}

				newTree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
				innerImporter, err := newTree.Import(tree.Version())
				require.NoError(t, err)
				defer innerImporter.Close()

				importer := NodeImporter(innerImporter)
				if compress {
					importer = NewCompressImporter(innerImporter)
				}

				for {
					item, err := exporter.Next()
					if err == ErrorExportDone {
						err = innerImporter.Commit()
						require.NoError(t, err)
						break
					}
					require.NoError(t, err)
					err = importer.Add(item)
					require.NoError(t, err)
				}

				treeHash := tree.Hash()
				newTreeHash := newTree.Hash()

				require.Equal(t, treeHash, newTreeHash, "Tree hash mismatch")
				require.Equal(t, tree.Size(), newTree.Size(), "Tree size mismatch")
				require.Equal(t, tree.Version(), newTree.Version(), "Tree version mismatch")

				tree.Iterate(func(key, value []byte) bool { //nolint:errcheck
					index, _, err := tree.GetWithIndex(key)
					require.NoError(t, err)
					newIndex, newValue, err := newTree.GetWithIndex(key)
					require.NoError(t, err)
					require.Equal(t, index, newIndex, "Index mismatch for key %v", key)
					require.Equal(t, value, newValue, "Value mismatch for key %v", key)
					return false
				})
			})
		}
	}
}

func TestExporter_Close(t *testing.T) {
	tree := setupExportTreeSized(t, 4096)
	exporter, err := tree.Export()
	require.NoError(t, err)

	node, err := exporter.Next()
	require.NoError(t, err)
	require.NotNil(t, node)

	exporter.Close()
	node, err = exporter.Next()
	require.Error(t, err)
	require.Equal(t, ErrorExportDone, err)
	require.Nil(t, node)

	node, err = exporter.Next()
	require.Error(t, err)
	require.Equal(t, ErrorExportDone, err)
	require.Nil(t, node)

	exporter.Close()
	exporter.Close()
}

func TestExporter_DeleteVersionErrors(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	_, err := tree.Set([]byte("a"), []byte{1})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	_, err = tree.Set([]byte("b"), []byte{2})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	_, err = tree.Set([]byte("c"), []byte{3})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	itree, err := tree.GetImmutable(2)
	require.NoError(t, err)
	exporter, err := itree.Export()
	require.NoError(t, err)
	defer exporter.Close()

	err = tree.DeleteVersionsTo(1)
	require.NoError(t, err)

	err = tree.DeleteVersionsTo(2)
	require.Error(t, err)

	exporter.Close()
	err = tree.DeleteVersionsTo(2)
	require.NoError(t, err)
}

func BenchmarkExport(b *testing.B) {
	b.StopTimer()
	tree := setupExportTreeSized(b, 4096)
	b.StartTimer()
	for n := 0; n < b.N; n++ {
		exporter, err := tree.Export()
		require.NoError(b, err)
		for {
			_, err := exporter.Next()
			if err == ErrorExportDone {
				break
			} else if err != nil {
				b.Error(err)
			}
		}
		exporter.Close()
	}
}
