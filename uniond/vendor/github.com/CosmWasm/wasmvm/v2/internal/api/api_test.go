package api

import (
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/CosmWasm/wasmvm/v2/types"
)

func TestValidateAddressFailure(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	// create contract
	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)
	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	// instantiate it with this store
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	// if the human address is larger than 32 bytes, this will lead to an error in the go side
	longName := "long123456789012345678901234567890long"
	msg := []byte(`{"verifier": "` + longName + `", "beneficiary": "bob"}`)

	// make sure the call doesn't error, but we get a JSON-encoded error result from ContractResult
	igasMeter := types.GasMeter(gasMeter)
	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)

	// ensure the error message is what we expect
	require.Nil(t, result.Ok)
	// with this error
	require.Equal(t, "Generic error: addr_validate errored: human encoding too long", result.Err)
}
