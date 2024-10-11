package consensus

import (
	"context"
	"fmt"
	"os"
	"path"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtcons "github.com/cometbft/cometbft/api/cometbft/consensus/v1"
	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/cometbft/cometbft/internal/bits"
	cstypes "github.com/cometbft/cometbft/internal/consensus/types"
	"github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/p2p"
	p2pmock "github.com/cometbft/cometbft/p2p/mock"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	statemocks "github.com/cometbft/cometbft/state/mocks"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
	cmterrors "github.com/cometbft/cometbft/types/errors"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// ----------------------------------------------
// in-process testnets

var defaultTestTime = time.Date(2019, 1, 1, 0, 0, 0, 0, time.UTC)

func startConsensusNet(t *testing.T, css []*State, n int) (
	[]*Reactor,
	[]types.Subscription,
	[]*types.EventBus,
) {
	t.Helper()
	reactors := make([]*Reactor, n)
	blocksSubs := make([]types.Subscription, 0)
	eventBuses := make([]*types.EventBus, n)
	for i := 0; i < n; i++ {
		// logger, err := cmtflags.ParseLogLevel("consensus:info,*:error", logger, "info")
		// if err != nil {	t.Fatal(err)}
		reactors[i] = NewReactor(css[i], true) // so we dont start the consensus states
		reactors[i].SetLogger(css[i].Logger)

		// eventBus is already started with the cs
		eventBuses[i] = css[i].eventBus
		reactors[i].SetEventBus(eventBuses[i])

		blocksSub, err := eventBuses[i].Subscribe(context.Background(), testSubscriber, types.EventQueryNewBlock)
		require.NoError(t, err)
		blocksSubs = append(blocksSubs, blocksSub)

		if css[i].state.LastBlockHeight == 0 { // simulate handle initChain in handshake
			if err := css[i].blockExec.Store().Save(css[i].state); err != nil {
				t.Error(err)
			}
		}
	}
	// make connected switches and start all reactors
	p2p.MakeConnectedSwitches(config.P2P, n, func(i int, s *p2p.Switch) *p2p.Switch {
		s.AddReactor("CONSENSUS", reactors[i])
		s.SetLogger(reactors[i].conS.Logger.With("module", "p2p"))
		return s
	}, p2p.Connect2Switches)

	// now that everyone is connected,  start the state machines
	// If we started the state machines before everyone was connected,
	// we'd block when the cs fires NewBlockEvent and the peers are trying to start their reactors
	// TODO: is this still true with new pubsub?
	for i := 0; i < n; i++ {
		s := reactors[i].conS.GetState()
		reactors[i].SwitchToConsensus(s, false)
	}
	return reactors, blocksSubs, eventBuses
}

func stopConsensusNet(logger log.Logger, reactors []*Reactor, eventBuses []*types.EventBus) {
	logger.Info("stopConsensusNet", "n", len(reactors))
	for i, r := range reactors {
		logger.Info("stopConsensusNet: Stopping Reactor", "i", i)
		if err := r.Switch.Stop(); err != nil {
			logger.Error("error trying to stop switch", "error", err)
		}
	}
	for i, b := range eventBuses {
		logger.Info("stopConsensusNet: Stopping eventBus", "i", i)
		if err := b.Stop(); err != nil {
			logger.Error("error trying to stop eventbus", "error", err)
		}
	}
	logger.Info("stopConsensusNet: DONE", "n", len(reactors))
}

// Ensure a testnet makes blocks.
func TestReactorBasic(t *testing.T) {
	n := 4
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore)
	defer cleanup()
	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)
	// wait till everyone makes the first new block
	timeoutWaitGroup(n, func(j int) {
		<-blocksSubs[j].Out()
	})
}

