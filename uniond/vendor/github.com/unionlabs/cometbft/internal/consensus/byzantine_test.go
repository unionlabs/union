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
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	abcicli "github.com/cometbft/cometbft/abci/client"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtcons "github.com/cometbft/cometbft/api/cometbft/consensus/v1"
	"github.com/cometbft/cometbft/internal/evidence"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// ----------------------------------------------
// byzantine failures

// Byzantine node sends two different prevotes (nil and blockID) to the same validator.
func TestByzantinePrevoteEquivocation(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	const nValidators = 4
	const byzantineNode = 0
	const prevoteHeight = int64(2)
	testName := "consensus_byzantine_test"
	tickerFunc := newMockTickerFunc(true)
	appFunc := newKVStore

	genDoc, privVals := randGenesisDoc(nValidators, 30, nil, cmttime.Now())
	css := make([]*State, nValidators)

	for i := 0; i < nValidators; i++ {
		logger := consensusLogger().With("test", "byzantine", "validator", i)
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

		blockDB := dbm.NewMemDB()
		blockStore := store.NewBlockStore(blockDB)

		mtx := new(cmtsync.Mutex)
		// one for mempool, one for consensus
		proxyAppConnCon := proxy.NewAppConnConsensus(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())
		proxyAppConnMem := proxy.NewAppConnMempool(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())

		// Make Mempool
		mempool := mempl.NewCListMempool(config.Mempool,
			proxyAppConnMem,
			state.LastBlockHeight,
			mempl.WithPreCheck(sm.TxPreCheck(state)),
			mempl.WithPostCheck(sm.TxPostCheck(state)))

		if thisConfig.Consensus.WaitForTxs() {
			mempool.EnableTxsAvailable()
		}

		// Make a full instance of the evidence pool
		evidenceDB := dbm.NewMemDB()
		evpool, err := evidence.NewPool(evidenceDB, stateStore, blockStore)
		require.NoError(t, err)
		evpool.SetLogger(logger.With("module", "evidence"))

		// Make State
		blockExec := sm.NewBlockExecutor(stateStore, log.TestingLogger(), proxyAppConnCon, mempool, evpool, blockStore)
		cs := NewState(thisConfig.Consensus, state, blockExec, blockStore, mempool, evpool)
		cs.SetLogger(cs.Logger)
		// set private validator
		pv := privVals[i]
		cs.SetPrivValidator(pv)

		eventBus := types.NewEventBus()
		eventBus.SetLogger(log.TestingLogger().With("module", "events"))
		err = eventBus.Start()
		require.NoError(t, err)
		cs.SetEventBus(eventBus)

		cs.SetTimeoutTicker(tickerFunc())
		cs.SetLogger(logger)

		css[i] = cs
	}

	// initialize the reactors for each of the validators
	reactors := make([]*Reactor, nValidators)
	blocksSubs := make([]types.Subscription, 0)
	eventBuses := make([]*types.EventBus, nValidators)
	for i := 0; i < nValidators; i++ {
		reactors[i] = NewReactor(css[i], true) // so we dont start the consensus states
		reactors[i].SetLogger(css[i].Logger)

		// eventBus is already started with the cs
		eventBuses[i] = css[i].eventBus
		reactors[i].SetEventBus(eventBuses[i])

		blocksSub, err := eventBuses[i].Subscribe(context.Background(), testSubscriber, types.EventQueryNewBlock, 100)
		require.NoError(t, err)
		blocksSubs = append(blocksSubs, blocksSub)

		if css[i].state.LastBlockHeight == 0 { // simulate handle initChain in handshake
			err = css[i].blockExec.Store().Save(css[i].state)
			require.NoError(t, err)
		}
	}
	// make connected switches and start all reactors
	p2p.MakeConnectedSwitches(config.P2P, nValidators, func(i int, s *p2p.Switch) *p2p.Switch {
		s.AddReactor("CONSENSUS", reactors[i])
		s.SetLogger(reactors[i].conS.Logger.With("module", "p2p"))
		return s
	}, p2p.Connect2Switches)

	// create byzantine validator
	bcs := css[byzantineNode]

	// alter prevote so that the byzantine node double votes when height is 2
	bcs.doPrevote = func(height int64, round int32) {
		// allow first height to happen normally so that byzantine validator is no longer proposer
		if height == prevoteHeight {
			bcs.Logger.Info("Sending two votes")
			prevote1, err := bcs.signVote(types.PrevoteType, bcs.ProposalBlock.Hash(), bcs.ProposalBlockParts.Header(), bcs.ProposalBlock)
			require.NoError(t, err)
			prevote2, err := bcs.signVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
			require.NoError(t, err)
			peerList := reactors[byzantineNode].Switch.Peers().Copy()
			bcs.Logger.Info("Getting peer list", "peers", peerList)
			// send two votes to all peers (1st to one half, 2nd to another half)
			for i, peer := range peerList {
				if i < len(peerList)/2 {
					bcs.Logger.Info("Signed and pushed vote", "vote", prevote1, "peer", peer)
					peer.Send(p2p.Envelope{
						Message:   &cmtcons.Vote{Vote: prevote1.ToProto()},
						ChannelID: VoteChannel,
					})
				} else {
					bcs.Logger.Info("Signed and pushed vote", "vote", prevote2, "peer", peer)
					peer.Send(p2p.Envelope{
						Message:   &cmtcons.Vote{Vote: prevote2.ToProto()},
						ChannelID: VoteChannel,
					})
				}
			}
		} else {
			bcs.Logger.Info("Behaving normally")
			bcs.defaultDoPrevote(height, round)
		}
	}

	// introducing a lazy proposer means that the time of the block committed is different to the
	// timestamp that the other nodes have. This tests to ensure that the evidence that finally gets
	// proposed will have a valid timestamp
	lazyProposer := css[1]

	lazyProposer.decideProposal = func(height int64, round int32) {
		lazyProposer.Logger.Info("Lazy Proposer proposing condensed commit")
		if lazyProposer.privValidator == nil {
			panic("entered createProposalBlock with privValidator being nil")
		}

		var extCommit *types.ExtendedCommit
		switch {
		case lazyProposer.Height == lazyProposer.state.InitialHeight:
			// We're creating a proposal for the first block.
			// The commit is empty, but not nil.
			extCommit = &types.ExtendedCommit{}
		case lazyProposer.LastCommit.HasTwoThirdsMajority():
			// Make the commit from LastCommit
			// Vote extensions are enabled by default for test units
			veHeightParam := lazyProposer.state.ConsensusParams.Feature
			extCommit = lazyProposer.LastCommit.MakeExtendedCommit(veHeightParam)
		default: // This shouldn't happen.
			lazyProposer.Logger.Error("enterPropose: Cannot propose anything: No commit for the previous block")
			return
		}

		// omit the last signature in the commit
		extCommit.ExtendedSignatures[len(extCommit.ExtendedSignatures)-1] = types.NewExtendedCommitSigAbsent()

		if lazyProposer.privValidatorPubKey == nil {
			// If this node is a validator & proposer in the current round, it will
			// miss the opportunity to create a block.
			lazyProposer.Logger.Error(fmt.Sprintf("enterPropose: %v", ErrPubKeyIsNotSet))
			return
		}
		proposerAddr := lazyProposer.privValidatorPubKey.Address()

		block, err := lazyProposer.blockExec.CreateProposalBlock(
			ctx, lazyProposer.Height, lazyProposer.state, extCommit, proposerAddr)
		require.NoError(t, err)
		blockParts, err := block.MakePartSet(types.BlockPartSizeBytes)
		require.NoError(t, err)

		// Flush the WAL. Otherwise, we may not recompute the same proposal to sign,
		// and the privValidator will refuse to sign anything.
		if err := lazyProposer.wal.FlushAndSync(); err != nil {
			lazyProposer.Logger.Error("Error flushing to disk")
		}

		// Make proposal
		propBlockID := types.BlockID{Hash: block.Hash(), PartSetHeader: blockParts.Header()}
		proposal := types.NewProposal(height, round, lazyProposer.ValidRound, propBlockID, block.Header.Time)
		p := proposal.ToProto()
		if err := lazyProposer.privValidator.SignProposal(lazyProposer.state.ChainID, p); err == nil {
			proposal.Signature = p.Signature

			// send proposal and block parts on internal msg queue
			lazyProposer.sendInternalMessage(msgInfo{&ProposalMessage{proposal}, "", cmttime.Now()})
			for i := 0; i < int(blockParts.Total()); i++ {
				part := blockParts.GetPart(i)
				lazyProposer.sendInternalMessage(msgInfo{&BlockPartMessage{lazyProposer.Height, lazyProposer.Round, part}, "", time.Time{}})
			}
			lazyProposer.Logger.Info("Signed proposal", "height", height, "round", round, "proposal", proposal)
			lazyProposer.Logger.Debug(fmt.Sprintf("Signed proposal block: %v", block))
		} else if !lazyProposer.replayMode {
			lazyProposer.Logger.Error("enterPropose: Error signing proposal", "height", height, "round", round, "err", err)
		}
	}

	// start the consensus reactors
	for i := 0; i < nValidators; i++ {
		s := reactors[i].conS.GetState()
		reactors[i].SwitchToConsensus(s, false)
	}
	defer stopConsensusNet(log.TestingLogger(), reactors, eventBuses)

	// Evidence should be submitted and committed at the third height but
	// we will check the first six just in case
	evidenceFromEachValidator := make([]types.Evidence, nValidators)

	wg := new(sync.WaitGroup)
	for i := 0; i < nValidators; i++ {
		wg.Add(1)
		go func(i int) {
			defer wg.Done()
			for msg := range blocksSubs[i].Out() {
				block := msg.Data().(types.EventDataNewBlock).Block
				if len(block.Evidence.Evidence) != 0 {
					evidenceFromEachValidator[i] = block.Evidence.Evidence[0]
					return
				}
			}
		}(i)
	}

	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	pubkey, err := bcs.privValidator.GetPubKey()
	require.NoError(t, err)

	select {
	case <-done:
		for idx, ev := range evidenceFromEachValidator {
			if assert.NotNil(t, ev, idx) {
				ev, ok := ev.(*types.DuplicateVoteEvidence)
				assert.True(t, ok)
				assert.Equal(t, pubkey.Address(), ev.VoteA.ValidatorAddress)
				assert.Equal(t, prevoteHeight, ev.Height())
			}
		}
	case <-time.After(20 * time.Second):
		t.Fatalf("Timed out waiting for validators to commit evidence")
	}
}

