package math_test

import (
	"fmt"
	"math"
	"math/big"
	"math/rand"
	"strings"
	"testing"

	"github.com/stretchr/testify/suite"

	sdkmath "cosmossdk.io/math"
)

type uintTestSuite struct {
	suite.Suite
}

func TestUnitTestSuite(t *testing.T) {
	suite.Run(t, new(uintTestSuite))
}

func (s *uintTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *uintTestSuite) TestUintPanics() {
	// Max Uint = 1.15e+77
	// Min Uint = 0
	u1 := sdkmath.NewUint(0)
	u2 := sdkmath.OneUint()

	s.Require().Equal(uint64(0), u1.Uint64())
	s.Require().Equal(uint64(1), u2.Uint64())

	s.Require().Panics(func() { sdkmath.NewUintFromBigInt(big.NewInt(-5)) })
	s.Require().Panics(func() { sdkmath.NewUintFromString("-1") })
	s.Require().NotPanics(func() {
		s.Require().True(sdkmath.NewUintFromString("0").Equal(sdkmath.ZeroUint()))
		s.Require().True(sdkmath.NewUintFromString("5").Equal(sdkmath.NewUint(5)))
	})

	// Overflow check
	s.Require().True(u1.Add(u1).Equal(sdkmath.ZeroUint()))
	s.Require().True(u1.Add(sdkmath.OneUint()).Equal(sdkmath.OneUint()))
	s.Require().Equal(uint64(0), u1.Uint64())
	s.Require().Equal(uint64(1), sdkmath.OneUint().Uint64())
	s.Require().Panics(func() { u1.SubUint64(2) })
	s.Require().True(u1.SubUint64(0).Equal(sdkmath.ZeroUint()))
	s.Require().True(u2.Add(sdkmath.OneUint()).Sub(sdkmath.OneUint()).Equal(sdkmath.OneUint()))    // i2 == 1
	s.Require().True(u2.Add(sdkmath.OneUint()).Mul(sdkmath.NewUint(5)).Equal(sdkmath.NewUint(10))) // i2 == 10
	s.Require().True(sdkmath.NewUint(7).Quo(sdkmath.NewUint(2)).Equal(sdkmath.NewUint(3)))
	s.Require().True(sdkmath.NewUint(0).Quo(sdkmath.NewUint(2)).Equal(sdkmath.ZeroUint()))
	s.Require().True(sdkmath.NewUint(5).MulUint64(4).Equal(sdkmath.NewUint(20)))
	s.Require().True(sdkmath.NewUint(5).MulUint64(0).Equal(sdkmath.ZeroUint()))

	uintmax := sdkmath.NewUintFromBigInt(new(big.Int).Sub(new(big.Int).Exp(big.NewInt(2), big.NewInt(256), nil), big.NewInt(1)))
	uintmin := sdkmath.ZeroUint()

	// divs by zero
	s.Require().Panics(func() { sdkmath.OneUint().Mul(sdkmath.ZeroUint().SubUint64(uint64(1))) })
	s.Require().Panics(func() { sdkmath.OneUint().QuoUint64(0) })
	s.Require().Panics(func() { sdkmath.OneUint().Quo(sdkmath.ZeroUint()) })
	s.Require().Panics(func() { sdkmath.ZeroUint().QuoUint64(0) })
	s.Require().Panics(func() { sdkmath.OneUint().Quo(sdkmath.ZeroUint().Sub(sdkmath.OneUint())) })
	s.Require().Panics(func() { uintmax.Add(sdkmath.OneUint()) })
	s.Require().Panics(func() { uintmax.Incr() })
	s.Require().Panics(func() { uintmin.Sub(sdkmath.OneUint()) })
	s.Require().Panics(func() { uintmin.Decr() })

	s.Require().NotPanics(func() { sdkmath.Uint{}.BigInt() })

	s.Require().Equal(uint64(0), sdkmath.MinUint(sdkmath.ZeroUint(), sdkmath.OneUint()).Uint64())
	s.Require().Equal(uint64(1), sdkmath.MaxUint(sdkmath.ZeroUint(), sdkmath.OneUint()).Uint64())

	// comparison ops
	s.Require().True(
		sdkmath.OneUint().GT(sdkmath.ZeroUint()),
	)
	s.Require().False(
		sdkmath.OneUint().LT(sdkmath.ZeroUint()),
	)
	s.Require().True(
		sdkmath.OneUint().GTE(sdkmath.ZeroUint()),
	)
	s.Require().False(
		sdkmath.OneUint().LTE(sdkmath.ZeroUint()),
	)

	s.Require().False(sdkmath.ZeroUint().GT(sdkmath.OneUint()))
	s.Require().True(sdkmath.ZeroUint().LT(sdkmath.OneUint()))
	s.Require().False(sdkmath.ZeroUint().GTE(sdkmath.OneUint()))
	s.Require().True(sdkmath.ZeroUint().LTE(sdkmath.OneUint()))
}

