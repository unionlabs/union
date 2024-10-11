package monitor

import (
	"context"
	"errors"
	"sort"
	"sync"

	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

// Error when the context is closed while waiting for sync.
var errContextClosedWhileWaiting = errors.New("context closed while waiting for beacon to sync to latest Head")

// ValidatorLatestPerformance keeps track of the latest participation of the validator
type ValidatorLatestPerformance struct {
	attestedSlot  primitives.Slot
	inclusionSlot primitives.Slot
	timelySource  bool
	timelyTarget  bool
	timelyHead    bool
	balance       uint64
	balanceChange int64
}

// ValidatorAggregatedPerformance keeps track of the accumulated performance of
// the tracked validator since start of monitor service.
type ValidatorAggregatedPerformance struct {
	startEpoch                      primitives.Epoch
	startBalance                    uint64
	totalAttestedCount              uint64
	totalRequestedCount             uint64
	totalDistance                   uint64
	totalCorrectSource              uint64
	totalCorrectTarget              uint64
	totalCorrectHead                uint64
	totalProposedCount              uint64
	totalAggregations               uint64
	totalSyncCommitteeContributions uint64
	totalSyncCommitteeAggregations  uint64
}

// ValidatorMonitorConfig contains the list of validator indices that the
// monitor service tracks, and the event feed notifier that the
// monitor needs to subscribe.
type ValidatorMonitorConfig struct {
	StateNotifier       statefeed.Notifier
	AttestationNotifier operation.Notifier
	HeadFetcher         blockchain.HeadFetcher
	StateGen            stategen.StateManager
	InitialSyncComplete chan struct{}
}

// Service is the main structure that tracks validators and reports logs and
// metrics of their performances throughout their lifetime.
type Service struct {
	config    *ValidatorMonitorConfig
	ctx       context.Context
	cancel    context.CancelFunc
	isLogging bool

	// Locks access to TrackedValidators, latestPerformance, aggregatedPerformance,
	// trackedSyncedCommitteeIndices and lastSyncedEpoch
	sync.RWMutex

	TrackedValidators           map[primitives.ValidatorIndex]bool
	latestPerformance           map[primitives.ValidatorIndex]ValidatorLatestPerformance
	aggregatedPerformance       map[primitives.ValidatorIndex]ValidatorAggregatedPerformance
	trackedSyncCommitteeIndices map[primitives.ValidatorIndex][]primitives.CommitteeIndex
	lastSyncedEpoch             primitives.Epoch
}

// NewService sets up a new validator monitor service instance when given a list of validator indices to track.
func NewService(ctx context.Context, config *ValidatorMonitorConfig, tracked []primitives.ValidatorIndex) (*Service, error) {
	ctx, cancel := context.WithCancel(ctx)
	r := &Service{
		config:                      config,
		ctx:                         ctx,
		cancel:                      cancel,
		TrackedValidators:           make(map[primitives.ValidatorIndex]bool, len(tracked)),
		latestPerformance:           make(map[primitives.ValidatorIndex]ValidatorLatestPerformance),
		aggregatedPerformance:       make(map[primitives.ValidatorIndex]ValidatorAggregatedPerformance),
		trackedSyncCommitteeIndices: make(map[primitives.ValidatorIndex][]primitives.CommitteeIndex),
		isLogging:                   false,
	}
	for _, idx := range tracked {
		r.TrackedValidators[idx] = true
	}
	return r, nil
}

// Start sets up the TrackedValidators map and then calls to wait until the beacon is synced.
func (s *Service) Start() {
	s.Lock()
	defer s.Unlock()

	tracked := make([]primitives.ValidatorIndex, 0, len(s.TrackedValidators))
	for idx := range s.TrackedValidators {
		tracked = append(tracked, idx)
	}
	sort.Slice(tracked, func(i, j int) bool { return tracked[i] < tracked[j] })

	log.WithFields(logrus.Fields{
		"ValidatorIndices": tracked,
	}).Info("Starting service")

	go s.run()
}

// run waits until the beacon is synced and starts the monitoring system.
func (s *Service) run() {
	if err := s.waitForSync(s.config.InitialSyncComplete); err != nil {
		log.WithError(err)
		return
	}
	st, err := s.config.HeadFetcher.HeadState(s.ctx)
	if err != nil {
		log.WithError(err).Error("Could not get head state")
		return
	}
	if st == nil {
		log.Error("Head state is nil")
		return
	}

	epoch := slots.ToEpoch(st.Slot())
	log.WithField("Epoch", epoch).Info("Synced to head epoch, starting reporting performance")

	s.Lock()
	s.initializePerformanceStructures(st, epoch)
	s.Unlock()

	s.updateSyncCommitteeTrackedVals(st)

	s.Lock()
	s.isLogging = true
	s.Unlock()

	stateChannel := make(chan *feed.Event, 1)
	stateSub := s.config.StateNotifier.StateFeed().Subscribe(stateChannel)
	s.monitorRoutine(stateChannel, stateSub)
}

// initializePerformanceStructures initializes the validatorLatestPerformance
// and validatorAggregatedPerformance for each tracked validator.
func (s *Service) initializePerformanceStructures(state state.BeaconState, epoch primitives.Epoch) {
	for idx := range s.TrackedValidators {
		balance, err := state.BalanceAtIndex(idx)
		if err != nil {
			log.WithError(err).WithField("ValidatorIndex", idx).Error(
				"Could not fetch starting balance, skipping aggregated logs.")
			balance = 0
		}
		s.aggregatedPerformance[idx] = ValidatorAggregatedPerformance{
			startEpoch:   epoch,
			startBalance: balance,
		}
		s.latestPerformance[idx] = ValidatorLatestPerformance{
			balance: balance,
		}
	}
}

// Status retrieves the status of the service.
func (s *Service) Status() error {
	if s.isLogging {
		return nil
	}
	return errors.New("not running")
}

// Stop stops the service.
func (s *Service) Stop() error {
	defer s.cancel()
	s.isLogging = false
	return nil
}

// waitForSync waits until the beacon node is synced to the latest head.
func (s *Service) waitForSync(syncChan chan struct{}) error {
	select {
	case <-syncChan:
		return nil
	case <-s.ctx.Done():
		log.Debug("Context closed, exiting goroutine")
		return errContextClosedWhileWaiting
	}
}

// monitorRoutine is the main dispatcher, it registers event channels for the
// state feed and the operation feed. It then calls the appropriate function
// when we get messages after syncing a block or processing attestations/sync
// committee contributions.
func (s *Service) monitorRoutine(stateChannel chan *feed.Event, stateSub event.Subscription) {
	defer stateSub.Unsubscribe()

	opChannel := make(chan *feed.Event, 1)
	opSub := s.config.AttestationNotifier.OperationFeed().Subscribe(opChannel)
	defer opSub.Unsubscribe()

	for {
		select {
		case e := <-stateChannel:
			if e.Type == statefeed.BlockProcessed {
				data, ok := e.Data.(*statefeed.BlockProcessedData)
				if !ok {
					log.Error("Event feed data is not of type *statefeed.BlockProcessedData")
				} else if data.Verified {
					// We only process blocks that have been verified
					s.processBlock(s.ctx, data.SignedBlock)
				}
			}
		case e := <-opChannel:
			switch e.Type {
			case operation.UnaggregatedAttReceived:
				data, ok := e.Data.(*operation.UnAggregatedAttReceivedData)
				if !ok {
					log.Error("Event feed data is not of type *operation.UnAggregatedAttReceivedData")
				} else {
					s.processUnaggregatedAttestation(s.ctx, data.Attestation)
				}
			case operation.AggregatedAttReceived:
				data, ok := e.Data.(*operation.AggregatedAttReceivedData)
				if !ok {
					log.Error("Event feed data is not of type *operation.AggregatedAttReceivedData")
				} else {
					s.processAggregatedAttestation(s.ctx, data.Attestation)
				}
			case operation.ExitReceived:
				data, ok := e.Data.(*operation.ExitReceivedData)
				if !ok {
					log.Error("Event feed data is not of type *operation.ExitReceivedData")
				} else {
					s.processExit(data.Exit)
				}
			case operation.SyncCommitteeContributionReceived:
				data, ok := e.Data.(*operation.SyncCommitteeContributionReceivedData)
				if !ok {
					log.Error("Event feed data is not of type *operation.SyncCommitteeContributionReceivedData")
				} else {
					s.processSyncCommitteeContribution(data.Contribution)
				}
			}
		case <-s.ctx.Done():
			log.Debug("Context closed, exiting goroutine")
			return
		case err := <-stateSub.Err():
			log.WithError(err).Error("Could not subscribe to state notifier")
			return
		}
	}
}

// TrackedIndex returns true if input  validator index exists in tracked validator list.
// It assumes the caller holds the service Lock
func (s *Service) trackedIndex(idx primitives.ValidatorIndex) bool {
	_, ok := s.TrackedValidators[idx]
	return ok
}

// updateSyncCommitteeTrackedVals updates the sync committee assignments of our
// tracked validators. It gets called when we sync a block after the Sync Period changes.
func (s *Service) updateSyncCommitteeTrackedVals(state state.BeaconState) {
	s.Lock()
	defer s.Unlock()
	for idx := range s.TrackedValidators {
		syncIdx, err := helpers.CurrentPeriodSyncSubcommitteeIndices(state, idx)
		if err != nil {
			log.WithError(err).WithField("ValidatorIndex", idx).Error(
				"Sync committee assignments will not be reported")
			delete(s.trackedSyncCommitteeIndices, idx)
		} else if len(syncIdx) == 0 {
			delete(s.trackedSyncCommitteeIndices, idx)
		} else {
			s.trackedSyncCommitteeIndices[idx] = syncIdx
		}
	}
	s.lastSyncedEpoch = slots.ToEpoch(state.Slot())
}
