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

package test

import (
	"fmt"
	"math/big"
	"path/filepath"
	"reflect"
	"runtime"
	"strconv"
	"strings"
	"sync/atomic"

	"github.com/bits-and-blooms/bitset"
	"github.com/consensys/gnark/constraint"

	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/debug"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/logger"
	"golang.org/x/crypto/sha3"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/field/pool"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/internal/circuitdefer"
	"github.com/consensys/gnark/internal/kvstore"
	"github.com/consensys/gnark/internal/utils"
)

// engine implements frontend.API
//
// it is used for a faster verification of witness in tests
// and more importantly, for fuzzing purposes
//
// it converts the inputs to the API to big.Int (after a mod reduce using the curve base field)
type engine struct {
	curveID ecc.ID
	q       *big.Int
	opt     backend.ProverConfig
	// mHintsFunctions map[hint.ID]hintFunction
	constVars bool
	kvstore.Store
	blueprints        []constraint.Blueprint
	internalVariables []*big.Int
}

// TestEngineOption defines an option for the test engine.
type TestEngineOption func(e *engine) error

// SetAllVariablesAsConstants is a test engine option which makes the calls to
// IsConstant() and ConstantValue() always return true. If this test engine
// option is not set, then all variables are considered as non-constant,
// regardless if it is constructed by a call to ConstantValue().
func SetAllVariablesAsConstants() TestEngineOption {
	return func(e *engine) error {
		e.constVars = true
		return nil
	}
}

// WithBackendProverOptions is a test engine option which allows to define
// prover options. If not set, then default prover configuration is used.
func WithBackendProverOptions(opts ...backend.ProverOption) TestEngineOption {
	return func(e *engine) error {
		cfg, err := backend.NewProverConfig(opts...)
		if err != nil {
			return fmt.Errorf("new prover config: %w", err)
		}
		e.opt = cfg
		return nil
	}
}

// IsSolved returns an error if the test execution engine failed to execute the given circuit
// with provided witness as input.
//
// The test execution engine implements frontend.API using big.Int operations.
//
// This is an experimental feature.
func IsSolved(circuit, witness frontend.Circuit, field *big.Int, opts ...TestEngineOption) (err error) {
	e := &engine{
		curveID:   utils.FieldToCurve(field),
		q:         new(big.Int).Set(field),
		constVars: false,
		Store:     kvstore.New(),
	}
	for _, opt := range opts {
		if err := opt(e); err != nil {
			return fmt.Errorf("apply option: %w", err)
		}
	}

	// TODO handle opt.LoggerOut ?

	// we clone the circuit, in case the circuit has some attributes it uses in its Define function
	// set by the user.
	// then, we set all the variables values to the ones from the witness

	// clone the circuit
	c := shallowClone(circuit)

	// set the witness values
	copyWitness(c, witness)

	defer func() {
		if r := recover(); r != nil {
			err = fmt.Errorf("%v\n%s", r, string(debug.Stack()))
		}
	}()

	log := logger.Logger()
	log.Debug().Msg("running circuit in test engine")
	cptAdd, cptMul, cptSub, cptToBinary, cptFromBinary, cptAssertIsEqual = 0, 0, 0, 0, 0, 0

	// first we reset the stateful blueprints
	for i := range e.blueprints {
		if b, ok := e.blueprints[i].(constraint.BlueprintStateful); ok {
			b.Reset()
		}
	}

	if err = c.Define(e); err != nil {
		return fmt.Errorf("define: %w", err)
	}
	if err = callDeferred(e); err != nil {
		return fmt.Errorf("deferred: %w", err)
	}

	log.Debug().Uint64("add", cptAdd).
		Uint64("sub", cptSub).
		Uint64("mul", cptMul).
		Uint64("equals", cptAssertIsEqual).
		Uint64("toBinary", cptToBinary).
		Uint64("fromBinary", cptFromBinary).Msg("counters")

	return
}

