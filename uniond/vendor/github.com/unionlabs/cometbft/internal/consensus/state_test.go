package consensus

import (
	"bytes"
	"context"
	"fmt"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/mock"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	abcimocks "github.com/cometbft/cometbft/abci/types/mocks"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cometbft/cometbft/crypto/tmhash"
	cstypes "github.com/cometbft/cometbft/internal/consensus/types"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	"github.com/cometbft/cometbft/internal/test"
	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/protoio"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	p2pmock "github.com/cometbft/cometbft/p2p/mock"
	"github.com/cometbft/cometbft/types"
)

/*

ProposeSuite
x * TestProposerSelection0 - round robin ordering, round 0
x * TestProposerSelection2 - round robin ordering, round 2++
x * TestEnterProposeNoValidator - timeout into prevote round
x * TestEnterPropose - finish propose without timing out (we have the proposal)
x * TestBadProposal - 2 vals, bad proposal (bad block state hash), should prevote and precommit nil
x * TestOversizedBlock - block with too many txs should be rejected
FullRoundSuite
x * TestFullRound1 - 1 val, full successful round
x * TestFullRoundNil - 1 val, full round of nil
x * TestFullRound2 - 2 vals, both required for full round
LockSuite
x * TestStateLock_NoPOL - 2 vals, 4 rounds. one val locked, precommits nil every round except first.
x * TestStateLock_POLUpdateLock - 4 vals, one precommits,
other 3 polka at next round, so we unlock and precomit the polka
x * TestStateLock_POLRelock - 4 vals, polka in round 1 and polka in round 2.
Ensure validator updates locked round.
x * TestStateLock_POLDoesNotUnlock 4 vals, one precommits, other 3 polka nil at
next round, so we precommit nil but maintain lock
x * TestStateLock_MissingProposalWhenPOLSeenDoesNotUpdateLock - 4 vals, 1 misses proposal but sees POL.
x * TestStateLock_MissingProposalWhenPOLSeenDoesNotUnlock - 4 vals, 1 misses proposal but sees POL.
  * TestStateLock_MissingProposalWhenPOLForLockedBlock - 4 vals, 1 misses proposal but sees POL for locked block.
x * TestState_MissingProposalValidBlockReceivedTimeout - 4 vals, 1 misses proposal but receives full block.
x * TestState_MissingProposalValidBlockReceivedPrecommit - 4 vals, 1 misses proposal but receives full block.
x * TestStateLock_POLSafety1 - 4 vals. We shouldn't change lock based on polka at earlier round
x * TestStateLock_POLSafety2 - 4 vals. We shouldn't accept a proposal with POLRound smaller than our locked round.
x * TestState_PrevotePOLFromPreviousRound 4 vals, prevote a proposal if a POL was seen for it in a previous round.
  * TestNetworkLock - once +1/3 precommits, network should be locked
  * TestNetworkLockPOL - once +1/3 precommits, the block with more recent polka is committed
SlashingSuite
x * TestStateSlashing_Prevotes - a validator prevoting twice in a round gets slashed
x * TestStateSlashing_Precommits - a validator precomitting twice in a round gets slashed
CatchupSuite
  * TestCatchup - if we might be behind and we've seen any 2/3 prevotes, round skip to new round, precommit, or prevote
HaltSuite
x * TestHalt1 - if we see +2/3 precommits after timing out into new round, we should still commit

*/

// ----------------------------------------------------------------------------------------------------
// ProposeSuite

func TestStateProposerSelection0(t *testing.T) {
	cs1, vss := randState(4)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	pv, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)

	startTestRound(cs1, height, round)

	// Wait for new round so proposer is set.
	ensureNewRound(newRoundCh, height, round)

	// Commit a block and ensure proposer for the next height is correct.
	prop := cs1.GetRoundState().Validators.GetProposer()
	address := pv.Address()
	require.Truef(t, bytes.Equal(prop.Address, address), "expected proposer to be validator 0 (%X). Got %X", address, prop.Address)

	// Wait for complete proposal.
	ensureNewProposal(proposalCh, height, round)

	rs := cs1.GetRoundState()
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}, true, vss[1:]...)

	// Wait for new round so next validator is set.
	ensureNewRound(newRoundCh, height+1, 0)

	prop = cs1.GetRoundState().Validators.GetProposer()
	pv1, err := vss[1].GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	require.Truef(t, bytes.Equal(prop.Address, addr), "expected proposer to be validator 1 (%X). Got %X", addr, prop.Address)
}

// Now let's do it all again, but starting from round 2 instead of 0.
func TestStateProposerSelection2(t *testing.T) {
	cs1, vss := randState(4) // test needs more work for more than 3 validators
	height, chainID := cs1.Height, cs1.state.ChainID
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	// this time we jump in at round 2
	incrementRound(vss[1:]...)
	incrementRound(vss[1:]...)

	var round int32 = 2
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round) // wait for the new round

	// everyone just votes nil. we get a new proposer each round
	for i := int32(0); int(i) < len(vss); i++ {
		prop := cs1.GetRoundState().Validators.GetProposer()
		pvk, err := vss[int(i+round)%len(vss)].GetPubKey()
		require.NoError(t, err)
		addr := pvk.Address()
		correctProposer := addr
		require.Truef(t, bytes.Equal(prop.Address, correctProposer),
			"expected RoundState.Validators.GetProposer() to be validator %d (%X). Got %X",
			int(i+2)%len(vss), correctProposer, prop.Address,
		)
		signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vss[1:]...)
		ensureNewRound(newRoundCh, height, i+round+1) // wait for the new round event each round
		incrementRound(vss[1:]...)
	}
}

// a non-validator should timeout into the prevote round.
func TestStateEnterProposeNoPrivValidator(t *testing.T) {
	cs, _ := randState(1)
	cs.SetPrivValidator(nil)
	height, round := cs.Height, cs.Round

	// Listen for propose timeout event
	timeoutCh := subscribe(cs.eventBus, types.EventQueryTimeoutPropose)

	startTestRound(cs, height, round)

	// if we're not a validator, EnterPropose should timeout
	ensureNewTimeout(timeoutCh, height, round, cs.config.TimeoutPropose.Nanoseconds())

	if cs.GetRoundState().Proposal != nil {
		t.Error("Expected to make no proposal, since no privValidator")
	}
}

// a validator should not timeout of the prevote round (TODO: unless the block is really big!)
func TestStateEnterProposeYesPrivValidator(t *testing.T) {
	cs, _ := randState(1)
	height, round := cs.Height, cs.Round

	// Listen for propose timeout event

	timeoutCh := subscribe(cs.eventBus, types.EventQueryTimeoutPropose)
	proposalCh := subscribe(cs.eventBus, types.EventQueryCompleteProposal)

	cs.enterNewRound(height, round)
	cs.startRoutines(3)

	ensureNewProposal(proposalCh, height, round)

	// Check that Proposal, ProposalBlock, ProposalBlockParts are set.
	rs := cs.GetRoundState()
	if rs.Proposal == nil {
		t.Error("rs.Proposal should be set")
	}
	if rs.ProposalBlock == nil {
		t.Error("rs.ProposalBlock should be set")
	}
	if rs.ProposalBlockParts.Total() == 0 {
		t.Error("rs.ProposalBlockParts should be set")
	}

	// if we're a validator, enterPropose should not timeout
	ensureNoNewTimeout(timeoutCh, cs.config.TimeoutPropose.Nanoseconds())
}

func TestStateBadProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(2)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	vs2 := vss[1]

	partSize := types.BlockPartSizeBytes

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	voteCh := subscribe(cs1.eventBus, types.EventQueryVote)

	propBlock, err := cs1.createProposalBlock(ctx) // changeProposer(t, cs1, vs2)
	require.NoError(t, err)

	// make the second validator the proposer by incrementing round
	round++
	incrementRound(vss[1:]...)

	// make the block bad by tampering with statehash
	stateHash := propBlock.AppHash
	if len(stateHash) == 0 {
		stateHash = make([]byte, 32)
	}
	stateHash[0] = (stateHash[0] + 1) % 255
	propBlock.AppHash = stateHash
	propBlockParts, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{Hash: propBlock.Hash(), PartSetHeader: propBlockParts.Header()}
	proposal := types.NewProposal(vs2.Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vs2)

	// set the proposal block
	err = cs1.SetProposalAndBlock(proposal, propBlockParts, "some peer")
	require.NoError(t, err)

	// start the machine
	startTestRound(cs1, height, round)

	// wait for proposal
	ensureProposal(proposalCh, height, round, blockID)

	// wait for prevote
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// add bad prevote from vs2 and wait for it
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2)
	ensurePrevote(voteCh, height, round)

	// wait for precommit
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs2)
}

func TestStateOversizedBlock(t *testing.T) {
	const maxBytes = int64(types.BlockPartSizeBytes)

	for _, testCase := range []struct {
		name      string
		oversized bool
	}{
		{
			name:      "max size, correct block",
			oversized: false,
		},
		{
			name:      "off-by-1 max size, incorrect block",
			oversized: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			cs1, vss := randState(2)
			cs1.state.ConsensusParams.Block.MaxBytes = maxBytes
			height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
			vs2 := vss[1]

			partSize := types.BlockPartSizeBytes

			propBlock, propBlockParts := findBlockSizeLimit(t, height, maxBytes, cs1, partSize, testCase.oversized)

			timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
			voteCh := subscribe(cs1.eventBus, types.EventQueryVote)

			// make the second validator the proposer by incrementing round
			round++
			incrementRound(vss[1:]...)

			blockID := types.BlockID{Hash: propBlock.Hash(), PartSetHeader: propBlockParts.Header()}
			proposal := types.NewProposal(height, round, -1, blockID, propBlock.Header.Time)
			signProposal(t, proposal, chainID, vs2)

			totalBytes := 0
			for i := 0; i < int(propBlockParts.Total()); i++ {
				part := propBlockParts.GetPart(i)
				totalBytes += len(part.Bytes)
			}

			maxBlockParts := maxBytes / int64(types.BlockPartSizeBytes)
			if maxBytes > maxBlockParts*int64(types.BlockPartSizeBytes) {
				maxBlockParts++
			}
			numBlockParts := int64(propBlockParts.Total())

			err := cs1.SetProposalAndBlock(proposal, propBlockParts, "some peer")
			require.NoError(t, err)

			// start the machine
			startTestRound(cs1, height, round)

			t.Log("Block Sizes;", "Limit", maxBytes, "Current", totalBytes)
			t.Log("Proposal Parts;", "Maximum", maxBlockParts, "Current", numBlockParts)

			validateHash := propBlock.Hash()
			lockedRound := int32(1)
			if testCase.oversized {
				validateHash = nil
				lockedRound = -1
				// if the block is oversized cs1 should log an error with the block part message as it exceeds
				// the consensus params. The block is not added to cs.ProposalBlock so the node timeouts.
				ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())
				// and then should send nil prevote and precommit regardless of whether other validators prevote and
				// precommit on it
			}
			ensurePrevote(voteCh, height, round)
			validatePrevote(t, cs1, round, vss[0], validateHash)

			// Should not accept a Proposal with too many block parts
			if numBlockParts > maxBlockParts {
				require.Nil(t, cs1.Proposal)
			}

			signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2)
			ensurePrevote(voteCh, height, round)
			ensurePrecommit(voteCh, height, round)
			validatePrecommit(t, cs1, round, lockedRound, vss[0], validateHash, validateHash)

			signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs2)
		})
	}
}

