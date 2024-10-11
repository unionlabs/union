package prime

import "testing"

// Rudimentary tests to guard against transcription errors.

func TestDistinguishedArePrime(t *testing.T) {
	for _, p := range Distinguished {
		if !p.Int().ProbablyPrime(20) {
			t.Errorf("%s is not prime", p)
		}
	}
}

func TestDistinguishedDecimal(t *testing.T) {
	cases := []struct {
		P       Prime
		Decimal string
	}{
		{NISTP192, "6277101735386680763835789423207666416083908700390324961279"},
		{NISTP224, "26959946667150639794667015087019630673557916260026308143510066298881"},
		{NISTP256, "115792089210356248762697446949407573530086143415290314195533631308867097853951"},
		{NISTP384, "39402006196394479212279040100143613805079739270465446667948293404245721771496870329047266088258938001861606973112319"},
	}
	for _, c := range cases {
		got := c.P.Int().String()
		if got != c.Decimal {
			t.Fail()
		}
	}
}
