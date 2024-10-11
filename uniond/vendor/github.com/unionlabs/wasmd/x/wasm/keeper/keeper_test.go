package keeper

import (
	"bytes"
	_ "embed"
	"encoding/json"
	"errors"
	"fmt"
	stdrand "math/rand"
	"os"
	"testing"
	"time"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	abci "github.com/cometbft/cometbft/abci/types"
	"cosmossdk.io/math/unsafe"
	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	fuzz "github.com/google/gofuzz"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/auth/vesting"
	vestingtypes "github.com/cosmos/cosmos-sdk/x/auth/vesting/types"
	banktypes "cosmossdk.io/x/bank/types"
	distributiontypes "cosmossdk.io/x/distribution/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

//go:embed testdata/hackatom.wasm
var hackatomWasm []byte

var AvailableCapabilities = []string{
	"iterator", "staking", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3",
	"cosmwasm_1_4", "cosmwasm_2_0", "cosmwasm_2_1",
}

func TestNewKeeper(t *testing.T) {
	_, keepers := CreateTestInput(t, false, AvailableCapabilities)
	require.NotNil(t, keepers.ContractKeeper)
}

func TestCreateSuccess(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	em := sdk.NewEventManager()
	contractID, _, err := keeper.Create(ctx.WithEventManager(em), creator, hackatomWasm, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), contractID)
	// and verify content
	storedCode, err := keepers.WasmKeeper.GetByteCode(ctx, contractID)
	require.NoError(t, err)
	require.Equal(t, hackatomWasm, storedCode)
	// and events emitted
	codeHash := testdata.ChecksumHackatom
	exp := sdk.Events{sdk.NewEvent("store_code", sdk.NewAttribute("code_checksum", codeHash), sdk.NewAttribute("code_id", "1"))}
	assert.Equal(t, exp, em.Events())
}

func TestCreateNilCreatorAddress(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	_, _, err := keepers.ContractKeeper.Create(ctx, nil, hackatomWasm, nil)
	require.Error(t, err, "nil creator is not allowed")
}

func TestCreateNilWasmCode(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	_, _, err := keepers.ContractKeeper.Create(ctx, creator, nil, nil)
	require.Error(t, err, "nil WASM code is not allowed")
}

func TestCreateInvalidWasmCode(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	_, _, err := keepers.ContractKeeper.Create(ctx, creator, []byte("potatoes"), nil)
	require.Error(t, err, "potatoes are not valid WASM code")
}

func TestCreateStoresInstantiatePermission(t *testing.T) {
	var (
		deposit                = sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
		myAddr  sdk.AccAddress = bytes.Repeat([]byte{1}, types.SDKAddrLen)
	)

	specs := map[string]struct {
		srcPermission types.AccessType
		expInstConf   types.AccessConfig
	}{
		"default": {
			srcPermission: types.DefaultParams().InstantiateDefaultPermission,
			expInstConf:   types.AllowEverybody,
		},
		"everybody": {
			srcPermission: types.AccessTypeEverybody,
			expInstConf:   types.AllowEverybody,
		},
		"nobody": {
			srcPermission: types.AccessTypeNobody,
			expInstConf:   types.AllowNobody,
		},
		"anyAddress with matching address": {
			srcPermission: types.AccessTypeAnyOfAddresses,
			expInstConf:   types.AccessConfig{Permission: types.AccessTypeAnyOfAddresses, Addresses: []string{myAddr.String()}},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
			accKeeper, keeper, bankKeeper := keepers.AccountKeeper, keepers.ContractKeeper, keepers.BankKeeper
			err := keepers.WasmKeeper.SetParams(ctx, types.Params{
				CodeUploadAccess:             types.AllowEverybody,
				InstantiateDefaultPermission: spec.srcPermission,
			})
			require.NoError(t, err)
			fundAccounts(t, ctx, accKeeper, bankKeeper, myAddr, deposit)

			codeID, _, err := keeper.Create(ctx, myAddr, hackatomWasm, nil)
			require.NoError(t, err)

			codeInfo := keepers.WasmKeeper.GetCodeInfo(ctx, codeID)
			require.NotNil(t, codeInfo)
			assert.True(t, spec.expInstConf.Equals(codeInfo.InstantiateConfig), "got %#v", codeInfo.InstantiateConfig)
		})
	}
}

func TestCreateWithParamPermissions(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	otherAddr := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	specs := map[string]struct {
		policy      types.AuthorizationPolicy
		chainUpload types.AccessConfig
		expError    *errorsmod.Error
	}{
		"default": {
			policy:      DefaultAuthorizationPolicy{},
			chainUpload: types.DefaultUploadAccess,
		},
		"everybody": {
			policy:      DefaultAuthorizationPolicy{},
			chainUpload: types.AllowEverybody,
		},
		"nobody": {
			policy:      DefaultAuthorizationPolicy{},
			chainUpload: types.AllowNobody,
			expError:    sdkerrors.ErrUnauthorized,
		},
		"anyAddress with matching address": {
			policy:      DefaultAuthorizationPolicy{},
			chainUpload: types.AccessTypeAnyOfAddresses.With(creator),
		},
		"anyAddress with non matching address": {
			policy:      DefaultAuthorizationPolicy{},
			chainUpload: types.AccessTypeAnyOfAddresses.With(otherAddr),
			expError:    sdkerrors.ErrUnauthorized,
		},
		"gov: always allowed": {
			policy:      GovAuthorizationPolicy{},
			chainUpload: types.AllowNobody,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			params := types.DefaultParams()
			params.CodeUploadAccess = spec.chainUpload
			err := keepers.WasmKeeper.SetParams(ctx, params)
			require.NoError(t, err)
			keeper := NewPermissionedKeeper(keepers.WasmKeeper, spec.policy)
			_, _, err = keeper.Create(ctx, creator, hackatomWasm, nil)
			require.True(t, spec.expError.Is(err), err)
			if spec.expError != nil {
				return
			}
		})
	}
}

// ensure that the user cannot set the code instantiate permission to something more permissive
// than the default
func TestEnforceValidPermissionsOnCreate(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.WasmKeeper
	contractKeeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	other := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	onlyCreator := types.AccessTypeAnyOfAddresses.With(creator)
	onlyOther := types.AccessTypeAnyOfAddresses.With(other)

	specs := map[string]struct {
		defaultPermssion    types.AccessType
		requestedPermission *types.AccessConfig
		// grantedPermission is set iff no error
		grantedPermission types.AccessConfig
		// expError is nil iff the request is allowed
		expError *errorsmod.Error
	}{
		"override everybody": {
			defaultPermssion:    types.AccessTypeEverybody,
			requestedPermission: &onlyCreator,
			grantedPermission:   onlyCreator,
		},
		"default to everybody": {
			defaultPermssion:    types.AccessTypeEverybody,
			requestedPermission: nil,
			grantedPermission:   types.AccessConfig{Permission: types.AccessTypeEverybody},
		},
		"explicitly set everybody": {
			defaultPermssion:    types.AccessTypeEverybody,
			requestedPermission: &types.AccessConfig{Permission: types.AccessTypeEverybody},
			grantedPermission:   types.AccessConfig{Permission: types.AccessTypeEverybody},
		},
		"cannot override nobody": {
			defaultPermssion:    types.AccessTypeNobody,
			requestedPermission: &onlyCreator,
			expError:            sdkerrors.ErrUnauthorized,
		},
		"default to nobody": {
			defaultPermssion:    types.AccessTypeNobody,
			requestedPermission: nil,
			grantedPermission:   types.AccessConfig{Permission: types.AccessTypeNobody},
		},
		"only defaults to code creator": {
			defaultPermssion:    types.AccessTypeAnyOfAddresses,
			requestedPermission: nil,
			grantedPermission:   onlyCreator,
		},
		"can explicitly set to code creator": {
			defaultPermssion:    types.AccessTypeAnyOfAddresses,
			requestedPermission: &onlyCreator,
			grantedPermission:   onlyCreator,
		},
		"cannot override which address in only": {
			defaultPermssion:    types.AccessTypeAnyOfAddresses,
			requestedPermission: &onlyOther,
			expError:            sdkerrors.ErrUnauthorized,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			params := types.DefaultParams()
			params.InstantiateDefaultPermission = spec.defaultPermssion
			err := keeper.SetParams(ctx, params)
			require.NoError(t, err)
			codeID, _, err := contractKeeper.Create(ctx, creator, hackatomWasm, spec.requestedPermission)
			require.True(t, spec.expError.Is(err), err)
			if spec.expError == nil {
				codeInfo := keeper.GetCodeInfo(ctx, codeID)
				require.Equal(t, codeInfo.InstantiateConfig, spec.grantedPermission)
			}
		})
	}
}

func TestCreateDuplicate(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	// create one copy
	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), contractID)

	// create second copy
	duplicateID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(2), duplicateID)

	// and verify both content is proper
	storedCode, err := keepers.WasmKeeper.GetByteCode(ctx, contractID)
	require.NoError(t, err)
	require.Equal(t, hackatomWasm, storedCode)
	storedCode, err = keepers.WasmKeeper.GetByteCode(ctx, duplicateID)
	require.NoError(t, err)
	require.Equal(t, hackatomWasm, storedCode)
}

func TestCreateWithSimulation(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	ctx = ctx.WithBlockHeader(tmproto.Header{Height: 1}).
		WithGasMeter(storetypes.NewInfiniteGasMeter())

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	// create this once in simulation mode
	contractID, _, err := keepers.ContractKeeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), contractID)

	// then try to create it in non-simulation mode (should not fail)
	ctx, keepers = CreateTestInput(t, false, AvailableCapabilities)
	ctx = ctx.WithGasMeter(storetypes.NewGasMeter(10_000_000))
	creator = keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	contractID, _, err = keepers.ContractKeeper.Create(ctx, creator, hackatomWasm, nil)

	require.NoError(t, err)
	require.Equal(t, uint64(1), contractID)

	// and verify content
	code, err := keepers.WasmKeeper.GetByteCode(ctx, contractID)
	require.NoError(t, err)
	require.Equal(t, code, hackatomWasm)
}

func TestIsSimulationMode(t *testing.T) {
	specs := map[string]struct {
		ctx sdk.Context
		exp bool
	}{
		"genesis block": {
			ctx: sdk.Context{}.WithBlockHeader(tmproto.Header{}).WithGasMeter(storetypes.NewInfiniteGasMeter()),
			exp: false,
		},
		"any regular block": {
			ctx: sdk.Context{}.WithBlockHeader(tmproto.Header{Height: 1}).WithGasMeter(storetypes.NewGasMeter(10000000)),
			exp: false,
		},
		"simulation": {
			ctx: sdk.Context{}.WithBlockHeader(tmproto.Header{Height: 1}).WithGasMeter(storetypes.NewInfiniteGasMeter()),
			exp: true,
		},
	}
	for msg := range specs {
		t.Run(msg, func(t *testing.T) {
			// assert.Equal(t, spec.exp, isSimulationMode(spec.ctx))
		})
	}
}

func TestCreateWithGzippedPayload(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	wasmCode, err := os.ReadFile("./testdata/hackatom.wasm.gzip")
	require.NoError(t, err, "reading gzipped WASM code")

	contractID, _, err := keeper.Create(ctx, creator, wasmCode, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), contractID)
	// and verify content
	storedCode, err := keepers.WasmKeeper.GetByteCode(ctx, contractID)
	require.NoError(t, err)
	require.Equal(t, hackatomWasm, storedCode)
}

