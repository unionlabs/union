package mock

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// PoolMock is a fake implementation of PoolManager.
type PoolMock struct {
	Changes []*eth.SignedBLSToExecutionChange
}

// PendingBLSToExecChanges --
func (m *PoolMock) PendingBLSToExecChanges() ([]*eth.SignedBLSToExecutionChange, error) {
	return m.Changes, nil
}

// BLSToExecChangesForInclusion --
func (m *PoolMock) BLSToExecChangesForInclusion(_ state.ReadOnlyBeaconState) ([]*eth.SignedBLSToExecutionChange, error) {
	return m.Changes, nil
}

// InsertBLSToExecChange --
func (m *PoolMock) InsertBLSToExecChange(change *eth.SignedBLSToExecutionChange) {
	m.Changes = append(m.Changes, change)
}

// MarkIncluded --
func (*PoolMock) MarkIncluded(_ *eth.SignedBLSToExecutionChange) {
	panic("implement me")
}

// ValidatorExists --
func (*PoolMock) ValidatorExists(_ primitives.ValidatorIndex) bool {
	panic("implement me")
}
