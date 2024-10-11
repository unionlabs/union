package verification

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
)

type MockBlobVerifier struct {
	ErrBlobIndexInBounds            error
	ErrSlotTooEarly                 error
	ErrSlotAboveFinalized           error
	ErrValidProposerSignature       error
	ErrSidecarParentSeen            error
	ErrSidecarParentValid           error
	ErrSidecarParentSlotLower       error
	ErrSidecarDescendsFromFinalized error
	ErrSidecarInclusionProven       error
	ErrSidecarKzgProofVerified      error
	ErrSidecarProposerExpected      error
	cbVerifiedROBlob                func() (blocks.VerifiedROBlob, error)
}

func (m *MockBlobVerifier) VerifiedROBlob() (blocks.VerifiedROBlob, error) {
	return m.cbVerifiedROBlob()
}

func (m *MockBlobVerifier) BlobIndexInBounds() (err error) {
	return m.ErrBlobIndexInBounds
}

func (m *MockBlobVerifier) NotFromFutureSlot() (err error) {
	return m.ErrSlotTooEarly
}

func (m *MockBlobVerifier) SlotAboveFinalized() (err error) {
	return m.ErrSlotAboveFinalized
}

func (m *MockBlobVerifier) ValidProposerSignature(_ context.Context) (err error) {
	return m.ErrValidProposerSignature
}

func (m *MockBlobVerifier) SidecarParentSeen(_ func([32]byte) bool) (err error) {
	return m.ErrSidecarParentSeen
}

func (m *MockBlobVerifier) SidecarParentValid(_ func([32]byte) bool) (err error) {
	return m.ErrSidecarParentValid
}

func (m *MockBlobVerifier) SidecarParentSlotLower() (err error) {
	return m.ErrSidecarParentSlotLower
}

func (m *MockBlobVerifier) SidecarDescendsFromFinalized() (err error) {
	return m.ErrSidecarDescendsFromFinalized
}

func (m *MockBlobVerifier) SidecarInclusionProven() (err error) {
	return m.ErrSidecarInclusionProven
}

func (m *MockBlobVerifier) SidecarKzgProofVerified() (err error) {
	return m.ErrSidecarKzgProofVerified
}

func (m *MockBlobVerifier) SidecarProposerExpected(_ context.Context) (err error) {
	return m.ErrSidecarProposerExpected
}

func (*MockBlobVerifier) SatisfyRequirement(_ Requirement) {}

var _ BlobVerifier = &MockBlobVerifier{}