// ----------------------------------------------------------------------------------------------------
// FullRoundSuite

// propose, prevote, and precommit a block.
func TestStateFullRound1(t *testing.T) {
	cs, vss := randState(1)
	height, round := cs.Height, cs.Round

	// NOTE: buffer capacity of 0 ensures we can validate prevote and last commit
	// before consensus can move to the next height (and cause a race condition)
	if err := cs.eventBus.Stop(); err != nil {
		t.Error(err)
	}
	eventBus := types.NewEventBusWithBufferCapacity(0)
	eventBus.SetLogger(log.TestingLogger().With("module", "events"))
	cs.SetEventBus(eventBus)
	if err := eventBus.Start(); err != nil {
		t.Error(err)
	}

	voteCh := subscribeUnBuffered(cs.eventBus, types.EventQueryVote)
	propCh := subscribe(cs.eventBus, types.EventQueryCompleteProposal)
	newRoundCh := subscribe(cs.eventBus, types.EventQueryNewRound)

	// Maybe it would be better to call explicitly startRoutines(4)
	startTestRound(cs, height, round)

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(propCh, height, round)
	propBlockHash := cs.GetRoundState().ProposalBlock.Hash()

	ensurePrevote(voteCh, height, round) // wait for prevote
	validatePrevote(t, cs, round, vss[0], propBlockHash)

	ensurePrecommit(voteCh, height, round) // wait for precommit

	// we're going to roll right into new height
	ensureNewRound(newRoundCh, height+1, 0)

	validateLastPrecommit(t, cs, vss[0], propBlockHash)
}

// nil is proposed, so prevote and precommit nil.
func TestStateFullRoundNil(t *testing.T) {
	cs, _ := randState(1)
	height, round := cs.Height, cs.Round

	voteCh := subscribeUnBuffered(cs.eventBus, types.EventQueryVote)

	cs.enterPrevote(height, round)
	cs.startRoutines(4)

	ensurePrevoteMatch(t, voteCh, height, round, nil)   // prevote
	ensurePrecommitMatch(t, voteCh, height, round, nil) // precommit
}

// run through propose, prevote, precommit commit with two validators
// where the first validator has to wait for votes from the second.
func TestStateFullRound2(t *testing.T) {
	cs1, vss := randState(2)
	vs2 := vss[1]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	voteCh := subscribeUnBuffered(cs1.eventBus, types.EventQueryVote)
	newBlockCh := subscribe(cs1.eventBus, types.EventQueryNewBlock)

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)

	ensurePrevote(voteCh, height, round) // prevote

	// we should be stuck in limbo waiting for more prevotes
	rs := cs1.GetRoundState()
	blockID := types.BlockID{Hash: rs.ProposalBlock.Hash(), PartSetHeader: rs.ProposalBlockParts.Header()}

	// prevote arrives from vs2:
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2)
	ensurePrevote(voteCh, height, round) // prevote

	ensurePrecommit(voteCh, height, round) // precommit
	// the proposed block should now be locked and our precommit added
	validatePrecommit(t, cs1, 0, 0, vss[0], blockID.Hash, blockID.Hash)

	// we should be stuck in limbo waiting for more precommits

	// precommit arrives from vs2:
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs2)
	ensurePrecommit(voteCh, height, round)

	// wait to finish commit, propose in next height
	ensureNewBlock(newBlockCh, height)
}

// ------------------------------------------------------------------------------------------
// LockSuite

// two validators, 4 rounds.
// two vals take turns proposing. val1 locks on first one, precommits nil on everything else.
func TestStateLock_NoPOL(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(2)
	vs2 := vss[1]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	voteCh := subscribeUnBuffered(cs1.eventBus, types.EventQueryVote)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round1 (cs1, B) // B B // B B2
	*/

	// start round and wait for prevote
	cs1.enterNewRound(height, round)
	cs1.startRoutines(0)

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	roundState := cs1.GetRoundState()
	initialBlockID := types.BlockID{
		Hash:          roundState.ProposalBlock.Hash(),
		PartSetHeader: roundState.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round) // prevote

	// we should now be stuck in limbo forever, waiting for more prevotes
	// prevote arrives from vs2:
	signAddVotes(cs1, types.PrevoteType, chainID, initialBlockID, false, vs2)
	ensurePrevote(voteCh, height, round) // prevote
	validatePrevote(t, cs1, round, vss[0], initialBlockID.Hash)

	// the proposed block should now be locked and our precommit added
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], initialBlockID.Hash, initialBlockID.Hash)

	// we should now be stuck in limbo forever, waiting for more precommits
	// lets add one for a different block
	hash := make([]byte, len(initialBlockID.Hash))
	copy(hash, initialBlockID.Hash)
	hash[0] = (hash[0] + 1) % 255
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{
		Hash:          hash,
		PartSetHeader: initialBlockID.PartSetHeader,
	}, true, vs2)
	ensurePrecommit(voteCh, height, round) // precommit

	// (note we're entering precommit for a second time this round)
	// but with invalid args. then we enterPrecommitWait, and the timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	//

	round++ // moving to the next round
	ensureNewRound(newRoundCh, height, round)
	t.Log("#### ONTO ROUND 1")
	/*
		Round2 (cs1, B) // B B2
	*/

	incrementRound(vs2)

	// now we're on a new round and not the proposer, so wait for timeout
	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())

	rs := cs1.GetRoundState()

	require.Nil(t, rs.ProposalBlock, "Expected proposal block to be nil")

	// we should have prevoted nil since we did not see a proposal in the round.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// add a conflicting prevote from the other validator
	partSet, err := rs.LockedBlock.MakePartSet(partSize)
	require.NoError(t, err)
	conflictingBlockID := types.BlockID{Hash: hash, PartSetHeader: partSet.Header()}
	signAddVotes(cs1, types.PrevoteType, chainID, conflictingBlockID, false, vs2)
	ensurePrevote(voteCh, height, round)

	// now we're going to enter prevote again, but with invalid args
	// and then prevote wait, which should timeout. then wait for precommit
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Prevote(round).Nanoseconds())
	// the proposed block should still be locked block.
	// we should precommit nil and be locked on the proposal.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 0, vss[0], nil, initialBlockID.Hash)

	// add conflicting precommit from vs2
	signAddVotes(cs1, types.PrecommitType, chainID, conflictingBlockID, true, vs2)
	ensurePrecommit(voteCh, height, round)

	// (note we're entering precommit for a second time this round, but with invalid args
	// then we enterPrecommitWait and timeout into NewRound
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	round++ // entering new round
	ensureNewRound(newRoundCh, height, round)
	t.Log("#### ONTO ROUND 2")
	/*
		Round3 (vs2, _) // B, B2
	*/

	incrementRound(vs2)

	ensureNewProposal(proposalCh, height, round)
	rs = cs1.GetRoundState()

	// now we're on a new round and are the proposer
	require.Truef(t, bytes.Equal(rs.ProposalBlock.Hash(), rs.LockedBlock.Hash()),
		"expected proposal block to be locked block. Got %v, Expected %v",
		rs.ProposalBlock, rs.LockedBlock,
	)

	ensurePrevote(voteCh, height, round) // prevote
	validatePrevote(t, cs1, round, vss[0], rs.LockedBlock.Hash())
	partSet, err = rs.ProposalBlock.MakePartSet(partSize)
	require.NoError(t, err)
	newBlockID := types.BlockID{Hash: hash, PartSetHeader: partSet.Header()}
	signAddVotes(cs1, types.PrevoteType, chainID, newBlockID, false, vs2)
	ensurePrevote(voteCh, height, round)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Prevote(round).Nanoseconds())
	ensurePrecommit(voteCh, height, round) // precommit

	validatePrecommit(t, cs1, round, 0, vss[0], nil, initialBlockID.Hash) // precommit nil but be locked on proposal

	signAddVotes(
		cs1,
		types.PrecommitType,
		chainID,
		newBlockID,
		true,
		vs2) // NOTE: conflicting precommits at same height
	ensurePrecommit(voteCh, height, round)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	cs2, _ := randState(2) // needed so generated block is different than locked block
	// before we time out into new round, set next proposal block
	prop, propBlock := decideProposal(ctx, t, cs2, vs2, vs2.Height, vs2.Round+1)
	require.NotNil(t, propBlock, "Failed to create proposal block with vs2")
	require.NotNil(t, prop, "Failed to create proposal block with vs2")
	propBlockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	incrementRound(vs2)

	round++ // entering new round
	ensureNewRound(newRoundCh, height, round)
	t.Log("#### ONTO ROUND 3")
	/*
		Round4 (vs2, C) // B C // B C
	*/

	// now we're on a new round and not the proposer
	// so set the proposal block
	bps3, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	err = cs1.SetProposalAndBlock(prop, bps3, "")
	require.NoError(t, err)

	ensureNewProposal(proposalCh, height, round)

	// prevote for nil since we did not see a proposal for our locked block in the round.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, 3, vss[0], nil)

	// prevote for proposed block
	signAddVotes(cs1, types.PrevoteType, chainID, propBlockID, false, vs2)
	ensurePrevote(voteCh, height, round)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Prevote(round).Nanoseconds())
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 0, vss[0], nil, initialBlockID.Hash) // precommit nil but locked on proposal

	signAddVotes(
		cs1,
		types.PrecommitType,
		chainID,
		propBlockID,
		true,
		vs2) // NOTE: conflicting precommits at same height
	ensurePrecommit(voteCh, height, round)
}

