package policies

import "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"

// AfterNthEpoch runs for every epoch after the provided epoch.
func AfterNthEpoch(afterEpoch primitives.Epoch) func(epoch primitives.Epoch) bool {
	return func(currentEpoch primitives.Epoch) bool {
		return currentEpoch > afterEpoch
	}
}

// OnwardsNthEpoch runs for every epoch from the provided epoch.
func OnwardsNthEpoch(onwardsEpoch primitives.Epoch) func(epoch primitives.Epoch) bool {
	return func(currentEpoch primitives.Epoch) bool {
		return currentEpoch >= onwardsEpoch
	}
}

// AllEpochs runs for all epochs.
func AllEpochs(_ primitives.Epoch) bool {
	return true
}

// OnEpoch runs only for the provided epoch.
func OnEpoch(epoch primitives.Epoch) func(primitives.Epoch) bool {
	return func(currentEpoch primitives.Epoch) bool {
		return currentEpoch == epoch
	}
}

// BetweenEpochs runs for every epoch that is between the provided epochs.
func BetweenEpochs(fromEpoch, toEpoch primitives.Epoch) func(primitives.Epoch) bool {
	return func(currentEpoch primitives.Epoch) bool {
		return fromEpoch < currentEpoch && currentEpoch < toEpoch
	}
}

// EveryNEpochs runs every N epochs, starting with the provided epoch.
func EveryNEpochs(onwardsEpoch primitives.Epoch, n primitives.Epoch) func(epoch primitives.Epoch) bool {
	return func(currentEpoch primitives.Epoch) bool {
		return currentEpoch >= onwardsEpoch && ((currentEpoch-onwardsEpoch)%n == 0)
	}
}