func TestCreateWithBrokenGzippedPayload(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	wasmCode, err := os.ReadFile("./testdata/broken_crc.gzip")
	require.NoError(t, err, "reading gzipped WASM code")

	gm := storetypes.NewInfiniteGasMeter()
	codeID, checksum, err := keeper.Create(ctx.WithGasMeter(gm), creator, wasmCode, nil)
	require.Error(t, err)
	assert.Empty(t, codeID)
	assert.Empty(t, checksum)
	assert.GreaterOrEqual(t, gm.GasConsumed(), storetypes.Gas(121384)) // 809232 * 0.15 (default uncompress costs) = 121384
}

func TestInstantiate(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	keepers.Faucet.Fund(ctx, creator, deposit...)
	example := StoreHackatomExampleContract(t, ctx, keepers)

	initMsg := HackatomExampleInitMsg{
		Verifier:    RandomAccountAddress(t),
		Beneficiary: RandomAccountAddress(t),
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	gasBefore := ctx.GasMeter().GasConsumed()

	em := sdk.NewEventManager()
	// create with no balance is also legal
	gotContractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx.WithEventManager(em), example.CodeID, creator, nil, initMsgBz, "demo contract 1", nil)
	require.NoError(t, err)
	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", gotContractAddr.String())

	gasAfter := ctx.GasMeter().GasConsumed()
	if types.EnableGasVerification {
		require.Equal(t, uint64(0x1bc8f), gasAfter-gasBefore)
	}

	// ensure it is stored properly
	info := keepers.WasmKeeper.GetContractInfo(ctx, gotContractAddr)
	require.NotNil(t, info)
	assert.Equal(t, creator.String(), info.Creator)
	assert.Equal(t, example.CodeID, info.CodeID)
	assert.Equal(t, "demo contract 1", info.Label)

	exp := []types.ContractCodeHistoryEntry{{
		Operation: types.ContractCodeHistoryOperationTypeInit,
		CodeID:    example.CodeID,
		Updated:   types.NewAbsoluteTxPosition(ctx),
		Msg:       initMsgBz,
	}}
	assert.Equal(t, exp, keepers.WasmKeeper.GetContractHistory(ctx, gotContractAddr))

	// and events emitted
	expEvt := sdk.Events{
		sdk.NewEvent("instantiate",
			sdk.NewAttribute("_contract_address", gotContractAddr.String()), sdk.NewAttribute("code_id", "1")),
		sdk.NewEvent("wasm",
			sdk.NewAttribute("_contract_address", gotContractAddr.String()), sdk.NewAttribute("Let the", "hacking begin")),
	}
	assert.Equal(t, expEvt, em.Events())
}

func TestInstantiateWithDeposit(t *testing.T) {
	var (
		bob  = bytes.Repeat([]byte{1}, types.SDKAddrLen)
		fred = bytes.Repeat([]byte{2}, types.SDKAddrLen)

		deposit = sdk.NewCoins(sdk.NewInt64Coin("denom", 100))
		initMsg = mustMarshal(t, HackatomExampleInitMsg{Verifier: fred, Beneficiary: bob})
	)

	specs := map[string]struct {
		srcActor sdk.AccAddress
		expError bool
		fundAddr bool
	}{
		"address with funds": {
			srcActor: bob,
			fundAddr: true,
		},
		"address without funds": {
			srcActor: bob,
			expError: true,
		},
		"blocked address": {
			srcActor: authtypes.NewModuleAddress(authtypes.FeeCollectorName),
			fundAddr: true,
			expError: false,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
			accKeeper, bankKeeper, keeper := keepers.AccountKeeper, keepers.BankKeeper, keepers.ContractKeeper

			if spec.fundAddr {
				fundAccounts(t, ctx, accKeeper, bankKeeper, spec.srcActor, sdk.NewCoins(sdk.NewInt64Coin("denom", 200)))
			}
			contractID, _, err := keeper.Create(ctx, spec.srcActor, hackatomWasm, nil)
			require.NoError(t, err)

			// when
			addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, spec.srcActor, nil, initMsg, "my label", deposit)
			// then
			if spec.expError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			balances := bankKeeper.GetAllBalances(ctx, addr)
			assert.Equal(t, deposit, balances)
		})
	}
}

func TestInstantiateWithPermissions(t *testing.T) {
	var (
		deposit   = sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
		myAddr    = bytes.Repeat([]byte{1}, types.SDKAddrLen)
		otherAddr = bytes.Repeat([]byte{2}, types.SDKAddrLen)
		anyAddr   = bytes.Repeat([]byte{3}, types.SDKAddrLen)
	)

	initMsg := HackatomExampleInitMsg{
		Verifier:    anyAddr,
		Beneficiary: anyAddr,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	specs := map[string]struct {
		srcPermission types.AccessConfig
		srcActor      sdk.AccAddress
		expError      *errorsmod.Error
	}{
		"default": {
			srcPermission: types.DefaultUploadAccess,
			srcActor:      anyAddr,
		},
		"everybody": {
			srcPermission: types.AllowEverybody,
			srcActor:      anyAddr,
		},
		"nobody": {
			srcPermission: types.AllowNobody,
			srcActor:      myAddr,
			expError:      sdkerrors.ErrUnauthorized,
		},
		"anyAddress with matching address": {
			srcPermission: types.AccessTypeAnyOfAddresses.With(myAddr),
			srcActor:      myAddr,
		},
		"anyAddress with non matching address": {
			srcActor:      myAddr,
			srcPermission: types.AccessTypeAnyOfAddresses.With(otherAddr),
			expError:      sdkerrors.ErrUnauthorized,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
			accKeeper, bankKeeper, keeper := keepers.AccountKeeper, keepers.BankKeeper, keepers.ContractKeeper
			fundAccounts(t, ctx, accKeeper, bankKeeper, spec.srcActor, deposit)

			contractID, _, err := keeper.Create(ctx, myAddr, hackatomWasm, &spec.srcPermission) //nolint:gosec
			require.NoError(t, err)

			_, _, err = keepers.ContractKeeper.Instantiate(ctx, contractID, spec.srcActor, nil, initMsgBz, "demo contract 1", nil)
			assert.True(t, spec.expError.Is(err), "got %+v", err)
		})
	}
}

func TestInstantiateWithAccounts(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	example := StoreHackatomExampleContract(t, parentCtx, keepers)
	require.Equal(t, uint64(1), example.CodeID)
	initMsg := mustMarshal(t, HackatomExampleInitMsg{Verifier: RandomAccountAddress(t), Beneficiary: RandomAccountAddress(t)})

	senderAddr := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(parentCtx, senderAddr, sdk.NewInt64Coin("denom", 100000000))
	myTestLabel := "testing"
	mySalt := []byte(`my salt`)
	contractAddr := BuildContractAddressPredictable(example.Checksum, senderAddr, mySalt, []byte{})

	lastAccountNumber := keepers.AccountKeeper.GetAccount(parentCtx, senderAddr).GetAccountNumber()

	specs := map[string]struct {
		option      Option
		account     sdk.AccountI
		initBalance sdk.Coin
		deposit     sdk.Coins
		expErr      error
		expAccount  sdk.AccountI
		expBalance  sdk.Coins
	}{
		"unused BaseAccount exists": {
			account:     authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
			initBalance: sdk.NewInt64Coin("denom", 100000000),
			expAccount:  authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+1, 0), // +1 for next seq
			expBalance:  sdk.NewCoins(sdk.NewInt64Coin("denom", 100000000)),
		},
		"BaseAccount with sequence exists": {
			account: authtypes.NewBaseAccount(contractAddr, nil, 0, 1),
			expErr:  types.ErrAccountExists,
		},
		"BaseAccount with pubkey exists": {
			account: authtypes.NewBaseAccount(contractAddr, &ed25519.PubKey{}, 0, 0),
			expErr:  types.ErrAccountExists,
		},
		"no account existed": {
			expAccount: authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+1, 0), // +1 for next seq,
			expBalance: sdk.NewCoins(),
		},
		"no account existed before create with deposit": {
			expAccount: authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+1, 0), // +1 for next seq
			deposit:    sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))),
			expBalance: sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))),
		},
		"prunable DelayedVestingAccount gets overwritten": {
			account: must(vestingtypes.NewDelayedVestingAccount(
				authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
				sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))), time.Now().Add(30*time.Hour).Unix())),
			initBalance: sdk.NewCoin("denom", sdkmath.NewInt(1_000)),
			deposit:     sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1))),
			expAccount:  authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+2, 0), // +1 for next seq, +1 for spec.account created
			expBalance:  sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1))),
		},
		"prunable ContinuousVestingAccount gets overwritten": {
			account: must(vestingtypes.NewContinuousVestingAccount(
				authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
				sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))), time.Now().Add(time.Hour).Unix(), time.Now().Add(2*time.Hour).Unix())),
			initBalance: sdk.NewCoin("denom", sdkmath.NewInt(1_000)),
			deposit:     sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1))),
			expAccount:  authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+2, 0), // +1 for next seq, +1 for spec.account created
			expBalance:  sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1))),
		},
		// "prunable account without balance gets overwritten": { // todo : can not initialize vesting with empty balance
		//	account: must(vestingtypes.NewContinuousVestingAccount(
		//		authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
		//		sdk.NewCoins(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(0))), time.Now().Add(time.Hour).Unix(), time.Now().Add(2*time.Hour).Unix())),
		//	expAccount: authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+2, 0), // +1 for next seq, +1 for spec.account created
		//	expBalance: sdk.NewCoins(),
		// },
		"unknown account type is rejected with error": {
			account: authtypes.NewModuleAccount(
				authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
				"testing",
			),
			initBalance: sdk.NewCoin("denom", sdkmath.NewInt(1_000)),
			expErr:      types.ErrAccountExists,
		},
		"with option used to set non default type to accept list": {
			option: WithAcceptedAccountTypesOnContractInstantiation(&vestingtypes.DelayedVestingAccount{}),
			account: must(vestingtypes.NewDelayedVestingAccount(
				authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
				sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))), time.Now().Add(30*time.Hour).Unix())),
			initBalance: sdk.NewCoin("denom", sdkmath.NewInt(1_000)),
			deposit:     sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1))),
			expAccount: must(vestingtypes.NewDelayedVestingAccount(authtypes.NewBaseAccount(contractAddr, nil, lastAccountNumber+1, 0),
				sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))), time.Now().Add(30*time.Hour).Unix())),
			expBalance: sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_001))),
		},
		"pruning account fails": {
			option: WithAccountPruner(wasmtesting.AccountPrunerMock{CleanupExistingAccountFn: func(ctx sdk.Context, existingAccount sdk.AccountI) (handled bool, err error) {
				return false, types.ErrUnsupportedForContract.Wrap("testing")
			}}),
			account: must(vestingtypes.NewDelayedVestingAccount(
				authtypes.NewBaseAccount(contractAddr, nil, 0, 0),
				sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000))), time.Now().Add(30*time.Hour).Unix())),
			expErr: types.ErrUnsupportedForContract,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			if spec.account != nil {
				keepers.AccountKeeper.SetAccount(ctx, keepers.AccountKeeper.NewAccount(ctx, spec.account))
			}
			if !spec.initBalance.IsNil() {
				keepers.Faucet.Fund(ctx, spec.account.GetAddress(), spec.initBalance)
			}
			if spec.option != nil {
				spec.option.apply(keepers.WasmKeeper)
			}
			defer func() {
				if spec.option != nil { // reset
					WithAcceptedAccountTypesOnContractInstantiation(&authtypes.BaseAccount{}).apply(keepers.WasmKeeper)
					WithAccountPruner(NewVestingCoinBurner(keepers.BankKeeper)).apply(keepers.WasmKeeper)
				}
			}()
			// when
			gotAddr, _, gotErr := keepers.ContractKeeper.Instantiate2(ctx, 1, senderAddr, nil, initMsg, myTestLabel, spec.deposit, mySalt, false)
			if spec.expErr != nil {
				assert.ErrorIs(t, gotErr, spec.expErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, contractAddr, gotAddr)
			// and
			gotAcc := keepers.AccountKeeper.GetAccount(ctx, contractAddr)
			assert.Equal(t, spec.expAccount, gotAcc)
			// and
			gotBalance := keepers.BankKeeper.GetAllBalances(ctx, contractAddr)
			assert.Equal(t, spec.expBalance, gotBalance)
		})
	}
}

