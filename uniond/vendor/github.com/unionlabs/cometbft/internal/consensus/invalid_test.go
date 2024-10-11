package consensus

import (
	"testing"
	"time"

	cmtcons "github.com/cometbft/cometbft/api/cometbft/consensus/v1"
	cfg "github.com/cometbft/cometbft/config"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/types"
)

// ----------------------------------------------
// byzantine failures

// one byz val sends a precommit for a random block at each height
// Ensure a testnet makes blocks.
func TestReactorInvalidPrecommit(t *testing.T) {
	n := 4
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore,
		func(c *cfg.Config) {
			c.Consensus.TimeoutPropose = 3000 * time.Millisecond
			c.Consensus.TimeoutPrevote = 1000 * time.Millisecond
			c.Consensus.TimeoutPrecommit = 1000 * time.Millisecond
		})
	defer cleanup()

	for i := 0; i < n; i++ {
		ticker := NewTimeoutTicker()
		ticker.SetLogger(css[i].Logger)
		css[i].SetTimeoutTicker(ticker)
	}

	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// this val sends a random precommit at each height
	byzValIdx := n - 1
	byzVal := css[byzValIdx]
	byzR := reactors[byzValIdx]

	// update the doPrevote function to just send a valid precommit for a random block
	// and otherwise disable the priv validator
	byzVal.mtx.Lock()
	pv := byzVal.privValidator
	byzVal.doPrevote = func(int64, int32) {
		invalidDoPrevoteFunc(t, byzVal, byzR.Switch, pv)
	}
	byzVal.mtx.Unlock()

	// wait for a bunch of blocks
	// TODO: make this tighter by ensuring the halt happens by block 2
	for i := 0; i < 10; i++ {
		timeoutWaitGroup(n, func(j int) {
			<-blocksSubs[j].Out()
		})
	}
}

func invalidDoPrevoteFunc(t *testing.T, cs *State, sw *p2p.Switch, pv types.PrivValidator) {
	t.Helper()
	// routine to:
	// - precommit for a random block
	// - send precommit to all peers
	// - disable privValidator (so we don't do normal precommits)
	go func() {
		cs.mtx.Lock()
		defer cs.mtx.Unlock()
		cs.privValidator = pv
		pubKey, err := cs.privValidator.GetPubKey()
		if err != nil {
			panic(err)
		}
		addr := pubKey.Address()
		valIndex, _ := cs.Validators.GetByAddress(addr)

		// precommit a random block
		blockHash := bytes.HexBytes(cmtrand.Bytes(32))
		timestamp := cs.voteTime(cs.Height)

		precommit := &types.Vote{
			ValidatorAddress: addr,
			ValidatorIndex:   valIndex,
			Height:           cs.Height,
			Round:            cs.Round,
			Timestamp:        timestamp,
			Type:             types.PrecommitType,
			BlockID: types.BlockID{
				Hash:          blockHash,
				PartSetHeader: types.PartSetHeader{Total: 1, Hash: cmtrand.Bytes(32)},
			},
		}
		p := precommit.ToProto()
		err = cs.privValidator.SignVote(cs.state.ChainID, p, true)
		if err != nil {
			t.Error(err)
		}
		precommit.Signature = p.Signature
		precommit.ExtensionSignature = p.ExtensionSignature
		cs.privValidator = nil // disable priv val so we don't do normal votes

		peers := sw.Peers().Copy()
		for _, peer := range peers {
			cs.Logger.Info("Sending bad vote", "block", blockHash, "peer", peer)
			peer.Send(p2p.Envelope{
				Message:   &cmtcons.Vote{Vote: precommit.ToProto()},
				ChannelID: VoteChannel,
			})
		}
	}()
}
