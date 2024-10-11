package query

import (
	"context"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
)

func TestCollectionPagination(t *testing.T) {
	sk, ctx := deps()
	sb := collections.NewSchemaBuilder(sk)
	m := collections.NewMap(sb, collections.NewPrefix(0), "_", collections.Uint64Key, collections.Uint64Value)

	for i := uint64(0); i < 300; i++ {
		require.NoError(t, m.Set(ctx, i, i))
	}

	createResults := func(from, to uint64) []collections.KeyValue[uint64, uint64] {
		var res []collections.KeyValue[uint64, uint64]
		if from <= to {
			for i := from; i <= to; i++ {
				res = append(res, collections.KeyValue[uint64, uint64]{
					Key:   i,
					Value: i,
				})
			}
		} else {
			for i := from; i >= to; i-- {
				res = append(res, collections.KeyValue[uint64, uint64]{
					Key:   i,
					Value: i,
				})
			}
		}
		return res
	}

	encodeKey := func(key uint64) []byte {
		b, err := encodeCollKey[uint64, uint64](m, key)
		require.NoError(t, err)
		return b
	}

	type test struct {
		req        *PageRequest
		expResp    *PageResponse
		filter     func(key, value uint64) (bool, error)
		expResults []collections.KeyValue[uint64, uint64]
		wantErr    error
	}

	tcs := map[string]test{
		"nil pagination": {
			req: nil,
			expResp: &PageResponse{
				NextKey: encodeKey(100),
				Total:   300,
			},
			expResults: createResults(0, 99),
		},
		"with key and limit": {
			req: &PageRequest{
				Key:   encodeKey(100),
				Limit: 149,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(249),
			},
			expResults: createResults(100, 248),
		},
		"with reverse": {
			req: &PageRequest{
				Reverse: true,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(199),
				Total:   300,
			},
			expResults: createResults(299, 200),
		},
		"with key and reverse": {
			req: &PageRequest{
				Key:     encodeKey(199),
				Reverse: true,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(99),
			},
			expResults: createResults(199, 100),
		},
		"with offset and count total": {
			req: &PageRequest{
				Offset:     50,
				Limit:      100,
				CountTotal: true,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(150),
				Total:   300,
			},
			expResults: createResults(50, 149),
		},
		"filtered no key": {
			req: &PageRequest{
				Limit: 3,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(5),
			},
			filter: func(key, value uint64) (bool, error) {
				return key%2 == 0, nil
			},
			expResults: []collections.KeyValue[uint64, uint64]{
				{Key: 0, Value: 0},
				{Key: 2, Value: 2},
				{Key: 4, Value: 4},
			},
		},
		"filtered with key": {
			req: &PageRequest{
				Key:   encodeKey(2),
				Limit: 3,
			},
			expResp: &PageResponse{
				NextKey: encodeKey(5),
			},
			filter: func(key, value uint64) (bool, error) {
				return key%2 == 0, nil
			},
			expResults: []collections.KeyValue[uint64, uint64]{
				{Key: 2, Value: 2},
				{Key: 4, Value: 4},
			},
		},
	}

	for name, tc := range tcs {
		t.Run(name, func(t *testing.T) {
			gotResults, gotResponse, err := CollectionFilteredPaginate(
				ctx,
				m,
				tc.req,
				tc.filter,
				func(key, value uint64) (collections.KeyValue[uint64, uint64], error) {
					return collections.KeyValue[uint64, uint64]{Key: key, Value: value}, nil
				},
			)
			if tc.wantErr != nil {
				require.ErrorIs(t, err, tc.wantErr)
				return
			}
			require.NoError(t, err)
			require.Equal(t, tc.expResults, gotResults)
			require.Equal(t, tc.expResp, gotResponse)
		})
	}
}

type testStore struct {
	db store.KVStoreWithBatch
}

func (t testStore) OpenKVStore(ctx context.Context) store.KVStore {
	return t
}

func (t testStore) Get(key []byte) ([]byte, error) {
	return t.db.Get(key)
}

func (t testStore) Has(key []byte) (bool, error) {
	return t.db.Has(key)
}

func (t testStore) Set(key, value []byte) error {
	return t.db.Set(key, value)
}

func (t testStore) Delete(key []byte) error {
	return t.db.Delete(key)
}

func (t testStore) Iterator(start, end []byte) (store.Iterator, error) {
	return t.db.Iterator(start, end)
}

func (t testStore) ReverseIterator(start, end []byte) (store.Iterator, error) {
	return t.db.ReverseIterator(start, end)
}

var _ store.KVStore = testStore{}

func deps() (store.KVStoreService, context.Context) {
	kv := coretesting.NewMemDB()
	return &testStore{kv}, context.Background()
}
