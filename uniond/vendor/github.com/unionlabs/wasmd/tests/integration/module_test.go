package integration

import (
	"bytes"
	"encoding/json"
	"os"
	"strings"
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/module"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "cosmossdk.io/x/bank/keeper"
	stakingkeeper "cosmossdk.io/x/staking/keeper"

	"github.com/CosmWasm/wasmd/x/wasm"
	"github.com/CosmWasm/wasmd/x/wasm/exported"
	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	v2 "github.com/CosmWasm/wasmd/x/wasm/migrations/v2"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

type mockSubspace struct {
	ps v2.Params
}

func newMockSubspace(ps v2.Params) mockSubspace {
	return mockSubspace{ps: ps}
}

func (ms mockSubspace) GetParamSet(ctx sdk.Context, ps exported.ParamSet) {
	*ps.(*v2.Params) = ms.ps
}

type testData struct {
	module           wasm.AppModule
	ctx              sdk.Context
	acctKeeper       authkeeper.AccountKeeper
	keeper           keeper.Keeper
	bankKeeper       bankkeeper.Keeper
	stakingKeeper    *stakingkeeper.Keeper
	faucet           *keeper.TestFaucet
	grpcQueryRouter  *baseapp.GRPCQueryRouter
	msgServiceRouter *baseapp.MsgServiceRouter
	encConf          moduletestutil.TestEncodingConfig
}

func setupTest(t *testing.T) testData {
	t.Helper()
	DefaultParams := v2.Params{
		CodeUploadAccess:             v2.AccessConfig{Permission: v2.AccessTypeEverybody},
		InstantiateDefaultPermission: v2.AccessTypeEverybody,
	}

	ctx, keepers := keeper.CreateTestInput(t, false, []string{
		"iterator", "staking", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3",
		"cosmwasm_1_4", "cosmwasm_2_0", "cosmwasm_2_1",
	})
	encConf := keeper.MakeEncodingConfig(t)
	queryRouter := baseapp.NewGRPCQueryRouter()
	serviceRouter := baseapp.NewMsgServiceRouter()
	queryRouter.SetInterfaceRegistry(encConf.InterfaceRegistry)
	serviceRouter.SetInterfaceRegistry(encConf.InterfaceRegistry)
	data := testData{
		module:           wasm.NewAppModule(encConf.Codec, keepers.WasmKeeper, keepers.StakingKeeper, keepers.AccountKeeper, keepers.BankKeeper, nil, newMockSubspace(DefaultParams)),
		ctx:              ctx,
		acctKeeper:       keepers.AccountKeeper,
		keeper:           *keepers.WasmKeeper,
		bankKeeper:       keepers.BankKeeper,
		stakingKeeper:    keepers.StakingKeeper,
		faucet:           keepers.Faucet,
		grpcQueryRouter:  queryRouter,
		msgServiceRouter: serviceRouter,
		encConf:          encConf,
	}
	data.module.RegisterServices(module.NewConfigurator(encConf.Codec, serviceRouter, queryRouter))
	return data
}

func keyPubAddr() sdk.AccAddress {
	key := ed25519.GenPrivKey()
	pub := key.PubKey()
	addr := sdk.AccAddress(pub.Address())
	return addr
}

func mustLoad(path string) []byte {
	bz, err := os.ReadFile(path)
	if err != nil {
		panic(err)
	}
	return bz
}

var (
	addrAcc1     = keyPubAddr()
	addr1        = addrAcc1.String()
	testContract = mustLoad("./testdata/hackatom.wasm")
	maskContract = testdata.ReflectContractWasm()
	oldContract  = mustLoad("./testdata/escrow_0.7.wasm")
)

func TestHandleCreate(t *testing.T) {
	cases := map[string]struct {
		msg     sdk.Msg
		isValid bool
	}{
		"empty": {
			msg:     &types.MsgStoreCode{},
			isValid: false,
		},
		"invalid wasm": {
			msg: &types.MsgStoreCode{
				Sender:       addr1,
				WASMByteCode: []byte("foobar"),
			},
			isValid: false,
		},
		"valid wasm": {
			msg: &types.MsgStoreCode{
				Sender:       addr1,
				WASMByteCode: testContract,
			},
			isValid: true,
		},
		"other valid wasm": {
			msg: &types.MsgStoreCode{
				Sender:       addr1,
				WASMByteCode: maskContract,
			},
			isValid: true,
		},
		"old wasm (0.7)": {
			msg: &types.MsgStoreCode{
				Sender:       addr1,
				WASMByteCode: oldContract,
			},
			isValid: false,
		},
	}

	for name, tc := range cases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			data := setupTest(t)

			h := data.msgServiceRouter.Handler(tc.msg)
			// q := data.grpcQueryRouter.Route(sdk.MsgTypeURL(tc.msg))
			q := data.grpcQueryRouter

			res, err := h(data.ctx, tc.msg)
			if !tc.isValid {
				require.Error(t, err, "%#v", res)
				assertCodeList(t, q, data.ctx, 0, data.encConf.Codec)
				assertCodeBytes(t, q, data.ctx, 1, nil, data.encConf.Codec)
				return
			}
			require.NoError(t, err)
			assertCodeList(t, q, data.ctx, 1, data.encConf.Codec)
		})
	}
}

