package types_test

import (
	"context"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/x/bank/types"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// MintingRestrictionArgs are the args provided to a MintingRestrictionFn function.
type MintingRestrictionArgs struct {
	Name  string
	Coins sdk.Coins
}

// MintingRestrictionTestHelper is a struct with stuff helpful for testing the MintingRestrictionFn stuff.
type MintingRestrictionTestHelper struct {
	Calls []*MintingRestrictionArgs
}

func NewMintingRestrictionTestHelper() *MintingRestrictionTestHelper {
	return &MintingRestrictionTestHelper{Calls: make([]*MintingRestrictionArgs, 0, 2)}
}

// RecordCall makes note that the provided args were used as a function call.
func (s *MintingRestrictionTestHelper) RecordCall(name string, coins sdk.Coins) {
	s.Calls = append(s.Calls, s.NewArgs(name, coins))
}

// NewCalls is just a shorter way to create a []*MintingRestrictionArgs.
func (s *MintingRestrictionTestHelper) NewCalls(args ...*MintingRestrictionArgs) []*MintingRestrictionArgs {
	return args
}

// NewArgs creates a new MintingRestrictionArgs.
func (s *MintingRestrictionTestHelper) NewArgs(name string, coins sdk.Coins) *MintingRestrictionArgs {
	return &MintingRestrictionArgs{
		Name:  name,
		Coins: coins,
	}
}

// NamedRestriction creates a new MintingRestrictionFn function that records the arguments it's called with and returns nil.
func (s *MintingRestrictionTestHelper) NamedRestriction(name string) types.MintingRestrictionFn {
	return func(_ context.Context, coins sdk.Coins) error {
		s.RecordCall(name, coins)
		return nil
	}
}

// ErrorRestriction creates a new MintingRestrictionFn function that returns an error.
func (s *MintingRestrictionTestHelper) ErrorRestriction(message string) types.MintingRestrictionFn {
	return func(_ context.Context, coins sdk.Coins) error {
		s.RecordCall(message, coins)
		return errors.New(message)
	}
}

// MintingRestrictionTestParams are parameters to test regarding calling a MintingRestrictionFn.
type MintingRestrictionTestParams struct {
	// ExpNil is whether to expect the provided MintingRestrictionFn to be nil.
	// If it is true, the rest of these test params are ignored.
	ExpNil bool
	// Coins is the MintingRestrictionFn coins input.
	Coins sdk.Coins
	// ExpErr is the expected return error string.
	ExpErr string
	// ExpCalls is the args of all the MintingRestrictionFn calls that end up being made.
	ExpCalls []*MintingRestrictionArgs
}

// TestActual tests the provided MintingRestrictionFn using the provided test parameters.
func (s *MintingRestrictionTestHelper) TestActual(t *testing.T, tp *MintingRestrictionTestParams, actual types.MintingRestrictionFn) {
	t.Helper()
	if tp.ExpNil {
		require.Nil(t, actual, "resulting MintingRestrictionFn")
	} else {
		require.NotNil(t, actual, "resulting MintingRestrictionFn")
		s.Calls = s.Calls[:0]
		err := actual(sdk.Context{}, tp.Coins)
		if len(tp.ExpErr) != 0 {
			assert.EqualError(t, err, tp.ExpErr, "composite MintingRestrictionFn output error")
		} else {
			assert.NoError(t, err, "composite MintingRestrictionFn output error")
		}
		assert.Equal(t, tp.ExpCalls, s.Calls, "args given to funcs in composite MintingRestrictionFn")
	}
}

func TestMintingRestriction_Then(t *testing.T) {
	coins := sdk.NewCoins(sdk.NewInt64Coin("acoin", 2), sdk.NewInt64Coin("bcoin", 4))

	h := NewMintingRestrictionTestHelper()

	tests := []struct {
		name   string
		base   types.MintingRestrictionFn
		second types.MintingRestrictionFn
		exp    *MintingRestrictionTestParams
	}{
		{
			name:   "nil nil",
			base:   nil,
			second: nil,
			exp: &MintingRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:   "nil noop",
			base:   nil,
			second: h.NamedRestriction("noop"),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:   "noop nil",
			base:   h.NamedRestriction("noop"),
			second: nil,
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:   "noop noop",
			base:   h.NamedRestriction("noop1"),
			second: h.NamedRestriction("noop2"),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop1", coins), h.NewArgs("noop2", coins)),
			},
		},
		{
			name:   "noop error",
			base:   h.NamedRestriction("noop"),
			second: h.ErrorRestriction("this is a test error"),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "this is a test error",
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins), h.NewArgs("this is a test error", coins)),
			},
		},
		{
			name:   "error noop",
			base:   h.ErrorRestriction("another test error"),
			second: h.NamedRestriction("noop"),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "another test error",
				ExpCalls: h.NewCalls(h.NewArgs("another test error", coins)),
			},
		},
		{
			name:   "error error",
			base:   h.ErrorRestriction("first test error"),
			second: h.ErrorRestriction("second test error"),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "first test error",
				ExpCalls: h.NewCalls(h.NewArgs("first test error", coins)),
			},
		},
		{
			name:   "double chain",
			base:   types.ComposeMintingRestrictions(h.NamedRestriction("r1"), h.NamedRestriction("r2")),
			second: types.ComposeMintingRestrictions(h.NamedRestriction("r3"), h.NamedRestriction("r4")),
			exp: &MintingRestrictionTestParams{
				Coins: coins,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", coins),
					h.NewArgs("r2", coins),
					h.NewArgs("r3", coins),
					h.NewArgs("r4", coins),
				),
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var actual types.MintingRestrictionFn
			testFunc := func() {
				actual = tc.base.Then(tc.second)
			}
			require.NotPanics(t, testFunc, "MintingRestrictionFn.Then")
			h.TestActual(t, tc.exp, actual)
		})
	}
}

