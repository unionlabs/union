package bls

import (
	"encoding/hex"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/bls/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestAggregate(t *testing.T) {
	t.Run("blst", testAggregate)
}

func testAggregate(t *testing.T) {
	fNames, fContent := utils.RetrieveFiles("aggregate", t)
	for i, folder := range fNames {
		t.Run(folder, func(t *testing.T) {
			test := &AggregateTest{}
			require.NoError(t, yaml.Unmarshal(fContent[i], test))
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
			outputBytes, err := hex.DecodeString(test.Output[2:])
			require.NoError(t, err)
			require.DeepEqual(t, outputBytes, sig.Marshal())
		})
	}
}
