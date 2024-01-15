package test

import (
	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark/backend"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/backend/plonk"
	"github.com/consensys/gnark/backend/plonkfri"
	"github.com/consensys/gnark/backend/witness"
	"github.com/consensys/gnark/constraint"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/schema"
	"github.com/consensys/gnark/test/unsafekzg"
)

// CheckCircuit performs a series of check on the provided circuit.
//
//	go test -short                  --> testEngineChecks
//	go test                         --> testEngineChecks  + constraintSolverChecks
//	go test -tags=prover_checks     --> ... + proverChecks
//	go test -tags=release_checks    --> ... + releaseChecks (solidity, serialization, ...)
//
// Depending on the above flags, the following checks are performed:
//   - the circuit compiles
//   - the circuit can be solved with the test engine
//   - the circuit can be solved with the constraint system solver
//   - the circuit can be solved with the prover
//   - the circuit can be verified with the verifier
//   - the circuit can be verified with gnark-solidity-checker
//   - the circuit, witness, proving and verifying keys can be serialized and deserialized
func (assert *Assert) CheckCircuit(circuit frontend.Circuit, opts ...TestingOption) {
	// get the testing configuration
	opt := assert.options(opts...)

	// for each {curve, backend} tuple
	for _, curve := range opt.curves {
		curve := curve

		// run in sub-test to contextualize with curve
		assert.Run(func(assert *Assert) {

			// parse valid / invalid assignments
			var invalidWitnesses, validWitnesses []_witness
			for _, a := range opt.validAssignments {
				w := assert.parseAssignment(circuit, a, curve, opt.checkSerialization)
				validWitnesses = append(validWitnesses, w)

				// check that the assignment is valid with the test engine
				if !opt.skipTestEngine {
					err := IsSolved(circuit, w.assignment, curve.ScalarField())
					assert.noError(err, &w)
				}
			}

			for _, a := range opt.invalidAssignments {
				w := assert.parseAssignment(circuit, a, curve, opt.checkSerialization)
				invalidWitnesses = append(invalidWitnesses, w)

				// check that the assignment is invalid with the test engine
				if !opt.skipTestEngine {
					err := IsSolved(circuit, w.assignment, curve.ScalarField())
					assert.error(err, &w)
				}
			}

			// for each backend; compile, prove/verify or solve, check serialization if needed.
			for _, b := range opt.backends {
				b := b

				// run in sub-test to contextualize with backend
				assert.Run(func(assert *Assert) {

					// 1- check that the circuit compiles
					ccs, err := assert.compile(circuit, curve, b, opt.compileOpts)
					assert.noError(err, nil)

					// TODO @gbotrel check serialization round trip with constraint system.

					// 2- if we are not running the full prover;
					// we need to run the solver on the constraint system only
					if !opt.checkProver {
						for _, w := range invalidWitnesses {
							w := w
							assert.Run(func(assert *Assert) {
								_, err = ccs.Solve(w.full, opt.solverOpts...)
								assert.error(err, &w)
							}, "invalid_witness")
						}

						for _, w := range validWitnesses {
							w := w
							assert.Run(func(assert *Assert) {
								_, err = ccs.Solve(w.full, opt.solverOpts...)
								assert.noError(err, &w)
							}, "valid_witness")
						}

						return
					}

					// we need to run the setup, prove and verify and check serialization
					assert.t.Parallel()

					var concreteBackend tBackend

					switch b {
					case backend.GROTH16:
						concreteBackend = _groth16
					case backend.PLONK:
						concreteBackend = _plonk
					case backend.PLONKFRI:
						concreteBackend = _plonkfri
					default:
						panic("backend not implemented")
					}

					// proof system setup.
					pk, vk, pkBuilder, vkBuilder, proofBuilder, err := concreteBackend.setup(ccs, curve)
					assert.noError(err, nil)

					// for each valid witness, run the prover and verifier
					for _, w := range validWitnesses {
						w := w
						assert.Run(func(assert *Assert) {
							checkSolidity := opt.checkSolidity && curve == ecc.BN254
							proof, err := concreteBackend.prove(ccs, pk, w.full, opt.proverOpts...)
							assert.noError(err, &w)

							err = concreteBackend.verify(proof, vk, w.public, opt.verifierOpts...)
							assert.noError(err, &w)

							if checkSolidity {
								// check that the proof can be verified by gnark-solidity-checker
								if _vk, ok := vk.(verifyingKey); ok {
									assert.Run(func(assert *Assert) {
										assert.solidityVerification(b, _vk, proof, w.public)
									}, "solidity")
								}
							}

							// check proof serialization
							assert.roundTripCheck(proof, proofBuilder, "proof")
						}, "valid_witness")
					}

					// for each invalid witness, run the prover only, it should fail.
					for _, w := range invalidWitnesses {
						w := w
						assert.Run(func(assert *Assert) {
							_, err := concreteBackend.prove(ccs, pk, w.full, opt.proverOpts...)
							assert.error(err, &w)
						}, "invalid_witness")
					}

					// check serialization of proving and verifying keys
					if opt.checkSerialization && ccs.GetNbConstraints() <= serializationThreshold && (curve == ecc.BN254 || curve == ecc.BLS12_381) {
						assert.roundTripCheck(pk, pkBuilder, "proving_key")
						assert.roundTripCheck(vk, vkBuilder, "verifying_key")
					}

				}, b.String())
			}

		}, curve.String())
	}

	// TODO @gbotrel revisit this.
	if false && opt.fuzzing {
		// TODO may not be the right place, but ensures all our tests call these minimal tests
		// (like filling a witness with zeroes, or binary values, ...)
		assert.Run(func(assert *Assert) {
			assert.Fuzz(circuit, 5, opts...)
		}, "fuzz")
	}
}