func TestInstantiateWithNonExistingCodeID(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	initMsg := HackatomExampleInitMsg{}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	const nonExistingCodeID = 9999
	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, nonExistingCodeID, creator, nil, initMsgBz, "demo contract 2", nil)
	require.Equal(t, types.ErrNoSuchCodeFn(nonExistingCodeID).Wrapf("code id %d", nonExistingCodeID).Error(), err.Error())
	require.Nil(t, addr)
}

func TestContractErrorRedacting(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	keepers.Faucet.Fund(ctx, creator, deposit...)
	example := StoreHackatomExampleContract(t, ctx, keepers)

	initMsg := HackatomExampleInitMsg{
		Verifier:    []byte{1, 2, 3}, // invalid length
		Beneficiary: RandomAccountAddress(t),
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	em := sdk.NewEventManager()

	_, _, err = keepers.ContractKeeper.Instantiate(ctx.WithEventManager(em), example.CodeID, creator, nil, initMsgBz, "demo contract 1", nil)
	require.Error(t, err)
	require.Contains(t, err.Error(), "addr_validate errored: invalid address")

	err = redactError(err)
	// contract error should not be redacted
	require.Contains(t, err.Error(), "addr_validate errored: invalid address")
}

func TestContractErrorGetsForwarded(t *testing.T) {
	// This test makes sure that a contract gets the error message from its submessage execution
	// in a non-redacted form if that error comes from the contract in the submessage.
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	reflect1 := InstantiateReflectExampleContract(t, ctx, keepers)
	// reflect2 will be the contract that errors. It is owned by the other reflect contract.
	// This is necessary because the reflect contract only accepts messages from its owner.
	reflect2, _, err := keepers.ContractKeeper.Instantiate(ctx, reflect1.CodeID, reflect1.Contract, reflect1.Contract, []byte("{}"), "reflect2", sdk.NewCoins())
	require.NoError(t, err)

	const SubMsgID = 1
	// Make the reflect1 contract send a submessage to reflect2. That sub-message will error.
	execMsg := testdata.ReflectHandleMsg{
		ReflectSubMsg: &testdata.ReflectSubPayload{
			Msgs: []wasmvmtypes.SubMsg{
				{
					ID: SubMsgID,
					Msg: wasmvmtypes.CosmosMsg{
						Wasm: &wasmvmtypes.WasmMsg{
							Execute: &wasmvmtypes.ExecuteMsg{
								ContractAddr: reflect2.String(),
								// This message will error in the reflect contract as an empty "msgs" array is not allowed
								Msg: mustMarshal(t, testdata.ReflectHandleMsg{
									Reflect: &testdata.ReflectPayload{
										Msgs: []wasmvmtypes.CosmosMsg{},
									},
								}),
							},
						},
					},
					ReplyOn: wasmvmtypes.ReplyError, // we want to see the error
				},
			},
		},
	}
	execMsgBz := mustMarshal(t, execMsg)

	em := sdk.NewEventManager()
	_, err = keepers.ContractKeeper.Execute(
		ctx.WithEventManager(em),
		reflect1.Contract,
		reflect1.CreatorAddr,
		execMsgBz,
		nil,
	)
	require.NoError(t, err)

	// now query the submessage reply
	queryMsgBz := mustMarshal(t, testdata.ReflectQueryMsg{
		SubMsgResult: &testdata.SubCall{
			ID: SubMsgID,
		},
	})
	queryResponse, err := keepers.WasmKeeper.QuerySmart(ctx, reflect1.Contract, queryMsgBz)
	require.NoError(t, err)
	var submsgReply wasmvmtypes.Reply
	mustUnmarshal(t, queryResponse, &submsgReply)

	assert.Equal(t, "Messages empty. Must reflect at least one message: execute wasm contract failed", submsgReply.Result.Err)
}

func TestInstantiateWithContractDataResponse(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	wasmEngineMock := &wasmtesting.MockWasmEngine{
		InstantiateFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
			return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: []byte("my-response-data")}}, 0, nil
		},
		AnalyzeCodeFn: wasmtesting.WithoutIBCAnalyzeFn,
		StoreCodeFn:   wasmtesting.NoOpStoreCodeFn,
	}

	example := StoreRandomContract(t, ctx, keepers, wasmEngineMock)
	_, data, err := keepers.ContractKeeper.Instantiate(ctx, example.CodeID, example.CreatorAddr, nil, nil, "test", nil)
	require.NoError(t, err)
	assert.Equal(t, []byte("my-response-data"), data)
}

func TestInstantiateWithContractFactoryChildQueriesParent(t *testing.T) {
	// Scenario:
	// 	given a factory contract stored
	// 	when instantiated, the contract creates a new child contract instance
	// 	     and the child contracts queries the senders ContractInfo on instantiation
	//	then the factory contract's ContractInfo should be returned to the child contract
	//
	// see also: https://github.com/CosmWasm/wasmd/issues/896
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.WasmKeeper

	var instantiationCount int
	callbacks := make([]func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error), 2)
	wasmEngineMock := &wasmtesting.MockWasmEngine{
		// dispatch instantiation calls to callbacks
		InstantiateFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
			require.Greater(t, len(callbacks), instantiationCount, "unexpected call to instantiation")
			do := callbacks[instantiationCount]
			instantiationCount++
			return do(codeID, env, info, initMsg, store, goapi, querier, gasMeter, gasLimit, deserCost)
		},
		AnalyzeCodeFn: wasmtesting.WithoutIBCAnalyzeFn,
		StoreCodeFn:   wasmtesting.NoOpStoreCodeFn,
	}

	// overwrite wasmvm in router
	router := baseapp.NewMsgServiceRouter()
	router.SetInterfaceRegistry(keepers.EncodingConfig.InterfaceRegistry)
	types.RegisterMsgServer(router, NewMsgServerImpl(keeper))
	keeper.messenger = NewDefaultMessageHandler(nil, router, nil, nil, nil, nil, keepers.EncodingConfig.Codec, nil)
	// overwrite wasmvm in response handler
	keeper.wasmVMResponseHandler = NewDefaultWasmVMContractResponseHandler(NewMessageDispatcher(keeper.messenger, keeper))

	example := StoreRandomContract(t, ctx, keepers, wasmEngineMock)
	// factory contract
	callbacks[0] = func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		t.Log("called factory")
		return &wasmvmtypes.ContractResult{
			Ok: &wasmvmtypes.Response{
				Data: []byte("parent"),
				Messages: []wasmvmtypes.SubMsg{
					{
						ID: 1, ReplyOn: wasmvmtypes.ReplyNever,
						Msg: wasmvmtypes.CosmosMsg{
							Wasm: &wasmvmtypes.WasmMsg{
								Instantiate: &wasmvmtypes.InstantiateMsg{CodeID: example.CodeID, Msg: []byte(`{}`), Label: "child"},
							},
						},
					},
				},
			},
		}, 0, nil
	}

	// child contract
	var capturedSenderAddr string
	var capturedCodeInfo []byte
	callbacks[1] = func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, initMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		t.Log("called child")
		capturedSenderAddr = info.Sender
		var err error
		capturedCodeInfo, err = querier.Query(wasmvmtypes.QueryRequest{
			Wasm: &wasmvmtypes.WasmQuery{
				ContractInfo: &wasmvmtypes.ContractInfoQuery{ContractAddr: info.Sender},
			},
		}, gasLimit)
		require.NoError(t, err)
		return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{Data: []byte("child")}}, 0, nil
	}

	// when
	parentAddr, data, err := keepers.ContractKeeper.Instantiate(ctx, example.CodeID, example.CreatorAddr, nil, nil, "test", nil)

	// then
	require.NoError(t, err)
	assert.Equal(t, []byte("parent"), data)
	require.Equal(t, parentAddr.String(), capturedSenderAddr)
	expCodeInfo := fmt.Sprintf(`{"code_id":%d,"creator":%q,"pinned":false}`, example.CodeID, example.CreatorAddr.String())
	assert.JSONEq(t, expCodeInfo, string(capturedCodeInfo))
}

func TestExecute(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	accKeeper, keeper, bankKeeper := keepers.AccountKeeper, keepers.ContractKeeper, keepers.BankKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(ctx, topUp...)
	bob := RandomAccountAddress(t)

	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, creator, nil, initMsgBz, "demo contract 3", deposit)
	require.NoError(t, err)
	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", addr.String())

	// ensure bob doesn't exist
	bobAcct := accKeeper.GetAccount(ctx, bob)
	require.Nil(t, bobAcct)

	// ensure funder has reduced balance
	creatorAcct := accKeeper.GetAccount(ctx, creator)
	require.NotNil(t, creatorAcct)
	// we started at 2*deposit, should have spent one above
	assert.Equal(t, deposit, bankKeeper.GetAllBalances(ctx, creatorAcct.GetAddress()))

	// ensure contract has updated balance
	contractAcct := accKeeper.GetAccount(ctx, addr)
	require.NotNil(t, contractAcct)
	assert.Equal(t, deposit, bankKeeper.GetAllBalances(ctx, contractAcct.GetAddress()))

	// unauthorized - trialCtx so we don't change state
	trialCtx := ctx.WithMultiStore(ctx.MultiStore().CacheWrap().(storetypes.MultiStore))
	_, err = keepers.ContractKeeper.Execute(trialCtx, addr, creator, []byte(`{"release":{}}`), nil)
	require.Error(t, err)
	require.True(t, errors.Is(err, types.ErrExecuteFailed))
	require.Equal(t, "Unauthorized: execute wasm contract failed", err.Error())

	// verifier can execute, and get proper gas amount
	start := time.Now()
	gasBefore := ctx.GasMeter().GasConsumed()
	em := sdk.NewEventManager()
	// when
	res, err := keepers.ContractKeeper.Execute(ctx.WithEventManager(em), addr, fred, []byte(`{"release":{}}`), topUp)
	diff := time.Since(start)
	require.NoError(t, err)
	require.NotNil(t, res)

	// make sure gas is properly deducted from ctx
	gasAfter := ctx.GasMeter().GasConsumed()
	if types.EnableGasVerification {
		require.Equal(t, uint64(0x1acb4), gasAfter-gasBefore)
	}
	// ensure bob now exists and got both payments released
	bobAcct = accKeeper.GetAccount(ctx, bob)
	require.NotNil(t, bobAcct)
	balance := bankKeeper.GetAllBalances(ctx, bobAcct.GetAddress())
	assert.Equal(t, deposit.Add(topUp...), balance)

	// ensure contract has updated balance
	contractAcct = accKeeper.GetAccount(ctx, addr)
	require.NotNil(t, contractAcct)
	assert.Equal(t, sdk.Coins{}, bankKeeper.GetAllBalances(ctx, contractAcct.GetAddress()))

	// and events emitted
	require.Len(t, em.Events(), 9)
	expEvt := sdk.NewEvent("execute",
		sdk.NewAttribute("_contract_address", addr.String()))
	assert.Equal(t, expEvt, em.Events()[3], prettyEvents(t, em.Events()))

	t.Logf("Duration: %v (%d gas)\n", diff, gasAfter-gasBefore)
}

