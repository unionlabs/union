package surgeon_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/guts_cli"
	"go.etcd.io/bbolt/internal/surgeon"
)

func TestFindPathsToKey(t *testing.T) {
	db := btesting.MustCreateDB(t)
	assert.NoError(t,
		db.Fill([]byte("data"), 1, 500,
			func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
			func(tx int, k int) []byte { return make([]byte, 100) },
		))
	assert.NoError(t, db.Close())

	navigator := surgeon.NewXRay(db.Path())
	path1, err := navigator.FindPathsToKey([]byte("0451"))
	assert.NoError(t, err)
	assert.NotEmpty(t, path1)

	page := path1[0][len(path1[0])-1]
	p, _, err := guts_cli.ReadPage(db.Path(), uint64(page))
	assert.NoError(t, err)
	assert.GreaterOrEqual(t, []byte("0451"), p.LeafPageElement(0).Key())
	assert.LessOrEqual(t, []byte("0451"), p.LeafPageElement(p.Count()-1).Key())
}

func TestFindPathsToKey_Bucket(t *testing.T) {
	rootBucket := []byte("data")
	subBucket := []byte("0451A")

	db := btesting.MustCreateDB(t)
	assert.NoError(t,
		db.Fill(rootBucket, 1, 500,
			func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
			func(tx int, k int) []byte { return make([]byte, 100) },
		))
	require.NoError(t, db.Update(func(tx *bbolt.Tx) error {
		sb, err := tx.Bucket(rootBucket).CreateBucket(subBucket)
		require.NoError(t, err)
		require.NoError(t, sb.Put([]byte("foo"), []byte("bar")))
		return nil
	}))

	assert.NoError(t, db.Close())

	navigator := surgeon.NewXRay(db.Path())
	path1, err := navigator.FindPathsToKey(subBucket)
	assert.NoError(t, err)
	assert.NotEmpty(t, path1)

	page := path1[0][len(path1[0])-1]
	p, _, err := guts_cli.ReadPage(db.Path(), uint64(page))
	assert.NoError(t, err)
	assert.GreaterOrEqual(t, subBucket, p.LeafPageElement(0).Key())
	assert.LessOrEqual(t, subBucket, p.LeafPageElement(p.Count()-1).Key())
}
