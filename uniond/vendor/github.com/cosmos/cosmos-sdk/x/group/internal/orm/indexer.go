package orm

import (
	errorsmod "cosmossdk.io/errors"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// IndexerFunc creates one or multiple index keys for the source object.
type IndexerFunc func(value interface{}) ([]interface{}, error)

// IndexerFunc creates exactly one index key for the source object.
type UniqueIndexerFunc func(value interface{}) (interface{}, error)

// Indexer manages the persistence of an Index based on searchable keys and operations.
type Indexer struct {
	indexerFunc IndexerFunc
	addFunc     func(store storetypes.KVStore, secondaryIndexKey interface{}, rowID RowID) error
}

// NewIndexer returns an indexer that supports multiple reference keys for an entity.
func NewIndexer(indexerFunc IndexerFunc) (*Indexer, error) {
	if indexerFunc == nil {
		return nil, errors.ErrORMInvalidArgument.Wrap("Indexer func must not be nil")
	}
	return &Indexer{
		indexerFunc: pruneEmptyKeys(indexerFunc),
		addFunc:     multiKeyAddFunc,
	}, nil
}

// NewUniqueIndexer returns an indexer that requires exactly one reference keys for an entity.
func NewUniqueIndexer(f UniqueIndexerFunc) (*Indexer, error) {
	if f == nil {
		return nil, errors.ErrORMInvalidArgument.Wrap("Indexer func must not be nil")
	}
	adaptor := func(indexerFunc UniqueIndexerFunc) IndexerFunc {
		return func(v interface{}) ([]interface{}, error) {
			k, err := indexerFunc(v)
			return []interface{}{k}, err
		}
	}
	idx, err := NewIndexer(adaptor(f))
	if err != nil {
		return nil, err
	}
	idx.addFunc = uniqueKeysAddFunc
	return idx, nil
}

// IndexerFunc returns the indexer IndexerFunc,
// ensuring it has been prune from empty keys.
func (i Indexer) IndexerFunc() IndexerFunc {
	return i.indexerFunc
}

// OnCreate persists the secondary index entries for the new object.
func (i Indexer) OnCreate(store storetypes.KVStore, rowID RowID, value interface{}) error {
	secondaryIndexKeys, err := i.indexerFunc(value)
	if err != nil {
		return err
	}

	for _, secondaryIndexKey := range secondaryIndexKeys {
		if err := i.addFunc(store, secondaryIndexKey, []byte(rowID)); err != nil {
			return err
		}
	}
	return nil
}

// OnDelete removes the secondary index entries for the deleted object.
func (i Indexer) OnDelete(store storetypes.KVStore, rowID RowID, value interface{}) error {
	secondaryIndexKeys, err := i.indexerFunc(value)
	if err != nil {
		return err
	}

	for _, secondaryIndexKey := range secondaryIndexKeys {
		indexKey, err := buildKeyFromParts([]interface{}{secondaryIndexKey, []byte(rowID)})
		if err != nil {
			return err
		}
		store.Delete(indexKey)
	}
	return nil
}

// OnUpdate rebuilds the secondary index entries for the updated object.
func (i Indexer) OnUpdate(store storetypes.KVStore, rowID RowID, newValue, oldValue interface{}) error {
	oldSecIdxKeys, err := i.indexerFunc(oldValue)
	if err != nil {
		return err
	}
	newSecIdxKeys, err := i.indexerFunc(newValue)
	if err != nil {
		return err
	}
	oldKeys, err := difference(oldSecIdxKeys, newSecIdxKeys)
	if err != nil {
		return err
	}
	for _, oldIdxKey := range oldKeys {
		indexKey, err := buildKeyFromParts([]interface{}{oldIdxKey, []byte(rowID)})
		if err != nil {
			return err
		}
		store.Delete(indexKey)
	}
	newKeys, err := difference(newSecIdxKeys, oldSecIdxKeys)
	if err != nil {
		return err
	}
	for _, newIdxKey := range newKeys {
		if err := i.addFunc(store, newIdxKey, rowID); err != nil {
			return err
		}
	}
	return nil
}

// uniqueKeysAddFunc enforces keys to be unique
func uniqueKeysAddFunc(store storetypes.KVStore, secondaryIndexKey interface{}, rowID RowID) error {
	secondaryIndexKeyBytes, err := keyPartBytes(secondaryIndexKey, false)
	if err != nil {
		return err
	}
	if len(secondaryIndexKeyBytes) == 0 {
		return errorsmod.Wrap(errors.ErrORMInvalidArgument, "empty index key")
	}

	if err := checkUniqueIndexKey(store, secondaryIndexKeyBytes); err != nil {
		return err
	}

	indexKey, err := buildKeyFromParts([]interface{}{secondaryIndexKey, []byte(rowID)})
	if err != nil {
		return err
	}

	store.Set(indexKey, []byte{})
	return nil
}

// checkUniqueIndexKey checks that the given secondary index key is unique
func checkUniqueIndexKey(store storetypes.KVStore, secondaryIndexKeyBytes []byte) error {
	it := store.Iterator(PrefixRange(secondaryIndexKeyBytes))
	defer it.Close()
	if it.Valid() {
		return errors.ErrORMUniqueConstraint
	}
	return nil
}

// multiKeyAddFunc allows multiple entries for a key
func multiKeyAddFunc(store storetypes.KVStore, secondaryIndexKey interface{}, rowID RowID) error {
	secondaryIndexKeyBytes, err := keyPartBytes(secondaryIndexKey, false)
	if err != nil {
		return err
	}
	if len(secondaryIndexKeyBytes) == 0 {
		return errorsmod.Wrap(errors.ErrORMInvalidArgument, "empty index key")
	}

	encodedKey, err := buildKeyFromParts([]interface{}{secondaryIndexKey, []byte(rowID)})
	if err != nil {
		return err
	}
	if len(encodedKey) == 0 {
		return errorsmod.Wrap(errors.ErrORMInvalidArgument, "empty index key")
	}

	store.Set(encodedKey, []byte{})
	return nil
}

// difference returns the list of elements that are in a but not in b.
func difference(a, b []interface{}) ([]interface{}, error) {
	set := make(map[interface{}]struct{}, len(b))
	for _, v := range b {
		bt, err := keyPartBytes(v, true)
		if err != nil {
			return nil, err
		}
		set[string(bt)] = struct{}{}
	}
	var result []interface{}
	for _, v := range a {
		bt, err := keyPartBytes(v, true)
		if err != nil {
			return nil, err
		}
		if _, ok := set[string(bt)]; !ok {
			result = append(result, v)
		}
	}
	return result, nil
}

// pruneEmptyKeys drops any empty key from IndexerFunc f returned
func pruneEmptyKeys(f IndexerFunc) IndexerFunc {
	return func(v interface{}) ([]interface{}, error) {
		keys, err := f(v)
		if err != nil || keys == nil {
			return keys, err
		}
		r := make([]interface{}, 0, len(keys))
		for i := range keys {
			key, err := keyPartBytes(keys[i], true)
			if err != nil {
				return nil, err
			}
			if len(key) != 0 {
				r = append(r, keys[i])
			}
		}
		return r, nil
	}
}
