package emulated

import (
	"fmt"
	"math/big"
	"math/bits"

	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/multicommit"
)

// mulCheck represents a single multiplication check. Instead of doing a
// multiplication exactly where called, we compute the result using hint and
// return it. Additionally, we store the correctness check for later checking
// (together with every other multiplication) to share the verifier challenge
// computation.
//
// With this approach this is important that we do not change the [Element]
// values after they are returned from [mulMod] as mulCheck keeps pointers and
// the check will fail if the values refered to by the pointers change. By
// following the [Field] public methods this shouldn't happend as we always take
// and return pointers, and to change the values the user has to explicitly
// dereference.
//
// We store the values a, b, r, k, c. They are as follows:
//   - a, b - the inputs what we are multiplying. Do not have to be reduced.
//   - r - the multiplication result reduced modulo the emulation parameter.
//   - k - the quotient for integer multiplication a*b divided by emulation parameter.
//   - c - element representing carry. Used only for aligning the limb widths.
//
// Given these values, the following holds:
//
//	a * b = r * k*p
//
// But for asserting that the previous equation holds, we instead use the
// polynomial representation of the elements. If a non-native element a is given
// by its limbs
//
//	a = (a_0, ..., a_n)
//
// then
//
//	a(X) = \sum_i a_i * X^i.
//
// Now, the multiplication check instead becomes
//
//	a(X) * b(X) = r(X) + k(X) * p(X) + (2^t-X) c(X),
//
// which can be checked only at a single random point. Here we need an
// additional polynomial c(X) which is used for carrying the overflow bits to
// the consecutive limbs. By subtracting 2^t c(X) we can remove the bits from
// the corresponding coefficients in r(X)+k(X)*p(X) and by adding X c(X) we can
// add the bits to X(r(X) + k(X) * p(X)) (i.e. to the next coefficient).
type mulCheck[T FieldParams] struct {
	f *Field[T]
	// a * b = r + k*p + c
	a, b *Element[T] // inputs
	r    *Element[T] // reduced value
	k    *Element[T] // coefficient
	c    *Element[T] // carry
}

// evalRound1 evaluates first c(X), r(X) and k(X) at a given random point at[0].
// In the first round we do not assume that any of them is already evaluated as
// they come directly from hint.
func (mc *mulCheck[T]) evalRound1(api frontend.API, at []frontend.Variable) {
	mc.c = mc.f.evalWithChallenge(mc.c, at)
	mc.r = mc.f.evalWithChallenge(mc.r, at)
	mc.k = mc.f.evalWithChallenge(mc.k, at)
}

// evalRound2 now evaluates a and b at a given random point at[0]. However, it
// may happen that a or b is equal to r from a previous mulcheck. In that case
// we can reuse the evaluation to save constraints.
func (mc *mulCheck[T]) evalRound2(api frontend.API, at []frontend.Variable) {
	mc.a = mc.f.evalWithChallenge(mc.a, at)
	mc.b = mc.f.evalWithChallenge(mc.b, at)
}

// check checks a(ch) * b(ch) = r(ch) + k(ch) * p(ch) + (2^t - ch) c(ch). As the
// computation of p(ch) and (2^t-ch) can be shared over all mulCheck instances,
// then we get them already evaluated as peval and coef.
func (mc *mulCheck[T]) check(api frontend.API, peval, coef frontend.Variable) {
	ls := api.Mul(mc.a.evaluation, mc.b.evaluation)
	rs := api.Add(mc.r.evaluation, api.Mul(peval, mc.k.evaluation), api.Mul(mc.c.evaluation, coef))
	api.AssertIsEqual(ls, rs)
}

// cleanEvaluations cleans the cached evaluation values. This is necessary for
// ensuring the circuit stability over many compilations.
func (mc *mulCheck[T]) cleanEvaluations() {
	mc.a.evaluation = 0
	mc.a.isEvaluated = false
	mc.b.evaluation = 0
	mc.b.isEvaluated = false
	mc.r.evaluation = 0
	mc.r.isEvaluated = false
	mc.k.evaluation = 0
	mc.k.isEvaluated = false
	mc.c.evaluation = 0
	mc.c.isEvaluated = false
}

// mulMod returns a*b mod r. In practice it computes the result using a hint and
// defers the actual multiplication check.
func (f *Field[T]) mulMod(a, b *Element[T], _ uint) *Element[T] {
	f.enforceWidthConditional(a)
	f.enforceWidthConditional(b)
	k, r, c, err := f.callMulHint(a, b)
	if err != nil {
		panic(err)
	}
	mc := mulCheck[T]{
		f: f,
		a: a,
		b: b,
		c: c,
		k: k,
		r: r,
	}
	f.mulChecks = append(f.mulChecks, mc)
	return r
}

