package types

import (
	"io"

	"github.com/cosmos/cosmos-sdk/codec"
)

// WriteListener interface for streaming data out from a KVStore
type WriteListener interface {
	// if value is nil then it was deleted
	// storeKey indicates the source KVStore, to facilitate using the same WriteListener across separate KVStores
	// delete bool indicates if it was a delete; true: delete, false: set
	OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) error
}

// StoreKVPairWriteListener is used to configure listening to a KVStore by
// writing out length-prefixed Protobuf encoded StoreKVPairs to an underlying
// io.Writer object.
type StoreKVPairWriteListener struct {
	writer     io.Writer
	marshaller codec.BinaryCodec
}

// NewStoreKVPairWriteListener wraps creates a StoreKVPairWriteListener with a
// provided io.Writer and codec.BinaryCodec.
func NewStoreKVPairWriteListener(w io.Writer, m codec.Codec) *StoreKVPairWriteListener {
	return &StoreKVPairWriteListener{
		writer:     w,
		marshaller: m,
	}
}

// OnWrite satisfies the WriteListener interface by writing length-prefixed
// Protobuf encoded StoreKVPairs.
func (wl *StoreKVPairWriteListener) OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) error {
	kvPair := &StoreKVPair{
		StoreKey: storeKey.Name(),
		Key:      key,
		Value:    value,
		Delete:   delete,
	}

	by, err := wl.marshaller.MarshalLengthPrefixed(kvPair)
	if err != nil {
		return err
	}

	if _, err := wl.writer.Write(by); err != nil {
		return err
	}

	return nil
}

// MemoryListener listens to the state writes and accumulate the records in memory.
type MemoryListener struct {
	key        StoreKey
	stateCache []StoreKVPair
}

// NewMemoryListener creates a listener that accumulate the state writes in memory.
func NewMemoryListener(key StoreKey) *MemoryListener {
	return &MemoryListener{key: key}
}

// OnWrite implements WriteListener interface.
func (fl *MemoryListener) OnWrite(storeKey StoreKey, key []byte, value []byte, delete bool) error {
	fl.stateCache = append(fl.stateCache, StoreKVPair{
		StoreKey: storeKey.Name(),
		Delete:   delete,
		Key:      key,
		Value:    value,
	})

	return nil
}

// PopStateCache returns the current state caches and set to nil.
func (fl *MemoryListener) PopStateCache() []StoreKVPair {
	res := fl.stateCache
	fl.stateCache = nil

	return res
}

// StoreKey returns the storeKey it listens to.
func (fl *MemoryListener) StoreKey() StoreKey {
	return fl.key
}