// Ensure we can process blocks with evidence.
func TestReactorWithEvidence(t *testing.T) {
	nValidators := 4
	testName := "consensus_reactor_test"
	tickerFunc := newMockTickerFunc(true)
	appFunc := newKVStore

	// heed the advice from https://www.sandimetz.com/blog/2016/1/20/the-wrong-abstraction
	// to unroll unwieldy abstractions. Here we duplicate the code from:
	// css := randConsensusNet(N, "consensus_reactor_test", newMockTickerFunc(true), newKVStore)

	genDoc, privVals := randGenesisDoc(nValidators, 30, nil, cmttime.Now())
	css := make([]*State, nValidators)
	logger := consensusLogger()
	for i := 0; i < nValidators; i++ {
		stateDB := dbm.NewMemDB() // each state needs its own db
		stateStore := sm.NewStore(stateDB, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		state, _ := stateStore.LoadFromDBOrGenesisDoc(genDoc)
		thisConfig := ResetConfig(fmt.Sprintf("%s_%d", testName, i))
		defer os.RemoveAll(thisConfig.RootDir)
		ensureDir(path.Dir(thisConfig.Consensus.WalFile())) // dir for wal
		app := appFunc()
		vals := types.TM2PB.ValidatorUpdates(state.Validators)
		_, err := app.InitChain(context.Background(), &abci.InitChainRequest{Validators: vals})
		require.NoError(t, err)

		pv := privVals[i]
		// duplicate code from:
		// css[i] = newStateWithConfig(thisConfig, state, privVals[i], app)

		blockDB := dbm.NewMemDB()
		blockStore := store.NewBlockStore(blockDB)

		mtx := new(cmtsync.Mutex)
		memplMetrics := mempl.NopMetrics()
		// one for mempool, one for consensus
		proxyAppConnCon := proxy.NewAppConnConsensus(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())
		proxyAppConnMem := proxy.NewAppConnMempool(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())

		// Make Mempool
		mempool := mempl.NewCListMempool(config.Mempool,
			proxyAppConnMem,
			state.LastBlockHeight,
			mempl.WithMetrics(memplMetrics),
			mempl.WithPreCheck(sm.TxPreCheck(state)),
			mempl.WithPostCheck(sm.TxPostCheck(state)))

		if thisConfig.Consensus.WaitForTxs() {
			mempool.EnableTxsAvailable()
		}

		// mock the evidence pool
		// everyone includes evidence of another double signing
		vIdx := (i + 1) % nValidators
		ev, err := types.NewMockDuplicateVoteEvidenceWithValidator(1, defaultTestTime, privVals[vIdx], genDoc.ChainID)
		require.NoError(t, err)
		evpool := &statemocks.EvidencePool{}
		evpool.On("CheckEvidence", mock.AnythingOfType("types.EvidenceList")).Return(nil)
		evpool.On("PendingEvidence", mock.AnythingOfType("int64")).Return([]types.Evidence{
			ev,
		}, int64(len(ev.Bytes())))
		evpool.On("Update", mock.AnythingOfType("state.State"), mock.AnythingOfType("types.EvidenceList")).Return()

		evpool2 := sm.EmptyEvidencePool{}

		// Make State
		blockExec := sm.NewBlockExecutor(stateStore, log.TestingLogger(), proxyAppConnCon, mempool, evpool, blockStore)
		cs := NewState(thisConfig.Consensus, state, blockExec, blockStore, mempool, evpool2)
		cs.SetLogger(log.TestingLogger().With("module", "consensus"))
		cs.SetPrivValidator(pv)

		eventBus := types.NewEventBus()
		eventBus.SetLogger(log.TestingLogger().With("module", "events"))
		err = eventBus.Start()
		require.NoError(t, err)
		cs.SetEventBus(eventBus)

		cs.SetTimeoutTicker(tickerFunc())
		cs.SetLogger(logger.With("validator", i, "module", "consensus"))

		css[i] = cs
	}

	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, nValidators)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// we expect for each validator that is the proposer to propose one piece of evidence.
	for i := 0; i < nValidators; i++ {
		timeoutWaitGroup(nValidators, func(j int) {
			msg := <-blocksSubs[j].Out()
			block := msg.Data().(types.EventDataNewBlock).Block
			assert.Len(t, block.Evidence.Evidence, 1)
		})
	}
}

// ------------------------------------

// Ensure a testnet makes blocks when there are txs.
func TestReactorCreatesBlockWhenEmptyBlocksFalse(t *testing.T) {
	n := 4
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore,
		func(c *cfg.Config) {
			c.Consensus.CreateEmptyBlocks = false
		})
	defer cleanup()
	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// send a tx
	reqRes, err := assertMempool(css[3].txNotifier).CheckTx(kvstore.NewTxFromID(1), "")
	if err != nil {
		t.Error(err)
	}
	require.False(t, reqRes.Response.GetCheckTx().IsErr())

	// wait till everyone makes the first new block
	timeoutWaitGroup(n, func(j int) {
		<-blocksSubs[j].Out()
	})
}

func TestReactorReceiveDoesNotPanicIfAddPeerHasntBeenCalledYet(t *testing.T) {
	n := 1
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore)
	defer cleanup()
	reactors, _, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	var (
		reactor = reactors[0]
		peer    = p2pmock.NewPeer(nil)
	)

	reactor.InitPeer(peer)

	// simulate switch calling Receive before AddPeer
	assert.NotPanics(t, func() {
		reactor.Receive(p2p.Envelope{
			ChannelID: StateChannel,
			Src:       peer,
			Message: &cmtcons.HasVote{
				Height: 1,
				Round:  1,
				Index:  1,
				Type:   types.PrevoteType,
			},
		})
		reactor.AddPeer(peer)
	})
}

