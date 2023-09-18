package types

import (
	"testing"

	"github.com/stretchr/testify/suite"
)

var (
	atom  = "atom"  // 1 (base denom unit)
	matom = "matom" // 10^-3 (milli)
	uatom = "uatom" // 10^-6 (micro)
	natom = "natom" // 10^-9 (nano)
)

type internalDenomTestSuite struct {
	suite.Suite
}

func TestInternalDenomTestSuite(t *testing.T) {
	suite.Run(t, new(internalDenomTestSuite))
}

func (s *internalDenomTestSuite) TestRegisterDenom() {
	atomUnit := OneDec() // 1 (base denom unit)

	s.Require().NoError(RegisterDenom(atom, atomUnit))
	s.Require().Error(RegisterDenom(atom, atomUnit))

	res, ok := GetDenomUnit(atom)
	s.Require().True(ok)
	s.Require().Equal(atomUnit, res)

	res, ok = GetDenomUnit(matom)
	s.Require().False(ok)
	s.Require().Equal(ZeroDec(), res)

	// reset registration
	baseDenom = ""
	denomUnits = map[string]Dec{}
}

func (s *internalDenomTestSuite) TestConvertCoins() {
	atomUnit := OneDec() // 1 (base denom unit)
	s.Require().NoError(RegisterDenom(atom, atomUnit))

	matomUnit := NewDecWithPrec(1, 3) // 10^-3 (milli)
	s.Require().NoError(RegisterDenom(matom, matomUnit))

	uatomUnit := NewDecWithPrec(1, 6) // 10^-6 (micro)
	s.Require().NoError(RegisterDenom(uatom, uatomUnit))

	natomUnit := NewDecWithPrec(1, 9) // 10^-9 (nano)
	s.Require().NoError(RegisterDenom(natom, natomUnit))

	res, err := GetBaseDenom()
	s.Require().NoError(err)
	s.Require().Equal(res, natom)
	s.Require().Equal(NormalizeCoin(NewCoin(uatom, NewInt(1))), NewCoin(natom, NewInt(1000)))
	s.Require().Equal(NormalizeCoin(NewCoin(matom, NewInt(1))), NewCoin(natom, NewInt(1000000)))
	s.Require().Equal(NormalizeCoin(NewCoin(atom, NewInt(1))), NewCoin(natom, NewInt(1000000000)))

	coins, err := ParseCoinsNormalized("1atom,1matom,1uatom")
	s.Require().NoError(err)
	s.Require().Equal(coins, Coins{
		Coin{natom, NewInt(1000000000)},
		Coin{natom, NewInt(1000000)},
		Coin{natom, NewInt(1000)},
	})

	testCases := []struct {
		input  Coin
		denom  string
		result Coin
		expErr bool
	}{
		{NewCoin("foo", ZeroInt()), atom, Coin{}, true},
		{NewCoin(atom, ZeroInt()), "foo", Coin{}, true},
		{NewCoin(atom, ZeroInt()), "FOO", Coin{}, true},

		{NewCoin(atom, NewInt(5)), matom, NewCoin(matom, NewInt(5000)), false},       // atom => matom
		{NewCoin(atom, NewInt(5)), uatom, NewCoin(uatom, NewInt(5000000)), false},    // atom => uatom
		{NewCoin(atom, NewInt(5)), natom, NewCoin(natom, NewInt(5000000000)), false}, // atom => natom

		{NewCoin(uatom, NewInt(5000000)), matom, NewCoin(matom, NewInt(5000)), false},       // uatom => matom
		{NewCoin(uatom, NewInt(5000000)), natom, NewCoin(natom, NewInt(5000000000)), false}, // uatom => natom
		{NewCoin(uatom, NewInt(5000000)), atom, NewCoin(atom, NewInt(5)), false},            // uatom => atom

		{NewCoin(matom, NewInt(5000)), natom, NewCoin(natom, NewInt(5000000000)), false}, // matom => natom
		{NewCoin(matom, NewInt(5000)), uatom, NewCoin(uatom, NewInt(5000000)), false},    // matom => uatom
	}

	for i, tc := range testCases {
		res, err := ConvertCoin(tc.input, tc.denom)
		s.Require().Equal(
			tc.expErr, err != nil,
			"unexpected error; tc: #%d, input: %s, denom: %s", i+1, tc.input, tc.denom,
		)
		s.Require().Equal(
			tc.result, res,
			"invalid result; tc: #%d, input: %s, denom: %s", i+1, tc.input, tc.denom,
		)
	}

	// reset registration
	baseDenom = ""
	denomUnits = map[string]Dec{}
}

