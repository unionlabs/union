// Package slasher implements slashing detection for eth2, able to catch slashable attestations
// and proposals that it receives via two event feeds, respectively. Any found slashings
// are then submitted to the beacon node's slashing operations pool. See the design document
// here https://hackmd.io/@prysmaticlabs/slasher.
package slasher

import (
	"context"
	"time"

	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

const (
	shutdownTimeout = time.Minute * 5
)

// ServiceConfig for the slasher service in the beacon node.
// This struct allows us to specify required dependencies and
// parameters for slasher to function as needed.
type ServiceConfig struct {
	IndexedAttestationsFeed *event.Feed
	BeaconBlockHeadersFeed  *event.Feed
	Database                db.SlasherDatabase
	StateNotifier           statefeed.Notifier
	AttestationStateFetcher blockchain.AttestationStateFetcher
	StateGen                stategen.StateManager
	SlashingPoolInserter    slashings.PoolInserter
	HeadStateFetcher        blockchain.HeadFetcher
	SyncChecker             sync.Checker
	ClockWaiter             startup.ClockWaiter
}

// SlashingChecker is an interface for defining services that the beacon node may interact with to provide slashing data.
type SlashingChecker interface {
	IsSlashableBlock(ctx context.Context, proposal *ethpb.SignedBeaconBlockHeader) (*ethpb.ProposerSlashing, error)
	IsSlashableAttestation(ctx context.Context, attestation *ethpb.IndexedAttestation) ([]*ethpb.AttesterSlashing, error)
	HighestAttestations(
		ctx context.Context, indices []primitives.ValidatorIndex,
	) ([]*ethpb.HighestAttestation, error)
}

// Service defining a slasher implementation as part of
// the beacon node, able to detect eth2 slashable offenses.
type Service struct {
	params                         *Parameters
	serviceCfg                     *ServiceConfig
	indexedAttsChan                chan *ethpb.IndexedAttestation
	beaconBlockHeadersChan         chan *ethpb.SignedBeaconBlockHeader
	attsQueue                      *attestationsQueue
	blksQueue                      *blocksQueue
	ctx                            context.Context
	cancel                         context.CancelFunc
	genesisTime                    time.Time
	attsSlotTicker                 *slots.SlotTicker
	blocksSlotTicker               *slots.SlotTicker
	pruningSlotTicker              *slots.SlotTicker
	latestEpochWrittenForValidator map[primitives.ValidatorIndex]primitives.Epoch
}

// New instantiates a new slasher from configuration values.
func New(ctx context.Context, srvCfg *ServiceConfig) (*Service, error) {
	ctx, cancel := context.WithCancel(ctx)
	return &Service{
		params:                         DefaultParams(),
		serviceCfg:                     srvCfg,
		indexedAttsChan:                make(chan *ethpb.IndexedAttestation, 1),
		beaconBlockHeadersChan:         make(chan *ethpb.SignedBeaconBlockHeader, 1),
		attsQueue:                      newAttestationsQueue(),
		blksQueue:                      newBlocksQueue(),
		ctx:                            ctx,
		cancel:                         cancel,
		latestEpochWrittenForValidator: make(map[primitives.ValidatorIndex]primitives.Epoch),
	}, nil
}

// Start listening for received indexed attestations and blocks
// and perform slashing detection on them.
func (s *Service) Start() {
	go s.run() // Start functions must be non-blocking.
}

func (s *Service) run() {
	s.waitForChainInitialization()
	s.waitForSync(s.genesisTime)

	log.Info("Completed chain sync, starting slashing detection")

	// Get the latest epoch written for each validator from disk on startup.
	headState, err := s.serviceCfg.HeadStateFetcher.HeadState(s.ctx)
	if err != nil {
		log.WithError(err).Error("Failed to fetch head state")
		return
	}
	numVals := headState.NumValidators()
	validatorIndices := make([]primitives.ValidatorIndex, numVals)
	for i := 0; i < numVals; i++ {
		validatorIndices[i] = primitives.ValidatorIndex(i)
	}
	start := time.Now()
	log.Info("Reading last epoch written for each validator...")
	epochsByValidator, err := s.serviceCfg.Database.LastEpochWrittenForValidators(
		s.ctx, validatorIndices,
	)
	if err != nil {
		log.Error(err)
		return
	}
	for _, item := range epochsByValidator {
		s.latestEpochWrittenForValidator[item.ValidatorIndex] = item.Epoch
	}
	log.WithField("elapsed", time.Since(start)).Info(
		"Finished retrieving last epoch written per validator",
	)

	indexedAttsChan := make(chan *ethpb.IndexedAttestation, 1)
	beaconBlockHeadersChan := make(chan *ethpb.SignedBeaconBlockHeader, 1)
	go s.receiveAttestations(s.ctx, indexedAttsChan)
	go s.receiveBlocks(s.ctx, beaconBlockHeadersChan)

	secondsPerSlot := params.BeaconConfig().SecondsPerSlot
	s.attsSlotTicker = slots.NewSlotTicker(s.genesisTime, secondsPerSlot)
	s.blocksSlotTicker = slots.NewSlotTicker(s.genesisTime, secondsPerSlot)
	s.pruningSlotTicker = slots.NewSlotTicker(s.genesisTime, secondsPerSlot)
	go s.processQueuedAttestations(s.ctx, s.attsSlotTicker.C())
	go s.processQueuedBlocks(s.ctx, s.blocksSlotTicker.C())
	go s.pruneSlasherData(s.ctx, s.pruningSlotTicker.C())
}

// Stop the slasher service.
func (s *Service) Stop() error {
	s.cancel()
	if s.attsSlotTicker != nil {
		s.attsSlotTicker.Done()
	}
	if s.blocksSlotTicker != nil {
		s.blocksSlotTicker.Done()
	}
	if s.pruningSlotTicker != nil {
		s.pruningSlotTicker.Done()
	}
	// Flush the latest epoch written map to disk.
	start := time.Now()
	// New context as the service context has already been canceled.
	ctx, innerCancel := context.WithTimeout(context.Background(), shutdownTimeout)
	defer innerCancel()
	log.Info("Flushing last epoch written for each validator to disk, please wait")
	if err := s.serviceCfg.Database.SaveLastEpochsWrittenForValidators(
		ctx, s.latestEpochWrittenForValidator,
	); err != nil {
		log.Error(err)
	}
	log.WithField("elapsed", time.Since(start)).Debug(
		"Finished saving last epoch written per validator",
	)
	return nil
}

// Status of the slasher service.
func (*Service) Status() error {
	return nil
}

func (s *Service) waitForChainInitialization() {
	clock, err := s.serviceCfg.ClockWaiter.WaitForClock(s.ctx)
	if err != nil {
		log.WithError(err).Error("Could not receive chain start notification")
	}
	s.genesisTime = clock.GenesisTime()
	log.WithField("genesisTime", s.genesisTime).Info(
		"Slasher received chain initialization event",
	)
}

func (s *Service) waitForSync(genesisTime time.Time) {
	if slots.SinceGenesis(genesisTime) < params.BeaconConfig().SlotsPerEpoch || !s.serviceCfg.SyncChecker.Syncing() {
		return
	}
	slotTicker := slots.NewSlotTicker(s.genesisTime, params.BeaconConfig().SecondsPerSlot)
	defer slotTicker.Done()
	for {
		select {
		case <-slotTicker.C():
			// If node is still syncing, do not operate slasher.
			if s.serviceCfg.SyncChecker.Syncing() {
				continue
			}
			return
		case <-s.ctx.Done():
			return
		}
	}
}
