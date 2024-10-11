package surgeon_test

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"

	bolt "go.etcd.io/bbolt"
	"go.etcd.io/bbolt/internal/btesting"
	"go.etcd.io/bbolt/internal/surgeon"
)

func TestRevertMetaPage(t *testing.T) {
	db := btesting.MustCreateDB(t)
	assert.NoError(t,
		db.Fill([]byte("data"), 1, 500,
			func(tx int, k int) []byte { return []byte(fmt.Sprintf("%04d", k)) },
			func(tx int, k int) []byte { return make([]byte, 100) },
		))
	assert.NoError(t,
		db.Update(
			func(tx *bolt.Tx) error {
				b := tx.Bucket([]byte("data"))
				assert.NoError(t, b.Put([]byte("0123"), []byte("new Value for 123")))
				assert.NoError(t, b.Put([]byte("1234b"), []byte("additional object")))
				assert.NoError(t, b.Delete([]byte("0246")))
				return nil
			}))

	assert.NoError(t,
		db.View(
			func(tx *bolt.Tx) error {
				b := tx.Bucket([]byte("data"))
				assert.Equal(t, []byte("new Value for 123"), b.Get([]byte("0123")))
				assert.Equal(t, []byte("additional object"), b.Get([]byte("1234b")))
				assert.Nil(t, b.Get([]byte("0246")))
				return nil
			}))

	db.Close()

	// This causes the whole tree to be linked to the previous state
	assert.NoError(t, surgeon.RevertMetaPage(db.Path()))

	db.MustReopen()
	db.MustCheck()
	assert.NoError(t,
		db.View(
			func(tx *bolt.Tx) error {
				b := tx.Bucket([]byte("data"))
				assert.Equal(t, make([]byte, 100), b.Get([]byte("0123")))
				assert.Nil(t, b.Get([]byte("1234b")))
				assert.Equal(t, make([]byte, 100), b.Get([]byte("0246")))
				return nil
			}))
}