// evalWithChallenge represents element a as a polynomial a(X) and evaluates at
// at[0]. For efficiency, we use already evaluated powers of at[0] given by at.
// It stores the evaluation result inside the Element and marks it as evaluated.
// If the method is called for already evaluated a then returns the known value.
func (f *Field[T]) evalWithChallenge(a *Element[T], at []frontend.Variable) *Element[T] {
	if a.isEvaluated {
		return a
	}
	if len(at) < len(a.Limbs)-1 {
		panic("evaluation powers less than limbs")
	}
	sum := f.api.Mul(a.Limbs[0], 1) // copy because we use MulAcc
	for i := 1; i < len(a.Limbs); i++ {
		sum = f.api.MulAcc(sum, a.Limbs[i], at[i-1])
	}
	a.isEvaluated = true
	a.evaluation = sum
	return a
}

// performMulChecks should be deferred to actually perform all the
// multiplication checks.
func (f *Field[T]) performMulChecks(api frontend.API) error {
	// use given api. We are in defer and API may be different to what we have
	// stored.

	// there are no multiplication checks, nothing to do
	if len(f.mulChecks) == 0 {
		return nil
	}

	// we construct a list of elements we want to commit to. Even though we have
	// commited when doing range checks, do it again here explicitly for safety.
	// TODO: committing is actually expensive in PLONK. We create a constraint
	// for every variable we commit to (to set the selector polynomial). So, it
	// is actually better not to commit again. However, if we would be to use
	// multi-commit and range checks are in different commitment, then we have
	// problem.
	var toCommit []frontend.Variable
	for i := range f.mulChecks {
		toCommit = append(toCommit, f.mulChecks[i].a.Limbs...)
		toCommit = append(toCommit, f.mulChecks[i].b.Limbs...)
		toCommit = append(toCommit, f.mulChecks[i].r.Limbs...)
		toCommit = append(toCommit, f.mulChecks[i].k.Limbs...)
		toCommit = append(toCommit, f.mulChecks[i].c.Limbs...)
	}
	// we give all the inputs as inputs to obtain random verifier challenge.
	multicommit.WithCommitment(api, func(api frontend.API, commitment frontend.Variable) error {
		// for efficiency, we compute all powers of the challenge as slice at.
		coefsLen := 0
		for i := range f.mulChecks {
			coefsLen = max(coefsLen, len(f.mulChecks[i].c.Limbs))
		}
		at := make([]frontend.Variable, coefsLen)
		var prev frontend.Variable = 1
		for i := range at {
			at[i] = api.Mul(prev, commitment)
			prev = at[i]
		}
		// evaluate all r, k, c
		for i := range f.mulChecks {
			f.mulChecks[i].evalRound1(api, at)
		}
		// assuming r is input to some other multiplication, then is already evaluated
		for i := range f.mulChecks {
			f.mulChecks[i].evalRound2(api, at)
		}
		// evaluate p(X) at challenge
		pval := f.evalWithChallenge(f.Modulus(), at)
		// compute (2^t-X) at challenge
		coef := big.NewInt(1)
		coef.Lsh(coef, f.fParams.BitsPerLimb())
		ccoef := api.Sub(coef, commitment)
		// verify all mulchecks
		for i := range f.mulChecks {
			f.mulChecks[i].check(api, pval.evaluation, ccoef)
		}
		// clean cached evaluation. Helps in case we compile the same circuit
		// multiple times.
		for i := range f.mulChecks {
			f.mulChecks[i].cleanEvaluations()
		}
		return nil
	}, toCommit...)
	return nil
}