func TestExecuteWithDeposit(t *testing.T) {
	var (
		bob         = bytes.Repeat([]byte{1}, types.SDKAddrLen)
		fred        = bytes.Repeat([]byte{2}, types.SDKAddrLen)
		blockedAddr = authtypes.NewModuleAddress(distributiontypes.ModuleName)
		deposit     = sdk.NewCoins(sdk.NewInt64Coin("denom", 100))
	)

	specs := map[string]struct {
		srcActor      sdk.AccAddress
		beneficiary   sdk.AccAddress
		newBankParams *banktypes.Params
		expError      bool
		fundAddr      bool
	}{
		"actor with funds": {
			srcActor:    bob,
			fundAddr:    true,
			beneficiary: fred,
		},
		"actor without funds": {
			srcActor:    bob,
			beneficiary: fred,
			expError:    true,
		},
		"blocked address as actor": {
			srcActor:    blockedAddr,
			fundAddr:    true,
			beneficiary: fred,
			expError:    false,
		},
		"coin transfer with all transfers disabled": {
			srcActor:      bob,
			fundAddr:      true,
			beneficiary:   fred,
			newBankParams: &banktypes.Params{DefaultSendEnabled: false},
			expError:      true,
		},
		"coin transfer with transfer denom disabled": {
			srcActor:    bob,
			fundAddr:    true,
			beneficiary: fred,
			newBankParams: &banktypes.Params{
				DefaultSendEnabled: true,
				SendEnabled:        []*banktypes.SendEnabled{{Denom: "denom", Enabled: false}},
			},
			expError: true,
		},
		"blocked address as beneficiary": {
			srcActor:    bob,
			fundAddr:    true,
			beneficiary: blockedAddr,
			expError:    true,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
			accKeeper, bankKeeper, keeper := keepers.AccountKeeper, keepers.BankKeeper, keepers.ContractKeeper
			if spec.newBankParams != nil {
				err := bankKeeper.SetParams(ctx, *spec.newBankParams)
				require.NoError(t, err)
			}
			if spec.fundAddr {
				fundAccounts(t, ctx, accKeeper, bankKeeper, spec.srcActor, sdk.NewCoins(sdk.NewInt64Coin("denom", 200)))
			}
			codeID, _, err := keeper.Create(ctx, spec.srcActor, hackatomWasm, nil)
			require.NoError(t, err)

			initMsg := HackatomExampleInitMsg{Verifier: spec.srcActor, Beneficiary: spec.beneficiary}
			initMsgBz, err := json.Marshal(initMsg)
			require.NoError(t, err)

			contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, spec.srcActor, nil, initMsgBz, "my label", nil)
			require.NoError(t, err)

			// when
			_, err = keepers.ContractKeeper.Execute(ctx, contractAddr, spec.srcActor, []byte(`{"release":{}}`), deposit)

			// then
			if spec.expError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			balances := bankKeeper.GetAllBalances(ctx, spec.beneficiary)
			assert.Equal(t, deposit, balances)
		})
	}
}

func TestExecuteWithNonExistingAddress(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)

	// unauthorized - trialCtx so we don't change state
	nonExistingAddress := RandomAccountAddress(t)
	_, err := keeper.Execute(ctx, nonExistingAddress, creator, []byte(`{}`), nil)
	require.Equal(t, types.ErrNoSuchContractFn(nonExistingAddress.String()).Wrapf("address %s", nonExistingAddress.String()).Error(), err.Error())
}

func TestExecuteWithPanic(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(ctx, topUp...)

	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, bob := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, creator, nil, initMsgBz, "demo contract 4", deposit)
	require.NoError(t, err)

	// let's make sure we get a reasonable error, no panic/crash
	_, err = keepers.ContractKeeper.Execute(ctx, addr, fred, []byte(`{"panic":{}}`), topUp)
	require.Error(t, err)
	require.True(t, errors.Is(err, types.ErrVMError))
	// test with contains as "Display" implementation of the Wasmer "RuntimeError" is different for Mac and Linux
	assert.Contains(t, err.Error(), "Error calling the VM: Error executing Wasm: Wasmer runtime error: RuntimeError: Aborted: panicked at 'This page intentionally faulted', src/contract.rs:169:5: wasmvm error")
}

func TestExecuteWithCpuLoop(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(ctx, topUp...)

	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, bob := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, creator, nil, initMsgBz, "demo contract 5", deposit)
	require.NoError(t, err)

	// make sure we set a limit before calling
	var gasLimit uint64 = 400_000
	ctx = ctx.WithGasMeter(storetypes.NewGasMeter(gasLimit))
	require.Equal(t, uint64(0), ctx.GasMeter().GasConsumed())

	// ensure we get an out of gas panic
	defer func() {
		r := recover()
		require.NotNil(t, r)
		_, ok := r.(storetypes.ErrorOutOfGas)
		require.True(t, ok, "%v", r)
	}()

	// this should throw out of gas exception (panic)
	_, _ = keepers.ContractKeeper.Execute(ctx, addr, fred, []byte(`{"cpu_loop":{}}`), nil)
	require.True(t, false, "We must panic before this line")
}

func TestExecuteWithStorageLoop(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)

	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(ctx, topUp...)

	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, bob := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, creator, nil, initMsgBz, "demo contract 6", deposit)
	require.NoError(t, err)

	// make sure we set a limit before calling
	var gasLimit uint64 = 400_002
	ctx = ctx.WithGasMeter(storetypes.NewGasMeter(gasLimit))
	require.Equal(t, uint64(0), ctx.GasMeter().GasConsumed())

	// ensure we get an out of gas panic
	defer func() {
		r := recover()
		require.NotNil(t, r)
		_, ok := r.(storetypes.ErrorOutOfGas)
		require.True(t, ok, "%v", r)
	}()

	// this should throw out of gas exception (panic)
	_, _ = keepers.ContractKeeper.Execute(ctx, addr, fred, []byte(`{"storage_loop":{}}`), nil)
	require.True(t, false, "We must panic before this line")
}

func TestMigrate(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(parentCtx, creator, deposit.Add(deposit...)...)
	fred := DeterministicAccountAddress(t, 2)
	keepers.Faucet.Fund(parentCtx, fred, topUp...)

	originalCodeID := StoreHackatomExampleContract(t, parentCtx, keepers).CodeID
	newCodeID := StoreHackatomExampleContract(t, parentCtx, keepers).CodeID
	ibcCodeID := StoreIBCReflectContract(t, parentCtx, keepers).CodeID
	require.NotEqual(t, originalCodeID, newCodeID)

	restrictedCodeExample := StoreHackatomExampleContract(t, parentCtx, keepers)
	require.NoError(t, keeper.SetAccessConfig(parentCtx, restrictedCodeExample.CodeID, restrictedCodeExample.CreatorAddr, types.AllowNobody))
	require.NotEqual(t, originalCodeID, restrictedCodeExample.CodeID)

	// store hackatom contracts with "migrate_version" attributes
	hackatom42 := StoreExampleContract(t, parentCtx, keepers, "./testdata/hackatom_42.wasm")
	hackatom420 := StoreExampleContract(t, parentCtx, keepers, "./testdata/hackatom_420.wasm")

	anyAddr := RandomAccountAddress(t)
	newVerifierAddr := RandomAccountAddress(t)
	initMsgBz := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: anyAddr,
	}.GetBytes(t)

	migMsg := struct {
		Verifier sdk.AccAddress `json:"verifier"`
	}{Verifier: newVerifierAddr}
	migMsgBz, err := json.Marshal(migMsg)
	require.NoError(t, err)

	specs := map[string]struct {
		admin                sdk.AccAddress
		overrideContractAddr sdk.AccAddress
		caller               sdk.AccAddress
		fromCodeID           uint64
		toCodeID             uint64
		migrateMsg           []byte
		expErr               *errorsmod.Error
		expVerifier          sdk.AccAddress
		expIBCPort           bool
		initMsg              []byte
	}{
		"all good with same code id": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  originalCodeID,
			toCodeID:    originalCodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"all good with different code id": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  originalCodeID,
			toCodeID:    newCodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"all good with admin set": {
			admin:       fred,
			caller:      fred,
			initMsg:     initMsgBz,
			fromCodeID:  originalCodeID,
			toCodeID:    newCodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"adds IBC port for IBC enabled contracts": {
			admin:       fred,
			caller:      fred,
			initMsg:     initMsgBz,
			fromCodeID:  originalCodeID,
			toCodeID:    ibcCodeID,
			migrateMsg:  []byte(`{}`),
			expIBCPort:  true,
			expVerifier: fred, // not updated
		},
		"prevent migration when admin was not set on instantiate": {
			caller:     creator,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   originalCodeID,
			expErr:     sdkerrors.ErrUnauthorized,
		},
		"prevent migration when not sent by admin": {
			caller:     creator,
			admin:      fred,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   originalCodeID,
			expErr:     sdkerrors.ErrUnauthorized,
		},
		"prevent migration when new code is restricted": {
			admin:      creator,
			caller:     creator,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   restrictedCodeExample.CodeID,
			migrateMsg: migMsgBz,
			expErr:     sdkerrors.ErrUnauthorized,
		},
		"fail with non existing code id": {
			admin:      creator,
			caller:     creator,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   99999,
			expErr:     sdkerrors.ErrInvalidRequest,
		},
		"fail with non existing contract addr": {
			admin:                creator,
			caller:               creator,
			initMsg:              initMsgBz,
			overrideContractAddr: anyAddr,
			fromCodeID:           originalCodeID,
			toCodeID:             originalCodeID,
			expErr:               sdkerrors.ErrInvalidRequest,
		},
		"fail in contract with invalid migrate msg": {
			admin:      creator,
			caller:     creator,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   originalCodeID,
			migrateMsg: bytes.Repeat([]byte{0x1}, 7),
			expErr:     types.ErrMigrationFailed,
		},
		"fail in contract without migrate msg": {
			admin:      creator,
			caller:     creator,
			initMsg:    initMsgBz,
			fromCodeID: originalCodeID,
			toCodeID:   originalCodeID,
			expErr:     types.ErrVMError,
		},
		"fail when no IBC callbacks": {
			admin:      fred,
			caller:     fred,
			initMsg:    IBCReflectInitMsg{ReflectCodeID: StoreReflectContract(t, parentCtx, keepers).CodeID}.GetBytes(t),
			fromCodeID: ibcCodeID,
			toCodeID:   newCodeID,
			migrateMsg: migMsgBz,
			expErr:     types.ErrMigrationFailed,
		},
		"all good with migrate versions": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  hackatom42.CodeID,
			toCodeID:    hackatom420.CodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"all good with no migrate version to migrate version contract": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  originalCodeID,
			toCodeID:    hackatom42.CodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"all good with same migrate version": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  hackatom42.CodeID,
			toCodeID:    hackatom42.CodeID,
			migrateMsg:  migMsgBz,
			expVerifier: fred, // not updated
		},
		"all good with migrate version contract to no migrate version contract": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  hackatom42.CodeID,
			toCodeID:    originalCodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
		"all good with migration to older migrate version": {
			admin:       creator,
			caller:      creator,
			initMsg:     initMsgBz,
			fromCodeID:  hackatom420.CodeID,
			toCodeID:    hackatom42.CodeID,
			migrateMsg:  migMsgBz,
			expVerifier: newVerifierAddr,
		},
	}

	blockHeight := parentCtx.BlockHeight()
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			// given a contract instance
			ctx, _ := parentCtx.WithBlockHeight(blockHeight + 1).CacheContext()
			blockHeight++

			contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, spec.fromCodeID, creator, spec.admin, spec.initMsg, "demo contract", nil)
			require.NoError(t, err)
			if spec.overrideContractAddr != nil {
				contractAddr = spec.overrideContractAddr
			}
			// when
			_, err = keeper.Migrate(ctx, contractAddr, spec.caller, spec.toCodeID, spec.migrateMsg)

			// then
			require.True(t, spec.expErr.Is(err), "expected %v but got %+v", spec.expErr, err)
			if spec.expErr != nil {
				return
			}
			cInfo := keepers.WasmKeeper.GetContractInfo(ctx, contractAddr)
			assert.Equal(t, spec.toCodeID, cInfo.CodeID)
			assert.Equal(t, spec.expIBCPort, cInfo.IBCPortID != "", cInfo.IBCPortID)

			expHistory := []types.ContractCodeHistoryEntry{{
				Operation: types.ContractCodeHistoryOperationTypeInit,
				CodeID:    spec.fromCodeID,
				Updated:   types.NewAbsoluteTxPosition(ctx),
				Msg:       initMsgBz,
			}, {
				Operation: types.ContractCodeHistoryOperationTypeMigrate,
				CodeID:    spec.toCodeID,
				Updated:   types.NewAbsoluteTxPosition(ctx),
				Msg:       spec.migrateMsg,
			}}
			assert.Equal(t, expHistory, keepers.WasmKeeper.GetContractHistory(ctx, contractAddr))

			// and verify contract state
			raw := keepers.WasmKeeper.QueryRaw(ctx, contractAddr, []byte("config"))
			var stored map[string]string
			require.NoError(t, json.Unmarshal(raw, &stored))
			require.Contains(t, stored, "verifier")
			require.NoError(t, err)
			assert.Equal(t, spec.expVerifier.String(), stored["verifier"])
		})
	}
}

