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
	"errors"
	"fmt"
	"reflect"
	"strings"
	"testing"

	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/backend/plonkfri"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/frontend/cs/scs"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/stretchr/testify/require"
)

var (
	ErrCompilationNotDeterministic = errors.New("compilation is not deterministic")
	ErrInvalidWitnessSolvedCS      = errors.New("invalid witness solved the constraint system")
	ErrInvalidWitnessVerified      = errors.New("invalid witness resulted in a valid proof")
)

// Assert is a helper to test circuits
type Assert struct {
	t *testing.T
	*require.Assertions
	compiled map[string]constraint.ConstraintSystem // cache compilation
}

// NewAssert returns an Assert helper embedding a testify/require object for convenience
//
// The Assert object caches the compiled circuit:
//
// the first call to assert.ProverSucceeded/Failed will compile the circuit for n curves, m backends
// and subsequent calls will re-use the result of the compilation, if available.
func NewAssert(t *testing.T) *Assert {
	return &Assert{t: t, Assertions: require.New(t), compiled: make(map[string]constraint.ConstraintSystem)}
}

// Run runs the test function fn as a subtest. The subtest is parametrized by
// the description strings descs.
func (assert *Assert) Run(fn func(assert *Assert), descs ...string) {
	desc := strings.Join(descs, "/")
	assert.t.Run(desc, func(t *testing.T) {
		// TODO(ivokub): access to compiled cache is not synchronized -- running
		// the tests in parallel will result in undetermined behavior. A better
		// approach would be to synchronize compiled and run the tests in
		// parallel for a potential speedup.
		assert := &Assert{t, require.New(t), assert.compiled}
		fn(assert)
	})
}

// Log logs using the test instance logger.
func (assert *Assert) Log(v ...interface{}) {
	assert.t.Log(v...)
}

// ProverSucceeded fails the test if any of the following step errored:
//
// 1. compiles the circuit (or fetch it from the cache)
// 2. using the test execution engine, executes the circuit with provided witness
// 3. run Setup / Prove / Verify with the backend
// 4. if set, (de)serializes the witness and call ReadAndProve and ReadAndVerify on the backend
//
// By default, this tests on all curves and proving schemes supported by gnark. See available TestingOption.
func (assert *Assert) ProverSucceeded(circuit frontend.Circuit, validAssignment frontend.Circuit, opts ...TestingOption) {

	opt := assert.options(opts...)

	// for each {curve, backend} tuple
	for _, curve := range opt.curves {
		curve := curve
		// parse the assignment and instantiate the witness
		validWitness, err := frontend.NewWitness(validAssignment, curve.ScalarField())
		assert.NoError(err, "can't parse valid assignment")

		validPublicWitness, err := frontend.NewWitness(validAssignment, curve.ScalarField(), frontend.PublicOnly())
		assert.NoError(err, "can't parse valid assignment")

		if opt.witnessSerialization {
			// do a round trip marshalling test
			assert.Run(func(assert *Assert) {
				assert.t.Parallel()
				assert.marshalWitness(validWitness, curve, false)
			}, curve.String(), "marshal/binary")
			assert.Run(func(assert *Assert) {
				assert.t.Parallel()
				assert.marshalWitness(validPublicWitness, curve, true)
			}, curve.String(), "marshal-public/binary")

			if !testing.Short() {
				assert.Run(func(assert *Assert) {
					assert.t.Parallel()
					s := lazySchema(circuit)()
					assert.marshalWitnessJSON(validWitness, s, curve, false)
				}, curve.String(), "marshal/json")
				assert.Run(func(assert *Assert) {
					assert.t.Parallel()
					s := lazySchema(circuit)()
					assert.marshalWitnessJSON(validWitness, s, curve, true)
				}, curve.String(), "marshal-public/json")

			}
		}

		for _, b := range opt.backends {

			b := b
			assert.Run(func(assert *Assert) {

				checkError := func(err error) { assert.checkError(err, b, curve, validWitness, lazySchema(circuit)) }

				// 1- compile the circuit
				ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
				checkError(err)

				// must not error with big int test engine (only the curveID is needed for this test)
				err = IsSolved(circuit, validAssignment, curve.ScalarField())
				checkError(err)

				assert.t.Parallel()

				switch b {
				case backend.GROTH16:
					pk, vk, err := groth16.Setup(ccs)
					checkError(err)

					// ensure prove / verify works well with valid witnesses

					proof, err := groth16.Prove(ccs, pk, validWitness, opt.proverOpts...)
					checkError(err)

					err = groth16.Verify(proof, vk, validPublicWitness)
					checkError(err)

				case backend.PLONK:
					srs, err := NewKZGSRS(ccs)
					checkError(err)

					pk, vk, err := plonk.Setup(ccs, srs)
					checkError(err)

					correctProof, err := plonk.Prove(ccs, pk, validWitness, opt.proverOpts...)
					checkError(err)

					err = plonk.Verify(correctProof, vk, validPublicWitness)
					checkError(err)

				case backend.PLONKFRI:
					pk, vk, err := plonkfri.Setup(ccs)
					checkError(err)

					correctProof, err := plonkfri.Prove(ccs, pk, validWitness, opt.proverOpts...)
					checkError(err)

					err = plonkfri.Verify(correctProof, vk, validPublicWitness)
					checkError(err)

				default:
					panic("backend not implemented")
				}
			}, curve.String(), b.String())
		}
	}

	if opt.fuzzing {
		// TODO may not be the right place, but ensures all our tests call these minimal tests
		// (like filling a witness with zeroes, or binary values, ...)
		assert.Run(func(assert *Assert) {
			assert.Fuzz(circuit, 5, opts...)
		}, "fuzz")
	}
}

