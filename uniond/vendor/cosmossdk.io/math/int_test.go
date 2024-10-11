package math_test

import (
	"encoding/json"
	"fmt"
	"math/big"
	"math/rand"
	"os"
	"strconv"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/math"
)

type intTestSuite struct {
	suite.Suite
}

func TestIntTestSuite(t *testing.T) {
	suite.Run(t, new(intTestSuite))
}

func (s *intTestSuite) SetupSuite() {
	s.T().Parallel()
}

func (s *intTestSuite) TestFromInt64() {
	for n := 0; n < 20; n++ {
		r := rand.Int63()
		s.Require().Equal(r, math.NewInt(r).Int64())
	}
}

func (s *intTestSuite) TestFromUint64() {
	for n := 0; n < 20; n++ {
		r := rand.Uint64()
		s.Require().True(math.NewIntFromUint64(r).IsUint64())
		s.Require().Equal(r, math.NewIntFromUint64(r).Uint64())
	}
}

func (s *intTestSuite) TestNewIntFromBigInt() {
	i := math.NewIntFromBigInt(nil)
	s.Require().True(i.IsNil())

	r := big.NewInt(42)
	i = math.NewIntFromBigInt(r)
	s.Require().Equal(r, i.BigInt())

	// modify r and ensure i doesn't change
	r = r.SetInt64(100)
	s.Require().NotEqual(r, i.BigInt())
}

func (s *intTestSuite) TestNewIntFromBigIntMut() {
	im := math.NewIntFromBigIntMut(nil)
	s.Require().True(im.IsNil())

	r := big.NewInt(42)
	im = math.NewIntFromBigIntMut(r)
	s.Require().Equal(r, im.BigInt())

	// Compare value of NewIntFromBigInt and NewIntFromBigIntMut
	i := math.NewIntFromBigInt(r)
	s.Require().Equal(i, im)

	// modify r and ensure i doesn't change & im changes
	r = r.SetInt64(100)
	s.Require().NotEqual(r, i.BigInt())
	s.Require().Equal(r, im.BigInt())
}

func (s *intTestSuite) TestConvertToBigIntMutative() {
	r := big.NewInt(42)
	i := math.NewIntFromBigInt(r)

	// Compare value of BigInt & BigIntMut
	s.Require().Equal(i.BigInt(), i.BigIntMut())

	// Modify BigIntMut() pointer and ensure i.BigIntMut() & i.BigInt() change
	p := i.BigIntMut()
	p.SetInt64(50)
	s.Require().Equal(big.NewInt(50), i.BigIntMut())
	s.Require().Equal(big.NewInt(50), i.BigInt())

	// Modify big.Int() pointer and ensure i.BigIntMut() & i.BigInt() don't change
	p = i.BigInt()
	p.SetInt64(60)
	s.Require().NotEqual(big.NewInt(60), i.BigIntMut())
	s.Require().NotEqual(big.NewInt(60), i.BigInt())
}

