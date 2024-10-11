package orm

import (
	stdErrors "errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	corestore "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm/prefixstore"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestNewIndexer(t *testing.T) {
	testCases := []struct {
		name        string
		indexerFunc IndexerFunc
		expectErr   bool
		expectedErr string
	}{
		{
			name:        "nil indexer func",
			indexerFunc: nil,
			expectErr:   true,
			expectedErr: "Indexer func must not be nil",
		},
		{
			name:        "all not nil",
			indexerFunc: func(interface{}) ([]interface{}, error) { return nil, nil },
			expectErr:   false,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			indexer, err := NewIndexer(tc.indexerFunc)
			if tc.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expectedErr)
			} else {
				require.NoError(t, err)
				require.NotNil(t, indexer)
			}
		})
	}
}

func TestNewUniqueIndexer(t *testing.T) {
	testCases := []struct {
		name        string
		indexerFunc UniqueIndexerFunc
		expectErr   bool
		expectedErr string
	}{
		{
			name:        "nil indexer func",
			indexerFunc: nil,
			expectErr:   true,
			expectedErr: "Indexer func must not be nil",
		},
		{
			name:        "all not nil",
			indexerFunc: func(interface{}) (interface{}, error) { return nil, nil },
			expectErr:   false,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			indexer, err := NewUniqueIndexer(tc.indexerFunc)
			if tc.expectErr {
				require.Error(t, err)
				require.Contains(t, err.Error(), tc.expectedErr)
			} else {
				require.NoError(t, err)
				require.NotNil(t, indexer)
			}
		})
	}
}

func TestIndexerOnCreate(t *testing.T) {
	var myRowID RowID = EncodeSequence(1)

	specs := map[string]struct {
		srcFunc          IndexerFunc
		expIndexKeys     []interface{}
		expRowIDs        []RowID
		expAddFuncCalled bool
		expErr           error
	}{
		"single key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1)}, nil
			},
			expAddFuncCalled: true,
			expIndexKeys:     []interface{}{uint64(1)},
			expRowIDs:        []RowID{myRowID},
		},
		"multi key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1), uint64(128)}, nil
			},
			expAddFuncCalled: true,
			expIndexKeys:     []interface{}{uint64(1), uint64(128)},
			expRowIDs:        []RowID{myRowID, myRowID},
		},
		"empty key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{[]byte{}}, nil
			},
			expAddFuncCalled: false,
		},
		"nil key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{nil}, nil
			},
			expErr:           fmt.Errorf("type %T not allowed as key part", nil),
			expAddFuncCalled: false,
		},
		"empty key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{}, nil
			},
			expAddFuncCalled: false,
		},
		"nil key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, nil
			},
			expAddFuncCalled: false,
		},
		"error case": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, stdErrors.New("test")
			},
			expErr:           stdErrors.New("test"),
			expAddFuncCalled: false,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			mockPolicy := &addFuncRecorder{}
			idx, err := NewIndexer(spec.srcFunc)
			require.NoError(t, err)
			idx.addFunc = mockPolicy.add

			err = idx.OnCreate(nil, myRowID, nil)
			if spec.expErr != nil {
				require.Equal(t, spec.expErr, err)
				return
			}
			require.NoError(t, err)
			assert.Equal(t, spec.expIndexKeys, mockPolicy.secondaryIndexKeys)
			assert.Equal(t, spec.expRowIDs, mockPolicy.rowIDs)
			assert.Equal(t, spec.expAddFuncCalled, mockPolicy.called)
		})
	}
}

func TestIndexerOnDelete(t *testing.T) {
	myRowID := EncodeSequence(1)

	var multiKeyIndex MultiKeyIndex
	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := prefixstore.New(runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx), []byte{multiKeyIndex.prefix})

	specs := map[string]struct {
		srcFunc        IndexerFunc
		expDeletedKeys []RowID
		expErr         error
	}{
		"single key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1)}, nil
			},
			expDeletedKeys: []RowID{append(EncodeSequence(1), myRowID...)},
		},
		"multi key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1), uint64(128)}, nil
			},
			expDeletedKeys: []RowID{
				append(EncodeSequence(1), myRowID...),
				append(EncodeSequence(128), myRowID...),
			},
		},
		"empty key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{}, nil
			},
		},
		"nil key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, nil
			},
		},
		"empty key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{[]byte{}}, nil
			},
		},
		"nil key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{nil}, nil
			},
			expErr: fmt.Errorf("type %T not allowed as key part", nil),
		},
		"error case": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, stdErrors.New("test")
			},
			expErr: stdErrors.New("test"),
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			idx, err := NewIndexer(spec.srcFunc)
			require.NoError(t, err)

			if spec.expErr == nil {
				err = idx.OnCreate(store, myRowID, nil)
				require.NoError(t, err)
				for _, key := range spec.expDeletedKeys {
					has, err := store.Has(key)
					require.NoError(t, err)
					require.True(t, has)
				}
			}

			err = idx.OnDelete(store, myRowID, nil)
			if spec.expErr != nil {
				require.Equal(t, spec.expErr, err)
				return
			}
			require.NoError(t, err)
			for _, key := range spec.expDeletedKeys {
				has, err := store.Has(key)
				require.NoError(t, err)
				require.False(t, has)
			}
		})
	}
}

