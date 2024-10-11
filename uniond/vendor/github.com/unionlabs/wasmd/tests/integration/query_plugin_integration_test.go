package integration

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"testing"
	"time"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	"github.com/cosmos/gogoproto/proto"
	channeltypes "github.com/cosmos/ibc-go/v8/modules/core/04-channel/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/query"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/CosmWasm/wasmd/app"
	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	wasmKeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestMaskReflectCustomQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	// upload code
	codeID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	// creator instantiates a contract and gives it tokens
	contractStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "reflect contract 1", contractStart)
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// let's perform a normal query of state
	ownerQuery := testdata.ReflectQueryMsg{
		Owner: &struct{}{},
	}
	ownerQueryBz, err := json.Marshal(ownerQuery)
	require.NoError(t, err)
	ownerRes, err := keeper.QuerySmart(ctx, contractAddr, ownerQueryBz)
	require.NoError(t, err)
	var res testdata.OwnerResponse
	err = json.Unmarshal(ownerRes, &res)
	require.NoError(t, err)
	assert.Equal(t, res.Owner, creator.String())

	// and now making use of the custom querier callbacks
	customQuery := testdata.ReflectQueryMsg{
		Capitalized: &testdata.Text{
			Text: "all Caps noW",
		},
	}
	customQueryBz, err := json.Marshal(customQuery)
	require.NoError(t, err)
	custom, err := keeper.QuerySmart(ctx, contractAddr, customQueryBz)
	require.NoError(t, err)
	var resp capitalizedResponse
	err = json.Unmarshal(custom, &resp)
	require.NoError(t, err)
	assert.Equal(t, resp.Text, "ALL CAPS NOW")
}

func TestReflectStargateQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	funds := sdk.NewCoins(sdk.NewInt64Coin("denom", 320000))
	contractStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	expectedBalance := funds.Sub(contractStart...)
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, funds...)

	// upload code
	codeID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	// creator instantiates a contract and gives it tokens
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "reflect contract 1", contractStart)
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// first, normal query for the bank balance (to make sure our query is proper)
	bankQuery := wasmvmtypes.QueryRequest{
		Bank: &wasmvmtypes.BankQuery{
			AllBalances: &wasmvmtypes.AllBalancesQuery{
				Address: creator.String(),
			},
		},
	}
	simpleQueryBz, err := json.Marshal(testdata.ReflectQueryMsg{
		Chain: &testdata.ChainQuery{Request: &bankQuery},
	})
	require.NoError(t, err)
	simpleRes, err := keeper.QuerySmart(ctx, contractAddr, simpleQueryBz)
	require.NoError(t, err)
	var simpleChain testdata.ChainResponse
	mustUnmarshal(t, simpleRes, &simpleChain)
	var simpleBalance wasmvmtypes.AllBalancesResponse
	mustUnmarshal(t, simpleChain.Data, &simpleBalance)
	require.Equal(t, len(expectedBalance), len(simpleBalance.Amount))
	assert.Equal(t, simpleBalance.Amount[0].Amount, expectedBalance[0].Amount.String())
	assert.Equal(t, simpleBalance.Amount[0].Denom, expectedBalance[0].Denom)
}

func TestReflectGrpcQuery(t *testing.T) {
	queryPlugins := (*reflectPlugins()).Merge(&wasmKeeper.QueryPlugins{
		Grpc: func(ctx sdk.Context, request *wasmvmtypes.GrpcQuery) (proto.Message, error) {
			if request.Path == "cosmos.bank.v1beta1.Query/AllBalances" {
				return &banktypes.QueryAllBalancesResponse{
					Balances: sdk.NewCoins(),
				}, nil
			}
			return nil, errors.New("unsupported request")
		},
	})
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(&queryPlugins))
	keeper := keepers.WasmKeeper

	creator := wasmKeeper.RandomAccountAddress(t)

	// upload code
	codeID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	// creator instantiates a contract
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "reflect contract 1", sdk.NewCoins())
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// now grpc query for the bank balance
	cosmosBankQuery := banktypes.NewQueryAllBalancesRequest(creator, nil, false)
	cosmosBankQueryBz, err := proto.Marshal(cosmosBankQuery)
	require.NoError(t, err)
	reflectQuery := wasmvmtypes.QueryRequest{
		Grpc: &wasmvmtypes.GrpcQuery{
			Path: "cosmos.bank.v1beta1.Query/AllBalances",
			Data: cosmosBankQueryBz,
		},
	}
	reflectQueryBz, err := json.Marshal(testdata.ReflectQueryMsg{
		Chain: &testdata.ChainQuery{Request: &reflectQuery},
	})
	require.NoError(t, err)
	// query the reflect contract
	reflectRespBz, err := keeper.QuerySmart(ctx, contractAddr, reflectQueryBz)
	require.NoError(t, err)
	var reflectResp testdata.ChainResponse
	mustUnmarshal(t, reflectRespBz, &reflectResp)
	// now unmarshal the protobuf response
	var grpcBalance banktypes.QueryAllBalancesResponse
	err = proto.Unmarshal(reflectResp.Data, &grpcBalance)
	require.NoError(t, err)
}

func TestReflectTotalSupplyQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper
	// upload code
	codeID := wasmKeeper.StoreReflectContract(t, ctx, keepers).CodeID
	// creator instantiates a contract and gives it tokens
	creator := wasmKeeper.RandomAccountAddress(t)
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "testing", nil)
	require.NoError(t, err)

	currentStakeSupply := keepers.BankKeeper.GetSupply(ctx, "stake")
	require.NotEmpty(t, currentStakeSupply.Amount) // ensure we have real data
	specs := map[string]struct {
		denom     string
		expAmount wasmvmtypes.Coin
	}{
		"known denom": {
			denom:     "stake",
			expAmount: wasmKeeper.ConvertSdkCoinToWasmCoin(currentStakeSupply),
		},
		"unknown denom": {
			denom:     "unknown",
			expAmount: wasmvmtypes.Coin{Denom: "unknown", Amount: "0"},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			// when
			queryBz := mustMarshal(t, testdata.ReflectQueryMsg{
				Chain: &testdata.ChainQuery{
					Request: &wasmvmtypes.QueryRequest{
						Bank: &wasmvmtypes.BankQuery{
							Supply: &wasmvmtypes.SupplyQuery{Denom: spec.denom},
						},
					},
				},
			})
			simpleRes, err := keeper.QuerySmart(ctx, contractAddr, queryBz)

			// then
			require.NoError(t, err)
			var rsp testdata.ChainResponse
			mustUnmarshal(t, simpleRes, &rsp)
			var supplyRsp wasmvmtypes.SupplyResponse
			mustUnmarshal(t, rsp.Data, &supplyRsp)
			assert.Equal(t, spec.expAmount, supplyRsp.Amount, spec.expAmount)
		})
	}
}

func TestReflectInvalidStargateQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	funds := sdk.NewCoins(sdk.NewInt64Coin("denom", 320000))
	contractStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, funds...)

	// upload code
	codeID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	// creator instantiates a contract and gives it tokens
	contractAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "reflect contract 1", contractStart)
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// now, try to build a protobuf query
	protoQuery := banktypes.QueryAllBalancesRequest{
		Address: creator.String(),
	}
	protoQueryBin, err := proto.Marshal(&protoQuery)
	require.NoError(t, err)

	protoRequest := wasmvmtypes.QueryRequest{
		Stargate: &wasmvmtypes.StargateQuery{
			Path: "/cosmos.bank.v1beta1.Query/AllBalances",
			Data: protoQueryBin,
		},
	}
	protoQueryBz, err := json.Marshal(testdata.ReflectQueryMsg{
		Chain: &testdata.ChainQuery{Request: &protoRequest},
	})
	require.NoError(t, err)

	// make a query on the chain, should not be whitelisted
	_, err = keeper.QuerySmart(ctx, contractAddr, protoQueryBz)
	require.Error(t, err)
	require.Contains(t, err.Error(), "Unsupported query")

	// now, try to build a protobuf query
	protoRequest = wasmvmtypes.QueryRequest{
		Stargate: &wasmvmtypes.StargateQuery{
			Path: "/cosmos.tx.v1beta1.Service/GetTx",
			Data: []byte{},
		},
	}
	protoQueryBz, err = json.Marshal(testdata.ReflectQueryMsg{
		Chain: &testdata.ChainQuery{Request: &protoRequest},
	})
	require.NoError(t, err)

	// make a query on the chain, should be blacklisted
	_, err = keeper.QuerySmart(ctx, contractAddr, protoQueryBz)
	require.Error(t, err)
	require.Contains(t, err.Error(), "Unsupported query")

	// and another one
	protoRequest = wasmvmtypes.QueryRequest{
		Stargate: &wasmvmtypes.StargateQuery{
			Path: "/cosmos.base.tendermint.v1beta1.Service/GetNodeInfo",
			Data: []byte{},
		},
	}
	protoQueryBz, err = json.Marshal(testdata.ReflectQueryMsg{
		Chain: &testdata.ChainQuery{Request: &protoRequest},
	})
	require.NoError(t, err)

	// make a query on the chain, should be blacklisted
	_, err = keeper.QuerySmart(ctx, contractAddr, protoQueryBz)
	require.Error(t, err)
	require.Contains(t, err.Error(), "Unsupported query")
}

type reflectState struct {
	Owner string `json:"owner"`
}

func TestMaskReflectWasmQueries(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	// upload reflect code
	reflectID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), reflectID)

	// creator instantiates a contract and gives it tokens
	reflectStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	reflectAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, reflectID, creator, nil, []byte("{}"), "reflect contract 2", reflectStart)
	require.NoError(t, err)
	require.NotEmpty(t, reflectAddr)

	// for control, let's make some queries directly on the reflect
	ownerQuery := buildReflectQuery(t, &testdata.ReflectQueryMsg{Owner: &struct{}{}})
	res, err := keeper.QuerySmart(ctx, reflectAddr, ownerQuery)
	require.NoError(t, err)
	var ownerRes testdata.OwnerResponse
	mustUnmarshal(t, res, &ownerRes)
	require.Equal(t, ownerRes.Owner, creator.String())

	// and a raw query: cosmwasm_storage::Singleton uses 2 byte big-endian length-prefixed to store data
	configKey := append([]byte{0, 6}, []byte("config")...)
	raw := keeper.QueryRaw(ctx, reflectAddr, configKey)
	var stateRes reflectState
	mustUnmarshal(t, raw, &stateRes)
	require.Equal(t, stateRes.Owner, creator.String())

	// now, let's reflect a smart query into the x/wasm handlers and see if we get the same result
	reflectOwnerQuery := testdata.ReflectQueryMsg{Chain: &testdata.ChainQuery{Request: &wasmvmtypes.QueryRequest{Wasm: &wasmvmtypes.WasmQuery{
		Smart: &wasmvmtypes.SmartQuery{
			ContractAddr: reflectAddr.String(),
			Msg:          ownerQuery,
		},
	}}}}
	reflectOwnerBin := buildReflectQuery(t, &reflectOwnerQuery)
	res, err = keeper.QuerySmart(ctx, reflectAddr, reflectOwnerBin)
	require.NoError(t, err)
	// first we pull out the data from chain response, before parsing the original response
	var reflectRes testdata.ChainResponse
	mustUnmarshal(t, res, &reflectRes)
	var reflectOwnerRes testdata.OwnerResponse
	mustUnmarshal(t, reflectRes.Data, &reflectOwnerRes)
	require.Equal(t, reflectOwnerRes.Owner, creator.String())

	// and with queryRaw
	reflectStateQuery := testdata.ReflectQueryMsg{Chain: &testdata.ChainQuery{Request: &wasmvmtypes.QueryRequest{Wasm: &wasmvmtypes.WasmQuery{
		Raw: &wasmvmtypes.RawQuery{
			ContractAddr: reflectAddr.String(),
			Key:          configKey,
		},
	}}}}
	reflectStateBin := buildReflectQuery(t, &reflectStateQuery)
	res, err = keeper.QuerySmart(ctx, reflectAddr, reflectStateBin)
	require.NoError(t, err)
	// first we pull out the data from chain response, before parsing the original response
	var reflectRawRes testdata.ChainResponse
	mustUnmarshal(t, res, &reflectRawRes)
	// now, with the raw data, we can parse it into state
	var reflectStateRes reflectState
	mustUnmarshal(t, reflectRawRes.Data, &reflectStateRes)
	require.Equal(t, reflectStateRes.Owner, creator.String())
}

