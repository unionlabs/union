//go:build boltdb
// +build boltdb

package db

import (
	"fmt"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestBoltDBNewBoltDB(t *testing.T) {
	name := fmt.Sprintf("test_%x", randStr(12))
	dir := os.TempDir()
	defer cleanupDBDir(dir, name)

	db, err := NewBoltDB(name, dir)
	require.NoError(t, err)
	db.Close()
}

func BenchmarkBoltDBRandomReadsWrites(b *testing.B) {
	name := fmt.Sprintf("test_%x", randStr(12))
	db, err := NewBoltDB(name, "")
	if err != nil {
		b.Fatal(err)
	}
	defer func() {
		db.Close()
		cleanupDBDir("", name)
	}()

	benchmarkRandomReadsWrites(b, db)
}
