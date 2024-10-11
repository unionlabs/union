// Package shuffle contains all conformity specification tests
// for validator shuffling logic according to the Ethereum Beacon Node spec.
package shuffle

import (
	"encoding/hex"
	"path"
	"testing"

	"github.com/ethereum/go-ethereum/common"
	"github.com/go-yaml/yaml"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

// RunShuffleTests executes "shuffling/core/shuffle" tests.
func RunShuffleTests(t *testing.T, config string) {
	require.NoError(t, utils.SetConfig(t, config))

	testFolders, testsFolderPath := utils.TestFolders(t, config, "phase0", "shuffling/core/shuffle")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", config, "phase0", "shuffling/core/shuffle")
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			testCaseFile, err := util.BazelFileBytes(path.Join(testsFolderPath, folder.Name(), "mapping.yaml"))
			require.NoError(t, err, "Could not read YAML tests directory")

			testCase := &TestCase{}
			require.NoError(t, yaml.Unmarshal(testCaseFile, testCase), "Could not unmarshal YAML file into test struct")
			require.NoError(t, runShuffleTest(t, testCase), "Shuffle test failed")
		})
	}
}

// RunShuffleTest uses validator set specified from a YAML file, runs the validator shuffle
// algorithm, then compare the output with the expected output from the YAML file.
func runShuffleTest(t *testing.T, testCase *TestCase) error {
	baseSeed, err := hex.DecodeString(testCase.Seed[2:])
	if err != nil {
		return err
	}

	seed := common.BytesToHash(baseSeed)
	testIndices := make([]primitives.ValidatorIndex, testCase.Count)
	for i := primitives.ValidatorIndex(0); uint64(i) < testCase.Count; i++ {
		testIndices[i] = i
	}
	shuffledList := make([]primitives.ValidatorIndex, testCase.Count)
	for i := primitives.ValidatorIndex(0); uint64(i) < testCase.Count; i++ {
		si, err := helpers.ShuffledIndex(i, testCase.Count, seed)
		if err != nil {
			return err
		}
		shuffledList[i] = si
	}
	require.DeepSSZEqual(t, shuffledList, testCase.Mapping)
	return nil
}
