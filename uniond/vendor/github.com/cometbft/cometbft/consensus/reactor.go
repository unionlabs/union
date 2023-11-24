package consensus

import (
	"errors"
	"fmt"
	"reflect"
	"sync"
	"time"

	cstypes "github.com/cometbft/cometbft/consensus/types"
	"github.com/cometbft/cometbft/libs/bits"
	cmtevents "github.com/cometbft/cometbft/libs/events"
	cmtjson "github.com/cometbft/cometbft/libs/json"
	"github.com/cometbft/cometbft/libs/log"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	cmtcons "github.com/cometbft/cometbft/proto/tendermint/consensus"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
	cmttime "github.com/cometbft/cometbft/types/time"
)

const (
	StateChannel       = byte(0x20)
	DataChannel        = byte(0x21)
	VoteChannel        = byte(0x22)
	VoteSetBitsChannel = byte(0x23)

	maxMsgSize = 1048576 // 1MB; NOTE/TODO: keep in sync with types.PartSet sizes.

	blocksToContributeToBecomeGoodPeer = 10000
	votesToContributeToBecomeGoodPeer  = 10000
)

//-----------------------------------------------------------------------------

// Reactor defines a reactor for the consensus service.
type Reactor struct {
	p2p.BaseReactor // BaseService + p2p.Switch

	conS *State

	mtx      cmtsync.RWMutex
	waitSync bool
	eventBus *types.EventBus
	rs       *cstypes.RoundState

	Metrics *Metrics
}

type ReactorOption func(*Reactor)

// NewReactor returns a new Reactor with the given
// consensusState.
func NewReactor(consensusState *State, waitSync bool, options ...ReactorOption) *Reactor {
	conR := &Reactor{
		conS:     consensusState,
		waitSync: waitSync,
		rs:       consensusState.GetRoundState(),
		Metrics:  NopMetrics(),
	}
	conR.BaseReactor = *p2p.NewBaseReactor("Consensus", conR)

	for _, option := range options {
		option(conR)
	}

	return conR
}

// OnStart implements BaseService by subscribing to events, which later will be
// broadcasted to other peers and starting state if we're not in block sync.
func (conR *Reactor) OnStart() error {
	conR.Logger.Info("Reactor ", "waitSync", conR.WaitSync())

	// start routine that computes peer statistics for evaluating peer quality
	go conR.peerStatsRoutine()

	conR.subscribeToBroadcastEvents()
	go conR.updateRoundStateRoutine()

	if !conR.WaitSync() {
		err := conR.conS.Start()
		if err != nil {
			return err
		}
	}

	return nil
}

// OnStop implements BaseService by unsubscribing from events and stopping
// state.
func (conR *Reactor) OnStop() {
	conR.unsubscribeFromBroadcastEvents()
	if err := conR.conS.Stop(); err != nil {
		conR.Logger.Error("Error stopping consensus state", "err", err)
	}
	if !conR.WaitSync() {
		conR.conS.Wait()
	}
}

// SwitchToConsensus switches from block_sync mode to consensus mode.
// It resets the state, turns off block_sync, and starts the consensus state-machine
func (conR *Reactor) SwitchToConsensus(state sm.State, skipWAL bool) {
	conR.Logger.Info("SwitchToConsensus")

	func() {
		// We need to lock, as we are not entering consensus state from State's `handleMsg` or `handleTimeout`
		conR.conS.mtx.Lock()
		defer conR.conS.mtx.Unlock()
		// We have no votes, so reconstruct LastCommit from SeenCommit
		if state.LastBlockHeight > 0 {
			conR.conS.reconstructLastCommit(state)
		}

		// NOTE: The line below causes broadcastNewRoundStepRoutine() to broadcast a
		// NewRoundStepMessage.
		conR.conS.updateToState(state)
	}()

	conR.mtx.Lock()
	conR.waitSync = false
	conR.mtx.Unlock()

	if skipWAL {
		conR.conS.doWALCatchup = false
	}
	err := conR.conS.Start()
	if err != nil {
		panic(fmt.Sprintf(`Failed to start consensus state: %v

conS:
%+v

conR:
%+v`, err, conR.conS, conR))
	}
}

// GetChannels implements Reactor
func (conR *Reactor) GetChannels() []*p2p.ChannelDescriptor {
	// TODO optimize
	return []*p2p.ChannelDescriptor{
		{
			ID:                  StateChannel,
			Priority:            6,
			SendQueueCapacity:   100,
			RecvMessageCapacity: maxMsgSize,
			MessageType:         &cmtcons.Message{},
		},
		{
			ID: DataChannel, // maybe split between gossiping current block and catchup stuff
			// once we gossip the whole block there's nothing left to send until next height or round
			Priority:            10,
			SendQueueCapacity:   100,
			RecvBufferCapacity:  50 * 4096,
			RecvMessageCapacity: maxMsgSize,
			MessageType:         &cmtcons.Message{},
		},
		{
			ID:                  VoteChannel,
			Priority:            7,
			SendQueueCapacity:   100,
			RecvBufferCapacity:  100 * 100,
			RecvMessageCapacity: maxMsgSize,
			MessageType:         &cmtcons.Message{},
		},
		{
			ID:                  VoteSetBitsChannel,
			Priority:            1,
			SendQueueCapacity:   2,
			RecvBufferCapacity:  1024,
			RecvMessageCapacity: maxMsgSize,
			MessageType:         &cmtcons.Message{},
		},
	}
}

// InitPeer implements Reactor by creating a state for the peer.
func (conR *Reactor) InitPeer(peer p2p.Peer) p2p.Peer {
	peerState := NewPeerState(peer).SetLogger(conR.Logger)
	peer.Set(types.PeerStateKey, peerState)
	return peer
}

// AddPeer implements Reactor by spawning multiple gossiping goroutines for the
// peer.
func (conR *Reactor) AddPeer(peer p2p.Peer) {
	if !conR.IsRunning() {
		return
	}

	peerState, ok := peer.Get(types.PeerStateKey).(*PeerState)
	if !ok {
		panic(fmt.Sprintf("peer %v has no state", peer))
	}
	// Begin routines for this peer.
	go conR.gossipDataRoutine(peer, peerState)
	go conR.gossipVotesRoutine(peer, peerState)
	go conR.queryMaj23Routine(peer, peerState)

	// Send our state to peer.
	// If we're block_syncing, broadcast a RoundStepMessage later upon SwitchToConsensus().
	if !conR.WaitSync() {
		conR.sendNewRoundStepMessage(peer)
	}
}

// RemovePeer is a noop.
func (conR *Reactor) RemovePeer(p2p.Peer, interface{}) {
	if !conR.IsRunning() {
		return
	}
	// TODO
	// ps, ok := peer.Get(PeerStateKey).(*PeerState)
	// if !ok {
	// 	panic(fmt.Sprintf("Peer %v has no state", peer))
	// }
	// ps.Disconnect()
}