// 4 validators. 1 is byzantine. The other three are partitioned into A (1 val) and B (2 vals).
// byzantine validator sends conflicting proposals into A and B,
// and prevotes/precommits on both of them.
// B sees a commit, A doesn't.
// Heal partition and ensure A sees the commit.
func TestByzantineConflictingProposalsWithPartition(t *testing.T) {
	n := 4
	logger := consensusLogger().With("test", "byzantine")

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	app := newKVStore
	css, cleanup := randConsensusNet(t, n, "consensus_byzantine_test", newMockTickerFunc(false), app)
	defer cleanup()

	// give the byzantine validator a normal ticker
	ticker := NewTimeoutTicker()
	ticker.SetLogger(css[0].Logger)
	css[0].SetTimeoutTicker(ticker)

	switches := make([]*p2p.Switch, n)
	p2pLogger := logger.With("module", "p2p")
	for i := 0; i < n; i++ {
		switches[i] = p2p.MakeSwitch(
			config.P2P,
			i,
			func(_ int, sw *p2p.Switch) *p2p.Switch {
				return sw
			})
		switches[i].SetLogger(p2pLogger.With("validator", i))
	}

	blocksSubs := make([]types.Subscription, n)
	reactors := make([]p2p.Reactor, n)
	for i := 0; i < n; i++ {
		// enable txs so we can create different proposals
		assertMempool(css[i].txNotifier).EnableTxsAvailable()
		// make first val byzantine
		if i == 0 {
			// NOTE: Now, test validators are MockPV, which by default doesn't
			// do any safety checks.
			css[i].privValidator.(types.MockPV).DisableChecks()
			j := i
			css[i].decideProposal = func(height int64, round int32) {
				byzantineDecideProposalFunc(ctx, t, height, round, css[j], switches[j])
			}
			// We are setting the prevote function to do nothing because the prevoting
			// and precommitting are done alongside the proposal.
			css[i].doPrevote = func(_ int64, _ int32) {}
		}

		eventBus := css[i].eventBus
		eventBus.SetLogger(logger.With("module", "events", "validator", i))

		var err error
		blocksSubs[i], err = eventBus.Subscribe(context.Background(), testSubscriber, types.EventQueryNewBlock)
		require.NoError(t, err)

		conR := NewReactor(css[i], true) // so we don't start the consensus states
		conR.SetLogger(logger.With("validator", i))
		conR.SetEventBus(eventBus)

		var conRI p2p.Reactor = conR

		// make first val byzantine
		if i == 0 {
			conRI = NewByzantineReactor(conR)
		}

		reactors[i] = conRI
		err = css[i].blockExec.Store().Save(css[i].state) // for save height 1's validators info
		require.NoError(t, err)
	}

	defer func() {
		for _, r := range reactors {
			if rr, ok := r.(*ByzantineReactor); ok {
				err := rr.reactor.Switch.Stop()
				require.NoError(t, err)
			} else {
				err := r.(*Reactor).Switch.Stop()
				require.NoError(t, err)
			}
		}
	}()

	p2p.MakeConnectedSwitches(config.P2P, n, func(i int, _ *p2p.Switch) *p2p.Switch {
		// ignore new switch s, we already made ours
		switches[i].AddReactor("CONSENSUS", reactors[i])
		return switches[i]
	}, func(sws []*p2p.Switch, i, j int) {
		// the network starts partitioned with globally active adversary
		if i != 0 {
			return
		}
		p2p.Connect2Switches(sws, i, j)
	})

	// start the non-byz state machines.
	// note these must be started before the byz
	for i := 1; i < n; i++ {
		cr := reactors[i].(*Reactor)
		cr.SwitchToConsensus(cr.conS.GetState(), false)
	}

	// start the byzantine state machine
	byzR := reactors[0].(*ByzantineReactor)
	s := byzR.reactor.conS.GetState()
	byzR.reactor.SwitchToConsensus(s, false)

	// byz proposer sends one block to peers[0]
	// and the other block to peers[1] and peers[2].
	// note peers and switches order don't match.
	peers := switches[0].Peers().Copy()

	// partition A
	ind0 := getSwitchIndex(switches, peers[0])

	// partition B
	ind1 := getSwitchIndex(switches, peers[1])
	ind2 := getSwitchIndex(switches, peers[2])
	p2p.Connect2Switches(switches, ind1, ind2)

	// wait for someone in the big partition (B) to make a block
	<-blocksSubs[ind2].Out()

	t.Log("A block has been committed. Healing partition")
	p2p.Connect2Switches(switches, ind0, ind1)
	p2p.Connect2Switches(switches, ind0, ind2)

	// wait till everyone makes the first new block
	// (one of them already has)
	wg := new(sync.WaitGroup)
	for i := 1; i < n-1; i++ {
		wg.Add(1)
		go func(j int) {
			<-blocksSubs[j].Out()
			wg.Done()
		}(i)
	}

	done := make(chan struct{})
	go func() {
		wg.Wait()
		close(done)
	}()

	tick := time.NewTicker(time.Second * 10)
	select {
	case <-done:
	case <-tick.C:
		for i, reactor := range reactors {
			t.Logf("Consensus Reactor %v", i)
			t.Logf("%v", reactor)
		}
		t.Fatalf("Timed out waiting for all validators to commit first block")
	}
}

