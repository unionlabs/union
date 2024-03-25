// Copyright 2020 ConsenSys AG
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package backend implements Zero Knowledge Proof systems: it consumes circuit compiled with gnark/frontend.
package backend

import (
	"crypto/sha256"
	"hash"

	"github.com/consensys/gnark/constraint/solver"
)

// ID represent a unique ID for a proving scheme
type ID uint16

const (
	UNKNOWN ID = iota
	GROTH16
	PLONK
)

// Implemented return the list of proof systems implemented in gnark
func Implemented() []ID {
	return []ID{GROTH16, PLONK}
}

// String returns the string representation of a proof system
func (id ID) String() string {
	switch id {
	case GROTH16:
		return "groth16"
	case PLONK:
		return "plonk"
	default:
		return "unknown"
	}
}

// ProverOption defines option for altering the behavior of the prover in
// Prove, ReadAndProve and IsSolved methods. See the descriptions of functions
// returning instances of this type for implemented options.
type ProverOption func(*ProverConfig) error

// ProverConfig is the configuration for the prover with the options applied.
type ProverConfig struct {
	SolverOpts     []solver.Option
	HashToFieldFn  hash.Hash
	ChallengeHash  hash.Hash
	KZGFoldingHash hash.Hash
	Accelerator    string
}

// NewProverConfig returns a default ProverConfig with given prover options opts
// applied.
func NewProverConfig(opts ...ProverOption) (ProverConfig, error) {
	opt := ProverConfig{
		// we cannot initialize HashToFieldFn here as we use different domain
		// separation tags for PLONK and Groth16
		ChallengeHash:  sha256.New(),
		KZGFoldingHash: sha256.New(),
	}
	for _, option := range opts {
		if err := option(&opt); err != nil {
			return ProverConfig{}, err
		}
	}
	return opt, nil
}

// WithSolverOptions specifies the constraint system solver options.
func WithSolverOptions(solverOpts ...solver.Option) ProverOption {
	return func(opt *ProverConfig) error {
		opt.SolverOpts = solverOpts
		return nil
	}
}

// WithProverHashToFieldFunction changes the hash function used for hashing
// bytes to field. If not set then the default hash function based on RFC 9380
// is used. Used mainly for compatibility between different systems and
// efficient recursion.
func WithProverHashToFieldFunction(hFunc hash.Hash) ProverOption {
	return func(cfg *ProverConfig) error {
		cfg.HashToFieldFn = hFunc
		return nil
	}
}

// WithProverChallengeHashFunction sets the hash function used for computing
// non-interactive challenges in Fiat-Shamir heuristic. If not set then by
// default SHA2-256 is used. Used mainly for compatibility between different
// systems and efficient recursion.
func WithProverChallengeHashFunction(hFunc hash.Hash) ProverOption {
	return func(pc *ProverConfig) error {
		pc.ChallengeHash = hFunc
		return nil
	}
}

// WithProverKZGFoldingHashFunction sets the hash function used for computing
// the challenge when folding the KZG opening proofs. If not set then by default
// SHA2-256 is used. Used mainly for compatibility between different systems and
// efficient recursion.
func WithProverKZGFoldingHashFunction(hFunc hash.Hash) ProverOption {
	return func(pc *ProverConfig) error {
		pc.KZGFoldingHash = hFunc
		return nil
	}
}

// WithIcicleAcceleration requests to use [ICICLE] GPU proving backend for the
// prover. This option requires that the program is compiled with `icicle` build
// tag and the ICICLE dependencies are properly installed. See [ICICLE] for
// installation description.
//
// [ICICLE]: https://github.com/ingonyama-zk/icicle
func WithIcicleAcceleration() ProverOption {
	return func(pc *ProverConfig) error {
		pc.Accelerator = "icicle"
		return nil
	}
}

// VerifierOption defines option for altering the behavior of the verifier. See
// the descriptions of functions returning instances of this type for
// implemented options.
type VerifierOption func(*VerifierConfig) error

// VerifierConfig is the configuration for the verifier with the options applied.
type VerifierConfig struct {
	HashToFieldFn  hash.Hash
	ChallengeHash  hash.Hash
	KZGFoldingHash hash.Hash
}

// NewVerifierConfig returns a default [VerifierConfig] with given verifier
// options applied.
func NewVerifierConfig(opts ...VerifierOption) (VerifierConfig, error) {
	opt := VerifierConfig{
		// we cannot initialize HashToFieldFn here as we use different domain
		// separation tags for PLONK and Groth16
		ChallengeHash:  sha256.New(),
		KZGFoldingHash: sha256.New(),
	}
	for _, option := range opts {
		if err := option(&opt); err != nil {
			return VerifierConfig{}, err
		}
	}
	return opt, nil
}

// WithVerifierHashToFieldFunction changes the hash function used for hashing
// bytes to field. If not set then the default hash function based on RFC 9380
// is used. Used mainly for compatibility between different systems and
// efficient recursion.
func WithVerifierHashToFieldFunction(hFunc hash.Hash) VerifierOption {
	return func(cfg *VerifierConfig) error {
		cfg.HashToFieldFn = hFunc
		return nil
	}
}

// WithVerifierChallengeHashFunction sets the hash function used for computing
// non-interactive challenges in Fiat-Shamir heuristic. If not set then by
// default SHA2-256 is used. Used mainly for compatibility between different
// systems and efficient recursion.
func WithVerifierChallengeHashFunction(hFunc hash.Hash) VerifierOption {
	return func(pc *VerifierConfig) error {
		pc.ChallengeHash = hFunc
		return nil
	}
}

// WithVerifierKZGFoldingHashFunction sets the hash function used for computing
// the challenge when folding the KZG opening proofs. If not set then by default
// SHA2-256 is used. Used mainly for compatibility between different systems and
// efficient recursion.
func WithVerifierKZGFoldingHashFunction(hFunc hash.Hash) VerifierOption {
	return func(pc *VerifierConfig) error {
		pc.KZGFoldingHash = hFunc
		return nil
	}
}