func TestWasmRawQueryWithNil(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)

	// upload reflect code
	reflectID, _, err := keepers.ContractKeeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), reflectID)

	// creator instantiates a contract and gives it tokens
	reflectStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	reflectAddr, _, err := keepers.ContractKeeper.Instantiate(ctx, reflectID, creator, nil, []byte("{}"), "reflect contract 2", reflectStart)
	require.NoError(t, err)
	require.NotEmpty(t, reflectAddr)

	// control: query directly
	missingKey := []byte{0, 1, 2, 3, 4}
	raw := keeper.QueryRaw(ctx, reflectAddr, missingKey)
	require.Nil(t, raw)

	// and with queryRaw
	reflectQuery := testdata.ReflectQueryMsg{Chain: &testdata.ChainQuery{Request: &wasmvmtypes.QueryRequest{Wasm: &wasmvmtypes.WasmQuery{
		Raw: &wasmvmtypes.RawQuery{
			ContractAddr: reflectAddr.String(),
			Key:          missingKey,
		},
	}}}}
	reflectStateBin := buildReflectQuery(t, &reflectQuery)
	res, err := keeper.QuerySmart(ctx, reflectAddr, reflectStateBin)
	require.NoError(t, err)

	// first we pull out the data from chain response, before parsing the original response
	var reflectRawRes testdata.ChainResponse
	mustUnmarshal(t, res, &reflectRawRes)
	// and make sure there is no data
	require.Empty(t, reflectRawRes.Data)
	// we get an empty byte slice not nil (if anyone care in go-land)
	require.Equal(t, []byte{}, reflectRawRes.Data)
}

func TestQueryDenomsIntegration(t *testing.T) {
	ctx, keepers := wasmKeeper.CreateTestInput(t, false, CyberpunkCapabilities)
	ck, k := keepers.ContractKeeper, keepers.WasmKeeper
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))...)

	// upload code
	codeID, _, err := ck.Create(ctx, creator, testdata.CyberpunkContractWasm(), nil)
	require.NoError(t, err)

	contractAddr, _, err := ck.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "cyberpunk contract", nil)
	require.NoError(t, err)

	var (
		metadata1 = banktypes.Metadata{
			Description: "testing",
			DenomUnits: []*banktypes.DenomUnit{
				{Denom: "ualx", Exponent: 0, Aliases: []string{"microalx"}},
				{Denom: "alx", Exponent: 6, Aliases: []string{"ALX"}},
			},
			Base:    "ualx",
			Display: "alx",
			Name:    "my test denom",
			Symbol:  "XALX",
			URI:     "https://example.com/ualx",
			URIHash: "my_hash",
		}
		metadata2 = banktypes.Metadata{
			Description: "testing2",
			DenomUnits: []*banktypes.DenomUnit{
				{Denom: "ublx", Exponent: 0, Aliases: []string{"microblx"}},
				{Denom: "blx", Exponent: 6, Aliases: []string{"BLX"}},
			},
			Base:    "ublx",
			Display: "blx",
			Name:    "my other test denom",
			Symbol:  "XBLX",
		}
	)
	type dict map[string]any

	keepers.BankKeeper.SetDenomMetaData(ctx, metadata1)
	keepers.BankKeeper.SetDenomMetaData(ctx, metadata2)

	specs := map[string]struct {
		query  string
		exp    []byte
		expErr *errorsmod.Error
	}{
		"all denoms": {
			query: `{"denoms":{}}`,
			exp: mustMarshal(t, []dict{
				{
					"description": "testing",
					"denom_units": []dict{
						{"denom": "ualx", "exponent": 0, "aliases": []string{"microalx"}},
						{"denom": "alx", "exponent": 6, "aliases": []string{"ALX"}},
					},
					"base":     "ualx",
					"display":  "alx",
					"name":     "my test denom",
					"symbol":   "XALX",
					"uri":      "https://example.com/ualx",
					"uri_hash": "my_hash",
				}, {
					"description": "testing2",
					"denom_units": []dict{
						{"denom": "ublx", "exponent": 0, "aliases": []string{"microblx"}},
						{"denom": "blx", "exponent": 6, "aliases": []string{"BLX"}},
					},
					"base":     "ublx",
					"display":  "blx",
					"name":     "my other test denom",
					"symbol":   "XBLX",
					"uri":      "",
					"uri_hash": "",
				},
			}),
		},
		"single denom": {
			query: `{"denom":{"denom":"ublx"}}`,
			exp: mustMarshal(t, dict{
				"description": "testing2",
				"denom_units": []dict{
					{"denom": "ublx", "exponent": 0, "aliases": []string{"microblx"}},
					{"denom": "blx", "exponent": 6, "aliases": []string{"BLX"}},
				},
				"base":     "ublx",
				"display":  "blx",
				"name":     "my other test denom",
				"symbol":   "XBLX",
				"uri":      "",
				"uri_hash": "",
			}),
		},
		"unknown denom": {
			query:  `{"denom":{"denom":"unknown"}}`,
			expErr: sdkerrors.ErrNotFound,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			gotData, gotErr := k.QuerySmart(ctx, contractAddr, []byte(spec.query))
			if spec.expErr != nil {
				require.Error(t, gotErr)
				assert.Contains(t, gotErr.Error(), fmt.Sprintf("codespace: %s, code: %d:", spec.expErr.Codespace(), spec.expErr.ABCICode()))
				return
			}
			require.NoError(t, gotErr)
			assert.JSONEq(t, string(spec.exp), string(gotData), string(gotData))
		})
	}
}

func TestDistributionQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	pCtx, keepers := wasmKeeper.CreateTestInput(t, false, ReflectCapabilities, wasmKeeper.WithMessageEncoders(reflectEncoders(cdc)), wasmKeeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper

	example := wasmKeeper.InstantiateReflectExampleContract(t, pCtx, keepers)
	delegator := keepers.Faucet.NewFundedRandomAccount(pCtx, sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 100_000_000))...)
	otherAddr := keepers.Faucet.NewFundedRandomAccount(pCtx, sdk.NewCoins(sdk.NewInt64Coin(sdk.DefaultBondDenom, 100_000_000))...)

	val1Addr := addValidator(t, pCtx, keepers.StakingKeeper, keepers.Faucet, sdk.NewInt64Coin(sdk.DefaultBondDenom, 10_000_000))
	val2Addr := addValidator(t, pCtx, keepers.StakingKeeper, keepers.Faucet, sdk.NewInt64Coin(sdk.DefaultBondDenom, 20_000_000))
	_ = val2Addr
	pCtx = nextBlock(pCtx, keepers.StakingKeeper)

	noopSetup := func(t *testing.T, ctx sdk.Context) sdk.Context { return ctx }
	specs := map[string]struct {
		setup  func(t *testing.T, ctx sdk.Context) sdk.Context
		query  *wasmvmtypes.DistributionQuery
		expErr bool
		assert func(t *testing.T, d []byte)
	}{
		"delegator address - no withdrawal addr set": {
			setup: noopSetup,
			query: &wasmvmtypes.DistributionQuery{
				DelegatorWithdrawAddress: &wasmvmtypes.DelegatorWithdrawAddressQuery{DelegatorAddress: delegator.String()},
			},
			assert: func(t *testing.T, d []byte) {
				rsp := unmarshalReflect[wasmvmtypes.DelegatorWithdrawAddressResponse](t, d)
				assert.Equal(t, delegator.String(), rsp.WithdrawAddress)
			},
		},
		"delegator address -  withdrawal addr set": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				require.NoError(t, keepers.DistKeeper.SetWithdrawAddr(ctx, delegator, otherAddr))
				return ctx
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegatorWithdrawAddress: &wasmvmtypes.DelegatorWithdrawAddressQuery{DelegatorAddress: delegator.String()},
			},
			assert: func(t *testing.T, d []byte) {
				var rsp wasmvmtypes.DelegatorWithdrawAddressResponse
				mustUnmarshal(t, d, &rsp)
				assert.Equal(t, otherAddr.String(), rsp.WithdrawAddress)
			},
		},
		"delegator address - empty": {
			setup: noopSetup,
			query: &wasmvmtypes.DistributionQuery{
				DelegatorWithdrawAddress: &wasmvmtypes.DelegatorWithdrawAddressQuery{},
			},
			expErr: true,
		},
		"delegation rewards - existing delegation": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				val1, err := keepers.StakingKeeper.GetValidator(ctx, val1Addr)
				require.NoError(t, err)
				_, err = keepers.StakingKeeper.Delegate(ctx, delegator, sdkmath.NewInt(10_000_000), stakingtypes.Unbonded, val1, true)
				require.NoError(t, err)
				setValidatorRewards(ctx, keepers.StakingKeeper, keepers.DistKeeper, val1Addr, "100000000")
				return nextBlock(ctx, keepers.StakingKeeper)
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegationRewards: &wasmvmtypes.DelegationRewardsQuery{DelegatorAddress: delegator.String(), ValidatorAddress: val1Addr.String()},
			},
			assert: func(t *testing.T, d []byte) {
				var rsp wasmvmtypes.DelegationRewardsResponse
				mustUnmarshal(t, d, &rsp)
				expRewards := []wasmvmtypes.DecCoin{{Amount: "45000000.000000000000000000", Denom: "stake"}}
				assert.Equal(t, expRewards, rsp.Rewards)
			},
		},
		"delegation rewards - no delegation": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				setValidatorRewards(ctx, keepers.StakingKeeper, keepers.DistKeeper, val1Addr, "100000000")
				return nextBlock(ctx, keepers.StakingKeeper)
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegationRewards: &wasmvmtypes.DelegationRewardsQuery{DelegatorAddress: delegator.String(), ValidatorAddress: val1Addr.String()},
			},
			expErr: true,
		},
		"delegation rewards - validator empty": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				val, err := keepers.StakingKeeper.GetValidator(ctx, val1Addr)
				require.NoError(t, err)
				_, err = keepers.StakingKeeper.Delegate(ctx, delegator, sdkmath.NewInt(10_000_000), stakingtypes.Unbonded, val, true)
				require.NoError(t, err)
				return ctx
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegationRewards: &wasmvmtypes.DelegationRewardsQuery{DelegatorAddress: delegator.String()},
			},
			expErr: true,
		},
		"delegation total rewards": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				val, err := keepers.StakingKeeper.GetValidator(ctx, val1Addr)
				require.NoError(t, err)
				_, err = keepers.StakingKeeper.Delegate(ctx, delegator, sdkmath.NewInt(10_000_000), stakingtypes.Unbonded, val, true)
				require.NoError(t, err)
				setValidatorRewards(ctx, keepers.StakingKeeper, keepers.DistKeeper, val1Addr, "100000000")
				return nextBlock(ctx, keepers.StakingKeeper)
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegationTotalRewards: &wasmvmtypes.DelegationTotalRewardsQuery{DelegatorAddress: delegator.String()},
			},
			assert: func(t *testing.T, d []byte) {
				var rsp wasmvmtypes.DelegationTotalRewardsResponse
				mustUnmarshal(t, d, &rsp)
				expRewards := []wasmvmtypes.DelegatorReward{
					{
						Reward:           []wasmvmtypes.DecCoin{{Amount: "45000000.000000000000000000", Denom: "stake"}},
						ValidatorAddress: val1Addr.String(),
					},
				}
				assert.Equal(t, expRewards, rsp.Rewards)
				expTotal := []wasmvmtypes.DecCoin{{Amount: "45000000.000000000000000000", Denom: "stake"}}
				assert.Equal(t, expTotal, rsp.Total)
			},
		},
		"delegator validators": {
			setup: func(t *testing.T, ctx sdk.Context) sdk.Context {
				for _, v := range []sdk.ValAddress{val1Addr, val2Addr} {
					val, err := keepers.StakingKeeper.GetValidator(ctx, v)
					require.NoError(t, err)
					_, err = keepers.StakingKeeper.Delegate(ctx, delegator, sdkmath.NewInt(10_000_000), stakingtypes.Unbonded, val, true)
					require.NoError(t, err)
				}
				return ctx
			},
			query: &wasmvmtypes.DistributionQuery{
				DelegatorValidators: &wasmvmtypes.DelegatorValidatorsQuery{DelegatorAddress: delegator.String()},
			},
			assert: func(t *testing.T, d []byte) {
				var rsp wasmvmtypes.DelegatorValidatorsResponse
				mustUnmarshal(t, d, &rsp)
				expVals := []string{val1Addr.String(), val2Addr.String()}
				if bytes.Compare(val1Addr, val2Addr) > 0 {
					expVals = []string{expVals[1], expVals[0]}
				}
				assert.Equal(t, expVals, rsp.Validators)
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := pCtx.CacheContext()
			ctx = spec.setup(t, ctx)

			// when
			queryBz := mustMarshal(t, testdata.ReflectQueryMsg{
				Chain: &testdata.ChainQuery{
					Request: &wasmvmtypes.QueryRequest{Distribution: spec.query},
				},
			})
			simpleRes, gotErr := keeper.QuerySmart(ctx, example.Contract, queryBz)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			// then
			require.NoError(t, gotErr)
			var rsp testdata.ChainResponse
			mustUnmarshal(t, simpleRes, &rsp)
			spec.assert(t, rsp.Data)
		})
	}
}

