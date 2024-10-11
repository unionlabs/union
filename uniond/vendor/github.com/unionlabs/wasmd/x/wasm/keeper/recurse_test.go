package keeper

import (
	"encoding/json"
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	storetypes "cosmossdk.io/store/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"

	"github.com/CosmWasm/wasmd/x/wasm/types"
)

type Recurse struct {
	Depth uint32 `json:"depth"`
	Work  uint32 `json:"work"`
}

type recurseWrapper struct {
	Recurse Recurse `json:"recurse"`
}

func buildRecurseQuery(t *testing.T, msg Recurse) []byte {
	t.Helper()
	wrapper := recurseWrapper{Recurse: msg}
	bz, err := json.Marshal(wrapper)
	require.NoError(t, err)
	return bz
}

type recurseResponse struct {
	Hashed []byte `json:"hashed"`
}

// number os wasm queries called from a contract
var totalWasmQueryCounter int

func initRecurseContract(t *testing.T) (contract sdk.AccAddress, ctx sdk.Context, keeper *Keeper) {
	t.Helper()
	countingQuerierDec := func(realWasmQuerier WasmVMQueryHandler) WasmVMQueryHandler {
		return WasmVMQueryHandlerFn(func(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error) {
			totalWasmQueryCounter++
			return realWasmQuerier.HandleQuery(ctx, caller, request)
		})
	}
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities, WithQueryHandlerDecorator(countingQuerierDec))
	keeper = keepers.WasmKeeper
	exampleContract := InstantiateHackatomExampleContract(t, ctx, keepers)
	return exampleContract.Contract, ctx, keeper
}

func TestGasCostOnQuery(t *testing.T) {
	const (
		GasNoWork uint64 = 63_968
		// Note: about 100 SDK gas (10k CosmWasm gas) for each round of sha256
		GasWork50 uint64 = 64_207 // this is a little shy of 50k gas - to keep an eye on the limit

		GasReturnUnhashed uint64 = 55
		GasReturnHashed   uint64 = 46
	)

	cases := map[string]struct {
		gasLimit    uint64
		msg         Recurse
		expectedGas uint64
	}{
		"no recursion, no work": {
			gasLimit:    400_000,
			msg:         Recurse{},
			expectedGas: GasNoWork,
		},
		"no recursion, some work": {
			gasLimit: 400_000,
			msg: Recurse{
				Work: 50, // 50 rounds of sha256 inside the contract
			},
			expectedGas: GasWork50,
		},
		"recursion 1, no work": {
			gasLimit: 400_000,
			msg: Recurse{
				Depth: 1,
			},
			expectedGas: 2*GasNoWork + GasReturnUnhashed,
		},
		"recursion 1, some work": {
			gasLimit: 400_000,
			msg: Recurse{
				Depth: 1,
				Work:  50,
			},
			expectedGas: 2*GasWork50 + GasReturnHashed,
		},
		"recursion 4, some work": {
			gasLimit: 400_000,
			msg: Recurse{
				Depth: 4,
				Work:  50,
			},
			expectedGas: 5*GasWork50 + 4*GasReturnHashed,
		},
	}

	contractAddr, ctx, keeper := initRecurseContract(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			// external limit has no effect (we get a panic if this is enforced)
			keeper.queryGasLimit = 1000

			// make sure we set a limit before calling
			ctx = ctx.WithGasMeter(storetypes.NewGasMeter(tc.gasLimit))
			require.Equal(t, uint64(0), ctx.GasMeter().GasConsumed())

			// do the query
			recurse := tc.msg
			msg := buildRecurseQuery(t, recurse)
			data, err := keeper.QuerySmart(ctx, contractAddr, msg)
			require.NoError(t, err)

			// check the gas is what we expected
			if types.EnableGasVerification {
				assert.Equal(t, tc.expectedGas, ctx.GasMeter().GasConsumed())
			}
			// assert result is 32 byte sha256 hash (if hashed), or contractAddr if not
			var resp recurseResponse
			err = json.Unmarshal(data, &resp)
			require.NoError(t, err)
			if recurse.Work == 0 {
				assert.Equal(t, len(contractAddr.String()), len(resp.Hashed))
			} else {
				assert.Equal(t, 32, len(resp.Hashed))
			}
		})
	}
}