// ProverSucceeded fails the test if any of the following step errored:
//
// 1. compiles the circuit (or fetch it from the cache)
// 2. using the test execution engine, executes the circuit with provided witness (must fail)
// 3. run Setup / Prove / Verify with the backend (must fail)
//
// By default, this tests on all curves and proving schemes supported by gnark. See available TestingOption.
func (assert *Assert) ProverFailed(circuit frontend.Circuit, invalidAssignment frontend.Circuit, opts ...TestingOption) {

	opt := assert.options(opts...)

	for _, curve := range opt.curves {

		// parse assignment
		invalidWitness, err := frontend.NewWitness(invalidAssignment, curve.ScalarField())
		assert.NoError(err, "can't parse invalid assignment")

		for _, b := range opt.backends {
			curve := curve
			b := b
			assert.Run(func(assert *Assert) {

				checkError := func(err error) { assert.checkError(err, b, curve, invalidWitness, lazySchema(circuit)) }
				mustError := func(err error) { assert.mustError(err, b, curve, invalidWitness, lazySchema(circuit)) }

				// 1- compile the circuit
				ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
				checkError(err)

				// must error with big int test engine (only the curveID is needed here)
				err = IsSolved(circuit, invalidAssignment, curve.ScalarField())
				mustError(err)

				assert.t.Parallel()
				err = ccs.IsSolved(invalidWitness)
				mustError(err)
			}, curve.String(), b.String())
		}
	}
}

func (assert *Assert) SolvingSucceeded(circuit frontend.Circuit, validWitness frontend.Circuit, opts ...TestingOption) {

	opt := assert.options(opts...)

	for _, curve := range opt.curves {
		for _, b := range opt.backends {
			curve := curve
			b := b
			assert.Run(func(assert *Assert) {
				assert.solvingSucceeded(circuit, validWitness, b, curve, &opt)
			}, curve.String(), b.String())
		}
	}
}

func (assert *Assert) solvingSucceeded(circuit frontend.Circuit, validAssignment frontend.Circuit, b backend.ID, curve ecc.ID, opt *testingConfig) {
	// parse assignment
	validWitness, err := frontend.NewWitness(validAssignment, curve.ScalarField())
	assert.NoError(err, "can't parse valid assignment")

	checkError := func(err error) { assert.checkError(err, b, curve, validWitness, lazySchema(circuit)) }

	// 1- compile the circuit
	ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
	checkError(err)

	// must not error with big int test engine
	err = IsSolved(circuit, validAssignment, curve.ScalarField())
	checkError(err)

	err = ccs.IsSolved(validWitness, opt.solverOpts...)
	checkError(err)

}

func (assert *Assert) SolvingFailed(circuit frontend.Circuit, invalidWitness frontend.Circuit, opts ...TestingOption) {
	opt := assert.options(opts...)

	for _, curve := range opt.curves {
		for _, b := range opt.backends {
			curve := curve
			b := b
			assert.Run(func(assert *Assert) {
				assert.solvingFailed(circuit, invalidWitness, b, curve, &opt)
			}, curve.String(), b.String())
		}
	}
}

