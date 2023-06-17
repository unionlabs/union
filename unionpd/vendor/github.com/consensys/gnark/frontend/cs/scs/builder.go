/*
Copyright © 2021 ConsenSys Software Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

package scs

import (
	"math/big"
	"reflect"
	"sort"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/internal/expr"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/internal/circuitdefer"
	"github.com/consensys/gnark/internal/kvstore"
	"github.com/consensys/gnark/internal/tinyfield"
	"github.com/consensys/gnark/internal/utils"
	"github.com/consensys/gnark/logger"

	bls12377r1cs "github.com/consensys/gnark/constraint/bls12-377"
	bls12381r1cs "github.com/consensys/gnark/constraint/bls12-381"
	bls24315r1cs "github.com/consensys/gnark/constraint/bls24-315"
	bls24317r1cs "github.com/consensys/gnark/constraint/bls24-317"
	bn254r1cs "github.com/consensys/gnark/constraint/bn254"
	bw6633r1cs "github.com/consensys/gnark/constraint/bw6-633"
	bw6761r1cs "github.com/consensys/gnark/constraint/bw6-761"
	"github.com/consensys/gnark/constraint/solver"
	tinyfieldr1cs "github.com/consensys/gnark/constraint/tinyfield"
)

func NewBuilder(field *big.Int, config frontend.CompileConfig) (frontend.Builder, error) {
	return newBuilder(field, config), nil
}

type builder struct {
	cs     constraint.SparseR1CS
	config frontend.CompileConfig
	kvstore.Store

	// map for recording boolean constrained variables (to not constrain them twice)
	mtBooleans map[expr.Term]struct{}

	// records multiplications constraint to avoid duplicate.
	// see mulConstraintExist(...)
	mMulConstraints map[uint64]int

	// same thing for addition gates
	// see addConstraintExist(...)
	mAddConstraints map[uint64]int

	// frequently used coefficients
	tOne, tMinusOne constraint.Coeff
}

// initialCapacity has quite some impact on frontend performance, especially on large circuits size
// we may want to add build tags to tune that
func newBuilder(field *big.Int, config frontend.CompileConfig) *builder {
	b := builder{
		mtBooleans:      make(map[expr.Term]struct{}),
		mMulConstraints: make(map[uint64]int, config.Capacity/2),
		mAddConstraints: make(map[uint64]int, config.Capacity/2),
		config:          config,
		Store:           kvstore.New(),
	}

	curve := utils.FieldToCurve(field)

	switch curve {
	case ecc.BLS12_377:
		b.cs = bls12377r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BLS12_381:
		b.cs = bls12381r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BN254:
		b.cs = bn254r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BW6_761:
		b.cs = bw6761r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BW6_633:
		b.cs = bw6633r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BLS24_315:
		b.cs = bls24315r1cs.NewSparseR1CS(config.Capacity)
	case ecc.BLS24_317:
		b.cs = bls24317r1cs.NewSparseR1CS(config.Capacity)
	default:
		if field.Cmp(tinyfield.Modulus()) == 0 {
			b.cs = tinyfieldr1cs.NewSparseR1CS(config.Capacity)
			break
		}
		panic("not implemented")
	}

	b.tOne = b.cs.One()
	b.tMinusOne = b.cs.FromInterface(-1)

	return &b
}

func (builder *builder) Field() *big.Int {
	return builder.cs.Field()
}

func (builder *builder) FieldBitLen() int {
	return builder.cs.FieldBitLen()
}

// TODO @gbotrel doing a 2-step refactoring for now, frontend only. need to update constraint/SparseR1C.
// qL⋅xa + qR⋅xb + qO⋅xc + qM⋅(xaxb) + qC == 0
type sparseR1C struct {
	xa, xb, xc         int              // wires
	qL, qR, qO, qM, qC constraint.Coeff // coefficients
	commitment         constraint.CommitmentConstraint
}

// a * b == c
func (builder *builder) addMulGate(a, b, c expr.Term, debug ...constraint.DebugInfo) {
	qO := builder.tMinusOne
	if c.Coeff != builder.tOne {
		// slow path
		t := c.Coeff
		builder.cs.Neg(&t)
		qO = t
	}
	qM := a.Coeff
	builder.cs.Mul(&qM, &b.Coeff)
	builder.addPlonkConstraint(sparseR1C{
		xa: a.VID,
		xb: b.VID,
		xc: c.VID,
		qM: qM,
		qO: qO,
	}, debug...)
}

// addPlonkConstraint adds a sparseR1C to the underlying constraint system
func (builder *builder) addPlonkConstraint(c sparseR1C, debug ...constraint.DebugInfo) {
	if !c.qM.IsZero() && (c.xa == 0 || c.xb == 0) {
		// TODO this is internal but not easy to detect; if qM is set, but one or both of xa / xb is not,
		// since wireID == 0 is a valid wire, it may trigger unexpected behavior.
		log := logger.Logger()
		log.Warn().Msg("adding a plonk constraint with qM set but xa or xb == 0 (wire 0)")
	}
	L := builder.cs.MakeTerm(&c.qL, c.xa)
	R := builder.cs.MakeTerm(&c.qR, c.xb)
	O := builder.cs.MakeTerm(&c.qO, c.xc)
	U := builder.cs.MakeTerm(&c.qM, c.xa)
	V := builder.cs.MakeTerm(&builder.tOne, c.xb)
	K := builder.cs.MakeTerm(&c.qC, 0)
	K.MarkConstant()
	builder.cs.AddConstraint(constraint.SparseR1C{L: L, R: R, O: O, M: [2]constraint.Term{U, V}, K: K.CoeffID(), Commitment: c.commitment}, debug...)
}

// newInternalVariable creates a new wire, appends it on the list of wires of the circuit, sets
// the wire's id to the number of wires, and returns it
func (builder *builder) newInternalVariable() expr.Term {
	idx := builder.cs.AddInternalVariable()
	return expr.NewTerm(idx, builder.tOne)
}

// PublicVariable creates a new Public Variable
func (builder *builder) PublicVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddPublicVariable(f.FullName())
	return expr.NewTerm(idx, builder.tOne)
}

// SecretVariable creates a new Secret Variable
func (builder *builder) SecretVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddSecretVariable(f.FullName())
	return expr.NewTerm(idx, builder.tOne)
}

// reduces redundancy in linear expression
// It factorizes Variable that appears multiple times with != coeff Ids
// To ensure the determinism in the compile process, Variables are stored as public∥secret∥internal∥unset
// for each visibility, the Variables are sorted from lowest ID to highest ID
func (builder *builder) reduce(l expr.LinearExpression) expr.LinearExpression {

	// ensure our linear expression is sorted, by visibility and by Variable ID
	sort.Sort(l)

	for i := 1; i < len(l); i++ {
		if l[i-1].VID == l[i].VID {
			// we have redundancy
			builder.cs.Add(&l[i-1].Coeff, &l[i].Coeff)
			l = append(l[:i], l[i+1:]...)
			i--
		}
	}
	return l
}

// IsBoolean returns true if given variable was marked as boolean in the compiler (see MarkBoolean)
// Use with care; variable may not have been **constrained** to be boolean
// This returns true if the v is a constant and v == 0 || v == 1.
func (builder *builder) IsBoolean(v frontend.Variable) bool {
	if b, ok := builder.constantValue(v); ok {
		return (b.IsZero() || builder.cs.IsOne(&b))
	}
	_, ok := builder.mtBooleans[v.(expr.Term)]
	return ok
}

// MarkBoolean sets (but do not constraint!) v to be boolean
// This is useful in scenarios where a variable is known to be boolean through a constraint
// that is not api.AssertIsBoolean. If v is a constant, this is a no-op.
func (builder *builder) MarkBoolean(v frontend.Variable) {
	if _, ok := builder.constantValue(v); ok {
		if !builder.IsBoolean(v) {
			panic("MarkBoolean called a non-boolean constant")
		}
		return
	}
	builder.mtBooleans[v.(expr.Term)] = struct{}{}
}

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
}

func (builder *builder) Compile() (constraint.ConstraintSystem, error) {
	log := logger.Logger()
	log.Info().
		Int("nbConstraints", builder.cs.GetNbConstraints()).
		Msg("building constraint builder")

	// ensure all inputs and hints are constrained
	err := builder.cs.CheckUnconstrainedWires()
	if err != nil {
		log.Warn().Msg("circuit has unconstrained inputs")
		if !builder.config.IgnoreUnconstrainedInputs {
			return nil, err
		}
	}

	return builder.cs, nil
}

// ConstantValue returns the big.Int value of v.
// Will panic if v.IsConstant() == false
func (builder *builder) ConstantValue(v frontend.Variable) (*big.Int, bool) {
	coeff, ok := builder.constantValue(v)
	if !ok {
		return nil, false
	}
	return builder.cs.ToBigInt(&coeff), true
}

func (builder *builder) constantValue(v frontend.Variable) (constraint.Coeff, bool) {
	if _, ok := v.(expr.Term); ok {
		return constraint.Coeff{}, false
	}
	return builder.cs.FromInterface(v), true
}

// NewHint initializes internal variables whose value will be evaluated using
// the provided hint function at run time from the inputs. Inputs must be either
// variables or convertible to *big.Int. The function returns an error if the
// number of inputs is not compatible with f.
//
// The hint function is provided at the proof creation time and is not embedded
// into the circuit. From the backend point of view, the variable returned by
// the hint function is equivalent to the user-supplied witness, but its actual
// value is assigned by the solver, not the caller.
//
// No new constraints are added to the newly created wire and must be added
// manually in the circuit. Failing to do so leads to solver failure.
func (builder *builder) NewHint(f solver.Hint, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	hintInputs := make([]constraint.LinearExpression, len(inputs))

	// ensure inputs are set and pack them in a []uint64
	for i, in := range inputs {
		switch t := in.(type) {
		case expr.Term:
			hintInputs[i] = constraint.LinearExpression{builder.cs.MakeTerm(&t.Coeff, t.VID)}
		default:
			c := builder.cs.FromInterface(in)
			term := builder.cs.MakeTerm(&c, 0)
			term.MarkConstant()
			hintInputs[i] = constraint.LinearExpression{term}
		}
	}

	internalVariables, err := builder.cs.AddSolverHint(f, hintInputs, nbOutputs)
	if err != nil {
		return nil, err
	}

	// make the variables
	res := make([]frontend.Variable, len(internalVariables))
	for i, idx := range internalVariables {
		res[i] = expr.NewTerm(idx, builder.tOne)
	}
	return res, nil

}

// returns in split into a slice of compiledTerm and the sum of all constants in in as a bigInt
func (builder *builder) filterConstantSum(in []frontend.Variable) (expr.LinearExpression, constraint.Coeff) {
	res := make(expr.LinearExpression, 0, len(in))
	b := constraint.Coeff{}
	for i := 0; i < len(in); i++ {
		if c, ok := builder.constantValue(in[i]); ok {
			builder.cs.Add(&b, &c)
		} else {
			res = append(res, in[i].(expr.Term))
		}
	}
	return res, b
}

// returns in split into a slice of compiledTerm and the product of all constants in in as a coeff
func (builder *builder) filterConstantProd(in []frontend.Variable) (expr.LinearExpression, constraint.Coeff) {
	res := make(expr.LinearExpression, 0, len(in))
	b := builder.tOne
	for i := 0; i < len(in); i++ {
		if c, ok := builder.constantValue(in[i]); ok {
			builder.cs.Mul(&b, &c)
		} else {
			res = append(res, in[i].(expr.Term))
		}
	}
	return res, b
}

func (builder *builder) splitSum(acc expr.Term, r expr.LinearExpression, k *constraint.Coeff) expr.Term {
	// floor case
	if len(r) == 0 {
		if k != nil {
			// we need to return acc + k
			o, found := builder.addConstraintExist(acc, expr.Term{}, *k)
			if !found {
				o = builder.newInternalVariable()
				builder.addPlonkConstraint(sparseR1C{
					xa: acc.VID,
					xc: o.VID,
					qL: acc.Coeff,
					qO: builder.tMinusOne,
					qC: *k,
				})
			}

			return o
		}
		return acc
	}

	// constraint to add: acc + r[0] (+ k) == o
	qC := constraint.Coeff{}
	if k != nil {
		qC = *k
	}
	o, found := builder.addConstraintExist(acc, r[0], qC)
	if !found {
		o = builder.newInternalVariable()

		builder.addPlonkConstraint(sparseR1C{
			xa: acc.VID,
			xb: r[0].VID,
			xc: o.VID,
			qL: acc.Coeff,
			qR: r[0].Coeff,
			qO: builder.tMinusOne,
			qC: qC,
		})
	}

	return builder.splitSum(o, r[1:], nil)
}

// addConstraintExist check if we recorded a constraint in the form
// q1*xa + q2*xb + qC - xc == 0
//
// if we find one, this function returns the xc wire with the correct coefficients.
// if we don't, and no previous addition was recorded with xa and xb, add an entry in the map
// (this assumes that the caller will add a constraint just after this call if it's not found!)
//
// idea:
// 1. take (xa | (xb << 32)) as a identifier of an addition that used wires xa and xb.
// 2. look for an entry in builder.mAddConstraints for a previously added constraint that matches.
// 3. if so, check that the coefficients matches and we can re-use xc wire.
//
// limitations:
// 1. for efficiency, we just store the first addition that occurred with with xa and xb;
// so if we do 2*xa + 3*xb == c, then want to compute xa + xb == d multiple times, the compiler is
// not going to catch these duplicates.
// 2. this piece of code assumes some behavior from constraint/ package (like coeffIDs, or append-style
// constraint management)
func (builder *builder) addConstraintExist(a, b expr.Term, k constraint.Coeff) (expr.Term, bool) {
	// ensure deterministic combined identifier;
	if a.VID < b.VID {
		a, b = b, a
	}
	h := uint64(a.WireID()) | uint64(b.WireID()<<32)

	if cID, ok := builder.mAddConstraints[h]; ok {
		// seems likely we have a fit, let's double check
		if c := builder.cs.GetConstraint(cID); c != nil {
			if c.M[0].CoeffID() != constraint.CoeffIdZero {
				panic("sanity check failed; recorded a add constraint with qM set")
			}

			if a.WireID() == c.R.WireID() {
				a, b = b, a // ensure a is in qL
			}
			if (a.WireID() != c.L.WireID()) || (b.WireID() != c.R.WireID()) {
				// that shouldn't happen; it means we added an entry in the duplicate add constraint
				// map with a key that don't match the entries.
				log := logger.Logger()
				log.Error().Msg("mAddConstraints entry doesn't match key")
				return expr.Term{}, false
			}

			// qO == -1
			if c.O.CoeffID() != constraint.CoeffIdMinusOne {
				// we could probably handle that case, but it shouldn't
				// happen with our current APIs --> each time we record a add gate in the duplicate
				// map qO == -1
				return expr.Term{}, false
			}

			tk := builder.cs.MakeTerm(&k, 0)
			if tk.CoeffID() != c.K {
				// the constant part of the addition differs, no point going forward
				// since we will need to add a new constraint anyway.
				return expr.Term{}, false
			}

			// check that the coeff matches
			qL := a.Coeff
			qR := b.Coeff
			ta := builder.cs.MakeTerm(&qL, 0)
			tb := builder.cs.MakeTerm(&qR, 0)
			if c.L.CoeffID() != ta.CoeffID() || c.R.CoeffID() != tb.CoeffID() {
				if !k.IsZero() {
					// may be for some edge cases we could avoid adding a constraint here.
					return expr.Term{}, false
				}
				// we recorded an addition in the form q1*a + q2*b == c
				// we want to record a new one in the form q3*a + q4*b == n*c
				// question is; can we re-use c to avoid introducing a new wire & new constraint
				// this is possible only if n == q3/q1 == q4/q2, that is, q3q2 == q1q4
				q1 := builder.cs.GetCoefficient(c.L.CoeffID())
				q2 := builder.cs.GetCoefficient(c.R.CoeffID())
				q3 := qL
				q4 := qR
				builder.cs.Mul(&q3, &q2)
				builder.cs.Mul(&q1, &q4)
				if q1 == q3 {
					// no need to introduce a new constraint;
					// compute n, the coefficient for the output wire
					builder.cs.Inverse(&q2)
					builder.cs.Mul(&q2, &q4)
					return expr.NewTerm(c.O.WireID(), q2), true
				}
				// we will need an additional constraint
				return expr.Term{}, false
			}

			// we found the same constraint!
			return expr.NewTerm(c.O.WireID(), builder.tOne), true
		}
	}
	// we are going to add this constraint, so we mark it.
	// ! assumes the caller add a constraint immediately  after the call to this function
	builder.mAddConstraints[h] = builder.cs.GetNbConstraints()
	return expr.Term{}, false
}

// mulConstraintExist check if we recorded a constraint in the form
// qM*xa*xb - xc == 0
//
// if we find one, this function returns the xc wire with the correct coefficients.
// if we don't, and no previous multiplication was recorded with xa and xb, add an entry in the map
// (this assumes that the caller will add a constraint just after this call if it's not found!)
//
// idea:
// 1. take (xa | (xb << 32)) as a identifier of a multiplication that used wires xa and xb.
// 2. look for an entry in builder.mMulConstraints for a previously added constraint that matches.
// 3. if so, compute correct coefficient N for xc wire that matches qM'*xa*xb - N*xc == 0
//
// limitations:
// 1. this piece of code assumes some behavior from constraint/ package (like coeffIDs, or append-style
// constraint management)
func (builder *builder) mulConstraintExist(a, b expr.Term) (expr.Term, bool) {
	// ensure deterministic combined identifier;
	if a.VID < b.VID {
		a, b = b, a
	}
	h := uint64(a.WireID()) | uint64(b.WireID()<<32)
	if a.VID < b.VID {
		a, b = b, a
	}

	if cID, ok := builder.mMulConstraints[h]; ok {
		// seems likely we have a fit, let's double check
		if c := builder.cs.GetConstraint(cID); c != nil {
			if !(c.K|c.L.CoeffID()|c.R.CoeffID() == constraint.CoeffIdZero) {
				panic("sanity check failed; recorded a mul constraint with qL, qR or qC set")
			}

			// qO == -1
			if c.O.CoeffID() != constraint.CoeffIdMinusOne {
				// we could probably handle that case, but it shouldn't
				// happen with our current APIs --> each time we record a mul gate in the duplicate
				// map qO == -1
				return expr.Term{}, false
			}

			if a.WireID() == c.R.WireID() {
				a, b = b, a // ensure a is in qL
			}
			if (a.WireID() != c.L.WireID()) || (b.WireID() != c.R.WireID()) {
				// that shouldn't happen; it means we added an entry in the duplicate mul constraint
				// map with a key that don't match the entries.
				log := logger.Logger()
				log.Error().Msg("mMulConstraints entry doesn't match key")
				return expr.Term{}, false
			}

			// recompute the qM coeff and check that it matches;
			qM := a.Coeff
			builder.cs.Mul(&qM, &b.Coeff)
			tm := builder.cs.MakeTerm(&qM, 0)
			if c.M[0].CoeffID() != tm.CoeffID() {
				// so we wanted to compute
				// N * xC == qM*xA*xB
				// but found a constraint
				// xC == qM'*xA*xB
				// the coefficient for our resulting wire is different;
				// N = qM / qM'
				N := builder.cs.GetCoefficient(c.M[0].CoeffID())
				builder.cs.Inverse(&N)
				builder.cs.Mul(&N, &qM)

				return expr.NewTerm(c.O.WireID(), N), true
			}

			// we found the exact same constraint
			return expr.NewTerm(c.O.WireID(), builder.tOne), true
		}
	}

	// we are going to add this constraint, so we mark it.
	// ! assumes the caller add a constraint immediately  after the call to this function
	builder.mMulConstraints[h] = builder.cs.GetNbConstraints()
	return expr.Term{}, false
}

func (builder *builder) splitProd(acc expr.Term, r expr.LinearExpression) expr.Term {
	// floor case
	if len(r) == 0 {
		return acc
	}
	// we want to add a constraint such that acc * r[0] == o
	// let's check if we didn't already constrain a similar product
	o, found := builder.mulConstraintExist(acc, r[0])

	if !found {
		// constraint to add: acc * r[0] == o
		o = builder.newInternalVariable()
		builder.addMulGate(acc, r[0], o)
	}

	return builder.splitProd(o, r[1:])
}

// newDebugInfo this is temporary to restore debug logs
// something more like builder.sprintf("my message %le %lv", l0, l1)
// to build logs for both debug and println
// and append some program location.. (see other todo in debug_info.go)
func (builder *builder) newDebugInfo(errName string, in ...interface{}) constraint.DebugInfo {
	for i := 0; i < len(in); i++ {
		// for inputs that are LinearExpressions or Term, we need to "Make" them in the backend.
		// TODO @gbotrel this is a duplicate effort with adding a constraint and should be taken care off

		switch t := in[i].(type) {
		case *expr.LinearExpression, expr.LinearExpression:
			// shouldn't happen
		case expr.Term:
			in[i] = builder.cs.MakeTerm(&t.Coeff, t.VID)
		case *expr.Term:
			in[i] = builder.cs.MakeTerm(&t.Coeff, t.VID)
		case constraint.Coeff:
			in[i] = builder.cs.String(&t)
		case *constraint.Coeff:
			in[i] = builder.cs.String(t)
		}
	}

	return builder.cs.NewDebugInfo(errName, in...)

}

func (builder *builder) Defer(cb func(frontend.API) error) {
	circuitdefer.Put(builder, cb)
}
