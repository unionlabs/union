/*
Package orm is a convenient object to data store mapper.
*/
package orm

import (
	"io"
	"reflect"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// Unique identifier of a persistent table.
type RowID []byte

// Bytes returns raw bytes.
func (r RowID) Bytes() []byte {
	return r
}

// Validateable is an interface that ProtoMarshaler types can implement and is called on any orm save or update operation.
type Validateable interface {
	// ValidateBasic is a sanity check on the data. Any error returned prevents create or updates.
	ValidateBasic() error
}

// Index allows efficient prefix scans is stored as key = concat(indexKeyBytes, rowIDUint64) with value empty
// so that the row PrimaryKey is allows a fixed with 8 byte integer. This allows the MultiKeyIndex key bytes to be
// variable length and scanned iteratively.
type Index interface {
	// Has checks if a key exists. Panics on nil key.
	Has(store sdk.KVStore, key interface{}) (bool, error)

	// Get returns a result iterator for the searchKey.
	// searchKey must not be nil.
	Get(store sdk.KVStore, searchKey interface{}) (Iterator, error)

	// GetPaginated returns a result iterator for the searchKey and optional pageRequest.
	// searchKey must not be nil.
	GetPaginated(store sdk.KVStore, searchKey interface{}, pageRequest *query.PageRequest) (Iterator, error)

	// PrefixScan returns an Iterator over a domain of keys in ascending order. End is exclusive.
	// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid and error is returned.
	// Iterator must be closed by caller.
	// To iterate over entire domain, use PrefixScan(nil, nil)
	//
	// WARNING: The use of a PrefixScan can be very expensive in terms of Gas. Please make sure you do not expose
	// this as an endpoint to the public without further limits.
	// Example:
	//			it, err := idx.PrefixScan(ctx, start, end)
	//			if err !=nil {
	//				return err
	//			}
	//			const defaultLimit = 20
	//			it = LimitIterator(it, defaultLimit)
	//
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	PrefixScan(store sdk.KVStore, startI interface{}, endI interface{}) (Iterator, error)

	// ReversePrefixScan returns an Iterator over a domain of keys in descending order. End is exclusive.
	// Start is an MultiKeyIndex key or prefix. It must be less than end, or the Iterator is invalid  and error is returned.
	// Iterator must be closed by caller.
	// To iterate over entire domain, use PrefixScan(nil, nil)
	//
	// WARNING: The use of a ReversePrefixScan can be very expensive in terms of Gas. Please make sure you do not expose
	// this as an endpoint to the public without further limits. See `LimitIterator`
	//
	// CONTRACT: No writes may happen within a domain while an iterator exists over it.
	ReversePrefixScan(store sdk.KVStore, startI interface{}, endI interface{}) (Iterator, error)
}

// Iterator allows iteration through a sequence of key value pairs
type Iterator interface {
	// LoadNext loads the next value in the sequence into the pointer passed as dest and returns the key. If there
	// are no more items the ErrORMIteratorDone error is returned
	// The key is the rowID.
	LoadNext(dest codec.ProtoMarshaler) (RowID, error)
	// Close releases the iterator and should be called at the end of iteration
	io.Closer
}

// Indexable types are used to setup new tables.
// This interface provides a set of functions that can be called by indexes to register and interact with the tables.
type Indexable interface {
	RowGetter() RowGetter
	AddAfterSetInterceptor(interceptor AfterSetInterceptor)
	AddAfterDeleteInterceptor(interceptor AfterDeleteInterceptor)
}

// AfterSetInterceptor defines a callback function to be called on Create + Update.
type AfterSetInterceptor func(store sdk.KVStore, rowID RowID, newValue, oldValue codec.ProtoMarshaler) error

// AfterDeleteInterceptor defines a callback function to be called on Delete operations.
type AfterDeleteInterceptor func(store sdk.KVStore, rowID RowID, value codec.ProtoMarshaler) error

// RowGetter loads a persistent object by row ID into the destination object. The dest parameter must therefore be a pointer.
// Any implementation must return `sdkerrors.ErrNotFound` when no object for the rowID exists
type RowGetter func(store sdk.KVStore, rowID RowID, dest codec.ProtoMarshaler) error

// NewTypeSafeRowGetter returns a `RowGetter` with type check on the dest parameter.
func NewTypeSafeRowGetter(prefixKey [2]byte, model reflect.Type, cdc codec.Codec) RowGetter {
	return func(store sdk.KVStore, rowID RowID, dest codec.ProtoMarshaler) error {
		if len(rowID) == 0 {
			return sdkerrors.Wrap(errors.ErrORMEmptyKey, "key must not be nil")
		}
		if err := assertCorrectType(model, dest); err != nil {
			return err
		}

		pStore := prefix.NewStore(store, prefixKey[:])
		bz := pStore.Get(rowID)
		if len(bz) == 0 {
			return sdkerrors.ErrNotFound
		}

		return cdc.Unmarshal(bz, dest)
	}
}

func assertCorrectType(model reflect.Type, obj codec.ProtoMarshaler) error {
	tp := reflect.TypeOf(obj)
	if tp.Kind() != reflect.Ptr {
		return sdkerrors.Wrap(sdkerrors.ErrInvalidType, "model destination must be a pointer")
	}
	if model != tp.Elem() {
		return sdkerrors.Wrapf(sdkerrors.ErrInvalidType, "can not use %T with this bucket", obj)
	}
	return nil
}
