// Package heuristic implements heuristic-based addition sequence algorithms
// with the Bos-Coster Makesequence structure.
package heuristic

import (
	"errors"
	"fmt"
	"math/big"
	"strings"

	"github.com/mmcloughlin/addchain"
	"github.com/mmcloughlin/addchain/internal/bigint"
	"github.com/mmcloughlin/addchain/internal/bigints"
)

// References:
//
//	[boscoster]                Bos, Jurjen and Coster, Matthijs. Addition Chain Heuristics. In Advances in
//	                           Cryptology --- CRYPTO' 89 Proceedings, pages 400--407. 1990.
//	                           https://link.springer.com/content/pdf/10.1007/0-387-34805-0_37.pdf
//	[github:kwantam/addchain]  Riad S. Wahby. kwantam/addchain. Github Repository. Apache License, Version 2.0.
//	                           2018. https://github.com/kwantam/addchain
//	[hehcc:exp]                Christophe Doche. Exponentiation. Handbook of Elliptic and Hyperelliptic Curve
//	                           Cryptography, chapter 9. 2006.
//	                           http://koclab.cs.ucsb.edu/teaching/ecc/eccPapers/Doche-ch09.pdf
//	[modboscoster]             Ayan Nandy. Modifications of Bos and Coster’s Heuristics in search of a
//	                           shorter addition chain for faster exponentiation. Masters thesis, Indian
//	                           Statistical Institute Kolkata. 2011.
//	                           http://library.isical.ac.in:8080/jspui/bitstream/10263/6441/1/DISS-285.pdf
//	[mpnt]                     F. L. Ţiplea, S. Iftene, C. Hriţcu, I. Goriac, R. Gordân and E. Erbiceanu.
//	                           MpNT: A Multi-Precision Number Theory Package, Number Theoretical Algorithms
//	                           (I). Technical Report TR03-02, Faculty of Computer Science, "Alexandru Ioan
//	                           Cuza" University, Iasi. 2003. https://profs.info.uaic.ro/~tr/tr03-02.pdf
//	[speedsubgroup]            Stam, Martijn. Speeding up subgroup cryptosystems. PhD thesis, Technische
//	                           Universiteit Eindhoven. 2003. https://cr.yp.to/bib/2003/stam-thesis.pdf

// Heuristic suggests insertions given a current protosequence.
type Heuristic interface {
	// Suggest insertions given a target and protosequence f. Protosequence must
	// contain sorted distinct integers.
	Suggest(f []*big.Int, target *big.Int) []*big.Int

	// String returns a name for the heuristic.
	String() string
}

// Algorithm searches for an addition sequence using a heuristic at each step.
// This implements the framework given in [mpnt], page 63, with the heuristic
// playing the role of the "newnumbers" function.
type Algorithm struct {
	heuristic Heuristic
}

// NewAlgorithm builds a heuristic algorithm.
func NewAlgorithm(h Heuristic) *Algorithm {
	return &Algorithm{
		heuristic: h,
	}
}

func (h Algorithm) String() string {
	return fmt.Sprintf("heuristic(%v)", h.heuristic)
}

// FindSequence searches for an addition sequence for the given targets.
func (h Algorithm) FindSequence(targets []*big.Int) (addchain.Chain, error) {
	// Skip the special case when targets is just {1}.
	if len(targets) == 1 && bigint.EqualInt64(targets[0], 1) {
		return targets, nil
	}

	// Initialize protosequence.
	leader := bigints.Int64s(1, 2)
	proto := append(leader, targets...)
	bigints.Sort(proto)
	proto = bigints.Unique(proto)
	c := []*big.Int{}

	for len(proto) > 2 {
		// Pop the target element.
		top := len(proto) - 1
		target := proto[top]
		proto = proto[:top]
		c = bigints.InsertSortedUnique(c, target)

		// Apply heuristic.
		insert := h.heuristic.Suggest(proto, target)
		if insert == nil {
			return nil, errors.New("failed to find sequence")
		}

		// Update protosequence.
		proto = bigints.MergeUnique(proto, insert)
	}

	// Prepare the chain to return.
	c = bigints.MergeUnique(leader, c)

	return addchain.Chain(c), nil
}

