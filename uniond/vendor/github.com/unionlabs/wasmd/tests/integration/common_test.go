package integration

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	secp256k1 "github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	distributionkeeper "cosmossdk.io/x/distribution/keeper"
	stakingkeeper "cosmossdk.io/x/staking/keeper"
	stakingtypes "cosmossdk.io/x/staking/types"

	wasmKeeper "github.com/CosmWasm/wasmd/x/wasm/keeper"
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

func must[t any](s t, err error) t {
	if err != nil {
		panic(err)
	}
	return s
}

func mustUnmarshal(t *testing.T, data []byte, res interface{}) {
	t.Helper()
	err := json.Unmarshal(data, res)
	require.NoError(t, err)
}

func mustMarshal(t *testing.T, r interface{}) []byte {
	t.Helper()
	bz, err := json.Marshal(r)
	require.NoError(t, err)
	return bz
}

// this will commit the current set, update the block height and set historic info
// basically, letting two blocks pass
func nextBlock(ctx sdk.Context, stakingKeeper *stakingkeeper.Keeper) sdk.Context {
	if _, err := stakingKeeper.EndBlocker(ctx); err != nil {
		panic(err)
	}
	ctx = ctx.WithBlockHeight(ctx.BlockHeight() + 1)
	_ = stakingKeeper.BeginBlocker(ctx)
	return ctx
}

func setValidatorRewards(ctx sdk.Context, stakingKeeper *stakingkeeper.Keeper, distKeeper distributionkeeper.Keeper, valAddr sdk.ValAddress, reward string) {
	// allocate some rewards
	vali, err := stakingKeeper.Validator(ctx, valAddr)
	if err != nil {
		panic(err)
	}
	amount, err := sdkmath.LegacyNewDecFromStr(reward)
	if err != nil {
		panic(err)
	}
	payout := sdk.DecCoins{{Denom: "stake", Amount: amount}}
	err = distKeeper.AllocateTokensToValidator(ctx, vali, payout)
	if err != nil {
		panic(err)
	}
}

// adds a few validators and returns a list of validators that are registered
func addValidator(t *testing.T, ctx sdk.Context, stakingKeeper *stakingkeeper.Keeper, faucet *wasmKeeper.TestFaucet, value sdk.Coin) sdk.ValAddress {
	owner := faucet.NewFundedRandomAccount(ctx, value)

	privKey := secp256k1.GenPrivKey()
	pubKey := privKey.PubKey()
	valAddr := sdk.ValAddress(owner)

	pkAny, err := codectypes.NewAnyWithValue(pubKey)
	require.NoError(t, err)
	msg := &stakingtypes.MsgCreateValidator{
		Description: stakingtypes.Description{
			Moniker: "Validator power",
		},
		Commission: stakingtypes.CommissionRates{
			Rate:          sdkmath.LegacyMustNewDecFromStr("0.1"),
			MaxRate:       sdkmath.LegacyMustNewDecFromStr("0.2"),
			MaxChangeRate: sdkmath.LegacyMustNewDecFromStr("0.01"),
		},
		MinSelfDelegation: sdkmath.OneInt(),
		DelegatorAddress:  owner.String(),
		ValidatorAddress:  valAddr.String(),
		Pubkey:            pkAny,
		Value:             value,
	}
	_, err = stakingkeeper.NewMsgServerImpl(stakingKeeper).CreateValidator(ctx, msg)
	require.NoError(t, err)
	return valAddr
}

// reflectEncoders needs to be registered in test setup to handle custom message callbacks
func reflectEncoders(cdc codec.Codec) *wasmKeeper.MessageEncoders {
	return &wasmKeeper.MessageEncoders{
		Custom: fromReflectRawMsg(cdc),
	}
}

/**** Code to support custom messages *****/

type reflectCustomMsg struct {
	Debug string `json:"debug,omitempty"`
	Raw   []byte `json:"raw,omitempty"`
}

// fromReflectRawMsg decodes msg.Data to an sdk.Msg using proto Any and json encoding.
// this needs to be registered on the Encoders
func fromReflectRawMsg(cdc codec.Codec) wasmKeeper.CustomEncoder {
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
