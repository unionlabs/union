package fp25519

import (
	"crypto/rand"
	"math/big"
	"testing"
)

func RandElt(t *testing.T) *Elt {
	t.Helper()
	one := new(big.Int).SetInt64(1)
	max := new(big.Int).Sub(p, one)
	x, err := rand.Int(rand.Reader, max)
	if err != nil {
		t.Fatal(err)
	}
	x.Add(x, one)
	return new(Elt).SetInt(x)
}

func TestInv(t *testing.T) {
	const trials = 1 << 12
	for trial := 0; trial < trials; trial++ {
		x := RandElt(t)
		got := new(Elt).Inv(x)
		expect := new(big.Int).ModInverse(x.Int(), p)
		if got.Int().Cmp(expect) != 0 {
			t.FailNow()
		}
	}
}
