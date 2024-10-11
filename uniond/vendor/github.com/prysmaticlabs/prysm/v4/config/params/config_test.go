package params_test

import (
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

// Test cases can be executed in an arbitrary order. TestOverrideBeaconConfigTestTeardown checks
// that there's no state mutation leak from the previous test, therefore we need a sentinel flag,
// to make sure that previous test case has already been completed and check can be run.
var testOverrideBeaconConfigExecuted bool

func TestConfig_OverrideBeaconConfig(t *testing.T) {
	// Ensure that param modifications are safe.
	params.SetupTestConfigCleanup(t)
	cfg := params.BeaconConfig()
	cfg.SlotsPerEpoch = 5
	params.OverrideBeaconConfig(cfg)
	if c := params.BeaconConfig(); c.SlotsPerEpoch != 5 {
		t.Errorf("Shardcount in BeaconConfig incorrect. Wanted %d, got %d", 5, c.SlotsPerEpoch)
	}
	testOverrideBeaconConfigExecuted = true
}

func TestConfig_OverrideBeaconConfigTestTeardown(t *testing.T) {
	if !testOverrideBeaconConfigExecuted {
		t.Skip("State leak can occur only if state mutating test has already completed")
	}
	cfg := params.BeaconConfig()
	if cfg.SlotsPerEpoch == 5 {
		t.Fatal("Parameter update has been leaked out of previous test")
	}
}

func TestConfig_DataRace(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	wg := new(sync.WaitGroup)
	for i := 0; i < 10; i++ {
		wg.Add(2)
		go func() {
			defer wg.Done()
			cfg := params.BeaconConfig()
			params.OverrideBeaconConfig(cfg)
		}()
		go func() uint64 {
			defer wg.Done()
			return params.BeaconConfig().MaxDeposits
		}()
	}
	wg.Wait()
}

func TestConfig_WithinDAPeriod(t *testing.T) {
	cases := []struct {
		name    string
		block   primitives.Epoch
		current primitives.Epoch
		within  bool
	}{
		{
			name:    "before",
			block:   0,
			current: params.BeaconConfig().MinEpochsForBlobsSidecarsRequest + 1,
			within:  false,
		},
		{
			name:    "same",
			block:   0,
			current: 0,
			within:  true,
		},
		{
			name:    "boundary",
			block:   0,
			current: params.BeaconConfig().MinEpochsForBlobsSidecarsRequest,
			within:  true,
		},
		{
			name:    "one less",
			block:   params.BeaconConfig().MinEpochsForBlobsSidecarsRequest - 1,
			current: params.BeaconConfig().MinEpochsForBlobsSidecarsRequest,
			within:  true,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			require.Equal(t, c.within, params.WithinDAPeriod(c.block, c.current))
		})
	}
}
