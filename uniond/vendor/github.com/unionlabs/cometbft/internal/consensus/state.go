package consensus

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"runtime/debug"
	"sort"
	"strconv"
	"time"

	"github.com/cosmos/gogoproto/proto"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cfg "github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto"
	cstypes "github.com/cometbft/cometbft/internal/consensus/types"
	cmtevents "github.com/cometbft/cometbft/internal/events"
	"github.com/cometbft/cometbft/internal/fail"
	cmtos "github.com/cometbft/cometbft/internal/os"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	cmterrors "github.com/cometbft/cometbft/types/errors"
	cmttime "github.com/cometbft/cometbft/types/time"
)

var msgQueueSize = 1000

// msgs from the reactor which may update the state.
type msgInfo struct {
	Msg         Message   `json:"msg"`
	PeerID      p2p.ID    `json:"peer_key"`
	ReceiveTime time.Time `json:"receive_time"`
}

// internally generated messages which may update the state.
type timeoutInfo struct {
	Duration time.Duration         `json:"duration"`
	Height   int64                 `json:"height"`
	Round    int32                 `json:"round"`
	Step     cstypes.RoundStepType `json:"step"`
}

func (ti *timeoutInfo) String() string {
	return fmt.Sprintf("%v ; %d/%d %v", ti.Duration, ti.Height, ti.Round, ti.Step)
}

// interface to the mempool.
type txNotifier interface {
	TxsAvailable() <-chan struct{}
}

// interface to the evidence pool.
type evidencePool interface {
	// reports conflicting votes to the evidence pool to be processed into evidence
	ReportConflictingVotes(voteA, voteB *types.Vote)
}

// State handles execution of the consensus algorithm.
// It processes votes and proposals, and upon reaching agreement,
// commits blocks to the chain and executes them against the application.
// The internal state machine receives input from peers, the internal validator, and from a timer.
type State struct {
	service.BaseService

	// config details
	config        *cfg.ConsensusConfig
	privValidator types.PrivValidator // for signing votes

	// store blocks and commits
	blockStore sm.BlockStore

	// create and execute blocks
	blockExec *sm.BlockExecutor

	// notify us if txs are available
	txNotifier txNotifier

	// add evidence to the pool
	// when it's detected
	evpool evidencePool

	// internal state
	mtx cmtsync.RWMutex
	cstypes.RoundState
	state sm.State // State until height-1.
	// privValidator pubkey, memoized for the duration of one block
	// to avoid extra requests to HSM
	privValidatorPubKey crypto.PubKey

	// state changes may be triggered by: msgs from peers,
	// msgs from ourself, or by timeouts
	peerMsgQueue     chan msgInfo
	internalMsgQueue chan msgInfo
	timeoutTicker    TimeoutTicker

	// information about about added votes and block parts are written on this channel
	// so statistics can be computed by reactor
	statsMsgQueue chan msgInfo

	// we use eventBus to trigger msg broadcasts in the reactor,
	// and to notify external subscribers, eg. through a websocket
	eventBus *types.EventBus

	// a Write-Ahead Log ensures we can recover from any kind of crash
	// and helps us avoid signing conflicting votes
	wal          WAL
	replayMode   bool // so we don't log signing errors during replay
	doWALCatchup bool // determines if we even try to do the catchup

	// for tests where we want to limit the number of transitions the state makes
	nSteps int

	// some functions can be overwritten for testing
	decideProposal func(height int64, round int32)
	doPrevote      func(height int64, round int32)
	setProposal    func(proposal *types.Proposal, t time.Time) error

	// closed when we finish shutting down
	done chan struct{}

	// synchronous pubsub between consensus state and reactor.
	// state only emits EventNewRoundStep and EventVote
	evsw cmtevents.EventSwitch

	// for reporting metrics
	metrics *Metrics

	// offline state sync height indicating to which height the node synced offline
	offlineStateSyncHeight int64

	// a buffer to store the concatenated proposal block parts (serialization format)
	// should only be accessed under the cs.mtx lock
	serializedBlockBuffer []byte
}

// StateOption sets an optional parameter on the State.
type StateOption func(*State)

// NewState returns a new State.
func NewState(
	config *cfg.ConsensusConfig,
	state sm.State,
	blockExec *sm.BlockExecutor,
	blockStore sm.BlockStore,
	txNotifier txNotifier,
	evpool evidencePool,
	options ...StateOption,
) *State {
	cs := &State{
		config:           config,
		blockExec:        blockExec,
		blockStore:       blockStore,
		txNotifier:       txNotifier,
		peerMsgQueue:     make(chan msgInfo, msgQueueSize),
		internalMsgQueue: make(chan msgInfo, msgQueueSize),
		timeoutTicker:    NewTimeoutTicker(),
		statsMsgQueue:    make(chan msgInfo, msgQueueSize),
		done:             make(chan struct{}),
		doWALCatchup:     true,
		wal:              nilWAL{},
		evpool:           evpool,
		evsw:             cmtevents.NewEventSwitch(),
		metrics:          NopMetrics(),
	}
	for _, option := range options {
		option(cs)
	}
	// set function defaults (may be overwritten before calling Start)
	cs.decideProposal = cs.defaultDecideProposal
	cs.doPrevote = cs.defaultDoPrevote
	cs.setProposal = cs.defaultSetProposal

	// We have no votes, so reconstruct LastCommit from SeenCommit.
	if state.LastBlockHeight > 0 {
		// In case of out of band performed statesync, the state store
		// will have a state but no extended commit (as no block has been downloaded).
		// If the height at which the vote extensions are enabled is lower
		// than the height at which we statesync, consensus will panic because
		// it will try to reconstruct the extended commit here.
		if cs.offlineStateSyncHeight != 0 {
			cs.reconstructSeenCommit(state)
		} else {
			cs.reconstructLastCommit(state)
		}
	}

	cs.updateToState(state)

	// NOTE: we do not call scheduleRound0 yet, we do that upon Start()

	cs.BaseService = *service.NewBaseService(nil, "State", cs)

	return cs
}

// SetLogger implements Service.
func (cs *State) SetLogger(l log.Logger) {
	cs.BaseService.Logger = l
	cs.timeoutTicker.SetLogger(l)
}

// SetEventBus sets event bus.
func (cs *State) SetEventBus(b *types.EventBus) {
	cs.eventBus = b
	cs.blockExec.SetEventBus(b)
}

// StateMetrics sets the metrics.
func StateMetrics(metrics *Metrics) StateOption {
	return func(cs *State) { cs.metrics = metrics }
}

// OfflineStateSyncHeight indicates the height at which the node
// statesync offline - before booting sets the metrics.
func OfflineStateSyncHeight(height int64) StateOption {
	return func(cs *State) { cs.offlineStateSyncHeight = height }
}

// String returns a string.
func (*State) String() string {
	// better not to access shared variables
	return "ConsensusState"
}

// GetState returns a copy of the chain state.
func (cs *State) GetState() sm.State {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cs.state.Copy()
}

// GetLastHeight returns the last height committed.
// If there were no blocks, returns 0.
func (cs *State) GetLastHeight() int64 {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cs.RoundState.Height - 1
}

// GetRoundState returns a shallow copy of the internal consensus state.
// This function is thread-safe.
func (cs *State) GetRoundState() *cstypes.RoundState {
	cs.mtx.RLock()
	rs := cs.getRoundState()
	cs.mtx.RUnlock()
	return rs
}

// getRoundState returns a shallow copy of the internal consensus state.
// This function is not thread-safe. Use GetRoundState for the thread-safe version.
func (cs *State) getRoundState() *cstypes.RoundState {
	rs := cs.RoundState // copy
	return &rs
}

// GetRoundStateJSON returns a json of RoundState.
func (cs *State) GetRoundStateJSON() ([]byte, error) {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cmtjson.Marshal(cs.RoundState)
}

// GetRoundStateSimpleJSON returns a json of RoundStateSimple.
func (cs *State) GetRoundStateSimpleJSON() ([]byte, error) {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cmtjson.Marshal(cs.RoundState.RoundStateSimple())
}

// GetValidators returns a copy of the current validators.
func (cs *State) GetValidators() (int64, []*types.Validator) {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cs.state.LastBlockHeight, cs.state.Validators.Copy().Validators
}

// SetPrivValidator sets the private validator account for signing votes. It
// immediately requests pubkey and caches it.
func (cs *State) SetPrivValidator(priv types.PrivValidator) {
	cs.mtx.Lock()
	defer cs.mtx.Unlock()

	cs.privValidator = priv

	if err := cs.updatePrivValidatorPubKey(); err != nil {
		cs.Logger.Error("Failed to get private validator pubkey", "err", err)
	}
}

// SetTimeoutTicker sets the local timer. It may be useful to overwrite for
// testing.
func (cs *State) SetTimeoutTicker(timeoutTicker TimeoutTicker) {
	cs.mtx.Lock()
	cs.timeoutTicker = timeoutTicker
	cs.mtx.Unlock()
}

// LoadCommit loads the commit for a given height.
func (cs *State) LoadCommit(height int64) *types.Commit {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()

	if height == cs.blockStore.Height() {
		return cs.blockStore.LoadSeenCommit(height)
	}

	return cs.blockStore.LoadBlockCommit(height)
}

// OnStart loads the latest state via the WAL, and starts the timeout and
// receive routines.
func (cs *State) OnStart() error {
	// We may set the WAL in testing before calling Start, so only OpenWAL if its
	// still the nilWAL.
	if _, ok := cs.wal.(nilWAL); ok {
		if err := cs.loadWalFile(); err != nil {
			return err
		}
	}

	// we need the timeoutRoutine for replay so
	// we don't block on the tick chan.
	// NOTE: we will get a build up of garbage go routines
	// firing on the tockChan until the receiveRoutine is started
	// to deal with them (by that point, at most one will be valid)
	if err := cs.timeoutTicker.Start(); err != nil {
		return err
	}

	// We may have lost some votes if the process crashed reload from consensus
	// log to catchup.
	if cs.doWALCatchup {
		repairAttempted := false

	LOOP:
		for {
			err := cs.catchupReplay(cs.Height)
			switch {
			case err == nil:
				break LOOP

			case !IsDataCorruptionError(err):
				cs.Logger.Error("Error on catchup replay; proceeding to start state anyway", "err", err)
				break LOOP

			case repairAttempted:
				return err
			}

			cs.Logger.Error("WAL file is corrupted; attempting repair", "err", err)

			// 1) prep work
			if err := cs.wal.Stop(); err != nil {
				return err
			}

			repairAttempted = true

			// 2) backup original WAL file
			corruptedFile := cs.config.WalFile() + ".CORRUPTED"
			if err := cmtos.CopyFile(cs.config.WalFile(), corruptedFile); err != nil {
				return err
			}

			cs.Logger.Debug("Backed up WAL file", "src", cs.config.WalFile(), "dst", corruptedFile)

			// 3) try to repair (WAL file will be overwritten!)
			if err := repairWalFile(corruptedFile, cs.config.WalFile()); err != nil {
				cs.Logger.Error("WAL repair failed", "err", err)
				return err
			}

			cs.Logger.Info("Successfully repaired WAL file")

			// reload WAL file
			if err := cs.loadWalFile(); err != nil {
				return err
			}
		}
	}

	if err := cs.evsw.Start(); err != nil {
		return err
	}

	// Double Signing Risk Reduction
	if err := cs.checkDoubleSigningRisk(cs.Height); err != nil {
		return err
	}

	// now start the receiveRoutine
	go cs.receiveRoutine(0)

	// schedule the first round!
	// use GetRoundState so we don't race the receiveRoutine for access
	cs.scheduleRound0(cs.GetRoundState())

	return nil
}

// timeoutRoutine: receive requests for timeouts on tickChan and fire timeouts on tockChan
// receiveRoutine: serializes processing of proposoals, block parts, votes; coordinates state transitions.
func (cs *State) startRoutines(maxSteps int) {
	err := cs.timeoutTicker.Start()
	if err != nil {
		cs.Logger.Error("Failed to start timeout ticker", "err", err)
		return
	}

	go cs.receiveRoutine(maxSteps)
}

// loadWalFile loads WAL data from file. It overwrites cs.wal.
func (cs *State) loadWalFile() error {
	wal, err := cs.OpenWAL(cs.config.WalFile())
	if err != nil {
		cs.Logger.Error("Failed to load state WAL", "err", err)
		return err
	}

	cs.wal = wal
	return nil
}

