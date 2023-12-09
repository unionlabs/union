package codec

import (
	"encoding/json"
	"fmt"
	"math"
)

// MaxBytesKeyNonTerminalSize defines the maximum length of a bytes key encoded
// using the BytesKey KeyCodec.
const MaxBytesKeyNonTerminalSize = math.MaxUint8

func NewBytesKey[T ~[]byte]() KeyCodec[T] { return bytesKey[T]{} }

type bytesKey[T ~[]byte] struct{}

func (b bytesKey[T]) Encode(buffer []byte, key T) (int, error) {
	return copy(buffer, key), nil
}

func (bytesKey[T]) Decode(buffer []byte) (int, T, error) {
	// todo: should we copy it? collections will just discard the buffer, so from coll POV is not needed.
	return len(buffer), buffer, nil
}

func (bytesKey[T]) Size(key T) int {
	return len(key)
}

func (bytesKey[T]) EncodeJSON(value T) ([]byte, error) {
	return json.Marshal(value)
}

func (bytesKey[T]) DecodeJSON(b []byte) (T, error) {
	var t T
	err := json.Unmarshal(b, &t)
	return t, err
}

func (b bytesKey[T]) Stringify(key T) string {
	return fmt.Sprintf("hexBytes:%x", key)
}

func (b bytesKey[T]) KeyType() string {
	return "bytes"
}

func (b bytesKey[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	if len(key) > MaxBytesKeyNonTerminalSize {
		return 0, fmt.Errorf(
			"%w: bytes key non terminal size cannot exceed: %d, got: %d",
			ErrEncoding, MaxBytesKeyNonTerminalSize, len(key),
		)
	}

	buffer[0] = uint8(len(key))
	written := copy(buffer[1:], key)
	return written + 1, nil
}

func (bytesKey[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	l := len(buffer)
	if l == 0 {
		return 0, nil, fmt.Errorf("%w: bytes key non terminal decoding cannot have an empty buffer", ErrEncoding)
	}

	keyLength := int(buffer[0])
	if len(buffer[1:]) < keyLength {
		return 0, nil, fmt.Errorf(
			"%w: bytes key non terminal decoding isn't big enough, want at least: %d, got: %d",
			ErrEncoding, keyLength, len(buffer[1:]),
		)
	}
	return 1 + keyLength, buffer[1 : keyLength+1], nil
}

func (bytesKey[T]) SizeNonTerminal(key T) int {
	return len(key) + 1
}