func (s *intTestSuite) TestIntPanic() {
	// Max Int = 2^256-1 = 1.1579209e+77
	// Min Int = -(2^256-1) = -1.1579209e+77
	s.Require().NotPanics(func() { math.NewIntWithDecimal(4, 76) })
	i1 := math.NewIntWithDecimal(4, 76)
	s.Require().NotPanics(func() { math.NewIntWithDecimal(5, 76) })
	i2 := math.NewIntWithDecimal(5, 76)
	s.Require().NotPanics(func() { math.NewIntWithDecimal(6, 76) })
	i3 := math.NewIntWithDecimal(6, 76)

	s.Require().Panics(func() { math.NewIntWithDecimal(2, 77) })
	s.Require().Panics(func() { math.NewIntWithDecimal(9, 80) })

	// Overflow check
	s.Require().NotPanics(func() { i1.Add(i1) })
	s.Require().NotPanics(func() { i2.Add(i2) })
	s.Require().Panics(func() { i3.Add(i3) })
	_, err := i1.SafeAdd(i1)
	s.Require().Nil(err)
	_, err = i2.SafeAdd(i2)
	s.Require().Nil(err)
	_, err = i3.SafeAdd(i3)
	s.Require().Error(err)

	s.Require().NotPanics(func() { i1.Sub(i1.Neg()) })
	s.Require().NotPanics(func() { i2.Sub(i2.Neg()) })
	s.Require().Panics(func() { i3.Sub(i3.Neg()) })
	_, err = i1.SafeSub(i1.Neg())
	s.Require().Nil(err)
	_, err = i2.SafeSub(i2.Neg())
	s.Require().Nil(err)
	_, err = i3.SafeSub(i3.Neg())
	s.Require().Error(err)

	s.Require().Panics(func() { i1.Mul(i1) })
	s.Require().Panics(func() { i2.Mul(i2) })
	s.Require().Panics(func() { i3.Mul(i3) })
	_, err = i1.SafeMul(i1)
	s.Require().Error(err)
	_, err = i2.SafeMul(i2)
	s.Require().Error(err)
	_, err = i3.SafeMul(i3)
	s.Require().Error(err)

	s.Require().Panics(func() { i1.Neg().Mul(i1.Neg()) })
	s.Require().Panics(func() { i2.Neg().Mul(i2.Neg()) })
	s.Require().Panics(func() { i3.Neg().Mul(i3.Neg()) })
	_, err = i1.Neg().SafeMul(i1.Neg())
	s.Require().Error(err)
	_, err = i2.Neg().SafeMul(i2.Neg())
	s.Require().Error(err)
	_, err = i3.Neg().SafeMul(i3.Neg())
	s.Require().Error(err)

	// Underflow check
	i3n := i3.Neg()
	s.Require().NotPanics(func() { i3n.Sub(i1) })
	s.Require().NotPanics(func() { i3n.Sub(i2) })
	s.Require().Panics(func() { i3n.Sub(i3) })
	_, err = i3n.SafeSub(i3)
	s.Require().Error(err)

	s.Require().NotPanics(func() { i3n.Add(i1.Neg()) })
	s.Require().NotPanics(func() { i3n.Add(i2.Neg()) })
	s.Require().Panics(func() { i3n.Add(i3.Neg()) })
	_, err = i3n.SafeAdd(i3.Neg())
	s.Require().Error(err)

	s.Require().Panics(func() { i1.Mul(i1.Neg()) })
	s.Require().Panics(func() { i2.Mul(i2.Neg()) })
	s.Require().Panics(func() { i3.Mul(i3.Neg()) })
	_, err = i1.SafeMul(i1.Neg())
	s.Require().Error(err)
	_, err = i2.SafeMul(i2.Neg())
	s.Require().Error(err)
	_, err = i3.SafeMul(i3.Neg())
	s.Require().Error(err)

	// Bound check
	intmax := math.NewIntFromBigInt(new(big.Int).Sub(new(big.Int).Exp(big.NewInt(2), big.NewInt(256), nil), big.NewInt(1)))
	intmin := intmax.Neg()
	s.Require().NotPanics(func() { intmax.Add(math.ZeroInt()) })
	s.Require().NotPanics(func() { intmin.Sub(math.ZeroInt()) })
	s.Require().Panics(func() { intmax.Add(math.OneInt()) })
	s.Require().Panics(func() { intmin.Sub(math.OneInt()) })
	_, err = intmax.SafeAdd(math.OneInt())
	s.Require().Error(err)
	_, err = intmin.SafeSub(math.OneInt())
	s.Require().Error(err)

	s.Require().NotPanics(func() { math.NewIntFromBigInt(nil) })
	s.Require().True(math.NewIntFromBigInt(nil).IsNil())

	// Division-by-zero check
	s.Require().Panics(func() { i1.Quo(math.NewInt(0)) })
	_, err = i1.SafeQuo(math.NewInt(0))
	s.Require().Error(err)

	s.Require().NotPanics(func() { math.Int{}.BigInt() })
}

