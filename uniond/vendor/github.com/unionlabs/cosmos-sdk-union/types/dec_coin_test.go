package types_test

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/suite"

	"cosmossdk.io/math"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

type decCoinTestSuite struct {
	suite.Suite
}

func TestDecCoinTestSuite(t *testing.T) {
	suite.Run(t, new(decCoinTestSuite))
}

func (s *decCoinTestSuite) TestNewDecCoin() {
	s.Require().NotPanics(func() {
		sdk.NewInt64DecCoin(testDenom1, 5)
	})
	s.Require().NotPanics(func() {
		sdk.NewInt64DecCoin(testDenom1, 0)
	})
	s.Require().NotPanics(func() {
		sdk.NewInt64DecCoin(strings.ToUpper(testDenom1), 5)
	})
	s.Require().Panics(func() {
		sdk.NewInt64DecCoin(testDenom1, -5)
	})
}

func (s *decCoinTestSuite) TestNewDecCoinFromDec() {
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromDec(testDenom1, math.LegacyNewDec(5))
	})
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromDec(testDenom1, math.LegacyZeroDec())
	})
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromDec(strings.ToUpper(testDenom1), math.LegacyNewDec(5))
	})
	s.Require().Panics(func() {
		sdk.NewDecCoinFromDec(testDenom1, math.LegacyNewDec(-5))
	})
}

func (s *decCoinTestSuite) TestNewDecCoinFromCoin() {
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromCoin(sdk.Coin{testDenom1, math.NewInt(5)})
	})
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromCoin(sdk.Coin{testDenom1, math.NewInt(0)})
	})
	s.Require().NotPanics(func() {
		sdk.NewDecCoinFromCoin(sdk.Coin{strings.ToUpper(testDenom1), math.NewInt(5)})
	})
	s.Require().Panics(func() {
		sdk.NewDecCoinFromCoin(sdk.Coin{testDenom1, math.NewInt(-5)})
	})
}

func (s *decCoinTestSuite) TestDecCoinIsPositive() {
	dc := sdk.NewInt64DecCoin(testDenom1, 5)
	s.Require().True(dc.IsPositive())

	dc = sdk.NewInt64DecCoin(testDenom1, 0)
	s.Require().False(dc.IsPositive())
}

func (s *decCoinTestSuite) TestAddDecCoin() {
	decCoinA1 := sdk.NewDecCoinFromDec(testDenom1, math.LegacyNewDecWithPrec(11, 1))
	decCoinA2 := sdk.NewDecCoinFromDec(testDenom1, math.LegacyNewDecWithPrec(22, 1))
	decCoinB1 := sdk.NewDecCoinFromDec(testDenom2, math.LegacyNewDecWithPrec(11, 1))

	// regular add
	res := decCoinA1.Add(decCoinA1)
	s.Require().Equal(decCoinA2, res, "sum of coins is incorrect")

	// bad denom add
	s.Require().Panics(func() {
		decCoinA1.Add(decCoinB1)
	}, "expected panic on sum of different denoms")
}

func (s *decCoinTestSuite) TestAddDecCoins() {
	one := math.LegacyNewDec(1)
	zero := math.LegacyNewDec(0)
	two := math.LegacyNewDec(2)

	cases := []struct {
		inputOne sdk.DecCoins
		inputTwo sdk.DecCoins
		expected sdk.DecCoins
	}{
		{sdk.DecCoins{{testDenom1, one}, {testDenom2, one}}, sdk.DecCoins{{testDenom1, one}, {testDenom2, one}}, sdk.DecCoins{{testDenom1, two}, {testDenom2, two}}},
		{sdk.DecCoins{{testDenom1, zero}, {testDenom2, one}}, sdk.DecCoins{{testDenom1, zero}, {testDenom2, zero}}, sdk.DecCoins{{testDenom2, one}}},
		{sdk.DecCoins{{testDenom1, zero}, {testDenom2, zero}}, sdk.DecCoins{{testDenom1, zero}, {testDenom2, zero}}, sdk.DecCoins{}},
	}

	for tcIndex, tc := range cases {
		res := tc.inputOne.Add(tc.inputTwo...)
		s.Require().Equal(tc.expected, res, "sum of coins is incorrect, tc #%d", tcIndex)
	}
}

