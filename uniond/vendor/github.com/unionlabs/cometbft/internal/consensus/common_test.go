package consensus

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"os"
	"path"
	"path/filepath"
	"sort"
	"sync"
	"testing"
	"time"

	"github.com/go-kit/log/term"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	abcicli "github.com/cometbft/cometbft/abci/client"
	"github.com/cometbft/cometbft/abci/example/kvstore"
	abci "github.com/cometbft/cometbft/abci/types"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto"
	cstypes "github.com/cometbft/cometbft/internal/consensus/types"
	cmtos "github.com/cometbft/cometbft/internal/os"
	"github.com/cometbft/cometbft/internal/test"
	cmtbytes "github.com/cometbft/cometbft/libs/bytes"
	"github.com/cometbft/cometbft/libs/log"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	mempl "github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/store"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

const (
	testSubscriber = "test-client"
)

// A cleanupFunc cleans up any config / test files created for a particular
// test.
type cleanupFunc func()

// genesis, chain_id, priv_val.
var (
	config                *cfg.Config // NOTE: must be reset for each _test.go file
	consensusReplayConfig *cfg.Config
	ensureTimeout         = time.Millisecond * 200
)

func ensureDir(dir string) {
	if err := cmtos.EnsureDir(dir, 0o700); err != nil {
		panic(err)
	}
}

func ResetConfig(name string) *cfg.Config {
	return test.ResetTestRoot(name)
}

// -------------------------------------------------------------------------------
// validator stub (a kvstore consensus peer we control)

type validatorStub struct {
	Index  int32 // Validator index. NOTE: we don't assume validator set changes.
	Height int64
	Round  int32
	clock  cmttime.Source
	types.PrivValidator
	VotingPower int64
	lastVote    *types.Vote
}

var testMinPower int64 = 10

func newValidatorStub(privValidator types.PrivValidator, valIndex int32) *validatorStub {
	return &validatorStub{
		Index:         valIndex,
		PrivValidator: privValidator,
		VotingPower:   testMinPower,
		clock:         cmttime.DefaultSource{},
	}
}

func signProposal(t *testing.T, proposal *types.Proposal, chainID string, vss *validatorStub) {
	t.Helper()
	p := proposal.ToProto()
	err := vss.SignProposal(chainID, p)
	require.NoError(t, err)
	proposal.Signature = p.Signature
}

func (vs *validatorStub) signVote(
	voteType types.SignedMsgType,
	chainID string,
	blockID types.BlockID,
	voteExtension []byte,
	extEnabled bool,
	timestamp time.Time,
) (*types.Vote, error) {
	pubKey, err := vs.PrivValidator.GetPubKey()
	if err != nil {
		return nil, fmt.Errorf("can't get pubkey: %w", err)
	}
	vote := &types.Vote{
		Type:             voteType,
		Height:           vs.Height,
		Round:            vs.Round,
		BlockID:          blockID,
		Timestamp:        timestamp,
		ValidatorAddress: pubKey.Address(),
		ValidatorIndex:   vs.Index,
		Extension:        voteExtension,
	}
	v := vote.ToProto()
	if err = vs.PrivValidator.SignVote(chainID, v, true); err != nil {
		return nil, fmt.Errorf("sign vote failed: %w", err)
	}

	// ref: signVote in FilePV, the vote should use the previous vote info when the sign data is the same.
	if signDataIsEqual(vs.lastVote, v) {
		v.Signature = vs.lastVote.Signature
		v.Timestamp = vs.lastVote.Timestamp
		v.ExtensionSignature = vs.lastVote.ExtensionSignature
	}

	vote.Signature = v.Signature
	vote.Timestamp = v.Timestamp
	vote.ExtensionSignature = v.ExtensionSignature

	if !extEnabled {
		vote.ExtensionSignature = nil
	}

	return vote, err
}

// Sign vote for type/hash/header.
func signVoteWithTimestamp(vs *validatorStub, voteType types.SignedMsgType, chainID string,
	blockID types.BlockID, extEnabled bool, timestamp time.Time,
) *types.Vote {
	var ext []byte
	// Only non-nil precommits are allowed to carry vote extensions.
	if extEnabled {
		if voteType != types.PrecommitType {
			panic(errors.New("vote type is not precommit but extensions enabled"))
		}
		if len(blockID.Hash) != 0 || !blockID.PartSetHeader.IsZero() {
			ext = []byte("extension")
		}
	}
	v, err := vs.signVote(voteType, chainID, blockID, ext, extEnabled, timestamp)
	if err != nil {
		panic(fmt.Errorf("failed to sign vote: %v", err))
	}

	vs.lastVote = v

	return v
}

