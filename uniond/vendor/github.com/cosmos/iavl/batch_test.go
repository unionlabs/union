package iavl

import (
	"encoding/binary"
	"fmt"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"

	dbm "github.com/cosmos/iavl/db"
)

func cleanupDBDir(dir, name string) {
	err := os.RemoveAll(filepath.Join(dir, name) + ".db")
	if err != nil {
		panic(err)
	}
}

var bytesArrayOfSize10KB = [10000]byte{}

func makeKey(n uint16) []byte {
	key := make([]byte, 2)
	binary.BigEndian.PutUint16(key, n)
	return key
}

func TestBatchWithFlusher(t *testing.T) {
	testBatchWithFlusher(t)
}

func testBatchWithFlusher(t *testing.T) {
	name := fmt.Sprintf("test_%x", randstr(12))
	dir := t.TempDir()
	db, err := dbm.NewGoLevelDB(name, dir)
	require.NoError(t, err)
	defer cleanupDBDir(dir, name)

	batchWithFlusher := NewBatchWithFlusher(db, DefaultOptions().FlushThreshold)

	// we'll try to to commit 10MBs (1000 * 10KBs each entries) of data into the db
	for keyNonce := uint16(0); keyNonce < 1000; keyNonce++ {
		// each value is 10 KBs of zero bytes
		key := makeKey(keyNonce)
		err := batchWithFlusher.Set(key, bytesArrayOfSize10KB[:])
		if err != nil {
			panic(err)
		}
	}
	require.NoError(t, batchWithFlusher.Write())

	itr, err := db.Iterator(nil, nil)
	require.NoError(t, err)

	var keyNonce uint16
	for itr.Valid() {
		expectedKey := makeKey(keyNonce)
		require.Equal(t, expectedKey, itr.Key())
		require.Equal(t, bytesArrayOfSize10KB[:], itr.Value())
		itr.Next()
		keyNonce++
	}
}
