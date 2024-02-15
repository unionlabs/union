package params

import (
	"testing"
)

// SetupTestConfigCleanup preserves configurations allowing to modify them within tests without any
// restrictions, everything is restored after the test.
func SetupTestConfigCleanup(t testing.TB) {
	prevDefaultBeaconConfig := mainnetBeaconConfig.Copy()
	temp := configs.getActive().Copy()
	undo, err := SetActiveWithUndo(temp)
	if err != nil {
		t.Fatal(err)
	}
	prevNetworkCfg := networkConfig.Copy()
	t.Cleanup(func() {
		mainnetBeaconConfig = prevDefaultBeaconConfig
		err = undo()
		if err != nil {
			t.Fatal(err)
		}
		networkConfig = prevNetworkCfg
	})
}

// SetActiveTestCleanup sets an active config,
// and adds a test cleanup hook to revert to the default config after the test completes.
func SetActiveTestCleanup(t *testing.T, cfg *BeaconChainConfig) {
	undo, err := SetActiveWithUndo(cfg)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		err = undo()
		if err != nil {
			t.Fatal(err)
		}
	})
}
