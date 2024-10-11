package types_test

import (
	"fmt"
	"math/big"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/math"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var (
	testDenom1 = "atom"
	testDenom2 = "muon"
)

type coinTestSuite struct {
	suite.Suite
	ca0, ca1, ca2, ca4, cm0, cm1, cm2, cm4 sdk.Coin
	emptyCoins                             sdk.Coins
}

func TestCoinTestSuite(t *testing.T) {
	suite.Run(t, new(coinTestSuite))
}

func (s *coinTestSuite) SetupSuite() {
	zero := math.NewInt(0)
	one := math.OneInt()
	two := math.NewInt(2)
	four := math.NewInt(4)

	s.ca0, s.ca1, s.ca2, s.ca4 = sdk.NewCoin(testDenom1, zero), sdk.NewCoin(testDenom1, one), sdk.NewCoin(testDenom1, two), sdk.NewCoin(testDenom1, four)
	s.cm0, s.cm1, s.cm2, s.cm4 = sdk.NewCoin(testDenom2, zero), sdk.NewCoin(testDenom2, one), sdk.NewCoin(testDenom2, two), sdk.NewCoin(testDenom2, four)
	s.emptyCoins = sdk.Coins{}
}

// ----------------------------------------------------------------------------
// Coin tests

func (s *coinTestSuite) TestCoin() {
	s.Require().Panics(func() { sdk.NewInt64Coin(testDenom1, -1) })
	s.Require().Panics(func() { sdk.NewCoin(testDenom1, math.NewInt(-1)) })
	s.Require().Equal(math.NewInt(10), sdk.NewInt64Coin(strings.ToUpper(testDenom1), 10).Amount)
	s.Require().Equal(math.NewInt(10), sdk.NewCoin(strings.ToUpper(testDenom1), math.NewInt(10)).Amount)
	s.Require().Equal(math.NewInt(5), sdk.NewInt64Coin(testDenom1, 5).Amount)
	s.Require().Equal(math.NewInt(5), sdk.NewCoin(testDenom1, math.NewInt(5)).Amount)
}

func (s *coinTestSuite) TestCoin_String() {
	coin := sdk.NewCoin(testDenom1, math.NewInt(10))
	s.Require().Equal(fmt.Sprintf("10%s", testDenom1), coin.String())
}

func (s *coinTestSuite) TestIsEqualCoin() {
	cases := []struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), true},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), false},
		{sdk.NewInt64Coin("stake", 1), sdk.NewInt64Coin("stake", 10), false},
	}

	for tcIndex, tc := range cases {
		res := tc.inputOne.IsEqual(tc.inputTwo)
		s.Require().Equal(tc.expected, res, "coin equality relation is incorrect, tc #%d", tcIndex)
	}
}

func (s *coinTestSuite) TestCoinIsValid() {
	loremIpsum := `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nam viverra dui vel nulla aliquet, non dictum elit aliquam. Proin consequat leo in consectetur mattis. Phasellus eget odio luctus, rutrum dolor at, venenatis ante. Praesent metus erat, sodales vitae sagittis eget, commodo non ipsum. Duis eget urna quis erat mattis pulvinar. Vivamus egestas imperdiet sem, porttitor hendrerit lorem pulvinar in. Vivamus laoreet sapien eget libero euismod tristique. Suspendisse tincidunt nulla quis luctus mattis.
	Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Sed id turpis at erat placerat fermentum id sed sapien. Fusce mattis enim id nulla viverra, eget placerat eros aliquet. Nunc fringilla urna ac condimentum ultricies. Praesent in eros ac neque fringilla sodales. Donec ut venenatis eros. Quisque iaculis lectus neque, a various sem ullamcorper nec. Cras tincidunt dignissim libero nec volutpat. Donec molestie enim sed metus venenatis, quis elementum sem various. Curabitur eu venenatis nulla.
	Cras sit amet ligula vel turpis placerat sollicitudin. Nunc massa odio, eleifend id lacus nec, ultricies elementum arcu. Donec imperdiet nulla lacus, a venenatis lacus fermentum nec. Proin vestibulum dolor enim, vitae posuere velit aliquet non. Suspendisse pharetra condimentum nunc tincidunt viverra. Etiam posuere, ligula ut maximus congue, mauris orci consectetur velit, vel finibus eros metus non tellus. Nullam et dictum metus. Aliquam maximus fermentum mauris elementum aliquet. Class aptent taciti sociosqu ad litora torquent per conubia nostra, per inceptos himenaeos. Etiam dapibus lectus sed tellus rutrum tincidunt. Nulla at dolor sem. Ut non dictum arcu, eget congue sem.`

	loremIpsum = strings.ReplaceAll(loremIpsum, " ", "")
	loremIpsum = strings.ReplaceAll(loremIpsum, ".", "")
	loremIpsum = strings.ReplaceAll(loremIpsum, ",", "")

	cases := []struct {
		coin       sdk.Coin
		expectPass bool
	}{
		{sdk.Coin{testDenom1, math.NewInt(-1)}, false},
		{sdk.Coin{testDenom1, math.NewInt(0)}, true},
		{sdk.Coin{testDenom1, math.OneInt()}, true},
		{sdk.Coin{"Atom", math.OneInt()}, true},
		{sdk.Coin{"ATOM", math.OneInt()}, true},
		{sdk.Coin{"a", math.OneInt()}, false},
		{sdk.Coin{loremIpsum, math.OneInt()}, false},
		{sdk.Coin{"ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", math.OneInt()}, true},
		{sdk.Coin{"atOm", math.OneInt()}, true},
		{sdk.Coin{"x:y-z.1_2", math.OneInt()}, true},
		{sdk.Coin{"     ", math.OneInt()}, false},
	}

	for i, tc := range cases {
		s.Require().Equal(tc.expectPass, tc.coin.IsValid(), "unexpected result for IsValid, tc #%d", i)
	}
}

func (s *coinTestSuite) TestCustomValidation() {
	newDnmRegex := `[\x{1F600}-\x{1F6FF}]`
	reDnmString := `[a-zA-Z][a-zA-Z0-9/:._-]{2,127}`

	sdk.SetCoinDenomRegex(func() string {
		return newDnmRegex
	})

	cases := []struct {
		coin       sdk.Coin
		expectPass bool
	}{
		{sdk.Coin{"🙂", math.NewInt(1)}, true},
		{sdk.Coin{"🙁", math.NewInt(1)}, true},
		{sdk.Coin{"🌶", math.NewInt(1)}, false}, // outside the unicode range listed above
		{sdk.Coin{"asdf", math.NewInt(1)}, false},
		{sdk.Coin{"", math.NewInt(1)}, false},
	}

	for i, tc := range cases {
		s.Require().Equal(tc.expectPass, tc.coin.IsValid(), "unexpected result for IsValid, tc #%d", i)
	}
	sdk.SetCoinDenomRegex(func() string { return reDnmString })
}