// TestStateLock_POLUpdateLock tests that a validator updates its locked
// block if the following conditions are met within a round:
// 1. The validator received a valid proposal for the block
// 2. The validator received prevotes representing greater than 2/3 of the voting
// power on the network for the block.
func TestStateLock_POLUpdateLock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	initialBlockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, initialBlockID, false, vs2, vs3, vs4)

	// check that the validator generates a Lock event.
	ensureLock(lockCh, height, round)

	// the proposed block should now be locked and our precommit added.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], initialBlockID.Hash, initialBlockID.Hash)

	// add precommits from the rest of the validators.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Create a block, D and send a proposal for it to cs1
		Send a prevote for D from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		Check that cs1 is now locked on the new block, D and no longer on the old block.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++

	// Generate a new proposal block.
	cs2 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	propR1, propBlockR1 := decideProposal(ctx, t, cs2, vs2, vs2.Height, vs2.Round)
	propBlockR1Parts, err := propBlockR1.MakePartSet(partSize)
	require.NoError(t, err)
	propBlockR1Hash := propBlockR1.Hash()
	r1BlockID := types.BlockID{
		Hash:          propBlockR1Hash,
		PartSetHeader: propBlockR1Parts.Header(),
	}
	require.NotEqual(t, propBlockR1Hash, initialBlockID.Hash)
	err = cs1.SetProposalAndBlock(propR1, propBlockR1Parts, "some peer")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)

	// ensure that the validator receives the proposal.
	ensureNewProposal(proposalCh, height, round)

	// Prevote our nil since the proposal does not match our locked block.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// Add prevotes from the remainder of the validators for the new locked block.
	signAddVotes(cs1, types.PrevoteType, chainID, r1BlockID, false, vs2, vs3, vs4)

	// Check that we lock on a new block.
	ensureLock(lockCh, height, round)

	ensurePrecommit(voteCh, height, round)

	// We should now be locked on the new block and prevote it since we saw a sufficient amount
	// prevote for the block.
	validatePrecommit(t, cs1, round, round, vss[0], propBlockR1Hash, propBlockR1Hash)
}

// TestStateLock_POLRelock tests that a validator updates its locked round if
// it receives votes representing over 2/3 of the voting power on the network
// for a block that it is already locked in.
func TestStateLock_POLRelock(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	relockCh := subscribe(cs1.eventBus, types.EventQueryRelock)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.
		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")

	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	theBlock := rs.ProposalBlock
	theBlockParts := rs.ProposalBlockParts
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// check that the validator generates a Lock event.
	ensureLock(lockCh, height, round)

	// the proposed block should now be locked and our precommit added.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits from the rest of the validators.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Create a proposal for block B, the same block from round 1.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		Check that cs1 updates its 'locked round' value to the current round.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++
	propR1 := types.NewProposal(height, round, cs1.ValidRound, blockID, theBlock.Header.Time)
	signProposal(t, propR1, chainID, vs2)
	err = cs1.SetProposalAndBlock(propR1, theBlockParts, "")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)

	// ensure that the validator receives the proposal.
	ensureNewProposal(proposalCh, height, round)

	// Prevote our locked block since it matches the propsal seen in this round.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	// Add prevotes from the remainder of the validators for the locked block.
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// Check that we relock.
	ensureRelock(relockCh, height, round)

	ensurePrecommit(voteCh, height, round)

	// We should now be locked on the same block but with an updated locked round.
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)
}

// TestStateLock_PrevoteNilWhenLockedAndMissProposal tests that a validator prevotes nil
// if it is locked on a block and misses the proposal in a round.
func TestStateLock_PrevoteNilWhenLockedAndMissProposal(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")

	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// check that the validator generates a Lock event.
	ensureLock(lockCh, height, round)

	// the proposed block should now be locked and our precommit added.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits from the rest of the validators.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Send a prevote for nil from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		Check that cs1 prevotes nil instead of its locked block, but ensure
		that it maintains its locked block.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++

	ensureNewRound(newRoundCh, height, round)

	// Prevote our nil.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// Add prevotes from the remainder of the validators nil.
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	// We should now be locked on the same block but with an updated locked round.
	validatePrecommit(t, cs1, round, 0, vss[0], nil, blockID.Hash)
}

// TestStateLock_PrevoteNilWhenLockedAndDifferentProposal tests that a validator prevotes nil
// if it is locked on a block and gets a different proposal in a round.
func TestStateLock_PrevoteNilWhenLockedAndDifferentProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	/*
		All of the assertions in this test occur on the `cs1` validator.
		The test sends signed votes from the other validators to cs1 and
		cs1's state is then examined to verify that it now matches the expected
		state.
	*/

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// check that the validator generates a Lock event.
	ensureLock(lockCh, height, round)

	// the proposed block should now be locked and our precommit added.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits from the rest of the validators.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Create a proposal for a new block.
		Send a prevote for nil from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		Check that cs1 prevotes nil instead of its locked block, but ensure
		that it maintains its locked block.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++
	cs2 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	propR1, propBlockR1 := decideProposal(ctx, t, cs2, vs2, vs2.Height, vs2.Round)
	propBlockR1Parts, err := propBlockR1.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	propBlockR1Hash := propBlockR1.Hash()
	require.NotEqual(t, propBlockR1Hash, blockID.Hash)
	err = cs1.SetProposalAndBlock(propR1, propBlockR1Parts, "some peer")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)

	// Prevote our nil.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// Add prevotes from the remainder of the validators for nil.
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	// We should now be locked on the same block but prevote nil.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 0, vss[0], nil, blockID.Hash)
}

// TestStateLock_POLDoesNotUnlock tests that a validator maintains its locked block
// despite receiving +2/3 nil prevotes and nil precommits from other validators.
// Tendermint used to 'unlock' its locked block when greater than 2/3 prevotes
// for a nil block were seen. This behavior has been removed and this test ensures
// that it has been completely removed.
func TestStateLock_POLDoesNotUnlock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	/*
		All of the assertions in this test occur on the `cs1` validator.
		The test sends signed votes from the other validators to cs1 and
		cs1's state is then examined to verify that it now matches the expected
		state.
	*/

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	/*
		Round 0:
		Create a block, B
		Send a prevote for B from each of the validators to `cs1`.
		Send a precommit for B from one of the validators to `cs1`.

		This ensures that cs1 will lock on B in this round.
	*/
	t.Log("#### ONTO ROUND 0")

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// the validator should have locked a block in this round.
	ensureLock(lockCh, height, round)

	ensurePrecommit(voteCh, height, round)
	// the proposed block should now be locked and our should be for this locked block.

	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// Add precommits from the other validators.
	// We only issue 1/2 Precommits for the block in this round.
	// This ensures that the validator being tested does not commit the block.
	// We do not want the validator to commit the block because we want the test
	// test to proceeds to the next consensus round.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs4)
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs3)

	// timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Send a prevote for nil from >2/3 of the validators to `cs1`.
		Check that cs1 maintains its lock on B but precommits nil.
		Send a precommit for nil from >2/3 of the validators to `cs1`.
	*/
	t.Log("#### ONTO ROUND 1")
	round++
	incrementRound(vs2, vs3, vs4)
	cs2 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	prop, propBlock := decideProposal(ctx, t, cs2, vs2, vs2.Height, vs2.Round)
	propBlockParts, err := propBlock.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	require.NotEqual(t, propBlock.Hash(), blockID.Hash)
	err = cs1.SetProposalAndBlock(prop, propBlockParts, "")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)

	// Prevote for nil since the proposed block does not match our locked block.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// add >2/3 prevotes for nil from all other validators
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)

	// verify that we haven't update our locked block since the first round
	validatePrecommit(t, cs1, round, 0, vss[0], nil, blockID.Hash)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 2:
		The validator cs1 saw >2/3 precommits for nil in the previous round.
		Send the validator >2/3 prevotes for nil and ensure that it did not
		unlock its block at the end of the previous round.
	*/
	t.Log("#### ONTO ROUND 2")
	round++
	incrementRound(vs2, vs3, vs4)
	cs3 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	prop, propBlock = decideProposal(ctx, t, cs3, vs3, vs3.Height, vs3.Round)
	propBlockParts, err = propBlock.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	err = cs1.SetProposalAndBlock(prop, propBlockParts, "")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)

	// Prevote for nil since the proposal does not match our locked block.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)

	// verify that we haven't update our locked block since the first round
	validatePrecommit(t, cs1, round, 0, vss[0], nil, blockID.Hash)
}

// TestStateLock_MissingProposalWhenPOLSeenDoesNotUpdateLock tests that observing
// a two thirds majority for a block does not cause a validator to update its lock on the
// new block if a proposal was not seen for that block.
func TestStateLock_MissingProposalWhenPOLSeenDoesNotUpdateLock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	firstBlockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round) // prevote

	signAddVotes(cs1, types.PrevoteType, chainID, firstBlockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round) // our precommit
	// the proposed block should now be locked and our precommit added
	validatePrecommit(t, cs1, round, round, vss[0], firstBlockID.Hash, firstBlockID.Hash)

	// add precommits from the rest
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Create a new block, D but do not send it to cs1.
		Send a prevote for D from each of the validators to cs1.

		Check that cs1 does not update its locked block to this missed block D.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++
	cs2 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	prop, propBlock := decideProposal(ctx, t, cs2, vs2, vs2.Height, vs2.Round)
	require.NotNil(t, propBlock, "Failed to create proposal block with vs2")
	require.NotNil(t, prop, "Failed to create proposal block with vs2")
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	secondBlockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}
	require.NotEqual(t, secondBlockID.Hash, firstBlockID.Hash)

	ensureNewRound(newRoundCh, height, round)

	// prevote for nil since the proposal was not seen.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// now lets add prevotes from everyone else for the new block
	signAddVotes(cs1, types.PrevoteType, chainID, secondBlockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 0, vss[0], nil, firstBlockID.Hash)
}

// TestStateLock_MissingProposalWhenPOLForLockedBlock tests that observing
// a two thirds majority for a block that matches the validator's locked block
// causes a validator to update its lock round and Precommit the locked block.
func TestStateLock_MissingProposalWhenPOLForLockedBlock(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not commit it.
	*/
	t.Log("### Starting Round 0")
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round) // prevote

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round) // our precommit
	// the proposed block should now be locked and our precommit added
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits from the rest
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		The same block B is re-proposed, but it is not sent to cs1.
		Send a prevote for B from each of the validators to cs1.

		Check that cs1 precommits B, since B matches its locked value.
		Check that cs1 maintains its lock on block B but updates its locked round.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++

	ensureNewRound(newRoundCh, height, round)

	// prevote for nil since the proposal was not seen (although it matches the locked block)
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// now lets add prevotes from everyone else for the locked block
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// the validator precommits the valid block (as it received 2/3+
	// prevotes) which matches its locked block (which also received 2/3+
	// prevotes in the previous round).
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)
}

