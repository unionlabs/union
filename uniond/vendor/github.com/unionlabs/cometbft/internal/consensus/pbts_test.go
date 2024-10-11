package consensus

import (
	"bytes"
	"context"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	abcimocks "github.com/cometbft/cometbft/abci/types/mocks"
	"github.com/cometbft/cometbft/internal/test"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
	cmttimemocks "github.com/cometbft/cometbft/types/time/mocks"
)

const (
	// blockTimeIota is used in the test harness as the time between
	// blocks when not otherwise specified.
	blockTimeIota = time.Millisecond
)

// pbtsTestHarness constructs a CometBFT network that can be used for testing the
// implementation of the Proposer-Based timestamps algorithm.
// It runs a series of consensus heights and captures timing of votes and events.
type pbtsTestHarness struct {
	// configuration options set by the user of the test harness.
	pbtsTestConfiguration

	// The timestamp of the first block produced by the network.
	firstBlockTime time.Time

	// The CometBFT consensus state machine being run during
	// a run of the pbtsTestHarness.
	observedState *State

	// A stub for signing votes and messages using the key
	// from the observedState.
	observedValidator *validatorStub

	// A list of simulated validators that interact with the observedState and are
	// fully controlled by the test harness.
	otherValidators []*validatorStub

	// The mock time source used by all of the validator stubs in the test harness.
	// This mock clock allows the test harness to produce votes and blocks with arbitrary
	// timestamps.
	validatorClock *cmttimemocks.Source

	chainID string

	// channels for verifying that the observed validator completes certain actions.
	ensureProposalCh, roundCh, blockCh, ensureVoteCh <-chan cmtpubsub.Message

	// channel of events from the observed validator annotated with the timestamp
	// the event was received.
	eventCh <-chan timestampedEvent

	currentHeight int64
	currentRound  int32
}

type pbtsTestConfiguration struct {
	// The timestamp consensus parameters to be used by the state machine under test.
	synchronyParams types.SynchronyParams

	// The setting to use for the TimeoutPropose configuration parameter.
	timeoutPropose time.Duration

	// The genesis time
	genesisTime time.Time

	// The times offset from height 1 block time of the block proposed at height 2.
	height2ProposedBlockOffset time.Duration

	// The time offset from height 1 block time at which the proposal at height 2 should be delivered.
	height2ProposalTimeDeliveryOffset time.Duration

	// The time offset from height 1 block time of the block proposed at height 4.
	// At height 4, the proposed block and the deliver offsets are the same so
	// that timely-ness does not affect height 4.
	height4ProposedBlockOffset time.Duration
}

func newPBTSTestHarness(ctx context.Context, t *testing.T, tc pbtsTestConfiguration) pbtsTestHarness {
	t.Helper()
	const validators = 4
	cfg := test.ResetTestRoot("newPBTSTestHarness")
	clock := new(cmttimemocks.Source)

	if tc.genesisTime.IsZero() {
		tc.genesisTime = cmttime.Now()
	}

	if tc.height4ProposedBlockOffset == 0 {
		// Set a default height4ProposedBlockOffset.
		// Use a proposed block time that is greater than the time that the
		// block at height 2 was delivered. Height 3 is not relevant for testing
		// and always occurs blockTimeIota before height 4. If not otherwise specified,
		// height 4 therefore occurs 2*blockTimeIota after height 2.
		tc.height4ProposedBlockOffset = tc.height2ProposalTimeDeliveryOffset + 2*blockTimeIota
	}
	cfg.Consensus.TimeoutPropose = tc.timeoutPropose
	consensusParams := types.DefaultConsensusParams()
	consensusParams.Synchrony = tc.synchronyParams
	consensusParams.Feature.PbtsEnableHeight = 1

	state, privVals := randGenesisStateWithTime(validators, consensusParams, tc.genesisTime)
	cs := newStateWithConfig(cfg, state, privVals[0], kvstore.NewInMemoryApplication())
	vss := make([]*validatorStub, validators)
	for i := 0; i < validators; i++ {
		vss[i] = newValidatorStub(privVals[i], int32(i))
	}
	incrementHeight(vss[1:]...)

	for _, vs := range vss {
		vs.clock = clock
	}
	pubKey, err := vss[0].PrivValidator.GetPubKey()
	require.NoError(t, err)

	eventCh := timestampedCollector(ctx, t, cs.eventBus)

	return pbtsTestHarness{
		pbtsTestConfiguration: tc,
		observedValidator:     vss[0],
		observedState:         cs,
		otherValidators:       vss[1:],
		validatorClock:        clock,
		currentHeight:         1,
		chainID:               cs.state.ChainID,
		roundCh:               subscribe(cs.eventBus, types.EventQueryNewRound),
		ensureProposalCh:      subscribe(cs.eventBus, types.EventQueryCompleteProposal),
		blockCh:               subscribe(cs.eventBus, types.EventQueryNewBlock),
		ensureVoteCh:          subscribeToVoterBuffered(cs, pubKey.Address()),
		eventCh:               eventCh,
	}
}

