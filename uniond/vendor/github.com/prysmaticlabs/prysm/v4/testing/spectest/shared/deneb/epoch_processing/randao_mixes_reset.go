package epoch_processing

import (
	"path"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
)

// RunRandaoMixesResetTests executes "epoch_processing/randao_mixes_reset" tests.
func RunRandaoMixesResetTests(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))

	testFolders, testsFolderPath := utils.TestFolders(t, config, "deneb", "epoch_processing/randao_mixes_reset/pyspec_tests")
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			folderPath := path.Join(testsFolderPath, folder.Name())
			RunEpochOperationTest(t, folderPath, processRandaoMixesResetWrapper)
		})
	}
}

func processRandaoMixesResetWrapper(t *testing.T, st state.BeaconState) (state.BeaconState, error) {
	st, err := epoch.ProcessRandaoMixesReset(st)
	require.NoError(t, err, "Could not process final updates")
	return st, nil
}
