package primitives

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
)

var _ fssz.HashRoot = (ValidatorIndex)(0)
var _ fssz.Marshaler = (*ValidatorIndex)(nil)
var _ fssz.Unmarshaler = (*ValidatorIndex)(nil)

// ValidatorIndex in eth2.
type ValidatorIndex uint64

// Div divides validator index by x.
func (v ValidatorIndex) Div(x uint64) ValidatorIndex {
	if x == 0 {
		panic("divbyzero")
	}
	return ValidatorIndex(uint64(v) / x)
}

// Add increases validator index by x.
func (v ValidatorIndex) Add(x uint64) ValidatorIndex {
	return ValidatorIndex(uint64(v) + x)
}

// Sub subtracts x from the validator index.
func (v ValidatorIndex) Sub(x uint64) ValidatorIndex {
	if uint64(v) < x {
		panic("underflow")
	}
	return ValidatorIndex(uint64(v) - x)
}

// Mod returns result of `validator index % x`.
func (v ValidatorIndex) Mod(x uint64) ValidatorIndex {
	return ValidatorIndex(uint64(v) % x)
}

// HashTreeRoot --
func (v ValidatorIndex) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(v)
}

// HashTreeRootWith --
func (v ValidatorIndex) HashTreeRootWith(hh *fssz.Hasher) error {
	hh.PutUint64(uint64(v))
	return nil
}

// UnmarshalSSZ --
func (v *ValidatorIndex) UnmarshalSSZ(buf []byte) error {
	if len(buf) != v.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d received %d", v.SizeSSZ(), len(buf))
	}
	*v = ValidatorIndex(fssz.UnmarshallUint64(buf))
	return nil
}

// MarshalSSZTo --
func (v *ValidatorIndex) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := v.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ --
func (v *ValidatorIndex) MarshalSSZ() ([]byte, error) {
	marshalled := fssz.MarshalUint64([]byte{}, uint64(*v))
	return marshalled, nil
}

// SizeSSZ --
func (v *ValidatorIndex) SizeSSZ() int {
	return 8
}
