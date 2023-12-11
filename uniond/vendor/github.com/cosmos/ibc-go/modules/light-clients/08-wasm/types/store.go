package types

import (
	"bytes"
	"context"
	"errors"
	"io"
	"reflect"
	"strings"

	wasmvm "github.com/CosmWasm/wasmvm"
	wasmvmtypes "github.com/CosmWasm/wasmvm/types"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/cachekv"
	storeprefix "cosmossdk.io/store/prefix"
	"cosmossdk.io/store/tracekv"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/ibc-go/modules/light-clients/08-wasm/internal/ibcwasm"
)

var (
	_ wasmvmtypes.KVStore = &storeAdapter{}
	_ storetypes.KVStore  = &migrateClientWrappedStore{}

	subjectPrefix    = []byte("subject/")
	substitutePrefix = []byte("substitute/")
)

// Checksum is a type alias used for wasm byte code checksums.
type Checksum = wasmvmtypes.Checksum

// CreateChecksum creates a sha256 checksum from the given wasm code, it forwards the
// call to the wasmvm package. The code is checked for the following conditions:
// - code length is zero.
// - code length is less than 4 bytes (magic number length).
// - code does not start with the wasm magic number.
func CreateChecksum(code []byte) (Checksum, error) {
	return wasmvm.CreateChecksum(code)
}

// GetAllChecksums is a helper to get all checksums from the store.
// It returns an empty slice if no checksums are found
func GetAllChecksums(ctx context.Context) ([]Checksum, error) {
	iterator, err := ibcwasm.Checksums.Iterate(ctx, nil)
	if err != nil {
		return nil, err
	}

	keys, err := iterator.Keys()
	if err != nil {
		return nil, err
	}

	checksums := []Checksum{}
	for _, key := range keys {
		checksums = append(checksums, key)
	}

	return checksums, nil
}

// HasChecksum returns true if the given checksum exists in the store and
// false otherwise.
func HasChecksum(ctx context.Context, checksum Checksum) bool {
	found, err := ibcwasm.Checksums.Has(ctx, checksum)
	if err != nil {
		return false
	}

	return found
}

// migrateClientWrappedStore combines two KVStores into one.
//
// Both stores are used for reads, but only the subjectStore is used for writes. For all operations, the key
// is checked to determine which store to use and must be prefixed with either "subject/" or "substitute/" accordingly.
// If the key is not prefixed with either "subject/" or "substitute/", a default action is taken (e.g. no-op for Set/Delete).
type migrateClientWrappedStore struct {
	subjectStore    storetypes.KVStore
	substituteStore storetypes.KVStore
}

func newMigrateClientWrappedStore(subjectStore, substituteStore storetypes.KVStore) migrateClientWrappedStore {
	if subjectStore == nil {
		panic(errors.New("subjectStore must not be nil"))
	}
	if substituteStore == nil {
		panic(errors.New("substituteStore must not be nil"))
	}

	return migrateClientWrappedStore{
		subjectStore:    subjectStore,
		substituteStore: substituteStore,
	}
}

// Get implements the storetypes.KVStore interface. It allows reads from both the subjectStore and substituteStore.
//
// Get will return an empty byte slice if the key is not prefixed with either "subject/" or "substitute/".
func (ws migrateClientWrappedStore) Get(key []byte) []byte {
	prefix, key := splitPrefix(key)

	store, found := ws.getStore(prefix)
	if !found {
		// return a nil byte slice as KVStore.Get() does by default
		return []byte(nil)
	}

	return store.Get(key)
}

// Has implements the storetypes.KVStore interface. It allows reads from both the subjectStore and substituteStore.
//
// Note: contracts do not have access to the Has method, it is only implemented here to satisfy the storetypes.KVStore interface.
func (ws migrateClientWrappedStore) Has(key []byte) bool {
	prefix, key := splitPrefix(key)

	store, found := ws.getStore(prefix)
	if !found {
		// return false as value when store is not found
		return false
	}

	return store.Has(key)
}