func (s *coinTestSuite) TestCoinsDenoms() {
	cases := []struct {
		coins      sdk.Coins
		testOutput []string
		expectPass bool
	}{
		{sdk.NewCoins(sdk.Coin{"ATOM", math.NewInt(1)}, sdk.Coin{"JUNO", math.NewInt(1)}, sdk.Coin{"OSMO", math.NewInt(1)}, sdk.Coin{"RAT", math.NewInt(1)}), []string{"ATOM", "JUNO", "OSMO", "RAT"}, true},
		{sdk.NewCoins(sdk.Coin{"ATOM", math.NewInt(1)}, sdk.Coin{"JUNO", math.NewInt(1)}), []string{"ATOM"}, false},
	}

	for i, tc := range cases {
		expectedOutput := tc.coins.Denoms()
		count := 0
		if len(expectedOutput) == len(tc.testOutput) {
			for k := range tc.testOutput {
				if tc.testOutput[k] != expectedOutput[k] {
					count++
					break
				}
			}
		} else {
			count++
		}
		s.Require().Equal(count == 0, tc.expectPass, "unexpected result for coins.Denoms, tc #%d", i)
	}
}

func (s *coinTestSuite) TestAddCoin() {
	// 2**256 - 1 value to check for overflows
	maxUint256 := math.NewIntFromBigInt(new(big.Int).Sub(new(big.Int).Lsh(big.NewInt(1), 256), big.NewInt(1)))

	cases := []struct {
		inputOne    sdk.Coin
		inputTwo    sdk.Coin
		expected    sdk.Coin
		shouldPanic bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 2), false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom1, 1), false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), sdk.NewInt64Coin(testDenom1, 1), true},

		// addition cannot lead to a number with more than 256 bits
		{sdk.NewCoin(testDenom1, maxUint256), sdk.NewCoin(testDenom1, math.NewInt(0)), sdk.NewCoin(testDenom1, maxUint256), false},
		{sdk.NewCoin(testDenom1, maxUint256), sdk.NewCoin(testDenom1, math.NewInt(1)), sdk.NewInt64Coin(testDenom1, 1), true},
	}

	for tcIndex, tc := range cases {
		if tc.shouldPanic {
			s.Require().Panics(func() { tc.inputOne.Add(tc.inputTwo) })
		} else {
			res := tc.inputOne.Add(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "sum of coins is incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestAddCoinAmount() {
	cases := []struct {
		coin     sdk.Coin
		amount   math.Int
		expected sdk.Coin
	}{
		{sdk.NewInt64Coin(testDenom1, 1), math.NewInt(1), sdk.NewInt64Coin(testDenom1, 2)},
		{sdk.NewInt64Coin(testDenom1, 1), math.NewInt(0), sdk.NewInt64Coin(testDenom1, 1)},
	}
	for i, tc := range cases {
		res := tc.coin.AddAmount(tc.amount)
		s.Require().Equal(tc.expected, res, "result of addition is incorrect, tc #%d", i)
	}
}

func (s *coinTestSuite) TestSubCoin() {
	cases := []struct {
		inputOne    sdk.Coin
		inputTwo    sdk.Coin
		expected    sdk.Coin
		shouldPanic bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), sdk.NewInt64Coin(testDenom1, 1), true},
		{sdk.NewInt64Coin(testDenom1, 10), sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 9), false},
		{sdk.NewInt64Coin(testDenom1, 5), sdk.NewInt64Coin(testDenom1, 3), sdk.NewInt64Coin(testDenom1, 2), false},
		{sdk.NewInt64Coin(testDenom1, 5), sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom1, 5), false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 5), sdk.Coin{}, true},
	}

	for tcIndex, tc := range cases {
		if tc.shouldPanic {
			s.Require().Panics(func() { tc.inputOne.Sub(tc.inputTwo) })
		} else {
			res := tc.inputOne.Sub(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "difference of coins is incorrect, tc #%d", tcIndex)
		}
	}

	tc := struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected int64
	}{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), 0}
	res := tc.inputOne.Sub(tc.inputTwo)
	s.Require().Equal(tc.expected, res.Amount.Int64())
}

func (s *coinTestSuite) TestSubCoinAmount() {
	cases := []struct {
		coin        sdk.Coin
		amount      math.Int
		expected    sdk.Coin
		shouldPanic bool
	}{
		{sdk.NewInt64Coin(testDenom1, 2), math.NewInt(1), sdk.NewInt64Coin(testDenom1, 1), false},
		{sdk.NewInt64Coin(testDenom1, 10), math.NewInt(1), sdk.NewInt64Coin(testDenom1, 9), false},
		{sdk.NewInt64Coin(testDenom1, 5), math.NewInt(3), sdk.NewInt64Coin(testDenom1, 2), false},
		{sdk.NewInt64Coin(testDenom1, 5), math.NewInt(0), sdk.NewInt64Coin(testDenom1, 5), false},
		{sdk.NewInt64Coin(testDenom1, 1), math.NewInt(5), sdk.Coin{}, true},
	}

	for i, tc := range cases {
		if tc.shouldPanic {
			s.Require().Panics(func() { tc.coin.SubAmount(tc.amount) })
		} else {
			res := tc.coin.SubAmount(tc.amount)
			s.Require().Equal(tc.expected, res, "result of subtraction is incorrect, tc #%d", i)
		}
	}
}

func (s *coinTestSuite) TestMulIntCoins() {
	testCases := []struct {
		input       sdk.Coins
		multiplier  math.Int
		expected    sdk.Coins
		shouldPanic bool
	}{
		{sdk.Coins{s.ca2}, math.NewInt(0), sdk.Coins{s.ca0}, true},
		{sdk.Coins{s.ca2}, math.NewInt(2), sdk.Coins{s.ca4}, false},
		{sdk.Coins{s.ca1, s.cm2}, math.NewInt(2), sdk.Coins{s.ca2, s.cm4}, false},
	}

	assert := s.Assert()
	for i, tc := range testCases {
		if tc.shouldPanic {
			assert.Panics(func() { tc.input.MulInt(tc.multiplier) })
		} else {
			res := tc.input.MulInt(tc.multiplier)
			assert.True(res.IsValid())
			assert.Equal(tc.expected, res, "multiplication of coins is incorrect, tc #%d", i)
		}
	}
}