type initMsg struct {
	Verifier    sdk.AccAddress `json:"verifier"`
	Beneficiary sdk.AccAddress `json:"beneficiary"`
}

type state struct {
	Verifier    string `json:"verifier"`
	Beneficiary string `json:"beneficiary"`
	Funder      string `json:"funder"`
}

func TestHandleInstantiate(t *testing.T) {
	data := setupTest(t)
	creator := data.faucet.NewFundedRandomAccount(data.ctx, sdk.NewInt64Coin("denom", 100000))

	msg := &types.MsgStoreCode{
		Sender:       creator.String(),
		WASMByteCode: testContract,
	}

	h := data.msgServiceRouter.Handler(msg)
	q := data.grpcQueryRouter

	res, err := h(data.ctx, msg)
	require.NoError(t, err)
	assertStoreCodeResponse(t, res.Data, 1)

	bob := keyPubAddr()
	fred := keyPubAddr()

	initPayload := initMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initPayload)
	require.NoError(t, err)

	// create with no balance is also legal
	initMsg := &types.MsgInstantiateContract{
		Sender: creator.String(),
		CodeID: firstCodeID,
		Msg:    initMsgBz,
		Funds:  nil,
		Label:  "testing",
	}
	h = data.msgServiceRouter.Handler(initMsg)
	res, err = h(data.ctx, initMsg)
	require.NoError(t, err)
	contractBech32Addr := parseInitResponse(t, res.Data)

	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", contractBech32Addr)
	// this should be standard x/wasm init event, nothing from contract
	require.Equal(t, 2, len(res.Events), prettyEvents(res.Events))
	require.Equal(t, "instantiate", res.Events[0].Type)
	require.Equal(t, "wasm", res.Events[1].Type)
	assertAttribute(t, "_contract_address", contractBech32Addr, res.Events[1].Attributes[0])

	assertCodeList(t, q, data.ctx, 1, data.encConf.Codec)
	assertCodeBytes(t, q, data.ctx, 1, testContract, data.encConf.Codec)

	assertContractList(t, q, data.ctx, 1, []string{contractBech32Addr}, data.encConf.Codec)
	assertContractInfo(t, q, data.ctx, contractBech32Addr, 1, creator, data.encConf.Codec)
	assertContractState(t, q, data.ctx, contractBech32Addr, state{
		Verifier:    fred.String(),
		Beneficiary: bob.String(),
		Funder:      creator.String(),
	}, data.encConf.Codec)
}