func TestComposeMintingRestrictions(t *testing.T) {
	rz := func(rs ...types.MintingRestrictionFn) []types.MintingRestrictionFn {
		return rs
	}
	coins := sdk.NewCoins(sdk.NewInt64Coin("ccoin", 8), sdk.NewInt64Coin("dcoin", 16))

	h := NewMintingRestrictionTestHelper()

	tests := []struct {
		name  string
		input []types.MintingRestrictionFn
		exp   *MintingRestrictionTestParams
	}{
		{
			name:  "nil list",
			input: nil,
			exp: &MintingRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "empty list",
			input: rz(),
			exp: &MintingRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "only nil entry",
			input: rz(nil),
			exp: &MintingRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "five nil entries",
			input: rz(nil, nil, nil, nil, nil),
			exp: &MintingRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "only noop entry",
			input: rz(h.NamedRestriction("noop")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:  "only error entry",
			input: rz(h.ErrorRestriction("test error")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "test error",
				ExpCalls: h.NewCalls(h.NewArgs("test error", coins)),
			},
		},
		{
			name:  "noop nil nil",
			input: rz(h.NamedRestriction("noop"), nil, nil),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:  "nil noop nil",
			input: rz(nil, h.NamedRestriction("noop"), nil),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:  "nil nil noop",
			input: rz(nil, nil, h.NamedRestriction("noop")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("noop", coins)),
			},
		},
		{
			name:  "noop noop nil",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), nil),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("r2", coins)),
			},
		},
		{
			name:  "noop nil noop",
			input: rz(h.NamedRestriction("r1"), nil, h.NamedRestriction("r2")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("r2", coins)),
			},
		},
		{
			name:  "nil noop noop",
			input: rz(nil, h.NamedRestriction("r1"), h.NamedRestriction("r2")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("r2", coins)),
			},
		},
		{
			name:  "noop noop noop",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), h.NamedRestriction("r3")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("r2", coins), h.NewArgs("r3", coins)),
			},
		},
		{
			name:  "err noop noop",
			input: rz(h.ErrorRestriction("first error"), h.NamedRestriction("r2"), h.NamedRestriction("r3")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "first error",
				ExpCalls: h.NewCalls(h.NewArgs("first error", coins)),
			},
		},
		{
			name:  "noop err noop",
			input: rz(h.NamedRestriction("r1"), h.ErrorRestriction("second error"), h.NamedRestriction("r3")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "second error",
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("second error", coins)),
			},
		},
		{
			name:  "noop noop err",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), h.ErrorRestriction("third error")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "third error",
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("r2", coins), h.NewArgs("third error", coins)),
			},
		},
		{
			name:  "noop err err",
			input: rz(h.NamedRestriction("r1"), h.ErrorRestriction("second error"), h.ErrorRestriction("third error")),
			exp: &MintingRestrictionTestParams{
				Coins:    coins,
				ExpErr:   "second error",
				ExpCalls: h.NewCalls(h.NewArgs("r1", coins), h.NewArgs("second error", coins)),
			},
		},
		{
			name: "big bang",
			input: rz(
				h.NamedRestriction("r1"), nil, h.NamedRestriction("r2"), nil,
				h.NamedRestriction("r3"), h.NamedRestriction("r4"), h.NamedRestriction("r5"),
				nil, h.NamedRestriction("r6"), h.NamedRestriction("r7"), nil,
				h.NamedRestriction("r8"), nil, nil, h.ErrorRestriction("oops, an error"),
				h.NamedRestriction("r9"), nil, h.NamedRestriction("ra"), // Not called.
			),
			exp: &MintingRestrictionTestParams{
				Coins:  coins,
				ExpErr: "oops, an error",
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", coins),
					h.NewArgs("r2", coins),
					h.NewArgs("r3", coins),
					h.NewArgs("r4", coins),
					h.NewArgs("r5", coins),
					h.NewArgs("r6", coins),
					h.NewArgs("r7", coins),
					h.NewArgs("r8", coins),
					h.NewArgs("oops, an error", coins),
				),
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var actual types.MintingRestrictionFn
			testFunc := func() {
				actual = types.ComposeMintingRestrictions(tc.input...)
			}
			require.NotPanics(t, testFunc, "ComposeMintingRestrictions")
			h.TestActual(t, tc.exp, actual)
		})
	}
}

