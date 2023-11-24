package orm

import (
	"encoding/binary"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/prefix"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/x/group/errors"
)

// sequenceStorageKey is a fix key to read/ write data on the storage layer
var sequenceStorageKey = []byte{0x1}

// sequence is a persistent unique key generator based on a counter.
type Sequence struct {
	prefix byte
}

func NewSequence(prefix byte) Sequence {
	return Sequence{
		prefix: prefix,
	}
}

// NextVal increments and persists the counter by one and returns the value.
func (s Sequence) NextVal(store storetypes.KVStore) uint64 {
	pStore := prefix.NewStore(store, []byte{s.prefix})
	v := pStore.Get(sequenceStorageKey)
	seq := DecodeSequence(v)
	seq++
	pStore.Set(sequenceStorageKey, EncodeSequence(seq))
	return seq
}

// CurVal returns the last value used. 0 if none.
func (s Sequence) CurVal(store storetypes.KVStore) uint64 {
	pStore := prefix.NewStore(store, []byte{s.prefix})
	v := pStore.Get(sequenceStorageKey)
	return DecodeSequence(v)
}

// PeekNextVal returns the CurVal + increment step. Not persistent.
func (s Sequence) PeekNextVal(store storetypes.KVStore) uint64 {
	pStore := prefix.NewStore(store, []byte{s.prefix})
	v := pStore.Get(sequenceStorageKey)
	return DecodeSequence(v) + 1
}

// InitVal sets the start value for the sequence. It must be called only once on an empty DB.
// Otherwise an error is returned when the key exists. The given start value is stored as current
// value.
//
// It is recommended to call this method only for a sequence start value other than `1` as the
// method consumes unnecessary gas otherwise. A scenario would be an import from genesis.
func (s Sequence) InitVal(store storetypes.KVStore, seq uint64) error {
	pStore := prefix.NewStore(store, []byte{s.prefix})
	if pStore.Has(sequenceStorageKey) {
		return errorsmod.Wrap(errors.ErrORMUniqueConstraint, "already initialized")
	}
	pStore.Set(sequenceStorageKey, EncodeSequence(seq))
	return nil
}

// DecodeSequence converts the binary representation into an Uint64 value.
func DecodeSequence(bz []byte) uint64 {
	if bz == nil {
		return 0
	}
	val := binary.BigEndian.Uint64(bz)
	return val
}

// EncodedSeqLength number of bytes used for the binary representation of a sequence value.
const EncodedSeqLength = 8

// EncodeSequence converts the sequence value into the binary representation.
func EncodeSequence(val uint64) []byte {
	bz := make([]byte, EncodedSeqLength)
	binary.BigEndian.PutUint64(bz, val)
	return bz
}