// TestState_MissingProposalValidBlockReceivedTimeout tests if a node that
// misses the round's Proposal but receives a Polka for a block and the full
// block will not prevote for the valid block because the Proposal was missing.
func TestState_MissingProposalValidBlockReceivedTimeout(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	voteCh := subscribe(cs1.eventBus, types.EventQueryVote)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)

	// Produce a block
	block, err := cs1.createProposalBlock(ctx)
	require.NoError(t, err)
	blockParts, err := block.MakePartSet(types.BlockPartSizeBytes)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          block.Hash(),
		PartSetHeader: blockParts.Header(),
	}

	// Skip round 0 and start consensus threads
	round++
	incrementRound(vss[1:]...)
	startTestRound(cs1, height, round)

	// Receive prevotes(height, round=1, blockID) from all other validators.
	for i := 1; i < len(vss); i++ {
		signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[i])
		ensurePrevote(voteCh, height, round)
	}

	// We have polka for blockID so we can accept the associated full block.
	for i := 0; i < int(blockParts.Total()); i++ {
		err := cs1.AddProposalBlockPart(height, round, blockParts.GetPart(i), "peer")
		require.NoError(t, err)
	}
	ensureNewValidBlock(validBlockCh, height, round)

	// We don't prevote right now because we didn't receive the round's
	// Proposal. Wait for the propose timeout.
	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())

	rs := cs1.GetRoundState()
	assert.Equal(t, rs.ValidRound, round)
	assert.Equal(t, rs.ValidBlock.Hash(), blockID.Hash)

	// Since we didn't see the round's Proposal, we should prevote nil.
	// NOTE: introduced by https://github.com/cometbft/cometbft/pull/1203.
	// In branches v0.{34,37,38}.x, the node prevotes for the valid block.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)
}

// TestState_MissingProposalValidBlockReceivedPrecommit tests if a node that
// misses the round's Proposal, but receives a Polka for a block and the full
// block, precommits the valid block even though the Proposal is missing.
func TestState_MissingProposalValidBlockReceivedPrecommit(t *testing.T) {
	cs1, vss := randState(4)
	height, round := cs1.Height, cs1.Round
	chainID := cs1.state.ChainID

	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)
	voteCh := subscribe(cs1.eventBus, types.EventQueryVote)

	// Produce a block
	_, blockParts, blockID := createProposalBlock(t, cs1)

	// Skip round 0 and start consensus
	round++
	incrementRound(vss[1:]...)
	startTestRound(cs1, height, round)

	// We are late, so we already receive prevotes for the block
	for i := 1; i < len(vss); i++ {
		signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[i])
		ensurePrevote(voteCh, height, round)
	}
	// We received a Polka for blockID, which is now valid
	ensureNewValidBlock(validBlockCh, height, round)

	// We don't have the Proposal, so we wait for timeout propose
	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// We accept the full block associated with the valid blockID
	for i := 0; i < int(blockParts.Total()); i++ {
		err := cs1.AddProposalBlockPart(height, round, blockParts.GetPart(i), "peer")
		require.NoError(t, err)
	}

	// we don't have the Proposal for blockID, but we have the full block
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)
}

// TestStateLock_DoesNotLockOnOldProposal tests that observing
// a two thirds majority for a block does not cause a validator to lock on the
// block if a proposal was not seen for that block in the current round, but
// was seen in a previous round.
func TestStateLock_DoesNotLockOnOldProposal(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for nil from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will not lock on B.
	*/
	t.Log("### Starting Round 0")
	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	firstBlockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	// The proposed block should not have been locked.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	incrementRound(vs2, vs3, vs4)

	// timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		No proposal new proposal is created.
		Send a prevote for B, the block from round 0, from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		cs1 saw a POL for the block it saw in round 0. We ensure that it does not
		lock on this block, since it did not see a proposal for it in this round.
	*/
	t.Log("### Starting Round 1")
	round++
	ensureNewRound(newRoundCh, height, round)

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// All validators prevote for the old block.
	signAddVotes(cs1, types.PrevoteType, chainID, firstBlockID, false, vs2, vs3, vs4)

	// Make sure that cs1 did not lock on the block since it did not receive a proposal for it.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)
}

// TestStateLock_POLSafety1 tests that a node should not change a lock based on
// polka in a round earlier than the locked round. The nodes proposes a block
// in round 0, this value receive a polka, not seen by anyone. A second block
// is proposed in round 1, we see the polka and lock it. Then we receive the
// polka from round 0. We don't do anything and remaining locked on round 1.
func TestStateLock_POLSafety1(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// block for round 1, from vs2, empty
	// we build it now, to prevent timeouts
	block1, blockParts1, blockID1 := createProposalBlock(t, cs1)
	prop1 := types.NewProposal(vs2.Height, vs2.Round+1, -1, blockID1, block1.Time)
	signProposal(t, prop1, chainID, vs2)

	// add a tx to the mempool
	tx := kvstore.NewRandomTx(22)
	reqRes, err := assertMempool(cs1.txNotifier).CheckTx(tx, "")
	require.NoError(t, err)
	require.False(t, reqRes.Response.GetCheckTx().IsErr())

	// start the machine
	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)

	// our proposal, it includes tx
	ensureNewProposal(proposalCh, height, round)
	blockID := cs1.GetRoundState().Proposal.BlockID
	require.NotEqual(t, blockID, blockID1)

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	// the others sign a polka in round 0, but no one sees it
	prevotes := signVotes(types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// the others precommit nil
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// precommit nil, locked value remains unset
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	t.Log("### ONTO ROUND 1")
	incrementRound(vs2, vs3, vs4)
	round++

	ensureNewRound(newRoundCh, height, round)
	err = cs1.SetProposalAndBlock(prop1, blockParts1, "some peer")
	require.NoError(t, err)

	// prevote for proposal for block1
	ensureNewProposal(proposalCh, height, round)
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID1.Hash)

	// we see prevotes for it, so we should lock on and precommit it
	signAddVotes(cs1, types.PrevoteType, chainID, blockID1, false, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID1.Hash, blockID1.Hash)

	// the other don't see the polka, so precommit nil
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	t.Log("### ONTO ROUND 2")
	incrementRound(vs2, vs3, vs4)
	round++

	// new round, no proposal, prevote nil
	ensureNewRound(newRoundCh, height, round)
	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// prevotes from the round-2 are added, nothing should change, no new round step
	newStepCh := subscribe(cs1.eventBus, types.EventQueryNewRoundStep)
	addVotes(cs1, prevotes...)
	ensureNoNewRoundStep(newStepCh)

	// receive prevotes for nil, precommit nil, locked round is the same
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round-1, vss[0], nil, blockID1.Hash)
}

// TestStateLock_POLSafety2 tests that a node should not accept a proposal with
// POLRound lower that its locked round. The nodes proposes a block in round 0,
// this value receives a polka, only seen by v3. A second block is proposed in
// round 1, we see the polka and lock it. Then we receive the polka from round
// 0 and the proposal from v3 re-proposing the block originally from round 0.
// We must reject this proposal, since we are locked on round 1.
func TestStateLock_POLSafety2(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// block for round 1, from vs2, empty
	// we build it now, to prevent timeouts
	block1, blockParts1, blockID1 := createProposalBlock(t, cs1)
	prop1 := types.NewProposal(vs2.Height, vs2.Round+1, -1, blockID1, block1.Time)
	signProposal(t, prop1, chainID, vs2)

	// add a tx to the mempool
	tx := kvstore.NewRandomTx(22)
	reqRes, err := assertMempool(cs1.txNotifier).CheckTx(tx, "")
	require.NoError(t, err)
	require.False(t, reqRes.Response.GetCheckTx().IsErr())

	// start the machine
	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)

	// our proposal, it includes tx
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	block0, blockParts0 := rs.ProposalBlock, rs.ProposalBlockParts
	blockID0 := rs.Proposal.BlockID
	require.NotEqual(t, blockID0, blockID1)

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID0.Hash)

	// the others sign a polka in round 0
	prevotes := signVotes(types.PrevoteType, chainID, blockID0, false, vs2, vs3, vs4)

	// v2, v4 precommit nil, as they don't see the polka
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs4)
	// v3 precommits the block, it has seen the polka
	signAddVotes(cs1, types.PrecommitType, chainID, blockID0, true, vs3)

	// conflicting prevots, precommit nil, locked value remains unset
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	t.Log("### ONTO ROUND 1")
	incrementRound(vs2, vs3, vs4)
	round++

	ensureNewRound(newRoundCh, height, round)
	err = cs1.SetProposalAndBlock(prop1, blockParts1, "some peer")
	require.NoError(t, err)

	// prevote for proposal for block1
	ensureNewProposal(proposalCh, height, round)
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID1.Hash)

	// we see prevotes for it, so we should lock on and precommit it
	signAddVotes(cs1, types.PrevoteType, chainID, blockID1, false, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID1.Hash, blockID1.Hash)

	// prevotes from round 0 are late received
	newStepCh := subscribe(cs1.eventBus, types.EventQueryNewRoundStep)
	addVotes(cs1, prevotes...)
	ensureNoNewRoundStep(newStepCh)

	// the other don't see the polka, so precommit nil
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	t.Log("### ONTO ROUND 2")
	incrementRound(vs2, vs3, vs4)
	round++

	// v3 has seen a polka for our block in round 0
	// it re-proposes block 0 with POLRound == 0
	prop2 := types.NewProposal(vs3.Height, vs3.Round, 0, blockID0, block0.Time)
	signProposal(t, prop2, chainID, vs3)

	ensureNewRound(newRoundCh, height, round)
	err = cs1.SetProposalAndBlock(prop2, blockParts0, "some peer")
	require.NoError(t, err)
	ensureNewProposal(proposalCh, height, round)

	// our locked round is 1, so we reject the proposal from v3
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// receive prevotes for nil, precommit nil, locked round is the same
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round-1, vss[0], nil, blockID1.Hash)
}

