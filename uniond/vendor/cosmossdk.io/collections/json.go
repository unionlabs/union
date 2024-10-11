package collections

import (
	"encoding/json"
	"fmt"

	"cosmossdk.io/collections/codec"
)

func NewJSONValueCodec[T any]() codec.ValueCodec[T] {
	return jsonValue[T]{
		typeName: fmt.Sprintf("%T", new(T)),
	}
}

type jsonValue[T any] struct {
	typeName string
}

// Decode implements codec.ValueCodec.
func (jsonValue[T]) Decode(b []byte) (T, error) {
	var t T
	if err := json.Unmarshal(b, &t); err != nil {
		return t, err
	}

	return t, nil
}

// DecodeJSON implements codec.ValueCodec.
func (jsonValue[T]) DecodeJSON(b []byte) (T, error) {
	var t T
	if err := json.Unmarshal(b, &t); err != nil {
		return t, err
	}

	return t, nil
}

// Encode implements codec.ValueCodec.
func (jsonValue[T]) Encode(value T) ([]byte, error) {
	return json.Marshal(value)
}

// EncodeJSON implements codec.ValueCodec.
func (jsonValue[T]) EncodeJSON(value T) ([]byte, error) {
	return json.Marshal(value)
}

// Stringify implements codec.ValueCodec.
func (jsonValue[T]) Stringify(value T) string {
	return fmt.Sprintf("%v", value)
}

// ValueType implements codec.ValueCodec.
func (jv jsonValue[T]) ValueType() string {
	return fmt.Sprintf("json(%s)", jv.typeName)
}
