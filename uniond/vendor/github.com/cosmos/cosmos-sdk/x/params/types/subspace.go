package types

import (
	"fmt"
	"reflect"

	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

const (
	// StoreKey is the string store key for the param store
	StoreKey = "params"

	// TStoreKey is the string store key for the param transient store
	TStoreKey = "transient_params"
)

// Individual parameter store for each keeper
// Transient store persists for a block, so we use it for
// recording whether the parameter has been changed or not
type Subspace struct {
	cdc         codec.BinaryCodec
	legacyAmino *codec.LegacyAmino
	key         storetypes.StoreKey // []byte -> []byte, stores parameter
	tkey        storetypes.StoreKey // []byte -> bool, stores parameter change
	name        []byte
	table       KeyTable
}

// NewSubspace constructs a store with namestore
func NewSubspace(cdc codec.BinaryCodec, legacyAmino *codec.LegacyAmino, key, tkey storetypes.StoreKey, name string) Subspace {
	return Subspace{
		cdc:         cdc,
		legacyAmino: legacyAmino,
		key:         key,
		tkey:        tkey,
		name:        []byte(name),
		table:       NewKeyTable(),
	}
}

// HasKeyTable returns if the Subspace has a KeyTable registered.
func (s Subspace) HasKeyTable() bool {
	return len(s.table.m) > 0
}

// WithKeyTable initializes KeyTable and returns modified Subspace
func (s Subspace) WithKeyTable(table KeyTable) Subspace {
	if table.m == nil {
		panic("WithKeyTable() called with nil KeyTable")
	}
	if len(s.table.m) != 0 {
		panic("WithKeyTable() called on already initialized Subspace")
	}

	for k, v := range table.m {
		s.table.m[k] = v
	}

	// Allocate additional capacity for Subspace.name
	// So we don't have to allocate extra space each time appending to the key
	name := s.name
	s.name = make([]byte, len(name), len(name)+table.maxKeyLength())
	copy(s.name, name)

	return s
}

// Returns a KVStore identical with ctx.KVStore(s.key).Prefix()
func (s Subspace) kvStore(ctx sdk.Context) storetypes.KVStore {
	// append here is safe, appends within a function won't cause
	// weird side effects when its singlethreaded
	return prefix.NewStore(ctx.KVStore(s.key), append(s.name, '/'))
}

// Returns a transient store for modification
func (s Subspace) transientStore(ctx sdk.Context) storetypes.KVStore {
	// append here is safe, appends within a function won't cause
	// weird side effects when its singlethreaded
	return prefix.NewStore(ctx.TransientStore(s.tkey), append(s.name, '/'))
}

// Validate attempts to validate a parameter value by its key. If the key is not
// registered or if the validation of the value fails, an error is returned.
func (s Subspace) Validate(ctx sdk.Context, key []byte, value interface{}) error {
	attr, ok := s.table.m[string(key)]
	if !ok {
		return fmt.Errorf("parameter %s not registered", key)
	}

	if err := attr.vfn(value); err != nil {
		return fmt.Errorf("invalid parameter value: %s", err)
	}

	return nil
}

// Get queries for a parameter by key from the Subspace's KVStore and sets the
// value to the provided pointer. If the value does not exist, it will panic.
func (s Subspace) Get(ctx sdk.Context, key []byte, ptr interface{}) {
	s.checkType(key, ptr)

	store := s.kvStore(ctx)
	bz := store.Get(key)

	if err := s.legacyAmino.UnmarshalJSON(bz, ptr); err != nil {
		panic(err)
	}
}

// GetIfExists queries for a parameter by key from the Subspace's KVStore and
// sets the value to the provided pointer. If the value does not exist, it will
// perform a no-op.
func (s Subspace) GetIfExists(ctx sdk.Context, key []byte, ptr interface{}) {
	store := s.kvStore(ctx)
	bz := store.Get(key)
	if bz == nil {
		return
	}

	s.checkType(key, ptr)

	if err := s.legacyAmino.UnmarshalJSON(bz, ptr); err != nil {
		panic(err)
	}
}

// IterateKeys iterates over all the keys in the subspace and executes the
// provided callback. If the callback returns true for a given key, iteration
// will halt.
func (s Subspace) IterateKeys(ctx sdk.Context, cb func(key []byte) bool) {
	store := s.kvStore(ctx)

	iter := storetypes.KVStorePrefixIterator(store, nil)
	defer iter.Close()

	for ; iter.Valid(); iter.Next() {
		if cb(iter.Key()) {
			break
		}
	}
}

// GetRaw queries for the raw values bytes for a parameter by key.
func (s Subspace) GetRaw(ctx sdk.Context, key []byte) []byte {
	store := s.kvStore(ctx)
	return store.Get(key)
}

// Has returns if a parameter key exists or not in the Subspace's KVStore.
func (s Subspace) Has(ctx sdk.Context, key []byte) bool {
	store := s.kvStore(ctx)
	return store.Has(key)
}

// Modified returns true if the parameter key is set in the Subspace's transient
// KVStore.
func (s Subspace) Modified(ctx sdk.Context, key []byte) bool {
	tstore := s.transientStore(ctx)
	return tstore.Has(key)
}

// checkType verifies that the provided key and value are comptable and registered.
func (s Subspace) checkType(key []byte, value interface{}) {
	attr, ok := s.table.m[string(key)]
	if !ok {
		panic(fmt.Sprintf("parameter %s not registered", key))
	}

	ty := attr.ty
	pty := reflect.TypeOf(value)
	if pty.Kind() == reflect.Ptr {
		pty = pty.Elem()
	}

	if pty != ty {
		panic("type mismatch with registered table")
	}
}

// Set stores a value for given a parameter key assuming the parameter type has
// been registered. It will panic if the parameter type has not been registered
// or if the value cannot be encoded. A change record is also set in the Subspace's
// transient KVStore to mark the parameter as modified.
func (s Subspace) Set(ctx sdk.Context, key []byte, value interface{}) {
	s.checkType(key, value)
	store := s.kvStore(ctx)

	bz, err := s.legacyAmino.MarshalJSON(value)
	if err != nil {
		panic(err)
	}

	store.Set(key, bz)

	tstore := s.transientStore(ctx)
	tstore.Set(key, []byte{})
}

// Update stores an updated raw value for a given parameter key assuming the
// parameter type has been registered. It will panic if the parameter type has
// not been registered or if the value cannot be encoded. An error is returned
// if the raw value is not compatible with the registered type for the parameter
// key or if the new value is invalid as determined by the registered type's
// validation function.
func (s Subspace) Update(ctx sdk.Context, key, value []byte) error {
	attr, ok := s.table.m[string(key)]
	if !ok {
		panic(fmt.Sprintf("parameter %s not registered", key))
	}

	ty := attr.ty
	dest := reflect.New(ty).Interface()
	s.GetIfExists(ctx, key, dest)

	if err := s.legacyAmino.UnmarshalJSON(value, dest); err != nil {
		return err
	}

	// destValue contains the dereferenced value of dest so validation function do
	// not have to operate on pointers.
	destValue := reflect.Indirect(reflect.ValueOf(dest)).Interface()
	if err := s.Validate(ctx, key, destValue); err != nil {
		return err
	}

	s.Set(ctx, key, dest)
	return nil
}

// GetParamSet iterates through each ParamSetPair where for each pair, it will
// retrieve the value and set it to the corresponding value pointer provided
// in the ParamSetPair by calling Subspace#Get.
func (s Subspace) GetParamSet(ctx sdk.Context, ps ParamSet) {
	for _, pair := range ps.ParamSetPairs() {
		s.Get(ctx, pair.Key, pair.Value)
	}
}

// GetParamSetIfExists iterates through each ParamSetPair where for each pair, it will
// retrieve the value and set it to the corresponding value pointer provided
// in the ParamSetPair by calling Subspace#GetIfExists.
func (s Subspace) GetParamSetIfExists(ctx sdk.Context, ps ParamSet) {
	for _, pair := range ps.ParamSetPairs() {
		s.GetIfExists(ctx, pair.Key, pair.Value)
	}
}

// SetParamSet iterates through each ParamSetPair and sets the value with the
// corresponding parameter key in the Subspace's KVStore.
func (s Subspace) SetParamSet(ctx sdk.Context, ps ParamSet) {
	for _, pair := range ps.ParamSetPairs() {
		// pair.Field is a pointer to the field, so indirecting the ptr.
		// go-amino automatically handles it but just for sure,
		// since SetStruct is meant to be used in InitGenesis
		// so this method will not be called frequently
		v := reflect.Indirect(reflect.ValueOf(pair.Value)).Interface()

		if err := pair.ValidatorFn(v); err != nil {
			panic(fmt.Sprintf("value from ParamSetPair is invalid: %s", err))
		}

		s.Set(ctx, pair.Key, v)
	}
}

// Name returns the name of the Subspace.
func (s Subspace) Name() string {
	return string(s.name)
}

// Wrapper of Subspace, provides immutable functions only
type ReadOnlySubspace struct {
	s Subspace
}

// Get delegates a read-only Get call to the Subspace.
func (ros ReadOnlySubspace) Get(ctx sdk.Context, key []byte, ptr interface{}) {
	ros.s.Get(ctx, key, ptr)
}

// GetRaw delegates a read-only GetRaw call to the Subspace.
func (ros ReadOnlySubspace) GetRaw(ctx sdk.Context, key []byte) []byte {
	return ros.s.GetRaw(ctx, key)
}

// Has delegates a read-only Has call to the Subspace.
func (ros ReadOnlySubspace) Has(ctx sdk.Context, key []byte) bool {
	return ros.s.Has(ctx, key)
}

// Modified delegates a read-only Modified call to the Subspace.
func (ros ReadOnlySubspace) Modified(ctx sdk.Context, key []byte) bool {
	return ros.s.Modified(ctx, key)
}

// Name delegates a read-only Name call to the Subspace.
func (ros ReadOnlySubspace) Name() string {
	return ros.s.Name()
}
