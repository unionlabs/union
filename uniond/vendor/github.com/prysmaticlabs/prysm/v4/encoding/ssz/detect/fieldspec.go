package detect

import (
	"encoding/binary"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

type fieldType int

const (
	typeUndefined fieldType = iota
	typeUint64
	typeBytes4
)

func (f fieldType) String() string {
	switch f {
	case typeUint64:
		return "uint64"
	case typeBytes4:
		return "bytes4"
	case typeUndefined:
		return "undefined"
	default:
		return "invalid"
	}
}

func (f fieldType) Size() int {
	switch f {
	case typeUint64:
		return 8
	case typeBytes4:
		return 4
	default:
		panic("can't determine size for unrecognizedtype ")
	}
}

var errWrongMethodForType = errors.New("wrong fieldSpec method for type")
var errIndexOutOfRange = errors.New("value index would exceed byte length")

type fieldSpec struct {
	offset int
	t      fieldType
}

func (f *fieldSpec) uint64(state []byte) (uint64, error) {
	if f.t != typeUint64 {
		return 0, errors.Wrapf(errWrongMethodForType, "called uint64() for type=%s", f.t)
	}
	s, err := f.slice(state)
	if err != nil {
		return 0, err
	}
	return binary.LittleEndian.Uint64(s), nil
}

func (f *fieldSpec) bytes4(state []byte) ([4]byte, error) {
	var b4 [4]byte
	if f.t != typeBytes4 {
		return b4, errors.Wrapf(errWrongMethodForType, "called bytes4() with fieldType=%s", f.t)
	}
	val, err := f.slice(state)
	if err != nil {
		return b4, err
	}
	return bytesutil.ToBytes4(val), nil
}

func (f *fieldSpec) slice(value []byte) ([]byte, error) {
	size := f.t.Size()
	if len(value) < f.offset+size {
		return nil, errors.Wrapf(errIndexOutOfRange, "offset=%d, size=%d, byte len=%d", f.offset, size, len(value))
	}
	return value[f.offset : f.offset+size], nil
}
