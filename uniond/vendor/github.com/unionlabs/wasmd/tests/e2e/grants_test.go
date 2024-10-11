package e2e_test

import (
	"fmt"
	"os"
	"testing"
	"time"

	wasmvm "github.com/CosmWasm/wasmvm/v2"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	errorsmod "cosmossdk.io/errors"
	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"cosmossdk.io/x/authz"

	"github.com/CosmWasm/wasmd/tests/e2e"
	"github.com/CosmWasm/wasmd/tests/ibctesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestGrants(t *testing.T) {
	// Given a contract by address A
	// And   a grant for address B by A created
	// When  B sends an execute with tokens from A
	// Then	 the grant is executed as defined
	// And
	// - balance A reduced (on success)
	// - balance B not touched

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))
	contractAddr := e2e.InstantiateReflectContract(t, chain)
	require.NotEmpty(t, contractAddr)

	granterAddr := chain.SenderAccount.GetAddress()
	granteePrivKey := secp256k1.GenPrivKey()
	granteeAddr := sdk.AccAddress(granteePrivKey.PubKey().Address().Bytes())
	otherPrivKey := secp256k1.GenPrivKey()
	otherAddr := sdk.AccAddress(otherPrivKey.PubKey().Address().Bytes())

	chain.Fund(granteeAddr, sdkmath.NewInt(1_000_000))
	chain.Fund(otherAddr, sdkmath.NewInt(1_000_000))
	assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)

	myAmount := sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.NewInt(2_000_000))

	specs := map[string]struct {
		limit          types.ContractAuthzLimitX
		filter         types.ContractAuthzFilterX
		transferAmount sdk.Coin
		senderKey      cryptotypes.PrivKey
		expErr         *errorsmod.Error
	}{
		"in limits and filter": {
			limit:          types.NewMaxFundsLimit(myAmount),
			filter:         types.NewAllowAllMessagesFilter(),
			transferAmount: myAmount,
			senderKey:      granteePrivKey,
		},
		"exceed limits": {
			limit:          types.NewMaxFundsLimit(myAmount),
			filter:         types.NewAllowAllMessagesFilter(),
			transferAmount: myAmount.Add(sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt())),
			senderKey:      granteePrivKey,
			expErr:         sdkerrors.ErrUnauthorized,
		},
		"not match filter": {
			limit:          types.NewMaxFundsLimit(myAmount),
			filter:         types.NewAcceptedMessageKeysFilter("foo"),
			transferAmount: sdk.NewCoin(sdk.DefaultBondDenom, sdkmath.OneInt()),
			senderKey:      granteePrivKey,
			expErr:         sdkerrors.ErrUnauthorized,
		},
		"non authorized sender address": { // sanity check - testing sdk
			limit:          types.NewMaxFundsLimit(myAmount),
			filter:         types.NewAllowAllMessagesFilter(),
			senderKey:      otherPrivKey,
			transferAmount: myAmount,
			expErr:         authz.ErrNoAuthorizationFound,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			// setup grant
			grant, err := types.NewContractGrant(contractAddr, spec.limit, spec.filter)
			require.NoError(t, err)
			authorization := types.NewContractExecutionAuthorization(*grant)
			expiry := time.Now().Add(time.Hour)
			grantMsg, err := authz.NewMsgGrant(granterAddr, granteeAddr, authorization, &expiry)
			require.NoError(t, err)
			_, err = chain.SendMsgs(grantMsg)
			require.NoError(t, err)

			granterStartBalance := chain.Balance(granterAddr, sdk.DefaultBondDenom).Amount

			// when
			anyValidReflectMsg := []byte(fmt.Sprintf(`{"reflect_msg": {"msgs": [{"bank":{"burn":{"amount":[{"denom":%q, "amount": %q}]}}}]}}`, sdk.DefaultBondDenom, myAmount.Amount.String()))
			execMsg := authz.NewMsgExec(spec.senderKey.PubKey().Address().Bytes(), []sdk.Msg{&types.MsgExecuteContract{
				Sender:   granterAddr.String(),
				Contract: contractAddr.String(),
				Msg:      anyValidReflectMsg,
				Funds:    sdk.NewCoins(spec.transferAmount),
			}})
			_, gotErr := chain.SendNonDefaultSenderMsgs(spec.senderKey, &execMsg)

			// then
			if spec.expErr != nil {
				require.ErrorContains(t, gotErr, fmt.Sprintf("%s/%d:", spec.expErr.Codespace(), spec.expErr.ABCICode()))
				assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)
				assert.Equal(t, granterStartBalance, chain.Balance(granterAddr, sdk.DefaultBondDenom).Amount)
				return
			}
			require.NoError(t, gotErr)
			assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)
			assert.Equal(t, granterStartBalance.Sub(spec.transferAmount.Amount), chain.Balance(granterAddr, sdk.DefaultBondDenom).Amount)
		})
	}
}

