package fp25519

import "math/big"

// p is the field prime modulus.
var p, _ = new(big.Int).SetString("57896044618658097711785492504343953926634992332820282019728792003956564819949", 10)

// Elt is an element of the field modulo 2²⁵⁵-19.
type Elt struct{ n big.Int }

// SetInt sets z = x (mod p) and returns it.
func (z *Elt) SetInt(x *big.Int) *Elt {
	z.n.Set(x)
	return z.modp()
}

// Int returns z as a big integer.
func (z *Elt) Int() *big.Int {
	return new(big.Int).Set(&z.n)
}

// Mul computes z = x*y (mod p) and returns it.
func (z *Elt) Mul(x, y *Elt) *Elt {
	z.n.Mul(&x.n, &y.n)
	return z.modp()
}

// Sqr computes z = x² (mod p) and returns it.
func (z *Elt) Sqr(x *Elt) *Elt {
	return z.Mul(x, x)
}

// modp reduces z modulo p, ensuring it's in the range [0, p).
func (z *Elt) modp() *Elt {
	z.n.Mod(&z.n, p)
	return z
}
