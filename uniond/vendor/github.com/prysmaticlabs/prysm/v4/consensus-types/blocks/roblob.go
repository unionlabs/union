package blocks

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var errNilBlockHeader = errors.New("received nil beacon block header")

// ROBlob represents a read-only blob sidecar with its block root.
type ROBlob struct {
	*ethpb.BlobSidecar
	root [32]byte
}

// NewROBlobWithRoot creates a new ROBlob with a given root.
func NewROBlobWithRoot(b *ethpb.BlobSidecar, root [32]byte) (ROBlob, error) {
	if b == nil {
		return ROBlob{}, errNilBlock
	}
	return ROBlob{BlobSidecar: b, root: root}, nil
}

// NewROBlob creates a new ROBlob by computing the HashTreeRoot of the header.
func NewROBlob(b *ethpb.BlobSidecar) (ROBlob, error) {
	if b == nil {
		return ROBlob{}, errNilBlock
	}
	if b.SignedBlockHeader == nil || b.SignedBlockHeader.Header == nil {
		return ROBlob{}, errNilBlockHeader
	}
	root, err := b.SignedBlockHeader.Header.HashTreeRoot()
	if err != nil {
		return ROBlob{}, err
	}
	return ROBlob{BlobSidecar: b, root: root}, nil
}

// BlockRoot returns the root of the block.
func (b *ROBlob) BlockRoot() [32]byte {
	return b.root
}

// Slot returns the slot of the blob sidecar.
func (b *ROBlob) Slot() primitives.Slot {
	return b.SignedBlockHeader.Header.Slot
}

// ParentRoot returns the parent root of the blob sidecar.
func (b *ROBlob) ParentRoot() [32]byte {
	return bytesutil.ToBytes32(b.SignedBlockHeader.Header.ParentRoot)
}

// ParentRootSlice returns the parent root as a byte slice.
func (b *ROBlob) ParentRootSlice() []byte {
	return b.SignedBlockHeader.Header.ParentRoot
}

// BodyRoot returns the body root of the blob sidecar.
func (b *ROBlob) BodyRoot() [32]byte {
	return bytesutil.ToBytes32(b.SignedBlockHeader.Header.BodyRoot)
}

// ProposerIndex returns the proposer index of the blob sidecar.
func (b *ROBlob) ProposerIndex() primitives.ValidatorIndex {
	return b.SignedBlockHeader.Header.ProposerIndex
}

// BlockRootSlice returns the block root as a byte slice. This is often more convenient/concise
// than setting a tmp var to BlockRoot(), just so that it can be sliced.
func (b *ROBlob) BlockRootSlice() []byte {
	return b.root[:]
}

// ROBlobSlice is a custom type for a []ROBlob, allowing methods to be defined that act on a slice of ROBlob.
type ROBlobSlice []ROBlob

// Protos is a helper to make a more concise conversion from []ROBlob->[]*ethpb.BlobSidecar.
func (s ROBlobSlice) Protos() []*ethpb.BlobSidecar {
	pb := make([]*ethpb.BlobSidecar, len(s))
	for i := range s {
		pb[i] = s[i].BlobSidecar
	}
	return pb
}

// VerifiedROBlob represents an ROBlob that has undergone full verification (eg block sig, inclusion proof, commitment check).
type VerifiedROBlob struct {
	ROBlob
}

// NewVerifiedROBlob "upgrades" an ROBlob to a VerifiedROBlob. This method should only be used by the verification package.
func NewVerifiedROBlob(rob ROBlob) VerifiedROBlob {
	return VerifiedROBlob{ROBlob: rob}
}