func lazySchema(circuit frontend.Circuit) func() *schema.Schema {
	return func() *schema.Schema {
		// we only parse the schema if we need to display the witness in json.
		s, err := schema.New(circuit, tVariable)
		if err != nil {
			panic("couldn't parse schema from circuit: " + err.Error())
		}
		return s
	}
}

func (assert *Assert) solvingFailed(circuit frontend.Circuit, invalidAssignment frontend.Circuit, b backend.ID, curve ecc.ID, opt *testingConfig) {
	// parse assignment
	invalidWitness, err := frontend.NewWitness(invalidAssignment, curve.ScalarField())
	assert.NoError(err, "can't parse invalid assignment")

	checkError := func(err error) { assert.checkError(err, b, curve, invalidWitness, lazySchema(circuit)) }
	mustError := func(err error) { assert.mustError(err, b, curve, invalidWitness, lazySchema(circuit)) }

	// 1- compile the circuit
	ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
	checkError(err)

	// must error with big int test engine
	err = IsSolved(circuit, invalidAssignment, curve.ScalarField())
	mustError(err)

	err = ccs.IsSolved(invalidWitness, opt.solverOpts...)
	mustError(err)

}

// Fuzz fuzzes the given circuit by instantiating "randomized" witnesses and cross checking
// execution result between constraint system solver and big.Int test execution engine
//
// note: this is experimental and will be more tightly integrated with go1.18 built-in fuzzing
func (assert *Assert) Fuzz(circuit frontend.Circuit, fuzzCount int, opts ...TestingOption) {
	opt := assert.options(opts...)

	// first we clone the circuit
	// then we parse the frontend.Variable and set them to a random value  or from our interesting pool
	// (% of allocations to be tuned)
	w := shallowClone(circuit)

	fillers := []filler{randomFiller, binaryFiller, seedFiller}

	for _, curve := range opt.curves {
		for _, b := range opt.backends {
			curve := curve
			b := b
			assert.Run(func(assert *Assert) {
				// this puts the compiled circuit in the cache
				// we do this here in case our fuzzWitness method mutates some references in the circuit
				// (like []frontend.Variable) before cleaning up
				_, err := assert.compile(circuit, curve, b, opt.compileOpts)
				assert.NoError(err)
				valid := 0
				// "fuzz" with zeros
				valid += assert.fuzzer(zeroFiller, circuit, w, b, curve, &opt)

				for i := 0; i < fuzzCount; i++ {
					for _, f := range fillers {
						valid += assert.fuzzer(f, circuit, w, b, curve, &opt)
					}
				}

			}, curve.String(), b.String())

		}
	}
}

func (assert *Assert) fuzzer(fuzzer filler, circuit, w frontend.Circuit, b backend.ID, curve ecc.ID, opt *testingConfig) int {
	// fuzz a witness
	fuzzer(w, curve)

	errVars := IsSolved(circuit, w, curve.ScalarField())
	errConsts := IsSolved(circuit, w, curve.ScalarField(), SetAllVariablesAsConstants())

	if (errVars == nil) != (errConsts == nil) {
		w, err := frontend.NewWitness(w, curve.ScalarField())
		if err != nil {
			panic(err)
		}
		s, err := frontend.NewSchema(circuit)
		if err != nil {
			panic(err)
		}
		bb, err := w.ToJSON(s)
		if err != nil {
			panic(err)
		}

		assert.Log("errVars", errVars)
		assert.Log("errConsts", errConsts)
		assert.Log("fuzzer witness", string(bb))
		assert.FailNow("solving circuit with values as constants vs non-constants mismatched result")
	}

	if errVars == nil && errConsts == nil {
		// valid witness
		assert.solvingSucceeded(circuit, w, b, curve, opt)
		return 1
	}

	// invalid witness
	assert.solvingFailed(circuit, w, b, curve, opt)
	return 0
}

func (assert *Assert) getCircuitAddr(circuit frontend.Circuit) (uintptr, error) {
	vCircuit := reflect.ValueOf(circuit)
	if vCircuit.Kind() != reflect.Ptr {
		return 0, errors.New("frontend.Circuit methods must be defined on pointer receiver")
	}
	return vCircuit.Pointer(), nil
}

