package algtest

import (
	"math/big"
	"testing"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

// References:
//
//	[curvechains]       Brian Smith. The Most Efficient Known Addition Chains for Field Element and
//	                    Scalar Inversion for the Most Popular and Most Unpopular Elliptic Curves. 2017.
//	                    https://briansmith.org/ecc-inversion-addition-chains-01 (accessed June 30, 2019)
//	[hehcc:exp]         Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve
//	                    Cryptography, chapter 9. 2006.
//	                    http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf
//	[pairingsfinalexp]  Michael Scott, Naomi Benger, Manuel Charlemagne, Luis J. Dominguez Perez and
//	                    Ezekiel J. Kachisa. On the final exponentiation for calculating pairings on
//	                    ordinary elliptic curves. Cryptology ePrint Archive, Report 2008/490. 2008.
//	                    https://eprint.iacr.org/2008/490

// SequenceAlgorithmSuite builds a generic suite of tests for an addition
// sequence algorithm.
type SequenceAlgorithmSuite struct {
	// Algorithm under test.
	Algorithm alg.SequenceAlgorithm

	// AcceptsLargeInputs indicates whether the algorithm can tolerate large
	// inputs. As a rule of thumb, set this to true if the algorithm is
	// logarithmic in the largest input.
	AcceptsLargeInputs bool
}

// Tests builds the test suite function. Suitable to run as a subtest with
// testing.T.Run.
func (s SequenceAlgorithmSuite) Tests() func(t *testing.T) {
	return func(t *testing.T) {
		t.Run("known_sequences", checkKnownSequences(s.Algorithm))
		if s.AcceptsLargeInputs {
			t.Run("as_chain_algorithm", ChainAlgorithmSuite(alg.AsChainAlgorithm(s.Algorithm)))
		}
	}
}

func checkKnownSequences(a alg.SequenceAlgorithm) func(t *testing.T) {
	cases := []struct {
		Targets  []*big.Int
		Solution addchain.Chain
	}{
		// Example 9.39 in [hehcc:exp].
		{
			Targets:  bigints.Int64s(47, 117, 343, 499, 933, 5689),
			Solution: addchain.Int64s(1, 2, 4, 8, 10, 11, 18, 36, 47, 55, 91, 109, 117, 226, 343, 434, 489, 499, 933, 1422, 2844, 5688, 5689),
		},
		// [pairingsfinalexp] page 5.
		{
			Targets:  bigints.Int64s(6, 12, 18, 30, 36),
			Solution: addchain.Int64s(1, 2, 3, 6, 12, 18, 30, 36),
		},
		// [pairingsfinalexp] page 8.
		{
			Targets:  bigints.Int64s(4, 5, 7, 10, 15, 26, 28, 30, 55, 75, 80, 100, 108, 144),
			Solution: addchain.Int64s(1, 2, 4, 5, 7, 10, 15, 25, 26, 28, 30, 36, 50, 55, 75, 80, 100, 108, 144),
		},
		// [pairingsfinalexp] page 9.
		{
			Targets:  bigints.Int64s(3, 5, 7, 14, 15, 21, 25, 35, 49, 54, 62, 70, 87, 98, 112, 245, 273, 319, 343, 434, 450, 581, 609, 784, 931, 1407, 1911, 4802, 6517),
			Solution: addchain.Int64s(1, 2, 3, 4, 5, 7, 8, 14, 15, 16, 21, 25, 28, 35, 42, 49, 54, 62, 70, 87, 98, 112, 147, 245, 273, 294, 319, 343, 392, 434, 450, 581, 609, 784, 931, 1162, 1407, 1862, 1911, 3724, 4655, 4802, 6517),
		},
		// Curve25519 field inversion in [curvechains].
		{
			Targets:  bigints.Int64s(1, 2, 250),
			Solution: bigints.Int64s(1, 2, 3, 5, 10, 20, 40, 50, 100, 200, 250),
		},
		// P-256 field inversion in [curvechains].
		{
			Targets:  bigints.Int64s(32, 94),
			Solution: bigints.Int64s(1, 2, 3, 6, 12, 15, 30, 32, 64, 94),
		},
		// P-384 field inversion in [curvechains].
		{
			Targets:  bigints.Int64s(30, 32, 255),
			Solution: bigints.Int64s(1, 2, 3, 6, 12, 15, 30, 32, 60, 120, 240, 255),
		},
		// secp256k1 field inversion in [curvechains].
		{
			Targets:  bigints.Int64s(22, 223),
			Solution: bigints.Int64s(1, 2, 3, 6, 9, 11, 22, 44, 88, 176, 220, 223),
		},
	}

	for _, c := range cases {
		if err := c.Solution.Superset(c.Targets); err != nil {
			panic(err)
		}
	}

	return func(t *testing.T) {
		for _, c := range cases {
			got := AssertSequenceAlgorithmProduces(t, a, c.Targets)
			t.Logf("     got: length=%d sequence=%v", len(got), got)
			t.Logf("solution: length=%d sequence=%v", len(c.Solution), c.Solution)
			if len(got) > len(c.Solution) {
				t.Logf("sub-optimal")
			}
		}
	}
}
