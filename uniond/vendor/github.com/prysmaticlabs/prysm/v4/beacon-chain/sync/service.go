// Package sync includes all chain-synchronization logic for the beacon node,
// including gossip-sub validators for blocks, attestations, and other p2p
// messages, as well as ability to process and respond to block requests
// by peers.
package sync

import (
	"context"
	"sync"
	"time"

	lru "github.com/hashicorp/golang-lru"
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/peer"
	gcache "github.com/patrickmn/go-cache"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/async"
	"github.com/prysmaticlabs/prysm/v4/async/abool"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	blockfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/block"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/synccommittee"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	lruwrpr "github.com/prysmaticlabs/prysm/v4/cache/lru"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/trailofbits/go-mutexasserts"
)

var _ runtime.Service = (*Service)(nil)

const rangeLimit uint64 = 1024
const seenBlockSize = 1000
const seenBlobSize = seenBlockSize * 4 // Each block can have max 4 blobs. Worst case 164kB for cache.
const seenUnaggregatedAttSize = 20000
const seenAggregatedAttSize = 1024
const seenSyncMsgSize = 1000         // Maximum of 512 sync committee members, 1000 is a safe amount.
const seenSyncContributionSize = 512 // Maximum of SYNC_COMMITTEE_SIZE as specified by the spec.
const seenExitSize = 100
const seenProposerSlashingSize = 100
const badBlockSize = 1000
const syncMetricsInterval = 10 * time.Second

var (
	// Seconds in one epoch.
	pendingBlockExpTime = time.Duration(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot)) * time.Second
	// time to allow processing early blocks.
	earlyBlockProcessingTolerance = slots.MultiplySlotBy(2)
	// time to allow processing early attestations.
	earlyAttestationProcessingTolerance = params.BeaconConfig().MaximumGossipClockDisparityDuration()
	errWrongMessage                     = errors.New("wrong pubsub message")
	errNilMessage                       = errors.New("nil pubsub message")
)

// Common type for functional p2p validation options.
type validationFn func(ctx context.Context) (pubsub.ValidationResult, error)

// config to hold dependencies for the sync service.
type config struct {
	attestationNotifier           operation.Notifier
	p2p                           p2p.P2P
	beaconDB                      db.NoHeadAccessDatabase
	attPool                       attestations.Pool
	exitPool                      voluntaryexits.PoolManager
	slashingPool                  slashings.PoolManager
	syncCommsPool                 synccommittee.Pool
	blsToExecPool                 blstoexec.PoolManager
	chain                         blockchainService
	initialSync                   Checker
	blockNotifier                 blockfeed.Notifier
	operationNotifier             operation.Notifier
	executionPayloadReconstructor execution.PayloadReconstructor
	stateGen                      *stategen.State
	slasherAttestationsFeed       *event.Feed
	slasherBlockHeadersFeed       *event.Feed
	clock                         *startup.Clock
	stateNotifier                 statefeed.Notifier
	blobStorage                   *filesystem.BlobStorage
}

// This defines the interface for interacting with block chain service
type blockchainService interface {
	blockchain.BlockReceiver
	blockchain.BlobReceiver
	blockchain.HeadFetcher
	blockchain.FinalizationFetcher
	blockchain.ForkFetcher
	blockchain.AttestationReceiver
	blockchain.TimeFetcher
	blockchain.GenesisFetcher
	blockchain.CanonicalFetcher
	blockchain.OptimisticModeFetcher
	blockchain.SlashingReceiver
	blockchain.ForkchoiceFetcher
}

