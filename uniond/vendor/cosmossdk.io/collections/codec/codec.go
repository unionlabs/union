package codec

import (
	"errors"
	"fmt"
)

var ErrEncoding = errors.New("collections: encoding error")

// KeyCodec defines a generic interface which is implemented
// by types that are capable of encoding and decoding collections keys.
type KeyCodec[T any] interface {
	// Encode writes the key bytes into the buffer. Returns the number of
	// bytes written. The implementer must expect the buffer to be at least
	// of length equal to Size(K) for all encodings.
	// It must also return the number of written bytes which must be
	// equal to Size(K) for all encodings not involving varints.
	// In case of encodings involving varints then the returned
	// number of written bytes is allowed to be smaller than Size(K).
	Encode(buffer []byte, key T) (int, error)
	// Decode reads from the provided bytes buffer to decode
	// the key T. Returns the number of bytes read, the type T
	// or an error in case of decoding failure.
	Decode(buffer []byte) (int, T, error)
	// Size returns the buffer size need to encode key T in binary format.
	// The returned value must match what is computed by Encode for all
	// encodings except the ones involving varints. Varints are expected
	// to return the maximum varint bytes buffer length, at the risk of
	// over-estimating in order to pick the most performant path.
	Size(key T) int
	// EncodeJSON encodes the value as JSON.
	EncodeJSON(value T) ([]byte, error)
	// DecodeJSON decodes the provided JSON bytes into an instance of T.
	DecodeJSON(b []byte) (T, error)
	// Stringify returns a string representation of T.
	Stringify(key T) string
	// KeyType returns a string identifier for the type of the key.
	KeyType() string

	// MULTIPART keys

	// EncodeNonTerminal writes the key bytes into the buffer.
	// EncodeNonTerminal is used in multipart keys like Pair
	// when the part of the key being encoded is not the last one,
	// and there needs to be a way to distinguish after how many bytes
	// the first part of the key is finished. The buffer is expected to be
	// at least as big as SizeNonTerminal(key) returns. It returns
	// the amount of bytes written.
	EncodeNonTerminal(buffer []byte, key T) (int, error)
	// DecodeNonTerminal reads the buffer provided and returns
	// the key T. DecodeNonTerminal is used in multipart keys
	// like Pair when the part of the key being decoded is not the
	// last one. It returns the amount of bytes read.
	DecodeNonTerminal(buffer []byte) (int, T, error)
	// SizeNonTerminal returns the maximum size of the key K when used in
	// multipart keys like Pair.
	SizeNonTerminal(key T) int
}

// ValueCodec defines a generic interface which is implemented
// by types that are capable of encoding and decoding collection values.
type ValueCodec[T any] interface {
	// Encode encodes the value T into binary format.
	Encode(value T) ([]byte, error)
	// Decode returns the type T given its binary representation.
	Decode(b []byte) (T, error)
	// EncodeJSON encodes the value as JSON.
	EncodeJSON(value T) ([]byte, error)
	// DecodeJSON decodes the provided JSON bytes into an instance of T.
	DecodeJSON(b []byte) (T, error)
	// Stringify returns a string representation of T.
	Stringify(value T) string
	// ValueType returns the identifier for the type.
	ValueType() string
}

// NewUntypedValueCodec returns an UntypedValueCodec for the provided ValueCodec.
func NewUntypedValueCodec[V any](v ValueCodec[V]) UntypedValueCodec {
	typeName := fmt.Sprintf("%T", *new(V))
	checkType := func(value interface{}) (v V, err error) {
		concrete, ok := value.(V)
		if !ok {
			return v, fmt.Errorf("%w: expected value of type %s, got %T", ErrEncoding, typeName, value)
		}
		return concrete, nil
	}
	return UntypedValueCodec{
		Decode: func(b []byte) (interface{}, error) { return v.Decode(b) },
		Encode: func(value interface{}) ([]byte, error) {
			concrete, err := checkType(value)
			if err != nil {
				return nil, err
			}
			return v.Encode(concrete)
		},
		DecodeJSON: func(b []byte) (interface{}, error) {
			return v.DecodeJSON(b)
		},
		EncodeJSON: func(value interface{}) ([]byte, error) {
			concrete, err := checkType(value)
			if err != nil {
				return nil, err
			}
			return v.EncodeJSON(concrete)
		},
		Stringify: func(value interface{}) (string, error) {
			concrete, err := checkType(value)
			if err != nil {
				return "", err
			}
			return v.Stringify(concrete), nil
		},
		ValueType: func() string { return v.ValueType() },
	}
}

// UntypedValueCodec wraps a ValueCodec to expose an untyped API for encoding and decoding values.
type UntypedValueCodec struct {
	Decode     func(b []byte) (interface{}, error)
	Encode     func(value interface{}) ([]byte, error)
	DecodeJSON func(b []byte) (interface{}, error)
	EncodeJSON func(value interface{}) ([]byte, error)
	Stringify  func(value interface{}) (string, error)
	ValueType  func() string
}

// KeyToValueCodec converts a KeyCodec into a ValueCodec.
func KeyToValueCodec[K any](keyCodec KeyCodec[K]) ValueCodec[K] { return keyToValueCodec[K]{keyCodec} }

// keyToValueCodec is a ValueCodec that wraps a KeyCodec to make it behave like a ValueCodec.
type keyToValueCodec[K any] struct {
	kc KeyCodec[K]
}

func (k keyToValueCodec[K]) EncodeJSON(value K) ([]byte, error) {
	return k.kc.EncodeJSON(value)
}

func (k keyToValueCodec[K]) DecodeJSON(b []byte) (K, error) {
	return k.kc.DecodeJSON(b)
}

func (k keyToValueCodec[K]) Encode(value K) ([]byte, error) {
	buf := make([]byte, k.kc.Size(value))
	_, err := k.kc.Encode(buf, value)
	return buf, err
}

func (k keyToValueCodec[K]) Decode(b []byte) (K, error) {
	r, key, err := k.kc.Decode(b)
	if err != nil {
		var key K
		return key, err
	}

	if r != len(b) {
		var key K
		return key, fmt.Errorf("%w: was supposed to fully consume the key '%x', consumed %d out of %d", ErrEncoding, b, r, len(b))
	}
	return key, nil
}

func (k keyToValueCodec[K]) Stringify(value K) string {
	return k.kc.Stringify(value)
}

func (k keyToValueCodec[K]) ValueType() string {
	return k.kc.KeyType()
}
