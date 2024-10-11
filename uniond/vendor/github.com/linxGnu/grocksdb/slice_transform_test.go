package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestSliceTransform(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetPrefixExtractor(&testSliceTransform{})
	})
	defer db.Close()

	wo := NewDefaultWriteOptions()
	require.Nil(t, db.Put(wo, []byte("foo1"), []byte("foo")))
	require.Nil(t, db.Put(wo, []byte("foo2"), []byte("foo")))
	require.Nil(t, db.Put(wo, []byte("bar1"), []byte("bar")))

	iter := db.NewIterator(NewDefaultReadOptions())
	defer iter.Close()
	prefix := []byte("foo")
	numFound := 0
	for iter.Seek(prefix); iter.ValidForPrefix(prefix); iter.Next() {
		numFound++
	}
	require.Nil(t, iter.Err())
	require.EqualValues(t, numFound, 2)
}

func TestFixedPrefixTransformOpen(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetPrefixExtractor(NewFixedPrefixTransform(3))
	})
	defer db.Close()
}

func TestNewNoopPrefixTransform(t *testing.T) {
	t.Parallel()

	db := newTestDB(t, func(opts *Options) {
		opts.SetPrefixExtractor(NewNoopPrefixTransform())
	})
	defer db.Close()
}

type testSliceTransform struct{}

func (st *testSliceTransform) Name() string                { return "grocksdb.test" }
func (st *testSliceTransform) Transform(src []byte) []byte { return src[0:3] }
func (st *testSliceTransform) InDomain(src []byte) bool    { return len(src) >= 3 }
func (st *testSliceTransform) InRange(src []byte) bool     { return len(src) == 3 }
func (st *testSliceTransform) Destroy()                    {}