func (s *coinTestSuite) TestQuoIntCoins() {
	testCases := []struct {
		input       sdk.Coins
		divisor     math.Int
		expected    sdk.Coins
		isValid     bool
		shouldPanic bool
	}{
		{sdk.Coins{s.ca2, s.ca1}, math.NewInt(0), sdk.Coins{s.ca0, s.ca0}, true, true},
		{sdk.Coins{s.ca2}, math.NewInt(4), sdk.Coins{s.ca0}, false, false},
		{sdk.Coins{s.ca2, s.cm4}, math.NewInt(2), sdk.Coins{s.ca1, s.cm2}, true, false},
		{sdk.Coins{s.ca4}, math.NewInt(2), sdk.Coins{s.ca2}, true, false},
	}

	assert := s.Assert()
	for i, tc := range testCases {
		if tc.shouldPanic {
			assert.Panics(func() { tc.input.QuoInt(tc.divisor) })
		} else {
			res := tc.input.QuoInt(tc.divisor)
			assert.Equal(tc.isValid, res.IsValid())
			assert.Equal(tc.expected, res, "quotient of coins is incorrect, tc #%d", i)
		}
	}
}

func (s *coinTestSuite) TestIsGTCoin() {
	cases := []struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected bool
		panics   bool
	}{
		{sdk.NewInt64Coin(testDenom1, 2), sdk.NewInt64Coin(testDenom1, 1), true, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), false, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 2), false, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), false, true},
	}

	for tcIndex, tc := range cases {
		if tc.panics {
			s.Require().Panics(func() { tc.inputOne.IsGT(tc.inputTwo) })
		} else {
			res := tc.inputOne.IsGT(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "coin GT relation is incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestIsGTECoin() {
	cases := []struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected bool
		panics   bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), true, false},
		{sdk.NewInt64Coin(testDenom1, 2), sdk.NewInt64Coin(testDenom1, 1), true, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 2), false, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), false, true},
	}

	for tcIndex, tc := range cases {
		if tc.panics {
			s.Require().Panics(func() { tc.inputOne.IsGTE(tc.inputTwo) })
		} else {
			res := tc.inputOne.IsGTE(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "coin GTE relation is incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestIsLTECoin() {
	cases := []struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected bool
		panics   bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), true, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 2), true, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), false, true},
		{sdk.NewInt64Coin(testDenom1, 2), sdk.NewInt64Coin(testDenom1, 1), false, false},
	}

	for tcIndex, tc := range cases {
		if tc.panics {
			s.Require().Panics(func() { tc.inputOne.IsLTE(tc.inputTwo) })
		} else {
			res := tc.inputOne.IsLTE(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "coin LTE relation is incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestIsLTCoin() {
	cases := []struct {
		inputOne sdk.Coin
		inputTwo sdk.Coin
		expected bool
		panics   bool
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), false, false},
		{sdk.NewInt64Coin(testDenom1, 2), sdk.NewInt64Coin(testDenom1, 1), false, false},
		{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1), false, true},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), false, true},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 1), false, false},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 2), true, false},
	}

	for tcIndex, tc := range cases {
		if tc.panics {
			s.Require().Panics(func() { tc.inputOne.IsLT(tc.inputTwo) })
		} else {
			res := tc.inputOne.IsLT(tc.inputTwo)
			s.Require().Equal(tc.expected, res, "coin LT relation is incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestCoinIsZero() {
	coin := sdk.NewInt64Coin(testDenom1, 0)
	res := coin.IsZero()
	s.Require().True(res)

	coin = sdk.NewInt64Coin(testDenom1, 1)
	res = coin.IsZero()
	s.Require().False(res)
}

func (s *coinTestSuite) TestCoinIsNil() {
	coin := sdk.Coin{}
	res := coin.IsNil()
	s.Require().True(res)

	coin = sdk.Coin{Denom: "uatom"}
	res = coin.IsNil()
	s.Require().True(res)

	coin = sdk.NewInt64Coin(testDenom1, 1)
	res = coin.IsNil()
	s.Require().False(res)
}

func (s *coinTestSuite) TestFilteredZeroCoins() {
	cases := []struct {
		name     string
		input    sdk.Coins
		original string
		expected string
	}{
		{
			name: "all greater than zero",
			input: sdk.Coins{
				{"testa", math.OneInt()},
				{"testb", math.NewInt(2)},
				{"testc", math.NewInt(3)},
				{"testd", math.NewInt(4)},
				{"teste", math.NewInt(5)},
			},
			original: "1testa,2testb,3testc,4testd,5teste",
			expected: "1testa,2testb,3testc,4testd,5teste",
		},
		{
			name: "zero coin in middle",
			input: sdk.Coins{
				{"testa", math.OneInt()},
				{"testb", math.NewInt(2)},
				{"testc", math.NewInt(0)},
				{"testd", math.NewInt(4)},
				{"teste", math.NewInt(5)},
			},
			original: "1testa,2testb,0testc,4testd,5teste",
			expected: "1testa,2testb,4testd,5teste",
		},
		{
			name: "zero coin end (unordered)",
			input: sdk.Coins{
				{"teste", math.NewInt(5)},
				{"testc", math.NewInt(3)},
				{"testa", math.OneInt()},
				{"testd", math.NewInt(4)},
				{"testb", math.NewInt(0)},
			},
			original: "5teste,3testc,1testa,4testd,0testb",
			expected: "1testa,3testc,4testd,5teste",
		},
	}

	for _, tt := range cases {
		undertest := sdk.NewCoins(tt.input...)
		s.Require().Equal(tt.expected, undertest.String(), "NewCoins must return expected results")
		s.Require().Equal(tt.original, tt.input.String(), "input must be unmodified and match original")
	}
}

// ----------------------------------------------------------------------------
// Coins tests

func (s *coinTestSuite) TestCoins_String() {
	cases := []struct {
		name     string
		input    sdk.Coins
		expected string
	}{
		{
			"empty coins",
			sdk.Coins{},
			"",
		},
		{
			"single coin",
			sdk.Coins{{"tree", math.OneInt()}},
			"1tree",
		},
		{
			"multiple coins",
			sdk.Coins{
				{"tree", math.OneInt()},
				{"gas", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			"1tree,1gas,1mineral",
		},
	}

	for _, tt := range cases {
		s.Require().Equal(tt.expected, tt.input.String())
	}
}

func (s *coinTestSuite) TestIsZeroCoins() {
	cases := []struct {
		inputOne sdk.Coins
		expected bool
	}{
		{sdk.Coins{}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 0)}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 1)}, false},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, false},
	}

	for _, tc := range cases {
		res := tc.inputOne.IsZero()
		s.Require().Equal(tc.expected, res)
	}
}

func (s *coinTestSuite) TestEqualCoins() {
	cases := []struct {
		inputOne sdk.Coins
		inputTwo sdk.Coins
		expected bool
	}{
		{sdk.Coins{}, sdk.Coins{}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, true},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, sdk.Coins{sdk.NewInt64Coin(testDenom2, 0)}, false},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, sdk.Coins{sdk.NewInt64Coin(testDenom1, 1)}, false},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0)}, sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, false},
		{sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, sdk.Coins{sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom2, 1)}, true},
	}

	for tcnum, tc := range cases {
		res := tc.inputOne.Equal(tc.inputTwo)
		s.Require().Equal(tc.expected, res, "Equality is differed from exported. tc #%d, expected %b, actual %b.", tcnum, tc.expected, res)
	}
}

