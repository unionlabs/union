package synccommittee

import (
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var _ = Pool(&Store{})

// Pool defines the necessary methods for Prysm sync pool to serve
// validators. In the current design, aggregated attestations
// are used by proposers and sync committee messages are used by
// sync aggregators.
type Pool interface {
	// Methods for Sync Contributions.
	SaveSyncCommitteeContribution(contr *ethpb.SyncCommitteeContribution) error
	SyncCommitteeContributions(slot primitives.Slot) ([]*ethpb.SyncCommitteeContribution, error)

	// Methods for Sync Committee Messages.
	SaveSyncCommitteeMessage(sig *ethpb.SyncCommitteeMessage) error
	SyncCommitteeMessages(slot primitives.Slot) ([]*ethpb.SyncCommitteeMessage, error)
}

// NewPool returns the sync committee store fulfilling the pool interface.
func NewPool() Pool {
	return NewStore()
}