func (s *decCoinTestSuite) TestFilteredZeroDecCoins() {
	cases := []struct {
		name     string
		input    sdk.DecCoins
		original string
		expected string
		panic    bool
	}{
		{
			name: "all greater than zero",
			input: sdk.DecCoins{
				{"testa", math.LegacyNewDec(1)},
				{"testb", math.LegacyNewDec(2)},
				{"testc", math.LegacyNewDec(3)},
				{"testd", math.LegacyNewDec(4)},
				{"teste", math.LegacyNewDec(5)},
			},
			original: "1.000000000000000000testa,2.000000000000000000testb,3.000000000000000000testc,4.000000000000000000testd,5.000000000000000000teste",
			expected: "1.000000000000000000testa,2.000000000000000000testb,3.000000000000000000testc,4.000000000000000000testd,5.000000000000000000teste",
			panic:    false,
		},
		{
			name: "zero coin in middle",
			input: sdk.DecCoins{
				{"testa", math.LegacyNewDec(1)},
				{"testb", math.LegacyNewDec(2)},
				{"testc", math.LegacyNewDec(0)},
				{"testd", math.LegacyNewDec(4)},
				{"teste", math.LegacyNewDec(5)},
			},
			original: "1.000000000000000000testa,2.000000000000000000testb,0.000000000000000000testc,4.000000000000000000testd,5.000000000000000000teste",
			expected: "1.000000000000000000testa,2.000000000000000000testb,4.000000000000000000testd,5.000000000000000000teste",
			panic:    false,
		},
		{
			name: "zero coin end (unordered)",
			input: sdk.DecCoins{
				{"teste", math.LegacyNewDec(5)},
				{"testc", math.LegacyNewDec(3)},
				{"testa", math.LegacyNewDec(1)},
				{"testd", math.LegacyNewDec(4)},
				{"testb", math.LegacyNewDec(0)},
			},
			original: "5.000000000000000000teste,3.000000000000000000testc,1.000000000000000000testa,4.000000000000000000testd,0.000000000000000000testb",
			expected: "1.000000000000000000testa,3.000000000000000000testc,4.000000000000000000testd,5.000000000000000000teste",
			panic:    false,
		},

		{
			name: "panic when same denoms in multiple coins",
			input: sdk.DecCoins{
				{"testa", math.LegacyNewDec(5)},
				{"testa", math.LegacyNewDec(3)},
				{"testa", math.LegacyNewDec(1)},
				{"testd", math.LegacyNewDec(4)},
				{"testb", math.LegacyNewDec(2)},
			},
			original: "5.000000000000000000teste,3.000000000000000000testc,1.000000000000000000testa,4.000000000000000000testd,0.000000000000000000testb",
			expected: "1.000000000000000000testa,3.000000000000000000testc,4.000000000000000000testd,5.000000000000000000teste",
			panic:    true,
		},
	}

	for _, tt := range cases {
		if tt.panic {
			s.Require().Panics(func() { sdk.NewDecCoins(tt.input...) }, "Should panic due to multiple coins with same denom")
		} else {
			undertest := sdk.NewDecCoins(tt.input...)
			s.Require().Equal(tt.expected, undertest.String(), "NewDecCoins must return expected results")
			s.Require().Equal(tt.original, tt.input.String(), "input must be unmodified and match original")
		}
	}
}

