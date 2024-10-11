package sync

import (
	"context"
	"testing"
	"time"

	gcache "github.com/patrickmn/go-cache"
	"github.com/prysmaticlabs/prysm/v4/async/abool"
	mockChain "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestService_StatusZeroEpoch(t *testing.T) {
	bState, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{Slot: 0})
	require.NoError(t, err)
	chain := &mockChain.ChainService{
		Genesis: time.Now(),
		State:   bState,
	}
	r := &Service{
		cfg: &config{
			p2p:         p2ptest.NewTestP2P(t),
			initialSync: new(mockSync.Sync),
			chain:       chain,
			clock:       startup.NewClock(chain.Genesis, chain.ValidatorsRoot),
		},
		chainStarted: abool.New(),
	}
	r.chainStarted.Set()

	assert.NoError(t, r.Status(), "Wanted non failing status")
}

func TestSyncHandlers_WaitToSync(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	gs := startup.NewClockSynchronizer()
	r := Service{
		ctx: context.Background(),
		cfg: &config{
			p2p:         p2p,
			chain:       chainService,
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		chainStarted: abool.New(),
		clockWaiter:  gs,
	}

	topic := "/eth2/%x/beacon_block"
	go r.registerHandlers()
	time.Sleep(100 * time.Millisecond)

	var vr [32]byte
	require.NoError(t, gs.SetClock(startup.NewClock(time.Now(), vr)))
	b := []byte("sk")
	b32 := bytesutil.ToBytes32(b)
	sk, err := bls.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)

	msg := util.NewBeaconBlock()
	msg.Block.ParentRoot = util.Random32Bytes(t)
	msg.Signature = sk.Sign([]byte("data")).Marshal()
	p2p.ReceivePubSub(topic, msg)
	// wait for chainstart to be sent
	time.Sleep(400 * time.Millisecond)
	require.Equal(t, true, r.chainStarted.IsSet(), "Did not receive chain start event.")
}

func TestSyncHandlers_WaitForChainStart(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	gs := startup.NewClockSynchronizer()
	r := Service{
		ctx: context.Background(),
		cfg: &config{
			p2p:         p2p,
			chain:       chainService,
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		chainStarted:        abool.New(),
		slotToPendingBlocks: gcache.New(time.Second, 2*time.Second),
		clockWaiter:         gs,
	}

	var vr [32]byte
	require.NoError(t, gs.SetClock(startup.NewClock(time.Now(), vr)))
	r.waitForChainStart()

	require.Equal(t, true, r.chainStarted.IsSet(), "Did not receive chain start event.")
}

func TestSyncHandlers_WaitTillSynced(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()
	gs := startup.NewClockSynchronizer()
	r := Service{
		ctx: ctx,
		cfg: &config{
			p2p:           p2p,
			beaconDB:      dbTest.SetupDB(t),
			chain:         chainService,
			blockNotifier: chainService.BlockNotifier(),
			initialSync:   &mockSync.Sync{IsSyncing: false},
		},
		chainStarted:        abool.New(),
		subHandler:          newSubTopicHandler(),
		clockWaiter:         gs,
		initialSyncComplete: make(chan struct{}),
	}
	r.initCaches()

	var vr [32]byte
	require.NoError(t, gs.SetClock(startup.NewClock(time.Now(), vr)))
	r.waitForChainStart()
	require.Equal(t, true, r.chainStarted.IsSet(), "Did not receive chain start event.")

	var err error
	p2p.Digest, err = r.currentForkDigest()
	require.NoError(t, err)

	syncCompleteCh := make(chan bool)
	go func() {
		r.registerHandlers()
		syncCompleteCh <- true
	}()

	blockChan := make(chan *feed.Event, 1)
	sub := r.cfg.blockNotifier.BlockFeed().Subscribe(blockChan)
	defer sub.Unsubscribe()

	b := []byte("sk")
	b32 := bytesutil.ToBytes32(b)
	sk, err := bls.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)
	msg := util.NewBeaconBlock()
	msg.Block.ParentRoot = util.Random32Bytes(t)
	msg.Signature = sk.Sign([]byte("data")).Marshal()

	// Save block into DB so that validateBeaconBlockPubSub() process gets short cut.
	util.SaveBlock(t, ctx, r.cfg.beaconDB, msg)

	topic := "/eth2/%x/beacon_block"
	p2p.ReceivePubSub(topic, msg)
	assert.Equal(t, 0, len(blockChan), "block was received by sync service despite not being fully synced")

	close(r.initialSyncComplete)
	<-syncCompleteCh

	p2p.ReceivePubSub(topic, msg)

	select {
	case <-blockChan:
	case <-ctx.Done():
	}
	assert.NoError(t, ctx.Err())
}

func TestSyncService_StopCleanly(t *testing.T) {
	p2p := p2ptest.NewTestP2P(t)
	chainService := &mockChain.ChainService{
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{'A'},
	}
	ctx, cancel := context.WithCancel(context.Background())
	gs := startup.NewClockSynchronizer()
	r := Service{
		ctx:    ctx,
		cancel: cancel,
		cfg: &config{
			p2p:         p2p,
			chain:       chainService,
			initialSync: &mockSync.Sync{IsSyncing: false},
		},
		chainStarted:        abool.New(),
		subHandler:          newSubTopicHandler(),
		clockWaiter:         gs,
		initialSyncComplete: make(chan struct{}),
	}

	go r.registerHandlers()
	var vr [32]byte
	require.NoError(t, gs.SetClock(startup.NewClock(time.Now(), vr)))
	r.waitForChainStart()

	var err error
	p2p.Digest, err = r.currentForkDigest()
	require.NoError(t, err)

	// wait for chainstart to be sent
	time.Sleep(2 * time.Second)
	require.Equal(t, true, r.chainStarted.IsSet(), "Did not receive chain start event.")

	close(r.initialSyncComplete)
	time.Sleep(1 * time.Second)

	require.NotEqual(t, 0, len(r.cfg.p2p.PubSub().GetTopics()))
	require.NotEqual(t, 0, len(r.cfg.p2p.Host().Mux().Protocols()))

	// Both pubsub and rpc topcis should be unsubscribed.
	require.NoError(t, r.Stop())

	// Sleep to allow pubsub topics to be deregistered.
	time.Sleep(1 * time.Second)
	require.Equal(t, 0, len(r.cfg.p2p.PubSub().GetTopics()))
	require.Equal(t, 0, len(r.cfg.p2p.Host().Mux().Protocols()))
}