func TestStoreCodeGrant(t *testing.T) {
	reflectWasmCode, err := os.ReadFile("../../x/wasm/keeper/testdata/reflect_1_1.wasm")
	require.NoError(t, err)

	reflectCodeChecksum, err := wasmvm.CreateChecksum(reflectWasmCode)
	require.NoError(t, err)

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))

	granterAddr := chain.SenderAccount.GetAddress()
	granteePrivKey := secp256k1.GenPrivKey()
	granteeAddr := sdk.AccAddress(granteePrivKey.PubKey().Address().Bytes())
	otherPrivKey := secp256k1.GenPrivKey()
	otherAddr := sdk.AccAddress(otherPrivKey.PubKey().Address().Bytes())

	chain.Fund(granteeAddr, sdkmath.NewInt(1_000_000))
	chain.Fund(otherAddr, sdkmath.NewInt(1_000_000))
	assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)

	specs := map[string]struct {
		codeHash              []byte
		instantiatePermission types.AccessConfig
		senderKey             cryptotypes.PrivKey
		expErr                *errorsmod.Error
	}{
		"any code hash": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
		},
		"match code hash and permission": {
			codeHash:              reflectCodeChecksum,
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
		},
		"not match code hash": {
			codeHash:              []byte("any_valid_checksum"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
			expErr:                sdkerrors.ErrUnauthorized,
		},
		"not match permission": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowNobody,
			senderKey:             granteePrivKey,
			expErr:                sdkerrors.ErrUnauthorized,
		},
		"non authorized sender address": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             otherPrivKey,
			expErr:                authz.ErrNoAuthorizationFound,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			// setup grant
			grant, err := types.NewCodeGrant(spec.codeHash, &spec.instantiatePermission) //nolint:gosec
			require.NoError(t, err)
			authorization := types.NewStoreCodeAuthorization(*grant)
			expiry := time.Now().Add(time.Hour)
			grantMsg, err := authz.NewMsgGrant(granterAddr, granteeAddr, authorization, &expiry)
			require.NoError(t, err)
			_, err = chain.SendMsgs(grantMsg)
			require.NoError(t, err)

			// when
			execMsg := authz.NewMsgExec(spec.senderKey.PubKey().Address().Bytes(), []sdk.Msg{&types.MsgStoreCode{
				Sender:                granterAddr.String(),
				WASMByteCode:          reflectWasmCode,
				InstantiatePermission: &types.AllowEverybody,
			}})
			_, gotErr := chain.SendNonDefaultSenderMsgs(spec.senderKey, &execMsg)

			// then
			if spec.expErr != nil {
				assert.ErrorContains(t, gotErr, fmt.Sprintf("%s/%d:", spec.expErr.Codespace(), spec.expErr.ABCICode()))
				return
			}
			require.NoError(t, gotErr)
		})
	}
}

