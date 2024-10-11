package bls

import (
	"encoding/hex"
	"path"
	"strings"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestAggregate(t *testing.T) {
	t.Run("blst", testAggregate)
}

func testAggregate(t *testing.T) {
	testFolders, testFolderPath := utils.TestFolders(t, "general", "phase0", "bls/aggregate/bls")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", "general", "phase0", "bls/aggregate/bls")
	}
	for _, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			file, err := util.BazelFileBytes(path.Join(testFolderPath, folder.Name(), "data.yaml"))
			require.NoError(t, err)
			test := &AggregateTest{}
			require.NoError(t, yaml.Unmarshal(file, test))
			var sigs []common.Signature
			for _, s := range test.Input {
				sigBytes, err := hex.DecodeString(s[2:])
				require.NoError(t, err)
				sig, err := bls.SignatureFromBytes(sigBytes)
				require.NoError(t, err)
				sigs = append(sigs, sig)
			}
			if len(test.Input) == 0 {
				if test.Output != "" {
					t.Fatalf("Output Aggregate is not of zero length:Output %s", test.Output)
				}
				return
			}
			sig := bls.AggregateSignatures(sigs)
			if strings.Contains(folder.Name(), "aggregate_na_pubkeys") {
				if sig != nil {
					t.Errorf("Expected nil signature, received: %v", sig)
				}
				return
			}
			outputBytes, err := hex.DecodeString(test.Output[2:])
			require.NoError(t, err)
			require.DeepEqual(t, outputBytes, sig.Marshal())
		})
	}
}