func TestIndexerOnUpdate(t *testing.T) {
	myRowID := EncodeSequence(1)

	var multiKeyIndex MultiKeyIndex
	key := storetypes.NewKVStoreKey("test")
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	store := prefixstore.New(runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx), []byte{multiKeyIndex.prefix})

	specs := map[string]struct {
		srcFunc        IndexerFunc
		expAddedKeys   []RowID
		expDeletedKeys []RowID
		expErr         error
		addFunc        func(corestore.KVStore, interface{}, RowID) error
	}{
		"single key - same key, no update": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1)}, nil
			},
		},
		"single key - different key, replaced": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				keys := []uint64{1, 2}
				return []interface{}{keys[value.(int)]}, nil
			},
			expAddedKeys: []RowID{
				append(EncodeSequence(2), myRowID...),
			},
			expDeletedKeys: []RowID{
				append(EncodeSequence(1), myRowID...),
			},
		},
		"multi key - same key, no update": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{uint64(1), uint64(2)}, nil
			},
		},
		"multi key - replaced": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				keys := []uint64{1, 2, 3, 4}
				return []interface{}{keys[value.(int)], keys[value.(int)+2]}, nil
			},
			expAddedKeys: []RowID{
				append(EncodeSequence(2), myRowID...),
				append(EncodeSequence(4), myRowID...),
			},
			expDeletedKeys: []RowID{
				append(EncodeSequence(1), myRowID...),
				append(EncodeSequence(3), myRowID...),
			},
		},
		"empty key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{}, nil
			},
		},
		"nil key": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, nil
			},
		},
		"empty key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{[]byte{}}, nil
			},
		},
		"nil key in slice": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return []interface{}{nil}, nil
			},
			expErr: fmt.Errorf("type %T not allowed as key part", nil),
		},
		"error case with new value": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				return nil, stdErrors.New("test")
			},
			expErr: stdErrors.New("test"),
		},
		"error case with old value": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				var err error
				if value.(int)%2 == 1 {
					err = stdErrors.New("test")
				}
				return []interface{}{uint64(1)}, err
			},
			expErr: stdErrors.New("test"),
		},
		"error case on persisting new keys": {
			srcFunc: func(value interface{}) ([]interface{}, error) {
				keys := []uint64{1, 2}
				return []interface{}{keys[value.(int)]}, nil
			},
			addFunc: func(_ corestore.KVStore, _ interface{}, _ RowID) error {
				return stdErrors.New("test")
			},
			expErr: stdErrors.New("test"),
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			idx, err := NewIndexer(spec.srcFunc)
			require.NoError(t, err)

			if spec.expErr == nil {
				err = idx.OnCreate(store, myRowID, 0)
				require.NoError(t, err)
			}

			if spec.addFunc != nil {
				idx.addFunc = spec.addFunc
			}
			err = idx.OnUpdate(store, myRowID, 1, 0)
			if spec.expErr != nil {
				require.Equal(t, spec.expErr, err)
				return
			}
			require.NoError(t, err)
			for _, key := range spec.expAddedKeys {
				has, err := store.Has(key)
				assert.NoError(t, err)
				assert.True(t, has)
			}
			for _, key := range spec.expDeletedKeys {
				has, err := store.Has(key)
				assert.NoError(t, err)
				assert.False(t, has)
			}
		})
	}
}

func TestUniqueKeyAddFunc(t *testing.T) {
	myRowID := EncodeSequence(1)
	presetKeyPart := []byte("my-preset-key")
	presetKey := append(AddLengthPrefix(presetKeyPart), myRowID...)

	specs := map[string]struct {
		srcKey           []byte
		expErr           *errorsmod.Error
		expExistingEntry []byte
	}{
		"create when not exists": {
			srcKey:           []byte("my-index-key"),
			expExistingEntry: append(AddLengthPrefix([]byte("my-index-key")), myRowID...),
		},
		"error when exists already": {
			srcKey: presetKeyPart,
			expErr: errors.ErrORMUniqueConstraint,
		},
		"nil key not allowed": {
			srcKey: nil,
			expErr: errors.ErrORMInvalidArgument,
		},
		"empty key not allowed": {
			srcKey: []byte{},
			expErr: errors.ErrORMInvalidArgument,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			key := storetypes.NewKVStoreKey("test")
			testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
			store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)
			require.NoError(t, store.Set(presetKey, []byte{}))

			err := uniqueKeysAddFunc(store, spec.srcKey, myRowID)
			require.True(t, spec.expErr.Is(err))
			if spec.expErr != nil {
				return
			}

			has, err := store.Has(spec.expExistingEntry)
			assert.NoError(t, err)
			assert.True(t, has, "not found")
		})
	}
}