func (p *pbtsTestHarness) observedValidatorProposerHeight(t *testing.T, previousBlockTime time.Time) (heightResult, time.Time) {
	t.Helper()
	p.validatorClock.On("Now").Return(p.genesisTime.Add(p.height2ProposedBlockOffset)).Times(2 * len(p.otherValidators))

	ensureNewRound(p.roundCh, p.currentHeight, p.currentRound)

	timeout := cmttime.Until(previousBlockTime.Add(ensureTimeout))
	if timeout < ensureTimeout {
		timeout = ensureTimeout
	}
	ensureProposalWithTimeout(p.ensureProposalCh, p.currentHeight, p.currentRound, nil, timeout)

	rs := p.observedState.GetRoundState()
	bid := types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}
	ensurePrevote(p.ensureVoteCh, p.currentHeight, p.currentRound)
	signAddVotes(p.observedState, types.PrevoteType, p.chainID, bid, false, p.otherValidators...)

	ensurePrecommit(p.ensureVoteCh, p.currentHeight, p.currentRound)
	signAddVotes(p.observedState, types.PrecommitType, p.chainID, bid, false, p.otherValidators...)

	ensureNewBlock(p.blockCh, p.currentHeight)

	vk, err := p.observedValidator.GetPubKey()
	require.NoError(t, err)
	res := collectHeightResults(t, p.eventCh, p.currentHeight, vk.Address())

	p.currentHeight++
	incrementHeight(p.otherValidators...)
	return res, rs.ProposalBlock.Time
}

func (p *pbtsTestHarness) height2(ctx context.Context, t *testing.T) heightResult {
	t.Helper()
	signer := p.otherValidators[0].PrivValidator
	return p.nextHeight(ctx, t, signer,
		p.firstBlockTime.Add(p.height2ProposalTimeDeliveryOffset),
		p.firstBlockTime.Add(p.height2ProposedBlockOffset),
		p.firstBlockTime.Add(p.height2ProposedBlockOffset+10*blockTimeIota))
}

func (p *pbtsTestHarness) intermediateHeights(ctx context.Context, t *testing.T) {
	t.Helper()
	signer := p.otherValidators[1].PrivValidator
	p.nextHeight(ctx, t, signer,
		p.firstBlockTime.Add(p.height2ProposedBlockOffset+10*blockTimeIota),
		p.firstBlockTime.Add(p.height2ProposedBlockOffset+10*blockTimeIota),
		p.firstBlockTime.Add(p.height4ProposedBlockOffset))

	signer = p.otherValidators[2].PrivValidator
	p.nextHeight(ctx, t, signer,
		p.firstBlockTime.Add(p.height4ProposedBlockOffset),
		p.firstBlockTime.Add(p.height4ProposedBlockOffset),
		cmttime.Now())
}

func (p *pbtsTestHarness) height5(t *testing.T) (heightResult, time.Time) {
	t.Helper()
	return p.observedValidatorProposerHeight(t, p.firstBlockTime.Add(p.height4ProposedBlockOffset))
}