func TestGasOnExternalQuery(t *testing.T) {
	const (
		GasWork50 uint64 = types.DefaultInstanceCost + 8_464
	)

	cases := map[string]struct {
		gasLimit    uint64
		msg         Recurse
		expOutOfGas bool
	}{
		"no recursion, plenty gas": {
			gasLimit: 400_000,
			msg: Recurse{
				Work: 50, // 50 rounds of sha256 inside the contract
			},
		},
		"recursion 4, plenty gas": {
			// this uses 244708 gas
			gasLimit: 400_000,
			msg: Recurse{
				Depth: 4,
				Work:  50,
			},
		},
		"no recursion, external gas limit": {
			gasLimit: 5000, // this is not enough
			msg: Recurse{
				Work: 50,
			},
			expOutOfGas: true,
		},
		"recursion 4, external gas limit": {
			// this uses 244708 gas but give less
			gasLimit: 4 * GasWork50,
			msg: Recurse{
				Depth: 4,
				Work:  50,
			},
			expOutOfGas: true,
		},
	}

	contractAddr, ctx, keeper := initRecurseContract(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			recurse := tc.msg
			msg := buildRecurseQuery(t, recurse)

			querier := NewGrpcQuerier(keeper.cdc, keeper.storeService, keeper, tc.gasLimit)
			req := &types.QuerySmartContractStateRequest{Address: contractAddr.String(), QueryData: msg}
			_, gotErr := querier.SmartContractState(ctx, req)
			if tc.expOutOfGas {
				require.Error(t, gotErr, sdkerrors.ErrOutOfGas)
				return
			}
			require.NoError(t, gotErr)
		})
	}
}

func TestLimitRecursiveQueryGas(t *testing.T) {
	// The point of this test from https://github.com/CosmWasm/cosmwasm/issues/456
	// Basically, if I burn 90% of gas in CPU loop, then query out (to my self)
	// the sub-query will have all the original gas (minus the 40k instance charge)
	// and can burn 90% and call a sub-contract again...
	// This attack would allow us to use far more than the provided gas before
	// eventually hitting an OutOfGas panic.

	const (
		// Note: about 100 SDK gas (10k CosmWasm gas) for each round of sha256
		GasWork2k uint64 = 76_264 // = SetupContractCost + x // we have 6x gas used in cpu than in the instance
		// This is overhead for calling into a sub-contract
		GasReturnHashed uint64 = 48
	)

	cases := map[string]struct {
		gasLimit                  uint64
		msg                       Recurse
		expectQueriesFromContract int
		expectedGas               uint64
		expectOutOfGas            bool
		expectError               string
	}{
		"no recursion, lots of work": {
			gasLimit: 4_000_000,
			msg: Recurse{
				Depth: 0,
				Work:  2000,
			},
			expectQueriesFromContract: 0,
			expectedGas:               GasWork2k,
		},
		"recursion 5, lots of work": {
			gasLimit: 4_000_000,
			msg: Recurse{
				Depth: 5,
				Work:  2000,
			},
			expectQueriesFromContract: 5,
			expectedGas:               GasWork2k + 5*(GasWork2k+GasReturnHashed),
		},
		// this is where we expect an error...
		// it has enough gas to run 5 times and die on the 6th (5th time dispatching to sub-contract)
		// however, if we don't charge the cpu gas before sub-dispatching, we can recurse over 20 times
		"deep recursion, should die on 5th level": {
			gasLimit: 400_000,
			msg: Recurse{
				Depth: 50,
				Work:  2000,
			},
			expectQueriesFromContract: 5,
			expectOutOfGas:            true,
		},
		"very deep recursion, hits recursion limit": {
			gasLimit: 10_000_000,
			msg: Recurse{
				Depth: 100,
				Work:  2000,
			},
			expectQueriesFromContract: 10,
			expectOutOfGas:            false,
			expectError:               "query wasm contract failed",          // Error we get from the contract instance doing the failing query, not wasmd
			expectedGas:               10*(GasWork2k+GasReturnHashed) + 3279, // lots of additional gas for long error message
		},
	}

	contractAddr, ctx, keeper := initRecurseContract(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			// reset the counter before test
			totalWasmQueryCounter = 0

			// make sure we set a limit before calling
			ctx = ctx.WithGasMeter(storetypes.NewGasMeter(tc.gasLimit))
			require.Equal(t, uint64(0), ctx.GasMeter().GasConsumed())

			// prepare the query
			recurse := tc.msg
			msg := buildRecurseQuery(t, recurse)

			// if we expect out of gas, make sure this panics
			if tc.expectOutOfGas {
				require.Panics(t, func() {
					_, err := keeper.QuerySmart(ctx, contractAddr, msg)
					t.Logf("Got error not panic: %#v", err)
				})
				assert.Equal(t, tc.expectQueriesFromContract, totalWasmQueryCounter)
				return
			}

			// otherwise, we expect a successful call
			_, err := keeper.QuerySmart(ctx, contractAddr, msg)
			if tc.expectError != "" {
				require.ErrorContains(t, err, tc.expectError)
			} else {
				require.NoError(t, err)
			}
			if types.EnableGasVerification {
				assert.Equal(t, tc.expectedGas, ctx.GasMeter().GasConsumed())
			}
			assert.Equal(t, tc.expectQueriesFromContract, totalWasmQueryCounter)
		})
	}
}