func signVote(vs *validatorStub, voteType types.SignedMsgType, chainID string, blockID types.BlockID, extEnabled bool) *types.Vote {
	return signVoteWithTimestamp(vs, voteType, chainID, blockID, extEnabled, vs.clock.Now())
}

func signVotes(
	voteType types.SignedMsgType,
	chainID string,
	blockID types.BlockID,
	extEnabled bool,
	vss ...*validatorStub,
) []*types.Vote {
	votes := make([]*types.Vote, len(vss))
	for i, vs := range vss {
		votes[i] = signVote(vs, voteType, chainID, blockID, extEnabled)
	}
	return votes
}

func incrementHeight(vss ...*validatorStub) {
	for _, vs := range vss {
		vs.Height++
		vs.Round = 0
	}
}

func incrementRound(vss ...*validatorStub) {
	for _, vs := range vss {
		vs.Round++
	}
}

type ValidatorStubsByPower []*validatorStub

func (vss ValidatorStubsByPower) Len() int {
	return len(vss)
}

func (vss ValidatorStubsByPower) Less(i, j int) bool {
	vssi, err := vss[i].GetPubKey()
	if err != nil {
		panic(err)
	}
	vssj, err := vss[j].GetPubKey()
	if err != nil {
		panic(err)
	}

	if vss[i].VotingPower == vss[j].VotingPower {
		return bytes.Compare(vssi.Address(), vssj.Address()) == -1
	}
	return vss[i].VotingPower > vss[j].VotingPower
}

func (vss ValidatorStubsByPower) Swap(i, j int) {
	it := vss[i]
	vss[i] = vss[j]
	vss[i].Index = int32(i)
	vss[j] = it
	vss[j].Index = int32(j)
}

// -------------------------------------------------------------------------------
// Functions for transitioning the consensus state

func startTestRound(cs *State, height int64, round int32) {
	cs.enterNewRound(height, round)
	cs.startRoutines(0)
}

func createProposalBlockWithTime(t *testing.T, cs *State, time time.Time) (*types.Block, *types.PartSet, types.BlockID) {
	t.Helper()
	block, err := cs.createProposalBlock(context.Background())
	if !time.IsZero() {
		block.Time = cmttime.Canonical(time)
	}
	assert.NoError(t, err)
	blockParts, err := block.MakePartSet(types.BlockPartSizeBytes)
	assert.NoError(t, err)
	blockID := types.BlockID{Hash: block.Hash(), PartSetHeader: blockParts.Header()}
	return block, blockParts, blockID
}

func createProposalBlock(t *testing.T, cs *State) (*types.Block, *types.PartSet, types.BlockID) {
	t.Helper()
	return createProposalBlockWithTime(t, cs, time.Time{})
}

// Create proposal block from cs1 but sign it with vs.
func decideProposal(
	_ context.Context,
	t *testing.T,
	cs1 *State,
	vs *validatorStub,
	height int64,
	round int32,
) (*types.Proposal, *types.Block) {
	t.Helper()

	cs1.mtx.Lock()
	block, _, propBlockID := createProposalBlock(t, cs1)

	validRound := cs1.ValidRound
	chainID := cs1.state.ChainID
	cs1.mtx.Unlock()
	if block == nil {
		panic("Failed to createProposalBlock. Did you forget to add commit for previous block?")
	}

	// Make proposal
	proposal := types.NewProposal(height, round, validRound, propBlockID, block.Header.Time)
	p := proposal.ToProto()
	if err := vs.SignProposal(chainID, p); err != nil {
		panic(err)
	}

	proposal.Signature = p.Signature

	return proposal, block
}

func addVotes(to *State, votes ...*types.Vote) {
	for _, vote := range votes {
		to.peerMsgQueue <- msgInfo{Msg: &VoteMessage{vote}}
	}
}

func signAddVotes(
	to *State,
	voteType types.SignedMsgType,
	chainID string,
	blockID types.BlockID,
	extEnabled bool,
	vss ...*validatorStub,
) {
	votes := signVotes(voteType, chainID, blockID, extEnabled, vss...)
	addVotes(to, votes...)
}