func (p *pbtsTestHarness) nextHeight(
	ctx context.Context,
	t *testing.T,
	proposer types.PrivValidator,
	deliverTime, proposedTime, nextProposedTime time.Time,
) heightResult {
	t.Helper()

	p.validatorClock.On("Now").Return(nextProposedTime).Times(2 * len(p.otherValidators))

	ensureNewRound(p.roundCh, p.currentHeight, p.currentRound)

	b, err := p.observedState.createProposalBlock(ctx)
	require.NoError(t, err)
	require.Equal(t, b.Height, p.currentHeight)
	b.Time = proposedTime

	k, err := proposer.GetPubKey()
	require.NoError(t, err)
	b.Header.ProposerAddress = k.Address()
	ps, err := b.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	bid := types.BlockID{Hash: b.Hash(), PartSetHeader: ps.Header()}
	prop := types.NewProposal(p.currentHeight, 0, -1, bid, proposedTime)
	tp := prop.ToProto()

	err = proposer.SignProposal(p.chainID, tp)
	require.NoError(t, err)

	time.Sleep(cmttime.Until(deliverTime))
	prop.Signature = tp.Signature
	err = p.observedState.SetProposalAndBlock(prop, ps, "peerID")
	require.NoError(t, err)
	ensureProposal(p.ensureProposalCh, p.currentHeight, 0, bid)

	ensurePrevote(p.ensureVoteCh, p.currentHeight, p.currentRound)
	signAddVotes(p.observedState, types.PrevoteType, p.chainID, bid, false, p.otherValidators...)

	ensurePrecommit(p.ensureVoteCh, p.currentHeight, p.currentRound)
	signAddVotes(p.observedState, types.PrecommitType, p.chainID, bid, false, p.otherValidators...)

	vk, err := p.observedValidator.GetPubKey()
	require.NoError(t, err)
	res := collectHeightResults(t, p.eventCh, p.currentHeight, vk.Address())
	ensureNewBlock(p.blockCh, p.currentHeight)

	p.currentHeight++
	incrementHeight(p.otherValidators...)
	return res
}

func timestampedCollector(ctx context.Context, t *testing.T, eb *types.EventBus) <-chan timestampedEvent {
	t.Helper()

	// Since eventCh is not read until the end of each height, it must be large
	// enough to hold all of the events produced during a single height.
	eventCh := make(chan timestampedEvent, 100)

	const tsCollectorClient = "timestampedCollector"
	proposalSub, err := eb.Subscribe(ctx, tsCollectorClient, types.EventQueryCompleteProposal)
	require.NoError(t, err)
	// We set a capacity of since there are several votes produced.
	// With capacity 1 (default) the tests deadlock sometimes.
	voteSub, err := eb.Subscribe(ctx, tsCollectorClient, types.EventQueryVote, 10)
	require.NoError(t, err)

	go func(ctx context.Context, t *testing.T) {
		t.Helper()
		for {
			var msg cmtpubsub.Message
			select {
			case <-ctx.Done():
				return
			case msg = <-proposalSub.Out():
			case msg = <-voteSub.Out():
			}
			eventCh <- timestampedEvent{
				ts: cmttime.Now(),
				m:  msg,
			}
		}
	}(ctx, t)

	return eventCh
}

func collectHeightResults(t *testing.T, eventCh <-chan timestampedEvent, height int64, address []byte) heightResult {
	t.Helper()
	var res heightResult
	for event := range eventCh {
		require.False(t, event.ts.IsZero())
		switch v := event.m.Data().(type) {
		case types.EventDataVote:
			if v.Vote.Height > height {
				t.Fatalf("received prevote from unexpected height, expected: %d, saw: %d", height, v.Vote.Height)
			}
			if !bytes.Equal(address, v.Vote.ValidatorAddress) {
				continue
			}
			if v.Vote.Type != types.PrevoteType {
				continue
			}
			if res.prevote != nil {
				t.Fatalf("received duplicate prevote, previous %v, current %v", res.prevote, v.Vote)
			}
			res.prevote = v.Vote
			res.prevoteIssuedAt = event.ts

		case types.EventDataCompleteProposal:
			if v.Height > height {
				t.Fatalf("received proposal from unexpected height, expected: %d, saw: %d", height, v.Height)
			}
			res.proposalIssuedAt = event.ts
		}
		if res.isComplete() {
			return res
		}
	}
	t.Fatalf("complete height result never seen for height %d", height) //nolint:revive // this is part of an unreachable code test

	panic("unreachable")
}

type timestampedEvent struct {
	ts time.Time
	m  cmtpubsub.Message
}

