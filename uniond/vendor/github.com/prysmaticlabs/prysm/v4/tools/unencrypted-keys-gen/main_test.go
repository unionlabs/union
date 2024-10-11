package main

import (
	"bytes"
	"encoding/json"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/tools/unencrypted-keys-gen/keygen"
)

func TestSavesUnencryptedKeys(t *testing.T) {
	keys := 2
	numKeys = &keys
	ctnr := generateUnencryptedKeys(0 /* start index */)
	buf := new(bytes.Buffer)
	require.NoError(t, keygen.SaveUnencryptedKeysToFile(buf, ctnr))
	enc := buf.Bytes()
	dec := &keygen.UnencryptedKeysContainer{}
	require.NoError(t, json.Unmarshal(enc, dec))
	assert.DeepEqual(t, ctnr, dec)
}
