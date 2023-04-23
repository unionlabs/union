package orm

import (
	"bytes"
	"reflect"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	"github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
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
}

// newTable creates a new table
func newTable(prefix [2]byte, model codec.ProtoMarshaler, cdc codec.Codec) (*table, error) {
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
func (a table) Create(store sdk.KVStore, rowID RowID, obj codec.ProtoMarshaler) error {
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
func (a table) Update(store sdk.KVStore, rowID RowID, newValue codec.ProtoMarshaler) error {
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
func (a table) Set(store sdk.KVStore, rowID RowID, newValue codec.ProtoMarshaler) error {
	if len(rowID) == 0 {
		return errors.ErrORMEmptyKey
	}
	if err := assertCorrectType(a.model, newValue); err != nil {
		return err
	}
	if err := assertValid(newValue); err != nil {
		return err
	}

	pStore := prefix.NewStore(store, a.prefix[:])

	var oldValue codec.ProtoMarshaler
	if a.Has(store, rowID) {
		oldValue = reflect.New(a.model).Interface().(codec.ProtoMarshaler)
		a.GetOne(store, rowID, oldValue)
	}

	newValueEncoded, err := a.cdc.Marshal(newValue)
	if err != nil {
		return sdkerrors.Wrapf(err, "failed to serialize %T", newValue)
	}

	pStore.Set(rowID, newValueEncoded)
	for i, itc := range a.afterSet {
		if err := itc(store, rowID, newValue, oldValue); err != nil {
			return sdkerrors.Wrapf(err, "interceptor %d failed", i)
		}
	}
	return nil
}

func assertValid(obj codec.ProtoMarshaler) error {
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
func (a table) Delete(store sdk.KVStore, rowID RowID) error {
	pStore := prefix.NewStore(store, a.prefix[:])

	oldValue := reflect.New(a.model).Interface().(codec.ProtoMarshaler)
	if err := a.GetOne(store, rowID, oldValue); err != nil {
		return sdkerrors.Wrap(err, "load old value")
	}
	pStore.Delete(rowID)

	for i, itc := range a.afterDelete {
		if err := itc(store, rowID, oldValue); err != nil {
			return sdkerrors.Wrapf(err, "delete interceptor %d failed", i)
		}
	}
	return nil
}

// Has checks if a key exists. Returns false when the key is empty or nil
// because we don't allow creation of values without a key.
func (a table) Has(store sdk.KVStore, key RowID) bool {
	if len(key) == 0 {
		return false
	}
	pStore := prefix.NewStore(store, a.prefix[:])
	return pStore.Has(key)
}

// GetOne load the object persisted for the given RowID into the dest parameter.
// If none exists or `rowID==nil` then `sdkerrors.ErrNotFound` is returned instead.
// Parameters must not be nil - we don't allow creation of values with empty keys.
func (a table) GetOne(store sdk.KVStore, rowID RowID, dest codec.ProtoMarshaler) error {
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
func (a table) PrefixScan(store sdk.KVStore, start, end RowID) (Iterator, error) {
	if start != nil && end != nil && bytes.Compare(start, end) >= 0 {
		return NewInvalidIterator(), sdkerrors.Wrap(errors.ErrORMInvalidArgument, "start must be before end")
	}
	pStore := prefix.NewStore(store, a.prefix[:])
	return &typeSafeIterator{
		store:     store,
		rowGetter: NewTypeSafeRowGetter(a.prefix, a.model, a.cdc),
		it:        pStore.Iterator(start, end),
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
func (a table) ReversePrefixScan(store sdk.KVStore, start, end RowID) (Iterator, error) {
	if start != nil && end != nil && bytes.Compare(start, end) >= 0 {
		return NewInvalidIterator(), sdkerrors.Wrap(errors.ErrORMInvalidArgument, "start must be before end")
	}
	pStore := prefix.NewStore(store, a.prefix[:])
	return &typeSafeIterator{
		store:     store,
		rowGetter: NewTypeSafeRowGetter(a.prefix, a.model, a.cdc),
		it:        pStore.ReverseIterator(start, end),
	}, nil
}

// Export stores all the values in the table in the passed ModelSlicePtr.
func (a table) Export(store sdk.KVStore, dest ModelSlicePtr) (uint64, error) {
	it, err := a.PrefixScan(store, nil, nil)
	if err != nil {
		return 0, sdkerrors.Wrap(err, "table Export failure when exporting table data")
	}
	_, err = ReadAll(it, dest)
	if err != nil {
		return 0, err
	}
	return 0, nil
}

// Import clears the table and initializes it from the given data interface{}.
// data should be a slice of structs that implement PrimaryKeyed.
func (a table) Import(store sdk.KVStore, data interface{}, _ uint64) error {
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
		return sdkerrors.Wrap(errors.ErrORMInvalidArgument, "data must be a slice")
	}

	// Import values from slice
	for i := 0; i < modelSlice.Len(); i++ {
		obj, ok := modelSlice.Index(i).Interface().(PrimaryKeyed)
		if !ok {
			return sdkerrors.Wrapf(errors.ErrORMInvalidArgument, "unsupported type :%s", reflect.TypeOf(data).Elem().Elem())
		}
		err := a.Create(store, PrimaryKey(obj), obj)
		if err != nil {
			return err
		}
	}

	return nil
}

func (a table) keys(store sdk.KVStore) [][]byte {
	pStore := prefix.NewStore(store, a.prefix[:])
	it := pStore.Iterator(nil, nil)
	defer it.Close()

	var keys [][]byte
	for ; it.Valid(); it.Next() {
		keys = append(keys, it.Key())
	}
	return keys
}

// typeSafeIterator is initialized with a type safe RowGetter only.
type typeSafeIterator struct {
	store     sdk.KVStore
	rowGetter RowGetter
	it        types.Iterator
}

func (i typeSafeIterator) LoadNext(dest codec.ProtoMarshaler) (RowID, error) {
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