// TestState_PrevotePOLFromPreviousRound tests that a validator will prevote
// for a block if it is locked on a different block but saw a POL for the block
// it is not locked on in a previous round.
func TestState_PrevotePOLFromPreviousRound(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	lockCh := subscribe(cs1.eventBus, types.EventQueryLock)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	/*
		Round 0:
		cs1 creates a proposal for block B.
		Send a prevote for B from each of the validators to cs1.
		Send a precommit for nil from all of the validators to cs1.

		This ensures that cs1 will lock on B in this round but not precommit it.
	*/
	t.Log("### Starting Round 0")

	startTestRound(cs1, height, round)

	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	r0BlockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, r0BlockID, false, vs2, vs3, vs4)

	// check that the validator generates a Lock event.
	ensureLock(lockCh, height, round)

	// the proposed block should now be locked and our precommit added.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], r0BlockID.Hash, r0BlockID.Hash)

	// add precommits from the rest of the validators.
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 1:
		Create a block, D but do not send a proposal for it to cs1.
		Send a prevote for D from each of the validators to cs1 so that cs1 sees a POL.
		Send a precommit for nil from all of the validators to cs1.

		cs1 has now seen greater than 2/3 of the voting power prevote D in this round
		but cs1 did not see the proposal for D in this round so it will not prevote or precommit it.
	*/
	t.Log("### Starting Round 1")
	incrementRound(vs2, vs3, vs4)
	round++
	// Generate a new proposal block.
	cs2 := newState(cs1.state, vs2, kvstore.NewInMemoryApplication())
	cs2.ValidRound = 1
	propR1, propBlockR1 := decideProposal(ctx, t, cs2, vs2, vs2.Height, round)
	t.Log(propR1.POLRound)
	propBlockR1Parts, err := propBlockR1.MakePartSet(partSize)
	require.NoError(t, err)
	r1BlockID := types.BlockID{
		Hash:          propBlockR1.Hash(),
		PartSetHeader: propBlockR1Parts.Header(),
	}
	require.NotEqual(t, r1BlockID.Hash, r0BlockID.Hash)

	ensureNewRound(newRoundCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, r1BlockID, false, vs2, vs3, vs4)

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)

	// timeout to new round.
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	/*
		Round 2:
		Create a new proposal for D, the same block from Round 1.
		cs1 already saw greater than 2/3 of the voting power on the network vote for
		D in a previous round, so it should prevote D once it receives a proposal for it.

		cs1 does not need to receive prevotes from other validators before the proposal
		in this round. It will still prevote the block.

		Send cs1 prevotes for nil and check that it still prevotes its locked block
		and not the block that it prevoted.
	*/
	t.Log("### Starting Round 2")
	incrementRound(vs2, vs3, vs4)
	round++
	propR2 := types.NewProposal(height, round, 1, r1BlockID, propBlockR1.Header.Time)
	signProposal(t, propR2, chainID, vs3)

	// cs1 receives a proposal for D, the block that received a POL in round 1.
	err = cs1.SetProposalAndBlock(propR2, propBlockR1Parts, "")
	require.NoError(t, err)

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)

	// We should now prevote this block, despite being locked on the block from
	// round 0.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], r1BlockID.Hash)

	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	// cs1 did not receive a POL within this round, so it should remain locked
	// on the block from round 0.
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 0, vss[0], nil, r0BlockID.Hash)
}

// 4 vals.
// polka P0 at R0 for B0. We lock B0 on P0 at R0.

// What we want:
// P0 proposes B0 at R3.
func TestProposeValidBlock(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round and wait for propose and prevote
	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	propBlock := rs.ProposalBlock
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	// the others sign a polka
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	// we should have precommitted the proposed block in this round.

	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	incrementRound(vs2, vs3, vs4)
	round++ // moving to the next round

	ensureNewRound(newRoundCh, height, round)
	t.Log("### ONTO ROUND 1")

	// timeout of propose
	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())

	// We did not see a valid proposal within this round, so prevote nil.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	// we should have precommitted nil during this round because we received
	// >2/3 precommits for nil from the other validators.
	validatePrecommit(t, cs1, round, 0, vss[0], nil, blockID.Hash)

	incrementRound(vs2, vs3, vs4)
	incrementRound(vs2, vs3, vs4)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	round += 2 // increment by multiple rounds

	ensureNewRound(newRoundCh, height, round)
	t.Log("### ONTO ROUND 3")

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	round++ // moving to the next round

	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)

	rs = cs1.GetRoundState()
	assert.True(t, bytes.Equal(rs.ProposalBlock.Hash(), blockID.Hash))
	assert.True(t, bytes.Equal(rs.ProposalBlock.Hash(), rs.ValidBlock.Hash()))
	assert.Equal(t, rs.Proposal.POLRound, rs.ValidRound)
	assert.True(t, bytes.Equal(rs.Proposal.BlockID.Hash, rs.ValidBlock.Hash()))
}

// What we want:
// P0 miss to lock B but set valid block to B after receiving delayed prevote.
func TestSetValidBlockOnDelayedPrevote(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round and wait for propose and prevote
	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	propBlock := rs.ProposalBlock
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	// vs2 send prevote for propBlock
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2)

	// vs3 send prevote nil
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs3)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Prevote(round).Nanoseconds())

	ensurePrecommit(voteCh, height, round)
	// we should have precommitted
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)

	rs = cs1.GetRoundState()

	assert.Nil(t, rs.ValidBlock)
	assert.Nil(t, rs.ValidBlockParts)
	assert.Equal(t, int32(-1), rs.ValidRound)

	// vs2 send (delayed) prevote for propBlock
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs4)

	ensureNewValidBlock(validBlockCh, height, round)

	rs = cs1.GetRoundState()

	assert.True(t, bytes.Equal(rs.ValidBlock.Hash(), blockID.Hash))
	assert.True(t, rs.ValidBlockParts.Header().Equals(blockID.PartSetHeader))
	assert.Equal(t, rs.ValidRound, round)
}

// What we want:
// P0 miss to lock B as Proposal Block is missing, but set valid block to B after
// receiving delayed Block Proposal.
func TestSetValidBlockOnDelayedProposal(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)

	round++ // move to round in which P0 is not proposer
	incrementRound(vs2, vs3, vs4)

	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	prop, propBlock := decideProposal(ctx, t, cs1, vs2, vs2.Height, vs2.Round+1)
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	// vs2, vs3 and vs4 send prevote for propBlock
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)
	ensureNewValidBlock(validBlockCh, height, round)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Prevote(round).Nanoseconds())

	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)

	partSet, err = propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	err = cs1.SetProposalAndBlock(prop, partSet, "some peer")
	require.NoError(t, err)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()

	assert.True(t, bytes.Equal(rs.ValidBlock.Hash(), blockID.Hash))
	assert.True(t, rs.ValidBlockParts.Header().Equals(blockID.PartSetHeader))
	assert.Equal(t, rs.ValidRound, round)
}

func TestProcessProposalAccept(t *testing.T) {
	for _, testCase := range []struct {
		name               string
		accept             bool
		expectedNilPrevote bool
	}{
		{
			name:               "accepted block is prevoted",
			accept:             true,
			expectedNilPrevote: false,
		},
		{
			name:               "rejected block is not prevoted",
			accept:             false,
			expectedNilPrevote: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			m := abcimocks.NewApplication(t)
			status := abci.PROCESS_PROPOSAL_STATUS_REJECT
			if testCase.accept {
				status = abci.PROCESS_PROPOSAL_STATUS_ACCEPT
			}
			m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{Status: status}, nil)
			m.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil).Maybe()
			cs1, _ := randStateWithApp(4, m)
			height, round := cs1.Height, cs1.Round

			proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
			newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
			pv1, err := cs1.privValidator.GetPubKey()
			require.NoError(t, err)
			addr := pv1.Address()
			voteCh := subscribeToVoter(cs1, addr)

			startTestRound(cs1, cs1.Height, round)
			ensureNewRound(newRoundCh, height, round)

			ensureNewProposal(proposalCh, height, round)
			rs := cs1.GetRoundState()
			var prevoteHash cmtbytes.HexBytes
			if !testCase.expectedNilPrevote {
				prevoteHash = rs.ProposalBlock.Hash()
			}
			ensurePrevoteMatch(t, voteCh, height, round, prevoteHash)
		})
	}
}

// TestExtendVoteCalledWhenEnabled tests that the vote extension methods are called at the
// correct point in the consensus algorithm when vote extensions are enabled.
func TestExtendVoteCalledWhenEnabled(t *testing.T) {
	for _, testCase := range []struct {
		name    string
		enabled bool
	}{
		{
			name:    "enabled",
			enabled: true,
		},
		{
			name:    "disabled",
			enabled: false,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			m := abcimocks.NewApplication(t)
			m.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil)
			m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil)
			if testCase.enabled {
				m.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{
					VoteExtension: []byte("extension"),
				}, nil)
				m.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{
					Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
				}, nil)
			}
			m.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()
			m.On("FinalizeBlock", mock.Anything, mock.Anything).Return(&abci.FinalizeBlockResponse{}, nil).Maybe()
			height := int64(1)
			if !testCase.enabled {
				height = 0
			}
			cs1, vss := randStateWithAppWithHeight(4, m, height)
			height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

			proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
			newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
			pv1, err := cs1.privValidator.GetPubKey()
			require.NoError(t, err)
			addr := pv1.Address()
			voteCh := subscribeToVoter(cs1, addr)

			startTestRound(cs1, cs1.Height, round)
			ensureNewRound(newRoundCh, height, round)
			ensureNewProposal(proposalCh, height, round)

			m.AssertNotCalled(t, "ExtendVote", mock.Anything, mock.Anything)

			rs := cs1.GetRoundState()
			blockID := types.BlockID{
				Hash:          rs.ProposalBlock.Hash(),
				PartSetHeader: rs.ProposalBlockParts.Header(),
			}
			signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[1:]...)
			ensurePrevoteMatch(t, voteCh, height, round, blockID.Hash)

			ensurePrecommit(voteCh, height, round)

			if testCase.enabled {
				m.AssertCalled(t, "ExtendVote", context.TODO(), &abci.ExtendVoteRequest{
					Height:             height,
					Hash:               blockID.Hash,
					Time:               rs.ProposalBlock.Time,
					Txs:                rs.ProposalBlock.Txs.ToSliceOfBytes(),
					ProposedLastCommit: abci.CommitInfo{},
					Misbehavior:        rs.ProposalBlock.Evidence.Evidence.ToABCI(),
					NextValidatorsHash: rs.ProposalBlock.NextValidatorsHash,
					ProposerAddress:    rs.ProposalBlock.ProposerAddress,
				})
			} else {
				m.AssertNotCalled(t, "ExtendVote", mock.Anything, mock.Anything)
			}

			signAddVotes(cs1, types.PrecommitType, chainID, blockID, testCase.enabled, vss[1:]...)
			ensureNewRound(newRoundCh, height+1, 0)
			m.AssertExpectations(t)

			// Only 3 of the vote extensions are seen, as consensus proceeds as soon as the +2/3 threshold
			// is observed by the consensus engine.
			for _, pv := range vss[1:3] {
				pv, err := pv.GetPubKey()
				require.NoError(t, err)
				addr := pv.Address()
				if testCase.enabled {
					m.AssertCalled(t, "VerifyVoteExtension", context.TODO(), &abci.VerifyVoteExtensionRequest{
						Hash:             blockID.Hash,
						ValidatorAddress: addr,
						Height:           height,
						VoteExtension:    []byte("extension"),
					})
				} else {
					m.AssertNotCalled(t, "VerifyVoteExtension", mock.Anything, mock.Anything)
				}
			}
		})
	}
}

