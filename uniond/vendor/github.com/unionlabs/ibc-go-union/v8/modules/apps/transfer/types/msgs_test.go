package types_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	sdkmath "cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/cosmos/ibc-go/v8/modules/apps/transfer/types"
	clienttypes "github.com/cosmos/ibc-go/v8/modules/core/02-client/types"
	ibctesting "github.com/cosmos/ibc-go/v8/testing"
)

// define constants used for testing
const (
	validPort        = "testportid"
	invalidPort      = "(invalidport1)"
	invalidShortPort = "p"
	// 195 characters
	invalidLongPort = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Duis eros neque, ultricies vel ligula ac, convallis porttitor elit. Maecenas tincidunt turpis elit, vel faucibus nisl pellentesque sodales"

	validChannel        = "testchannel"
	invalidChannel      = "(invalidchannel1)"
	invalidShortChannel = "invalid"
	invalidLongChannel  = "invalidlongchannelinvalidlongchannelinvalidlongchannelinvalidlongchannel"

	invalidAddress = "invalid"
)

var (
	sender    = sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address()).String()
	receiver  = sdk.AccAddress("testaddr2").String()
	emptyAddr string

	coin             = sdk.NewCoin("atom", sdkmath.NewInt(100))
	ibcCoin          = sdk.NewCoin("ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", sdkmath.NewInt(100))
	invalidIBCCoin   = sdk.NewCoin("ibc/7F1D3FCF4AE79E1554", sdkmath.NewInt(100))
	invalidDenomCoin = sdk.Coin{Denom: "0atom", Amount: sdkmath.NewInt(100)}
	zeroCoin         = sdk.Coin{Denom: "atoms", Amount: sdkmath.NewInt(0)}

	timeoutHeight = clienttypes.NewHeight(0, 10)
)

// TestMsgTransferValidation tests ValidateBasic for MsgTransfer
func TestMsgTransferValidation(t *testing.T) {
	testCases := []struct {
		name    string
		msg     *types.MsgTransfer
		expPass bool
	}{
		{"valid msg with base denom", types.NewMsgTransfer(validPort, validChannel, coin, sender, receiver, timeoutHeight, 0, ""), true},
		{"valid msg with trace hash", types.NewMsgTransfer(validPort, validChannel, ibcCoin, sender, receiver, timeoutHeight, 0, ""), true},
		{"invalid ibc denom", types.NewMsgTransfer(validPort, validChannel, invalidIBCCoin, sender, receiver, timeoutHeight, 0, ""), false},
		{"too short port id", types.NewMsgTransfer(invalidShortPort, validChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"too long port id", types.NewMsgTransfer(invalidLongPort, validChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"port id contains non-alpha", types.NewMsgTransfer(invalidPort, validChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"too short channel id", types.NewMsgTransfer(validPort, invalidShortChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"too long channel id", types.NewMsgTransfer(validPort, invalidLongChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"too long memo", types.NewMsgTransfer(validPort, validChannel, coin, sender, receiver, timeoutHeight, 0, ibctesting.GenerateString(types.MaximumMemoLength+1)), false},
		{"channel id contains non-alpha", types.NewMsgTransfer(validPort, invalidChannel, coin, sender, receiver, timeoutHeight, 0, ""), false},
		{"invalid denom", types.NewMsgTransfer(validPort, validChannel, invalidDenomCoin, sender, receiver, timeoutHeight, 0, ""), false},
		{"zero coin", types.NewMsgTransfer(validPort, validChannel, zeroCoin, sender, receiver, timeoutHeight, 0, ""), false},
		{"missing sender address", types.NewMsgTransfer(validPort, validChannel, coin, emptyAddr, receiver, timeoutHeight, 0, ""), false},
		{"missing recipient address", types.NewMsgTransfer(validPort, validChannel, coin, sender, "", timeoutHeight, 0, ""), false},
		{"too long recipient address", types.NewMsgTransfer(validPort, validChannel, coin, sender, ibctesting.GenerateString(types.MaximumReceiverLength+1), timeoutHeight, 0, ""), false},
		{"empty coin", types.NewMsgTransfer(validPort, validChannel, sdk.Coin{}, sender, receiver, timeoutHeight, 0, ""), false},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

// TestMsgTransferGetSigners tests GetSigners for MsgTransfer
func TestMsgTransferGetSigners(t *testing.T) {
	addr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())

	msg := types.NewMsgTransfer(validPort, validChannel, coin, addr.String(), receiver, timeoutHeight, 0, "")
	res := msg.GetSigners()

	require.Equal(t, []sdk.AccAddress{addr}, res)
}

// TestMsgUpdateParamsValidateBasic tests ValidateBasic for MsgUpdateParams
func TestMsgUpdateParamsValidateBasic(t *testing.T) {
	testCases := []struct {
		name    string
		msg     *types.MsgUpdateParams
		expPass bool
	}{
		{"success: valid signer and valid params", types.NewMsgUpdateParams(ibctesting.TestAccAddress, types.DefaultParams()), true},
		{"failure: invalid signer with valid params", types.NewMsgUpdateParams(invalidAddress, types.DefaultParams()), false},
		{"failure: empty signer with valid params", types.NewMsgUpdateParams(emptyAddr, types.DefaultParams()), false},
	}

	for i, tc := range testCases {
		tc := tc

		err := tc.msg.ValidateBasic()
		if tc.expPass {
			require.NoError(t, err, "valid test case %d failed: %s", i, tc.name)
		} else {
			require.Error(t, err, "invalid test case %d passed: %s", i, tc.name)
		}
	}
}

// TestMsgUpdateParamsGetSigners tests GetSigners for MsgUpdateParams
func TestMsgUpdateParamsGetSigners(t *testing.T) {
	testCases := []struct {
		name    string
		address sdk.AccAddress
		expPass bool
	}{
		{"success: valid address", sdk.AccAddress(ibctesting.TestAccAddress), true},
		{"failure: nil address", nil, false},
	}

	for _, tc := range testCases {
		tc := tc

		msg := types.MsgUpdateParams{
			Signer: tc.address.String(),
			Params: types.DefaultParams(),
		}
		if tc.expPass {
			require.Equal(t, []sdk.AccAddress{tc.address}, msg.GetSigners())
		} else {
			require.Panics(t, func() {
				msg.GetSigners()
			})
		}
	}
}
