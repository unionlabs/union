package keyformat

import (
	"encoding/binary"
	"fmt"
	"math/big"
)

// Provides a fixed-width lexicographically sortable []byte key format
type KeyFormat struct {
	layout    []int
	length    int
	prefix    byte
	unbounded bool
}

// Create a []byte key format based on a single byte prefix and fixed width key segments each of whose length is
// specified by by the corresponding element of layout.
//
// For example, to store keys that could index some objects by a version number and their SHA256 hash using the form:
// 'c<version uint64><hash [32]byte>' then you would define the KeyFormat with:
//
//	var keyFormat = NewKeyFormat('c', 8, 32)
//
// Then you can create a key with:
//
//	func ObjectKey(version uint64, objectBytes []byte) []byte {
//		hasher := sha256.New()
//		hasher.Sum(nil)
//		return keyFormat.Key(version, hasher.Sum(nil))
//	}
//
// If the last term of the layout is 0 it is unbounded, and will accept any length.
//
// NewKeyFormat panics if any other term than the last is 0.
func NewKeyFormat(prefix byte, layout ...int) *KeyFormat {
	// For prefix byte
	length := 1
	for i, l := range layout {
		length += l
		if l == 0 && i != len(layout)-1 {
			panic("Only the last item in a key format can be 0")
		}
	}
	return &KeyFormat{
		prefix:    prefix,
		layout:    layout,
		length:    length,
		unbounded: len(layout) > 0 && layout[len(layout)-1] == 0,
	}
}

// Format the byte segments into the key format - will panic if the segment lengths do not match the layout.
func (kf *KeyFormat) KeyBytes(segments ...[]byte) []byte {
	keyLen := kf.length
	// In case segments length is less than layouts length,
	// we don't have to allocate the whole kf.length, just
	// enough space to store the segments.
	if len(segments) < len(kf.layout) {
		keyLen = 1
		for i := range segments {
			keyLen += kf.layout[i]
		}
	}

	if kf.unbounded {
		if len(segments) > 0 {
			keyLen += len(segments[len(segments)-1])
		}
	}
	key := make([]byte, keyLen)
	key[0] = kf.prefix
	n := 1
	for i, s := range segments {
		l := kf.layout[i]

		switch l {
		case 0:
			// If the expected segment length is unbounded, increase it by `string length`
			n += len(s)
		default:
			if len(s) > l {
				panic(fmt.Errorf("length of segment %X provided to KeyFormat.KeyBytes() is longer than the %d bytes "+
					"required by layout for segment %d", s, l, i))
			}
			// Otherwise increase n by the segment length
			n += l

		}
		// Big endian so pad on left if not given the full width for this segment
		copy(key[n-len(s):n], s)
	}
	return key[:n]
}

// Format the args passed into the key format - will panic if the arguments passed do not match the length
// of the segment to which they correspond. When called with no arguments returns the raw prefix (useful as a start
// element of the entire keys space when sorted lexicographically).
func (kf *KeyFormat) Key(args ...interface{}) []byte {
	if len(args) > len(kf.layout) {
		panic(fmt.Errorf("keyFormat.Key() is provided with %d args but format only has %d segments",
			len(args), len(kf.layout)))
	}
	segments := make([][]byte, len(args))
	for i, a := range args {
		segments[i] = format(a)
	}
	return kf.KeyBytes(segments...)
}

// Reads out the bytes associated with each segment of the key format from key.
func (kf *KeyFormat) ScanBytes(key []byte) [][]byte {
	segments := make([][]byte, len(kf.layout))
	n := 1
	for i, l := range kf.layout {
		n += l
		// if current section is longer than key, then there are no more subsequent segments.
		if n > len(key) {
			return segments[:i]
		}
		// if unbounded, segment is rest of key
		if l == 0 {
			segments[i] = key[n:]
			break
		}
		segments[i] = key[n-l : n]
	}
	return segments
}

// Extracts the segments into the values pointed to by each of args. Each arg must be a pointer to int64, uint64, or
// []byte, and the width of the args must match layout.
func (kf *KeyFormat) Scan(key []byte, args ...interface{}) {
	segments := kf.ScanBytes(key)
	if len(args) > len(segments) {
		panic(fmt.Errorf("keyFormat.Scan() is provided with %d args but format only has %d segments in key %X",
			len(args), len(segments), key))
	}
	for i, a := range args {
		scan(a, segments[i])
	}
}

// Length of the key format.
func (kf *KeyFormat) Length() int {
	return kf.length
}

// Return the prefix as a string.
func (kf *KeyFormat) Prefix() string {
	return string([]byte{kf.prefix})
}

func scan(a interface{}, value []byte) {
	switch v := a.(type) {
	case *int64:
		// Negative values will be mapped correctly when read in as uint64 and then type converted
		*v = int64(binary.BigEndian.Uint64(value))
	case *uint64:
		*v = binary.BigEndian.Uint64(value)
	case *uint32:
		*v = binary.BigEndian.Uint32(value)
	case *int32:
		*v = int32(binary.BigEndian.Uint32(value))
	case *[]byte:
		*v = value
	case *big.Int:
		*v = *big.NewInt(0).SetBytes(value)
	default:
		panic(fmt.Errorf("keyFormat scan() does not support scanning value of type %T: %v", a, a))
	}
}

func format(a interface{}) []byte {
	switch v := a.(type) {
	case uint64:
		return formatUint64(v)
	case int64:
		return formatUint64(uint64(v))
	// Provide formatting from int,uint as a convenience to avoid casting arguments
	case uint:
		return formatUint64(uint64(v))
	case int:
		return formatUint64(uint64(v))
	case uint32:
		return formatUint32(v)
	case int32:
		return formatUint32(uint32(v))
	case []byte:
		return v
	default:
		panic(fmt.Errorf("keyFormat format() does not support formatting value of type %T: %v", a, a))
	}
}

func formatUint64(v uint64) []byte {
	bs := make([]byte, 8)
	binary.BigEndian.PutUint64(bs, v)
	return bs
}

func formatUint32(v uint32) []byte {
	bs := make([]byte, 4)
	binary.BigEndian.PutUint32(bs, v)
	return bs
}