func TestReactorReceivePanicsIfInitPeerHasntBeenCalledYet(t *testing.T) {
	n := 1
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore)
	defer cleanup()
	reactors, _, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	var (
		reactor = reactors[0]
		peer    = p2pmock.NewPeer(nil)
	)

	// we should call InitPeer here

	// simulate switch calling Receive before AddPeer
	assert.Panics(t, func() {
		reactor.Receive(p2p.Envelope{
			ChannelID: StateChannel,
			Src:       peer,
			Message: &cmtcons.HasVote{
				Height: 1,
				Round:  1,
				Index:  1,
				Type:   types.PrevoteType,
			},
		})
	})
}

// TestSwitchToConsensusVoteExtensions tests that the SwitchToConsensus correctly
// checks for vote extension data when required.
func TestSwitchToConsensusVoteExtensions(t *testing.T) {
	for _, testCase := range []struct {
		name                  string
		storedHeight          int64
		initialRequiredHeight int64
		includeExtensions     bool
		shouldPanic           bool
	}{
		{
			name:                  "no vote extensions but not required",
			initialRequiredHeight: 0,
			storedHeight:          2,
			includeExtensions:     false,
			shouldPanic:           false,
		},
		{
			name:                  "no vote extensions but required this height",
			initialRequiredHeight: 2,
			storedHeight:          2,
			includeExtensions:     false,
			shouldPanic:           true,
		},
		{
			name:                  "no vote extensions and required in future",
			initialRequiredHeight: 3,
			storedHeight:          2,
			includeExtensions:     false,
			shouldPanic:           false,
		},
		{
			name:                  "no vote extensions and required previous height",
			initialRequiredHeight: 1,
			storedHeight:          2,
			includeExtensions:     false,
			shouldPanic:           true,
		},
		{
			name:                  "vote extensions and required previous height",
			initialRequiredHeight: 1,
			storedHeight:          2,
			includeExtensions:     true,
			shouldPanic:           false,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()

			cs, vs := randState(1)
			validator := vs[0]
			validator.Height = testCase.storedHeight

			cs.state.LastBlockHeight = testCase.storedHeight
			cs.state.LastValidators = cs.state.Validators.Copy()
			cs.state.ConsensusParams.Feature.VoteExtensionsEnableHeight = testCase.initialRequiredHeight

			propBlock, err := cs.createProposalBlock(ctx)
			require.NoError(t, err)

			// Consensus is preparing to do the next height after the stored height.
			cs.Height = testCase.storedHeight + 1
			propBlock.Height = testCase.storedHeight
			blockParts, err := propBlock.MakePartSet(types.BlockPartSizeBytes)
			require.NoError(t, err)

			var voteSet *types.VoteSet
			if testCase.includeExtensions {
				voteSet = types.NewExtendedVoteSet(cs.state.ChainID, testCase.storedHeight, 0, types.PrecommitType, cs.state.Validators)
			} else {
				voteSet = types.NewVoteSet(cs.state.ChainID, testCase.storedHeight, 0, types.PrecommitType, cs.state.Validators)
			}
			signedVote := signVote(validator, types.PrecommitType, cs.state.ChainID, types.BlockID{
				Hash:          propBlock.Hash(),
				PartSetHeader: blockParts.Header(),
			}, testCase.includeExtensions)

			var veHeight int64
			if testCase.includeExtensions {
				require.NotNil(t, signedVote.ExtensionSignature)
				veHeight = testCase.storedHeight
			} else {
				require.Nil(t, signedVote.Extension)
				require.Nil(t, signedVote.ExtensionSignature)
			}

			added, err := voteSet.AddVote(signedVote)
			require.NoError(t, err)
			require.True(t, added)

			veHeightParam := types.DefaultFeatureParams()
			veHeightParam.VoteExtensionsEnableHeight = veHeight
			if testCase.includeExtensions {
				cs.blockStore.SaveBlockWithExtendedCommit(propBlock, blockParts, voteSet.MakeExtendedCommit(veHeightParam))
			} else {
				cs.blockStore.SaveBlock(propBlock, blockParts, voteSet.MakeExtendedCommit(veHeightParam).ToCommit())
			}
			reactor := NewReactor(
				cs,
				true,
			)

			if testCase.shouldPanic {
				assert.Panics(t, func() {
					reactor.SwitchToConsensus(cs.state, false)
				})
			} else {
				reactor.SwitchToConsensus(cs.state, false)
			}
		})
	}
}

// Test we record stats about votes and block parts from other peers.
func TestReactorRecordsVotesAndBlockParts(t *testing.T) {
	n := 4
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_test", newMockTickerFunc(true), newKVStore)
	defer cleanup()
	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, n)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// wait till everyone makes the first new block
	timeoutWaitGroup(n, func(j int) {
		<-blocksSubs[j].Out()
	})

	// Get peer
	peer := reactors[1].Switch.Peers().Copy()[0]
	// Get peer state
	ps := peer.Get(types.PeerStateKey).(*PeerState)

	assert.Greater(t, ps.VotesSent(), 0, "number of votes sent should have increased")
	assert.Greater(t, ps.BlockPartsSent(), 0, "number of votes sent should have increased")
}