// Receive implements Reactor
// NOTE: We process these messages even when we're block_syncing.
// Messages affect either a peer state or the consensus state.
// Peer state updates can happen in parallel, but processing of
// proposals, block parts, and votes are ordered by the receiveRoutine
// NOTE: blocks on consensus state for proposals, block parts, and votes
func (conR *Reactor) Receive(e p2p.Envelope) {
	if !conR.IsRunning() {
		conR.Logger.Debug("Receive", "src", e.Src, "chId", e.ChannelID)
		return
	}
	msg, err := MsgFromProto(e.Message)
	if err != nil {
		conR.Logger.Error("Error decoding message", "src", e.Src, "chId", e.ChannelID, "err", err)
		conR.Switch.StopPeerForError(e.Src, err)
		return
	}

	if err = msg.ValidateBasic(); err != nil {
		conR.Logger.Error("Peer sent us invalid msg", "peer", e.Src, "msg", e.Message, "err", err)
		conR.Switch.StopPeerForError(e.Src, err)
		return
	}

	conR.Logger.Debug("Receive", "src", e.Src, "chId", e.ChannelID, "msg", msg)

	// Get peer states
	ps, ok := e.Src.Get(types.PeerStateKey).(*PeerState)
	if !ok {
		panic(fmt.Sprintf("Peer %v has no state", e.Src))
	}

	switch e.ChannelID {
	case StateChannel:
		switch msg := msg.(type) {
		case *NewRoundStepMessage:
			conR.conS.mtx.Lock()
			initialHeight := conR.conS.state.InitialHeight
			conR.conS.mtx.Unlock()
			if err = msg.ValidateHeight(initialHeight); err != nil {
				conR.Logger.Error("Peer sent us invalid msg", "peer", e.Src, "msg", msg, "err", err)
				conR.Switch.StopPeerForError(e.Src, err)
				return
			}
			ps.ApplyNewRoundStepMessage(msg)
		case *NewValidBlockMessage:
			ps.ApplyNewValidBlockMessage(msg)
		case *HasVoteMessage:
			ps.ApplyHasVoteMessage(msg)
		case *VoteSetMaj23Message:
			cs := conR.conS
			cs.mtx.Lock()
			height, votes := cs.Height, cs.Votes
			cs.mtx.Unlock()
			if height != msg.Height {
				return
			}
			// Peer claims to have a maj23 for some BlockID at H,R,S,
			err := votes.SetPeerMaj23(msg.Round, msg.Type, ps.peer.ID(), msg.BlockID)
			if err != nil {
				conR.Switch.StopPeerForError(e.Src, err)
				return
			}
			// Respond with a VoteSetBitsMessage showing which votes we have.
			// (and consequently shows which we don't have)
			var ourVotes *bits.BitArray
			switch msg.Type {
			case cmtproto.PrevoteType:
				ourVotes = votes.Prevotes(msg.Round).BitArrayByBlockID(msg.BlockID)
			case cmtproto.PrecommitType:
				ourVotes = votes.Precommits(msg.Round).BitArrayByBlockID(msg.BlockID)
			default:
				panic("Bad VoteSetBitsMessage field Type. Forgot to add a check in ValidateBasic?")
			}
			eMsg := &cmtcons.VoteSetBits{
				Height:  msg.Height,
				Round:   msg.Round,
				Type:    msg.Type,
				BlockID: msg.BlockID.ToProto(),
			}
			if votes := ourVotes.ToProto(); votes != nil {
				eMsg.Votes = *votes
			}
			e.Src.TrySend(p2p.Envelope{
				ChannelID: VoteSetBitsChannel,
				Message:   eMsg,
			})
		default:
			conR.Logger.Error(fmt.Sprintf("Unknown message type %v", reflect.TypeOf(msg)))
		}

	case DataChannel:
		if conR.WaitSync() {
			conR.Logger.Info("Ignoring message received during sync", "msg", msg)
			return
		}
		switch msg := msg.(type) {
		case *ProposalMessage:
			ps.SetHasProposal(msg.Proposal)
			conR.conS.peerMsgQueue <- msgInfo{msg, e.Src.ID()}
		case *ProposalPOLMessage:
			ps.ApplyProposalPOLMessage(msg)
		case *BlockPartMessage:
			ps.SetHasProposalBlockPart(msg.Height, msg.Round, int(msg.Part.Index))
			conR.Metrics.BlockParts.With("peer_id", string(e.Src.ID())).Add(1)
			conR.conS.peerMsgQueue <- msgInfo{msg, e.Src.ID()}
		default:
			conR.Logger.Error(fmt.Sprintf("Unknown message type %v", reflect.TypeOf(msg)))
		}

	case VoteChannel:
		if conR.WaitSync() {
			conR.Logger.Info("Ignoring message received during sync", "msg", msg)
			return
		}
		switch msg := msg.(type) {
		case *VoteMessage:
			cs := conR.conS
			cs.mtx.RLock()
			height, valSize, lastCommitSize := cs.Height, cs.Validators.Size(), cs.LastCommit.Size()
			cs.mtx.RUnlock()
			ps.EnsureVoteBitArrays(height, valSize)
			ps.EnsureVoteBitArrays(height-1, lastCommitSize)
			ps.SetHasVote(msg.Vote)

			cs.peerMsgQueue <- msgInfo{msg, e.Src.ID()}

		default:
			// don't punish (leave room for soft upgrades)
			conR.Logger.Error(fmt.Sprintf("Unknown message type %v", reflect.TypeOf(msg)))
		}

	case VoteSetBitsChannel:
		if conR.WaitSync() {
			conR.Logger.Info("Ignoring message received during sync", "msg", msg)
			return
		}
		switch msg := msg.(type) {
		case *VoteSetBitsMessage:
			cs := conR.conS
			cs.mtx.Lock()
			height, votes := cs.Height, cs.Votes
			cs.mtx.Unlock()

			if height == msg.Height {
				var ourVotes *bits.BitArray
				switch msg.Type {
				case cmtproto.PrevoteType:
					ourVotes = votes.Prevotes(msg.Round).BitArrayByBlockID(msg.BlockID)
				case cmtproto.PrecommitType:
					ourVotes = votes.Precommits(msg.Round).BitArrayByBlockID(msg.BlockID)
				default:
					panic("Bad VoteSetBitsMessage field Type. Forgot to add a check in ValidateBasic?")
				}
				ps.ApplyVoteSetBitsMessage(msg, ourVotes)
			} else {
				ps.ApplyVoteSetBitsMessage(msg, nil)
			}
		default:
			// don't punish (leave room for soft upgrades)
			conR.Logger.Error(fmt.Sprintf("Unknown message type %v", reflect.TypeOf(msg)))
		}

	default:
		conR.Logger.Error(fmt.Sprintf("Unknown chId %X", e.ChannelID))
	}
}

// SetEventBus sets event bus.
func (conR *Reactor) SetEventBus(b *types.EventBus) {
	conR.eventBus = b
	conR.conS.SetEventBus(b)
}

// WaitSync returns whether the consensus reactor is waiting for state/block sync.
func (conR *Reactor) WaitSync() bool {
	conR.mtx.RLock()
	defer conR.mtx.RUnlock()
	return conR.waitSync
}

//--------------------------------------

// subscribeToBroadcastEvents subscribes for new round steps and votes
// using internal pubsub defined on state to broadcast
// them to peers upon receiving.
func (conR *Reactor) subscribeToBroadcastEvents() {
	const subscriber = "consensus-reactor"
	if err := conR.conS.evsw.AddListenerForEvent(subscriber, types.EventNewRoundStep,
		func(data cmtevents.EventData) {
			conR.broadcastNewRoundStepMessage(data.(*cstypes.RoundState))
		}); err != nil {
		conR.Logger.Error("Error adding listener for events", "err", err)
	}

	if err := conR.conS.evsw.AddListenerForEvent(subscriber, types.EventValidBlock,
		func(data cmtevents.EventData) {
			conR.broadcastNewValidBlockMessage(data.(*cstypes.RoundState))
		}); err != nil {
		conR.Logger.Error("Error adding listener for events", "err", err)
	}

	if err := conR.conS.evsw.AddListenerForEvent(subscriber, types.EventVote,
		func(data cmtevents.EventData) {
			conR.broadcastHasVoteMessage(data.(*types.Vote))
		}); err != nil {
		conR.Logger.Error("Error adding listener for events", "err", err)
	}
}

func (conR *Reactor) unsubscribeFromBroadcastEvents() {
	const subscriber = "consensus-reactor"
	conR.conS.evsw.RemoveListener(subscriber)
}

func (conR *Reactor) broadcastNewRoundStepMessage(rs *cstypes.RoundState) {
	nrsMsg := makeRoundStepMessage(rs)
	conR.Switch.Broadcast(p2p.Envelope{
		ChannelID: StateChannel,
		Message:   nrsMsg,
	})
}