func TestMigrateReplacesTheSecondIndex(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	example := InstantiateHackatomExampleContract(t, ctx, keepers)

	// then assert a second index exists
	store := keepers.WasmKeeper.storeService.OpenKVStore(ctx)
	oldContractInfo := keepers.WasmKeeper.GetContractInfo(ctx, example.Contract)
	require.NotNil(t, oldContractInfo)
	createHistoryEntry := types.ContractCodeHistoryEntry{
		CodeID:  example.CodeID,
		Updated: types.NewAbsoluteTxPosition(ctx),
	}
	exists, err := store.Has(types.GetContractByCreatedSecondaryIndexKey(example.Contract, createHistoryEntry))
	require.NoError(t, err)
	require.True(t, exists)

	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1) // increment for different block
	// when do migrate
	newCodeExample := StoreBurnerExampleContract(t, ctx, keepers)
	migMsgBz := BurnerExampleInitMsg{Payout: example.CreatorAddr}.GetBytes(t)
	_, err = keepers.ContractKeeper.Migrate(ctx, example.Contract, example.CreatorAddr, newCodeExample.CodeID, migMsgBz)
	require.NoError(t, err)

	// then the new index exists
	migrateHistoryEntry := types.ContractCodeHistoryEntry{
		CodeID:  newCodeExample.CodeID,
		Updated: types.NewAbsoluteTxPosition(ctx),
	}
	exists, err = store.Has(types.GetContractByCreatedSecondaryIndexKey(example.Contract, migrateHistoryEntry))
	require.NoError(t, err)
	require.True(t, exists)
	// and the old index was removed
	exists, err = store.Has(types.GetContractByCreatedSecondaryIndexKey(example.Contract, createHistoryEntry))
	require.NoError(t, err)
	require.False(t, exists)
}

func TestMigrateWithDispatchedMessage(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(ctx, sdk.NewInt64Coin("denom", 5000))

	burnerCode, err := os.ReadFile("./testdata/burner.wasm")
	require.NoError(t, err)

	originalContractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)
	burnerContractID, _, err := keeper.Create(ctx, creator, burnerCode, nil)
	require.NoError(t, err)
	require.NotEqual(t, originalContractID, burnerContractID)

	_, myPayoutAddr := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: fred,
	}
	initMsgBz := initMsg.GetBytes(t)

	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, originalContractID, creator, fred, initMsgBz, "demo contract", deposit)
	require.NoError(t, err)

	migMsgBz := BurnerExampleInitMsg{Payout: myPayoutAddr, Delete: 100}.GetBytes(t)
	ctx = ctx.WithEventManager(sdk.NewEventManager()).WithBlockHeight(ctx.BlockHeight() + 1)
	_, err = keeper.Migrate(ctx, contractAddr, fred, burnerContractID, migMsgBz)
	require.NoError(t, err)
	type dict map[string]interface{}
	expEvents := []dict{
		{
			"Type": "migrate",
			"Attr": []dict{
				{"code_id": "2"},
				{"_contract_address": contractAddr},
			},
		},
		{
			"Type": "wasm",
			"Attr": []dict{
				{"_contract_address": contractAddr},
				{"action": "migrate"},
				{"payout": myPayoutAddr},
				{"deleted_entries": "1"},
			},
		},
		{
			"Type": "coin_spent",
			"Attr": []dict{
				{"spender": contractAddr},
				{"amount": "100000denom"},
			},
		},
		{
			"Type": "coin_received",
			"Attr": []dict{
				{"receiver": myPayoutAddr},
				{"amount": "100000denom"},
			},
		},
		{
			"Type": "transfer",
			"Attr": []dict{
				{"recipient": myPayoutAddr},
				{"sender": contractAddr},
				{"amount": "100000denom"},
			},
		},
	}
	expJSONEvts := string(mustMarshal(t, expEvents))
	assert.JSONEq(t, expJSONEvts, prettyEvents(t, ctx.EventManager().Events()), prettyEvents(t, ctx.EventManager().Events()))

	// all persistent data cleared
	m := keepers.WasmKeeper.QueryRaw(ctx, contractAddr, []byte("config"))
	require.Len(t, m, 0)

	// and all deposit tokens sent to myPayoutAddr
	balance := keepers.BankKeeper.GetAllBalances(ctx, myPayoutAddr)
	assert.Equal(t, deposit, balance)
}

func TestIterateContractsByCode(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k, c := keepers.WasmKeeper, keepers.ContractKeeper
	example1 := InstantiateHackatomExampleContract(t, ctx, keepers)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	example2 := InstantiateIBCReflectContract(t, ctx, keepers)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	initMsg := HackatomExampleInitMsg{
		Verifier:    RandomAccountAddress(t),
		Beneficiary: RandomAccountAddress(t),
	}.GetBytes(t)
	contractAddr3, _, err := c.Instantiate(ctx, example1.CodeID, example1.CreatorAddr, nil, initMsg, "foo", nil)
	require.NoError(t, err)
	specs := map[string]struct {
		codeID uint64
		exp    []sdk.AccAddress
	}{
		"multiple results": {
			codeID: example1.CodeID,
			exp:    []sdk.AccAddress{example1.Contract, contractAddr3},
		},
		"single results": {
			codeID: example2.CodeID,
			exp:    []sdk.AccAddress{example2.Contract},
		},
		"empty results": {
			codeID: 99999,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var gotAddr []sdk.AccAddress
			k.IterateContractsByCode(ctx, spec.codeID, func(address sdk.AccAddress) bool {
				gotAddr = append(gotAddr, address)
				return false
			})
			assert.Equal(t, spec.exp, gotAddr)
		})
	}
}

func TestIterateContractsByCodeWithMigration(t *testing.T) {
	// mock migration so that it does not fail when migrate example1 to example2.codeID
	mockWasmVM := wasmtesting.MockWasmEngine{MigrateFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, migrateMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{}}, 1, nil
	}}
	wasmtesting.MakeInstantiable(&mockWasmVM)
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities, WithWasmEngine(&mockWasmVM))
	k, c := keepers.WasmKeeper, keepers.ContractKeeper
	example1 := InstantiateHackatomExampleContract(t, ctx, keepers)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	example2 := InstantiateIBCReflectContract(t, ctx, keepers)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	_, err := c.Migrate(ctx, example1.Contract, example1.CreatorAddr, example2.CodeID, []byte("{}"))
	require.NoError(t, err)

	// when
	var gotAddr []sdk.AccAddress
	k.IterateContractsByCode(ctx, example2.CodeID, func(address sdk.AccAddress) bool {
		gotAddr = append(gotAddr, address)
		return false
	})

	// then
	exp := []sdk.AccAddress{example2.Contract, example1.Contract}
	assert.Equal(t, exp, gotAddr)
}

type sudoMsg struct {
	// This is a tongue-in-check demo command. This is not the intended purpose of Sudo.
	// Here we show that some priviledged Go module can make a call that should never be exposed
	// to end users (via Tx/Execute).
	//
	// The contract developer can choose to expose anything to sudo. This functionality is not a true
	// backdoor (it can never be called by end users), but allows the developers of the native blockchain
	// code to make special calls. This can also be used as an authentication mechanism, if you want to expose
	// some callback that only can be triggered by some system module and not faked by external users.
	StealFunds stealFundsMsg `json:"steal_funds"`
}

type stealFundsMsg struct {
	Recipient string                              `json:"recipient"`
	Amount    wasmvmtypes.Array[wasmvmtypes.Coin] `json:"amount"`
}

func TestSudo(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	accKeeper, keeper, bankKeeper := keepers.AccountKeeper, keepers.ContractKeeper, keepers.BankKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(ctx, creator, deposit.Add(deposit...)...)

	contractID, _, err := keeper.Create(ctx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, bob := keyPubAddr()
	_, fred := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)
	addr, _, err := keepers.ContractKeeper.Instantiate(ctx, contractID, creator, nil, initMsgBz, "demo contract 3", deposit)
	require.NoError(t, err)
	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", addr.String())

	// the community is broke
	_, community := keyPubAddr()
	comAcct := accKeeper.GetAccount(ctx, community)
	require.Nil(t, comAcct)

	// now the community wants to get paid via sudo
	msg := sudoMsg{
		// This is a tongue-in-check demo command. This is not the intended purpose of Sudo.
		// Here we show that some priviledged Go module can make a call that should never be exposed
		// to end users (via Tx/Execute).
		StealFunds: stealFundsMsg{
			Recipient: community.String(),
			Amount:    wasmvmtypes.Array[wasmvmtypes.Coin]{wasmvmtypes.NewCoin(76543, "denom")},
		},
	}
	sudoMsg, err := json.Marshal(msg)
	require.NoError(t, err)

	em := sdk.NewEventManager()

	// when
	_, err = keepers.WasmKeeper.Sudo(ctx.WithEventManager(em), addr, sudoMsg)
	require.NoError(t, err)

	// ensure community now exists and got paid
	comAcct = accKeeper.GetAccount(ctx, community)
	require.NotNil(t, comAcct)
	balance := bankKeeper.GetBalance(ctx, comAcct.GetAddress(), "denom")
	assert.Equal(t, sdk.NewInt64Coin("denom", 76543), balance)
	// and events emitted
	require.Len(t, em.Events(), 4, prettyEvents(t, em.Events()))
	expEvt := sdk.NewEvent("sudo",
		sdk.NewAttribute("_contract_address", addr.String()))
	assert.Equal(t, expEvt, em.Events()[0])
}