// Tests below uses randomness
// Since we are using *big.Int as underlying value
// and (U/)Int is immutable value(see TestImmutability(U/)Int)
// it is safe to use randomness in the tests
func (s *intTestSuite) TestIdentInt() {
	for d := 0; d < 1000; d++ {
		n := rand.Int63()
		i := math.NewInt(n)

		ifromstr, ok := math.NewIntFromString(strconv.FormatInt(n, 10))
		s.Require().True(ok)

		cases := []int64{
			i.Int64(),
			i.BigInt().Int64(),
			ifromstr.Int64(),
			math.NewIntFromBigInt(big.NewInt(n)).Int64(),
			math.NewIntWithDecimal(n, 0).Int64(),
		}

		for tcnum, tc := range cases {
			s.Require().Equal(n, tc, "Int is modified during conversion. tc #%d", tcnum)
		}
	}
}

func minint(i1, i2 int64) int64 {
	if i1 < i2 {
		return i1
	}
	return i2
}

func maxint(i1, i2 int64) int64 {
	if i1 > i2 {
		return i1
	}
	return i2
}

func (s *intTestSuite) TestArithInt() {
	for d := 0; d < 1000; d++ {
		n1 := int64(rand.Int31())
		i1 := math.NewInt(n1)
		n2 := int64(rand.Int31())
		i2 := math.NewInt(n2)

		cases := []struct {
			ires math.Int
			nres int64
		}{
			{i1.Add(i2), n1 + n2},
			{i1.Sub(i2), n1 - n2},
			{i1.Mul(i2), n1 * n2},
			{i1.Quo(i2), n1 / n2},
			{i1.AddRaw(n2), n1 + n2},
			{i1.SubRaw(n2), n1 - n2},
			{i1.MulRaw(n2), n1 * n2},
			{i1.QuoRaw(n2), n1 / n2},
			{math.MinInt(i1, i2), minint(n1, n2)},
			{math.MaxInt(i1, i2), maxint(n1, n2)},
			{i1.Neg(), -n1},
			{i1.Abs(), n1},
			{i1.Neg().Abs(), n1},
		}

		for tcnum, tc := range cases {
			s.Require().Equal(tc.nres, tc.ires.Int64(), "Int arithmetic operation does not match with int64 operation. tc #%d", tcnum)
		}
	}
}

func (s *intTestSuite) TestCompInt() {
	for d := 0; d < 1000; d++ {
		n1 := int64(rand.Int31())
		i1 := math.NewInt(n1)
		n2 := int64(rand.Int31())
		i2 := math.NewInt(n2)

		cases := []struct {
			ires bool
			nres bool
		}{
			{i1.Equal(i2), n1 == n2},
			{i1.GT(i2), n1 > n2},
			{i1.LT(i2), n1 < n2},
			{i1.LTE(i2), n1 <= n2},
		}

		for tcnum, tc := range cases {
			s.Require().Equal(tc.nres, tc.ires, "Int comparison operation does not match with int64 operation. tc #%d", tcnum)
		}
	}
}

func randint() math.Int {
	return math.NewInt(rand.Int63())
}

func (s *intTestSuite) TestImmutabilityAllInt() {
	ops := []func(*math.Int){
		func(i *math.Int) { _ = i.Add(randint()) },
		func(i *math.Int) { _ = i.Sub(randint()) },
		func(i *math.Int) { _ = i.Mul(randint()) },
		func(i *math.Int) { _ = i.Quo(randint()) },
		func(i *math.Int) { _ = i.AddRaw(rand.Int63()) },
		func(i *math.Int) { _ = i.SubRaw(rand.Int63()) },
		func(i *math.Int) { _ = i.MulRaw(rand.Int63()) },
		func(i *math.Int) { _ = i.QuoRaw(rand.Int63()) },
		func(i *math.Int) { _ = i.Neg() },
		func(i *math.Int) { _ = i.Abs() },
		func(i *math.Int) { _ = i.IsZero() },
		func(i *math.Int) { _ = i.Sign() },
		func(i *math.Int) { _ = i.Equal(randint()) },
		func(i *math.Int) { _ = i.GT(randint()) },
		func(i *math.Int) { _ = i.LT(randint()) },
		func(i *math.Int) { _ = i.String() },
	}

	for i := 0; i < 1000; i++ {
		n := rand.Int63()
		ni := math.NewInt(n)

		for opnum, op := range ops {
			op(&ni)

			s.Require().Equal(n, ni.Int64(), "Int is modified by operation. tc #%d", opnum)
			s.Require().Equal(math.NewInt(n), ni, "Int is modified by operation. tc #%d", opnum)
		}
	}
}