func (conR *Reactor) broadcastNewValidBlockMessage(rs *cstypes.RoundState) {
	psh := rs.ProposalBlockParts.Header()
	csMsg := &cmtcons.NewValidBlock{
		Height:             rs.Height,
		Round:              rs.Round,
		BlockPartSetHeader: psh.ToProto(),
		BlockParts:         rs.ProposalBlockParts.BitArray().ToProto(),
		IsCommit:           rs.Step == cstypes.RoundStepCommit,
	}
	conR.Switch.Broadcast(p2p.Envelope{
		ChannelID: StateChannel,
		Message:   csMsg,
	})
}

// Broadcasts HasVoteMessage to peers that care.
func (conR *Reactor) broadcastHasVoteMessage(vote *types.Vote) {
	msg := &cmtcons.HasVote{
		Height: vote.Height,
		Round:  vote.Round,
		Type:   vote.Type,
		Index:  vote.ValidatorIndex,
	}
	conR.Switch.Broadcast(p2p.Envelope{
		ChannelID: StateChannel,
		Message:   msg,
	})
	/*
		// TODO: Make this broadcast more selective.
		for _, peer := range conR.Switch.Peers().List() {
			ps, ok := peer.Get(PeerStateKey).(*PeerState)
			if !ok {
				panic(fmt.Sprintf("Peer %v has no state", peer))
			}
			prs := ps.GetRoundState()
			if prs.Height == vote.Height {
				// TODO: Also filter on round?
				e := p2p.Envelope{
					ChannelID: StateChannel, struct{ ConsensusMessage }{msg},
					Message: p,
				}
				peer.TrySend(e)
			} else {
				// Height doesn't match
				// TODO: check a field, maybe CatchupCommitRound?
				// TODO: But that requires changing the struct field comment.
			}
		}
	*/
}

func makeRoundStepMessage(rs *cstypes.RoundState) (nrsMsg *cmtcons.NewRoundStep) {
	nrsMsg = &cmtcons.NewRoundStep{
		Height:                rs.Height,
		Round:                 rs.Round,
		Step:                  uint32(rs.Step),
		SecondsSinceStartTime: int64(time.Since(rs.StartTime).Seconds()),
		LastCommitRound:       rs.LastCommit.GetRound(),
	}
	return
}

func (conR *Reactor) sendNewRoundStepMessage(peer p2p.Peer) {
	rs := conR.getRoundState()
	nrsMsg := makeRoundStepMessage(rs)
	peer.Send(p2p.Envelope{
		ChannelID: StateChannel,
		Message:   nrsMsg,
	})
}

func (conR *Reactor) updateRoundStateRoutine() {
	t := time.NewTicker(100 * time.Microsecond)
	defer t.Stop()
	for range t.C {
		if !conR.IsRunning() {
			return
		}
		rs := conR.conS.GetRoundState()
		conR.mtx.Lock()
		conR.rs = rs
		conR.mtx.Unlock()
	}
}

func (conR *Reactor) getRoundState() *cstypes.RoundState {
	conR.mtx.RLock()
	defer conR.mtx.RUnlock()
	return conR.rs
}

func (conR *Reactor) gossipDataRoutine(peer p2p.Peer, ps *PeerState) {
	logger := conR.Logger.With("peer", peer)

OUTER_LOOP:
	for {
		// Manage disconnects from self or peer.
		if !peer.IsRunning() || !conR.IsRunning() {
			return
		}
		rs := conR.getRoundState()
		prs := ps.GetRoundState()

		// Send proposal Block parts?
		if rs.ProposalBlockParts.HasHeader(prs.ProposalBlockPartSetHeader) {
			if index, ok := rs.ProposalBlockParts.BitArray().Sub(prs.ProposalBlockParts.Copy()).PickRandom(); ok {
				part := rs.ProposalBlockParts.GetPart(index)
				parts, err := part.ToProto()
				if err != nil {
					panic(err)
				}
				logger.Debug("Sending block part", "height", prs.Height, "round", prs.Round)
				if peer.Send(p2p.Envelope{
					ChannelID: DataChannel,
					Message: &cmtcons.BlockPart{
						Height: rs.Height, // This tells peer that this part applies to us.
						Round:  rs.Round,  // This tells peer that this part applies to us.
						Part:   *parts,
					},
				}) {
					ps.SetHasProposalBlockPart(prs.Height, prs.Round, index)
				}
				continue OUTER_LOOP
			}
		}

		// If the peer is on a previous height that we have, help catch up.
		blockStoreBase := conR.conS.blockStore.Base()
		if blockStoreBase > 0 && 0 < prs.Height && prs.Height < rs.Height && prs.Height >= blockStoreBase {
			heightLogger := logger.With("height", prs.Height)

			// if we never received the commit message from the peer, the block parts wont be initialized
			if prs.ProposalBlockParts == nil {
				blockMeta := conR.conS.blockStore.LoadBlockMeta(prs.Height)
				if blockMeta == nil {
					heightLogger.Error("Failed to load block meta",
						"blockstoreBase", blockStoreBase, "blockstoreHeight", conR.conS.blockStore.Height())
					time.Sleep(conR.conS.config.PeerGossipSleepDuration)
				} else {
					ps.InitProposalBlockParts(blockMeta.BlockID.PartSetHeader)
				}
				// continue the loop since prs is a copy and not effected by this initialization
				continue OUTER_LOOP
			}
			conR.gossipDataForCatchup(heightLogger, rs, prs, ps, peer)
			continue OUTER_LOOP
		}

		// If height and round don't match, sleep.
		if (rs.Height != prs.Height) || (rs.Round != prs.Round) {
			// logger.Info("Peer Height|Round mismatch, sleeping",
			// "peerHeight", prs.Height, "peerRound", prs.Round, "peer", peer)
			time.Sleep(conR.conS.config.PeerGossipSleepDuration)
			continue OUTER_LOOP
		}

		// By here, height and round match.
		// Proposal block parts were already matched and sent if any were wanted.
		// (These can match on hash so the round doesn't matter)
		// Now consider sending other things, like the Proposal itself.

		// Send Proposal && ProposalPOL BitArray?
		if rs.Proposal != nil && !prs.Proposal {
			// Proposal: share the proposal metadata with peer.
			{
				logger.Debug("Sending proposal", "height", prs.Height, "round", prs.Round)
				if peer.Send(p2p.Envelope{
					ChannelID: DataChannel,
					Message:   &cmtcons.Proposal{Proposal: *rs.Proposal.ToProto()},
				}) {
					// NOTE[ZM]: A peer might have received different proposal msg so this Proposal msg will be rejected!
					ps.SetHasProposal(rs.Proposal)
				}
			}
			// ProposalPOL: lets peer know which POL votes we have so far.
			// Peer must receive ProposalMessage first.
			// rs.Proposal was validated, so rs.Proposal.POLRound <= rs.Round,
			// so we definitely have rs.Votes.Prevotes(rs.Proposal.POLRound).
			if 0 <= rs.Proposal.POLRound {
				logger.Debug("Sending POL", "height", prs.Height, "round", prs.Round)
				peer.Send(p2p.Envelope{
					ChannelID: DataChannel,
					Message: &cmtcons.ProposalPOL{
						Height:           rs.Height,
						ProposalPolRound: rs.Proposal.POLRound,
						ProposalPol:      *rs.Votes.Prevotes(rs.Proposal.POLRound).BitArray().ToProto(),
					},
				})
			}
			continue OUTER_LOOP
		}

		// Nothing to do. Sleep.
		time.Sleep(conR.conS.config.PeerGossipSleepDuration)
		continue OUTER_LOOP
	}
}

