package scorers

import (
	"fmt"
	"math"
	"sort"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/peerdata"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
)

var _ Scorer = (*BlockProviderScorer)(nil)

const (
	// DefaultBlockProviderProcessedBatchWeight is a default reward weight of a processed batch of blocks.
	DefaultBlockProviderProcessedBatchWeight = float64(0.1)
	// DefaultBlockProviderProcessedBlocksCap defines default value for processed blocks cap.
	// e.g. 20 * 64 := 20 batches of size 64 (with 0.05 per batch reward, 20 batches result in score of 1.0).
	DefaultBlockProviderProcessedBlocksCap = uint64(10 * 64)
	// DefaultBlockProviderDecayInterval defines how often the decaying routine is called.
	DefaultBlockProviderDecayInterval = 30 * time.Second
	// DefaultBlockProviderDecay defines default blocks that are to be subtracted from stats on each
	// decay interval. Effectively, this param provides minimum expected performance for a peer to remain
	// high scorer.
	DefaultBlockProviderDecay = uint64(1 * 64)
	// DefaultBlockProviderStalePeerRefreshInterval defines default interval at which peers should be given
	// opportunity to provide blocks (their score gets boosted, up until they are selected for
	// fetching).
	DefaultBlockProviderStalePeerRefreshInterval = 5 * time.Minute
)

// BlockProviderScorer represents block provider scoring service.
type BlockProviderScorer struct {
	config *BlockProviderScorerConfig
	store  *peerdata.Store
	// maxScore is a cached value for maximum attainable block provider score.
	// It is calculated, on startup, as following: (processedBlocksCap / batchSize) * batchWeight.
	maxScore float64
}

// BlockProviderScorerConfig holds configuration parameters for block providers scoring service.
type BlockProviderScorerConfig struct {
	// ProcessedBatchWeight defines a reward for a single processed batch of blocks.
	ProcessedBatchWeight float64
	// ProcessedBlocksCap defines the highest number of processed blocks that are counted towards peer's score.
	// Once that cap is attained, peer is considered good to fetch from (and several peers having the
	// same score, are picked at random). To stay at max score, peer must continue to perform, as
	// stats decays quickly.
	ProcessedBlocksCap uint64
	// DecayInterval defines how often stats should be decayed.
	DecayInterval time.Duration
	// Decay specifies number of blocks subtracted from stats on each decay step.
	Decay uint64
	// StalePeerRefreshInterval is an interval at which peers should be given an opportunity
	// to provide blocks (scores are boosted to max up until such peers are selected).
	StalePeerRefreshInterval time.Duration
}

// newBlockProviderScorer creates block provider scoring service.
func newBlockProviderScorer(store *peerdata.Store, config *BlockProviderScorerConfig) *BlockProviderScorer {
	if config == nil {
		config = &BlockProviderScorerConfig{}
	}
	scorer := &BlockProviderScorer{
		config: config,
		store:  store,
	}
	if scorer.config.ProcessedBatchWeight == 0.0 {
		scorer.config.ProcessedBatchWeight = DefaultBlockProviderProcessedBatchWeight
	}
	if scorer.config.DecayInterval == 0 {
		scorer.config.DecayInterval = DefaultBlockProviderDecayInterval
	}
	if scorer.config.ProcessedBlocksCap == 0 {
		scorer.config.ProcessedBlocksCap = DefaultBlockProviderProcessedBlocksCap
	}
	if scorer.config.Decay == 0 {
		scorer.config.Decay = DefaultBlockProviderDecay
	}
	if scorer.config.StalePeerRefreshInterval == 0 {
		scorer.config.StalePeerRefreshInterval = DefaultBlockProviderStalePeerRefreshInterval
	}
	batchSize := uint64(flags.Get().BlockBatchLimit)
	scorer.maxScore = 1.0
	if batchSize > 0 {
		totalBatches := float64(scorer.config.ProcessedBlocksCap / batchSize)
		scorer.maxScore = totalBatches * scorer.config.ProcessedBatchWeight
		scorer.maxScore = math.Round(scorer.maxScore*ScoreRoundingFactor) / ScoreRoundingFactor
	}
	return scorer
}

// Score calculates and returns block provider score.
func (s *BlockProviderScorer) Score(pid peer.ID) float64 {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.score(pid)
}

// score is a lock-free version of Score.
func (s *BlockProviderScorer) score(pid peer.ID) float64 {
	score := float64(0)
	peerData, ok := s.store.PeerData(pid)
	// Boost score of new peers or peers that haven't been accessed for too long.
	if !ok || time.Since(peerData.BlockProviderUpdated) >= s.config.StalePeerRefreshInterval {
		return s.maxScore
	}
	batchSize := uint64(flags.Get().BlockBatchLimit)
	if batchSize > 0 {
		processedBatches := float64(peerData.ProcessedBlocks / batchSize)
		score += processedBatches * s.config.ProcessedBatchWeight
	}
	return math.Round(score*ScoreRoundingFactor) / ScoreRoundingFactor
}

// Params exposes scorer's parameters.
func (s *BlockProviderScorer) Params() *BlockProviderScorerConfig {
	return s.config
}

// IncrementProcessedBlocks increments the number of blocks that have been successfully processed.
func (s *BlockProviderScorer) IncrementProcessedBlocks(pid peer.ID, cnt uint64) {
	s.store.Lock()
	defer s.store.Unlock()
	defer s.touch(pid)

	if cnt <= 0 {
		return
	}
	peerData := s.store.PeerDataGetOrCreate(pid)
	if peerData.ProcessedBlocks+cnt > s.config.ProcessedBlocksCap {
		cnt = s.config.ProcessedBlocksCap - peerData.ProcessedBlocks
	}
	if cnt > 0 {
		peerData.ProcessedBlocks += cnt
	}
}

