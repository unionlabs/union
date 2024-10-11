package api

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/CosmWasm/wasmvm/v2/types"
)

const (
	TESTING_PRINT_DEBUG  = false
	TESTING_GAS_LIMIT    = uint64(500_000_000_000) // ~0.5ms
	TESTING_MEMORY_LIMIT = 32                      // MiB
	TESTING_CACHE_SIZE   = 100                     // MiB
)

var TESTING_CAPABILITIES = []string{"staking", "stargate", "iterator", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3"}

func TestInitAndReleaseCache(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "wasmvm-testing")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir)

	cache, err := InitCache(tmpdir, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err)
	ReleaseCache(cache)
}

// wasmd expects us to create the base directory
// https://github.com/CosmWasm/wasmd/blob/v0.30.0/x/wasm/keeper/keeper.go#L128
func TestInitCacheWorksForNonExistentDir(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "wasmvm-testing")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir)

	createMe := filepath.Join(tmpdir, "does-not-yet-exist")
	cache, err := InitCache(createMe, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err)
	ReleaseCache(cache)
}

func TestInitCacheErrorsForBrokenDir(t *testing.T) {
	// Use colon to make this fail on Windows
	// https://gist.github.com/doctaphred/d01d05291546186941e1b7ddc02034d3
	// On Unix we should not have permission to create this.
	cannotBeCreated := "/foo:bar"
	_, err := InitCache(cannotBeCreated, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.ErrorContains(t, err, "Could not create base directory")
}

func TestInitLockingPreventsConcurrentAccess(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "wasmvm-testing")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir)

	cache1, err1 := InitCache(tmpdir, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err1)

	_, err2 := InitCache(tmpdir, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.ErrorContains(t, err2, "Could not lock exclusive.lock")

	ReleaseCache(cache1)

	// Now we can try again
	cache3, err3 := InitCache(tmpdir, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err3)
	ReleaseCache(cache3)
}

func TestInitLockingAllowsMultipleInstancesInDifferentDirs(t *testing.T) {
	tmpdir1, err := os.MkdirTemp("", "wasmvm-testing1")
	require.NoError(t, err)
	tmpdir2, err := os.MkdirTemp("", "wasmvm-testing2")
	require.NoError(t, err)
	tmpdir3, err := os.MkdirTemp("", "wasmvm-testing3")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir1)
	defer os.RemoveAll(tmpdir2)
	defer os.RemoveAll(tmpdir3)

	cache1, err1 := InitCache(tmpdir1, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err1)
	cache2, err2 := InitCache(tmpdir2, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err2)
	cache3, err3 := InitCache(tmpdir3, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err3)

	ReleaseCache(cache1)
	ReleaseCache(cache2)
	ReleaseCache(cache3)
}

func TestInitCacheEmptyCapabilities(t *testing.T) {
	tmpdir, err := os.MkdirTemp("", "wasmvm-testing")
	require.NoError(t, err)
	defer os.RemoveAll(tmpdir)
	cache, err := InitCache(tmpdir, []string{}, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err)
	ReleaseCache(cache)
}

func withCache(t testing.TB) (Cache, func()) {
	tmpdir, err := os.MkdirTemp("", "wasmvm-testing")
	require.NoError(t, err)
	cache, err := InitCache(tmpdir, TESTING_CAPABILITIES, TESTING_CACHE_SIZE, TESTING_MEMORY_LIMIT)
	require.NoError(t, err)

	cleanup := func() {
		os.RemoveAll(tmpdir)
		ReleaseCache(cache)
	}
	return cache, cleanup
}

func TestStoreCodeAndGetCode(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)

	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)
	expectedChecksum := sha256.Sum256(wasm)
	require.Equal(t, expectedChecksum[:], checksum)

	code, err := GetCode(cache, checksum)
	require.NoError(t, err)
	require.Equal(t, wasm, code)
}

func TestRemoveCode(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)

	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	// First removal works
	err = RemoveCode(cache, checksum)
	require.NoError(t, err)

	// Second removal fails
	err = RemoveCode(cache, checksum)
	require.ErrorContains(t, err, "Wasm file does not exist")
}

func TestStoreCodeFailsWithBadData(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm := []byte("some invalid data")
	_, err := StoreCode(cache, wasm)
	require.Error(t, err)
}

func TestStoreCodeUnchecked(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)

	checksum, err := StoreCodeUnchecked(cache, wasm)
	require.NoError(t, err)
	expectedChecksum := sha256.Sum256(wasm)
	require.Equal(t, expectedChecksum[:], checksum)

	code, err := GetCode(cache, checksum)
	require.NoError(t, err)
	require.Equal(t, wasm, code)
}

