package bls

import (
	"encoding/hex"
	"errors"
	"path"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestFastAggregateVerify(t *testing.T) {
	t.Run("blst", testFastAggregateVerify)
}

func testFastAggregateVerify(t *testing.T) {
	testFolders, testFolderPath := utils.TestFolders(t, "general", "phase0", "bls/fast_aggregate_verify/bls")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", "general", "phase0", "bls/fast_aggregate_verify/bls")
	}
	for i, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			file, err := util.BazelFileBytes(path.Join(testFolderPath, folder.Name(), "data.yaml"))
			require.NoError(t, err)
			test := &FastAggregateVerifyTest{}
			require.NoError(t, yaml.Unmarshal(file, test))

			pubkeys := make([]common.PublicKey, len(test.Input.Pubkeys))
			for j, raw := range test.Input.Pubkeys {
				pkBytes, err := hex.DecodeString(raw[2:])
				require.NoError(t, err)
				pk, err := bls.PublicKeyFromBytes(pkBytes)
				if err != nil {
					if test.Output == false && errors.Is(err, common.ErrInfinitePubKey) {
						return
					}
					t.Fatalf("cannot unmarshal pubkey: %v", err)
				}
				pubkeys[j] = pk
			}

			msg := test.Input.Message
			// TODO(#7632): Remove when https://github.com/ethereum/consensus-spec-tests/issues/22 is resolved.
			if msg == "" {
				msg = test.Input.Messages
			}
			msgBytes, err := hex.DecodeString(msg[2:])
			require.NoError(t, err)
			sigBytes, err := hex.DecodeString(test.Input.Signature[2:])
			require.NoError(t, err)
			sig, err := bls.SignatureFromBytes(sigBytes)
			if err != nil {
				if test.Output == false {
					return
				}
				t.Fatalf("Cannot unmarshal input to signature: %v", err)
			}

			verified := sig.FastAggregateVerify(pubkeys, bytesutil.ToBytes32(msgBytes))
			if verified != test.Output {
				t.Fatalf("Signature does not match the expected verification output. "+
					"Expected %#v but received %#v for test case %d", test.Output, verified, i)
			}
			t.Log("Success")
		})
	}
}
