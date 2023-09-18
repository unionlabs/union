package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	govtypes "github.com/cosmos/cosmos-sdk/x/gov/types"
)

func TestMsgSendRoute(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("from"))
	addr2 := sdk.AccAddress([]byte("to"))
	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := NewMsgSend(addr1, addr2, coins)

	require.Equal(t, msg.Route(), RouterKey)
	require.Equal(t, msg.Type(), "send")
}

func TestMsgSendValidation(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("from________________"))
	addr2 := sdk.AccAddress([]byte("to__________________"))
	addrEmpty := sdk.AccAddress([]byte(""))
	addrLong := sdk.AccAddress([]byte("Purposefully long address"))

	atom123 := sdk.NewCoins(sdk.NewInt64Coin("atom", 123))
	atom0 := sdk.NewCoins(sdk.NewInt64Coin("atom", 0))
	atom123eth123 := sdk.NewCoins(sdk.NewInt64Coin("atom", 123), sdk.NewInt64Coin("eth", 123))
	atom123eth0 := sdk.Coins{sdk.NewInt64Coin("atom", 123), sdk.NewInt64Coin("eth", 0)}

	cases := []struct {
		expectedErr string // empty means no error expected
		msg         *MsgSend
	}{
		{"", NewMsgSend(addr1, addr2, atom123)},                                // valid send
		{"", NewMsgSend(addr1, addr2, atom123eth123)},                          // valid send with multiple coins
		{"", NewMsgSend(addrLong, addr2, atom123)},                             // valid send with long addr sender
		{"", NewMsgSend(addr1, addrLong, atom123)},                             // valid send with long addr recipient
		{": invalid coins", NewMsgSend(addr1, addr2, atom0)},                   // non positive coin
		{"123atom,0eth: invalid coins", NewMsgSend(addr1, addr2, atom123eth0)}, // non positive coin in multicoins
		{"invalid from address: empty address string is not allowed: invalid address", NewMsgSend(addrEmpty, addr2, atom123)},
		{"invalid to address: empty address string is not allowed: invalid address", NewMsgSend(addr1, addrEmpty, atom123)},
	}

	for _, tc := range cases {
		err := tc.msg.ValidateBasic()
		if tc.expectedErr == "" {
			require.Nil(t, err)
		} else {
			require.EqualError(t, err, tc.expectedErr)
		}
	}
}

func TestMsgSendGetSignBytes(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("input"))
	addr2 := sdk.AccAddress([]byte("output"))
	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := NewMsgSend(addr1, addr2, coins)
	res := msg.GetSignBytes()

	expected := `{"type":"cosmos-sdk/MsgSend","value":{"amount":[{"amount":"10","denom":"atom"}],"from_address":"cosmos1d9h8qat57ljhcm","to_address":"cosmos1da6hgur4wsmpnjyg"}}`
	require.Equal(t, expected, string(res))
}

func TestMsgMultiSendRoute(t *testing.T) {
	// Construct a MsgSend
	addr1 := sdk.AccAddress([]byte("input"))
	addr2 := sdk.AccAddress([]byte("output"))
	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := MsgMultiSend{
		Inputs:  []Input{NewInput(addr1, coins)},
		Outputs: []Output{NewOutput(addr2, coins)},
	}

	// TODO some failures for bad result
	require.Equal(t, msg.Route(), RouterKey)
	require.Equal(t, msg.Type(), "multisend")
}

func TestInputValidation(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("_______alice________"))
	addr2 := sdk.AccAddress([]byte("________bob_________"))
	addrEmpty := sdk.AccAddress([]byte(""))
	addrLong := sdk.AccAddress([]byte("Purposefully long address"))

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
	addr1 := sdk.AccAddress([]byte("_______alice________"))
	addr2 := sdk.AccAddress([]byte("________bob_________"))
	addrEmpty := sdk.AccAddress([]byte(""))
	addrLong := sdk.AccAddress([]byte("Purposefully long address"))

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