// Service is responsible for handling all run time p2p related operations as the
// main entry point for network messages.
type Service struct {
	cfg                              *config
	ctx                              context.Context
	cancel                           context.CancelFunc
	slotToPendingBlocks              *gcache.Cache
	seenPendingBlocks                map[[32]byte]bool
	blkRootToPendingAtts             map[[32]byte][]*ethpb.SignedAggregateAttestationAndProof
	subHandler                       *subTopicHandler
	pendingAttsLock                  sync.RWMutex
	pendingQueueLock                 sync.RWMutex
	chainStarted                     *abool.AtomicBool
	validateBlockLock                sync.RWMutex
	rateLimiter                      *limiter
	seenBlockLock                    sync.RWMutex
	seenBlockCache                   *lru.Cache
	seenBlobLock                     sync.RWMutex
	seenBlobCache                    *lru.Cache
	seenAggregatedAttestationLock    sync.RWMutex
	seenAggregatedAttestationCache   *lru.Cache
	seenUnAggregatedAttestationLock  sync.RWMutex
	seenUnAggregatedAttestationCache *lru.Cache
	seenExitLock                     sync.RWMutex
	seenExitCache                    *lru.Cache
	seenProposerSlashingLock         sync.RWMutex
	seenProposerSlashingCache        *lru.Cache
	seenAttesterSlashingLock         sync.RWMutex
	seenAttesterSlashingCache        map[uint64]bool
	seenSyncMessageLock              sync.RWMutex
	seenSyncMessageCache             *lru.Cache
	seenSyncContributionLock         sync.RWMutex
	seenSyncContributionCache        *lru.Cache
	badBlockCache                    *lru.Cache
	badBlockLock                     sync.RWMutex
	syncContributionBitsOverlapLock  sync.RWMutex
	syncContributionBitsOverlapCache *lru.Cache
	signatureChan                    chan *signatureVerifier
	clockWaiter                      startup.ClockWaiter
	initialSyncComplete              chan struct{}
	verifierWaiter                   *verification.InitializerWaiter
	newBlobVerifier                  verification.NewBlobVerifier
}

// NewService initializes new regular sync service.
func NewService(ctx context.Context, opts ...Option) *Service {
	c := gcache.New(pendingBlockExpTime /* exp time */, 0 /* disable janitor */)
	ctx, cancel := context.WithCancel(ctx)
	r := &Service{
		ctx:                  ctx,
		cancel:               cancel,
		chainStarted:         abool.New(),
		cfg:                  &config{clock: startup.NewClock(time.Unix(0, 0), [32]byte{})},
		slotToPendingBlocks:  c,
		seenPendingBlocks:    make(map[[32]byte]bool),
		blkRootToPendingAtts: make(map[[32]byte][]*ethpb.SignedAggregateAttestationAndProof),
		signatureChan:        make(chan *signatureVerifier, verifierLimit),
	}
	for _, opt := range opts {
		if err := opt(r); err != nil {
			return nil
		}
	}
	// Correctly remove it from our seen pending block map.
	// The eviction method always assumes that the mutex is held.
	r.slotToPendingBlocks.OnEvicted(func(s string, i interface{}) {
		if !mutexasserts.RWMutexLocked(&r.pendingQueueLock) {
			log.Errorf("Mutex is not locked during cache eviction of values")
			// Continue on to allow elements to be properly removed.
		}
		blks, ok := i.([]interfaces.ReadOnlySignedBeaconBlock)
		if !ok {
			log.Errorf("Invalid type retrieved from the cache: %T", i)
			return
		}

		for _, b := range blks {
			root, err := b.Block().HashTreeRoot()
			if err != nil {
				log.WithError(err).Error("Could not calculate htr of block")
				continue
			}
			delete(r.seenPendingBlocks, root)
		}
	})
	r.subHandler = newSubTopicHandler()
	r.rateLimiter = newRateLimiter(r.cfg.p2p)
	r.initCaches()

	return r
}

func newBlobVerifierFromInitializer(ini *verification.Initializer) verification.NewBlobVerifier {
	return func(b blocks.ROBlob, reqs []verification.Requirement) verification.BlobVerifier {
		return ini.NewBlobVerifier(b, reqs)
	}
}

// Start the regular sync service.
func (s *Service) Start() {
	v, err := s.verifierWaiter.WaitForInitializer(s.ctx)
	if err != nil {
		log.WithError(err).Error("Could not get verification initializer")
		return
	}
	s.newBlobVerifier = newBlobVerifierFromInitializer(v)

	go s.verifierRoutine()
	go s.registerHandlers()

	s.cfg.p2p.AddConnectionHandler(s.reValidatePeer, s.sendGoodbye)
	s.cfg.p2p.AddDisconnectionHandler(func(_ context.Context, _ peer.ID) error {
		// no-op
		return nil
	})
	s.cfg.p2p.AddPingMethod(s.sendPingRequest)
	s.processPendingBlocksQueue()
	s.processPendingAttsQueue()
	s.maintainPeerStatuses()
	s.resyncIfBehind()

	// Update sync metrics.
	async.RunEvery(s.ctx, syncMetricsInterval, s.updateMetrics)
}

