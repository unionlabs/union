package mock

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// StateManager is a fake implementation of StateManager.
type StateManager struct {
	StatesByRoot map[[32]byte]state.BeaconState
	StatesBySlot map[primitives.Slot]state.BeaconState
}

// NewService --
func NewService() *StateManager {
	return &StateManager{
		StatesByRoot: make(map[[32]byte]state.BeaconState),
		StatesBySlot: make(map[primitives.Slot]state.BeaconState),
	}
}

// StateByRootIfCachedNoCopy --
func (_ *StateManager) StateByRootIfCachedNoCopy(_ [32]byte) state.BeaconState {
	panic("implement me")
}

// Resume --
func (_ *StateManager) Resume(_ context.Context, _ state.BeaconState) (state.BeaconState, error) {
	panic("implement me")
}

// SaveFinalizedState --
func (_ *StateManager) SaveFinalizedState(_ primitives.Slot, _ [32]byte, _ state.BeaconState) {
	panic("implement me")
}

// MigrateToCold --
func (_ *StateManager) MigrateToCold(_ context.Context, _ [32]byte) error {
	panic("implement me")
}

// HasState --
func (_ *StateManager) HasState(_ context.Context, _ [32]byte) (bool, error) {
	panic("implement me")
}

// StateByRoot --
func (m *StateManager) StateByRoot(_ context.Context, blockRoot [32]byte) (state.BeaconState, error) {
	return m.StatesByRoot[blockRoot], nil
}

// ActiveNonSlashedBalancesByRoot --
func (*StateManager) ActiveNonSlashedBalancesByRoot(_ context.Context, _ [32]byte) ([]uint64, error) {
	return []uint64{}, nil
}

// StateByRootInitialSync --
func (_ *StateManager) StateByRootInitialSync(_ context.Context, _ [32]byte) (state.BeaconState, error) {
	panic("implement me")
}

// StateBySlot --
func (m *StateManager) StateBySlot(_ context.Context, slot primitives.Slot) (state.BeaconState, error) {
	return m.StatesBySlot[slot], nil
}

// SaveState --
func (_ *StateManager) SaveState(_ context.Context, _ [32]byte, _ state.BeaconState) error {
	panic("implement me")
}

// ForceCheckpoint --
func (_ *StateManager) ForceCheckpoint(_ context.Context, _ []byte) error {
	panic("implement me")
}

// EnableSaveHotStateToDB --
func (_ *StateManager) EnableSaveHotStateToDB(_ context.Context) {
	panic("implement me")
}

// DisableSaveHotStateToDB --
func (_ *StateManager) DisableSaveHotStateToDB(_ context.Context) error {
	panic("implement me")
}

// AddStateForRoot --
func (m *StateManager) AddStateForRoot(state state.BeaconState, blockRoot [32]byte) {
	m.StatesByRoot[blockRoot] = state
}

// AddStateForSlot --
func (m *StateManager) AddStateForSlot(state state.BeaconState, slot primitives.Slot) {
	m.StatesBySlot[slot] = state
}

// DeleteStateFromCaches --
func (m *StateManager) DeleteStateFromCaches(context.Context, [32]byte) error {
	return nil
}