type _witness struct {
	full       witness.Witness
	public     witness.Witness
	assignment frontend.Circuit
}

func (assert *Assert) parseAssignment(circuit frontend.Circuit, assignment frontend.Circuit, curve ecc.ID, checkSerialization bool) _witness {
	if assignment == nil {
		return _witness{}
	}
	full, err := frontend.NewWitness(assignment, curve.ScalarField())
	assert.NoError(err, "can't parse assignment into full witness")

	public, err := frontend.NewWitness(assignment, curve.ScalarField(), frontend.PublicOnly())
	assert.NoError(err, "can't parse assignment into public witness")

	if checkSerialization {
		witnessBuilder := func() any {
			w, err := witness.New(curve.ScalarField())
			if err != nil {
				panic(err)
			}
			return w
		}
		assert.roundTripCheck(full, witnessBuilder, "witness", "full")
		assert.roundTripCheck(public, witnessBuilder, "witness", "public")

		// count number of element in witness.
		// if too many, we don't do JSON serialization.
		s, err := schema.Walk(assignment, tVariable, nil)
		assert.NoError(err)

		if s.Public+s.Secret <= serializationThreshold {
			assert.Run(func(assert *Assert) {
				s := lazySchema(circuit)()
				assert.marshalWitnessJSON(full, s, curve, false)
			}, curve.String(), "marshal/json")
			assert.Run(func(assert *Assert) {
				s := lazySchema(circuit)()
				assert.marshalWitnessJSON(public, s, curve, true)
			}, curve.String(), "marshal-public/json")
		}
	}

	return _witness{full: full, public: public, assignment: assignment}
}

type fnSetup func(ccs constraint.ConstraintSystem, curve ecc.ID) (
	pk, vk any,
	pkBuilder, vkBuilder, proofBuilder func() any,
	err error)
type fnProve func(ccs constraint.ConstraintSystem, pk any, fullWitness witness.Witness, opts ...backend.ProverOption) (proof any, err error)
type fnVerify func(proof, vk any, publicWitness witness.Witness, opts ...backend.VerifierOption) error

// tBackend abstracts the backend implementation in the test package.
type tBackend struct {
	setup  fnSetup
	prove  fnProve
	verify fnVerify
}

var (
	_groth16 = tBackend{
		setup: func(ccs constraint.ConstraintSystem, curve ecc.ID) (
			pk, vk any,
			pkBuilder, vkBuilder, proofBuilder func() any,
			err error) {
			pk, vk, err = groth16.Setup(ccs)
			return pk, vk, func() any { return groth16.NewProvingKey(curve) }, func() any { return groth16.NewVerifyingKey(curve) }, func() any { return groth16.NewProof(curve) }, err
		},
		prove: func(ccs constraint.ConstraintSystem, pk any, fullWitness witness.Witness, opts ...backend.ProverOption) (proof any, err error) {
			return groth16.Prove(ccs, pk.(groth16.ProvingKey), fullWitness, opts...)
		},
		verify: func(proof, vk any, publicWitness witness.Witness, opts ...backend.VerifierOption) error {
			return groth16.Verify(proof.(groth16.Proof), vk.(groth16.VerifyingKey), publicWitness, opts...)
		},
	}

	_plonk = tBackend{
		setup: func(ccs constraint.ConstraintSystem, curve ecc.ID) (
			pk, vk any,
			pkBuilder, vkBuilder, proofBuilder func() any,
			err error) {
			srs, srsLagrange, err := unsafekzg.NewSRS(ccs)
			if err != nil {
				return nil, nil, nil, nil, nil, err
			}
			pk, vk, err = plonk.Setup(ccs, srs, srsLagrange)
			return pk, vk, func() any { return plonk.NewProvingKey(curve) }, func() any { return plonk.NewVerifyingKey(curve) }, func() any { return plonk.NewProof(curve) }, err
		},
		prove: func(ccs constraint.ConstraintSystem, pk any, fullWitness witness.Witness, opts ...backend.ProverOption) (proof any, err error) {
			return plonk.Prove(ccs, pk.(plonk.ProvingKey), fullWitness, opts...)
		},
		verify: func(proof, vk any, publicWitness witness.Witness, opts ...backend.VerifierOption) error {
			return plonk.Verify(proof.(plonk.Proof), vk.(plonk.VerifyingKey), publicWitness, opts...)
		},
	}

	_plonkfri = tBackend{
		setup: func(ccs constraint.ConstraintSystem, curve ecc.ID) (
			pk, vk any,
			pkBuilder, vkBuilder, proofBuilder func() any,
			err error) {
			pk, vk, err = plonkfri.Setup(ccs)
			return pk, vk, func() any { return nil }, func() any { return nil }, func() any { return nil }, err
		},
		prove: func(ccs constraint.ConstraintSystem, pk any, fullWitness witness.Witness, opts ...backend.ProverOption) (proof any, err error) {
			return plonkfri.Prove(ccs, pk.(plonkfri.ProvingKey), fullWitness, opts...)
		},
		verify: func(proof, vk any, publicWitness witness.Witness, opts ...backend.VerifierOption) error {
			return plonkfri.Verify(proof, vk.(plonkfri.VerifyingKey), publicWitness, opts...)
		},
	}
)
