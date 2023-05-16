<a name="v0.7.0"></a>
## [v0.7.0] - 2022-03-25

### Build
- go.mod: go version upgrade 1.16 --> go1.17
- update to gnark-crpto v0.7.0

### Feat
- adds gnark logger. closes [#202](https://github.com/consensys/gnark/issues/202)
- added internal/stats package: measure number of constraints of circuit snippets for regression
- adds std/math/bits/ToNAF ToBinary ToTernary

### Fix
- enables recursive hints solving [#293](https://github.com/consensys/gnark/issues/293) and
- move init() behind sync.Once. remove verbose option in stats binary
- fixes [#266](https://github.com/consensys/gnark/issues/266) by adding constant path in Lookup2 and Select
- incorrect handling of nbBits == 1 in api.ToBinary


### Perf
- restored frontend.WithCapacity option...
- **plonk:** IsConstant -> ConstantValue
- **sw:** no need for Lookup2 in constScalarMul
- remove offset shifts in plonk compile
- remove post-compile offset id in R1CS builder

### Refactor
- `frontend.Compile` now takes a builder instead of backendID as parameter
- `std/signature/eddsa` `Verify` api now takes explicit hash and curve objects
- make nboutputs of a hint explicit at compile time
- std/pairing have more consistent apis
- remove StaticHint wrapper, log duplicate hints ([#289](https://github.com/consensys/gnark/issues/289))
- backend.WithOutput -> backend.WithCircuitLogger
- remove all internal circuits from stats, keep important snippets only
- frontend: split compiler, api and builder interface into interfaces
- remove IsBoolean from R1CS variables
- moved internal/compiled to frontend/compiled


### Pull Requests
- Merge pull request [#295](https://github.com/consensys/gnark/issues/295) from ConsenSys/fix/test-println
- Merge pull request [#294](https://github.com/consensys/gnark/issues/294) from ConsenSys/fix/recursivehhints
- Merge pull request [#291](https://github.com/consensys/gnark/issues/291) from ConsenSys/refactor/std/pairing
- Merge pull request [#281](https://github.com/consensys/gnark/issues/281) from ConsenSys/feat/logger
- Merge pull request [#280](https://github.com/consensys/gnark/issues/280) from ConsenSys/simplify-r1cs-compile
- Merge pull request [#279](https://github.com/consensys/gnark/issues/279) from ConsenSys/feat/statistics
- Merge pull request [#276](https://github.com/consensys/gnark/issues/276) from ConsenSys/feat-math-bits
- Merge pull request [#278](https://github.com/consensys/gnark/issues/278) from ConsenSys/perf-constant-lookup2
- Merge pull request [#272](https://github.com/consensys/gnark/issues/272) from ConsenSys/refactor-hint
- Merge pull request [#275](https://github.com/consensys/gnark/issues/275) from ConsenSys/refactor-compiler-builder
- Merge pull request [#271](https://github.com/consensys/gnark/issues/271) from ConsenSys/refactor-compiled
- Merge pull request [#267](https://github.com/consensys/gnark/issues/267) from ConsenSys/perf/tEd-add
- Merge pull request [#265](https://github.com/consensys/gnark/issues/265) from ConsenSys/perf/SW-constScalarMul




<a name="v0.6.4"></a>

## [v0.6.4] - 2022-02-15

### Build

- update to gnark-crpto v0.6.1

### Feat

- Constraint system solvers (Groth16 and PlonK) now run in parallel

### Fix

- `api.DivUnchecked` with PlonK between 2 constants was incorrect

### Perf

- **EdDSA:** `std/algebra/twistededwards` takes ~2K less constraints (Groth16). Bandersnatch benefits from same improvments.


### Pull Requests

- Merge pull request [#259](https://github.com/consensys/gnark/issues/259) from ConsenSys/perf-parallel-solver
- Merge pull request [#261](https://github.com/consensys/gnark/issues/261) from ConsenSys/feat/kzg_updated
- Merge pull request [#257](https://github.com/consensys/gnark/issues/257) from ConsenSys/perf/EdDSA
- Merge pull request [#253](https://github.com/consensys/gnark/issues/253) from ConsenSys/feat/fft_cosets

<a name="v0.6.3"></a>

## [v0.6.3] - 2022-02-13

### Feat

- MiMC changes: api doesn't take a "seed" parameter. MiMC impl matches Ethereum one. 

### Fix

- fixes [#255](https://github.com/consensys/gnark/issues/255) variable visibility inheritance regression
- counter was set with PLONK backend ID in R1CS
- R1CS Solver was incorrectly calling a "MulByCoeff" instead of "DivByCoeff" (no impact, coeff was always 1 or -1)
- SparseR1CS cbor unmarshal failed [#247](https://github.com/consensys/gnark/issues/247) for compiled.Term


### Pull Requests

- Merge pull request [#256](https://github.com/consensys/gnark/issues/256) from ConsenSys/fix-bug-compile-visibility
- Merge pull request [#249](https://github.com/consensys/gnark/issues/249) from ConsenSys/perf-ccs-hint
- Merge pull request [#248](https://github.com/consensys/gnark/issues/248) from ConsenSys/perf-ccs-solver
- Merge pull request [#247](https://github.com/consensys/gnark/issues/247) from ConsenSys/fix/plonk_cbor


<a name="v0.6.1"></a>

## [v0.6.1] - 2022-01-28

### Build

- go version dependency bumped from 1.16 to 1.17

### Feat

- added witness.MarshalJSON and witness.MarshalBinary
- added `ccs.GetSchema()` - the schema of a circuit is required for witness json (de)serialization
- added `ccs.GetConstraints()` - returns a list of human-readable constraints
- added `ccs.IsSolved()` - moved from groth16 / plonk to the CompiledConstraintSystem interface
- added `witness.Public()` to return Public part of the witness
- addition of `Cmp` in the circuit API

### Refactor

- compiled.Visbility -> schema.Visibiility
- witness.WriteSequence -> schema.WriteSequence
- killed `ReadAndProve` and `ReadAndVerify` (plonk)
- killed `ReadAndProve` and `ReadAndVerify` (groth16)
- remove embbed struct tag for frontend.Variable fields

### Docs

- **backend:** unify documentation for options
- **frontend:** unify docs for options
- **test:** unify documentation for options

### Pull Requests

- Merge pull request [#244](https://github.com/consensys/gnark/issues/244) from ConsenSys/plonk-human-readable
- Merge pull request [#237](https://github.com/consensys/gnark/issues/237) from ConsenSys/ccs-get-constraints
- Merge pull request [#233](https://github.com/consensys/gnark/issues/233) from ConsenSys/feat/api_cmp
- Merge pull request [#235](https://github.com/consensys/gnark/issues/235) from ConsenSys/witness-public-api
- Merge pull request [#232](https://github.com/consensys/gnark/issues/232) from ConsenSys/cleanup-231-group-options
- Merge pull request [#230](https://github.com/consensys/gnark/issues/230) from ConsenSys/ccs-schema
- Merge pull request [#229](https://github.com/consensys/gnark/issues/229) from ConsenSys/ccs-issolved-api
- Merge pull request [#228](https://github.com/consensys/gnark/issues/228) from ConsenSys/witness-json
- Merge pull request [#226](https://github.com/consensys/gnark/issues/226) from ConsenSys/feat-circuit-schema
- Merge pull request [#227](https://github.com/consensys/gnark/issues/227) from ConsenSys/build-update-go1.17
- Merge pull request [#222](https://github.com/consensys/gnark/issues/222) from ConsenSys/perf/std-sw-glv


<a name="v0.6.0"></a>

## [v0.6.0] - 2022-01-03

**Important: next release (v0.7.0) will be compatible with Go1.18+ only**

### Breaking changes

- `circuit.Define(curveID, api)` -> `circuit.Define(api)`; added `api.Curve()` to retrieve curve info
- `api.Constant(...)` was removed. Can now directy assign values with `=` operator in the circuit definition and the witness assignment.
- `frontend.Variable` is now an alias for `interface{}`
- assert helper is now under `gnark/test`. Instead of taking a `CompiledConstraintSystem` it takes a `Circuit` as input, enabling easier tests accross curves and proving schemes through the use of `test/TestingOption` (`WithBackends(backend.GROTH16)`, `WithCurves(ecc.BN254)`, ...)
- `api.NewHint` handles multiple outputs and custom solver `Hint` definition has changed


### Feat
- added explicit warning when parser encounters unadressable struct field [#169](https://github.com/consensys/gnark/issues/169)
- `FromInterface` supports uintXX and intXX types closes [#197](https://github.com/consensys/gnark/issues/197)
- lighter stack trace by default for circuits, more verbose when `-tags=debug` provided
- added `api.Tag` and `api.AddCounter` to measure number of constraints in portion of circuit
- `api.DivUnchecked` does not handle zero divisor. `api.Div` does.
- added `frontend.IsConstant` and `ConstantValue` apis
- add support for bw6-633 curve
- added `api.Lookup2` method  (2-bit lookup)
- **frontend:** plonk frontend directly implements the `frontend.API` interface instead of building on top of the `R1CS` builder
- **std:** fields and pairing over BLS24-315 in BW6-633 circuit
- **test:** add `Run` for running circuit test configurations as subtests
- **test:** add `Log` method for logging in subtests
- **test:** assert helper cross check constraint system solver results with `big.Int` test execution engine

### Perf
- **std:** verifying a Pairing (bls12-377, Groth16) inside a circuit went from ~40k constraints to less than <12k constraints

### Fix
- fixes [#169](https://github.com/consensys/gnark/issues/169) ensure frontend.Circuit methods are defined on pointer receiver
- fixes [#178](https://github.com/consensys/gnark/issues/178) by adding cbor.MaxMapPairs options when reading R1CS
- fixed `AssertIsBoolean` in plonk (mul by constant failed)
- fixes [#168](https://github.com/consensys/gnark/issues/168) adds context to a non-deterministic compilation error in the Assert object
- **frontend:** reduce constant by modulus
- **frontend:** plonk compiler now outputs a reasonable number of constraints  [#186](https://github.com/consensys/gnark/issues/186)

### Build
- updated to gnark-crypto v0.6.0

### Pull Requests
- Merge pull request [#192](https://github.com/consensys/gnark/issues/192) from ConsenSys/multi-hint
- Merge pull request [#220](https://github.com/consensys/gnark/issues/220) from ConsenSys/feat-from-interface
- Merge pull request [#217](https://github.com/consensys/gnark/issues/217) from ConsenSys/fix-internal-compiled
- Merge pull request [#191](https://github.com/consensys/gnark/issues/191) from ConsenSys/assert-subtests
- Merge pull request [#200](https://github.com/consensys/gnark/issues/200) from ConsenSys/refactor/frontend
- Merge pull request [#205](https://github.com/consensys/gnark/issues/205) from ConsenSys/fix/constant-mod-reduction
- Merge pull request [#186](https://github.com/consensys/gnark/issues/186) from ConsenSys/fix/plonk_constraints
- Merge pull request [#185](https://github.com/consensys/gnark/issues/185) from ConsenSys/feat/bw6-633
- Merge pull request [#189](https://github.com/consensys/gnark/issues/189) from ConsenSys/lookup2
- Merge pull request [#183](https://github.com/consensys/gnark/issues/183) from ivokub/hint-registry
- Merge pull request [#182](https://github.com/consensys/gnark/issues/182) from ConsenSys/std/pairing
- Merge pull request [#176](https://github.com/consensys/gnark/issues/176) from ConsenSys/feat-constraint-counter
- Merge pull request [#180](https://github.com/consensys/gnark/issues/180) from ConsenSys/refactor-variable-interface
- Merge pull request [#173](https://github.com/consensys/gnark/issues/173) from ConsenSys/feat-debug-tag


<a name="v0.5.2"></a>

## [v0.5.2] - 2021-11-03


### Breaking changes

- circuit `Define(... cs *frontend.ConstraintSystem)` now takes an interface `Define( ... api frontend.API)`
- assert helper is now under `backend/` (instead of `backend/groth16` and `backend/plonk`)

### Build
- updated to gnark-crypto v0.5.3

### Feat
- added `api.DivUnchecked` and `api.Div`. `DivUnchecked` will accept 0 / 0 == 0 as valid.
- `api.Sub` takes a variadic list of input (same as `api.Add`)
- plonk: kzg test srs is cached up to a certain size for faster tests
- removed hardcoded bit size in ToBinary, which now depends by default on `fr.Element` bit size
- api.ToBinary: constraint unsatisfied now has debugInfo
- unset variables: error message comes with a stack trace
- added bandersnatch circuit component under `std/`
- `frontend.Compile` now check that all secret and public inputs are constrained. Check can be ignore through compile option (`frontend.IgnoreUnconstrainedInputs`)
- added UnsafeReadFrom for groth16 Proving and Verifying keys, which doesn't perform subgroup checks on the decoded points

### Fix
- incorrect formula in api.Select when parameters are constant
- plonk: reduce duplicate constraint when splitting r1c in the frontend
- plonk: build permutation index out of bounds 

### Tests

- added a `test/` package with a test execution engine cross checking constraint system with `big.Int` arithmetic
- bases for `Fuzzing` witness and cross checking constraint system output with test execution engine

### Perf
- `api.AssertIsLessOrEqual`: reduced redundant constraints in range check
- std/groth16: ml with short addition chain (13245cs -> 12297, marginal gain)

### Refactor
- all circuits use frontend.API in place of *frontend.ConstraintSystem
- use ecc.Info() where possible to get modulus or field size


### Pull Requests
- Merge pull request [#159](https://github.com/consensys/gnark/issues/159) from ConsenSys/std/bandersnatch
- Merge pull request [#164](https://github.com/consensys/gnark/issues/164) from ConsenSys/perf-scs-compile
- Merge pull request [#161](https://github.com/consensys/gnark/issues/161) from ConsenSys/test-engine-with-hints
- Merge pull request [#162](https://github.com/consensys/gnark/issues/162) from ConsenSys/std/pairing
- Merge pull request [#160](https://github.com/consensys/gnark/issues/160) from ConsenSys/perf-unsafe-decoding
- Merge pull request [#156](https://github.com/consensys/gnark/issues/156) from ConsenSys/std/twistedEdwards
- Merge pull request [#151](https://github.com/consensys/gnark/issues/151) from ConsenSys/testable-circuits
- Merge pull request [#153](https://github.com/consensys/gnark/issues/153) from ConsenSys/fix/plonk_constraints_blowup
- Merge pull request [#146](https://github.com/consensys/gnark/issues/146) from ConsenSys/feat/ml-snark-pairing
- Merge pull request [#148](https://github.com/consensys/gnark/issues/148) from ConsenSys/perf-range-check

<a name="v0.5.1"></a>
## [v0.5.1] - 2021-09-20

### Build
- updated to gnark-crypto@v0.5.1

### Feat
- adds solver hints `cs.NewHint` [#139](https://github.com/ConsenSys/gnark/pull/139)
- adds `cs.AssertIsDifferent` [#131](https://github.com/ConsenSys/gnark/pull/131)
- consistent support of `cs.Println` and `debugInfo` [#142](https://github.com/ConsenSys/gnark/pull/142)
- adds sanity check in `frontend.Compile` to ensure constraint validity
- adds `witness.WriteSequence` to export expected witness sequence
- adds sanity checks in R1CS and SparseR1CS solvers
- adds ToHTML on SparseR1CS and R1CS
- add witness reconstruction methods. closes [#135](https://github.com/consensys/gnark/issues/135)

### Perf
- IsZero is now implemented with 3 constraints [#134](https://github.com/ConsenSys/gnark/pull/134)
- Groth16 maximize number of infinity points in G2 part of the ProvingKey [#130](https://github.com/ConsenSys/gnark/pull/130)

### Fix
- can constraint linear expressions to be boolean fixes [#136](https://github.com/consensys/gnark/issues/136)
- **eddsa:** addition of isOnCurve check
- **eddsa:** S not splitted (s<r) + 2,3 Double instead of [cofactor]G

### Refactor
- `groth16.Prove` and `plonk.Prove` takes `backend.ProverOption` as parameter
- factorized structs between `compiled.SparseR1CS` and `compiled.R1CS`
- assertions -> constraints in `R1CS` and `SparseR1CS`
- removed `r1c.SolvingMethod` in favor of `cs.NewHint`
- `cs.IsZero` doesn't need curveID anymore

### Test
- ensure `frontend.Compile` is deterministic
- added non regression for `cs.Println` and `debugInfo` traces
- added circuit statistic non regression tests
- added plonk path to `integration_test.go`
- added test from [#136](https://github.com/consensys/gnark/issues/136)


<a name="v0.5.0"></a>

## [v0.5.0] - 2021-08-20

### Build
- updated to latest gnark-crypto v0.5.0

### Feat
- add bls24-315 to gnark
- PlonK implementation as-in-the-paper
- removed `gnarkd` and `examples/benchmark`
- better errors for incorrect variable assignments [#120](https://github.com/consensys/gnark/issues/120)
- call stack displayed when AssertIsEqual fails
- remove term.CoeffValue and use constant coeff ID for special values instead
- added NbG1 and NbG2 apis on groth16 Proving and Verifying keys closes [#116](https://github.com/consensys/gnark/issues/116)
- addition of circuit component FiatShamir in std
- remove serialization test by default in assert helper
- addition of unit test for cyclo square in std/../e12.go
- LinearExpression implements Sort interface. replaced quickSort() by sort.Sort(...)
- SparseR1CS and PlonK objects implements io.ReaderFrom and io.WriterTo
- invalid gnark struct tag options return error at compile time fixes [#111](https://github.com/consensys/gnark/issues/111)

### Fix

- use of doubling formula instead of add(x,x) fixes [#114](https://github.com/consensys/gnark/issues/114)
- create cbor decoder with MaxArrayElements set to max value
- fix [#96](https://github.com/consensys/gnark/issues/96)
- r1cs compilation is deterministic, fixes [#90](https://github.com/consensys/gnark/issues/90)
- plonk circuit compiled with no constraints [#112](https://github.com/consensys/gnark/issues/112)
  
### Perf
- **frontend:** compile takes optional expected constraint number to reserve memory and speed up compile time
- **plonk:** prover uses available CPUs, memory allocation clean up
- **plonk:** frontend have fast path for -1, 0, 1 and 2 coefficients. less mem allocs.
- replaced string concat in frontend with strings.Builder
- **plonk:** when doing fft on domainH with coset, don't scale zero values
- **plonk:** minor tweaks, removing un-needed bitreverse and mem allocs
- **scs:** sparse r1cs have fast path for special coeffs operations
- **std:** adds E2/E12 square and cyclo square in E12 (used FinalExp)

### Refactor
- mimc uses Write(data) then Sum() instead of Sum(data)
- Hash-->Sum in mimc gadget
- **groth16:** SizePublicWitness to NbPublicWitness
- renamed GetCurveID() to CurveID() on groth16 objects

### Test
- test for Fiat Shamir gadget
- added reference frontend.Compile benchmarks



<a name="v0.4.0"></a>
## [v0.4.0]

### Docs
- added [`gnark` User Documentation]
- updated [Go package documentation](https://pkg.go.dev/mod/github.com/consensys/gnark)

### Feat
- **gnarkd:** exposing gnark APIs through RPCs ([#54](https://github.com/consensys/gnark/issues/54))
- **PlonK:** adding functionality to convert a constraint system to PlonK constraints ([#56](https://github.com/consensys/gnark/issues/56))
- **PlonK:** added experimental support for PlonK backend

### Fix
- inverse and div in frontend had some variable ID offset issues ([#62](https://github.com/consensys/gnark/issues/62))
- cs.Println doesn't trigger panic anymore
- Split S in EdDSA signature to prevent overflow [#88](https://github.com/consensys/gnark/issues/88)
- **fft:** fixed the ordering of cosets factor according to DIF/DIT

### Groth16
- VerifyingKey data structure change to ensure compatibility with other impl and Solidity in Ethereum. Serialization format change.

### Integration_test
- added witness serialization tests

### Refactor
- gurvy -> gnark-crypto
- moved fft in gnark-crypto
- bn256 -> bn254, bls377 -> bls12-377, etc. following gnark-crypto v0.4.0
- removed the Curve field in the R part of eddsa signature

### Test
- added frontend and backend fuzz.go, go-fuzz compatible format
- added cs.Println must not panic base test

### Build
- moved solidity integration tests in github.com/consensys/gnark-tests
- added integration fuzz test in backend/groth16/fuzz_test.go



[v0.4.0]: https://github.com/consensys/gnark/compare/v0.4.0...v0.4.0
[`gnark` User Documentation]: https://docs.gnark.consensys.net