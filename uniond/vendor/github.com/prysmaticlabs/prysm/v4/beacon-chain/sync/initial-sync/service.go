// Package initialsync includes all initial block download and processing
// logic for the beacon node, using a round robin strategy and a finite-state-machine
// to handle edge-cases in a beacon node's sync status.
package initialsync

import (
	"context"
	"time"

	"github.com/paulbellamy/ratecounter"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async/abool"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

var _ runtime.Service = (*Service)(nil)

// blockchainService defines the interface for interaction with block chain service.
type blockchainService interface {
	blockchain.BlockReceiver
	blockchain.ChainInfoFetcher
}

// Config to set up the initial sync service.
type Config struct {
	P2P                 p2p.P2P
	DB                  db.NoHeadAccessDatabase
	Chain               blockchainService
	StateNotifier       statefeed.Notifier
	BlockNotifier       blockfeed.Notifier
	ClockWaiter         startup.ClockWaiter
	InitialSyncComplete chan struct{}
	BlobStorage         *filesystem.BlobStorage
}

// Service service.
type Service struct {
	cfg             *Config
	ctx             context.Context
	cancel          context.CancelFunc
	synced          *abool.AtomicBool
	chainStarted    *abool.AtomicBool
	counter         *ratecounter.RateCounter
	genesisChan     chan time.Time
	clock           *startup.Clock
	verifierWaiter  *verification.InitializerWaiter
	newBlobVerifier verification.NewBlobVerifier
}

// Option is a functional option for the initial-sync Service.
type Option func(*Service)

// WithVerifierWaiter sets the verification.InitializerWaiter
// for the initial-sync Service.
func WithVerifierWaiter(viw *verification.InitializerWaiter) Option {
	return func(s *Service) {
		s.verifierWaiter = viw
	}
}

// NewService configures the initial sync service responsible for bringing the node up to the
// latest head of the blockchain.
func NewService(ctx context.Context, cfg *Config, opts ...Option) *Service {
	ctx, cancel := context.WithCancel(ctx)
	s := &Service{
		cfg:          cfg,
		ctx:          ctx,
		cancel:       cancel,
		synced:       abool.New(),
		chainStarted: abool.New(),
		counter:      ratecounter.NewRateCounter(counterSeconds * time.Second),
		genesisChan:  make(chan time.Time),
		clock:        startup.NewClock(time.Unix(0, 0), [32]byte{}), // default clock to prevent panic
	}
	for _, o := range opts {
		o(s)
	}
	return s
}

// Start the initial sync service.
func (s *Service) Start() {
	log.Info("Waiting for state to be initialized")
	clock, err := s.cfg.ClockWaiter.WaitForClock(s.ctx)
	if err != nil {
		log.WithError(err).Error("initial-sync failed to receive startup event")
		return
	}
	s.clock = clock
	log.Info("Received state initialized event")

	v, err := s.verifierWaiter.WaitForInitializer(s.ctx)
	if err != nil {
		log.WithError(err).Error("Could not get verification initializer")
		return
	}
	s.newBlobVerifier = newBlobVerifierFromInitializer(v)

	gt := clock.GenesisTime()
	if gt.IsZero() {
		log.Debug("Exiting Initial Sync Service")
		return
	}
	// Exit entering round-robin sync if we require 0 peers to sync.
	if flags.Get().MinimumSyncPeers == 0 {
		s.markSynced()
		log.WithField("genesisTime", gt).Info("Due to number of peers required for sync being set at 0, entering regular sync immediately.")
		return
	}
	if gt.After(prysmTime.Now()) {
		s.markSynced()
		log.WithField("genesisTime", gt).Info("Genesis time has not arrived - not syncing")
		return
	}
	currentSlot := clock.CurrentSlot()
	if slots.ToEpoch(currentSlot) == 0 {
		log.WithField("genesisTime", gt).Info("Chain started within the last epoch - not syncing")
		s.markSynced()
		return
	}
	s.chainStarted.Set()
	log.Info("Starting initial chain sync...")
	// Are we already in sync, or close to it?
	if slots.ToEpoch(s.cfg.Chain.HeadSlot()) == slots.ToEpoch(currentSlot) {
		log.Info("Already synced to the current chain head")
		s.markSynced()
		return
	}
	s.waitForMinimumPeers()
	if err := s.roundRobinSync(gt); err != nil {
		if errors.Is(s.ctx.Err(), context.Canceled) {
			return
		}
		panic(err)
	}
	log.Infof("Synced up to slot %d", s.cfg.Chain.HeadSlot())
	s.markSynced()
}

// Stop initial sync.
func (s *Service) Stop() error {
	s.cancel()
	return nil
}

// Status of initial sync.
func (s *Service) Status() error {
	if s.synced.IsNotSet() && s.chainStarted.IsSet() {
		return errors.New("syncing")
	}
	return nil
}

// Syncing returns true if initial sync is still running.
func (s *Service) Syncing() bool {
	return s.synced.IsNotSet()
}

// Initialized returns true if initial sync has been started.
func (s *Service) Initialized() bool {
	return s.chainStarted.IsSet()
}

// Synced returns true if initial sync has been completed.
func (s *Service) Synced() bool {
	return s.synced.IsSet()
}

// Resync allows a node to start syncing again if it has fallen
// behind the current network head.
func (s *Service) Resync() error {
	headState, err := s.cfg.Chain.HeadState(s.ctx)
	if err != nil || headState == nil || headState.IsNil() {
		return errors.Errorf("could not retrieve head state: %v", err)
	}

	// Set it to false since we are syncing again.
	s.synced.UnSet()
	defer func() { s.synced.Set() }()                       // Reset it at the end of the method.
	genesis := time.Unix(int64(headState.GenesisTime()), 0) // lint:ignore uintcast -- Genesis time will not exceed int64 in your lifetime.

	s.waitForMinimumPeers()
	if err = s.roundRobinSync(genesis); err != nil {
		log = log.WithError(err)
	}
	log.WithField("slot", s.cfg.Chain.HeadSlot()).Info("Resync attempt complete")
	return nil
}

func (s *Service) waitForMinimumPeers() {
	required := params.BeaconConfig().MaxPeersToSync
	if flags.Get().MinimumSyncPeers < required {
		required = flags.Get().MinimumSyncPeers
	}
	for {
		cp := s.cfg.Chain.FinalizedCheckpt()
		_, peers := s.cfg.P2P.Peers().BestNonFinalized(flags.Get().MinimumSyncPeers, cp.Epoch)
		if len(peers) >= required {
			break
		}
		log.WithFields(logrus.Fields{
			"suitable": len(peers),
			"required": required,
		}).Info("Waiting for enough suitable peers before syncing")
		time.Sleep(handshakePollingInterval)
	}
}

// markSynced marks node as synced and notifies feed listeners.
func (s *Service) markSynced() {
	s.synced.Set()
	close(s.cfg.InitialSyncComplete)
}
