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

func TestAggregateVerify(t *testing.T) {
	t.Run("blst", testAggregateVerify)
}

func testAggregateVerify(t *testing.T) {
	fNames, fContent := utils.RetrieveFiles("aggregate_verify", t)

	for i, file := range fNames {
		t.Run(file, func(t *testing.T) {
			test := &AggregateVerifyTest{}
			require.NoError(t, yaml.Unmarshal(fContent[i], test))
			pubkeys := make([]common.PublicKey, 0, len(test.Input.Pubkeys))
			msgs := make([][32]byte, 0, len(test.Input.Messages))
			for _, pubKey := range test.Input.Pubkeys {
				pkBytes, err := hex.DecodeString(pubKey[2:])
				require.NoError(t, err)
				pk, err := bls.PublicKeyFromBytes(pkBytes)
				if err != nil {
					if test.Output == false && errors.Is(err, common.ErrInfinitePubKey) {
						return
					}
					t.Fatalf("cannot unmarshal pubkey: %v", err)
				}
				pubkeys = append(pubkeys, pk)
			}
			for _, msg := range test.Input.Messages {
				msgBytes, err := hex.DecodeString(msg[2:])
				require.NoError(t, err)
				require.Equal(t, 32, len(msgBytes))
				msgs = append(msgs, bytesutil.ToBytes32(msgBytes))
			}
			sigBytes, err := hex.DecodeString(test.Input.Signature[2:])
			require.NoError(t, err)
			sig, err := bls.SignatureFromBytes(sigBytes)
			if err != nil {
				if test.Output == false {
					return
				}
				t.Fatalf("Cannot unmarshal input to signature: %v", err)
			}

			verified := sig.AggregateVerify(pubkeys, msgs)
			if verified != test.Output {
				t.Fatalf("Signature does not match the expected verification output. "+
					"Expected %#v but received %#v for test case %d", test.Output, verified, i)
			}
		})
	}
}
