package statesync

import (
	"context"
	"errors"
	"sort"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/config"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	ssproto "github.com/cometbft/cometbft/proto/tendermint/statesync"
	"github.com/cometbft/cometbft/proxy"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
)

const (
	// SnapshotChannel exchanges snapshot metadata
	SnapshotChannel = byte(0x60)
	// ChunkChannel exchanges chunk contents
	ChunkChannel = byte(0x61)
	// recentSnapshots is the number of recent snapshots to send and receive per peer.
	recentSnapshots = 10
)

// Reactor handles state sync, both restoring snapshots for the local node and serving snapshots
// for other nodes.
type Reactor struct {
	p2p.BaseReactor

	cfg       config.StateSyncConfig
	conn      proxy.AppConnSnapshot
	connQuery proxy.AppConnQuery
	tempDir   string
	metrics   *Metrics

	// This will only be set when a state sync is in progress. It is used to feed received
	// snapshots and chunks into the sync.
	mtx    cmtsync.RWMutex
	syncer *syncer
}

// NewReactor creates a new state sync reactor.
func NewReactor(
	cfg config.StateSyncConfig,
	conn proxy.AppConnSnapshot,
	connQuery proxy.AppConnQuery,
	metrics *Metrics,
) *Reactor {
	r := &Reactor{
		cfg:       cfg,
		conn:      conn,
		connQuery: connQuery,
		metrics:   metrics,
	}
	r.BaseReactor = *p2p.NewBaseReactor("StateSync", r)

	return r
}

// GetChannels implements p2p.Reactor.
func (r *Reactor) GetChannels() []*p2p.ChannelDescriptor {
	return []*p2p.ChannelDescriptor{
		{
			ID:                  SnapshotChannel,
			Priority:            5,
			SendQueueCapacity:   10,
			RecvMessageCapacity: snapshotMsgSize,
			MessageType:         &ssproto.Message{},
		},
		{
			ID:                  ChunkChannel,
			Priority:            3,
			SendQueueCapacity:   10,
			RecvMessageCapacity: chunkMsgSize,
			MessageType:         &ssproto.Message{},
		},
	}
}

// OnStart implements p2p.Reactor.
func (r *Reactor) OnStart() error {
	return nil
}

// AddPeer implements p2p.Reactor.
func (r *Reactor) AddPeer(peer p2p.Peer) {
	r.mtx.RLock()
	defer r.mtx.RUnlock()
	if r.syncer != nil {
		r.syncer.AddPeer(peer)
	}
}

// RemovePeer implements p2p.Reactor.
func (r *Reactor) RemovePeer(peer p2p.Peer, _ interface{}) {
	r.mtx.RLock()
	defer r.mtx.RUnlock()
	if r.syncer != nil {
		r.syncer.RemovePeer(peer)
	}
}

