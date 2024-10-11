package scorers_test

import (
	"context"
	"fmt"
	"sort"
	"strconv"
	"testing"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/time"
)

func TestScorers_BlockProvider_Score(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	batchSize := uint64(flags.Get().BlockBatchLimit)
	tests := []struct {
		name   string
		update func(scorer *scorers.BlockProviderScorer)
		check  func(scorer *scorers.BlockProviderScorer)
	}{
		{
			name: "nonexistent peer",
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, scorer.MaxScore(), scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "existent peer with zero score",
			update: func(scorer *scorers.BlockProviderScorer) {
				scorer.Touch("peer1")
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, 0.0, scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "existent peer via increment",
			update: func(scorer *scorers.BlockProviderScorer) {
				scorer.IncrementProcessedBlocks("peer1", 0)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, 0.0, scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "boost score of stale peer",
			update: func(scorer *scorers.BlockProviderScorer) {
				batchWeight := scorer.Params().ProcessedBatchWeight
				scorer.IncrementProcessedBlocks("peer1", batchSize*3)
				assert.Equal(t, roundScore(batchWeight*3), scorer.Score("peer1"), "Unexpected score")
				scorer.Touch("peer1", time.Now().Add(-1*scorer.Params().StalePeerRefreshInterval))
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, scorer.MaxScore(), scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "increment with 0 score",
			update: func(scorer *scorers.BlockProviderScorer) {
				// Increment to zero (provider is added to cache but score is unchanged).
				scorer.IncrementProcessedBlocks("peer1", 0)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, 0.0, scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "partial score",
			update: func(scorer *scorers.BlockProviderScorer) {
				// Partial score (less than a single batch of blocks processed).
				scorer.IncrementProcessedBlocks("peer1", batchSize/2)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, 0.0, scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "single batch",
			update: func(scorer *scorers.BlockProviderScorer) {
				scorer.IncrementProcessedBlocks("peer1", batchSize)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				batchWeight := scorer.Params().ProcessedBatchWeight
				assert.Equal(t, roundScore(batchWeight), scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "multiple batches",
			update: func(scorer *scorers.BlockProviderScorer) {
				scorer.IncrementProcessedBlocks("peer1", batchSize*7)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				batchWeight := scorer.Params().ProcessedBatchWeight
				assert.Equal(t, roundScore(batchWeight*7), scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "maximum score cap",
			update: func(scorer *scorers.BlockProviderScorer) {
				batchWeight := scorer.Params().ProcessedBatchWeight
				scorer.IncrementProcessedBlocks("peer1", batchSize*2)
				assert.Equal(t, roundScore(batchWeight*2), scorer.Score("peer1"), "Unexpected score")
				scorer.IncrementProcessedBlocks("peer1", scorer.Params().ProcessedBlocksCap)
			},
			check: func(scorer *scorers.BlockProviderScorer) {
				assert.Equal(t, scorer.Params().ProcessedBlocksCap, scorer.ProcessedBlocks("peer1"))
				assert.Equal(t, 1.0, scorer.Score("peer1"))
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
				PeerLimit: 30,
				ScorerParams: &scorers.Config{
					BlockProviderScorerConfig: &scorers.BlockProviderScorerConfig{},
				},
			})
			scorer := peerStatuses.Scorers().BlockProviderScorer()
			if tt.update != nil {
				tt.update(scorer)
			}
			tt.check(scorer)
		})
	}
}

func TestScorers_BlockProvider_GettersSetters(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
		ScorerParams: &scorers.Config{},
	})
	scorer := peerStatuses.Scorers().BlockProviderScorer()

	assert.Equal(t, uint64(0), scorer.ProcessedBlocks("peer1"), "Unexpected count for unregistered peer")
	scorer.IncrementProcessedBlocks("peer1", 64)
	assert.Equal(t, uint64(64), scorer.ProcessedBlocks("peer1"))
}

func TestScorers_BlockProvider_WeightSorted(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
		ScorerParams: &scorers.Config{
			BlockProviderScorerConfig: &scorers.BlockProviderScorerConfig{
				ProcessedBatchWeight: 0.01,
			},
		},
	})
	scorer := peerStatuses.Scorers().BlockProviderScorer()
	batchSize := uint64(flags.Get().BlockBatchLimit)
	r := rand.NewDeterministicGenerator()

	reverse := func(pids []peer.ID) []peer.ID {
		tmp := make([]peer.ID, len(pids))
		copy(tmp, pids)
		for i, j := 0, len(tmp)-1; i < j; i, j = i+1, j-1 {
			tmp[i], tmp[j] = tmp[j], tmp[i]
		}
		return tmp
	}

	shuffle := func(pids []peer.ID) []peer.ID {
		tmp := make([]peer.ID, len(pids))
		copy(tmp, pids)
		r.Shuffle(len(tmp), func(i, j int) {
			tmp[i], tmp[j] = tmp[j], tmp[i]
		})
		return tmp
	}

	var pids []peer.ID
	for i := uint64(0); i < 10; i++ {
		pid := peer.ID(strconv.FormatUint(i, 10))
		scorer.IncrementProcessedBlocks(pid, i*batchSize)
		pids = append(pids, pid)
	}
	// Make sure that peers scores are correct (peer(n).score > peer(n-1).score).
	// Peers should be returned in descending order (by score).
	assert.DeepEqual(t, reverse(pids), scorer.Sorted(pids, nil))

	// Run weighted sort lots of time, to get accurate statistics of whether more heavy items
	// are indeed preferred when sorting.
	scores := make(map[peer.ID]int, len(pids))
	for i := 0; i < 1000; i++ {
		score := len(pids) - 1
		// The earlier in the list the item is, the more of a score will it get.
		for _, pid := range scorer.WeightSorted(r, shuffle(pids), nil) {
			scores[pid] += score
			score--
		}
	}
	var scoredPIDs []peer.ID
	for pid := range scores {
		scoredPIDs = append(scoredPIDs, pid)
	}
	sort.Slice(scoredPIDs, func(i, j int) bool {
		return scores[scoredPIDs[i]] > scores[scoredPIDs[j]]
	})
	assert.Equal(t, len(pids), len(scoredPIDs))
	assert.DeepEqual(t, reverse(pids), scoredPIDs, "Expected items with more weight to be picked more often")
}

func TestScorers_BlockProvider_Sorted(t *testing.T) {
	batchSize := uint64(flags.Get().BlockBatchLimit)
	tests := []struct {
		name   string
		update func(s *scorers.BlockProviderScorer)
		score  func(pid peer.ID, score float64) float64
		have   []peer.ID
		want   []peer.ID
	}{
		{
			name:   "no peers",
			update: func(s *scorers.BlockProviderScorer) {},
			have:   []peer.ID{},
			want:   []peer.ID{},
		},
		{
			name: "same scores",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", 16)
				s.IncrementProcessedBlocks("peer2", 16)
				s.IncrementProcessedBlocks("peer3", 16)
			},
			have: []peer.ID{"peer1", "peer2", "peer3"},
			want: []peer.ID{"peer1", "peer2", "peer3"},
		},
		{
			name: "same scores multiple batches",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*7+16)
				s.IncrementProcessedBlocks("peer2", batchSize*7+16)
				s.IncrementProcessedBlocks("peer3", batchSize*7+16)
			},
			have: []peer.ID{"peer1", "peer2", "peer3"},
			want: []peer.ID{"peer1", "peer2", "peer3"},
		},
		{
			name: "same scores multiple batches unequal blocks",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*7+6)
				s.IncrementProcessedBlocks("peer2", batchSize*7+16)
				s.IncrementProcessedBlocks("peer3", batchSize*7+26)
			},
			have: []peer.ID{"peer1", "peer2", "peer3"},
			want: []peer.ID{"peer1", "peer2", "peer3"},
		},
		{
			name: "different scores",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*3)
				s.IncrementProcessedBlocks("peer2", batchSize*1)
				s.IncrementProcessedBlocks("peer3", batchSize*2)
			},
			have: []peer.ID{"peer3", "peer2", "peer1"},
			want: []peer.ID{"peer1", "peer3", "peer2"},
		},
		{
			name: "custom scorer",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*3)
				s.IncrementProcessedBlocks("peer2", batchSize*1)
				s.IncrementProcessedBlocks("peer3", batchSize*2)
			},
			score: func(pid peer.ID, score float64) float64 {
				if pid == "peer2" {
					return score + 0.3 // 0.2 + 0.3 = 0.5 > 0.4 (of peer3)
				}
				if pid == "peer1" {
					return 0.0
				}
				return score
			},
			have: []peer.ID{"peer3", "peer2", "peer1"},
			want: []peer.ID{"peer2", "peer3", "peer1"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
				ScorerParams: &scorers.Config{
					BlockProviderScorerConfig: &scorers.BlockProviderScorerConfig{
						ProcessedBatchWeight: 0.2,
					},
				},
			})
			scorer := peerStatuses.Scorers().BlockProviderScorer()
			tt.update(scorer)
			assert.DeepEqual(t, tt.want, scorer.Sorted(tt.have, tt.score))
		})
	}
}