// Set implements the storetypes.KVStore interface. It allows writes solely to the subjectStore.
//
// Set will no-op if the key is not prefixed with "subject/".
func (ws migrateClientWrappedStore) Set(key, value []byte) {
	prefix, key := splitPrefix(key)
	if !bytes.Equal(prefix, subjectPrefix) {
		return // no-op
	}

	ws.subjectStore.Set(key, value)
}

// Delete implements the storetypes.KVStore interface. It allows deletions solely to the subjectStore.
//
// Delete will no-op if the key is not prefixed with "subject/".
func (ws migrateClientWrappedStore) Delete(key []byte) {
	prefix, key := splitPrefix(key)
	if !bytes.Equal(prefix, subjectPrefix) {
		return // no-op
	}

	ws.subjectStore.Delete(key)
}

// Iterator implements the storetypes.KVStore interface. It allows iteration over both the subjectStore and substituteStore.
//
// Iterator will return a closed iterator if the start or end keys are not prefixed with either "subject/" or "substitute/".
func (ws migrateClientWrappedStore) Iterator(start, end []byte) storetypes.Iterator {
	prefixStart, start := splitPrefix(start)
	prefixEnd, end := splitPrefix(end)

	if !bytes.Equal(prefixStart, prefixEnd) {
		return ws.closedIterator()
	}

	store, found := ws.getStore(prefixStart)
	if !found {
		return ws.closedIterator()
	}

	return store.Iterator(start, end)
}

// ReverseIterator implements the storetypes.KVStore interface. It allows iteration over both the subjectStore and substituteStore.
//
// ReverseIterator will return a closed iterator if the start or end keys are not prefixed with either "subject/" or "substitute/".
func (ws migrateClientWrappedStore) ReverseIterator(start, end []byte) storetypes.Iterator {
	prefixStart, start := splitPrefix(start)
	prefixEnd, end := splitPrefix(end)

	if !bytes.Equal(prefixStart, prefixEnd) {
		return ws.closedIterator()
	}

	store, found := ws.getStore(prefixStart)
	if !found {
		return ws.closedIterator()
	}

	return store.ReverseIterator(start, end)
}

// GetStoreType implements the storetypes.KVStore interface, it is implemented solely to satisfy the interface.
func (ws migrateClientWrappedStore) GetStoreType() storetypes.StoreType {
	return ws.substituteStore.GetStoreType()
}

// CacheWrap implements the storetypes.KVStore interface, it is implemented solely to satisfy the interface.
func (ws migrateClientWrappedStore) CacheWrap() storetypes.CacheWrap {
	return cachekv.NewStore(ws)
}

// CacheWrapWithTrace implements the storetypes.KVStore interface, it is implemented solely to satisfy the interface.
func (ws migrateClientWrappedStore) CacheWrapWithTrace(w io.Writer, tc storetypes.TraceContext) storetypes.CacheWrap {
	return cachekv.NewStore(tracekv.NewStore(ws, w, tc))
}

// getStore returns the store to be used for the given key and a boolean flag indicating if that store was found.
// If the key is prefixed with "subject/", the subjectStore is returned. If the key is prefixed with "substitute/",
// the substituteStore is returned.
//
// If the key is not prefixed with either "subject/" or "substitute/", a nil store is returned and the boolean flag is false.
func (ws migrateClientWrappedStore) getStore(prefix []byte) (storetypes.KVStore, bool) {
	if bytes.Equal(prefix, subjectPrefix) {
		return ws.subjectStore, true
	} else if bytes.Equal(prefix, substitutePrefix) {
		return ws.substituteStore, true
	}

	return nil, false
}

