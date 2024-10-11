package types

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

var config *cfg.Config // NOTE: must be reset for each _test.go file

func TestMain(m *testing.M) {
	config = test.ResetTestRoot("consensus_height_vote_set_test")
	code := m.Run()
	os.RemoveAll(config.RootDir)
	os.Exit(code)
}

func TestPeerCatchupRounds(t *testing.T) {
	valSet, privVals := types.RandValidatorSet(10, 1)

	hvs := NewExtendedHeightVoteSet(test.DefaultTestChainID, 1, valSet)

	vote999_0 := makeVoteHR(999, privVals)
	added, err := hvs.AddVote(vote999_0, "peer1", true)
	if !added || err != nil {
		t.Error("Expected to successfully add vote from peer", added, err)
	}

	vote1000_0 := makeVoteHR(1000, privVals)
	added, err = hvs.AddVote(vote1000_0, "peer1", true)
	if !added || err != nil {
		t.Error("Expected to successfully add vote from peer", added, err)
	}

	vote1001_0 := makeVoteHR(1001, privVals)
	added, err = hvs.AddVote(vote1001_0, "peer1", true)
	if err != ErrGotVoteFromUnwantedRound {
		t.Errorf("expected GotVoteFromUnwantedRoundError, but got %v", err)
	}
	if added {
		t.Error("Expected to *not* add vote from peer, too many catchup rounds.")
	}

	added, err = hvs.AddVote(vote1001_0, "peer2", true)
	if !added || err != nil {
		t.Error("Expected to successfully add vote from another peer")
	}
}

func TestInconsistentExtensionData(t *testing.T) {
	valSet, privVals := types.RandValidatorSet(10, 1)

	hvsE := NewExtendedHeightVoteSet(test.DefaultTestChainID, 1, valSet)
	voteNoExt := makeVoteHR(20, privVals)
	voteNoExt.Extension, voteNoExt.ExtensionSignature = nil, nil
	require.Panics(t, func() {
		_, _ = hvsE.AddVote(voteNoExt, "peer1", false)
	})

	hvsNoE := NewHeightVoteSet(test.DefaultTestChainID, 1, valSet)
	voteExt := makeVoteHR(20, privVals)
	require.Panics(t, func() {
		_, _ = hvsNoE.AddVote(voteExt, "peer1", true)
	})
}

func makeVoteHR(
	round int32,
	privVals []types.PrivValidator,
) *types.Vote {
	height := int64(1)
	valIndex := 0
	privVal := privVals[valIndex]
	randBytes := cmtrand.Bytes(tmhash.Size)

	vote, err := types.MakeVote(
		privVal,
		test.DefaultTestChainID,
		0,
		height,
		round,
		types.PrecommitType,
		types.BlockID{Hash: randBytes, PartSetHeader: types.PartSetHeader{}},
		cmttime.Now(),
	)
	if err != nil {
		panic(err)
	}

	return vote
}