func TestPin(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)

	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	err = Pin(cache, checksum)
	require.NoError(t, err)

	// Can be called again with no effect
	err = Pin(cache, checksum)
	require.NoError(t, err)
}

func TestPinErrors(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	var err error

	// Nil checksum (errors in wasmvm Rust code)
	var nilChecksum []byte
	err = Pin(cache, nilChecksum)
	require.ErrorContains(t, err, "Null/Nil argument: checksum")

	// Checksum too short (errors in wasmvm Rust code)
	brokenChecksum := []byte{0x3f, 0xd7, 0x5a, 0x76}
	err = Pin(cache, brokenChecksum)
	require.ErrorContains(t, err, "Checksum not of length 32")

	// Unknown checksum (errors in cosmwasm-vm)
	unknownChecksum := []byte{
		0x72, 0x2c, 0x8c, 0x99, 0x3f, 0xd7, 0x5a, 0x76, 0x27, 0xd6, 0x9e, 0xd9, 0x41, 0x34,
		0x4f, 0xe2, 0xa1, 0x42, 0x3a, 0x3e, 0x75, 0xef, 0xd3, 0xe6, 0x77, 0x8a, 0x14, 0x28,
		0x84, 0x22, 0x71, 0x04,
	}
	err = Pin(cache, unknownChecksum)
	require.ErrorContains(t, err, "Error opening Wasm file for reading")
}

func TestUnpin(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)

	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	err = Pin(cache, checksum)
	require.NoError(t, err)

	err = Unpin(cache, checksum)
	require.NoError(t, err)

	// Can be called again with no effect
	err = Unpin(cache, checksum)
	require.NoError(t, err)
}

func TestUnpinErrors(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	var err error

	// Nil checksum (errors in wasmvm Rust code)
	var nilChecksum []byte
	err = Unpin(cache, nilChecksum)
	require.ErrorContains(t, err, "Null/Nil argument: checksum")

	// Checksum too short (errors in wasmvm Rust code)
	brokenChecksum := []byte{0x3f, 0xd7, 0x5a, 0x76}
	err = Unpin(cache, brokenChecksum)
	require.ErrorContains(t, err, "Checksum not of length 32")

	// No error case triggered in cosmwasm-vm is known right now
}

