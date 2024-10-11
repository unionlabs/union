package mock

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// PoolMock is a fake implementation of PoolManager.
type PoolMock struct {
	PendingAttSlashings  []*ethpb.AttesterSlashing
	PendingPropSlashings []*ethpb.ProposerSlashing
}

// PendingAttesterSlashings --
func (m *PoolMock) PendingAttesterSlashings(_ context.Context, _ state.ReadOnlyBeaconState, _ bool) []*ethpb.AttesterSlashing {
	return m.PendingAttSlashings
}

// PendingProposerSlashings --
func (m *PoolMock) PendingProposerSlashings(_ context.Context, _ state.ReadOnlyBeaconState, _ bool) []*ethpb.ProposerSlashing {
	return m.PendingPropSlashings
}

// InsertAttesterSlashing --
func (m *PoolMock) InsertAttesterSlashing(_ context.Context, _ state.ReadOnlyBeaconState, slashing *ethpb.AttesterSlashing) error {
	m.PendingAttSlashings = append(m.PendingAttSlashings, slashing)
	return nil
}

// InsertProposerSlashing --
func (m *PoolMock) InsertProposerSlashing(_ context.Context, _ state.ReadOnlyBeaconState, slashing *ethpb.ProposerSlashing) error {
	m.PendingPropSlashings = append(m.PendingPropSlashings, slashing)
	return nil
}

// MarkIncludedAttesterSlashing --
func (*PoolMock) MarkIncludedAttesterSlashing(_ *ethpb.AttesterSlashing) {
	panic("implement me")
}

// MarkIncludedProposerSlashing --
func (*PoolMock) MarkIncludedProposerSlashing(_ *ethpb.ProposerSlashing) {
	panic("implement me")
}
