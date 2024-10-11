package mock

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// PoolMock --
type PoolMock struct {
	AggregatedAtts []*ethpb.Attestation
}

// AggregateUnaggregatedAttestations --
func (*PoolMock) AggregateUnaggregatedAttestations(_ context.Context) error {
	panic("implement me")
}

// AggregateUnaggregatedAttestationsBySlotIndex --
func (*PoolMock) AggregateUnaggregatedAttestationsBySlotIndex(_ context.Context, _ primitives.Slot, _ primitives.CommitteeIndex) error {
	panic("implement me")
}

// SaveAggregatedAttestation --
func (*PoolMock) SaveAggregatedAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// SaveAggregatedAttestations --
func (m *PoolMock) SaveAggregatedAttestations(atts []*ethpb.Attestation) error {
	m.AggregatedAtts = append(m.AggregatedAtts, atts...)
	return nil
}

// AggregatedAttestations --
func (m *PoolMock) AggregatedAttestations() []*ethpb.Attestation {
	return m.AggregatedAtts
}

// AggregatedAttestationsBySlotIndex --
func (*PoolMock) AggregatedAttestationsBySlotIndex(_ context.Context, _ primitives.Slot, _ primitives.CommitteeIndex) []*ethpb.Attestation {
	panic("implement me")
}

// DeleteAggregatedAttestation --
func (*PoolMock) DeleteAggregatedAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// HasAggregatedAttestation --
func (*PoolMock) HasAggregatedAttestation(_ *ethpb.Attestation) (bool, error) {
	panic("implement me")
}

// AggregatedAttestationCount --
func (*PoolMock) AggregatedAttestationCount() int {
	panic("implement me")
}

// SaveUnaggregatedAttestation --
func (*PoolMock) SaveUnaggregatedAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// SaveUnaggregatedAttestations --
func (*PoolMock) SaveUnaggregatedAttestations(_ []*ethpb.Attestation) error {
	panic("implement me")
}

// UnaggregatedAttestations --
func (*PoolMock) UnaggregatedAttestations() ([]*ethpb.Attestation, error) {
	panic("implement me")
}

// UnaggregatedAttestationsBySlotIndex --
func (*PoolMock) UnaggregatedAttestationsBySlotIndex(_ context.Context, _ primitives.Slot, _ primitives.CommitteeIndex) []*ethpb.Attestation {
	panic("implement me")
}

// DeleteUnaggregatedAttestation --
func (*PoolMock) DeleteUnaggregatedAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// DeleteSeenUnaggregatedAttestations --
func (*PoolMock) DeleteSeenUnaggregatedAttestations() (int, error) {
	panic("implement me")
}

// UnaggregatedAttestationCount --
func (*PoolMock) UnaggregatedAttestationCount() int {
	panic("implement me")
}

// SaveBlockAttestation --
func (*PoolMock) SaveBlockAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// SaveBlockAttestations --
func (*PoolMock) SaveBlockAttestations(_ []*ethpb.Attestation) error {
	panic("implement me")
}

// BlockAttestations --
func (*PoolMock) BlockAttestations() []*ethpb.Attestation {
	panic("implement me")
}

// DeleteBlockAttestation --
func (*PoolMock) DeleteBlockAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// SaveForkchoiceAttestation --
func (*PoolMock) SaveForkchoiceAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// SaveForkchoiceAttestations --
func (*PoolMock) SaveForkchoiceAttestations(_ []*ethpb.Attestation) error {
	panic("implement me")
}

// ForkchoiceAttestations --
func (*PoolMock) ForkchoiceAttestations() []*ethpb.Attestation {
	panic("implement me")
}

// DeleteForkchoiceAttestation --
func (*PoolMock) DeleteForkchoiceAttestation(_ *ethpb.Attestation) error {
	panic("implement me")
}

// ForkchoiceAttestationCount --
func (*PoolMock) ForkchoiceAttestationCount() int {
	panic("implement me")
}
