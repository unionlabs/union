package types

import (
	"encoding/binary"
	fmt "fmt"

	proto "github.com/cosmos/gogoproto/proto"
)

// Codec defines a interface needed for the store package to marshal data
type Codec interface {
	// Marshal returns binary encoding of v.
	Marshal(proto.Message) ([]byte, error)

	// MarshalLengthPrefixed returns binary encoding of v with bytes length prefix.
	MarshalLengthPrefixed(proto.Message) ([]byte, error)

	// Unmarshal parses the data encoded with Marshal method and stores the result
	// in the value pointed to by v.
	Unmarshal(bz []byte, ptr proto.Message) error

	// UnmarshalLengthPrefixed parses the data encoded with UnmarshalLengthPrefixed method and stores
	// the result in the value pointed to by v.
	UnmarshalLengthPrefixed(bz []byte, ptr proto.Message) error
}

// ============= TestCodec =============

// TestCodec defines a codec that utilizes Protobuf for both binary and JSON
// encoding.
type TestCodec struct{}

var _ Codec = &TestCodec{}

func NewTestCodec() Codec {
	return &TestCodec{}
}

// Marshal implements BinaryMarshaler.Marshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.MarshalInterface
func (pc *TestCodec) Marshal(o proto.Message) ([]byte, error) {
	// Size() check can catch the typed nil value.
	if o == nil || proto.Size(o) == 0 {
		// return empty bytes instead of nil, because nil has special meaning in places like store.Set
		return []byte{}, nil
	}
	return proto.Marshal(o)
}

// MarshalLengthPrefixed implements BinaryMarshaler.MarshalLengthPrefixed method.
func (pc *TestCodec) MarshalLengthPrefixed(o proto.Message) ([]byte, error) {
	bz, err := pc.Marshal(o)
	if err != nil {
		return nil, err
	}

	var sizeBuf [binary.MaxVarintLen64]byte
	n := binary.PutUvarint(sizeBuf[:], uint64(len(bz)))
	return append(sizeBuf[:n], bz...), nil
}

// Unmarshal implements BinaryMarshaler.Unmarshal method.
// NOTE: this function must be used with a concrete type which
// implements proto.Message. For interface please use the codec.UnmarshalInterface
func (pc *TestCodec) Unmarshal(bz []byte, ptr proto.Message) error {
	err := proto.Unmarshal(bz, ptr)
	if err != nil {
		return err
	}

	return nil
}

// UnmarshalLengthPrefixed implements BinaryMarshaler.UnmarshalLengthPrefixed method.
func (pc *TestCodec) UnmarshalLengthPrefixed(bz []byte, ptr proto.Message) error {
	size, n := binary.Uvarint(bz)
	if n < 0 {
		return fmt.Errorf("invalid number of bytes read from length-prefixed encoding: %d", n)
	}

	if size > uint64(len(bz)-n) {
		return fmt.Errorf("not enough bytes to read; want: %v, got: %v", size, len(bz)-n)
	} else if size < uint64(len(bz)-n) {
		return fmt.Errorf("too many bytes to read; want: %v, got: %v", size, len(bz)-n)
	}

	bz = bz[n:]
	return proto.Unmarshal(bz, ptr)
}
