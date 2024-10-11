package keeper

import (
	"encoding/json"
	"os"
	"strings"
	"testing"

	wasmvmtypes "github.com/CosmWasm/wasmvm/v2/types"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	authkeeper "github.com/cosmos/cosmos-sdk/x/auth/keeper"
	bankkeeper "cosmossdk.io/x/bank/keeper"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper/testdata"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

var (
	CyberpunkCapabilities = []string{"staking", "mask", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3", "cosmwasm_1_4"}
	ReflectCapabilities   = []string{"staking", "mask", "stargate", "cosmwasm_1_1", "cosmwasm_1_2", "cosmwasm_1_3", "cosmwasm_1_4", "cosmwasm_2_0"}
)

func mustUnmarshal(t *testing.T, data []byte, res interface{}) {
	t.Helper()
	err := json.Unmarshal(data, res)
	require.NoError(t, err)
}

func TestReflectContractSend(t *testing.T) {
	cdc := MakeEncodingConfig(t).Codec
	ctx, keepers := CreateTestInput(t, false, ReflectCapabilities, WithMessageEncoders(reflectEncoders(cdc)))
	accKeeper, keeper, bankKeeper := keepers.AccountKeeper, keepers.ContractKeeper, keepers.BankKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	_, bob := keyPubAddr()

	// upload reflect code
	reflectID, _, err := keeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), reflectID)

	// upload hackatom escrow code
	escrowCode, err := os.ReadFile("./testdata/hackatom.wasm")
	require.NoError(t, err)
	escrowID, _, err := keeper.Create(ctx, creator, escrowCode, nil)
	require.NoError(t, err)
	require.Equal(t, uint64(2), escrowID)

	// creator instantiates a contract and gives it tokens
	reflectStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	reflectAddr, _, err := keeper.Instantiate(ctx, reflectID, creator, nil, []byte("{}"), "reflect contract 2", reflectStart)
	require.NoError(t, err)
	require.NotEmpty(t, reflectAddr)

	// now we set contract as verifier of an escrow
	initMsg := HackatomExampleInitMsg{
		Verifier:    reflectAddr,
		Beneficiary: bob,
	}
	initMsgBz, err := json.Marshal(initMsg)
	require.NoError(t, err)
	escrowStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 25000))
	escrowAddr, _, err := keeper.Instantiate(ctx, escrowID, creator, nil, initMsgBz, "escrow contract 2", escrowStart)
	require.NoError(t, err)
	require.NotEmpty(t, escrowAddr)

	// let's make sure all balances make sense
	checkAccount(t, ctx, accKeeper, bankKeeper, creator, sdk.NewCoins(sdk.NewInt64Coin("denom", 35000))) // 100k - 40k - 25k
	checkAccount(t, ctx, accKeeper, bankKeeper, reflectAddr, reflectStart)
	checkAccount(t, ctx, accKeeper, bankKeeper, escrowAddr, escrowStart)
	checkAccount(t, ctx, accKeeper, bankKeeper, bob, nil)

	// now for the trick.... we reflect a message through the reflect to call the escrow
	// we also send an additional 14k tokens there.
	// this should reduce the reflect balance by 14k (to 26k)
	// this 14k is added to the escrow, then the entire balance is sent to bob (total: 39k)
	approveMsg := []byte(`{"release":{}}`)
	msgs := []wasmvmtypes.CosmosMsg{{
		Wasm: &wasmvmtypes.WasmMsg{
			Execute: &wasmvmtypes.ExecuteMsg{
				ContractAddr: escrowAddr.String(),
				Msg:          approveMsg,
				Funds: []wasmvmtypes.Coin{{
					Denom:  "denom",
					Amount: "14000",
				}},
			},
		},
	}}
	reflectSend := testdata.ReflectHandleMsg{
		Reflect: &testdata.ReflectPayload{
			Msgs: msgs,
		},
	}
	reflectSendBz, err := json.Marshal(reflectSend)
	require.NoError(t, err)
	_, err = keeper.Execute(ctx, reflectAddr, creator, reflectSendBz, nil)
	require.NoError(t, err)

	// did this work???
	checkAccount(t, ctx, accKeeper, bankKeeper, creator, sdk.NewCoins(sdk.NewInt64Coin("denom", 35000)))     // same as before
	checkAccount(t, ctx, accKeeper, bankKeeper, reflectAddr, sdk.NewCoins(sdk.NewInt64Coin("denom", 26000))) // 40k - 14k (from send)
	checkAccount(t, ctx, accKeeper, bankKeeper, escrowAddr, sdk.Coins{})                                     // emptied reserved
	checkAccount(t, ctx, accKeeper, bankKeeper, bob, sdk.NewCoins(sdk.NewInt64Coin("denom", 39000)))         // all escrow of 25k + 14k
}

