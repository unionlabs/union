package consensus

import (
	"bytes"
	"errors"
	"fmt"
	"io"
	"os"
	"runtime/debug"
	"sort"
	"time"

	"github.com/cosmos/gogoproto/proto"

	cfg "github.com/cometbft/cometbft/config"
	cstypes "github.com/cometbft/cometbft/consensus/types"
	"github.com/cometbft/cometbft/crypto"
	cmtevents "github.com/cometbft/cometbft/libs/events"
	"github.com/cometbft/cometbft/libs/fail"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	cmtos "github.com/cometbft/cometbft/libs/os"
	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

// Consensus sentinel errors
var (
	ErrInvalidProposalSignature   = errors.New("error invalid proposal signature")
	ErrInvalidProposalPOLRound    = errors.New("error invalid proposal POL round")
	ErrAddingVote                 = errors.New("error adding vote")
	ErrSignatureFoundInPastBlocks = errors.New("found signature from the same key")

	errPubKeyIsNotSet = errors.New("pubkey is not set. Look for \"Can't get private validator pubkey\" errors")
)

var msgQueueSize = 1000

// msgs from the reactor which may update the state
type msgInfo struct {
	Msg    Message `json:"msg"`
	PeerID p2p.ID  `json:"peer_key"`
}

// internally generated messages which may update the state
type timeoutInfo struct {
	Duration time.Duration         `json:"duration"`
	Height   int64                 `json:"height"`
	Round    int32                 `json:"round"`
	Step     cstypes.RoundStepType `json:"step"`
}

func (ti *timeoutInfo) String() string {
	return fmt.Sprintf("%v ; %d/%d %v", ti.Duration, ti.Height, ti.Round, ti.Step)
}

// interface to the mempool
type txNotifier interface {
	TxsAvailable() <-chan struct{}
}

// interface to the evidence pool
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
	setProposal    func(proposal *types.Proposal) error

	// closed when we finish shutting down
	done chan struct{}

	// synchronous pubsub between consensus state and reactor.
	// state only emits EventNewRoundStep and EventVote
	evsw cmtevents.EventSwitch

	// for reporting metrics
	metrics *Metrics
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

	// set function defaults (may be overwritten before calling Start)
	cs.decideProposal = cs.defaultDecideProposal
	cs.doPrevote = cs.defaultDoPrevote
	cs.setProposal = cs.defaultSetProposal

	// We have no votes, so reconstruct LastCommit from SeenCommit.
	if state.LastBlockHeight > 0 {
		cs.reconstructLastCommit(state)
	}

	cs.updateToState(state)

	// NOTE: we do not call scheduleRound0 yet, we do that upon Start()

	cs.BaseService = *service.NewBaseService(nil, "State", cs)
	for _, option := range options {
		option(cs)
	}

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

// String returns a string.
func (cs *State) String() string {
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
func (cs *State) GetRoundState() *cstypes.RoundState {
	cs.mtx.RLock()
	rs := cs.RoundState // copy
	cs.mtx.RUnlock()
	return &rs
}

// GetRoundStateJSON returns a json of RoundState.
func (cs *State) GetRoundStateJSON() ([]byte, error) {
	cs.mtx.RLock()
	defer cs.mtx.RUnlock()
	return cmtjson.Marshal(cs.RoundState)
}

// GetRoundStateSimpleJSON returns a json of RoundStateSimple
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
		cs.Logger.Error("failed to get private validator pubkey", "err", err)
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
				cs.Logger.Error("error on catchup replay; proceeding to start state anyway", "err", err)
				break LOOP

			case repairAttempted:
				return err
			}

			cs.Logger.Error("the WAL file is corrupted; attempting repair", "err", err)

			// 1) prep work
			if err := cs.wal.Stop(); err != nil {
				return err
			}

			repairAttempted = true

			// 2) backup original WAL file
			corruptedFile := fmt.Sprintf("%s.CORRUPTED", cs.config.WalFile())
			if err := cmtos.CopyFile(cs.config.WalFile(), corruptedFile); err != nil {
				return err
			}

			cs.Logger.Debug("backed up WAL file", "src", cs.config.WalFile(), "dst", corruptedFile)

			// 3) try to repair (WAL file will be overwritten!)
			if err := repairWalFile(corruptedFile, cs.config.WalFile()); err != nil {
				cs.Logger.Error("the WAL repair failed", "err", err)
				return err
			}

			cs.Logger.Info("successful WAL repair")

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
// receiveRoutine: serializes processing of proposoals, block parts, votes; coordinates state transitions
func (cs *State) startRoutines(maxSteps int) {
	err := cs.timeoutTicker.Start()
	if err != nil {
		cs.Logger.Error("failed to start timeout ticker", "err", err)
		return
	}

	go cs.receiveRoutine(maxSteps)
}

// loadWalFile loads WAL data from file. It overwrites cs.wal.
func (cs *State) loadWalFile() error {
	wal, err := cs.OpenWAL(cs.config.WalFile())
	if err != nil {
		cs.Logger.Error("failed to load state WAL", "err", err)
		return err
	}

	cs.wal = wal
	return nil
}

// OnStop implements service.Service.
func (cs *State) OnStop() {
	if err := cs.evsw.Stop(); err != nil {
		cs.Logger.Error("failed trying to stop eventSwitch", "error", err)
	}

	if err := cs.timeoutTicker.Stop(); err != nil {
		cs.Logger.Error("failed trying to stop timeoutTicket", "error", err)
	}
	// WAL is stopped in receiveRoutine.
}

// Wait waits for the the main routine to return.
// NOTE: be sure to Stop() the event switch and drain
// any event channels or this may deadlock
func (cs *State) Wait() {
	<-cs.done
}

// OpenWAL opens a file to log all consensus messages and timeouts for
// deterministic accountability.
func (cs *State) OpenWAL(walFile string) (WAL, error) {
	wal, err := NewWAL(walFile)
	if err != nil {
		cs.Logger.Error("failed to open WAL", "file", walFile, "err", err)
		return nil, err
	}

	wal.SetLogger(cs.Logger.With("wal", walFile))

	if err := wal.Start(); err != nil {
		cs.Logger.Error("failed to start WAL", "err", err)
		return nil, err
	}

	return wal, nil
}

//------------------------------------------------------------
// Public interface for passing messages into the consensus state, possibly causing a state transition.
// If peerID == "", the msg is considered internal.
// Messages are added to the appropriate queue (peer or internal).
// If the queue is full, the function may block.
// TODO: should these return anything or let callers just use events?

// AddVote inputs a vote.
func (cs *State) AddVote(vote *types.Vote, peerID p2p.ID) (added bool, err error) {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&VoteMessage{vote}, ""}
	} else {
		cs.peerMsgQueue <- msgInfo{&VoteMessage{vote}, peerID}
	}

	// TODO: wait for event?!
	return false, nil
}

