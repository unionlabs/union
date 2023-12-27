package collections

import (
	"context"
	"errors"
	io "io"
	"math"

	"cosmossdk.io/collections/codec"
)

var (
	// ErrNotFound is returned when the provided key is not present in the StorageProvider.
	ErrNotFound = errors.New("collections: not found")
	// ErrEncoding is returned when something fails during key or value encoding/decoding.
	ErrEncoding = codec.ErrEncoding
	// ErrConflict is returned when there are conflicts, for example in UniqueIndex.
	ErrConflict = errors.New("collections: conflict")
)

// KEYS

var (
	// Uint16Key can be used to encode uint16 keys. Encoding is big endian to retain ordering.
	Uint16Key = codec.NewUint16Key[uint16]()
	// Uint32Key can be used to encode uint32 keys. Encoding is big endian to retain ordering.
	Uint32Key = codec.NewUint32Key[uint32]()
	// Uint64Key can be used to encode uint64 keys. Encoding is big endian to retain ordering.
	Uint64Key = codec.NewUint64Key[uint64]()
	// Int32Key can be used to encode int32 keys. Encoding retains ordering by toggling the MSB.
	Int32Key = codec.NewInt32Key[int32]()
	// Int64Key can be used to encode int64. Encoding retains ordering by toggling the MSB.
	Int64Key = codec.NewInt64Key[int64]()
	// StringKey can be used to encode string keys. The encoding just converts the string
	// to bytes.
	// Non-terminality in multipart keys is handled by appending the StringDelimiter,
	// this means that a string key when used as the non final part of a multipart key cannot
	// contain the StringDelimiter.
	// Lexicographical ordering is retained both in non and multipart keys.
	StringKey = codec.NewStringKeyCodec[string]()
	// BytesKey can be used to encode bytes keys. The encoding will just use
	// the provided bytes.
	// When used as the non-terminal part of a multipart key, we prefix the bytes key
	// with a single byte representing the length of the key. This means two things:
	// 1. When used in multipart keys the length can be at maximum 255 (max number that
	// can be represented with a single byte).
	// 2. When used in multipart keys the lexicographical ordering is lost due to the
	// length prefixing.
	// JSON encoding represents a bytes key as a hex encoded string.
	BytesKey = codec.NewBytesKey[[]byte]()
	// BoolKey can be used to encode booleans. It uses a single byte to represent the boolean.
	// 0x0 is used to represent false, and 0x1 is used to represent true.
	BoolKey = codec.NewBoolKey[bool]()
)

// VALUES

var (
	// BoolValue implements a ValueCodec for bool.
	BoolValue = codec.KeyToValueCodec(BoolKey)
	// Uint16Value implements a ValueCodec for uint16.
	Uint16Value = codec.KeyToValueCodec(Uint16Key)
	// Uint32Value implements a ValueCodec for uint32.
	Uint32Value = codec.KeyToValueCodec(Uint32Key)
	// Uint64Value implements a ValueCodec for uint64.
	Uint64Value = codec.KeyToValueCodec(Uint64Key)
	// Int32Value implements a ValueCodec for int32.
	Int32Value = codec.KeyToValueCodec(Int32Key)
	// Int64Value implements a ValueCodec for int64.
	Int64Value = codec.KeyToValueCodec(Int64Key)
	// StringValue implements a ValueCodec for string.
	StringValue = codec.KeyToValueCodec(StringKey)
	// BytesValue implements a ValueCodec for bytes.
	BytesValue = codec.KeyToValueCodec(BytesKey)
)

// Collection is the interface that all collections implement. It will eventually
// include methods for importing/exporting genesis data and schema
// reflection for clients.
// NOTE: Unstable.
type Collection interface {
	// GetName is the unique name of the collection within a schema. It must
	// match format specified by NameRegex.
	GetName() string

	// GetPrefix is the unique prefix of the collection within a schema.
	GetPrefix() []byte

	// ValueCodec returns the codec used to encode/decode values of the collection.
	ValueCodec() codec.UntypedValueCodec

	genesisHandler
}

// Prefix defines a segregation bytes namespace for specific collections objects.
type Prefix []byte

// Bytes returns the raw Prefix bytes.
func (n Prefix) Bytes() []byte { return n }

// NewPrefix returns a Prefix given the provided namespace identifier.
// In the same module, no prefixes should share the same starting bytes
// meaning that having two namespaces whose bytes representation is:
// p1 := []byte("prefix")
// p2 := []byte("prefix1")
// yields to iterations of p1 overlapping over p2.
// If a numeric prefix is provided, it must be between 0 and 255 (uint8).
// If out of bounds this function will panic.
// Reason for which this function is constrained to `int` instead of `uint8` is for
// API ergonomics, golang's type inference will infer int properly but not uint8
// meaning that developers would need to write NewPrefix(uint8(number)) for numeric
// prefixes.
func NewPrefix[T interface{ int | string | []byte }](identifier T) Prefix {
	i := any(identifier)
	var prefix []byte
	switch c := i.(type) {
	case int:
		if c > math.MaxUint8 || c < 0 {
			panic("invalid integer prefix value: must be between 0 and 255")
		}
		prefix = []byte{uint8(c)}
	case string:
		prefix = []byte(c)
	case []byte:
		identifierCopy := make([]byte, len(c))
		copy(identifierCopy, c)
		prefix = identifierCopy
	}
	return prefix
}

var _ Collection = (*collectionImpl[string, string])(nil)

// collectionImpl wraps a Map and implements Collection. This properly splits
// the generic and untyped Collection interface from the typed Map, which every
// collection builds on.
type collectionImpl[K, V any] struct {
	m Map[K, V]
}

func (c collectionImpl[K, V]) ValueCodec() codec.UntypedValueCodec {
	return codec.NewUntypedValueCodec(c.m.vc)
}

func (c collectionImpl[K, V]) GetName() string { return c.m.name }

func (c collectionImpl[K, V]) GetPrefix() []byte { return NewPrefix(c.m.prefix) }

func (c collectionImpl[K, V]) validateGenesis(r io.Reader) error { return c.m.validateGenesis(r) }

func (c collectionImpl[K, V]) importGenesis(ctx context.Context, r io.Reader) error {
	return c.m.importGenesis(ctx, r)
}

func (c collectionImpl[K, V]) exportGenesis(ctx context.Context, w io.Writer) error {
	return c.m.exportGenesis(ctx, w)
}

func (c collectionImpl[K, V]) defaultGenesis(w io.Writer) error { return c.m.defaultGenesis(w) }