// OnStop implements service.Service.
func (cs *State) OnStop() {
	if err := cs.evsw.Stop(); err != nil {
		cs.Logger.Error("Failed trying to stop eventSwitch", "error", err)
	}

	if err := cs.timeoutTicker.Stop(); err != nil {
		cs.Logger.Error("Failed trying to stop timeoutTicket", "error", err)
	}
	// WAL is stopped in receiveRoutine.
}

// Wait waits for the the main routine to return.
// NOTE: be sure to Stop() the event switch and drain
// any event channels or this may deadlock.
func (cs *State) Wait() {
	<-cs.done
}

// OpenWAL opens a file to log all consensus messages and timeouts for
// deterministic accountability.
func (cs *State) OpenWAL(walFile string) (WAL, error) {
	wal, err := NewWAL(walFile)
	if err != nil {
		cs.Logger.Error("Failed to open WAL", "file", walFile, "err", err)
		return nil, err
	}

	wal.SetLogger(cs.Logger.With("wal", walFile))

	if err := wal.Start(); err != nil {
		cs.Logger.Error("Failed to start WAL", "err", err)
		return nil, err
	}

	return wal, nil
}

// ------------------------------------------------------------
// Public interface for passing messages into the consensus state, possibly causing a state transition.
// If peerID == "", the msg is considered internal.
// Messages are added to the appropriate queue (peer or internal).
// If the queue is full, the function may block.
// TODO: should these return anything or let callers just use events?

// AddVote inputs a vote.
func (cs *State) AddVote(vote *types.Vote, peerID p2p.ID) (added bool, err error) {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&VoteMessage{vote}, "", time.Time{}}
	} else {
		cs.peerMsgQueue <- msgInfo{&VoteMessage{vote}, peerID, time.Time{}}
	}

	// TODO: wait for event?!
	return false, nil
}

// SetProposal inputs a proposal.
func (cs *State) SetProposal(proposal *types.Proposal, peerID p2p.ID) error {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&ProposalMessage{proposal}, "", cmttime.Now()}
	} else {
		cs.peerMsgQueue <- msgInfo{&ProposalMessage{proposal}, peerID, cmttime.Now()}
	}

	// TODO: wait for event?!
	return nil
}

// AddProposalBlockPart inputs a part of the proposal block.
func (cs *State) AddProposalBlockPart(height int64, round int32, part *types.Part, peerID p2p.ID) error {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&BlockPartMessage{height, round, part}, "", time.Time{}}
	} else {
		cs.peerMsgQueue <- msgInfo{&BlockPartMessage{height, round, part}, peerID, time.Time{}}
	}

	// TODO: wait for event?!
	return nil
}

// SetProposalAndBlock inputs the proposal and all block parts.
func (cs *State) SetProposalAndBlock(
	proposal *types.Proposal,
	parts *types.PartSet,
	peerID p2p.ID,
) error {
	// TODO: Since the block parameter is not used, we should instead expose just a SetProposal method.
	if err := cs.SetProposal(proposal, peerID); err != nil {
		return err
	}

	for i := 0; i < int(parts.Total()); i++ {
		part := parts.GetPart(i)
		if err := cs.AddProposalBlockPart(proposal.Height, proposal.Round, part, peerID); err != nil {
			return err
		}
	}

	return nil
}

// ------------------------------------------------------------
// internal functions for managing the state

func (cs *State) updateHeight(height int64) {
	cs.metrics.Height.Set(float64(height))
	cs.Height = height
}

func (cs *State) updateRoundStep(round int32, step cstypes.RoundStepType) {
	if !cs.replayMode {
		if round != cs.Round || round == 0 && step == cstypes.RoundStepNewRound {
			cs.metrics.MarkRound(cs.Round, cs.StartTime)
		}
		if cs.Step != step {
			cs.metrics.MarkStep(cs.Step)
		}
	}
	cs.Round = round
	cs.Step = step
}

// enterNewRound(height, 0) at cs.StartTime.
func (cs *State) scheduleRound0(rs *cstypes.RoundState) {
	// cs.Logger.Info("scheduleRound0", "now", cmttime.Now(), "startTime", cs.StartTime)
	sleepDuration := rs.StartTime.Sub(cmttime.Now())
	cs.scheduleTimeout(sleepDuration, rs.Height, 0, cstypes.RoundStepNewHeight)
}

// Attempt to schedule a timeout (by sending timeoutInfo on the tickChan).
func (cs *State) scheduleTimeout(duration time.Duration, height int64, round int32, step cstypes.RoundStepType) {
	cs.timeoutTicker.ScheduleTimeout(timeoutInfo{duration, height, round, step})
}

// send a msg into the receiveRoutine regarding our own proposal, block part, or vote.
func (cs *State) sendInternalMessage(mi msgInfo) {
	select {
	case cs.internalMsgQueue <- mi:
	default:
		// NOTE: using the go-routine means our votes can
		// be processed out of order.
		// TODO: use CList here for strict determinism and
		// attempt push to internalMsgQueue in receiveRoutine
		cs.Logger.Debug("internal msg queue is full; using a go-routine")
		go func() { cs.internalMsgQueue <- mi }()
	}
}

// ReconstructSeenCommit reconstructs the seen commit
// This function is meant to be called after statesync
// that was performed offline as to avoid interfering with vote
// extensions.
func (cs *State) reconstructSeenCommit(state sm.State) {
	votes, err := cs.votesFromSeenCommit(state)
	if err != nil {
		panic(fmt.Sprintf("failed to reconstruct last commit; %s", err))
	}
	cs.LastCommit = votes
}

// Reconstruct the LastCommit from either SeenCommit or the ExtendedCommit. SeenCommit
// and ExtendedCommit are saved along with the block. If VoteExtensions are required
// the method will panic on an absent ExtendedCommit or an ExtendedCommit without
// extension data.
func (cs *State) reconstructLastCommit(state sm.State) {
	extensionsEnabled := state.ConsensusParams.Feature.VoteExtensionsEnabled(state.LastBlockHeight)
	if !extensionsEnabled {
		cs.reconstructSeenCommit(state)
		return
	}
	votes, err := cs.votesFromExtendedCommit(state)
	if err != nil {
		panic(fmt.Sprintf("failed to reconstruct last extended commit; %s", err))
	}
	cs.LastCommit = votes
}

func (cs *State) votesFromExtendedCommit(state sm.State) (*types.VoteSet, error) {
	ec := cs.blockStore.LoadBlockExtendedCommit(state.LastBlockHeight)
	if ec == nil {
		return nil, fmt.Errorf("extended commit for height %v not found", state.LastBlockHeight)
	}
	if ec.Height != state.LastBlockHeight {
		return nil, fmt.Errorf("heights don't match in votesFromExtendedCommit %v!=%v",
			ec.Height, state.LastBlockHeight)
	}
	vs := ec.ToExtendedVoteSet(state.ChainID, state.LastValidators)
	if !vs.HasTwoThirdsMajority() {
		return nil, ErrCommitQuorumNotMet
	}
	return vs, nil
}

func (cs *State) votesFromSeenCommit(state sm.State) (*types.VoteSet, error) {
	commit := cs.blockStore.LoadSeenCommit(state.LastBlockHeight)
	if commit == nil {
		commit = cs.blockStore.LoadBlockCommit(state.LastBlockHeight)
	}
	if commit == nil {
		return nil, fmt.Errorf("commit for height %v not found", state.LastBlockHeight)
	}
	if commit.Height != state.LastBlockHeight {
		return nil, fmt.Errorf("heights don't match in votesFromSeenCommit %v!=%v",
			commit.Height, state.LastBlockHeight)
	}
	vs := commit.ToVoteSet(state.ChainID, state.LastValidators)
	if !vs.HasTwoThirdsMajority() {
		return nil, ErrCommitQuorumNotMet
	}
	return vs, nil
}

// Updates State and increments height to match that of state.
// The round becomes 0 and cs.Step becomes cstypes.RoundStepNewHeight.
func (cs *State) updateToState(state sm.State) {
	if cs.CommitRound > -1 && 0 < cs.Height && cs.Height != state.LastBlockHeight {
		panic(fmt.Sprintf(
			"updateToState() expected state height of %v but found %v",
			cs.Height, state.LastBlockHeight,
		))
	}

	if !cs.state.IsEmpty() {
		if cs.state.LastBlockHeight > 0 && cs.state.LastBlockHeight+1 != cs.Height {
			// This might happen when someone else is mutating cs.state.
			// Someone forgot to pass in state.Copy() somewhere?!
			panic(fmt.Sprintf(
				"inconsistent cs.state.LastBlockHeight+1 %v vs cs.Height %v",
				cs.state.LastBlockHeight+1, cs.Height,
			))
		}
		if cs.state.LastBlockHeight > 0 && cs.Height == cs.state.InitialHeight {
			panic(fmt.Sprintf(
				"inconsistent cs.state.LastBlockHeight %v, expected 0 for initial height %v",
				cs.state.LastBlockHeight, cs.state.InitialHeight,
			))
		}

		// If state isn't further out than cs.state, just ignore.
		// This happens when SwitchToConsensus() is called in the reactor.
		// We don't want to reset e.g. the Votes, but we still want to
		// signal the new round step, because other services (eg. txNotifier)
		// depend on having an up-to-date peer state!
		if state.LastBlockHeight <= cs.state.LastBlockHeight {
			cs.Logger.Debug(
				"ignoring updateToState()",
				"new_height", state.LastBlockHeight+1,
				"old_height", cs.state.LastBlockHeight+1,
			)
			cs.newStep()
			return
		}
	}

	// Reset fields based on state.
	validators := state.Validators

	switch {
	case state.LastBlockHeight == 0: // Very first commit should be empty.
		cs.LastCommit = (*types.VoteSet)(nil)
	case cs.CommitRound > -1 && cs.Votes != nil: // Otherwise, use cs.Votes
		if !cs.Votes.Precommits(cs.CommitRound).HasTwoThirdsMajority() {
			panic(fmt.Sprintf(
				"wanted to form a commit, but precommits (H/R: %d/%d) didn't have 2/3+: %v",
				state.LastBlockHeight, cs.CommitRound, cs.Votes.Precommits(cs.CommitRound),
			))
		}

		cs.LastCommit = cs.Votes.Precommits(cs.CommitRound)

	case cs.LastCommit == nil:
		// NOTE: when consensus starts, it has no votes. reconstructLastCommit
		// must be called to reconstruct LastCommit from SeenCommit.
		panic(fmt.Sprintf(
			"last commit cannot be empty after initial block (H:%d)",
			state.LastBlockHeight+1,
		))
	}

	// Next desired block height
	height := state.LastBlockHeight + 1
	if height == 1 {
		height = state.InitialHeight
	}

	// RoundState fields
	cs.updateHeight(height)
	cs.updateRoundStep(0, cstypes.RoundStepNewHeight)

	if cs.CommitTime.IsZero() {
		// "Now" makes it easier to sync up dev nodes.
		// We add timeoutCommit to allow transactions
		// to be gathered for the first block.
		// And alternative solution that relies on clocks:
		// cs.StartTime = state.LastBlockTime.Add(timeoutCommit)
		cs.StartTime = cs.config.Commit(cmttime.Now())
	} else {
		cs.StartTime = cs.config.Commit(cs.CommitTime)
	}

	cs.Validators = validators
	cs.Proposal = nil
	cs.ProposalReceiveTime = time.Time{}
	cs.ProposalBlock = nil
	cs.ProposalBlockParts = nil
	cs.LockedRound = -1
	cs.LockedBlock = nil
	cs.LockedBlockParts = nil
	cs.ValidRound = -1
	cs.ValidBlock = nil
	cs.ValidBlockParts = nil
	if state.ConsensusParams.Feature.VoteExtensionsEnabled(height) {
		cs.Votes = cstypes.NewExtendedHeightVoteSet(state.ChainID, height, validators)
	} else {
		cs.Votes = cstypes.NewHeightVoteSet(state.ChainID, height, validators)
	}
	cs.CommitRound = -1
	cs.LastValidators = state.LastValidators
	cs.TriggeredTimeoutPrecommit = false

	cs.state = state

	// Finally, broadcast RoundState
	cs.newStep()
}