func TestMultiKeyAddFunc(t *testing.T) {
	myRowID := EncodeSequence(1)
	presetKeyPart := []byte("my-preset-key")
	presetKey := append(AddLengthPrefix(presetKeyPart), myRowID...)

	specs := map[string]struct {
		srcKey           []byte
		expErr           *errorsmod.Error
		expExistingEntry []byte
	}{
		"create when not exists": {
			srcKey:           []byte("my-index-key"),
			expExistingEntry: append(AddLengthPrefix([]byte("my-index-key")), myRowID...),
		},
		"noop when exists already": {
			srcKey:           presetKeyPart,
			expExistingEntry: presetKey,
		},
		"nil key not allowed": {
			srcKey: nil,
			expErr: errors.ErrORMInvalidArgument,
		},
		"empty key not allowed": {
			srcKey: []byte{},
			expErr: errors.ErrORMInvalidArgument,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			key := storetypes.NewKVStoreKey("test")
			testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
			store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)
			require.NoError(t, store.Set(presetKey, []byte{}))

			err := multiKeyAddFunc(store, spec.srcKey, myRowID)
			require.True(t, spec.expErr.Is(err))
			if spec.expErr != nil {
				return
			}

			has, err := store.Has(spec.expExistingEntry)
			assert.NoError(t, err)
			assert.True(t, has)
		})
	}
}

func TestDifference(t *testing.T) {
	specs := map[string]struct {
		srcA      []interface{}
		srcB      []interface{}
		expResult []interface{}
		expErr    bool
	}{
		"all of A": {
			srcA:      []interface{}{"a", "b"},
			srcB:      []interface{}{"c"},
			expResult: []interface{}{"a", "b"},
		},
		"A - B": {
			srcA:      []interface{}{"a", "b"},
			srcB:      []interface{}{"b", "c", "d"},
			expResult: []interface{}{"a"},
		},
		"type in A not allowed": {
			srcA:   []interface{}{1},
			srcB:   []interface{}{"b", "c", "d"},
			expErr: true,
		},
		"type in B not allowed": {
			srcA:   []interface{}{"b", "c", "d"},
			srcB:   []interface{}{1},
			expErr: true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			got, err := difference(spec.srcA, spec.srcB)
			if spec.expErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				assert.Equal(t, spec.expResult, got)
			}
		})
	}
}

func TestPruneEmptyKeys(t *testing.T) {
	specs := map[string]struct {
		srcFunc   IndexerFunc
		expResult []interface{}
		expError  error
	}{
		"non empty": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return []interface{}{uint64(0), uint64(1)}, nil
			},
			expResult: []interface{}{uint64(0), uint64(1)},
		},
		"empty": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return []interface{}{}, nil
			},
			expResult: []interface{}{},
		},
		"nil": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return nil, nil
			},
		},
		"empty in the beginning": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return []interface{}{[]byte{}, uint64(0), uint64(1)}, nil
			},
			expResult: []interface{}{uint64(0), uint64(1)},
		},
		"empty in the middle": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return []interface{}{uint64(0), []byte{}, uint64(1)}, nil
			},
			expResult: []interface{}{uint64(0), uint64(1)},
		},
		"empty at the end": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return []interface{}{uint64(0), uint64(1), []byte{}}, nil
			},
			expResult: []interface{}{uint64(0), uint64(1)},
		},
		"error passed": {
			srcFunc: func(v interface{}) ([]interface{}, error) {
				return nil, stdErrors.New("test")
			},
			expError: stdErrors.New("test"),
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			r, err := pruneEmptyKeys(spec.srcFunc)(nil)
			require.Equal(t, spec.expError, err)
			if spec.expError != nil {
				return
			}
			assert.Equal(t, spec.expResult, r)
		})
	}
}

type addFuncRecorder struct {
	secondaryIndexKeys []interface{}
	rowIDs             []RowID
	called             bool
}

func (c *addFuncRecorder) add(_ corestore.KVStore, key interface{}, rowID RowID) error {
	c.secondaryIndexKeys = append(c.secondaryIndexKeys, key)
	c.rowIDs = append(c.rowIDs, rowID)
	c.called = true
	return nil
}