func (p *pbtsTestHarness) run(ctx context.Context, t *testing.T) resultSet {
	t.Helper()
	startTestRound(p.observedState, p.currentHeight, p.currentRound)

	r1, proposalBlockTime := p.observedValidatorProposerHeight(t, p.genesisTime)
	p.firstBlockTime = proposalBlockTime
	r2 := p.height2(ctx, t)
	p.intermediateHeights(ctx, t)
	r5, _ := p.height5(t)
	return resultSet{
		genesisHeight: r1,
		height2:       r2,
		height5:       r5,
	}
}

type resultSet struct {
	genesisHeight heightResult
	height2       heightResult
	height5       heightResult
}

type heightResult struct {
	proposalIssuedAt time.Time
	prevote          *types.Vote
	prevoteIssuedAt  time.Time
}

func (hr heightResult) isComplete() bool {
	return !hr.proposalIssuedAt.IsZero() && !hr.prevoteIssuedAt.IsZero() && hr.prevote != nil
}

// TestProposerWaitsForGenesisTime tests that a proposer will not propose a block
// until after the genesis time has passed. The test sets the genesis time in the
// future and then ensures that the observed validator waits to propose a block.
func TestPBTSProposerWaitsForGenesisTime(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// create a genesis time far (enough) in the future.
	initialTime := cmttime.Now().Add(800 * time.Millisecond)
	cfg := pbtsTestConfiguration{
		synchronyParams: types.SynchronyParams{
			Precision:    10 * time.Millisecond,
			MessageDelay: 10 * time.Millisecond,
		},
		timeoutPropose:                    10 * time.Millisecond,
		genesisTime:                       initialTime,
		height2ProposalTimeDeliveryOffset: 10 * time.Millisecond,
		height2ProposedBlockOffset:        10 * time.Millisecond,
		height4ProposedBlockOffset:        30 * time.Millisecond,
	}

	pbtsTest := newPBTSTestHarness(ctx, t, cfg)
	results := pbtsTest.run(ctx, t)

	// ensure that the proposal was issued after the genesis time.
	assert.True(t, results.genesisHeight.proposalIssuedAt.After(cfg.genesisTime))
}

// TestProposerWaitsForPreviousBlock tests that the proposer of a block waits until
// the block time of the previous height has passed to propose the next block.
// The test harness ensures that the observed validator will be the proposer at
// height 1 and height 5. The test sets the block time of height 4 in the future
// and then verifies that the observed validator waits until after the block time
// of height 4 to propose a block at height 5.
func TestPBTSProposerWaitsForPreviousBlock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	initialTime := cmttime.Now().Add(time.Millisecond * 50)
	cfg := pbtsTestConfiguration{
		synchronyParams: types.SynchronyParams{
			Precision:    100 * time.Millisecond,
			MessageDelay: 500 * time.Millisecond,
		},
		timeoutPropose:                    50 * time.Millisecond,
		genesisTime:                       initialTime,
		height2ProposalTimeDeliveryOffset: 150 * time.Millisecond,
		height2ProposedBlockOffset:        100 * time.Millisecond,
		height4ProposedBlockOffset:        800 * time.Millisecond,
	}

	pbtsTest := newPBTSTestHarness(ctx, t, cfg)
	results := pbtsTest.run(ctx, t)

	// the observed validator is the proposer at height 5.
	// ensure that the observed validator did not propose a block until after
	// the time configured for height 4.
	assert.True(t, results.height5.proposalIssuedAt.After(pbtsTest.firstBlockTime.Add(cfg.height4ProposedBlockOffset)))

	// Ensure that the validator issued a prevote for a non-nil block.
	assert.NotNil(t, results.height5.prevote.BlockID.Hash)
}