func TestNoOpMintingRestrictionFn(t *testing.T) {
	var err error
	testFunc := func() {
		err = types.NoOpMintingRestrictionFn(sdk.Context{}, sdk.Coins{})
	}
	require.NotPanics(t, testFunc, "NoOpMintingRestrictionFn")
	assert.NoError(t, err, "NoOpSendRestrictionFn error")
}

// SendRestrictionArgs are the args provided to a SendRestrictionFn function.
type SendRestrictionArgs struct {
	Name     string
	FromAddr sdk.AccAddress
	ToAddr   sdk.AccAddress
	Coins    sdk.Coins
}

// SendRestrictionTestHelper is a struct with stuff helpful for testing the SendRestrictionFn stuff.
type SendRestrictionTestHelper struct {
	Calls []*SendRestrictionArgs
}

func NewSendRestrictionTestHelper() *SendRestrictionTestHelper {
	return &SendRestrictionTestHelper{Calls: make([]*SendRestrictionArgs, 0, 2)}
}

// RecordCall makes note that the provided args were used as a function call.
func (s *SendRestrictionTestHelper) RecordCall(name string, fromAddr, toAddr sdk.AccAddress, coins sdk.Coins) {
	s.Calls = append(s.Calls, s.NewArgs(name, fromAddr, toAddr, coins))
}

// NewCalls is just a shorter way to create a []*SendRestrictionArgs.
func (s *SendRestrictionTestHelper) NewCalls(args ...*SendRestrictionArgs) []*SendRestrictionArgs {
	return args
}

// NewArgs creates a new SendRestrictionArgs.
func (s *SendRestrictionTestHelper) NewArgs(name string, fromAddr, toAddr sdk.AccAddress, coins sdk.Coins) *SendRestrictionArgs {
	return &SendRestrictionArgs{
		Name:     name,
		FromAddr: fromAddr,
		ToAddr:   toAddr,
		Coins:    coins,
	}
}

// NamedRestriction creates a new SendRestrictionFn function that records the arguments it's called with and returns the provided toAddr.
func (s *SendRestrictionTestHelper) NamedRestriction(name string) types.SendRestrictionFn {
	return func(_ context.Context, fromAddr, toAddr sdk.AccAddress, coins sdk.Coins) (sdk.AccAddress, error) {
		s.RecordCall(name, fromAddr, toAddr, coins)
		return toAddr, nil
	}
}

// NewToRestriction creates a new SendRestrictionFn function that returns a different toAddr than provided.
func (s *SendRestrictionTestHelper) NewToRestriction(name string, addr sdk.AccAddress) types.SendRestrictionFn {
	return func(_ context.Context, fromAddr, toAddr sdk.AccAddress, coins sdk.Coins) (sdk.AccAddress, error) {
		s.RecordCall(name, fromAddr, toAddr, coins)
		return addr, nil
	}
}

