// Package results stores results of this library on popular cryptographic
// exponents for testing and documentation purposes.
package results

import (
	"fmt"
	"math/big"

	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/prime"
)

// References:
//
//	[curvechains]  Brian Smith. The Most Efficient Known Addition Chains for Field Element and
//	               Scalar Inversion for the Most Popular and Most Unpopular Elliptic Curves. 2017.
//	               https://briansmith.org/ecc-inversion-addition-chains-01 (accessed June 30, 2019)

// Integer of interest.
type Integer interface {
	Int() *big.Int
	String() string
}

type integer struct {
	x *big.Int
}

// Hex constructs an integer target from a hex string.
func Hex(s string) Integer { return integer{x: bigint.MustHex(s)} }

func (i integer) String() string { return fmt.Sprintf("%x", i.x) }
func (i integer) Int() *big.Int  { return i.x }

// Result represents the best result of this library on a particular input.
type Result struct {
	Name string
	Slug string
	N    Integer
	D    int64

	// Length is the length of the most efficient chain produced by this library.
	Length int

	// AlgorithmName is the name of the algorithm that found the most efficient
	// chain.
	AlgorithmName string

	// BestKnown is the length of the most efficient chain known, found by any
	// method. These are currently all from [curvechains], it's possible there
	// are better results elsewhere.
	BestKnown int
}

// Target computes the addition chain target value N-d.
func (r Result) Target() *big.Int {
	return new(big.Int).Sub(r.N.Int(), big.NewInt(r.D))
}

// Delta relative to best known.
func (r Result) Delta() int { return r.Length - r.BestKnown }

// Results on inversion exponents for various interesting or popular fields.
// These results set a baseline to prevent regressions, as well as to compare
// against the best hand-crafted chains [curvechains].
var Results = []Result{
	{
		Name:          "Curve25519 Field Inversion",
		Slug:          "curve25519_field",
		N:             prime.P25519,
		D:             2,
		Length:        266,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
		BestKnown:     265,
	},
	{
		Name:          "NIST P-256 Field Inversion",
		Slug:          "p256_field",
		N:             prime.NISTP256,
		D:             3,
		Length:        266,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
		BestKnown:     266,
	},
	{
		Name:          "NIST P-384 Field Inversion",
		Slug:          "p384_field",
		N:             prime.NISTP384,
		D:             3,
		Length:        397,
		AlgorithmName: "opt(runs(heuristic(use_first(halving,approximation))))",
		BestKnown:     396,
	},
	{
		Name:          "secp256k1 (Bitcoin) Field Inversion",
		Slug:          "secp256k1_field",
		N:             prime.Secp256k1,
		D:             3,
		Length:        269,
		AlgorithmName: "opt(runs(heuristic(use_first(halving,delta_largest))))",
		BestKnown:     269,
	},
	{
		Name:          "Curve25519 Scalar Inversion",
		Slug:          "curve25519_scalar",
		N:             Hex("1000000000000000000000000000000014def9dea2f79cd65812631a5cf5d3ed"),
		D:             2,
		Length:        283,
		AlgorithmName: "opt(dictionary(hybrid(4,0),continued_fractions(binary)))",
		BestKnown:     284,
	},
	{
		Name:          "NIST P-256 Scalar Inversion",
		Slug:          "p256_scalar",
		N:             Hex("ffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551"),
		D:             2,
		Length:        294,
		AlgorithmName: "opt(dictionary(hybrid(8,16),heuristic(use_first(halving,delta_largest))))",
		BestKnown:     292,
	},
	{
		Name:          "NIST P-384 Scalar Inversion",
		Slug:          "p384_scalar",
		N:             Hex("ffffffffffffffffffffffffffffffffffffffffffffffffc7634d81f4372ddf581a0db248b0a77aecec196accc52973"),
		D:             2,
		Length:        434,
		AlgorithmName: "opt(dictionary(hybrid(4,0),continued_fractions(dichotomic)))",
		BestKnown:     433,
	},
	{
		Name:          "secp256k1 (Bitcoin) Scalar Inversion",
		Slug:          "secp256k1_scalar",
		N:             Hex("fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141"),
		D:             2,
		Length:        293,
		AlgorithmName: "opt(dictionary(hybrid(4,0),continued_fractions(dichotomic)))",
		BestKnown:     290,
	},
	{
		Name:          "M-221 Field Inversion",
		Slug:          "p2213_field",
		N:             prime.P2213,
		D:             2,
		Length:        231,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
	},
	{
		Name:          "E-222 Field Inversion",
		Slug:          "p222117_field",
		N:             prime.P222117,
		D:             2,
		Length:        233,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
	},
	{
		Name:          "Curve1174 Field Inversion",
		Slug:          "p2519_field",
		N:             prime.P2519,
		D:             2,
		Length:        263,
		AlgorithmName: "opt(dictionary(hybrid(3,64),continued_fractions(dichotomic)))",
	},
	{
		Name:          "E-382 Field Inversion",
		Slug:          "p382105_field",
		AlgorithmName: "opt(dictionary(hybrid(5,0),continued_fractions(dichotomic)))",
		N:             prime.P382105,
		D:             2,
		Length:        395,
	},
	{
		Name:          "M-383/Curve383187 Field Inversion",
		Slug:          "p383187_field",
		N:             prime.P383187,
		D:             2,
		Length:        396,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
	},
	{
		Name:          "Curve41417 Field Inversion",
		Slug:          "p41417_field",
		N:             prime.P41417,
		D:             2,
		Length:        426,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
	},
	{
		Name:          "M-511 Field Inversion",
		Slug:          "p511187_field",
		N:             prime.P511187,
		D:             2,
		Length:        525,
		AlgorithmName: "opt(runs(continued_fractions(dichotomic)))",
	},
	{
		Name:          "NIST P-192 Field Inversion",
		Slug:          "p192_field",
		N:             prime.NISTP192,
		D:             2,
		Length:        203,
		AlgorithmName: "opt(dictionary(hybrid(2,0),continued_fractions(dichotomic)))",
	},
	{
		Name:          "NIST P-224 Field Inversion",
		Slug:          "p224_field",
		N:             prime.NISTP224,
		D:             2,
		Length:        234,
		AlgorithmName: "opt(runs(heuristic(use_first(halving,approximation))))",
	},
	{
		Name:          "Goldilocks Field Inversion",
		Slug:          "goldilocks_field",
		N:             prime.Goldilocks,
		D:             2,
		Length:        460,
		AlgorithmName: "opt(runs(heuristic(use_first(halving,approximation))))",
	},
	{
		Name:          "secp192k1 Field Inversion",
		Slug:          "secp192k1_field",
		N:             prime.Secp192k1,
		D:             2,
		Length:        205,
		AlgorithmName: "opt(dictionary(hybrid(3,0),continued_fractions(dichotomic)))",
	},
	{
		Name:          "secp224k1 Field Inversion",
		Slug:          "secp224k1_field",
		N:             prime.Secp224k1,
		D:             2,
		Length:        238,
		AlgorithmName: "opt(dictionary(hybrid(5,0),continued_fractions(dichotomic)))",
	},
}