func (s *uintTestSuite) TestIsNil() {
	s.Require().False(sdkmath.OneUint().IsNil())
	s.Require().True(sdkmath.Uint{}.IsNil())
}

func (s *uintTestSuite) TestConvertToBigIntMutativeForUint() {
	r := big.NewInt(30)
	i := sdkmath.NewUintFromBigInt(r)

	// Compare value of BigInt & BigIntMut
	s.Require().Equal(i.BigInt(), i.BigIntMut())

	// Modify BigIntMut() pointer and ensure i.BigIntMut() & i.BigInt() change
	p1 := i.BigIntMut()
	p1.SetInt64(40)
	s.Require().Equal(big.NewInt(40), i.BigIntMut())
	s.Require().Equal(big.NewInt(40), i.BigInt())

	// Modify big.Int() pointer and ensure i.BigIntMut() & i.BigInt() don't change
	p2 := i.BigInt()
	p2.SetInt64(50)
	s.Require().NotEqual(big.NewInt(50), i.BigIntMut())
	s.Require().NotEqual(big.NewInt(50), i.BigInt())
}

func (s *uintTestSuite) TestArithUint() {
	for d := 0; d < 1000; d++ {
		n1 := uint64(rand.Uint32())
		u1 := sdkmath.NewUint(n1)
		n2 := uint64(rand.Uint32())
		u2 := sdkmath.NewUint(n2)

		cases := []struct {
			ures sdkmath.Uint
			nres uint64
		}{
			{u1.Add(u2), n1 + n2},
			{u1.Mul(u2), n1 * n2},
			{u1.Quo(u2), n1 / n2},
			{u1.AddUint64(n2), n1 + n2},
			{u1.MulUint64(n2), n1 * n2},
			{u1.QuoUint64(n2), n1 / n2},
			{sdkmath.MinUint(u1, u2), minuint(n1, n2)},
			{sdkmath.MaxUint(u1, u2), maxuint(n1, n2)},
			{u1.Incr(), n1 + 1},
		}

		for tcnum, tc := range cases {
			s.Require().Equal(tc.nres, tc.ures.Uint64(), "Uint arithmetic operation does not match with uint64 operation. tc #%d", tcnum)
		}

		if n2 > n1 {
			n1, n2 = n2, n1
			u1, u2 = sdkmath.NewUint(n1), sdkmath.NewUint(n2)
		}

		subs := []struct {
			ures sdkmath.Uint
			nres uint64
		}{
			{u1.Sub(u2), n1 - n2},
			{u1.SubUint64(n2), n1 - n2},
			{u1.Decr(), n1 - 1},
		}

		for tcnum, tc := range subs {
			s.Require().Equal(tc.nres, tc.ures.Uint64(), "Uint subtraction does not match with uint64 operation. tc #%d", tcnum)
		}
	}
}

func (s *uintTestSuite) TestCompUint() {
	for d := 0; d < 10000; d++ {
		n1 := rand.Uint64()
		i1 := sdkmath.NewUint(n1)
		n2 := rand.Uint64()
		i2 := sdkmath.NewUint(n2)

		cases := []struct {
			ires bool
			nres bool
		}{
			{i1.Equal(i2), n1 == n2},
			{i1.GT(i2), n1 > n2},
			{i1.LT(i2), n1 < n2},
			{i1.GTE(i2), !i1.LT(i2)},
			{!i1.GTE(i2), i1.LT(i2)},
			{i1.LTE(i2), n1 <= n2},
			{i2.LTE(i1), n2 <= n1},
		}

		for tcnum, tc := range cases {
			s.Require().Equal(tc.nres, tc.ires, "Uint comparison operation does not match with uint64 operation. tc #%d", tcnum)
		}
	}
}