func (s *decCoinTestSuite) TestIsValid() {
	tests := []struct {
		coin       sdk.DecCoin
		expectPass bool
		msg        string
	}{
		{
			sdk.NewDecCoin("mytoken", math.NewInt(10)),
			true,
			"valid coins should have passed",
		},
		{
			sdk.DecCoin{Denom: "BTC", Amount: math.LegacyNewDec(10)},
			true,
			"valid uppercase denom",
		},
		{
			sdk.DecCoin{Denom: "Bitcoin", Amount: math.LegacyNewDec(10)},
			true,
			"valid mixed case denom",
		},
		{
			sdk.DecCoin{Denom: "btc", Amount: math.LegacyNewDec(-10)},
			false,
			"negative amount",
		},
	}

	for _, tc := range tests {
		if tc.expectPass {
			s.Require().True(tc.coin.IsValid(), tc.msg)
		} else {
			s.Require().False(tc.coin.IsValid(), tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestSubDecCoin() {
	tests := []struct {
		coin       sdk.DecCoin
		expectPass bool
		msg        string
	}{
		{
			sdk.NewDecCoin("mytoken", math.NewInt(20)),
			true,
			"valid coins should have passed",
		},
		{
			sdk.NewDecCoin("othertoken", math.NewInt(20)),
			false,
			"denom mismatch",
		},
		{
			sdk.NewDecCoin("mytoken", math.NewInt(9)),
			false,
			"negative amount",
		},
	}

	decCoin := sdk.NewDecCoin("mytoken", math.NewInt(10))

	for _, tc := range tests {
		if tc.expectPass {
			equal := tc.coin.Sub(decCoin)
			s.Require().Equal(equal, decCoin, tc.msg)
		} else {
			s.Require().Panics(func() { tc.coin.Sub(decCoin) }, tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestSubDecCoins() {
	tests := []struct {
		coins      sdk.DecCoins
		expectPass bool
		msg        string
	}{
		{
			sdk.NewDecCoinsFromCoins(sdk.NewCoin("mytoken", math.NewInt(10)), sdk.NewCoin("btc", math.NewInt(20)), sdk.NewCoin("eth", math.NewInt(30))),
			true,
			"sorted coins should have passed",
		},
		{
			sdk.DecCoins{sdk.NewDecCoin("mytoken", math.NewInt(10)), sdk.NewDecCoin("btc", math.NewInt(20)), sdk.NewDecCoin("eth", math.NewInt(30))},
			false,
			"unorted coins should panic",
		},
		{
			sdk.DecCoins{sdk.DecCoin{Denom: "BTC", Amount: math.LegacyNewDec(10)}, sdk.NewDecCoin("eth", math.NewInt(15)), sdk.NewDecCoin("mytoken", math.NewInt(5))},
			false,
			"invalid denoms",
		},
	}

	decCoins := sdk.NewDecCoinsFromCoins(sdk.NewCoin("btc", math.NewInt(10)), sdk.NewCoin("eth", math.NewInt(15)), sdk.NewCoin("mytoken", math.NewInt(5)))

	for _, tc := range tests {
		if tc.expectPass {
			equal := tc.coins.Sub(decCoins)
			s.Require().Equal(equal, decCoins, tc.msg)
		} else {
			s.Require().Panics(func() { tc.coins.Sub(decCoins) }, tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestSortDecCoins() {
	good := sdk.DecCoins{
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("mineral", 1),
		sdk.NewInt64DecCoin("tree", 1),
	}
	empty := sdk.DecCoins{
		sdk.NewInt64DecCoin("gold", 0),
	}
	badSort1 := sdk.DecCoins{
		sdk.NewInt64DecCoin("tree", 1),
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("mineral", 1),
	}
	badSort2 := sdk.DecCoins{ // both are after the first one, but the second and third are in the wrong order
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("tree", 1),
		sdk.NewInt64DecCoin("mineral", 1),
	}
	badAmt := sdk.DecCoins{
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("tree", 0),
		sdk.NewInt64DecCoin("mineral", 1),
	}
	dup := sdk.DecCoins{
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("gas", 1),
		sdk.NewInt64DecCoin("mineral", 1),
	}
	cases := []struct {
		name          string
		coins         sdk.DecCoins
		before, after bool // valid before/after sort
	}{
		{"valid coins", good, true, true},
		{"empty coins", empty, false, false},
		{"unsorted coins (1)", badSort1, false, true},
		{"unsorted coins (2)", badSort2, false, true},
		{"zero amount coins", badAmt, false, false},
		{"duplicate coins", dup, false, false},
	}

	for _, tc := range cases {
		s.Require().Equal(tc.before, tc.coins.IsValid(), "coin validity is incorrect before sorting; %s", tc.name)
		tc.coins.Sort()
		s.Require().Equal(tc.after, tc.coins.IsValid(), "coin validity is incorrect after sorting;  %s", tc.name)
	}
}

func (s *decCoinTestSuite) TestDecCoinsValidate() {
	testCases := []struct {
		input        sdk.DecCoins
		expectedPass bool
	}{
		{sdk.DecCoins{}, true},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, true},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(100000)}}, true},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(-5)}}, false},
		{sdk.DecCoins{sdk.DecCoin{"BTC", math.LegacyNewDec(5)}}, true},
		{sdk.DecCoins{sdk.DecCoin{"0BTC", math.LegacyNewDec(5)}}, false},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}, sdk.DecCoin{"B", math.LegacyNewDec(100000)}}, false},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(-100000)}}, false},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(-5)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(100000)}}, false},
		{sdk.DecCoins{sdk.DecCoin{"BTC", math.LegacyNewDec(5)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(100000)}}, true},
		{sdk.DecCoins{sdk.DecCoin{"0BTC", math.LegacyNewDec(5)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(100000)}}, false},
	}

	for i, tc := range testCases {
		err := tc.input.Validate()
		if tc.expectedPass {
			s.Require().NoError(err, "unexpected result for test case #%d, input: %v", i, tc.input)
		} else {
			s.Require().Error(err, "unexpected result for test case #%d, input: %v", i, tc.input)
		}
	}
}