func TestGzipStoreCodeGrant(t *testing.T) {
	hackatomWasmCode, err := os.ReadFile("../../x/wasm/keeper/testdata/hackatom.wasm")
	require.NoError(t, err)

	hackatomGzipWasmCode, err := os.ReadFile("../../x/wasm/keeper/testdata/hackatom.wasm.gzip")
	require.NoError(t, err)

	hackatomCodeChecksum, err := wasmvm.CreateChecksum(hackatomWasmCode)
	require.NoError(t, err)

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))

	granterAddr := chain.SenderAccount.GetAddress()
	granteePrivKey := secp256k1.GenPrivKey()
	granteeAddr := sdk.AccAddress(granteePrivKey.PubKey().Address().Bytes())
	otherPrivKey := secp256k1.GenPrivKey()
	otherAddr := sdk.AccAddress(otherPrivKey.PubKey().Address().Bytes())

	chain.Fund(granteeAddr, sdkmath.NewInt(1_000_000))
	chain.Fund(otherAddr, sdkmath.NewInt(1_000_000))
	assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)

	specs := map[string]struct {
		codeHash              []byte
		instantiatePermission types.AccessConfig
		senderKey             cryptotypes.PrivKey
		expErr                *errorsmod.Error
	}{
		"any code hash": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
		},
		"match code hash and permission": {
			codeHash:              hackatomCodeChecksum,
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
		},
		"not match code hash": {
			codeHash:              []byte("any_valid_checksum"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             granteePrivKey,
			expErr:                sdkerrors.ErrUnauthorized,
		},
		"not match permission": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowNobody,
			senderKey:             granteePrivKey,
			expErr:                sdkerrors.ErrUnauthorized,
		},
		"non authorized sender address": {
			codeHash:              []byte("*"),
			instantiatePermission: types.AllowEverybody,
			senderKey:             otherPrivKey,
			expErr:                authz.ErrNoAuthorizationFound,
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			// setup grant
			grant, err := types.NewCodeGrant(spec.codeHash, &spec.instantiatePermission) //nolint:gosec
			require.NoError(t, err)
			authorization := types.NewStoreCodeAuthorization(*grant)
			expiry := time.Now().Add(time.Hour)
			grantMsg, err := authz.NewMsgGrant(granterAddr, granteeAddr, authorization, &expiry)
			require.NoError(t, err)
			_, err = chain.SendMsgs(grantMsg)
			require.NoError(t, err)

			// when
			execMsg := authz.NewMsgExec(spec.senderKey.PubKey().Address().Bytes(), []sdk.Msg{&types.MsgStoreCode{
				Sender:                granterAddr.String(),
				WASMByteCode:          hackatomGzipWasmCode,
				InstantiatePermission: &types.AllowEverybody,
			}})
			_, gotErr := chain.SendNonDefaultSenderMsgs(spec.senderKey, &execMsg)

			// then
			if spec.expErr != nil {
				assert.ErrorContains(t, gotErr, fmt.Sprintf("%s/%d:", spec.expErr.Codespace(), spec.expErr.ABCICode()))
				return
			}
			require.NoError(t, gotErr)
		})
	}
}

func TestBrokenGzipStoreCodeGrant(t *testing.T) {
	brokenGzipWasmCode, err := os.ReadFile("../../x/wasm/keeper/testdata/broken_crc.gzip")
	require.NoError(t, err)

	coord := ibctesting.NewCoordinator(t, 1)
	chain := coord.GetChain(ibctesting.GetChainID(1))

	granterAddr := chain.SenderAccount.GetAddress()
	granteePrivKey := secp256k1.GenPrivKey()
	granteeAddr := sdk.AccAddress(granteePrivKey.PubKey().Address().Bytes())
	otherPrivKey := secp256k1.GenPrivKey()
	otherAddr := sdk.AccAddress(otherPrivKey.PubKey().Address().Bytes())

	chain.Fund(granteeAddr, sdkmath.NewInt(1_000_000))
	chain.Fund(otherAddr, sdkmath.NewInt(1_000_000))
	assert.Equal(t, sdkmath.NewInt(1_000_000), chain.Balance(granteeAddr, sdk.DefaultBondDenom).Amount)

	codeHash := []byte("*")
	instantiatePermission := types.AllowEverybody
	senderKey := granteePrivKey

	// setup grant
	grant, err := types.NewCodeGrant(codeHash, &instantiatePermission)
	require.NoError(t, err)
	authorization := types.NewStoreCodeAuthorization(*grant)
	expiry := time.Now().Add(time.Hour)
	grantMsg, err := authz.NewMsgGrant(granterAddr, granteeAddr, authorization, &expiry)
	require.NoError(t, err)
	_, err = chain.SendMsgs(grantMsg)
	require.NoError(t, err)

	// when
	execMsg := authz.NewMsgExec(senderKey.PubKey().Address().Bytes(), []sdk.Msg{&types.MsgStoreCode{
		Sender:                granterAddr.String(),
		WASMByteCode:          brokenGzipWasmCode,
		InstantiatePermission: &types.AllowEverybody,
	}})
	_, gotErr := chain.SendNonDefaultSenderMsgs(senderKey, &execMsg)

	// then
	require.Error(t, gotErr)
}
