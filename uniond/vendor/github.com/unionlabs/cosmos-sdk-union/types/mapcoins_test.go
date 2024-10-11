package types_test

import (
	"github.com/stretchr/testify/require"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

func (s *coinTestSuite) TestMapCoinsAdd() {
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
		expected := tc.inputOne.Add(tc.inputTwo...)
		m := sdk.NewMapCoins(tc.inputOne)
		m.Add(tc.inputTwo...)
		res := m.ToCoins()
		s.Require().True(res.IsValid())
		require.Equal(s.T(), expected, res, tc.msg)
	}
}