func (s *coinTestSuite) TestAddCoins() {
	cA0M0 := sdk.Coins{s.ca0, s.cm0}
	cA0M1 := sdk.Coins{s.ca0, s.cm1}
	cA1M1 := sdk.Coins{s.ca1, s.cm1}
	cases := []struct {
		name     string
		inputOne sdk.Coins
		inputTwo sdk.Coins
		expected sdk.Coins
		msg      string
	}{
		{"adding two empty lists", s.emptyCoins, s.emptyCoins, s.emptyCoins, "empty, non list should be returned"},
		{"empty list + set", s.emptyCoins, cA0M1, sdk.Coins{s.cm1}, "zero coins should be removed"},
		{"empty list + set", s.emptyCoins, cA1M1, cA1M1, "zero + a_non_zero = a_non_zero"},
		{"set + empty list", cA0M1, s.emptyCoins, sdk.Coins{s.cm1}, "zero coins should be removed"},
		{"set + empty list", cA1M1, s.emptyCoins, cA1M1, "a_non_zero + zero  = a_non_zero"},
		{
			"{1atom,1muon}+{1atom,1muon}", cA1M1, cA1M1,
			sdk.Coins{s.ca2, s.cm2},
			"a + a = 2a",
		},
		{
			"{0atom,1muon}+{0atom,0muon}", cA0M1, cA0M0,
			sdk.Coins{s.cm1},
			"zero coins should be removed",
		},
		{
			"{2atom}+{0muon}",
			sdk.Coins{s.ca2},
			sdk.Coins{s.cm0},
			sdk.Coins{s.ca2},
			"zero coins should be removed",
		},
		{
			"{1atom}+{1atom,2muon}",
			sdk.Coins{s.ca1},
			sdk.Coins{s.ca1, s.cm2},
			sdk.Coins{s.ca2, s.cm2},
			"should be correctly added",
		},
		{
			"{0atom,0muon}+{0atom,0muon}", cA0M0, cA0M0, s.emptyCoins,
			"sets with zero coins should return empty set",
		},
	}

	for _, tc := range cases {
		s.T().Run(tc.name, func(t *testing.T) {
			res := tc.inputOne.Add(tc.inputTwo...)
			require.True(t, res.IsValid(), fmt.Sprintf("%s + %s = %s", tc.inputOne, tc.inputTwo, res))
			require.Equal(t, tc.expected, res, tc.msg)
		})
	}
}

// Tests that even if coins with repeated denominations are passed into .Add that they
// are correctly coalesced. Please see issue https://github.com/cosmos/cosmos-sdk/issues/13234
func TestCoinsAddCoalescesDuplicateDenominations(t *testing.T) {
	A := sdk.Coins{
		{"den", math.NewInt(2)},
		{"den", math.NewInt(3)},
	}
	B := sdk.Coins{
		{"den", math.NewInt(3)},
		{"den", math.NewInt(2)},
		{"den", math.NewInt(1)},
	}

	A = A.Sort()
	B = B.Sort()
	got := A.Add(B...)

	want := sdk.Coins{
		{"den", math.NewInt(11)},
	}

	if !got.Equal(want) {
		t.Fatalf("Wrong result\n\tGot:   %s\n\tWant: %s", got, want)
	}
}

func TestCoinsAddUnorderedDenominations(t *testing.T) {
	A := sdk.Coins{
		{"aaa", math.NewInt(2)},
		{"bbb", math.NewInt(3)},
	}
	B := sdk.Coins{
		{"bbb", math.NewInt(3)},
		{"aaa", math.NewInt(2)},
	}

	require.Panics(t, func() {
		A.Add(B...)
	})
}

func (s *coinTestSuite) TestSubCoins() {
	cA0M0 := sdk.Coins{s.ca0, s.cm0}
	cA0M1 := sdk.Coins{s.ca0, s.cm1}
	testCases := []struct {
		inputOne    sdk.Coins
		inputTwo    sdk.Coins
		expected    sdk.Coins
		shouldPanic bool
	}{
		{s.emptyCoins, s.emptyCoins, s.emptyCoins, false},
		{cA0M0, s.emptyCoins, s.emptyCoins, false},
		{cA0M0, sdk.Coins{s.cm0}, s.emptyCoins, false},
		{sdk.Coins{s.cm0}, cA0M0, s.emptyCoins, false},
		{cA0M1, s.emptyCoins, sdk.Coins{s.cm1}, false},
		// denoms are not sorted - should panic
		{sdk.Coins{s.ca2}, sdk.Coins{s.cm2, s.ca1}, sdk.Coins{}, true},
		{sdk.Coins{s.cm2, s.ca2}, sdk.Coins{s.ca1}, sdk.Coins{}, true},
		// test cases for sorted denoms
		{sdk.Coins{s.ca2}, sdk.Coins{s.ca1, s.cm2}, sdk.Coins{s.ca1, s.cm2}, true},
		{sdk.Coins{s.ca2}, sdk.Coins{s.cm0}, sdk.Coins{s.ca2}, false},
		{sdk.Coins{s.ca1}, sdk.Coins{s.cm0}, sdk.Coins{s.ca1}, false},
		{sdk.Coins{s.ca1, s.cm1}, sdk.Coins{s.ca1}, sdk.Coins{s.cm1}, false},
		{sdk.Coins{s.ca1, s.cm1}, sdk.Coins{s.ca2}, sdk.Coins{}, true},
	}

	assert := s.Assert()
	for i, tc := range testCases {
		if tc.shouldPanic {
			assert.Panics(func() { tc.inputOne.Sub(tc.inputTwo...) })
		} else {
			res := tc.inputOne.Sub(tc.inputTwo...)
			assert.True(res.IsValid())
			assert.Equal(tc.expected, res, "sum of coins is incorrect, tc #%d", i)
		}
	}
}

func (s *coinTestSuite) TestSafeSubCoin() {
	cases := []struct {
		inputOne  sdk.Coin
		inputTwo  sdk.Coin
		expected  sdk.Coin
		expErrMsg string
	}{
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom2, 1), sdk.NewInt64Coin(testDenom1, 1), "invalid coin denoms"},
		{sdk.NewInt64Coin(testDenom1, 10), sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 9), ""},
		{sdk.NewInt64Coin(testDenom1, 5), sdk.NewInt64Coin(testDenom1, 0), sdk.NewInt64Coin(testDenom1, 5), ""},
		{sdk.NewInt64Coin(testDenom1, 1), sdk.NewInt64Coin(testDenom1, 5), sdk.Coin{}, "negative coin amount"},
	}

	for _, tc := range cases {

		res, err := tc.inputOne.SafeSub(tc.inputTwo)
		if err != nil {
			s.Require().Contains(err.Error(), tc.expErrMsg)
			return
		}
		s.Require().Equal(tc.expected, res)
	}
}

