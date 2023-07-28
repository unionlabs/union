// Package dict implements dictionary and run-length addition chain algorithms.
package dict

import (
	"errors"
	"fmt"
	"math/big"
	"sort"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/alg"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
	"github.com/mmcloughlin/addchain/internal/bigvector"
)

// References:
//
//	[braueraddsubchains]  Martin Otto. Brauer addition-subtraction chains. PhD thesis, Universitat
//	                      Paderborn. 2001.
//	                      http://www.martin-otto.de/publications/docs/2001_MartinOtto_Diplom_BrauerAddition-SubtractionChains.pdf
//	[genshortchains]      Kunihiro, Noboru and Yamamoto, Hirosuke. New Methods for Generating Short
//	                      Addition Chains. IEICE Transactions on Fundamentals of Electronics
//	                      Communications and Computer Sciences. 2000.
//	                      https://pdfs.semanticscholar.org/b398/d10faca35af9ce5a6026458b251fd0a5640c.pdf
//	[hehcc:exp]           Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve
//	                      Cryptography, chapter 9. 2006.
//	                      http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf

// Term represents the integer D * 2ᴱ.
type Term struct {
	D *big.Int
	E uint
}

// Int converts the term to an integer.
func (t Term) Int() *big.Int {
	return new(big.Int).Lsh(t.D, t.E)
}

// Sum is the representation of an integer as a sum of dictionary terms. See
// [hehcc:exp] definition 9.34.
type Sum []Term

// Int computes the dictionary sum as an integer.
func (s Sum) Int() *big.Int {
	x := bigint.Zero()
	for _, t := range s {
		x.Add(x, t.Int())
	}
	return x
}

// SortByExponent sorts terms in ascending order of the exponent E.
func (s Sum) SortByExponent() {
	sort.Slice(s, func(i, j int) bool { return s[i].E < s[j].E })
}

// Dictionary returns the distinct D values in the terms of this sum. The values
// are returned in ascending order.
func (s Sum) Dictionary() []*big.Int {
	dict := make([]*big.Int, 0, len(s))
	for _, t := range s {
		dict = append(dict, t.D)
	}
	bigints.Sort(dict)
	return bigints.Unique(dict)
}

// Decomposer is a method of breaking an integer into a dictionary sum.
type Decomposer interface {
	Decompose(x *big.Int) Sum
	String() string
}

// FixedWindow breaks integers into k-bit windows.
type FixedWindow struct {
	K uint // Window size.
}

func (w FixedWindow) String() string { return fmt.Sprintf("fixed_window(%d)", w.K) }

// Decompose represents x in terms of k-bit windows from left to right.
func (w FixedWindow) Decompose(x *big.Int) Sum {
	sum := Sum{}
	h := x.BitLen()
	for h > 0 {
		l := max(h-int(w.K), 0)
		d := bigint.Extract(x, uint(l), uint(h))
		if bigint.IsNonZero(d) {
			sum = append(sum, Term{D: d, E: uint(l)})
		}
		h = l
	}
	sum.SortByExponent()
	return sum
}

// SlidingWindow breaks integers into k-bit windows, skipping runs of zeros
// where possible. See [hehcc:exp] section 9.1.3 or [braueraddsubchains] section
// 1.2.3.
type SlidingWindow struct {
	K uint // Window size.
}

func (w SlidingWindow) String() string { return fmt.Sprintf("sliding_window(%d)", w.K) }

// Decompose represents x in base 2ᵏ.
func (w SlidingWindow) Decompose(x *big.Int) Sum {
	sum := Sum{}
	h := x.BitLen() - 1
	for h >= 0 {
		// Find first 1.
		for h >= 0 && x.Bit(h) == 0 {
			h--
		}

		if h < 0 {
			break
		}

		// Look down k positions.
		l := max(h-int(w.K)+1, 0)

		// Advance to the next 1.
		for x.Bit(l) == 0 {
			l++
		}

		sum = append(sum, Term{
			D: bigint.Extract(x, uint(l), uint(h+1)),
			E: uint(l),
		})

		h = l - 1
	}
	sum.SortByExponent()
	return sum
}

