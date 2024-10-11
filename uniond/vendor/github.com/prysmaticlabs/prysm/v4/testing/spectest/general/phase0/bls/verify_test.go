package bls

import (
	"encoding/hex"
	"errors"
	"path"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestVerify(t *testing.T) {
	t.Run("blst", testVerify)
}

func testVerify(t *testing.T) {
	testFolders, testFolderPath := utils.TestFolders(t, "general", "phase0", "bls/verify/bls")
	if len(testFolders) == 0 {
		t.Fatalf("No test folders found for %s/%s/%s", "general", "phase0", "bls/verify/bls")
	}
	for i, folder := range testFolders {
		t.Run(folder.Name(), func(t *testing.T) {
			file, err := util.BazelFileBytes(path.Join(testFolderPath, folder.Name(), "data.yaml"))
			require.NoError(t, err)
			test := &VerifyMsgTest{}
			require.NoError(t, yaml.Unmarshal(file, test))

			pkBytes, err := hex.DecodeString(test.Input.Pubkey[2:])
			require.NoError(t, err)
			pk, err := bls.PublicKeyFromBytes(pkBytes)
			if err != nil {
				if test.Output == false && errors.Is(err, common.ErrInfinitePubKey) {
					return
				}
				t.Fatalf("cannot unmarshal pubkey: %v", err)
			}
			msgBytes, err := hex.DecodeString(test.Input.Message[2:])
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

			verified := sig.Verify(pk, msgBytes)
			if verified != test.Output {
				t.Fatalf("Signature does not match the expected verification output. "+
					"Expected %#v but received %#v for test case %d", test.Output, verified, i)
			}
			t.Log("Success")
		})
	}
}