func TestGetMetrics(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	// GetMetrics 1
	metrics, err := GetMetrics(cache)
	require.NoError(t, err)
	assert.Equal(t, &types.Metrics{}, metrics)

	// Store contract
	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)
	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	// GetMetrics 2
	metrics, err = GetMetrics(cache)
	require.NoError(t, err)
	assert.Equal(t, &types.Metrics{}, metrics)

	// Instantiate 1
	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")
	msg1 := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	_, _, err = Instantiate(cache, checksum, env, info, msg1, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// GetMetrics 3
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(0), metrics.HitsMemoryCache)
	require.Equal(t, uint32(1), metrics.HitsFsCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)

	// Instantiate 2
	msg2 := []byte(`{"verifier": "fred", "beneficiary": "susi"}`)
	_, _, err = Instantiate(cache, checksum, env, info, msg2, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// GetMetrics 4
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(1), metrics.HitsMemoryCache)
	require.Equal(t, uint32(1), metrics.HitsFsCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)

	// Pin
	err = Pin(cache, checksum)
	require.NoError(t, err)

	// GetMetrics 5
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(1), metrics.HitsMemoryCache)
	require.Equal(t, uint32(2), metrics.HitsFsCache)
	require.Equal(t, uint64(1), metrics.ElementsPinnedMemoryCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizePinnedMemoryCache, 0.25)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)

	// Instantiate 3
	msg3 := []byte(`{"verifier": "fred", "beneficiary": "bert"}`)
	_, _, err = Instantiate(cache, checksum, env, info, msg3, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// GetMetrics 6
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(1), metrics.HitsPinnedMemoryCache)
	require.Equal(t, uint32(1), metrics.HitsMemoryCache)
	require.Equal(t, uint32(2), metrics.HitsFsCache)
	require.Equal(t, uint64(1), metrics.ElementsPinnedMemoryCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizePinnedMemoryCache, 0.25)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)

	// Unpin
	err = Unpin(cache, checksum)
	require.NoError(t, err)

	// GetMetrics 7
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(1), metrics.HitsPinnedMemoryCache)
	require.Equal(t, uint32(1), metrics.HitsMemoryCache)
	require.Equal(t, uint32(2), metrics.HitsFsCache)
	require.Equal(t, uint64(0), metrics.ElementsPinnedMemoryCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.Equal(t, uint64(0), metrics.SizePinnedMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)

	// Instantiate 4
	msg4 := []byte(`{"verifier": "fred", "beneficiary": "jeff"}`)
	_, _, err = Instantiate(cache, checksum, env, info, msg4, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// GetMetrics 8
	metrics, err = GetMetrics(cache)
	assert.NoError(t, err)
	require.Equal(t, uint32(1), metrics.HitsPinnedMemoryCache)
	require.Equal(t, uint32(2), metrics.HitsMemoryCache)
	require.Equal(t, uint32(2), metrics.HitsFsCache)
	require.Equal(t, uint64(0), metrics.ElementsPinnedMemoryCache)
	require.Equal(t, uint64(1), metrics.ElementsMemoryCache)
	require.Equal(t, uint64(0), metrics.SizePinnedMemoryCache)
	require.InEpsilon(t, 3700000, metrics.SizeMemoryCache, 0.25)
}

func TestGetPinnedMetrics(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	// GetMetrics 1
	metrics, err := GetPinnedMetrics(cache)
	require.NoError(t, err)
	assert.Equal(t, &types.PinnedMetrics{PerModule: make([]types.PerModuleEntry, 0)}, metrics)

	// Store contract 1
	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)
	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	err = Pin(cache, checksum)
	require.NoError(t, err)

	// Store contract 2
	cyberpunkWasm, err := os.ReadFile("../../testdata/cyberpunk.wasm")
	require.NoError(t, err)
	cyberpunkChecksum, err := StoreCode(cache, cyberpunkWasm)
	require.NoError(t, err)

	err = Pin(cache, cyberpunkChecksum)
	require.NoError(t, err)

	findMetrics := func(list []types.PerModuleEntry, checksum types.Checksum) *types.PerModuleMetrics {
		found := (*types.PerModuleMetrics)(nil)

		for _, structure := range list {
			if bytes.Equal(structure.Checksum, checksum) {
				found = &structure.Metrics
				break
			}
		}

		return found
	}

	// GetMetrics 2
	metrics, err = GetPinnedMetrics(cache)
	require.NoError(t, err)
	assert.Equal(t, 2, len(metrics.PerModule))

	hackatomMetrics := findMetrics(metrics.PerModule, checksum)
	cyberpunkMetrics := findMetrics(metrics.PerModule, cyberpunkChecksum)

	assert.Equal(t, uint32(0), hackatomMetrics.Hits)
	assert.NotEqual(t, uint32(0), hackatomMetrics.Size)
	assert.Equal(t, uint32(0), cyberpunkMetrics.Hits)
	assert.NotEqual(t, uint32(0), cyberpunkMetrics.Size)

	// Instantiate 1
	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")
	msg1 := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	_, _, err = Instantiate(cache, checksum, env, info, msg1, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// GetMetrics 3
	metrics, err = GetPinnedMetrics(cache)
	require.NoError(t, err)
	assert.Equal(t, 2, len(metrics.PerModule))

	hackatomMetrics = findMetrics(metrics.PerModule, checksum)
	cyberpunkMetrics = findMetrics(metrics.PerModule, cyberpunkChecksum)

	assert.Equal(t, uint32(1), hackatomMetrics.Hits)
	assert.NotEqual(t, uint32(0), hackatomMetrics.Size)
	assert.Equal(t, uint32(0), cyberpunkMetrics.Hits)
	assert.NotEqual(t, uint32(0), cyberpunkMetrics.Size)
}

func TestInstantiate(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()

	// create contract
	wasm, err := os.ReadFile("../../testdata/hackatom.wasm")
	require.NoError(t, err)
	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)

	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	// instantiate it with this store
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)

	res, cost, err := Instantiate(cache, checksum, env, info, msg, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(0x8c288c), cost.UsedInternally)

	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Equal(t, "", result.Err)
	require.Equal(t, 0, len(result.Ok.Messages))
}

