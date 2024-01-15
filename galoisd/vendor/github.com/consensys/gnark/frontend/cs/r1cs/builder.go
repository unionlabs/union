/*
Copyright © 2020 ConsenSys

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

package r1cs

import (
	"errors"
	"math/big"
	"reflect"
	"sort"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/internal/expr"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/internal/circuitdefer"
	"github.com/consensys/gnark/internal/frontendtype"
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

// NewBuilder returns a new R1CS builder which implements frontend.API.
// Additionally, this builder also implements [frontend.Committer].
func NewBuilder(field *big.Int, config frontend.CompileConfig) (frontend.Builder, error) {
	return newBuilder(field, config), nil
}

type builder struct {
	cs     constraint.R1CS
	config frontend.CompileConfig
	kvstore.Store

	// map for recording boolean constrained variables (to not constrain them twice)
	mtBooleans map[uint64][]expr.LinearExpression

	tOne        constraint.Element
	eZero, eOne expr.LinearExpression
	cZero, cOne constraint.LinearExpression

	// helps merge k sorted linear expressions
	heap minHeap

	// buffers used to do in place api.MAC
	mbuf1 expr.LinearExpression
	mbuf2 expr.LinearExpression

	genericGate constraint.BlueprintID
}

// initialCapacity has quite some impact on frontend performance, especially on large circuits size
// we may want to add build tags to tune that
func newBuilder(field *big.Int, config frontend.CompileConfig) *builder {
	macCapacity := 100
	if config.CompressThreshold != 0 {
		macCapacity = config.CompressThreshold
	}
	builder := builder{
		mtBooleans: make(map[uint64][]expr.LinearExpression, config.Capacity/10),
		config:     config,
		heap:       make(minHeap, 0, 100),
		mbuf1:      make(expr.LinearExpression, 0, macCapacity),
		mbuf2:      make(expr.LinearExpression, 0, macCapacity),
		Store:      kvstore.New(),
	}

	// by default the circuit is given a public wire equal to 1

	curve := utils.FieldToCurve(field)

	switch curve {
	case ecc.BLS12_377:
		builder.cs = bls12377r1cs.NewR1CS(config.Capacity)
	case ecc.BLS12_381:
		builder.cs = bls12381r1cs.NewR1CS(config.Capacity)
	case ecc.BN254:
		builder.cs = bn254r1cs.NewR1CS(config.Capacity)
	case ecc.BW6_761:
		builder.cs = bw6761r1cs.NewR1CS(config.Capacity)
	case ecc.BW6_633:
		builder.cs = bw6633r1cs.NewR1CS(config.Capacity)
	case ecc.BLS24_315:
		builder.cs = bls24315r1cs.NewR1CS(config.Capacity)
	case ecc.BLS24_317:
		builder.cs = bls24317r1cs.NewR1CS(config.Capacity)
	default:
		if field.Cmp(tinyfield.Modulus()) == 0 {
			builder.cs = tinyfieldr1cs.NewR1CS(config.Capacity)
			break
		}
		panic("not implemented")
	}

	builder.tOne = builder.cs.One()
	builder.cs.AddPublicVariable("1")

	builder.genericGate = builder.cs.AddBlueprint(&constraint.BlueprintGenericR1C{})

	builder.eZero = expr.NewLinearExpression(0, constraint.Element{})
	builder.eOne = expr.NewLinearExpression(0, builder.tOne)

	builder.cOne = constraint.LinearExpression{constraint.Term{VID: 0, CID: constraint.CoeffIdOne}}
	builder.cZero = constraint.LinearExpression{constraint.Term{VID: 0, CID: constraint.CoeffIdZero}}

	return &builder
}

// newInternalVariable creates a new wire, appends it on the list of wires of the circuit, sets
// the wire's id to the number of wires, and returns it
func (builder *builder) newInternalVariable() expr.LinearExpression {
	idx := builder.cs.AddInternalVariable()
	return expr.NewLinearExpression(idx, builder.tOne)
}

// PublicVariable creates a new public Variable
func (builder *builder) PublicVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddPublicVariable(f.FullName())
	return expr.NewLinearExpression(idx, builder.tOne)
}

// SecretVariable creates a new secret Variable
func (builder *builder) SecretVariable(f schema.LeafInfo) frontend.Variable {
	idx := builder.cs.AddSecretVariable(f.FullName())
	return expr.NewLinearExpression(idx, builder.tOne)
}

// cstOne return the one constant
func (builder *builder) cstOne() expr.LinearExpression {
	return builder.eOne
}

// cstZero return the zero constant
func (builder *builder) cstZero() expr.LinearExpression {
	return builder.eZero
}

func (builder *builder) isCstOne(c constraint.Element) bool {
	return builder.cs.IsOne(c)
}

func (builder *builder) Field() *big.Int {
	return builder.cs.Field()
}

func (builder *builder) FieldBitLen() int {
	return builder.cs.FieldBitLen()
}

// newR1C clones the linear expression associated with the Variables (to avoid offsetting the ID multiple time)
// and return a R1C
func (builder *builder) newR1C(l, r, o frontend.Variable) constraint.R1C {
	L := builder.getLinearExpression(l)
	R := builder.getLinearExpression(r)
	O := builder.getLinearExpression(o)

	// interestingly, this is key to groth16 performance.
	// l * r == r * l == o
	// but the "l" linear expression is going to end up in the A matrix
	// the "r" linear expression is going to end up in the B matrix
	// the less Variable we have appearing in the B matrix, the more likely groth16.Setup
	// is going to produce infinity points in pk.G1.B and pk.G2.B, which will speed up proving time
	if len(L) > len(R) {
		// TODO @gbotrel shouldn't we do the opposite? Code doesn't match comment.
		L, R = R, L
	}

	return constraint.R1C{L: L, R: R, O: O}
}

func (builder *builder) getLinearExpression(_l interface{}) constraint.LinearExpression {
	var L constraint.LinearExpression
	switch tl := _l.(type) {
	case expr.LinearExpression:
		if len(tl) == 1 && tl[0].VID == 0 {
			if tl[0].Coeff.IsZero() {
				return builder.cZero
			} else if tl[0].Coeff == builder.tOne {
				return builder.cOne
			}
		}
		L = make(constraint.LinearExpression, 0, len(tl))
		for _, t := range tl {
			L = append(L, builder.cs.MakeTerm(t.Coeff, t.VID))
		}
	case constraint.LinearExpression:
		L = tl
	default:
		if debug.Debug {
			panic("invalid input for getLinearExpression") // sanity check
		}
	}

	return L
}

// MarkBoolean sets (but do not **constraint**!) v to be boolean
// This is useful in scenarios where a variable is known to be boolean through a constraint
// that is not api.AssertIsBoolean. If v is a constant, this is a no-op.
func (builder *builder) MarkBoolean(v frontend.Variable) {
	if b, ok := builder.constantValue(v); ok {
		if !(b.IsZero() || builder.isCstOne(b)) {
			panic("MarkBoolean called a non-boolean constant")
		}
		return
	}
	// v is a linear expression
	l := v.(expr.LinearExpression)
	sort.Sort(l)

	key := l.HashCode()
	list := builder.mtBooleans[key]
	list = append(list, l)
	builder.mtBooleans[key] = list
}

// IsBoolean returns true if given variable was marked as boolean in the compiler (see MarkBoolean)
// Use with care; variable may not have been **constrained** to be boolean
// This returns true if the v is a constant and v == 0 || v == 1.
func (builder *builder) IsBoolean(v frontend.Variable) bool {
	if b, ok := builder.constantValue(v); ok {
		return (b.IsZero() || builder.isCstOne(b))
	}
	// v is a linear expression
	l := v.(expr.LinearExpression)
	sort.Sort(l)

	key := l.HashCode()
	list, ok := builder.mtBooleans[key]
	if !ok {
		return false
	}

	for _, v := range list {
		if v.Equal(l) {
			return true
		}
	}
	return false
}

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
}

// Compile constructs a rank-1 constraint system
func (builder *builder) Compile() (constraint.ConstraintSystem, error) {
	// TODO if already compiled, return builder.cs object
	log := logger.Logger()
	log.Info().
		Int("nbConstraints", builder.cs.GetNbConstraints()).
		Msg("building constraint builder")

	// ensure all inputs and hints are constrained
	if err := builder.cs.CheckUnconstrainedWires(); err != nil {
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
	return builder.cs.ToBigInt(coeff), true
}

func (builder *builder) constantValue(v frontend.Variable) (constraint.Element, bool) {
	if _v, ok := v.(expr.LinearExpression); ok {
		assertIsSet(_v)

		if len(_v) != 1 {
			// TODO @gbotrel this assumes linear expressions of coeff are not possible
			// and are always reduced to one element. may not always be true?
			return constraint.Element{}, false
		}
		if !(_v[0].WireID() == 0) { // public ONE WIRE
			return constraint.Element{}, false
		}
		return _v[0].Coeff, true
	}
	return builder.cs.FromInterface(v), true
}

// toVariable will return (and allocate if necessary) a linearExpression from given value
//
// if input is already a linearExpression, does nothing
// else, attempts to convert input to a big.Int (see utils.FromInterface) and returns a toVariable linearExpression
func (builder *builder) toVariable(input interface{}) expr.LinearExpression {

	switch t := input.(type) {
	case expr.LinearExpression:
		// this is already a "kwown" variable
		assertIsSet(t)
		return t
	case *expr.LinearExpression:
		assertIsSet(*t)
		return *t
	case constraint.Element:
		return expr.NewLinearExpression(0, t)
	case *constraint.Element:
		return expr.NewLinearExpression(0, *t)
	default:
		// try to make it into a constant
		c := builder.cs.FromInterface(t)
		return expr.NewLinearExpression(0, c)
	}
}

// toVariables return frontend.Variable corresponding to inputs and the total size of the linear expressions
func (builder *builder) toVariables(in ...frontend.Variable) ([]expr.LinearExpression, int) {
	r := make([]expr.LinearExpression, 0, len(in))
	s := 0
	e := func(i frontend.Variable) {
		v := builder.toVariable(i)
		r = append(r, v)
		s += len(v)
	}
	// e(i1)
	// e(i2)
	for i := 0; i < len(in); i++ {
		e(in[i])
	}
	return r, s
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
	return builder.newHint(f, solver.GetHintID(f), nbOutputs, inputs)
}

func (builder *builder) NewHintForId(id solver.HintID, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	return builder.newHint(nil, id, nbOutputs, inputs)
}

func (builder *builder) newHint(f solver.Hint, id solver.HintID, nbOutputs int, inputs []frontend.Variable) ([]frontend.Variable, error) {
	hintInputs := make([]constraint.LinearExpression, len(inputs))

	// TODO @gbotrel hint input pass
	// ensure inputs are set and pack them in a []uint64
	for i, in := range inputs {
		if t, ok := in.(expr.LinearExpression); ok {
			assertIsSet(t)
			hintInputs[i] = builder.getLinearExpression(t)
		} else {
			c := builder.cs.FromInterface(in)
			term := builder.cs.MakeTerm(c, 0)
			term.MarkConstant()
			hintInputs[i] = constraint.LinearExpression{term}
		}
	}

	internalVariables, err := builder.cs.AddSolverHint(f, id, hintInputs, nbOutputs)
	if err != nil {
		return nil, err
	}

	// make the variables
	res := make([]frontend.Variable, len(internalVariables))
	for i, idx := range internalVariables {
		res[i] = expr.NewLinearExpression(idx, builder.tOne)
	}
	return res, nil
}

// assertIsSet panics if the variable is unset
// this may happen if inside a Define we have
// var a variable
// cs.Mul(a, 1)
// since a was not in the circuit struct it is not a secret variable
func assertIsSet(l expr.LinearExpression) {
	if len(l) == 0 {
		// errNoValue triggered when trying to access a variable that was not allocated
		errNoValue := errors.New("can't determine API input value")
		panic(errNoValue)
	}

	if debug.Debug {
		// frontend/ package must build linear expressions that are sorted.
		if !sort.IsSorted(l) {
			panic("unsorted linear expression")
		}
	}
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
		case *expr.LinearExpression:
			in[i] = builder.getLinearExpression(*t)
		case expr.LinearExpression:
			in[i] = builder.getLinearExpression(t)
		case expr.Term:
			in[i] = builder.getLinearExpression(expr.LinearExpression{t})
		case *expr.Term:
			in[i] = builder.getLinearExpression(expr.LinearExpression{*t})
		case constraint.Element:
			in[i] = builder.cs.String(t)
		case *constraint.Element:
			in[i] = builder.cs.String(*t)
		}
	}

	return builder.cs.NewDebugInfo(errName, in...)

}

// compress checks the length of the linear expression le and if it is larger or
// equal than CompressThreshold in the configuration, replaces it with a linear
// expression of one term. In that case it adds an equality constraint enforcing
// the correctness of the returned linear expression.
func (builder *builder) compress(le expr.LinearExpression) expr.LinearExpression {
	if builder.config.CompressThreshold <= 0 || len(le) < builder.config.CompressThreshold {
		return le
	}

	one := builder.cstOne()
	t := builder.newInternalVariable()
	builder.cs.AddR1C(builder.newR1C(le, one, t), builder.genericGate)
	return t
}

func (builder *builder) Defer(cb func(frontend.API) error) {
	circuitdefer.Put(builder, cb)
}

func (*builder) FrontendType() frontendtype.Type {
	return frontendtype.R1CS
}

// AddInstruction is used to add custom instructions to the constraint system.
func (builder *builder) AddInstruction(bID constraint.BlueprintID, calldata []uint32) []uint32 {
	return builder.cs.AddInstruction(bID, calldata)
}

// AddBlueprint adds a custom blueprint to the constraint system.
func (builder *builder) AddBlueprint(b constraint.Blueprint) constraint.BlueprintID {
	return builder.cs.AddBlueprint(b)
}

func (builder *builder) InternalVariable(wireID uint32) frontend.Variable {
	return expr.NewLinearExpression(int(wireID), builder.tOne)
}

// ToCanonicalVariable converts a frontend.Variable to a constraint system specific Variable
// ! Experimental: use in conjunction with constraint.CustomizableSystem
func (builder *builder) ToCanonicalVariable(in frontend.Variable) frontend.CanonicalVariable {
	if t, ok := in.(expr.LinearExpression); ok {
		assertIsSet(t)
		return builder.getLinearExpression(t)
	} else {
		c := builder.cs.FromInterface(in)
		term := builder.cs.MakeTerm(c, 0)
		term.MarkConstant()
		return constraint.LinearExpression{term}
	}
}