func (s *decCoinTestSuite) TestParseDecCoins() {
	testCases := []struct {
		input          string
		expectedResult sdk.DecCoins
		expectedErr    bool
	}{
		{"", nil, false},
		{"4stake", sdk.DecCoins{sdk.NewDecCoinFromDec("stake", math.LegacyNewDecFromInt(math.NewInt(4)))}, false},
		{"5.5atom", sdk.DecCoins{
			sdk.NewDecCoinFromDec("atom", math.LegacyNewDecWithPrec(5500000000000000000, math.LegacyPrecision)),
		}, false},
		{"5.5atom,4stake", sdk.DecCoins{
			sdk.NewDecCoinFromDec("atom", math.LegacyNewDecWithPrec(5500000000000000000, math.LegacyPrecision)),
			sdk.NewDecCoinFromDec("stake", math.LegacyNewDec(4)),
		}, false},
		{"0.0stake", sdk.DecCoins{}, false}, // remove zero coins
		{"10.0btc,1.0atom,20.0btc", nil, true},
		{
			"0.004STAKE",
			sdk.DecCoins{sdk.NewDecCoinFromDec("STAKE", math.LegacyNewDecWithPrec(4000000000000000, math.LegacyPrecision))},
			false,
		},
		{
			"0.004stake",
			sdk.DecCoins{sdk.NewDecCoinFromDec("stake", math.LegacyNewDecWithPrec(4000000000000000, math.LegacyPrecision))},
			false,
		},
		{
			"5.04atom,0.004stake",
			sdk.DecCoins{
				sdk.NewDecCoinFromDec("atom", math.LegacyNewDecWithPrec(5040000000000000000, math.LegacyPrecision)),
				sdk.NewDecCoinFromDec("stake", math.LegacyNewDecWithPrec(4000000000000000, math.LegacyPrecision)),
			},
			false,
		},
		{
			"0.0stake,0.004stake,5.04atom", // remove zero coins
			sdk.DecCoins{
				sdk.NewDecCoinFromDec("atom", math.LegacyNewDecWithPrec(5040000000000000000, math.LegacyPrecision)),
				sdk.NewDecCoinFromDec("stake", math.LegacyNewDecWithPrec(4000000000000000, math.LegacyPrecision)),
			},
			false,
		},
	}

	for i, tc := range testCases {
		res, err := sdk.ParseDecCoins(tc.input)
		if tc.expectedErr {
			s.Require().Error(err, "expected error for test case #%d, input: %v", i, tc.input)
		} else {
			s.Require().NoError(err, "unexpected error for test case #%d, input: %v", i, tc.input)
			s.Require().Equal(tc.expectedResult, res, "unexpected result for test case #%d, input: %v", i, tc.input)
		}
	}
}

func (s *decCoinTestSuite) TestDecCoinsString() {
	testCases := []struct {
		input    sdk.DecCoins
		expected string
	}{
		{sdk.DecCoins{}, ""},
		{
			sdk.DecCoins{
				sdk.NewDecCoinFromDec("atom", math.LegacyNewDecWithPrec(5040000000000000000, math.LegacyPrecision)),
				sdk.NewDecCoinFromDec("stake", math.LegacyNewDecWithPrec(4000000000000000, math.LegacyPrecision)),
			},
			"5.040000000000000000atom,0.004000000000000000stake",
		},
	}

	for i, tc := range testCases {
		out := tc.input.String()
		s.Require().Equal(tc.expected, out, "unexpected result for test case #%d, input: %v", i, tc.input)
	}
}

func (s *decCoinTestSuite) TestDecCoinsIntersect() {
	testCases := []struct {
		input1         string
		input2         string
		expectedResult string
	}{
		{"", "", ""},
		{"1.0stake", "", ""},
		{"1.0stake", "1.0stake", "1.0stake"},
		{"", "1.0stake", ""},
		{"1.0stake", "", ""},
		{"2.0stake,1.0trope", "1.9stake", "1.9stake"},
		{"2.0stake,1.0trope", "2.1stake", "2.0stake"},
		{"2.0stake,1.0trope", "0.9trope", "0.9trope"},
		{"2.0stake,1.0trope", "1.9stake,0.9trope", "1.9stake,0.9trope"},
		{"2.0stake,1.0trope", "1.9stake,0.9trope,20.0other", "1.9stake,0.9trope"},
		{"2.0stake,1.0trope", "1.0other", ""},
	}

	for i, tc := range testCases {
		in1, err := sdk.ParseDecCoins(tc.input1)
		s.Require().NoError(err, "unexpected parse error in %v", i)
		in2, err := sdk.ParseDecCoins(tc.input2)
		s.Require().NoError(err, "unexpected parse error in %v", i)
		exr, err := sdk.ParseDecCoins(tc.expectedResult)
		s.Require().NoError(err, "unexpected parse error in %v", i)
		s.Require().True(in1.Intersect(in2).Equal(exr), "in1.cap(in2) != exr in %v", i)
	}
}

func (s *decCoinTestSuite) TestDecCoinsTruncateDecimal() {
	decCoinA := sdk.NewDecCoinFromDec("bar", math.LegacyMustNewDecFromStr("5.41"))
	decCoinB := sdk.NewDecCoinFromDec("foo", math.LegacyMustNewDecFromStr("6.00"))

	testCases := []struct {
		input          sdk.DecCoins
		truncatedCoins sdk.Coins
		changeCoins    sdk.DecCoins
	}{
		{sdk.DecCoins{}, sdk.Coins(nil), sdk.DecCoins(nil)},
		{
			sdk.DecCoins{decCoinA, decCoinB},
			sdk.Coins{sdk.NewInt64Coin(decCoinA.Denom, 5), sdk.NewInt64Coin(decCoinB.Denom, 6)},
			sdk.DecCoins{sdk.NewDecCoinFromDec(decCoinA.Denom, math.LegacyMustNewDecFromStr("0.41"))},
		},
		{
			sdk.DecCoins{decCoinB},
			sdk.Coins{sdk.NewInt64Coin(decCoinB.Denom, 6)},
			sdk.DecCoins(nil),
		},
	}

	for i, tc := range testCases {
		truncatedCoins, changeCoins := tc.input.TruncateDecimal()
		s.Require().Equal(
			tc.truncatedCoins, truncatedCoins,
			"unexpected truncated coins; tc #%d, input: %s", i, tc.input,
		)
		s.Require().Equal(
			tc.changeCoins, changeCoins,
			"unexpected change coins; tc #%d, input: %s", i, tc.input,
		)
	}
}

