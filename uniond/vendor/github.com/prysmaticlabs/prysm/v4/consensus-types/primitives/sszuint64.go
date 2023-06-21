package primitives

import (
	"encoding/binary"
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
)

var _ fssz.HashRoot = (Epoch)(0)
var _ fssz.Marshaler = (*Epoch)(nil)
var _ fssz.Unmarshaler = (*Epoch)(nil)

// SSZUint64 --
type SSZUint64 uint64

// SizeSSZ --
func (s *SSZUint64) SizeSSZ() int {
	return 8
}

// MarshalSSZTo --
func (s *SSZUint64) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := s.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ --
func (s *SSZUint64) MarshalSSZ() ([]byte, error) {
	marshalled := fssz.MarshalUint64([]byte{}, uint64(*s))
	return marshalled, nil
}

// UnmarshalSSZ --
func (s *SSZUint64) UnmarshalSSZ(buf []byte) error {
	if len(buf) != s.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", s.SizeSSZ(), len(buf))
	}
	*s = SSZUint64(fssz.UnmarshallUint64(buf))
	return nil
}

// HashTreeRoot --
func (s *SSZUint64) HashTreeRoot() ([32]byte, error) {
	buf := make([]byte, 8)
	binary.LittleEndian.PutUint64(buf, uint64(*s))
	var root [32]byte
	copy(root[:], buf)
	return root, nil
}

// HashTreeRootWith --
func (s *SSZUint64) HashTreeRootWith(hh *fssz.Hasher) error {
	indx := hh.Index()
	hh.PutUint64(uint64(*s))
	hh.Merkleize(indx)
	return nil
}
