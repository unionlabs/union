package initialsync

import (
	"context"
	"math"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
)

func TestBlocksFetcher_selectFailOverPeer(t *testing.T) {
	type args struct {
		excludedPID peer.ID
		peers       []peer.ID
	}
	fetcher := newBlocksFetcher(context.Background(), &blocksFetcherConfig{})
	tests := []struct {
		name    string
		args    args
		want    peer.ID
		wantErr error
	}{
		{
			name: "No peers provided",
			args: args{
				excludedPID: "a",
				peers:       []peer.ID{},
			},
			want:    "",
			wantErr: errNoPeersAvailable,
		},
		{
			name: "Single peer which needs to be excluded",
			args: args{
				excludedPID: "a",
				peers: []peer.ID{
					"a",
				},
			},
			want:    "",
			wantErr: errNoPeersAvailable,
		},
		{
			name: "Single peer available",
			args: args{
				excludedPID: "a",
				peers: []peer.ID{
					"cde",
				},
			},
			want:    "cde",
			wantErr: nil,
		},
		{
			name: "Two peers available, excluded first",
			args: args{
				excludedPID: "a",
				peers: []peer.ID{
					"a", "cde",
				},
			},
			want:    "cde",
			wantErr: nil,
		},
		{
			name: "Two peers available, excluded second",
			args: args{
				excludedPID: "a",
				peers: []peer.ID{
					"cde", "a",
				},
			},
			want:    "cde",
			wantErr: nil,
		},
		{
			name: "Multiple peers available",
			args: args{
				excludedPID: "a",
				peers: []peer.ID{
					"a", "cde", "cde", "cde",
				},
			},
			want:    "cde",
			wantErr: nil,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := fetcher.selectFailOverPeer(tt.args.excludedPID, tt.args.peers)
			if tt.wantErr != nil {
				assert.ErrorContains(t, tt.wantErr.Error(), err)
			} else {
				assert.Equal(t, tt.want, got)
			}
		})
	}
}