// ErrorRestriction creates a new SendRestrictionFn function that returns a nil toAddr and an error.
func (s *SendRestrictionTestHelper) ErrorRestriction(message string) types.SendRestrictionFn {
	return func(_ context.Context, fromAddr, toAddr sdk.AccAddress, coins sdk.Coins) (sdk.AccAddress, error) {
		s.RecordCall(message, fromAddr, toAddr, coins)
		return nil, errors.New(message)
	}
}

// SendRestrictionTestParams are parameters to test regarding calling a SendRestrictionFn.
type SendRestrictionTestParams struct {
	// ExpNil is whether to expect the provided SendRestrictionFn to be nil.
	// If it is true, the rest of these test params are ignored.
	ExpNil bool
	// FromAddr is the SendRestrictionFn fromAddr input.
	FromAddr sdk.AccAddress
	// ToAddr is the SendRestrictionFn toAddr input.
	ToAddr sdk.AccAddress
	// Coins is the SendRestrictionFn coins input.
	Coins sdk.Coins
	// ExpAddr is the expected return address.
	ExpAddr sdk.AccAddress
	// ExpErr is the expected return error string.
	ExpErr string
	// ExpCalls is the args of all the SendRestrictionFn calls that end up being made.
	ExpCalls []*SendRestrictionArgs
}

// TestActual tests the provided SendRestrictionFn using the provided test parameters.
func (s *SendRestrictionTestHelper) TestActual(t *testing.T, tp *SendRestrictionTestParams, actual types.SendRestrictionFn) {
	t.Helper()
	if tp.ExpNil {
		require.Nil(t, actual, "resulting SendRestrictionFn")
	} else {
		require.NotNil(t, actual, "resulting SendRestrictionFn")
		s.Calls = s.Calls[:0]
		addr, err := actual(sdk.Context{}, tp.FromAddr, tp.ToAddr, tp.Coins)
		if len(tp.ExpErr) != 0 {
			assert.EqualError(t, err, tp.ExpErr, "composite SendRestrictionFn output error")
		} else {
			assert.NoError(t, err, "composite SendRestrictionFn output error")
		}
		assert.Equal(t, tp.ExpAddr, addr, "composite SendRestrictionFn output address")
		assert.Equal(t, tp.ExpCalls, s.Calls, "args given to funcs in composite SendRestrictionFn")
	}
}

func TestSendRestriction_Then(t *testing.T) {
	fromAddr := sdk.AccAddress("fromaddr____________")
	addr0 := sdk.AccAddress("0addr_______________")
	addr1 := sdk.AccAddress("1addr_______________")
	addr2 := sdk.AccAddress("2addr_______________")
	addr3 := sdk.AccAddress("3addr_______________")
	addr4 := sdk.AccAddress("4addr_______________")
	coins := sdk.NewCoins(sdk.NewInt64Coin("ecoin", 32), sdk.NewInt64Coin("fcoin", 64))

	h := NewSendRestrictionTestHelper()

	tests := []struct {
		name   string
		base   types.SendRestrictionFn
		second types.SendRestrictionFn
		exp    *SendRestrictionTestParams
	}{
		{
			name:   "nil nil",
			base:   nil,
			second: nil,
			exp: &SendRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:   "nil noop",
			base:   nil,
			second: h.NamedRestriction("noop"),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr1,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr1, coins)),
			},
		},
		{
			name:   "noop nil",
			base:   h.NamedRestriction("noop"),
			second: nil,
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr1,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr1, coins)),
			},
		},
		{
			name:   "noop noop",
			base:   h.NamedRestriction("noop1"),
			second: h.NamedRestriction("noop2"),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr1,
				ExpCalls: h.NewCalls(
					h.NewArgs("noop1", fromAddr, addr1, coins),
					h.NewArgs("noop2", fromAddr, addr1, coins),
				),
			},
		},
		{
			name:   "setter setter",
			base:   h.NewToRestriction("r1", addr2),
			second: h.NewToRestriction("r2", addr3),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr3,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr1, coins),
					h.NewArgs("r2", fromAddr, addr2, coins),
				),
			},
		},
		{
			name:   "setter error",
			base:   h.NewToRestriction("r1", addr2),
			second: h.ErrorRestriction("this is a test error"),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "this is a test error",
				ExpCalls: h.NewCalls(h.NewArgs(
					"r1", fromAddr, addr1, coins),
					h.NewArgs("this is a test error", fromAddr, addr2, coins),
				),
			},
		},
		{
			name:   "error setter",
			base:   h.ErrorRestriction("another test error"),
			second: h.NewToRestriction("r2", addr3),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "another test error",
				ExpCalls: h.NewCalls(h.NewArgs("another test error", fromAddr, addr1, coins)),
			},
		},
		{
			name:   "error error",
			base:   h.ErrorRestriction("first test error"),
			second: h.ErrorRestriction("second test error"),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "first test error",
				ExpCalls: h.NewCalls(h.NewArgs("first test error", fromAddr, addr1, coins)),
			},
		},
		{
			name:   "double chain",
			base:   types.ComposeSendRestrictions(h.NewToRestriction("r1", addr1), h.NewToRestriction("r2", addr2)),
			second: types.ComposeSendRestrictions(h.NewToRestriction("r3", addr3), h.NewToRestriction("r4", addr4)),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  addr4,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr0, coins),
					h.NewArgs("r2", fromAddr, addr1, coins),
					h.NewArgs("r3", fromAddr, addr2, coins),
					h.NewArgs("r4", fromAddr, addr3, coins),
				),
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var actual types.SendRestrictionFn
			testFunc := func() {
				actual = tc.base.Then(tc.second)
			}
			require.NotPanics(t, testFunc, "SendRestrictionFn.Then")
			h.TestActual(t, tc.exp, actual)
		})
	}
}

