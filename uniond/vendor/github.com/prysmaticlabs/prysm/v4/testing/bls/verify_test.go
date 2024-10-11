package bls

import (
	"encoding/hex"
	"errors"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/bls/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestVerify(t *testing.T) {
	t.Run("blst", testVerify)
}

func testVerify(t *testing.T) {
	fNames, fContent := utils.RetrieveFiles("verify", t)

	for i, file := range fNames {
		t.Run(file, func(t *testing.T) {
			test := &VerifyMsgTest{}
			require.NoError(t, yaml.Unmarshal(fContent[i], test))

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