// SetProposal inputs a proposal.
func (cs *State) SetProposal(proposal *types.Proposal, peerID p2p.ID) error {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&ProposalMessage{proposal}, ""}
	} else {
		cs.peerMsgQueue <- msgInfo{&ProposalMessage{proposal}, peerID}
	}

	// TODO: wait for event?!
	return nil
}

// AddProposalBlockPart inputs a part of the proposal block.
func (cs *State) AddProposalBlockPart(height int64, round int32, part *types.Part, peerID p2p.ID) error {
	if peerID == "" {
		cs.internalMsgQueue <- msgInfo{&BlockPartMessage{height, round, part}, ""}
	} else {
		cs.peerMsgQueue <- msgInfo{&BlockPartMessage{height, round, part}, peerID}
	}

	// TODO: wait for event?!
	return nil
}

// SetProposalAndBlock inputs the proposal and all block parts.
func (cs *State) SetProposalAndBlock(
	proposal *types.Proposal,
	block *types.Block,
	parts *types.PartSet,
	peerID p2p.ID,
) error {
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

//------------------------------------------------------------
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

// Attempt to schedule a timeout (by sending timeoutInfo on the tickChan)
func (cs *State) scheduleTimeout(duration time.Duration, height int64, round int32, step cstypes.RoundStepType) {
	cs.timeoutTicker.ScheduleTimeout(timeoutInfo{duration, height, round, step})
}

// send a msg into the receiveRoutine regarding our own proposal, block part, or vote
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

// Reconstruct LastCommit from SeenCommit, which we saved along with the block,
// (which happens even before saving the state)
func (cs *State) reconstructLastCommit(state sm.State) {
	seenCommit := cs.blockStore.LoadSeenCommit(state.LastBlockHeight)
	if seenCommit == nil {
		panic(fmt.Sprintf(
			"failed to reconstruct last commit; seen commit for height %v not found",
			state.LastBlockHeight,
		))
	}

	lastPrecommits := types.CommitToVoteSet(state.ChainID, seenCommit, state.LastValidators)
	if !lastPrecommits.HasTwoThirdsMajority() {
		panic("failed to reconstruct last commit; does not have +2/3 maj")
	}

	cs.LastCommit = lastPrecommits
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
	cs.ProposalBlock = nil
	cs.ProposalBlockParts = nil
	cs.LockedRound = -1
	cs.LockedBlock = nil
	cs.LockedBlockParts = nil
	cs.ValidRound = -1
	cs.ValidBlock = nil
	cs.ValidBlockParts = nil
	cs.Votes = cstypes.NewHeightVoteSet(state.ChainID, height, validators)
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
		cs.Logger.Error("failed writing to WAL", "err", err)
	}

	cs.nSteps++

	// newStep is called by updateToState in NewState before the eventBus is set!
	if cs.eventBus != nil {
		if err := cs.eventBus.PublishEventNewRoundStep(rs); err != nil {
			cs.Logger.Error("failed publishing new round step", "err", err)
		}

		cs.evsw.FireEvent(types.EventNewRoundStep, &cs.RoundState)
	}
}

//-----------------------------------------
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
			cs.Logger.Error("failed trying to stop WAL", "error", err)
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
				cs.Logger.Debug("reached max steps; exiting receive routine")
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
				cs.Logger.Error("failed writing to WAL", "err", err)
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

// state transitions on complete-proposal, 2/3-any, 2/3-one
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
		err = cs.setProposal(msg.Proposal)

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
				"received block part from wrong round",
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
		cs.Logger.Error("unknown msg type", "type", fmt.Sprintf("%T", msg))
		return
	}

	if err != nil {
		cs.Logger.Error(
			"failed to process message",
			"height", cs.Height,
			"round", cs.Round,
			"peer", peerID,
			"msg_type", fmt.Sprintf("%T", msg),
			"err", err,
		)
	}
}