// -------------------------------
// byzantine consensus functions

func byzantineDecideProposalFunc(_ context.Context, t *testing.T, height int64, round int32, cs *State, sw *p2p.Switch) {
	t.Helper()
	// byzantine user should create two proposals and try to split the vote.
	// Avoid sending on internalMsgQueue and running consensus state.

	// Create a new proposal block from state/txs from the mempool.
	block1, blockParts1, propBlockID := createProposalBlock(t, cs)
	polRound := cs.ValidRound
	proposal1 := types.NewProposal(height, round, polRound, propBlockID, block1.Time)
	p1 := proposal1.ToProto()
	if err := cs.privValidator.SignProposal(cs.state.ChainID, p1); err != nil {
		t.Error(err)
	}

	proposal1.Signature = p1.Signature

	// some new transactions come in (this ensures that the proposals are different)
	deliverTxsRange(t, cs, 1)

	// Create a new proposal block from state/txs from the mempool.
	block2, blockParts2, propBlockID := createProposalBlock(t, cs)
	polRound = cs.ValidRound
	proposal2 := types.NewProposal(height, round, polRound, propBlockID, block2.Time)
	p2 := proposal2.ToProto()
	if err := cs.privValidator.SignProposal(cs.state.ChainID, p2); err != nil {
		t.Error(err)
	}

	proposal2.Signature = p2.Signature

	block1Hash := block1.Hash()
	block2Hash := block2.Hash()
	require.NotEqual(t, block1Hash, block2Hash)

	// broadcast conflicting proposals/block parts to peers
	peers := sw.Peers().Copy()
	t.Logf("Byzantine: broadcasting conflicting proposals to %d peers", len(peers))
	for i, peer := range peers {
		if i < len(peers)/2 {
			go sendProposalAndParts(height, round, cs, peer, proposal1, block1, block1Hash, blockParts1)
		} else {
			go sendProposalAndParts(height, round, cs, peer, proposal2, block2, block2Hash, blockParts2)
		}
	}
}