// -------------------------------------------------------------
// ensure we can make blocks despite cycling a validator set

func TestReactorVotingPowerChange(t *testing.T) {
	nVals := 4
	logger := log.TestingLogger()
	css, cleanup := randConsensusNet(
		t,
		nVals,
		"consensus_voting_power_changes_test",
		newMockTickerFunc(true),
		newPersistentKVStore)
	defer cleanup()
	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, nVals)
	defer stopConsensusNet(logger, reactors, eventBuses)

	// map of active validators
	activeVals := make(map[string]struct{})
	for i := 0; i < nVals; i++ {
		pubKey, err := css[i].privValidator.GetPubKey()
		require.NoError(t, err)
		addr := pubKey.Address()
		activeVals[string(addr)] = struct{}{}
	}

	// wait till everyone makes block 1
	timeoutWaitGroup(nVals, func(j int) {
		<-blocksSubs[j].Out()
	})

	// ---------------------------------------------------------------------------
	logger.Debug("---------------------------- Testing changing the voting power of one validator a few times")

	val1PubKey, err := css[0].privValidator.GetPubKey()
	require.NoError(t, err)

	updateValidatorTx := updateValTx(val1PubKey, 25)
	previousTotalVotingPower := css[0].GetRoundState().LastValidators.TotalVotingPower()

	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlockWithTx(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)

	if css[0].GetRoundState().LastValidators.TotalVotingPower() == previousTotalVotingPower {
		t.Fatalf(
			"expected voting power to change (before: %d, after: %d)",
			previousTotalVotingPower,
			css[0].GetRoundState().LastValidators.TotalVotingPower())
	}

	updateValidatorTx = updateValTx(val1PubKey, 2)
	previousTotalVotingPower = css[0].GetRoundState().LastValidators.TotalVotingPower()

	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlockWithTx(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)

	if css[0].GetRoundState().LastValidators.TotalVotingPower() == previousTotalVotingPower {
		t.Fatalf(
			"expected voting power to change (before: %d, after: %d)",
			previousTotalVotingPower,
			css[0].GetRoundState().LastValidators.TotalVotingPower())
	}

	updateValidatorTx = updateValTx(val1PubKey, 26)
	previousTotalVotingPower = css[0].GetRoundState().LastValidators.TotalVotingPower()

	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlockWithTx(t, nVals, activeVals, blocksSubs, css, updateValidatorTx)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)
	waitForAndValidateBlock(t, nVals, activeVals, blocksSubs, css)

	if css[0].GetRoundState().LastValidators.TotalVotingPower() == previousTotalVotingPower {
		t.Fatalf(
			"expected voting power to change (before: %d, after: %d)",
			previousTotalVotingPower,
			css[0].GetRoundState().LastValidators.TotalVotingPower())
	}
}