func TestHandleExecute(t *testing.T) {
	data := setupTest(t)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))

	creator := data.faucet.NewFundedRandomAccount(data.ctx, deposit.Add(deposit...)...)
	fred := data.faucet.NewFundedRandomAccount(data.ctx, topUp...)

	msg := &types.MsgStoreCode{
		Sender:       creator.String(),
		WASMByteCode: testContract,
	}
	h := data.msgServiceRouter.Handler(msg)
	q := data.grpcQueryRouter
	res, err := h(data.ctx, msg)
	require.NoError(t, err)
	assertStoreCodeResponse(t, res.Data, 1)

	bob := keyPubAddr()
	initMsg := initMsg{
		Verifier:    fred,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	initCmd := &types.MsgInstantiateContract{
		Sender: creator.String(),
		CodeID: firstCodeID,
		Msg:    initMsgBz,
		Funds:  deposit,
		Label:  "testing",
	}
	h = data.msgServiceRouter.Handler(initCmd)
	res, err = h(data.ctx, initCmd)
	require.NoError(t, err)
	contractBech32Addr := parseInitResponse(t, res.Data)

	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", contractBech32Addr)
	// this should be standard x/wasm message event,  init event, plus a bank send event (2), with no custom contract events
	require.Equal(t, 5, len(res.Events), prettyEvents(res.Events))
	require.Equal(t, "coin_spent", res.Events[0].Type)
	require.Equal(t, "coin_received", res.Events[1].Type)
	require.Equal(t, "transfer", res.Events[2].Type)
	require.Equal(t, "instantiate", res.Events[3].Type)
	require.Equal(t, "wasm", res.Events[4].Type)
	assertAttribute(t, "_contract_address", contractBech32Addr, res.Events[4].Attributes[0])

	// ensure bob doesn't exist
	bobAcct := data.acctKeeper.GetAccount(data.ctx, bob)
	require.Nil(t, bobAcct)

	// ensure funder has reduced balance
	creatorAcct := data.acctKeeper.GetAccount(data.ctx, creator)
	require.NotNil(t, creatorAcct)
	// we started at 2*deposit, should have spent one above
	assert.Equal(t, deposit, data.bankKeeper.GetAllBalances(data.ctx, creatorAcct.GetAddress()))

	// ensure contract has updated balance
	contractAddr, _ := sdk.AccAddressFromBech32(contractBech32Addr)
	contractAcct := data.acctKeeper.GetAccount(data.ctx, contractAddr)
	require.NotNil(t, contractAcct)
	assert.Equal(t, deposit, data.bankKeeper.GetAllBalances(data.ctx, contractAcct.GetAddress()))

	execCmd := &types.MsgExecuteContract{
		Sender:   fred.String(),
		Contract: contractBech32Addr,
		Msg:      []byte(`{"release":{}}`),
		Funds:    topUp,
	}
	h = data.msgServiceRouter.Handler(execCmd)
	res, err = h(data.ctx, execCmd)
	require.NoError(t, err)
	// from https://github.com/CosmWasm/cosmwasm/blob/master/contracts/hackatom/src/contract.rs#L167
	assertExecuteResponse(t, res.Data, []byte{0xf0, 0x0b, 0xaa})

	// this should be standard message event, plus x/wasm init event, plus 2 bank send event, plus a special event from the contract
	require.Equal(t, 9, len(res.Events), prettyEvents(res.Events))

	assert.Equal(t, "coin_spent", res.Events[0].Type)
	assert.Equal(t, "coin_received", res.Events[1].Type)

	require.Equal(t, "transfer", res.Events[2].Type)
	require.Len(t, res.Events[2].Attributes, 3)
	assertAttribute(t, "recipient", contractBech32Addr, res.Events[2].Attributes[0])
	assertAttribute(t, "sender", fred.String(), res.Events[2].Attributes[1])
	assertAttribute(t, "amount", "5000denom", res.Events[2].Attributes[2])

	assert.Equal(t, "execute", res.Events[3].Type)

	// custom contract event attribute
	assert.Equal(t, "wasm", res.Events[4].Type)
	assertAttribute(t, "_contract_address", contractBech32Addr, res.Events[4].Attributes[0])
	assertAttribute(t, "action", "release", res.Events[4].Attributes[1])
	// custom contract event
	assert.Equal(t, "wasm-hackatom", res.Events[5].Type)
	assertAttribute(t, "_contract_address", contractBech32Addr, res.Events[5].Attributes[0])
	assertAttribute(t, "action", "release", res.Events[5].Attributes[1])
	// second transfer (this without conflicting message)
	assert.Equal(t, "coin_spent", res.Events[6].Type)
	assert.Equal(t, "coin_received", res.Events[7].Type)

	assert.Equal(t, "transfer", res.Events[8].Type)
	assertAttribute(t, "recipient", bob.String(), res.Events[8].Attributes[0])
	assertAttribute(t, "sender", contractBech32Addr, res.Events[8].Attributes[1])
	assertAttribute(t, "amount", "105000denom", res.Events[8].Attributes[2])
	// finally, standard x/wasm tag

	// ensure bob now exists and got both payments released
	bobAcct = data.acctKeeper.GetAccount(data.ctx, bob)
	require.NotNil(t, bobAcct)
	balance := data.bankKeeper.GetAllBalances(data.ctx, bobAcct.GetAddress())
	assert.Equal(t, deposit.Add(topUp...), balance)

	// ensure contract has updated balance

	contractAcct = data.acctKeeper.GetAccount(data.ctx, contractAddr)
	require.NotNil(t, contractAcct)
	assert.Equal(t, sdk.Coins{}, data.bankKeeper.GetAllBalances(data.ctx, contractAcct.GetAddress()))

	// ensure all contract state is as after init
	assertCodeList(t, q, data.ctx, 1, data.encConf.Codec)
	assertCodeBytes(t, q, data.ctx, 1, testContract, data.encConf.Codec)

	assertContractList(t, q, data.ctx, 1, []string{contractBech32Addr}, data.encConf.Codec)
	assertContractInfo(t, q, data.ctx, contractBech32Addr, 1, creator, data.encConf.Codec)
	assertContractState(t, q, data.ctx, contractBech32Addr, state{
		Verifier:    fred.String(),
		Beneficiary: bob.String(),
		Funder:      creator.String(),
	}, data.encConf.Codec)
}

