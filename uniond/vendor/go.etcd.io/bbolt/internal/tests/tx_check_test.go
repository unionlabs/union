package tests_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/guts_cli"
	"go.etcd.io/bbolt/internal/surgeon"
)

func TestTx_RecursivelyCheckPages_MisplacedPage(t *testing.T) {
	db := btesting.MustCreateDB(t)
	db.ForceDisableStrictMode()
	require.NoError(t,
		db.Fill([]byte("data"), 1, 10000,
			func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
			func(tx int, k int) []byte { return make([]byte, 100) },
		))
	require.NoError(t, db.Close())

	xRay := surgeon.NewXRay(db.Path())

	path1, err := xRay.FindPathsToKey([]byte("0451"))
	require.NoError(t, err, "cannot find page that contains key:'0451'")
	require.Len(t, path1, 1, "Expected only one page that contains key:'0451'")

	path2, err := xRay.FindPathsToKey([]byte("7563"))
	require.NoError(t, err, "cannot find page that contains key:'7563'")
	require.Len(t, path2, 1, "Expected only one page that contains key:'7563'")

	srcPage := path1[0][len(path1[0])-1]
	targetPage := path2[0][len(path2[0])-1]
	require.NoError(t, surgeon.CopyPage(db.Path(), srcPage, targetPage))

	db.MustReopen()
	db.ForceDisableStrictMode()
	require.NoError(t, db.Update(func(tx *bolt.Tx) error {
		// Collect all the errors.
		var errors []error
		for err := range tx.Check() {
			errors = append(errors, err)
		}
		require.Len(t, errors, 1)
		require.ErrorContains(t, errors[0], fmt.Sprintf("leaf page(%v) needs to be >= the key in the ancestor", targetPage))
		return nil
	}))
	require.NoError(t, db.Close())
}

func TestTx_RecursivelyCheckPages_CorruptedLeaf(t *testing.T) {
	db := btesting.MustCreateDB(t)
	db.ForceDisableStrictMode()
	require.NoError(t,
		db.Fill([]byte("data"), 1, 10000,
			func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
			func(tx int, k int) []byte { return make([]byte, 100) },
		))
	require.NoError(t, db.Close())

	xray := surgeon.NewXRay(db.Path())

	path1, err := xray.FindPathsToKey([]byte("0451"))
	require.NoError(t, err, "cannot find page that contains key:'0451'")
	require.Len(t, path1, 1, "Expected only one page that contains key:'0451'")

	srcPage := path1[0][len(path1[0])-1]
	p, pbuf, err := guts_cli.ReadPage(db.Path(), uint64(srcPage))
	require.NoError(t, err)
	require.Positive(t, p.Count(), "page must be not empty")
	p.LeafPageElement(p.Count() / 2).Key()[0] = 'z'
	require.NoError(t, guts_cli.WritePage(db.Path(), pbuf))

	db.MustReopen()
	db.ForceDisableStrictMode()
	require.NoError(t, db.Update(func(tx *bolt.Tx) error {
		// Collect all the errors.
		var errors []error
		for err := range tx.Check() {
			errors = append(errors, err)
		}
		require.Len(t, errors, 2)
		require.ErrorContains(t, errors[0], fmt.Sprintf("leaf page(%v) needs to be < than key of the next element in ancestor", srcPage))
		require.ErrorContains(t, errors[1], fmt.Sprintf("leaf page(%v) needs to be > (found <) than previous element", srcPage))
		return nil
	}))
	require.NoError(t, db.Close())
}