func TestReactorValidatorSetChanges(t *testing.T) {
	nPeers := 7
	nVals := 4
	css, _, _, cleanup := randConsensusNetWithPeers(
		t,
		nVals,
		nPeers,
		"consensus_val_set_changes_test",
		newMockTickerFunc(true),
		newPersistentKVStoreWithPath)

	defer cleanup()
	logger := log.TestingLogger()

	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, nPeers)
	defer stopConsensusNet(logger, reactors, eventBuses)

	// map of active validators
	activeVals := make(map[string]struct{})
	for i := 0; i < nVals; i++ {
		pubKey, err := css[i].privValidator.GetPubKey()
		require.NoError(t, err)
		activeVals[string(pubKey.Address())] = struct{}{}
	}

	// wait till everyone makes block 1
	timeoutWaitGroup(nPeers, func(j int) {
		<-blocksSubs[j].Out()
	})

	t.Run("Testing adding one validator", func(t *testing.T) {
		newValidatorPubKey1, err := css[nVals].privValidator.GetPubKey()
		require.NoError(t, err)
		newValidatorTx1 := updateValTx(newValidatorPubKey1, testMinPower)

		// wait till everyone makes block 2
		// ensure the commit includes all validators
		// send newValTx to change vals in block 3
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css, newValidatorTx1)

		// wait till everyone makes block 3.
		// it includes the commit for block 2, which is by the original validator set
		waitForAndValidateBlockWithTx(t, nPeers, activeVals, blocksSubs, css, newValidatorTx1)

		// wait till everyone makes block 4.
		// it includes the commit for block 3, which is by the original validator set
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css)

		// the commits for block 4 should be with the updated validator set
		activeVals[string(newValidatorPubKey1.Address())] = struct{}{}

		// wait till everyone makes block 5
		// it includes the commit for block 4, which should have the updated validator set
		waitForBlockWithUpdatedValsAndValidateIt(t, nPeers, activeVals, blocksSubs, css)
	})

	t.Run("Testing changing the voting power of one validator", func(t *testing.T) {
		updateValidatorPubKey1, err := css[nVals].privValidator.GetPubKey()
		require.NoError(t, err)
		updateValidatorTx1 := updateValTx(updateValidatorPubKey1, 25)
		previousTotalVotingPower := css[nVals].GetRoundState().LastValidators.TotalVotingPower()

		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css, updateValidatorTx1)
		waitForAndValidateBlockWithTx(t, nPeers, activeVals, blocksSubs, css, updateValidatorTx1)
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css)
		waitForBlockWithUpdatedValsAndValidateIt(t, nPeers, activeVals, blocksSubs, css)

		if css[nVals].GetRoundState().LastValidators.TotalVotingPower() == previousTotalVotingPower {
			t.Errorf(
				"expected voting power to change (before: %d, after: %d)",
				previousTotalVotingPower,
				css[nVals].GetRoundState().LastValidators.TotalVotingPower())
		}
	})

	newValidatorPubKey2, err := css[nVals+1].privValidator.GetPubKey()
	require.NoError(t, err)
	newValidatorTx2 := updateValTx(newValidatorPubKey2, testMinPower)

	newValidatorPubKey3, err := css[nVals+2].privValidator.GetPubKey()
	require.NoError(t, err)
	newValidatorTx3 := updateValTx(newValidatorPubKey3, testMinPower)

	t.Run("Testing adding two validators at once", func(t *testing.T) {
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css, newValidatorTx2, newValidatorTx3)
		waitForAndValidateBlockWithTx(t, nPeers, activeVals, blocksSubs, css, newValidatorTx2, newValidatorTx3)
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css)
		activeVals[string(newValidatorPubKey2.Address())] = struct{}{}
		activeVals[string(newValidatorPubKey3.Address())] = struct{}{}
		waitForBlockWithUpdatedValsAndValidateIt(t, nPeers, activeVals, blocksSubs, css)
	})

	t.Run("Testing removing two validators at once", func(t *testing.T) {
		removeValidatorTx2 := updateValTx(newValidatorPubKey2, 0)
		removeValidatorTx3 := updateValTx(newValidatorPubKey3, 0)

		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css, removeValidatorTx2, removeValidatorTx3)
		waitForAndValidateBlockWithTx(t, nPeers, activeVals, blocksSubs, css, removeValidatorTx2, removeValidatorTx3)
		waitForAndValidateBlock(t, nPeers, activeVals, blocksSubs, css)
		delete(activeVals, string(newValidatorPubKey2.Address()))
		delete(activeVals, string(newValidatorPubKey3.Address()))
		waitForBlockWithUpdatedValsAndValidateIt(t, nPeers, activeVals, blocksSubs, css)
	})
}

// Check we can make blocks with timeout_commit=0.
func TestReactorWithDefaultTimeoutCommit(t *testing.T) {
	n := 4
	css, cleanup := randConsensusNet(t, n, "consensus_reactor_with_timeout_commit_test", newMockTickerFunc(false), newKVStore)
	defer cleanup()
	// override default TimeoutCommit == 0 for tests
	for i := 0; i < n; i++ {
		css[i].config.TimeoutCommit = 1 * time.Second
		css[i].config.SkipTimeoutCommit = false //nolint:staticcheck
	}

	reactors, blocksSubs, eventBuses := startConsensusNet(t, css, n-1)
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// wait till everyone makes the first new block
	timeoutWaitGroup(n-1, func(j int) {
		<-blocksSubs[j].Out()
	})
}

func waitForAndValidateBlock(
	t *testing.T,
	n int,
	activeVals map[string]struct{},
	blocksSubs []types.Subscription,
	css []*State,
	txs ...[]byte,
) {
	t.Helper()
	timeoutWaitGroup(n, func(j int) {
		css[j].Logger.Debug("waitForAndValidateBlock")
		msg := <-blocksSubs[j].Out()
		newBlock := msg.Data().(types.EventDataNewBlock).Block
		css[j].Logger.Debug("waitForAndValidateBlock: Got block", "height", newBlock.Height)
		err := validateBlock(newBlock, activeVals)
		require.NoError(t, err)

		// optionally add transactions for the next block
		for _, tx := range txs {
			reqRes, err := assertMempool(css[j].txNotifier).CheckTx(tx, "")
			require.NoError(t, err)
			require.False(t, reqRes.Response.GetCheckTx().IsErr())
		}
	})
}