func TestBlocksFetcher_filterPeers(t *testing.T) {
	type weightedPeer struct {
		peer.ID
		usedCapacity int64
	}
	type args struct {
		peers           []weightedPeer
		peersPercentage float64
		capacityWeight  float64
	}

	batchSize := uint64(flags.Get().BlockBatchLimit)
	tests := []struct {
		name   string
		args   args
		update func(s *scorers.BlockProviderScorer)
		want   []peer.ID
	}{
		{
			name: "no peers available",
			args: args{
				peers:           []weightedPeer{},
				peersPercentage: 1.0,
				capacityWeight:  0.2,
			},
			want: []peer.ID{},
		},
		{
			name: "single peer",
			args: args{
				peers: []weightedPeer{
					{"a", 1200},
				},
				peersPercentage: 1.0,
				capacityWeight:  0.2,
			},
			want: []peer.ID{"a"},
		},
		{
			name: "multiple peers same capacity",
			args: args{
				peers: []weightedPeer{
					{"a", 2400},
					{"b", 2400},
					{"c", 2400},
				},
				peersPercentage: 1.0,
				capacityWeight:  0.2,
			},
			want: []peer.ID{"a", "b", "c"},
		},
		{
			name: "multiple peers capacity as tie-breaker",
			args: args{
				peers: []weightedPeer{
					{"a", 6000},
					{"b", 3000},
					{"c", 0},
					{"d", 9000},
					{"e", 6000},
				},
				peersPercentage: 1.0,
				capacityWeight:  0.2,
			},
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("a", batchSize*2)
				s.IncrementProcessedBlocks("b", batchSize*2)
				s.IncrementProcessedBlocks("c", batchSize*2)
				s.IncrementProcessedBlocks("d", batchSize*2)
				s.IncrementProcessedBlocks("e", batchSize*2)
			},
			want: []peer.ID{"c", "b", "a", "e", "d"},
		},
		{
			name: "multiple peers same capacity different scores",
			args: args{
				peers: []weightedPeer{
					{"a", 9000},
					{"b", 9000},
					{"c", 9000},
					{"d", 9000},
					{"e", 9000},
				},
				peersPercentage: 0.8,
				capacityWeight:  0.2,
			},
			update: func(s *scorers.BlockProviderScorer) {
				s.IncrementProcessedBlocks("e", s.Params().ProcessedBlocksCap)
				s.IncrementProcessedBlocks("b", s.Params().ProcessedBlocksCap/2)
				s.IncrementProcessedBlocks("c", s.Params().ProcessedBlocksCap/4)
				s.IncrementProcessedBlocks("a", s.Params().ProcessedBlocksCap/8)
				s.IncrementProcessedBlocks("d", 0)
			},
			want: []peer.ID{"e", "b", "c", "a"},
		},
		{
			name: "multiple peers different capacities and scores",
			args: args{
				peers: []weightedPeer{
					{"a", 6500},
					{"b", 2500},
					{"c", 1000},
					{"d", 9000},
					{"e", 6500},
				},
				peersPercentage: 0.8,
				capacityWeight:  0.2,
			},
			update: func(s *scorers.BlockProviderScorer) {
				// Make sure that score takes priority over capacity.
				s.IncrementProcessedBlocks("c", batchSize*5)
				s.IncrementProcessedBlocks("b", batchSize*15)
				// Break tie using capacity as a tie-breaker (a and ghi have the same score).
				s.IncrementProcessedBlocks("a", batchSize*3)
				s.IncrementProcessedBlocks("e", batchSize*3)
				// Exclude peer (peers percentage is 80%).
				s.IncrementProcessedBlocks("d", batchSize)
			},
			want: []peer.ID{"b", "c", "a", "e"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})
			fetcher := newBlocksFetcher(context.Background(), &blocksFetcherConfig{
				chain:                    mc,
				p2p:                      p2p,
				peerFilterCapacityWeight: tt.args.capacityWeight,
			})
			// Non-leaking bucket, with initial capacity of 10000.
			fetcher.rateLimiter = leakybucket.NewCollector(0.000001, 10000, 1*time.Second, false)
			peerIDs := make([]peer.ID, 0)
			for _, pid := range tt.args.peers {
				peerIDs = append(peerIDs, pid.ID)
				fetcher.rateLimiter.Add(pid.ID.String(), pid.usedCapacity)
			}
			if tt.update != nil {
				tt.update(fetcher.p2p.Peers().Scorers().BlockProviderScorer())
			}
			// Since peer selection is probabilistic (weighted, with high scorers having higher
			// chance of being selected), we need multiple rounds of filtering to test the order:
			// over multiple attempts, top scorers should be picked on high positions more often.
			peerStats := make(map[peer.ID]int, len(tt.want))
			var filteredPIDs []peer.ID
			var err error
			for i := 0; i < 1000; i++ {
				filteredPIDs = fetcher.filterPeers(context.Background(), peerIDs, tt.args.peersPercentage)
				if len(filteredPIDs) <= 1 {
					break
				}
				require.NoError(t, err)
				for j, pid := range filteredPIDs {
					// The higher peer in the list, the more "points" will it get.
					peerStats[pid] += len(tt.want) - j
				}
			}

			// If percentage of peers was requested, rebuild combined filtered peers list.
			if len(filteredPIDs) != len(peerStats) && len(peerStats) > 0 {
				filteredPIDs = []peer.ID{}
				for pid := range peerStats {
					filteredPIDs = append(filteredPIDs, pid)
				}
			}

			// Sort by frequency of appearance in high positions on filtering.
			sort.Slice(filteredPIDs, func(i, j int) bool {
				return peerStats[filteredPIDs[i]] > peerStats[filteredPIDs[j]]
			})
			if tt.args.peersPercentage < 1.0 {
				limit := uint64(math.Round(float64(len(filteredPIDs)) * tt.args.peersPercentage))
				filteredPIDs = filteredPIDs[:limit]
			}

			// Re-arrange peers with the same remaining capacity, deterministically .
			// They are deliberately shuffled - so that on the same capacity any of
			// such peers can be selected. That's why they are sorted here.
			sort.SliceStable(filteredPIDs, func(i, j int) bool {
				score1 := fetcher.p2p.Peers().Scorers().BlockProviderScorer().Score(filteredPIDs[i])
				score2 := fetcher.p2p.Peers().Scorers().BlockProviderScorer().Score(filteredPIDs[j])
				if score1 == score2 {
					cap1 := fetcher.rateLimiter.Remaining(filteredPIDs[i].String())
					cap2 := fetcher.rateLimiter.Remaining(filteredPIDs[j].String())
					if cap1 == cap2 {
						return filteredPIDs[i].String() < filteredPIDs[j].String()
					}
				}
				return i < j
			})
			assert.DeepEqual(t, tt.want, filteredPIDs)
		})
	}
}

