package grocksdb

import (
	"bytes"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestCompactionFilter(t *testing.T) {
	t.Parallel()

	var (
		changeKey    = []byte("change")
		changeValOld = []byte("old")
		changeValNew = []byte("new")
		deleteKey    = []byte("delete")
	)
	db := newTestDB(t, func(opts *Options) {
		opts.SetCompactionFilter(&mockCompactionFilter{
			filter: func(_ int, key, val []byte) (remove bool, newVal []byte) {
				if bytes.Equal(key, changeKey) {
					return false, changeValNew
				}
				if bytes.Equal(key, deleteKey) {
					return true, val
				}
				t.Errorf("key %q not expected during compaction", key)
				return false, nil
			},
		})
	})
	defer db.Close()

	// insert the test keys
	wo := NewDefaultWriteOptions()
	require.Nil(t, db.Put(wo, changeKey, changeValOld))
	require.Nil(t, db.Put(wo, deleteKey, changeValNew))

	// trigger a compaction
	db.CompactRange(Range{})
	require.NoError(t, db.SuggestCompactRange(Range{}))

	// ensure that the value is changed after compaction
	ro := NewDefaultReadOptions()
	v1, err := db.Get(ro, changeKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), changeValNew)

	// ensure that the key is deleted after compaction
	v2, err := db.Get(ro, deleteKey)
	require.Nil(t, err)
	require.Nil(t, v2.Data())
}

type mockCompactionFilter struct {
	filter func(level int, key, val []byte) (remove bool, newVal []byte)
}

func (m *mockCompactionFilter) Name() string { return "grocksdb.test" }

func (m *mockCompactionFilter) Filter(level int, key, val []byte) (bool, []byte) {
	return m.filter(level, key, val)
}

func (m *mockCompactionFilter) SetIgnoreSnapshots(value bool) {
}

func (m *mockCompactionFilter) Destroy() {}
