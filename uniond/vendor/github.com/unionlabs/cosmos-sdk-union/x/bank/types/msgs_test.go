package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/codec/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestMsgSendGetSignBytes(t *testing.T) {
	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := NewMsgSend("cosmos1d9h8qat57ljhcm", "cosmos1da6hgur4wsmpnjyg", coins)
	res, err := codec.NewProtoCodec(types.NewInterfaceRegistry()).MarshalAminoJSON(msg)
	require.NoError(t, err)

	expected := `{"type":"cosmos-sdk/MsgSend","value":{"amount":[{"amount":"10","denom":"atom"}],"from_address":"cosmos1d9h8qat57ljhcm","to_address":"cosmos1da6hgur4wsmpnjyg"}}`
	require.Equal(t, expected, string(res))
}

func TestInputValidation(t *testing.T) {
	ac := testutil.CodecOptions{}.GetAddressCodec()
	addr1, err := ac.BytesToString([]byte("_______alice________"))
	require.NoError(t, err)
	addr2, err := ac.BytesToString([]byte("________bob_________"))
	require.NoError(t, err)
	addrEmpty, err := ac.BytesToString([]byte(""))
	require.NoError(t, err)
	addrLong, err := ac.BytesToString([]byte("Purposefully long address"))
	require.NoError(t, err)

	someCoins := sdk.NewCoins(sdk.NewInt64Coin("atom", 123))
	multiCoins := sdk.NewCoins(sdk.NewInt64Coin("atom", 123), sdk.NewInt64Coin("eth", 20))

	emptyCoins := sdk.NewCoins()
	emptyCoins2 := sdk.NewCoins(sdk.NewInt64Coin("eth", 0))
	someEmptyCoins := sdk.Coins{sdk.NewInt64Coin("eth", 10), sdk.NewInt64Coin("atom", 0)}
	unsortedCoins := sdk.Coins{sdk.NewInt64Coin("eth", 1), sdk.NewInt64Coin("atom", 1)}

	cases := []struct {
		expectedErr string // empty means no error expected
		txIn        Input
	}{
		// auth works with different apps
		{"", NewInput(addr1, someCoins)},
		{"", NewInput(addr2, someCoins)},
		{"", NewInput(addr2, multiCoins)},
		{"", NewInput(addrLong, someCoins)},

		{"invalid input address: empty address string is not allowed: invalid address", NewInput(addrEmpty, someCoins)},
		{": invalid coins", NewInput(addr1, emptyCoins)},                // invalid coins
		{": invalid coins", NewInput(addr1, emptyCoins2)},               // invalid coins
		{"10eth,0atom: invalid coins", NewInput(addr1, someEmptyCoins)}, // invalid coins
		{"1eth,1atom: invalid coins", NewInput(addr1, unsortedCoins)},   // unsorted coins
	}

	for i, tc := range cases {
		err := tc.txIn.ValidateBasic()
		if tc.expectedErr == "" {
			require.Nil(t, err, "%d: %+v", i, err)
		} else {
			require.EqualError(t, err, tc.expectedErr, "%d", i)
		}
	}
}

