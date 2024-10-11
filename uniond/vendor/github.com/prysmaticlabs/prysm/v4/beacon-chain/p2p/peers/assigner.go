package peers

import (
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/sirupsen/logrus"
)

// FinalizedCheckpointer describes the minimum capability that Assigner needs from forkchoice.
// That is, the ability to retrieve the latest finalized checkpoint to help with peer evaluation.
type FinalizedCheckpointer interface {
	FinalizedCheckpoint() *forkchoicetypes.Checkpoint
}

// NewAssigner assists in the correct construction of an Assigner by code in other packages,
// assuring all the important private member fields are given values.
// The FinalizedCheckpointer is used to retrieve the latest finalized checkpoint each time peers are requested.
// Peers that report an older finalized checkpoint are filtered out.
func NewAssigner(s *Status, fc FinalizedCheckpointer) *Assigner {
	return &Assigner{
		ps: s,
		fc: fc,
	}
}

// Assigner uses the "BestFinalized" peer scoring method to pick the next-best peer to receive rpc requests.
type Assigner struct {
	ps *Status
	fc FinalizedCheckpointer
}

// ErrInsufficientSuitable is a sentinel error, signaling that a peer couldn't be assigned because there are currently
// not enough peers that match our selection criteria to serve rpc requests. It is the responsibility of the caller to
// look for this error and continue to try calling Assign with appropriate backoff logic.
var ErrInsufficientSuitable = errors.New("no suitable peers")

func (a *Assigner) freshPeers() ([]peer.ID, error) {
	required := params.BeaconConfig().MaxPeersToSync
	if flags.Get().MinimumSyncPeers < required {
		required = flags.Get().MinimumSyncPeers
	}
	_, peers := a.ps.BestFinalized(params.BeaconConfig().MaxPeersToSync, a.fc.FinalizedCheckpoint().Epoch)
	if len(peers) < required {
		log.WithFields(logrus.Fields{
			"suitable": len(peers),
			"required": required}).Warn("Unable to assign peer while suitable peers < required ")
		return nil, ErrInsufficientSuitable
	}
	return peers, nil
}

// Assign uses the "BestFinalized" method to select the best peers that agree on a canonical block
// for the configured finalized epoch. At most `n` peers will be returned. The `busy` param can be used
// to filter out peers that we know we don't want to connect to, for instance if we are trying to limit
// the number of outbound requests to each peer from a given component.
func (a *Assigner) Assign(busy map[peer.ID]bool, n int) ([]peer.ID, error) {
	best, err := a.freshPeers()
	if err != nil {
		return nil, err
	}
	return pickBest(busy, n, best), nil
}

func pickBest(busy map[peer.ID]bool, n int, best []peer.ID) []peer.ID {
	ps := make([]peer.ID, 0, n)
	for _, p := range best {
		if len(ps) == n {
			return ps
		}
		if !busy[p] {
			ps = append(ps, p)
		}
	}
	return ps
}
