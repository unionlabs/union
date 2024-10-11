package verification

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
)

// BlobVerifier defines the methods implemented by the ROBlobVerifier.
// It is mainly intended to make mocks and tests more straightforward, and to deal
// with the awkwardness of mocking a concrete type that returns a concrete type
// in tests outside of this package.
type BlobVerifier interface {
	VerifiedROBlob() (blocks.VerifiedROBlob, error)
	BlobIndexInBounds() (err error)
	NotFromFutureSlot() (err error)
	SlotAboveFinalized() (err error)
	ValidProposerSignature(ctx context.Context) (err error)
	SidecarParentSeen(parentSeen func([32]byte) bool) (err error)
	SidecarParentValid(badParent func([32]byte) bool) (err error)
	SidecarParentSlotLower() (err error)
	SidecarDescendsFromFinalized() (err error)
	SidecarInclusionProven() (err error)
	SidecarKzgProofVerified() (err error)
	SidecarProposerExpected(ctx context.Context) (err error)
	SatisfyRequirement(Requirement)
}

// NewBlobVerifier is a function signature that can be used by code that needs to be
// able to mock Initializer.NewBlobVerifier without complex setup.
type NewBlobVerifier func(b blocks.ROBlob, reqs []Requirement) BlobVerifier
