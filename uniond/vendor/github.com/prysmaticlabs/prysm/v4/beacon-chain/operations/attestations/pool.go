package attestations

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations/kv"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// Pool defines the necessary methods for Prysm attestations pool to serve
// fork choice and validators. In the current design, aggregated attestations
// are used by proposer actor. Unaggregated attestations are used by
// aggregator actor.
type Pool interface {
	// For Aggregated attestations
	AggregateUnaggregatedAttestations(ctx context.Context) error
	SaveAggregatedAttestation(att *ethpb.Attestation) error
	SaveAggregatedAttestations(atts []*ethpb.Attestation) error
	AggregatedAttestations() []*ethpb.Attestation
	AggregatedAttestationsBySlotIndex(ctx context.Context, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) []*ethpb.Attestation
	DeleteAggregatedAttestation(att *ethpb.Attestation) error
	HasAggregatedAttestation(att *ethpb.Attestation) (bool, error)
	AggregatedAttestationCount() int
	// For unaggregated attestations.
	SaveUnaggregatedAttestation(att *ethpb.Attestation) error
	SaveUnaggregatedAttestations(atts []*ethpb.Attestation) error
	UnaggregatedAttestations() ([]*ethpb.Attestation, error)
	UnaggregatedAttestationsBySlotIndex(ctx context.Context, slot primitives.Slot, committeeIndex primitives.CommitteeIndex) []*ethpb.Attestation
	DeleteUnaggregatedAttestation(att *ethpb.Attestation) error
	DeleteSeenUnaggregatedAttestations() (int, error)
	UnaggregatedAttestationCount() int
	// For attestations that were included in the block.
	SaveBlockAttestation(att *ethpb.Attestation) error
	BlockAttestations() []*ethpb.Attestation
	DeleteBlockAttestation(att *ethpb.Attestation) error
	// For attestations to be passed to fork choice.
	SaveForkchoiceAttestation(att *ethpb.Attestation) error
	SaveForkchoiceAttestations(atts []*ethpb.Attestation) error
	ForkchoiceAttestations() []*ethpb.Attestation
	DeleteForkchoiceAttestation(att *ethpb.Attestation) error
	ForkchoiceAttestationCount() int
}

// NewPool initializes a new attestation pool.
func NewPool() *kv.AttCaches {
	return kv.NewAttCaches()
}
