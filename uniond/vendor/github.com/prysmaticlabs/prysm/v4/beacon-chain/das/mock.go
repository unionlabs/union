package das

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// MockAvailabilityStore is an implementation of AvailabilityStore that can be used by other packages in tests.
type MockAvailabilityStore struct {
	VerifyAvailabilityCallback func(ctx context.Context, current primitives.Slot, b blocks.ROBlock) error
	PersistBlobsCallback       func(current primitives.Slot, sc ...blocks.ROBlob) error
}

var _ AvailabilityStore = &MockAvailabilityStore{}

// IsDataAvailable satisfies the corresponding method of the AvailabilityStore interface in a way that is useful for tests.
func (m *MockAvailabilityStore) IsDataAvailable(ctx context.Context, current primitives.Slot, b blocks.ROBlock) error {
	if m.VerifyAvailabilityCallback != nil {
		return m.VerifyAvailabilityCallback(ctx, current, b)
	}
	return nil
}

// Persist satisfies the corresponding method of the AvailabilityStore interface in a way that is useful for tests.
func (m *MockAvailabilityStore) Persist(current primitives.Slot, sc ...blocks.ROBlob) error {
	if m.PersistBlobsCallback != nil {
		return m.PersistBlobsCallback(current, sc...)
	}
	return nil
}
