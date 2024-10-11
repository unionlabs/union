package scorers

import (
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

var _ Scorer = (*GossipScorer)(nil)

const (
	// The boundary till which a peer's gossip score is acceptable.
	gossipThreshold = -100.0
)

// GossipScorer represents scorer that evaluates peers based on their gossip performance.
// Gossip scoring metrics are periodically calculated in libp2p's internal pubsub module.
type GossipScorer struct {
	config *GossipScorerConfig
	store  *peerdata.Store
}

// GossipScorerConfig holds configuration parameters for gossip scoring service.
type GossipScorerConfig struct{}

// newGossipScorer creates new gossip scoring service.
func newGossipScorer(store *peerdata.Store, config *GossipScorerConfig) *GossipScorer {
	if config == nil {
		config = &GossipScorerConfig{}
	}
	return &GossipScorer{
		config: config,
		store:  store,
	}
}

// Score returns calculated peer score.
func (s *GossipScorer) Score(pid peer.ID) float64 {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.score(pid)
}

// score is a lock-free version of Score.
func (s *GossipScorer) score(pid peer.ID) float64 {
	peerData, ok := s.store.PeerData(pid)
	if !ok {
		return 0
	}
	return peerData.GossipScore
}

// IsBadPeer states if the peer is to be considered bad.
func (s *GossipScorer) IsBadPeer(pid peer.ID) bool {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.isBadPeer(pid)
}

// isBadPeer is lock-free version of IsBadPeer.
func (s *GossipScorer) isBadPeer(pid peer.ID) bool {
	peerData, ok := s.store.PeerData(pid)
	if !ok {
		return false
	}
	return peerData.GossipScore < gossipThreshold
}

// BadPeers returns the peers that are considered bad.
func (s *GossipScorer) BadPeers() []peer.ID {
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

// SetGossipData sets the gossip related data of a peer.
func (s *GossipScorer) SetGossipData(pid peer.ID, gScore float64,
	bPenalty float64, topicScores map[string]*pbrpc.TopicScoreSnapshot) {
	s.store.Lock()
	defer s.store.Unlock()

	peerData := s.store.PeerDataGetOrCreate(pid)
	peerData.GossipScore = gScore
	peerData.BehaviourPenalty = bPenalty
	peerData.TopicScores = topicScores
}

// GossipData gets the gossip related information of the given remote peer.
// This can return nil if there is no known gossip record the peer.
// This will error if the peer does not exist.
func (s *GossipScorer) GossipData(pid peer.ID) (float64, float64, map[string]*pbrpc.TopicScoreSnapshot, error) {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.gossipData(pid)
}

// gossipData lock-free version of GossipData.
func (s *GossipScorer) gossipData(pid peer.ID) (float64, float64, map[string]*pbrpc.TopicScoreSnapshot, error) {
	if peerData, ok := s.store.PeerData(pid); ok {
		return peerData.GossipScore, peerData.BehaviourPenalty, peerData.TopicScores, nil
	}
	return 0, 0, nil, peerdata.ErrPeerUnknown
}