func (cs *State) newStep() {
	rs := cs.RoundStateEvent()
	if err := cs.wal.Write(rs); err != nil {
		cs.Logger.Error("Failed writing to WAL", "err", err)
	}

	cs.nSteps++

	// newStep is called by updateToState in NewState before the eventBus is set!
	if cs.eventBus != nil {
		if err := cs.eventBus.PublishEventNewRoundStep(rs); err != nil {
			cs.Logger.Error("Failed publishing new round step", "err", err)
		}

		cs.evsw.FireEvent(types.EventNewRoundStep, &cs.RoundState)
	}
}

// -----------------------------------------
// the main go routines

// receiveRoutine handles messages which may cause state transitions.
// it's argument (n) is the number of messages to process before exiting - use 0 to run forever
// It keeps the RoundState and is the only thing that updates it.
// Updates (state transitions) happen on timeouts, complete proposals, and 2/3 majorities.
// State must be locked before any internal state is updated.
func (cs *State) receiveRoutine(maxSteps int) {
	onExit := func(cs *State) {
		// NOTE: the internalMsgQueue may have signed messages from our
		// priv_val that haven't hit the WAL, but its ok because
		// priv_val tracks LastSig

		// close wal now that we're done writing to it
		if err := cs.wal.Stop(); err != nil {
			cs.Logger.Error("Failed trying to stop WAL", "error", err)
		}

		cs.wal.Wait()
		close(cs.done)
	}

	defer func() {
		if r := recover(); r != nil {
			cs.Logger.Error("CONSENSUS FAILURE!!!", "err", r, "stack", string(debug.Stack()))
			// stop gracefully
			//
			// NOTE: We most probably shouldn't be running any further when there is
			// some unexpected panic. Some unknown error happened, and so we don't
			// know if that will result in the validator signing an invalid thing. It
			// might be worthwhile to explore a mechanism for manual resuming via
			// some console or secure RPC system, but for now, halting the chain upon
			// unexpected consensus bugs sounds like the better option.
			onExit(cs)
		}
	}()

	for {
		if maxSteps > 0 {
			if cs.nSteps >= maxSteps {
				cs.Logger.Debug("Reached max steps; exiting receive routine")
				cs.nSteps = 0
				return
			}
		}

		rs := cs.RoundState
		var mi msgInfo

		select {
		case <-cs.txNotifier.TxsAvailable():
			cs.handleTxsAvailable()

		case mi = <-cs.peerMsgQueue:
			if err := cs.wal.Write(mi); err != nil {
				cs.Logger.Error("Failed writing to WAL", "err", err)
			}
			// handles proposals, block parts, votes
			// may generate internal events (votes, complete proposals, 2/3 majorities)
			cs.handleMsg(mi)

		case mi = <-cs.internalMsgQueue:
			err := cs.wal.WriteSync(mi) // NOTE: fsync
			if err != nil {
				panic(fmt.Sprintf(
					"failed to write %v msg to consensus WAL due to %v; check your file system and restart the node",
					mi, err,
				))
			}

			if _, ok := mi.Msg.(*VoteMessage); ok {
				// we actually want to simulate failing during
				// the previous WriteSync, but this isn't easy to do.
				// Equivalent would be to fail here and manually remove
				// some bytes from the end of the wal.
				fail.Fail() // XXX
			}

			// handles proposals, block parts, votes
			cs.handleMsg(mi)

		case ti := <-cs.timeoutTicker.Chan(): // tockChan:
			if err := cs.wal.Write(ti); err != nil {
				cs.Logger.Error("failed writing to WAL", "err", err)
			}

			// if the timeout is relevant to the rs
			// go to the next step
			cs.handleTimeout(ti, rs)

		case <-cs.Quit():
			onExit(cs)
			return
		}
	}
}

// state transitions on complete-proposal, 2/3-any, 2/3-one.
func (cs *State) handleMsg(mi msgInfo) {
	cs.mtx.Lock()
	defer cs.mtx.Unlock()
	var (
		added bool
		err   error
	)

	msg, peerID := mi.Msg, mi.PeerID

	switch msg := msg.(type) {
	case *ProposalMessage:
		// will not cause transition.
		// once proposal is set, we can receive block parts
		err = cs.setProposal(msg.Proposal, mi.ReceiveTime)

	case *BlockPartMessage:
		// if the proposal is complete, we'll enterPrevote or tryFinalizeCommit
		added, err = cs.addProposalBlockPart(msg, peerID)

		// We unlock here to yield to any routines that need to read the the RoundState.
		// Previously, this code held the lock from the point at which the final block
		// part was received until the block executed against the application.
		// This prevented the reactor from being able to retrieve the most updated
		// version of the RoundState. The reactor needs the updated RoundState to
		// gossip the now completed block.
		//
		// This code can be further improved by either always operating on a copy
		// of RoundState and only locking when switching out State's copy of
		// RoundState with the updated copy or by emitting RoundState events in
		// more places for routines depending on it to listen for.
		cs.mtx.Unlock()

		cs.mtx.Lock()
		if added && cs.ProposalBlockParts.IsComplete() {
			cs.handleCompleteProposal(msg.Height)
		}
		if added {
			cs.statsMsgQueue <- mi
		}

		if err != nil && msg.Round != cs.Round {
			cs.Logger.Debug(
				"Received block part from wrong round",
				"height", cs.Height,
				"cs_round", cs.Round,
				"block_round", msg.Round,
			)
			err = nil
		}

	case *VoteMessage:
		// attempt to add the vote and dupeout the validator if its a duplicate signature
		// if the vote gives us a 2/3-any or 2/3-one, we transition
		added, err = cs.tryAddVote(msg.Vote, peerID)
		if added {
			cs.statsMsgQueue <- mi
		}

		// if err == ErrAddingVote {
		// TODO: punish peer
		// We probably don't want to stop the peer here. The vote does not
		// necessarily comes from a malicious peer but can be just broadcasted by
		// a typical peer.
		// https://github.com/tendermint/tendermint/issues/1281
		// }

		// NOTE: the vote is broadcast to peers by the reactor listening
		// for vote events

		// TODO: If rs.Height == vote.Height && rs.Round < vote.Round,
		// the peer is sending us CatchupCommit precommits.
		// We could make note of this and help filter in broadcastHasVoteMessage().

	default:
		cs.Logger.Error("Unknown msg type", "type", fmt.Sprintf("%T", msg))
		return
	}

	if err != nil {
		cs.Logger.Error(
			"Failed to process message",
			"height", cs.Height,
			"round", cs.Round,
			"peer", peerID,
			"msg_type", fmt.Sprintf("%T", msg),
			"err", err,
		)
	}
}

func (cs *State) handleTimeout(ti timeoutInfo, rs cstypes.RoundState) {
	cs.Logger.Debug("Received tock", "timeout", ti.Duration, "height", ti.Height, "round", ti.Round, "step", ti.Step)

	// timeouts must be for current height, round, step
	if ti.Height != rs.Height || ti.Round < rs.Round || (ti.Round == rs.Round && ti.Step < rs.Step) {
		cs.Logger.Debug("Ignoring tock because we are ahead", "height", rs.Height, "round", rs.Round, "step", rs.Step)
		return
	}

	// the timeout will now cause a state transition
	cs.mtx.Lock()
	defer cs.mtx.Unlock()

	switch ti.Step {
	case cstypes.RoundStepNewHeight:
		// NewRound event fired from enterNewRound.
		// XXX: should we fire timeout here (for timeout commit)?
		cs.enterNewRound(ti.Height, 0)

	case cstypes.RoundStepNewRound:
		cs.enterPropose(ti.Height, ti.Round)

	case cstypes.RoundStepPropose:
		if err := cs.eventBus.PublishEventTimeoutPropose(cs.RoundStateEvent()); err != nil {
			cs.Logger.Error("failed publishing timeout propose", "err", err)
		}

		cs.enterPrevote(ti.Height, ti.Round)

	case cstypes.RoundStepPrevoteWait:
		if err := cs.eventBus.PublishEventTimeoutWait(cs.RoundStateEvent()); err != nil {
			cs.Logger.Error("Failed publishing timeout wait", "err", err)
		}

		cs.enterPrecommit(ti.Height, ti.Round)

	case cstypes.RoundStepPrecommitWait:
		if err := cs.eventBus.PublishEventTimeoutWait(cs.RoundStateEvent()); err != nil {
			cs.Logger.Error("Failed publishing timeout wait", "err", err)
		}

		cs.enterPrecommit(ti.Height, ti.Round)
		cs.enterNewRound(ti.Height, ti.Round+1)

	default:
		panic(cmterrors.ErrInvalidField{Field: "timeout_step"})
	}
}

func (cs *State) handleTxsAvailable() {
	cs.mtx.Lock()
	defer cs.mtx.Unlock()

	// We only need to do this for round 0.
	if cs.Round != 0 {
		return
	}

	switch cs.Step {
	case cstypes.RoundStepNewHeight: // timeoutCommit phase
		if cs.needProofBlock(cs.Height) {
			// enterPropose will be called by enterNewRound
			return
		}

		// +1ms to ensure RoundStepNewRound timeout always happens after RoundStepNewHeight
		timeoutCommit := cs.StartTime.Sub(cmttime.Now()) + 1*time.Millisecond
		cs.scheduleTimeout(timeoutCommit, cs.Height, 0, cstypes.RoundStepNewRound)

	case cstypes.RoundStepNewRound: // after timeoutCommit
		cs.enterPropose(cs.Height, 0)
	}
}

// -----------------------------------------------------------------------------
// State functions
// Used internally by handleTimeout and handleMsg to make state transitions

// Enter: `timeoutNewHeight` by startTime (commitTime+timeoutCommit),
//
//	or, if TimeoutCommit==0, after receiving all precommits from (height,round-1)
//
// Enter: `timeoutPrecommits` after any +2/3 precommits from (height,round-1)
// Enter: +2/3 precommits for nil at (height,round-1)
// Enter: +2/3 prevotes any or +2/3 precommits for block or any from (height, round)
// NOTE: cs.StartTime was already set for height.
func (cs *State) enterNewRound(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cs.Step != cstypes.RoundStepNewHeight) {
		logger.Debug(
			"Entering new round with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	if now := cmttime.Now(); cs.StartTime.After(now) {
		logger.Debug("Need to set a buffer and log message here for sanity", "start_time", cs.StartTime, "now", now)
	}

	prevHeight, prevRound, prevStep := cs.Height, cs.Round, cs.Step

	// increment validators if necessary
	validators := cs.Validators
	if cs.Round < round {
		validators = validators.Copy()
		validators.IncrementProposerPriority(cmtmath.SafeSubInt32(round, cs.Round))
	}

	// Setup new round
	// we don't fire newStep for this step,
	// but we fire an event, so update the round step first
	cs.updateRoundStep(round, cstypes.RoundStepNewRound)
	cs.Validators = validators
	// If round == 0, we've already reset these upon new height, and meanwhile
	// we might have received a proposal for round 0.
	propAddress := validators.GetProposer().PubKey.Address()
	if round != 0 {
		logger.Info("Resetting proposal info", "proposer", propAddress)
		cs.Proposal = nil
		cs.ProposalReceiveTime = time.Time{}
		cs.ProposalBlock = nil
		cs.ProposalBlockParts = nil
	}

	logger.Debug("Entering new round",
		"previous", log.NewLazySprintf("%v/%v/%v", prevHeight, prevRound, prevStep),
		"proposer", propAddress,
	)

	cs.Votes.SetRound(cmtmath.SafeAddInt32(round, 1)) // also track next round (round+1) to allow round-skipping
	cs.TriggeredTimeoutPrecommit = false

	if err := cs.eventBus.PublishEventNewRound(cs.NewRoundEvent()); err != nil {
		cs.Logger.Error("Failed publishing new round", "err", err)
	}
	// Wait for txs to be available in the mempool
	// before we enterPropose in round 0. If the last block changed the app hash,
	// we may need an empty "proof" block, and enterPropose immediately.
	waitForTxs := cs.config.WaitForTxs() && round == 0 && !cs.needProofBlock(height)
	if waitForTxs {
		if cs.config.CreateEmptyBlocksInterval > 0 {
			cs.scheduleTimeout(cs.config.CreateEmptyBlocksInterval, height, round,
				cstypes.RoundStepNewRound)
		}
		return
	}

	cs.enterPropose(height, round)
}