func (conR *Reactor) gossipDataForCatchup(logger log.Logger, rs *cstypes.RoundState,
	prs *cstypes.PeerRoundState, ps *PeerState, peer p2p.Peer,
) {
	if index, ok := prs.ProposalBlockParts.Not().PickRandom(); ok {
		// Ensure that the peer's PartSetHeader is correct
		blockMeta := conR.conS.blockStore.LoadBlockMeta(prs.Height)
		if blockMeta == nil {
			logger.Error("Failed to load block meta", "ourHeight", rs.Height,
				"blockstoreBase", conR.conS.blockStore.Base(), "blockstoreHeight", conR.conS.blockStore.Height())
			time.Sleep(conR.conS.config.PeerGossipSleepDuration)
			return
		} else if !blockMeta.BlockID.PartSetHeader.Equals(prs.ProposalBlockPartSetHeader) {
			logger.Info("Peer ProposalBlockPartSetHeader mismatch, sleeping",
				"blockPartSetHeader", blockMeta.BlockID.PartSetHeader, "peerBlockPartSetHeader", prs.ProposalBlockPartSetHeader)
			time.Sleep(conR.conS.config.PeerGossipSleepDuration)
			return
		}
		// Load the part
		part := conR.conS.blockStore.LoadBlockPart(prs.Height, index)
		if part == nil {
			logger.Error("Could not load part", "index", index,
				"blockPartSetHeader", blockMeta.BlockID.PartSetHeader, "peerBlockPartSetHeader", prs.ProposalBlockPartSetHeader)
			time.Sleep(conR.conS.config.PeerGossipSleepDuration)
			return
		}
		// Send the part
		logger.Debug("Sending block part for catchup", "round", prs.Round, "index", index)
		pp, err := part.ToProto()
		if err != nil {
			logger.Error("Could not convert part to proto", "index", index, "error", err)
			return
		}
		if peer.Send(p2p.Envelope{
			ChannelID: DataChannel,
			Message: &cmtcons.BlockPart{
				Height: prs.Height, // Not our height, so it doesn't matter.
				Round:  prs.Round,  // Not our height, so it doesn't matter.
				Part:   *pp,
			},
		}) {
			ps.SetHasProposalBlockPart(prs.Height, prs.Round, index)
		} else {
			logger.Debug("Sending block part for catchup failed")
			// sleep to avoid retrying too fast
			time.Sleep(conR.conS.config.PeerGossipSleepDuration)
		}
		return
	}
	//  logger.Info("No parts to send in catch-up, sleeping")
	time.Sleep(conR.conS.config.PeerGossipSleepDuration)
}

func (conR *Reactor) gossipVotesRoutine(peer p2p.Peer, ps *PeerState) {
	logger := conR.Logger.With("peer", peer)

	// Simple hack to throttle logs upon sleep.
	sleeping := 0

OUTER_LOOP:
	for {
		// Manage disconnects from self or peer.
		if !peer.IsRunning() || !conR.IsRunning() {
			return
		}
		rs := conR.getRoundState()
		prs := ps.GetRoundState()

		switch sleeping {
		case 1: // First sleep
			sleeping = 2
		case 2: // No more sleep
			sleeping = 0
		}

		// logger.Debug("gossipVotesRoutine", "rsHeight", rs.Height, "rsRound", rs.Round,
		// "prsHeight", prs.Height, "prsRound", prs.Round, "prsStep", prs.Step)

		// If height matches, then send LastCommit, Prevotes, Precommits.
		if rs.Height == prs.Height {
			heightLogger := logger.With("height", prs.Height)
			if conR.gossipVotesForHeight(heightLogger, rs, prs, ps) {
				continue OUTER_LOOP
			}
		}

		// Special catchup logic.
		// If peer is lagging by height 1, send LastCommit.
		if prs.Height != 0 && rs.Height == prs.Height+1 {
			if ps.PickSendVote(rs.LastCommit) {
				logger.Debug("Picked rs.LastCommit to send", "height", prs.Height)
				continue OUTER_LOOP
			}
		}

		// Catchup logic
		// If peer is lagging by more than 1, send Commit.
		blockStoreBase := conR.conS.blockStore.Base()
		if blockStoreBase > 0 && prs.Height != 0 && rs.Height >= prs.Height+2 && prs.Height >= blockStoreBase {
			// Load the block's extended commit for prs.Height,
			// which contains precommit signatures for prs.Height.
			var ec *types.ExtendedCommit
			var veEnabled bool
			func() {
				conR.conS.mtx.RLock()
				defer conR.conS.mtx.RUnlock()
				veEnabled = conR.conS.state.ConsensusParams.ABCI.VoteExtensionsEnabled(prs.Height)
			}()
			if veEnabled {
				ec = conR.conS.blockStore.LoadBlockExtendedCommit(prs.Height)
			} else {
				c := conR.conS.blockStore.LoadBlockCommit(prs.Height)
				if c == nil {
					continue
				}
				ec = c.WrappedExtendedCommit()
			}
			if ec == nil {
				continue
			}
			if ps.PickSendVote(ec) {
				logger.Debug("Picked Catchup commit to send", "height", prs.Height)
				continue OUTER_LOOP
			}
		}

		if sleeping == 0 {
			// We sent nothing. Sleep...
			sleeping = 1
			logger.Debug("No votes to send, sleeping", "rs.Height", rs.Height, "prs.Height", prs.Height,
				"localPV", rs.Votes.Prevotes(rs.Round).BitArray(), "peerPV", prs.Prevotes,
				"localPC", rs.Votes.Precommits(rs.Round).BitArray(), "peerPC", prs.Precommits)
		} else if sleeping == 2 {
			// Continued sleep...
			sleeping = 1
		}

		time.Sleep(conR.conS.config.PeerGossipSleepDuration)
		continue OUTER_LOOP
	}
}

func (conR *Reactor) gossipVotesForHeight(
	logger log.Logger,
	rs *cstypes.RoundState,
	prs *cstypes.PeerRoundState,
	ps *PeerState,
) bool {
	// If there are lastCommits to send...
	if prs.Step == cstypes.RoundStepNewHeight {
		if ps.PickSendVote(rs.LastCommit) {
			logger.Debug("Picked rs.LastCommit to send")
			return true
		}
	}
	// If there are POL prevotes to send...
	if prs.Step <= cstypes.RoundStepPropose && prs.Round != -1 && prs.Round <= rs.Round && prs.ProposalPOLRound != -1 {
		if polPrevotes := rs.Votes.Prevotes(prs.ProposalPOLRound); polPrevotes != nil {
			if ps.PickSendVote(polPrevotes) {
				logger.Debug("Picked rs.Prevotes(prs.ProposalPOLRound) to send",
					"round", prs.ProposalPOLRound)
				return true
			}
		}
	}
	// If there are prevotes to send...
	if prs.Step <= cstypes.RoundStepPrevoteWait && prs.Round != -1 && prs.Round <= rs.Round {
		if ps.PickSendVote(rs.Votes.Prevotes(prs.Round)) {
			logger.Debug("Picked rs.Prevotes(prs.Round) to send", "round", prs.Round)
			return true
		}
	}
	// If there are precommits to send...
	if prs.Step <= cstypes.RoundStepPrecommitWait && prs.Round != -1 && prs.Round <= rs.Round {
		if ps.PickSendVote(rs.Votes.Precommits(prs.Round)) {
			logger.Debug("Picked rs.Precommits(prs.Round) to send", "round", prs.Round)
			return true
		}
	}
	// If there are prevotes to send...Needed because of validBlock mechanism
	if prs.Round != -1 && prs.Round <= rs.Round {
		if ps.PickSendVote(rs.Votes.Prevotes(prs.Round)) {
			logger.Debug("Picked rs.Prevotes(prs.Round) to send", "round", prs.Round)
			return true
		}
	}
	// If there are POLPrevotes to send...
	if prs.ProposalPOLRound != -1 {
		if polPrevotes := rs.Votes.Prevotes(prs.ProposalPOLRound); polPrevotes != nil {
			if ps.PickSendVote(polPrevotes) {
				logger.Debug("Picked rs.Prevotes(prs.ProposalPOLRound) to send",
					"round", prs.ProposalPOLRound)
				return true
			}
		}
	}

	return false
}

