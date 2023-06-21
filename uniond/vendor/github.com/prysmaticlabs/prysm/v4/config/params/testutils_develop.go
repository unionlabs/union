//go:build develop

package params

import "testing"

// SetupTestConfigCleanupWithLock preserves configurations allowing to modify them within tests without any
// restrictions, everything is restored after the test. This locks our config when undoing our config
// change in order to satisfy the race detector.
func SetupTestConfigCleanupWithLock(t testing.TB) {
	prevDefaultBeaconConfig := mainnetBeaconConfig.Copy()
	temp := configs.getActive().Copy()
	undo, err := SetActiveWithUndo(temp)
	if err != nil {
		t.Error(err)
	}
	prevNetworkCfg := networkConfig.Copy()
	t.Cleanup(func() {
		mainnetBeaconConfig = prevDefaultBeaconConfig
		cfgrw.Lock()
		err = undo()
		cfgrw.Unlock()
		if err != nil {
			t.Error(err)
		}
		networkConfig = prevNetworkCfg
	})
}
