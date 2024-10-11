// Package cbor implements just enough of the CBOR (Concise Binary Object
// Representation, RFC 8948) to deterministically encode simple data. It does
// not include decoding as it is not needed for the purpose of this package.
package cbor

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"io"
	"math"
	"sort"
)

const (
	majorUint       byte = 0
	majorNegInt     byte = 1
	majorByteString byte = 2
	majorTextString byte = 3
	majorArray      byte = 4
	majorMap        byte = 5
	majorTagged     byte = 6
	majorSimple     byte = 7
)

func encodeFirstByte(major, extra byte) byte {
	return (major << 5) | extra&0x1F
}

func encodePrefix(major byte, arg uint64, w io.Writer) error {
	switch {
	case arg < 24:
		_, err := w.Write([]byte{encodeFirstByte(major, byte(arg))})
		return err
	case arg <= math.MaxUint8:
		_, err := w.Write([]byte{encodeFirstByte(major, 24), byte(arg)})
		return err
	case arg <= math.MaxUint16:
		_, err := w.Write([]byte{encodeFirstByte(major, 25)})
		if err != nil {
			return err
		}
		// #nosec G701
		// Since we're under the limit, narrowing is safe.
		return binary.Write(w, binary.BigEndian, uint16(arg))
	case arg <= math.MaxUint32:
		_, err := w.Write([]byte{encodeFirstByte(major, 26)})
		if err != nil {
			return err
		}
		// #nosec G701
		// Since we're under the limit, narrowing is safe.
		return binary.Write(w, binary.BigEndian, uint32(arg))
	}
	_, err := w.Write([]byte{encodeFirstByte(major, 27)})
	if err != nil {
		return err
	}
	return binary.Write(w, binary.BigEndian, arg)
}

// Cbor is a CBOR (RFC8949) data item that can be encoded to a stream.
type Cbor interface {
	// Encode deterministically writes the CBOR-encoded data to the stream.
	Encode(w io.Writer) error
}

// Uint is the CBOR unsigned integer type.
type Uint uint64

// NewUint returns a CBOR unsigned integer data item.
func NewUint(n uint64) Uint {
	return Uint(n)
}

var _ Cbor = NewUint(0)

// Encode implements the Cbor interface.
func (n Uint) Encode(w io.Writer) error {
	// #nosec G701
	// Widening is safe.
	return encodePrefix(majorUint, uint64(n), w)
}

// Text is the CBOR text string type.
type Text string

// NewText returns a CBOR text string data item.
func NewText(s string) Text {
	return Text(s)
}

var _ Cbor = NewText("")

// Encode implements the Cbor interface.
func (s Text) Encode(w io.Writer) error {
	err := encodePrefix(majorTextString, uint64(len(s)), w)
	if err != nil {
		return err
	}
	_, err = w.Write([]byte(string(s)))
	return err
}

// Array is the CBOR array type.
type Array struct {
	elts []Cbor
}

// NewArray returns a CBOR array data item,
// containing the specified elements.
func NewArray(elts ...Cbor) Array {
	return Array{elts: elts}
}

var _ Cbor = NewArray()

// Append appends CBOR data items to an existing Array.
func (a Array) Append(c Cbor) Array {
	a.elts = append(a.elts, c)
	return a
}

// Encode implements the Cbor interface.
func (a Array) Encode(w io.Writer) error {
	err := encodePrefix(majorArray, uint64(len(a.elts)), w)
	if err != nil {
		return err
	}
	for _, elt := range a.elts {
		err = elt.Encode(w)
		if err != nil {
			return err
		}
	}
	return nil
}

// Entry is a key/value pair in a CBOR map.
type Entry struct {
	key, val Cbor
}

// NewEntry returns a CBOR key/value pair for use in a Map.
func NewEntry(key, val Cbor) Entry {
	return Entry{key: key, val: val}
}

// Map is the CBOR map type.
type Map struct {
	entries []Entry
}

// NewMap returns a CBOR map data item containing the specified entries.
// Duplicate keys in the Map will cause an error when Encode is called.
func NewMap(entries ...Entry) Map {
	return Map{entries: entries}
}

// Add adds a key/value entry to an existing Map.
// Duplicate keys in the Map will cause an error when Encode is called.
func (m Map) Add(key, val Cbor) Map {
	m.entries = append(m.entries, NewEntry(key, val))
	return m
}

type keyIdx struct {
	key []byte
	idx int
}

// Encode implements the Cbor interface.
func (m Map) Encode(w io.Writer) error {
	err := encodePrefix(majorMap, uint64(len(m.entries)), w)
	if err != nil {
		return err
	}
	// For deterministic encoding, map entries must be sorted by their
	// encoded keys in bytewise lexicographic order (RFC 8949, section 4.2.1).
	renderedKeys := make([]keyIdx, len(m.entries))
	for i, entry := range m.entries {
		var buf bytes.Buffer
		err := entry.key.Encode(&buf)
		if err != nil {
			return err
		}
		renderedKeys[i] = keyIdx{key: buf.Bytes(), idx: i}
	}
	sort.SliceStable(renderedKeys, func(i, j int) bool {
		return bytes.Compare(renderedKeys[i].key, renderedKeys[j].key) < 0
	})
	var prevKey []byte
	for i, rk := range renderedKeys {
		if i > 0 && bytes.Equal(prevKey, rk.key) {
			return fmt.Errorf("duplicate map keys at %d and %d", rk.idx, renderedKeys[i-1].idx)
		}
		prevKey = rk.key
		_, err = w.Write(rk.key)
		if err != nil {
			return err
		}
		err = m.entries[rk.idx].val.Encode(w)
		if err != nil {
			return err
		}
	}
	return nil
}

const (
	simpleFalse     byte = 20
	simpleTrue      byte = 21
	simpleNull      byte = 22
	simpleUndefined byte = 32
)

func encodeSimple(b byte, w io.Writer) error {
	// #nosec G701
	// Widening is safe.
	return encodePrefix(majorSimple, uint64(b), w)
}

// Bool is the type of CBOR booleans.
type Bool byte

// NewBool returns a CBOR boolean data item.
func NewBool(b bool) Bool {
	if b {
		return Bool(simpleTrue)
	}
	return Bool(simpleFalse)
}

var _ Cbor = NewBool(false)

// Encode implements the Cbor interface.
func (b Bool) Encode(w io.Writer) error {
	return encodeSimple(byte(b), w)
}