func TestReflectCustomMsg(t *testing.T) {
	cdc := MakeEncodingConfig(t).Codec
	ctx, keepers := CreateTestInput(t, false, ReflectCapabilities, WithMessageEncoders(reflectEncoders(cdc)), WithQueryPlugins(reflectPlugins()))
	accKeeper, keeper, bankKeeper := keepers.AccountKeeper, keepers.ContractKeeper, keepers.BankKeeper

	deposit := sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))
	creator := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	bob := keepers.Faucet.NewFundedRandomAccount(ctx, deposit...)
	_, fred := keyPubAddr()

	// upload code
	codeID, _, err := keeper.Create(ctx, creator, testdata.ReflectContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	// creator instantiates a contract and gives it tokens
	contractStart := sdk.NewCoins(sdk.NewInt64Coin("denom", 40000))
	contractAddr, _, err := keeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "reflect contract 1", contractStart)
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// set owner to bob
	transfer := testdata.ReflectHandleMsg{
		ChangeOwner: &testdata.OwnerPayload{
			Owner: bob,
		},
	}
	transferBz, err := json.Marshal(transfer)
	require.NoError(t, err)
	_, err = keeper.Execute(ctx, contractAddr, creator, transferBz, nil)
	require.NoError(t, err)

	// check some account values
	checkAccount(t, ctx, accKeeper, bankKeeper, contractAddr, contractStart)
	checkAccount(t, ctx, accKeeper, bankKeeper, bob, deposit)
	checkAccount(t, ctx, accKeeper, bankKeeper, fred, nil)

	// bob can send contract's tokens to fred (using SendMsg)
	msgs := []wasmvmtypes.CosmosMsg{{
		Bank: &wasmvmtypes.BankMsg{
			Send: &wasmvmtypes.SendMsg{
				ToAddress: fred.String(),
				Amount: []wasmvmtypes.Coin{{
					Denom:  "denom",
					Amount: "15000",
				}},
			},
		},
	}}
	reflectSend := testdata.ReflectHandleMsg{
		Reflect: &testdata.ReflectPayload{
			Msgs: msgs,
		},
	}
	reflectSendBz, err := json.Marshal(reflectSend)
	require.NoError(t, err)
	_, err = keeper.Execute(ctx, contractAddr, bob, reflectSendBz, nil)
	require.NoError(t, err)

	// fred got coins
	checkAccount(t, ctx, accKeeper, bankKeeper, fred, sdk.NewCoins(sdk.NewInt64Coin("denom", 15000)))
	// contract lost them
	checkAccount(t, ctx, accKeeper, bankKeeper, contractAddr, sdk.NewCoins(sdk.NewInt64Coin("denom", 25000)))
	checkAccount(t, ctx, accKeeper, bankKeeper, bob, deposit)

	// construct an opaque message
	var sdkSendMsg sdk.Msg = &banktypes.MsgSend{
		FromAddress: contractAddr.String(),
		ToAddress:   fred.String(),
		Amount:      sdk.NewCoins(sdk.NewInt64Coin("denom", 23000)),
	}
	opaque, err := toReflectRawMsg(cdc, sdkSendMsg)
	require.NoError(t, err)
	reflectOpaque := testdata.ReflectHandleMsg{
		Reflect: &testdata.ReflectPayload{
			Msgs: []wasmvmtypes.CosmosMsg{opaque},
		},
	}
	reflectOpaqueBz, err := json.Marshal(reflectOpaque)
	require.NoError(t, err)

	_, err = keeper.Execute(ctx, contractAddr, bob, reflectOpaqueBz, nil)
	require.NoError(t, err)

	// fred got more coins
	checkAccount(t, ctx, accKeeper, bankKeeper, fred, sdk.NewCoins(sdk.NewInt64Coin("denom", 38000)))
	// contract lost them
	checkAccount(t, ctx, accKeeper, bankKeeper, contractAddr, sdk.NewCoins(sdk.NewInt64Coin("denom", 2000)))
	checkAccount(t, ctx, accKeeper, bankKeeper, bob, deposit)
}