func validatePrevote(t *testing.T, cs *State, round int32, privVal *validatorStub, blockHash []byte) {
	t.Helper()
	prevotes := cs.Votes.Prevotes(round)
	pubKey, err := privVal.GetPubKey()
	require.NoError(t, err)
	address := pubKey.Address()
	var vote *types.Vote
	if vote = prevotes.GetByAddress(address); vote == nil {
		panic("Failed to find prevote from validator")
	}
	if blockHash == nil {
		if vote.BlockID.Hash != nil {
			panic(fmt.Sprintf("Expected prevote to be for nil, got %X", vote.BlockID.Hash))
		}
	} else {
		if vote.BlockID.Hash == nil {
			panic(fmt.Sprintf("Expected prevote to be for %X, got <nil>", blockHash))
		} else if !bytes.Equal(vote.BlockID.Hash, blockHash) {
			panic(fmt.Sprintf("Expected prevote to be for %X, got %X", blockHash, vote.BlockID.Hash))
		}
	}
}

func validateLastPrecommit(t *testing.T, cs *State, privVal *validatorStub, blockHash []byte) {
	t.Helper()
	votes := cs.LastCommit
	pv, err := privVal.GetPubKey()
	require.NoError(t, err)
	address := pv.Address()
	var vote *types.Vote
	if vote = votes.GetByAddress(address); vote == nil {
		panic("Failed to find precommit from validator")
	}
	if !bytes.Equal(vote.BlockID.Hash, blockHash) {
		panic(fmt.Sprintf("Expected precommit to be for %X, got %X", blockHash, vote.BlockID.Hash))
	}
}

func validatePrecommit(
	t *testing.T,
	cs *State,
	thisRound,
	lockRound int32,
	privVal *validatorStub,
	votedBlockHash,
	lockedBlockHash []byte,
) {
	t.Helper()
	precommits := cs.Votes.Precommits(thisRound)
	pv, err := privVal.GetPubKey()
	require.NoError(t, err)
	address := pv.Address()
	var vote *types.Vote
	if vote = precommits.GetByAddress(address); vote == nil {
		panic("Failed to find precommit from validator")
	}

	if votedBlockHash == nil {
		if vote.BlockID.Hash != nil {
			panic("Expected precommit to be for nil")
		}
	} else {
		if !bytes.Equal(vote.BlockID.Hash, votedBlockHash) {
			panic("Expected precommit to be for proposal block")
		}
	}

	rs := cs.GetRoundState()
	if lockedBlockHash == nil {
		if rs.LockedRound != lockRound || rs.LockedBlock != nil {
			panic(fmt.Sprintf(
				"Expected to be locked on nil at round %d. Got locked at round %d with block %v",
				lockRound,
				rs.LockedRound,
				rs.LockedBlock))
		}
	} else {
		if rs.LockedRound != lockRound || !bytes.Equal(rs.LockedBlock.Hash(), lockedBlockHash) {
			panic(fmt.Sprintf(
				"Expected block to be locked on round %d, got %d. Got locked block %X, expected %X",
				lockRound,
				rs.LockedRound,
				rs.LockedBlock.Hash(),
				lockedBlockHash))
		}
	}
}

func subscribeToVoter(cs *State, addr []byte) <-chan cmtpubsub.Message {
	votesSub, err := cs.eventBus.SubscribeUnbuffered(context.Background(), testSubscriber, types.EventQueryVote)
	if err != nil {
		panic(fmt.Sprintf("failed to subscribe %s to %v", testSubscriber, types.EventQueryVote))
	}
	ch := make(chan cmtpubsub.Message)
	go func() {
		for msg := range votesSub.Out() {
			vote := msg.Data().(types.EventDataVote)
			// we only fire for our own votes
			if bytes.Equal(addr, vote.Vote.ValidatorAddress) {
				ch <- msg
			}
		}
	}()
	return ch
}

func subscribeToVoterBuffered(cs *State, addr []byte) <-chan cmtpubsub.Message {
	votesSub, err := cs.eventBus.Subscribe(context.Background(), testSubscriber, types.EventQueryVote, 10)
	if err != nil {
		panic(fmt.Sprintf("failed to subscribe %s to %v with outcapacity 10", testSubscriber, types.EventQueryVote))
	}
	ch := make(chan cmtpubsub.Message, 10)
	go func() {
		for msg := range votesSub.Out() {
			vote := msg.Data().(types.EventDataVote)
			// we only fire for our own votes
			if bytes.Equal(addr, vote.Vote.ValidatorAddress) {
				ch <- msg
			}
		}
	}()
	return ch
}