func prettyEvents(t *testing.T, events sdk.Events) string {
	t.Helper()
	type prettyEvent struct {
		Type string
		Attr []map[string]string
	}

	r := make([]prettyEvent, len(events))
	for i, e := range events {
		attr := make([]map[string]string, len(e.Attributes))
		for j, a := range e.Attributes {
			attr[j] = map[string]string{a.Key: a.Value}
		}
		r[i] = prettyEvent{Type: e.Type, Attr: attr}
	}
	return string(mustMarshal(t, r))
}

func mustMarshal(t *testing.T, r interface{}) []byte {
	t.Helper()
	bz, err := json.Marshal(r)
	require.NoError(t, err)
	return bz
}

func TestUpdateContractAdmin(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(parentCtx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(parentCtx, topUp...)

	originalContractID, _, err := keeper.Create(parentCtx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, anyAddr := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: anyAddr,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)
	specs := map[string]struct {
		instAdmin            sdk.AccAddress
		newAdmin             sdk.AccAddress
		overrideContractAddr sdk.AccAddress
		caller               sdk.AccAddress
		expErr               *errorsmod.Error
	}{
		"all good with admin set": {
			instAdmin: fred,
			newAdmin:  anyAddr,
			caller:    fred,
		},
		"prevent update when admin was not set on instantiate": {
			caller:   creator,
			newAdmin: fred,
			expErr:   sdkerrors.ErrUnauthorized,
		},
		"prevent updates from non admin address": {
			instAdmin: creator,
			newAdmin:  fred,
			caller:    fred,
			expErr:    sdkerrors.ErrUnauthorized,
		},
		"fail with non existing contract addr": {
			instAdmin:            creator,
			newAdmin:             anyAddr,
			caller:               creator,
			overrideContractAddr: anyAddr,
			expErr:               sdkerrors.ErrInvalidRequest,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			addr, _, err := keepers.ContractKeeper.Instantiate(ctx, originalContractID, creator, spec.instAdmin, initMsgBz, "demo contract", nil)
			require.NoError(t, err)
			if spec.overrideContractAddr != nil {
				addr = spec.overrideContractAddr
			}
			err = keeper.UpdateContractAdmin(ctx, addr, spec.caller, spec.newAdmin)
			require.True(t, spec.expErr.Is(err), "expected %v but got %+v", spec.expErr, err)
			if spec.expErr != nil {
				return
			}
			cInfo := keepers.WasmKeeper.GetContractInfo(ctx, addr)
			assert.Equal(t, spec.newAdmin.String(), cInfo.Admin)
		})
	}
}

func TestClearContractAdmin(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(parentCtx, creator, deposit.Add(deposit...)...)
	fred := keepers.Faucet.NewFundedRandomAccount(parentCtx, topUp...)

	originalContractID, _, err := keeper.Create(parentCtx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	_, anyAddr := keyPubAddr()
	initMsg := HackatomExampleInitMsg{
		Verifier:    fred,
		Beneficiary: anyAddr,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)
	specs := map[string]struct {
		instAdmin            sdk.AccAddress
		overrideContractAddr sdk.AccAddress
		caller               sdk.AccAddress
		expErr               *errorsmod.Error
	}{
		"all good when called by proper admin": {
			instAdmin: fred,
			caller:    fred,
		},
		"prevent update when admin was not set on instantiate": {
			caller: creator,
			expErr: sdkerrors.ErrUnauthorized,
		},
		"prevent updates from non admin address": {
			instAdmin: creator,
			caller:    fred,
			expErr:    sdkerrors.ErrUnauthorized,
		},
		"fail with non existing contract addr": {
			instAdmin:            creator,
			caller:               creator,
			overrideContractAddr: anyAddr,
			expErr:               sdkerrors.ErrInvalidRequest,
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			addr, _, err := keepers.ContractKeeper.Instantiate(ctx, originalContractID, creator, spec.instAdmin, initMsgBz, "demo contract", nil)
			require.NoError(t, err)
			if spec.overrideContractAddr != nil {
				addr = spec.overrideContractAddr
			}
			err = keeper.ClearContractAdmin(ctx, addr, spec.caller)
			require.True(t, spec.expErr.Is(err), "expected %v but got %+v", spec.expErr, err)
			if spec.expErr != nil {
				return
			}
			cInfo := keepers.WasmKeeper.GetContractInfo(ctx, addr)
			assert.Empty(t, cInfo.Admin)
		})
	}
}

func TestPinCode(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper

	var capturedChecksums []wasmvm.Checksum
	mock := wasmtesting.MockWasmEngine{PinFn: func(checksum wasmvm.Checksum) error {
		capturedChecksums = append(capturedChecksums, checksum)
		return nil
	}}
	wasmtesting.MakeInstantiable(&mock)
	myCodeID := StoreRandomContract(t, ctx, keepers, &mock).CodeID
	require.Equal(t, uint64(1), myCodeID)
	em := sdk.NewEventManager()

	// when
	gotErr := k.pinCode(ctx.WithEventManager(em), myCodeID)

	// then
	require.NoError(t, gotErr)
	assert.NotEmpty(t, capturedChecksums)
	assert.True(t, k.IsPinnedCode(ctx, myCodeID))

	// and events
	exp := sdk.Events{sdk.NewEvent("pin_code", sdk.NewAttribute("code_id", "1"))}
	assert.Equal(t, exp, em.Events())
}

func TestUnpinCode(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper

	var capturedChecksums []wasmvm.Checksum
	mock := wasmtesting.MockWasmEngine{
		PinFn: func(checksum wasmvm.Checksum) error {
			return nil
		},
		UnpinFn: func(checksum wasmvm.Checksum) error {
			capturedChecksums = append(capturedChecksums, checksum)
			return nil
		},
	}
	wasmtesting.MakeInstantiable(&mock)
	myCodeID := StoreRandomContract(t, ctx, keepers, &mock).CodeID
	require.Equal(t, uint64(1), myCodeID)
	err := k.pinCode(ctx, myCodeID)
	require.NoError(t, err)
	em := sdk.NewEventManager()

	// when
	gotErr := k.unpinCode(ctx.WithEventManager(em), myCodeID)

	// then
	require.NoError(t, gotErr)
	assert.NotEmpty(t, capturedChecksums)
	assert.False(t, k.IsPinnedCode(ctx, myCodeID))

	// and events
	exp := sdk.Events{sdk.NewEvent("unpin_code", sdk.NewAttribute("code_id", "1"))}
	assert.Equal(t, exp, em.Events())
}

func TestInitializePinnedCodes(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper

	var capturedChecksums []wasmvm.Checksum
	mock := wasmtesting.MockWasmEngine{PinFn: func(checksum wasmvm.Checksum) error {
		capturedChecksums = append(capturedChecksums, checksum)
		return nil
	}}
	wasmtesting.MakeInstantiable(&mock)

	const testItems = 3
	myCodeIDs := make([]uint64, testItems)
	for i := 0; i < testItems; i++ {
		myCodeIDs[i] = StoreRandomContract(t, ctx, keepers, &mock).CodeID
		require.NoError(t, k.pinCode(ctx, myCodeIDs[i]))
	}
	capturedChecksums = nil

	// when
	gotErr := k.InitializePinnedCodes(ctx)

	// then
	require.NoError(t, gotErr)
	require.Len(t, capturedChecksums, testItems)
	for i, c := range myCodeIDs {
		var exp wasmvm.Checksum = k.GetCodeInfo(ctx, c).CodeHash
		assert.Equal(t, exp, capturedChecksums[i])
	}
}

func TestPinnedContractLoops(t *testing.T) {
	var capturedChecksums []wasmvm.Checksum
	mock := wasmtesting.MockWasmEngine{PinFn: func(checksum wasmvm.Checksum) error {
		capturedChecksums = append(capturedChecksums, checksum)
		return nil
	}}
	wasmtesting.MakeInstantiable(&mock)

	// a pinned contract that calls itself via submessages should terminate with an
	// error at some point
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities, WithWasmEngine(&mock))
	k := keepers.WasmKeeper

	example := SeedNewContractInstance(t, ctx, keepers, &mock)
	require.NoError(t, k.pinCode(ctx, example.CodeID))
	var loops int
	anyMsg := []byte(`{}`)
	mock.ExecuteFn = func(codeID wasmvm.Checksum, env wasmvmtypes.Env, info wasmvmtypes.MessageInfo, executeMsg []byte, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		loops++
		return &wasmvmtypes.ContractResult{
			Ok: &wasmvmtypes.Response{
				Messages: []wasmvmtypes.SubMsg{
					{
						ID:      1,
						ReplyOn: wasmvmtypes.ReplyNever,
						Msg: wasmvmtypes.CosmosMsg{
							Wasm: &wasmvmtypes.WasmMsg{
								Execute: &wasmvmtypes.ExecuteMsg{
									ContractAddr: example.Contract.String(),
									Msg:          anyMsg,
								},
							},
						},
					},
				},
			},
		}, 0, nil
	}
	ctx = ctx.WithGasMeter(storetypes.NewGasMeter(30_000))
	require.PanicsWithValue(t, storetypes.ErrorOutOfGas{Descriptor: "ReadFlat"}, func() {
		_, err := k.execute(ctx, example.Contract, RandomAccountAddress(t), anyMsg, nil)
		require.NoError(t, err)
	})
	assert.True(t, ctx.GasMeter().IsOutOfGas())
	assert.Greater(t, loops, 2)
}

func TestNewDefaultWasmVMContractResponseHandler(t *testing.T) {
	specs := map[string]struct {
		srcData []byte
		setup   func(m *wasmtesting.MockMsgDispatcher)
		expErr  bool
		expData []byte
		expEvts sdk.Events
	}{
		"submessage overwrites result when set": {
			srcData: []byte("otherData"),
			setup: func(m *wasmtesting.MockMsgDispatcher) {
				m.DispatchSubmessagesFn = func(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error) {
					return []byte("mySubMsgData"), nil
				}
			},
			expErr:  false,
			expData: []byte("mySubMsgData"),
			expEvts: sdk.Events{},
		},
		"submessage overwrites result when empty": {
			srcData: []byte("otherData"),
			setup: func(m *wasmtesting.MockMsgDispatcher) {
				m.DispatchSubmessagesFn = func(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error) {
					return []byte(""), nil
				}
			},
			expErr:  false,
			expData: []byte(""),
			expEvts: sdk.Events{},
		},
		"submessage do not overwrite result when nil": {
			srcData: []byte("otherData"),
			setup: func(m *wasmtesting.MockMsgDispatcher) {
				m.DispatchSubmessagesFn = func(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error) {
					return nil, nil
				}
			},
			expErr:  false,
			expData: []byte("otherData"),
			expEvts: sdk.Events{},
		},
		"submessage error aborts process": {
			setup: func(m *wasmtesting.MockMsgDispatcher) {
				m.DispatchSubmessagesFn = func(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error) {
					return nil, errors.New("test - ignore")
				}
			},
			expErr: true,
		},
		"message emit non message events": {
			setup: func(m *wasmtesting.MockMsgDispatcher) {
				m.DispatchSubmessagesFn = func(ctx sdk.Context, contractAddr sdk.AccAddress, ibcPort string, msgs []wasmvmtypes.SubMsg) ([]byte, error) {
					ctx.EventManager().EmitEvent(sdk.NewEvent("myEvent"))
					return nil, nil
				}
			},
			expEvts: sdk.Events{sdk.NewEvent("myEvent")},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var msgs []wasmvmtypes.SubMsg
			var mock wasmtesting.MockMsgDispatcher
			spec.setup(&mock)
			d := NewDefaultWasmVMContractResponseHandler(&mock)
			em := sdk.NewEventManager()

			// when
			gotData, gotErr := d.Handle(sdk.Context{}.WithEventManager(em), RandomAccountAddress(t), "ibc-port", msgs, spec.srcData)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expData, gotData)
			assert.Equal(t, spec.expEvts, em.Events())
		})
	}
}