// NOTE: `queryMaj23Routine` has a simple crude design since it only comes
// into play for liveness when there's a signature DDoS attack happening.
func (conR *Reactor) queryMaj23Routine(peer p2p.Peer, ps *PeerState) {
OUTER_LOOP:
	for {
		// Manage disconnects from self or peer.
		if !peer.IsRunning() || !conR.IsRunning() {
			return
		}

		// Maybe send Height/Round/Prevotes
		{
			rs := conR.getRoundState()
			prs := ps.GetRoundState()
			if rs.Height == prs.Height {
				if maj23, ok := rs.Votes.Prevotes(prs.Round).TwoThirdsMajority(); ok {

					peer.TrySend(p2p.Envelope{
						ChannelID: StateChannel,
						Message: &cmtcons.VoteSetMaj23{
							Height:  prs.Height,
							Round:   prs.Round,
							Type:    cmtproto.PrevoteType,
							BlockID: maj23.ToProto(),
						},
					})
					time.Sleep(conR.conS.config.PeerQueryMaj23SleepDuration)
				}
			}
		}

		// Maybe send Height/Round/Precommits
		{
			rs := conR.getRoundState()
			prs := ps.GetRoundState()
			if rs.Height == prs.Height {
				if maj23, ok := rs.Votes.Precommits(prs.Round).TwoThirdsMajority(); ok {
					peer.TrySend(p2p.Envelope{
						ChannelID: StateChannel,
						Message: &cmtcons.VoteSetMaj23{
							Height:  prs.Height,
							Round:   prs.Round,
							Type:    cmtproto.PrecommitType,
							BlockID: maj23.ToProto(),
						},
					})
					time.Sleep(conR.conS.config.PeerQueryMaj23SleepDuration)
				}
			}
		}

		// Maybe send Height/Round/ProposalPOL
		{
			rs := conR.getRoundState()
			prs := ps.GetRoundState()
			if rs.Height == prs.Height && prs.ProposalPOLRound >= 0 {
				if maj23, ok := rs.Votes.Prevotes(prs.ProposalPOLRound).TwoThirdsMajority(); ok {

					peer.TrySend(p2p.Envelope{
						ChannelID: StateChannel,
						Message: &cmtcons.VoteSetMaj23{
							Height:  prs.Height,
							Round:   prs.ProposalPOLRound,
							Type:    cmtproto.PrevoteType,
							BlockID: maj23.ToProto(),
						},
					})
					time.Sleep(conR.conS.config.PeerQueryMaj23SleepDuration)
				}
			}
		}

		// Little point sending LastCommitRound/LastCommit,
		// These are fleeting and non-blocking.

		// Maybe send Height/CatchupCommitRound/CatchupCommit.
		{
			prs := ps.GetRoundState()
			if prs.CatchupCommitRound != -1 && prs.Height > 0 && prs.Height <= conR.conS.blockStore.Height() &&
				prs.Height >= conR.conS.blockStore.Base() {
				if commit := conR.conS.LoadCommit(prs.Height); commit != nil {
					peer.TrySend(p2p.Envelope{
						ChannelID: StateChannel,
						Message: &cmtcons.VoteSetMaj23{
							Height:  prs.Height,
							Round:   commit.Round,
							Type:    cmtproto.PrecommitType,
							BlockID: commit.BlockID.ToProto(),
						},
					})
					time.Sleep(conR.conS.config.PeerQueryMaj23SleepDuration)
				}
			}
		}

		time.Sleep(conR.conS.config.PeerQueryMaj23SleepDuration)

		continue OUTER_LOOP
	}
}

func (conR *Reactor) peerStatsRoutine() {
	for {
		if !conR.IsRunning() {
			conR.Logger.Info("Stopping peerStatsRoutine")
			return
		}

		select {
		case msg := <-conR.conS.statsMsgQueue:
			// Get peer
			peer := conR.Switch.Peers().Get(msg.PeerID)
			if peer == nil {
				conR.Logger.Debug("Attempt to update stats for non-existent peer",
					"peer", msg.PeerID)
				continue
			}
			// Get peer state
			ps, ok := peer.Get(types.PeerStateKey).(*PeerState)
			if !ok {
				panic(fmt.Sprintf("Peer %v has no state", peer))
			}
			switch msg.Msg.(type) {
			case *VoteMessage:
				if numVotes := ps.RecordVote(); numVotes%votesToContributeToBecomeGoodPeer == 0 {
					conR.Switch.MarkPeerAsGood(peer)
				}
			case *BlockPartMessage:
				if numParts := ps.RecordBlockPart(); numParts%blocksToContributeToBecomeGoodPeer == 0 {
					conR.Switch.MarkPeerAsGood(peer)
				}
			}
		case <-conR.conS.Quit():
			return

		case <-conR.Quit():
			return
		}
	}
}

// String returns a string representation of the Reactor.
// NOTE: For now, it is just a hard-coded string to avoid accessing unprotected shared variables.
// TODO: improve!
func (conR *Reactor) String() string {
	// better not to access shared variables
	return "ConsensusReactor" // conR.StringIndented("")
}

// StringIndented returns an indented string representation of the Reactor
func (conR *Reactor) StringIndented(indent string) string {
	s := "ConsensusReactor{\n"
	s += indent + "  " + conR.conS.StringIndented(indent+"  ") + "\n"
	for _, peer := range conR.Switch.Peers().List() {
		ps, ok := peer.Get(types.PeerStateKey).(*PeerState)
		if !ok {
			panic(fmt.Sprintf("Peer %v has no state", peer))
		}
		s += indent + "  " + ps.StringIndented(indent+"  ") + "\n"
	}
	s += indent + "}"
	return s
}

// ReactorMetrics sets the metrics
func ReactorMetrics(metrics *Metrics) ReactorOption {
	return func(conR *Reactor) { conR.Metrics = metrics }
}

//-----------------------------------------------------------------------------

var (
	ErrPeerStateHeightRegression = errors.New("error peer state height regression")
	ErrPeerStateInvalidStartTime = errors.New("error peer state invalid startTime")
)

// PeerState contains the known state of a peer, including its connection and
// threadsafe access to its PeerRoundState.
// NOTE: THIS GETS DUMPED WITH rpc/core/consensus.go.
// Be mindful of what you Expose.
type PeerState struct {
	peer   p2p.Peer
	logger log.Logger

	mtx   sync.Mutex             // NOTE: Modify below using setters, never directly.
	PRS   cstypes.PeerRoundState `json:"round_state"` // Exposed.
	Stats *peerStateStats        `json:"stats"`       // Exposed.
}

// peerStateStats holds internal statistics for a peer.
type peerStateStats struct {
	Votes      int `json:"votes"`
	BlockParts int `json:"block_parts"`
}

func (pss peerStateStats) String() string {
	return fmt.Sprintf("peerStateStats{votes: %d, blockParts: %d}",
		pss.Votes, pss.BlockParts)
}

// NewPeerState returns a new PeerState for the given Peer
func NewPeerState(peer p2p.Peer) *PeerState {
	return &PeerState{
		peer:   peer,
		logger: log.NewNopLogger(),
		PRS: cstypes.PeerRoundState{
			Round:              -1,
			ProposalPOLRound:   -1,
			LastCommitRound:    -1,
			CatchupCommitRound: -1,
		},
		Stats: &peerStateStats{},
	}
}

// SetLogger allows to set a logger on the peer state. Returns the peer state
// itself.
func (ps *PeerState) SetLogger(logger log.Logger) *PeerState {
	ps.logger = logger
	return ps
}

// GetRoundState returns an shallow copy of the PeerRoundState.
// There's no point in mutating it since it won't change PeerState.
func (ps *PeerState) GetRoundState() *cstypes.PeerRoundState {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	prs := ps.PRS // copy
	return &prs
}