func TestComposeSendRestrictions(t *testing.T) {
	rz := func(rs ...types.SendRestrictionFn) []types.SendRestrictionFn {
		return rs
	}
	fromAddr := sdk.AccAddress("fromaddr____________")
	addr0 := sdk.AccAddress("0addr_______________")
	addr1 := sdk.AccAddress("1addr_______________")
	addr2 := sdk.AccAddress("2addr_______________")
	addr3 := sdk.AccAddress("3addr_______________")
	addr4 := sdk.AccAddress("4addr_______________")
	coins := sdk.NewCoins(sdk.NewInt64Coin("gcoin", 128), sdk.NewInt64Coin("hcoin", 256))

	h := NewSendRestrictionTestHelper()

	tests := []struct {
		name  string
		input []types.SendRestrictionFn
		exp   *SendRestrictionTestParams
	}{
		{
			name:  "nil list",
			input: nil,
			exp: &SendRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "empty list",
			input: rz(),
			exp: &SendRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "only nil entry",
			input: rz(nil),
			exp: &SendRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "five nil entries",
			input: rz(nil, nil, nil, nil, nil),
			exp: &SendRestrictionTestParams{
				ExpNil: true,
			},
		},
		{
			name:  "only noop entry",
			input: rz(h.NamedRestriction("noop")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  addr0,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr0, coins)),
			},
		},
		{
			name:  "only error entry",
			input: rz(h.ErrorRestriction("test error")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "test error",
				ExpCalls: h.NewCalls(h.NewArgs("test error", fromAddr, addr0, coins)),
			},
		},
		{
			name:  "noop nil nil",
			input: rz(h.NamedRestriction("noop"), nil, nil),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  addr0,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr0, coins)),
			},
		},
		{
			name:  "nil noop nil",
			input: rz(nil, h.NamedRestriction("noop"), nil),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr1,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr1, coins)),
			},
		},
		{
			name:  "nil nil noop",
			input: rz(nil, nil, h.NamedRestriction("noop")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr2,
				Coins:    coins,
				ExpAddr:  addr2,
				ExpCalls: h.NewCalls(h.NewArgs("noop", fromAddr, addr2, coins)),
			},
		},
		{
			name:  "noop noop nil",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), nil),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  addr0,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr0, coins),
					h.NewArgs("r2", fromAddr, addr0, coins),
				),
			},
		},
		{
			name:  "noop nil noop",
			input: rz(h.NamedRestriction("r1"), nil, h.NamedRestriction("r2")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr1,
				Coins:    coins,
				ExpAddr:  addr1,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr1, coins),
					h.NewArgs("r2", fromAddr, addr1, coins),
				),
			},
		},
		{
			name:  "nil noop noop",
			input: rz(nil, h.NamedRestriction("r1"), h.NamedRestriction("r2")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr2,
				Coins:    coins,
				ExpAddr:  addr2,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr2, coins),
					h.NewArgs("r2", fromAddr, addr2, coins),
				),
			},
		},
		{
			name:  "noop noop noop",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), h.NamedRestriction("r3")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr3,
				Coins:    coins,
				ExpAddr:  addr3,
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr3, coins),
					h.NewArgs("r2", fromAddr, addr3, coins),
					h.NewArgs("r3", fromAddr, addr3, coins),
				),
			},
		},
		{
			name:  "err noop noop",
			input: rz(h.ErrorRestriction("first error"), h.NamedRestriction("r2"), h.NamedRestriction("r3")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr4,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "first error",
				ExpCalls: h.NewCalls(h.NewArgs("first error", fromAddr, addr4, coins)),
			},
		},
		{
			name:  "noop err noop",
			input: rz(h.NamedRestriction("r1"), h.ErrorRestriction("second error"), h.NamedRestriction("r3")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr4,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "second error",
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr4, coins),
					h.NewArgs("second error", fromAddr, addr4, coins),
				),
			},
		},
		{
			name:  "noop noop err",
			input: rz(h.NamedRestriction("r1"), h.NamedRestriction("r2"), h.ErrorRestriction("third error")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr4,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "third error",
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr4, coins),
					h.NewArgs("r2", fromAddr, addr4, coins),
					h.NewArgs("third error", fromAddr, addr4, coins),
				),
			},
		},
		{
			name:  "new-to err err",
			input: rz(h.NewToRestriction("r1", addr0), h.ErrorRestriction("second error"), h.ErrorRestriction("third error")),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr4,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "second error",
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr4, coins),
					h.NewArgs("second error", fromAddr, addr0, coins),
				),
			},
		},
		{
			name: "big bang",
			input: rz(
				h.NamedRestriction("r1"), nil, h.NewToRestriction("r2", addr1), // Called with orig toAddr.
				nil, h.NamedRestriction("r3"), h.NewToRestriction("r4", addr2), // Called with addr1 toAddr.
				h.NewToRestriction("r5", addr3),                                // Called with addr2 toAddr.
				nil, h.NamedRestriction("r6"), h.NewToRestriction("r7", addr4), // Called with addr3 toAddr.
				nil, h.NamedRestriction("r8"), nil, nil, h.ErrorRestriction("oops, an error"), // Called with addr4 toAddr.
				h.NewToRestriction("r9", addr0), nil, h.NamedRestriction("ra"), // Not called.
			),
			exp: &SendRestrictionTestParams{
				FromAddr: fromAddr,
				ToAddr:   addr0,
				Coins:    coins,
				ExpAddr:  nil,
				ExpErr:   "oops, an error",
				ExpCalls: h.NewCalls(
					h.NewArgs("r1", fromAddr, addr0, coins),
					h.NewArgs("r2", fromAddr, addr0, coins),
					h.NewArgs("r3", fromAddr, addr1, coins),
					h.NewArgs("r4", fromAddr, addr1, coins),
					h.NewArgs("r5", fromAddr, addr2, coins),
					h.NewArgs("r6", fromAddr, addr3, coins),
					h.NewArgs("r7", fromAddr, addr3, coins),
					h.NewArgs("r8", fromAddr, addr4, coins),
					h.NewArgs("oops, an error", fromAddr, addr4, coins),
				),
			},
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			var actual types.SendRestrictionFn
			testFunc := func() {
				actual = types.ComposeSendRestrictions(tc.input...)
			}
			require.NotPanics(t, testFunc, "ComposeSendRestrictions")
			h.TestActual(t, tc.exp, actual)
		})
	}
}

func TestNoOpSendRestrictionFn(t *testing.T) {
	expAddr := sdk.AccAddress("__expectedaddr__")
	var addr sdk.AccAddress
	var err error
	testFunc := func() {
		addr, err = types.NoOpSendRestrictionFn(sdk.Context{}, sdk.AccAddress("first_addr"), expAddr, sdk.Coins{})
	}
	require.NotPanics(t, testFunc, "NoOpSendRestrictionFn")
	assert.NoError(t, err, "NoOpSendRestrictionFn error")
	assert.Equal(t, expAddr, addr, "NoOpSendRestrictionFn addr")
}