func TestHandleExecuteEscrow(t *testing.T) {
	data := setupTest(t)

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	topUp := sdk.NewCoins(sdk.NewInt64Coin("denom", 5000))
	creator := sdk.AccAddress(bytes.Repeat([]byte{1}, address.Len))
	data.faucet.Fund(data.ctx, creator, sdk.NewInt64Coin("denom", 100000))
	fred := data.faucet.NewFundedRandomAccount(data.ctx, topUp...)

	msg := &types.MsgStoreCode{
		Sender:       creator.String(),
		WASMByteCode: testContract,
	}

	h := data.msgServiceRouter.Handler(msg)
	_, err := h(data.ctx, msg)
	require.NoError(t, err)

	bob := keyPubAddr()
	initMsg := map[string]interface{}{
		"verifier":    fred.String(),
		"beneficiary": bob.String(),
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)

	initCmd := types.MsgInstantiateContract{
		Sender: creator.String(),
		CodeID: firstCodeID,
		Msg:    initMsgBz,
		Funds:  deposit,
		Label:  "testing",
	}
	h = data.msgServiceRouter.Handler(&initCmd)
	res, err := h(data.ctx, &initCmd)
	require.NoError(t, err)
	contractBech32Addr := parseInitResponse(t, res.Data)
	require.Equal(t, "cosmos14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9s4hmalr", contractBech32Addr)

	handleMsg := map[string]interface{}{
		"release": map[string]interface{}{},
	}
	handleMsgBz, err := json.Marshal(handleMsg)
	require.NoError(t, err)

	execCmd := types.MsgExecuteContract{
		Sender:   fred.String(),
		Contract: contractBech32Addr,
		Msg:      handleMsgBz,
		Funds:    topUp,
	}
	h = data.msgServiceRouter.Handler(&execCmd)
	res, err = h(data.ctx, &execCmd)
	require.NoError(t, err)
	// from https://github.com/CosmWasm/cosmwasm/blob/master/contracts/hackatom/src/contract.rs#L167
	assertExecuteResponse(t, res.Data, []byte{0xf0, 0x0b, 0xaa})

	// ensure bob now exists and got both payments released
	bobAcct := data.acctKeeper.GetAccount(data.ctx, bob)
	require.NotNil(t, bobAcct)
	balance := data.bankKeeper.GetAllBalances(data.ctx, bobAcct.GetAddress())
	assert.Equal(t, deposit.Add(topUp...), balance)

	// ensure contract has updated balance
	contractAddr, _ := sdk.AccAddressFromBech32(contractBech32Addr)
	contractAcct := data.acctKeeper.GetAccount(data.ctx, contractAddr)
	require.NotNil(t, contractAcct)
	assert.Equal(t, sdk.Coins{}, data.bankKeeper.GetAllBalances(data.ctx, contractAcct.GetAddress()))
}

