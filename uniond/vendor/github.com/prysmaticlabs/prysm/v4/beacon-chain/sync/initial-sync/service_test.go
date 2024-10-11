package initialsync

import (
	"context"
	"sync"
	"testing"
	"time"

	"github.com/paulbellamy/ratecounter"
	"github.com/prysmaticlabs/prysm/v4/async/abool"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	p2pt "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestService_Constants(t *testing.T) {
	if params.BeaconConfig().MaxPeersToSync*flags.Get().BlockBatchLimit > 1000 {
		t.Fatal("rpc rejects requests over 1000 range slots")
	}
}

func TestService_InitStartStop(t *testing.T) {
	hook := logTest.NewGlobal()
	resetFlags := flags.Get()
	flags.Init(&flags.GlobalFlags{
		MinimumSyncPeers: 1,
	})
	defer func() {
		flags.Init(resetFlags)
	}()

	tests := []struct {
		name         string
		assert       func()
		setGenesis   func() *startup.Clock
		chainService func() *mock.ChainService
	}{
		{
			name: "head is not ready",
			assert: func() {
				assert.LogsContain(t, hook, "Waiting for state to be initialized")
			},
		},
		{
			name: "future genesis",
			chainService: func() *mock.ChainService {
				// Set to future time (genesis time hasn't arrived yet).
				st, err := util.NewBeaconState()
				require.NoError(t, err)

				return &mock.ChainService{
					State: st,
					FinalizedCheckPoint: &eth.Checkpoint{
						Epoch: 0,
					},
					Genesis:        time.Unix(4113849600, 0),
					ValidatorsRoot: [32]byte{},
				}
			},
			setGenesis: func() *startup.Clock {
				var vr [32]byte
				return startup.NewClock(time.Unix(4113849600, 0), vr)
			},
			assert: func() {
				assert.LogsContain(t, hook, "Genesis time has not arrived - not syncing")
				assert.LogsContain(t, hook, "Waiting for state to be initialized")
			},
		},
		{
			name: "zeroth epoch",
			chainService: func() *mock.ChainService {
				// Set to nearby slot.
				st, err := util.NewBeaconState()
				require.NoError(t, err)
				return &mock.ChainService{
					State: st,
					FinalizedCheckPoint: &eth.Checkpoint{
						Epoch: 0,
					},
					Genesis:        time.Now().Add(-5 * time.Minute),
					ValidatorsRoot: [32]byte{},
				}
			},
			setGenesis: func() *startup.Clock {
				var vr [32]byte
				return startup.NewClock(time.Now().Add(-5*time.Minute), vr)
			},
			assert: func() {
				assert.LogsContain(t, hook, "Chain started within the last epoch - not syncing")
				assert.LogsDoNotContain(t, hook, "Genesis time has not arrived - not syncing")
				assert.LogsContain(t, hook, "Waiting for state to be initialized")
			},
		},
		{
			name: "already synced",
			chainService: func() *mock.ChainService {
				// Set to some future slot, and then make sure that current head matches it.
				st, err := util.NewBeaconState()
				require.NoError(t, err)
				futureSlot := primitives.Slot(27354)
				require.NoError(t, st.SetSlot(futureSlot))
				return &mock.ChainService{
					State: st,
					FinalizedCheckPoint: &eth.Checkpoint{
						Epoch: slots.ToEpoch(futureSlot),
					},
					Genesis:        makeGenesisTime(futureSlot),
					ValidatorsRoot: [32]byte{},
				}
			},
			setGenesis: func() *startup.Clock {
				futureSlot := primitives.Slot(27354)
				var vr [32]byte
				return startup.NewClock(makeGenesisTime(futureSlot), vr)
			},
			assert: func() {
				assert.LogsContain(t, hook, "Starting initial chain sync...")
				assert.LogsContain(t, hook, "Already synced to the current chain head")
				assert.LogsDoNotContain(t, hook, "Chain started within the last epoch - not syncing")
				assert.LogsDoNotContain(t, hook, "Genesis time has not arrived - not syncing")
				assert.LogsContain(t, hook, "Waiting for state to be initialized")
			},
		},
	}

	p := p2pt.NewTestP2P(t)
	connectPeers(t, p, []*peerData{}, p.Peers())
	for i, tt := range tests {
		if i == 0 {
			continue
		}
		t.Run(tt.name, func(t *testing.T) {
			defer hook.Reset()
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			mc := &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}}
			// Allow overriding with customized chain service.
			if tt.chainService != nil {
				mc = tt.chainService()
			}
			// Initialize feed
			gs := startup.NewClockSynchronizer()
			s := NewService(ctx, &Config{
				P2P:                 p,
				Chain:               mc,
				ClockWaiter:         gs,
				StateNotifier:       &mock.MockStateNotifier{},
				InitialSyncComplete: make(chan struct{}),
			})
			s.verifierWaiter = verification.NewInitializerWaiter(gs, nil, nil)
			time.Sleep(500 * time.Millisecond)
			assert.NotNil(t, s)
			if tt.setGenesis != nil {
				require.NoError(t, gs.SetClock(tt.setGenesis()))
			}

			wg := &sync.WaitGroup{}
			wg.Add(1)
			go func() {
				s.Start()
				wg.Done()
			}()

			go func() {
				// Allow to exit from test (on no head loop waiting for head is started).
				// In most tests, this is redundant, as Start() already exited.
				time.AfterFunc(3*time.Second, func() {
					cancel()
				})
			}()
			if util.WaitTimeout(wg, time.Second*4) {
				t.Fatalf("Test should have exited by now, timed out")
			}
			tt.assert()
		})
	}
}

