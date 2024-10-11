package types

import (
	"encoding/binary"
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

	// UintValue represents a collections.ValueCodec to work with Uint.
	UintValue collcodec.ValueCodec[math.Uint] = uintValueCodec{}

	// LegacyDecValue represents a collections.ValueCodec to work with LegacyDec.
	LegacyDecValue collcodec.ValueCodec[math.LegacyDec] = legacyDecValueCodec{}

	// TimeKey represents a collections.KeyCodec to work with time.Time
	// Deprecated: exists only for state compatibility reasons, should not
	// be used for new storage keys using time. Please use the time KeyCodec
	// provided in the collections package.
	TimeKey collcodec.KeyCodec[time.Time] = timeKeyCodec{}

	// LEUint64Key is a collections KeyCodec that encodes uint64 using little endian.
	// NOTE: it MUST NOT be used by other modules, distribution relies on this only for
	// state backwards compatibility.
	// Deprecated: use collections.Uint64Key instead.
	LEUint64Key collcodec.KeyCodec[uint64] = leUint64Key{}

	// LengthPrefixedBytesKey is a collections KeyCodec to work with []byte.
	// Deprecated: exists only for state compatibility reasons, should not be
	// used for new storage keys using []byte. Please use the BytesKey provided
	// in the collections package.
	LengthPrefixedBytesKey collcodec.KeyCodec[[]byte] = lengthPrefixedBytesKey{collections.BytesKey}
)

const (
	Int       string = "math.Int"
	Uint      string = "math.Uint"
	LegacyDec string = "math.LegacyDec"
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

// Deprecated: lengthPrefixedBytesKey is a special key codec used to retain state backwards compatibility
// when a bytes key is used as an index key.
type lengthPrefixedBytesKey struct {
	collcodec.KeyCodec[[]byte]
}

func (g lengthPrefixedBytesKey) Encode(buffer, key []byte) (int, error) {
	return g.EncodeNonTerminal(buffer, key)
}

func (g lengthPrefixedBytesKey) Decode(buffer []byte) (int, []byte, error) {
	return g.DecodeNonTerminal(buffer)
}

func (g lengthPrefixedBytesKey) Size(key []byte) int {
	return g.SizeNonTerminal(key)
}

func (g lengthPrefixedBytesKey) KeyType() string {
	return "index_key/" + g.KeyCodec.KeyType()
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
	return Int
}

type uintValueCodec struct{}

func (i uintValueCodec) Encode(value math.Uint) ([]byte, error) {
	return value.Marshal()
}

func (i uintValueCodec) Decode(b []byte) (math.Uint, error) {
	v := new(math.Uint)
	err := v.Unmarshal(b)
	if err != nil {
		return math.Uint{}, err
	}
	return *v, nil
}

func (i uintValueCodec) EncodeJSON(value math.Uint) ([]byte, error) {
	return value.MarshalJSON()
}

func (i uintValueCodec) DecodeJSON(b []byte) (math.Uint, error) {
	v := new(math.Uint)
	err := v.UnmarshalJSON(b)
	if err != nil {
		return math.Uint{}, err
	}
	return *v, nil
}

func (i uintValueCodec) Stringify(value math.Uint) string {
	return value.String()
}

func (i uintValueCodec) ValueType() string {
	return Uint
}

type legacyDecValueCodec struct{}

func (i legacyDecValueCodec) Encode(value math.LegacyDec) ([]byte, error) {
	return value.Marshal()
}

func (i legacyDecValueCodec) Decode(b []byte) (math.LegacyDec, error) {
	v := new(math.LegacyDec)
	err := v.Unmarshal(b)
	if err != nil {
		return math.LegacyDec{}, err
	}
	return *v, nil
}

func (i legacyDecValueCodec) EncodeJSON(value math.LegacyDec) ([]byte, error) {
	return value.MarshalJSON()
}

func (i legacyDecValueCodec) DecodeJSON(b []byte) (math.LegacyDec, error) {
	v := new(math.LegacyDec)
	err := v.UnmarshalJSON(b)
	if err != nil {
		return math.LegacyDec{}, err
	}
	return *v, nil
}

func (i legacyDecValueCodec) Stringify(value math.LegacyDec) string {
	return value.String()
}

func (i legacyDecValueCodec) ValueType() string {
	return LegacyDec
}

type timeKeyCodec struct{}

func (timeKeyCodec) Encode(buffer []byte, key time.Time) (int, error) {
	return copy(buffer, FormatTimeBytes(key)), nil
}

var timeSize = len(FormatTimeBytes(time.Time{}))

func (timeKeyCodec) Decode(buffer []byte) (int, time.Time, error) {
	if len(buffer) != timeSize {
		return 0, time.Time{}, fmt.Errorf("invalid time buffer size")
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

type leUint64Key struct{}

func (l leUint64Key) Encode(buffer []byte, key uint64) (int, error) {
	binary.LittleEndian.PutUint64(buffer, key)
	return 8, nil
}

func (l leUint64Key) Decode(buffer []byte) (int, uint64, error) {
	if size := len(buffer); size < 8 {
		return 0, 0, fmt.Errorf("invalid buffer size, wanted 8 at least got %d", size)
	}
	return 8, binary.LittleEndian.Uint64(buffer), nil
}

func (l leUint64Key) Size(_ uint64) int { return 8 }

func (l leUint64Key) EncodeJSON(value uint64) ([]byte, error) {
	return collections.Uint64Key.EncodeJSON(value)
}

func (l leUint64Key) DecodeJSON(b []byte) (uint64, error) { return collections.Uint64Key.DecodeJSON(b) }

func (l leUint64Key) Stringify(key uint64) string { return collections.Uint64Key.Stringify(key) }

func (l leUint64Key) KeyType() string { return "little-endian-uint64" }

func (l leUint64Key) EncodeNonTerminal(buffer []byte, key uint64) (int, error) {
	return l.Encode(buffer, key)
}

func (l leUint64Key) DecodeNonTerminal(buffer []byte) (int, uint64, error) { return l.Decode(buffer) }

func (l leUint64Key) SizeNonTerminal(_ uint64) int { return 8 }