func callDeferred(builder *engine) error {
	for i := 0; i < len(circuitdefer.GetAll[func(frontend.API) error](builder)); i++ {
		if err := circuitdefer.GetAll[func(frontend.API) error](builder)[i](builder); err != nil {
			return fmt.Errorf("defer fn %d: %w", i, err)
		}
	}
	return nil
}

var cptAdd, cptMul, cptSub, cptToBinary, cptFromBinary, cptAssertIsEqual uint64

func (e *engine) Add(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	atomic.AddUint64(&cptAdd, 1)
	res := new(big.Int)
	res.Add(e.toBigInt(i1), e.toBigInt(i2))
	for i := 0; i < len(in); i++ {
		atomic.AddUint64(&cptAdd, 1)
		res.Add(res, e.toBigInt(in[i]))
	}
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) MulAcc(a, b, c frontend.Variable) frontend.Variable {
	bc := pool.BigInt.Get()
	bc.Mul(e.toBigInt(b), e.toBigInt(c))

	res := new(big.Int)
	_a := e.toBigInt(a)
	res.Add(_a, bc).Mod(res, e.modulus())

	pool.BigInt.Put(bc)
	return res
}

func (e *engine) Sub(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	atomic.AddUint64(&cptSub, 1)
	res := new(big.Int)
	res.Sub(e.toBigInt(i1), e.toBigInt(i2))
	for i := 0; i < len(in); i++ {
		atomic.AddUint64(&cptSub, 1)
		res.Sub(res, e.toBigInt(in[i]))
	}
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) Neg(i1 frontend.Variable) frontend.Variable {
	res := new(big.Int)
	res.Neg(e.toBigInt(i1))
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) Mul(i1, i2 frontend.Variable, in ...frontend.Variable) frontend.Variable {
	atomic.AddUint64(&cptMul, 1)
	b2 := e.toBigInt(i2)
	if len(in) == 0 && b2.IsUint64() && b2.Uint64() <= 1 {
		// special path to avoid useless allocations
		if b2.Uint64() == 0 {
			return 0
		}
		return i1
	}
	b1 := e.toBigInt(i1)
	res := new(big.Int)
	res.Mul(b1, b2)
	res.Mod(res, e.modulus())
	for i := 0; i < len(in); i++ {
		atomic.AddUint64(&cptMul, 1)
		res.Mul(res, e.toBigInt(in[i]))
		res.Mod(res, e.modulus())
	}
	return res
}

func (e *engine) Div(i1, i2 frontend.Variable) frontend.Variable {
	res := new(big.Int)
	if res.ModInverse(e.toBigInt(i2), e.modulus()) == nil {
		panic("no inverse")
	}
	res.Mul(res, e.toBigInt(i1))
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) DivUnchecked(i1, i2 frontend.Variable) frontend.Variable {
	res := new(big.Int)
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	if b1.IsUint64() && b2.IsUint64() && b1.Uint64() == 0 && b2.Uint64() == 0 {
		return 0
	}
	if res.ModInverse(b2, e.modulus()) == nil {
		panic("no inverse")
	}
	res.Mul(res, b1)
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) Inverse(i1 frontend.Variable) frontend.Variable {
	res := new(big.Int)
	if res.ModInverse(e.toBigInt(i1), e.modulus()) == nil {
		panic("no inverse")
	}
	return res
}

func (e *engine) BatchInvert(in []frontend.Variable) []frontend.Variable {
	// having a batch invert saves a lot of ops in the test engine (ModInverse is terribly inefficient)
	_in := make([]*big.Int, len(in))
	for i := 0; i < len(_in); i++ {
		_in[i] = e.toBigInt(in[i])
	}

	_out := e.batchInvert(_in)

	res := make([]frontend.Variable, len(in))
	for i := 0; i < len(in); i++ {
		res[i] = _out[i]
	}
	return res
}

