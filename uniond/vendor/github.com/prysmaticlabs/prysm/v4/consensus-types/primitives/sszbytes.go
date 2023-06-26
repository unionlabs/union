package primitives

import (
	fssz "github.com/prysmaticlabs/fastssz"
)

// SSZBytes --
type SSZBytes []byte

// HashTreeRoot --
func (b *SSZBytes) HashTreeRoot() ([32]byte, error) {
	return fssz.HashWithDefaultHasher(b)
}

// HashTreeRootWith --
func (b *SSZBytes) HashTreeRootWith(hh *fssz.Hasher) error {
	indx := hh.Index()
	hh.PutBytes(*b)
	hh.Merkleize(indx)
	return nil
}
