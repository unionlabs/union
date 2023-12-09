package types

import (
	"fmt"
	"time"

	"cosmossdk.io/collections"
	collcodec "cosmossdk.io/collections/codec"
	"cosmossdk.io/math"
)

var (
	// AccAddressKey follows the same semantics of collections.BytesKey.
	// It just uses humanized format for the String() and EncodeJSON().
	AccAddressKey collcodec.KeyCodec[AccAddress] = genericAddressKey[AccAddress]{
		stringDecoder: AccAddressFromBech32,
		keyType:       "sdk.AccAddress",
	}

	// ValAddressKey follows the same semantics as AccAddressKey.
	ValAddressKey collcodec.KeyCodec[ValAddress] = genericAddressKey[ValAddress]{
		stringDecoder: ValAddressFromBech32,
		keyType:       "sdk.ValAddress",
	}

	// ConsAddressKey follows the same semantics as ConsAddressKey.
	ConsAddressKey collcodec.KeyCodec[ConsAddress] = genericAddressKey[ConsAddress]{
		stringDecoder: ConsAddressFromBech32,
		keyType:       "sdk.ConsAddress",
	}

	// IntValue represents a collections.ValueCodec to work with Int.
	IntValue collcodec.ValueCodec[math.Int] = intValueCodec{}

	// TimeKey represents a collections.KeyCodec to work with time.Time
	// Deprecated: exists only for state compatibility reasons, should not
	// be used for new storage keys using time. Please use the time KeyCodec
	// provided in the collections package.
	TimeKey collcodec.KeyCodec[time.Time] = timeKeyCodec{}
)

type addressUnion interface {
	AccAddress | ValAddress | ConsAddress
	String() string
}

type genericAddressKey[T addressUnion] struct {
	stringDecoder func(string) (T, error)
	keyType       string
}

func (a genericAddressKey[T]) Encode(buffer []byte, key T) (int, error) {
	return collections.BytesKey.Encode(buffer, key)
}

func (a genericAddressKey[T]) Decode(buffer []byte) (int, T, error) {
	return collections.BytesKey.Decode(buffer)
}

func (a genericAddressKey[T]) Size(key T) int {
	return collections.BytesKey.Size(key)
}

func (a genericAddressKey[T]) EncodeJSON(value T) ([]byte, error) {
	return collections.StringKey.EncodeJSON(value.String())
}

func (a genericAddressKey[T]) DecodeJSON(b []byte) (v T, err error) {
	s, err := collections.StringKey.DecodeJSON(b)
	if err != nil {
		return
	}
	v, err = a.stringDecoder(s)
	return
}

func (a genericAddressKey[T]) Stringify(key T) string {
	return key.String()
}

func (a genericAddressKey[T]) KeyType() string {
	return a.keyType
}

func (a genericAddressKey[T]) EncodeNonTerminal(buffer []byte, key T) (int, error) {
	return collections.BytesKey.EncodeNonTerminal(buffer, key)
}

func (a genericAddressKey[T]) DecodeNonTerminal(buffer []byte) (int, T, error) {
	return collections.BytesKey.DecodeNonTerminal(buffer)
}

func (a genericAddressKey[T]) SizeNonTerminal(key T) int {
	return collections.BytesKey.SizeNonTerminal(key)
}

// Deprecated: lengthPrefixedAddressKey is a special key codec used to retain state backwards compatibility
// when a generic address key (be: AccAddress, ValAddress, ConsAddress), is used as an index key.
// More docs can be found in the LengthPrefixedAddressKey function.
type lengthPrefixedAddressKey[T addressUnion] struct {
	collcodec.KeyCodec[T]
}

func (g lengthPrefixedAddressKey[T]) Encode(buffer []byte, key T) (int, error) {
	return g.EncodeNonTerminal(buffer, key)
}

func (g lengthPrefixedAddressKey[T]) Decode(buffer []byte) (int, T, error) {
	return g.DecodeNonTerminal(buffer)
}

func (g lengthPrefixedAddressKey[T]) Size(key T) int { return g.SizeNonTerminal(key) }

func (g lengthPrefixedAddressKey[T]) KeyType() string { return "index_key/" + g.KeyCodec.KeyType() }

// Deprecated: LengthPrefixedAddressKey implements an SDK backwards compatible indexing key encoder
// for addresses.
// The status quo in the SDK is that address keys are length prefixed even when they're the
// last part of a composite key. This should never be used unless to retain state compatibility.
// For example, a composite key composed of `[string, address]` in theory would need you only to
// define a way to understand when the string part finishes, we usually do this by appending a null
// byte to the string, then when you know when the string part finishes, it's logical that the
// part which remains is the address key. In the SDK instead we prepend to the address key its
// length too.
func LengthPrefixedAddressKey[T addressUnion](keyCodec collcodec.KeyCodec[T]) collcodec.KeyCodec[T] {
	return lengthPrefixedAddressKey[T]{
		keyCodec,
	}
}

// Collection Codecs

type intValueCodec struct{}

func (i intValueCodec) Encode(value math.Int) ([]byte, error) {
	return value.Marshal()
}

func (i intValueCodec) Decode(b []byte) (math.Int, error) {
	v := new(math.Int)
	err := v.Unmarshal(b)
	if err != nil {
		return math.Int{}, err
	}
	return *v, nil
}

func (i intValueCodec) EncodeJSON(value math.Int) ([]byte, error) {
	return value.MarshalJSON()
}

func (i intValueCodec) DecodeJSON(b []byte) (math.Int, error) {
	v := new(math.Int)
	err := v.UnmarshalJSON(b)
	if err != nil {
		return math.Int{}, err
	}
	return *v, nil
}

func (i intValueCodec) Stringify(value math.Int) string {
	return value.String()
}

func (i intValueCodec) ValueType() string {
	return "math.Int"
}

type timeKeyCodec struct{}

func (timeKeyCodec) Encode(buffer []byte, key time.Time) (int, error) {
	return copy(buffer, FormatTimeBytes(key)), nil
}

var timeSize = len(FormatTimeBytes(time.Time{}))

func (timeKeyCodec) Decode(buffer []byte) (int, time.Time, error) {
	if len(buffer) != timeSize {
		return 0, time.Time{}, fmt.Errorf("invalid time buffer buffer size")
	}
	t, err := ParseTimeBytes(buffer)
	if err != nil {
		return 0, time.Time{}, err
	}
	return timeSize, t, nil
}

func (timeKeyCodec) Size(key time.Time) int { return timeSize }

func (timeKeyCodec) EncodeJSON(value time.Time) ([]byte, error) { return value.MarshalJSON() }

func (timeKeyCodec) DecodeJSON(b []byte) (time.Time, error) {
	t := time.Time{}
	err := t.UnmarshalJSON(b)
	return t, err
}

func (timeKeyCodec) Stringify(key time.Time) string { return key.String() }
func (timeKeyCodec) KeyType() string                { return "sdk/time.Time" }
func (t timeKeyCodec) EncodeNonTerminal(buffer []byte, key time.Time) (int, error) {
	return t.Encode(buffer, key)
}

func (t timeKeyCodec) DecodeNonTerminal(buffer []byte) (int, time.Time, error) {
	if len(buffer) < timeSize {
		return 0, time.Time{}, fmt.Errorf("invalid time buffer size, wanted: %d at least, got: %d", timeSize, len(buffer))
	}
	return t.Decode(buffer[:timeSize])
}
func (t timeKeyCodec) SizeNonTerminal(key time.Time) int { return t.Size(key) }