func TestReadWasmConfig(t *testing.T) {
	withViper := func(s string) *viper.Viper {
		v := viper.New()
		v.SetConfigType("toml")
		require.NoError(t, v.ReadConfig(strings.NewReader(s)))
		return v
	}
	var one uint64 = 1
	defaults := types.DefaultWasmConfig()

	specs := map[string]struct {
		src servertypes.AppOptions
		exp types.WasmConfig
	}{
		"set query gas limit via opts": {
			src: AppOptionsMock{
				"wasm.query_gas_limit": 1,
			},
			exp: types.WasmConfig{
				SmartQueryGasLimit: 1,
				MemoryCacheSize:    defaults.MemoryCacheSize,
			},
		},
		"set cache via opts": {
			src: AppOptionsMock{
				"wasm.memory_cache_size": 2,
			},
			exp: types.WasmConfig{
				MemoryCacheSize:    2,
				SmartQueryGasLimit: defaults.SmartQueryGasLimit,
			},
		},
		"set debug via opts": {
			src: AppOptionsMock{
				"trace": true,
			},
			exp: types.WasmConfig{
				SmartQueryGasLimit: defaults.SmartQueryGasLimit,
				MemoryCacheSize:    defaults.MemoryCacheSize,
				ContractDebugMode:  true,
			},
		},
		"all defaults when no options set": {
			src: AppOptionsMock{},
			exp: defaults,
		},
		"default config template values": {
			src: withViper(types.DefaultConfigTemplate()),
			exp: defaults,
		},
		"custom config template values": {
			src: withViper(types.ConfigTemplate(types.WasmConfig{
				SimulationGasLimit: &one,
				SmartQueryGasLimit: 2,
				MemoryCacheSize:    3,
			})),
			exp: types.WasmConfig{
				SimulationGasLimit: &one,
				SmartQueryGasLimit: 2,
				MemoryCacheSize:    3,
				ContractDebugMode:  false,
			},
		},
	}
	for msg, spec := range specs {
		t.Run(msg, func(t *testing.T) {
			got, err := wasm.ReadWasmConfig(spec.src)
			require.NoError(t, err)
			assert.Equal(t, spec.exp, got)
		})
	}
}

type AppOptionsMock map[string]interface{}

func (a AppOptionsMock) Get(s string) interface{} {
	return a[s]
}

type prettyEvent struct {
	Type string
	Attr []sdk.Attribute
}

func prettyEvents(evts []abci.Event) string {
	res := make([]prettyEvent, len(evts))
	for i, e := range evts {
		res[i] = prettyEvent{
			Type: e.Type,
			Attr: prettyAttrs(e.Attributes),
		}
	}
	bz, err := json.MarshalIndent(res, "", "  ")
	if err != nil {
		panic(err)
	}
	return string(bz)
}

func prettyAttrs(attrs []abci.EventAttribute) []sdk.Attribute {
	pretty := make([]sdk.Attribute, len(attrs))
	for i, a := range attrs {
		pretty[i] = prettyAttr(a)
	}
	return pretty
}

func prettyAttr(attr abci.EventAttribute) sdk.Attribute {
	return sdk.NewAttribute(attr.Key, attr.Value)
}

func assertAttribute(t *testing.T, key, value string, attr abci.EventAttribute) {
	t.Helper()
	assert.Equal(t, key, attr.Key, prettyAttr(attr))
	assert.Equal(t, value, attr.Value, prettyAttr(attr))
}

func assertCodeList(t *testing.T, q *baseapp.GRPCQueryRouter, ctx sdk.Context, expectedNum int, marshaler codec.Codec) {
	t.Helper()
	path := "/cosmwasm.wasm.v1.Query/Codes"
	resp, sdkerr := q.Route(path)(ctx, &abci.RequestQuery{Path: path})
	require.NoError(t, sdkerr)
	require.True(t, resp.IsOK())

	bz := resp.Value
	if len(bz) == 0 {
		require.Equal(t, expectedNum, 0)
		return
	}

	var res types.QueryCodesResponse
	require.NoError(t, marshaler.Unmarshal(bz, &res))
	assert.Equal(t, expectedNum, len(res.CodeInfos))
}