func TestRustPanicIsHandled(t *testing.T) {
	ctx, keepers := CreateTestInput(t, false, CyberpunkCapabilities)
	keeper := keepers.ContractKeeper

	creator := keepers.Faucet.NewFundedRandomAccount(ctx, sdk.NewCoins(sdk.NewInt64Coin("denom", 100000))...)

	// upload code
	codeID, _, err := keeper.Create(ctx, creator, testdata.CyberpunkContractWasm(), nil)
	require.NoError(t, err)
	require.Equal(t, uint64(1), codeID)

	contractAddr, _, err := keeper.Instantiate(ctx, codeID, creator, nil, []byte("{}"), "cyberpunk contract", nil)
	require.NoError(t, err)
	require.NotEmpty(t, contractAddr)

	// when panic is triggered
	msg := []byte(`{"panic":{}}`)
	gotData, err := keeper.Execute(ctx, contractAddr, creator, msg, nil)
	require.ErrorIs(t, err, types.ErrVMError)
	assert.Contains(t, err.Error(), "panicked at 'This page intentionally faulted'")
	assert.Nil(t, gotData)
}

func checkAccount(t *testing.T, ctx sdk.Context, accKeeper authkeeper.AccountKeeper, bankKeeper bankkeeper.Keeper, addr sdk.AccAddress, expected sdk.Coins) {
	acct := accKeeper.GetAccount(ctx, addr)
	if expected == nil {
		assert.Nil(t, acct)
	} else {
		assert.NotNil(t, acct)
		if expected.Empty() {
			// there is confusion between nil and empty slice... let's just treat them the same
			assert.True(t, bankKeeper.GetAllBalances(ctx, acct.GetAddress()).Empty())
		} else {
			assert.Equal(t, bankKeeper.GetAllBalances(ctx, acct.GetAddress()), expected)
		}
	}
}

/**** Code to support custom messages *****/

type reflectCustomMsg struct {
	Debug string `json:"debug,omitempty"`
	Raw   []byte `json:"raw,omitempty"`
}

// toReflectRawMsg encodes an sdk msg using any type with json encoding.
// Then wraps it as an opaque message
func toReflectRawMsg(cdc codec.Codec, msg sdk.Msg) (wasmvmtypes.CosmosMsg, error) {
	codecAny, err := codectypes.NewAnyWithValue(msg)
	if err != nil {
		return wasmvmtypes.CosmosMsg{}, err
	}
	rawBz, err := cdc.MarshalJSON(codecAny)
	if err != nil {
		return wasmvmtypes.CosmosMsg{}, errorsmod.Wrap(sdkerrors.ErrJSONMarshal, err.Error())
	}
	customMsg, _ := json.Marshal(reflectCustomMsg{
		Raw: rawBz,
	})
	res := wasmvmtypes.CosmosMsg{
		Custom: customMsg,
	}
	return res, err
}

// reflectEncoders needs to be registered in test setup to handle custom message callbacks
func reflectEncoders(cdc codec.Codec) *MessageEncoders {
	return &MessageEncoders{
		Custom: fromReflectRawMsg(cdc),
	}
}

// fromReflectRawMsg decodes msg.Data to an sdk.Msg using proto Any and json encoding.
// this needs to be registered on the Encoders
func fromReflectRawMsg(cdc codec.Codec) CustomEncoder {
	return func(_sender sdk.AccAddress, msg json.RawMessage) ([]sdk.Msg, error) {
		var custom reflectCustomMsg
		err := json.Unmarshal(msg, &custom)
		if err != nil {
			return nil, errorsmod.Wrap(sdkerrors.ErrJSONUnmarshal, err.Error())
		}
		if custom.Raw != nil {
			var codecAny codectypes.Any
			if err := cdc.UnmarshalJSON(custom.Raw, &codecAny); err != nil {
				return nil, errorsmod.Wrap(sdkerrors.ErrJSONUnmarshal, err.Error())
			}
			var msg sdk.Msg
			if err := cdc.UnpackAny(&codecAny, &msg); err != nil {
				return nil, err
			}
			return []sdk.Msg{msg}, nil
		}
		if custom.Debug != "" {
			return nil, errorsmod.Wrapf(types.ErrInvalidMsg, "Custom Debug: %s", custom.Debug)
		}
		return nil, errorsmod.Wrap(types.ErrInvalidMsg, "Unknown Custom message variant")
	}
}

type reflectCustomQuery struct {
	Ping        *struct{}      `json:"ping,omitempty"`
	Capitalized *testdata.Text `json:"capitalized,omitempty"`
}

// this is from the go code back to the contract (capitalized or ping)
type customQueryResponse struct {
	Msg string `json:"msg"`
}

// reflectPlugins needs to be registered in test setup to handle custom query callbacks
func reflectPlugins() *QueryPlugins {
	return &QueryPlugins{
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
