package orm

import (
	"github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/core/address"
	storetypes "cosmossdk.io/core/store"
	"cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
)

var (
	_ Indexable       = &AutoUInt64Table{}
	_ TableExportable = &AutoUInt64Table{}
)

// AutoUInt64Table is the table type with an auto incrementing ID.
type AutoUInt64Table struct {
	*table
	seq Sequence
}

// NewAutoUInt64Table creates a new AutoUInt64Table.
func NewAutoUInt64Table(prefixData [2]byte, prefixSeq byte, model proto.Message, cdc codec.Codec, addressCodec address.Codec) (*AutoUInt64Table, error) {
	table, err := newTable(prefixData, model, cdc, addressCodec)
	if err != nil {
		return nil, err
	}
	return &AutoUInt64Table{
		table: table,
		seq:   NewSequence(prefixSeq),
	}, nil
}

// Create a new persistent object with an auto generated uint64 primary key. The
// key is returned.
//
// Create iterates through the registered callbacks that may add secondary index
// keys.
func (a AutoUInt64Table) Create(store storetypes.KVStore, obj proto.Message) (uint64, error) {
	autoIncID := a.seq.NextVal(store)
	err := a.table.Create(store, EncodeSequence(autoIncID), obj)
	if err != nil {
		return 0, err
	}
	return autoIncID, nil
}

// Update updates the given object under the rowID key. It expects the key to
// exists already and fails with an `ErrNotFound` otherwise. Any caller must
// therefore make sure that this contract is fulfilled. Parameters must not be
// nil.
//
// Update iterates through the registered callbacks that may add or remove
// secondary index keys.
func (a AutoUInt64Table) Update(store storetypes.KVStore, rowID uint64, newValue proto.Message) error {
	return a.table.Update(store, EncodeSequence(rowID), newValue)
}

// Delete removes the object under the rowID key. It expects the key to exists already
// and fails with a `ErrNotFound` otherwise. Any caller must therefore make sure that this contract
// is fulfilled.
//
// Delete iterates though the registered callbacks and removes secondary index keys by them.
func (a AutoUInt64Table) Delete(store storetypes.KVStore, rowID uint64) error {
	return a.table.Delete(store, EncodeSequence(rowID))
}

// Has checks if a rowID exists.
func (a AutoUInt64Table) Has(store storetypes.KVStore, rowID uint64) bool {
	return a.table.Has(store, EncodeSequence(rowID))
}

// GetOne load the object persisted for the given RowID into the dest parameter.
// If none exists `ErrNotFound` is returned instead. Parameters must not be nil.
func (a AutoUInt64Table) GetOne(store storetypes.KVStore, rowID uint64, dest proto.Message) (RowID, error) {
	rawRowID := EncodeSequence(rowID)
	if err := a.table.GetOne(store, rawRowID, dest); err != nil {
		return nil, err
	}
	return rawRowID, nil
}

// PrefixScan returns an Iterator over a domain of keys in ascending order. End is exclusive.
// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid and error is returned.
// Iterator must be closed by caller.
// To iterate over entire domain, use PrefixScan(1, math.MaxUint64)
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
func (a AutoUInt64Table) PrefixScan(store storetypes.KVStore, start, end uint64) (Iterator, error) {
	return a.table.PrefixScan(store, EncodeSequence(start), EncodeSequence(end))
}

// ReversePrefixScan returns an Iterator over a domain of keys in descending order. End is exclusive.
// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid  and error is returned.
// Iterator must be closed by caller.
// To iterate over entire domain, use PrefixScan(1, math.MaxUint64)
//
// WARNING: The use of a ReversePrefixScan can be very expensive in terms of Gas. Please make sure you do not expose
// this as an endpoint to the public without further limits. See `LimitIterator`
//
// CONTRACT: No writes may happen within a domain while an iterator exists over it.
func (a AutoUInt64Table) ReversePrefixScan(store storetypes.KVStore, start, end uint64) (Iterator, error) {
	return a.table.ReversePrefixScan(store, EncodeSequence(start), EncodeSequence(end))
}

// Sequence returns the sequence used by this table
func (a AutoUInt64Table) Sequence() Sequence {
	return a.seq
}

// Export stores all the values in the table in the passed ModelSlicePtr and
// returns the current value of the associated sequence.
func (a AutoUInt64Table) Export(store storetypes.KVStore, dest ModelSlicePtr) (uint64, error) {
	_, err := a.table.Export(store, dest)
	if err != nil {
		return 0, err
	}
	return a.seq.CurVal(store), nil
}

// Import clears the table and initializes it from the given data interface{}.
// data should be a slice of structs that implement PrimaryKeyed.
func (a AutoUInt64Table) Import(store storetypes.KVStore, data interface{}, seqValue uint64) error {
	if err := a.seq.InitVal(store, seqValue); err != nil {
		return errors.Wrap(err, "sequence")
	}
	return a.table.Import(store, data, seqValue)
}
