package simulator

import (
	"context"
	"fmt"
	"time"

	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

var log = logrus.WithField("prefix", "simulator")

// ServiceConfig for the simulator.
type ServiceConfig struct {
	Params                      *Parameters
	Database                    db.SlasherDatabase
	StateNotifier               statefeed.Notifier
	AttestationStateFetcher     blockchain.AttestationStateFetcher
	HeadStateFetcher            blockchain.HeadFetcher
	StateGen                    stategen.StateManager
	SlashingsPool               slashings.PoolManager
	PrivateKeysByValidatorIndex map[primitives.ValidatorIndex]bls.SecretKey
	SyncChecker                 sync.Checker
	ClockWaiter                 startup.ClockWaiter
	ClockSetter                 startup.ClockSetter
}

// Parameters for a slasher simulator.
type Parameters struct {
	SecondsPerSlot         uint64
	SlotsPerEpoch          primitives.Slot
	AggregationPercent     float64
	ProposerSlashingProbab float64
	AttesterSlashingProbab float64
	NumValidators          uint64
	NumEpochs              uint64
}

// Simulator defines a struct which can launch a slasher simulation
// at scale using configuration parameters.
type Simulator struct {
	ctx                   context.Context
	slasher               *slasher.Service
	srvConfig             *ServiceConfig
	indexedAttsFeed       *event.Feed
	beaconBlocksFeed      *event.Feed
	sentAttSlashingFeed   *event.Feed
	sentBlockSlashingFeed *event.Feed
	sentProposerSlashings map[[32]byte]*ethpb.ProposerSlashing
	sentAttesterSlashings map[[32]byte]*ethpb.AttesterSlashing
	genesisTime           time.Time
}

// DefaultParams for launching a slasher simulator.
func DefaultParams() *Parameters {
	return &Parameters{
		SecondsPerSlot:         params.BeaconConfig().SecondsPerSlot,
		SlotsPerEpoch:          4,
		AggregationPercent:     1.0,
		ProposerSlashingProbab: 0.3,
		AttesterSlashingProbab: 0.3,
		NumValidators:          params.BeaconConfig().MinGenesisActiveValidatorCount,
		NumEpochs:              4,
	}
}

// New initializes a slasher simulator from a beacon database
// and configuration parameters.
func New(ctx context.Context, srvConfig *ServiceConfig) (*Simulator, error) {
	indexedAttsFeed := new(event.Feed)
	beaconBlocksFeed := new(event.Feed)
	sentBlockSlashingFeed := new(event.Feed)
	sentAttSlashingFeed := new(event.Feed)

	slasherSrv, err := slasher.New(ctx, &slasher.ServiceConfig{
		IndexedAttestationsFeed: indexedAttsFeed,
		BeaconBlockHeadersFeed:  beaconBlocksFeed,
		Database:                srvConfig.Database,
		StateNotifier:           srvConfig.StateNotifier,
		HeadStateFetcher:        srvConfig.HeadStateFetcher,
		AttestationStateFetcher: srvConfig.AttestationStateFetcher,
		StateGen:                srvConfig.StateGen,
		SlashingPoolInserter:    srvConfig.SlashingsPool,
		SyncChecker:             srvConfig.SyncChecker,
		ClockWaiter:             srvConfig.ClockWaiter,
	})
	if err != nil {
		return nil, err
	}
	return &Simulator{
		ctx:                   ctx,
		slasher:               slasherSrv,
		srvConfig:             srvConfig,
		indexedAttsFeed:       indexedAttsFeed,
		beaconBlocksFeed:      beaconBlocksFeed,
		sentAttSlashingFeed:   sentAttSlashingFeed,
		sentBlockSlashingFeed: sentBlockSlashingFeed,
		sentProposerSlashings: make(map[[32]byte]*ethpb.ProposerSlashing),
		sentAttesterSlashings: make(map[[32]byte]*ethpb.AttesterSlashing),
	}, nil
}

// Start a simulator.
func (s *Simulator) Start() {
	log.WithFields(logrus.Fields{
		"numValidators":          s.srvConfig.Params.NumValidators,
		"numEpochs":              s.srvConfig.Params.NumEpochs,
		"secondsPerSlot":         s.srvConfig.Params.SecondsPerSlot,
		"proposerSlashingProbab": s.srvConfig.Params.ProposerSlashingProbab,
		"attesterSlashingProbab": s.srvConfig.Params.AttesterSlashingProbab,
	}).Info("Starting slasher simulator")

	// Override global configuration for simulation purposes.
	config := params.BeaconConfig().Copy()
	config.SecondsPerSlot = s.srvConfig.Params.SecondsPerSlot
	config.SlotsPerEpoch = s.srvConfig.Params.SlotsPerEpoch
	undo, err := params.SetActiveWithUndo(config)
	if err != nil {
		panic(err)
	}
	defer func() {
		if err := undo(); err != nil {
			panic(err)
		}
	}()

	// Start slasher in the background.
	go s.slasher.Start()

	// Wait some time and then send a "chain started" event over a notifier
	// for slasher to pick up a genesis time.
	time.Sleep(time.Second)
	s.genesisTime = time.Now()
	var vr [32]byte
	if err := s.srvConfig.ClockSetter.SetClock(startup.NewClock(s.genesisTime, vr)); err != nil {
		panic(err)
	}

	// We simulate blocks and attestations for N epochs.
	s.simulateBlocksAndAttestations(s.ctx)

	// Verify the slashings we detected are the same as those the
	// simulator produced, effectively checking slasher caught all slashable offenses.
	s.verifySlashingsWereDetected(s.ctx)
}

// Stop the simulator.
func (s *Simulator) Stop() error {
	return s.slasher.Stop()
}

func (s *Simulator) simulateBlocksAndAttestations(ctx context.Context) {
	// Add a small offset to producing blocks and attestations a little bit after a slot starts.
	ticker := slots.NewSlotTicker(s.genesisTime.Add(time.Millisecond*500), params.BeaconConfig().SecondsPerSlot)
	defer ticker.Done()
	for {
		select {
		case slot := <-ticker.C():
			// We only run the simulator for a specified number of epochs.
			totalEpochs := primitives.Epoch(s.srvConfig.Params.NumEpochs)
			if slots.ToEpoch(slot) >= totalEpochs {
				return
			}

			// Since processing slashings requires at least one slot, we do nothing
			// if we are a few slots from the end of the simulation.
			endSlot, err := slots.EpochStart(totalEpochs)
			if err != nil {
				log.WithError(err).Fatal("Could not get epoch start slot")
			}
			if slot+3 > endSlot {
				continue
			}

			blockHeaders, propSlashings, err := s.generateBlockHeadersForSlot(ctx, slot)
			if err != nil {
				log.WithError(err).Fatal("Could not generate block headers for slot")
			}
			log.WithFields(logrus.Fields{
				"numBlocks":    len(blockHeaders),
				"numSlashable": len(propSlashings),
			}).Infof("Producing blocks for slot %d", slot)
			for _, sl := range propSlashings {
				slashingRoot, err := sl.HashTreeRoot()
				if err != nil {
					log.WithError(err).Fatal("Could not hash tree root slashing")
				}
				s.sentProposerSlashings[slashingRoot] = sl
			}
			for _, bb := range blockHeaders {
				s.beaconBlocksFeed.Send(bb)
			}

			atts, attSlashings, err := s.generateAttestationsForSlot(ctx, slot)
			if err != nil {
				log.WithError(err).Fatal("Could not generate attestations for slot")
			}
			log.WithFields(logrus.Fields{
				"numAtts":      len(atts),
				"numSlashable": len(propSlashings),
			}).Infof("Producing attestations for slot %d", slot)
			for _, sl := range attSlashings {
				slashingRoot, err := sl.HashTreeRoot()
				if err != nil {
					log.WithError(err).Fatal("Could not hash tree root slashing")
				}
				s.sentAttesterSlashings[slashingRoot] = sl
			}
			for _, aa := range atts {
				s.indexedAttsFeed.Send(aa)
			}
		case <-ctx.Done():
			return
		}
	}
}

func (s *Simulator) verifySlashingsWereDetected(ctx context.Context) {
	poolProposerSlashings := s.srvConfig.SlashingsPool.PendingProposerSlashings(
		ctx, nil, true, /* no limit */
	)
	poolAttesterSlashings := s.srvConfig.SlashingsPool.PendingAttesterSlashings(
		ctx, nil, true, /* no limit */
	)
	detectedProposerSlashings := make(map[[32]byte]*ethpb.ProposerSlashing)
	detectedAttesterSlashings := make(map[[32]byte]*ethpb.AttesterSlashing)
	for _, slashing := range poolProposerSlashings {
		slashingRoot, err := slashing.HashTreeRoot()
		if err != nil {
			log.WithError(err).Error("Could not determine slashing root")
		}
		detectedProposerSlashings[slashingRoot] = slashing
	}
	for _, slashing := range poolAttesterSlashings {
		slashingRoot, err := slashing.HashTreeRoot()
		if err != nil {
			log.WithError(err).Error("Could not determine slashing root")
		}
		detectedAttesterSlashings[slashingRoot] = slashing
	}

	// Check if the sent slashings made it into the slashings pool.
	for slashingRoot, slashing := range s.sentProposerSlashings {
		if _, ok := detectedProposerSlashings[slashingRoot]; !ok {
			log.WithFields(logrus.Fields{
				"slot":          slashing.Header_1.Header.Slot,
				"proposerIndex": slashing.Header_1.Header.ProposerIndex,
			}).Errorf("Did not detect simulated proposer slashing")
			continue
		}
		log.WithFields(logrus.Fields{
			"slot":          slashing.Header_1.Header.Slot,
			"proposerIndex": slashing.Header_1.Header.ProposerIndex,
		}).Info("Correctly detected simulated proposer slashing")
	}
	for slashingRoot, slashing := range s.sentAttesterSlashings {
		if _, ok := detectedAttesterSlashings[slashingRoot]; !ok {
			log.WithFields(logrus.Fields{
				"targetEpoch":         slashing.Attestation_1.Data.Target.Epoch,
				"prevTargetEpoch":     slashing.Attestation_2.Data.Target.Epoch,
				"sourceEpoch":         slashing.Attestation_1.Data.Source.Epoch,
				"prevSourceEpoch":     slashing.Attestation_2.Data.Source.Epoch,
				"prevBeaconBlockRoot": fmt.Sprintf("%#x", slashing.Attestation_1.Data.BeaconBlockRoot),
				"newBeaconBlockRoot":  fmt.Sprintf("%#x", slashing.Attestation_2.Data.BeaconBlockRoot),
			}).Errorf("Did not detect simulated attester slashing")
			continue
		}
		log.WithFields(logrus.Fields{
			"targetEpoch":     slashing.Attestation_1.Data.Target.Epoch,
			"prevTargetEpoch": slashing.Attestation_2.Data.Target.Epoch,
			"sourceEpoch":     slashing.Attestation_1.Data.Source.Epoch,
			"prevSourceEpoch": slashing.Attestation_2.Data.Source.Epoch,
		}).Info("Correctly detected simulated attester slashing")
	}
}
