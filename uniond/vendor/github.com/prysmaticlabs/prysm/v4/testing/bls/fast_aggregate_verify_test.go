package bls

import (
	"encoding/hex"
	"errors"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/bls/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestFastAggregateVerify(t *testing.T) {
	t.Run("blst", testFastAggregateVerify)
}

func testFastAggregateVerify(t *testing.T) {
	fNames, fContent := utils.RetrieveFiles("fast_aggregate_verify", t)

	for i, file := range fNames {
		t.Run(file, func(t *testing.T) {
			test := &FastAggregateVerifyTest{}
			require.NoError(t, yaml.Unmarshal(fContent[i], test))

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