func (e *engine) batchInvert(a []*big.Int) []*big.Int {
	res := make([]*big.Int, len(a))
	for i := range res {
		res[i] = new(big.Int)
	}
	if len(a) == 0 {
		return res
	}

	zeroes := bitset.New(uint(len(a)))
	accumulator := new(big.Int).SetUint64(1)

	for i := 0; i < len(a); i++ {
		if a[i].Sign() == 0 {
			zeroes.Set(uint(i))
			continue
		}
		res[i].Set(accumulator)

		accumulator.Mul(accumulator, a[i])
		accumulator.Mod(accumulator, e.modulus())
	}

	accumulator.ModInverse(accumulator, e.modulus())

	for i := len(a) - 1; i >= 0; i-- {
		if zeroes.Test(uint(i)) {
			continue
		}
		res[i].Mul(res[i], accumulator)
		res[i].Mod(res[i], e.modulus())
		accumulator.Mul(accumulator, a[i])
		accumulator.Mod(accumulator, e.modulus())
	}

	return res
}

func (e *engine) ToBinary(i1 frontend.Variable, n ...int) []frontend.Variable {
	atomic.AddUint64(&cptToBinary, 1)
	nbBits := e.FieldBitLen()
	if len(n) == 1 {
		nbBits = n[0]
		if nbBits < 0 {
			panic("invalid n")
		}
	}

	b1 := e.toBigInt(i1)

	if b1.BitLen() > nbBits {
		panic(fmt.Sprintf("[ToBinary] decomposing %s (bitLen == %d) with %d bits", b1.String(), b1.BitLen(), nbBits))
	}

	r := make([]frontend.Variable, nbBits)
	ri := make([]frontend.Variable, nbBits)
	for i := 0; i < len(r); i++ {
		r[i] = (b1.Bit(i))
		ri[i] = r[i]
	}

	// this is a sanity check, it should never happen
	value := e.toBigInt(e.FromBinary(ri...))
	if value.Cmp(b1) != 0 {

		panic(fmt.Sprintf("[ToBinary] decomposing %s (bitLen == %d) with %d bits reconstructs into %s", b1.String(), b1.BitLen(), nbBits, value.String()))
	}
	return r
}

func (e *engine) FromBinary(v ...frontend.Variable) frontend.Variable {
	atomic.AddUint64(&cptFromBinary, 1)
	bits := make([]bool, len(v))
	for i := 0; i < len(v); i++ {
		be := e.toBigInt(v[i])
		e.mustBeBoolean(be)
		bits[i] = be.Uint64() == 1

	}

	// Σ (2**i * bits[i]) == r
	c := new(big.Int)
	r := new(big.Int)
	c.SetUint64(1)

	for i := 0; i < len(bits); i++ {
		if bits[i] {
			r.Add(r, c)
		}
		c.Lsh(c, 1)
	}
	r.Mod(r, e.modulus())

	return r
}

func (e *engine) Xor(i1, i2 frontend.Variable) frontend.Variable {
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	e.mustBeBoolean(b1)
	e.mustBeBoolean(b2)
	res := new(big.Int)
	res.Xor(b1, b2)
	return res
}

func (e *engine) Or(i1, i2 frontend.Variable) frontend.Variable {
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	e.mustBeBoolean(b1)
	e.mustBeBoolean(b2)
	res := new(big.Int)
	res.Or(b1, b2)
	return res
}

func (e *engine) And(i1, i2 frontend.Variable) frontend.Variable {
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	e.mustBeBoolean(b1)
	e.mustBeBoolean(b2)
	res := new(big.Int)
	res.And(b1, b2)
	return res
}

// Select if b is true, yields i1 else yields i2
func (e *engine) Select(b frontend.Variable, i1, i2 frontend.Variable) frontend.Variable {
	b1 := e.toBigInt(b)
	e.mustBeBoolean(b1)

	if b1.Uint64() == 1 {
		return e.toBigInt(i1)
	}
	return (e.toBigInt(i2))
}