// MarshalJSON implements the json.Marshaler interface.
func (ps *PeerState) MarshalJSON() ([]byte, error) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	type jsonPeerState PeerState
	return cmtjson.Marshal((*jsonPeerState)(ps))
}

// GetHeight returns an atomic snapshot of the PeerRoundState's height
// used by the mempool to ensure peers are caught up before broadcasting new txs
func (ps *PeerState) GetHeight() int64 {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()
	return ps.PRS.Height
}

// SetHasProposal sets the given proposal as known for the peer.
func (ps *PeerState) SetHasProposal(proposal *types.Proposal) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.Height != proposal.Height || ps.PRS.Round != proposal.Round {
		return
	}

	if ps.PRS.Proposal {
		return
	}

	ps.PRS.Proposal = true

	// ps.PRS.ProposalBlockParts is set due to NewValidBlockMessage
	if ps.PRS.ProposalBlockParts != nil {
		return
	}

	ps.PRS.ProposalBlockPartSetHeader = proposal.BlockID.PartSetHeader
	ps.PRS.ProposalBlockParts = bits.NewBitArray(int(proposal.BlockID.PartSetHeader.Total))
	ps.PRS.ProposalPOLRound = proposal.POLRound
	ps.PRS.ProposalPOL = nil // Nil until ProposalPOLMessage received.
}

// InitProposalBlockParts initializes the peer's proposal block parts header and bit array.
func (ps *PeerState) InitProposalBlockParts(partSetHeader types.PartSetHeader) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.ProposalBlockParts != nil {
		return
	}

	ps.PRS.ProposalBlockPartSetHeader = partSetHeader
	ps.PRS.ProposalBlockParts = bits.NewBitArray(int(partSetHeader.Total))
}

// SetHasProposalBlockPart sets the given block part index as known for the peer.
func (ps *PeerState) SetHasProposalBlockPart(height int64, round int32, index int) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.Height != height || ps.PRS.Round != round {
		return
	}

	ps.PRS.ProposalBlockParts.SetIndex(index, true)
}

// PickSendVote picks a vote and sends it to the peer.
// Returns true if vote was sent.
func (ps *PeerState) PickSendVote(votes types.VoteSetReader) bool {
	if vote, ok := ps.PickVoteToSend(votes); ok {
		ps.logger.Debug("Sending vote message", "ps", ps, "vote", vote)
		if ps.peer.Send(p2p.Envelope{
			ChannelID: VoteChannel,
			Message: &cmtcons.Vote{
				Vote: vote.ToProto(),
			},
		}) {
			ps.SetHasVote(vote)
			return true
		}
		return false
	}
	return false
}

// PickVoteToSend picks a vote to send to the peer.
// Returns true if a vote was picked.
// NOTE: `votes` must be the correct Size() for the Height().
func (ps *PeerState) PickVoteToSend(votes types.VoteSetReader) (vote *types.Vote, ok bool) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if votes.Size() == 0 {
		return nil, false
	}

	height, round, votesType, size := votes.GetHeight(), votes.GetRound(), cmtproto.SignedMsgType(votes.Type()), votes.Size()

	// Lazily set data using 'votes'.
	if votes.IsCommit() {
		ps.ensureCatchupCommitRound(height, round, size)
	}
	ps.ensureVoteBitArrays(height, size)

	psVotes := ps.getVoteBitArray(height, round, votesType)
	if psVotes == nil {
		return nil, false // Not something worth sending
	}
	if index, ok := votes.BitArray().Sub(psVotes).PickRandom(); ok {
		return votes.GetByIndex(int32(index)), true
	}
	return nil, false
}

func (ps *PeerState) getVoteBitArray(height int64, round int32, votesType cmtproto.SignedMsgType) *bits.BitArray {
	if !types.IsVoteTypeValid(votesType) {
		return nil
	}

	if ps.PRS.Height == height {
		if ps.PRS.Round == round {
			switch votesType {
			case cmtproto.PrevoteType:
				return ps.PRS.Prevotes
			case cmtproto.PrecommitType:
				return ps.PRS.Precommits
			}
		}
		if ps.PRS.CatchupCommitRound == round {
			switch votesType {
			case cmtproto.PrevoteType:
				return nil
			case cmtproto.PrecommitType:
				return ps.PRS.CatchupCommit
			}
		}
		if ps.PRS.ProposalPOLRound == round {
			switch votesType {
			case cmtproto.PrevoteType:
				return ps.PRS.ProposalPOL
			case cmtproto.PrecommitType:
				return nil
			}
		}
		return nil
	}
	if ps.PRS.Height == height+1 {
		if ps.PRS.LastCommitRound == round {
			switch votesType {
			case cmtproto.PrevoteType:
				return nil
			case cmtproto.PrecommitType:
				return ps.PRS.LastCommit
			}
		}
		return nil
	}
	return nil
}

// 'round': A round for which we have a +2/3 commit.
func (ps *PeerState) ensureCatchupCommitRound(height int64, round int32, numValidators int) {
	if ps.PRS.Height != height {
		return
	}
	/*
		NOTE: This is wrong, 'round' could change.
		e.g. if orig round is not the same as block LastCommit round.
		if ps.CatchupCommitRound != -1 && ps.CatchupCommitRound != round {
			panic(fmt.Sprintf(
				"Conflicting CatchupCommitRound. Height: %v,
				Orig: %v,
				New: %v",
				height,
				ps.CatchupCommitRound,
				round))
		}
	*/
	if ps.PRS.CatchupCommitRound == round {
		return // Nothing to do!
	}
	ps.PRS.CatchupCommitRound = round
	if round == ps.PRS.Round {
		ps.PRS.CatchupCommit = ps.PRS.Precommits
	} else {
		ps.PRS.CatchupCommit = bits.NewBitArray(numValidators)
	}
}

// EnsureVoteBitArrays ensures the bit-arrays have been allocated for tracking
// what votes this peer has received.
// NOTE: It's important to make sure that numValidators actually matches
// what the node sees as the number of validators for height.
func (ps *PeerState) EnsureVoteBitArrays(height int64, numValidators int) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()
	ps.ensureVoteBitArrays(height, numValidators)
}

func (ps *PeerState) ensureVoteBitArrays(height int64, numValidators int) {
	if ps.PRS.Height == height {
		if ps.PRS.Prevotes == nil {
			ps.PRS.Prevotes = bits.NewBitArray(numValidators)
		}
		if ps.PRS.Precommits == nil {
			ps.PRS.Precommits = bits.NewBitArray(numValidators)
		}
		if ps.PRS.CatchupCommit == nil {
			ps.PRS.CatchupCommit = bits.NewBitArray(numValidators)
		}
		if ps.PRS.ProposalPOL == nil {
			ps.PRS.ProposalPOL = bits.NewBitArray(numValidators)
		}
	} else if ps.PRS.Height == height+1 {
		if ps.PRS.LastCommit == nil {
			ps.PRS.LastCommit = bits.NewBitArray(numValidators)
		}
	}
}

// RecordVote increments internal votes related statistics for this peer.
// It returns the total number of added votes.
func (ps *PeerState) RecordVote() int {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	ps.Stats.Votes++

	return ps.Stats.Votes
}

// VotesSent returns the number of blocks for which peer has been sending us
// votes.
func (ps *PeerState) VotesSent() int {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	return ps.Stats.Votes
}

// RecordBlockPart increments internal block part related statistics for this peer.
// It returns the total number of added block parts.
func (ps *PeerState) RecordBlockPart() int {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	ps.Stats.BlockParts++
	return ps.Stats.BlockParts
}

// BlockPartsSent returns the number of useful block parts the peer has sent us.
func (ps *PeerState) BlockPartsSent() int {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	return ps.Stats.BlockParts
}