func (s *decCoinTestSuite) TestDecCoinsQuoDecTruncate() {
	x := math.LegacyMustNewDecFromStr("1.00")
	y := math.LegacyMustNewDecFromStr("10000000000000000000.00")

	testCases := []struct {
		coins  sdk.DecCoins
		input  math.LegacyDec
		result sdk.DecCoins
		panics bool
	}{
		{sdk.DecCoins{}, math.LegacyZeroDec(), sdk.DecCoins(nil), true},
		{sdk.DecCoins{sdk.NewDecCoinFromDec("foo", x)}, y, sdk.DecCoins(nil), false},
		{sdk.DecCoins{sdk.NewInt64DecCoin("foo", 5)}, math.LegacyNewDec(2), sdk.DecCoins{sdk.NewDecCoinFromDec("foo", math.LegacyMustNewDecFromStr("2.5"))}, false},
	}

	for i, tc := range testCases {
		if tc.panics {
			s.Require().Panics(func() { tc.coins.QuoDecTruncate(tc.input) })
		} else {
			res := tc.coins.QuoDecTruncate(tc.input)
			s.Require().Equal(tc.result, res, "unexpected result; tc #%d, coins: %s, input: %s", i, tc.coins, tc.input)
		}
	}
}

func (s *decCoinTestSuite) TestNewDecCoinsWithIsValid() {
	fake1 := append(sdk.NewDecCoins(sdk.NewDecCoin("mytoken", math.NewInt(10))), sdk.DecCoin{Denom: "10BTC", Amount: math.LegacyNewDec(10)})
	fake2 := append(sdk.NewDecCoins(sdk.NewDecCoin("mytoken", math.NewInt(10))), sdk.DecCoin{Denom: "BTC", Amount: math.LegacyNewDec(-10)})

	tests := []struct {
		coin       sdk.DecCoins
		expectPass bool
		msg        string
	}{
		{
			sdk.NewDecCoins(sdk.NewDecCoin("mytoken", math.NewInt(10))),
			true,
			"valid coins should have passed",
		},
		{
			fake1,
			false,
			"invalid denoms",
		},
		{
			fake2,
			false,
			"negative amount",
		},
	}

	for _, tc := range tests {
		if tc.expectPass {
			s.Require().True(tc.coin.IsValid(), tc.msg)
		} else {
			s.Require().False(tc.coin.IsValid(), tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestNewDecCoinsWithZeroCoins() {
	zeroCoins := append(sdk.NewCoins(sdk.NewCoin("mytoken", math.NewInt(0))), sdk.Coin{Denom: "wbtc", Amount: math.NewInt(10)})

	tests := []struct {
		coins        sdk.Coins
		expectLength int
	}{
		{
			sdk.NewCoins(sdk.NewCoin("mytoken", math.NewInt(10)), sdk.NewCoin("wbtc", math.NewInt(10))),
			2,
		},
		{
			zeroCoins,
			1,
		},
	}

	for _, tc := range tests {
		s.Require().Equal(sdk.NewDecCoinsFromCoins(tc.coins...).Len(), tc.expectLength)
	}
}

func (s *decCoinTestSuite) TestDecCoins_AddDecCoinWithIsValid() {
	lengthTestDecCoins := sdk.NewDecCoins().Add(sdk.NewDecCoin("mytoken", math.NewInt(10))).Add(sdk.DecCoin{Denom: "BTC", Amount: math.LegacyNewDec(10)})
	s.Require().Equal(2, len(lengthTestDecCoins), "should be 2")

	tests := []struct {
		coin       sdk.DecCoins
		expectPass bool
		msg        string
	}{
		{
			sdk.NewDecCoins().Add(sdk.NewDecCoin("mytoken", math.NewInt(10))),
			true,
			"valid coins should have passed",
		},
		{
			sdk.NewDecCoins().Add(sdk.NewDecCoin("mytoken", math.NewInt(10))).Add(sdk.DecCoin{Denom: "0BTC", Amount: math.LegacyNewDec(10)}),
			false,
			"invalid denoms",
		},
		{
			sdk.NewDecCoins().Add(sdk.NewDecCoin("mytoken", math.NewInt(10))).Add(sdk.DecCoin{Denom: "BTC", Amount: math.LegacyNewDec(-10)}),
			false,
			"negative amount",
		},
	}

	for _, tc := range tests {
		if tc.expectPass {
			s.Require().True(tc.coin.IsValid(), tc.msg)
		} else {
			s.Require().False(tc.coin.IsValid(), tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestDecCoins_Empty() {
	testCases := []struct {
		input          sdk.DecCoins
		expectedResult bool
		msg            string
	}{
		{sdk.DecCoins{}, true, "No coins as expected."},
		{sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, false, "DecCoins is not empty"},
	}

	for _, tc := range testCases {
		if tc.expectedResult {
			s.Require().True(tc.input.Empty(), tc.msg)
		} else {
			s.Require().False(tc.input.Empty(), tc.msg)
		}
	}
}

func (s *decCoinTestSuite) TestDecCoins_GetDenomByIndex() {
	testCases := []struct {
		name           string
		input          sdk.DecCoins
		index          int
		expectedResult string
		expectedErr    bool
	}{
		{
			"No DecCoins in Slice",
			sdk.DecCoins{},
			0,
			"",
			true,
		},
		{"When index out of bounds", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, 2, "", true},
		{"When negative index", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, -1, "", true},
		{
			"Appropriate index case",
			sdk.DecCoins{
				sdk.DecCoin{testDenom1, math.LegacyNewDec(5)},
				sdk.DecCoin{testDenom2, math.LegacyNewDec(57)},
			},
			1, testDenom2, false,
		},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedErr {
				s.Require().Panics(func() { tc.input.GetDenomByIndex(tc.index) }, "Test should have panicked")
			} else {
				res := tc.input.GetDenomByIndex(tc.index)
				s.Require().Equal(tc.expectedResult, res, "Unexpected result for test case #%d, expected output: %s, input: %v", i, tc.expectedResult, tc.input)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_IsAllPositive() {
	testCases := []struct {
		name           string
		input          sdk.DecCoins
		expectedResult bool
	}{
		{"No Coins", sdk.DecCoins{}, false},

		{"One Coin - Zero value", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(0)}}, false},

		{"One Coin - Positive value", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, true},

		{"One Coin - Negative value", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(-15)}}, false},

		{"Multiple Coins - All positive value", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(51)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(123)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(50)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(92233720)},
		}, true},

		{"Multiple Coins - Some negative value", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(51)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(-123)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(92233720)},
		}, false},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedResult {
				s.Require().True(tc.input.IsAllPositive(), "Test case #%d: %s", i, tc.name)
			} else {
				s.Require().False(tc.input.IsAllPositive(), "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_IsLT() {
	testCases := []struct {
		name           string
		coin           sdk.DecCoin
		otherCoin      sdk.DecCoin
		expectedResult bool
		expectedPanic  bool
	}{
		{"Same Denom - Less than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(3)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(19)}, true, false},

		{"Same Denom - Greater than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(343340)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(14)}, false, false},

		{"Same Denom - Same as other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, false, false},

		{"Different Denom - Less than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(3)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(19)}, true, true},

		{"Different Denom - Greater than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(343340)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(14)}, true, true},

		{"Different Denom - Same as other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(20)}, true, true},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedPanic {
				s.Require().Panics(func() { tc.coin.IsLT(tc.otherCoin) }, "Test case #%d: %s", i, tc.name)
			} else {
				res := tc.coin.IsLT(tc.otherCoin)
				if tc.expectedResult {
					s.Require().True(res, "Test case #%d: %s", i, tc.name)
				} else {
					s.Require().False(res, "Test case #%d: %s", i, tc.name)
				}
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_IsGTE() {
	testCases := []struct {
		name           string
		coin           sdk.DecCoin
		otherCoin      sdk.DecCoin
		expectedResult bool
		expectedPanic  bool
	}{
		{"Same Denom - Less than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(3)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(19)}, false, false},

		{"Same Denom - Greater than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(343340)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(14)}, true, false},

		{"Same Denom - Same as other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, true, false},

		{"Different Denom - Less than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(3)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(19)}, true, true},

		{"Different Denom - Greater than other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(343340)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(14)}, true, true},

		{"Different Denom - Same as other coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(20)}, sdk.DecCoin{testDenom2, math.LegacyNewDec(20)}, true, true},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedPanic {
				s.Require().Panics(func() { tc.coin.IsGTE(tc.otherCoin) }, "Test case #%d: %s", i, tc.name)
			} else {
				res := tc.coin.IsGTE(tc.otherCoin)
				if tc.expectedResult {
					s.Require().True(res, "Test case #%d: %s", i, tc.name)
				} else {
					s.Require().False(res, "Test case #%d: %s", i, tc.name)
				}
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_IsZero() {
	testCases := []struct {
		name           string
		coins          sdk.DecCoins
		expectedResult bool
	}{
		{"No Coins", sdk.DecCoins{}, true},

		{"One Coin - Zero value", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(0)}}, true},

		{"One Coin - Positive value", sdk.DecCoins{sdk.DecCoin{testDenom1, math.LegacyNewDec(5)}}, false},

		{"Multiple Coins - All zero value", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
		}, true},

		{"Multiple Coins - Some positive value", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(0)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(92233720)},
		}, false},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedResult {
				s.Require().True(tc.coins.IsZero(), "Test case #%d: %s", i, tc.name)
			} else {
				s.Require().False(tc.coins.IsZero(), "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_MulDec() {
	testCases := []struct {
		name           string
		coins          sdk.DecCoins
		multiplier     math.LegacyDec
		expectedResult sdk.DecCoins
	}{
		{"No Coins", sdk.DecCoins{}, math.LegacyNewDec(1), sdk.DecCoins(nil)},

		{"Multiple coins - zero multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(10)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(30)},
		}, math.LegacyNewDec(0), sdk.DecCoins(nil)},

		{"Multiple coins - positive multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(1)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(2)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
		}},

		{"Multiple coins - negative multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(1)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(2)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(-2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(-20)},
		}},

		{"Multiple coins - Different denom", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(1)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(2)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(8)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(12)},
		}},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			res := tc.coins.MulDec(tc.multiplier)
			s.Require().Equal(tc.expectedResult, res, "Test case #%d: %s", i, tc.name)
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_MulDecTruncate() {
	testCases := []struct {
		name           string
		coins          sdk.DecCoins
		multiplier     math.LegacyDec
		expectedResult sdk.DecCoins
		expectedPanic  bool
	}{
		{"No Coins", sdk.DecCoins{}, math.LegacyNewDec(1), sdk.DecCoins(nil), false},

		{"Multiple coins - zero multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(10, 3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(30, 2)},
		}, math.LegacyNewDec(0), sdk.DecCoins{}, false},

		{"Multiple coins - positive multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
		}, math.LegacyNewDec(1), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(3, 0)},
		}, false},

		{"Multiple coins - positive multiplier", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
		}, math.LegacyNewDec(-2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(-6, 0)},
		}, false},

		{"Multiple coins - Different denom", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
			sdk.DecCoin{testDenom2, math.LegacyNewDecWithPrec(3333, 4)},
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(15, 1)},
			sdk.DecCoin{testDenom2, math.LegacyNewDecWithPrec(333, 4)},
		}, math.LegacyNewDec(10), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(30, 0)},
			sdk.DecCoin{testDenom2, math.LegacyNewDecWithPrec(3666, 3)},
		}, false},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.expectedPanic {
				s.Require().Panics(func() { tc.coins.MulDecTruncate(tc.multiplier) }, "Test case #%d: %s", i, tc.name)
			} else {
				res := tc.coins.MulDecTruncate(tc.multiplier)
				s.Require().Equal(tc.expectedResult, res, "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_QuoDec() {
	testCases := []struct {
		name           string
		coins          sdk.DecCoins
		input          math.LegacyDec
		expectedResult sdk.DecCoins
		panics         bool
	}{
		{"No Coins", sdk.DecCoins{}, math.LegacyNewDec(1), sdk.DecCoins(nil), false},

		{"Multiple coins - zero input", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(10)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(30)},
		}, math.LegacyNewDec(0), sdk.DecCoins(nil), true},

		{"Multiple coins - positive input", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(35, 1)},
		}, false},

		{"Multiple coins - negative input", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(-2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDecWithPrec(-35, 1)},
		}, false},

		{"Multiple coins - Different input", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(1)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(2)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(4)},
		}, math.LegacyNewDec(2), sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(2)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(3)},
		}, false},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			if tc.panics {
				s.Require().Panics(func() { tc.coins.QuoDec(tc.input) }, "Test case #%d: %s", i, tc.name)
			} else {
				res := tc.coins.QuoDec(tc.input)
				s.Require().Equal(tc.expectedResult, res, "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_IsEqual() {
	testCases := []struct {
		name           string
		coin           sdk.DecCoin
		otherCoin      sdk.DecCoin
		expectedResult bool
	}{
		{
			"Different Denom Same Amount",
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(20)},
			false,
		},

		{
			"Different Denom Different Amount",
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(10)},
			false,
		},

		{
			"Same Denom Different Amount",
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(10)},
			false,
		},

		{
			"Same Denom Same Amount",
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(20)},
			true,
		},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			res := tc.coin.IsEqual(tc.otherCoin)
			if tc.expectedResult {
				s.Require().True(res, "Test case #%d: %s", i, tc.name)
			} else {
				s.Require().False(res, "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoins_IsEqual() {
	testCases := []struct {
		name           string
		coinsA         sdk.DecCoins
		coinsB         sdk.DecCoins
		expectedResult bool
	}{
		{"Different length sets", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(35)},
		}, false},

		{"Same length - different denoms", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, sdk.DecCoins{
			sdk.DecCoin{testDenom2, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom2, math.LegacyNewDec(4)},
		}, false},

		{"Same length - different amounts", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(3)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(4)},
		}, sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(41)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(343)},
		}, false},

		{"Same length - same amounts", sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(33)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(344)},
		}, sdk.DecCoins{
			sdk.DecCoin{testDenom1, math.LegacyNewDec(33)},
			sdk.DecCoin{testDenom1, math.LegacyNewDec(344)},
		}, true},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			res := tc.coinsA.Equal(tc.coinsB)
			if tc.expectedResult {
				s.Require().True(res, "Test case #%d: %s", i, tc.name)
			} else {
				s.Require().False(res, "Test case #%d: %s", i, tc.name)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_Validate() {
	var empty sdk.DecCoin
	testCases := []struct {
		name         string
		input        sdk.DecCoin
		expectedPass bool
	}{
		{"Uninitialized deccoin", empty, false},

		{"Invalid denom string", sdk.DecCoin{"(){9**&})", math.LegacyNewDec(33)}, false},

		{"Negative coin amount", sdk.DecCoin{testDenom1, math.LegacyNewDec(-33)}, false},

		{"Valid coin", sdk.DecCoin{testDenom1, math.LegacyNewDec(33)}, true},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			err := tc.input.Validate()
			if tc.expectedPass {
				s.Require().NoError(err, "unexpected result for test case #%d %s, input: %v", i, tc.name, tc.input)
			} else {
				s.Require().Error(err, "unexpected result for test case #%d %s, input: %v", i, tc.name, tc.input)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_ParseDecCoin() {
	var empty sdk.DecCoin
	testCases := []struct {
		name           string
		input          string
		expectedResult sdk.DecCoin
		expectedErr    bool
	}{
		{"Empty input", "", empty, true},

		{"Bad input", "✨🌟⭐", empty, true},

		{"Invalid decimal coin", "9.3.0stake", empty, true},

		{"Precision over limit", "9.11111111111111111111stake", empty, true},

		{"Valid upper case denom", "9.3STAKE", sdk.DecCoin{"STAKE", math.LegacyNewDecWithPrec(93, 1)}, false},

		{"Valid input - amount and denom separated by space", "9.3 stake", sdk.DecCoin{"stake", math.LegacyNewDecWithPrec(93, 1)}, false},

		{"Valid input - amount and denom concatenated", "9.3stake", sdk.DecCoin{"stake", math.LegacyNewDecWithPrec(93, 1)}, false},
	}

	for i, tc := range testCases {
		s.T().Run(tc.name, func(t *testing.T) {
			res, err := sdk.ParseDecCoin(tc.input)
			if tc.expectedErr {
				s.Require().Error(err, "expected error for test case #%d %s, input: %v", i, tc.name, tc.input)
			} else {
				s.Require().NoError(err, "unexpected error for test case #%d %s, input: %v", i, tc.name, tc.input)
				s.Require().Equal(tc.expectedResult, res, "unexpected result for test case #%d %s, input: %v", i, tc.name, tc.input)
			}
		})
	}
}

func (s *decCoinTestSuite) TestDecCoin_ParseDecAmount() {
	testCases := map[string]struct {
		input          string
		expectedAmount string
		expectedDenom  string
		expectedErr    bool
	}{
		"Parse empty string": {"", "", "", false},
		"Parse string which start with character different from digit, space, and letter": {"✨🌟⭐", "", "", true},
		"Parse string started with letter":                                                {"h10", "", "h10", false},
		"Parse string started with dot":                                                   {".atom", ".", "atom", false},

		"Parse string that contain only numbers": {"10", "10", "", false},
		"Use number for denom":                   {"10 11", "", "", true},
		"Use invalid character '&' for denom ":   {"10f&", "", "", true},
		"Use space between amount and denom":     {"10 atom", "10", "atom", false},
		"Use two space between amount and denom": {"10  atom", "10", "atom", false},

		"Denom start with digit": {"1 1atom", "", "", true},
		"Denom start with '/'":   {"1 /atom", "", "", true},
		"Denom start with ':'":   {"1 :atom", "", "", true},
		"Denom start with '.'":   {"1 .atom", "", "", true},
		"Denom start with '_'":   {"1 _atom", "", "", true},
		"Denom start with '-'":   {"1 -atom", "", "", true},

		"Denom contains '/'":   {"1 at/om", "1", "at/om", false},
		"Denom contains ':'":   {"2atom:", "2", "atom:", false},
		"Denom contains '.'":   {"3ato.m", "3", "ato.m", false},
		"Denom contains '_'":   {"4 a_tom", "4", "a_tom", false},
		"Denom contains '-'":   {"5 at-om", "5", "at-om", false},
		"Denom contains space": {"5 at om", "", "", true},
	}

	for name, tc := range testCases {
		s.T().Run(name, func(t *testing.T) {
			actualAmount, actualDenom, err := sdk.ParseDecAmount(tc.input)
			if tc.expectedErr {
				s.Require().Error(err, "expected error for test case %s, input: %v", name, tc.input)
			} else {
				s.Require().NoError(err, "unexpected error for test case %s, input: %v", name, tc.input)
				s.Require().Equal(tc.expectedAmount, actualAmount)
				s.Require().Equal(tc.expectedDenom, actualDenom)
			}
		})
	}
}
