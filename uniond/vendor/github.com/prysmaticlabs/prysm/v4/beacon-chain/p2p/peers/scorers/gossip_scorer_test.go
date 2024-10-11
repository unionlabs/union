package scorers_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers/scorers"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestScorers_Gossip_Score(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	tests := []struct {
		name   string
		update func(scorer *scorers.GossipScorer)
		check  func(scorer *scorers.GossipScorer)
	}{
		{
			name: "nonexistent peer",
			update: func(scorer *scorers.GossipScorer) {
			},
			check: func(scorer *scorers.GossipScorer) {
				assert.Equal(t, 0.0, scorer.Score("peer1"), "Unexpected score")
			},
		},
		{
			name: "existent bad peer",
			update: func(scorer *scorers.GossipScorer) {
				scorer.SetGossipData("peer1", -101.0, 1, nil)
			},
			check: func(scorer *scorers.GossipScorer) {
				assert.Equal(t, -101.0, scorer.Score("peer1"), "Unexpected score")
				assert.Equal(t, true, scorer.IsBadPeer("peer1"), "Unexpected good peer")
			},
		},
		{
			name: "good peer",
			update: func(scorer *scorers.GossipScorer) {
				scorer.SetGossipData("peer1", 10.0, 0, map[string]*pbrpc.TopicScoreSnapshot{"a": {TimeInMesh: 100}})
			},
			check: func(scorer *scorers.GossipScorer) {
				assert.Equal(t, 10.0, scorer.Score("peer1"), "Unexpected score")
				assert.Equal(t, false, scorer.IsBadPeer("peer1"), "Unexpected bad peer")
				_, _, topicMap, err := scorer.GossipData("peer1")
				assert.NoError(t, err)
				assert.Equal(t, uint64(100), topicMap["a"].TimeInMesh, "incorrect time in mesh")
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			peerStatuses := peers.NewStatus(ctx, &peers.StatusConfig{
				ScorerParams: &scorers.Config{},
			})
			scorer := peerStatuses.Scorers().GossipScorer()
			if tt.update != nil {
				tt.update(scorer)
			}
			tt.check(scorer)
		})
	}
}
