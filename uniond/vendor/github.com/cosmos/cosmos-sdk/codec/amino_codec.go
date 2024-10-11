package codec

import (
	"github.com/cosmos/gogoproto/proto"
)

// Deprecated: AminoCodec defines a codec that utilizes Codec for both binary and JSON
// encoding. Any usage of amino should be done using the LegacyAmino type directly.
// Usage of amino with the Codec type is not well-supported and may be removed in the future.
type AminoCodec struct {
	*LegacyAmino
}

var (
	_ BinaryCodec = &AminoCodec{}
	_ JSONCodec   = &AminoCodec{}
)

// Deprecated: NewAminoCodec returns a reference to a new AminoCodec.
// Use NewLegacyAmino instead.
func NewAminoCodec(codec *LegacyAmino) *AminoCodec {
	return &AminoCodec{LegacyAmino: codec}
}

// Marshal implements BinaryMarshaler.Marshal method.
func (ac *AminoCodec) Marshal(o proto.Message) ([]byte, error) {
	return ac.LegacyAmino.Marshal(o)
}

// MustMarshal implements BinaryMarshaler.MustMarshal method.
func (ac *AminoCodec) MustMarshal(o proto.Message) []byte {
	return ac.LegacyAmino.MustMarshal(o)
}

// MarshalLengthPrefixed implements BinaryMarshaler.MarshalLengthPrefixed method.
func (ac *AminoCodec) MarshalLengthPrefixed(o proto.Message) ([]byte, error) {
	return ac.LegacyAmino.MarshalLengthPrefixed(o)
}

// MustMarshalLengthPrefixed implements BinaryMarshaler.MustMarshalLengthPrefixed method.
func (ac *AminoCodec) MustMarshalLengthPrefixed(o proto.Message) []byte {
	return ac.LegacyAmino.MustMarshalLengthPrefixed(o)
}

// Unmarshal implements BinaryMarshaler.Unmarshal method.
func (ac *AminoCodec) Unmarshal(bz []byte, ptr proto.Message) error {
	return ac.LegacyAmino.Unmarshal(bz, ptr)
}

// MustUnmarshal implements BinaryMarshaler.MustUnmarshal method.
func (ac *AminoCodec) MustUnmarshal(bz []byte, ptr proto.Message) {
	ac.LegacyAmino.MustUnmarshal(bz, ptr)
}

// UnmarshalLengthPrefixed implements BinaryMarshaler.UnmarshalLengthPrefixed method.
func (ac *AminoCodec) UnmarshalLengthPrefixed(bz []byte, ptr proto.Message) error {
	return ac.LegacyAmino.UnmarshalLengthPrefixed(bz, ptr)
}

// MustUnmarshalLengthPrefixed implements BinaryMarshaler.MustUnmarshalLengthPrefixed method.
func (ac *AminoCodec) MustUnmarshalLengthPrefixed(bz []byte, ptr proto.Message) {
	ac.LegacyAmino.MustUnmarshalLengthPrefixed(bz, ptr)
}

// MarshalJSON implements JSONCodec.MarshalJSON method,
// it marshals to JSON using legacy amino codec.
func (ac *AminoCodec) MarshalJSON(o proto.Message) ([]byte, error) {
	return ac.LegacyAmino.MarshalJSON(o)
}

// MustMarshalJSON implements JSONCodec.MustMarshalJSON method,
// it executes MarshalJSON except it panics upon failure.
func (ac *AminoCodec) MustMarshalJSON(o proto.Message) []byte {
	return ac.LegacyAmino.MustMarshalJSON(o)
}

// UnmarshalJSON implements JSONCodec.UnmarshalJSON method,
// it unmarshals from JSON using legacy amino codec.
func (ac *AminoCodec) UnmarshalJSON(bz []byte, ptr proto.Message) error {
	return ac.LegacyAmino.UnmarshalJSON(bz, ptr)
}

// MustUnmarshalJSON implements JSONCodec.MustUnmarshalJSON method,
// it executes UnmarshalJSON except it panics upon failure.
func (ac *AminoCodec) MustUnmarshalJSON(bz []byte, ptr proto.Message) {
	ac.LegacyAmino.MustUnmarshalJSON(bz, ptr)
}

// MarshalInterface is a convenience function for amino marshaling interfaces.
// The `i` must be an interface.
// NOTE: to marshal a concrete type, you should use Marshal instead
func (ac *AminoCodec) MarshalInterface(i proto.Message) ([]byte, error) {
	if err := assertNotNil(i); err != nil {
		return nil, err
	}
	return ac.LegacyAmino.Marshal(i)
}

// UnmarshalInterface is a convenience function for amino unmarshaling interfaces.
// `ptr` must be a pointer to an interface.
// NOTE: to unmarshal a concrete type, you should use Unmarshal instead
//
// Example:
//
//	var x MyInterface
//	err := cdc.UnmarshalInterface(bz, &x)
func (ac *AminoCodec) UnmarshalInterface(bz []byte, ptr interface{}) error {
	return ac.LegacyAmino.Unmarshal(bz, ptr)
}

// MarshalInterfaceJSON is a convenience function for amino marshaling interfaces.
// The `i` must be an interface.
// NOTE: to marshal a concrete type, you should use MarshalJSON instead
func (ac *AminoCodec) MarshalInterfaceJSON(i proto.Message) ([]byte, error) {
	if err := assertNotNil(i); err != nil {
		return nil, err
	}
	return ac.LegacyAmino.MarshalJSON(i)
}

// UnmarshalInterfaceJSON is a convenience function for amino unmarshaling interfaces.
// `ptr` must be a pointer to an interface.
// NOTE: to unmarshal a concrete type, you should use UnmarshalJSON instead
//
// Example:
//
//	var x MyInterface
//	err := cdc.UnmarshalInterfaceJSON(bz, &x)
func (ac *AminoCodec) UnmarshalInterfaceJSON(bz []byte, ptr interface{}) error {
	return ac.LegacyAmino.UnmarshalJSON(bz, ptr)
}