func TestService_waitForStateInitialization(t *testing.T) {
	hook := logTest.NewGlobal()
	newService := func(ctx context.Context, mc *mock.ChainService) (*Service, *startup.ClockSynchronizer) {
		cs := startup.NewClockSynchronizer()
		ctx, cancel := context.WithCancel(ctx)
		s := &Service{
			cfg:          &Config{Chain: mc, StateNotifier: mc.StateNotifier(), ClockWaiter: cs, InitialSyncComplete: make(chan struct{})},
			ctx:          ctx,
			cancel:       cancel,
			synced:       abool.New(),
			chainStarted: abool.New(),
			counter:      ratecounter.NewRateCounter(counterSeconds * time.Second),
			genesisChan:  make(chan time.Time),
		}
		s.verifierWaiter = verification.NewInitializerWaiter(cs, nil, nil)
		return s, cs
	}

	t.Run("no state and context close", func(t *testing.T) {
		defer hook.Reset()
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		s, _ := newService(ctx, &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}})
		wg := &sync.WaitGroup{}
		wg.Add(1)
		go func() {
			s.Start()
			wg.Done()
		}()
		go func() {
			time.AfterFunc(500*time.Millisecond, func() {
				cancel()
			})
		}()

		if util.WaitTimeout(wg, time.Second*2) {
			t.Fatalf("Test should have exited by now, timed out")
		}
		assert.LogsContain(t, hook, "Waiting for state to be initialized")
		assert.LogsContain(t, hook, "initial-sync failed to receive startup event")
		assert.LogsDoNotContain(t, hook, "Subscription to state notifier failed")
	})

	t.Run("no state and state init event received", func(t *testing.T) {
		defer hook.Reset()
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()

		st, err := util.NewBeaconState()
		require.NoError(t, err)
		gt := time.Unix(int64(st.GenesisTime()), 0)
		s, gs := newService(ctx, &mock.ChainService{State: st, Genesis: gt, ValidatorsRoot: [32]byte{}})

		expectedGenesisTime := gt
		wg := &sync.WaitGroup{}
		wg.Add(1)
		go func() {
			s.Start()
			wg.Done()
		}()
		rg := func() time.Time { return gt.Add(time.Second * 12) }
		go func() {
			time.AfterFunc(200*time.Millisecond, func() {
				var vr [32]byte
				require.NoError(t, gs.SetClock(startup.NewClock(expectedGenesisTime, vr, startup.WithNower(rg))))
			})
		}()

		if util.WaitTimeout(wg, time.Second*2) {
			t.Fatalf("Test should have exited by now, timed out")
		}
		assert.LogsContain(t, hook, "Waiting for state to be initialized")
		assert.LogsContain(t, hook, "Received state initialized event")
		assert.LogsDoNotContain(t, hook, "Context closed, exiting goroutine")
	})

	t.Run("no state and state init event received and service start", func(t *testing.T) {
		defer hook.Reset()
		ctx, cancel := context.WithCancel(context.Background())
		defer cancel()
		s, gs := newService(ctx, &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}})
		// Initialize mock feed
		_ = s.cfg.StateNotifier.StateFeed()

		expectedGenesisTime := time.Now().Add(60 * time.Second)
		wg := &sync.WaitGroup{}
		wg.Add(1)
		go func() {
			time.AfterFunc(500*time.Millisecond, func() {
				var vr [32]byte
				require.NoError(t, gs.SetClock(startup.NewClock(expectedGenesisTime, vr)))
			})
			s.Start()
			wg.Done()
		}()

		if util.WaitTimeout(wg, time.Second*5) {
			t.Fatalf("Test should have exited by now, timed out")
		}
		assert.LogsContain(t, hook, "Waiting for state to be initialized")
		assert.LogsContain(t, hook, "Received state initialized event")
		assert.LogsDoNotContain(t, hook, "Context closed, exiting goroutine")
	})
}