// TestVerifyVoteExtensionNotCalledOnAbsentPrecommit tests that the VerifyVoteExtension
// method is not called for a validator's vote that is never delivered.
func TestVerifyVoteExtensionNotCalledOnAbsentPrecommit(t *testing.T) {
	m := abcimocks.NewApplication(t)
	m.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil)
	m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil)
	m.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{
		VoteExtension: []byte("extension"),
	}, nil)
	m.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{
		Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
	}, nil)
	m.On("FinalizeBlock", mock.Anything, mock.Anything).Return(&abci.FinalizeBlockResponse{}, nil).Maybe()
	m.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()
	cs1, vss := randStateWithApp(4, m)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	cs1.state.ConsensusParams.Feature.VoteExtensionsEnableHeight = cs1.Height

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	startTestRound(cs1, cs1.Height, round)
	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()

	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss...)
	ensurePrevoteMatch(t, voteCh, height, round, blockID.Hash)

	ensurePrecommit(voteCh, height, round)

	m.AssertCalled(t, "ExtendVote", context.TODO(), &abci.ExtendVoteRequest{
		Height:             height,
		Hash:               blockID.Hash,
		Time:               rs.ProposalBlock.Time,
		Txs:                rs.ProposalBlock.Txs.ToSliceOfBytes(),
		ProposedLastCommit: abci.CommitInfo{},
		Misbehavior:        rs.ProposalBlock.Evidence.Evidence.ToABCI(),
		NextValidatorsHash: rs.ProposalBlock.NextValidatorsHash,
		ProposerAddress:    rs.ProposalBlock.ProposerAddress,
	})

	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vss[2:]...)
	ensureNewRound(newRoundCh, height+1, 0)
	m.AssertExpectations(t)

	// vss[1] did not issue a precommit for the block, ensure that a vote extension
	// for its address was not sent to the application.
	pv, err := vss[1].GetPubKey()
	require.NoError(t, err)
	addr = pv.Address()

	m.AssertNotCalled(t, "VerifyVoteExtension", context.TODO(), &abci.VerifyVoteExtensionRequest{
		Hash:             blockID.Hash,
		ValidatorAddress: addr,
		Height:           height,
		VoteExtension:    []byte("extension"),
	})
}

// TestPrepareProposalReceivesVoteExtensions tests that the PrepareProposal method
// is called with the vote extensions from the previous height. The test functions
// by completing a consensus height with a mock application as the proposer. The
// test then proceeds to fail several rounds of consensus until the mock application
// is the proposer again and ensures that the mock application receives the set of
// vote extensions from the previous consensus instance.
func TestPrepareProposalReceivesVoteExtensions(t *testing.T) {
	// create a list of vote extensions, one for each validator.
	voteExtensions := [][]byte{
		[]byte("extension 0"),
		[]byte("extension 1"),
		[]byte("extension 2"),
		[]byte("extension 3"),
	}

	m := abcimocks.NewApplication(t)
	m.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{
		VoteExtension: voteExtensions[0],
	}, nil)
	m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT}, nil)

	// capture the prepare proposal request.
	rpp := &abci.PrepareProposalRequest{}
	m.On("PrepareProposal", mock.Anything, mock.MatchedBy(func(r *abci.PrepareProposalRequest) bool {
		rpp = r
		return true
	})).Return(&abci.PrepareProposalResponse{}, nil)

	m.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT}, nil)
	m.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()
	m.On("FinalizeBlock", mock.Anything, mock.Anything).Return(&abci.FinalizeBlockResponse{}, nil)

	cs1, vss := randStateWithApp(4, m)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)

	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[1:]...)

	// create a precommit for each validator with the associated vote extension.
	for i, vs := range vss[1:] {
		signAddPrecommitWithExtension(t, cs1, chainID, blockID, voteExtensions[i+1], vs)
	}

	ensurePrevote(voteCh, height, round)

	// ensure that the height is committed.
	ensurePrecommitMatch(t, voteCh, height, round, blockID.Hash)
	incrementHeight(vss[1:]...)

	height++
	round = 0
	ensureNewRound(newRoundCh, height, round)
	incrementRound(vss[1:]...)
	incrementRound(vss[1:]...)
	incrementRound(vss[1:]...)
	round = 3

	blockID2 := types.BlockID{}
	signAddVotes(cs1, types.PrecommitType, chainID, blockID2, true, vss[1:]...)
	ensureNewRound(newRoundCh, height, round)
	ensureNewProposal(proposalCh, height, round)

	// ensure that the proposer received the list of vote extensions from the
	// previous height.
	require.Len(t, rpp.LocalLastCommit.Votes, len(vss))
	for i := range vss {
		vote := &rpp.LocalLastCommit.Votes[i]
		require.Equal(t, vote.VoteExtension, voteExtensions[i])

		require.NotZero(t, len(vote.ExtensionSignature))
		cve := cmtproto.CanonicalVoteExtension{
			Extension: vote.VoteExtension,
			Height:    height - 1, // the vote extension was signed in the previous height
			Round:     int64(rpp.LocalLastCommit.Round),
			ChainId:   test.DefaultTestChainID,
		}
		extSignBytes, err := protoio.MarshalDelimited(&cve)
		require.NoError(t, err)
		pubKey, err := vss[i].PrivValidator.GetPubKey()
		require.NoError(t, err)
		require.True(t, pubKey.VerifySignature(extSignBytes, vote.ExtensionSignature))
	}
}

func TestFinalizeBlockCalled(t *testing.T) {
	for _, testCase := range []struct {
		name         string
		voteNil      bool
		expectCalled bool
	}{
		{
			name:         "finalize block called when block committed",
			voteNil:      false,
			expectCalled: true,
		},
		{
			name:         "not called when block not committed",
			voteNil:      true,
			expectCalled: false,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			m := abcimocks.NewApplication(t)
			m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{
				Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT,
			}, nil)
			m.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil)
			// We only expect VerifyVoteExtension to be called on non-nil precommits.
			// https://github.com/tendermint/tendermint/issues/8487
			if !testCase.voteNil {
				m.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{}, nil)
				m.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{
					Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
				}, nil)
			}
			r := &abci.FinalizeBlockResponse{AppHash: []byte("the_hash")}
			m.On("FinalizeBlock", mock.Anything, mock.Anything).Return(r, nil).Maybe()
			m.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()

			cs1, vss := randStateWithApp(4, m)
			height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

			proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
			newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
			pv1, err := cs1.privValidator.GetPubKey()
			require.NoError(t, err)
			addr := pv1.Address()
			voteCh := subscribeToVoter(cs1, addr)

			startTestRound(cs1, cs1.Height, round)
			ensureNewRound(newRoundCh, height, round)
			ensureNewProposal(proposalCh, height, round)
			rs := cs1.GetRoundState()

			blockID := types.BlockID{}
			nextRound := round + 1
			nextHeight := height
			if !testCase.voteNil {
				nextRound = 0
				nextHeight = height + 1
				blockID = types.BlockID{
					Hash:          rs.ProposalBlock.Hash(),
					PartSetHeader: rs.ProposalBlockParts.Header(),
				}
			}

			signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[1:]...)
			ensurePrevoteMatch(t, voteCh, height, round, rs.ProposalBlock.Hash())

			signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vss[1:]...)
			ensurePrecommit(voteCh, height, round)

			ensureNewRound(newRoundCh, nextHeight, nextRound)
			m.AssertExpectations(t)

			if !testCase.expectCalled {
				m.AssertNotCalled(t, "FinalizeBlock", context.TODO(), mock.Anything)
			} else {
				m.AssertCalled(t, "FinalizeBlock", context.TODO(), mock.Anything)
			}
		})
	}
}

// TestVoteExtensionEnableHeight tests that 'ExtensionRequireHeight' correctly
// enforces that vote extensions be present in consensus for heights greater than
// or equal to the configured value.
func TestVoteExtensionEnableHeight(t *testing.T) {
	for _, testCase := range []struct {
		name                  string
		enableHeight          int64
		hasExtension          bool
		expectExtendCalled    bool
		expectVerifyCalled    bool
		expectSuccessfulRound bool
	}{
		{
			name:                  "extension present but not enabled",
			hasExtension:          true,
			enableHeight:          0,
			expectExtendCalled:    false,
			expectVerifyCalled:    false,
			expectSuccessfulRound: false,
		},
		{
			name:                  "extension absent but not required",
			hasExtension:          false,
			enableHeight:          0,
			expectExtendCalled:    false,
			expectVerifyCalled:    false,
			expectSuccessfulRound: true,
		},
		{
			name:                  "extension present and required",
			hasExtension:          true,
			enableHeight:          1,
			expectExtendCalled:    true,
			expectVerifyCalled:    true,
			expectSuccessfulRound: true,
		},
		{
			name:                  "extension absent but required",
			hasExtension:          false,
			enableHeight:          1,
			expectExtendCalled:    true,
			expectVerifyCalled:    false,
			expectSuccessfulRound: false,
		},
		{
			name:                  "extension absent but required in future height",
			hasExtension:          false,
			enableHeight:          2,
			expectExtendCalled:    false,
			expectVerifyCalled:    false,
			expectSuccessfulRound: true,
		},
	} {
		t.Run(testCase.name, func(t *testing.T) {
			numValidators := 3
			m := abcimocks.NewApplication(t)
			m.On("ProcessProposal", mock.Anything, mock.Anything).Return(&abci.ProcessProposalResponse{
				Status: abci.PROCESS_PROPOSAL_STATUS_ACCEPT,
			}, nil)
			m.On("PrepareProposal", mock.Anything, mock.Anything).Return(&abci.PrepareProposalResponse{}, nil)
			if testCase.expectExtendCalled {
				m.On("ExtendVote", mock.Anything, mock.Anything).Return(&abci.ExtendVoteResponse{}, nil)
			}
			if testCase.expectVerifyCalled {
				m.On("VerifyVoteExtension", mock.Anything, mock.Anything).Return(&abci.VerifyVoteExtensionResponse{
					Status: abci.VERIFY_VOTE_EXTENSION_STATUS_ACCEPT,
				}, nil).Times(numValidators - 1)
			}
			m.On("FinalizeBlock", mock.Anything, mock.Anything).Return(&abci.FinalizeBlockResponse{}, nil).Maybe()
			m.On("Commit", mock.Anything, mock.Anything).Return(&abci.CommitResponse{}, nil).Maybe()
			cs1, vss := randStateWithAppWithHeight(numValidators, m, testCase.enableHeight)
			height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
			cs1.state.ConsensusParams.Feature.VoteExtensionsEnableHeight = testCase.enableHeight

			timeoutCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
			proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
			newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
			pv1, err := cs1.privValidator.GetPubKey()
			require.NoError(t, err)
			addr := pv1.Address()
			voteCh := subscribeToVoter(cs1, addr)

			startTestRound(cs1, cs1.Height, round)
			ensureNewRound(newRoundCh, height, round)
			ensureNewProposal(proposalCh, height, round)
			rs := cs1.GetRoundState()
			blockID := types.BlockID{
				Hash:          rs.ProposalBlock.Hash(),
				PartSetHeader: rs.ProposalBlockParts.Header(),
			}

			// sign all of the votes
			signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vss[1:]...)
			ensurePrevoteMatch(t, voteCh, height, round, rs.ProposalBlock.Hash())

			var ext []byte
			if testCase.hasExtension {
				ext = []byte("extension")
			}

			for _, vs := range vss[1:] {
				vote, err := vs.signVote(types.PrecommitType, chainID, blockID, ext, testCase.hasExtension, vs.clock.Now())
				require.NoError(t, err)
				addVotes(cs1, vote)
			}
			if testCase.expectSuccessfulRound {
				ensurePrecommit(voteCh, height, round)
				height++
				ensureNewRound(newRoundCh, height, round)
			} else {
				ensureNoNewTimeout(timeoutCh, cs1.config.Precommit(round).Nanoseconds())
			}

			m.AssertExpectations(t)
		})
	}
}