func (cs *State) handleTimeout(ti timeoutInfo, rs cstypes.RoundState) {
	cs.Logger.Debug("received tock", "timeout", ti.Duration, "height", ti.Height, "round", ti.Round, "step", ti.Step)

	// timeouts must be for current height, round, step
	if ti.Height != rs.Height || ti.Round < rs.Round || (ti.Round == rs.Round && ti.Step < rs.Step) {
		cs.Logger.Debug("ignoring tock because we are ahead", "height", rs.Height, "round", rs.Round, "step", rs.Step)
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
			cs.Logger.Error("failed publishing timeout wait", "err", err)
		}

		cs.enterPrecommit(ti.Height, ti.Round)

	case cstypes.RoundStepPrecommitWait:
		if err := cs.eventBus.PublishEventTimeoutWait(cs.RoundStateEvent()); err != nil {
			cs.Logger.Error("failed publishing timeout wait", "err", err)
		}

		cs.enterPrecommit(ti.Height, ti.Round)
		cs.enterNewRound(ti.Height, ti.Round+1)

	default:
		panic(fmt.Sprintf("invalid timeout step: %v", ti.Step))
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

//-----------------------------------------------------------------------------
// State functions
// Used internally by handleTimeout and handleMsg to make state transitions

// Enter: `timeoutNewHeight` by startTime (commitTime+timeoutCommit),
//
//	or, if SkipTimeoutCommit==true, after receiving all precommits from (height,round-1)
//
// Enter: `timeoutPrecommits` after any +2/3 precommits from (height,round-1)
// Enter: +2/3 precommits for nil at (height,round-1)
// Enter: +2/3 prevotes any or +2/3 precommits for block or any from (height, round)
// NOTE: cs.StartTime was already set for height.
func (cs *State) enterNewRound(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cs.Step != cstypes.RoundStepNewHeight) {
		logger.Debug(
			"entering new round with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	if now := cmttime.Now(); cs.StartTime.After(now) {
		logger.Debug("need to set a buffer and log message here for sanity", "start_time", cs.StartTime, "now", now)
	}

	logger.Debug("entering new round", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

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
	if round == 0 {
		// We've already reset these upon new height,
		// and meanwhile we might have received a proposal
		// for round 0.
	} else {
		logger.Debug("resetting proposal info")
		cs.Proposal = nil
		cs.ProposalBlock = nil
		cs.ProposalBlockParts = nil
	}

	cs.Votes.SetRound(cmtmath.SafeAddInt32(round, 1)) // also track next round (round+1) to allow round-skipping
	cs.TriggeredTimeoutPrecommit = false

	if err := cs.eventBus.PublishEventNewRound(cs.NewRoundEvent()); err != nil {
		cs.Logger.Error("failed publishing new round", "err", err)
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
	} else {
		cs.enterPropose(height, round)
	}
}

// needProofBlock returns true on the first height (so the genesis app hash is signed right away)
// and where the last block (height-1) caused the app hash to change
func (cs *State) needProofBlock(height int64) bool {
	if height == cs.state.InitialHeight {
		return true
	}

	lastBlockMeta := cs.blockStore.LoadBlockMeta(height - 1)
	if lastBlockMeta == nil {
		// See https://github.com/cometbft/cometbft/issues/370
		cs.Logger.Info("short-circuited needProofBlock", "height", height, "InitialHeight", cs.state.InitialHeight)
		return true
	}

	return !bytes.Equal(cs.state.AppHash, lastBlockMeta.Header.AppHash)
}

// Enter (CreateEmptyBlocks): from enterNewRound(height,round)
// Enter (CreateEmptyBlocks, CreateEmptyBlocksInterval > 0 ):
//
//	after enterNewRound(height,round), after timeout of CreateEmptyBlocksInterval
//
// Enter (!CreateEmptyBlocks) : after enterNewRound(height,round), once txs are in the mempool
func (cs *State) enterPropose(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPropose <= cs.Step) {
		logger.Debug(
			"entering propose step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	logger.Debug("entering propose step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

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
		logger.Debug("node is not a validator")
		return
	}

	logger.Debug("node is a validator")

	if cs.privValidatorPubKey == nil {
		// If this node is a validator & proposer in the current round, it will
		// miss the opportunity to create a block.
		logger.Error("propose step; empty priv validator public key", "err", errPubKeyIsNotSet)
		return
	}

	address := cs.privValidatorPubKey.Address()

	// if not a validator, we're done
	if !cs.Validators.HasAddress(address) {
		logger.Debug("node is not a validator", "addr", address, "vals", cs.Validators)
		return
	}

	if cs.isProposer(address) {
		logger.Debug("propose step; our turn to propose", "proposer", address)
		cs.decideProposal(height, round)
	} else {
		logger.Debug("propose step; not our turn to propose", "proposer", cs.Validators.GetProposer().Address)
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
		block, err = cs.createProposalBlock()
		if err != nil {
			cs.Logger.Error("unable to create proposal block", "error", err)
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
	proposal := types.NewProposal(height, round, cs.ValidRound, propBlockID)
	p := proposal.ToProto()
	if err := cs.privValidator.SignProposal(cs.state.ChainID, p); err == nil {
		proposal.Signature = p.Signature

		// send proposal and block parts on internal msg queue
		cs.sendInternalMessage(msgInfo{&ProposalMessage{proposal}, ""})

		for i := 0; i < int(blockParts.Total()); i++ {
			part := blockParts.GetPart(i)
			cs.sendInternalMessage(msgInfo{&BlockPartMessage{cs.Height, cs.Round, part}, ""})
		}

		cs.Logger.Debug("signed proposal", "height", height, "round", round, "proposal", proposal)
	} else if !cs.replayMode {
		cs.Logger.Error("propose step; failed signing proposal", "height", height, "round", round, "err", err)
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
func (cs *State) createProposalBlock() (*types.Block, error) {
	if cs.privValidator == nil {
		return nil, errors.New("entered createProposalBlock with privValidator being nil")
	}

	var commit *types.Commit
	switch {
	case cs.Height == cs.state.InitialHeight:
		// We're creating a proposal for the first block.
		// The commit is empty, but not nil.
		commit = types.NewCommit(0, 0, types.BlockID{}, nil)

	case cs.LastCommit.HasTwoThirdsMajority():
		// Make the commit from LastCommit
		commit = cs.LastCommit.MakeCommit()

	default: // This shouldn't happen.
		return nil, errors.New("propose step; cannot propose anything without commit for the previous block")
	}

	if cs.privValidatorPubKey == nil {
		// If this node is a validator & proposer in the current round, it will
		// miss the opportunity to create a block.
		return nil, fmt.Errorf("propose step; empty priv validator public key, error: %w", errPubKeyIsNotSet)
	}

	proposerAddr := cs.privValidatorPubKey.Address()

	ret, err := cs.blockExec.CreateProposalBlock(cs.Height, cs.state, commit, proposerAddr)
	if err != nil {
		panic(err)
	}
	return ret, nil
}

// Enter: `timeoutPropose` after entering Propose.
// Enter: proposal block and POL is ready.
// Prevote for LockedBlock if we're locked, or ProposalBlock if valid.
// Otherwise vote nil.
func (cs *State) enterPrevote(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrevote <= cs.Step) {
		logger.Debug(
			"entering prevote step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	defer func() {
		// Done enterPrevote:
		cs.updateRoundStep(round, cstypes.RoundStepPrevote)
		cs.newStep()
	}()

	logger.Debug("entering prevote step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	// Sign and broadcast vote as necessary
	cs.doPrevote(height, round)

	// Once `addVote` hits any +2/3 prevotes, we will go to PrevoteWait
	// (so we have more time to try and collect +2/3 prevotes for a single block)
}

func (cs *State) defaultDoPrevote(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	// If a block is locked, prevote that.
	if cs.LockedBlock != nil {
		logger.Debug("prevote step; already locked on a block; prevoting locked block")
		cs.signAddVote(cmtproto.PrevoteType, cs.LockedBlock.Hash(), cs.LockedBlockParts.Header())
		return
	}

	// If ProposalBlock is nil, prevote nil.
	if cs.ProposalBlock == nil {
		logger.Debug("prevote step: ProposalBlock is nil")
		cs.signAddVote(cmtproto.PrevoteType, nil, types.PartSetHeader{})
		return
	}

	// Validate proposal block, from consensus' perspective
	err := cs.blockExec.ValidateBlock(cs.state, cs.ProposalBlock)
	if err != nil {
		// ProposalBlock is invalid, prevote nil.
		logger.Error("prevote step: consensus deems this block invalid; prevoting nil",
			"err", err)
		cs.signAddVote(cmtproto.PrevoteType, nil, types.PartSetHeader{})
		return
	}

	/*
		Before prevoting on the block received from the proposer for the current round and height,
		we request the Application, via `ProcessProposal` ABCI call, to confirm that the block is
		valid. If the Application does not accept the block, consensus prevotes `nil`.

		WARNING: misuse of block rejection by the Application can seriously compromise
		the liveness properties of consensus.
		Please see `PrepareProosal`-`ProcessProposal` coherence and determinism properties
		in the ABCI++ specification.
	*/
	isAppValid, err := cs.blockExec.ProcessProposal(cs.ProposalBlock, cs.state)
	if err != nil {
		panic(fmt.Sprintf(
			"state machine returned an error (%v) when calling ProcessProposal", err,
		))
	}
	cs.metrics.MarkProposalProcessed(isAppValid)

	// Vote nil if the Application rejected the block
	if !isAppValid {
		logger.Error("prevote step: state machine rejected a proposed block; this should not happen:"+
			"the proposer may be misbehaving; prevoting nil", "err", err)
		cs.signAddVote(cmtproto.PrevoteType, nil, types.PartSetHeader{})
		return
	}

	// Prevote cs.ProposalBlock
	// NOTE: the proposal signature is validated when it is received,
	// and the proposal block parts are validated as they are received (against the merkle hash in the proposal)
	logger.Debug("prevote step: ProposalBlock is valid")
	cs.signAddVote(cmtproto.PrevoteType, cs.ProposalBlock.Hash(), cs.ProposalBlockParts.Header())
}

// Enter: any +2/3 prevotes at next round.
func (cs *State) enterPrevoteWait(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrevoteWait <= cs.Step) {
		logger.Debug(
			"entering prevote wait step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	if !cs.Votes.Prevotes(round).HasTwoThirdsAny() {
		panic(fmt.Sprintf(
			"entering prevote wait step (%v/%v), but prevotes does not have any +2/3 votes",
			height, round,
		))
	}

	logger.Debug("entering prevote wait step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

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
// else, unlock an existing lock and precommit nil if +2/3 of prevotes were nil,
// else, precommit nil otherwise.
func (cs *State) enterPrecommit(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cstypes.RoundStepPrecommit <= cs.Step) {
		logger.Debug(
			"entering precommit step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	logger.Debug("entering precommit step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

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
			logger.Debug("precommit step; no +2/3 prevotes during enterPrecommit while we are locked; precommitting nil")
		} else {
			logger.Debug("precommit step; no +2/3 prevotes during enterPrecommit; precommitting nil")
		}

		cs.signAddVote(cmtproto.PrecommitType, nil, types.PartSetHeader{})
		return
	}

	// At this point +2/3 prevoted for a particular block or nil.
	if err := cs.eventBus.PublishEventPolka(cs.RoundStateEvent()); err != nil {
		logger.Error("failed publishing polka", "err", err)
	}

	// the latest POLRound should be this round.
	polRound, _ := cs.Votes.POLInfo()
	if polRound < round {
		panic(fmt.Sprintf("this POLRound should be %v but got %v", round, polRound))
	}

	// +2/3 prevoted nil. Unlock and precommit nil.
	if len(blockID.Hash) == 0 {
		if cs.LockedBlock == nil {
			logger.Debug("precommit step; +2/3 prevoted for nil")
		} else {
			logger.Debug("precommit step; +2/3 prevoted for nil; unlocking")
			cs.LockedRound = -1
			cs.LockedBlock = nil
			cs.LockedBlockParts = nil

			if err := cs.eventBus.PublishEventUnlock(cs.RoundStateEvent()); err != nil {
				logger.Error("failed publishing event unlock", "err", err)
			}
		}

		cs.signAddVote(cmtproto.PrecommitType, nil, types.PartSetHeader{})
		return
	}

	// At this point, +2/3 prevoted for a particular block.

	// If we're already locked on that block, precommit it, and update the LockedRound
	if cs.LockedBlock.HashesTo(blockID.Hash) {
		logger.Debug("precommit step; +2/3 prevoted locked block; relocking")
		cs.LockedRound = round

		if err := cs.eventBus.PublishEventRelock(cs.RoundStateEvent()); err != nil {
			logger.Error("failed publishing event relock", "err", err)
		}

		cs.signAddVote(cmtproto.PrecommitType, blockID.Hash, blockID.PartSetHeader)
		return
	}

	// If +2/3 prevoted for proposal block, stage and precommit it
	if cs.ProposalBlock.HashesTo(blockID.Hash) {
		logger.Debug("precommit step; +2/3 prevoted proposal block; locking", "hash", blockID.Hash)

		// Validate the block.
		if err := cs.blockExec.ValidateBlock(cs.state, cs.ProposalBlock); err != nil {
			panic(fmt.Sprintf("precommit step; +2/3 prevoted for an invalid block: %v", err))
		}

		cs.LockedRound = round
		cs.LockedBlock = cs.ProposalBlock
		cs.LockedBlockParts = cs.ProposalBlockParts

		if err := cs.eventBus.PublishEventLock(cs.RoundStateEvent()); err != nil {
			logger.Error("failed publishing event lock", "err", err)
		}

		cs.signAddVote(cmtproto.PrecommitType, blockID.Hash, blockID.PartSetHeader)
		return
	}

	// There was a polka in this round for a block we don't have.
	// Fetch that block, unlock, and precommit nil.
	// The +2/3 prevotes for this round is the POL for our unlock.
	logger.Debug("precommit step; +2/3 prevotes for a block we do not have; voting nil", "block_id", blockID)

	cs.LockedRound = -1
	cs.LockedBlock = nil
	cs.LockedBlockParts = nil

	if !cs.ProposalBlockParts.HasHeader(blockID.PartSetHeader) {
		cs.ProposalBlock = nil
		cs.ProposalBlockParts = types.NewPartSetFromHeader(blockID.PartSetHeader)
	}

	if err := cs.eventBus.PublishEventUnlock(cs.RoundStateEvent()); err != nil {
		logger.Error("failed publishing event unlock", "err", err)
	}

	cs.signAddVote(cmtproto.PrecommitType, nil, types.PartSetHeader{})
}

// Enter: any +2/3 precommits for next round.
func (cs *State) enterPrecommitWait(height int64, round int32) {
	logger := cs.Logger.With("height", height, "round", round)

	if cs.Height != height || round < cs.Round || (cs.Round == round && cs.TriggeredTimeoutPrecommit) {
		logger.Debug(
			"entering precommit wait step with invalid args",
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

	logger.Debug("entering precommit wait step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

	defer func() {
		// Done enterPrecommitWait:
		cs.TriggeredTimeoutPrecommit = true
		cs.newStep()
	}()

	// wait for some more precommits; enterNewRound
	cs.scheduleTimeout(cs.config.Precommit(round), height, round, cstypes.RoundStepPrecommitWait)
}

// Enter: +2/3 precommits for block
func (cs *State) enterCommit(height int64, commitRound int32) {
	logger := cs.Logger.With("height", height, "commit_round", commitRound)

	if cs.Height != height || cstypes.RoundStepCommit <= cs.Step {
		logger.Debug(
			"entering commit step with invalid args",
			"current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step),
		)
		return
	}

	logger.Debug("entering commit step", "current", log.NewLazySprintf("%v/%v/%v", cs.Height, cs.Round, cs.Step))

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
	if !ok {
		panic("RunActionCommit() expects +2/3 precommits")
	}

	// The Locked* fields no longer matter.
	// Move them over to ProposalBlock if they match the commit hash,
	// otherwise they'll be cleared in updateToState.
	if cs.LockedBlock.HashesTo(blockID.Hash) {
		logger.Debug("commit is for a locked block; set ProposalBlock=LockedBlock", "block_hash", blockID.Hash)
		cs.ProposalBlock = cs.LockedBlock
		cs.ProposalBlockParts = cs.LockedBlockParts
	}

	// If we don't have the block being committed, set up to get it.
	if !cs.ProposalBlock.HashesTo(blockID.Hash) {
		if !cs.ProposalBlockParts.HasHeader(blockID.PartSetHeader) {
			logger.Info(
				"commit is for a block we do not know about; set ProposalBlock=nil",
				"proposal", log.NewLazyBlockHash(cs.ProposalBlock),
				"commit", blockID.Hash,
			)

			// We're getting the wrong block.
			// Set up ProposalBlockParts and keep waiting.
			cs.ProposalBlock = nil
			cs.ProposalBlockParts = types.NewPartSetFromHeader(blockID.PartSetHeader)

			if err := cs.eventBus.PublishEventValidBlock(cs.RoundStateEvent()); err != nil {
				logger.Error("failed publishing valid block", "err", err)
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
	if !ok || len(blockID.Hash) == 0 {
		logger.Error("failed attempt to finalize commit; there was no +2/3 majority or +2/3 was for nil")
		return
	}

	if !cs.ProposalBlock.HashesTo(blockID.Hash) {
		// TODO: this happens every time if we're not a validator (ugly logs)
		// TODO: ^^ wait, why does it matter that we're a validator?
		logger.Debug(
			"failed attempt to finalize commit; we do not have the commit block",
			"proposal_block", log.NewLazyBlockHash(cs.ProposalBlock),
			"commit_block", blockID.Hash,
		)
		return
	}

	cs.finalizeCommit(height)
}

// Increment height and goto cstypes.RoundStepNewHeight
func (cs *State) finalizeCommit(height int64) {
	logger := cs.Logger.With("height", height)

	if cs.Height != height || cs.Step != cstypes.RoundStepCommit {
		logger.Debug(
			"entering finalize commit step",
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
		"finalizing commit of block",
		"hash", log.NewLazyBlockHash(block),
		"root", block.AppHash,
		"num_txs", len(block.Txs),
	)
	logger.Debug("committed block", "block", log.NewLazySprintf("%v", block))

	fail.Fail() // XXX

	// Save to blockStore.
	if cs.blockStore.Height() < block.Height {
		// NOTE: the seenCommit is local justification to commit this block,
		// but may differ from the LastCommit included in the next block
		precommits := cs.Votes.Precommits(cs.CommitRound)
		seenCommit := precommits.MakeCommit()
		cs.blockStore.SaveBlock(block, blockParts, seenCommit)
	} else {
		// Happens during replay if we already saved the block but didn't commit
		logger.Debug("calling finalizeCommit on already stored block", "height", block.Height)
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
	// NOTE The block.AppHash wont reflect these txs until the next block.
	var (
		err          error
		retainHeight int64
	)

	stateCopy, retainHeight, err = cs.blockExec.ApplyBlock(
		stateCopy,
		types.BlockID{
			Hash:          block.Hash(),
			PartSetHeader: blockParts.Header(),
		},
		block,
	)
	if err != nil {
		panic(fmt.Sprintf("failed to apply block; error %v", err))
	}

	fail.Fail() // XXX

	// Prune old heights, if requested by ABCI app.
	if retainHeight > 0 {
		pruned, err := cs.pruneBlocks(retainHeight)
		if err != nil {
			logger.Error("failed to prune blocks", "retain_height", retainHeight, "err", err)
		} else {
			logger.Debug("pruned blocks", "pruned", pruned, "retain_height", retainHeight)
		}
	}

	// must be called before we update state
	cs.recordMetrics(height, block)

	// NewHeightStep!
	cs.updateToState(stateCopy)

	fail.Fail() // XXX

	// Private validator might have changed it's key pair => refetch pubkey.
	if err := cs.updatePrivValidatorPubKey(); err != nil {
		logger.Error("failed to get private validator pubkey", "err", err)
	}

	// cs.StartTime is already set.
	// Schedule Round0 to start soon.
	cs.scheduleRound0(&cs.RoundState)

	// By here,
	// * cs.Height has been increment to height+1
	// * cs.Step is now cstypes.RoundStepNewHeight
	// * cs.StartTime is set to when we will start round0.
}

func (cs *State) pruneBlocks(retainHeight int64) (uint64, error) {
	base := cs.blockStore.Base()
	if retainHeight <= base {
		return 0, nil
	}
	pruned, err := cs.blockStore.PruneBlocks(retainHeight)
	if err != nil {
		return 0, fmt.Errorf("failed to prune block store: %w", err)
	}
	err = cs.blockExec.Store().PruneStates(base, retainHeight)
	if err != nil {
		return 0, fmt.Errorf("failed to prune state database: %w", err)
	}
	return pruned, nil
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
				cs.Logger.Error(fmt.Sprintf("recordMetrics: %v", errPubKeyIsNotSet))
			} else {
				address = cs.privValidatorPubKey.Address()
			}
		}

		for i, val := range cs.LastValidators.Validators {
			commitSig := block.LastCommit.Signatures[i]
			if commitSig.Absent() {
				missingValidators++
				missingValidatorsPower += val.VotingPower
			}

			if bytes.Equal(val.Address, address) {
				label := []string{
					"validator_address", val.Address.String(),
				}
				cs.metrics.ValidatorPower.With(label...).Set(float64(val.VotingPower))
				if commitSig.ForBlock() {
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
			if _, val := cs.Validators.GetByAddress(dve.VoteA.ValidatorAddress); val != nil {
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
	cs.metrics.CommittedHeight.Set(float64(block.Height))
}

//-----------------------------------------------------------------------------

func (cs *State) defaultSetProposal(proposal *types.Proposal) error {
	// Already have one
	// TODO: possibly catch double proposals
	if cs.Proposal != nil {
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
	if !cs.Validators.GetProposer().PubKey.VerifySignature(
		types.ProposalSignBytes(cs.state.ChainID, p), proposal.Signature,
	) {
		return ErrInvalidProposalSignature
	}

	proposal.Signature = p.Signature
	cs.Proposal = proposal
	// We don't update cs.ProposalBlockParts if it is already set.
	// This happens if we're already in cstypes.RoundStepCommit or if there is a valid block in the current round.
	// TODO: We can check if Proposal is for a different block as this is a sign of misbehavior!
	if cs.ProposalBlockParts == nil {
		cs.ProposalBlockParts = types.NewPartSetFromHeader(proposal.BlockID.PartSetHeader)
	}

	cs.Logger.Info("received proposal", "proposal", proposal)
	return nil
}

// NOTE: block is not necessarily valid.
// Asynchronously triggers either enterPrevote (before we timeout of propose) or tryFinalizeCommit,
// once we have the full block.
func (cs *State) addProposalBlockPart(msg *BlockPartMessage, peerID p2p.ID) (added bool, err error) {
	height, round, part := msg.Height, msg.Round, msg.Part

	// Blocks might be reused, so round mismatch is OK
	if cs.Height != height {
		cs.Logger.Debug("received block part from wrong height", "height", height, "round", round)
		cs.metrics.BlockGossipPartsReceived.With("matches_current", "false").Add(1)
		return false, nil
	}

	// We're not expecting a block part.
	if cs.ProposalBlockParts == nil {
		cs.metrics.BlockGossipPartsReceived.With("matches_current", "false").Add(1)
		// NOTE: this can happen when we've gone to a higher round and
		// then receive parts from the previous round - not necessarily a bad peer.
		cs.Logger.Debug(
			"received a block part when we are not expecting any",
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

	if cs.ProposalBlockParts.ByteSize() > cs.state.ConsensusParams.Block.MaxBytes {
		return added, fmt.Errorf("total size of proposal block parts exceeds maximum block bytes (%d > %d)",
			cs.ProposalBlockParts.ByteSize(), cs.state.ConsensusParams.Block.MaxBytes,
		)
	}
	if added && cs.ProposalBlockParts.IsComplete() {
		bz, err := io.ReadAll(cs.ProposalBlockParts.GetReader())
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
		cs.Logger.Info("received complete proposal block", "height", cs.ProposalBlock.Height, "hash", cs.ProposalBlock.Hash())

		if err := cs.eventBus.PublishEventCompleteProposal(cs.CompleteProposalEvent()); err != nil {
			cs.Logger.Error("failed publishing event complete proposal", "err", err)
		}
	}
	return added, nil
}

func (cs *State) handleCompleteProposal(blockHeight int64) {
	// Update Valid* if we can.
	prevotes := cs.Votes.Prevotes(cs.Round)
	blockID, hasTwoThirds := prevotes.TwoThirdsMajority()
	if hasTwoThirds && !blockID.IsZero() && (cs.ValidRound < cs.Round) {
		if cs.ProposalBlock.HashesTo(blockID.Hash) {
			cs.Logger.Debug(
				"updating valid block to new proposal block",
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

// Attempt to add the vote. if its a duplicate signature, dupeout the validator
func (cs *State) tryAddVote(vote *types.Vote, peerID p2p.ID) (bool, error) {
	added, err := cs.addVote(vote, peerID)
	if err != nil {
		// If the vote height is off, we'll just ignore it,
		// But if it's a conflicting sig, add it to the cs.evpool.
		// If it's otherwise invalid, punish peer.
		//nolint: gocritic
		if voteErr, ok := err.(*types.ErrVoteConflictingVotes); ok {
			if cs.privValidatorPubKey == nil {
				return false, errPubKeyIsNotSet
			}

			if bytes.Equal(vote.ValidatorAddress, cs.privValidatorPubKey.Address()) {
				cs.Logger.Error(
					"found conflicting vote from ourselves; did you unsafe_reset a validator?",
					"height", vote.Height,
					"round", vote.Round,
					"type", vote.Type,
				)

				return added, err
			}

			// report conflicting votes to the evidence pool
			cs.evpool.ReportConflictingVotes(voteErr.VoteA, voteErr.VoteB)
			cs.Logger.Debug(
				"found and sent conflicting votes to the evidence pool",
				"vote_a", voteErr.VoteA,
				"vote_b", voteErr.VoteB,
			)

			return added, err
		} else if errors.Is(err, types.ErrVoteNonDeterministicSignature) {
			cs.Logger.Debug("vote has non-deterministic signature", "err", err)
		} else {
			// Either
			// 1) bad peer OR
			// 2) not a bad peer? this can also err sometimes with "Unexpected step" OR
			// 3) tmkms use with multiple validators connecting to a single tmkms instance
			// 		(https://github.com/tendermint/tendermint/issues/3839).
			cs.Logger.Info("failed attempting to add vote", "err", err)
			return added, ErrAddingVote
		}
	}

	return added, nil
}

func (cs *State) addVote(vote *types.Vote, peerID p2p.ID) (added bool, err error) {
	cs.Logger.Debug(
		"adding vote",
		"vote_height", vote.Height,
		"vote_type", vote.Type,
		"val_index", vote.ValidatorIndex,
		"cs_height", cs.Height,
	)

	if vote.Height < cs.Height || (vote.Height == cs.Height && vote.Round < cs.Round) {
		cs.metrics.MarkLateVote(vote.Type)
	}

	// A precommit for the previous height?
	// These come in while we wait timeoutCommit
	if vote.Height+1 == cs.Height && vote.Type == cmtproto.PrecommitType {
		if cs.Step != cstypes.RoundStepNewHeight {
			// Late precommit at prior height is ignored
			cs.Logger.Debug("precommit vote came in after commit timeout and has been ignored", "vote", vote)
			return
		}

		added, err = cs.LastCommit.AddVote(vote)
		if !added {
			return
		}

		cs.Logger.Debug("added vote to last precommits", "last_commit", cs.LastCommit.StringShort())
		if err := cs.eventBus.PublishEventVote(types.EventDataVote{Vote: vote}); err != nil {
			return added, err
		}

		cs.evsw.FireEvent(types.EventVote, vote)

		// if we can skip timeoutCommit and have all the votes now,
		if cs.config.SkipTimeoutCommit && cs.LastCommit.HasAll() {
			// go straight to new round (skip timeout commit)
			// cs.scheduleTimeout(time.Duration(0), cs.Height, 0, cstypes.RoundStepNewHeight)
			cs.enterNewRound(cs.Height, 0)
		}

		return
	}

	// Height mismatch is ignored.
	// Not necessarily a bad peer, but not favorable behavior.
	if vote.Height != cs.Height {
		cs.Logger.Debug("vote ignored and not added", "vote_height", vote.Height, "cs_height", cs.Height, "peer", peerID)
		return
	}

	height := cs.Height
	added, err = cs.Votes.AddVote(vote, peerID)
	if !added {
		// Either duplicate, or error upon cs.Votes.AddByIndex()
		return
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
	case cmtproto.PrevoteType:
		prevotes := cs.Votes.Prevotes(vote.Round)
		cs.Logger.Debug("added vote to prevote", "vote", vote, "prevotes", prevotes.StringShort())

		// If +2/3 prevotes for a block or nil for *any* round:
		if blockID, ok := prevotes.TwoThirdsMajority(); ok {
			// There was a polka!
			// If we're locked but this is a recent polka, unlock.
			// If it matches our ProposalBlock, update the ValidBlock

			// Unlock if `cs.LockedRound < vote.Round <= cs.Round`
			// NOTE: If vote.Round > cs.Round, we'll deal with it when we get to vote.Round
			if (cs.LockedBlock != nil) &&
				(cs.LockedRound < vote.Round) &&
				(vote.Round <= cs.Round) &&
				!cs.LockedBlock.HashesTo(blockID.Hash) {

				cs.Logger.Debug("unlocking because of POL", "locked_round", cs.LockedRound, "pol_round", vote.Round)

				cs.LockedRound = -1
				cs.LockedBlock = nil
				cs.LockedBlockParts = nil

				if err := cs.eventBus.PublishEventUnlock(cs.RoundStateEvent()); err != nil {
					return added, err
				}
			}

			// Update Valid* if we can.
			// NOTE: our proposal block may be nil or not what received a polka..
			if len(blockID.Hash) != 0 && (cs.ValidRound < vote.Round) && (vote.Round == cs.Round) {
				if cs.ProposalBlock.HashesTo(blockID.Hash) {
					cs.Logger.Debug("updating valid block because of POL", "valid_round", cs.ValidRound, "pol_round", vote.Round)
					cs.ValidRound = vote.Round
					cs.ValidBlock = cs.ProposalBlock
					cs.ValidBlockParts = cs.ProposalBlockParts
				} else {
					cs.Logger.Debug(
						"valid block we do not know about; set ProposalBlock=nil",
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
			if ok && (cs.isProposalComplete() || len(blockID.Hash) == 0) {
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

	case cmtproto.PrecommitType:
		precommits := cs.Votes.Precommits(vote.Round)
		cs.Logger.Debug("added vote to precommit",
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

			if len(blockID.Hash) != 0 {
				cs.enterCommit(height, vote.Round)
				if cs.config.SkipTimeoutCommit && precommits.HasAll() {
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
	msgType cmtproto.SignedMsgType,
	hash []byte,
	header types.PartSetHeader,
) (*types.Vote, error) {
	// Flush the WAL. Otherwise, we may not recompute the same vote to sign,
	// and the privValidator will refuse to sign anything.
	if err := cs.wal.FlushAndSync(); err != nil {
		return nil, err
	}

	if cs.privValidatorPubKey == nil {
		return nil, errPubKeyIsNotSet
	}

	addr := cs.privValidatorPubKey.Address()
	valIdx, _ := cs.Validators.GetByAddress(addr)

	vote := &types.Vote{
		ValidatorAddress: addr,
		ValidatorIndex:   valIdx,
		Height:           cs.Height,
		Round:            cs.Round,
		Timestamp:        cs.voteTime(),
		Type:             msgType,
		BlockID:          types.BlockID{Hash: hash, PartSetHeader: header},
	}

	v := vote.ToProto()
	err := cs.privValidator.SignVote(cs.state.ChainID, v)
	vote.Signature = v.Signature
	vote.Timestamp = v.Timestamp

	return vote, err
}

func (cs *State) voteTime() time.Time {
	now := cmttime.Now()
	minVoteTime := now
	// Minimum time increment between blocks
	const timeIota = time.Millisecond
	// TODO: We should remove next line in case we don't vote for v in case cs.ProposalBlock == nil,
	// even if cs.LockedBlock != nil. See https://github.com/cometbft/cometbft/tree/v0.37.x/spec/.
	if cs.LockedBlock != nil {
		// See the BFT time spec
		// https://github.com/cometbft/cometbft/blob/v0.37.x/spec/consensus/bft-time.md
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
func (cs *State) signAddVote(msgType cmtproto.SignedMsgType, hash []byte, header types.PartSetHeader) *types.Vote {
	if cs.privValidator == nil { // the node does not have a key
		return nil
	}

	if cs.privValidatorPubKey == nil {
		// Vote won't be signed, but it's not critical.
		cs.Logger.Error(fmt.Sprintf("signAddVote: %v", errPubKeyIsNotSet))
		return nil
	}

	// If the node not in the validator set, do nothing.
	if !cs.Validators.HasAddress(cs.privValidatorPubKey.Address()) {
		return nil
	}

	// TODO: pass pubKey to signVote
	vote, err := cs.signVote(msgType, hash, header)
	if err == nil {
		cs.sendInternalMessage(msgInfo{&VoteMessage{vote}, ""})
		cs.Logger.Debug("signed and pushed vote", "height", cs.Height, "round", cs.Round, "vote", vote)
		return vote
	}

	cs.Logger.Error("failed signing vote", "height", cs.Height, "round", cs.Round, "vote", vote, "err", err)
	return nil
}

// updatePrivValidatorPubKey get's the private validator public key and
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

// look back to check existence of the node's consensus votes before joining consensus
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
						cs.Logger.Info("found signature from the same key", "sig", s, "idx", sigIdx, "height", height-i)
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
		_, val := cs.Validators.GetByAddress(v.ValidatorAddress)
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

//---------------------------------------------------------

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