// SetHasVote sets the given vote as known by the peer
func (ps *PeerState) SetHasVote(vote *types.Vote) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	ps.setHasVote(vote.Height, vote.Round, vote.Type, vote.ValidatorIndex)
}

func (ps *PeerState) setHasVote(height int64, round int32, voteType cmtproto.SignedMsgType, index int32) {
	ps.logger.Debug("setHasVote",
		"peerH/R",
		log.NewLazySprintf("%d/%d", ps.PRS.Height, ps.PRS.Round),
		"H/R",
		log.NewLazySprintf("%d/%d", height, round),
		"type", voteType, "index", index)

	// NOTE: some may be nil BitArrays -> no side effects.
	psVotes := ps.getVoteBitArray(height, round, voteType)
	if psVotes != nil {
		psVotes.SetIndex(int(index), true)
	}
}

// ApplyNewRoundStepMessage updates the peer state for the new round.
func (ps *PeerState) ApplyNewRoundStepMessage(msg *NewRoundStepMessage) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	// Ignore duplicates or decreases
	if CompareHRS(msg.Height, msg.Round, msg.Step, ps.PRS.Height, ps.PRS.Round, ps.PRS.Step) <= 0 {
		return
	}

	// Just remember these values.
	psHeight := ps.PRS.Height
	psRound := ps.PRS.Round
	psCatchupCommitRound := ps.PRS.CatchupCommitRound
	psCatchupCommit := ps.PRS.CatchupCommit
	lastPrecommits := ps.PRS.Precommits

	startTime := cmttime.Now().Add(-1 * time.Duration(msg.SecondsSinceStartTime) * time.Second)
	ps.PRS.Height = msg.Height
	ps.PRS.Round = msg.Round
	ps.PRS.Step = msg.Step
	ps.PRS.StartTime = startTime
	if psHeight != msg.Height || psRound != msg.Round {
		ps.PRS.Proposal = false
		ps.PRS.ProposalBlockPartSetHeader = types.PartSetHeader{}
		ps.PRS.ProposalBlockParts = nil
		ps.PRS.ProposalPOLRound = -1
		ps.PRS.ProposalPOL = nil
		// We'll update the BitArray capacity later.
		ps.PRS.Prevotes = nil
		ps.PRS.Precommits = nil
	}
	if psHeight == msg.Height && psRound != msg.Round && msg.Round == psCatchupCommitRound {
		// Peer caught up to CatchupCommitRound.
		// Preserve psCatchupCommit!
		// NOTE: We prefer to use prs.Precommits if
		// pr.Round matches pr.CatchupCommitRound.
		ps.PRS.Precommits = psCatchupCommit
	}
	if psHeight != msg.Height {
		// Shift Precommits to LastCommit.
		if psHeight+1 == msg.Height && psRound == msg.LastCommitRound {
			ps.PRS.LastCommitRound = msg.LastCommitRound
			ps.PRS.LastCommit = lastPrecommits
		} else {
			ps.PRS.LastCommitRound = msg.LastCommitRound
			ps.PRS.LastCommit = nil
		}
		// We'll update the BitArray capacity later.
		ps.PRS.CatchupCommitRound = -1
		ps.PRS.CatchupCommit = nil
	}
}

// ApplyNewValidBlockMessage updates the peer state for the new valid block.
func (ps *PeerState) ApplyNewValidBlockMessage(msg *NewValidBlockMessage) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.Height != msg.Height {
		return
	}

	if ps.PRS.Round != msg.Round && !msg.IsCommit {
		return
	}

	ps.PRS.ProposalBlockPartSetHeader = msg.BlockPartSetHeader
	ps.PRS.ProposalBlockParts = msg.BlockParts
}

// ApplyProposalPOLMessage updates the peer state for the new proposal POL.
func (ps *PeerState) ApplyProposalPOLMessage(msg *ProposalPOLMessage) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.Height != msg.Height {
		return
	}
	if ps.PRS.ProposalPOLRound != msg.ProposalPOLRound {
		return
	}

	// TODO: Merge onto existing ps.PRS.ProposalPOL?
	// We might have sent some prevotes in the meantime.
	ps.PRS.ProposalPOL = msg.ProposalPOL
}

// ApplyHasVoteMessage updates the peer state for the new vote.
func (ps *PeerState) ApplyHasVoteMessage(msg *HasVoteMessage) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	if ps.PRS.Height != msg.Height {
		return
	}

	ps.setHasVote(msg.Height, msg.Round, msg.Type, msg.Index)
}

// ApplyVoteSetBitsMessage updates the peer state for the bit-array of votes
// it claims to have for the corresponding BlockID.
// `ourVotes` is a BitArray of votes we have for msg.BlockID
// NOTE: if ourVotes is nil (e.g. msg.Height < rs.Height),
// we conservatively overwrite ps's votes w/ msg.Votes.
func (ps *PeerState) ApplyVoteSetBitsMessage(msg *VoteSetBitsMessage, ourVotes *bits.BitArray) {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()

	votes := ps.getVoteBitArray(msg.Height, msg.Round, msg.Type)
	if votes != nil {
		if ourVotes == nil {
			votes.Update(msg.Votes)
		} else {
			otherVotes := votes.Sub(ourVotes)
			hasVotes := otherVotes.Or(msg.Votes)
			votes.Update(hasVotes)
		}
	}
}

// String returns a string representation of the PeerState
func (ps *PeerState) String() string {
	return ps.StringIndented("")
}

// StringIndented returns a string representation of the PeerState
func (ps *PeerState) StringIndented(indent string) string {
	ps.mtx.Lock()
	defer ps.mtx.Unlock()
	return fmt.Sprintf(`PeerState{
%s  Key        %v
%s  RoundState %v
%s  Stats      %v
%s}`,
		indent, ps.peer.ID(),
		indent, ps.PRS.StringIndented(indent+"  "),
		indent, ps.Stats,
		indent)
}

//-----------------------------------------------------------------------------
// Messages

// Message is a message that can be sent and received on the Reactor
type Message interface {
	ValidateBasic() error
}

func init() {
	cmtjson.RegisterType(&NewRoundStepMessage{}, "tendermint/NewRoundStepMessage")
	cmtjson.RegisterType(&NewValidBlockMessage{}, "tendermint/NewValidBlockMessage")
	cmtjson.RegisterType(&ProposalMessage{}, "tendermint/Proposal")
	cmtjson.RegisterType(&ProposalPOLMessage{}, "tendermint/ProposalPOL")
	cmtjson.RegisterType(&BlockPartMessage{}, "tendermint/BlockPart")
	cmtjson.RegisterType(&VoteMessage{}, "tendermint/Vote")
	cmtjson.RegisterType(&HasVoteMessage{}, "tendermint/HasVote")
	cmtjson.RegisterType(&VoteSetMaj23Message{}, "tendermint/VoteSetMaj23")
	cmtjson.RegisterType(&VoteSetBitsMessage{}, "tendermint/VoteSetBits")
}

//-------------------------------------

// NewRoundStepMessage is sent for every step taken in the ConsensusState.
// For every height/round/step transition
type NewRoundStepMessage struct {
	Height                int64
	Round                 int32
	Step                  cstypes.RoundStepType
	SecondsSinceStartTime int64
	LastCommitRound       int32
}

// ValidateBasic performs basic validation.
func (m *NewRoundStepMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.Round < 0 {
		return errors.New("negative Round")
	}
	if !m.Step.IsValid() {
		return errors.New("invalid Step")
	}

	// NOTE: SecondsSinceStartTime may be negative

	// LastCommitRound will be -1 for the initial height, but we don't know what height this is
	// since it can be specified in genesis. The reactor will have to validate this via
	// ValidateHeight().
	if m.LastCommitRound < -1 {
		return errors.New("invalid LastCommitRound (cannot be < -1)")
	}

	return nil
}