func waitForAndValidateBlockWithTx(
	t *testing.T,
	n int,
	activeVals map[string]struct{},
	blocksSubs []types.Subscription,
	css []*State,
	txs ...[]byte,
) {
	t.Helper()
	timeoutWaitGroup(n, func(j int) {
		ntxs := 0
	BLOCK_TX_LOOP:
		for {
			css[j].Logger.Debug("waitForAndValidateBlockWithTx", "ntxs", ntxs)
			msg := <-blocksSubs[j].Out()
			newBlock := msg.Data().(types.EventDataNewBlock).Block
			css[j].Logger.Debug("waitForAndValidateBlockWithTx: Got block", "height", newBlock.Height)
			err := validateBlock(newBlock, activeVals)
			require.NoError(t, err)

			// check that txs match the txs we're waiting for.
			// note they could be spread over multiple blocks,
			// but they should be in order.
			for _, tx := range newBlock.Data.Txs {
				assert.EqualValues(t, txs[ntxs], tx)
				ntxs++
			}

			if ntxs == len(txs) {
				break BLOCK_TX_LOOP
			}
		}
	})
}

func waitForBlockWithUpdatedValsAndValidateIt(
	t *testing.T,
	n int,
	updatedVals map[string]struct{},
	blocksSubs []types.Subscription,
	css []*State,
) {
	t.Helper()
	timeoutWaitGroup(n, func(j int) {
		var newBlock *types.Block
	LOOP:
		for {
			css[j].Logger.Debug("waitForBlockWithUpdatedValsAndValidateIt")
			msg := <-blocksSubs[j].Out()
			newBlock = msg.Data().(types.EventDataNewBlock).Block
			if newBlock.LastCommit.Size() == len(updatedVals) {
				css[j].Logger.Debug("waitForBlockWithUpdatedValsAndValidateIt: Got block", "height", newBlock.Height)
				break LOOP
			}
			css[j].Logger.Debug(
				"waitForBlockWithUpdatedValsAndValidateIt: Got block with no new validators. Skipping",
				"height", newBlock.Height, "last_commit", newBlock.LastCommit.Size(), "updated_vals", len(updatedVals),
			)
		}

		err := validateBlock(newBlock, updatedVals)
		require.NoError(t, err)
	})
}

// expects high synchrony!
func validateBlock(block *types.Block, activeVals map[string]struct{}) error {
	if block.LastCommit.Size() != len(activeVals) {
		return fmt.Errorf(
			"commit size doesn't match number of active validators. Got %d, expected %d",
			block.LastCommit.Size(),
			len(activeVals))
	}

	for _, commitSig := range block.LastCommit.Signatures {
		if _, ok := activeVals[string(commitSig.ValidatorAddress)]; !ok {
			return fmt.Errorf("found vote for inactive validator %X", commitSig.ValidatorAddress)
		}
	}
	return nil
}

func timeoutWaitGroup(n int, f func(int)) {
	wg := new(sync.WaitGroup)
	wg.Add(n)
	for i := 0; i < n; i++ {
		go func(j int) {
			f(j)
			wg.Done()
		}(i)
	}

	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	// we're running many nodes in-process, possibly in in a virtual machine,
	// and spewing debug messages - making a block could take a while,
	timeout := time.Second * 20

	select {
	case <-done:
	case <-time.After(timeout):
		panic("Timed out waiting for all validators to commit a block")
	}
}

// -------------------------------------------------------------
// Ensure basic validation of structs is functioning

func TestNewRoundStepMessageValidateBasic(t *testing.T) {
	testCases := []struct { //nolint: maligned
		expectErr              bool
		messageRound           int32
		messageLastCommitRound int32
		messageHeight          int64
		testName               string
		messageStep            cstypes.RoundStepType
	}{
		{false, 0, 0, 0, "Valid Message", cstypes.RoundStepNewHeight},
		{true, -1, 0, 0, "Negative round", cstypes.RoundStepNewHeight},
		{true, 0, 0, -1, "Negative height", cstypes.RoundStepNewHeight},
		{true, 0, 0, 0, "Invalid Step", cstypes.RoundStepCommit + 1},
		// The following cases will be handled by ValidateHeight
		{false, 0, 0, 1, "H == 1 but LCR != -1 ", cstypes.RoundStepNewHeight},
		{false, 0, -1, 2, "H > 1 but LCR < 0", cstypes.RoundStepNewHeight},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			message := NewRoundStepMessage{
				Height:          tc.messageHeight,
				Round:           tc.messageRound,
				Step:            tc.messageStep,
				LastCommitRound: tc.messageLastCommitRound,
			}

			err := message.ValidateBasic()
			if tc.expectErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestNewRoundStepMessageValidateHeight(t *testing.T) {
	initialHeight := int64(10)
	testCases := []struct { //nolint: maligned
		expectErr              bool
		messageLastCommitRound int32
		messageHeight          int64
		testName               string
	}{
		{false, 0, 11, "Valid Message"},
		{true, 0, -1, "Negative height"},
		{true, 0, 0, "Zero height"},
		{true, 0, 10, "Initial height but LCR != -1 "},
		{true, -1, 11, "Normal height but LCR < 0"},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			message := NewRoundStepMessage{
				Height:          tc.messageHeight,
				Round:           0,
				Step:            cstypes.RoundStepNewHeight,
				LastCommitRound: tc.messageLastCommitRound,
			}

			err := message.ValidateHeight(initialHeight)
			if tc.expectErr {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
			}
		})
	}
}

