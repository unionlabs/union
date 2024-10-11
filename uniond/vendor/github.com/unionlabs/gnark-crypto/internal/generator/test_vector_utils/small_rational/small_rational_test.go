package small_rational

import (
	"github.com/stretchr/testify/assert"
	"math/big"
	"testing"
)

func TestBigDivides(t *testing.T) {
	assert.True(t, bigDivides(big.NewInt(-1), big.NewInt(4)))
	assert.False(t, bigDivides(big.NewInt(-3), big.NewInt(4)))
}

func TestCmp(t *testing.T) {

	cases := make([]SmallRational, 36)

	for i := int64(0); i < 9; i++ {
		if i%2 == 0 {
			cases[4*i].numerator.SetInt64((i - 4) / 2)
			cases[4*i].denominator.SetInt64(1)
		} else {
			cases[4*i].numerator.SetInt64(i - 4)
			cases[4*i].denominator.SetInt64(2)
		}

		cases[4*i+1].numerator.Neg(&cases[4*i].numerator)
		cases[4*i+1].denominator.Neg(&cases[4*i].denominator)

		cases[4*i+2].numerator.Lsh(&cases[4*i].numerator, 1)
		cases[4*i+2].denominator.Lsh(&cases[4*i].denominator, 1)

		cases[4*i+3].numerator.Neg(&cases[4*i+2].numerator)
		cases[4*i+3].denominator.Neg(&cases[4*i+2].denominator)
	}

	for i := range cases {
		for j := range cases {
			I, J := i/4, j/4
			var expectedCmp int
			cmp := cases[i].Cmp(&cases[j])
			if I < J {
				expectedCmp = -1
			} else if I == J {
				expectedCmp = 0
			} else {
				expectedCmp = 1
			}
			assert.Equal(t, expectedCmp, cmp, "comparing index %d, index %d", i, j)
		}
	}

	zeroIndex := len(cases) / 8
	var weirdZero SmallRational
	for i := range cases {
		I := i / 4
		var expectedCmp int
		cmp := cases[i].Cmp(&weirdZero)
		cmpNeg := weirdZero.Cmp(&cases[i])
		if I < zeroIndex {
			expectedCmp = -1
		} else if I == zeroIndex {
			expectedCmp = 0
		} else {
			expectedCmp = 1
		}

		assert.Equal(t, expectedCmp, cmp, "comparing index %d, 0/0", i)
		assert.Equal(t, -expectedCmp, cmpNeg, "comparing 0/0, index %d", i)
	}
}

func TestDouble(t *testing.T) {
	values := []interface{}{1, 2, 3, 4, 5, "2/3", "3/2", "-3/-2"}
	valsDoubled := []interface{}{2, 4, 6, 8, 10, "-4/-3", 3, 3}

	for i := range values {
		var v, vDoubled, vDoubledExpected SmallRational
		_, err := v.SetInterface(values[i])
		assert.NoError(t, err)
		_, err = vDoubledExpected.SetInterface(valsDoubled[i])
		assert.NoError(t, err)
		vDoubled.Double(&v)
		assert.True(t, vDoubled.Equal(&vDoubledExpected),
			"mismatch at %d: expected 2×%s = %s, saw %s", i, v.text, vDoubledExpected.text, vDoubled.text)

	}
}

func TestOperandConstancy(t *testing.T) {
	var p0, p, pPure SmallRational
	p0.SetInt64(1)
	p.SetInt64(-3)
	pPure.SetInt64(-3)

	res := p
	res.Add(&res, &p0)
	assert.True(t, p.Equal(&pPure))
}

func TestSquare(t *testing.T) {
	var two, four, x SmallRational
	two.SetInt64(2)
	four.SetInt64(4)

	x.Square(&two)

	assert.True(t, x.Equal(&four), "expected 4, saw %s", x.Text(10))
}

func TestSetBytes(t *testing.T) {
	var c SmallRational
	c.SetBytes([]byte("firstChallenge.0"))

}
