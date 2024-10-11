package sync

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// Is a background routine that observes for new incoming forks. Depending on the epoch
// it will be in charge of subscribing/unsubscribing the relevant topics at the fork boundaries.
func (s *Service) forkWatcher() {
	slotTicker := slots.NewSlotTicker(s.cfg.clock.GenesisTime(), params.BeaconConfig().SecondsPerSlot)
	for {
		select {
		// In the event of a node restart, we will still end up subscribing to the correct
		// topics during/after the fork epoch. This routine is to ensure correct
		// subscriptions for nodes running before a fork epoch.
		case currSlot := <-slotTicker.C():
			currEpoch := slots.ToEpoch(currSlot)
			if err := s.registerForUpcomingFork(currEpoch); err != nil {
				log.WithError(err).Error("Unable to check for fork in the next epoch")
				continue
			}
			if err := s.deregisterFromPastFork(currEpoch); err != nil {
				log.WithError(err).Error("Unable to check for fork in the previous epoch")
				continue
			}
			// Broadcast BLS changes at the Capella fork boundary
			s.broadcastBLSChanges(currSlot)

		case <-s.ctx.Done():
			log.Debug("Context closed, exiting goroutine")
			slotTicker.Done()
			return
		}
	}
}

// Checks if there is a fork in the next epoch and if there is
// it registers the appropriate gossip and rpc topics.
func (s *Service) registerForUpcomingFork(currEpoch primitives.Epoch) error {
	genRoot := s.cfg.clock.GenesisValidatorsRoot()
	isNextForkEpoch, err := forks.IsForkNextEpoch(s.cfg.clock.GenesisTime(), genRoot[:])
	if err != nil {
		return errors.Wrap(err, "Could not retrieve next fork epoch")
	}
	// In preparation for the upcoming fork
	// in the following epoch, the node
	// will subscribe the new topics in advance.
	if isNextForkEpoch {
		nextEpoch := currEpoch + 1
		digest, err := forks.ForkDigestFromEpoch(nextEpoch, genRoot[:])
		if err != nil {
			return errors.Wrap(err, "could not retrieve fork digest")
		}
		if s.subHandler.digestExists(digest) {
			return nil
		}
		s.registerSubscribers(nextEpoch, digest)
		if nextEpoch == params.BeaconConfig().AltairForkEpoch {
			s.registerRPCHandlersAltair()
		}
		if nextEpoch == params.BeaconConfig().DenebForkEpoch {
			s.registerRPCHandlersDeneb()
		}
	}
	return nil
}

// Checks if there was a fork in the previous epoch, and if there
// was then we deregister the topics from that particular fork.
func (s *Service) deregisterFromPastFork(currEpoch primitives.Epoch) error {
	genRoot := s.cfg.clock.GenesisValidatorsRoot()
	// This method takes care of the de-registration of
	// old gossip pubsub handlers. Once we are at the epoch
	// after the fork, we de-register from all the outdated topics.
	currFork, err := forks.Fork(currEpoch)
	if err != nil {
		return err
	}
	// If we are still in our genesis fork version then
	// we simply exit early.
	if currFork.Epoch == params.BeaconConfig().GenesisEpoch {
		return nil
	}
	epochAfterFork := currFork.Epoch + 1
	// If we are in the epoch after the fork, we start de-registering.
	if epochAfterFork == currEpoch {
		// Look at the previous fork's digest.
		epochBeforeFork := currFork.Epoch - 1
		prevDigest, err := forks.ForkDigestFromEpoch(epochBeforeFork, genRoot[:])
		if err != nil {
			return errors.Wrap(err, "Failed to determine previous epoch fork digest")
		}

		// Exit early if there are no topics with that particular
		// digest.
		if !s.subHandler.digestExists(prevDigest) {
			return nil
		}
		prevFork, err := forks.Fork(epochBeforeFork)
		if err != nil {
			return errors.Wrap(err, "failed to determine previous epoch fork data")
		}
		if prevFork.Epoch == params.BeaconConfig().GenesisEpoch {
			s.unregisterPhase0Handlers()
		}
		// Run through all our current active topics and see
		// if there are any subscriptions to be removed.
		for _, t := range s.subHandler.allTopics() {
			retDigest, err := p2p.ExtractGossipDigest(t)
			if err != nil {
				log.WithError(err).Error("Could not retrieve digest")
				continue
			}
			if retDigest == prevDigest {
				s.unSubscribeFromTopic(t)
			}
		}
	}
	return nil
}
