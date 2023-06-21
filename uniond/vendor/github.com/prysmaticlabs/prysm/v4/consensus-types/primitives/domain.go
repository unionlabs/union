package primitives

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
)

var _ fssz.HashRoot = (Domain)([]byte{})
var _ fssz.Marshaler = (*Domain)(nil)
var _ fssz.Unmarshaler = (*Domain)(nil)

// Domain represents a 32 bytes domain object in Ethereum beacon chain consensus.
type Domain []byte

// HashTreeRoot --
func (d Domain) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(d)
}

// HashTreeRootWith --
func (d Domain) HashTreeRootWith(hh *fssz.Hasher) error {
	hh.PutBytes(d[:])
	return nil
}

// UnmarshalSSZ --
func (d *Domain) UnmarshalSSZ(buf []byte) error {
	if len(buf) != d.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", d.SizeSSZ(), len(buf))
	}

	var b [32]byte
	item := Domain(b[:])
	copy(item, buf)
	*d = item
	return nil
}

// MarshalSSZTo --
func (d *Domain) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := d.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ --
func (d *Domain) MarshalSSZ() ([]byte, error) {
	return *d, nil
}

// SizeSSZ --
func (_ *Domain) SizeSSZ() int {
	return 32
}