func TestReply(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper
	var mock wasmtesting.MockWasmEngine
	wasmtesting.MakeInstantiable(&mock)
	example := SeedNewContractInstance(t, ctx, keepers, &mock)

	specs := map[string]struct {
		replyFn func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error)
		expData []byte
		expErr  bool
		expEvt  sdk.Events
	}{
		"all good": {
			replyFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
				return &wasmvmtypes.Response{Data: []byte("foo")}, 1, nil
			},
			expData: []byte("foo"),
			expEvt:  sdk.Events{sdk.NewEvent("reply", sdk.NewAttribute("_contract_address", example.Contract.String()))},
		},
		"with query": {
			replyFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
				bzRsp, err := querier.Query(wasmvmtypes.QueryRequest{
					Bank: &wasmvmtypes.BankQuery{
						Balance: &wasmvmtypes.BalanceQuery{Address: env.Contract.Address, Denom: "stake"},
					},
				}, 10_000*types.DefaultGasMultiplier)
				require.NoError(t, err)
				var gotBankRsp wasmvmtypes.BalanceResponse
				require.NoError(t, json.Unmarshal(bzRsp, &gotBankRsp))
				assert.Equal(t, wasmvmtypes.BalanceResponse{Amount: wasmvmtypes.NewCoin(0, "stake")}, gotBankRsp)
				return &wasmvmtypes.Response{Data: []byte("foo")}, 1, nil
			},
			expData: []byte("foo"),
			expEvt:  sdk.Events{sdk.NewEvent("reply", sdk.NewAttribute("_contract_address", example.Contract.String()))},
		},
		"with query error handled": {
			replyFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
				bzRsp, err := querier.Query(wasmvmtypes.QueryRequest{}, 0)
				require.Error(t, err)
				assert.Nil(t, bzRsp)
				return &wasmvmtypes.Response{Data: []byte("foo")}, 1, nil
			},
			expData: []byte("foo"),
			expEvt:  sdk.Events{sdk.NewEvent("reply", sdk.NewAttribute("_contract_address", example.Contract.String()))},
		},
		"error": {
			replyFn: func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.Response, uint64, error) {
				return nil, 1, errors.New("testing")
			},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			mock.ReplyFn = func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
				resp, gasUsed, err := spec.replyFn(codeID, env, reply, store, goapi, querier, gasMeter, gasLimit, deserCost)
				return &wasmvmtypes.ContractResult{
					Ok: resp,
				}, gasUsed, err
			}
			em := sdk.NewEventManager()
			gotData, gotErr := k.reply(ctx.WithEventManager(em), example.Contract, wasmvmtypes.Reply{})
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expData, gotData)
			assert.Equal(t, spec.expEvt, em.Events())
		})
	}
}

func TestQueryIsolation(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper
	var mock wasmtesting.MockWasmEngine
	wasmtesting.MakeInstantiable(&mock)
	example := SeedNewContractInstance(t, ctx, keepers, &mock)
	WithQueryHandlerDecorator(func(other WasmVMQueryHandler) WasmVMQueryHandler {
		return WasmVMQueryHandlerFn(func(ctx sdk.Context, caller sdk.AccAddress, request wasmvmtypes.QueryRequest) ([]byte, error) {
			if request.Custom == nil {
				return other.HandleQuery(ctx, caller, request)
			}
			// here we write to DB which should not be persisted
			err := k.storeService.OpenKVStore(ctx).Set([]byte(`set_in_query`), []byte(`this_is_allowed`))
			return nil, err
		})
	}).apply(k)

	// when
	mock.ReplyFn = func(codeID wasmvm.Checksum, env wasmvmtypes.Env, reply wasmvmtypes.Reply, store wasmvm.KVStore, goapi wasmvm.GoAPI, querier wasmvm.Querier, gasMeter wasmvm.GasMeter, gasLimit uint64, deserCost wasmvmtypes.UFraction) (*wasmvmtypes.ContractResult, uint64, error) {
		_, err := querier.Query(wasmvmtypes.QueryRequest{
			Custom: []byte(`{}`),
		}, 10000*types.DefaultGasMultiplier)
		require.NoError(t, err)
		return &wasmvmtypes.ContractResult{Ok: &wasmvmtypes.Response{}}, 0, nil
	}
	em := sdk.NewEventManager()
	_, gotErr := k.reply(ctx.WithEventManager(em), example.Contract, wasmvmtypes.Reply{})
	require.NoError(t, gotErr)
	got, err := k.storeService.OpenKVStore(ctx).Get([]byte(`set_in_query`))
	require.NoError(t, err)
	assert.Nil(t, got)
}

func TestSetAccessConfig(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper
	creatorAddr := RandomAccountAddress(t)
	nonCreatorAddr := RandomAccountAddress(t)
	const codeID = 1

	specs := map[string]struct {
		authz           types.AuthorizationPolicy
		chainPermission types.AccessType
		newConfig       types.AccessConfig
		caller          sdk.AccAddress
		expErr          bool
		expEvts         map[string]string
	}{
		"user with new permissions == chain permissions": {
			authz:           DefaultAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowEverybody,
			caller:          creatorAddr,
			expEvts: map[string]string{
				"code_id":         "1",
				"code_permission": "Everybody",
			},
		},
		"user with new permissions < chain permissions": {
			authz:           DefaultAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowNobody,
			caller:          creatorAddr,
			expEvts: map[string]string{
				"code_id":         "1",
				"code_permission": "Nobody",
			},
		},
		"user with new permissions > chain permissions": {
			authz:           DefaultAuthorizationPolicy{},
			chainPermission: types.AccessTypeNobody,
			newConfig:       types.AllowEverybody,
			caller:          creatorAddr,
			expErr:          true,
		},
		"different actor": {
			authz:           DefaultAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowEverybody,
			caller:          nonCreatorAddr,
			expErr:          true,
		},
		"gov with new permissions == chain permissions": {
			authz:           GovAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowEverybody,
			caller:          creatorAddr,
			expEvts: map[string]string{
				"code_id":         "1",
				"code_permission": "Everybody",
			},
		},
		"gov with new permissions < chain permissions": {
			authz:           GovAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowNobody,
			caller:          creatorAddr,
			expEvts: map[string]string{
				"code_id":         "1",
				"code_permission": "Nobody",
			},
		},
		"gov with new permissions > chain permissions - multiple addresses": {
			authz:           GovAuthorizationPolicy{},
			chainPermission: types.AccessTypeNobody,
			newConfig:       types.AccessTypeAnyOfAddresses.With(creatorAddr, nonCreatorAddr),
			caller:          creatorAddr,
			expEvts: map[string]string{
				"code_id":              "1",
				"code_permission":      "AnyOfAddresses",
				"authorized_addresses": creatorAddr.String() + "," + nonCreatorAddr.String(),
			},
		},
		"gov without actor": {
			authz:           GovAuthorizationPolicy{},
			chainPermission: types.AccessTypeEverybody,
			newConfig:       types.AllowEverybody,
			expEvts: map[string]string{
				"code_id":         "1",
				"code_permission": "Everybody",
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			em := sdk.NewEventManager()
			ctx = ctx.WithEventManager(em)

			newParams := types.DefaultParams()
			newParams.InstantiateDefaultPermission = spec.chainPermission
			err := k.SetParams(ctx, newParams)
			require.NoError(t, err)

			k.mustStoreCodeInfo(ctx, codeID, types.NewCodeInfo(nil, creatorAddr, types.AllowNobody))
			// when
			gotErr := k.setAccessConfig(ctx, codeID, spec.caller, spec.newConfig, spec.authz)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			// and event emitted
			require.Len(t, em.Events(), 1)
			assert.Equal(t, "update_code_access_config", em.Events()[0].Type)
			assert.Equal(t, spec.expEvts, attrsToStringMap(em.Events()[0].Attributes))
		})
	}
}

func TestAppendToContractHistory(t *testing.T) {
	f := fuzz.New().Funcs(ModelFuzzers...)
	pCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper

	variableLengthAddresses := []sdk.AccAddress{
		bytes.Repeat([]byte{0x1}, types.ContractAddrLen),
		append([]byte{0x00}, bytes.Repeat([]byte{0x1}, types.ContractAddrLen-1)...),
		append(bytes.Repeat([]byte{0x1}, types.ContractAddrLen-1), 0x00),
		append([]byte{0xff}, bytes.Repeat([]byte{0x1}, types.ContractAddrLen-1)...),
		append(bytes.Repeat([]byte{0x1}, types.ContractAddrLen-1), 0xff),
		bytes.Repeat([]byte{0x1}, types.SDKAddrLen),
		append([]byte{0x00}, bytes.Repeat([]byte{0x1}, types.SDKAddrLen-1)...),
		append(bytes.Repeat([]byte{0x1}, types.SDKAddrLen-1), 0x00),
		append([]byte{0xff}, bytes.Repeat([]byte{0x1}, types.SDKAddrLen-1)...),
		append(bytes.Repeat([]byte{0x1}, types.SDKAddrLen-1), 0xff),
	}
	sRandom := stdrand.New(stdrand.NewSource(0))
	for n := 0; n < 100; n++ {
		t.Run(fmt.Sprintf("iteration %d", n), func(t *testing.T) {
			sRandom.Seed(int64(n))
			sRandom.Shuffle(len(variableLengthAddresses), func(i, j int) {
				variableLengthAddresses[i], variableLengthAddresses[j] = variableLengthAddresses[j], variableLengthAddresses[i]
			})
			orderedEntries := make([][]types.ContractCodeHistoryEntry, len(variableLengthAddresses))

			ctx, _ := pCtx.CacheContext()
			for j, addr := range variableLengthAddresses {
				for i := 0; i < 10; i++ {
					var entry types.ContractCodeHistoryEntry
					f.RandSource(sRandom).Fuzz(&entry)
					require.NoError(t, k.appendToContractHistory(ctx, addr, entry))
					orderedEntries[j] = append(orderedEntries[j], entry)
				}
			}
			// when
			for j, addr := range variableLengthAddresses {
				gotHistory := k.GetContractHistory(ctx, addr)
				assert.Equal(t, orderedEntries[j], gotHistory, "%d: %X", j, addr)
				assert.Equal(t, orderedEntries[j][len(orderedEntries[j])-1], k.mustGetLastContractHistoryEntry(ctx, addr))
			}
		})
	}
}

