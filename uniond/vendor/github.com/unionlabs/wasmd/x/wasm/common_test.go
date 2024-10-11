package wasm

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

const firstCodeID = 1

// ensure store code returns the expected response
func assertStoreCodeResponse(t *testing.T, data []byte, expected uint64) {
	var pStoreResp types.MsgStoreCodeResponse
	require.NoError(t, pStoreResp.Unmarshal(data))
	require.Equal(t, pStoreResp.CodeID, expected)
}

// ensure execution returns the expected data
func assertExecuteResponse(t *testing.T, data, expected []byte) {
	var pExecResp types.MsgExecuteContractResponse
	require.NoError(t, pExecResp.Unmarshal(data))
	require.Equal(t, pExecResp.Data, expected)
}

// ensures this returns a valid bech32 address and returns it
func parseInitResponse(t *testing.T, data []byte) string {
	var pInstResp types.MsgInstantiateContractResponse
	require.NoError(t, pInstResp.Unmarshal(data))
	require.NotEmpty(t, pInstResp.Address)
	addr := pInstResp.Address
	// ensure this is a valid sdk address
	_, err := sdk.AccAddressFromBech32(addr)
	require.NoError(t, err)
	return addr
}