func (s *coinTestSuite) TestCoins_Validate() {
	testCases := []struct {
		name    string
		coins   sdk.Coins
		expPass bool
	}{
		{
			"valid lowercase coins",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"mineral", math.OneInt()},
				{"tree", math.OneInt()},
			},
			true,
		},
		{
			"valid uppercase coins",
			sdk.Coins{
				{"GAS", math.OneInt()},
				{"MINERAL", math.OneInt()},
				{"TREE", math.OneInt()},
			},
			true,
		},
		{
			"valid uppercase coin",
			sdk.Coins{
				{"ATOM", math.OneInt()},
			},
			true,
		},
		{
			"valid lower and uppercase coins (1)",
			sdk.Coins{
				{"GAS", math.OneInt()},
				{"gAs", math.OneInt()},
			},
			true,
		},
		{
			"valid lower and uppercase coins (2)",
			sdk.Coins{
				{"ATOM", math.OneInt()},
				{"Atom", math.OneInt()},
				{"atom", math.OneInt()},
			},
			true,
		},
		{
			"mixed case (1)",
			sdk.Coins{
				{"MineraL", math.OneInt()},
				{"TREE", math.OneInt()},
				{"gAs", math.OneInt()},
			},
			true,
		},
		{
			"mixed case (2)",
			sdk.Coins{
				{"gAs", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			true,
		},
		{
			"mixed case (3)",
			sdk.Coins{
				{"gAs", math.OneInt()},
			},
			true,
		},
		{
			"unicode letters and numbers",
			sdk.Coins{
				{"𐀀𐀆𐀉Ⅲ", math.OneInt()},
			},
			false,
		},
		{
			"emojis",
			sdk.Coins{
				{"🤑😋🤔", math.OneInt()},
			},
			false,
		},
		{
			"IBC denominations (ADR 001)",
			sdk.Coins{
				{"ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", math.OneInt()},
				{"ibc/876563AAAACF739EB061C67CDB5EDF2B7C9FD4AA9D876450CC21210807C2820A", math.NewInt(2)},
			},
			true,
		},
		{
			"empty (1)",
			sdk.NewCoins(),
			true,
		},
		{
			"empty (2)",
			sdk.Coins{},
			true,
		},
		{
			"invalid denomination (1)",
			sdk.Coins{
				{"MineraL", math.OneInt()},
				{"0TREE", math.OneInt()},
				{"gAs", math.OneInt()},
			},
			false,
		},
		{
			"invalid denomination (2)",
			sdk.Coins{
				{"-GAS", math.OneInt()},
				{"gAs", math.OneInt()},
			},
			false,
		},
		{
			"bad sort (1)",
			sdk.Coins{
				{"tree", math.OneInt()},
				{"gas", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			false,
		},
		{
			"bad sort (2)",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"tree", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			false,
		},
		{
			"bad sort (3)",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"tree", math.OneInt()},
				{"gas", math.OneInt()},
			},
			false,
		},
		{
			"non-positive amount (1)",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"tree", math.NewInt(0)},
				{"mineral", math.OneInt()},
			},
			false,
		},
		{
			"non-positive amount (2)",
			sdk.Coins{
				{"gas", math.NewInt(-1)},
				{"tree", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			false,
		},
		{
			"duplicate denomination (1)",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"gas", math.OneInt()},
				{"mineral", math.OneInt()},
			},
			false,
		},
		{
			"duplicate denomination (2)",
			sdk.Coins{
				{"gold", math.OneInt()},
				{"gold", math.OneInt()},
			},
			false,
		},
		{
			"duplicate denomination (3)",
			sdk.Coins{
				{"gas", math.OneInt()},
				{"mineral", math.OneInt()},
				{"silver", math.OneInt()},
				{"silver", math.OneInt()},
			},
			false,
		},
	}

	for _, tc := range testCases {
		err := tc.coins.Validate()
		if tc.expPass {
			s.Require().NoError(err, tc.name)
		} else {
			s.Require().Error(err, tc.name)
		}
	}
}

func (s *coinTestSuite) TestMinMax() {
	one := math.OneInt()
	two := math.NewInt(2)

	cases := []struct {
		name   string
		input1 sdk.Coins
		input2 sdk.Coins
		min    sdk.Coins
		max    sdk.Coins
	}{
		{"zero-zero", sdk.Coins{}, sdk.Coins{}, sdk.Coins{}, sdk.Coins{}},
		{"zero-one", sdk.Coins{}, sdk.Coins{{testDenom1, one}}, sdk.Coins{}, sdk.Coins{{testDenom1, one}}},
		{"two-zero", sdk.Coins{{testDenom2, two}}, sdk.Coins{}, sdk.Coins{}, sdk.Coins{{testDenom2, two}}},
		{"disjoint", sdk.Coins{{testDenom1, one}}, sdk.Coins{{testDenom2, two}}, sdk.Coins{}, sdk.Coins{{testDenom1, one}, {testDenom2, two}}},
		{
			"overlap",
			sdk.Coins{{testDenom1, one}, {testDenom2, two}},
			sdk.Coins{{testDenom1, two}, {testDenom2, one}},
			sdk.Coins{{testDenom1, one}, {testDenom2, one}},
			sdk.Coins{{testDenom1, two}, {testDenom2, two}},
		},
		{
			"one-one",
			sdk.Coins{{testDenom2, one}},
			sdk.Coins{{testDenom1, one}},
			sdk.Coins{},
			sdk.Coins{{testDenom1, one}, {testDenom2, one}},
		},
	}

	for _, tc := range cases {
		min := tc.input1.Min(tc.input2)
		max := tc.input1.Max(tc.input2)
		s.Require().True(min.Equal(tc.min), tc.name)
		s.Require().True(max.Equal(tc.max), tc.name)
	}
}