// -------------------------------------------------------------------------------
// consensus states

func newState(state sm.State, pv types.PrivValidator, app abci.Application) *State {
	config := test.ResetTestRoot("consensus_state_test")
	return newStateWithConfig(config, state, pv, app)
}

func newStateWithConfig(
	thisConfig *cfg.Config,
	state sm.State,
	pv types.PrivValidator,
	app abci.Application,
) *State {
	blockDB := dbm.NewMemDB()
	return newStateWithConfigAndBlockStore(thisConfig, state, pv, app, blockDB)
}

func newStateWithConfigAndBlockStore(
	thisConfig *cfg.Config,
	state sm.State,
	pv types.PrivValidator,
	app abci.Application,
	blockDB dbm.DB,
) *State {
	// Get BlockStore
	blockStore := store.NewBlockStore(blockDB)

	// one for mempool, one for consensus
	mtx := new(cmtsync.Mutex)

	proxyAppConnCon := proxy.NewAppConnConsensus(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())
	proxyAppConnMem := proxy.NewAppConnMempool(abcicli.NewLocalClient(mtx, app), proxy.NopMetrics())
	// Make Mempool
	memplMetrics := mempl.NopMetrics()

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

	evpool := sm.EmptyEvidencePool{}

	// Make State
	stateDB := blockDB
	stateStore := sm.NewStore(stateDB, sm.StoreOptions{
		DiscardABCIResponses: false,
	})

	if err := stateStore.Save(state); err != nil { // for save height 1's validators info
		panic(err)
	}

	blockExec := sm.NewBlockExecutor(stateStore, log.TestingLogger(), proxyAppConnCon, mempool, evpool, blockStore)
	cs := NewState(thisConfig.Consensus, state, blockExec, blockStore, mempool, evpool)
	cs.SetLogger(log.TestingLogger().With("module", "consensus"))
	cs.SetPrivValidator(pv)

	eventBus := types.NewEventBus()
	eventBus.SetLogger(log.TestingLogger().With("module", "events"))
	err := eventBus.Start()
	if err != nil {
		panic(err)
	}
	cs.SetEventBus(eventBus)
	return cs
}

func loadPrivValidator(config *cfg.Config) (*privval.FilePV, error) {
	privValidatorKeyFile := config.PrivValidatorKeyFile()
	ensureDir(filepath.Dir(privValidatorKeyFile))
	privValidatorStateFile := config.PrivValidatorStateFile()
	privValidator, err := privval.LoadOrGenFilePV(privValidatorKeyFile, privValidatorStateFile, nil)
	if err != nil {
		return nil, err
	}
	privValidator.Reset()
	return privValidator, nil
}

func randState(nValidators int) (*State, []*validatorStub) {
	return randStateWithApp(nValidators, kvstore.NewInMemoryApplication())
}

func randStateWithAppWithHeight(
	nValidators int,
	app abci.Application,
	height int64,
) (*State, []*validatorStub) {
	c := test.ConsensusParams()
	c.Feature.VoteExtensionsEnableHeight = height
	return randStateWithAppImpl(nValidators, app, c)
}

func randStateWithApp(nValidators int, app abci.Application) (*State, []*validatorStub) {
	c := test.ConsensusParams()
	return randStateWithAppImpl(nValidators, app, c)
}

func randStateWithAppImpl(
	nValidators int,
	app abci.Application,
	consensusParams *types.ConsensusParams,
) (*State, []*validatorStub) {
	return randStateWithAppImplGenesisTime(nValidators, app, consensusParams, cmttime.Now())
}

func randStateWithAppImplGenesisTime(
	nValidators int,
	app abci.Application,
	consensusParams *types.ConsensusParams,
	genesisTime time.Time,
) (*State, []*validatorStub) {
	// Get State
	state, privVals := randGenesisStateWithTime(nValidators, consensusParams, genesisTime)

	vss := make([]*validatorStub, nValidators)

	cs := newState(state, privVals[0], app)

	for i := 0; i < nValidators; i++ {
		vss[i] = newValidatorStub(privVals[i], int32(i))
	}
	// since cs1 starts at 1
	incrementHeight(vss[1:]...)

	return cs, vss
}

// -------------------------------------------------------------------------------

func ensureNoNewEvent(ch <-chan cmtpubsub.Message, timeout time.Duration,
	errorMessage string,
) {
	select {
	case <-time.After(timeout):
	case <-ch:
		panic(errorMessage)
	}
}