// Stop the regular sync service.
func (s *Service) Stop() error {
	defer func() {
		if s.rateLimiter != nil {
			s.rateLimiter.free()
		}
	}()
	// Removing RPC Stream handlers.
	for _, p := range s.cfg.p2p.Host().Mux().Protocols() {
		s.cfg.p2p.Host().RemoveStreamHandler(p)
	}
	// Deregister Topic Subscribers.
	for _, t := range s.cfg.p2p.PubSub().GetTopics() {
		s.unSubscribeFromTopic(t)
	}
	defer s.cancel()
	return nil
}

// Status of the currently running regular sync service.
func (s *Service) Status() error {
	// If our head slot is on a previous epoch and our peers are reporting their head block are
	// in the most recent epoch, then we might be out of sync.
	if headEpoch := slots.ToEpoch(s.cfg.chain.HeadSlot()); headEpoch+1 < slots.ToEpoch(s.cfg.clock.CurrentSlot()) &&
		headEpoch+1 < s.cfg.p2p.Peers().HighestEpoch() {
		return errors.New("out of sync")
	}
	return nil
}

// This initializes the caches to update seen beacon objects coming in from the wire
// and prevent DoS.
func (s *Service) initCaches() {
	s.seenBlockCache = lruwrpr.New(seenBlockSize)
	s.seenBlobCache = lruwrpr.New(seenBlobSize)
	s.seenAggregatedAttestationCache = lruwrpr.New(seenAggregatedAttSize)
	s.seenUnAggregatedAttestationCache = lruwrpr.New(seenUnaggregatedAttSize)
	s.seenSyncMessageCache = lruwrpr.New(seenSyncMsgSize)
	s.seenSyncContributionCache = lruwrpr.New(seenSyncContributionSize)
	s.syncContributionBitsOverlapCache = lruwrpr.New(seenSyncContributionSize)
	s.seenExitCache = lruwrpr.New(seenExitSize)
	s.seenAttesterSlashingCache = make(map[uint64]bool)
	s.seenProposerSlashingCache = lruwrpr.New(seenProposerSlashingSize)
	s.badBlockCache = lruwrpr.New(badBlockSize)
}

func (s *Service) waitForChainStart() {
	clock, err := s.clockWaiter.WaitForClock(s.ctx)
	if err != nil {
		log.WithError(err).Error("sync service failed to receive genesis data")
		return
	}
	s.cfg.clock = clock
	startTime := clock.GenesisTime()
	log.WithField("starttime", startTime).Debug("Received state initialized event")
	// Register respective rpc handlers at state initialized event.
	s.registerRPCHandlers()
	// Wait for chainstart in separate routine.
	if startTime.After(prysmTime.Now()) {
		time.Sleep(prysmTime.Until(startTime))
	}
	log.WithField("starttime", startTime).Debug("Chain started in sync service")
	s.markForChainStart()
}

func (s *Service) registerHandlers() {
	s.waitForChainStart()
	select {
	case <-s.initialSyncComplete:
		// Register respective pubsub handlers at state synced event.
		digest, err := s.currentForkDigest()
		if err != nil {
			log.WithError(err).Error("Could not retrieve current fork digest")
			return
		}
		currentEpoch := slots.ToEpoch(slots.CurrentSlot(uint64(s.cfg.clock.GenesisTime().Unix())))
		s.registerSubscribers(currentEpoch, digest)
		go s.forkWatcher()
		return
	case <-s.ctx.Done():
		log.Debug("Context closed, exiting goroutine")
		return
	}
}

func (s *Service) writeErrorResponseToStream(responseCode byte, reason string, stream libp2pcore.Stream) {
	writeErrorResponseToStream(responseCode, reason, stream, s.cfg.p2p)
}

func (s *Service) setRateCollector(topic string, c *leakybucket.Collector) {
	s.rateLimiter.limiterMap[topic] = c
}

// marks the chain as having started.
func (s *Service) markForChainStart() {
	s.chainStarted.Set()
}

func (s *Service) chainIsStarted() bool {
	return s.chainStarted.IsSet()
}

// Checker defines a struct which can verify whether a node is currently
// synchronizing a chain with the rest of peers in the network.
type Checker interface {
	Initialized() bool
	Syncing() bool
	Synced() bool
	Status() error
	Resync() error
}
