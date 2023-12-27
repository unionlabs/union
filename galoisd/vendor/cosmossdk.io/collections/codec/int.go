package codec

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"strconv"
)

func NewInt64Key[T ~int64]() KeyCodec[T] { return int64Key[T]{} }

type int64Key[T ~int64] struct{}

func (i int64Key[T]) Encode(buffer []byte, key T) (int, error) {
	binary.BigEndian.PutUint64(buffer, (uint64)(key))
	buffer[0] ^= 0x80
	return 8, nil
}

func (i int64Key[T]) Decode(buffer []byte) (int, T, error) {
	if len(buffer) < 8 {
		return 0, 0, fmt.Errorf("%w: invalid buffer size, wanted: 8", ErrEncoding)
	}
	u := uint64(buffer[7]) | uint64(buffer[6])<<8 | uint64(buffer[5])<<16 | uint64(buffer[4])<<24 |
		uint64(buffer[3])<<32 | uint64(buffer[2])<<40 | uint64(buffer[1])<<48 | uint64(buffer[0]^0x80)<<56

	return 8, (T)(u), nil
}

func (i int64Key[T]) Size(_ T) int { return 8 }

func (i int64Key[T]) EncodeJSON(value T) ([]byte, error) {
	return []byte(`"` + strconv.FormatInt((int64)(value), 10) + `"`), nil
}

func (i int64Key[T]) DecodeJSON(b []byte) (T, error) {
	var s string
	err := json.Unmarshal(b, &s)
	if err != nil {
		return 0, err
	}
	k, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		return 0, err
	}
	return (T)(k), nil
}

func (i int64Key[T]) Stringify(key T) string { return strconv.FormatInt((int64)(key), 10) }

func (i int64Key[T]) KeyType() string {
	return "int64"
}

func (i int64Key[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return i.Encode(buffer, key)
}

func (i int64Key[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	return i.Decode(buffer)
}

func (i int64Key[T]) SizeNonTerminal(_ T) int {
	return 8
}

func NewInt32Key[T ~int32]() KeyCodec[T] {
	return int32Key[T]{}
}

type int32Key[T ~int32] struct{}

func (i int32Key[T]) Encode(buffer []byte, key T) (int, error) {
	binary.BigEndian.PutUint32(buffer, (uint32)(key))
	buffer[0] ^= 0x80
	return 4, nil
}

func (i int32Key[T]) Decode(buffer []byte) (int, T, error) {
	if len(buffer) < 4 {
		return 0, 0, fmt.Errorf("%w: invalid buffer size, wanted: 4", ErrEncoding)
	}
	u := uint32(buffer[3]) | uint32(buffer[2])<<8 | uint32(buffer[1])<<16 | uint32(buffer[0]^0x80)<<24

	return 4, (T)(u), nil
}

func (i int32Key[T]) Size(_ T) int { return 4 }

func (i int32Key[T]) EncodeJSON(value T) ([]byte, error) {
	return []byte(`"` + strconv.FormatInt((int64)(value), 10) + `"`), nil
}

func (i int32Key[T]) DecodeJSON(b []byte) (T, error) {
	var s string
	err := json.Unmarshal(b, &s)
	if err != nil {
		return 0, err
	}
	k, err := strconv.ParseInt(s, 10, 32)
	if err != nil {
		return 0, err
	}
	return (T)(k), nil
}

func (i int32Key[T]) Stringify(key T) string { return strconv.FormatInt((int64)(key), 10) }

func (i int32Key[T]) KeyType() string {
	return "int32"
}

func (i int32Key[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return i.Encode(buffer, key)
}

func (i int32Key[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	return i.Decode(buffer)
}

func (i int32Key[T]) SizeNonTerminal(_ T) int {
	return 4
}