func sendProposalAndParts(
	height int64,
	round int32,
	cs *State,
	peer p2p.Peer,
	proposal *types.Proposal,
	block *types.Block,
	blockHash []byte,
	parts *types.PartSet,
) {
	// proposal
	peer.Send(p2p.Envelope{
		ChannelID: DataChannel,
		Message:   &cmtcons.Proposal{Proposal: *proposal.ToProto()},
	})

	// parts
	for i := 0; i < int(parts.Total()); i++ {
		part := parts.GetPart(i)
		pp, err := part.ToProto()
		if err != nil {
			panic(err) // TODO: wbanfield better error handling
		}
		peer.Send(p2p.Envelope{
			ChannelID: DataChannel,
			Message: &cmtcons.BlockPart{
				Height: height, // This tells peer that this part applies to us.
				Round:  round,  // This tells peer that this part applies to us.
				Part:   *pp,
			},
		})
	}

	// votes
	cs.mtx.Lock()
	prevote, _ := cs.signVote(types.PrevoteType, blockHash, parts.Header(), nil)
	precommit, _ := cs.signVote(types.PrecommitType, blockHash, parts.Header(), block)
	cs.mtx.Unlock()
	peer.Send(p2p.Envelope{
		ChannelID: VoteChannel,
		Message:   &cmtcons.Vote{Vote: prevote.ToProto()},
	})
	peer.Send(p2p.Envelope{
		ChannelID: VoteChannel,
		Message:   &cmtcons.Vote{Vote: precommit.ToProto()},
	})
}