// Lookup2 performs a 2-bit lookup between i1, i2, i3, i4 based on bits b0
// and b1. Returns i0 if b0=b1=0, i1 if b0=1 and b1=0, i2 if b0=0 and b1=1
// and i3 if b0=b1=1.
func (e *engine) Lookup2(b0, b1 frontend.Variable, i0, i1, i2, i3 frontend.Variable) frontend.Variable {
	s0 := e.toBigInt(b0)
	s1 := e.toBigInt(b1)
	e.mustBeBoolean(s0)
	e.mustBeBoolean(s1)
	lookup := new(big.Int).Lsh(s1, 1)
	lookup.Or(lookup, s0)
	return e.toBigInt([]frontend.Variable{i0, i1, i2, i3}[lookup.Uint64()])
}

// IsZero returns 1 if a is zero, 0 otherwise
func (e *engine) IsZero(i1 frontend.Variable) frontend.Variable {
	b1 := e.toBigInt(i1)

	if b1.IsUint64() && b1.Uint64() == 0 {
		return big.NewInt(1)
	}

	return big.NewInt(0)
}

// Cmp returns 1 if i1>i2, 0 if i1==i2, -1 if i1<i2
func (e *engine) Cmp(i1, i2 frontend.Variable) frontend.Variable {
	b1 := e.toBigInt(i1)
	b2 := e.toBigInt(i2)
	res := big.NewInt(int64(b1.Cmp(b2)))
	res.Mod(res, e.modulus())
	return res
}

func (e *engine) AssertIsEqual(i1, i2 frontend.Variable) {
	atomic.AddUint64(&cptAssertIsEqual, 1)
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	if b1.Cmp(b2) != 0 {
		panic(fmt.Sprintf("[assertIsEqual] %s == %s", b1.String(), b2.String()))
	}
}

func (e *engine) AssertIsDifferent(i1, i2 frontend.Variable) {
	b1, b2 := e.toBigInt(i1), e.toBigInt(i2)
	if b1.Cmp(b2) == 0 {
		panic(fmt.Sprintf("[assertIsDifferent] %s != %s", b1.String(), b2.String()))
	}
}

func (e *engine) AssertIsBoolean(i1 frontend.Variable) {
	b1 := e.toBigInt(i1)
	e.mustBeBoolean(b1)
}

func (e *engine) AssertIsCrumb(i1 frontend.Variable) {
	i1 = e.MulAcc(e.Mul(-3, i1), i1, i1)
	i1 = e.MulAcc(e.Mul(2, i1), i1, i1)
	e.AssertIsEqual(i1, 0)
}

func (e *engine) AssertIsLessOrEqual(v frontend.Variable, bound frontend.Variable) {

	bValue := e.toBigInt(bound)

	if bValue.Sign() == -1 {
		panic(fmt.Sprintf("[assertIsLessOrEqual] bound (%s) must be positive", bValue.String()))
	}

	b1 := e.toBigInt(v)
	if b1.Cmp(bValue) == 1 {
		panic(fmt.Sprintf("[assertIsLessOrEqual] %s > %s", b1.String(), bValue.String()))
	}
}

func (e *engine) Println(a ...frontend.Variable) {
	var sbb strings.Builder
	sbb.WriteString("(test.engine) ")

	// prefix log line with file.go:line
	if _, file, line, ok := runtime.Caller(1); ok {
		sbb.WriteString(filepath.Base(file))
		sbb.WriteByte(':')
		sbb.WriteString(strconv.Itoa(line))
		sbb.WriteByte(' ')
	}

	for i := 0; i < len(a); i++ {
		e.print(&sbb, a[i])
		sbb.WriteByte(' ')
	}
	fmt.Println(sbb.String())
}

func (e *engine) print(sbb *strings.Builder, x interface{}) {
	switch v := x.(type) {
	case string:
		sbb.WriteString(v)
	case []frontend.Variable:
		sbb.WriteRune('[')
		for i := range v {
			e.print(sbb, v[i])
			if i+1 != len(v) {
				sbb.WriteRune(',')
			}
		}
		sbb.WriteRune(']')
	default:
		i := e.toBigInt(v)
		var iAsNeg big.Int
		iAsNeg.Sub(i, e.q)
		if iAsNeg.IsInt64() {
			sbb.WriteString(strconv.FormatInt(iAsNeg.Int64(), 10))
		} else {
			sbb.WriteString(i.String())
		}
	}
}