func TestMsgMultiSendValidation(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("_______alice________"))
	addr2 := sdk.AccAddress([]byte("________bob_________"))
	addr3 := sdk.AccAddress([]byte("_______addr3________"))
	atom123 := sdk.NewCoins(sdk.NewInt64Coin("atom", 123))
	atom124 := sdk.NewCoins(sdk.NewInt64Coin("atom", 124))
	atom246 := sdk.NewCoins(sdk.NewInt64Coin("atom", 246))

	input1 := NewInput(addr1, atom123)
	input2 := NewInput(addr1, atom246)
	output1 := NewOutput(addr2, atom123)
	output2 := NewOutput(addr2, atom124)
	output3 := NewOutput(addr2, atom123)
	output4 := NewOutput(addr3, atom123)

	var emptyAddr sdk.AccAddress

	cases := []struct {
		valid     bool
		tx        MsgMultiSend
		expErrMsg string
	}{
		{false, MsgMultiSend{}, "no inputs to send transaction"},                           // no input or output
		{false, MsgMultiSend{Inputs: []Input{input1}}, "no outputs to send transaction"},   // just input
		{false, MsgMultiSend{Outputs: []Output{output1}}, "no inputs to send transaction"}, // just output
		{
			false,
			MsgMultiSend{
				Inputs:  []Input{NewInput(emptyAddr, atom123)}, // invalid input
				Outputs: []Output{output1},
			},
			"invalid input address",
		},
		{
			false,
			MsgMultiSend{
				Inputs:  []Input{input1},
				Outputs: []Output{{emptyAddr.String(), atom123}}, // invalid output
			},
			"invalid output address",
		},
		{
			false,
			MsgMultiSend{
				Inputs:  []Input{input1},
				Outputs: []Output{output2}, // amounts don't match
			},
			"sum inputs != sum outputs",
		},
		{
			true,
			MsgMultiSend{
				Inputs:  []Input{input1},
				Outputs: []Output{output1},
			},
			"",
		},
		{
			false,
			MsgMultiSend{
				Inputs:  []Input{input1, input2},
				Outputs: []Output{output3, output4},
			},
			"multiple senders not allowed",
		},
		{
			true,
			MsgMultiSend{
				Inputs:  []Input{NewInput(addr2, atom123.MulInt(sdk.NewInt(2)))},
				Outputs: []Output{output1, output1},
			},
			"",
		},
	}

	for i, tc := range cases {
		err := tc.tx.ValidateBasic()
		if tc.valid {
			require.Nil(t, err, "%d: %+v", i, err)
			require.Nil(t, err)
		} else {
			require.NotNil(t, err, "%d", i)
			require.Contains(t, err.Error(), tc.expErrMsg)
		}
	}
}

func TestMsgMultiSendGetSignBytes(t *testing.T) {
	addr1 := sdk.AccAddress([]byte("input"))
	addr2 := sdk.AccAddress([]byte("output"))
	coins := sdk.NewCoins(sdk.NewInt64Coin("atom", 10))
	msg := MsgMultiSend{
		Inputs:  []Input{NewInput(addr1, coins)},
		Outputs: []Output{NewOutput(addr2, coins)},
	}
	res := msg.GetSignBytes()

	expected := `{"type":"cosmos-sdk/MsgMultiSend","value":{"inputs":[{"address":"cosmos1d9h8qat57ljhcm","coins":[{"amount":"10","denom":"atom"}]}],"outputs":[{"address":"cosmos1da6hgur4wsmpnjyg","coins":[{"amount":"10","denom":"atom"}]}]}}`
	require.Equal(t, expected, string(res))
}

