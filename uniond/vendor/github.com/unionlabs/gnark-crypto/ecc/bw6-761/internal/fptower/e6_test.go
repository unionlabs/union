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

	"github.com/consensys/gnark-crypto/ecc/bw6-761/fp"
	"github.com/leanovate/gopter"
	"github.com/leanovate/gopter/prop"
)

// ------------------------------------------------------------
// tests

func TestE6Serialization(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE6()

	properties.Property("[BW6-761] SetBytes(Bytes()) should stay constant", prop.ForAll(
		func(a *E6) bool {
			var b E6
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

func TestE6ReceiverIsOperand(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE6()
	genB := GenE6()

	properties.Property("[BW6-761] Having the receiver as operand (addition) should output the same result", prop.ForAll(
		func(a, b *E6) bool {
			var c, d E6
			d.Set(a)
			c.Add(a, b)
			a.Add(a, b)
			b.Add(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BW6-761] Having the receiver as operand (sub) should output the same result", prop.ForAll(
		func(a, b *E6) bool {
			var c, d E6
			d.Set(a)
			c.Sub(a, b)
			a.Sub(a, b)
			b.Sub(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BW6-761] Having the receiver as operand (mul) should output the same result", prop.ForAll(
		func(a, b *E6) bool {
			var c, d E6
			d.Set(a)
			c.Mul(a, b)
			a.Mul(a, b)
			b.Mul(&d, b)
			return a.Equal(b) && a.Equal(&c) && b.Equal(&c)
		},
		genA,
		genB,
	))

	properties.Property("[BW6-761] Having the receiver as operand (square) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Square(a)
			a.Square(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] Having the receiver as operand (double) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Double(a)
			a.Double(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] Having the receiver as operand (Inverse) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Inverse(a)
			a.Inverse(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] Having the receiver as operand (Cyclotomic square) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.CyclotomicSquare(a)
			a.CyclotomicSquare(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] Having the receiver as operand (Conjugate) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Conjugate(a)
			a.Conjugate(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] Having the receiver as operand (Frobenius) should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Frobenius(a)
			a.Frobenius(a)
			return a.Equal(&b)
		},
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))
}

func TestE6Ops(t *testing.T) {

	parameters := gopter.DefaultTestParameters()
	parameters.MinSuccessfulTests = 100

	properties := gopter.NewProperties(parameters)

	genA := GenE6()
	genB := GenE6()
	genExp := GenFp()

	properties.Property("[BW6-761] sub & add should leave an element invariant", prop.ForAll(
		func(a, b *E6) bool {
			var c E6
			c.Set(a)
			c.Add(&c, b).Sub(&c, b)
			return c.Equal(a)
		},
		genA,
		genB,
	))

	properties.Property("[BW6-761] mul & inverse should leave an element invariant", prop.ForAll(
		func(a, b *E6) bool {
			var c, d E6
			d.Inverse(b)
			c.Set(a)
			c.Mul(&c, b).Mul(&c, &d)
			return c.Equal(a)
		},
		genA,
		genB,
	))

	properties.Property("[BW6-761] inverse twice should leave an element invariant", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Inverse(a).Inverse(&b)
			return a.Equal(&b)
		},
		genA,
	))

	properties.Property("[BW6-761] square and mul should output the same result", prop.ForAll(
		func(a *E6) bool {
			var b, c E6
			b.Mul(a, a)
			c.Square(a)
			return b.Equal(&c)
		},
		genA,
	))

	properties.Property("[BW6-761] a + pi(a), a-pi(a) should be real", prop.ForAll(
		func(a *E6) bool {
			var b, c, d E6
			var e, f, g E3
			b.Conjugate(a)
			c.Add(a, &b)
			d.Sub(a, &b)
			e.Double(&a.B0)
			f.Double(&a.B1)
			return c.B1.Equal(&g) && d.B0.Equal(&g) && e.Equal(&c.B0) && f.Equal(&d.B1)
		},
		genA,
	))

	properties.Property("[BW6-761] Torus-based Compress/decompress E6 elements in the cyclotomic subgroup", prop.ForAll(
		func(a *E6) bool {
			var b E6
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.Frobenius(&b).Mul(a, &b)

			c, _ := a.CompressTorus()
			d := c.DecompressTorus()
			return a.Equal(&d)
		},
		genA,
	))

	properties.Property("[BW6-761] Torus-based batch Compress/decompress E6 elements in the cyclotomic subgroup", prop.ForAll(
		func(a, e, f *E6) bool {
			var b E6
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.Frobenius(&b).Mul(a, &b)

			e.CyclotomicSquare(a)
			f.CyclotomicSquare(e)

			c, _ := BatchCompressTorus([]E6{*a, *e, *f})
			d, _ := BatchDecompressTorus(c)
			return a.Equal(&d[0]) && e.Equal(&d[1]) && f.Equal(&d[2])
		},
		genA,
		genA,
		genA,
	))

	properties.Property("[BW6-761] pi**12=id", prop.ForAll(
		func(a *E6) bool {
			var b E6
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
				Frobenius(&b)
			return b.Equal(a)
		},
		genA,
	))

	properties.Property("[BW6-761] cyclotomic square (Granger-Scott) and square should be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E6) bool {
			var b, c, d E6
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.Frobenius(&b).Mul(a, &b)
			c.Square(a)
			d.CyclotomicSquare(a)
			return c.Equal(&d)
		},
		genA,
	))

	properties.Property("[BW6-761] compressed cyclotomic square (Karabina) and square should be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E6) bool {
			var _a, b, c, d, _c, _d E6
			_a.SetOne().Double(&_a)

			// put a and _a in the cyclotomic subgroup
			// a (g3 != 0 probably)
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.Frobenius(&b).Mul(a, &b)
			// _a (g3 == 0)
			b.Conjugate(&_a)
			_a.Inverse(&_a)
			b.Mul(&b, &_a)
			_a.Frobenius(&b).Mul(&_a, &b)

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

	properties.Property("[BW6-761] Exp and CyclotomicExp results must be the same in the cyclotomic subgroup", prop.ForAll(
		func(a *E6, e fp.Element) bool {
			var b, c, d E6
			// put in the cyclo subgroup
			b.Conjugate(a)
			a.Inverse(a)
			b.Mul(&b, a)
			a.Frobenius(&b).Mul(a, &b)

			var _e big.Int
			k := new(big.Int).SetUint64(6)
			e.Exp(e, k)
			e.BigInt(&_e)

			c.Exp(*a, &_e)
			d.CyclotomicExp(*a, &_e)

			return c.Equal(&d)
		},
		genA,
		genExp,
	))

	properties.Property("[BW6-761] Frobenius of x in E6 should be equal to x^q", prop.ForAll(
		func(a *E6) bool {
			var b, c E6
			q := fp.Modulus()
			b.Frobenius(a)
			c.Exp(*a, q)
			return c.Equal(&b)
		},
		genA,
	))

	properties.TestingRun(t, gopter.ConsoleReporter(false))

}

// ------------------------------------------------------------
// benches

func BenchmarkE6Add(b *testing.B) {
	var a, c E6
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Add(&a, &c)
	}
}

func BenchmarkE6Sub(b *testing.B) {
	var a, c E6
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Sub(&a, &c)
	}
}

func BenchmarkE6Mul(b *testing.B) {
	var a, c E6
	a.SetRandom()
	c.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Mul(&a, &c)
	}
}

func BenchmarkE6Cyclosquare(b *testing.B) {
	var a E6
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.CyclotomicSquare(&a)
	}
}

func BenchmarkE6Square(b *testing.B) {
	var a E6
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Square(&a)
	}
}

func BenchmarkE6Inverse(b *testing.B) {
	var a E6
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Inverse(&a)
	}
}

func BenchmarkE6Conjugate(b *testing.B) {
	var a E6
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Conjugate(&a)
	}
}

func BenchmarkE6Frobenius(b *testing.B) {
	var a E6
	a.SetRandom()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		a.Frobenius(&a)
	}
}

func BenchmarkE6Expt(b *testing.B) {
	var a, c E6
	a.SetRandom()
	b.ResetTimer()
	c.Conjugate(&a)
	a.Inverse(&a)
	c.Mul(&c, &a)

	a.Frobenius(&c).
		Mul(&a, &c)

	for i := 0; i < b.N; i++ {
		a.Expt(&a)
	}
}