// compile the given circuit for given curve and backend, if not already present in cache
func (assert *Assert) compile(circuit frontend.Circuit, curveID ecc.ID, backendID backend.ID, compileOpts []frontend.CompileOption) (constraint.ConstraintSystem, error) {
	addr, err := assert.getCircuitAddr(circuit)
	if err != nil {
		return nil, err
	}

	key := fmt.Sprintf("%d%d%s%d", curveID, backendID, reflect.TypeOf(circuit).String(), addr)

	// check if we already compiled it
	if ccs, ok := assert.compiled[key]; ok {
		return ccs, nil
	}

	var newBuilder frontend.NewBuilder

	switch backendID {
	case backend.GROTH16:
		newBuilder = r1cs.NewBuilder
	case backend.PLONK:
		newBuilder = scs.NewBuilder
	case backend.PLONKFRI:
		newBuilder = scs.NewBuilder
	default:
		panic("not implemented")
	}

	// else compile it and ensure it is deterministic
	ccs, err := frontend.Compile(curveID.ScalarField(), newBuilder, circuit, compileOpts...)
	if err != nil {
		return nil, err
	}

	_ccs, err := frontend.Compile(curveID.ScalarField(), newBuilder, circuit, compileOpts...)
	if err != nil {
		return nil, fmt.Errorf("%w: %v", ErrCompilationNotDeterministic, err)
	}

	if !reflect.DeepEqual(ccs, _ccs) {
		return nil, ErrCompilationNotDeterministic
	}

	// // add the compiled circuit to the cache
	assert.compiled[key] = ccs

	return ccs, nil
}

// default options
func (assert *Assert) options(opts ...TestingOption) testingConfig {
	// apply options
	opt := testingConfig{
		witnessSerialization: true,
		backends:             backend.Implemented(),
		curves:               gnark.Curves(),
		fuzzing:              true,
	}
	for _, option := range opts {
		err := option(&opt)
		assert.NoError(err, "parsing TestingOption")
	}

	if testing.Short() {
		// if curves are all there, we just test with bn254
		if reflect.DeepEqual(opt.curves, gnark.Curves()) {
			opt.curves = []ecc.ID{ecc.BN254}
		}
	}
	return opt
}

// ensure the error is set, else fails the test
func (assert *Assert) mustError(err error, backendID backend.ID, curve ecc.ID, w witness.Witness, lazyS func() *schema.Schema) {
	if err != nil {
		return
	}
	var json string
	bjson, err := w.ToJSON(lazyS())
	if err != nil {
		json = err.Error()
	} else {
		json = string(bjson)
	}

	e := fmt.Errorf("did not error (but should have) %s(%s)\nwitness:%s", backendID.String(), curve.String(), json)
	assert.FailNow(e.Error())
}

// ensure the error is nil, else fails the test
func (assert *Assert) checkError(err error, backendID backend.ID, curve ecc.ID, w witness.Witness, lazyS func() *schema.Schema) {
	if err == nil {
		return
	}

	var json string
	e := fmt.Errorf("%s(%s): %w", backendID.String(), curve.String(), err)

	bjson, err := w.ToJSON(lazyS())
	if err != nil {
		json = err.Error()
	} else {
		json = string(bjson)
	}
	e = fmt.Errorf("%w\nwitness:%s", e, json)

	assert.FailNow(e.Error())
}

func (assert *Assert) marshalWitness(w witness.Witness, curveID ecc.ID, publicOnly bool) {
	// serialize the vector to binary
	var err error
	if publicOnly {
		w, err = w.Public()
		assert.NoError(err)
	}
	data, err := w.MarshalBinary()
	assert.NoError(err)

	// re-read
	witness, err := witness.New(curveID.ScalarField())
	assert.NoError(err)
	err = witness.UnmarshalBinary(data)
	assert.NoError(err)

	witnessMatch := reflect.DeepEqual(w, witness)

	assert.True(witnessMatch, "round trip marshaling failed")
}

func (assert *Assert) marshalWitnessJSON(w witness.Witness, s *schema.Schema, curveID ecc.ID, publicOnly bool) {
	var err error
	if publicOnly {
		w, err = w.Public()
		assert.NoError(err)
	}

	// serialize the vector to binary
	data, err := w.ToJSON(s)
	assert.NoError(err)

	// re-read
	witness, err := witness.New(curveID.ScalarField())
	assert.NoError(err)
	err = witness.FromJSON(s, data)
	assert.NoError(err)

	witnessMatch := reflect.DeepEqual(w, witness)
	assert.True(witnessMatch, "round trip marshaling failed")
}