func (s *uintTestSuite) TestImmutabilityAllUint() {
	ops := []func(*sdkmath.Uint){
		func(i *sdkmath.Uint) { _ = i.Add(sdkmath.NewUint(rand.Uint64())) },
		func(i *sdkmath.Uint) { _ = i.Sub(sdkmath.NewUint(rand.Uint64() % i.Uint64())) },
		func(i *sdkmath.Uint) { _ = i.Mul(randuint()) },
		func(i *sdkmath.Uint) { _ = i.Quo(randuint()) },
		func(i *sdkmath.Uint) { _ = i.AddUint64(rand.Uint64()) },
		func(i *sdkmath.Uint) { _ = i.SubUint64(rand.Uint64() % i.Uint64()) },
		func(i *sdkmath.Uint) { _ = i.MulUint64(rand.Uint64()) },
		func(i *sdkmath.Uint) { _ = i.QuoUint64(rand.Uint64()) },
		func(i *sdkmath.Uint) { _ = i.IsZero() },
		func(i *sdkmath.Uint) { _ = i.Equal(randuint()) },
		func(i *sdkmath.Uint) { _ = i.GT(randuint()) },
		func(i *sdkmath.Uint) { _ = i.GTE(randuint()) },
		func(i *sdkmath.Uint) { _ = i.LT(randuint()) },
		func(i *sdkmath.Uint) { _ = i.LTE(randuint()) },
		func(i *sdkmath.Uint) { _ = i.String() },
		func(i *sdkmath.Uint) { _ = i.Incr() },
		func(i *sdkmath.Uint) {
			if i.IsZero() {
				return
			}

			_ = i.Decr()
		},
	}

	for i := 0; i < 1000; i++ {
		n := rand.Uint64()
		ni := sdkmath.NewUint(n)

		for opnum, op := range ops {
			op(&ni)

			s.Require().Equal(n, ni.Uint64(), "Uint is modified by operation. #%d", opnum)
			s.Require().Equal(sdkmath.NewUint(n), ni, "Uint is modified by operation. #%d", opnum)
		}
	}
}

func (s *uintTestSuite) TestSafeSub() {
	testCases := []struct {
		x, y     sdkmath.Uint
		expected uint64
		panic    bool
	}{
		{sdkmath.NewUint(0), sdkmath.NewUint(0), 0, false},
		{sdkmath.NewUint(10), sdkmath.NewUint(5), 5, false},
		{sdkmath.NewUint(5), sdkmath.NewUint(10), 5, true},
		{sdkmath.NewUint(math.MaxUint64), sdkmath.NewUint(0), math.MaxUint64, false},
	}

	for i, tc := range testCases {
		tc := tc
		if tc.panic {
			s.Require().Panics(func() { tc.x.Sub(tc.y) })
			continue
		}
		s.Require().Equal(
			tc.expected, tc.x.Sub(tc.y).Uint64(),
			"invalid subtraction result; x: %s, y: %s, tc: #%d", tc.x, tc.y, i,
		)
	}
}

func (s *uintTestSuite) TestParseUint() {
	type args struct {
		s string
	}
	tests := []struct {
		name    string
		args    args
		want    sdkmath.Uint
		wantErr bool
	}{
		{"malformed", args{"malformed"}, sdkmath.Uint{}, true},
		{"empty", args{""}, sdkmath.Uint{}, true},
		{"positive", args{"50"}, sdkmath.NewUint(uint64(50)), false},
		{"negative", args{"-1"}, sdkmath.Uint{}, true},
		{"zero", args{"0"}, sdkmath.ZeroUint(), false},
	}
	for _, tt := range tests {
		got, err := sdkmath.ParseUint(tt.args.s)
		if tt.wantErr {
			s.Require().Error(err)
			continue
		}
		s.Require().NoError(err)
		s.Require().True(got.Equal(tt.want))
	}
}

func (s *uintTestSuite) TestNewUintFromBigInt() {
	r := big.NewInt(42)
	i := sdkmath.NewUintFromBigInt(r)
	s.Require().Equal(r, i.BigInt())

	// modify r and ensure i doesn't change
	r = r.SetInt64(100)
	s.Require().NotEqual(r, i.BigInt())
}

