// Package types contains all the respective p2p types that are required for sync
// but cannot be represented as a protobuf schema. This package also contains those
// types associated fast ssz methods.
package types

import (
	"bytes"
	"sort"

	"github.com/pkg/errors"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

const rootLength = 32

const maxErrorLength = 256

// SSZBytes is a bytes slice that satisfies the fast-ssz interface.
type SSZBytes []byte

// HashTreeRoot hashes the uint64 object following the SSZ standard.
func (b *SSZBytes) HashTreeRoot() ([32]byte, error) {
	return ssz.HashWithDefaultHasher(b)
}

// HashTreeRootWith hashes the uint64 object with the given hasher.
func (b *SSZBytes) HashTreeRootWith(hh *ssz.Hasher) error {
	indx := hh.Index()
	hh.PutBytes(*b)
	hh.Merkleize(indx)
	return nil
}

// BeaconBlockByRootsReq specifies the block by roots request type.
type BeaconBlockByRootsReq [][rootLength]byte

// MarshalSSZTo marshals the block by roots request with the provided byte slice.
func (r *BeaconBlockByRootsReq) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalledObj, err := r.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalledObj...), nil
}

// MarshalSSZ Marshals the block by roots request type into the serialized object.
func (r *BeaconBlockByRootsReq) MarshalSSZ() ([]byte, error) {
	if len(*r) > int(params.BeaconConfig().MaxRequestBlocks) {
		return nil, errors.Errorf("beacon block by roots request exceeds max size: %d > %d", len(*r), params.BeaconConfig().MaxRequestBlocks)
	}
	buf := make([]byte, 0, r.SizeSSZ())
	for _, r := range *r {
		buf = append(buf, r[:]...)
	}
	return buf, nil
}

// SizeSSZ returns the size of the serialized representation.
func (r *BeaconBlockByRootsReq) SizeSSZ() int {
	return len(*r) * rootLength
}

// UnmarshalSSZ unmarshals the provided bytes buffer into the
// block by roots request object.
func (r *BeaconBlockByRootsReq) UnmarshalSSZ(buf []byte) error {
	bufLen := len(buf)
	maxLength := int(params.BeaconConfig().MaxRequestBlocks * rootLength)
	if bufLen > maxLength {
		return errors.Errorf("expected buffer with length of up to %d but received length %d", maxLength, bufLen)
	}
	if bufLen%rootLength != 0 {
		return ssz.ErrIncorrectByteSize
	}
	numOfRoots := bufLen / rootLength
	roots := make([][rootLength]byte, 0, numOfRoots)
	for i := 0; i < numOfRoots; i++ {
		var rt [rootLength]byte
		copy(rt[:], buf[i*rootLength:(i+1)*rootLength])
		roots = append(roots, rt)
	}
	*r = roots
	return nil
}

// ErrorMessage describes the error message type.
type ErrorMessage []byte

// MarshalSSZTo marshals the error message with the provided byte slice.
func (m *ErrorMessage) MarshalSSZTo(dst []byte) ([]byte, error) {
	marshalledObj, err := m.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalledObj...), nil
}

// MarshalSSZ Marshals the error message into the serialized object.
func (m *ErrorMessage) MarshalSSZ() ([]byte, error) {
	if len(*m) > maxErrorLength {
		return nil, errors.Errorf("error message exceeds max size: %d > %d", len(*m), maxErrorLength)
	}
	buf := make([]byte, m.SizeSSZ())
	copy(buf, *m)
	return buf, nil
}

// SizeSSZ returns the size of the serialized representation.
func (m *ErrorMessage) SizeSSZ() int {
	return len(*m)
}

// UnmarshalSSZ unmarshals the provided bytes buffer into the
// error message object.
func (m *ErrorMessage) UnmarshalSSZ(buf []byte) error {
	bufLen := len(buf)
	maxLength := maxErrorLength
	if bufLen > maxLength {
		return errors.Errorf("expected buffer with length of upto %d but received length %d", maxLength, bufLen)
	}
	errMsg := make([]byte, bufLen)
	copy(errMsg, buf)
	*m = errMsg
	return nil
}

// BlobSidecarsByRootReq is used to specify a list of blob targets (root+index) in a BlobSidecarsByRoot RPC request.
type BlobSidecarsByRootReq []*eth.BlobIdentifier

// BlobIdentifier is a fixed size value, so we can compute its fixed size at start time (see init below)
var blobIdSize int

// SizeSSZ returns the size of the serialized representation.
func (b *BlobSidecarsByRootReq) SizeSSZ() int {
	return len(*b) * blobIdSize
}

// MarshalSSZTo appends the serialized BlobSidecarsByRootReq value to the provided byte slice.
func (b *BlobSidecarsByRootReq) MarshalSSZTo(dst []byte) ([]byte, error) {
	// A List without an enclosing container is marshaled exactly like a vector, no length offset required.
	marshalledObj, err := b.MarshalSSZ()
	if err != nil {
		return nil, err
	}
	return append(dst, marshalledObj...), nil
}

// MarshalSSZ serializes the BlobSidecarsByRootReq value to a byte slice.
func (b *BlobSidecarsByRootReq) MarshalSSZ() ([]byte, error) {
	buf := make([]byte, len(*b)*blobIdSize)
	for i, id := range *b {
		by, err := id.MarshalSSZ()
		if err != nil {
			return nil, err
		}
		copy(buf[i*blobIdSize:(i+1)*blobIdSize], by)
	}
	return buf, nil
}

// UnmarshalSSZ unmarshals the provided bytes buffer into the
// BlobSidecarsByRootReq value.
func (b *BlobSidecarsByRootReq) UnmarshalSSZ(buf []byte) error {
	bufLen := len(buf)
	maxLength := int(params.BeaconConfig().MaxRequestBlobSidecars) * blobIdSize
	if bufLen > maxLength {
		return errors.Errorf("expected buffer with length of up to %d but received length %d", maxLength, bufLen)
	}
	if bufLen%blobIdSize != 0 {
		return errors.Wrapf(ssz.ErrIncorrectByteSize, "size=%d", bufLen)
	}
	count := bufLen / blobIdSize
	*b = make([]*eth.BlobIdentifier, count)
	for i := 0; i < count; i++ {
		id := &eth.BlobIdentifier{}
		err := id.UnmarshalSSZ(buf[i*blobIdSize : (i+1)*blobIdSize])
		if err != nil {
			return err
		}
		(*b)[i] = id
	}
	return nil
}

var _ sort.Interface = BlobSidecarsByRootReq{}

// Less reports whether the element with index i must sort before the element with index j.
// BlobIdentifier will be sorted in lexicographic order by root, with Blob Index as tiebreaker for a given root.
func (s BlobSidecarsByRootReq) Less(i, j int) bool {
	rootCmp := bytes.Compare(s[i].BlockRoot, s[j].BlockRoot)
	if rootCmp != 0 {
		// They aren't equal; return true if i < j, false if i > j.
		return rootCmp < 0
	}
	// They are equal; blob index is the tie breaker.
	return s[i].Index < s[j].Index
}

// Swap swaps the elements with indexes i and j.
func (s BlobSidecarsByRootReq) Swap(i, j int) {
	s[i], s[j] = s[j], s[i]
}

// Len is the number of elements in the collection.
func (s BlobSidecarsByRootReq) Len() int {
	return len(s)
}

func init() {
	sizer := &eth.BlobIdentifier{}
	blobIdSize = sizer.SizeSSZ()
}
