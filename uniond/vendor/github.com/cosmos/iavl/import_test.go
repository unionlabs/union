package iavl

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

func ExampleImporter() {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())

	_, err := tree.Set([]byte("a"), []byte{1})
	if err != nil {
		panic(err)
	}

	_, err = tree.Set([]byte("b"), []byte{2})
	if err != nil {
		panic(err)
	}
	_, err = tree.Set([]byte("c"), []byte{3})
	if err != nil {
		panic(err)
	}
	_, version, err := tree.SaveVersion()
	if err != nil {
		panic(err)
	}

	itree, err := tree.GetImmutable(version)
	if err != nil {
		panic(err)
	}
	exporter, err := itree.Export()
	if err != nil {
		panic(err)
	}
	defer exporter.Close()
	exported := []*ExportNode{}
	for {
		var node *ExportNode
		node, err = exporter.Next()
		if err == ErrorExportDone {
			break
		} else if err != nil {
			panic(err)
		}
		exported = append(exported, node)
	}

	newTree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := newTree.Import(version)
	if err != nil {
		panic(err)
	}
	defer importer.Close()
	for _, node := range exported {
		err = importer.Add(node)
		if err != nil {
			panic(err)
		}
	}
	err = importer.Commit()
	if err != nil {
		panic(err)
	}
}

func TestImporter_NegativeVersion(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	_, err := tree.Import(-1)
	require.Error(t, err)
}

func TestImporter_NotEmpty(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	_, err := tree.Set([]byte("a"), []byte{1})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	_, err = tree.Import(1)
	require.Error(t, err)
}

func TestImporter_NotEmptyDatabase(t *testing.T) {
	db := dbm.NewMemDB()

	tree := NewMutableTree(db, 0, false, NewNopLogger())
	_, err := tree.Set([]byte("a"), []byte{1})
	require.NoError(t, err)
	_, _, err = tree.SaveVersion()
	require.NoError(t, err)

	tree = NewMutableTree(db, 0, false, NewNopLogger())
	_, err = tree.Load()
	require.NoError(t, err)

	_, err = tree.Import(1)
	require.Error(t, err)
}

func TestImporter_NotEmptyUnsaved(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	_, err := tree.Set([]byte("a"), []byte{1})
	require.NoError(t, err)

	_, err = tree.Import(1)
	require.Error(t, err)
}

func TestImporter_Add(t *testing.T) {
	k := []byte("key")
	v := []byte("value")

	testcases := map[string]struct {
		node  *ExportNode
		valid bool
	}{
		"nil node":          {nil, false},
		"valid":             {&ExportNode{Key: k, Value: v, Version: 1, Height: 0}, true},
		"no key":            {&ExportNode{Key: nil, Value: v, Version: 1, Height: 0}, false},
		"no value":          {&ExportNode{Key: k, Value: nil, Version: 1, Height: 0}, false},
		"version too large": {&ExportNode{Key: k, Value: v, Version: 2, Height: 0}, false},
		"no version":        {&ExportNode{Key: k, Value: v, Version: 0, Height: 0}, false},
		// further cases will be handled by Node.validate()
	}
	for desc, tc := range testcases {
		tc := tc // appease scopelint
		t.Run(desc, func(t *testing.T) {
			tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
			importer, err := tree.Import(1)
			require.NoError(t, err)
			defer importer.Close()

			err = importer.Add(tc.node)
			if tc.valid {
				require.NoError(t, err)
			} else {
				if err == nil {
					err = importer.Commit()
				}
				require.Error(t, err)
			}
		})
	}
}

func TestImporter_Add_Closed(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(1)
	require.NoError(t, err)

	importer.Close()
	err = importer.Add(&ExportNode{Key: []byte("key"), Value: []byte("value"), Version: 1, Height: 0})
	require.Error(t, err)
	require.Equal(t, ErrNoImport, err)
}

func TestImporter_Close(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(1)
	require.NoError(t, err)

	err = importer.Add(&ExportNode{Key: []byte("key"), Value: []byte("value"), Version: 1, Height: 0})
	require.NoError(t, err)

	importer.Close()
	has, err := tree.Has([]byte("key"))
	require.NoError(t, err)
	require.False(t, has)

	importer.Close()
}

func TestImporter_Commit(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(1)
	require.NoError(t, err)

	err = importer.Add(&ExportNode{Key: []byte("key"), Value: []byte("value"), Version: 1, Height: 0})
	require.NoError(t, err)

	err = importer.Commit()
	require.NoError(t, err)
	has, err := tree.Has([]byte("key"))
	require.NoError(t, err)
	require.True(t, has)
}

func TestImporter_Commit_ForwardVersion(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(2)
	require.NoError(t, err)

	err = importer.Add(&ExportNode{Key: []byte("key"), Value: []byte("value"), Version: 1, Height: 0})
	require.NoError(t, err)

	err = importer.Commit()
	require.NoError(t, err)
	has, err := tree.Has([]byte("key"))
	require.NoError(t, err)
	require.True(t, has)
}

func TestImporter_Commit_Closed(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(1)
	require.NoError(t, err)

	err = importer.Add(&ExportNode{Key: []byte("key"), Value: []byte("value"), Version: 1, Height: 0})
	require.NoError(t, err)

	importer.Close()
	err = importer.Commit()
	require.Error(t, err)
	require.Equal(t, ErrNoImport, err)
}

func TestImporter_Commit_Empty(t *testing.T) {
	tree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
	importer, err := tree.Import(3)
	require.NoError(t, err)
	defer importer.Close()

	err = importer.Commit()
	require.NoError(t, err)
	assert.EqualValues(t, 3, tree.Version())
}

func BenchmarkImport(b *testing.B) {
	benchmarkImport(b, 4096)
}

func BenchmarkImportBatch(b *testing.B) {
	benchmarkImport(b, maxBatchSize*10)
}

func benchmarkImport(b *testing.B, nodes int) {
	b.StopTimer()
	tree := setupExportTreeSized(b, nodes)
	exported := make([]*ExportNode, 0, nodes)
	exporter, err := tree.Export()
	require.NoError(b, err)
	for {
		item, err := exporter.Next()
		if err == ErrorExportDone {
			break
		} else if err != nil {
			b.Error(err)
		}
		exported = append(exported, item)
	}
	exporter.Close()
	b.StartTimer()

	for n := 0; n < b.N; n++ {
		newTree := NewMutableTree(dbm.NewMemDB(), 0, false, NewNopLogger())
		importer, err := newTree.Import(tree.Version())
		require.NoError(b, err)
		for _, item := range exported {
			err = importer.Add(item)
			if err != nil {
				b.Error(err)
			}
		}
		err = importer.Commit()
		require.NoError(b, err)
	}
}