// Touch updates last access time for a given peer. This allows to detect peers that are
// stale and boost their scores to increase chances in block fetching participation.
func (s *BlockProviderScorer) Touch(pid peer.ID, t ...time.Time) {
	s.store.Lock()
	defer s.store.Unlock()
	s.touch(pid, t...)
}

// touch is a lock-free version of Touch.
func (s *BlockProviderScorer) touch(pid peer.ID, t ...time.Time) {
	peerData := s.store.PeerDataGetOrCreate(pid)
	if len(t) == 1 {
		peerData.BlockProviderUpdated = t[0]
	} else {
		peerData.BlockProviderUpdated = time.Now()
	}
}

// ProcessedBlocks returns number of peer returned blocks that are successfully processed.
func (s *BlockProviderScorer) ProcessedBlocks(pid peer.ID) uint64 {
	s.store.RLock()
	defer s.store.RUnlock()
	return s.processedBlocks(pid)
}

// processedBlocks is a lock-free version of ProcessedBlocks.
func (s *BlockProviderScorer) processedBlocks(pid peer.ID) uint64 {
	if peerData, ok := s.store.PeerData(pid); ok {
		return peerData.ProcessedBlocks
	}
	return 0
}

// IsBadPeer states if the peer is to be considered bad.
// Block provider scorer cannot guarantee that lower score of a peer is indeed a sign of a bad peer.
// Therefore this scorer never marks peers as bad, and relies on scores to probabilistically sort
// out low-scorers (see WeightSorted method).
func (_ *BlockProviderScorer) IsBadPeer(_ peer.ID) bool {
	return false
}

// BadPeers returns the peers that are considered bad.
// No peers are considered bad by block providers scorer.
func (_ *BlockProviderScorer) BadPeers() []peer.ID {
	return []peer.ID{}
}

// Decay updates block provider counters by decaying them.
// This urges peers to keep up the performance to continue getting a high score (and allows
// new peers to contest previously high scoring ones).
func (s *BlockProviderScorer) Decay() {
	s.store.Lock()
	defer s.store.Unlock()

	for _, peerData := range s.store.Peers() {
		if peerData.ProcessedBlocks > s.config.Decay {
			peerData.ProcessedBlocks -= s.config.Decay
		} else {
			peerData.ProcessedBlocks = 0
		}
	}
}

// WeightSorted returns a list of block providers weight sorted by score, where items are selected
// probabilistically with more "heavy" items having a higher chance of being picked.
func (s *BlockProviderScorer) WeightSorted(
	r *rand.Rand, pids []peer.ID, scoreFn func(pid peer.ID, score float64) float64,
) []peer.ID {
	if len(pids) == 0 {
		return pids
	}
	s.store.Lock()
	defer s.store.Unlock()

	// See http://eli.thegreenplace.net/2010/01/22/weighted-random-generation-in-python/ for details.
	nextPID := func(weights map[peer.ID]float64) peer.ID {
		totalWeight := 0
		for _, w := range weights {
			// Factor by 100, to allow weights in (0; 1) range.
			totalWeight += int(w * 100)
		}
		if totalWeight <= 0 {
			return ""
		}
		rnd := r.Intn(totalWeight)
		for pid, w := range weights {
			rnd -= int(w * 100)
			if rnd < 0 {
				return pid
			}
		}
		return ""
	}

	scores, _ := s.mapScoresAndPeers(pids, scoreFn)
	peers := make([]peer.ID, 0)
	for i := 0; i < len(pids); i++ {
		if pid := nextPID(scores); pid != "" {
			peers = append(peers, pid)
			delete(scores, pid)
		}
	}
	// Left over peers (like peers having zero weight), are added at the end of the list.
	for pid := range scores {
		peers = append(peers, pid)
	}

	return peers
}

// Sorted returns a list of block providers sorted by score in descending order.
// When custom scorer function is provided, items are returned in order provided by it.
func (s *BlockProviderScorer) Sorted(
	pids []peer.ID, scoreFn func(pid peer.ID, score float64) float64,
) []peer.ID {
	if len(pids) == 0 {
		return pids
	}
	s.store.Lock()
	defer s.store.Unlock()

	scores, peers := s.mapScoresAndPeers(pids, scoreFn)
	sort.Slice(peers, func(i, j int) bool {
		return scores[peers[i]] > scores[peers[j]]
	})
	return peers
}

// mapScoresAndPeers is a utility function to map peers and their respective scores (using custom
// scoring function if necessary).
func (s *BlockProviderScorer) mapScoresAndPeers(
	pids []peer.ID, scoreFn func(pid peer.ID, score float64) float64,
) (map[peer.ID]float64, []peer.ID) {
	scores := make(map[peer.ID]float64, len(pids))
	peers := make([]peer.ID, len(pids))
	for i, pid := range pids {
		if scoreFn != nil {
			scores[pid] = scoreFn(pid, s.score(pid))
		} else {
			scores[pid] = s.score(pid)
		}
		peers[i] = pid
	}
	return scores, peers
}

// FormatScorePretty returns full scoring information in a human-readable format.
func (s *BlockProviderScorer) FormatScorePretty(pid peer.ID) string {
	s.store.RLock()
	defer s.store.RUnlock()
	if !features.Get().EnablePeerScorer {
		return "disabled"
	}
	score := s.score(pid)
	return fmt.Sprintf("[%0.1f%%, raw: %0.2f,  blocks: %d/%d]",
		(score/s.MaxScore())*100, score, s.processedBlocks(pid), s.config.ProcessedBlocksCap)
}

// MaxScore exposes maximum score attainable by peers.
func (s *BlockProviderScorer) MaxScore() float64 {
	return s.maxScore
}
