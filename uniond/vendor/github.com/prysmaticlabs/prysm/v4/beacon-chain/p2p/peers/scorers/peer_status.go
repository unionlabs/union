package scorers

import (
	"errors"
	"math"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var _ Scorer = (*PeerStatusScorer)(nil)

// PeerStatusScorer represents scorer that evaluates peers based on their statuses.
// Peer statuses are updated by regularly polling peers (see sync/rpc_status.go).
type PeerStatusScorer struct {
	config              *PeerStatusScorerConfig
	store               *peerdata.Store
	ourHeadSlot         primitives.Slot
	highestPeerHeadSlot primitives.Slot
}

// PeerStatusScorerConfig holds configuration parameters for peer status scoring service.
type PeerStatusScorerConfig struct{}

// newPeerStatusScorer creates new peer status scoring service.
func newPeerStatusScorer(store *peerdata.Store, config *PeerStatusScorerConfig) *PeerStatusScorer {
	if config == nil {
		config = &PeerStatusScorerConfig{}
	}
	return &PeerStatusScorer{
		config: config,
		store:  store,
	}
}

// Score returns calculated peer score.
func (s *PeerStatusScorer) Score(pid peer.ID) float64 {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.score(pid)
}

// score is a lock-free version of Score.
func (s *PeerStatusScorer) score(pid peer.ID) float64 {
	if s.isBadPeer(pid) {
		return BadPeerScore
	}
	score := float64(0)
	peerData, ok := s.store.PeerData(pid)
	if !ok || peerData.ChainState == nil {
		return score
	}
	if peerData.ChainState.HeadSlot < s.ourHeadSlot {
		return score
	}
	// Calculate score as a ratio to the known maximum head slot.
	// The closer the current peer's head slot to the maximum, the higher is the calculated score.
	if s.highestPeerHeadSlot > 0 {
		score = float64(peerData.ChainState.HeadSlot) / float64(s.highestPeerHeadSlot)
		return math.Round(score*ScoreRoundingFactor) / ScoreRoundingFactor
	}
	return score
}

// IsBadPeer states if the peer is to be considered bad.
func (s *PeerStatusScorer) IsBadPeer(pid peer.ID) bool {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.isBadPeer(pid)
}

// isBadPeer is lock-free version of IsBadPeer.
func (s *PeerStatusScorer) isBadPeer(pid peer.ID) bool {
	peerData, ok := s.store.PeerData(pid)
	if !ok {
		return false
	}
	// Mark peer as bad, if the latest error is one of the terminal ones.
	terminalErrs := []error{
		p2ptypes.ErrWrongForkDigestVersion,
		p2ptypes.ErrInvalidFinalizedRoot,
		p2ptypes.ErrInvalidRequest,
	}
	for _, err := range terminalErrs {
		if errors.Is(peerData.ChainStateValidationError, err) {
			return true
		}
	}
	return false
}

// BadPeers returns the peers that are considered bad.
func (s *PeerStatusScorer) BadPeers() []peer.ID {
	s.store.RLock()
	defer s.store.RUnlock()

	badPeers := make([]peer.ID, 0)
	for pid := range s.store.Peers() {
		if s.isBadPeer(pid) {
			badPeers = append(badPeers, pid)
		}
	}
	return badPeers
}

// SetPeerStatus sets chain state data for a given peer.
func (s *PeerStatusScorer) SetPeerStatus(pid peer.ID, chainState *pb.Status, validationError error) {
	s.store.Lock()
	defer s.store.Unlock()

	peerData := s.store.PeerDataGetOrCreate(pid)
	peerData.ChainState = chainState
	peerData.ChainStateLastUpdated = time.Now()
	peerData.ChainStateValidationError = validationError

	// Update maximum known head slot (scores will be calculated with respect to that maximum value).
	if chainState != nil && chainState.HeadSlot > s.highestPeerHeadSlot {
		s.highestPeerHeadSlot = chainState.HeadSlot
	}
}

// PeerStatus gets the chain state of the given remote peer.
// This can return nil if there is no known chain state for the peer.
// This will error if the peer does not exist.
func (s *PeerStatusScorer) PeerStatus(pid peer.ID) (*pb.Status, error) {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.peerStatus(pid)
}

// peerStatus lock-free version of PeerStatus.
func (s *PeerStatusScorer) peerStatus(pid peer.ID) (*pb.Status, error) {
	if peerData, ok := s.store.PeerData(pid); ok {
		if peerData.ChainState == nil {
			return nil, peerdata.ErrNoPeerStatus
		}
		return peerData.ChainState, nil
	}
	return nil, peerdata.ErrPeerUnknown
}

// SetHeadSlot updates known head slot.
func (s *PeerStatusScorer) SetHeadSlot(slot primitives.Slot) {
	s.store.Lock()
	defer s.store.Unlock()
	s.ourHeadSlot = slot
}
