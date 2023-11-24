package codec

import (
	"bytes"
	"encoding/json"
	"fmt"
)

func NewStringKeyCodec[T ~string]() KeyCodec[T] { return stringKey[T]{} }

const (
	// StringDelimiter defines the delimiter of a string key when used in non-terminal encodings.
	StringDelimiter uint8 = 0x0
)

type stringKey[T ~string] struct{}

func (stringKey[T]) Encode(buffer []byte, key T) (int, error) {
	return copy(buffer, key), nil
}

func (stringKey[T]) Decode(buffer []byte) (int, T, error) {
	return len(buffer), T(buffer), nil
}

func (stringKey[T]) EncodeJSON(value T) ([]byte, error) {
	return json.Marshal(value)
}

func (stringKey[T]) DecodeJSON(b []byte) (T, error) {
	var value T
	err := json.Unmarshal(b, &value)
	return value, err
}

func (stringKey[T]) Size(key T) int {
	return len(key)
}

func (stringKey[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	for i := range key {
		c := key[i]
		if c == StringDelimiter {
			return 0, fmt.Errorf("%w: string is not allowed to have the string delimiter (%c) in non terminal encodings of strings", ErrEncoding, StringDelimiter)
		}
		buffer[i] = c
	}

	return len(key) + 1, nil
}

func (stringKey[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	i := bytes.IndexByte(buffer, StringDelimiter)
	if i == -1 {
		return 0, "", fmt.Errorf("%w: not a valid non terminal buffer, no instances of the string delimiter %c found", ErrEncoding, StringDelimiter)
	}
	return i + 1, T(buffer[:i]), nil
}

func (stringKey[T]) SizeNonTerminal(key T) int { return len(key) + 1 }

func (stringKey[T]) Stringify(key T) string {
	return (string)(key)
}

func (stringKey[T]) KeyType() string {
	return "string"
}