func (e *engine) NewHint(f solver.Hint, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {

	if nbOutputs <= 0 {
		return nil, fmt.Errorf("hint function must return at least one output")
	}

	in := make([]*big.Int, len(inputs))

	for i := 0; i < len(inputs); i++ {
		in[i] = e.toBigInt(inputs[i])
	}
	res := make([]*big.Int, nbOutputs)
	for i := range res {
		res[i] = new(big.Int)
	}

	err := f(e.Field(), in, res)

	if err != nil {
		panic("NewHint: " + err.Error())
	}

	out := make([]frontend.Variable, len(res))
	for i := range res {
		res[i].Mod(res[i], e.q)
		out[i] = res[i]
	}

	return out, nil
}

func (e *engine) NewHintForId(id solver.HintID, nbOutputs int, inputs ...frontend.Variable) ([]frontend.Variable, error) {
	if f := solver.GetRegisteredHint(id); f != nil {
		return e.NewHint(f, nbOutputs, inputs...)
	}

	return nil, fmt.Errorf("no hint registered with id #%d. Use solver.RegisterHint or solver.RegisterNamedHint", id)
}

// IsConstant returns true if v is a constant known at compile time
func (e *engine) IsConstant(v frontend.Variable) bool {
	return e.constVars
}

// ConstantValue returns the big.Int value of v
func (e *engine) ConstantValue(v frontend.Variable) (*big.Int, bool) {
	r := e.toBigInt(v)
	return r, e.constVars
}

func (e *engine) IsBoolean(v frontend.Variable) bool {
	r := e.toBigInt(v)
	return r.IsUint64() && r.Uint64() <= 1
}

func (e *engine) MarkBoolean(v frontend.Variable) {
	if !e.IsBoolean(v) {
		panic("mark boolean a non-boolean value")
	}
}

func (e *engine) toBigInt(i1 frontend.Variable) *big.Int {
	switch vv := i1.(type) {
	case *big.Int:
		return vv
	case big.Int:
		return &vv
	default:
		b := utils.FromInterface(i1)
		b.Mod(&b, e.modulus())
		return &b
	}
}

// FieldBitLen returns the number of bits needed to represent a fr.Element
func (e *engine) FieldBitLen() int {
	return e.q.BitLen()
}

func (e *engine) mustBeBoolean(b *big.Int) {
	if !b.IsUint64() || !(b.Uint64() == 0 || b.Uint64() == 1) {
		panic(fmt.Sprintf("[assertIsBoolean] %s", b.String()))
	}
}

func (e *engine) modulus() *big.Int {
	return e.q
}

// shallowClone clones given circuit
// this is actually a shallow copy → if the circuits contains maps or slices
// only the reference is copied.
func shallowClone(circuit frontend.Circuit) frontend.Circuit {

	cValue := reflect.ValueOf(circuit).Elem()
	newCircuit := reflect.New(cValue.Type())
	newCircuit.Elem().Set(cValue)

	circuitCopy, ok := newCircuit.Interface().(frontend.Circuit)
	if !ok {
		panic("couldn't clone the circuit")
	}

	if !reflect.DeepEqual(circuitCopy, circuit) {
		panic("clone failed")
	}

	return circuitCopy
}

func copyWitness(to, from frontend.Circuit) {
	var wValues []reflect.Value

	collectHandler := func(f schema.LeafInfo, tInput reflect.Value) error {
		if tInput.IsNil() {
			// TODO @gbotrel test for missing assignment
			return fmt.Errorf("when parsing variable %s: missing assignment", f.FullName())
		}
		wValues = append(wValues, tInput)
		return nil
	}
	if _, err := schema.Walk(from, tVariable, collectHandler); err != nil {
		panic(err)
	}

	i := 0
	setHandler := func(f schema.LeafInfo, tInput reflect.Value) error {
		tInput.Set(wValues[i])
		i++
		return nil
	}
	// this can't error.
	_, _ = schema.Walk(to, tVariable, setHandler)

}

func (e *engine) Field() *big.Int {
	return e.q
}

func (e *engine) Compiler() frontend.Compiler {
	return e
}

func (e *engine) Commit(v ...frontend.Variable) (frontend.Variable, error) {
	nb := (e.FieldBitLen() + 7) / 8
	buf := make([]byte, nb)
	hasher := sha3.NewCShake128(nil, []byte("gnark test engine"))
	for i := range v {
		vs := e.toBigInt(v[i])
		bs := vs.FillBytes(buf)
		hasher.Write(bs)
	}
	hasher.Read(buf)
	res := new(big.Int).SetBytes(buf)
	res.Mod(res, e.modulus())
	if res.Sign() == 0 {
		// a commit == 0 is unlikely; happens quite often in tests
		// with tinyfield
		res.SetUint64(1)
	}
	return res, nil
}

func (e *engine) Defer(cb func(frontend.API) error) {
	circuitdefer.Put(e, cb)
}

// AddInstruction is used to add custom instructions to the constraint system.
// In constraint system, this is asynchronous. In here, we do it synchronously.
func (e *engine) AddInstruction(bID constraint.BlueprintID, calldata []uint32) []uint32 {
	blueprint := e.blueprints[bID].(constraint.BlueprintSolvable)

	// create a dummy instruction
	inst := constraint.Instruction{
		Calldata:   calldata,
		WireOffset: uint32(len(e.internalVariables)),
	}

	// blueprint declared nbOutputs; add as many internal variables
	// and return their indices
	nbOutputs := blueprint.NbOutputs(inst)
	var r []uint32
	for i := 0; i < nbOutputs; i++ {
		r = append(r, uint32(len(e.internalVariables)))
		e.internalVariables = append(e.internalVariables, new(big.Int))
	}

	// solve the blueprint synchronously
	s := blueprintSolver{
		internalVariables: e.internalVariables,
		q:                 e.q,
	}
	if err := blueprint.Solve(&s, inst); err != nil {
		panic(err)
	}

	return r
}

// AddBlueprint adds a custom blueprint to the constraint system.
func (e *engine) AddBlueprint(b constraint.Blueprint) constraint.BlueprintID {
	if _, ok := b.(constraint.BlueprintSolvable); !ok {
		panic("unsupported blueprint in test engine")
	}
	e.blueprints = append(e.blueprints, b)
	return constraint.BlueprintID(len(e.blueprints) - 1)
}

// InternalVariable returns the value of an internal variable. This is used in custom blueprints.
// The variableID is the index of the variable in the internalVariables slice, as
// filled by AddInstruction.
func (e *engine) InternalVariable(vID uint32) frontend.Variable {
	if vID >= uint32(len(e.internalVariables)) {
		panic("internal variable not found")
	}
	return new(big.Int).Set(e.internalVariables[vID])
}

// ToCanonicalVariable converts a frontend.Variable to a frontend.CanonicalVariable
// this is used in custom blueprints to return a variable than can be encoded in blueprints
func (e *engine) ToCanonicalVariable(v frontend.Variable) frontend.CanonicalVariable {
	r := e.toBigInt(v)
	return wrappedBigInt{r}
}

func (e *engine) SetGkrInfo(info constraint.GkrInfo) error {
	return fmt.Errorf("not implemented")
}

// MustBeLessOrEqCst implements method comparing value given by its bits aBits
// to a bound.
func (e *engine) MustBeLessOrEqCst(aBits []frontend.Variable, bound *big.Int, aForDebug frontend.Variable) {
	v := new(big.Int)
	for i, b := range aBits {
		bb, ok := b.(*big.Int)
		if !ok {
			panic("not big.Int bit")
		}
		if !bb.IsUint64() {
			panic("given bit large")
		}
		bbu := uint(bb.Uint64())
		if bbu > 1 {
			fmt.Println(bbu)
			panic("given bit is not a bit")
		}
		v.SetBit(v, i, bbu)
	}
	if v.Cmp(bound) > 0 {
		panic(fmt.Sprintf("%d > %d", v, bound))
	}
}