func TestMsgMultiSendGetSigners(t *testing.T) {
	addrs := make([]string, 3)
	inputs := make([]Input, 3)
	for i, v := range []string{"input111111111111111", "input222222222222222", "input333333333333333"} {
		addr := sdk.AccAddress([]byte(v))
		inputs[i] = NewInput(addr, nil)
		addrs[i] = addr.String()
	}
	msg := NewMsgMultiSend(inputs, nil)

	res := msg.GetSigners()
	for i, signer := range res {
		require.Equal(t, signer.String(), addrs[i])
	}
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

func TestMsgSendGetSigners(t *testing.T) {
	from := sdk.AccAddress([]byte("input111111111111111"))
	msg := NewMsgSend(from, sdk.AccAddress{}, sdk.NewCoins())
	res := msg.GetSigners()
	require.Equal(t, 1, len(res))
	require.True(t, from.Equals(res[0]))
}

func TestMsgSetSendEnabledGetSignBytes(t *testing.T) {
	msg := NewMsgSetSendEnabled("cartman", []*SendEnabled{{"casafiestacoin", false}, {"kylecoin", true}}, nil)
	expected := `{"type":"cosmos-sdk/MsgSetSendEnabled","value":{"authority":"cartman","send_enabled":[{"denom":"casafiestacoin"},{"denom":"kylecoin","enabled":true}]}}`
	actualBz := msg.GetSignBytes()
	actual := string(actualBz)
	assert.Equal(t, expected, actual)
}

func TestMsgSetSendEnabledGetSigners(t *testing.T) {
	govModuleAddr := authtypes.NewModuleAddress(govtypes.ModuleName)
	msg := NewMsgSetSendEnabled(govModuleAddr.String(), nil, nil)
	expected := []sdk.AccAddress{govModuleAddr}
	actual := msg.GetSigners()
	assert.Equal(t, expected, actual)
}

func TestMsgSetSendEnabledValidateBasic(t *testing.T) {
	govModuleAddr := authtypes.NewModuleAddress(govtypes.ModuleName).String()
	tests := []struct {
		name string
		msg  MsgSetSendEnabled
		exp  string
	}{
		{
			name: "valid with two entries",
			msg: MsgSetSendEnabled{
				Authority: govModuleAddr,
				SendEnabled: []*SendEnabled{
					{"somecoina", true},
					{"somecoinb", false},
				},
				UseDefaultFor: []string{"defcoinc", "defcoind"},
			},
			exp: "",
		},
		{
			name: "valid with two entries but no authority",
			msg: MsgSetSendEnabled{
				Authority: "",
				SendEnabled: []*SendEnabled{
					{"somecoina", true},
					{"somecoinb", false},
				},
				UseDefaultFor: []string{"defcoinc", "defcoind"},
			},
			exp: "",
		},
		{
			name: "bad authority",
			msg: MsgSetSendEnabled{
				Authority: "farva",
				SendEnabled: []*SendEnabled{
					{"somecoina", true},
					{"somecoinb", false},
				},
			},
			exp: "invalid authority address: decoding bech32 failed: invalid bech32 string length 5: invalid address",
		},
		{
			name: "bad first denom name",
			msg: MsgSetSendEnabled{
				Authority: govModuleAddr,
				SendEnabled: []*SendEnabled{
					{"Not A Denom", true},
					{"somecoinb", false},
				},
			},
			exp: `invalid SendEnabled denom "Not A Denom": invalid denom: Not A Denom: invalid request`,
		},
		{
			name: "bad second denom",
			msg: MsgSetSendEnabled{
				Authority: govModuleAddr,
				SendEnabled: []*SendEnabled{
					{"somecoina", true},
					{"", false},
				},
			},
			exp: `invalid SendEnabled denom "": invalid denom: : invalid request`,
		},
		{
			name: "duplicate denom",
			msg: MsgSetSendEnabled{
				Authority: govModuleAddr,
				SendEnabled: []*SendEnabled{
					{"copycoin", true},
					{"copycoin", false},
				},
			},
			exp: `duplicate denom entries found for "copycoin": invalid request`,
		},
		{
			name: "bad denom to delete",
			msg: MsgSetSendEnabled{
				Authority:     govModuleAddr,
				UseDefaultFor: []string{"very \t bad denom string~~~!"},
			},
			exp: "invalid UseDefaultFor denom \"very \\t bad denom string~~~!\": invalid denom: very \t bad denom string~~~!: invalid request",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(tt *testing.T) {
			actual := tc.msg.ValidateBasic()
			if len(tc.exp) > 0 {
				require.EqualError(tt, actual, tc.exp)
			} else {
				require.NoError(tt, actual)
			}
		})
	}
}