func TestExecute(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)

	start := time.Now()
	res, cost, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	diff := time.Since(start)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(0x8c288c), cost.UsedInternally)
	t.Logf("Time (%d gas): %s\n", cost.UsedInternally, diff)

	// execute with the same store
	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	env = MockEnvBin(t)
	info = MockInfoBin(t, "fred")
	start = time.Now()
	res, cost, err = Execute(cache, checksum, env, info, []byte(`{"release":{}}`), &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	diff = time.Since(start)
	require.NoError(t, err)
	assert.Equal(t, uint64(0xfaea3d), cost.UsedInternally)
	t.Logf("Time (%d gas): %s\n", cost.UsedInternally, diff)

	// make sure it read the balance properly and we got 250 atoms
	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Equal(t, "", result.Err)
	require.Equal(t, 1, len(result.Ok.Messages))

	// Ensure we got our custom event
	assert.Equal(t, len(result.Ok.Events), 1)
	ev := result.Ok.Events[0]
	assert.Equal(t, ev.Type, "hackatom")
	assert.Equal(t, len(ev.Attributes), 1)
	assert.Equal(t, ev.Attributes[0].Key, "action")
	assert.Equal(t, ev.Attributes[0].Value, "release")

	dispatch := result.Ok.Messages[0].Msg
	require.NotNil(t, dispatch.Bank, "%#v", dispatch)
	require.NotNil(t, dispatch.Bank.Send, "%#v", dispatch)
	send := dispatch.Bank.Send
	assert.Equal(t, "bob", send.ToAddress)
	assert.Equal(t, balance, send.Amount)
	// check the data is properly formatted
	expectedData := []byte{0xF0, 0x0B, 0xAA}
	assert.Equal(t, expectedData, result.Ok.Data)
}

