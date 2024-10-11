package orm

import (
	"bytes"

	"github.com/cosmos/gogoproto/proto"

	storetypes "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm/prefixstore"

	"github.com/cosmos/cosmos-sdk/types/query"
)

// indexer creates and modifies the second MultiKeyIndex based on the operations and changes on the primary object.
type indexer interface {
	OnCreate(store storetypes.KVStore, rowID RowID, value interface{}) error
	OnDelete(store storetypes.KVStore, rowID RowID, value interface{}) error
	OnUpdate(store storetypes.KVStore, rowID RowID, newValue, oldValue interface{}) error
}

var _ Index = &MultiKeyIndex{}

// MultiKeyIndex is an index where multiple entries can point to the same underlying object as opposite to a unique index
// where only one entry is allowed.
type MultiKeyIndex struct {
	prefix      byte
	rowGetter   RowGetter
	indexer     indexer
	indexerFunc IndexerFunc
	indexKey    interface{}
}

// NewIndex builds a MultiKeyIndex.
// Only single-field indexes are supported and `indexKey` represents such a field value,
// which can be []byte, string or uint64.
func NewIndex(tb Indexable, prefix byte, indexerF IndexerFunc, indexKey interface{}) (MultiKeyIndex, error) {
	indexer, err := NewIndexer(indexerF)
	if err != nil {
		return MultiKeyIndex{}, err
	}
	return newIndex(tb, prefix, indexer, indexer.IndexerFunc(), indexKey)
}

func newIndex(tb Indexable, prefix byte, indexer *Indexer, indexerF IndexerFunc, indexKey interface{}) (MultiKeyIndex, error) {
	rowGetter := tb.RowGetter()
	if rowGetter == nil {
		return MultiKeyIndex{}, errors.ErrORMInvalidArgument.Wrap("rowGetter must not be nil")
	}
	if indexKey == nil {
		return MultiKeyIndex{}, errors.ErrORMInvalidArgument.Wrap("indexKey must not be nil")
	}

	// Verify indexKey type is bytes, string or uint64
	switch indexKey.(type) {
	case []byte, string, uint64:
	default:
		return MultiKeyIndex{}, errors.ErrORMInvalidArgument.Wrap("indexKey must be []byte, string or uint64")
	}

	idx := MultiKeyIndex{
		prefix:      prefix,
		rowGetter:   rowGetter,
		indexer:     indexer,
		indexerFunc: indexerF,
		indexKey:    indexKey,
	}
	tb.AddAfterSetInterceptor(idx.onSet)
	tb.AddAfterDeleteInterceptor(idx.onDelete)
	return idx, nil
}

// Has checks if a key exists. Returns an error on nil key.
func (i MultiKeyIndex) Has(store storetypes.KVStore, key interface{}) (bool, error) {
	encodedKey, err := keyPartBytes(key, false)
	if err != nil {
		return false, err
	}

	pStore := prefixstore.New(store, []byte{i.prefix})
	it, err := pStore.Iterator(PrefixRange(encodedKey))
	if err != nil {
		return false, err
	}
	defer it.Close()
	return it.Valid(), nil
}

// Get returns a result iterator for the searchKey. Parameters must not be nil.
func (i MultiKeyIndex) Get(store storetypes.KVStore, searchKey interface{}) (Iterator, error) {
	encodedKey, err := keyPartBytes(searchKey, false)
	if err != nil {
		return nil, err
	}

	pStore := prefixstore.New(store, []byte{i.prefix})
	it, err := pStore.Iterator(PrefixRange(encodedKey))
	if err != nil {
		return nil, err
	}
	return indexIterator{store: store, it: it, rowGetter: i.rowGetter, indexKey: i.indexKey}, nil
}

// GetPaginated creates an iterator for the searchKey
// starting from pageRequest.Key if provided.
// The pageRequest.Key is the rowID while searchKey is a MultiKeyIndex key.
func (i MultiKeyIndex) GetPaginated(store storetypes.KVStore, searchKey interface{}, pageRequest *query.PageRequest) (Iterator, error) {
	encodedKey, err := keyPartBytes(searchKey, false)
	if err != nil {
		return nil, err
	}
	start, end := PrefixRange(encodedKey)

	if pageRequest != nil && len(pageRequest.Key) != 0 {
		var err error
		start, err = buildKeyFromParts([]interface{}{searchKey, pageRequest.Key})
		if err != nil {
			return nil, err
		}
	}

	pStore := prefixstore.New(store, []byte{i.prefix})
	it, err := pStore.Iterator(start, end)
	if err != nil {
		return nil, err
	}
	return indexIterator{store: store, it: it, rowGetter: i.rowGetter, indexKey: i.indexKey}, nil
}

// PrefixScan returns an Iterator over a domain of keys in ascending order. End is exclusive.
// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid and error is returned.
// Iterator must be closed by caller.
// To iterate over entire domain, use PrefixScan(nil, nil)
//
// WARNING: The use of a PrefixScan can be very expensive in terms of Gas. Please make sure you do not expose
// this as an endpoint to the public without further limits.
// Example:
//
//	it, err := idx.PrefixScan(ctx, start, end)
//	if err !=nil {
//		return err
//	}
//	const defaultLimit = 20
//	it = LimitIterator(it, defaultLimit)
//
// CONTRACT: No writes may happen within a domain while an iterator exists over it.
func (i MultiKeyIndex) PrefixScan(store storetypes.KVStore, startI, endI interface{}) (Iterator, error) {
	start, end, err := getStartEndBz(startI, endI)
	if err != nil {
		return nil, err
	}

	pStore := prefixstore.New(store, []byte{i.prefix})
	it, err := pStore.Iterator(start, end)
	if err != nil {
		return nil, err
	}

	return indexIterator{store: store, it: it, rowGetter: i.rowGetter, indexKey: i.indexKey}, nil
}