func TestIBCListChannelsQuery(t *testing.T) {
	cdc := wasmKeeper.MakeEncodingConfig(t).Codec
	pCtx, keepers := keeper.CreateTestInput(t, false, ReflectCapabilities, keeper.WithMessageEncoders(reflectEncoders(cdc)), keeper.WithQueryPlugins(reflectPlugins()))
	keeper := keepers.WasmKeeper
	nonIbcExample := wasmKeeper.InstantiateReflectExampleContract(t, pCtx, keepers)
	// add an ibc port for testing
	myIBCPortID := "myValidPortID"
	ibcExample := wasmKeeper.InstantiateReflectExampleContractWithPortID(t, pCtx, keepers, myIBCPortID)

	// store a random channel to be ignored in queries
	unusedChan := channeltypes.Channel{
		State:    channeltypes.OPEN,
		Ordering: channeltypes.UNORDERED,
		Counterparty: channeltypes.Counterparty{
			PortId:    "counterPartyPortID",
			ChannelId: "counterPartyChannelID",
		},
		ConnectionHops: []string{"any"},
		Version:        "any",
	}
	keepers.IBCKeeper.ChannelKeeper.SetChannel(pCtx, "nonContractPortID", "channel-99", unusedChan)

	// mixed channel examples for testing
	myExampleChannels := []channeltypes.Channel{
		{
			State:    channeltypes.OPEN,
			Ordering: channeltypes.ORDERED,
			Counterparty: channeltypes.Counterparty{
				PortId:    "counterPartyPortID",
				ChannelId: "counterPartyChannelID",
			},
			ConnectionHops: []string{"one"},
			Version:        "v1",
		},
		{
			State:    channeltypes.INIT,
			Ordering: channeltypes.UNORDERED,
			Counterparty: channeltypes.Counterparty{
				PortId: "foobar",
			},
			ConnectionHops: []string{"one"},
			Version:        "initversion",
		},
		{
			State:    channeltypes.OPEN,
			Ordering: channeltypes.UNORDERED,
			Counterparty: channeltypes.Counterparty{
				PortId:    "otherCounterPartyPortID",
				ChannelId: "otherCounterPartyChannelID",
			},
			ConnectionHops: []string{"other", "second"},
			Version:        "otherVersion",
		},
		{
			State:    channeltypes.CLOSED,
			Ordering: channeltypes.ORDERED,
			Counterparty: channeltypes.Counterparty{
				PortId:    "super",
				ChannelId: "duper",
			},
			ConnectionHops: []string{"no-more"},
			Version:        "closedVersion",
		},
	}

	withChannelsStored := func(portID string, channels ...channeltypes.Channel) func(t *testing.T, ctx sdk.Context) sdk.Context {
		return func(t *testing.T, ctx sdk.Context) sdk.Context {
			for i, v := range channels {
				keepers.IBCKeeper.ChannelKeeper.SetChannel(ctx, portID, fmt.Sprintf("channel-%d", i), v)
			}
			return ctx
		}
	}
	noopSetup := func(t *testing.T, ctx sdk.Context) sdk.Context { return ctx }

	specs := map[string]struct {
		setup    func(t *testing.T, ctx sdk.Context) sdk.Context
		contract sdk.AccAddress
		query    *wasmvmtypes.IBCQuery
		expErr   bool
		assert   func(t *testing.T, d []byte)
	}{
		"open channels - with query portID empty": {
			contract: ibcExample.Contract,
			setup:    withChannelsStored(myIBCPortID, myExampleChannels...),
			query:    &wasmvmtypes.IBCQuery{ListChannels: &wasmvmtypes.ListChannelsQuery{}},
			assert: func(t *testing.T, d []byte) {
				rsp := unmarshalReflect[wasmvmtypes.ListChannelsResponse](t, d)
				exp := wasmvmtypes.ListChannelsResponse{Channels: []wasmvmtypes.IBCChannel{
					{
						Endpoint: wasmvmtypes.IBCEndpoint{PortID: myIBCPortID, ChannelID: "channel-0"},
						CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
							PortID:    "counterPartyPortID",
							ChannelID: "counterPartyChannelID",
						},
						Order:        channeltypes.ORDERED.String(),
						Version:      "v1",
						ConnectionID: "one",
					}, {
						Endpoint: wasmvmtypes.IBCEndpoint{PortID: myIBCPortID, ChannelID: "channel-2"},
						CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
							PortID:    "otherCounterPartyPortID",
							ChannelID: "otherCounterPartyChannelID",
						},
						Order:        channeltypes.UNORDERED.String(),
						Version:      "otherVersion",
						ConnectionID: "other",
					},
				}}
				assert.Equal(t, exp, rsp)
			},
		},
		"open channels - with query portID passed": {
			contract: ibcExample.Contract,
			setup:    withChannelsStored("OtherPortID", myExampleChannels...),
			query:    &wasmvmtypes.IBCQuery{ListChannels: &wasmvmtypes.ListChannelsQuery{PortID: "OtherPortID"}},
			assert: func(t *testing.T, d []byte) {
				rsp := unmarshalReflect[wasmvmtypes.ListChannelsResponse](t, d)
				exp := wasmvmtypes.ListChannelsResponse{Channels: []wasmvmtypes.IBCChannel{
					{
						Endpoint: wasmvmtypes.IBCEndpoint{PortID: "OtherPortID", ChannelID: "channel-0"},
						CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
							PortID:    "counterPartyPortID",
							ChannelID: "counterPartyChannelID",
						},
						Order:        channeltypes.ORDERED.String(),
						Version:      "v1",
						ConnectionID: "one",
					}, {
						Endpoint: wasmvmtypes.IBCEndpoint{PortID: "OtherPortID", ChannelID: "channel-2"},
						CounterpartyEndpoint: wasmvmtypes.IBCEndpoint{
							PortID:    "otherCounterPartyPortID",
							ChannelID: "otherCounterPartyChannelID",
						},
						Order:        channeltypes.UNORDERED.String(),
						Version:      "otherVersion",
						ConnectionID: "other",
					},
				}}
				assert.Equal(t, exp, rsp)
			},
		},
		"non ibc contract - with query portID empty": {
			contract: nonIbcExample.Contract,
			setup:    withChannelsStored(myIBCPortID, myExampleChannels...),
			query:    &wasmvmtypes.IBCQuery{ListChannels: &wasmvmtypes.ListChannelsQuery{}},
			assert: func(t *testing.T, d []byte) {
				rsp := unmarshalReflect[wasmvmtypes.ListChannelsResponse](t, d)
				assert.Empty(t, rsp.Channels)
			},
		},
		"no matching channels": {
			contract: ibcExample.Contract,
			setup:    noopSetup,
			query:    &wasmvmtypes.IBCQuery{ListChannels: &wasmvmtypes.ListChannelsQuery{}},
			assert: func(t *testing.T, d []byte) {
				rsp := unmarshalReflect[wasmvmtypes.ListChannelsResponse](t, d)
				assert.Empty(t, rsp.Channels)
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx, _ := pCtx.CacheContext()
			ctx = spec.setup(t, ctx)

			// when
			queryBz := mustMarshal(t, testdata.ReflectQueryMsg{
				Chain: &testdata.ChainQuery{
					Request: &wasmvmtypes.QueryRequest{IBC: spec.query},
				},
			})
			simpleRes, gotErr := keeper.QuerySmart(ctx, spec.contract, queryBz)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			// then
			require.NoError(t, gotErr)
			var rsp testdata.ChainResponse
			mustUnmarshal(t, simpleRes, &rsp)
			spec.assert(t, rsp.Data)
		})
	}
}

