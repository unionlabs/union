package bls

import (
	"encoding/hex"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/testing/bls/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestDeserializationG2(t *testing.T) {
	t.Run("blst", testDeserializationG2)
}

func testDeserializationG2(t *testing.T) {
	fNames, fContent := utils.RetrieveFiles("deserialization_G2", t)

	for i, file := range fNames {
		content := fContent[i]
		t.Run(file, func(t *testing.T) {
			test := &DeserializationG2Test{}
			require.NoError(t, yaml.Unmarshal(content, test))
			rawKey, err := hex.DecodeString(test.Input.Signature)
			require.NoError(t, err)

			_, err = bls.SignatureFromBytes(rawKey)
			require.Equal(t, test.Output, err == nil)
			t.Log("Success")
		})
	}
}
