package blockchain

import (
	"context"
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache/depositcache"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	testDB "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/protobuf/proto"
)

type mockBeaconNode struct {
	stateFeed *event.Feed
	mu        sync.Mutex
}

// StateFeed mocks the same method in the beacon node.
func (mbn *mockBeaconNode) StateFeed() *event.Feed {
	mbn.mu.Lock()
	defer mbn.mu.Unlock()
	if mbn.stateFeed == nil {
		mbn.stateFeed = new(event.Feed)
	}
	return mbn.stateFeed
}

type mockBroadcaster struct {
	broadcastCalled bool
}

func (mb *mockBroadcaster) Broadcast(_ context.Context, _ proto.Message) error {
	mb.broadcastCalled = true
	return nil
}

func (mb *mockBroadcaster) BroadcastAttestation(_ context.Context, _ uint64, _ *ethpb.Attestation) error {
	mb.broadcastCalled = true
	return nil
}

func (mb *mockBroadcaster) BroadcastSyncCommitteeMessage(_ context.Context, _ uint64, _ *ethpb.SyncCommitteeMessage) error {
	mb.broadcastCalled = true
	return nil
}

func (mb *mockBroadcaster) BroadcastBlob(_ context.Context, _ uint64, _ *ethpb.BlobSidecar) error {
	mb.broadcastCalled = true
	return nil
}

func (mb *mockBroadcaster) BroadcastBLSChanges(_ context.Context, _ []*ethpb.SignedBLSToExecutionChange) {
}

var _ p2p.Broadcaster = (*mockBroadcaster)(nil)

type testServiceRequirements struct {
	ctx     context.Context
	db      db.Database
	fcs     forkchoice.ForkChoicer
	sg      *stategen.State
	notif   statefeed.Notifier
	cs      *startup.ClockSynchronizer
	attPool attestations.Pool
	attSrv  *attestations.Service
	blsPool *blstoexec.Pool
	dc      *depositcache.DepositCache
}

func minimalTestService(t *testing.T, opts ...Option) (*Service, *testServiceRequirements) {
	ctx := context.Background()
	beaconDB := testDB.SetupDB(t)
	fcs := doublylinkedtree.New()
	sg := stategen.New(beaconDB, fcs)
	notif := &mockBeaconNode{}
	fcs.SetBalancesByRooter(sg.ActiveNonSlashedBalancesByRoot)
	cs := startup.NewClockSynchronizer()
	attPool := attestations.NewPool()
	attSrv, err := attestations.NewService(ctx, &attestations.Config{Pool: attPool})
	require.NoError(t, err)
	blsPool := blstoexec.NewPool()
	dc, err := depositcache.New()
	require.NoError(t, err)
	req := &testServiceRequirements{
		ctx:     ctx,
		db:      beaconDB,
		fcs:     fcs,
		sg:      sg,
		notif:   notif,
		cs:      cs,
		attPool: attPool,
		attSrv:  attSrv,
		blsPool: blsPool,
		dc:      dc,
	}
	defOpts := []Option{WithDatabase(req.db),
		WithStateNotifier(req.notif),
		WithStateGen(req.sg),
		WithForkChoiceStore(req.fcs),
		WithClockSynchronizer(req.cs),
		WithAttestationPool(req.attPool),
		WithAttestationService(req.attSrv),
		WithBLSToExecPool(req.blsPool),
		WithDepositCache(dc),
		WithTrackedValidatorsCache(cache.NewTrackedValidatorsCache()),
		WithBlobStorage(filesystem.NewEphemeralBlobStorage(t)),
	}
	// append the variadic opts so they override the defaults by being processed afterwards
	opts = append(defOpts, opts...)
	s, err := NewService(req.ctx, opts...)

	require.NoError(t, err)
	return s, req
}