func unmarshalReflect[T any](t *testing.T, d []byte) T {
	var v T
	mustUnmarshal(t, d, &v)
	return v
}

type reflectCustomQuery struct {
	Ping        *struct{}      `json:"ping,omitempty"`
	Capitalized *testdata.Text `json:"capitalized,omitempty"`
}

// this is from the go code back to the contract (capitalized or ping)
type customQueryResponse struct {
	Msg string `json:"msg"`
}

// this is from the contract to the go code (capitalized or ping)
type capitalizedResponse struct {
	Text string `json:"text"`
}

// reflectPlugins needs to be registered in test setup to handle custom query callbacks
func reflectPlugins() *wasmKeeper.QueryPlugins {
	return &wasmKeeper.QueryPlugins{
		Custom: performCustomQuery,
	}
}

func performCustomQuery(_ sdk.Context, request json.RawMessage) ([]byte, error) {
	var custom reflectCustomQuery
	err := json.Unmarshal(request, &custom)
	if err != nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrJSONUnmarshal, err.Error())
	}
	if custom.Capitalized != nil {
		msg := strings.ToUpper(custom.Capitalized.Text)
		return json.Marshal(customQueryResponse{Msg: msg})
	}
	if custom.Ping != nil {
		return json.Marshal(customQueryResponse{Msg: "pong"})
	}
	return nil, errorsmod.Wrap(types.ErrInvalidMsg, "Unknown Custom query variant")
}

