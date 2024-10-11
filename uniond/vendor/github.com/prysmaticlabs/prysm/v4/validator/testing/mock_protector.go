package testing

import (
	"context"

	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// MockProtector mocks the protector.
type MockProtector struct {
	AllowAttestation        bool
	AllowBlock              bool
	VerifyAttestationCalled bool
	VerifyBlockCalled       bool
	StatusCalled            bool
}

// CheckAttestationSafety returns bool with allow attestation value.
func (mp *MockProtector) CheckAttestationSafety(_ context.Context, _ *eth.IndexedAttestation) bool {
	mp.VerifyAttestationCalled = true
	return mp.AllowAttestation
}

// CheckBlockSafety returns bool with allow block value.
func (mp *MockProtector) CheckBlockSafety(_ context.Context, _ *eth.SignedBeaconBlockHeader) bool {
	mp.VerifyBlockCalled = true
	return mp.AllowBlock
}

// Status returns nil.
func (mp *MockProtector) Status() error {
	mp.StatusCalled = true
	return nil
}