// needProofBlock returns true on the first height (so the genesis app hash is signed right away)
// and where the last block (height-1) caused the app hash to change.
func (cs *State) needProofBlock(height int64) bool {
	if height == cs.state.InitialHeight {
		return true
	}

	lastBlockMeta := cs.blockStore.LoadBlockMeta(height - 1)
	if lastBlockMeta == nil {
		// See https://github.com/cometbft/cometbft/issues/370
		cs.Logger.Info("Short-circuited needProofBlock", "height", height, "InitialHeight", cs.state.InitialHeight)
		return true
	}

	return !bytes.Equal(cs.state.AppHash, lastBlockMeta.Header.AppHash)
}

// Enter (CreateEmptyBlocks): from enterNewRound(height,round)
// Enter (CreateEmptyBlocks, CreateEmptyBlocksInterval > 0 ):
//
//	after enterNewRound(height,round), after timeout of CreateEmptyBlocksInterval
//
// Enter (!CreateEmptyBlocks) : after enterNewRound(height,round), once txs are in the mempool.
func (cs *State) enterPropose(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPropose <= cs.Step) {
		logger.Debug(
			"entering propose step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	// If this validator is the proposer of this round, and the previous block time is later than
	// our local clock time, wait to propose until our local clock time has passed the block time.
	if cs.isPBTSEnabled(height) && cs.privValidatorPubKey != nil && cs.isProposer(cs.privValidatorPubKey.Address()) {
		proposerWaitTime := proposerWaitTime(cmttime.DefaultSource{}, cs.state.LastBlockTime)
		if proposerWaitTime > 0 {
			cs.scheduleTimeout(proposerWaitTime, height, round, cstypes.RoundStepNewRound)
			return
		}
	}

	logger.Debug("Entering propose step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterPropose:
		cs.updateRoundStep(round, cstypes.RoundStepPropose)
		cs.newStep()

		// If we have the whole proposal + POL, then goto Prevote now.
		// else, we'll enterPrevote when the rest of the proposal is received (in AddProposalBlockPart),
		// or else after timeoutPropose
		if cs.isProposalComplete() {
			cs.enterPrevote(height, cs.Round)
		}
	}()

	// If we don't get the proposal and all block parts quick enough, enterPrevote
	cs.scheduleTimeout(cs.config.Propose(round), height, round, cstypes.RoundStepPropose)

	// Nothing more to do if we're not a validator
	if cs.privValidator == nil {
		logger.Debug("Propose step; not proposing since node is not a validator")
		return
	}

	if cs.privValidatorPubKey == nil {
		// If this node is a validator & proposer in the current round, it will
		// miss the opportunity to create a block.
		logger.Error("Propose step; empty priv validator public key", "err", ErrPubKeyIsNotSet)
		return
	}

	addr := cs.privValidatorPubKey.Address()

	// if not a validator, we're done
	if !cs.Validators.HasAddress(addr) {
		logger.Debug("Propose step; not proposing since node is not in the validator set",
			"addr", addr,
			"vals", cs.Validators)
		return
	}

	if cs.isProposer(addr) {
		logger.Debug("Propose step; our turn to propose", "proposer", addr)
		cs.decideProposal(height, round)
	} else {
		logger.Debug("Propose step; not our turn to propose", "proposer", cs.Validators.GetProposer().Address)
	}
}

func (cs *State) isProposer(address []byte) bool {
	return bytes.Equal(cs.Validators.GetProposer().Address, address)
}

func (cs *State) defaultDecideProposal(height int64, round int32) {
	var block *types.Block
	var blockParts *types.PartSet

	// Decide on block
	if cs.ValidBlock != nil {
		// If there is valid block, choose that.
		block, blockParts = cs.ValidBlock, cs.ValidBlockParts
	} else {
		// Create a new proposal block from state/txs from the mempool.
		var err error
		block, err = cs.createProposalBlock(context.TODO())
		if err != nil {
			cs.Logger.Error("Unable to create proposal block", "error", err)
			return
		} else if block == nil {
			panic("Method createProposalBlock should not provide a nil block without errors")
		}
		cs.metrics.ProposalCreateCount.Add(1)
		blockParts, err = block.MakePartSet(types.BlockPartSizeBytes)
		if err != nil {
			cs.Logger.Error("unable to create proposal block part set", "error", err)
			return
		}
	}

	// Flush the WAL. Otherwise, we may not recompute the same proposal to sign,
	// and the privValidator will refuse to sign anything.
	if err := cs.wal.FlushAndSync(); err != nil {
		cs.Logger.Error("failed flushing WAL to disk")
	}

	// Make proposal
	propBlockID := types.BlockID{Hash: block.Hash(), PartSetHeader: blockParts.Header()}
	proposal := types.NewProposal(height, round, cs.ValidRound, propBlockID, block.Header.Time)
	p := proposal.ToProto()
	if err := cs.privValidator.SignProposal(cs.state.ChainID, p); err == nil {
		proposal.Signature = p.Signature

		// send proposal and block parts on internal msg queue
		cs.sendInternalMessage(msgInfo{&ProposalMessage{proposal}, "", cmttime.Now()})

		for i := 0; i < int(blockParts.Total()); i++ {
			part := blockParts.GetPart(i)
			cs.sendInternalMessage(msgInfo{&BlockPartMessage{cs.Height, cs.Round, part}, "", time.Time{}})
		}

		cs.Logger.Debug("Signed proposal", "height", height, "round", round, "proposal", proposal)
	} else if !cs.replayMode {
		cs.Logger.Error("Propose step; failed signing proposal", "height", height, "round", round, "err", err)
	}
}

// Returns true if the proposal block is complete &&
// (if POLRound was proposed, we have +2/3 prevotes from there).
func (cs *State) isProposalComplete() bool {
	if cs.Proposal == nil || cs.ProposalBlock == nil {
		return false
	}
	// we have the proposal. if there's a POLRound,
	// make sure we have the prevotes from it too
	if cs.Proposal.POLRound < 0 {
		return true
	}
	// if this is false the proposer is lying or we haven't received the POL yet
	return cs.Votes.Prevotes(cs.Proposal.POLRound).HasTwoThirdsMajority()
}

// Create the next block to propose and return it. Returns nil block upon error.
//
// We really only need to return the parts, but the block is returned for
// convenience so we can log the proposal block.
//
// NOTE: keep it side-effect free for clarity.
// CONTRACT: cs.privValidator is not nil.
func (cs *State) createProposalBlock(ctx context.Context) (*types.Block, error) {
	if cs.privValidator == nil {
		return nil, ErrNilPrivValidator
	}

	// TODO(sergio): wouldn't it be easier if CreateProposalBlock accepted cs.LastCommit directly?
	var lastExtCommit *types.ExtendedCommit
	switch {
	case cs.Height == cs.state.InitialHeight:
		// We're creating a proposal for the first block.
		// The commit is empty, but not nil.
		lastExtCommit = &types.ExtendedCommit{}

	case cs.LastCommit.HasTwoThirdsMajority():
		// Make the commit from LastCommit
		lastExtCommit = cs.LastCommit.MakeExtendedCommit(cs.state.ConsensusParams.Feature)

	default: // This shouldn't happen.
		return nil, ErrProposalWithoutPreviousCommit
	}

	if cs.privValidatorPubKey == nil {
		// If this node is a validator & proposer in the current round, it will
		// miss the opportunity to create a block.
		return nil, fmt.Errorf("propose step; empty priv validator public key, error: %w", ErrPubKeyIsNotSet)
	}

	proposerAddr := cs.privValidatorPubKey.Address()

	ret, err := cs.blockExec.CreateProposalBlock(ctx, cs.Height, cs.state, lastExtCommit, proposerAddr)
	if err != nil {
		panic(err)
	}
	return ret, nil
}

// Enter: isProposalComplete() and Step <= RoundStepPropose.
// Enter: `timeout_propose` (timeout of RoundStepPropose type) expires.
//
// If we received a valid proposal and the associated proposed block within
// this round and: (i) we are not locked on a block, or we are locked on the
// proposed block, or (ii) the proposed block received a POL in a round greater
// or equal than our locked round, we will prevote for the poroposed block ID.
// Otherwise, we prevote nil.
func (cs *State) enterPrevote(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrevote <= cs.Step) {
		logger.Debug(
			"Entering prevote step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	defer func() {
		// Done enterPrevote:
		cs.updateRoundStep(round, cstypes.RoundStepPrevote)
		cs.newStep()
	}()

	logger.Debug("Entering prevote step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	// Sign and broadcast vote as necessary
	cs.doPrevote(height, round)

	// Once `addVote` hits any +2/3 prevotes, we will go to PrevoteWait
	// (so we have more time to try and collect +2/3 prevotes for a single block)
}

func (cs *State) timelyProposalMargins() (time.Duration, time.Duration) {
	sp := cs.state.ConsensusParams.Synchrony.InRound(cs.Round)

	// cs.ProposalReceiveTime - cs.Proposal.Timestamp >= -1 * Precision
	// cs.ProposalReceiveTime - cs.Proposal.Timestamp <= MessageDelay + Precision
	return -sp.Precision, sp.MessageDelay + sp.Precision
}

func (cs *State) proposalIsTimely() bool {
	sp := cs.state.ConsensusParams.Synchrony.InRound(cs.Proposal.Round)

	return cs.Proposal.IsTimely(cs.ProposalReceiveTime, sp)
}

// Implements doPrevote. Called by enterPrevote(height, round) provided that
// round == cs.Round, height == cs.Height, and cs.Step <= // RoundStepPropose.
func (cs *State) defaultDoPrevote(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	// We did not receive a valid proposal for this round (and thus executing this from a timeout).
	if cs.Proposal == nil {
		logger.Debug("Prevote step: did not receive a valid Proposal; prevoting nil")
		cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
		return
	}

	// We did not (fully) receive the proposed block (and thus executing this from a timeout).
	if cs.ProposalBlock == nil {
		logger.Debug("Prevote step: did not receive the ProposalBlock; prevoting nil")
		cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
		return
	}

	// Timestamp validation using Proposed-Based TimeStamp (PBTS) algorithm.
	// See: https://github.com/cometbft/cometbft/blob/main/spec/consensus/proposer-based-timestamp/
	if cs.isPBTSEnabled(height) {
		if !cs.Proposal.Timestamp.Equal(cs.ProposalBlock.Header.Time) {
			logger.Debug("prevote step: proposal timestamp not equal; prevoting nil")
			cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
			return
		}

		if cs.Proposal.POLRound == -1 && !cs.proposalIsTimely() {
			lowerBound, upperBound := cs.timelyProposalMargins()
			// TODO: use Warn level once available.
			logger.Info("prevote step: Proposal is not timely; prevoting nil",
				"timestamp", cs.Proposal.Timestamp.Format(time.RFC3339Nano),
				"receive_time", cs.ProposalReceiveTime.Format(time.RFC3339Nano),
				"timestamp_difference", cs.ProposalReceiveTime.Sub(cs.Proposal.Timestamp),
				"lower_bound", lowerBound,
				"upper_bound", upperBound)
			cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
			return
		}

		if cs.Proposal.POLRound == -1 {
			logger.Debug("prevote step: Proposal is timely",
				"timestamp", cs.Proposal.Timestamp.Format(time.RFC3339Nano),
				"receive_time", cs.ProposalReceiveTime.Format(time.RFC3339Nano),
				"timestamp_difference", cs.ProposalReceiveTime.Sub(cs.Proposal.Timestamp))
		}
	}

	// Validate proposal block, from consensus' perspective
	err := cs.blockExec.ValidateBlock(cs.state, cs.ProposalBlock)
	if err != nil {
		// ProposalBlock is invalid, prevote nil.
		logger.Error("prevote step: consensus deems this block invalid; prevoting nil",
			"err", err)
		cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
		return
	}

	/*
		22: upon <PROPOSAL, h_p, round_p, v, −1> from proposer(h_p, round_p) while step_p = propose do
		23: if valid(v) && (lockedRound_p = −1 || lockedValue_p = v) then
		24:   broadcast <PREVOTE, h_p, round_p, id(v)>
		25: else
		26:   broadcast <PREVOTE, h_p, round_p, nil>

		Here, cs.Proposal.POLRound corresponds to the -1 in the rule of the pseude-code (line 22).
		This means that the proposer is producing a new proposal that has not previously
		seen a 2/3 majority by the network.

		If the application deems the proposal as valid AND we're not locked on a
		block OR the proposal matches our locked block (line 23), we prevote the
		proposal (line 24).

		Otherwise, we have already locked on a value that is different from the
		proposed value, so we prevote nil (line 26).

		Note that there are two cases on which we know that the proposal is
		application-valid, that is, it was validated by the application at least
		by one correct node in a previous step:
		- when the proposal matches our non-nil valid block AND we're not locked on a block, and
		- when the proposal matches our non-nil locked block.
		In these cases we do not need to query the application to validate the
		proposal.
	*/
	if cs.Proposal.POLRound == -1 {
		if cs.LockedRound == -1 {
			if cs.ValidRound != -1 && cs.ProposalBlock.HashesTo(cs.ValidBlock.Hash()) {
				logger.Debug("Prevote step: ProposalBlock matches our valid block; prevoting the proposal")
				cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
				return
			}

			// We request the Application, via a `ProcessProposal` ABCI call, to
			// confirm that the block is valid. If the application does not
			// accept the block, consensus prevotes nil.
			//
			// WARNING: misuse of block rejection by the Application can seriously compromise
			// the liveness properties of consensus.
			// Please see `PrepareProosal`-`ProcessProposal` coherence and determinism properties
			// in the ABCI++ specification.
			isAppValid, err := cs.blockExec.ProcessProposal(cs.ProposalBlock, cs.state)
			if err != nil {
				panic(fmt.Sprintf(
					"state machine returned an error (%v) when calling ProcessProposal", err,
				))
			}
			cs.metrics.MarkProposalProcessed(isAppValid)

			if !isAppValid {
				logger.Error("Prevote step: state machine rejected a proposed block; this should not happen:"+
					"the proposer may be misbehaving; prevoting nil", "err", err)
				cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
				return
			}

			logger.Debug("Prevote step: ProposalBlock is valid and there is no locked block; prevoting the proposal")
			cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
			return
		}

		if cs.ProposalBlock.HashesTo(cs.LockedBlock.Hash()) {
			logger.Debug("Prevote step: ProposalBlock is valid (POLRound is -1) and matches our locked block; prevoting the proposal")
			cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
			return
		}

		logger.Debug("Prevote step: ProposalBlock is valid (POLRound is -1), but doesn't match our locked block; prevoting nil")
		cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
		return
	}

	/*
		28: upon <PROPOSAL, h_p, round_p, v, v_r> from proposer(h_p, round_p) AND 2f + 1 <PREVOTE, h_p, v_r, id(v)> while
		step_p = propose && (v_r ≥ 0 && v_r < round_p) do
		29: if valid(v) && (lockedRound_p ≤ v_r || lockedValue_p = v) then
		30:   broadcast <PREVOTE, h_p, round_p, id(v)>
		31: else
		32:   broadcast <PREVOTE, h_p, round_p, nil>

		This rule is a bit confusing but breaks down as follows:

		First note that 'valid(v)' in line 29 states that we should request the
		application to validate the proposal. We know that the proposal was
		prevoted by a +2/3 majority, so it must have been prevoted and validated
		at least by one correct node. Therefore it must be valid and in the
		following cases we don't need to query the application again.

		If we see a proposal in the current round for value 'v' that lists its valid round as 'v_r'
		AND this validator saw a 2/3 majority of the voting power prevote for 'v' in round 'v_r' (line 28),
		then we will issue a prevote for 'v' in this round (line 30) if 'v' either matches our locked value OR
		'v_r' is a round greater than or equal to our current locked round (line 29).
		Otherwise we prevote nil (line 32).

		Note that 'v_r' can be a round greater than to our current locked round if a 2/3 majority of
		the network prevoted a value in round 'v_r' but we did not lock on it, possibly because we
		missed the proposal in round 'v_r'.
	*/
	blockID, ok := cs.Votes.Prevotes(cs.Proposal.POLRound).TwoThirdsMajority()
	ok = ok && !blockID.IsNil()
	if ok && cs.ProposalBlock.HashesTo(blockID.Hash) && cs.Proposal.POLRound < cs.Round {
		if cs.LockedRound < cs.Proposal.POLRound {
			logger.Debug("Prevote step: ProposalBlock is valid and received a 2/3" +
				"majority in a round later than the locked round; prevoting the proposal")
			cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
			return
		}
		if cs.ProposalBlock.HashesTo(cs.LockedBlock.Hash()) {
			logger.Debug("Prevote step: ProposalBlock is valid and matches our locked block; prevoting the proposal")
			cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
			return
		}
		// If v_r = lockedRound_p we expect v to match lockedValue_p. If it is not the case,
		// we have two 2/3+ majorities for different values at round v_r, meaning that the
		// assumption of a 2/3+ majority of honest processes was violated. We should at
		// least log this scenario, see: https://github.com/cometbft/cometbft/issues/1309.
		if cs.LockedRound == cs.Proposal.POLRound {
			logger.Info("Prevote step: ProposalBlock is valid and received a 2/3" +
				"majority at our locked round, while not matching our locked value;" +
				"this can only happen when 1/3 or more validators are double signing; prevoting the proposal")
			cs.signAddVote(types.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header(), nil)
			return
		}
	}

	logger.Debug("Prevote step: ProposalBlock is valid but was not our locked block or " +
		"did not receive a more recent majority; prevoting nil")
	cs.signAddVote(types.PrevoteType, nil, types.PartSetHeader{}, nil)
}

// Enter: any +2/3 prevotes at next round.
func (cs *State) enterPrevoteWait(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrevoteWait <= cs.Step) {
		logger.Debug(
			"Entering prevote wait step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	if !cs.Votes.Prevotes(round).HasTwoThirdsAny() {
		panic(fmt.Sprintf(
			"Entering prevote wait step (%v/%v), but prevotes does not have any +2/3 votes",
			height, round,
		))
	}

	logger.Debug("Entering prevote wait step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterPrevoteWait:
		cs.updateRoundStep(round, cstypes.RoundStepPrevoteWait)
		cs.newStep()
	}()

	// Wait for some more prevotes; enterPrecommit
	cs.scheduleTimeout(cs.config.Prevote(round), height, round, cstypes.RoundStepPrevoteWait)
}

// Enter: `timeoutPrevote` after any +2/3 prevotes.
// Enter: `timeoutPrecommit` after any +2/3 precommits.
// Enter: +2/3 precomits for block or nil.
// Lock & precommit the ProposalBlock if we have enough prevotes for it (a POL in this round)
// else, precommit nil otherwise.
func (cs *State) enterPrecommit(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrecommit <= cs.Step) {
		logger.Debug(
			"Entering precommit step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	logger.Debug("Entering precommit step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterPrecommit:
		cs.updateRoundStep(round, cstypes.RoundStepPrecommit)
		cs.newStep()
	}()

	// check for a polka
	blockID, ok := cs.Votes.Prevotes(round).TwoThirdsMajority()

	// If we don't have a polka, we must precommit nil.
	if !ok {
		if cs.LockedBlock != nil {
			logger.Debug("Precommit step; no +2/3 prevotes during enterPrecommit while we are locked; precommitting nil")
		} else {
			logger.Debug("Precommit step; no +2/3 prevotes during enterPrecommit; precommitting nil")
		}

		cs.signAddVote(types.PrecommitType, nil, types.PartSetHeader{}, nil)
		return
	}

	// At this point +2/3 prevoted for a particular block or nil.
	if err := cs.eventBus.PublishEventPolka(cs.RoundStateEvent()); err != nil {
		logger.Error("Failed publishing polka", "err", err)
	}

	// the latest POLRound should be this round.
	polRound, _ := cs.Votes.POLInfo()
	if polRound < round {
		panic(fmt.Sprintf("this POLRound should be %v but got %v", round, polRound))
	}

	// +2/3 prevoted nil. Precommit nil.
	if blockID.IsNil() {
		logger.Debug("Precommit step; +2/3 prevoted for nil; precommitting nil")
		cs.signAddVote(types.PrecommitType, nil, types.PartSetHeader{}, nil)
		return
	}
	// At this point, +2/3 prevoted for a particular block.

	// If we're already locked on that block, precommit it, and update the LockedRound
	if cs.LockedBlock.HashesTo(blockID.Hash) {
		logger.Debug("Precommit step; +2/3 prevoted locked block; relocking")
		cs.LockedRound = round

		if err := cs.eventBus.PublishEventRelock(cs.RoundStateEvent()); err != nil {
			logger.Error("Precommit step; failed publishing event relock", "err", err)
		}

		cs.signAddVote(types.PrecommitType, blockID.Hash, blockID.PartSetHeader, cs.LockedBlock)
		return
	}

	// If greater than 2/3 of the voting power on the network prevoted for
	// the proposed block, update our locked block to this block and issue a
	// precommit vote for it.
	if cs.ProposalBlock.HashesTo(blockID.Hash) {
		logger.Debug("Precommit step: +2/3 prevoted proposal block; locking", "hash", blockID.Hash)

		// Validate the block.
		if err := cs.blockExec.ValidateBlock(cs.state, cs.ProposalBlock); err != nil {
			panic(fmt.Sprintf("Precommit step; +2/3 prevoted for an invalid block: %v; relocking", err))
		}

		cs.LockedRound = round
		cs.LockedBlock = cs.ProposalBlock
		cs.LockedBlockParts = cs.ProposalBlockParts

		if err := cs.eventBus.PublishEventLock(cs.RoundStateEvent()); err != nil {
			logger.Error("Precommit step; failed publishing event lock", "err", err)
		}

		cs.signAddVote(types.PrecommitType, blockID.Hash, blockID.PartSetHeader, cs.ProposalBlock)
		return
	}

	// There was a polka in this round for a block we don't have.
	// Fetch that block, and precommit nil.
	logger.Debug("Precommit step; +2/3 prevotes for a block we do not have; voting nil", "block_id", blockID)

	if !cs.ProposalBlockParts.HasHeader(blockID.PartSetHeader) {
		cs.ProposalBlock = nil
		cs.ProposalBlockParts = types.NewPartSetFromHeader(blockID.PartSetHeader)
	}

	cs.signAddVote(types.PrecommitType, nil, types.PartSetHeader{}, nil)
}

// Enter: any +2/3 precommits for next round.
func (cs *State) enterPrecommitWait(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cs.TriggeredTimeoutPrecommit) {
		logger.Debug(
			"Entering precommit wait step with invalid args",
			"triggered_timeout", cs.TriggeredTimeoutPrecommit,
			"current", log.NewLazySprintf("%v/%v", cs.Height, cs.Round),
		)
		return
	}

	if !cs.Votes.Precommits(round).HasTwoThirdsAny() {
		panic(fmt.Sprintf(
			"entering precommit wait step (%v/%v), but precommits does not have any +2/3 votes",
			height, round,
		))
	}

	logger.Debug("Entering precommit wait step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterPrecommitWait:
		cs.TriggeredTimeoutPrecommit = true
		cs.newStep()
	}()

	// wait for some more precommits; enterNewRound
	cs.scheduleTimeout(cs.config.Precommit(round), height, round, cstypes.RoundStepPrecommitWait)
}

// Enter: +2/3 precommits for block.
func (cs *State) enterCommit(height int64, commitRound int32) {
	logger := cs.Logger.With("height", height, "commit_round", commitRound)

	if cs.Height != height || cstypes.RoundStepCommit <= cs.Step {
		logger.Debug(
			"Entering commit step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	logger.Debug("Entering commit step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterCommit:
		// keep cs.Round the same, commitRound points to the right Precommits set.
		cs.updateRoundStep(cs.Round, cstypes.RoundStepCommit)
		cs.CommitRound = commitRound
		cs.CommitTime = cmttime.Now()
		cs.newStep()

		// Maybe finalize immediately.
		cs.tryFinalizeCommit(height)
	}()

	blockID, ok := cs.Votes.Precommits(commitRound).TwoThirdsMajority()
	if !ok || blockID.IsNil() {
		panic("RunActionCommit() expects +2/3 precommits")
	}

	// The Locked* fields no longer matter.
	// Move them over to ProposalBlock if they match the commit hash,
	// otherwise they'll be cleared in updateToState.
	if cs.LockedBlock.HashesTo(blockID.Hash) {
		logger.Debug("Commit is for a locked block; set ProposalBlock=LockedBlock", "block_hash", blockID.Hash)
		cs.ProposalBlock = cs.LockedBlock
		cs.ProposalBlockParts = cs.LockedBlockParts
	}

	// If we don't have the block being committed, set up to get it.
	if !cs.ProposalBlock.HashesTo(blockID.Hash) {
		if !cs.ProposalBlockParts.HasHeader(blockID.PartSetHeader) {
			logger.Info(
				"Commit is for a block we do not know about; set ProposalBlock=nil",
				"proposal", log.NewLazyBlockHash(cs.ProposalBlock),
				"commit", blockID.Hash,
			)

			// We're getting the wrong block.
			// Set up ProposalBlockParts and keep waiting.
			cs.ProposalBlock = nil
			cs.ProposalBlockParts = types.NewPartSetFromHeader(blockID.PartSetHeader)

			if err := cs.eventBus.PublishEventValidBlock(cs.RoundStateEvent()); err != nil {
				logger.Error("Failed publishing valid block", "err", err)
			}

			cs.evsw.FireEvent(types.EventValidBlock, &cs.RoundState)
		}
	}
}

// If we have the block AND +2/3 commits for it, finalize.
func (cs *State) tryFinalizeCommit(height int64) {
	logger := cs.Logger.With("height", height)

	if cs.Height != height {
		panic(fmt.Sprintf("tryFinalizeCommit() cs.Height: %v vs height: %v", cs.Height, height))
	}

	blockID, ok := cs.Votes.Precommits(cs.CommitRound).TwoThirdsMajority()
	if !ok || blockID.IsNil() {
		logger.Error("Failed attempt to finalize commit; there was no +2/3 majority or +2/3 was for nil")
		return
	}

	if !cs.ProposalBlock.HashesTo(blockID.Hash) {
		// TODO: this happens every time if we're not a validator (ugly logs)
		// TODO: ^^ wait, why does it matter that we're a validator?
		logger.Debug(
			"Failed attempt to finalize commit; we do not have the commit block",
			"proposal_block", log.NewLazyBlockHash(cs.ProposalBlock),
			"commit_block", blockID.Hash,
		)
		return
	}

	cs.finalizeCommit(height)
}

// Increment height and goto cstypes.RoundStepNewHeight.
func (cs *State) finalizeCommit(height int64) {
	logger := cs.Logger.With("height", height)

	if cs.Height != height || cs.Step != cstypes.RoundStepCommit {
		logger.Debug(
			"Entering finalize commit step",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	cs.calculatePrevoteMessageDelayMetrics()

	blockID, ok := cs.Votes.Precommits(cs.CommitRound).TwoThirdsMajority()
	block, blockParts := cs.ProposalBlock, cs.ProposalBlockParts

	if !ok {
		panic("cannot finalize commit; commit does not have 2/3 majority")
	}
	if !blockParts.HasHeader(blockID.PartSetHeader) {
		panic("expected ProposalBlockParts header to be commit header")
	}
	if !block.HashesTo(blockID.Hash) {
		panic("cannot finalize commit; proposal block does not hash to commit hash")
	}

	if err := cs.blockExec.ValidateBlock(cs.state, block); err != nil {
		panic(fmt.Errorf("+2/3 committed an invalid block: %w", err))
	}

	logger.Info(
		"Finalizing commit of block",
		"hash", log.NewLazyBlockHash(block),
		"root", block.AppHash,
		"num_txs", len(block.Txs),
	)
	logger.Debug("Committed block", "block", log.NewLazySprintf("%v", block))

	fail.Fail() // XXX

	// Save to blockStore.
	if cs.blockStore.Height() < block.Height {
		// NOTE: the seenCommit is local justification to commit this block,
		// but may differ from the LastCommit included in the next block
		seenExtendedCommit := cs.Votes.Precommits(cs.CommitRound).MakeExtendedCommit(cs.state.ConsensusParams.Feature)
		if cs.state.ConsensusParams.Feature.VoteExtensionsEnabled(block.Height) {
			cs.blockStore.SaveBlockWithExtendedCommit(block, blockParts, seenExtendedCommit)
		} else {
			cs.blockStore.SaveBlock(block, blockParts, seenExtendedCommit.ToCommit())
		}
	} else {
		// Happens during replay if we already saved the block but didn't commit
		logger.Debug("Calling finalizeCommit on already stored block", "height", block.Height)
	}

	fail.Fail() // XXX

	// Write EndHeightMessage{} for this height, implying that the blockstore
	// has saved the block.
	//
	// If we crash before writing this EndHeightMessage{}, we will recover by
	// running ApplyBlock during the ABCI handshake when we restart.  If we
	// didn't save the block to the blockstore before writing
	// EndHeightMessage{}, we'd have to change WAL replay -- currently it
	// complains about replaying for heights where an #ENDHEIGHT entry already
	// exists.
	//
	// Either way, the State should not be resumed until we
	// successfully call ApplyBlock (ie. later here, or in Handshake after
	// restart).
	endMsg := EndHeightMessage{height}
	if err := cs.wal.WriteSync(endMsg); err != nil { // NOTE: fsync
		panic(fmt.Sprintf(
			"failed to write %v msg to consensus WAL due to %v; check your file system and restart the node",
			endMsg, err,
		))
	}

	fail.Fail() // XXX

	// Create a copy of the state for staging and an event cache for txs.
	stateCopy := cs.state.Copy()

	// Execute and commit the block, update and save the state, and update the mempool.
	// We use apply verified block here because we have verified the block in this function already.
	// NOTE The block.AppHash won't reflect these txs until the next block.
	stateCopy, err := cs.blockExec.ApplyVerifiedBlock(
		stateCopy,
		types.BlockID{
			Hash:          block.Hash(),
			PartSetHeader: blockParts.Header(),
		},
		block,
		block.Height,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to apply block; error %v", err))
	}

	fail.Fail() // XXX

	// must be called before we update state
	cs.recordMetrics(height, block)

	// NewHeightStep!
	cs.updateToState(stateCopy)

	fail.Fail() // XXX

	// Private validator might have changed it's key pair => refetch pubkey.
	if err := cs.updatePrivValidatorPubKey(); err != nil {
		logger.Error("Failed to get private validator pubkey", "err", err)
	}

	// cs.StartTime is already set.
	// Schedule Round0 to start soon.
	cs.scheduleRound0(&cs.RoundState)

	// By here,
	// * cs.Height has been increment to height+1
	// * cs.Step is now cstypes.RoundStepNewHeight
	// * cs.StartTime is set to when we will start round0.
}

func (cs *State) recordMetrics(height int64, block *types.Block) {
	cs.metrics.Validators.Set(float64(cs.Validators.Size()))
	cs.metrics.ValidatorsPower.Set(float64(cs.Validators.TotalVotingPower()))

	var (
		missingValidators      int
		missingValidatorsPower int64
	)
	// height=0 -> MissingValidators and MissingValidatorsPower are both 0.
	// Remember that the first LastCommit is intentionally empty, so it's not
	// fair to increment missing validators number.
	if height > cs.state.InitialHeight {
		// Sanity check that commit size matches validator set size - only applies
		// after first block.
		var (
			commitSize = block.LastCommit.Size()
			valSetLen  = len(cs.LastValidators.Validators)
			address    types.Address
		)
		if commitSize != valSetLen {
			panic(fmt.Sprintf("commit size (%d) doesn't match valset length (%d) at height %d\n\n%v\n\n%v",
				commitSize, valSetLen, block.Height, block.LastCommit.Signatures, cs.LastValidators.Validators))
		}

		if cs.privValidator != nil {
			if cs.privValidatorPubKey == nil {
				// Metrics won't be updated, but it's not critical.
				cs.Logger.Error(fmt.Sprintf("recordMetrics: %v", ErrPubKeyIsNotSet))
			} else {
				address = cs.privValidatorPubKey.Address()
			}
		}

		for i, val := range cs.LastValidators.Validators {
			commitSig := block.LastCommit.Signatures[i]
			if commitSig.BlockIDFlag == types.BlockIDFlagAbsent {
				missingValidators++
				missingValidatorsPower += val.VotingPower
			}

			if bytes.Equal(val.Address, address) {
				label := []string{
					"validator_address", val.Address.String(),
				}
				cs.metrics.ValidatorPower.With(label...).Set(float64(val.VotingPower))
				if commitSig.BlockIDFlag == types.BlockIDFlagCommit {
					cs.metrics.ValidatorLastSignedHeight.With(label...).Set(float64(height))
				} else {
					cs.metrics.ValidatorMissedBlocks.With(label...).Add(float64(1))
				}
			}
		}
	}
	cs.metrics.MissingValidators.Set(float64(missingValidators))
	cs.metrics.MissingValidatorsPower.Set(float64(missingValidatorsPower))

	// NOTE: byzantine validators power and count is only for consensus evidence i.e. duplicate vote
	var (
		byzantineValidatorsPower = int64(0)
		byzantineValidatorsCount = int64(0)
	)
	for _, ev := range block.Evidence.Evidence {
		if dve, ok := ev.(*types.DuplicateVoteEvidence); ok {
			if _, val := cs.Validators.GetByAddressMut(dve.VoteA.ValidatorAddress); val != nil {
				byzantineValidatorsCount++
				byzantineValidatorsPower += val.VotingPower
			}
		}
	}
	cs.metrics.ByzantineValidators.Set(float64(byzantineValidatorsCount))
	cs.metrics.ByzantineValidatorsPower.Set(float64(byzantineValidatorsPower))

	if height > 1 {
		lastBlockMeta := cs.blockStore.LoadBlockMeta(height - 1)
		if lastBlockMeta != nil {
			cs.metrics.BlockIntervalSeconds.Observe(
				block.Time.Sub(lastBlockMeta.Header.Time).Seconds(),
			)
		}
	}

	cs.metrics.NumTxs.Set(float64(len(block.Data.Txs)))
	cs.metrics.TotalTxs.Add(float64(len(block.Data.Txs)))
	cs.metrics.BlockSizeBytes.Set(float64(block.Size()))
	cs.metrics.ChainSizeBytes.Add(float64(block.Size()))
	cs.metrics.CommittedHeight.Set(float64(block.Height))
}

// -----------------------------------------------------------------------------

func (cs *State) defaultSetProposal(proposal *types.Proposal, recvTime time.Time) error {
	// Already have one
	// TODO: possibly catch double proposals
	if cs.Proposal != nil || proposal == nil {
		return nil
	}

	// Does not apply
	if proposal.Height != cs.Height || proposal.Round != cs.Round {
		return nil
	}

	// Verify POLRound, which must be -1 or in range [0, proposal.Round).
	if proposal.POLRound < -1 ||
		(proposal.POLRound >= 0 && proposal.POLRound >= proposal.Round) {
		return ErrInvalidProposalPOLRound
	}

	p := proposal.ToProto()
	// Verify signature
	pubKey := cs.Validators.GetProposer().PubKey
	if !pubKey.VerifySignature(
		types.ProposalSignBytes(cs.state.ChainID, p), proposal.Signature,
	) {
		return ErrInvalidProposalSignature
	}

	// Validate the proposed block size, derived from its PartSetHeader
	maxBytes := cs.state.ConsensusParams.Block.MaxBytes
	if maxBytes == -1 {
		maxBytes = int64(types.MaxBlockSizeBytes)
	}
	if int64(proposal.BlockID.PartSetHeader.Total) > (maxBytes-1)/int64(types.BlockPartSizeBytes)+1 {
		return ErrProposalTooManyParts
	}

	proposal.Signature = p.Signature
	cs.Proposal = proposal
	cs.ProposalReceiveTime = recvTime
	cs.calculateProposalTimestampDifferenceMetric()
	// We don't update cs.ProposalBlockParts if it is already set.
	// This happens if we're already in cstypes.RoundStepCommit or if there is a valid block in the current round.
	// TODO: We can check if Proposal is for a different block as this is a sign of misbehavior!
	if cs.ProposalBlockParts == nil {
		cs.ProposalBlockParts = types.NewPartSetFromHeader(proposal.BlockID.PartSetHeader)
	}

	cs.Logger.Info("Received proposal", "proposal", proposal, "proposer", pubKey.Address())
	return nil
}

func (cs *State) readSerializedBlockFromBlockParts() ([]byte, error) {
	// reuse a serialized block buffer from cs
	var serializedBlockBuffer []byte
	if len(cs.serializedBlockBuffer) < int(cs.ProposalBlockParts.ByteSize()) {
		serializedBlockBuffer = make([]byte, cs.ProposalBlockParts.ByteSize())
		cs.serializedBlockBuffer = serializedBlockBuffer
	} else {
		serializedBlockBuffer = cs.serializedBlockBuffer[:cs.ProposalBlockParts.ByteSize()]
	}

	n, err := io.ReadFull(cs.ProposalBlockParts.GetReader(), serializedBlockBuffer)
	if err != nil {
		return nil, err
	}
	// Consistency check, should be impossible to fail.
	if n != len(serializedBlockBuffer) {
		return nil, fmt.Errorf("unexpected error in reading block parts, expected to read %d bytes, read %d", len(serializedBlockBuffer), n)
	}
	return serializedBlockBuffer, nil
}

// NOTE: block is not necessarily valid.
// Asynchronously triggers either enterPrevote (before we timeout of propose) or tryFinalizeCommit,
// once we have the full block.
func (cs *State) addProposalBlockPart(msg *BlockPartMessage, peerID p2p.ID) (added bool, err error) {
	height, round, part := msg.Height, msg.Round, msg.Part

	// Blocks might be reused, so round mismatch is OK
	if cs.Height != height {
		cs.Logger.Debug("Received block part from wrong height", "height", height, "round", round)
		cs.metrics.BlockGossipPartsReceived.With("matches_current", "false").Add(1)
		return false, nil
	}

	// We're not expecting a block part.
	if cs.ProposalBlockParts == nil {
		cs.metrics.BlockGossipPartsReceived.With("matches_current", "false").Add(1)
		// NOTE: this can happen when we've gone to a higher round and
		// then receive parts from the previous round - not necessarily a bad peer.
		cs.Logger.Debug(
			"Received a block part when we are not expecting any",
			"height", height,
			"round", round,
			"index", part.Index,
			"peer", peerID,
		)
		return false, nil
	}

	added, err = cs.ProposalBlockParts.AddPart(part)
	if err != nil {
		if errors.Is(err, types.ErrPartSetInvalidProof) || errors.Is(err, types.ErrPartSetUnexpectedIndex) {
			cs.metrics.BlockGossipPartsReceived.With("matches_current", "false").Add(1)
		}
		return added, err
	}

	cs.metrics.BlockGossipPartsReceived.With("matches_current", "true").Add(1)
	if !added {
		// NOTE: we are disregarding possible duplicates above where heights dont match or we're not expecting block parts yet
		// but between the matches_current = true and false, we have all the info.
		cs.metrics.DuplicateBlockPart.Add(1)
	} else {
		cs.evsw.FireEvent(types.EventProposalBlockPart, msg)
	}

	count, total := cs.ProposalBlockParts.Count(), cs.ProposalBlockParts.Total()
	cs.Logger.Debug("Receive block part", "height", height, "round", round,
		"index", part.Index, "count", count, "total", total, "from", peerID)

	maxBytes := cs.state.ConsensusParams.Block.MaxBytes
	if maxBytes == -1 {
		maxBytes = int64(types.MaxBlockSizeBytes)
	}
	if cs.ProposalBlockParts.ByteSize() > maxBytes {
		return added, fmt.Errorf("total size of proposal block parts exceeds maximum block bytes (%d > %d)",
			cs.ProposalBlockParts.ByteSize(), maxBytes,
		)
	}
	if added && cs.ProposalBlockParts.IsComplete() {
		bz, err := cs.readSerializedBlockFromBlockParts()
		if err != nil {
			return added, err
		}

		pbb := new(cmtproto.Block)
		err = proto.Unmarshal(bz, pbb)
		if err != nil {
			return added, err
		}

		block, err := types.BlockFromProto(pbb)
		if err != nil {
			return added, err
		}

		cs.ProposalBlock = block

		// NOTE: it's possible to receive complete proposal blocks for future rounds without having the proposal
		cs.Logger.Info("Received complete proposal block", "height", cs.ProposalBlock.Height, "hash", cs.ProposalBlock.Hash())

		if err := cs.eventBus.PublishEventCompleteProposal(cs.CompleteProposalEvent()); err != nil {
			cs.Logger.Error("Failed publishing event complete proposal", "err", err)
		}
	}
	return added, nil
}

func (cs *State) handleCompleteProposal(blockHeight int64) {
	// Update Valid* if we can.
	prevotes := cs.Votes.Prevotes(cs.Round)
	blockID, hasTwoThirds := prevotes.TwoThirdsMajority()
	if hasTwoThirds && !blockID.IsNil() && (cs.ValidRound < cs.Round) {
		if cs.ProposalBlock.HashesTo(blockID.Hash) {
			cs.Logger.Debug(
				"Updating valid block to new proposal block",
				"valid_round", cs.Round,
				"valid_block_hash", log.NewLazyBlockHash(cs.ProposalBlock),
			)

			cs.ValidRound = cs.Round
			cs.ValidBlock = cs.ProposalBlock
			cs.ValidBlockParts = cs.ProposalBlockParts
		}
		// TODO: In case there is +2/3 majority in Prevotes set for some
		// block and cs.ProposalBlock contains different block, either
		// proposer is faulty or voting power of faulty processes is more
		// than 1/3. We should trigger in the future accountability
		// procedure at this point.
	}

	if cs.Step <= cstypes.RoundStepPropose && cs.isProposalComplete() {
		// Move onto the next step
		cs.enterPrevote(blockHeight, cs.Round)
		if hasTwoThirds { // this is optimisation as this will be triggered when prevote is added
			cs.enterPrecommit(blockHeight, cs.Round)
		}
	} else if cs.Step == cstypes.RoundStepCommit {
		// If we're waiting on the proposal block...
		cs.tryFinalizeCommit(blockHeight)
	}
}

// Attempt to add the vote. if its a duplicate signature, dupeout the validator.
func (cs *State) tryAddVote(vote *types.Vote, peerID p2p.ID) (bool, error) {
	added, err := cs.addVote(vote, peerID)
	// NOTE: some of these errors are swallowed here
	if err != nil {
		// If the vote height is off, we'll just ignore it,
		// But if it's a conflicting sig, add it to the cs.evpool.
		// If it's otherwise invalid, punish peer.

		if voteErr, ok := err.(*types.ErrVoteConflictingVotes); ok {
			if cs.privValidatorPubKey == nil {
				return false, ErrPubKeyIsNotSet
			}

			if bytes.Equal(vote.ValidatorAddress, cs.privValidatorPubKey.Address()) {
				cs.Logger.Error(
					"Found conflicting vote from ourselves; did you unsafe_reset a validator?",
					"height", vote.Height,
					"round", vote.Round,
					"type", vote.Type,
				)

				return added, err
			}

			// report conflicting votes to the evidence pool
			cs.evpool.ReportConflictingVotes(voteErr.VoteA, voteErr.VoteB)
			cs.Logger.Debug(
				"Found and sent conflicting votes to the evidence pool",
				"vote_a", voteErr.VoteA,
				"vote_b", voteErr.VoteB,
			)

			return added, err
		}

		// Either
		// 1) bad peer OR
		// 2) not a bad peer? this can also err sometimes with "Unexpected step" OR
		// 3) tmkms use with multiple validators connecting to a single tmkms instance
		// 		(https://github.com/tendermint/tendermint/issues/3839).
		return added, ErrAddingVote{Err: err}
	}

	return added, nil
}

func (cs *State) addVote(vote *types.Vote, peerID p2p.ID) (added bool, err error) {
	cs.Logger.Debug(
		"Adding vote",
		"vote_height", vote.Height,
		"vote_type", vote.Type,
		"val_index", vote.ValidatorIndex,
		"cs_height", cs.Height,
		"extLen", len(vote.Extension),
		"extSigLen", len(vote.ExtensionSignature),
	)

	if vote.Height < cs.Height || (vote.Height == cs.Height && vote.Round < cs.Round) {
		cs.metrics.MarkLateVote(vote.Type)
	}

	// A precommit for the previous height?
	// These come in while we wait timeoutCommit
	if vote.Height+1 == cs.Height && vote.Type == types.PrecommitType {
		if cs.Step != cstypes.RoundStepNewHeight {
			// Late precommit at prior height is ignored
			cs.Logger.Debug("Precommit vote came in after commit timeout and has been ignored", "vote", vote)
			return added, err
		}

		added, err = cs.LastCommit.AddVote(vote)
		if !added {
			// If the vote wasn't added but there's no error, its a duplicate vote
			if err == nil {
				cs.metrics.DuplicateVote.Add(1)
			}
			return added, err
		}

		cs.Logger.Debug("Added vote to last precommits", "last_commit", cs.LastCommit.StringShort())
		if err := cs.eventBus.PublishEventVote(types.EventDataVote{Vote: vote}); err != nil {
			return added, err
		}

		cs.evsw.FireEvent(types.EventVote, vote)

		// if we can skip timeoutCommit and have all the votes now,
		skipTimeoutCommit := cs.config.SkipTimeoutCommit || cs.config.TimeoutCommit == 0 //nolint:staticcheck
		if skipTimeoutCommit && cs.LastCommit.HasAll() {
			// go straight to new round (skip timeout commit)
			// cs.scheduleTimeout(time.Duration(0), cs.Height, 0, cstypes.RoundStepNewHeight)
			cs.enterNewRound(cs.Height, 0)
		}

		return added, err
	}

	// Height mismatch is ignored.
	// Not necessarily a bad peer, but not favorable behavior.
	if vote.Height != cs.Height {
		cs.Logger.Debug("Vote ignored and not added", "vote_height", vote.Height, "cs_height", cs.Height, "peer", peerID)
		return added, err
	}

	// Check to see if the chain is configured to extend votes.
	extEnabled := cs.state.ConsensusParams.Feature.VoteExtensionsEnabled(vote.Height)
	if extEnabled {
		// The chain is configured to extend votes, check that the vote is
		// not for a nil block and verify the extensions signature against the
		// corresponding public key.

		var myAddr []byte
		if cs.privValidatorPubKey != nil {
			myAddr = cs.privValidatorPubKey.Address()
		}
		// Verify VoteExtension if precommit and not nil
		// https://github.com/tendermint/tendermint/issues/8487
		if vote.Type == types.PrecommitType && !vote.BlockID.IsNil() &&
			!bytes.Equal(vote.ValidatorAddress, myAddr) { // Skip the VerifyVoteExtension call if the vote was issued by this validator.
			// The core fields of the vote message were already validated in the
			// consensus reactor when the vote was received.
			// Here, we verify the signature of the vote extension included in the vote
			// message.
			_, val := cs.state.Validators.GetByIndex(vote.ValidatorIndex)
			if err := vote.VerifyExtension(cs.state.ChainID, val.PubKey); err != nil {
				return false, err
			}

			err := cs.blockExec.VerifyVoteExtension(context.TODO(), vote)
			cs.metrics.MarkVoteExtensionReceived(err == nil)
			if err != nil {
				return false, err
			}
		}
	} else if len(vote.Extension) > 0 || len(vote.ExtensionSignature) > 0 {
		// Vote extensions are not enabled on the network.
		// Reject the vote, as it is malformed
		//
		// TODO punish a peer if it sent a vote with an extension when the feature
		// is disabled on the network.
		// https://github.com/tendermint/tendermint/issues/8565
		return false, fmt.Errorf("received vote with vote extension for height %v (extensions disabled) from peer ID %s", vote.Height, peerID)
	}

	height := cs.Height
	added, err = cs.Votes.AddVote(vote, peerID, extEnabled)
	if !added {
		// Either duplicate, or error upon cs.Votes.AddByIndex()

		// If the vote wasn't added but there's no error, its a duplicate vote
		if err == nil {
			cs.metrics.DuplicateVote.Add(1)
		}
		return added, err
	}
	if vote.Round == cs.Round {
		vals := cs.state.Validators
		_, val := vals.GetByIndex(vote.ValidatorIndex)
		cs.metrics.MarkVoteReceived(vote.Type, val.VotingPower, vals.TotalVotingPower())
	}

	if err := cs.eventBus.PublishEventVote(types.EventDataVote{Vote: vote}); err != nil {
		return added, err
	}
	cs.evsw.FireEvent(types.EventVote, vote)

	switch vote.Type {
	case types.PrevoteType:
		prevotes := cs.Votes.Prevotes(vote.Round)
		cs.Logger.Debug("Added vote to prevote", "vote", vote, "prevotes", prevotes.StringShort())

		// Check to see if >2/3 of the voting power on the network voted for any non-nil block.
		if blockID, ok := prevotes.TwoThirdsMajority(); ok && !blockID.IsNil() {
			// Greater than 2/3 of the voting power on the network voted for some
			// non-nil block

			// Update Valid* if we can.
			if cs.ValidRound < vote.Round && vote.Round == cs.Round {
				if cs.ProposalBlock.HashesTo(blockID.Hash) {
					cs.Logger.Debug("Updating valid block because of POL", "valid_round", cs.ValidRound, "pol_round", vote.Round)
					cs.ValidRound = vote.Round
					cs.ValidBlock = cs.ProposalBlock
					cs.ValidBlockParts = cs.ProposalBlockParts
				} else {
					cs.Logger.Debug(
						"Valid block we do not know about; set ProposalBlock=nil",
						"proposal", log.NewLazyBlockHash(cs.ProposalBlock),
						"block_id", blockID.Hash,
					)

					// we're getting the wrong block
					cs.ProposalBlock = nil
				}

				if !cs.ProposalBlockParts.HasHeader(blockID.PartSetHeader) {
					cs.ProposalBlockParts = types.NewPartSetFromHeader(blockID.PartSetHeader)
				}

				cs.evsw.FireEvent(types.EventValidBlock, &cs.RoundState)
				if err := cs.eventBus.PublishEventValidBlock(cs.RoundStateEvent()); err != nil {
					return added, err
				}
			}
		}

		// If +2/3 prevotes for *anything* for future round:
		switch {
		case cs.Round < vote.Round && prevotes.HasTwoThirdsAny():
			// Round-skip if there is any 2/3+ of votes ahead of us
			cs.enterNewRound(height, vote.Round)

		case cs.Round == vote.Round && cstypes.RoundStepPrevote <= cs.Step: // current round
			blockID, ok := prevotes.TwoThirdsMajority()
			if ok && (cs.isProposalComplete() || blockID.IsNil()) {
				cs.enterPrecommit(height, vote.Round)
			} else if prevotes.HasTwoThirdsAny() {
				cs.enterPrevoteWait(height, vote.Round)
			}

		case cs.Proposal != nil && 0 <= cs.Proposal.POLRound && cs.Proposal.POLRound == vote.Round:
			// If the proposal is now complete, enter prevote of cs.Round.
			if cs.isProposalComplete() {
				cs.enterPrevote(height, cs.Round)
			}
		}

	case types.PrecommitType:
		precommits := cs.Votes.Precommits(vote.Round)
		cs.Logger.Debug("Added vote to precommit",
			"height", vote.Height,
			"round", vote.Round,
			"validator", vote.ValidatorAddress.String(),
			"vote_timestamp", vote.Timestamp,
			"data", precommits.LogString())

		blockID, ok := precommits.TwoThirdsMajority()
		if ok {
			// Executed as TwoThirdsMajority could be from a higher round
			cs.enterNewRound(height, vote.Round)
			cs.enterPrecommit(height, vote.Round)

			if !blockID.IsNil() {
				cs.enterCommit(height, vote.Round)
				skipTimeoutCommit := cs.config.SkipTimeoutCommit || cs.config.TimeoutCommit == 0 //nolint:staticcheck
				if skipTimeoutCommit && precommits.HasAll() {
					cs.enterNewRound(cs.Height, 0)
				}
			} else {
				cs.enterPrecommitWait(height, vote.Round)
			}
		} else if cs.Round <= vote.Round && precommits.HasTwoThirdsAny() {
			cs.enterNewRound(height, vote.Round)
			cs.enterPrecommitWait(height, vote.Round)
		}

	default:
		panic(fmt.Sprintf("unexpected vote type %v", vote.Type))
	}

	return added, err
}

// CONTRACT: cs.privValidator is not nil.
func (cs *State) signVote(
	msgType types.SignedMsgType,
	hash []byte,
	header types.PartSetHeader,
	block *types.Block,
) (*types.Vote, error) {
	// Flush the WAL. Otherwise, we may not recompute the same vote to sign,
	// and the privValidator will refuse to sign anything.
	if err := cs.wal.FlushAndSync(); err != nil {
		return nil, err
	}

	if cs.privValidatorPubKey == nil {
		return nil, ErrPubKeyIsNotSet
	}

	addr := cs.privValidatorPubKey.Address()
	valIdx, _ := cs.Validators.GetByAddress(addr)
	timestamp := cs.voteTime(cs.Height)

	vote := &types.Vote{
		ValidatorAddress: addr,
		ValidatorIndex:   valIdx,
		Height:           cs.Height,
		Round:            cs.Round,
		Timestamp:        timestamp,
		Type:             msgType,
		BlockID:          types.BlockID{Hash: hash, PartSetHeader: header},
	}

	extEnabled := cs.state.ConsensusParams.Feature.VoteExtensionsEnabled(vote.Height)
	if msgType == types.PrecommitType && !vote.BlockID.IsNil() {
		// if the signedMessage type is for a non-nil precommit, add
		// VoteExtension
		if extEnabled {
			ext, err := cs.blockExec.ExtendVote(context.TODO(), vote, block, cs.state)
			if err != nil {
				return nil, err
			}
			vote.Extension = ext
		}
	}

	recoverable, err := types.SignAndCheckVote(vote, cs.privValidator, cs.state.ChainID, extEnabled && (msgType == types.PrecommitType))
	if err != nil && !recoverable {
		panic(fmt.Sprintf("non-recoverable error when signing vote %v: %v", vote, err))
	}

	return vote, err
}

func (cs *State) voteTime(height int64) time.Time {
	if cs.isPBTSEnabled(height) {
		return cmttime.Now()
	}
	now := cmttime.Now()
	minVoteTime := now

	// Minimum time increment between blocks
	const timeIota = time.Millisecond
	// TODO: We should remove next line in case we don't vote for v in case cs.ProposalBlock == nil,
	// even if cs.LockedBlock != nil. See https://github.com/cometbft/cometbft/tree/main/spec/.
	if cs.LockedBlock != nil {
		// See the BFT time spec
		// https://github.com/cometbft/cometbft/blob/main/spec/consensus/bft-time.md
		minVoteTime = cs.LockedBlock.Time.Add(timeIota)
	} else if cs.ProposalBlock != nil {
		minVoteTime = cs.ProposalBlock.Time.Add(timeIota)
	}

	if now.After(minVoteTime) {
		return now
	}
	return minVoteTime
}

// sign the vote and publish on internalMsgQueue
// block information is only used to extend votes (precommit only); should be nil in all other cases.
func (cs *State) signAddVote(
	msgType types.SignedMsgType,
	hash []byte,
	header types.PartSetHeader,
	block *types.Block,
) {
	if cs.privValidator == nil { // the node does not have a key
		return
	}

	if cs.privValidatorPubKey == nil {
		// Vote won't be signed, but it's not critical.
		cs.Logger.Error(fmt.Sprintf("signAddVote: %v", ErrPubKeyIsNotSet))
		return
	}

	// If the node not in the validator set, do nothing.
	if !cs.Validators.HasAddress(cs.privValidatorPubKey.Address()) {
		return
	}

	// TODO: pass pubKey to signVote
	vote, err := cs.signVote(msgType, hash, header, block)
	if err != nil {
		cs.Logger.Error("Failed signing vote", "height", cs.Height, "round", cs.Round, "vote", vote, "err", err)
		return
	}
	hasExt := len(vote.ExtensionSignature) > 0
	extEnabled := cs.state.ConsensusParams.Feature.VoteExtensionsEnabled(vote.Height)
	if vote.Type == types.PrecommitType && !vote.BlockID.IsNil() && hasExt != extEnabled {
		panic(fmt.Errorf("vote extension absence/presence does not match extensions enabled %t!=%t, height %d, type %v",
			hasExt, extEnabled, vote.Height, vote.Type))
	}
	cs.sendInternalMessage(msgInfo{&VoteMessage{vote}, "", time.Time{}})
	cs.Logger.Debug("Signed and pushed vote", "height", cs.Height, "round", cs.Round, "vote", vote)
}

// updatePrivValidatorPubKey gets the private validator public key and
// memoizes it. This func returns an error if the private validator is not
// responding or responds with an error.
func (cs *State) updatePrivValidatorPubKey() error {
	if cs.privValidator == nil {
		return nil
	}

	pubKey, err := cs.privValidator.GetPubKey()
	if err != nil {
		return err
	}
	cs.privValidatorPubKey = pubKey
	return nil
}

// look back to check existence of the node's consensus votes before joining consensus.
func (cs *State) checkDoubleSigningRisk(height int64) error {
	if cs.privValidator != nil && cs.privValidatorPubKey != nil && cs.config.DoubleSignCheckHeight > 0 && height > 0 {
		valAddr := cs.privValidatorPubKey.Address()
		doubleSignCheckHeight := cs.config.DoubleSignCheckHeight
		if doubleSignCheckHeight > height {
			doubleSignCheckHeight = height
		}

		for i := int64(1); i < doubleSignCheckHeight; i++ {
			lastCommit := cs.blockStore.LoadSeenCommit(height - i)
			if lastCommit != nil {
				for sigIdx, s := range lastCommit.Signatures {
					if s.BlockIDFlag == types.BlockIDFlagCommit && bytes.Equal(s.ValidatorAddress, valAddr) {
						cs.Logger.Info("Found signature from the same key", "sig", s, "idx", sigIdx, "height", height-i)
						return ErrSignatureFoundInPastBlocks
					}
				}
			}
		}
	}

	return nil
}

func (cs *State) calculatePrevoteMessageDelayMetrics() {
	if cs.Proposal == nil {
		return
	}

	ps := cs.Votes.Prevotes(cs.Round)
	pl := ps.List()

	sort.Slice(pl, func(i, j int) bool {
		return pl[i].Timestamp.Before(pl[j].Timestamp)
	})

	var votingPowerSeen int64
	for _, v := range pl {
		_, val := cs.Validators.GetByAddressMut(v.ValidatorAddress)
		votingPowerSeen += val.VotingPower
		if votingPowerSeen >= cs.Validators.TotalVotingPower()*2/3+1 {
			cs.metrics.QuorumPrevoteDelay.With("proposer_address", cs.Validators.GetProposer().Address.String()).Set(v.Timestamp.Sub(cs.Proposal.Timestamp).Seconds())
			break
		}
	}
	if ps.HasAll() {
		cs.metrics.FullPrevoteDelay.With("proposer_address", cs.Validators.GetProposer().Address.String()).Set(pl[len(pl)-1].Timestamp.Sub(cs.Proposal.Timestamp).Seconds())
	}
}

// ---------------------------------------------------------

func CompareHRS(h1 int64, r1 int32, s1 cstypes.RoundStepType, h2 int64, r2 int32, s2 cstypes.RoundStepType) int {
	if h1 < h2 {
		return -1
	} else if h1 > h2 {
		return 1
	}
	if r1 < r2 {
		return -1
	} else if r1 > r2 {
		return 1
	}
	if s1 < s2 {
		return -1
	} else if s1 > s2 {
		return 1
	}
	return 0
}

// repairWalFile decodes messages from src (until the decoder errors) and
// writes them to dst.
func repairWalFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()

	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	defer out.Close()

	var (
		dec = NewWALDecoder(in)
		enc = NewWALEncoder(out)
	)

	// best-case repair (until first error is encountered)
	for {
		msg, err := dec.Decode()
		if err != nil {
			break
		}

		err = enc.Encode(msg)
		if err != nil {
			return fmt.Errorf("failed to encode msg: %w", err)
		}
	}

	return nil
}

func (cs *State) calculateProposalTimestampDifferenceMetric() {
	if cs.Proposal != nil && cs.Proposal.POLRound == -1 {
		sp := cs.state.ConsensusParams.Synchrony.InRound(cs.Proposal.Round)

		isTimely := cs.Proposal.IsTimely(cs.ProposalReceiveTime, sp)
		cs.metrics.ProposalTimestampDifference.
			With("is_timely", strconv.FormatBool(isTimely)).
			Observe(cs.ProposalReceiveTime.Sub(cs.Proposal.Timestamp).Seconds())
	}
}

// proposerWaitTime determines how long the proposer should wait to propose its next block.
// If the result is zero, a block can be proposed immediately.
//
// Block times must be monotonically increasing, so if the block time of the previous
// block is larger than the proposer's current time, then the proposer will sleep
// until its local clock exceeds the previous block time.
func proposerWaitTime(lt cmttime.Source, bt time.Time) time.Duration {
	t := lt.Now()
	if bt.After(t) {
		return bt.Sub(t)
	}
	return 0
}

// isPBTSEnabled returns true if PBTS is enabled at the current height.
func (cs *State) isPBTSEnabled(height int64) bool {
	return cs.state.ConsensusParams.Feature.PbtsEnabled(height)
}