func buildReflectQuery(t *testing.T, query *testdata.ReflectQueryMsg) []byte {
	t.Helper()
	bz, err := json.Marshal(query)
	require.NoError(t, err)
	return bz
}

func TestAcceptListStargateQuerier(t *testing.T) {
	wasmApp := app.SetupWithEmptyStore(t)
	ctx := wasmApp.NewUncachedContext(false, cmtproto.Header{ChainID: "foo", Height: 1, Time: time.Now()})
	err := wasmApp.StakingKeeper.SetParams(ctx, stakingtypes.DefaultParams())
	require.NoError(t, err)

	addrs := app.AddTestAddrsIncremental(wasmApp, ctx, 2, sdkmath.NewInt(1_000_000))
	accepted := wasmKeeper.AcceptedQueries{
		"/cosmos.auth.v1beta1.Query/Account": &authtypes.QueryAccountResponse{},
		"/no/route/to/this":                  &authtypes.QueryAccountResponse{},
	}

	marshal := func(pb proto.Message) []byte {
		b, err := proto.Marshal(pb)
		require.NoError(t, err)
		return b
	}

	specs := map[string]struct {
		req     *wasmvmtypes.StargateQuery
		expErr  bool
		expResp string
	}{
		"in accept list - success result": {
			req: &wasmvmtypes.StargateQuery{
				Path: "/cosmos.auth.v1beta1.Query/Account",
				Data: marshal(&authtypes.QueryAccountRequest{Address: addrs[0].String()}),
			},
			expResp: fmt.Sprintf(`{"account":{"@type":"/cosmos.auth.v1beta1.BaseAccount","address":%q,"pub_key":null,"account_number":"1","sequence":"0"}}`, addrs[0].String()),
		},
		"in accept list - error result": {
			req: &wasmvmtypes.StargateQuery{
				Path: "/cosmos.auth.v1beta1.Query/Account",
				Data: marshal(&authtypes.QueryAccountRequest{Address: sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address()).String()}),
			},
			expErr: true,
		},
		"not in accept list": {
			req: &wasmvmtypes.StargateQuery{
				Path: "/cosmos.bank.v1beta1.Query/AllBalances",
				Data: marshal(&banktypes.QueryAllBalancesRequest{Address: addrs[0].String()}),
			},
			expErr: true,
		},
		"unknown route": {
			req: &wasmvmtypes.StargateQuery{
				Path: "/no/route/to/this",
				Data: marshal(&banktypes.QueryAllBalancesRequest{Address: addrs[0].String()}),
			},
			expErr: true,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			q := wasmKeeper.AcceptListStargateQuerier(accepted, wasmApp.GRPCQueryRouter(), wasmApp.AppCodec())
			gotBz, gotErr := q(ctx, spec.req)
			if spec.expErr {
				require.Error(t, gotErr)
				return
			}
			require.NoError(t, gotErr)
			assert.JSONEq(t, spec.expResp, string(gotBz), string(gotBz))
		})
	}
}

func TestResetProtoMarshalerAfterJsonMarshal(t *testing.T) {
	appCodec := app.MakeEncodingConfig(t).Codec

	protoMarshaler := &banktypes.QueryAllBalancesResponse{}
	expected := appCodec.MustMarshalJSON(&banktypes.QueryAllBalancesResponse{
		Balances: sdk.NewCoins(sdk.NewCoin("bar", sdkmath.NewInt(30))),
		Pagination: &query.PageResponse{
			NextKey: []byte("foo"),
		},
	})

	bz, err := hex.DecodeString("0a090a036261721202333012050a03666f6f")
	require.NoError(t, err)

	// first marshal
	response, err := wasmKeeper.ConvertProtoToJSONMarshal(appCodec, protoMarshaler, bz)
	require.NoError(t, err)
	require.Equal(t, expected, response)

	// second marshal
	response, err = wasmKeeper.ConvertProtoToJSONMarshal(appCodec, protoMarshaler, bz)
	require.NoError(t, err)
	require.Equal(t, expected, response)
}

