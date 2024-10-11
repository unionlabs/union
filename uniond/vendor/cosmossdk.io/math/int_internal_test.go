package math

import (
	"math/big"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/suite"
)

type internalIntTestSuite struct {
	suite.Suite
}

func TestInternalIntTestSuite(t *testing.T) {
	suite.Run(t, new(internalIntTestSuite))
}

func (s *internalIntTestSuite) TestEncodingRandom() {
	for i := 0; i < 1000; i++ {
		n := rand.Int63()
		ni := NewInt(n)
		var ri Int

		str, err := ni.Marshal()
		s.Require().Nil(err)
		err = (&ri).Unmarshal(str)
		s.Require().Nil(err)

		s.Require().Equal(ni, ri, "binary mismatch; tc #%d, expected %s, actual %s", i, ni.String(), ri.String())
		s.Require().True(ni.i != ri.i, "pointer addresses are equal; tc #%d", i)

		bz, err := ni.MarshalJSON()
		s.Require().Nil(err)
		err = (&ri).UnmarshalJSON(bz)
		s.Require().Nil(err)

		s.Require().Equal(ni, ri, "json mismatch; tc #%d, expected %s, actual %s", i, ni.String(), ri.String())
		s.Require().True(ni.i != ri.i, "pointer addresses are equal; tc #%d", i)
	}

	for i := 0; i < 1000; i++ {
		n := rand.Uint64()
		ni := NewUint(n)
		var ri Uint

		str, err := ni.Marshal()
		s.Require().Nil(err)
		err = (&ri).Unmarshal(str)
		s.Require().Nil(err)

		s.Require().Equal(ni, ri, "binary mismatch; tc #%d, expected %s, actual %s", i, ni.String(), ri.String())
		s.Require().True(ni.i != ri.i, "pointer addresses are equal; tc #%d", i)

		bz, err := ni.MarshalJSON()
		s.Require().Nil(err)
		err = (&ri).UnmarshalJSON(bz)
		s.Require().Nil(err)

		s.Require().Equal(ni, ri, "json mismatch; tc #%d, expected %s, actual %s", i, ni.String(), ri.String())
		s.Require().True(ni.i != ri.i, "pointer addresses are equal; tc #%d", i)
	}
}

func (s *internalIntTestSuite) TestSerializationOverflow() {
	bx, _ := new(big.Int).SetString("115792089237316195423570985008687907853269984665640564039457584007913129639936", 10)
	x := Int{bx}
	y := new(Int)

	bz, err := x.Marshal()
	s.Require().NoError(err)

	// require deserialization to fail due to overflow
	s.Require().Error(y.Unmarshal(bz))

	// require JSON deserialization to fail due to overflow
	bz, err = x.MarshalJSON()
	s.Require().NoError(err)

	s.Require().Error(y.UnmarshalJSON(bz))
}

func (s *internalIntTestSuite) TestDeserializeMaxERC20() {
	bx, _ := new(big.Int).SetString("115792089237316195423570985008687907853269984665640564039457584007913129639935", 10)
	x := Int{bx}
	y := new(Int)

	bz, err := x.Marshal()
	s.Require().NoError(err)

	// require deserialization to be successful
	s.Require().NoError(y.Unmarshal(bz))

	// require JSON deserialization to succeed
	bz, err = x.MarshalJSON()
	s.Require().NoError(err)

	s.Require().NoError(y.UnmarshalJSON(bz))
}

func (s *internalIntTestSuite) TestImmutabilityArithInt() {
	size := 500

	ops := []intOp{
		applyWithRand(Int.Add, (*big.Int).Add),
		applyWithRand(Int.Sub, (*big.Int).Sub),
		applyWithRand(Int.Mul, (*big.Int).Mul),
		applyWithRand(Int.Quo, (*big.Int).Quo),
		applyRawWithRand(Int.AddRaw, (*big.Int).Add),
		applyRawWithRand(Int.SubRaw, (*big.Int).Sub),
		applyRawWithRand(Int.MulRaw, (*big.Int).Mul),
		applyRawWithRand(Int.QuoRaw, (*big.Int).Quo),
	}

	for i := 0; i < 100; i++ {
		uis := make([]Int, size)
		bis := make([]*big.Int, size)

		n := rand.Int63()
		ui := NewInt(n)
		bi := new(big.Int).SetInt64(n)

		for j := 0; j < size; j++ {
			op := ops[rand.Intn(len(ops))]
			uis[j], bis[j] = op(ui, bi)
		}

		for j := 0; j < size; j++ {
			s.Require().Equal(0, bis[j].Cmp(uis[j].BigInt()), "Int is different from *big.Int. tc #%d, Int %s, *big.Int %s", j, uis[j].String(), bis[j].String())
			s.Require().Equal(NewIntFromBigInt(bis[j]), uis[j], "Int is different from *big.Int. tc #%d, Int %s, *big.Int %s", j, uis[j].String(), bis[j].String())
			s.Require().True(uis[j].i != bis[j], "Pointer addresses are equal. tc #%d, Int %s, *big.Int %s", j, uis[j].String(), bis[j].String())
		}
	}
}

type (
	intOp      func(Int, *big.Int) (Int, *big.Int)
	bigIntFunc func(*big.Int, *big.Int, *big.Int) *big.Int
)

func applyWithRand(intFn func(Int, Int) Int, bigIntFn bigIntFunc) intOp {
	return func(integer Int, bigInteger *big.Int) (Int, *big.Int) {
		r := rand.Int63()
		br := new(big.Int).SetInt64(r)
		return intFn(integer, NewInt(r)), bigIntFn(new(big.Int), bigInteger, br)
	}
}

func applyRawWithRand(intFn func(Int, int64) Int, bigIntFn bigIntFunc) intOp {
	return func(integer Int, bigInteger *big.Int) (Int, *big.Int) {
		r := rand.Int63()
		br := new(big.Int).SetInt64(r)
		return intFn(integer, r), bigIntFn(new(big.Int), bigInteger, br)
	}
}