func TestExecutePanic(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createCyberpunkContract(t, cache)

	maxGas := TESTING_GAS_LIMIT
	gasMeter1 := NewMockGasMeter(maxGas)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	res, _, err := Instantiate(cache, checksum, env, info, []byte(`{}`), &igasMeter1, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// execute a panic
	gasMeter2 := NewMockGasMeter(maxGas)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	info = MockInfoBin(t, "fred")
	_, _, err = Execute(cache, checksum, env, info, []byte(`{"panic":{}}`), &igasMeter2, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.ErrorContains(t, err, "RuntimeError: Aborted: panicked at 'This page intentionally faulted'")
}

func TestExecuteUnreachable(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createCyberpunkContract(t, cache)

	maxGas := TESTING_GAS_LIMIT
	gasMeter1 := NewMockGasMeter(maxGas)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	res, _, err := Instantiate(cache, checksum, env, info, []byte(`{}`), &igasMeter1, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// execute a panic
	gasMeter2 := NewMockGasMeter(maxGas)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	info = MockInfoBin(t, "fred")
	_, _, err = Execute(cache, checksum, env, info, []byte(`{"unreachable":{}}`), &igasMeter2, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.ErrorContains(t, err, "RuntimeError: unreachable")
}

func TestExecuteCpuLoop(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createCyberpunkContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{}`)

	start := time.Now()
	res, cost, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	diff := time.Since(start)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(0x64fb31), cost.UsedInternally)
	t.Logf("Time (%d gas): %s\n", cost.UsedInternally, diff)

	// execute a cpu loop
	maxGas := uint64(40_000_000)
	gasMeter2 := NewMockGasMeter(maxGas)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	info = MockInfoBin(t, "fred")
	start = time.Now()
	_, cost, err = Execute(cache, checksum, env, info, []byte(`{"cpu_loop":{}}`), &igasMeter2, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	diff = time.Since(start)
	require.Error(t, err)
	assert.Equal(t, cost.UsedInternally, maxGas)
	t.Logf("CPULoop Time (%d gas): %s\n", cost.UsedInternally, diff)
}

func TestExecuteStorageLoop(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createCyberpunkContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{}`)

	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// execute a storage loop
	maxGas := uint64(40_000_000)
	gasMeter2 := NewMockGasMeter(maxGas)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	info = MockInfoBin(t, "fred")
	start := time.Now()
	_, gasReport, err := Execute(cache, checksum, env, info, []byte(`{"storage_loop":{}}`), &igasMeter2, store, api, &querier, maxGas, TESTING_PRINT_DEBUG)
	diff := time.Since(start)
	require.Error(t, err)
	t.Logf("StorageLoop Time (%d gas): %s\n", gasReport.UsedInternally, diff)
	t.Logf("Gas used: %d\n", gasMeter2.GasConsumed())
	t.Logf("Wasm gas: %d\n", gasReport.UsedInternally)

	// the "sdk gas" * GasMultiplier + the wasm cost should equal the maxGas (or be very close)
	totalCost := gasReport.UsedInternally + gasMeter2.GasConsumed()
	require.Equal(t, int64(maxGas), int64(totalCost))
}

func BenchmarkContractCall(b *testing.B) {
	cache, cleanup := withCache(b)
	defer cleanup()

	checksum := createCyberpunkContract(b, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(b)
	info := MockInfoBin(b, "creator")

	msg := []byte(`{}`)

	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(b, err)
	requireOkResponse(b, res, 0)

	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
		igasMeter2 := types.GasMeter(gasMeter2)
		store.SetGasMeter(gasMeter2)
		info = MockInfoBin(b, "fred")
		msg := []byte(`{"allocate_large_memory":{"pages":0}}`) // replace with noop once we have it
		res, _, err = Execute(cache, checksum, env, info, msg, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
		require.NoError(b, err)
		requireOkResponse(b, res, 0)
	}
}

func Benchmark100ConcurrentContractCalls(b *testing.B) {
	cache, cleanup := withCache(b)
	defer cleanup()

	checksum := createCyberpunkContract(b, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(b)
	info := MockInfoBin(b, "creator")

	msg := []byte(`{}`)

	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(b, err)
	requireOkResponse(b, res, 0)

	const callCount = 100 // Calls per benchmark iteration

	b.ResetTimer()
	for n := 0; n < b.N; n++ {
		var wg sync.WaitGroup
		wg.Add(callCount)
		for i := 0; i < callCount; i++ {
			go func() {
				gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
				igasMeter2 := types.GasMeter(gasMeter2)
				store.SetGasMeter(gasMeter2)
				info = MockInfoBin(b, "fred")
				msg := []byte(`{"allocate_large_memory":{"pages":0}}`) // replace with noop once we have it
				res, _, err = Execute(cache, checksum, env, info, msg, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
				require.NoError(b, err)
				requireOkResponse(b, res, 0)

				wg.Done()
			}()
		}
		wg.Wait()
	}
}

func TestExecuteUserErrorsInApiCalls(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	maxGas := TESTING_GAS_LIMIT
	gasMeter1 := NewMockGasMeter(maxGas)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	defaultApi := NewMockAPI()
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, defaultApi, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	gasMeter2 := NewMockGasMeter(maxGas)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	info = MockInfoBin(t, "fred")
	failingApi := NewMockFailureAPI()
	res, _, err = Execute(cache, checksum, env, info, []byte(`{"user_errors_in_api_calls":{}}`), &igasMeter2, store, failingApi, &querier, maxGas, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
}

func TestMigrate(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	// instantiate it with this store
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)

	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// verifier is fred
	query := []byte(`{"verifier":{}}`)
	data, _, err := Query(cache, checksum, env, query, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult types.QueryResult
	err = json.Unmarshal(data, &qResult)
	require.NoError(t, err)
	require.Equal(t, "", qResult.Err)
	require.Equal(t, string(qResult.Ok), `{"verifier":"fred"}`)

	// migrate to a new verifier - alice
	// we use the same code blob as we are testing hackatom self-migration
	_, _, err = Migrate(cache, checksum, env, []byte(`{"verifier":"alice"}`), &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// should update verifier to alice
	data, _, err = Query(cache, checksum, env, query, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult2 types.QueryResult
	err = json.Unmarshal(data, &qResult2)
	require.NoError(t, err)
	require.Equal(t, "", qResult2.Err)
	require.Equal(t, `{"verifier":"alice"}`, string(qResult2.Ok))
}

func TestMultipleInstances(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	// instance1 controlled by fred
	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	store1 := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "regen")
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	res, cost, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store1, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
	// we now count wasm gas charges and db writes
	assert.Equal(t, uint64(0x8b0ca4), cost.UsedInternally)

	// instance2 controlled by mary
	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store2 := NewLookup(gasMeter2)
	info = MockInfoBin(t, "chrous")
	msg = []byte(`{"verifier": "mary", "beneficiary": "sue"}`)
	res, cost, err = Instantiate(cache, checksum, env, info, msg, &igasMeter2, store2, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)
	assert.Equal(t, uint64(0x8baa51), cost.UsedInternally)

	// fail to execute store1 with mary
	resp := exec(t, cache, checksum, "mary", store1, api, querier, 0x85e616)
	require.Equal(t, "Unauthorized", resp.Err)

	// succeed to execute store1 with fred
	resp = exec(t, cache, checksum, "fred", store1, api, querier, 0xfa70f3)
	require.Equal(t, "", resp.Err)
	require.Equal(t, 1, len(resp.Ok.Messages))
	attributes := resp.Ok.Attributes
	require.Equal(t, 2, len(attributes))
	require.Equal(t, "destination", attributes[1].Key)
	require.Equal(t, "bob", attributes[1].Value)

	// succeed to execute store2 with mary
	resp = exec(t, cache, checksum, "mary", store2, api, querier, 0xfaad98)
	require.Equal(t, "", resp.Err)
	require.Equal(t, 1, len(resp.Ok.Messages))
	attributes = resp.Ok.Attributes
	require.Equal(t, 2, len(attributes))
	require.Equal(t, "destination", attributes[1].Key)
	require.Equal(t, "sue", attributes[1].Value)
}

func TestSudo(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	balance := types.Array[types.Coin]{types.NewCoin(250, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, balance)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// call sudo with same store
	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	env = MockEnvBin(t)
	msg = []byte(`{"steal_funds":{"recipient":"community-pool","amount":[{"amount":"700","denom":"gold"}]}}`)
	res, _, err = Sudo(cache, checksum, env, msg, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// make sure it blindly followed orders
	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Equal(t, "", result.Err)
	require.Equal(t, 1, len(result.Ok.Messages))
	dispatch := result.Ok.Messages[0].Msg
	require.NotNil(t, dispatch.Bank, "%#v", dispatch)
	require.NotNil(t, dispatch.Bank.Send, "%#v", dispatch)
	send := dispatch.Bank.Send
	assert.Equal(t, "community-pool", send.ToAddress)
	expectedPayout := types.Array[types.Coin]{types.NewCoin(700, "gold")}
	assert.Equal(t, expectedPayout, send.Amount)
}

func TestDispatchSubmessage(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createReflectContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{}`)
	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// dispatch a submessage
	var id uint64 = 1234
	payload := types.SubMsg{
		ID: id,
		Msg: types.CosmosMsg{Bank: &types.BankMsg{Send: &types.SendMsg{
			ToAddress: "friend",
			Amount:    types.Array[types.Coin]{types.NewCoin(1, "token")},
		}}},
		ReplyOn: types.ReplyAlways,
	}
	payloadBin, err := json.Marshal(payload)
	require.NoError(t, err)
	payloadMsg := []byte(fmt.Sprintf(`{"reflect_sub_msg":{"msgs":[%s]}}`, string(payloadBin)))

	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	env = MockEnvBin(t)
	res, _, err = Execute(cache, checksum, env, info, payloadMsg, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// make sure it blindly followed orders
	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Equal(t, "", result.Err)
	require.Equal(t, 1, len(result.Ok.Messages))
	dispatch := result.Ok.Messages[0]
	assert.Equal(t, id, dispatch.ID)
	assert.Equal(t, payload.Msg, dispatch.Msg)
	assert.Nil(t, dispatch.GasLimit)
	assert.Equal(t, payload.ReplyOn, dispatch.ReplyOn)
}

func TestReplyAndQuery(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createReflectContract(t, cache)

	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	// instantiate it with this store
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")

	msg := []byte(`{}`)
	res, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	var id uint64 = 1234
	data := []byte("foobar")
	events := types.Array[types.Event]{{
		Type: "message",
		Attributes: types.Array[types.EventAttribute]{{
			Key:   "signer",
			Value: "caller-addr",
		}},
	}}
	reply := types.Reply{
		ID: id,
		Result: types.SubMsgResult{
			Ok: &types.SubMsgResponse{
				Events: events,
				Data:   data,
			},
		},
	}
	replyBin, err := json.Marshal(reply)
	require.NoError(t, err)

	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	env = MockEnvBin(t)
	res, _, err = Reply(cache, checksum, env, replyBin, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireOkResponse(t, res, 0)

	// now query the state to see if it stored the data properly
	badQuery := []byte(`{"sub_msg_result":{"id":7777}}`)
	res, _, err = Query(cache, checksum, env, badQuery, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	requireQueryError(t, res)

	query := []byte(`{"sub_msg_result":{"id":1234}}`)
	res, _, err = Query(cache, checksum, env, query, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	qResult := requireQueryOk(t, res)

	var stored types.Reply
	err = json.Unmarshal(qResult, &stored)
	require.NoError(t, err)
	assert.Equal(t, id, stored.ID)
	require.NotNil(t, stored.Result.Ok)
	val := stored.Result.Ok
	require.Equal(t, data, val.Data)
	require.Equal(t, events, val.Events)
}

func requireOkResponse(t testing.TB, res []byte, expectedMsgs int) {
	var result types.ContractResult
	err := json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Equal(t, "", result.Err)
	require.Equal(t, expectedMsgs, len(result.Ok.Messages))
}

func requireQueryError(t *testing.T, res []byte) {
	var result types.QueryResult
	err := json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Empty(t, result.Ok)
	require.NotEmpty(t, result.Err)
}

func requireQueryOk(t *testing.T, res []byte) []byte {
	var result types.QueryResult
	err := json.Unmarshal(res, &result)
	require.NoError(t, err)
	require.Empty(t, result.Err)
	require.NotEmpty(t, result.Ok)
	return result.Ok
}

func createHackatomContract(t testing.TB, cache Cache) []byte {
	return createContract(t, cache, "../../testdata/hackatom.wasm")
}

func createCyberpunkContract(t testing.TB, cache Cache) []byte {
	return createContract(t, cache, "../../testdata/cyberpunk.wasm")
}

func createQueueContract(t testing.TB, cache Cache) []byte {
	return createContract(t, cache, "../../testdata/queue.wasm")
}

func createReflectContract(t testing.TB, cache Cache) []byte {
	return createContract(t, cache, "../../testdata/reflect.wasm")
}

func createFloaty2(t testing.TB, cache Cache) []byte {
	return createContract(t, cache, "../../testdata/floaty_2.0.wasm")
}

func createContract(t testing.TB, cache Cache, wasmFile string) []byte {
	wasm, err := os.ReadFile(wasmFile)
	require.NoError(t, err)
	checksum, err := StoreCode(cache, wasm)
	require.NoError(t, err)
	return checksum
}

// exec runs the handle tx with the given signer
func exec(t *testing.T, cache Cache, checksum []byte, signer types.HumanAddress, store types.KVStore, api *types.GoAPI, querier Querier, gasExpected uint64) types.ContractResult {
	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	env := MockEnvBin(t)
	info := MockInfoBin(t, signer)
	res, cost, err := Execute(cache, checksum, env, info, []byte(`{"release":{}}`), &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	assert.Equal(t, gasExpected, cost.UsedInternally)

	var result types.ContractResult
	err = json.Unmarshal(res, &result)
	require.NoError(t, err)
	return result
}

func TestQuery(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	// set up contract
	gasMeter1 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter1 := types.GasMeter(gasMeter1)
	store := NewLookup(gasMeter1)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, types.Array[types.Coin]{types.NewCoin(100, "ATOM")})
	env := MockEnvBin(t)
	info := MockInfoBin(t, "creator")
	msg := []byte(`{"verifier": "fred", "beneficiary": "bob"}`)
	_, _, err := Instantiate(cache, checksum, env, info, msg, &igasMeter1, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)

	// invalid query
	gasMeter2 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter2 := types.GasMeter(gasMeter2)
	store.SetGasMeter(gasMeter2)
	query := []byte(`{"Raw":{"val":"config"}}`)
	data, _, err := Query(cache, checksum, env, query, &igasMeter2, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var badResult types.QueryResult
	err = json.Unmarshal(data, &badResult)
	require.NoError(t, err)
	require.Contains(t, badResult.Err, "Error parsing into type hackatom::msg::QueryMsg: unknown variant `Raw`, expected one of")

	// make a valid query
	gasMeter3 := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter3 := types.GasMeter(gasMeter3)
	store.SetGasMeter(gasMeter3)
	query = []byte(`{"verifier":{}}`)
	data, _, err = Query(cache, checksum, env, query, &igasMeter3, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult types.QueryResult
	err = json.Unmarshal(data, &qResult)
	require.NoError(t, err)
	require.Equal(t, "", qResult.Err)
	require.Equal(t, string(qResult.Ok), `{"verifier":"fred"}`)
}

func TestHackatomQuerier(t *testing.T) {
	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createHackatomContract(t, cache)

	// set up contract
	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	initBalance := types.Array[types.Coin]{types.NewCoin(1234, "ATOM"), types.NewCoin(65432, "ETH")}
	querier := DefaultQuerier("foobar", initBalance)

	// make a valid query to the other address
	query := []byte(`{"other_balance":{"address":"foobar"}}`)
	// TODO The query happens before the contract is initialized. How is this legal?
	env := MockEnvBin(t)
	data, _, err := Query(cache, checksum, env, query, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult types.QueryResult
	err = json.Unmarshal(data, &qResult)
	require.NoError(t, err)
	require.Equal(t, "", qResult.Err)
	var balances types.AllBalancesResponse
	err = json.Unmarshal(qResult.Ok, &balances)
	require.NoError(t, err)
	require.Equal(t, balances.Amount, initBalance)
}

func TestCustomReflectQuerier(t *testing.T) {
	type CapitalizedQuery struct {
		Text string `json:"text"`
	}

	type QueryMsg struct {
		Capitalized *CapitalizedQuery `json:"capitalized,omitempty"`
		// There are more queries but we don't use them yet
		// https://github.com/CosmWasm/cosmwasm/blob/v0.11.0-alpha3/contracts/reflect/src/msg.rs#L18-L28
	}

	type CapitalizedResponse struct {
		Text string `json:"text"`
	}

	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createReflectContract(t, cache)

	// set up contract
	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	initBalance := types.Array[types.Coin]{types.NewCoin(1234, "ATOM")}
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, initBalance)
	// we need this to handle the custom requests from the reflect contract
	innerQuerier := querier.(*MockQuerier)
	innerQuerier.Custom = ReflectCustom{}
	querier = Querier(innerQuerier)

	// make a valid query to the other address
	queryMsg := QueryMsg{
		Capitalized: &CapitalizedQuery{
			Text: "small Frys :)",
		},
	}
	query, err := json.Marshal(queryMsg)
	require.NoError(t, err)
	env := MockEnvBin(t)
	data, _, err := Query(cache, checksum, env, query, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult types.QueryResult
	err = json.Unmarshal(data, &qResult)
	require.NoError(t, err)
	require.Equal(t, "", qResult.Err)

	var response CapitalizedResponse
	err = json.Unmarshal(qResult.Ok, &response)
	require.NoError(t, err)
	require.Equal(t, "SMALL FRYS :)", response.Text)
}

// TestFloats is a port of the float_instrs_are_deterministic test in cosmwasm-vm
func TestFloats(t *testing.T) {
	type Value struct {
		U32 *uint32 `json:"u32,omitempty"`
		U64 *uint64 `json:"u64,omitempty"`
		F32 *uint32 `json:"f32,omitempty"`
		F64 *uint64 `json:"f64,omitempty"`
	}

	// helper to print the value in the same format as Rust's Debug trait
	debugStr := func(value Value) string {
		if value.U32 != nil {
			return fmt.Sprintf("U32(%d)", *value.U32)
		} else if value.U64 != nil {
			return fmt.Sprintf("U64(%d)", *value.U64)
		} else if value.F32 != nil {
			return fmt.Sprintf("F32(%d)", *value.F32)
		} else if value.F64 != nil {
			return fmt.Sprintf("F64(%d)", *value.F64)
		} else {
			t.FailNow()
			return ""
		}
	}

	cache, cleanup := withCache(t)
	defer cleanup()
	checksum := createFloaty2(t, cache)

	gasMeter := NewMockGasMeter(TESTING_GAS_LIMIT)
	igasMeter := types.GasMeter(gasMeter)
	// instantiate it with this store
	store := NewLookup(gasMeter)
	api := NewMockAPI()
	querier := DefaultQuerier(MOCK_CONTRACT_ADDR, nil)
	env := MockEnvBin(t)

	// query instructions
	query := []byte(`{"instructions":{}}`)
	data, _, err := Query(cache, checksum, env, query, &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
	require.NoError(t, err)
	var qResult types.QueryResult
	err = json.Unmarshal(data, &qResult)
	require.NoError(t, err)
	require.Equal(t, "", qResult.Err)
	var instructions []string
	err = json.Unmarshal(qResult.Ok, &instructions)
	require.NoError(t, err)
	// little sanity check
	require.Equal(t, 70, len(instructions))

	hasher := sha256.New()
	const RUNS_PER_INSTRUCTION = 150
	for _, instr := range instructions {
		for seed := 0; seed < RUNS_PER_INSTRUCTION; seed++ {
			// query some input values for the instruction
			msg := fmt.Sprintf(`{"random_args_for":{"instruction":"%s","seed":%d}}`, instr, seed)
			data, _, err = Query(cache, checksum, env, []byte(msg), &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
			require.NoError(t, err)
			err = json.Unmarshal(data, &qResult)
			require.NoError(t, err)
			require.Equal(t, "", qResult.Err)
			var args []Value
			err = json.Unmarshal(qResult.Ok, &args)
			require.NoError(t, err)

			// build the run message
			argStr, err := json.Marshal(args)
			require.NoError(t, err)
			msg = fmt.Sprintf(`{"run":{"instruction":"%s","args":%s}}`, instr, argStr)

			// run the instruction
			// this might throw a runtime error (e.g. if the instruction traps)
			data, _, err = Query(cache, checksum, env, []byte(msg), &igasMeter, store, api, &querier, TESTING_GAS_LIMIT, TESTING_PRINT_DEBUG)
			var result string
			if err != nil {
				assert.ErrorContains(t, err, "Error calling the VM: Error executing Wasm: ")
				// remove the prefix to make the error message the same as in the cosmwasm-vm test
				result = strings.Replace(err.Error(), "Error calling the VM: Error executing Wasm: ", "", 1)
			} else {
				err = json.Unmarshal(data, &qResult)
				require.NoError(t, err)
				require.Equal(t, "", qResult.Err)
				var response Value
				err = json.Unmarshal(qResult.Ok, &response)
				require.NoError(t, err)
				result = debugStr(response)
			}
			// add the result to the hash
			hasher.Write([]byte(fmt.Sprintf("%s%d%s", instr, seed, result)))
		}
	}

	hash := hasher.Sum(nil)
	require.Equal(t, "95f70fa6451176ab04a9594417a047a1e4d8e2ff809609b8f81099496bee2393", hex.EncodeToString(hash))
}