// closedIterator returns an iterator that is always closed, used when Iterator() or ReverseIterator() is called
// with an invalid prefix or start/end key.
func (ws migrateClientWrappedStore) closedIterator() storetypes.Iterator {
	// Create a dummy iterator that is always closed right away.
	it := ws.subjectStore.Iterator([]byte{0}, []byte{1})
	it.Close()

	return it
}

// splitPrefix splits the key into the prefix and the key itself, if the key is prefixed with either "subject/" or "substitute/".
// If the key is not prefixed with either "subject/" or "substitute/", the prefix is nil.
func splitPrefix(key []byte) ([]byte, []byte) {
	if bytes.HasPrefix(key, subjectPrefix) {
		return subjectPrefix, bytes.TrimPrefix(key, subjectPrefix)
	} else if bytes.HasPrefix(key, substitutePrefix) {
		return substitutePrefix, bytes.TrimPrefix(key, substitutePrefix)
	}

	return nil, key
}

// storeAdapter bridges the SDK store implementation to wasmvm one. It implements the wasmvmtypes.KVStore interface.
type storeAdapter struct {
	parent storetypes.KVStore
}

// newStoreAdapter constructor
func newStoreAdapter(s storetypes.KVStore) *storeAdapter {
	if s == nil {
		panic(errors.New("store must not be nil"))
	}
	return &storeAdapter{parent: s}
}

// Get implements the wasmvmtypes.KVStore interface.
func (s storeAdapter) Get(key []byte) []byte {
	return s.parent.Get(key)
}

// Set implements the wasmvmtypes.KVStore interface.
func (s storeAdapter) Set(key, value []byte) {
	s.parent.Set(key, value)
}

// Delete implements the wasmvmtypes.KVStore interface.
func (s storeAdapter) Delete(key []byte) {
	s.parent.Delete(key)
}

// Iterator implements the wasmvmtypes.KVStore interface.
func (s storeAdapter) Iterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.Iterator(start, end)
}

// ReverseIterator implements the wasmvmtypes.KVStore interface.
func (s storeAdapter) ReverseIterator(start, end []byte) wasmvmtypes.Iterator {
	return s.parent.ReverseIterator(start, end)
}

// getClientID extracts and validates the clientID from the clientStore's prefix.
//
// Due to the 02-client module not passing the clientID to the 08-wasm module,
// this function was devised to infer it from the store's prefix.
// The expected format of the clientStore prefix is "<placeholder>/{clientID}/".
// If the clientStore is of type migrateProposalWrappedStore, the subjectStore's prefix is utilized instead.
func getClientID(clientStore storetypes.KVStore) (string, error) {
	upws, isMigrateProposalWrappedStore := clientStore.(migrateClientWrappedStore)
	if isMigrateProposalWrappedStore {
		// if the clientStore is a migrateProposalWrappedStore, we retrieve the subjectStore
		// because the contract call will be made on the client with the ID of the subjectStore
		clientStore = upws.subjectStore
	}

	store, ok := clientStore.(storeprefix.Store)
	if !ok {
		return "", errorsmod.Wrap(ErrRetrieveClientID, "clientStore is not a prefix store")
	}

	// using reflect to retrieve the private prefix field
	r := reflect.ValueOf(&store).Elem()

	f := r.FieldByName("prefix")
	if !f.IsValid() {
		return "", errorsmod.Wrap(ErrRetrieveClientID, "prefix field not found")
	}

	prefix := string(f.Bytes())

	split := strings.Split(prefix, "/")
	if len(split) < 3 {
		return "", errorsmod.Wrap(ErrRetrieveClientID, "prefix is not of the expected form")
	}

	// the clientID is the second to last element of the prefix
	// the prefix is expected to be of the form "<placeholder>/{clientID}/"
	clientID := split[len(split)-2]
	if err := ValidateClientID(clientID); err != nil {
		return "", errorsmod.Wrapf(ErrRetrieveClientID, "prefix does not contain a valid clientID: %s", err.Error())
	}

	return clientID, nil
}