// 4 vals, 3 Nil Precommits at P0
// What we want:
// P0 waits for timeoutPrecommit before starting next round.
func TestWaitingTimeoutOnNilPolka(*testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)

	// start round
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())
	ensureNewRound(newRoundCh, height, round+1)
}

// 4 vals, 3 Prevotes for nil from the higher round.
// What we want:
// P0 waits for timeoutPropose in the next round before entering prevote.
func TestWaitingTimeoutProposeOnNewRound(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensurePrevote(voteCh, height, round)

	incrementRound(vss[1:]...)
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	round++ // moving to the next round
	ensureNewRound(newRoundCh, height, round)

	rs := cs1.GetRoundState()
	assert.Equal(t, cstypes.RoundStepPropose, rs.Step) // P0 does not prevote before timeoutPropose expires

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Propose(round).Nanoseconds())

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)
}

// 4 vals, 3 Precommits for nil from the higher round.
// What we want:
// P0 jump to higher round, precommit and start precommit wait.
func TestRoundSkipOnNilPolkaFromHigherRound(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensurePrevote(voteCh, height, round)

	incrementRound(vss[1:]...)
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2, vs3, vs4)

	round++ // moving to the next round
	ensureNewRound(newRoundCh, height, round)

	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, -1, vss[0], nil, nil)

	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	round++ // moving to the next round
	ensureNewRound(newRoundCh, height, round)
}

// 4 vals, 3 Prevotes for nil in the current round.
// What we want:
// P0 wait for timeoutPropose to expire before sending prevote.
func TestWaitTimeoutProposeOnNilPolkaForTheCurrentRound(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, int32(1), cs1.state.ChainID

	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round in which PO is not proposer
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	incrementRound(vss[1:]...)
	signAddVotes(cs1, types.PrevoteType, chainID, types.BlockID{}, false, vs2, vs3, vs4)

	ensureNewTimeout(timeoutProposeCh, height, round, cs1.config.Propose(round).Nanoseconds())

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)
}

// What we want:
// P0 emit NewValidBlock event upon receiving 2/3+ Precommit for B but hasn't received block B yet.
func TestEmitNewValidBlockEventOnCommitWithoutBlock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, int32(1), cs1.state.ChainID

	incrementRound(vs2, vs3, vs4)

	partSize := types.BlockPartSizeBytes

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)

	_, propBlock := decideProposal(ctx, t, cs1, vs2, vs2.Height, vs2.Round)
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	// start round in which PO is not proposer
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	// vs2, vs3 and vs4 send precommit for propBlock
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs2, vs3, vs4)
	ensureNewValidBlock(validBlockCh, height, round)

	rs := cs1.GetRoundState()
	assert.Equal(t, rs.Step, cstypes.RoundStepCommit) //nolint:testifylint // this will tell us to reverse the items being compared no matter what
	assert.Nil(t, rs.ProposalBlock)
	assert.True(t, rs.ProposalBlockParts.Header().Equals(blockID.PartSetHeader))
}

// What we want:
// P0 receives 2/3+ Precommit for B for round 0, while being in round 1. It emits NewValidBlock event.
// After receiving block, it executes block and moves to the next height.
func TestCommitFromPreviousRound(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, int32(1), cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	validBlockCh := subscribe(cs1.eventBus, types.EventQueryValidBlock)
	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)

	prop, propBlock := decideProposal(ctx, t, cs1, vs2, vs2.Height, vs2.Round)
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	// start round in which PO is not proposer
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	// vs2, vs3 and vs4 send precommit for propBlock for the previous round
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs2, vs3, vs4)

	ensureNewValidBlock(validBlockCh, height, round)

	rs := cs1.GetRoundState()
	assert.Equal(t, cstypes.RoundStepCommit, rs.Step)
	assert.Equal(t, vs2.Round, rs.CommitRound)
	assert.Nil(t, rs.ProposalBlock, nil)
	assert.True(t, rs.ProposalBlockParts.Header().Equals(blockID.PartSetHeader))
	partSet, err = propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	err = cs1.SetProposalAndBlock(prop, partSet, "some peer")
	require.NoError(t, err)

	ensureNewProposal(proposalCh, height, round)
	ensureNewRound(newRoundCh, height+1, 0)
}

type fakeTxNotifier struct {
	ch chan struct{}
}

func (n *fakeTxNotifier) TxsAvailable() <-chan struct{} {
	return n.ch
}

func (n *fakeTxNotifier) Notify() {
	n.ch <- struct{}{}
}

// 2 vals precommit votes for a block but node times out waiting for the third. Move to next round
// and third precommit arrives which leads to the commit of that header and the correct
// start of the next round.
func TestStartNextHeightCorrectlyAfterTimeout(t *testing.T) {
	config.Consensus.SkipTimeoutCommit = false //nolint:staticcheck
	config.Consensus.TimeoutCommit = 10 * time.Millisecond
	cs1, vss := randState(4)
	cs1.txNotifier = &fakeTxNotifier{ch: make(chan struct{})}

	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutProposeCh := subscribe(cs1.eventBus, types.EventQueryTimeoutPropose)
	precommitTimeoutCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	newBlockHeader := subscribe(cs1.eventBus, types.EventQueryNewBlockHeader)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	// the proposed block should now be locked and our precommit added
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2)
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs3)

	// wait till timeout occurs
	ensureNewTimeout(precommitTimeoutCh, height, round, cs1.config.TimeoutPrecommit.Nanoseconds())

	ensureNewRound(newRoundCh, height, round+1)

	// majority is now reached
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs4)

	ensureNewBlockHeader(newBlockHeader, height, blockID.Hash)

	cs1.txNotifier.(*fakeTxNotifier).Notify()

	ensureNewTimeout(timeoutProposeCh, height+1, round, cs1.config.Propose(round).Nanoseconds())
	rs = cs1.GetRoundState()
	assert.False(
		t,
		rs.TriggeredTimeoutPrecommit,
		"triggeredTimeoutPrecommit should be false at the beginning of each round")
}

func TestResetTimeoutPrecommitUponNewHeight(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	config.Consensus.SkipTimeoutCommit = false //nolint:staticcheck
	config.Consensus.TimeoutCommit = 10 * time.Millisecond
	cs1, vss := randState(4)

	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID

	partSize := types.BlockPartSizeBytes

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)

	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	newBlockHeader := subscribe(cs1.eventBus, types.EventQueryNewBlockHeader)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	blockID := types.BlockID{
		Hash:          rs.ProposalBlock.Hash(),
		PartSetHeader: rs.ProposalBlockParts.Header(),
	}

	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], blockID.Hash)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)

	// add precommits
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2)
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs3)
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs4)

	ensureNewBlockHeader(newBlockHeader, height, blockID.Hash)

	prop, propBlock := decideProposal(ctx, t, cs1, vs2, height+1, 0)
	propBlockParts, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)

	err = cs1.SetProposalAndBlock(prop, propBlockParts, "some peer")
	require.NoError(t, err)
	ensureNewProposal(proposalCh, height+1, 0)

	rs = cs1.GetRoundState()
	assert.False(
		t,
		rs.TriggeredTimeoutPrecommit,
		"triggeredTimeoutPrecommit should be false at the beginning of each height")
}

// ------------------------------------------------------------------------------------------
// CatchupSuite

// ------------------------------------------------------------------------------------------
// HaltSuite

// 4 vals.
// we receive a final precommit after going into next round, but others might have gone to commit already!
func TestStateHalt1(t *testing.T) {
	cs1, vss := randState(4)
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	partSize := types.BlockPartSizeBytes

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	timeoutWaitCh := subscribe(cs1.eventBus, types.EventQueryTimeoutWait)
	newRoundCh := subscribe(cs1.eventBus, types.EventQueryNewRound)
	newBlockCh := subscribe(cs1.eventBus, types.EventQueryNewBlock)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	// start round and wait for propose and prevote
	startTestRound(cs1, height, round)
	ensureNewRound(newRoundCh, height, round)

	ensureNewProposal(proposalCh, height, round)
	rs := cs1.GetRoundState()
	propBlock := rs.ProposalBlock
	partSet, err := propBlock.MakePartSet(partSize)
	require.NoError(t, err)
	blockID := types.BlockID{
		Hash:          propBlock.Hash(),
		PartSetHeader: partSet.Header(),
	}

	ensurePrevote(voteCh, height, round)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	ensurePrecommit(voteCh, height, round)
	// the proposed block should now be locked and our precommit added
	validatePrecommit(t, cs1, round, round, vss[0], propBlock.Hash(), propBlock.Hash())

	// add precommits from the rest
	signAddVotes(cs1, types.PrecommitType, chainID, types.BlockID{}, true, vs2) // didn't receive proposal
	signAddVotes(cs1, types.PrecommitType, chainID, blockID, true, vs3)
	// we receive this later, but vs3 might receive it earlier and with ours will go to commit!
	precommit4 := signVote(vs4, types.PrecommitType, chainID, blockID, true)

	incrementRound(vs2, vs3, vs4)

	// timeout to new round
	ensureNewTimeout(timeoutWaitCh, height, round, cs1.config.Precommit(round).Nanoseconds())

	round++ // moving to the next round

	ensureNewRound(newRoundCh, height, round)

	t.Log("### ONTO ROUND 1")
	/* Round2
	// we timeout and prevote
	// a polka happened but we didn't see it!
	*/

	// prevote for nil since we did not receive a proposal in this round.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// now we receive the precommit from the previous round
	addVotes(cs1, precommit4)

	// receiving that precommit should take us straight to commit
	ensureNewBlock(newBlockCh, height)

	ensureNewRound(newRoundCh, height+1, 0)
}