func randuint() sdkmath.Uint {
	return sdkmath.NewUint(rand.Uint64())
}

func (s *uintTestSuite) TestRelativePow() {
	tests := []struct {
		args []sdkmath.Uint
		want sdkmath.Uint
	}{
		{[]sdkmath.Uint{sdkmath.ZeroUint(), sdkmath.ZeroUint(), sdkmath.OneUint()}, sdkmath.OneUint()},
		{[]sdkmath.Uint{sdkmath.ZeroUint(), sdkmath.ZeroUint(), sdkmath.NewUint(10)}, sdkmath.NewUint(1)},
		{[]sdkmath.Uint{sdkmath.ZeroUint(), sdkmath.OneUint(), sdkmath.NewUint(10)}, sdkmath.ZeroUint()},
		{[]sdkmath.Uint{sdkmath.NewUint(10), sdkmath.NewUint(2), sdkmath.OneUint()}, sdkmath.NewUint(100)},
		{[]sdkmath.Uint{sdkmath.NewUint(210), sdkmath.NewUint(2), sdkmath.NewUint(100)}, sdkmath.NewUint(441)},
		{[]sdkmath.Uint{sdkmath.NewUint(2100), sdkmath.NewUint(2), sdkmath.NewUint(1000)}, sdkmath.NewUint(4410)},
		{[]sdkmath.Uint{sdkmath.NewUint(1000000001547125958), sdkmath.NewUint(600), sdkmath.NewUint(1000000000000000000)}, sdkmath.NewUint(1000000928276004850)},
	}
	for i, tc := range tests {
		res := sdkmath.RelativePow(tc.args[0], tc.args[1], tc.args[2])
		s.Require().Equal(tc.want, res, "unexpected result for test case %d, input: %v, got: %v", i, tc.args, res)
	}
}

func minuint(i1, i2 uint64) uint64 {
	if i1 < i2 {
		return i1
	}
	return i2
}

func maxuint(i1, i2 uint64) uint64 {
	if i1 > i2 {
		return i1
	}
	return i2
}

func TestRoundTripMarshalToUint(t *testing.T) {
	values := []uint64{
		0,
		1,
		1 << 10,
		1<<10 - 3,
		1<<63 - 1,
		1<<32 - 7,
		1<<22 - 8,
		math.MaxUint64,
	}

	for _, value := range values {
		value := value
		t.Run(fmt.Sprintf("%d", value), func(t *testing.T) {
			t.Parallel()

			var scratch [20]byte
			uv := sdkmath.NewUint(value)
			n, err := uv.MarshalTo(scratch[:])
			if err != nil {
				t.Fatal(err)
			}
			rt := new(sdkmath.Uint)
			if err := rt.Unmarshal(scratch[:n]); err != nil {
				t.Fatal(err)
			}
			if !rt.Equal(uv) {
				t.Fatalf("roundtrip=%q != original=%q", rt, uv)
			}
		})
	}
}

func TestWeakUnmarshalNegativeSign(t *testing.T) {
	neg10, _ := new(big.Int).SetString("-10", 0)
	blob, err := neg10.MarshalText()
	if err != nil {
		t.Fatal(err)
	}

	ui := new(sdkmath.Uint)
	err = ui.Unmarshal(blob)
	if err == nil {
		t.Fatal("Failed to catch the negative value")
	}
	if errStr := err.Error(); !strings.Contains(errStr, "non-positive") {
		t.Fatalf("negative value not reported, got instead %q", errStr)
	}
}

func TestWeakUnmarshalOverflow(t *testing.T) {
	exp := new(big.Int).SetUint64(256)
	pos10, _ := new(big.Int).SetString("10", 0)
	exp10Pow256 := new(big.Int).Exp(pos10, exp, nil)
	blob, err := exp10Pow256.MarshalText()
	if err != nil {
		t.Fatal(err)
	}

	ui := new(sdkmath.Uint)
	err = ui.Unmarshal(blob)
	if err == nil {
		t.Fatal("Failed to catch the overflowed value")
	}
	if errStr := err.Error(); !strings.Contains(errStr, "out of range") {
		t.Fatalf("out of range value not reported, got instead %q", errStr)
	}
}
