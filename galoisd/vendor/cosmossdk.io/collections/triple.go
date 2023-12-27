package collections

import (
	"encoding/json"
	"fmt"
	"strings"

	"cosmossdk.io/collections/codec"
)

// Triple defines a multipart key composed of three keys.
type Triple[K1, K2, K3 any] struct {
	k1 *K1
	k2 *K2
	k3 *K3
}

// Join3 instantiates a new Triple instance composed of the three provided keys, in order.
func Join3[K1, K2, K3 any](k1 K1, k2 K2, k3 K3) Triple[K1, K2, K3] {
	return Triple[K1, K2, K3]{&k1, &k2, &k3}
}

// K1 returns the first part of the key. If nil, the zero value is returned.
func (t Triple[K1, K2, K3]) K1() (x K1) {
	if t.k1 != nil {
		return *t.k1
	}
	return x
}

// K2 returns the second part of the key. If nil, the zero value is returned.
func (t Triple[K1, K2, K3]) K2() (x K2) {
	if t.k2 != nil {
		return *t.k2
	}
	return x
}

// K3 returns the third part of the key. If nil, the zero value is returned.
func (t Triple[K1, K2, K3]) K3() (x K3) {
	if t.k3 != nil {
		return *t.k3
	}
	return x
}

// TriplePrefix creates a new Triple instance composed only of the first part of the key.
func TriplePrefix[K1, K2, K3 any](k1 K1) Triple[K1, K2, K3] {
	return Triple[K1, K2, K3]{k1: &k1}
}

// TripleSuperPrefix creates a new Triple instance composed only of the first two parts of the key.
func TripleSuperPrefix[K1, K2, K3 any](k1 K1, k2 K2) Triple[K1, K2, K3] {
	return Triple[K1, K2, K3]{k1: &k1, k2: &k2}
}

// TripleKeyCodec instantiates a new KeyCodec instance that can encode the Triple, given
// the KeyCodecs of the three parts of the key, in order.
func TripleKeyCodec[K1, K2, K3 any](keyCodec1 codec.KeyCodec[K1], keyCodec2 codec.KeyCodec[K2], keyCodec3 codec.KeyCodec[K3]) codec.KeyCodec[Triple[K1, K2, K3]] {
	return tripleKeyCodec[K1, K2, K3]{
		keyCodec1: keyCodec1,
		keyCodec2: keyCodec2,
		keyCodec3: keyCodec3,
	}
}

type tripleKeyCodec[K1, K2, K3 any] struct {
	keyCodec1 codec.KeyCodec[K1]
	keyCodec2 codec.KeyCodec[K2]
	keyCodec3 codec.KeyCodec[K3]
}

type jsonTripleKey [3]json.RawMessage

func (t tripleKeyCodec[K1, K2, K3]) EncodeJSON(value Triple[K1, K2, K3]) ([]byte, error) {
	json1, err := t.keyCodec1.EncodeJSON(*value.k1)
	if err != nil {
		return nil, err
	}

	json2, err := t.keyCodec2.EncodeJSON(*value.k2)
	if err != nil {
		return nil, err
	}

	json3, err := t.keyCodec3.EncodeJSON(*value.k3)
	if err != nil {
		return nil, err
	}

	return json.Marshal(jsonTripleKey{json1, json2, json3})
}

func (t tripleKeyCodec[K1, K2, K3]) DecodeJSON(b []byte) (Triple[K1, K2, K3], error) {
	var jsonKey jsonTripleKey
	err := json.Unmarshal(b, &jsonKey)
	if err != nil {
		return Triple[K1, K2, K3]{}, err
	}

	key1, err := t.keyCodec1.DecodeJSON(jsonKey[0])
	if err != nil {
		return Triple[K1, K2, K3]{}, err
	}

	key2, err := t.keyCodec2.DecodeJSON(jsonKey[1])
	if err != nil {
		return Triple[K1, K2, K3]{}, err
	}

	key3, err := t.keyCodec3.DecodeJSON(jsonKey[2])
	if err != nil {
		return Triple[K1, K2, K3]{}, err
	}

	return Join3(key1, key2, key3), nil
}

func (t tripleKeyCodec[K1, K2, K3]) Stringify(key Triple[K1, K2, K3]) string {
	b := new(strings.Builder)
	b.WriteByte('(')
	if key.k1 != nil {
		b.WriteByte('"')
		b.WriteString(t.keyCodec1.Stringify(*key.k1))
		b.WriteByte('"')
	} else {
		b.WriteString("<nil>")
	}

	b.WriteString(", ")
	if key.k2 != nil {
		b.WriteByte('"')
		b.WriteString(t.keyCodec2.Stringify(*key.k2))
		b.WriteByte('"')
	} else {
		b.WriteString("<nil>")
	}

	b.WriteString(", ")
	if key.k3 != nil {
		b.WriteByte('"')
		b.WriteString(t.keyCodec3.Stringify(*key.k3))
		b.WriteByte('"')
	} else {
		b.WriteString("<nil>")
	}

	b.WriteByte(')')
	return b.String()
}