func TestNewValidBlockMessageValidateBasic(t *testing.T) {
	testCases := []struct {
		malleateFn func(*NewValidBlockMessage)
		expErr     string
	}{
		{func(_ *NewValidBlockMessage) {}, ""},
		{func(msg *NewValidBlockMessage) { msg.Height = -1 }, cmterrors.ErrNegativeField{Field: "Height"}.Error()},
		{func(msg *NewValidBlockMessage) { msg.Round = -1 }, cmterrors.ErrNegativeField{Field: "Round"}.Error()},
		{
			func(msg *NewValidBlockMessage) { msg.BlockPartSetHeader.Total = 2 },
			"blockParts bit array size 1 not equal to BlockPartSetHeader.Total 2",
		},
		{
			func(msg *NewValidBlockMessage) {
				msg.BlockPartSetHeader.Total = 0
				msg.BlockParts = bits.NewBitArray(0)
			},
			cmterrors.ErrRequiredField{Field: "blockParts"}.Error(),
		},
		{
			func(msg *NewValidBlockMessage) { msg.BlockParts = bits.NewBitArray(int(types.MaxBlockPartsCount) + 1) },
			"blockParts bit array size 1602 not equal to BlockPartSetHeader.Total 1",
		},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("#%d", i), func(t *testing.T) {
			msg := &NewValidBlockMessage{
				Height: 1,
				Round:  0,
				BlockPartSetHeader: types.PartSetHeader{
					Total: 1,
				},
				BlockParts: bits.NewBitArray(1),
			}

			tc.malleateFn(msg)
			err := msg.ValidateBasic()
			if tc.expErr != "" && assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
				assert.Contains(t, err.Error(), tc.expErr)
			}
		})
	}
}

func TestProposalPOLMessageValidateBasic(t *testing.T) {
	testCases := []struct {
		malleateFn func(*ProposalPOLMessage)
		expErr     string
	}{
		{func(_ *ProposalPOLMessage) {}, ""},
		{func(msg *ProposalPOLMessage) { msg.Height = -1 }, cmterrors.ErrNegativeField{Field: "Height"}.Error()},
		{func(msg *ProposalPOLMessage) { msg.ProposalPOLRound = -1 }, cmterrors.ErrNegativeField{Field: "ProposalPOLRound"}.Error()},
		{func(msg *ProposalPOLMessage) { msg.ProposalPOL = bits.NewBitArray(0) }, cmterrors.ErrRequiredField{Field: "ProposalPOL"}.Error()},
		{
			func(msg *ProposalPOLMessage) { msg.ProposalPOL = bits.NewBitArray(types.MaxVotesCount + 1) },
			"proposalPOL bit array is too big: 10001, max: 10000",
		},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("#%d", i), func(t *testing.T) {
			msg := &ProposalPOLMessage{
				Height:           1,
				ProposalPOLRound: 1,
				ProposalPOL:      bits.NewBitArray(1),
			}

			tc.malleateFn(msg)
			err := msg.ValidateBasic()
			if tc.expErr != "" && assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
				assert.Contains(t, err.Error(), tc.expErr)
			}
		})
	}
}

