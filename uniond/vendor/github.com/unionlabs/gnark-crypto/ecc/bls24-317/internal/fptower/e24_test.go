// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package fptower

import (
	"math/big"
	"testing"

	"github.com/consensys/gnark-crypto/ecc/bls24-317/fp"
	"github.com/leanovate/gopter"
	"github.com/leanovate/gopter/prop"
)

// ------------------------------------------------------------
// tests

func TestE24Serialization(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE24()

	properties.Property("[BLS24-317] SetBytes(Bytes()) should stay constant", prop.ForAll(
		func(a *E24) bool {
			var b E24
			buf := a.Bytes()
			if err := b.SetBytes(buf[:]); err != nil {
				return false
			}
			return a.Equal(&b)
		},
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}

func TestE24ReceiverIsOperand(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE24()
	genB := GenE24()

	properties.Property("[BLS24-317] Having the receiver as operand (addition) should output the same result", prop.ForAll(
		func(a, b *E24) bool {
			var c, d E24
			d.Set(a)
			c.Add(a, b)
			a.Add(a, b)
			b.Add(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (sub) should output the same result", prop.ForAll(
		func(a, b *E24) bool {
			var c, d E24
			d.Set(a)
			c.Sub(a, b)
			a.Sub(a, b)
			b.Sub(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (mul) should output the same result", prop.ForAll(
		func(a, b *E24) bool {
			var c, d E24
			d.Set(a)
			c.Mul(a, b)
			a.Mul(a, b)
			b.Mul(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (square) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Square(a)
			a.Square(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (double) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Double(a)
			a.Double(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (Inverse) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Inverse(a)
			a.Inverse(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (Cyclotomic square) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.CyclotomicSquare(a)
			a.CyclotomicSquare(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (Conjugate) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Conjugate(a)
			a.Conjugate(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (Frobenius) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Frobenius(a)
			a.Frobenius(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (FrobeniusSquare) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.FrobeniusSquare(a)
			a.FrobeniusSquare(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] Having the receiver as operand (FrobeniusQuad) should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.FrobeniusQuad(a)
			a.FrobeniusQuad(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}

func TestE24Ops(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE24()
	genB := GenE24()
	genExp := GenFp()

	properties.Property("[BLS24-317] sub & add should leave an element invariant", prop.ForAll(
		func(a, b *E24) bool {
			var c E24
			c.Set(a)
			c.Add(&c, b).Sub(&c, b)
			return c.Equal(a)
		},
		genA,
		genB,
	))

	properties.Property("[BLS24-317] mul & inverse should leave an element invariant", prop.ForAll(
		func(a, b *E24) bool {
			var c, d E24
			d.Inverse(b)
			c.Set(a)
			c.Mul(&c, b).Mul(&c, &d)
			return c.Equal(a)
		},
		genA,
		genB,
	))

	properties.Property("[BLS24-317] inverse twice should leave an element invariant", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Inverse(a).Inverse(&b)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] square and mul should output the same result", prop.ForAll(
		func(a *E24) bool {
			var b, c E24
			b.Mul(a, a)
			c.Square(a)
			return b.Equal(&c)
		},
		genA,
	))

	properties.Property("[BLS24-317] a + pi(a), a-pi(a) should be real", prop.ForAll(
		func(a *E24) bool {
			var b, c, d E24
			var e, f, g E12
			b.Conjugate(a)
			c.Add(a, &b)
			d.Sub(a, &b)
			e.Double(&a.D0)
			f.Double(&a.D1)
			return c.D1.Equal(&g) && d.D0.Equal(&g) && e.Equal(&c.D0) && f.Equal(&d.D1)
		},
		genA,
	))

	properties.Property("[BLS24-315] Torus-based Compress/decompress E24 elements in the cyclotomic subgroup", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.FrobeniusQuad(&b).Mul(a, &b)

			c, _ := a.CompressTorus()
			d := c.DecompressTorus()
			return a.Equal(&d)
		},
		genA,
	))

	properties.Property("[BLS24-315] Torus-based batch Compress/decompress E24 elements in the cyclotomic subgroup", prop.ForAll(
		func(a, e, f *E24) bool {
			var b E24
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.FrobeniusQuad(&b).Mul(a, &b)

			e.CyclotomicSquare(a)
			f.CyclotomicSquare(e)

			c, _ := BatchCompressTorus([]E24{*a, *e, *f})
			d, _ := BatchDecompressTorus(c)
			return a.Equal(&d[0]) && e.Equal(&d[1]) && f.Equal(&d[2])
		},
		genA,
		genA,
		genA,
	))

	properties.Property("[BLS24-317] pi**24=id", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.Frobenius(a).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b).
				Frobenius(&b)
			return b.Equal(a)
		},
		genA,
	))

	properties.Property("[BLS24-317] (pi**2)**12=id", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.FrobeniusSquare(a).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b).
				FrobeniusSquare(&b)

			return b.Equal(a)
		},
		genA,
	))

	properties.Property("[BLS24-317] (pi**4)**6=id", prop.ForAll(
		func(a *E24) bool {
			var b E24
			b.FrobeniusQuad(a).
				FrobeniusQuad(&b).
				FrobeniusQuad(&b).
				FrobeniusQuad(&b).
				FrobeniusQuad(&b).
				FrobeniusQuad(&b)

			return b.Equal(a)
		},
		genA,
	))

	properties.Property("[BLS24-317] cyclotomic square (Granger-Scott) and square should be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E24) bool {
			var b, c, d E24
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.FrobeniusQuad(&b).Mul(a, &b)
			c.Square(a)
			d.CyclotomicSquare(a)
			return c.Equal(&d)
		},
		genA,
	))

	properties.Property("[BLS24-317] compressed cyclotomic square (Karabina) and square should be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E24) bool {
			var _a, b, c, d, _c, _d E24
			_a.SetOne().Double(&_a)

			// put a and _a in the cyclotomic subgroup
			// a (g3 != 0 probably)
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.FrobeniusQuad(&b).Mul(a, &b)
			// _a (g3 == 0)
			b.Conjugate(&_a)
			_a.Inverse(&_a)
			b.Mul(&b, &_a)
			_a.FrobeniusQuad(&b).Mul(&_a, &b)

			// case g3 != 0
			c.Square(a)
			d.CyclotomicSquareCompressed(a).DecompressKarabina(&d)

			// case g3 == 0
			_c.Square(&_a)
			_d.CyclotomicSquareCompressed(&_a).DecompressKarabina(&_d)

			return c.Equal(&d)
		},
		genA,
	))

	properties.Property("[BLS24-315] Exp and CyclotomicExp results must be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E24, e fp.Element) bool {
			var b, c, d E24
			// put in the cyclo subgroup
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.FrobeniusQuad(&b).Mul(a, &b)

			var _e big.Int
			k := new(big.Int).SetUint64(24)
			e.Exp(e, k)
			e.BigInt(&_e)

			c.Exp(*a, &_e)
			d.CyclotomicExp(*a, &_e)

			return c.Equal(&d)
		},
		genA,
		genExp,
	))

	properties.Property("[BLS24-317] Frobenius of x in E24 should be equal to x^q", prop.ForAll(
		func(a *E24) bool {
			var b, c E24
			q := fp.Modulus()
			b.Frobenius(a)
			c.Set(a)
			c.Exp(c, q)
			return c.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] FrobeniusSquare of x in E24 should be equal to x^(q^2)", prop.ForAll(
		func(a *E24) bool {
			var b, c E24
			q := fp.Modulus()
			b.FrobeniusSquare(a)
			c.Exp(*a, q).Exp(c, q)
			return c.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] FrobeniusQuad of x in E24 should be equal to x^(q^4)", prop.ForAll(
		func(a *E24) bool {
			var b, c E24
			q := fp.Modulus()
			b.FrobeniusQuad(a)
			c.Exp(*a, q).Exp(c, q).Exp(c, q).Exp(c, q)
			return c.Equal(&b)
		},
		genA,
	))

	properties.Property("[BLS24-317] MulBy014 and Mul by (c0, c1, 0, 0, c4, 0) should output the same result", prop.ForAll(
		func(a, b *E24) bool {
			var c E24
			b.D0.C2.SetZero()
			b.D1.C0.SetZero()
			b.D1.C2.SetZero()
			c.Mul(a, b)
			a.MulBy014(&b.D0.C0, &b.D0.C1, &b.D1.C1)
			return c.Equal(a)
		},
		genA,
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))

}

// ------------------------------------------------------------
// benches

func BenchmarkE24Add(b *testing.B) {
	var a, c E24
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Add(&a, &c)
	}
}

func BenchmarkE24Sub(b *testing.B) {
	var a, c E24
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Sub(&a, &c)
	}
}

func BenchmarkE24Mul(b *testing.B) {
	var a, c E24
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Mul(&a, &c)
	}
}

func BenchmarkE24Cyclosquare(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.CyclotomicSquare(&a)
	}
}

func BenchmarkE24Square(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Square(&a)
	}
}

func BenchmarkE24Inverse(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Inverse(&a)
	}
}

func BenchmarkE24Conjugate(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Conjugate(&a)
	}
}

func BenchmarkE24Frobenius(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Frobenius(&a)
	}
}

func BenchmarkE24FrobeniusSquare(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.FrobeniusSquare(&a)
	}
}

func BenchmarkE24FrobeniusQuad(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.FrobeniusQuad(&a)
	}
}

func BenchmarkE24Expt(b *testing.B) {
	var a E24
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Expt(&a)
	}
}
