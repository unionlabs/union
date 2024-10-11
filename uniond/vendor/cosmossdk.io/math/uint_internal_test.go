package math

import (
	"math/big"
	"math/rand"
	"strconv"
	"testing"

	"github.com/stretchr/testify/suite"
)

type uintInternalTestSuite struct {
	suite.Suite
}

func TestUintInternalTestSuite(t *testing.T) {
	suite.Run(t, new(uintInternalTestSuite))
}

func (s *uintInternalTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *uintInternalTestSuite) TestIdentUint() {
	for d := 0; d < 1000; d++ {
		n := rand.Uint64()
		i := NewUint(n)

		ifromstr := NewUintFromString(strconv.FormatUint(n, 10))

		cases := []uint64{
			i.Uint64(),
			i.BigInt().Uint64(),
			i.i.Uint64(),
			ifromstr.Uint64(),
			NewUintFromBigInt(new(big.Int).SetUint64(n)).Uint64(),
		}

		for tcnum, tc := range cases {
			s.Require().Equal(n, tc, "Uint is modified during conversion. tc #%d", tcnum)
		}
	}
}

func (s *uintInternalTestSuite) TestUintSize() {
	x := Uint{i: nil}
	s.Require().Equal(1, x.Size())
	x = NewUint(0)
	s.Require().Equal(1, x.Size())
	x = NewUint(10)
	s.Require().Equal(2, x.Size())
	x = NewUint(100)
	s.Require().Equal(3, x.Size())
}