func (s *intTestSuite) TestEncodingTableInt() {
	var i math.Int

	cases := []struct {
		i      math.Int
		jsonBz []byte
		rawBz  []byte
	}{
		{
			math.NewInt(0),
			[]byte("\"0\""),
			[]byte{0x30},
		},
		{
			math.NewInt(100),
			[]byte("\"100\""),
			[]byte{0x31, 0x30, 0x30},
		},
		{
			math.NewInt(-100),
			[]byte("\"-100\""),
			[]byte{0x2d, 0x31, 0x30, 0x30},
		},
		{
			math.NewInt(51842),
			[]byte("\"51842\""),
			[]byte{0x35, 0x31, 0x38, 0x34, 0x32},
		},
		{
			math.NewInt(-51842),
			[]byte("\"-51842\""),
			[]byte{0x2d, 0x35, 0x31, 0x38, 0x34, 0x32},
		},
		{
			math.NewInt(19513368),
			[]byte("\"19513368\""),
			[]byte{0x31, 0x39, 0x35, 0x31, 0x33, 0x33, 0x36, 0x38},
		},
		{
			math.NewInt(-19513368),
			[]byte("\"-19513368\""),
			[]byte{0x2d, 0x31, 0x39, 0x35, 0x31, 0x33, 0x33, 0x36, 0x38},
		},
		{
			math.NewInt(999999999999),
			[]byte("\"999999999999\""),
			[]byte{0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39},
		},
		{
			math.NewInt(-999999999999),
			[]byte("\"-999999999999\""),
			[]byte{0x2d, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39},
		},
	}

	for tcnum, tc := range cases {
		bz, err := tc.i.MarshalJSON()
		s.Require().Nil(err, "Error marshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.jsonBz, bz, "Marshaled value is different from exported. tc #%d", tcnum)

		err = (&i).UnmarshalJSON(bz)
		s.Require().Nil(err, "Error unmarshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.i, i, "Unmarshaled value is different from exported. tc #%d", tcnum)

		bz, err = tc.i.Marshal()
		s.Require().Nil(err, "Error marshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.rawBz, bz, "Marshaled value is different from exported. tc #%d", tcnum)

		err = (&i).Unmarshal(bz)
		s.Require().Nil(err, "Error unmarshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.i, i, "Unmarshaled value is different from exported. tc #%d", tcnum)
	}
}

func (s *intTestSuite) TestEncodingTableUint() {
	var i math.Uint

	cases := []struct {
		i      math.Uint
		jsonBz []byte
		rawBz  []byte
	}{
		{
			math.NewUint(0),
			[]byte("\"0\""),
			[]byte{0x30},
		},
		{
			math.NewUint(100),
			[]byte("\"100\""),
			[]byte{0x31, 0x30, 0x30},
		},
		{
			math.NewUint(51842),
			[]byte("\"51842\""),
			[]byte{0x35, 0x31, 0x38, 0x34, 0x32},
		},
		{
			math.NewUint(19513368),
			[]byte("\"19513368\""),
			[]byte{0x31, 0x39, 0x35, 0x31, 0x33, 0x33, 0x36, 0x38},
		},
		{
			math.NewUint(999999999999),
			[]byte("\"999999999999\""),
			[]byte{0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39, 0x39},
		},
	}

	for tcnum, tc := range cases {
		bz, err := tc.i.MarshalJSON()
		s.Require().Nil(err, "Error marshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.jsonBz, bz, "Marshaled value is different from exported. tc #%d", tcnum)

		err = (&i).UnmarshalJSON(bz)
		s.Require().Nil(err, "Error unmarshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.i, i, "Unmarshaled value is different from exported. tc #%d", tcnum)

		bz, err = tc.i.Marshal()
		s.Require().Nil(err, "Error marshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.rawBz, bz, "Marshaled value is different from exported. tc #%d", tcnum)

		err = (&i).Unmarshal(bz)
		s.Require().Nil(err, "Error unmarshaling Int. tc #%d, err %s", tcnum, err)
		s.Require().Equal(tc.i, i, "Unmarshaled value is different from exported. tc #%d", tcnum)
	}
}

func (s *intTestSuite) TestIntMod() {
	tests := []struct {
		name      string
		x         int64
		y         int64
		ret       int64
		wantPanic bool
	}{
		{"3 % 10", 3, 10, 3, false},
		{"10 % 3", 10, 3, 1, false},
		{"4 % 2", 4, 2, 0, false},
		{"2 % 0", 2, 0, 0, true},
	}

	for _, tt := range tests {
		if tt.wantPanic {
			s.Require().Panics(func() { math.NewInt(tt.x).Mod(math.NewInt(tt.y)) })
			s.Require().Panics(func() { math.NewInt(tt.x).ModRaw(tt.y) })
			return
		}
		s.Require().True(math.NewInt(tt.x).Mod(math.NewInt(tt.y)).Equal(math.NewInt(tt.ret)))
		s.Require().True(math.NewInt(tt.x).ModRaw(tt.y).Equal(math.NewInt(tt.ret)))
	}
}

func (s *intTestSuite) TestIntEq() {
	_, resp, _, _, _ := math.IntEq(s.T(), math.ZeroInt(), math.ZeroInt())
	s.Require().True(resp)
	_, resp, _, _, _ = math.IntEq(s.T(), math.OneInt(), math.ZeroInt())
	s.Require().False(resp)
}

func TestRoundTripMarshalToInt(t *testing.T) {
	values := []int64{
		0,
		1,
		1 << 10,
		1<<10 - 3,
		1<<63 - 1,
		1<<32 - 7,
		1<<22 - 8,
	}

	for _, value := range values {
		value := value
		t.Run(fmt.Sprintf("%d", value), func(t *testing.T) {
			t.Parallel()

			var scratch [20]byte
			iv := math.NewInt(value)
			n, err := iv.MarshalTo(scratch[:])
			if err != nil {
				t.Fatal(err)
			}
			rt := new(math.Int)
			if err := rt.Unmarshal(scratch[:n]); err != nil {
				t.Fatal(err)
			}
			if !rt.Equal(iv) {
				t.Fatalf("roundtrip=%q != original=%q", rt, iv)
			}
		})
	}
}

func TestFormatInt(t *testing.T) {
	type integerTest []string
	var testcases []integerTest
	raw, err := os.ReadFile("testdata/integers.json")
	require.NoError(t, err)
	err = json.Unmarshal(raw, &testcases)
	require.NoError(t, err)

	for _, tc := range testcases {
		out, err := math.FormatInt(tc[0])
		require.NoError(t, err)
		require.Equal(t, tc[1], out)
	}
}

func TestFormatIntNonDigits(t *testing.T) {
	badCases := []string{
		"a10",
		"1a10",
		"p1a10",
		"10p",
		"--10",
		"😎😎",
		"11111111111133333333333333333333333333333a",
		"11111111111133333333333333333333333333333 192892",
	}

	for _, value := range badCases {
		value := value
		t.Run(value, func(t *testing.T) {
			s, err := math.FormatInt(value)
			if err == nil {
				t.Fatal("Expected an error")
			}
			if g, w := err.Error(), "but got non-digits in"; !strings.Contains(g, w) {
				t.Errorf("Error mismatch\nGot:  %q\nWant substring: %q", g, w)
			}
			if s != "" {
				t.Fatalf("Got a non-empty string: %q", s)
			}
		})
	}
}

func TestFormatIntEmptyString(t *testing.T) {
	_, err := math.FormatInt("")
	require.ErrorContains(t, err, "cannot format empty string")
}

func TestFormatIntCorrectness(t *testing.T) {
	tests := []struct {
		in   string
		want string
	}{
		{"0", "0"},
		{"-2", "-2"},
		{"10", "10"},
		{"123", "123"},
		{"1234", "1'234"},
		{"12345", "12'345"},
		{"123456", "123'456"},
		{"-123456", "-123'456"},
		{"1234567", "1'234'567"},
		{"12345678", "12'345'678"},
		{"123456789", "123'456'789"},
		{"12345678910", "12'345'678'910"},
		{"9999999999999999", "9'999'999'999'999'999"},
		{"-9999999999999999", "-9'999'999'999'999'999"},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.in, func(t *testing.T) {
			got, err := math.FormatInt(tt.in)
			if err != nil {
				t.Fatal(err)
			}

			if got != tt.want {
				t.Fatalf("Mismatch:\n\tGot:  %q\n\tWant: %q", got, tt.want)
			}
		})
	}
}

var sizeTests = []struct {
	s    string
	want int
}{
	{"", 1},
	{"0", 1},
	{"-0", 1},
	{"-10", 3},
	{"-10000", 6},
	{"10000", 5},
	{"100000", 6},
	{"99999", 5},
	{"9999999999", 10},
	{"10000000000", 11},
	{"99999999999", 11},
	{"999999999999", 12},
	{"9999999999999", 13},
	{"99999999999999", 14},
	{"999999999999999", 15},
	{"1000000000000000", 16},
	{"9999999999999999", 16},
	{"99999999999999999", 17},
	{"999999999999999999", 18},
	{"-999999999999999999", 19},
	{"9000000000000000000", 19},
	{"-9999999999999990000", 20},
	{"9999999999999990000", 19},
	{"9999999999999999000", 19},
	{"9999999999999999999", 19},
	{"-9999999999999999999", 20},
	{"18446744073709551616", 20},
	{"18446744073709551618", 20},
	{"184467440737095516181", 21},
	{"100000000000000000000000", 24},
	{"1000000000000000000000000000", 28},
	{"9000000000099999999999999999", 28},
	{"9999999999999999999999999999", 28},
	{"9903520314283042199192993792", 28},
	{"340282366920938463463374607431768211456", 39},
	{"3402823669209384634633746074317682114569999", 43},
	{"9999999999999999999999999999999999999999999", 43},
	{"99999999999999999999999999999999999999999999", 44},
	{"999999999999999999999999999999999999999999999", 45},
	{"90000000000999999999999999999000000000099999999999999999", 56},
	{"-90000000000999999999999999999000000000099999999999999999", 57},
	{"9000000000099999999999999999900000000009999999999999999990", 58},
	{"990000000009999999999999999990000000000999999999999999999999", 60},
	{"99000000000999999999999999999000000000099999999999999999999919", 62},
	{"90000000000999999990000000000000000000000000000000000000000000", 62},
	{"99999999999999999999999999990000000000000000000000000000000000", 62},
	{"11111111111111119999999999990000000000000000000000000000000000", 62},
	{"99000000000999999999999999999000000000099999999999999999999919", 62},
	{"10000000000000000000000000000000000000000000000000000000000000", 62},
	{"10000000000000000000000000000000000000000000000000000000000000000000000000000", 77},
	{"99999999999999999999999999999999999999999999999999999999999999999999999999999", 77},
	{"110000000000000000000000000000000000000000000000000000000000000000000000000009", 78},
}

func TestNewIntFromString(t *testing.T) {
	for _, st := range sizeTests {
		ii, _ := math.NewIntFromString(st.s)
		require.Equal(t, st.want, ii.Size(), "size mismatch for %q", st.s)
	}
}

func BenchmarkIntSize(b *testing.B) {
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for _, st := range sizeTests {
			ii, _ := math.NewIntFromString(st.s)
			got := ii.Size()
			if got != st.want {
				b.Errorf("%q:: got=%d, want=%d", st.s, got, st.want)
			}
			sink = got
		}
	}
	if sink == nil {
		b.Fatal("Benchmark did not run!")
	}
	sink = nil
}

func BenchmarkIntOverflowCheckTime(b *testing.B) {
	ints := []*big.Int{}

	for _, st := range sizeTests {
		ii, _ := math.NewIntFromString(st.s)
		ints = append(ints, ii.BigInt())
	}
	b.ResetTimer()

	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for j := range sizeTests {
			got := math.NewIntFromBigIntMut(ints[j])
			sink = got
		}
	}
	if sink == nil {
		b.Fatal("Benchmark did not run!")
	}
	sink = nil
}
