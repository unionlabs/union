package scorers

import (
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
)

var _ Scorer = (*BadResponsesScorer)(nil)

const (
	// DefaultBadResponsesThreshold defines how many bad responses to tolerate before peer is deemed bad.
	DefaultBadResponsesThreshold = 6
	// DefaultBadResponsesDecayInterval defines how often to decay previous statistics.
	// Every interval bad responses counter will be decremented by 1.
	DefaultBadResponsesDecayInterval = time.Hour
	// DefaultBadResponsesPenaltyFactor defines the penalty factor applied to a peer based on their bad
	// response count.
	DefaultBadResponsesPenaltyFactor = 10
)

// BadResponsesScorer represents bad responses scoring service.
type BadResponsesScorer struct {
	config *BadResponsesScorerConfig
	store  *peerdata.Store
}

// BadResponsesScorerConfig holds configuration parameters for bad response scoring service.
type BadResponsesScorerConfig struct {
	// Threshold specifies number of bad responses tolerated, before peer is banned.
	Threshold int
	// DecayInterval specifies how often bad response stats should be decayed.
	DecayInterval time.Duration
}

// newBadResponsesScorer creates new bad responses scoring service.
func newBadResponsesScorer(store *peerdata.Store, config *BadResponsesScorerConfig) *BadResponsesScorer {
	if config == nil {
		config = &BadResponsesScorerConfig{}
	}
	scorer := &BadResponsesScorer{
		config: config,
		store:  store,
	}
	if scorer.config.Threshold == 0 {
		scorer.config.Threshold = DefaultBadResponsesThreshold
	}
	if scorer.config.DecayInterval == 0 {
		scorer.config.DecayInterval = DefaultBadResponsesDecayInterval
	}
	return scorer
}

// Score returns score (penalty) of bad responses peer produced.
func (s *BadResponsesScorer) Score(pid peer.ID) float64 {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.score(pid)
}

// score is a lock-free version of Score.
func (s *BadResponsesScorer) score(pid peer.ID) float64 {
	if s.isBadPeer(pid) {
		return BadPeerScore
	}
	score := float64(0)
	peerData, ok := s.store.PeerData(pid)
	if !ok {
		return score
	}
	if peerData.BadResponses > 0 {
		score = float64(peerData.BadResponses) / float64(s.config.Threshold)
		// Since score represents a penalty, negate it and multiply
		// it by a factor.
		score *= -DefaultBadResponsesPenaltyFactor
	}
	return score
}

// Params exposes scorer's parameters.
func (s *BadResponsesScorer) Params() *BadResponsesScorerConfig {
	return s.config
}

// Count obtains the number of bad responses we have received from the given remote peer.
func (s *BadResponsesScorer) Count(pid peer.ID) (int, error) {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.count(pid)
}

// count is a lock-free version of Count.
func (s *BadResponsesScorer) count(pid peer.ID) (int, error) {
	if peerData, ok := s.store.PeerData(pid); ok {
		return peerData.BadResponses, nil
	}
	return -1, peerdata.ErrPeerUnknown
}

// Increment increments the number of bad responses we have received from the given remote peer.
// If peer doesn't exist this method is no-op.
func (s *BadResponsesScorer) Increment(pid peer.ID) {
	s.store.Lock()
	defer s.store.Unlock()

	peerData, ok := s.store.PeerData(pid)
	if !ok {
		s.store.SetPeerData(pid, &peerdata.PeerData{
			BadResponses: 1,
		})
		return
	}
	peerData.BadResponses++
}

// IsBadPeer states if the peer is to be considered bad.
// If the peer is unknown this will return `false`, which makes using this function easier than returning an error.
func (s *BadResponsesScorer) IsBadPeer(pid peer.ID) bool {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.isBadPeer(pid)
}

// isBadPeer is lock-free version of IsBadPeer.
func (s *BadResponsesScorer) isBadPeer(pid peer.ID) bool {
	if peerData, ok := s.store.PeerData(pid); ok {
		return peerData.BadResponses >= s.config.Threshold
	}
	return false
}

// BadPeers returns the peers that are considered bad.
func (s *BadResponsesScorer) BadPeers() []peer.ID {
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

// Decay reduces the bad responses of all peers, giving reformed peers a chance to join the network.
// This can be run periodically, although note that each time it runs it does give all bad peers another chance as well
// to clog up the network with bad responses, so should not be run too frequently; once an hour would be reasonable.
func (s *BadResponsesScorer) Decay() {
	s.store.Lock()
	defer s.store.Unlock()

	for _, peerData := range s.store.Peers() {
		if peerData.BadResponses > 0 {
			peerData.BadResponses--
		}
	}
}
