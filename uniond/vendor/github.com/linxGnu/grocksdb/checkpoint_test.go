package grocksdb

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCheckpoint(t *testing.T) {
	t.Parallel()

	dir := t.TempDir()
	err := os.RemoveAll(dir)
	require.Nil(t, err)

	db := newTestDB(t, nil)
	defer db.Close()

	// insert keys
	givenKeys := [][]byte{[]byte("key1"), []byte("key2"), []byte("key3")}
	givenVal := []byte("val")
	wo := NewDefaultWriteOptions()
	for _, k := range givenKeys {
		require.Nil(t, db.Put(wo, k, givenVal))
	}

	var dbCheck *DB
	var checkpoint *Checkpoint

	checkpoint, err = db.NewCheckpoint()
	defer checkpoint.Destroy()
	require.NotNil(t, checkpoint)
	require.Nil(t, err)

	err = checkpoint.CreateCheckpoint(dir, 0)
	require.Nil(t, err)

	opts := NewDefaultOptions()
	opts.SetCreateIfMissing(true)
	dbCheck, err = OpenDb(opts, dir)
	require.Nil(t, err)
	defer dbCheck.Close()

	// test keys
	var value *Slice
	ro := NewDefaultReadOptions()
	for _, k := range givenKeys {
		value, err = dbCheck.Get(ro, k)
		require.Nil(t, err)
		require.EqualValues(t, value.Data(), givenVal)
		value.Free()
	}
}
