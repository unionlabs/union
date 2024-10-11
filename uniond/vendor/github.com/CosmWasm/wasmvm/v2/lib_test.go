package cosmwasm

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/CosmWasm/wasmvm/v2/types"
)

func TestCreateChecksum(t *testing.T) {
	// nil
	_, err := CreateChecksum(nil)
	require.ErrorContains(t, err, "nil or empty")

	// empty
	_, err = CreateChecksum([]byte{})
	require.ErrorContains(t, err, "nil or empty")

	// short
	_, err = CreateChecksum([]byte("\x00\x61\x73"))
	require.ErrorContains(t, err, " shorter than 4 bytes")

	// Wasm blob returns correct hash
	// echo "(module)" > my.wat && wat2wasm my.wat && hexdump -C my.wasm && sha256sum my.wasm
	checksum, err := CreateChecksum([]byte("\x00\x61\x73\x6d\x01\x00\x00\x00"))
	require.NoError(t, err)
	require.Equal(t, types.ForceNewChecksum("93a44bbb96c751218e4c00d479e4c14358122a389acca16205b1e4d0dc5f9476"), checksum)

	// Text file fails
	_, err = CreateChecksum([]byte("Hello world"))
	require.ErrorContains(t, err, "do not start with Wasm magic number")
}
