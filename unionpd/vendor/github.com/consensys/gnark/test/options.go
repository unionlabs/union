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
	backends             []backend.ID
	curves               []ecc.ID
	witnessSerialization bool
	solverOpts           []solver.Option
	proverOpts           []backend.ProverOption
	compileOpts          []frontend.CompileOption
	fuzzing              bool
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

// NoSerialization is a testing option which disables witness serialization tests
// in assertions.
func NoSerialization() TestingOption {
	return func(opt *testingConfig) error {
		opt.witnessSerialization = false
		return nil
	}
}

// NoFuzzing is a testing option which disables fuzzing tests in assertions.
func NoFuzzing() TestingOption {
	return func(opt *testingConfig) error {
		opt.fuzzing = false
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