func ensureNoNewEventOnChannel(ch <-chan cmtpubsub.Message) {
	ensureNoNewEvent(
		ch,
		ensureTimeout*8/10, // 20% leniency for goroutine scheduling uncertainty
		"We should be stuck waiting, not receiving new event on the channel")
}

func ensureNoNewRoundStep(stepCh <-chan cmtpubsub.Message) {
	ensureNoNewEvent(
		stepCh,
		ensureTimeout,
		"We should be stuck waiting, not receiving NewRoundStep event")
}

func ensureNoNewTimeout(stepCh <-chan cmtpubsub.Message, timeout int64) {
	timeoutDuration := time.Duration(timeout*10) * time.Nanosecond
	ensureNoNewEvent(
		stepCh,
		timeoutDuration,
		"We should be stuck waiting, not receiving NewTimeout event")
}

func ensureNewEvent(ch <-chan cmtpubsub.Message, height int64, round int32, timeout time.Duration, errorMessage string) {
	select {
	case <-time.After(timeout):
		panic(errorMessage)
	case msg := <-ch:
		roundStateEvent, ok := msg.Data().(types.EventDataRoundState)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataRoundState, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		if roundStateEvent.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, roundStateEvent.Height))
		}
		if roundStateEvent.Round != round {
			panic(fmt.Sprintf("expected round %v, got %v", round, roundStateEvent.Round))
		}
		// TODO: We could check also for a step at this point!
	}
}

func ensureNewRound(roundCh <-chan cmtpubsub.Message, height int64, round int32) {
	select {
	case <-time.After(ensureTimeout):
		panic("Timeout expired while waiting for NewRound event")
	case msg := <-roundCh:
		newRoundEvent, ok := msg.Data().(types.EventDataNewRound)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataNewRound, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		if newRoundEvent.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, newRoundEvent.Height))
		}
		if newRoundEvent.Round != round {
			panic(fmt.Sprintf("expected round %v, got %v", round, newRoundEvent.Round))
		}
	}
}

func ensureNewTimeout(timeoutCh <-chan cmtpubsub.Message, height int64, round int32, timeout int64) {
	timeoutDuration := time.Duration(timeout*10) * time.Nanosecond
	ensureNewEvent(timeoutCh, height, round, timeoutDuration,
		"Timeout expired while waiting for NewTimeout event")
}

func ensureNewValidBlock(validBlockCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureNewEvent(validBlockCh, height, round, ensureTimeout,
		"Timeout expired while waiting for NewValidBlock event")
}

func ensureNewBlock(blockCh <-chan cmtpubsub.Message, height int64) {
	select {
	case <-time.After(ensureTimeout):
		panic("Timeout expired while waiting for NewBlock event")
	case msg := <-blockCh:
		blockEvent, ok := msg.Data().(types.EventDataNewBlock)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataNewBlock, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		if blockEvent.Block.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, blockEvent.Block.Height))
		}
	}
}

func ensureNewBlockHeader(blockCh <-chan cmtpubsub.Message, height int64, blockHash cmtbytes.HexBytes) {
	select {
	case <-time.After(ensureTimeout):
		panic("Timeout expired while waiting for NewBlockHeader event")
	case msg := <-blockCh:
		blockHeaderEvent, ok := msg.Data().(types.EventDataNewBlockHeader)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataNewBlockHeader, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		if blockHeaderEvent.Header.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, blockHeaderEvent.Header.Height))
		}
		if !bytes.Equal(blockHeaderEvent.Header.Hash(), blockHash) {
			panic(fmt.Sprintf("expected header %X, got %X", blockHash, blockHeaderEvent.Header.Hash()))
		}
	}
}

func ensureLock(lockCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureNewEvent(lockCh, height, round, ensureTimeout,
		"Timeout expired while waiting for LockValue event")
}

func ensureRelock(relockCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureNewEvent(relockCh, height, round, ensureTimeout,
		"Timeout expired while waiting for RelockValue event")
}

func ensureProposalWithTimeout(proposalCh <-chan cmtpubsub.Message, height int64, round int32, propID *types.BlockID, timeout time.Duration) {
	select {
	case <-time.After(timeout):
		panic("Timeout expired while waiting for NewProposal event")
	case msg := <-proposalCh:
		proposalEvent, ok := msg.Data().(types.EventDataCompleteProposal)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataCompleteProposal, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		if proposalEvent.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, proposalEvent.Height))
		}
		if proposalEvent.Round != round {
			panic(fmt.Sprintf("expected round %v, got %v", round, proposalEvent.Round))
		}
		if propID != nil {
			if !proposalEvent.BlockID.Equals(*propID) {
				panic(fmt.Sprintf("Proposed block does not match expected block (%v != %v)", proposalEvent.BlockID, *propID))
			}
		}
	}
}