func TestStateOutputsBlockPartsStats(t *testing.T) {
	// create dummy peer
	cs, _ := randState(1)
	peer := p2pmock.NewPeer(nil)

	// 1) new block part
	parts := types.NewPartSetFromData(cmtrand.Bytes(100), 10)
	msg := &BlockPartMessage{
		Height: 1,
		Round:  0,
		Part:   parts.GetPart(0),
	}

	cs.ProposalBlockParts = types.NewPartSetFromHeader(parts.Header())
	cs.handleMsg(msgInfo{msg, peer.ID(), time.Time{}})

	statsMessage := <-cs.statsMsgQueue
	require.Equal(t, msg, statsMessage.Msg, "")
	require.Equal(t, peer.ID(), statsMessage.PeerID, "")

	// sending the same part from different peer
	cs.handleMsg(msgInfo{msg, "peer2", time.Time{}})

	// sending the part with the same height, but different round
	msg.Round = 1
	cs.handleMsg(msgInfo{msg, peer.ID(), time.Time{}})

	// sending the part from the smaller height
	msg.Height = 0
	cs.handleMsg(msgInfo{msg, peer.ID(), time.Time{}})

	// sending the part from the bigger height
	msg.Height = 3
	cs.handleMsg(msgInfo{msg, peer.ID(), time.Time{}})

	select {
	case <-cs.statsMsgQueue:
		t.Errorf("should not output stats message after receiving the known block part!")
	case <-time.After(50 * time.Millisecond):
	}
}

func TestStateOutputVoteStats(t *testing.T) {
	cs, vss := randState(2)
	chainID := cs.state.ChainID
	// create dummy peer
	peer := p2pmock.NewPeer(nil)

	randBytes := cmtrand.Bytes(tmhash.Size)
	blockID := types.BlockID{
		Hash: randBytes,
	}

	vote := signVote(vss[1], types.PrecommitType, chainID, blockID, true)

	voteMessage := &VoteMessage{vote}
	cs.handleMsg(msgInfo{voteMessage, peer.ID(), time.Time{}})

	statsMessage := <-cs.statsMsgQueue
	require.Equal(t, voteMessage, statsMessage.Msg, "")
	require.Equal(t, peer.ID(), statsMessage.PeerID, "")

	// sending the same part from different peer
	cs.handleMsg(msgInfo{&VoteMessage{vote}, "peer2", time.Time{}})

	// sending the vote for the bigger height
	incrementHeight(vss[1])
	vote = signVote(vss[1], types.PrecommitType, chainID, blockID, true)

	cs.handleMsg(msgInfo{&VoteMessage{vote}, peer.ID(), time.Time{}})

	select {
	case <-cs.statsMsgQueue:
		t.Errorf("should not output stats message after receiving the known vote or vote from bigger height")
	case <-time.After(50 * time.Millisecond):
	}
}

func TestSignSameVoteTwice(t *testing.T) {
	cs, vss := randState(2)
	chainID := cs.state.ChainID

	randBytes := cmtrand.Bytes(tmhash.Size)

	vote := signVote(vss[1],
		types.PrecommitType,
		chainID,
		types.BlockID{
			Hash:          randBytes,
			PartSetHeader: types.PartSetHeader{Total: 10, Hash: randBytes},
		},
		true,
	)

	vote2 := signVote(vss[1],
		types.PrecommitType,
		chainID,
		types.BlockID{
			Hash:          randBytes,
			PartSetHeader: types.PartSetHeader{Total: 10, Hash: randBytes},
		},
		true,
	)

	require.Equal(t, vote, vote2)
}

// TestStateTimestamp_ProposalNotMatch tests that a validator does not prevote a
// proposed block if the timestamp in the block does not match the timestamp in the
// corresponding proposal message.
func TestStateTimestamp_ProposalNotMatch(t *testing.T) {
	cs1, vss := randState(4)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	propBlock, propBlockParts, blockID := createProposalBlock(t, cs1)

	round++
	incrementRound(vss[1:]...)

	// Create a proposal with a timestamp that does not match the timestamp of the block.
	proposal := types.NewProposal(vs2.Height, round, -1, blockID, propBlock.Header.Time.Add(time.Millisecond))
	signProposal(t, proposal, chainID, vs2)
	require.NoError(t, cs1.SetProposalAndBlock(proposal, propBlockParts, "some peer"))

	startTestRound(cs1, height, round)
	ensureProposal(proposalCh, height, round, blockID)

	// ensure that the validator prevotes nil.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], nil)

	// This does not refer to the main concern of this test unit. Since
	// 2/3+ validators have seen the proposal, validated and prevoted for
	// it, it is a valid proposal. We should lock and precommit for it.
	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)
	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, round, vss[0], blockID.Hash, blockID.Hash)
}

// TestStateTimestamp_ProposalMatch tests that a validator prevotes a
// proposed block if the timestamp in the block matches the timestamp in the
// corresponding proposal message.
func TestStateTimestamp_ProposalMatch(t *testing.T) {
	cs1, vss := randState(4)
	height, round, chainID := cs1.Height, cs1.Round, cs1.state.ChainID
	vs2, vs3, vs4 := vss[1], vss[2], vss[3]

	proposalCh := subscribe(cs1.eventBus, types.EventQueryCompleteProposal)
	pv1, err := cs1.privValidator.GetPubKey()
	require.NoError(t, err)
	addr := pv1.Address()
	voteCh := subscribeToVoter(cs1, addr)

	propBlock, propBlockParts, blockID := createProposalBlock(t, cs1)

	round++
	incrementRound(vss[1:]...)

	// Create a proposal with a timestamp that matches the timestamp of the block.
	proposal := types.NewProposal(vs2.Height, round, -1, blockID, propBlock.Header.Time)
	signProposal(t, proposal, chainID, vs2)
	require.NoError(t, cs1.SetProposalAndBlock(proposal, propBlockParts, "some peer"))

	startTestRound(cs1, height, round)
	ensureProposal(proposalCh, height, round, blockID)

	signAddVotes(cs1, types.PrevoteType, chainID, blockID, false, vs2, vs3, vs4)

	// ensure that the validator prevotes the block.
	ensurePrevote(voteCh, height, round)
	validatePrevote(t, cs1, round, vss[0], propBlock.Hash())

	ensurePrecommit(voteCh, height, round)
	validatePrecommit(t, cs1, round, 1, vss[0], propBlock.Hash(), propBlock.Hash())
}

// subscribe subscribes test client to the given query and returns a channel with cap = 1.
func subscribe(eventBus *types.EventBus, q cmtpubsub.Query) <-chan cmtpubsub.Message {
	sub, err := eventBus.Subscribe(context.Background(), testSubscriber, q)
	if err != nil {
		panic(fmt.Sprintf("failed to subscribe %s to %v; err %v", testSubscriber, q, err))
	}
	return sub.Out()
}

// subscribe subscribes test client to the given query and returns a channel with cap = 1.
func unsubscribe(eventBus *types.EventBus, q cmtpubsub.Query) { //nolint: unused
	err := eventBus.Unsubscribe(context.Background(), testSubscriber, q)
	if err != nil {
		panic(fmt.Sprintf("failed to subscribe %s to %v; err %v", testSubscriber, q, err))
	}
}

// subscribe subscribes test client to the given query and returns a channel with cap = 0.
func subscribeUnBuffered(eventBus *types.EventBus, q cmtpubsub.Query) <-chan cmtpubsub.Message {
	sub, err := eventBus.SubscribeUnbuffered(context.Background(), testSubscriber, q)
	if err != nil {
		panic(fmt.Sprintf("failed to subscribe %s to %v; err %v", testSubscriber, q, err))
	}
	return sub.Out()
}

func signAddPrecommitWithExtension(
	t *testing.T,
	cs *State,
	chainID string,
	blockID types.BlockID,
	extension []byte,
	stub *validatorStub,
) {
	t.Helper()
	v, err := stub.signVote(types.PrecommitType, chainID, blockID, extension, true, stub.clock.Now())
	require.NoError(t, err, "failed to sign vote")
	addVotes(cs, v)
}

func findBlockSizeLimit(t *testing.T, height, maxBytes int64, cs *State, partSize uint32, oversized bool) (*types.Block, *types.PartSet) {
	t.Helper()
	var offset int64
	if !oversized {
		offset = -2
	}
	softMaxDataBytes := int(types.MaxDataBytes(maxBytes, 0, 0))
	for i := softMaxDataBytes; i < softMaxDataBytes*2; i++ {
		propBlock := cs.state.MakeBlock(
			height,
			[]types.Tx{[]byte("a=" + strings.Repeat("o", i-2))},
			&types.Commit{},
			nil,
			cs.privValidatorPubKey.Address(),
		)

		propBlockParts, err := propBlock.MakePartSet(partSize)
		require.NoError(t, err)
		if propBlockParts.ByteSize() > maxBytes+offset {
			s := "real max"
			if oversized {
				s = "off-by-1"
			}
			t.Log("Detected "+s+" data size for block;", "size", i, "softMaxDataBytes", softMaxDataBytes)
			return propBlock, propBlockParts
		}
	}
	require.Fail(t, "We shouldn't hit the end of the loop")
	return nil, nil
}

// TestReadSerializedBlockFromBlockParts tests that the readSerializedBlockFromBlockParts function
// reads the block correctly from the block parts.
func TestReadSerializedBlockFromBlockParts(t *testing.T) {
	sizes := []int{0, 5, 64, 70, 128, 200}

	// iterate through many initial buffer sizes and new block sizes.
	// (Skip new block size = 0, as that is not valid construction)
	// Ensure that we read back the correct block size, and the buffer is resized correctly.
	for i := 0; i < len(sizes); i++ {
		for j := 1; j < len(sizes); j++ {
			initialSize, newBlockSize := sizes[i], sizes[j]
			testName := fmt.Sprintf("initialSize=%d,newBlockSize=%d", initialSize, newBlockSize)
			t.Run(testName, func(t *testing.T) {
				blockData := cmtrand.Bytes(newBlockSize)
				ps := types.NewPartSetFromData(blockData, 64)
				cs := &State{
					serializedBlockBuffer: make([]byte, initialSize),
				}
				cs.ProposalBlockParts = ps

				serializedBlock, err := cs.readSerializedBlockFromBlockParts()
				require.NoError(t, err)
				require.Equal(t, blockData, serializedBlock)
				require.Equal(t, len(cs.serializedBlockBuffer), max(initialSize, newBlockSize))
			})
		}
	}
}