func TestPBTSProposerWaitTime(t *testing.T) {
	genesisTime, err := time.Parse(time.RFC3339, "2019-03-13T23:00:00Z")
	require.NoError(t, err)
	testCases := []struct {
		name              string
		previousBlockTime time.Time
		localTime         time.Time
		expectedWait      time.Duration
	}{
		{
			name:              "block time greater than local time",
			previousBlockTime: genesisTime.Add(5 * time.Nanosecond),
			localTime:         genesisTime.Add(1 * time.Nanosecond),
			expectedWait:      4 * time.Nanosecond,
		},
		{
			name:              "local time greater than block time",
			previousBlockTime: genesisTime.Add(1 * time.Nanosecond),
			localTime:         genesisTime.Add(5 * time.Nanosecond),
			expectedWait:      0,
		},
		{
			name:              "both times equal",
			previousBlockTime: genesisTime.Add(5 * time.Nanosecond),
			localTime:         genesisTime.Add(5 * time.Nanosecond),
			expectedWait:      0,
		},
	}
	for _, testCase := range testCases {
		t.Run(testCase.name, func(t *testing.T) {
			mockSource := new(cmttimemocks.Source)
			mockSource.On("Now").Return(testCase.localTime)

			ti := proposerWaitTime(mockSource, testCase.previousBlockTime)
			assert.Equal(t, testCase.expectedWait, ti)
		})
	}
}

func TestPBTSTimelyProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	initialTime := cmttime.Now()

	cfg := pbtsTestConfiguration{
		synchronyParams: types.SynchronyParams{
			Precision:    10 * time.Millisecond,
			MessageDelay: 140 * time.Millisecond,
		},
		timeoutPropose:                    40 * time.Millisecond,
		genesisTime:                       initialTime,
		height2ProposedBlockOffset:        15 * time.Millisecond,
		height2ProposalTimeDeliveryOffset: 30 * time.Millisecond,
	}

	pbtsTest := newPBTSTestHarness(ctx, t, cfg)
	results := pbtsTest.run(ctx, t)
	require.NotNil(t, results.height2.prevote.BlockID.Hash)
}

func TestPBTSTooFarInThePastProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// localtime > proposedBlockTime + MsgDelay + Precision
	cfg := pbtsTestConfiguration{
		synchronyParams: types.SynchronyParams{
			Precision:    1 * time.Millisecond,
			MessageDelay: 10 * time.Millisecond,
		},
		timeoutPropose:                    50 * time.Millisecond,
		height2ProposedBlockOffset:        15 * time.Millisecond,
		height2ProposalTimeDeliveryOffset: 27 * time.Millisecond,
	}

	pbtsTest := newPBTSTestHarness(ctx, t, cfg)
	results := pbtsTest.run(ctx, t)

	require.Nil(t, results.height2.prevote.BlockID.Hash)
}

func TestPBTSTooFarInTheFutureProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	// localtime < proposedBlockTime - Precision
	cfg := pbtsTestConfiguration{
		synchronyParams: types.SynchronyParams{
			Precision:    1 * time.Millisecond,
			MessageDelay: 10 * time.Millisecond,
		},
		timeoutPropose:                    50 * time.Millisecond,
		height2ProposedBlockOffset:        100 * time.Millisecond,
		height2ProposalTimeDeliveryOffset: 10 * time.Millisecond,
		height4ProposedBlockOffset:        150 * time.Millisecond,
	}

	pbtsTest := newPBTSTestHarness(ctx, t, cfg)
	results := pbtsTest.run(ctx, t)

	require.Nil(t, results.height2.prevote.BlockID.Hash)
}

