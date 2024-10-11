package testutil

import (
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// MockGenesisTimeFetcher is a fake implementation of the blockchain.TimeFetcher
type MockGenesisTimeFetcher struct {
	Genesis time.Time
}

func (m *MockGenesisTimeFetcher) GenesisTime() time.Time {
	return m.Genesis
}

func (m *MockGenesisTimeFetcher) CurrentSlot() primitives.Slot {
	return primitives.Slot(uint64(time.Now().Unix()-m.Genesis.Unix()) / params.BeaconConfig().SecondsPerSlot)
}
