package test

import (
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/constraint/solver"
	"github.com/consensys/gnark/frontend"
)

// TestingOption defines option for altering the behavior of Assert methods.
// See the descriptions of functions returning instances of this type for
// particular options.
type TestingOption func(*testingConfig) error

type testingConfig struct {
	profile
	solverOpts   []solver.Option
	proverOpts   []backend.ProverOption
	verifierOpts []backend.VerifierOption
	compileOpts  []frontend.CompileOption

	validAssignments   []frontend.Circuit
	invalidAssignments []frontend.Circuit
}

// default options
func (assert *Assert) options(opts ...TestingOption) testingConfig {
	var opt testingConfig

	// default options;
	// go test -short 					--> testEngineOnly
	// go test 							--> constraintOnlyProfile
	// go test -tags=prover_checks 		--> proverOnlyProfile
	// go test -tags=release_checks 	--> releaseProfile

	if releaseTestFlag {
		opt.profile = releaseChecks
	} else if proverTestFlag {
		opt.profile = proverChecks
	} else if testing.Short() {
		opt.profile = testEngineChecks
	} else {
		opt.profile = constraintSolverChecks
	}

	// apply user provided options.
	for _, option := range opts {
		err := option(&opt)
		assert.NoError(err, "parsing TestingOption")
	}

	return opt
}

// WithValidAssignment is a testing option which adds a valid assignment
func WithValidAssignment(validAssignment frontend.Circuit) TestingOption {
	return func(opt *testingConfig) error {
		opt.validAssignments = append(opt.validAssignments, validAssignment)
		return nil
	}
}

// WithInvalidAssignment is a testing option which adds an invalid assignment
func WithInvalidAssignment(invalidAssignment frontend.Circuit) TestingOption {
	return func(opt *testingConfig) error {
		opt.invalidAssignments = append(opt.invalidAssignments, invalidAssignment)
		return nil
	}
}

// WithBackends is testing option which restricts the backends the assertions are
// run. When not given, runs on all implemented backends.
func WithBackends(b backend.ID, backends ...backend.ID) TestingOption {
	return func(opt *testingConfig) error {
		opt.backends = []backend.ID{b}
		opt.backends = append(opt.backends, backends...)
		return nil
	}
}

// WithCurves is a testing option which restricts the curves the assertions are
// run. When not given, runs on all implemented curves.
func WithCurves(c ecc.ID, curves ...ecc.ID) TestingOption {
	return func(opt *testingConfig) error {
		opt.curves = []ecc.ID{c}
		opt.curves = append(opt.curves, curves...)
		return nil
	}
}

// NoSerializationChecks is a testing option which disables serialization checks,
// even when the build tag "release_checks" is set.
func NoSerializationChecks() TestingOption {
	return func(opt *testingConfig) error {
		opt.checkSerialization = false
		return nil
	}
}

// NoFuzzing is a testing option which disables fuzzing tests,
// even when the build tag "release_checks" is set.
func NoFuzzing() TestingOption {
	return func(opt *testingConfig) error {
		opt.fuzzing = false
		return nil
	}
}

// NoProverChecks is a testing option which disables prover checks,
// even when the build tag "prover_checks" or "release_checks" are set.
func NoProverChecks() TestingOption {
	return func(opt *testingConfig) error {
		opt.checkProver = false
		return nil
	}
}

// NoTestEngine is a testing option which disables test engine checks
func NoTestEngine() TestingOption {
	return func(opt *testingConfig) error {
		opt.skipTestEngine = true
		return nil
	}
}

// NoSolidityChecks is a testing option which disables solidity checks,
// even when the build tags "solccheck" and "release_checks" are set.
//
// When the tags are set; this requires gnark-solidity-checker to be installed, which in turns
// requires solc and abigen to be reachable in the PATH.
//
// See https://github.com/ConsenSys/gnark-solidity-checker for more details.
func NoSolidityChecks() TestingOption {
	return func(opt *testingConfig) error {
		opt.checkSolidity = false
		return nil
	}
}

// WithProverOpts is a testing option which uses the given proverOpts when
// calling backend.Prover, backend.ReadAndProve and backend.IsSolved methods in
// assertions.
func WithProverOpts(proverOpts ...backend.ProverOption) TestingOption {
	return func(opt *testingConfig) error {
		opt.proverOpts = proverOpts
		return nil
	}
}

// WithSolverOpts is a testing option which uses the given solverOpts when
// calling constraint system solver.
func WithSolverOpts(solverOpts ...solver.Option) TestingOption {
	return func(opt *testingConfig) error {
		opt.proverOpts = append(opt.proverOpts, backend.WithSolverOptions(solverOpts...))
		opt.solverOpts = solverOpts
		return nil
	}
}

// WithCompileOpts is a testing option which uses the given compileOpts when
// calling frontend.Compile in assertions.
func WithCompileOpts(compileOpts ...frontend.CompileOption) TestingOption {
	return func(opt *testingConfig) error {
		opt.compileOpts = compileOpts
		return nil
	}
}

// WithVerifierOpts is a testing option which uses the given verifierOpts when
// calling backend.Verify method.
func WithVerifierOpts(verifierOpts ...backend.VerifierOption) TestingOption {
	return func(tc *testingConfig) error {
		tc.verifierOpts = append(tc.verifierOpts, verifierOpts...)
		return nil
	}
}
