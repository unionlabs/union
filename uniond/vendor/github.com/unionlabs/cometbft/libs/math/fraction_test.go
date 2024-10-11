package math

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestParseFraction(t *testing.T) {
	testCases := []struct {
		f   string
		exp Fraction
		err bool
	}{
		{
			f:   "2/3",
			exp: Fraction{2, 3},
			err: false,
		},
		{
			f:   "15/5",
			exp: Fraction{15, 5},
			err: false,
		},
		// test divide by zero error
		{
			f:   "2/0",
			exp: Fraction{},
			err: true,
		},
		// test negative
		{
			f:   "-1/2",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "1/-2",
			exp: Fraction{},
			err: true,
		},
		// test overflow
		{
			f:   "9223372036854775808/2",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "2/9223372036854775808",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "2/3/4",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "123",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "1a2/4",
			exp: Fraction{},
			err: true,
		},
		{
			f:   "1/3bc4",
			exp: Fraction{},
			err: true,
		},
	}

	for idx, tc := range testCases {
		output, err := ParseFraction(tc.f)
		if tc.err {
			require.Error(t, err, idx)
		} else {
			require.NoError(t, err, idx)
		}
		assert.Equal(t, tc.exp, output, idx)
	}
}