// callMulHint uses hint to compute r, k and c.
func (f *Field[T]) callMulHint(a, b *Element[T]) (quo, rem, carries *Element[T], err error) {
	// inputs is always nblimbs
	// quotient may be larger if inputs have overflow
	// remainder is always nblimbs
	// carries is 2 * nblimbs - 2 (do not consider first limb)
	nextOverflow, _ := f.mulPreCond(a, b)
	// skip error handle - it happens when we are supposed to reduce. But we
	// already check it as a precondition. We only need the overflow here.
	nbLimbs, nbBits := f.fParams.NbLimbs(), f.fParams.BitsPerLimb()
	nbQuoLimbs := ((2*nbLimbs-1)*nbBits + nextOverflow + 1 - //
		uint(f.fParams.Modulus().BitLen()) + //
		nbBits - 1) /
		nbBits
	nbRemLimbs := nbLimbs
	nbCarryLimbs := (nbQuoLimbs + nbLimbs) - 2
	hintInputs := []frontend.Variable{
		nbBits,
		nbLimbs,
	}
	hintInputs = append(hintInputs, f.Modulus().Limbs...)
	hintInputs = append(hintInputs, a.Limbs...)
	hintInputs = append(hintInputs, b.Limbs...)
	ret, err := f.api.NewHint(mulHint, int(nbQuoLimbs)+int(nbRemLimbs)+int(nbCarryLimbs), hintInputs...)
	if err != nil {
		err = fmt.Errorf("call hint: %w", err)
		return
	}
	quo = f.packLimbs(ret[:nbQuoLimbs], false)
	rem = f.packLimbs(ret[nbQuoLimbs:nbQuoLimbs+nbRemLimbs], true)
	carries = f.newInternalElement(ret[nbQuoLimbs+nbRemLimbs:], 0)
	return
}

func mulHint(field *big.Int, inputs, outputs []*big.Int) error {
	nbBits := int(inputs[0].Int64())
	nbLimbs := int(inputs[1].Int64())
	ptr := 2
	plimbs := inputs[ptr : ptr+nbLimbs]
	ptr += nbLimbs
	alimbs := inputs[ptr : ptr+nbLimbs]
	ptr += nbLimbs
	blimbs := inputs[ptr : ptr+nbLimbs]

	nbQuoLen := (len(outputs) - 2*nbLimbs + 2) / 2
	nbCarryLen := nbLimbs + nbQuoLen - 2
	outptr := 0
	quoLimbs := outputs[outptr : outptr+nbQuoLen]
	outptr += nbQuoLen
	remLimbs := outputs[outptr : outptr+nbLimbs]
	outptr += nbLimbs
	carryLimbs := outputs[outptr : outptr+nbCarryLen]

	p := new(big.Int)
	a := new(big.Int)
	b := new(big.Int)
	if err := recompose(plimbs, uint(nbBits), p); err != nil {
		return fmt.Errorf("recompose p: %w", err)
	}
	if err := recompose(alimbs, uint(nbBits), a); err != nil {
		return fmt.Errorf("recompose a: %w", err)
	}
	if err := recompose(blimbs, uint(nbBits), b); err != nil {
		return fmt.Errorf("recompose b: %w", err)
	}
	quo := new(big.Int)
	rem := new(big.Int)
	ab := new(big.Int).Mul(a, b)
	quo.QuoRem(ab, p, rem)
	if err := decompose(quo, uint(nbBits), quoLimbs); err != nil {
		return fmt.Errorf("decompose quo: %w", err)
	}
	if err := decompose(rem, uint(nbBits), remLimbs); err != nil {
		return fmt.Errorf("decompose rem: %w", err)
	}
	xp := make([]*big.Int, nbLimbs+nbQuoLen-1)
	yp := make([]*big.Int, nbLimbs+nbQuoLen-1)
	for i := range xp {
		xp[i] = new(big.Int)
	}
	for i := range yp {
		yp[i] = new(big.Int)
	}
	tmp := new(big.Int)
	for i := 0; i < nbLimbs; i++ {
		for j := 0; j < nbLimbs; j++ {
			tmp.Mul(alimbs[i], blimbs[j])
			xp[i+j].Add(xp[i+j], tmp)
		}
		yp[i].Add(yp[i], remLimbs[i])
		for j := 0; j < nbQuoLen; j++ {
			tmp.Mul(quoLimbs[j], plimbs[i])
			yp[i+j].Add(yp[i+j], tmp)
		}
	}
	carry := new(big.Int)
	for i := range carryLimbs {
		carry.Add(carry, xp[i])
		carry.Sub(carry, yp[i])
		carry.Rsh(carry, uint(nbBits))
		carryLimbs[i] = new(big.Int).Set(carry)
	}
	return nil
}

// Mul computes a*b and reduces it modulo the field order. The returned Element
// has default number of limbs and zero overflow. If the result wouldn't fit
// into Element, then locally reduces the inputs first. Doesn't mutate inputs.
//
// For multiplying by a constant, use [Field[T].MulConst] method which is more
// efficient.
func (f *Field[T]) Mul(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.mulMod, f.mulPreCond, a, b)
}