// TestPBTSEnableHeight tests the transition between BFT Time and PBTS.
// The test runs multiple heights. BFT Time is used until the configured
// PbtsEnableHeight. During some of these heights, the timestamp of votes
// is shifted to the future to increase block timestamps. PBTS is enabled
// at pbtsSetHeight, via FinalizeBlock. From this point, some nodes select
// timestamps using PBTS, which is not yet enabled. When PbtsEnableHeight
// is reached, some nodes propose bad timestamps. At the end, only blocks
// proposed by the tested node are accepted, as they are not tweaked.
func TestPBTSEnableHeight(t *testing.T) {
	numValidators := 4
	election := func(h int64, r int32) int {
		return (int(h-1) + int(r)) % numValidators
	}

	c := test.ConsensusParams()
	c.Feature.PbtsEnableHeight = 0 // Start with PBTS disabled

	app := abcimocks.NewApplication(t)
	app.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{
		Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT,
	}, nil)
	app.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil)
	app.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{}, nil)
	app.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{
		Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
	}, nil)
	app.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()

	cs, vss := randStateWithAppImpl(numValidators, app, c)
	height, round, chainID := cs.Height, cs.Round, cs.state.ChainID

	proposalCh := subscribe(cs.eventBus, types.EventQueryCompleteProposal)
	newRoundCh := subscribe(cs.eventBus, types.EventQueryNewRound)
	voteCh := subscribe(cs.eventBus, types.EventQueryVote)

	lastHeight := height + 4
	pbtsSetHeight := height + 2
	pbtsEnableHeight := height + 3

	startTestRound(cs, height, round)
	for height <= lastHeight {
		var block *types.Block
		var blockID types.BlockID

		ensureNewRound(newRoundCh, height, round)
		proposer := election(height, round)
		pbtsEnabled := (height >= pbtsEnableHeight)
		rejectProposal := false

		// Propose step
		if proposer == 0 {
			// Wait until we receive our own proposal
			// This may take longer when switching to PBTS since
			// BFT Time timestamps are shifted to the future.
			ensureProposalWithTimeout(proposalCh, height, round, nil, 2*time.Second)
			rs := cs.GetRoundState()
			block, _ = rs.ProposalBlock, rs.ProposalBlockParts
			blockID = rs.Proposal.BlockID
		} else {
			var ts time.Time
			var blockParts *types.PartSet

			if height >= pbtsSetHeight && height < pbtsEnableHeight {
				// Use PBTS logic while PBTS is not yet activated
				ts = cmttime.Now()
				rejectProposal = true
			} else if height >= pbtsEnableHeight {
				// Shift timestamp to the future 2*PRECISION => not timely
				ts = cmttime.Now().Add(2 * c.Synchrony.Precision)
				rejectProposal = true
			}
			block, blockParts, blockID = createProposalBlockWithTime(t, cs, ts)
			proposal := types.NewProposal(height, round, -1, blockID, block.Header.Time)
			// BFT Time should not care about Proposal's timestamps
			if height < pbtsSetHeight {
				proposal.Timestamp = cmttime.Now()
			}
			signProposal(t, proposal, chainID, vss[proposer])
			err := cs.SetProposalAndBlock(proposal, blockParts, "p")
			require.NoError(t, err)
			ensureProposal(proposalCh, height, round, blockID)
		}

		delta := cmttime.Since(block.Time)
		t.Log("BLOCK", height, round, "PROPOSER", proposer, "PBTS", pbtsEnabled,
			"TIMESTAMP", block.Time, delta, "ACCEPTED", !rejectProposal)

		// Accept proposal and decide, or reject proposal and move to next round
		myVote := blockID.Hash
		lockedRound := round
		if rejectProposal {
			myVote = nil
			lockedRound = int32(-1)
		} else { // We are deciding, enable FinalizeBlock mock
			res := &abci.FinalizeBlockResponse{}
			// Enable PBTS from pbtsEnableHeight via consensus params
			if height == pbtsSetHeight {
				params := types.DefaultConsensusParams()
				params.Feature.VoteExtensionsEnableHeight = 1
				params.Feature.PbtsEnableHeight = pbtsEnableHeight
				paramsProto := params.ToProto()
				res.ConsensusParamUpdates = &paramsProto
			}
			app.On("FinalizeBlock", mock.Anything, mock.Anything).
				Return(res, nil).Once()
		}

		// Prevote step
		ensurePrevote(voteCh, height, round)
		validatePrevote(t, cs, round, vss[0], myVote)
		for _, vs := range vss[2:] {
			signAddVotes(cs, types.PrevoteType, chainID, blockID, false, vs)
			ensurePrevote(voteCh, height, round)
		}

		// Precommit step
		ensurePrecommit(voteCh, height, round)
		validatePrecommit(t, cs, round, lockedRound, vss[0], myVote, myVote)
		for _, vs := range vss[2:] {
			ts := cmttime.Now()
			// Shift the next block timestamp while running BFT Time
			if height >= pbtsSetHeight-1 && height < pbtsEnableHeight {
				ts = ts.Add(time.Second)
			}
			vote := signVoteWithTimestamp(vs, types.PrecommitType, chainID, blockID, true, ts)
			cs.peerMsgQueue <- msgInfo{Msg: &VoteMessage{vote}}
			ensurePrecommit(voteCh, height, round)
		}

		if myVote != nil {
			height, round = height+1, 0
			incrementHeight(vss[1:]...)
		} else {
			round = round + 1
			incrementRound(vss[1:]...)
		}
	}
	// Last call to FinalizeBlock
	ensureNewRound(newRoundCh, height, round)
}