func ensureProposal(proposalCh <-chan cmtpubsub.Message, height int64, round int32, propID types.BlockID) {
	ensureProposalWithTimeout(proposalCh, height, round, &propID, ensureTimeout)
}

// For the propose, as we do not know the blockID in advance.
func ensureNewProposal(proposalCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureProposalWithTimeout(proposalCh, height, round, nil, ensureTimeout)
}

func ensurePrecommit(voteCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureVote(voteCh, height, round, types.PrecommitType)
}

func ensurePrevote(voteCh <-chan cmtpubsub.Message, height int64, round int32) {
	ensureVote(voteCh, height, round, types.PrevoteType)
}

func ensureVote(voteCh <-chan cmtpubsub.Message, height int64, round int32,
	voteType types.SignedMsgType,
) {
	select {
	case <-time.After(ensureTimeout):
		panic("Timeout expired while waiting for NewVote event")
	case msg := <-voteCh:
		voteEvent, ok := msg.Data().(types.EventDataVote)
		if !ok {
			panic(fmt.Sprintf("expected a EventDataVote, got %T. Wrong subscription channel?",
				msg.Data()))
		}
		vote := voteEvent.Vote
		if vote.Height != height {
			panic(fmt.Sprintf("expected height %v, got %v", height, vote.Height))
		}
		if vote.Round != round {
			panic(fmt.Sprintf("expected round %v, got %v", round, vote.Round))
		}
		if vote.Type != voteType {
			panic(fmt.Sprintf("expected type %v, got %v", voteType, vote.Type))
		}
	}
}

func ensurePrevoteMatch(t *testing.T, voteCh <-chan cmtpubsub.Message, height int64, round int32, hash []byte) {
	t.Helper()
	ensureVoteMatch(t, voteCh, height, round, hash, types.PrevoteType)
}

func ensurePrecommitMatch(t *testing.T, voteCh <-chan cmtpubsub.Message, height int64, round int32, hash []byte) {
	t.Helper()
	ensureVoteMatch(t, voteCh, height, round, hash, types.PrecommitType)
}

func ensureVoteMatch(t *testing.T, voteCh <-chan cmtpubsub.Message, height int64, round int32, hash []byte, voteType types.SignedMsgType) {
	t.Helper()
	select {
	case <-time.After(ensureTimeout):
		t.Fatal("Timeout expired while waiting for NewVote event")
	case msg := <-voteCh:
		voteEvent, ok := msg.Data().(types.EventDataVote)
		require.True(t, ok, "expected a EventDataVote, got %T. Wrong subscription channel?",
			msg.Data())

		vote := voteEvent.Vote
		assert.Equal(t, height, vote.Height, "expected height %d, but got %d", height, vote.Height)
		assert.Equal(t, round, vote.Round, "expected round %d, but got %d", round, vote.Round)
		assert.Equal(t, voteType, vote.Type, "expected type %s, but got %s", voteType, vote.Type)
		if hash == nil {
			require.Nil(t, vote.BlockID.Hash, "Expected prevote to be for nil, got %X", vote.BlockID.Hash)
		} else {
			require.True(t, bytes.Equal(vote.BlockID.Hash, hash), "Expected prevote to be for %X, got %X", hash, vote.BlockID.Hash)
		}
	}
}

func ensureNewEventOnChannel(ch <-chan cmtpubsub.Message) {
	select {
	case <-time.After(ensureTimeout * 12 / 10): // 20% leniency for goroutine scheduling uncertainty
		panic("Timeout expired while waiting for new activity on the channel")
	case <-ch:
	}
}

// -------------------------------------------------------------------------------
// consensus nets

// consensusLogger is a TestingLogger which uses a different
// color for each validator ("validator" key must exist).
func consensusLogger() log.Logger {
	return log.TestingLoggerWithColorFn(func(keyvals ...any) term.FgBgColor {
		for i := 0; i < len(keyvals)-1; i += 2 {
			if keyvals[i] == "validator" {
				return term.FgBgColor{Fg: term.Color(uint8(keyvals[i+1].(int) + 1))}
			}
		}
		return term.FgBgColor{}
	}).With("module", "consensus")
}