func TestScorers_BlockProvider_MaxScore(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	batchSize := uint64(flags.Get().BlockBatchLimit)

	tests := []struct {
		name string
		cfg  *scorers.BlockProviderScorerConfig
		want float64
	}{
		{
			name: "default config",
			cfg:  &scorers.BlockProviderScorerConfig{},
			want: 1.0,
		},
		{
			name: "custom config",
			cfg: &scorers.BlockProviderScorerConfig{
				ProcessedBatchWeight: 0.5,
				ProcessedBlocksCap:   batchSize * 300,
			},
			want: 150.0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
				ScorerParams: &scorers.Config{
					BlockProviderScorerConfig: tt.cfg,
				},
			})
			scorer := peerStatuses.Scorers().BlockProviderScorer()
			assert.Equal(t, tt.want, scorer.MaxScore())
		})
	}
}

func TestScorers_BlockProvider_FormatScorePretty(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	batchSize := uint64(flags.Get().BlockBatchLimit)
	format := "[%0.1f%%, raw: %0.2f,  blocks: %d/1280]"

	tests := []struct {
		name   string
		update func(s *scorers.BlockProviderScorer)
		check  func(s *scorers.BlockProviderScorer)
	}{
		{
			name:   "peer not registered",
			update: nil,
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 100.0, 1.0, 0), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "peer registered zero blocks",
			update: func(s *scorers.BlockProviderScorer) {
				s.Touch("peer1")
			},
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 0.0, 0.0, 0), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "partial batch",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize/4)
			},
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 0.0, 0.0, batchSize/4), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "single batch",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize)
			},
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 5.0, 0.05, batchSize), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "3/2 of a batch",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*3/2)
			},
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 5.0, 0.05, batchSize*3/2), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "multiple batches",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*5)
			},
			check: func(s *scorers.BlockProviderScorer) {
				assert.Equal(t, fmt.Sprintf(format, 25.0, 0.05*5, batchSize*5), s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "multiple batches max score",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", s.Params().ProcessedBlocksCap*5)
			},
			check: func(s *scorers.BlockProviderScorer) {
				want := fmt.Sprintf(format, 100.0, 1.0, s.Params().ProcessedBlocksCap)
				assert.Equal(t, want, s.FormatScorePretty("peer1"))
			},
		},
		{
			name: "decaying",
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("peer1", batchSize*5)
				s.IncrementProcessedBlocks("peer1", batchSize)
				s.IncrementProcessedBlocks("peer1", batchSize/4)
				want := fmt.Sprintf(format, 30.0, 0.05*6, batchSize*6+batchSize/4)
				assert.Equal(t, want, s.FormatScorePretty("peer1"))
				// Maximize block count.
				s.IncrementProcessedBlocks("peer1", s.Params().ProcessedBlocksCap)
				want = fmt.Sprintf(format, 100.0, 1.0, s.Params().ProcessedBlocksCap)
				assert.Equal(t, want, s.FormatScorePretty("peer1"))
				// Half of blocks is to be decayed.
				s.Decay()
			},
			check: func(s *scorers.BlockProviderScorer) {
				want := fmt.Sprintf(format, 50.0, 0.5, s.Params().ProcessedBlocksCap/2)
				assert.Equal(t, want, s.FormatScorePretty("peer1"))
			},
		},
	}

	peerStatusGen := func() *peers.Status {
		return peers.NewStatus(ctx, &peers.StatusConfig{
			ScorerParams: &scorers.Config{
				BlockProviderScorerConfig: &scorers.BlockProviderScorerConfig{
					ProcessedBatchWeight: 0.05,
					ProcessedBlocksCap:   20 * batchSize,
					Decay:                10 * batchSize,
				},
			},
		})
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			peerStatuses := peerStatusGen()
			scorer := peerStatuses.Scorers().BlockProviderScorer()
			if tt.update != nil {
				tt.update(scorer)
			}
			tt.check(scorer)
		})
	}

	t.Run("peer scorer disabled", func(t *testing.T) {
		resetCfg := features.InitWithReset(&features.Flags{
			EnablePeerScorer: false,
		})
		defer resetCfg()
		peerStatuses := peerStatusGen()
		scorer := peerStatuses.Scorers().BlockProviderScorer()
		assert.Equal(t, "disabled", scorer.FormatScorePretty("peer1"))
	})
}

func TestScorers_BlockProvider_BadPeerMarking(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
		ScorerParams: &scorers.Config{},
	})
	scorer := peerStatuses.Scorers().BlockProviderScorer()

	assert.Equal(t, false, scorer.IsBadPeer("peer1"), "Unexpected status for unregistered peer")
	scorer.IncrementProcessedBlocks("peer1", 64)
	assert.Equal(t, false, scorer.IsBadPeer("peer1"))
	assert.Equal(t, 0, len(scorer.BadPeers()))
}
