package bls

import (
	"bytes"
	"encoding/hex"
	"strings"
	"testing"

	"github.com/ghodss/yaml"
	"github.com/prysmaticlabs/prysm/v4/testing/bls/utils"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	blst "github.com/supranational/blst/bindings/go"
)

func TestHashToG2(t *testing.T) {
	t.Run("blst", testHashToG2)
}

func testHashToG2(t *testing.T) {
	t.Skip("Hash To G2 needs co-ordinates exposed")
	fNames, fContent := utils.RetrieveFiles("hash_to_G2", t)

	for i, file := range fNames {
		content := fContent[i]
		t.Run(file, func(t *testing.T) {
			test := &HashToG2Test{}
			require.NoError(t, yaml.Unmarshal(content, test))

			msgBytes := []byte(test.Input.Message)

			splitX := strings.Split(test.Output.X, ",")
			outputX, err := hex.DecodeString(splitX[0][2:])
			require.NoError(t, err)

			point := blst.HashToG2(msgBytes, nil)
			val := point.Compress()
			if !bytes.Equal(val, outputX) {
				t.Fatalf("Retrieved X value does not match output. "+
					"Expected %#v but received %#v for test case %d", outputX, val, i)
			}
			t.Log("Success")
		})
	}
}