func TestOutputValidation(t *testing.T) {
	ac := testutil.CodecOptions{}.GetAddressCodec()
	addr1, err := ac.BytesToString([]byte("_______alice________"))
	require.NoError(t, err)
	addr2, err := ac.BytesToString([]byte("________bob_________"))
	require.NoError(t, err)
	addrEmpty, err := ac.BytesToString([]byte(""))
	require.NoError(t, err)
	addrLong, err := ac.BytesToString([]byte("Purposefully long address"))
	require.NoError(t, err)

	someCoins := sdk.NewCoins(sdk.NewInt64Coin("atom", 123))
	multiCoins := sdk.NewCoins(sdk.NewInt64Coin("atom", 123), sdk.NewInt64Coin("eth", 20))

	emptyCoins := sdk.NewCoins()
	emptyCoins2 := sdk.NewCoins(sdk.NewInt64Coin("eth", 0))
	someEmptyCoins := sdk.Coins{sdk.NewInt64Coin("eth", 10), sdk.NewInt64Coin("atom", 0)}
	unsortedCoins := sdk.Coins{sdk.NewInt64Coin("eth", 1), sdk.NewInt64Coin("atom", 1)}

	cases := []struct {
		expectedErr string // empty means no error expected
		txOut       Output
	}{
		// auth works with different apps
		{"", NewOutput(addr1, someCoins)},
		{"", NewOutput(addr2, someCoins)},
		{"", NewOutput(addr2, multiCoins)},
		{"", NewOutput(addrLong, someCoins)},

		{"invalid output address: empty address string is not allowed: invalid address", NewOutput(addrEmpty, someCoins)},
		{": invalid coins", NewOutput(addr1, emptyCoins)},                // invalid coins
		{": invalid coins", NewOutput(addr1, emptyCoins2)},               // invalid coins
		{"10eth,0atom: invalid coins", NewOutput(addr1, someEmptyCoins)}, // invalid coins
		{"1eth,1atom: invalid coins", NewOutput(addr1, unsortedCoins)},   // unsorted coins
	}

	for i, tc := range cases {
		err := tc.txOut.ValidateBasic()
		if tc.expectedErr == "" {
			require.Nil(t, err, "%d: %+v", i, err)
		} else {
			require.EqualError(t, err, tc.expectedErr, "%d", i)
		}
	}
}

func TestMsgMultiSendGetSignBytes(t *testing.T) {
	ac := testutil.CodecOptions{}.GetAddressCodec()
	addr1, err := ac.BytesToString([]byte("input"))
	require.NoError(t, err)
	addr2, err := ac.BytesToString([]byte("output"))
	require.NoError(t, err)

	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := &MsgMultiSend{
		Inputs:  []Input{NewInput(addr1, coins)},
		Outputs: []Output{NewOutput(addr2, coins)},
	}
	res, err := codec.NewProtoCodec(types.NewInterfaceRegistry()).MarshalAminoJSON(msg)
	require.NoError(t, err)

	expected := `{"type":"cosmos-sdk/MsgMultiSend","value":{"inputs":[{"address":"cosmos1d9h8qat57ljhcm","coins":[{"amount":"10","denom":"atom"}]}],"outputs":[{"address":"cosmos1da6hgur4wsmpnjyg","coins":[{"amount":"10","denom":"atom"}]}]}}`
	require.Equal(t, expected, string(res))
}

func TestNewMsgSetSendEnabled(t *testing.T) {
	// Punt. Just setting one to all non-default values and making sure they're as expected.
	msg := NewMsgSetSendEnabled("milton", []*SendEnabled{{"barrycoin", true}}, []string{"billcoin"})
	assert.Equal(t, "milton", msg.Authority, "msg.Authority")
	if assert.Len(t, msg.SendEnabled, 1, "msg.SendEnabled length") {
		assert.Equal(t, "barrycoin", msg.SendEnabled[0].Denom, "msg.SendEnabled[0].Denom")
		assert.True(t, msg.SendEnabled[0].Enabled, "msg.SendEnabled[0].Enabled")
	}
	if assert.Len(t, msg.UseDefaultFor, 1, "msg.UseDefault") {
		assert.Equal(t, "billcoin", msg.UseDefaultFor[0], "msg.UseDefault[0]")
	}
}

func TestMsgSetSendEnabledGetSignBytes(t *testing.T) {
	msg := NewMsgSetSendEnabled("cartman", []*SendEnabled{{"casafiestacoin", false}, {"kylecoin", true}}, nil)
	expected := `{"type":"cosmos-sdk/MsgSetSendEnabled","value":{"authority":"cartman","send_enabled":[{"denom":"casafiestacoin"},{"denom":"kylecoin","enabled":true}]}}`
	actualBz, err := codec.NewProtoCodec(types.NewInterfaceRegistry()).MarshalAminoJSON(msg)
	require.NoError(t, err)
	actual := string(actualBz)
	assert.Equal(t, expected, actual)
}