// TestPbtsAdaptiveMessageDelay tests whether proposals with timestamps in the
// past are eventually accepted by validators. The test runs multiple rounds.
// Rounds where the tested node is the proposer are skipped. Rounds with other
// proposers uses a Proposal with a tweaked timestamp, which is too far in the
// past. After a maximum number of rounds, if PBTS validation is adaptive, the
// synchronous parameters will be large enough to accept the proposal.
func TestPbtsAdaptiveMessageDelay(t *testing.T) {
	numValidators := 4
	election := func(h int64, r int32) int {
		return (int(h-1) + int(r)) % numValidators
	}

	c := test.ConsensusParams()
	app := kvstore.NewInMemoryApplication()
	genesisTime := cmttime.Now().Add(-10 * time.Second)
	cs, vss := randStateWithAppImplGenesisTime(numValidators, app, c, genesisTime)

	myPubKey, err := vss[0].GetPubKey()
	require.NoError(t, err)
	myAddress := myPubKey.Address()

	proposalCh := subscribe(cs.eventBus, types.EventQueryCompleteProposal)
	newRoundCh := subscribe(cs.eventBus, types.EventQueryNewRound)
	voteCh := subscribe(cs.eventBus, types.EventQueryVote)

	height, round := cs.Height, cs.Round
	chainID := cs.state.ChainID

	originMaxDelta := c.Synchrony.Precision + c.Synchrony.MessageDelay

	maximumRound := round + 10
	startTestRound(cs, height, round)

	for ; round < maximumRound; round++ {
		var vote types.BlockID
		assert.True(t, vote.IsNil()) // default is vote nil

		t.Log("Starting round", round)
		ensureNewRound(newRoundCh, height, round)
		proposer := election(height, round)
		ac := c.Synchrony.InRound(round)
		maxDelta := ac.Precision + ac.MessageDelay

		if proposer != 0 {
			shift := originMaxDelta + c.Synchrony.MessageDelay/2
			ts := cmttime.Now().Add(-shift)
			if ts.Before(genesisTime) {
				ts = genesisTime
			}

			// Create block and proposal with the tweaked timestamp
			block, blockParts, blockID := createProposalBlockWithTime(t, cs, ts)
			proposal := types.NewProposal(height, round, -1, blockID, block.Header.Time)
			signProposal(t, proposal, chainID, vss[proposer])

			require.NoError(t, cs.SetProposalAndBlock(proposal, blockParts, "p"))
			maxReceiveTime := cmttime.Now()

			ensureProposal(proposalCh, height, round, blockID)
			ensurePrevote(voteCh, height, round)
			vote = cs.Votes.Prevotes(round).GetByAddress(myAddress).BlockID

			delta := maxReceiveTime.Sub(ts)
			t.Log("Proposal timestamp", ts.Format(time.StampMicro),
				"maximum receive time", maxReceiveTime.Format(time.StampMicro),
				"delta", delta,
				"maximum allowed delta", maxDelta)
			t.Logf("Round %d, expected timely=%v, got timely=%v\n",
				round, (delta < maxDelta), !vote.IsNil())
		} else {
			// The node will accept its own proposal.
			// Just make everyone to vote nil and skip the round.
			ensureNewProposal(proposalCh, height, round)
			ensurePrevote(voteCh, height, round)
		}

		for _, vs := range vss[2:] {
			signAddVotes(cs, types.PrevoteType, chainID, vote, false, vs)
			ensurePrevote(voteCh, height, round)
		}
		ensurePrecommit(voteCh, height, round)

		for _, vs := range vss[2:] {
			signAddVotes(cs, types.PrecommitType, chainID, vote, true, vs)
			ensurePrecommit(voteCh, height, round)
		}

		if !vote.IsNil() {
			// Decide, so we are good!
			ensureNewRound(newRoundCh, height+1, 0)
			t.Log("Decided at round", round)
			return
		}
		// No decision, new round required
		incrementRound(vss[1:]...)
	}
	t.Error("Did not decide after round", round)
}