// RunLength decomposes integers in to runs of 1s up to a maximal length. See
// [genshortchains] Section 3.1.
type RunLength struct {
	T uint // Maximal run length. Zero means no limit.
}

func (r RunLength) String() string { return fmt.Sprintf("run_length(%d)", r.T) }

// Decompose breaks x into runs of 1 bits.
func (r RunLength) Decompose(x *big.Int) Sum {
	sum := Sum{}
	i := x.BitLen() - 1
	for i >= 0 {
		// Find first 1.
		for i >= 0 && x.Bit(i) == 0 {
			i--
		}

		if i < 0 {
			break
		}

		// Look for the end of the run.
		s := i
		for i >= 0 && x.Bit(i) == 1 && (r.T == 0 || uint(s-i) < r.T) {
			i--
		}

		// We have a run from s to i+1.
		sum = append(sum, Term{
			D: bigint.Ones(uint(s - i)),
			E: uint(i + 1),
		})
	}
	sum.SortByExponent()
	return sum
}

// Hybrid is a mix of the sliding window and run length decomposition methods,
// similar to the "Hybrid Method" of [genshortchains] Section 3.3.
type Hybrid struct {
	K uint // Window size.
	T uint // Maximal run length. Zero means no limit.
}

func (h Hybrid) String() string { return fmt.Sprintf("hybrid(%d,%d)", h.K, h.T) }

// Decompose breaks x into k-bit sliding windows or runs of 1s up to length T.
func (h Hybrid) Decompose(x *big.Int) Sum {
	sum := Sum{}

	// Clone since we'll be modifying it.
	y := bigint.Clone(x)

	// Process runs of length at least K.
	i := y.BitLen() - 1
	for i >= 0 {
		// Find first 1.
		for i >= 0 && y.Bit(i) == 0 {
			i--
		}

		if i < 0 {
			break
		}

		// Look for the end of the run.
		s := i
		for i >= 0 && y.Bit(i) == 1 && (h.T == 0 || uint(s-i) < h.T) {
			i--
		}

		// We have a run from s to i+1. Skip it if its short.
		n := uint(s - i)
		if n <= h.K {
			continue
		}

		// Add it to the sum and remove it from the integer.
		sum = append(sum, Term{
			D: bigint.Ones(n),
			E: uint(i + 1),
		})

		y.Xor(y, bigint.Mask(uint(i+1), uint(s+1)))
	}

	// Process what remains with a sliding window.
	w := SlidingWindow{K: h.K}
	rem := w.Decompose(y)

	sum = append(sum, rem...)
	sum.SortByExponent()

	return sum
}

// Algorithm implements a general dictionary-based chain construction algorithm,
// as in [braueraddsubchains] Algorithm 1.26. This operates in three stages:
// decompose the target into a sum of dictionray terms, use a sequence algorithm
// to generate the dictionary, then construct the target from the dictionary
// terms.
type Algorithm struct {
	decomp Decomposer
	seqalg alg.SequenceAlgorithm
}

// NewAlgorithm builds a dictionary algorithm that breaks up integers using the
// decomposer d and uses the sequence algorithm s to generate dictionary
// entries.
func NewAlgorithm(d Decomposer, a alg.SequenceAlgorithm) *Algorithm {
	return &Algorithm{
		decomp: d,
		seqalg: a,
	}
}

func (a Algorithm) String() string {
	return fmt.Sprintf("dictionary(%s,%s)", a.decomp, a.seqalg)
}

// FindChain builds an addition chain producing n. This works by using the
// configured Decomposer to represent n as a sum of dictionary terms, then
// delegating to the SequenceAlgorithm to build a chain producing the
// dictionary, and finally using the dictionary terms to construct n. See
// [genshortchains] Section 2 for a full description.
func (a Algorithm) FindChain(n *big.Int) (addchain.Chain, error) {
	// Decompose the target.
	sum := a.decomp.Decompose(n)
	sum.SortByExponent()

	// Extract dictionary.
	dict := sum.Dictionary()

	// Use the sequence algorithm to produce a chain for each element of the dictionary.
	c, err := a.seqalg.FindSequence(dict)
	if err != nil {
		return nil, err
	}

	// Reduce.
	sum, c, err = primitive(sum, c)
	if err != nil {
		return nil, err
	}

	// Build chain for n out of the dictionary.
	dc := dictsumchain(sum)
	c = append(c, dc...)
	bigints.Sort(c)
	c = addchain.Chain(bigints.Unique(c))

	return c, nil
}

