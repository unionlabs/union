package test

import (
	"crypto/rand"
	"math/big"
	mrand "math/rand"
	"reflect"
	"time"

	"github.com/consensys/gnark"
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
)

var seedCorpus []*big.Int

func init() {
	seedCorpus = make([]*big.Int, 0, 300)

	// small values, including bits
	for i := -5; i <= 5; i++ {
		seedCorpus = append(seedCorpus, big.NewInt(int64(i)))
	}

	// moduli
	for _, curve := range gnark.Curves() {
		fp := curve.BaseField()
		fr := curve.ScalarField()
		seedCorpus = append(seedCorpus, fp)
		seedCorpus = append(seedCorpus, fr)

		var bi big.Int
		for i := -3; i <= 3; i++ {
			bi.SetInt64(int64(i))
			var fp1, fr1 big.Int
			fp1.Add(fp, &bi)
			fr1.Add(fr, &bi)

			seedCorpus = append(seedCorpus, &fp1)
			seedCorpus = append(seedCorpus, &fr1)
		}
	}

	// powers of 2
	bi := big.NewInt(1)
	bi.Lsh(bi, 32)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 64)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 254)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 255)
	seedCorpus = append(seedCorpus, bi)

	bi = big.NewInt(1)
	bi.Lsh(bi, 256)
	seedCorpus = append(seedCorpus, bi)

}

type filler func(frontend.Circuit, ecc.ID)

func zeroFiller(w frontend.Circuit, curve ecc.ID) {
	fill(w, func() interface{} {
		return 0
	})
}

func binaryFiller(w frontend.Circuit, curve ecc.ID) {
	mrand := mrand.New(mrand.NewSource(time.Now().Unix())) //#nosec G404 weak rng is fine here

	fill(w, func() interface{} {
		return int(mrand.Uint32() % 2) //#nosec G404 weak rng is fine here
	})
}

func seedFiller(w frontend.Circuit, curve ecc.ID) {

	mrand := mrand.New(mrand.NewSource(time.Now().Unix())) //#nosec G404 weak rng is fine here

	m := curve.ScalarField()

	fill(w, func() interface{} {
		i := int(mrand.Uint32() % uint32(len(seedCorpus))) //#nosec G404 weak rng is fine here
		r := new(big.Int).Set(seedCorpus[i])
		return r.Mod(r, m)
	})
}

func randomFiller(w frontend.Circuit, curve ecc.ID) {

	r := mrand.New(mrand.NewSource(time.Now().Unix())) //#nosec G404 weak rng is fine here
	m := curve.ScalarField()

	fill(w, func() interface{} {
		i := int(mrand.Uint32() % uint32(len(seedCorpus)*2)) //#nosec G404 weak rng is fine here
		if i >= len(seedCorpus) {
			b1, _ := rand.Int(r, m) //#nosec G404 weak rng is fine here
			return b1
		}
		r := new(big.Int).Set(seedCorpus[i])
		return r.Mod(r, m)
	})
}

func fill(w frontend.Circuit, nextValue func() interface{}) {
	setHandler := func(f schema.LeafInfo, tInput reflect.Value) error {
		v := nextValue()
		tInput.Set(reflect.ValueOf((v)))
		return nil
	}
	// this can't error.
	// TODO @gbotrel it might error with .Walk?
	_, _ = schema.Walk(w, tVariable, setHandler)
}

var tVariable reflect.Type

func init() {
	tVariable = reflect.ValueOf(struct{ A frontend.Variable }{}).FieldByName("A").Type()
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

func (assert *Assert) solvingSucceeded(circuit frontend.Circuit, validAssignment frontend.Circuit, b backend.ID, curve ecc.ID, opt *testingConfig) {
	// parse assignment
	w := assert.parseAssignment(circuit, validAssignment, curve, opt.checkSerialization)

	checkError := func(err error) { assert.noError(err, &w) }

	// 1- compile the circuit
	ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
	checkError(err)

	// must not error with big int test engine
	err = IsSolved(circuit, validAssignment, curve.ScalarField())
	checkError(err)

	err = ccs.IsSolved(w.full, opt.solverOpts...)
	checkError(err)

}

func (assert *Assert) solvingFailed(circuit frontend.Circuit, invalidAssignment frontend.Circuit, b backend.ID, curve ecc.ID, opt *testingConfig) {
	// parse assignment
	w := assert.parseAssignment(circuit, invalidAssignment, curve, opt.checkSerialization)

	checkError := func(err error) { assert.noError(err, &w) }
	mustError := func(err error) { assert.error(err, &w) }

	// 1- compile the circuit
	ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
	checkError(err)

	// must error with big int test engine
	err = IsSolved(circuit, invalidAssignment, curve.ScalarField())
	mustError(err)

	err = ccs.IsSolved(w.full, opt.solverOpts...)
	mustError(err)

}
