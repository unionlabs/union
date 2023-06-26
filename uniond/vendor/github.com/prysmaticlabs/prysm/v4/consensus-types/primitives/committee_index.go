package primitives

import (
	"fmt"

	fssz "github.com/prysmaticlabs/fastssz"
)

var _ fssz.HashRoot = (CommitteeIndex)(0)
var _ fssz.Marshaler = (*CommitteeIndex)(nil)
var _ fssz.Unmarshaler = (*CommitteeIndex)(nil)

// CommitteeIndex --
type CommitteeIndex uint64

// HashTreeRoot returns calculated hash root.
func (c CommitteeIndex) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(c)
}

// HashTreeRootWith --
func (c CommitteeIndex) HashTreeRootWith(hh *fssz.Hasher) error {
	hh.PutUint64(uint64(c))
	return nil
}

// UnmarshalSSZ --
func (c *CommitteeIndex) UnmarshalSSZ(buf []byte) error {
	if len(buf) != c.SizeSSZ() {
		return fmt.Errorf("expected buffer of length %d receiced %d", c.SizeSSZ(), len(buf))
	}
	*c = CommitteeIndex(fssz.UnmarshallUint64(buf))
	return nil
}

// MarshalSSZTo --
func (c *CommitteeIndex) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalled, err := c.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalled...), nil
}

// MarshalSSZ --
func (c *CommitteeIndex) MarshalSSZ() ([]byte, error) {
	marshalled := fssz.MarshalUint64([]byte{}, uint64(*c))
	return marshalled, nil
}

// SizeSSZ returns the size of the serialized object.
func (c *CommitteeIndex) SizeSSZ() int {
	return 8
}