// dictsumchain builds a chain for the integer represented by sum, assuming that
// all the terms of the sum are already present. Therefore this is intended to
// be appended to a chain that already contains the dictionary terms.
func dictsumchain(sum Sum) addchain.Chain {
	c := addchain.Chain{}
	k := len(sum) - 1
	cur := bigint.Clone(sum[k].D)
	for ; k > 0; k-- {
		// Shift until the next exponent.
		for i := sum[k].E; i > sum[k-1].E; i-- {
			cur.Lsh(cur, 1)
			c.AppendClone(cur)
		}

		// Add in the dictionary term at this position.
		cur.Add(cur, sum[k-1].D)
		c.AppendClone(cur)
	}

	for i := sum[0].E; i > 0; i-- {
		cur.Lsh(cur, 1)
		c.AppendClone(cur)
	}

	return c
}

// primitive removes terms from the dictionary that are only required once.
//
// The general structure of dictionary based algorithm is to decompose the
// target into a sum of dictionary terms, then create a chain for the
// dictionary, and then create the target from that. In a case where a
// dictionary term is only required once in the target, this can cause extra
// work. In such a case, we will spend operations on creating the dictionary
// term independently, and then later add it into the result. Since it is only
// needed once, we can effectively construct the dictionary term "on the fly" as
// we build up the final target.
//
// This function looks for such opportunities. If it finds them it will produce
// an alternative dictionary sum that replaces that term with a sum of smaller
// terms.
func primitive(sum Sum, c addchain.Chain) (Sum, addchain.Chain, error) {
	// This optimization cannot apply if the sum has only one term.
	if len(sum) == 1 {
		return sum, c, nil
	}

	n := len(c)

	// We'll need a mapping from chain elements to where they appear in the chain.
	idx := map[string]int{}
	for i, x := range c {
		idx[x.String()] = i
	}

	// Build program for the chain.
	p, err := c.Program()
	if err != nil {
		return nil, nil, err
	}

	// How many times is each index read during construction, and during its use in the dictionary chain.
	reads := p.ReadCounts()

	for _, t := range sum {
		i := idx[t.D.String()]
		reads[i]++
	}

	// Now, the primitive dictionary elements are those that are read at least twice, and their dependencies.
	deps := p.Dependencies()
	primitive := make([]bool, n)

	for i, numreads := range reads {
		if numreads < 2 {
			continue
		}
		primitive[i] = true
		for _, j := range bigint.BitsSet(deps[i]) {
			primitive[j] = true
		}
	}

	// Express every position in the chain as a linear combination of dictionary
	// terms that are used more than once.
	vc := []bigvector.Vector{bigvector.NewBasis(n, 0)}
	for i, op := range p {
		var next bigvector.Vector
		if primitive[i+1] {
			next = bigvector.NewBasis(n, i+1)
		} else {
			next = bigvector.Add(vc[op.I], vc[op.J])
		}
		vc = append(vc, next)
	}

	// Now express the target sum in terms that are used more than once.
	v := bigvector.New(n)
	for _, t := range sum {
		i := idx[t.D.String()]
		v = bigvector.Add(v, bigvector.Lsh(vc[i], t.E))
	}

	// Rebuild this into a dictionary sum.
	out := Sum{}
	for i := 0; i < v.Len(); i++ {
		for _, e := range bigint.BitsSet(v.Idx(i)) {
			out = append(out, Term{
				D: c[i],
				E: uint(e),
			})
		}
	}

	out.SortByExponent()

	// We should have not changed the sum.
	if !bigint.Equal(out.Int(), sum.Int()) {
		return nil, nil, errors.New("reconstruction does not match")
	}

	// Prune any elements of the chain that are used only once.
	pruned := addchain.Chain{}
	for i, x := range c {
		if primitive[i] {
			pruned = append(pruned, x)
		}
	}

	return out, pruned, nil
}

// max returns the maximum of a and b.
func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