func (s *coinTestSuite) TestCoinsGT() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().False(sdk.Coins{}.IsAllGT(sdk.Coins{}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAllGT(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllGT(sdk.Coins{{testDenom2, two}}))
}

func (s *coinTestSuite) TestCoinsLT() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().False(sdk.Coins{}.IsAllLT(sdk.Coins{}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllLT(sdk.Coins{}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllLT(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllLT(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLT(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLT(sdk.Coins{{testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLT(sdk.Coins{{testDenom1, one}, {testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLT(sdk.Coins{{testDenom1, two}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{}.IsAllLT(sdk.Coins{{testDenom1, one}}))
}

func (s *coinTestSuite) TestCoinsLTE() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().True(sdk.Coins{}.IsAllLTE(sdk.Coins{}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllLTE(sdk.Coins{}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAllLTE(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllLTE(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLTE(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLTE(sdk.Coins{{testDenom2, two}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLTE(sdk.Coins{{testDenom1, one}, {testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllLTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{}.IsAllLTE(sdk.Coins{{testDenom1, one}}))
}

func (s *coinTestSuite) TestParseCoins() {
	one := math.OneInt()

	cases := []struct {
		input    string
		valid    bool      // if false, we expect an error on parse
		expected sdk.Coins // if valid is true, make sure this is returned
	}{
		{"", true, nil},
		{"0stake", true, sdk.Coins{}}, // remove zero coins
		{"0stake,1foo,99bar", true, sdk.Coins{{"bar", math.NewInt(99)}, {"foo", one}}}, // remove zero coins
		{"1foo", true, sdk.Coins{{"foo", one}}},
		{"10btc,1atom,20btc", false, nil},
		{"10bar", true, sdk.Coins{{"bar", math.NewInt(10)}}},
		{"99bar,1foo", true, sdk.Coins{{"bar", math.NewInt(99)}, {"foo", one}}},
		{"98 bar , 1 foo  ", true, sdk.Coins{{"bar", math.NewInt(98)}, {"foo", one}}},
		{"  55\t \t bling\n", true, sdk.Coins{{"bling", math.NewInt(55)}}},
		{"2foo, 97 bar", true, sdk.Coins{{"bar", math.NewInt(97)}, {"foo", math.NewInt(2)}}},
		{"5 mycoin,", false, nil},                            // no empty coins in a list
		{"2 3foo, 97 bar", false, nil},                       // 3foo is invalid coin name
		{"11me coin, 12you coin", false, nil},                // no spaces in coin names
		{"1.2btc", true, sdk.Coins{{"btc", math.NewInt(1)}}}, // amount can be decimal, will get truncated
		{"5foo:bar", true, sdk.Coins{{"foo:bar", math.NewInt(5)}}},
		{"10atom10", true, sdk.Coins{{"atom10", math.NewInt(10)}}},
		{"200transfer/channelToA/uatom", true, sdk.Coins{{"transfer/channelToA/uatom", math.NewInt(200)}}},
		{"50ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", true, sdk.Coins{{"ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", math.NewInt(50)}}},
		{"120000000000000000000000000000000000000000000000000000000000000000000000000000btc", false, nil},
	}

	for tcIndex, tc := range cases {
		res, err := sdk.ParseCoinsNormalized(tc.input)
		if !tc.valid {
			s.Require().Error(err, "%s: %#v. tc #%d", tc.input, res, tcIndex)
		} else if s.Assert().Nil(err, "%s: %+v", tc.input, err) {
			s.Require().Equal(tc.expected, res, "coin parsing was incorrect, tc #%d", tcIndex)
		}
	}
}

func (s *coinTestSuite) TestValidateDenom() {
	cases := []struct {
		input string
		valid bool
	}{
		{"", false},
		{"stake", true},
		{"stake,", false},
		{"me coin", false},
		{"me coin much", false},
		{"not a coin", false},
		{"foo:bar", true}, // special characters '/' | ':' | '.' | '_' | '-' are allowed
		{"atom10", true},  // number in denom is allowed
		{"transfer/channelToA/uatom", true},
		{"ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", true},
	}

	for _, tc := range cases {
		err := sdk.ValidateDenom(tc.input)
		if !tc.valid {
			s.Require().Error(err)
		} else {
			s.Require().NoError(err)
		}
	}
}

func (s *coinTestSuite) TestSortCoins() {
	good := sdk.Coins{
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("mineral", 1),
		sdk.NewInt64Coin("tree", 1),
	}
	empty := sdk.Coins{
		sdk.NewInt64Coin("gold", 0),
	}
	badSort1 := sdk.Coins{
		sdk.NewInt64Coin("tree", 1),
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("mineral", 1),
	}
	badSort2 := sdk.Coins{ // both are after the first one, but the second and third are in the wrong order
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("tree", 1),
		sdk.NewInt64Coin("mineral", 1),
	}
	badAmt := sdk.Coins{
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("tree", 0),
		sdk.NewInt64Coin("mineral", 1),
	}
	dup := sdk.Coins{
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("mineral", 1),
	}

	cases := []struct {
		name  string
		coins sdk.Coins
		validBefore,
		validAfter bool
	}{
		{"valid coins", good, true, true},
		{"empty coins", empty, false, false},
		{"bad sort (1)", badSort1, false, true},
		{"bad sort (2)", badSort2, false, true},
		{"zero value coin", badAmt, false, false},
		{"duplicate coins", dup, false, false},
	}

	for _, tc := range cases {
		err := tc.coins.Validate()
		if tc.validBefore {
			s.Require().NoError(err, tc.name)
		} else {
			s.Require().Error(err, tc.name)
		}

		tc.coins.Sort()

		err = tc.coins.Validate()
		if tc.validAfter {
			s.Require().NoError(err, tc.name)
		} else {
			s.Require().Error(err, tc.name)
		}
	}
}

func (s *coinTestSuite) TestSearch() {
	require := s.Require()
	case0 := sdk.Coins{}
	case1 := sdk.Coins{
		sdk.NewInt64Coin("gold", 0),
	}
	case2 := sdk.Coins{
		sdk.NewInt64Coin("gas", 1),
		sdk.NewInt64Coin("mineral", 1),
		sdk.NewInt64Coin("tree", 1),
	}
	case3 := sdk.Coins{
		sdk.NewInt64Coin("mineral", 1),
		sdk.NewInt64Coin("tree", 1),
	}
	case4 := sdk.Coins{
		sdk.NewInt64Coin("gas", 8),
	}

	amountOfCases := []struct {
		coins           sdk.Coins
		amountOf        int64
		amountOfSpace   int64
		amountOfGAS     int64
		amountOfMINERAL int64
		amountOfTREE    int64
	}{
		{case0, 0, 0, 0, 0, 0},
		{case1, 0, 0, 0, 0, 0},
		{case2, 0, 0, 1, 1, 1},
		{case3, 0, 0, 0, 1, 1},
		{case4, 0, 0, 8, 0, 0},
	}

	s.Run("AmountOf", func() {
		for i, tc := range amountOfCases {
			require.Equal(math.NewInt(tc.amountOfGAS), tc.coins.AmountOf("gas"), i)
			require.Equal(math.NewInt(tc.amountOfMINERAL), tc.coins.AmountOf("mineral"), i)
			require.Equal(math.NewInt(tc.amountOfTREE), tc.coins.AmountOf("tree"), i)
		}
		require.Panics(func() { amountOfCases[0].coins.AmountOf("10Invalid") })
	})

	zeroCoin := sdk.Coin{}
	findCases := []struct {
		coins        sdk.Coins
		denom        string
		expectedOk   bool
		expectedCoin sdk.Coin
	}{
		{case0, "any", false, zeroCoin},
		{case1, "other", false, zeroCoin},
		{case1, "gold", true, case1[0]},
		{case4, "gas", true, case4[0]},
		{case2, "gas", true, case2[0]},
		{case2, "mineral", true, case2[1]},
		{case2, "tree", true, case2[2]},
		{case2, "other", false, zeroCoin},
	}
	s.Run("Find", func() {
		for i, tc := range findCases {
			ok, c := tc.coins.Find(tc.denom)
			require.Equal(tc.expectedOk, ok, i)
			require.Equal(tc.expectedCoin, c, i)
		}
	})
}

func (s *coinTestSuite) TestCoinsIsAnyGTE() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().False(sdk.Coins{}.IsAnyGTE(sdk.Coins{}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAnyGTE(sdk.Coins{}))
	s.Require().False(sdk.Coins{}.IsAnyGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAnyGTE(sdk.Coins{{testDenom1, two}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAnyGTE(sdk.Coins{{testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAnyGTE(sdk.Coins{{testDenom1, two}, {testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAnyGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().True(sdk.Coins{{testDenom1, two}}.IsAnyGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAnyGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{{testDenom2, two}}.IsAnyGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom2, one}}.IsAnyGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAnyGTE(sdk.Coins{{testDenom1, one}, {testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAnyGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{{"xxx", one}, {"yyy", one}}.IsAnyGTE(sdk.Coins{{testDenom2, one}, {"ccc", one}, {"yyy", one}, {"zzz", one}}))
}

func (s *coinTestSuite) TestCoinsIsAllGT() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().False(sdk.Coins{}.IsAllGT(sdk.Coins{}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{}))
	s.Require().False(sdk.Coins{}.IsAllGT(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom1, two}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAllGT(sdk.Coins{{testDenom1, two}, {testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom1, one}}))
	s.Require().True(sdk.Coins{{testDenom1, two}}.IsAllGT(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGT(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom2, two}}.IsAllGT(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom2, one}}.IsAllGT(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAllGT(sdk.Coins{{testDenom1, one}, {testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllGT(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{"xxx", one}, {"yyy", one}}.IsAllGT(sdk.Coins{{testDenom2, one}, {"ccc", one}, {"yyy", one}, {"zzz", one}}))
}

func (s *coinTestSuite) TestCoinsIsAllGTE() {
	one := math.OneInt()
	two := math.NewInt(2)

	s.Require().True(sdk.Coins{}.IsAllGTE(sdk.Coins{}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAllGTE(sdk.Coins{}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllGTE(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllGTE(sdk.Coins{{testDenom2, two}}))
	s.Require().False(sdk.Coins{}.IsAllGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGTE(sdk.Coins{{testDenom1, two}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGTE(sdk.Coins{{testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAllGTE(sdk.Coins{{testDenom1, two}, {testDenom2, one}}))
	s.Require().True(sdk.Coins{{testDenom1, one}}.IsAllGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().True(sdk.Coins{{testDenom1, two}}.IsAllGTE(sdk.Coins{{testDenom1, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}}.IsAllGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom2, two}}.IsAllGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{testDenom2, one}}.IsAllGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().True(sdk.Coins{{testDenom1, one}, {testDenom2, two}}.IsAllGTE(sdk.Coins{{testDenom1, one}, {testDenom2, one}}))
	s.Require().False(sdk.Coins{{testDenom1, one}, {testDenom2, one}}.IsAllGTE(sdk.Coins{{testDenom1, one}, {testDenom2, two}}))
	s.Require().False(sdk.Coins{{"xxx", one}, {"yyy", one}}.IsAllGTE(sdk.Coins{{testDenom2, one}, {"ccc", one}, {"yyy", one}, {"zzz", one}}))
}

func (s *coinTestSuite) TestNewCoins() {
	tenatom := sdk.NewInt64Coin("atom", 10)
	tenbtc := sdk.NewInt64Coin("btc", 10)
	zeroeth := sdk.NewInt64Coin("eth", 0)
	invalidCoin := sdk.Coin{"0ETH", math.OneInt()}
	tests := []struct {
		name      string
		coins     sdk.Coins
		want      sdk.Coins
		wantPanic bool
	}{
		{"empty args", []sdk.Coin{}, sdk.Coins{}, false},
		{"one coin", []sdk.Coin{tenatom}, sdk.Coins{tenatom}, false},
		{"sort after create", []sdk.Coin{tenbtc, tenatom}, sdk.Coins{tenatom, tenbtc}, false},
		{"sort and remove zeroes", []sdk.Coin{zeroeth, tenbtc, tenatom}, sdk.Coins{tenatom, tenbtc}, false},
		{"panic on dups", []sdk.Coin{tenatom, tenatom}, sdk.Coins{}, true},
		{"panic on invalid coin", []sdk.Coin{invalidCoin, tenatom}, sdk.Coins{}, true},
	}
	for _, tt := range tests {
		if tt.wantPanic {
			s.Require().Panics(func() { sdk.NewCoins(tt.coins...) })
			continue
		}
		got := sdk.NewCoins(tt.coins...)
		s.Require().True(got.Equal(tt.want))
	}
}

func (s *coinTestSuite) TestCoinsIsAnyGT() {
	twoAtom := sdk.NewInt64Coin("atom", 2)
	fiveAtom := sdk.NewInt64Coin("atom", 5)
	threeEth := sdk.NewInt64Coin("eth", 3)
	sixEth := sdk.NewInt64Coin("eth", 6)
	twoBtc := sdk.NewInt64Coin("btc", 2)

	tests := []struct {
		name    string
		coinsA  sdk.Coins
		coinsB  sdk.Coins
		expPass bool
	}{
		{"{} ≤ {}", sdk.Coins{}, sdk.Coins{}, false},
		{"{} ≤ 5atom", sdk.Coins{}, sdk.Coins{fiveAtom}, false},
		{"5atom > 2atom", sdk.Coins{fiveAtom}, sdk.Coins{twoAtom}, true},
		{"2atom ≤ 5atom", sdk.Coins{twoAtom}, sdk.Coins{fiveAtom}, false},
		{"2atom,6eth > 2btc,5atom,3eth", sdk.Coins{twoAtom, sixEth}, sdk.Coins{twoBtc, fiveAtom, threeEth}, true},
		{"2btc,2atom,3eth ≤ 5atom,6eth", sdk.Coins{twoBtc, twoAtom, threeEth}, sdk.Coins{fiveAtom, sixEth}, false},
		{"2atom,6eth ≤ 2btc,5atom", sdk.Coins{twoAtom, sixEth}, sdk.Coins{twoBtc, fiveAtom}, false},
	}

	for _, tc := range tests {
		s.Require().True(tc.expPass == tc.coinsA.IsAnyGT(tc.coinsB), tc.name)
	}
}

func (s *coinTestSuite) TestCoinsIsAnyNil() {
	twoAtom := sdk.NewInt64Coin("atom", 2)
	fiveAtom := sdk.NewInt64Coin("atom", 5)
	threeEth := sdk.NewInt64Coin("eth", 3)
	nilAtom := sdk.Coin{Denom: "atom"}

	s.Require().True(sdk.Coins{twoAtom, fiveAtom, threeEth, nilAtom}.IsAnyNil())
	s.Require().True(sdk.Coins{twoAtom, nilAtom, fiveAtom, threeEth}.IsAnyNil())
	s.Require().True(sdk.Coins{nilAtom, twoAtom, fiveAtom, threeEth}.IsAnyNil())
	s.Require().False(sdk.Coins{twoAtom, fiveAtom, threeEth}.IsAnyNil())
}

func (s *coinTestSuite) TestMarshalJSONCoins() {
	cdc := codec.NewLegacyAmino()
	sdk.RegisterLegacyAminoCodec(cdc)

	testCases := []struct {
		name      string
		input     sdk.Coins
		strOutput string
	}{
		{"nil coins", nil, `[]`},
		{"empty coins", sdk.Coins{}, `[]`},
		{"non-empty coins", sdk.NewCoins(sdk.NewInt64Coin("foo", 50)), `[{"denom":"foo","amount":"50"}]`},
	}

	for _, tc := range testCases {
		bz, err := cdc.MarshalJSON(tc.input)
		s.Require().NoError(err)
		s.Require().Equal(tc.strOutput, string(bz))

		var newCoins sdk.Coins
		s.Require().NoError(cdc.UnmarshalJSON(bz, &newCoins))

		if tc.input.Empty() {
			s.Require().Nil(newCoins)
		} else {
			s.Require().Equal(tc.input, newCoins)
		}
	}
}

func (s *coinTestSuite) TestCoinValidate() {
	testCases := []struct {
		name    string
		coin    sdk.Coin
		wantErr string
	}{
		{"nil coin: nil Amount", sdk.Coin{}, "invalid denom"},
		{"non-blank coin, nil Amount", sdk.Coin{Denom: "atom"}, "amount is nil"},
		{"valid coin", sdk.Coin{Denom: "atom", Amount: math.NewInt(100)}, ""},
		{"negative coin", sdk.Coin{Denom: "atom", Amount: math.NewInt(-999)}, "negative coin amount"},
	}

	for _, tc := range testCases {

		t := s.T()
		t.Run(tc.name, func(t *testing.T) {
			err := tc.coin.Validate()
			switch {
			case tc.wantErr == "":
				if err != nil {
					t.Errorf("Unexpected error: %v", err)
				}
				return
			case err == nil:
				t.Error("Expected an error")
			case !strings.Contains(err.Error(), tc.wantErr):
				t.Errorf("Error mismatch\n\tGot:  %q\nWant: %q", err, tc.wantErr)
			}
		})
	}
}

func (s *coinTestSuite) TestCoinAminoEncoding() {
	cdc := codec.NewLegacyAmino()
	c := sdk.NewInt64Coin(testDenom1, 5)

	bz1, err := cdc.Marshal(c)
	s.Require().NoError(err)

	bz2, err := cdc.MarshalLengthPrefixed(c)
	s.Require().NoError(err)

	bz3, err := c.Marshal()
	s.Require().NoError(err)
	s.Require().Equal(bz1, bz3)
	s.Require().Equal(bz2[1:], bz3)
}

func (s *coinTestSuite) TestGetDenomByIndex() {
	twoAtom := sdk.NewInt64Coin("atom", 2)
	threeEth := sdk.NewInt64Coin("eth", 3)

	coins := sdk.Coins{twoAtom, threeEth}

	actual := coins.GetDenomByIndex(0)
	s.Require().Equal("atom", actual)
	actual = coins.GetDenomByIndex(1)
	s.Require().Equal("eth", actual)
}

func (s *coinTestSuite) TestIsAllPositive() {
	negative := sdk.Coin{
		Denom:  "atom",
		Amount: math.NewInt(-2),
	}

	cases := map[string]struct {
		coins    sdk.Coins
		expected bool
	}{
		"Coins with zero length": {
			coins:    sdk.Coins{},
			expected: false,
		},
		"Coins with negative value": {
			coins:    sdk.Coins{negative, sdk.NewInt64Coin("atom", 2)},
			expected: false,
		},
		"Coins only with positive values": {
			coins:    sdk.Coins{sdk.NewInt64Coin("atom", 2), sdk.NewInt64Coin("eth", 10)},
			expected: true,
		},
	}

	for name, tc := range cases {
		c := tc
		s.T().Run(name, func(t *testing.T) {
			s.Require().Equal(c.expected, c.coins.IsAllPositive())
		})
	}
}

func (s *coinTestSuite) TestParseCoin() {
	one := math.OneInt()

	cases := []struct {
		input    string
		valid    bool     // if false, we expect an error on parse
		expected sdk.Coin // if valid is true, make sure this is returned
	}{
		{"", false, sdk.Coin{}},
		{"0stake", true, sdk.Coin{Denom: "stake", Amount: math.NewInt(0)}}, // remove zero coins
		{"1foo", true, sdk.Coin{Denom: "foo", Amount: one}},
		{"10btc", true, sdk.Coin{Denom: "btc", Amount: math.NewInt(10)}},
		{"10bar", true, sdk.Coin{Denom: "bar", Amount: math.NewInt(10)}},
		{"98 bar", true, sdk.Coin{Denom: "bar", Amount: math.NewInt(98)}},
		{"  55\t \t bling\n", true, sdk.Coin{Denom: "bling", Amount: math.NewInt(55)}},
		{"2foo", true, sdk.Coin{Denom: "foo", Amount: math.NewInt(2)}},
		{"5 mycoin,", false, sdk.Coin{}},                                 // no empty coins in a list
		{"2 3foo", false, sdk.Coin{}},                                    // 3foo is invalid coin name
		{"11me coin", false, sdk.Coin{}},                                 // no spaces in coin names
		{"1.2btc", true, sdk.Coin{Denom: "btc", Amount: math.NewInt(1)}}, // amount can be decimal, will get truncated
		{"5foo:bar", true, sdk.Coin{Denom: "foo:bar", Amount: math.NewInt(5)}},
		{"10atom10", true, sdk.Coin{Denom: "atom10", Amount: math.NewInt(10)}},
		{"200transfer/channelToA/uatom", true, sdk.Coin{Denom: "transfer/channelToA/uatom", Amount: math.NewInt(200)}},
		{"50ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", true, sdk.Coin{Denom: "ibc/7F1D3FCF4AE79E1554D670D1AD949A9BA4E4A3C76C63093E17E446A46061A7A2", Amount: math.NewInt(50)}},
		{"120000000000000000000000000000000000000000000000000000000000000000000000000000btc", false, sdk.Coin{}},
	}

	for tcIndex, tc := range cases {
		res, err := sdk.ParseCoinNormalized(tc.input)
		if !tc.valid {
			s.Require().Error(err, "%s: %#v. tc #%d", tc.input, res, tcIndex)
		} else if s.Assert().Nil(err) {
			s.Require().Equal(tc.expected, res, "coin parsing was incorrect, tc #%d", tcIndex)
		}
	}
}
