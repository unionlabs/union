package tracekv

import (
	"encoding/base64"
	"encoding/json"
	"io"

	"github.com/cosmos/cosmos-sdk/store/types"
	"github.com/cosmos/cosmos-sdk/types/errors"
)

const (
	writeOp     operation = "write"
	readOp      operation = "read"
	deleteOp    operation = "delete"
	iterKeyOp   operation = "iterKey"
	iterValueOp operation = "iterValue"
)

type (
	// Store implements the KVStore interface with tracing enabled.
	// Operations are traced on each core KVStore call and written to the
	// underlying io.writer.
	//
	// TODO: Should we use a buffered writer and implement Commit on
	// Store?
	Store struct {
		parent  types.KVStore
		writer  io.Writer
		context types.TraceContext
	}

	// operation represents an IO operation
	operation string

	// traceOperation implements a traced KVStore operation
	traceOperation struct {
		Operation operation              `json:"operation"`
		Key       string                 `json:"key"`
		Value     string                 `json:"value"`
		Metadata  map[string]interface{} `json:"metadata"`
	}
)

// NewStore returns a reference to a new traceKVStore given a parent
// KVStore implementation and a buffered writer.
func NewStore(parent types.KVStore, writer io.Writer, tc types.TraceContext) *Store {
	return &Store{parent: parent, writer: writer, context: tc}
}

// Get implements the KVStore interface. It traces a read operation and
// delegates a Get call to the parent KVStore.
func (tkv *Store) Get(key []byte) []byte {
	value := tkv.parent.Get(key)

	writeOperation(tkv.writer, readOp, tkv.context, key, value)
	return value
}

// Set implements the KVStore interface. It traces a write operation and
// delegates the Set call to the parent KVStore.
func (tkv *Store) Set(key []byte, value []byte) {
	types.AssertValidKey(key)
	writeOperation(tkv.writer, writeOp, tkv.context, key, value)
	tkv.parent.Set(key, value)
}

// Delete implements the KVStore interface. It traces a write operation and
// delegates the Delete call to the parent KVStore.
func (tkv *Store) Delete(key []byte) {
	writeOperation(tkv.writer, deleteOp, tkv.context, key, nil)
	tkv.parent.Delete(key)
}

// Has implements the KVStore interface. It delegates the Has call to the
// parent KVStore.
func (tkv *Store) Has(key []byte) bool {
	return tkv.parent.Has(key)
}

// Iterator implements the KVStore interface. It delegates the Iterator call
// to the parent KVStore.
func (tkv *Store) Iterator(start, end []byte) types.Iterator {
	return tkv.iterator(start, end, true)
}

// ReverseIterator implements the KVStore interface. It delegates the
// ReverseIterator call to the parent KVStore.
func (tkv *Store) ReverseIterator(start, end []byte) types.Iterator {
	return tkv.iterator(start, end, false)
}

// iterator facilitates iteration over a KVStore. It delegates the necessary
// calls to it's parent KVStore.
func (tkv *Store) iterator(start, end []byte, ascending bool) types.Iterator {
	var parent types.Iterator

	if ascending {
		parent = tkv.parent.Iterator(start, end)
	} else {
		parent = tkv.parent.ReverseIterator(start, end)
	}

	return newTraceIterator(tkv.writer, parent, tkv.context)
}

type traceIterator struct {
	parent  types.Iterator
	writer  io.Writer
	context types.TraceContext
}

func newTraceIterator(w io.Writer, parent types.Iterator, tc types.TraceContext) types.Iterator {
	return &traceIterator{writer: w, parent: parent, context: tc}
}

// Domain implements the Iterator interface.
func (ti *traceIterator) Domain() (start []byte, end []byte) {
	return ti.parent.Domain()
}

// Valid implements the Iterator interface.
func (ti *traceIterator) Valid() bool {
	return ti.parent.Valid()
}

// Next implements the Iterator interface.
func (ti *traceIterator) Next() {
	ti.parent.Next()
}

// Key implements the Iterator interface.
func (ti *traceIterator) Key() []byte {
	key := ti.parent.Key()

	writeOperation(ti.writer, iterKeyOp, ti.context, key, nil)
	return key
}

// Value implements the Iterator interface.
func (ti *traceIterator) Value() []byte {
	value := ti.parent.Value()

	writeOperation(ti.writer, iterValueOp, ti.context, nil, value)
	return value
}

// Close implements the Iterator interface.
func (ti *traceIterator) Close() error {
	return ti.parent.Close()
}

// Error delegates the Error call to the parent iterator.
func (ti *traceIterator) Error() error {
	return ti.parent.Error()
}

// GetStoreType implements the KVStore interface. It returns the underlying
// KVStore type.
func (tkv *Store) GetStoreType() types.StoreType {
	return tkv.parent.GetStoreType()
}

// CacheWrap implements the KVStore interface. It panics because a Store
// cannot be branched.
func (tkv *Store) CacheWrap() types.CacheWrap {
	panic("cannot CacheWrap a TraceKVStore")
}

// CacheWrapWithTrace implements the KVStore interface. It panics as a
// Store cannot be branched.
func (tkv *Store) CacheWrapWithTrace(_ io.Writer, _ types.TraceContext) types.CacheWrap {
	panic("cannot CacheWrapWithTrace a TraceKVStore")
}

// writeOperation writes a KVStore operation to the underlying io.Writer as
// JSON-encoded data where the key/value pair is base64 encoded.
func writeOperation(w io.Writer, op operation, tc types.TraceContext, key, value []byte) {
	traceOp := traceOperation{
		Operation: op,
		Key:       base64.StdEncoding.EncodeToString(key),
		Value:     base64.StdEncoding.EncodeToString(value),
	}

	if tc != nil {
		traceOp.Metadata = tc
	}

	raw, err := json.Marshal(traceOp)
	if err != nil {
		panic(errors.Wrap(err, "failed to serialize trace operation"))
	}

	if _, err := w.Write(raw); err != nil {
		panic(errors.Wrap(err, "failed to write trace operation"))
	}

	io.WriteString(w, "\n")
}