// ----------------------------------------
// byzantine consensus reactor

type ByzantineReactor struct {
	service.Service
	reactor *Reactor
}

func NewByzantineReactor(conR *Reactor) *ByzantineReactor {
	return &ByzantineReactor{
		Service: conR,
		reactor: conR,
	}
}

func (br *ByzantineReactor) SetSwitch(s *p2p.Switch)               { br.reactor.SetSwitch(s) }
func (br *ByzantineReactor) GetChannels() []*p2p.ChannelDescriptor { return br.reactor.GetChannels() }
func (br *ByzantineReactor) AddPeer(peer p2p.Peer) {
	if !br.reactor.IsRunning() {
		return
	}

	// Create peerState for peer
	peerState := NewPeerState(peer).SetLogger(br.reactor.Logger)
	peer.Set(types.PeerStateKey, peerState)

	// Send our state to peer.
	// If we're syncing, broadcast a RoundStepMessage later upon SwitchToConsensus().
	if !br.reactor.WaitSync() {
		br.reactor.sendNewRoundStepMessage(peer)
	}
}

func (br *ByzantineReactor) RemovePeer(peer p2p.Peer, reason any) {
	br.reactor.RemovePeer(peer, reason)
}

// Receive forwards all messages to the underlying reactor.
func (br *ByzantineReactor) Receive(e p2p.Envelope) {
	br.reactor.Receive(e)
}

func (*ByzantineReactor) InitPeer(peer p2p.Peer) p2p.Peer { return peer }