func TestBlocksFetcher_removeStalePeerLocks(t *testing.T) {
	type peerData struct {
		peerID   peer.ID
		accessed time.Time
	}
	tests := []struct {
		name     string
		age      time.Duration
		peersIn  []peerData
		peersOut []peerData
	}{
		{
			name:     "empty map",
			age:      peerLockMaxAge,
			peersIn:  []peerData{},
			peersOut: []peerData{},
		},
		{
			name: "no stale peer locks",
			age:  peerLockMaxAge,
			peersIn: []peerData{
				{
					peerID:   "a",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "b",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "c",
					accessed: prysmTime.Now(),
				},
			},
			peersOut: []peerData{
				{
					peerID:   "a",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "b",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "c",
					accessed: prysmTime.Now(),
				},
			},
		},
		{
			name: "one stale peer lock",
			age:  peerLockMaxAge,
			peersIn: []peerData{
				{
					peerID:   "a",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "b",
					accessed: prysmTime.Now().Add(-peerLockMaxAge),
				},
				{
					peerID:   "c",
					accessed: prysmTime.Now(),
				},
			},
			peersOut: []peerData{
				{
					peerID:   "a",
					accessed: prysmTime.Now(),
				},
				{
					peerID:   "c",
					accessed: prysmTime.Now(),
				},
			},
		},
		{
			name: "all peer locks are stale",
			age:  peerLockMaxAge,
			peersIn: []peerData{
				{
					peerID:   "a",
					accessed: prysmTime.Now().Add(-peerLockMaxAge),
				},
				{
					peerID:   "b",
					accessed: prysmTime.Now().Add(-peerLockMaxAge),
				},
				{
					peerID:   "c",
					accessed: prysmTime.Now().Add(-peerLockMaxAge),
				},
			},
			peersOut: []peerData{},
		},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{})

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fetcher.peerLocks = make(map[peer.ID]*peerLock, len(tt.peersIn))
			for _, data := range tt.peersIn {
				fetcher.peerLocks[data.peerID] = &peerLock{
					Mutex:    sync.Mutex{},
					accessed: data.accessed,
				}
			}

			fetcher.removeStalePeerLocks(tt.age)

			var peersOut1, peersOut2 []peer.ID
			for _, data := range tt.peersOut {
				peersOut1 = append(peersOut1, data.peerID)
			}
			for peerID := range fetcher.peerLocks {
				peersOut2 = append(peersOut2, peerID)
			}
			sort.SliceStable(peersOut1, func(i, j int) bool {
				return peersOut1[i].String() < peersOut1[j].String()
			})
			sort.SliceStable(peersOut2, func(i, j int) bool {
				return peersOut2[i].String() < peersOut2[j].String()
			})
			assert.DeepEqual(t, peersOut1, peersOut2, "Unexpected peers map")
		})
	}
}