// MulMod computes a*b and reduces it modulo the field order. The returned Element
// has default number of limbs and zero overflow.
//
// Equivalent to [Field[T].Mul], kept for backwards compatibility.
func (f *Field[T]) MulMod(a, b *Element[T]) *Element[T] {
	return f.reduceAndOp(f.mulMod, f.mulPreCond, a, b)
}

// MulConst multiplies a by a constant c and returns it. We assume that the
// input constant is "small", so that we can compute the product by multiplying
// all individual limbs with the constant. If it is not small, then use the
// general [Field[T].Mul] or [Field[T].MulMod] with creating new Element from
// the constant on-the-fly.
func (f *Field[T]) MulConst(a *Element[T], c *big.Int) *Element[T] {
	switch c.Sign() {
	case -1:
		f.MulConst(f.Neg(a), new(big.Int).Neg(c))
	case 0:
		return f.Zero()
	}
	cbl := uint(c.BitLen())
	if cbl > f.maxOverflow() {
		panic(fmt.Sprintf("constant bit length %d exceeds max %d", cbl, f.maxOverflow()))
	}
	return f.reduceAndOp(
		func(a, _ *Element[T], u uint) *Element[T] {
			if ba, aConst := f.constantValue(a); aConst {
				ba.Mul(ba, c)
				return newConstElement[T](ba)
			}
			limbs := make([]frontend.Variable, len(a.Limbs))
			for i := range a.Limbs {
				limbs[i] = f.api.Mul(a.Limbs[i], c)
			}
			return f.newInternalElement(limbs, a.overflow+cbl)
		},
		func(a, _ *Element[T]) (nextOverflow uint, err error) {
			nextOverflow = a.overflow + uint(cbl)
			if nextOverflow > f.maxOverflow() {
				err = overflowError{op: "mulConst", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow()}
			}
			return
		},
		a, nil,
	)
}

func (f *Field[T]) mulPreCond(a, b *Element[T]) (nextOverflow uint, err error) {
	reduceRight := a.overflow < b.overflow
	nbResLimbs := nbMultiplicationResLimbs(len(a.Limbs), len(b.Limbs))
	nbLimbsOverflow := uint(1)
	if nbResLimbs > 0 {
		nbLimbsOverflow = uint(bits.Len(uint(2*nbResLimbs - 1)))
	}
	nextOverflow = f.fParams.BitsPerLimb() + nbLimbsOverflow + a.overflow + b.overflow
	if nextOverflow > f.maxOverflow() {
		err = overflowError{op: "mul", nextOverflow: nextOverflow, maxOverflow: f.maxOverflow(), reduceRight: reduceRight}
	}
	return
}

func (f *Field[T]) mul(a, b *Element[T], nextOverflow uint) *Element[T] {
	// TODO: kept for [AssertIsEqual]. Consider if this can be removed and we
	// can use MulMod for equality assertion.
	ba, aConst := f.constantValue(a)
	bb, bConst := f.constantValue(b)
	if aConst && bConst {
		ba.Mul(ba, bb).Mod(ba, f.fParams.Modulus())
		return newConstElement[T](ba)
	}

	// mulResult contains the result (out of circuit) of a * b school book multiplication
	// len(mulResult) == len(a) + len(b) - 1
	mulResult, err := f.computeMultiplicationHint(a.Limbs, b.Limbs)
	if err != nil {
		panic(fmt.Sprintf("multiplication hint: %s", err))
	}

	// we computed the result of the mul outside the circuit (mulResult)
	// and we want to constrain inside the circuit that this injected value
	// actually matches the in-circuit a * b values
	// create constraints (\sum_{i=0}^{m-1} a_i c^i) * (\sum_{i=0}^{m-1} b_i
	// c^i) = (\sum_{i=0}^{2m-2} z_i c^i) for c \in {1, 2m-1}
	w := new(big.Int)
	for c := 1; c <= len(mulResult); c++ {
		w.SetInt64(1) // c^i
		l := f.api.Mul(a.Limbs[0], 1)
		r := f.api.Mul(b.Limbs[0], 1)
		o := f.api.Mul(mulResult[0], 1)

		for i := 1; i < len(mulResult); i++ {
			w.Lsh(w, uint(c))
			if i < len(a.Limbs) {
				l = f.api.MulAcc(l, a.Limbs[i], w)
			}
			if i < len(b.Limbs) {
				r = f.api.MulAcc(r, b.Limbs[i], w)
			}
			o = f.api.MulAcc(o, mulResult[i], w)
		}
		f.api.AssertIsEqual(f.api.Mul(l, r), o)
	}
	return f.newInternalElement(mulResult, nextOverflow)
}