func randConsensusNet(t *testing.T, nValidators int, testName string, tickerFunc func() TimeoutTicker,
	appFunc func() abci.Application, configOpts ...func(*cfg.Config),
) ([]*State, cleanupFunc) {
	t.Helper()
	genDoc, privVals := randGenesisDoc(nValidators, 30, nil, cmttime.Now())
	css := make([]*State, nValidators)
	logger := consensusLogger()
	configRootDirs := make([]string, 0, nValidators)
	for i := 0; i < nValidators; i++ {
		stateDB := dbm.NewMemDB() // each state needs its own db
		stateStore := sm.NewStore(stateDB, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		state, _ := stateStore.LoadFromDBOrGenesisDoc(genDoc)
		thisConfig := ResetConfig(fmt.Sprintf("%s_%d", testName, i))
		configRootDirs = append(configRootDirs, thisConfig.RootDir)
		for _, opt := range configOpts {
			opt(thisConfig)
		}
		ensureDir(filepath.Dir(thisConfig.Consensus.WalFile())) // dir for wal
		app := appFunc()
		vals := types.TM2PB.ValidatorUpdates(state.Validators)
		_, err := app.InitChain(context.Background(), &abci.InitChainRequest{Validators: vals})
		require.NoError(t, err)

		css[i] = newStateWithConfigAndBlockStore(thisConfig, state, privVals[i], app, stateDB)
		css[i].SetTimeoutTicker(tickerFunc())
		css[i].SetLogger(logger.With("validator", i, "module", "consensus"))
	}
	return css, func() {
		for _, dir := range configRootDirs {
			os.RemoveAll(dir)
		}
	}
}

// nPeers = nValidators + nNotValidator.
func randConsensusNetWithPeers(
	t *testing.T,
	nValidators,
	nPeers int,
	testName string,
	tickerFunc func() TimeoutTicker,
	appFunc func(string) abci.Application,
) ([]*State, *types.GenesisDoc, *cfg.Config, cleanupFunc) {
	t.Helper()
	c := test.ConsensusParams()
	genDoc, privVals := randGenesisDoc(nValidators, testMinPower, c, cmttime.Now())
	css := make([]*State, nPeers)
	logger := consensusLogger()
	var peer0Config *cfg.Config
	configRootDirs := make([]string, 0, nPeers)
	for i := 0; i < nPeers; i++ {
		stateDB := dbm.NewMemDB() // each state needs its own db
		stateStore := sm.NewStore(stateDB, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		t.Cleanup(func() { _ = stateStore.Close() })
		state, err := stateStore.LoadFromDBOrGenesisDoc(genDoc)
		require.NoError(t, err)
		thisConfig := ResetConfig(fmt.Sprintf("%s_%d", testName, i))
		configRootDirs = append(configRootDirs, thisConfig.RootDir)
		ensureDir(filepath.Dir(thisConfig.Consensus.WalFile())) // dir for wal
		if i == 0 {
			peer0Config = thisConfig
		}
		var privVal types.PrivValidator
		if i < nValidators {
			privVal = privVals[i]
		} else {
			tempKeyFile, err := os.CreateTemp("", "priv_validator_key_")
			require.NoError(t, err)
			tempStateFile, err := os.CreateTemp("", "priv_validator_state_")
			require.NoError(t, err)
			privVal, err = privval.GenFilePV(tempKeyFile.Name(), tempStateFile.Name(), nil)
			require.NoError(t, err)
		}

		app := appFunc(path.Join(config.DBDir(), fmt.Sprintf("%s_%d", testName, i)))
		vals := types.TM2PB.ValidatorUpdates(state.Validators)
		if _, ok := app.(*kvstore.Application); ok {
			// simulate handshake, receive app version. If don't do this, replay test will fail
			state.Version.Consensus.App = kvstore.AppVersion
		}
		_, err = app.InitChain(context.Background(), &abci.InitChainRequest{Validators: vals})
		require.NoError(t, err)

		css[i] = newStateWithConfig(thisConfig, state, privVal, app)
		css[i].SetTimeoutTicker(tickerFunc())
		css[i].SetLogger(logger.With("validator", i, "module", "consensus"))
	}
	return css, genDoc, peer0Config, func() {
		for _, dir := range configRootDirs {
			os.RemoveAll(dir)
		}
	}
}

func getSwitchIndex(switches []*p2p.Switch, peer p2p.Peer) int {
	for i, s := range switches {
		if peer.NodeInfo().ID() == s.NodeInfo().ID() {
			return i
		}
	}
	panic("didn't find peer in switches")
}

// -------------------------------------------------------------------------------
// genesis

func randGenesisDoc(numValidators int,
	minPower int64,
	consensusParams *types.ConsensusParams,
	genesisTime time.Time,
) (*types.GenesisDoc, []types.PrivValidator) {
	randPower := false
	validators := make([]types.GenesisValidator, numValidators)
	privValidators := make([]types.PrivValidator, numValidators)
	for i := 0; i < numValidators; i++ {
		val, privVal := types.RandValidator(randPower, minPower)
		validators[i] = types.GenesisValidator{
			PubKey: val.PubKey,
			Power:  val.VotingPower,
		}
		privValidators[i] = privVal
	}
	sort.Sort(types.PrivValidatorsByAddress(privValidators))

	if consensusParams == nil {
		consensusParams = test.ConsensusParams()
	}

	return &types.GenesisDoc{
		GenesisTime:     genesisTime,
		InitialHeight:   1,
		ChainID:         test.DefaultTestChainID,
		Validators:      validators,
		ConsensusParams: consensusParams,
	}, privValidators
}

func randGenesisState(
	numValidators int, //nolint: unparam
	consensusParams *types.ConsensusParams, //nolint: unparam
) (sm.State, []types.PrivValidator) {
	if consensusParams == nil {
		consensusParams = test.ConsensusParams()
	}
	return randGenesisStateWithTime(numValidators, consensusParams, cmttime.Now())
}

func randGenesisStateWithTime(
	numValidators int,
	consensusParams *types.ConsensusParams,
	genesisTime time.Time,
) (sm.State, []types.PrivValidator) {
	minPower := int64(10)
	genDoc, privValidators := randGenesisDoc(numValidators, minPower, consensusParams, genesisTime)
	s0, _ := sm.MakeGenesisState(genDoc)
	return s0, privValidators
}

// ------------------------------------
// mock ticker

func newMockTickerFunc(onlyOnce bool) func() TimeoutTicker {
	return func() TimeoutTicker {
		return &mockTicker{
			c:        make(chan timeoutInfo, 100),
			onlyOnce: onlyOnce,
		}
	}
}

// mock ticker only fires on RoundStepNewHeight
// and only once if onlyOnce=true.
type mockTicker struct {
	c chan timeoutInfo

	mtx      sync.Mutex
	onlyOnce bool
	fired    bool
}

func (*mockTicker) Start() error {
	return nil
}

func (*mockTicker) Stop() error {
	return nil
}

func (m *mockTicker) ScheduleTimeout(ti timeoutInfo) {
	m.mtx.Lock()
	defer m.mtx.Unlock()
	if m.onlyOnce && m.fired {
		return
	}
	if ti.Step == cstypes.RoundStepNewHeight {
		m.c <- ti
		m.fired = true
	}
}

func (m *mockTicker) Chan() <-chan timeoutInfo {
	return m.c
}

func (*mockTicker) SetLogger(log.Logger) {}

func newPersistentKVStore() abci.Application {
	dir, err := os.MkdirTemp("", "persistent-kvstore")
	if err != nil {
		panic(err)
	}
	return kvstore.NewPersistentApplication(dir)
}

func newKVStore() abci.Application {
	return kvstore.NewInMemoryApplication()
}

func newPersistentKVStoreWithPath(dbDir string) abci.Application {
	return kvstore.NewPersistentApplication(dbDir)
}

func signDataIsEqual(v1 *types.Vote, v2 *cmtproto.Vote) bool {
	if v1 == nil || v2 == nil {
		return false
	}

	return v1.Type == v2.Type &&
		bytes.Equal(v1.BlockID.Hash, v2.BlockID.GetHash()) &&
		v1.Height == v2.GetHeight() &&
		v1.Round == v2.Round &&
		bytes.Equal(v1.ValidatorAddress.Bytes(), v2.GetValidatorAddress()) &&
		v1.ValidatorIndex == v2.GetValidatorIndex() &&
		bytes.Equal(v1.Extension, v2.Extension)
}

func updateValTx(pubKey crypto.PubKey, power int64) []byte {
	return kvstore.MakeValSetChangeTx(abci.NewValidatorUpdate(pubKey, power))
}
