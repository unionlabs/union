package codec

import (
	"encoding/json"
	"fmt"
	"strconv"
)

func NewBoolKey[T ~bool]() KeyCodec[T] { return boolKey[T]{} }

type boolKey[T ~bool] struct{}

func (b boolKey[T]) Encode(buffer []byte, key T) (int, error) {
	if key {
		buffer[0] = 0x1
		return 1, nil
	}
	buffer[0] = 0x0
	return 1, nil
}

func (b boolKey[T]) Decode(buffer []byte) (int, T, error) {
	if len(buffer) == 0 {
		return 0, false, fmt.Errorf("%w: wanted size to be at least 1", ErrEncoding)
	}
	switch buffer[0] {
	case 0:
		return 1, false, nil
	case 1:
		return 1, true, nil
	default:
		return 0, false, fmt.Errorf("%w: invalid bool value: %d", ErrEncoding, buffer[0])
	}
}

func (b boolKey[T]) Size(_ T) int { return 1 }

func (b boolKey[T]) EncodeJSON(value T) ([]byte, error) {
	return json.Marshal(value)
}

func (b boolKey[T]) DecodeJSON(buffer []byte) (T, error) {
	var t T
	err := json.Unmarshal(buffer, &t)
	return t, err
}

func (b boolKey[T]) Stringify(key T) string {
	return strconv.FormatBool((bool)(key))
}

func (b boolKey[T]) KeyType() string {
	return "bool"
}

func (b boolKey[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return b.Encode(buffer, key)
}

func (b boolKey[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	return b.Decode(buffer)
}

func (b boolKey[T]) SizeNonTerminal(key T) int {
	return b.Size(key)
}
