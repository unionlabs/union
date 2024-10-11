package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestMergeOperator(t *testing.T) {
	t.Parallel()

	var (
		givenKey    = []byte("hello")
		givenVal1   = []byte("foo")
		givenVal2   = []byte("bar")
		givenMerged = []byte("foobar")
	)
	merger := &mockMergeOperator{
		fullMerge: func(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
			require.EqualValues(t, key, givenKey)
			require.EqualValues(t, existingValue, givenVal1)
			require.EqualValues(t, operands, [][]byte{givenVal2})
			return givenMerged, true
		},
	}
	db := newTestDB(t, func(opts *Options) {
		opts.SetMergeOperator(merger)
	})
	defer db.Close()

	wo := NewDefaultWriteOptions()
	require.Nil(t, db.Put(wo, givenKey, givenVal1))
	require.Nil(t, db.Merge(wo, givenKey, givenVal2))

	// trigger a compaction to ensure that a merge is performed
	db.CompactRange(Range{nil, nil})

	ro := NewDefaultReadOptions()
	v1, err := db.Get(ro, givenKey)
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), givenMerged)
	v1.Free()
}

func TestPartialMergeOperator(t *testing.T) {
	t.Parallel()

	var (
		givenKey     = []byte("hello")
		startingVal  = []byte("foo")
		mergeVal1    = []byte("bar")
		mergeVal2    = []byte("baz")
		fMergeResult = []byte("foobarbaz")
		pMergeResult = []byte("barbaz")
	)

	merger := &mockMergePartialOperator{
		fullMerge: func(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
			require.EqualValues(t, key, givenKey)
			require.EqualValues(t, existingValue, startingVal)
			require.EqualValues(t, operands[0], pMergeResult)
			return fMergeResult, true
		},
		partialMerge: func(key, leftOperand, rightOperand []byte) ([]byte, bool) {
			require.EqualValues(t, key, givenKey)
			require.EqualValues(t, leftOperand, mergeVal1)
			require.EqualValues(t, rightOperand, mergeVal2)
			return pMergeResult, true
		},
	}
	db := newTestDB(t, func(opts *Options) {
		opts.SetMergeOperator(merger)
	})
	defer db.Close()

	wo := NewDefaultWriteOptions()
	defer wo.Destroy()

	// insert a starting value and compact to trigger merges
	require.Nil(t, db.Put(wo, givenKey, startingVal))

	// trigger a compaction to ensure that a merge is performed
	db.CompactRange(Range{nil, nil})

	// we expect these two operands to be passed to merge partial
	require.Nil(t, db.Merge(wo, givenKey, mergeVal1))
	require.Nil(t, db.Merge(wo, givenKey, mergeVal2))

	// trigger a compaction to ensure that a
	// partial and full merge are performed
	db.CompactRange(Range{nil, nil})

	ro := NewDefaultReadOptions()
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), fMergeResult)
}

func TestMergeMultiOperator(t *testing.T) {
	t.Parallel()

	var (
		givenKey     = []byte("hello")
		startingVal  = []byte("foo")
		mergeVal1    = []byte("bar")
		mergeVal2    = []byte("baz")
		fMergeResult = []byte("foobarbaz")
		pMergeResult = []byte("bar")
	)

	merger := &mockMergeMultiOperator{
		fullMerge: func(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
			require.EqualValues(t, key, givenKey)
			require.EqualValues(t, existingValue, startingVal)
			require.EqualValues(t, operands[0], pMergeResult)
			return fMergeResult, true
		},
		partialMergeMulti: func(key []byte, operands [][]byte) ([]byte, bool) {
			require.EqualValues(t, key, givenKey)
			require.EqualValues(t, operands[0], mergeVal1)
			require.EqualValues(t, operands[1], mergeVal2)
			return pMergeResult, true
		},
	}
	db := newTestDB(t, func(opts *Options) {
		opts.SetMergeOperator(merger)
	})
	defer db.Close()

	wo := NewDefaultWriteOptions()
	defer wo.Destroy()

	// insert a starting value and compact to trigger merges
	require.Nil(t, db.Put(wo, givenKey, startingVal))

	// trigger a compaction to ensure that a merge is performed
	db.CompactRange(Range{nil, nil})

	// we expect these two operands to be passed to merge multi
	require.Nil(t, db.Merge(wo, givenKey, mergeVal1))
	require.Nil(t, db.Merge(wo, givenKey, mergeVal2))

	// trigger a compaction to ensure that a
	// partial and full merge are performed
	db.CompactRange(Range{nil, nil})

	ro := NewDefaultReadOptions()
	v1, err := db.Get(ro, givenKey)
	defer v1.Free()
	require.Nil(t, err)
	require.EqualValues(t, v1.Data(), fMergeResult)
}

// Mock Objects
type mockMergeOperator struct {
	fullMerge func(key, existingValue []byte, operands [][]byte) ([]byte, bool)
}

func (m *mockMergeOperator) Name() string { return "grocksdb.test" }
func (m *mockMergeOperator) FullMerge(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
	return m.fullMerge(key, existingValue, operands)
}

type mockMergeMultiOperator struct {
	fullMerge         func(key, existingValue []byte, operands [][]byte) ([]byte, bool)
	partialMergeMulti func(key []byte, operands [][]byte) ([]byte, bool)
}

func (m *mockMergeMultiOperator) Name() string { return "grocksdb.multi" }
func (m *mockMergeMultiOperator) FullMerge(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
	return m.fullMerge(key, existingValue, operands)
}

func (m *mockMergeMultiOperator) PartialMergeMulti(key []byte, operands [][]byte) ([]byte, bool) {
	return m.partialMergeMulti(key, operands)
}

type mockMergePartialOperator struct {
	fullMerge    func(key, existingValue []byte, operands [][]byte) ([]byte, bool)
	partialMerge func(key, leftOperand, rightOperand []byte) ([]byte, bool)
}

func (m *mockMergePartialOperator) Name() string { return "grocksdb.partial" }
func (m *mockMergePartialOperator) FullMerge(key, existingValue []byte, operands [][]byte) ([]byte, bool) {
	return m.fullMerge(key, existingValue, operands)
}

func (m *mockMergePartialOperator) PartialMerge(key, leftOperand, rightOperand []byte) ([]byte, bool) {
	return m.partialMerge(key, leftOperand, rightOperand)
}