// ReversePrefixScan returns an Iterator over a domain of keys in descending order. End is exclusive.
// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid  and error is returned.
// Iterator must be closed by caller.
// To iterate over entire domain, use PrefixScan(nil, nil)
//
// WARNING: The use of a ReversePrefixScan can be very expensive in terms of Gas. Please make sure you do not expose
// this as an endpoint to the public without further limits. See `LimitIterator`
//
// CONTRACT: No writes may happen within a domain while an iterator exists over it.
func (i MultiKeyIndex) ReversePrefixScan(store storetypes.KVStore, startI, endI interface{}) (Iterator, error) {
	start, end, err := getStartEndBz(startI, endI)
	if err != nil {
		return nil, err
	}

	pStore := prefixstore.New(store, []byte{i.prefix})
	it, err := pStore.ReverseIterator(start, end)
	if err != nil {
		return nil, err
	}

	return indexIterator{store: store, it: it, rowGetter: i.rowGetter, indexKey: i.indexKey}, nil
}

// getStartEndBz gets the start and end bytes to be passed into the SDK store
// iterator.
func getStartEndBz(startI, endI interface{}) ([]byte, []byte, error) {
	start, err := getPrefixScanKeyBytes(startI)
	if err != nil {
		return nil, nil, err
	}
	end, err := getPrefixScanKeyBytes(endI)
	if err != nil {
		return nil, nil, err
	}

	if start != nil && end != nil && bytes.Compare(start, end) >= 0 {
		return nil, nil, errorsmod.Wrap(errors.ErrORMInvalidArgument, "start must be less than end")
	}

	return start, end, nil
}

func getPrefixScanKeyBytes(keyI interface{}) ([]byte, error) {
	var (
		key []byte
		err error
	)
	// nil value are accepted in the context of PrefixScans
	if keyI == nil {
		return nil, nil
	}
	key, err = keyPartBytes(keyI, false)
	if err != nil {
		return nil, err
	}
	return key, nil
}

func (i MultiKeyIndex) onSet(store storetypes.KVStore, rowID RowID, newValue, oldValue proto.Message) error {
	pStore := prefixstore.New(store, []byte{i.prefix})
	if oldValue == nil {
		return i.indexer.OnCreate(pStore, rowID, newValue)
	}
	return i.indexer.OnUpdate(pStore, rowID, newValue, oldValue)
}

func (i MultiKeyIndex) onDelete(store storetypes.KVStore, rowID RowID, oldValue proto.Message) error {
	pStore := prefixstore.New(store, []byte{i.prefix})
	return i.indexer.OnDelete(pStore, rowID, oldValue)
}

type UniqueIndex struct {
	MultiKeyIndex
}

// NewUniqueIndex create a new Index object where duplicate keys are prohibited.
func NewUniqueIndex(tb Indexable, prefix byte, uniqueIndexerFunc UniqueIndexerFunc, indexKey interface{}) (UniqueIndex, error) {
	uniqueIndexer, err := NewUniqueIndexer(uniqueIndexerFunc)
	if err != nil {
		return UniqueIndex{}, err
	}
	multiKeyIndex, err := newIndex(tb, prefix, uniqueIndexer, uniqueIndexer.IndexerFunc(), indexKey)
	if err != nil {
		return UniqueIndex{}, err
	}
	return UniqueIndex{
		MultiKeyIndex: multiKeyIndex,
	}, nil
}

// indexIterator uses rowGetter to lazy load new model values on request.
type indexIterator struct {
	store     storetypes.KVStore
	rowGetter RowGetter
	it        types.Iterator
	indexKey  interface{}
}

// LoadNext loads the next value in the sequence into the pointer passed as dest and returns the key. If there
// are no more items the errors.ErrORMIteratorDone error is returned
// The key is the rowID and not any MultiKeyIndex key.
func (i indexIterator) LoadNext(dest proto.Message) (RowID, error) {
	if !i.it.Valid() {
		return nil, errors.ErrORMIteratorDone
	}
	indexPrefixKey := i.it.Key()
	rowID, err := stripRowID(indexPrefixKey, i.indexKey)
	if err != nil {
		return nil, err
	}
	i.it.Next()
	return rowID, i.rowGetter(i.store, rowID, dest)
}

// Close releases the iterator and should be called at the end of iteration
func (i indexIterator) Close() error {
	return i.it.Close()
}

// PrefixRange turns a prefix into a (start, end) range. The start is the given prefix value and
// the end is calculated by adding 1 bit to the start value. Nil is not allowed as prefix.
//
//	Example: []byte{1, 3, 4} becomes []byte{1, 3, 5}
//			 []byte{15, 42, 255, 255} becomes []byte{15, 43, 0, 0}
//
// In case of an overflow the end is set to nil.
//
//	Example: []byte{255, 255, 255, 255} becomes nil
func PrefixRange(prefix []byte) ([]byte, []byte) {
	if prefix == nil {
		panic("nil key not allowed")
	}
	// special case: no prefix is whole range
	if len(prefix) == 0 {
		return nil, nil
	}

	// copy the prefix and update last byte
	end := make([]byte, len(prefix))
	copy(end, prefix)
	l := len(end) - 1
	end[l]++

	// wait, what if that overflowed?....
	for end[l] == 0 && l > 0 {
		l--
		end[l]++
	}

	// okay, funny guy, you gave us FFF, no end to this range...
	if l == 0 && end[0] == 0 {
		end = nil
	}
	return prefix, end
}