func TestService_markSynced(t *testing.T) {
	mc := &mock.ChainService{Genesis: time.Now(), ValidatorsRoot: [32]byte{}}
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	s := NewService(ctx, &Config{
		Chain:               mc,
		StateNotifier:       mc.StateNotifier(),
		InitialSyncComplete: make(chan struct{}),
	})
	require.NotNil(t, s)
	assert.Equal(t, false, s.chainStarted.IsSet())
	assert.Equal(t, false, s.synced.IsSet())
	assert.Equal(t, true, s.Syncing())
	assert.NoError(t, s.Status())
	s.chainStarted.Set()
	assert.ErrorContains(t, "syncing", s.Status())

	go func() {
		s.markSynced()
	}()

	select {
	case <-s.cfg.InitialSyncComplete:
	case <-ctx.Done():
		require.NoError(t, ctx.Err()) // this is an error because it means initial sync complete failed to close
	}

	assert.Equal(t, false, s.Syncing())
}

func TestService_Resync(t *testing.T) {
	p := p2pt.NewTestP2P(t)
	connectPeers(t, p, []*peerData{
		{blocks: makeSequence(1, 160), finalizedEpoch: 5, headSlot: 160},
	}, p.Peers())
	cache.initializeRootCache(makeSequence(1, 160), t)
	beaconDB := dbtest.SetupDB(t)
	util.SaveBlock(t, context.Background(), beaconDB, util.NewBeaconBlock())
	cache.RLock()
	genesisRoot := cache.rootCache[0]
	cache.RUnlock()

	hook := logTest.NewGlobal()
	tests := []struct {
		name         string
		assert       func(s *Service)
		chainService func() *mock.ChainService
		wantedErr    string
	}{
		{
			name:      "no head state",
			wantedErr: "could not retrieve head state",
		},
		{
			name: "resync ok",
			chainService: func() *mock.ChainService {
				st, err := util.NewBeaconState()
				require.NoError(t, err)
				futureSlot := primitives.Slot(160)
				require.NoError(t, st.SetGenesisTime(uint64(makeGenesisTime(futureSlot).Unix())))
				return &mock.ChainService{
					State: st,
					Root:  genesisRoot[:],
					DB:    beaconDB,
					FinalizedCheckPoint: &eth.Checkpoint{
						Epoch: slots.ToEpoch(futureSlot),
					},
					Genesis:        time.Now(),
					ValidatorsRoot: [32]byte{},
				}
			},
			assert: func(s *Service) {
				assert.LogsContain(t, hook, "Resync attempt complete")
				assert.Equal(t, primitives.Slot(160), s.cfg.Chain.HeadSlot())
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			defer hook.Reset()
			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()
			mc := &mock.ChainService{}
			// Allow overriding with customized chain service.
			if tt.chainService != nil {
				mc = tt.chainService()
			}
			s := NewService(ctx, &Config{
				DB:            beaconDB,
				P2P:           p,
				Chain:         mc,
				StateNotifier: mc.StateNotifier(),
				BlobStorage:   filesystem.NewEphemeralBlobStorage(t),
			})
			assert.NotNil(t, s)
			assert.Equal(t, primitives.Slot(0), s.cfg.Chain.HeadSlot())
			err := s.Resync()
			if tt.wantedErr != "" {
				assert.ErrorContains(t, tt.wantedErr, err)
			} else {
				assert.NoError(t, err)
			}
			if tt.assert != nil {
				tt.assert(s)
			}
		})
	}
}

func TestService_Initialized(t *testing.T) {
	s := NewService(context.Background(), &Config{
		StateNotifier: &mock.MockStateNotifier{},
	})
	s.chainStarted.Set()
	assert.Equal(t, true, s.Initialized())
	s.chainStarted.UnSet()
	assert.Equal(t, false, s.Initialized())
}

func TestService_Synced(t *testing.T) {
	s := NewService(context.Background(), &Config{})
	s.synced.UnSet()
	assert.Equal(t, false, s.Synced())
	s.synced.Set()
	assert.Equal(t, true, s.Synced())
}
