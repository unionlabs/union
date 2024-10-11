package math

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"
)

func TestDec(t *testing.T) {
	// Property tests
	t.Run("TestNewDecFromInt64", rapid.MakeCheck(testDecInt64))

	// Properties about addition
	t.Run("TestAddLeftIdentity", rapid.MakeCheck(testAddLeftIdentity))
	t.Run("TestAddRightIdentity", rapid.MakeCheck(testAddRightIdentity))
	t.Run("TestAddCommutative", rapid.MakeCheck(testAddCommutative))
	t.Run("TestAddAssociative", rapid.MakeCheck(testAddAssociative))

	// Properties about subtraction
	t.Run("TestSubRightIdentity", rapid.MakeCheck(testSubRightIdentity))
	t.Run("TestSubZero", rapid.MakeCheck(testSubZero))

	// Properties combining operations
	t.Run("TestSubAdd", rapid.MakeCheck(testSubAdd))
	t.Run("TestAddSub", rapid.MakeCheck(testAddSub))

	// Properties about comparison and equality
	t.Run("TestCmpInverse", rapid.MakeCheck(testCmpInverse))
	t.Run("TestEqualCommutative", rapid.MakeCheck(testEqualCommutative))

	// Properties about tests on a single Dec
	t.Run("TestIsNegative", rapid.MakeCheck(testIsNegative))

	// Unit tests
	zero := Dec{}
	one := NewDecFromInt64(1)
	two := NewDecFromInt64(2)
	three := NewDecFromInt64(3)
	four := NewDecFromInt64(4)
	five := NewDecFromInt64(5)
	minusOne := NewDecFromInt64(-1)

	onePointOneFive, err := NewDecFromString("1.15")
	require.NoError(t, err)
	twoPointThreeFour, err := NewDecFromString("2.34")
	require.NoError(t, err)
	threePointFourNine, err := NewDecFromString("3.49")
	require.NoError(t, err)
	onePointFourNine, err := NewDecFromString("1.49")
	require.NoError(t, err)
	minusFivePointZero, err := NewDecFromString("-5.0")
	require.NoError(t, err)
	_, err = NewDecFromString("inf")
	require.Error(t, err)
	_, err = NewDecFromString("Infinite")
	require.Error(t, err)
	_, err = NewDecFromString("foo")
	require.Error(t, err)
	_, err = NewDecFromString("NaN")
	require.Error(t, err)

	res, err := two.Add(zero)
	require.NoError(t, err)
	require.True(t, res.Equal(two))

	res, err = five.Sub(two)
	require.NoError(t, err)
	require.True(t, res.Equal(three))

	res, err = onePointOneFive.Add(twoPointThreeFour)
	require.NoError(t, err)
	require.True(t, res.Equal(threePointFourNine))

	res, err = threePointFourNine.Sub(two)
	require.NoError(t, err)
	require.True(t, res.Equal(onePointFourNine))

	res, err = minusOne.Sub(four)
	require.NoError(t, err)
	require.True(t, res.Equal(minusFivePointZero))

	_, err = four.Quo(zero)
	require.Error(t, err)

	res, err = four.Quo(two)
	require.NoError(t, err)
	require.True(t, res.Equal(two))

	require.False(t, zero.IsNegative())
	require.False(t, one.IsNegative())
	require.True(t, minusOne.IsNegative())
}

var genDec *rapid.Generator[Dec] = rapid.Custom(func(t *rapid.T) Dec {
	f := rapid.Float64().Draw(t, "f")
	dec, err := NewDecFromString(fmt.Sprintf("%g", f))
	require.NoError(t, err)
	return dec
})

// A Dec value and the float used to create it
type floatAndDec struct {
	float float64
	dec   Dec
}

// Generate a Dec value along with the float used to create it
var genFloatAndDec *rapid.Generator[floatAndDec] = rapid.Custom(func(t *rapid.T) floatAndDec {
	f := rapid.Float64().Draw(t, "f")
	dec, err := NewDecFromString(fmt.Sprintf("%g", f))
	require.NoError(t, err)
	return floatAndDec{f, dec}
})

// Property: n == NewDecFromInt64(n).Int64()
func testDecInt64(t *rapid.T) {
	nIn := rapid.Int64().Draw(t, "n")
	nOut, err := NewDecFromInt64(nIn).Int64()

	require.NoError(t, err)
	require.Equal(t, nIn, nOut)
}

// Property: 0 + a == a
func testAddLeftIdentity(t *rapid.T) {
	a := genDec.Draw(t, "a")
	zero := NewDecFromInt64(0)

	b, err := zero.Add(a)
	require.NoError(t, err)

	require.True(t, a.Equal(b))
}

// Property: a + 0 == a
func testAddRightIdentity(t *rapid.T) {
	a := genDec.Draw(t, "a")
	zero := NewDecFromInt64(0)

	b, err := a.Add(zero)
	require.NoError(t, err)

	require.True(t, a.Equal(b))
}

// Property: a + b == b + a
func testAddCommutative(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")

	c, err := a.Add(b)
	require.NoError(t, err)

	d, err := b.Add(a)
	require.NoError(t, err)

	require.True(t, c.Equal(d))
}

// Property: (a + b) + c == a + (b + c)
func testAddAssociative(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")
	c := genDec.Draw(t, "c")

	// (a + b) + c
	d, err := a.Add(b)
	require.NoError(t, err)

	e, err := d.Add(c)
	require.NoError(t, err)

	// a + (b + c)
	f, err := b.Add(c)
	require.NoError(t, err)

	g, err := a.Add(f)
	require.NoError(t, err)

	require.True(t, e.Equal(g))
}

// Property: a - 0 == a
func testSubRightIdentity(t *rapid.T) {
	a := genDec.Draw(t, "a")
	zero := NewDecFromInt64(0)

	b, err := a.Sub(zero)
	require.NoError(t, err)

	require.True(t, a.Equal(b))
}

// Property: a - a == 0
func testSubZero(t *rapid.T) {
	a := genDec.Draw(t, "a")
	zero := NewDecFromInt64(0)

	b, err := a.Sub(a)
	require.NoError(t, err)

	require.True(t, b.Equal(zero))
}

// Property: (a - b) + b == a
func testSubAdd(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")

	c, err := a.Sub(b)
	require.NoError(t, err)

	d, err := c.Add(b)
	require.NoError(t, err)

	require.True(t, a.Equal(d))
}

// Property: (a + b) - b == a
func testAddSub(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")

	c, err := a.Add(b)
	require.NoError(t, err)

	d, err := c.Sub(b)
	require.NoError(t, err)

	require.True(t, a.Equal(d))
}

// Property: Cmp(a, b) == -Cmp(b, a)
func testCmpInverse(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")

	require.Equal(t, a.Cmp(b), -b.Cmp(a))
}

// Property: Equal(a, b) == Equal(b, a)
func testEqualCommutative(t *rapid.T) {
	a := genDec.Draw(t, "a")
	b := genDec.Draw(t, "b")

	require.Equal(t, a.Equal(b), b.Equal(a))
}

// Property: isNegative(f) == isNegative(NewDecFromString(f.String()))
func testIsNegative(t *rapid.T) {
	floatAndDec := genFloatAndDec.Draw(t, "floatAndDec")
	f, dec := floatAndDec.float, floatAndDec.dec

	require.Equal(t, f < 0, dec.IsNegative())
}