// DeltaLargest implements the simple heuristic of adding the delta between the
// largest two entries in the protosequence.
type DeltaLargest struct{}

func (DeltaLargest) String() string { return "delta_largest" }

// Suggest proposes inserting target-max(f).
func (DeltaLargest) Suggest(f []*big.Int, target *big.Int) []*big.Int {
	n := len(f)
	delta := new(big.Int).Sub(target, f[n-1])
	if delta.Sign() <= 0 {
		panic("delta must be positive")
	}
	return []*big.Int{delta}
}

// Approximation is the "Approximation" heuristic from [boscoster].
type Approximation struct{}

func (Approximation) String() string { return "approximation" }

// Suggest applies the "Approximation" heuristic. This heuristic looks for two
// elements a, b in the list that sum to something close to the target element
// f. That is, we look for f-(a+b) = epsilon where a ⩽ b and epsilon is a
// "small" positive value.
func (Approximation) Suggest(f []*big.Int, target *big.Int) []*big.Int {
	delta := new(big.Int)
	insert := new(big.Int)
	mindelta := new(big.Int)
	best := new(big.Int)
	first := true

	// Leverage the fact that f contains sorted distinct integers to apply a
	// linear algorithm, similar to the 2-SUM problem.  Maintain left and right
	// pointers and adjust them based on whether the sum is above or below the
	// target.
	for l, r := 0, len(f)-1; l <= r; {
		a, b := f[l], f[r]

		// Compute the delta f-(a+b).
		delta.Add(a, b)
		delta.Sub(target, delta)
		if delta.Sign() < 0 {
			// Sum exceeds target, decrement r for smaller b value.
			r--
			continue
		}

		// Proposed insertion is a+delta.
		insert.Add(a, delta)

		// If it's actually in the sequence already, use it.
		if bigints.ContainsSorted(insert, f) {
			return []*big.Int{insert}
		}

		// Keep it if its the closest we've seen.
		if first || delta.Cmp(mindelta) < 0 {
			mindelta.Set(delta)
			best.Set(insert)
			first = false
		}

		// Advance to next a value.
		l++
	}

	return []*big.Int{best}
}

// Halving is the "Halving" heuristic from [boscoster].
type Halving struct{}

func (Halving) String() string { return "halving" }

// Suggest applies when the target is at least twice as big as the next largest.
// If so it will return a sequence of doublings to insert. Otherwise it will
// return nil.
func (Halving) Suggest(f []*big.Int, target *big.Int) []*big.Int {
	n := len(f)
	max, next := target, f[n-1]

	// Check the condition f / f₁ ⩾ 2ᵘ
	r := new(big.Int).Div(max, next)
	if r.BitLen() < 2 {
		return nil
	}
	u := r.BitLen() - 1

	// Compute k = floor( f / 2ᵘ ).
	k := new(big.Int).Rsh(max, uint(u))

	// Proposal to insert:
	// Delta d = f - k*2ᵘ
	// Sequence k, 2*k, ..., k*2ᵘ
	kshifts := []*big.Int{}
	for e := 0; e <= u; e++ {
		kshift := new(big.Int).Lsh(k, uint(e))
		kshifts = append(kshifts, kshift)
	}
	d := new(big.Int).Sub(max, kshifts[u])
	if bigint.IsZero(d) {
		return kshifts[:u]
	}

	return bigints.InsertSortedUnique(kshifts, d)
}

// UseFirst builds a compositite heuristic that will make the first non-nil
// suggestion from the sub-heuristics.
func UseFirst(heuristics ...Heuristic) Heuristic {
	return useFirst(heuristics)
}

type useFirst []Heuristic

func (h useFirst) String() string {
	names := []string{}
	for _, sub := range h {
		names = append(names, sub.String())
	}
	return "use_first(" + strings.Join(names, ",") + ")"
}

// Suggest delegates to each sub-heuristic in turn and returns the first non-nil suggestion.
func (h useFirst) Suggest(f []*big.Int, target *big.Int) []*big.Int {
	for _, heuristic := range h {
		if insert := heuristic.Suggest(f, target); insert != nil {
			return insert
		}
	}
	return nil
}