// ValidateHeight validates the height given the chain's initial height.
func (m *NewRoundStepMessage) ValidateHeight(initialHeight int64) error {
	if m.Height < initialHeight {
		return fmt.Errorf("invalid Height %v (lower than initial height %v)",
			m.Height, initialHeight)
	}
	if m.Height == initialHeight && m.LastCommitRound != -1 {
		return fmt.Errorf("invalid LastCommitRound %v (must be -1 for initial height %v)",
			m.LastCommitRound, initialHeight)
	}
	if m.Height > initialHeight && m.LastCommitRound < 0 {
		return fmt.Errorf("LastCommitRound can only be negative for initial height %v",
			initialHeight)
	}
	return nil
}

// String returns a string representation.
func (m *NewRoundStepMessage) String() string {
	return fmt.Sprintf("[NewRoundStep H:%v R:%v S:%v LCR:%v]",
		m.Height, m.Round, m.Step, m.LastCommitRound)
}

//-------------------------------------

// NewValidBlockMessage is sent when a validator observes a valid block B in some round r,
// i.e., there is a Proposal for block B and 2/3+ prevotes for the block B in the round r.
// In case the block is also committed, then IsCommit flag is set to true.
type NewValidBlockMessage struct {
	Height             int64
	Round              int32
	BlockPartSetHeader types.PartSetHeader
	BlockParts         *bits.BitArray
	IsCommit           bool
}

// ValidateBasic performs basic validation.
func (m *NewValidBlockMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.Round < 0 {
		return errors.New("negative Round")
	}
	if err := m.BlockPartSetHeader.ValidateBasic(); err != nil {
		return fmt.Errorf("wrong BlockPartSetHeader: %v", err)
	}
	if m.BlockParts.Size() == 0 {
		return errors.New("empty blockParts")
	}
	if m.BlockParts.Size() != int(m.BlockPartSetHeader.Total) {
		return fmt.Errorf("blockParts bit array size %d not equal to BlockPartSetHeader.Total %d",
			m.BlockParts.Size(),
			m.BlockPartSetHeader.Total)
	}
	if m.BlockParts.Size() > int(types.MaxBlockPartsCount) {
		return fmt.Errorf("blockParts bit array is too big: %d, max: %d", m.BlockParts.Size(), types.MaxBlockPartsCount)
	}
	return nil
}

// String returns a string representation.
func (m *NewValidBlockMessage) String() string {
	return fmt.Sprintf("[ValidBlockMessage H:%v R:%v BP:%v BA:%v IsCommit:%v]",
		m.Height, m.Round, m.BlockPartSetHeader, m.BlockParts, m.IsCommit)
}

//-------------------------------------

// ProposalMessage is sent when a new block is proposed.
type ProposalMessage struct {
	Proposal *types.Proposal
}

// ValidateBasic performs basic validation.
func (m *ProposalMessage) ValidateBasic() error {
	return m.Proposal.ValidateBasic()
}

// String returns a string representation.
func (m *ProposalMessage) String() string {
	return fmt.Sprintf("[Proposal %v]", m.Proposal)
}

//-------------------------------------

// ProposalPOLMessage is sent when a previous proposal is re-proposed.
type ProposalPOLMessage struct {
	Height           int64
	ProposalPOLRound int32
	ProposalPOL      *bits.BitArray
}

// ValidateBasic performs basic validation.
func (m *ProposalPOLMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.ProposalPOLRound < 0 {
		return errors.New("negative ProposalPOLRound")
	}
	if m.ProposalPOL.Size() == 0 {
		return errors.New("empty ProposalPOL bit array")
	}
	if m.ProposalPOL.Size() > types.MaxVotesCount {
		return fmt.Errorf("proposalPOL bit array is too big: %d, max: %d", m.ProposalPOL.Size(), types.MaxVotesCount)
	}
	return nil
}

// String returns a string representation.
func (m *ProposalPOLMessage) String() string {
	return fmt.Sprintf("[ProposalPOL H:%v POLR:%v POL:%v]", m.Height, m.ProposalPOLRound, m.ProposalPOL)
}

//-------------------------------------

// BlockPartMessage is sent when gossipping a piece of the proposed block.
type BlockPartMessage struct {
	Height int64
	Round  int32
	Part   *types.Part
}

// ValidateBasic performs basic validation.
func (m *BlockPartMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.Round < 0 {
		return errors.New("negative Round")
	}
	if err := m.Part.ValidateBasic(); err != nil {
		return fmt.Errorf("wrong Part: %v", err)
	}
	return nil
}

// String returns a string representation.
func (m *BlockPartMessage) String() string {
	return fmt.Sprintf("[BlockPart H:%v R:%v P:%v]", m.Height, m.Round, m.Part)
}

//-------------------------------------

// VoteMessage is sent when voting for a proposal (or lack thereof).
type VoteMessage struct {
	Vote *types.Vote
}

// ValidateBasic checks whether the vote within the message is well-formed.
func (m *VoteMessage) ValidateBasic() error {
	return m.Vote.ValidateBasic()
}

// String returns a string representation.
func (m *VoteMessage) String() string {
	return fmt.Sprintf("[Vote %v]", m.Vote)
}

//-------------------------------------

// HasVoteMessage is sent to indicate that a particular vote has been received.
type HasVoteMessage struct {
	Height int64
	Round  int32
	Type   cmtproto.SignedMsgType
	Index  int32
}

// ValidateBasic performs basic validation.
func (m *HasVoteMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.Round < 0 {
		return errors.New("negative Round")
	}
	if !types.IsVoteTypeValid(m.Type) {
		return errors.New("invalid Type")
	}
	if m.Index < 0 {
		return errors.New("negative Index")
	}
	return nil
}

// String returns a string representation.
func (m *HasVoteMessage) String() string {
	return fmt.Sprintf("[HasVote VI:%v V:{%v/%02d/%v}]", m.Index, m.Height, m.Round, m.Type)
}

//-------------------------------------

// VoteSetMaj23Message is sent to indicate that a given BlockID has seen +2/3 votes.
type VoteSetMaj23Message struct {
	Height  int64
	Round   int32
	Type    cmtproto.SignedMsgType
	BlockID types.BlockID
}

// ValidateBasic performs basic validation.
func (m *VoteSetMaj23Message) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if m.Round < 0 {
		return errors.New("negative Round")
	}
	if !types.IsVoteTypeValid(m.Type) {
		return errors.New("invalid Type")
	}
	if err := m.BlockID.ValidateBasic(); err != nil {
		return fmt.Errorf("wrong BlockID: %v", err)
	}
	return nil
}

// String returns a string representation.
func (m *VoteSetMaj23Message) String() string {
	return fmt.Sprintf("[VSM23 %v/%02d/%v %v]", m.Height, m.Round, m.Type, m.BlockID)
}

//-------------------------------------

// VoteSetBitsMessage is sent to communicate the bit-array of votes seen for the BlockID.
type VoteSetBitsMessage struct {
	Height  int64
	Round   int32
	Type    cmtproto.SignedMsgType
	BlockID types.BlockID
	Votes   *bits.BitArray
}

// ValidateBasic performs basic validation.
func (m *VoteSetBitsMessage) ValidateBasic() error {
	if m.Height < 0 {
		return errors.New("negative Height")
	}
	if !types.IsVoteTypeValid(m.Type) {
		return errors.New("invalid Type")
	}
	if err := m.BlockID.ValidateBasic(); err != nil {
		return fmt.Errorf("wrong BlockID: %v", err)
	}
	// NOTE: Votes.Size() can be zero if the node does not have any
	if m.Votes.Size() > types.MaxVotesCount {
		return fmt.Errorf("votes bit array is too big: %d, max: %d", m.Votes.Size(), types.MaxVotesCount)
	}
	return nil
}

// String returns a string representation.
func (m *VoteSetBitsMessage) String() string {
	return fmt.Sprintf("[VSB %v/%02d/%v %v %v]", m.Height, m.Round, m.Type, m.BlockID, m.Votes)
}

//-------------------------------------
