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
		t.Error(err)
	}
	prevNetworkCfg := networkConfig.Copy()
	t.Cleanup(func() {
		mainnetBeaconConfig = prevDefaultBeaconConfig
		err = undo()
		if err != nil {
			t.Error(err)
		}
		networkConfig = prevNetworkCfg
	})
}