func assertCodeBytes(t *testing.T, q *baseapp.GRPCQueryRouter, ctx sdk.Context, codeID uint64, expectedBytes []byte, marshaler codec.Codec) {
	t.Helper()
	bz, err := marshaler.Marshal(&types.QueryCodeRequest{CodeId: codeID})
	require.NoError(t, err)

	path := "/cosmwasm.wasm.v1.Query/Code"
	resp, err := q.Route(path)(ctx, &abci.RequestQuery{Path: path, Data: bz})
	if len(expectedBytes) == 0 {
		require.Equal(t, types.ErrNoSuchCodeFn(codeID).Wrapf("code id %d", codeID).Error(), err.Error())
		return
	}
	require.NoError(t, err)
	require.True(t, resp.IsOK())
	bz = resp.Value

	var rsp types.QueryCodeResponse
	require.NoError(t, marshaler.Unmarshal(bz, &rsp))
	assert.Equal(t, expectedBytes, rsp.Data)
}

func assertContractList(t *testing.T, q *baseapp.GRPCQueryRouter, ctx sdk.Context, codeID uint64, expContractAddrs []string, marshaler codec.Codec) {
	t.Helper()
	bz, err := marshaler.Marshal(&types.QueryContractsByCodeRequest{CodeId: codeID})
	require.NoError(t, err)

	path := "/cosmwasm.wasm.v1.Query/ContractsByCode"
	resp, sdkerr := q.Route(path)(ctx, &abci.RequestQuery{Path: path, Data: bz})
	if len(expContractAddrs) == 0 {
		assert.ErrorIs(t, err, types.ErrNotFound)
		return
	}
	require.NoError(t, sdkerr)
	require.True(t, resp.IsOK())
	bz = resp.Value

	var rsp types.QueryContractsByCodeResponse
	require.NoError(t, marshaler.Unmarshal(bz, &rsp))

	hasAddrs := make([]string, len(rsp.Contracts))
	for i, r := range rsp.Contracts { //nolint:gosimple
		hasAddrs[i] = r
	}
	assert.Equal(t, expContractAddrs, hasAddrs)
}

func assertContractState(t *testing.T, q *baseapp.GRPCQueryRouter, ctx sdk.Context, contractBech32Addr string, expected state, marshaler codec.Codec) {
	t.Helper()
	bz, err := marshaler.Marshal(&types.QueryRawContractStateRequest{Address: contractBech32Addr, QueryData: []byte("config")})
	require.NoError(t, err)

	path := "/cosmwasm.wasm.v1.Query/RawContractState"
	resp, sdkerr := q.Route(path)(ctx, &abci.RequestQuery{Path: path, Data: bz})
	require.NoError(t, sdkerr)
	require.True(t, resp.IsOK())
	bz = resp.Value

	var rsp types.QueryRawContractStateResponse
	require.NoError(t, marshaler.Unmarshal(bz, &rsp))
	expectedBz, err := json.Marshal(expected)
	require.NoError(t, err)
	assert.Equal(t, expectedBz, rsp.Data)
}

func assertContractInfo(t *testing.T, q *baseapp.GRPCQueryRouter, ctx sdk.Context, contractBech32Addr string, codeID uint64, creator sdk.AccAddress, marshaler codec.Codec) {
	t.Helper()
	bz, err := marshaler.Marshal(&types.QueryContractInfoRequest{Address: contractBech32Addr})
	require.NoError(t, err)

	path := "/cosmwasm.wasm.v1.Query/ContractInfo"
	resp, sdkerr := q.Route(path)(ctx, &abci.RequestQuery{Path: path, Data: bz})
	require.NoError(t, sdkerr)
	require.True(t, resp.IsOK())
	bz = resp.Value

	var rsp types.QueryContractInfoResponse
	require.NoError(t, marshaler.Unmarshal(bz, &rsp))

	assert.Equal(t, codeID, rsp.CodeID)
	assert.Equal(t, creator.String(), rsp.Creator)
}