func (s *internalDenomTestSuite) TestConvertDecCoins() {
	atomUnit := OneDec() // 1 (base denom unit)
	s.Require().NoError(RegisterDenom(atom, atomUnit))

	matomUnit := NewDecWithPrec(1, 3) // 10^-3 (milli)
	s.Require().NoError(RegisterDenom(matom, matomUnit))

	uatomUnit := NewDecWithPrec(1, 6) // 10^-6 (micro)
	s.Require().NoError(RegisterDenom(uatom, uatomUnit))

	natomUnit := NewDecWithPrec(1, 9) // 10^-9 (nano)
	s.Require().NoError(RegisterDenom(natom, natomUnit))

	res, err := GetBaseDenom()
	s.Require().NoError(err)
	s.Require().Equal(res, natom)
	s.Require().Equal(NormalizeDecCoin(NewDecCoin(uatom, NewInt(1))), NewDecCoin(natom, NewInt(1000)))
	s.Require().Equal(NormalizeDecCoin(NewDecCoin(matom, NewInt(1))), NewDecCoin(natom, NewInt(1000000)))
	s.Require().Equal(NormalizeDecCoin(NewDecCoin(atom, NewInt(1))), NewDecCoin(natom, NewInt(1000000000)))

	coins, err := ParseCoinsNormalized("0.1atom,0.1matom,0.1uatom")
	s.Require().NoError(err)
	s.Require().Equal(coins, Coins{
		Coin{natom, NewInt(100000000)},
		Coin{natom, NewInt(100000)},
		Coin{natom, NewInt(100)},
	})

	testCases := []struct {
		input  DecCoin
		denom  string
		result DecCoin
		expErr bool
	}{
		{NewDecCoin("foo", ZeroInt()), atom, DecCoin{}, true},
		{NewDecCoin(atom, ZeroInt()), "foo", DecCoin{}, true},
		{NewDecCoin(atom, ZeroInt()), "FOO", DecCoin{}, true},

		// 0.5atom
		{NewDecCoinFromDec(atom, NewDecWithPrec(5, 1)), matom, NewDecCoin(matom, NewInt(500)), false},       // atom => matom
		{NewDecCoinFromDec(atom, NewDecWithPrec(5, 1)), uatom, NewDecCoin(uatom, NewInt(500000)), false},    // atom => uatom
		{NewDecCoinFromDec(atom, NewDecWithPrec(5, 1)), natom, NewDecCoin(natom, NewInt(500000000)), false}, // atom => natom

		{NewDecCoin(uatom, NewInt(5000000)), matom, NewDecCoin(matom, NewInt(5000)), false},       // uatom => matom
		{NewDecCoin(uatom, NewInt(5000000)), natom, NewDecCoin(natom, NewInt(5000000000)), false}, // uatom => natom
		{NewDecCoin(uatom, NewInt(5000000)), atom, NewDecCoin(atom, NewInt(5)), false},            // uatom => atom

		{NewDecCoin(matom, NewInt(5000)), natom, NewDecCoin(natom, NewInt(5000000000)), false}, // matom => natom
		{NewDecCoin(matom, NewInt(5000)), uatom, NewDecCoin(uatom, NewInt(5000000)), false},    // matom => uatom
	}

	for i, tc := range testCases {
		res, err := ConvertDecCoin(tc.input, tc.denom)
		s.Require().Equal(
			tc.expErr, err != nil,
			"unexpected error; tc: #%d, input: %s, denom: %s", i+1, tc.input, tc.denom,
		)
		s.Require().Equal(
			tc.result, res,
			"invalid result; tc: #%d, input: %s, denom: %s", i+1, tc.input, tc.denom,
		)
	}

	// reset registration
	baseDenom = ""
	denomUnits = map[string]Dec{}
}

func (s *internalDenomTestSuite) TestDecOperationOrder() {
	dec, err := NewDecFromStr("11")
	s.Require().NoError(err)
	s.Require().NoError(RegisterDenom("unit1", dec))
	dec, err = NewDecFromStr("100000011")
	s.Require().NoError(RegisterDenom("unit2", dec))

	coin, err := ConvertCoin(NewCoin("unit1", NewInt(100000011)), "unit2")
	s.Require().NoError(err)
	s.Require().Equal(coin, NewCoin("unit2", NewInt(11)))

	// reset registration
	baseDenom = ""
	denomUnits = map[string]Dec{}
}
