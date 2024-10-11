package epoch_processing

import (
	"path"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/epoch"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
)

// RunEth1DataResetTests executes "epoch_processing/eth1_data_reset" tests.
func RunEth1DataResetTests(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))

	testFolders, testsFolderPath := utils.TestFolders(t, config, "deneb", "epoch_processing/eth1_data_reset/pyspec_tests")
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			folderPath := path.Join(testsFolderPath, folder.Name())
			RunEpochOperationTest(t, folderPath, processEth1DataResetWrapper)
		})
	}
}

func processEth1DataResetWrapper(t *testing.T, st state.BeaconState) (state.BeaconState, error) {
	st, err := epoch.ProcessEth1DataReset(st)
	require.NoError(t, err, "Could not process final updates")
	return st, nil
}