// Receive implements p2p.Reactor.
func (r *Reactor) Receive(e p2p.Envelope) {
	if !r.IsRunning() {
		return
	}

	err := validateMsg(e.Message)
	if err != nil {
		r.Logger.Error("Invalid message", "peer", e.Src, "msg", e.Message, "err", err)
		r.Switch.StopPeerForError(e.Src, err)
		return
	}

	switch e.ChannelID {
	case SnapshotChannel:
		switch msg := e.Message.(type) {
		case *ssproto.SnapshotsRequest:
			snapshots, err := r.recentSnapshots(recentSnapshots)
			if err != nil {
				r.Logger.Error("Failed to fetch snapshots", "err", err)
				return
			}
			for _, snapshot := range snapshots {
				r.Logger.Debug("Advertising snapshot", "height", snapshot.Height,
					"format", snapshot.Format, "peer", e.Src.ID())
				e.Src.Send(p2p.Envelope{
					ChannelID: e.ChannelID,
					Message: &ssproto.SnapshotsResponse{
						Height:   snapshot.Height,
						Format:   snapshot.Format,
						Chunks:   snapshot.Chunks,
						Hash:     snapshot.Hash,
						Metadata: snapshot.Metadata,
					},
				})
			}

		case *ssproto.SnapshotsResponse:
			r.mtx.RLock()
			defer r.mtx.RUnlock()
			if r.syncer == nil {
				r.Logger.Debug("Received unexpected snapshot, no state sync in progress")
				return
			}
			r.Logger.Debug("Received snapshot", "height", msg.Height, "format", msg.Format, "peer", e.Src.ID())
			_, err := r.syncer.AddSnapshot(e.Src, &snapshot{
				Height:   msg.Height,
				Format:   msg.Format,
				Chunks:   msg.Chunks,
				Hash:     msg.Hash,
				Metadata: msg.Metadata,
			})
			// TODO: We may want to consider punishing the peer for certain errors
			if err != nil {
				r.Logger.Error("Failed to add snapshot", "height", msg.Height, "format", msg.Format,
					"peer", e.Src.ID(), "err", err)
				return
			}

		default:
			r.Logger.Error("Received unknown message %T", msg)
		}

	case ChunkChannel:
		switch msg := e.Message.(type) {
		case *ssproto.ChunkRequest:
			r.Logger.Debug("Received chunk request", "height", msg.Height, "format", msg.Format,
				"chunk", msg.Index, "peer", e.Src.ID())
			resp, err := r.conn.LoadSnapshotChunk(context.TODO(), &abci.RequestLoadSnapshotChunk{
				Height: msg.Height,
				Format: msg.Format,
				Chunk:  msg.Index,
			})
			if err != nil {
				r.Logger.Error("Failed to load chunk", "height", msg.Height, "format", msg.Format,
					"chunk", msg.Index, "err", err)
				return
			}
			r.Logger.Debug("Sending chunk", "height", msg.Height, "format", msg.Format,
				"chunk", msg.Index, "peer", e.Src.ID())
			e.Src.Send(p2p.Envelope{
				ChannelID: ChunkChannel,
				Message: &ssproto.ChunkResponse{
					Height:  msg.Height,
					Format:  msg.Format,
					Index:   msg.Index,
					Chunk:   resp.Chunk,
					Missing: resp.Chunk == nil,
				},
			})

		case *ssproto.ChunkResponse:
			r.mtx.RLock()
			defer r.mtx.RUnlock()
			if r.syncer == nil {
				r.Logger.Debug("Received unexpected chunk, no state sync in progress", "peer", e.Src.ID())
				return
			}
			r.Logger.Debug("Received chunk, adding to sync", "height", msg.Height, "format", msg.Format,
				"chunk", msg.Index, "peer", e.Src.ID())
			_, err := r.syncer.AddChunk(&chunk{
				Height: msg.Height,
				Format: msg.Format,
				Index:  msg.Index,
				Chunk:  msg.Chunk,
				Sender: e.Src.ID(),
			})
			if err != nil {
				r.Logger.Error("Failed to add chunk", "height", msg.Height, "format", msg.Format,
					"chunk", msg.Index, "err", err)
				return
			}

		default:
			r.Logger.Error("Received unknown message %T", msg)
		}

	default:
		r.Logger.Error("Received message on invalid channel %x", e.ChannelID)
	}
}

// recentSnapshots fetches the n most recent snapshots from the app
func (r *Reactor) recentSnapshots(n uint32) ([]*snapshot, error) {
	resp, err := r.conn.ListSnapshots(context.TODO(), &abci.RequestListSnapshots{})
	if err != nil {
		return nil, err
	}
	sort.Slice(resp.Snapshots, func(i, j int) bool {
		a := resp.Snapshots[i]
		b := resp.Snapshots[j]
		switch {
		case a.Height > b.Height:
			return true
		case a.Height == b.Height && a.Format > b.Format:
			return true
		default:
			return false
		}
	})
	snapshots := make([]*snapshot, 0, n)
	for i, s := range resp.Snapshots {
		if i >= recentSnapshots {
			break
		}
		snapshots = append(snapshots, &snapshot{
			Height:   s.Height,
			Format:   s.Format,
			Chunks:   s.Chunks,
			Hash:     s.Hash,
			Metadata: s.Metadata,
		})
	}
	return snapshots, nil
}

// Sync runs a state sync, returning the new state and last commit at the snapshot height.
// The caller must store the state and commit in the state database and block store.
func (r *Reactor) Sync(stateProvider StateProvider, discoveryTime time.Duration) (sm.State, *types.Commit, error) {
	r.mtx.Lock()
	if r.syncer != nil {
		r.mtx.Unlock()
		return sm.State{}, nil, errors.New("a state sync is already in progress")
	}
	r.metrics.Syncing.Set(1)
	r.syncer = newSyncer(r.cfg, r.Logger, r.conn, r.connQuery, stateProvider, r.tempDir)
	r.mtx.Unlock()

	hook := func() {
		r.Logger.Debug("Requesting snapshots from known peers")
		// Request snapshots from all currently connected peers

		r.Switch.Broadcast(p2p.Envelope{
			ChannelID: SnapshotChannel,
			Message:   &ssproto.SnapshotsRequest{},
		})
	}

	hook()

	state, commit, err := r.syncer.SyncAny(discoveryTime, hook)

	r.mtx.Lock()
	r.syncer = nil
	r.metrics.Syncing.Set(0)
	r.mtx.Unlock()
	return state, commit, err
}