func (t tripleKeyCodec[K1, K2, K3]) KeyType() string {
	return fmt.Sprintf("Triple[%s,%s,%s]", t.keyCodec1.KeyType(), t.keyCodec2.KeyType(), t.keyCodec3.KeyType())
}

func (t tripleKeyCodec[K1, K2, K3]) Encode(buffer []byte, key Triple[K1, K2, K3]) (int, error) {
	writtenTotal := 0
	if key.k1 != nil {
		written, err := t.keyCodec1.EncodeNonTerminal(buffer, *key.k1)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	if key.k2 != nil {
		written, err := t.keyCodec2.EncodeNonTerminal(buffer[writtenTotal:], *key.k2)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	if key.k3 != nil {
		written, err := t.keyCodec3.Encode(buffer[writtenTotal:], *key.k3)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	return writtenTotal, nil
}

func (t tripleKeyCodec[K1, K2, K3]) Decode(buffer []byte) (int, Triple[K1, K2, K3], error) {
	readTotal := 0
	read, key1, err := t.keyCodec1.DecodeNonTerminal(buffer)
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	read, key2, err := t.keyCodec2.DecodeNonTerminal(buffer[readTotal:])
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	read, key3, err := t.keyCodec3.Decode(buffer[readTotal:])
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	return readTotal, Join3(key1, key2, key3), nil
}

func (t tripleKeyCodec[K1, K2, K3]) Size(key Triple[K1, K2, K3]) int {
	size := 0
	if key.k1 != nil {
		size += t.keyCodec1.SizeNonTerminal(*key.k1)
	}
	if key.k2 != nil {
		size += t.keyCodec2.SizeNonTerminal(*key.k2)
	}
	if key.k3 != nil {
		size += t.keyCodec3.Size(*key.k3)
	}
	return size
}

func (t tripleKeyCodec[K1, K2, K3]) EncodeNonTerminal(buffer []byte, key Triple[K1, K2, K3]) (int, error) {
	writtenTotal := 0
	if key.k1 != nil {
		written, err := t.keyCodec1.EncodeNonTerminal(buffer, *key.k1)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	if key.k2 != nil {
		written, err := t.keyCodec2.EncodeNonTerminal(buffer[writtenTotal:], *key.k2)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	if key.k3 != nil {
		written, err := t.keyCodec3.EncodeNonTerminal(buffer[writtenTotal:], *key.k3)
		if err != nil {
			return 0, err
		}
		writtenTotal += written
	}
	return writtenTotal, nil
}

func (t tripleKeyCodec[K1, K2, K3]) DecodeNonTerminal(buffer []byte) (int, Triple[K1, K2, K3], error) {
	readTotal := 0
	read, key1, err := t.keyCodec1.DecodeNonTerminal(buffer)
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	read, key2, err := t.keyCodec2.DecodeNonTerminal(buffer[readTotal:])
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	read, key3, err := t.keyCodec3.DecodeNonTerminal(buffer[readTotal:])
	if err != nil {
		return 0, Triple[K1, K2, K3]{}, err
	}
	readTotal += read
	return readTotal, Join3(key1, key2, key3), nil
}

func (t tripleKeyCodec[K1, K2, K3]) SizeNonTerminal(key Triple[K1, K2, K3]) int {
	size := 0
	if key.k1 != nil {
		size += t.keyCodec1.SizeNonTerminal(*key.k1)
	}
	if key.k2 != nil {
		size += t.keyCodec2.SizeNonTerminal(*key.k2)
	}
	if key.k3 != nil {
		size += t.keyCodec3.SizeNonTerminal(*key.k3)
	}
	return size
}

// NewPrefixUntilTripleRange defines a collection query which ranges until the provided Pair prefix.
// Unstable: this API might change in the future.
func NewPrefixUntilTripleRange[K1, K2, K3 any](k1 K1) Ranger[Triple[K1, K2, K3]] {
	key := TriplePrefix[K1, K2, K3](k1)
	return &Range[Triple[K1, K2, K3]]{
		end: RangeKeyPrefixEnd(key),
	}
}

// NewPrefixedTripleRange provides a Range for all keys prefixed with the given
// first part of the Triple key.
func NewPrefixedTripleRange[K1, K2, K3 any](k1 K1) Ranger[Triple[K1, K2, K3]] {
	key := TriplePrefix[K1, K2, K3](k1)
	return &Range[Triple[K1, K2, K3]]{
		start: RangeKeyExact(key),
		end:   RangeKeyPrefixEnd(key),
	}
}

// NewSuperPrefixedTripleRange provides a Range for all keys prefixed with the given
// first and second parts of the Triple key.
func NewSuperPrefixedTripleRange[K1, K2, K3 any](k1 K1, k2 K2) Ranger[Triple[K1, K2, K3]] {
	key := TripleSuperPrefix[K1, K2, K3](k1, k2)
	return &Range[Triple[K1, K2, K3]]{
		start: RangeKeyExact(key),
		end:   RangeKeyPrefixEnd(key),
	}
}
