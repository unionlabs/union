package orm

import (
	"bytes"
	"reflect"

	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/core/address"
	storetypes "cosmossdk.io/core/store"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/types"
	"cosmossdk.io/x/group/errors"
	"cosmossdk.io/x/group/internal/orm/prefixstore"

	"github.com/cosmos/cosmos-sdk/codec"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

var (
	_ Indexable       = &table{}
	_ TableExportable = &table{}
)

// table is the high level object to storage mapper functionality. Persistent
// entities are stored by an unique identifier called `RowID`. The table struct
// does not:
// - enforce uniqueness of the `RowID`
// - enforce prefix uniqueness of keys, i.e. not allowing one key to be a prefix
// of another
// - optimize Gas usage conditions
// The caller must ensure that these things are handled. The table struct is
// private, so that we only have custom tables built on top of table, that do satisfy
// these requirements.
type table struct {
	model       reflect.Type
	prefix      [2]byte
	afterSet    []AfterSetInterceptor
	afterDelete []AfterDeleteInterceptor
	cdc         codec.Codec
	ac          address.Codec
}

// newTable creates a new table
func newTable(prefix [2]byte, model proto.Message, cdc codec.Codec, addressCodec address.Codec) (*table, error) {
	if model == nil {
		return nil, errors.ErrORMInvalidArgument.Wrap("Model must not be nil")
	}
	tp := reflect.TypeOf(model)
	if tp.Kind() == reflect.Ptr {
		tp = tp.Elem()
	}
	return &table{
		prefix: prefix,
		model:  tp,
		cdc:    cdc,
		ac:     addressCodec,
	}, nil
}

// RowGetter returns a type safe RowGetter.
func (a table) RowGetter() RowGetter {
	return NewTypeSafeRowGetter(a.prefix, a.model, a.cdc)
}

// AddAfterSetInterceptor can be used to register a callback function that is executed after an object is created and/or updated.
func (a *table) AddAfterSetInterceptor(interceptor AfterSetInterceptor) {
	a.afterSet = append(a.afterSet, interceptor)
}

// AddAfterDeleteInterceptor can be used to register a callback function that is executed after an object is deleted.
func (a *table) AddAfterDeleteInterceptor(interceptor AfterDeleteInterceptor) {
	a.afterDelete = append(a.afterDelete, interceptor)
}

// Create persists the given object under the rowID key, returning an
// errors.ErrORMUniqueConstraint if a value already exists at that key.
//
// Create iterates through the registered callbacks that may add secondary index
// keys.
func (a table) Create(store storetypes.KVStore, rowID RowID, obj proto.Message) error {
	if a.Has(store, rowID) {
		return errors.ErrORMUniqueConstraint
	}

	return a.Set(store, rowID, obj)
}

// Update updates the given object under the rowID key. It expects the key to
// exists already and fails with an `sdkerrors.ErrNotFound` otherwise. Any caller must
// therefore make sure that this contract is fulfilled. Parameters must not be
// nil.
//
// Update triggers all "after set" hooks that may add or remove secondary index keys.
func (a table) Update(store storetypes.KVStore, rowID RowID, newValue proto.Message) error {
	if !a.Has(store, rowID) {
		return sdkerrors.ErrNotFound
	}

	return a.Set(store, rowID, newValue)
}

// Set persists the given object under the rowID key. It does not check if the
// key already exists and overwrites the value if it does.
//
// Set iterates through the registered callbacks that may add secondary index
// keys.
func (a table) Set(store storetypes.KVStore, rowID RowID, newValue proto.Message) error {
	if len(rowID) == 0 {
		return errors.ErrORMEmptyKey
	}
	if err := assertCorrectType(a.model, newValue); err != nil {
		return err
	}
	if err := assertValid(newValue); err != nil {
		return err
	}

	var oldValue proto.Message
	if a.Has(store, rowID) {
		oldValue = reflect.New(a.model).Interface().(proto.Message)
		err := a.GetOne(store, rowID, oldValue)
		if err != nil {
			return err
		}

	}

	newValueEncoded, err := a.cdc.Marshal(newValue)
	if err != nil {
		return errorsmod.Wrapf(err, "failed to serialize %T", newValue)
	}

	pStore := prefixstore.New(store, a.prefix[:])
	err = pStore.Set(rowID, newValueEncoded)
	if err != nil {
		return err
	}
	for i, itc := range a.afterSet {
		if err := itc(store, rowID, newValue, oldValue); err != nil {
			return errorsmod.Wrapf(err, "interceptor %d failed", i)
		}
	}
	return nil
}

func assertValid(obj proto.Message) error {
	if v, ok := obj.(Validateable); ok {
		if err := v.ValidateBasic(); err != nil {
			return err
		}
	}
	return nil
}

// Delete removes the object under the rowID key. It expects the key to exists
// already and fails with a `sdkerrors.ErrNotFound` otherwise. Any caller must therefore
// make sure that this contract is fulfilled.
//
// Delete iterates through the registered callbacks that remove secondary index
// keys.
func (a table) Delete(store storetypes.KVStore, rowID RowID) error {
	oldValue := reflect.New(a.model).Interface().(proto.Message)
	if err := a.GetOne(store, rowID, oldValue); err != nil {
		return errorsmod.Wrap(err, "load old value")
	}

	pStore := prefixstore.New(store, a.prefix[:])
	err := pStore.Delete(rowID)
	if err != nil {
		return err
	}

	for i, itc := range a.afterDelete {
		if err := itc(store, rowID, oldValue); err != nil {
			return errorsmod.Wrapf(err, "delete interceptor %d failed", i)
		}
	}
	return nil
}

// Has checks if a key exists. Returns false when the key is empty or nil
// because we don't allow creation of values without a key.
func (a table) Has(store storetypes.KVStore, key RowID) bool {
	if len(key) == 0 {
		return false
	}
	pStore := prefixstore.New(store, a.prefix[:])
	has, err := pStore.Has(key)
	if err != nil {
		panic(err)
	}
	return has
}

// GetOne load the object persisted for the given RowID into the dest parameter.
// If none exists or `rowID==nil` then `sdkerrors.ErrNotFound` is returned instead.
// Parameters must not be nil - we don't allow creation of values with empty keys.
func (a table) GetOne(store storetypes.KVStore, rowID RowID, dest proto.Message) error {
	if len(rowID) == 0 {
		return sdkerrors.ErrNotFound
	}
	x := NewTypeSafeRowGetter(a.prefix, a.model, a.cdc)
	return x(store, rowID, dest)
}

// PrefixScan returns an Iterator over a domain of keys in ascending order. End is exclusive.
// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid.
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
func (a table) PrefixScan(store storetypes.KVStore, start, end RowID) (Iterator, error) {
	if start != nil && end != nil && bytes.Compare(start, end) >= 0 {
		return NewInvalidIterator(), errorsmod.Wrap(errors.ErrORMInvalidArgument, "start must be before end")
	}

	pStore := prefixstore.New(store, a.prefix[:])
	it, err := pStore.Iterator(start, end)
	if err != nil {
		return nil, err
	}

	return &typeSafeIterator{
		store:     store,
		rowGetter: NewTypeSafeRowGetter(a.prefix, a.model, a.cdc),
		it:        it,
	}, nil
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
func (a table) ReversePrefixScan(store storetypes.KVStore, start, end RowID) (Iterator, error) {
	if start != nil && end != nil && bytes.Compare(start, end) >= 0 {
		return NewInvalidIterator(), errorsmod.Wrap(errors.ErrORMInvalidArgument, "start must be before end")
	}

	pStore := prefixstore.New(store, a.prefix[:])
	it, err := pStore.ReverseIterator(start, end)
	if err != nil {
		return nil, err
	}

	return &typeSafeIterator{
		store:     store,
		rowGetter: NewTypeSafeRowGetter(a.prefix, a.model, a.cdc),
		it:        it,
	}, nil
}

// Export stores all the values in the table in the passed ModelSlicePtr.
func (a table) Export(store storetypes.KVStore, dest ModelSlicePtr) (uint64, error) {
	it, err := a.PrefixScan(store, nil, nil)
	if err != nil {
		return 0, errorsmod.Wrap(err, "table Export failure when exporting table data")
	}
	_, err = ReadAll(it, dest)
	if err != nil {
		return 0, err
	}
	return 0, nil
}

// Import clears the table and initializes it from the given data interface{}.
// data should be a slice of structs that implement PrimaryKeyed.
func (a table) Import(store storetypes.KVStore, data interface{}, _ uint64) error {
	// Clear all data
	keys := a.keys(store)
	for _, key := range keys {
		if err := a.Delete(store, key); err != nil {
			return err
		}
	}

	// Provided data must be a slice
	modelSlice := reflect.ValueOf(data)
	if modelSlice.Kind() != reflect.Slice {
		return errorsmod.Wrap(errors.ErrORMInvalidArgument, "data must be a slice")
	}

	// Import values from slice
	for i := 0; i < modelSlice.Len(); i++ {
		obj, ok := modelSlice.Index(i).Interface().(PrimaryKeyed)
		if !ok {
			return errorsmod.Wrapf(errors.ErrORMInvalidArgument, "unsupported type :%s", reflect.TypeOf(data).Elem().Elem())
		}
		err := a.Create(store, PrimaryKey(obj, a.ac), obj)
		if err != nil {
			return err
		}
	}

	return nil
}

func (a table) keys(store storetypes.KVStore) [][]byte {
	pStore := prefixstore.New(store, a.prefix[:])
	it, err := pStore.ReverseIterator(nil, nil)
	if err != nil {
		panic(err)
	}
	defer it.Close()

	var keys [][]byte
	for ; it.Valid(); it.Next() {
		keys = append(keys, it.Key())
	}
	return keys
}

// typeSafeIterator is initialized with a type safe RowGetter only.
type typeSafeIterator struct {
	store     storetypes.KVStore
	rowGetter RowGetter
	it        types.Iterator
}

func (i typeSafeIterator) LoadNext(dest proto.Message) (RowID, error) {
	if !i.it.Valid() {
		return nil, errors.ErrORMIteratorDone
	}
	rowID := i.it.Key()
	i.it.Next()
	return rowID, i.rowGetter(i.store, rowID, dest)
}

func (i typeSafeIterator) Close() error {
	return i.it.Close()
}
