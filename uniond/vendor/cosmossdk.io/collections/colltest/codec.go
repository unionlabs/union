package colltest

import (
	"encoding/json"
	"fmt"
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"

	"cosmossdk.io/collections"
	"cosmossdk.io/collections/codec"
)

// TestKeyCodec asserts the correct behavior of a KeyCodec over the type T.
func TestKeyCodec[T any](t *testing.T, keyCodec codec.KeyCodec[T], key T) {
	buffer := make([]byte, keyCodec.Size(key))
	written, err := keyCodec.Encode(buffer, key)
	require.NoError(t, err)
	require.Equal(t, len(buffer), written, "the length of the buffer and the written bytes do not match")
	read, decodedKey, err := keyCodec.Decode(buffer)
	require.NoError(t, err)
	require.Equal(t, len(buffer), read, "encoded key and read bytes must have same size")
	require.Equal(t, key, decodedKey, "encoding and decoding produces different keys")
	// test if terminality is correctly applied
	pairCodec := collections.PairKeyCodec(keyCodec, collections.StringKey)
	pairKey := collections.Join(key, "TEST")
	buffer = make([]byte, pairCodec.Size(pairKey))
	written, err = pairCodec.Encode(buffer, pairKey)
	require.Equal(t, len(buffer), written, "the pair buffer should have been fully written")
	require.NoError(t, err)
	read, decodedPairKey, err := pairCodec.Decode(buffer)
	require.NoError(t, err)
	require.Equal(t, len(buffer), read, "encoded non terminal key and pair key read bytes must have same size")
	require.Equal(t, pairKey, decodedPairKey, "encoding and decoding produces different keys with non terminal encoding")

	// check JSON
	keyJSON, err := keyCodec.EncodeJSON(key)
	require.NoError(t, err)
	decoded, err := keyCodec.DecodeJSON(keyJSON)
	require.NoError(t, err)
	require.Equal(t, key, decoded, "json encoding and decoding did not produce the same results")

	// check type
	require.NotEmpty(t, keyCodec.KeyType())
	// check string
	_ = keyCodec.Stringify(key)
}

// TestValueCodec asserts the correct behavior of a ValueCodec over the type T.
func TestValueCodec[T any](t *testing.T, encoder codec.ValueCodec[T], value T) {
	encodedValue, err := encoder.Encode(value)
	require.NoError(t, err)
	decodedValue, err := encoder.Decode(encodedValue)
	require.NoError(t, err)
	require.Equal(t, value, decodedValue, "encoding and decoding produces different values")

	encodedJSONValue, err := encoder.EncodeJSON(value)
	require.NoError(t, err)
	decodedJSONValue, err := encoder.DecodeJSON(encodedJSONValue)
	require.NoError(t, err)
	require.Equal(t, value, decodedJSONValue, "encoding and decoding in json format produces different values")

	require.NotEmpty(t, encoder.ValueType())

	_ = encoder.Stringify(value)
}

// MockValueCodec returns a mock of collections.ValueCodec for type T, it
// can be used for collections Values testing. It also supports interfaces.
// For the interfaces cases, in order for an interface to be decoded it must
// have been encoded first. Not concurrency safe.
// EG:
// Let's say the value is interface Animal
// if I want to decode Dog which implements Animal, then I need to first encode
// it in order to make the type known by the MockValueCodec.
func MockValueCodec[T any]() codec.ValueCodec[T] {
	typ := reflect.ValueOf(new(T)).Elem().Type()
	isInterface := false
	if typ.Kind() == reflect.Interface {
		isInterface = true
	}
	return &mockValueCodec[T]{
		isInterface: isInterface,
		seenTypes:   map[string]reflect.Type{},
		valueType:   fmt.Sprintf("%s.%s", typ.PkgPath(), typ.Name()),
	}
}

type mockValueJSON struct {
	TypeName string          `json:"type_name"`
	Value    json.RawMessage `json:"value"`
}

type mockValueCodec[T any] struct {
	isInterface bool
	seenTypes   map[string]reflect.Type
	valueType   string
}

func (m mockValueCodec[T]) Encode(value T) ([]byte, error) {
	typeName := m.getTypeName(value)
	valueBytes, err := json.Marshal(value)
	if err != nil {
		return nil, err
	}

	return json.Marshal(mockValueJSON{
		TypeName: typeName,
		Value:    valueBytes,
	})
}

func (m mockValueCodec[T]) Decode(b []byte) (t T, err error) {
	wrappedValue := mockValueJSON{}
	err = json.Unmarshal(b, &wrappedValue)
	if err != nil {
		return
	}
	if !m.isInterface {
		err = json.Unmarshal(wrappedValue.Value, &t)
		return t, err
	}

	typ, exists := m.seenTypes[wrappedValue.TypeName]
	if !exists {
		return t, fmt.Errorf("unknown type %s, you're dealing with interfaces... in order to make the interface types known for the MockValueCodec, you need to first encode them", wrappedValue.TypeName)
	}

	newT := reflect.New(typ).Interface()
	err = json.Unmarshal(wrappedValue.Value, newT)
	if err != nil {
		return t, err
	}

	iface := new(T)
	reflect.ValueOf(iface).Elem().Set(reflect.ValueOf(newT).Elem())
	return *iface, nil
}

func (m mockValueCodec[T]) EncodeJSON(value T) ([]byte, error) {
	return m.Encode(value)
}

func (m mockValueCodec[T]) DecodeJSON(b []byte) (T, error) {
	return m.Decode(b)
}

func (m mockValueCodec[T]) Stringify(value T) string {
	return fmt.Sprintf("%#v", value)
}

func (m mockValueCodec[T]) ValueType() string {
	return m.valueType
}

func (m mockValueCodec[T]) getTypeName(value T) string {
	if !m.isInterface {
		return m.valueType
	}
	typ := reflect.TypeOf(value)
	name := fmt.Sprintf("%s.%s", typ.PkgPath(), typ.Name())
	m.seenTypes[name] = typ
	return name
}
