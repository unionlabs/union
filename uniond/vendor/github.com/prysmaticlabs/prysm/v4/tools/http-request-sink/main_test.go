package main

import (
	"bytes"
	"encoding/json"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type sampleRPCRequest struct {
	Name      string `json:"name"`
	ETHMethod string `json:"eth_method"`
	Address   string `json:"address"`
}

func Test_parseAndCaptureRequest(t *testing.T) {
	tmpFile := filepath.Join(t.TempDir(), "faketest.log")
	body := &sampleRPCRequest{
		Name:      "eth2",
		ETHMethod: "eth2_produceBlock",
		Address:   "0x0923920930923",
	}
	enc, err := json.Marshal(body)
	require.NoError(t, err)
	httpReq, err := http.NewRequest("GET", "/", bytes.NewBuffer(enc))
	require.NoError(t, err)

	reqContent := map[string]interface{}{}
	err = parseRequest(httpReq, &reqContent)
	require.NoError(t, err)

	// If the file doesn't exist, create it, or append to the file.
	f, err := os.OpenFile(
		tmpFile,
		os.O_APPEND|os.O_CREATE|os.O_RDWR,
		params.BeaconIoConfig().ReadWritePermissions,
	)
	require.NoError(t, err)

	err = captureRequest(f, reqContent)
	require.NoError(t, err)
	require.NoError(t, f.Close())

	f, err = os.Open(tmpFile)
	require.NoError(t, err)
	fileContents, err := io.ReadAll(f)
	require.NoError(t, err)

	receivedContent := map[string]interface{}{}
	err = json.Unmarshal(fileContents, &receivedContent)
	require.NoError(t, err)

	for key, val := range reqContent {
		receivedVal, ok := receivedContent[key]
		require.Equal(t, true, ok)
		require.DeepEqual(t, val, receivedVal)
	}
}
