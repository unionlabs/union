package codec

import (
	"encoding/binary"
	"encoding/json"
	"fmt"
	"strconv"
)

func NewUint64Key[T ~uint64]() KeyCodec[T] { return uint64Key[T]{} }

type uint64Key[T ~uint64] struct{}

func (uint64Key[T]) Encode(buffer []byte, key T) (int, error) {
	binary.BigEndian.PutUint64(buffer, (uint64)(key))
	return 8, nil
}

func (uint64Key[T]) Decode(buffer []byte) (int, T, error) {
	if size := len(buffer); size < 8 {
		return 0, 0, fmt.Errorf("%w: wanted at least 8, got: %d", ErrEncoding, size)
	}
	return 8, (T)(binary.BigEndian.Uint64(buffer)), nil
}

func (uint64Key[T]) EncodeJSON(value T) ([]byte, error) { return uintEncodeJSON((uint64)(value)) }

func (uint64Key[T]) DecodeJSON(b []byte) (T, error) {
	u, err := uintDecodeJSON(b, 64)
	if err != nil {
		return 0, err
	}
	return (T)(u), nil
}

func (uint64Key[T]) Size(_ T) int { return 8 }

func (u uint64Key[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return u.Encode(buffer, key)
}

func (u uint64Key[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	return u.Decode(buffer)
}

func (u uint64Key[T]) SizeNonTerminal(key T) int {
	return u.Size(key)
}

func (uint64Key[T]) Stringify(key T) string {
	return strconv.FormatUint((uint64)(key), 10)
}

func (uint64Key[T]) KeyType() string {
	return "uint64"
}

func NewUint32Key[T ~uint32]() KeyCodec[T] { return uint32Key[T]{} }

type uint32Key[T ~uint32] struct{}

func (uint32Key[T]) Encode(buffer []byte, key T) (int, error) {
	binary.BigEndian.PutUint32(buffer, (uint32)(key))
	return 4, nil
}

func (uint32Key[T]) Decode(buffer []byte) (int, T, error) {
	if len(buffer) < 4 {
		return 0, 0, fmt.Errorf("%w: expected buffer of size 4", ErrEncoding)
	}
	return 4, (T)(binary.BigEndian.Uint32(buffer)), nil
}

func (uint32Key[T]) Size(_ T) int { return 4 }

func (uint32Key[T]) EncodeJSON(value T) ([]byte, error) { return uintEncodeJSON((uint64)(value)) }

func (uint32Key[T]) DecodeJSON(b []byte) (T, error) {
	u, err := uintDecodeJSON(b, 32)
	if err != nil {
		return 0, err
	}
	return (T)(u), nil
}

func (uint32Key[T]) Stringify(key T) string { return strconv.FormatUint(uint64(key), 10) }

func (uint32Key[T]) KeyType() string { return "uint32" }

func (u uint32Key[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return u.Encode(buffer, key)
}

func (u uint32Key[T]) DecodeNonTerminal(buffer []byte) (int, T, error) { return u.Decode(buffer) }

func (uint32Key[T]) SizeNonTerminal(_ T) int { return 4 }

func NewUint16Key[T ~uint16]() KeyCodec[T] { return uint16Key[T]{} }

type uint16Key[T ~uint16] struct{}

func (uint16Key[T]) Encode(buffer []byte, key T) (int, error) {
	binary.BigEndian.PutUint16(buffer, (uint16)(key))
	return 2, nil
}

func (uint16Key[T]) Decode(buffer []byte) (int, T, error) {
	if len(buffer) < 2 {
		return 0, 0, fmt.Errorf("%w: invalid buffer size, wanted at least 2", ErrEncoding)
	}
	return 2, (T)(binary.BigEndian.Uint16(buffer)), nil
}

func (uint16Key[T]) Size(key T) int { return 2 }

func (uint16Key[T]) EncodeJSON(value T) ([]byte, error) { return uintEncodeJSON((uint64)(value)) }

func (uint16Key[T]) DecodeJSON(b []byte) (T, error) {
	u, err := uintDecodeJSON(b, 16)
	if err != nil {
		return 0, err
	}
	return (T)(u), nil
}

func (uint16Key[T]) Stringify(key T) string { return strconv.FormatUint((uint64)(key), 10) }

func (uint16Key[T]) KeyType() string { return "uint16" }

func (u uint16Key[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return u.Encode(buffer, key)
}

func (u uint16Key[T]) DecodeNonTerminal(buffer []byte) (int, T, error) { return u.Decode(buffer) }

func (u uint16Key[T]) SizeNonTerminal(key T) int { return u.Size(key) }

func uintEncodeJSON(value uint64) ([]byte, error) {
	str := `"` + strconv.FormatUint(value, 10) + `"`
	return []byte(str), nil
}

func uintDecodeJSON(b []byte, bitSize int) (uint64, error) {
	var str string
	err := json.Unmarshal(b, &str)
	if err != nil {
		return 0, err
	}
	return strconv.ParseUint(str, 10, bitSize)
}