// TestDeterministicJsonMarshal tests that we get deterministic JSON marshaled response upon
// proto struct update in the state machine.
func TestDeterministicJsonMarshal(t *testing.T) {
	testCases := []struct {
		name                string
		originalResponse    string
		updatedResponse     string
		queryPath           string
		responseProtoStruct proto.Message
		expectedProto       func() proto.Message
	}{
		/**
		   *
		   * Origin Response
		   * 0a530a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74122f0a2d636f736d6f7331346c3268686a6e676c3939367772703935673867646a6871653038326375367a7732706c686b
		   *
		   * Updated Response
		   * 0a530a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74122f0a2d636f736d6f7331646a783375676866736d6b6135386676673076616a6e6533766c72776b7a6a346e6377747271122d636f736d6f7331646a783375676866736d6b6135386676673076616a6e6533766c72776b7a6a346e6377747271
		  // Origin proto
		  message QueryAccountResponse {
		    // account defines the account of the corresponding address.
		    google.protobuf.Any account = 1 [(cosmos_proto.accepts_interface) = "AccountI"];
		  }
		  // Updated proto
		  message QueryAccountResponse {
		    // account defines the account of the corresponding address.
		    google.protobuf.Any account = 1 [(cosmos_proto.accepts_interface) = "AccountI"];
		    // address is the address to query for.
		  	string address = 2;
		  }
		*/
		{
			"Query Account",
			"0a530a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74122f0a2d636f736d6f733166387578756c746e3873717a687a6e72737a3371373778776171756867727367366a79766679",
			"0a530a202f636f736d6f732e617574682e763162657461312e426173654163636f756e74122f0a2d636f736d6f733166387578756c746e3873717a687a6e72737a3371373778776171756867727367366a79766679122d636f736d6f733166387578756c746e3873717a687a6e72737a3371373778776171756867727367366a79766679",
			"/cosmos.auth.v1beta1.Query/Account",
			&authtypes.QueryAccountResponse{},
			func() proto.Message {
				account := authtypes.BaseAccount{
					Address: "cosmos1f8uxultn8sqzhznrsz3q77xwaquhgrsg6jyvfy",
				}
				accountResponse, err := codectypes.NewAnyWithValue(&account)
				require.NoError(t, err)
				return &authtypes.QueryAccountResponse{
					Account: accountResponse,
				}
			},
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.name), func(t *testing.T) {
			appCodec := app.MakeEncodingConfig(t).Codec

			originVersionBz, err := hex.DecodeString(tc.originalResponse)
			require.NoError(t, err)
			jsonMarshalledOriginalBz, err := wasmKeeper.ConvertProtoToJSONMarshal(appCodec, tc.responseProtoStruct, originVersionBz)
			require.NoError(t, err)

			newVersionBz, err := hex.DecodeString(tc.updatedResponse)
			require.NoError(t, err)
			jsonMarshalledUpdatedBz, err := wasmKeeper.ConvertProtoToJSONMarshal(appCodec, tc.responseProtoStruct, newVersionBz)
			require.NoError(t, err)

			// json marshaled bytes should be the same since we use the same proto struct for unmarshalling
			require.Equal(t, jsonMarshalledOriginalBz, jsonMarshalledUpdatedBz)

			// raw build also make same result
			jsonMarshalExpectedResponse, err := appCodec.MarshalJSON(tc.expectedProto())
			require.NoError(t, err)
			require.Equal(t, jsonMarshalledUpdatedBz, jsonMarshalExpectedResponse)
		})
	}
}

func TestConvertProtoToJSONMarshal(t *testing.T) {
	testCases := []struct {
		name                  string
		queryPath             string
		protoResponseStruct   proto.Message
		originalResponse      string
		expectedProtoResponse proto.Message
		expectedError         bool
	}{
		{
			name:                "successful conversion from proto response to json marshaled response",
			queryPath:           "/cosmos.bank.v1beta1.Query/AllBalances",
			originalResponse:    "0a090a036261721202333012050a03666f6f",
			protoResponseStruct: &banktypes.QueryAllBalancesResponse{},
			expectedProtoResponse: &banktypes.QueryAllBalancesResponse{
				Balances: sdk.NewCoins(sdk.NewCoin("bar", sdkmath.NewInt(30))),
				Pagination: &query.PageResponse{
					NextKey: []byte("foo"),
				},
			},
		},
		{
			name:                "invalid proto response struct",
			queryPath:           "/cosmos.bank.v1beta1.Query/AllBalances",
			originalResponse:    "0a090a036261721202333012050a03666f6f",
			protoResponseStruct: &authtypes.QueryAccountResponse{},
			expectedError:       true,
		},
	}

	for _, tc := range testCases {
		t.Run(fmt.Sprintf("Case %s", tc.name), func(t *testing.T) {
			originalVersionBz, err := hex.DecodeString(tc.originalResponse)
			require.NoError(t, err)
			appCodec := app.MakeEncodingConfig(t).Codec

			jsonMarshalledResponse, err := wasmKeeper.ConvertProtoToJSONMarshal(appCodec, tc.protoResponseStruct, originalVersionBz)
			if tc.expectedError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)

			// check response by json marshaling proto response into json response manually
			jsonMarshalExpectedResponse, err := appCodec.MarshalJSON(tc.expectedProtoResponse)
			require.NoError(t, err)
			require.JSONEq(t, string(jsonMarshalledResponse), string(jsonMarshalExpectedResponse))
		})
	}
}