func TestCoinBurnerPruneBalances(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	amts := sdk.NewCoins(sdk.NewInt64Coin("denom", 100))
	senderAddr := keepers.Faucet.NewFundedRandomAccount(parentCtx, amts...)

	// create vesting account
	var vestingAddr sdk.AccAddress = rand.Bytes(types.ContractAddrLen)
	msgCreateVestingAccount := vestingtypes.NewMsgCreateVestingAccount(senderAddr, vestingAddr, amts, time.Now().Add(time.Minute).Unix(), false)
	_, err := vesting.NewMsgServerImpl(keepers.AccountKeeper, keepers.BankKeeper).CreateVestingAccount(parentCtx, msgCreateVestingAccount)
	require.NoError(t, err)
	myVestingAccount := keepers.AccountKeeper.GetAccount(parentCtx, vestingAddr)
	require.NotNil(t, myVestingAccount)

	specs := map[string]struct {
		setupAcc    func(t *testing.T, ctx sdk.Context) sdk.AccountI
		expBalances sdk.Coins
		expHandled  bool
		expErr      *errorsmod.Error
	}{
		"vesting account - all removed": {
			setupAcc:    func(t *testing.T, ctx sdk.Context) sdk.AccountI { return myVestingAccount },
			expBalances: sdk.NewCoins(),
			expHandled:  true,
		},
		"vesting account with other tokens - only original denoms removed": {
			setupAcc: func(t *testing.T, ctx sdk.Context) sdk.AccountI {
				keepers.Faucet.Fund(ctx, vestingAddr, sdk.NewCoin("other", sdkmath.NewInt(2)))
				return myVestingAccount
			},
			expBalances: sdk.NewCoins(sdk.NewCoin("other", sdkmath.NewInt(2))),
			expHandled:  true,
		},
		"non vesting account - not handled": {
			setupAcc: func(t *testing.T, ctx sdk.Context) sdk.AccountI {
				return &authtypes.BaseAccount{Address: myVestingAccount.GetAddress().String()}
			},
			expBalances: sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(100))),
			expHandled:  false,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			existingAccount := spec.setupAcc(t, ctx)
			// overwrite account in store as in keeper before calling prune
			keepers.AccountKeeper.SetAccount(ctx, keepers.AccountKeeper.NewAccountWithAddress(ctx, vestingAddr))

			// when
			noGasCtx := ctx.WithGasMeter(storetypes.NewGasMeter(0)) // should not use callers gas
			gotHandled, gotErr := NewVestingCoinBurner(keepers.BankKeeper).CleanupExistingAccount(noGasCtx, existingAccount)
			// then
			if spec.expErr != nil {
				require.ErrorIs(t, gotErr, spec.expErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expBalances, keepers.BankKeeper.GetAllBalances(ctx, vestingAddr))
			assert.Equal(t, spec.expHandled, gotHandled)
			// and no out of gas panic
		})
	}
}

func TestIteratorAllContract(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	example1 := InstantiateHackatomExampleContract(t, ctx, keepers)
	example2 := InstantiateHackatomExampleContract(t, ctx, keepers)
	example3 := InstantiateHackatomExampleContract(t, ctx, keepers)
	example4 := InstantiateHackatomExampleContract(t, ctx, keepers)

	var allContract []string
	keepers.WasmKeeper.IterateContractInfo(ctx, func(addr sdk.AccAddress, _ types.ContractInfo) bool {
		allContract = append(allContract, addr.String())
		return false
	})

	// IterateContractInfo not ordering
	expContracts := []string{example4.Contract.String(), example2.Contract.String(), example1.Contract.String(), example3.Contract.String()}
	require.Equal(t, allContract, expContracts)
}

func TestIteratorContractByCreator(t *testing.T) {
	// setup test
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	keeper := keepers.ContractKeeper

	depositFund := sdk.NewCoins(sdk.NewInt64Coin("denom", 1000000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := DeterministicAccountAddress(t, 1)
	keepers.Faucet.Fund(parentCtx, creator, depositFund.Add(depositFund...)...)
	mockAddress1 := keepers.Faucet.NewFundedRandomAccount(parentCtx, topUp...)
	mockAddress2 := keepers.Faucet.NewFundedRandomAccount(parentCtx, topUp...)
	mockAddress3 := keepers.Faucet.NewFundedRandomAccount(parentCtx, topUp...)

	contract1ID, _, err := keeper.Create(parentCtx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	contract2ID, _, err := keeper.Create(parentCtx, creator, hackatomWasm, nil)
	require.NoError(t, err)

	initMsgBz := HackatomExampleInitMsg{
		Verifier:    mockAddress1,
		Beneficiary: mockAddress1,
	}.GetBytes(t)

	depositContract := sdk.NewCoins(sdk.NewCoin("denom", sdkmath.NewInt(1_000)))

	gotAddr1, _, _ := keepers.ContractKeeper.Instantiate(parentCtx, contract1ID, mockAddress1, nil, initMsgBz, "label", depositContract)
	ctx := parentCtx.WithBlockHeight(parentCtx.BlockHeight() + 1)
	gotAddr2, _, _ := keepers.ContractKeeper.Instantiate(ctx, contract1ID, mockAddress2, nil, initMsgBz, "label", depositContract)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	gotAddr3, _, _ := keepers.ContractKeeper.Instantiate(ctx, contract1ID, gotAddr1, nil, initMsgBz, "label", depositContract)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	gotAddr4, _, _ := keepers.ContractKeeper.Instantiate(ctx, contract2ID, mockAddress2, nil, initMsgBz, "label", depositContract)
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	gotAddr5, _, _ := keepers.ContractKeeper.Instantiate(ctx, contract2ID, mockAddress2, nil, initMsgBz, "label", depositContract)

	specs := map[string]struct {
		creatorAddr   sdk.AccAddress
		contractsAddr []string
	}{
		"single contract": {
			creatorAddr:   mockAddress1,
			contractsAddr: []string{gotAddr1.String()},
		},
		"multiple contracts": {
			creatorAddr:   mockAddress2,
			contractsAddr: []string{gotAddr2.String(), gotAddr4.String(), gotAddr5.String()},
		},
		"contractAdress": {
			creatorAddr:   gotAddr1,
			contractsAddr: []string{gotAddr3.String()},
		},
		"no contracts- unknown": {
			creatorAddr:   mockAddress3,
			contractsAddr: nil,
		},
	}

	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			var allContract []string
			keepers.WasmKeeper.IterateContractsByCreator(parentCtx, spec.creatorAddr, func(addr sdk.AccAddress) bool {
				allContract = append(allContract, addr.String())
				return false
			})
			require.Equal(t,
				allContract,
				spec.contractsAddr,
			)
		})
	}
}

func TestSetContractAdmin(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper
	myAddr := RandomAccountAddress(t)
	example := InstantiateReflectExampleContract(t, parentCtx, keepers)
	specs := map[string]struct {
		newAdmin sdk.AccAddress
		caller   sdk.AccAddress
		policy   types.AuthorizationPolicy
		expAdmin string
		expErr   bool
	}{
		"update admin": {
			newAdmin: myAddr,
			caller:   example.CreatorAddr,
			policy:   DefaultAuthorizationPolicy{},
			expAdmin: myAddr.String(),
		},
		"update admin - unauthorized": {
			newAdmin: myAddr,
			caller:   RandomAccountAddress(t),
			policy:   DefaultAuthorizationPolicy{},
			expErr:   true,
		},
		"clear admin - default policy": {
			caller:   example.CreatorAddr,
			policy:   DefaultAuthorizationPolicy{},
			expAdmin: "",
		},
		"clear admin - unauthorized": {
			expAdmin: "",
			policy:   DefaultAuthorizationPolicy{},
			caller:   RandomAccountAddress(t),
			expErr:   true,
		},
		"clear admin - gov policy": {
			newAdmin: nil,
			policy:   GovAuthorizationPolicy{},
			caller:   example.CreatorAddr,
			expAdmin: "",
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			em := sdk.NewEventManager()
			ctx = ctx.WithEventManager(em)
			gotErr := k.setContractAdmin(ctx, example.Contract, spec.caller, spec.newAdmin, spec.policy)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.expAdmin, k.GetContractInfo(ctx, example.Contract).Admin)
			// and event emitted
			require.Len(t, em.Events(), 1)
			assert.Equal(t, "update_contract_admin", em.Events()[0].Type)
			exp := map[string]string{
				"_contract_address": example.Contract.String(),
				"new_admin_address": spec.expAdmin,
			}
			assert.Equal(t, exp, attrsToStringMap(em.Events()[0].Attributes))
		})
	}
}

func TestGasConsumed(t *testing.T) {
	specs := map[string]struct {
		originalMeter            storetypes.GasMeter
		gasRegister              types.WasmGasRegister
		consumeGas               storetypes.Gas
		expPanic                 bool
		expMultipliedGasConsumed uint64
	}{
		"all good": {
			originalMeter:            storetypes.NewGasMeter(100),
			gasRegister:              types.NewWasmGasRegister(types.DefaultGasRegisterConfig()),
			consumeGas:               storetypes.Gas(1),
			expMultipliedGasConsumed: 140000,
		},
		"consumeGas = limit": {
			originalMeter:            storetypes.NewGasMeter(1),
			gasRegister:              types.NewWasmGasRegister(types.DefaultGasRegisterConfig()),
			consumeGas:               storetypes.Gas(1),
			expMultipliedGasConsumed: 140000,
		},
		"consumeGas > limit": {
			originalMeter: storetypes.NewGasMeter(10),
			gasRegister:   types.NewWasmGasRegister(types.DefaultGasRegisterConfig()),
			consumeGas:    storetypes.Gas(11),
			expPanic:      true,
		},
		"nil original meter": {
			gasRegister: types.NewWasmGasRegister(types.DefaultGasRegisterConfig()),
			consumeGas:  storetypes.Gas(1),
			expPanic:    true,
		},
		"nil gas register": {
			originalMeter: storetypes.NewGasMeter(100),
			consumeGas:    storetypes.Gas(1),
			expPanic:      true,
		},
	}

	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			m := NewMultipliedGasMeter(spec.originalMeter, spec.gasRegister)
			if spec.expPanic {
				assert.Panics(t, func() {
					m.originalMeter.ConsumeGas(spec.consumeGas, "test-panic")
					_ = m.GasConsumed()
				})
				return
			}

			m.originalMeter.ConsumeGas(spec.consumeGas, "test")
			assert.Equal(t, spec.expMultipliedGasConsumed, m.GasConsumed())
		})
	}
}

func TestSetContractLabel(t *testing.T) {
	parentCtx, keepers := CreateTestInput(t, false, AvailableCapabilities)
	k := keepers.WasmKeeper
	example := InstantiateReflectExampleContract(t, parentCtx, keepers)

	specs := map[string]struct {
		newLabel string
		caller   sdk.AccAddress
		policy   types.AuthorizationPolicy
		contract sdk.AccAddress
		expErr   bool
	}{
		"update label - default policy": {
			newLabel: "new label",
			caller:   example.CreatorAddr,
			policy:   DefaultAuthorizationPolicy{},
			contract: example.Contract,
		},
		"update label - gov policy": {
			newLabel: "new label",
			policy:   GovAuthorizationPolicy{},
			caller:   RandomAccountAddress(t),
			contract: example.Contract,
		},
		"update label - unauthorized": {
			newLabel: "new label",
			caller:   RandomAccountAddress(t),
			policy:   DefaultAuthorizationPolicy{},
			contract: example.Contract,
			expErr:   true,
		},
		"update label - unknown contract": {
			newLabel: "new label",
			caller:   example.CreatorAddr,
			policy:   DefaultAuthorizationPolicy{},
			contract: RandomAccountAddress(t),
			expErr:   true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := parentCtx.CacheContext()
			em := sdk.NewEventManager()
			ctx = ctx.WithEventManager(em)
			gotErr := k.setContractLabel(ctx, spec.contract, spec.caller, spec.newLabel, spec.policy)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, spec.newLabel, k.GetContractInfo(ctx, spec.contract).Label)
			// and event emitted
			require.Len(t, em.Events(), 1)
			assert.Equal(t, "update_contract_label", em.Events()[0].Type)
			exp := map[string]string{
				"_contract_address": spec.contract.String(),
				"new_label":         spec.newLabel,
			}
			assert.Equal(t, exp, attrsToStringMap(em.Events()[0].Attributes))
		})
	}
}

func attrsToStringMap(attrs []abci.EventAttribute) map[string]string {
	r := make(map[string]string, len(attrs))
	for _, v := range attrs {
		r[v.Key] = v.Value
	}
	return r
}

func must[t any](s t, err error) t {
	if err != nil {
		panic(err)
	}
	return s
}