func TestBlockPartMessageValidateBasic(t *testing.T) {
	testPart := new(types.Part)
	testPart.Proof.LeafHash = tmhash.Sum([]byte("leaf"))
	testCases := []struct {
		testName      string
		messageHeight int64
		messageRound  int32
		messagePart   *types.Part
		expectErr     bool
	}{
		{"Valid Message", 0, 0, testPart, false},
		{"Invalid Message", -1, 0, testPart, true},
		{"Invalid Message", 0, -1, testPart, true},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			message := BlockPartMessage{
				Height: tc.messageHeight,
				Round:  tc.messageRound,
				Part:   tc.messagePart,
			}

			assert.Equal(t, tc.expectErr, message.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}

	message := BlockPartMessage{Height: 0, Round: 0, Part: new(types.Part)}
	message.Part.Index = 1

	require.Error(t, message.ValidateBasic())
}

func TestHasVoteMessageValidateBasic(t *testing.T) {
	const (
		validSignedMsgType   types.SignedMsgType = 0x01
		invalidSignedMsgType types.SignedMsgType = 0x03
	)

	testCases := []struct { //nolint: maligned
		expectErr     bool
		messageRound  int32
		messageIndex  int32
		messageHeight int64
		testName      string
		messageType   types.SignedMsgType
	}{
		{false, 0, 0, 0, "Valid Message", validSignedMsgType},
		{true, -1, 0, 0, "Invalid Message", validSignedMsgType},
		{true, 0, -1, 0, "Invalid Message", validSignedMsgType},
		{true, 0, 0, 0, "Invalid Message", invalidSignedMsgType},
		{true, 0, 0, -1, "Invalid Message", validSignedMsgType},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			message := HasVoteMessage{
				Height: tc.messageHeight,
				Round:  tc.messageRound,
				Type:   tc.messageType,
				Index:  tc.messageIndex,
			}

			assert.Equal(t, tc.expectErr, message.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestVoteSetMaj23MessageValidateBasic(t *testing.T) {
	const (
		validSignedMsgType   types.SignedMsgType = 0x01
		invalidSignedMsgType types.SignedMsgType = 0x03
	)

	validBlockID := types.BlockID{}
	invalidBlockID := types.BlockID{
		Hash: bytes.HexBytes{},
		PartSetHeader: types.PartSetHeader{
			Total: 1,
			Hash:  []byte{0},
		},
	}

	testCases := []struct { //nolint: maligned
		expectErr      bool
		messageRound   int32
		messageHeight  int64
		testName       string
		messageType    types.SignedMsgType
		messageBlockID types.BlockID
	}{
		{false, 0, 0, "Valid Message", validSignedMsgType, validBlockID},
		{true, -1, 0, "Invalid Message", validSignedMsgType, validBlockID},
		{true, 0, -1, "Invalid Message", validSignedMsgType, validBlockID},
		{true, 0, 0, "Invalid Message", invalidSignedMsgType, validBlockID},
		{true, 0, 0, "Invalid Message", validSignedMsgType, invalidBlockID},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			message := VoteSetMaj23Message{
				Height:  tc.messageHeight,
				Round:   tc.messageRound,
				Type:    tc.messageType,
				BlockID: tc.messageBlockID,
			}

			assert.Equal(t, tc.expectErr, message.ValidateBasic() != nil, "Validate Basic had an unexpected result")
		})
	}
}

func TestVoteSetBitsMessageValidateBasic(t *testing.T) {
	testCases := []struct {
		malleateFn func(*VoteSetBitsMessage)
		expErr     string
	}{
		{func(_ *VoteSetBitsMessage) {}, ""},
		{func(msg *VoteSetBitsMessage) { msg.Height = -1 }, cmterrors.ErrNegativeField{Field: "Height"}.Error()},
		{func(msg *VoteSetBitsMessage) { msg.Type = 0x03 }, cmterrors.ErrInvalidField{Field: "Type"}.Error()},
		{func(msg *VoteSetBitsMessage) {
			msg.BlockID = types.BlockID{
				Hash: bytes.HexBytes{},
				PartSetHeader: types.PartSetHeader{
					Total: 1,
					Hash:  []byte{0},
				},
			}
		}, "wrong BlockID: wrong PartSetHeader: wrong Hash:"},
		{
			func(msg *VoteSetBitsMessage) { msg.Votes = bits.NewBitArray(types.MaxVotesCount + 1) },
			"votes bit array is too big: 10001, max: 10000",
		},
	}

	for i, tc := range testCases {
		t.Run(fmt.Sprintf("#%d", i), func(t *testing.T) {
			msg := &VoteSetBitsMessage{
				Height:  1,
				Round:   0,
				Type:    0x01,
				Votes:   bits.NewBitArray(1),
				BlockID: types.BlockID{},
			}

			tc.malleateFn(msg)
			err := msg.ValidateBasic()
			if tc.expErr != "" && assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
				assert.Contains(t, err.Error(), tc.expErr)
			}
		})
	}
}

func TestMarshalJSONPeerState(t *testing.T) {
	ps := NewPeerState(nil)
	data, err := json.Marshal(ps)
	require.NoError(t, err)
	require.JSONEq(t, `{
		"round_state":{
			"height": "0",
			"round": -1,
			"step": 0,
			"start_time": "0001-01-01T00:00:00Z",
			"proposal": false,
			"proposal_block_part_set_header":
				{"total":0, "hash":""},
			"proposal_block_parts": null,
			"proposal_pol_round": -1,
			"proposal_pol": null,
			"prevotes": null,
			"precommits": null,
			"last_commit_round": -1,
			"last_commit": null,
			"catchup_commit_round": -1,
			"catchup_commit": null
		},
		"stats":{
			"votes":"0",
			"block_parts":"0"}
		}`, string(data))
}
