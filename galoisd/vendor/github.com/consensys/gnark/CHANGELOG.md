<a name="v0.8.1"></a>
## [v0.8.1] - 2023-07-11
### Chore
- update version
- update gnark-crypto dependency


<a name="v0.8.0"></a>
## [v0.8.0] - 2023-02-14
### Build
- update to latest gnark-crypto
- update to latest gnark-crypto
- make linter happy remove deprecated call
- updated to feat/commitment branch on gnark-crypto ([#408](https://github.com/consensys/gnark/issues/408))
- test only on go 1.18 for now
- re-ran go generate
- update CI ([#318](https://github.com/consensys/gnark/issues/318))

### Ci
- fix slack integration + adds golanglint-ci ([#316](https://github.com/consensys/gnark/issues/316))

### Clean
- remove deadcode and kill `api.Tag` and `api.Counter` ([#353](https://github.com/consensys/gnark/issues/353))

### Docs
- updated README.md
- remove logo references
- update doc link
- describe limb regrouping for equality
- add method documentation
- add explainer
- comments
- add method docs

### Feat
- added serialization header to CS and debug info to all constraints with -tags=debug ([#347](https://github.com/consensys/gnark/issues/347))
- checkpoint 3, computations are correct, some commitments are failing
- add cs.GetConstraint with examples, and pretty printer helpers ([#452](https://github.com/consensys/gnark/issues/452))
- cleaned plonk constraints evaluation
- code gen + cleaned code
- update gnark version to v0.8.0
- plonk constraints captured using Expressions
- code gen
- Evalute is used instead of manually evaluatin
- removed printings
- fixed verifier
- add automatic non-native witness element limb constraining ([#446](https://github.com/consensys/gnark/issues/446))
- cleaned code
- addition of missing commitments and openings in vk and pk
- checkpoint 2
- checkpoint refactor
- add ECDSA signature verification ([#372](https://github.com/consensys/gnark/issues/372))
- adds `api.MAC(..)` ([#427](https://github.com/consensys/gnark/issues/427))
- keccak-f permutation function ([#401](https://github.com/consensys/gnark/issues/401))
- add debug.SymbolTable into constraint system for storage efficiency of debug info ([#421](https://github.com/consensys/gnark/issues/421))
- split field in field emulation into Field and FieldAPI ([#395](https://github.com/consensys/gnark/issues/395))
- testing options and clearer errors
- add linear expression packing for R1CS ([#418](https://github.com/consensys/gnark/issues/418))
- merge develop
- add inherit circuit tag ([#387](https://github.com/consensys/gnark/issues/387))
- add gnark tags
- gkr verifier is NOT a witness object
- some sumcheck experiments
- poly functions and some tests
- gkr verifier
- some sumcheck experiments
- replaced full bit decomposition by rshift in emulated/assertIsEqual ([#354](https://github.com/consensys/gnark/issues/354))
- cleaned code
- add test.NoFuzzing() testing option ([#296](https://github.com/consensys/gnark/issues/296))
- map in snark
- add fake API for emulated arithmetics
- split add/mul/sub into cond and op
- addition of dynamic test for kzg bls24315
- addition of dynamic test kzg bls12377
- size of fiat shamir data is harcoded
- count fields in first schema parsing
- add VariableCount method to builder
- implement AssertIsLessEqualThan
- binary composition
- add Params.isEqual
- implement Lookup2 and select
- add panicking constant init
- added verifier (forgot to commit it)
- merge develop
- addition of proximity tests
- uncomment integration tests
- only mul is tested for plonk fri
- addition of plonkfri in test package
- code gen
- addition of templates
- Fiat Shamir done
- re activated blinding
- removed dead code + old commented code
- removed mock commitment scheme
- verification of Z, Zshifted
- addition of proofs for s1,s2,s3 and ccircuit coefficients
- **frontend:** add builder wrapper compile opt
- **hint:** allow registering multiple hints
- **plonk:** addition of skeleton prover with non homomorphic PCS
- **std:** KZG verifier gadget bls24-315 (with static witness)

### Fix
- mark and output boolean ([#459](https://github.com/consensys/gnark/issues/459))
- mimc pow7
- prover-verifier work with blinding
- handle recursive hints in level builder ([#441](https://github.com/consensys/gnark/issues/441))
- verifier working \o/
- pack full limbs for quotient ([#439](https://github.com/consensys/gnark/issues/439))
- add shortcut for const input in MulConst ([#438](https://github.com/consensys/gnark/issues/438))
- closes [#434](https://github.com/consensys/gnark/issues/434) returns a copy of the input slice when filtering groth16+commitment ([#435](https://github.com/consensys/gnark/issues/435))
- fix previous commit
- closes [#400](https://github.com/consensys/gnark/issues/400) path trimming correct, example with unix path separators
- tests expected to fail
- engine.Println to take strings ([#419](https://github.com/consensys/gnark/issues/419))
- verifier input building
- idiotic load circuit bug
- update test vector proofs to proper size, some bugs
- fix [#400](https://github.com/consensys/gnark/issues/400) with trim path handling in profile report ([#409](https://github.com/consensys/gnark/issues/409))
- fixed Lagrange polynomials construction ([#389](https://github.com/consensys/gnark/issues/389))
- staticcheck
- multi-fan-out input bug
- update test vectors, hash finalevalproofs
- less elegant "hollow", but it works
- some problems in sumcheck and gkr
- TestTranscript works
- Xor(var, constant) in scs corrected
- simple sumcheck test passes
- IsZero throws panic on ([#367](https://github.com/consensys/gnark/issues/367))
- fixes [#359](https://github.com/consensys/gnark/issues/359) missing fields in plonk serialized format ([#364](https://github.com/consensys/gnark/issues/364))
- Sumcheck verifier usable as circuit
- InterpolateOnRange works even when it doesn't really have to "inerpolate"
- extra nosec G404 in test file
- minor typo
- misspelled ("decsribes" ->  "describes") ([#339](https://github.com/consensys/gnark/issues/339))
- remove leq overwrite
- reduce element when init from const
- fixed comments
- kzg verifier test
- rebase on develop
- reduce div integration circuit
- remove broken equality fast-path
- remove subtraction padding optimisation
- use BaseField() to get modulus
- use scalar field bitlength directly
- change hint definitions
- reduce given argument not inline
- ToBits return nb of bits
- ToBits edge case for overflow=0
- consider carries in bit decomposition
- return element
- add/set use argument nb of limbs
- set overflow after reduce
- remove unused method
- more precise padding computation
- make conversion functions private
- remove debug calls
- multiplication top limb width
- remove second high limb exception in sub padding
- added DecomposeScalarG2 to std.RegisterHints
- enfore width after inverse and div
- remove api from constant init
- ignore plonk_fri in internal/stats for now
- remove unused error return
- check unchecked error
- copy instead of loop
- fixed position of the shifted opening
- fixed size Iop (error due to the blinding)
- fixed opening Merkle path
- fixed vanilla plonk fri
- removed unused debug function
- **emulated:** enforce widths of packed limbs ([#368](https://github.com/consensys/gnark/issues/368))
- **nonnative:** off by one error
- **plonk:** fixed generic verifier

### Fix
- minor typo ([#360](https://github.com/consensys/gnark/issues/360))

### Perf
- more precomputation in plonk/iop ([#471](https://github.com/consensys/gnark/issues/471))
- mimc on bls12-377/fr uses x^17 as a permutation
- api.IsZero generate less constraints ([#356](https://github.com/consensys/gnark/issues/356))
- minor improvments to big.Int test engine ([#344](https://github.com/consensys/gnark/issues/344))
- allocate less in test engine
- remove width enforcement in Reduce()
- lazy reduction
- constrain main bits in equality diff to be only zeros
- optimize equality check

### Refactor
- update to latest gnark crypto
- keeping up
- gnark-crypto iop 1
- clean up witness package, introduces clean `witness.Witness` interface ([#450](https://github.com/consensys/gnark/issues/450))
- emulated clean up fixes [#448](https://github.com/consensys/gnark/issues/448) ([#449](https://github.com/consensys/gnark/issues/449))
- reflect gnark-crypto gkr changes, debug pending
- add constraint package and improve memory management in frontend ([#412](https://github.com/consensys/gnark/issues/412))
- use polynomial package
- std/math/nonnative -> std/math/emulated ([#345](https://github.com/consensys/gnark/issues/345))
- remove in-method reductions
- use field in schema leaf handler
- Compile(ecc.ID) -> Compile(field *big.Int) ([#328](https://github.com/consensys/gnark/issues/328))
- VerifyFri -> Verify

### Refactor
- remove geth dependency ([#440](https://github.com/consensys/gnark/issues/440))

### Style
- clean up unused functions
- remove questions
- Multilin -> MultiLin
- Multilin -> MultiLin
- comment about scalar fields of in-circuit KZG
- moved kzg in circuit in commitment/ folder
- removed dead code
- removed dead comments
- factored code in integration_test
- remove dead code
- remove irrelevant TODOs
- typos
- rename n to r
- inline test struct init
- cleanup test comments
- change field to params in tests
- test name update
- remove unused documentation file
- removed dead debug printings
- uncomment verify.go in generic plonk
- removed printing functions

### Test
- add four-instance test case
- all pass except "two_input_single_identity_gate_two_instances"
- trying to feed proof as circuit input, reflect errors
- routine developed for gkr
- Transcript fails
- doubleMap works
- singleMap works
- fix incorrect proof
- circuit and witness cannot be the same object
- counter start from 2
- all parts of witness get unconstrained error
- sumcheck in circuit, getting errors
- pass
- sumcheck in circuit, getting errors
- add Goldilocks tests
- add fake API tests
- bench kzg verifier with plonk
- add test/solver_test.go ([#329](https://github.com/consensys/gnark/issues/329))
- binary composition test
- implement lookup2 test
- implement constant test
- fewer test cases
- implement large computation circuit
- implement select test

### Pull Requests
- Merge pull request [#469](https://github.com/consensys/gnark/issues/469) from ConsenSys/fix/mimc-pow7
- Merge pull request [#451](https://github.com/consensys/gnark/issues/451) from ConsenSys/feat/iop_refactor
- Merge pull request [#455](https://github.com/consensys/gnark/issues/455) from ConsenSys/develop
- Merge pull request [#453](https://github.com/consensys/gnark/issues/453) from ConsenSys/mimx/nb-rounds-bls12377
- Merge pull request [#393](https://github.com/consensys/gnark/issues/393) from ConsenSys/feat/gkr
- Merge pull request [#361](https://github.com/consensys/gnark/issues/361) from ConsenSys/feat/polynomial
- Merge pull request [#363](https://github.com/consensys/gnark/issues/363) from ConsenSys/fix/lde-0div
- Merge pull request [#362](https://github.com/consensys/gnark/issues/362) from ConsenSys/fix/mathrand
- Merge pull request [#250](https://github.com/consensys/gnark/issues/250) from ConsenSys/feat/plonk_generic
- Merge pull request [#325](https://github.com/consensys/gnark/issues/325) from ConsenSys/feat/emulated-api
- Merge pull request [#331](https://github.com/consensys/gnark/issues/331) from ConsenSys/perf/test-engine
- Merge pull request [#332](https://github.com/consensys/gnark/issues/332) from ConsenSys/refactor/schema-parsing
- Merge pull request [#334](https://github.com/consensys/gnark/issues/334) from ConsenSys/fix/nonnative-offbyone
- Merge pull request [#320](https://github.com/consensys/gnark/issues/320) from ConsenSys/perf/nonnative
- Merge pull request [#307](https://github.com/consensys/gnark/issues/307) from ConsenSys/feat/std/kzg-verifier
- Merge pull request [#302](https://github.com/consensys/gnark/issues/302) from ConsenSys/feat/nonnative-ff


<a name="v0.7.1"></a>
## [v0.7.1] - 2022-04-14
### Build
- fix gosec warnings
- updated to gnark-crypto v0.7.0

### Ci
- updated github actions
- test against go1.17 and go1.18 ([#288](https://github.com/consensys/gnark/issues/288))

### Clean
- std/groth16 uses same notation as out-of-circuit groth16 ([#304](https://github.com/consensys/gnark/issues/304))
- remove PairingContext and Extension objects from api calls in std/.../pairing ([#286](https://github.com/consensys/gnark/issues/286))

### Docs
- added security policy, gnark-announce and twitter link
- updated DOI
- updated README.md with same warning as in docs
- added Deprecated comments in front of APIs moved to Compiler interface
- clean up hint interface comment
- updated DOI

### Feat
- implement PR suggestions in std/math/bits
- disable logger in tests by default, unless debug tag present
- adds std/math/bits/ToNAF
- added VerifyingKey.Assign methods in std/groth16 ([#306](https://github.com/consensys/gnark/issues/306))
- add gnark/logger
- remove offset shifts in plonk compile
- remove post-compile offset id in R1CS builder
- added internal/stats package
- hint.NbOuputs should not be used at solve time, only at compile time
- added ivokub suggestion on logging duration values
- adds gnark logger. closes [#202](https://github.com/consensys/gnark/issues/202)
- added ToTernary closes [#269](https://github.com/consensys/gnark/issues/269)
- moved api.FromBinary to std/math/bits
- add ToBinary in std/math/bits
- added std.GetHints for convenience. fixes [#264](https://github.com/consensys/gnark/issues/264). error message when hint is missing now has hint name
- added NBits hint
- make nboutputs of a hint explicit at compile time
- **std:** added AssertIsTrit

### Fix
- move init() behind sync.Once. remove verbose option in stats binary
- fix previous commit
- err instead of panic when recursively solving hints
- add whitespace between vars in test.Println
- closes [#293](https://github.com/consensys/gnark/issues/293) and enables recursive hints solving
- replace stats snippet signature with newVariable() instead of fixed variable
- restored logger format
- re generated stats
- fixes [#266](https://github.com/consensys/gnark/issues/266) by adding constant path in Lookup2 and Select
- incorrect handling of nbBits == 1 in api.ToBinary
- gosec errors
- uncomment fuzz part of test
- std.GetHints() return bits.NNAF
- **stats:** fix pairing stats. added run flag for stats binary to filter with regexp

### Perf
- restored frontend.WithCapacity option...
- **plonk:** IsConstant -> ConstantValue
- **sw:** no need for Lookup2 in constScalarMul
- **tEd:** Add -1C

### Refactor
- delete dead code (TripleMillerLoop)
- std/pairing have more consistent apis
- std/pairing bls12377 api more coherent
- remove StaticHint wrapper, log duplicate hints ([#289](https://github.com/consensys/gnark/issues/289))
- backend.WithOutput -> backend.WithCircuitLogger
- remove all internal circuits from stats, keep important snippets only
- move circuit_stats_test.go into internal/stats
- move NBits hint to math/bits
- move ntrits hint to std/math/bits
- make api.ToBinary point to math/bits/ToBinary
- revert Builder -> Compiler internal name change
- compiler -> r1cs and scs internally
- start moving api.ToBinary to std/math/bits/
- compiled.Variable -> compiled.LinearExpression
- factorize coeff table initialization
- frontend/cs subpackages to match new interfaces
- split compiler, api and builder interface into interfaces
- remove IsBoolean from R1CS variables
- preparing frontend.Compiler interface
- frontend.Compile now takes a builder instead of backendID as parameter
- moved internal/compiled to frontend/compiled
- remove nb inputs from hint declaration

### Style
- code cleaning in std/pairing
- code cleaning
- added clearer error message for groth16 verifier missing init in circuit
- remove dead code
- remove duplicate import in template
- code cleaning
- remove dead code
- code cleaning

### Test
- add failing test for [#293](https://github.com/consensys/gnark/issues/293)

### Pull Requests
- Merge pull request [#298](https://github.com/consensys/gnark/issues/298) from ConsenSys/fix/hint-panic
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


<a name="v0.6.5"></a>
## [v0.6.5] - 2022-04-13
### Fix
- **plonk:** security vuln in fiat-shamir inputs


<a name="v0.7.0"></a>
## [v0.7.0] - 2022-03-25
### Build
- fix gosec warnings
- updated to gnark-crypto v0.7.0

### Ci
- updated github actions
- test against go1.17 and go1.18 ([#288](https://github.com/consensys/gnark/issues/288))

### Clean
- remove PairingContext and Extension objects from api calls in std/.../pairing ([#286](https://github.com/consensys/gnark/issues/286))

### Docs
- updated README.md with same warning as in docs
- added Deprecated comments in front of APIs moved to Compiler interface
- clean up hint interface comment
- updated DOI

### Feat
- adds std/math/bits/ToNAF
- remove offset shifts in plonk compile
- added ivokub suggestion on logging duration values
- add gnark/logger
- hint.NbOuputs should not be used at solve time, only at compile time
- remove post-compile offset id in R1CS builder
- added internal/stats package
- added ToTernary closes [#269](https://github.com/consensys/gnark/issues/269)
- adds gnark logger. closes [#202](https://github.com/consensys/gnark/issues/202)
- disable logger in tests by default, unless debug tag present
- implement PR suggestions in std/math/bits
- moved api.FromBinary to std/math/bits
- add ToBinary in std/math/bits
- added std.GetHints for convenience. fixes [#264](https://github.com/consensys/gnark/issues/264). error message when hint is missing now has hint name
- added NBits hint
- make nboutputs of a hint explicit at compile time
- **std:** added AssertIsTrit

### Fix
- add whitespace between vars in test.Println
- closes [#293](https://github.com/consensys/gnark/issues/293) and enables recursive hints solving
- replace stats snippet signature with newVariable() instead of fixed variable
- move init() behind sync.Once. remove verbose option in stats binary
- re generated stats
- gosec errors
- fixes [#266](https://github.com/consensys/gnark/issues/266) by adding constant path in Lookup2 and Select
- uncomment fuzz part of test
- std.GetHints() return bits.NNAF
- incorrect handling of nbBits == 1 in api.ToBinary
- **stats:** fix pairing stats. added run flag for stats binary to filter with regexp

### Perf
- restored frontend.WithCapacity option...
- **plonk:** IsConstant -> ConstantValue
- **sw:** no need for Lookup2 in constScalarMul
- **tEd:** Add -1C

### Refactor
- std/pairing have more consistent apis
- std/pairing bls12377 api more coherent
- remove StaticHint wrapper, log duplicate hints ([#289](https://github.com/consensys/gnark/issues/289))
- backend.WithOutput -> backend.WithCircuitLogger
- remove all internal circuits from stats, keep important snippets only
- move circuit_stats_test.go into internal/stats
- move NBits hint to math/bits
- move ntrits hint to std/math/bits
- make api.ToBinary point to math/bits/ToBinary
- revert Builder -> Compiler internal name change
- compiler -> r1cs and scs internally
- start moving api.ToBinary to std/math/bits/
- compiled.Variable -> compiled.LinearExpression
- factorize coeff table initialization
- frontend/cs subpackages to match new interfaces
- split compiler, api and builder interface into interfaces
- remove IsBoolean from R1CS variables
- preparing frontend.Compiler interface
- frontend.Compile now takes a builder instead of backendID as parameter
- moved internal/compiled to frontend/compiled
- remove nb inputs from hint declaration

### Style
- code cleaning
- added clearer error message for groth16 verifier missing init in circuit
- remove dead code
- remove duplicate import in template
- code cleaning
- remove dead code
- code cleaning

### Test
- add failing test for [#293](https://github.com/consensys/gnark/issues/293)

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
- updatd to latezst gnarkcrypto

### Docs
- updated changelog for v0.6.4
- updated README.md with playground link

### Feat
- plonk adapted to kzg modifications
- udpate gnark-crypto
- code gen for plonk
- polynomial --> []frElement
- groth16 prover adapted to new fft OK
- **plonk:** beta is dervied using Fiat Shamir
- **tEd:** implements double-base scalar mul

### Fix
- fixed trace and println tests
- fixed wrong bigInt op in plonk api
- resolve comments
- restored commented code blinding polynomial
- fixed verifier
- verifier obtains correct quotient
- missing beta in linearized polynomial
- linearized polynomial OK
- correct up to quotient
- fixed plonk up to permutation polynomial
- **tEd:** case when scalar size is odd
- **tEd:** case when scalar size is odd

### Perf
- sparse R1CS solver is parallel
- R1CS solver may now run in parallel
- **EdDSA:** eddsa gadget using double-base scalar mul
- **bandersnatch:** apply tEd perf changes to Bandersnatch

### Refactor
- **eddsa:** rearrange eddsa verif as cofactor clearing counts

### Style
- code cleaning
- removed debug comments

### Test
- **tEd:** test scalarMul for all curves and schemes

### Pull Requests
- Merge pull request [#259](https://github.com/consensys/gnark/issues/259) from ConsenSys/perf-parallel-solver
- Merge pull request [#261](https://github.com/consensys/gnark/issues/261) from ConsenSys/feat/kzg_updated
- Merge pull request [#257](https://github.com/consensys/gnark/issues/257) from ConsenSys/perf/EdDSA
- Merge pull request [#253](https://github.com/consensys/gnark/issues/253) from ConsenSys/feat/fft_cosets


<a name="v0.6.3"></a>
## [v0.6.3] - 2022-02-13
### Build
- make staticcheck happy

### Docs
- updated changelog for v0.6.3
- updated example in README.md

### Feat
- updated gnark-crypto
- removed seed in mimc
- mimc implem corresponds to ethereum implem

### Fix
- fixes [#255](https://github.com/consensys/gnark/issues/255) variable visibility inheritance regression
- mod reduce input in solve with hint when coming from interface
- counter was set with PLONK backend ID in R1CS
- fixed conflicts
- assign a, b, c to solution and div by coeff instead of mul
- use uint64 for cbor unmarshal and cast
- fixed mimc example

### Perf
- replace big int pool in hint solver by tmp slice
- r1cs solver faster linear expression eval
- r1cs solver 40% faster by avoiding redudnant check

### Test
- benchmark solve r1cs with large linear exp
- added r1cs solve benchmark

### Pull Requests
- Merge pull request [#256](https://github.com/consensys/gnark/issues/256) from ConsenSys/fix-bug-compile-visibility
- Merge pull request [#249](https://github.com/consensys/gnark/issues/249) from ConsenSys/perf-ccs-hint
- Merge pull request [#248](https://github.com/consensys/gnark/issues/248) from ConsenSys/perf-ccs-solver
- Merge pull request [#247](https://github.com/consensys/gnark/issues/247) from ConsenSys/fix/plonk_cbor


<a name="v0.6.2"></a>
## [v0.6.2] - 2022-01-28
### Fix
- r1cs.GetConstraint bad alloc


<a name="v0.6.1"></a>
## [v0.6.1] - 2022-01-28
### Build
- go 1.16 to go 1.17
- github workflow against go 1.17 only
- github workflow against go 1.17 and go 1.18

### Clean
- better errors in witness

### Docs
- updated CHANGELOG.md with v0.6.1 changes
- update backend/witness godoc
- added bibtex citation
- **backend:** unify documentation for options
- **frontend:** unify docs for options
- **test:** unify documentation for options

### Feat
- added ccs.GetConstraints
- added witness.Public() to return Public part of the witness
- addition of Cmp in the API
- added GetSchema in CompiledConstraintSystem
- witness unmarshal uses limit reader if schema is set
- schema now stores nbPublic and nbSecret
- add optional omitempty tag in json generated schema
- prepare new witness API
- added frontend/schema to build circuit and witness schemas

### Fix
- handle array of array of array... in schema
- remove limit reader when parsing json
- deal with zero value in partial JSON witness
- added witness MarshalJSON working test. few ugly hacks to remove
- implemented pr suggestions. restored, fixed and addded sub tests in tags_test.go
- remove gnark embed tag in example rollup
- remove embbed struct tag
- typo in hint fn name

### Perf
- minimize allocations in ccs.GetConstraints

### Refactor
- compiled.Visbility -> schema.Visibiility
- added IsSolved API on the CompiledConstraintSystem interface
- backend/witness has no more dependency on frontend
- reduce frontend dependency in witness and remove most of type switches
- witness.WriteSequence -> schema.WriteSequence
- CopyTo -> VectorToAssignment
- factorize code in witness_test
- rename some variables from witness to assignment to avoid confusion
- cleaned internal/witness unused code
- killed ReadAndProve and ReadAndVerify (plonk)
- killed ReadAndProve and ReadAndVerify (groth16)

### Style
- replace ①  by 1
- change main/companion to outer/inner curve
- clean up unsatisfied constraint error paths
- return constraint formatted as in the paper
- experiment 2 constraint str
- experiment 1 plonk constraint string repr
- plonk human readable constraint A+M+k == O
- constraint is not satified err -> constraint #id is not satisfied
- cosmetics in sparseR1C repr
- constraint cosmetics string repr. remove ToHTML deadcode
- code cleaning
- code cleaning, error refactoring
- code clean up, pr review changes
- remove defers in test/assert.go
- group WitnessOption under same type, implements PR review
- clean some todo
- **frontend:** create CompileOption type
- **frontend:** use functional opts for witness
- **test:** create TestingOption type for function opts

### Test
- added assert.marshalWitness subtest to ensure round trip json and binary serialization test coverage

### Tests
- run marshalling tests as subtests

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
## [v0.6.0] - 2022-01-04
### Build
- update to latest gnark-crypto with fix for empty addchain folders
- updated to latest gnark-crypto

### Clean
- remove TotalInputs

### Docs
- updated changelog.md
- updated README.md example
- draft release notes for v0.6.0
- zkteam -> gnark
- add suggestion to error

### Feat
- extended the Add test
- internal/parser handles interface values and outputs warning when a struct is non adressable [#169](https://github.com/consensys/gnark/issues/169)
- added explicit warning when parser encounters unadressable struct field [#169](https://github.com/consensys/gnark/issues/169)
- addition of test for mul
- lighter stack trace by default for circuits, more verbose when -tags=debug provided
- added api.Tag and api.AddCounter to measure number of constraints in portion of circuit
- add debug info for SparseR1CS inverse
- add option to define newBuilder
- register frontend for backends
- register frontend compilers
- removed txt logs
- handle non zero divisor in Div
- addition of boolean table for sparseR1cs
- restored  options (forgot to commit the file)
- extended select test
- addition of test for select api
- api.AddCounter now measure new plonk constraints too
- made inv test explicit
- test circuits specifiy curves (useful for Div,Inv)
- addition of test for final exp in sw_bls24315
- cleaned  frontend/
- code uses new Compile from compile.go
- hint inptus LinearExpression -> interface
- closes [#197](https://github.com/consensys/gnark/issues/197) FromInterface supports uintXX and intXX types
- extended add test
- addition of Compile method on System interface
- addition of tests in r1cs/
- addition of Tag, Counter in plonk/ and r1cs/
- populated r1cs/ plonk/ for separating schemes
- finished assertions plonk
- completion of API (except rangeCheck, lookup2, Hints)
- started the addition of plonk secific cs
- added frontend.IsConstant and ConstantValue apis
- added api.CurveID()
- boolean constraint are handled separately
- restored plonk/groth16 switch in frontend
- add new curve bw6-633
- triplePairing test uses 3 different pairs
- counters accesible through CompiledConstraintSystem
- **frontend:** add lookup2 API method
- **frontend:** NewHint returns multiple outputs
- **integration_test:** add variable-input/output hint test
- **std:** fields, SW and pairing over BLS24-315 in BW6-633 circuit
- **test:** add Run for running as subtests
- **test:** add Log method for logging in subtests
- **test:** solve multi-output hints in engine

### Fix
- fixed mul
- addition of mod reduction in plonk api to bound bigInt
- re-exposed compiled.ConstraintSystem (internal) as frontend.CompiledConstraintSystem
- fixed examples/plonk
- reduce variables modulo scalar order
- hint solver was not returning objects to pool
- use sr1cs debug representation
- restored check unconstrained inputs when compiling
- fixed staticcheck
- fixes [#168](https://github.com/consensys/gnark/issues/168) adds context to a non-deterministic compilation error in the Assert object
- fixed staticheck
- removed unused file
- ran go generate
- fixed signature of addPlonkConstraint
- fixed bigInt passed by copy in plonk api
- mul test for all curves
- pairing test on Plonk + Groth16
- removed dead folder
- fixed mul test
- fixed AssertIsEqual when an argument is constant
- fixed mul by zero
- fixed Div (plonk)
- fixed Div(constant, var)
- plonk solver handles both L and R to solve
- Div -> DivUnchecked test div
- fixed code gen
- fixed range_constant plonk
- restored simpler version for range_constant r1cs
- fixed lookup2
- fixed isZero
- fixed assertIsEqual
- fixed assertion (AssertIsEqual missing return)
- fixed AssertIsBoolean in plonk (mul by constant failed)
- fixes [#169](https://github.com/consensys/gnark/issues/169) ensure frontend.Circuit methods are defined on pointer receiver
- extended sub test
- fixed Groth16 integration tests
- restored bootloader
- removed Compile() method in engine
- fixed API missing methods
- incorrect handling of hints in r1cs solver
- fixes [#178](https://github.com/consensys/gnark/issues/178) by adding cbor.MaxMapPairs options when reading R1CS
- merge with develop branch
- remove deadcode (csfuzz targets)
- go:generate bw6633 and updated circuit stats
- fixed code gen :/
- fixed gosec
- ran gofmt
- set variable to Solved after Or,And,Xor when compiling to plonk
- handled nbBits=1 in toBinary
- fixed comments
- resolve comments
- fixed Div
- fixed tests in cs_api_test
- restored log in parser
- fixed XOR, OR, mustBeLessOrEqCst
- fixed select to reduce nb of plonk constraints
- fixed ToBinary
- fixed toSparseR1CS, removed dead code
- fixed Select constraint
- **debug_test:** reverse trace and expected trace order
- **examples:** omit unnecessary api call
- **frontend:** reduce constant by modulus
- **frontend:** set only hint internal variables constrained
- **init.stats:** update circuit statistics
- **integration_test:** remove fuzzing call

### Frontend
- update hint documentation

### Perf
- **std:** bls24 Miller loop in 2NAF + opt. Final exp

### Refactor
- frontend.Variable -> cs.Variable
- simplified cs_to_r1cs_sparse algo
- Variable is now an interface. std/ wip
- moved compile in compiler/
- std/ uses Variable interface{}
- remove api.Constant. can now assign directly
- move GetWitnessValue to internal test engine func
- unpublish frontend/cs/ structures
- rename frontend/R1CSRefactor to frontend/R1CS
- use compile option to define builder
- cs.Variable -> frontend.Variable
- rename frontend.System to frontend.Builder
- TotalOutputs -> NbOutputs
- use internal util
- move frontend util to internal
- use frontend.Compile
- added frontend.IsConstant and ConstantValue apis
- Define(curveID, api) -> Define(api); api.CurveId()
- modified definition of Variable in zkpScheme
- compiled.CompiledConstraintSystem -> compiled.ConstraintSystem
- move definitions to frontend
- changed Variable definition in cs.go and compiled/
- remove curve from system Compile
- frontend.Variable -> cs.Variable
- interface --> Variable
- CurveID() --> Curve() in API
- cleaning hierarchy of constraints systems
- addition of compiler for both systems
- constraintSystme -> R1CS
- Hint takes []LinearExpression as input
- VariableID -> WireID
- use subtests in backend serialization tests
- simplifed wire tracking in a cs
- api interface allows access to backendID
- refactored  frontend
- modified cs_api.go
- **frontend:** move Tag to frontend/
- **frontend:** add compiler registry
- **integration_test:** run as subtests
- **lookup2:** use multiple witnesses for tests
- **stats:** use subassert for errors
- **stats:** run as subtests
- **test:** use Tag
- **test:** run assert subtests using Run

### Style
- added few comments
- removed dead code
- remove counter example in std/
- remove unused displayCounters boolean
- remove frontend.DisplayCounters option
- remove witnessValue method
- remove bls377tobw761 conv func
- remove bls377tobw761 conv func
- error strings should not end with punctuation or a newline
- **Miller loop:** for loop instead of addition chain

### Test
- use BN254 in modulus-dependent tests for speed
- added failing test with multiple hints in one R1C
- **lookup2:** add integration test

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
### Build
- updated to gnark-crypto v0.5.3
- fix fuzz target compile error
- fix fuzz target compile error
- updated to latest gnark-crypto
- updated to latest gnark-crypto
- updated to gnark-crypto v0.5.2
- updated test timeout, while we improve plonk compile speeds
- fix staticcheck warning
- updated to latest gnark-crypto
- fix unchecked errors
- fix gofuzz target
- re-ran go generate
- fix gofuzz build

### Docs
- updated CHANGELOG.md for v0.5.2
- added pull requests in changelog.md
- added doc to frontend.API interface
- added documentation to new test pacakge

### Feat
- Sub matches Add api in circuit
- attempt at reducing nb constraints for ML by working in affine
- added frontend compile options to handle capacity and unconstrained inputs ignore flag
- added gcd for int64 coeff values in splitR1C, untested
- added test engine support for hints
- added UnsafeReadFrom for groth16 Proving and Verifying keys
- GetKey now returns an ID of a primitive linear expression
- added DivUnchecked. start factorizing some frontend.API with better Constant cases
- added post-compile check to ensure all inputs are constrained fixes [#163](https://github.com/consensys/gnark/issues/163)
- ignore zero coefficients for variable constraint check
- added frontend.API interface
- added fuzzing. div now takes 2 constraints
- caching test srs for faster tests
- added TestableCircuit interface
- added debugInfo for ToBinary API
- unset variables error message comes with a stack trace
- frontend bits operation adjusted to fr.Bits real size
- ml with short addition chain (13245cs -> 12297, marginal gain)
- **std:** add bandersnatch

### Fix
- fixes [#155](https://github.com/consensys/gnark/issues/155) slow compiling with plonk frontend
- don't check if one wire is constrained
- restore std/algebra/sw g1ScalarMul test circuit
- bug in Select when parameters are constant
- regenerated stats.
- remove unneeded mod operations in api
- fixed constraints blow up for plonk
- test engine mod reduce range check bound if not constant
- introduced bug in refactoring
- fix toHTML
- reduce duplicate when splitting r1c
- plonk build permutation index out of bounds
- std/twistededwards remove A
- std/twistededwards remove A
- added markBoolean on inplace boolean constraints to avoid duplicated constraints

### Perf
- fast path with int64 in divide linear expression
- cs.coeffID with gobencode
- fast path for coeffID when coeff is int64
- sparseR1CS reduce improvments
- plonk frontend split and reduce improvments
- plonk compile, replace sha256 with fast lookup with collisions
- minor perf adjustement on big.Int operations and allocations
- reduced redundant constraints in range check

### Refactor
- delete old curve typed hint functions
- hint function signature uses big.Int, no more ID from name
- splitBis -> split
- h and record are now part of scs
- moved testing in assert, code cleaning
- moved testing in assert, code cleaning
- reverted gnark to api in Define()
- all circuits use frontend.API in place of *frontend.ConstraintSystem
- in circuits *frontend.ConstraintSystem to frontend.API
- factorized assert helper accross curves and backends

### Style
- remove unused code (nSquare)
- removed commented code
- removed code for used for debugging purposes
- clean up std/algebra/g1
- removed useless lines in g1 Double
- clean up std/fp2
- use ecc.Info() where possible to get modulus or field size
- std/twisted edwards code cleaning
- std/fiat-shamir clean up
- cleaned up some TODO
- move stats tests in circuitstats_test.go
- remove plonk and groth16 assert.go
- added comment in range check
- remove code no longer used in the pairing
- commented ml test used to diplay nb constraints

### Test
- groth16 marshal 10 round only
- passing
- bypass fuzz test of frontend for now

### Tests
- added add and sub internal test circuits
- minor adjustements or better fuzzing

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
## [v0.5.1] - 2021-09-21
### Build
- go mod tidy
- remove dead code, makes staticcheck happier
- comment fuzz test part that depends on assertions
- increased CI test timeout and run race test on ubuntu only
- fix gosec unhandled error warning

### Docs
- added release notes for v0.5.1 in CHANGELOG.md

### Feat
- added ToHTML on R1CS
- cs.Println and debugInfo supported in groth16 and plonk
- add witness reconstruction methods. closes [#135](https://github.com/consensys/gnark/issues/135)
- added sanity check in frontend.Compile to ensure constraint validity
- add witness.WriteSequence to export expected witness sequence
- hintFunction now returns an error instead of panic
- added sanity checks in plonk solver
- plonk support for hints ok
- added Hints data struct in SparseR1CS
- added ToHTML on SparseR1CS
-  debugInfoComputation and debugInfoAssertion in a cs
- cs.Println now supports structures with Variables
- added assertions in ToHTML template
- debug info is displayed when div by 0 happens (Groth16 only)
- test circuit for AssertIsDifferent
- added skelleton for hintFunctions in r1cs
- IsZero is now implemented with 3 constraints fixes [#132](https://github.com/consensys/gnark/issues/132)
- **frontend:** added cs.NewHint
- **groth16:** added dummyInifinty counts for dummySetup accurate sizes generation of pk
- **plonk:** isZero with advice wire OK, binaryDec NOK
- **r1cs:** replaced isZero solver path by solver hint
- **r1cs:** removed binaryDec solving method in favor of cs.NewHint

### Fix
- hint functions use compiled.Term instead of variable IDs only
- fix previous fix
- remove debug stack trace from frontend error
- ProvingKey marshaling test with infinity flags, fixes [#141](https://github.com/consensys/gnark/issues/141)
- return empty slice when R1CS.Solve fails
- remove references to assertions in html templates
- sparseR1CS rebuilds hint map when deserializing
- ensure frontend.ConstraintSystem is not modified by compile process
- added Virtual variables to avoid creating wires for boolean constraints
- comment fuzz test
- fix serialization test. ensure we init hints when reading R1CS
- fixed error msg in Inverse, Div for groth16 fuzzer
- initialize InfinityX in dummy setup
- can constraint linear expressions to be boolean fixes [#136](https://github.com/consensys/gnark/issues/136)
- BinaryDecomposition solving assumes bits are in L, which is fine
- removed fmt.Println trace in setup
- **eddsa:** addition of isOnCurve check
- **eddsa:** S not splitted (s<r) + 2,3 Double instead of [cofactor]G
- **groth16:** Prove with force flag wasn't doing much since msm had lots of zeroes in input
- **test:** integration test failing due to missing witness assignment

### Frontend
- **groth16:** ensure R (as in LRO) as less variables than L

### Perf
- **groth16:** filter wire values after solve in go routines
- **groth16:** filter A and B for infinity points
- **groth16:** use batch invert in groth16.Setup

### Refactor
- groth16.Prove and plonk.Prove takes backend.ProverOption as parameter
- save hints in a map in ConstraintSystem instead of slice
- factorized structs between compiled.SparseR1Cs and compiled.R1CS
- remove NbConstraints from R1CS
- move mHints to compiled R1CS and SparseR1CS
- factorize logging between R1CS and SparseR1CS
- separated hint functions in their own file
- factorizing code between R1CS and SparseR1CS
- introduced solution struct shared between R1CS and SparseR1CS
- remove assertions in Groth16 R1CS
- remove Assertions in compiled sparseR1CS
- removed r1c.SolvingMethod in favor of cs.NewHint
- moved assertions in cs_assertions.go
- cs.IsZero doesn't need curveID anymore

### Style
- printArg doesn't return error
- code cleaning in cs_to_r1cs_sparse.go
- fixed comment in popConstant
- cleaned eddsa
- in-place filtering of points in setup
- fixed comments on S in eddsa
- cleaned eddsa_test

### Test
- ensure frontend.Compile is deterministic
- added non regression for cs.Println and debugInfo traces
- integration test remove Public in favor of Good in test circuits
- replaced Fatal by Error in stat check test
- added circuit statistic non regression tests
- added plonk path to integration_test.go
- fix gofuzz compile error
- added test from [#136](https://github.com/consensys/gnark/issues/136)

### Pull Requests
- Merge pull request [#142](https://github.com/consensys/gnark/issues/142) from ConsenSys/frontend-println-tests
- Merge pull request [#139](https://github.com/consensys/gnark/issues/139) from ConsenSys/cs-hint
- Merge pull request [#134](https://github.com/consensys/gnark/issues/134) from ConsenSys/is-zero
- Merge pull request [#130](https://github.com/consensys/gnark/issues/130) from ConsenSys/groth16-setup-filter-inf
- Merge pull request [#131](https://github.com/consensys/gnark/issues/131) from ConsenSys/fix/i_128
- Merge pull request [#129](https://github.com/consensys/gnark/issues/129) from ConsenSys/fix/reduce_constraints_eddsa


<a name="v0.5.0"></a>
## [v0.5.0] - 2021-08-23
### Build
- updated to latest gnark-crypto
- remove 32bit test for now, add timeout for github action
- updated to latest gnark-crypto. use ecc.NextPowerOfTwo
- updated to latest gnark-crypto
- add github CI action checks
- updated to latest gnark-crypto
- updated to msm-cpus branh of gnark-crypto
- updated to latest gnark-crypto
- updated to latest gnark-crypto
- updated to latest gnark crypto. fixes [#120](https://github.com/consensys/gnark/issues/120)
- updated to latest gnark-crypto
- updated to latest gnark-crypto
- run go mod tidy
- updated to gnark-crypto v0.5.0
- make gosec happy by handling un-handled errors
- run go generate
- updated to latest gnark-crypto
- restored gnark-crypto version
- updated to latest gnark-crypto. kzg api refactor
- updated to latest gnark-crypto
- updated to latest gnark-crypto. plonk Setup now takes kzg SRS, and re-uses fft domain
- updated to latest gnark-crypto
- ran go:generate for bls24 after merging develop into sself
- ran go:generate for bls24-315 plonk marshal
- re-ran go:generate
- **staticheck:** commented debugInfoUnsetVariable

### Chore
- cleaned plonk bn254, removed old version

### Ci
- added -mod=mod fix, maybe?
- replace go test sum by go test, CI check

### Clean
- cosmetics in plonk.Verify

### Cleanup
- removed to_delete.go file

### Docs
- fix go report card link
- prepare release notes for v0.5.0
- fix go report card link
- updated doc link and logo on README.md

### Feat
- plonk as-in-the-paper implem for bn254
- LinearExpression implements Sort interface. replaced quickSort() by sort.Sort(...)
- remove term.CoeffValue and use constant coeff ID for special values instead
- addition of circuit to test determinism
- update gnark-crypto[@feat](https://github.com/feat)/kzg/multi_points
- addition of FiatShamir in std
- updated go.mod
- modified example/benchmark with setup and run options
- updated go.sum, use of external hash function in plonk
- plonk verifier uses kzg BatchVerifyMultiPoints
- remove serialization test by default in assert helper
- replaced individual ScalarMul by MultiExp in plonk.Verify
- addition of unit test for cyclo square in std/../e12.go
- add bls24-315 to gnark
- added example for plonk (exponentiate circuit)
- added NewCS and NewPublicData on plonk package, with io.ReaderFrom and io.WriterTo unimplemented interfaces
- blind of a, b, c OK for bn254
- added reference benchmarks for plonk
- added NbG1 and NbG2 apis on groth16 Proving and Verifying keys closes [#116](https://github.com/consensys/gnark/issues/116)
- call stack displayed when AssertIsEqual fails
- gnarkd circuit data structure extension to support both groth16 and plonk
- added PublicRaw marhsal methods, ignoring KZG for now
- added WriteTo and ReadFrom to SparseR1CS objects
- added Neg on frontend API, cleaned Neg in twistededwards
- official implem of plonk, verifier in progress
- moving to strongly typed kzg
- **plonk:** code gen, started modifying backend interfaces
- **plonk:** added convenient method to create a kzg SRS from a compiled constraint system
- **plonk:** setup check srs size against fft domain cardinality
- **plonk:** added VerifyingKey serialization
- **plonk:** added VerifyingKey serialization with test
- **plonk:** added ProvingKey serialization
- **plonk:** modified folded commitment of h
- **plonk:** blinded z, modified test circuits to have nbConstraints>8
- **plonk:** code gen for proof blinding
- **plonk:** added ProvingKey serialization test
- **plonk:** added InitKZG methods on ProvingKey and VerifyingKey

### Fix
- fixed gnarkd tests for kzg srs
- shuffleVariables in fuzz testing with bad offset
- restore benchmark/main.go
- don't close channels that are use in the select as they become always ready to receive
- groth16 prove missing chan close in one path
- groth16.Prove handles multiExp error returns
- fixed conflicts
- fixed conflicts
- restored benchmark/main.go
- avoid code gen for bw633 until feat/bw633 is merged
- invalid gnark struct tag options return error at compile time fixes [#111](https://github.com/consensys/gnark/issues/111)
- kzg srs size +3
- r1cs compilation is deterministic, fixes [#90](https://github.com/consensys/gnark/issues/90)
- fixes [#112](https://github.com/consensys/gnark/issues/112)
- removed unused error variable
- make go vet happy
- use of doubling formula instead of add(x,x) fixes [#114](https://github.com/consensys/gnark/issues/114)
- updated go.mod
- go.mod points to gnark-crypto[@develop](https://github.com/develop), fixes [#96](https://github.com/consensys/gnark/issues/96)
- SetupDummyCommitment calls with Proving and Verifying key in return
- added BLS24_315 in plonk constructors
- backend plonk bls24 process error
- create cbor decoder with MaxArrayElements set to max value
- regenerated code
- **frontend:** restored isBoolean logic to avoid dupplicate constraints. remove dangling variable thing
- **frontend:** set initial capacity for constraint system slices to 0
- **gnarkd:** kzg srs generation in test cases with correct size
- **plonk:** fixed error in ComputeH  when nbConstraints+nbPublicInputs<6

### Perf
- start computeZ earlier
- plonk prove remove most fft.BitReverse
- replaced string concat in frontend with strings.Builder
- remove clone in computeLinearizedPoly
- minor optim
- use batch inversion in plonk.computeZ
- plonk.computeLinearizedPolynomial with less polynomial clones
- improving parallelism in prover
- minor change
- make constraint slice initalCapacity an optional paremeter in compile
- plonk frontend replace map by slice, avoid few allocs and useless slice copies
- frontend.ConstraintSystem special values fast path for coefficients
- plonk frontend fast path for -1, 0 and 1 as coeffs
- reduced memory allocations in plonk frontend
- evalIDCosets shiftEval done in parallel, better trace
- shiftZ in parallel with other stuff
- remove useless copy in computeH
- sparseR1CS.Solve few times faster by avoiding Div at constraint solving
- hunting memallocs - remove evaluid and uuid in evalConstraint
- blinded polynomials re-use input polynomial memory
- remove polynomial clone in foldedH computation
- remove bitReverse in evalIDCosets
- use ecc.CPUSemaphore in kzg.Commit to ensure pretty trace
- remove one additional plynomoial clone
- parralelize computeH
- **experimental:** start too many go routines in Prover
- **frontend:** rewrote linear expression reduce in place instead of multiple map allocs
- **frontend:** minor adjustements
- **frontend:** rewrote cs.ToBinary to avoid unecessary func calls
- **plonk:** shiftEval done without copy or bitReverse
- **plonk:** compute shifted Z element on the fly without allocating a new polynomial
- **plonk:** partly parallel computeBlindedZ
- **plonk:** when doing fft on domainH with coset, don't scale zero values
- **plonk:** minor tweaks, removing un-needed bitreverse and mem allocs
- **scs:** rewrote solve O
- **scs:** sparse r1cs have fast path for special coeffs operations
- **std:** adds E2/E12 square and cyclo square in E12 (used FinalExp)

### Refactor
- removed gnarkd and examples/benchmark
- mimc uses Write(data) then Sum() instead of Sum(data)
- Hash-->Sum in mimc gadget
- added deriveRandomness utility func in plonk verifier
- strongly typed KZG, Plonk test passing
- renamed Groth16 protobuf service to ZKSnark
- **groth16:** SizePublicWitness to NbPublicWitness

### Style
- cleaning plonk.prove
- renamed GetCurveID() to CurveID() on groth16 objects
- minor change
- more cleaning in cs_to_r1cs_sparse.go
- start clean up cs_to_r1cs_sparse.go
- replaced very long suite of if else by switch
- remove Bis suffixes in plonk prover
- moved derivedrandomness
- cleaning plonk APIs
- go fmt
- modified comment on reduce
- **kzg:** factorized some methods in the prover
- **plonk:** use close(chan) instead of send twice on it

### Test
- test for Fiat Shamir gadget
- added reference frontend.Compile benchmarks
- fix circuitID path
- **gnarkd:** plonk end to end pass
- **gnarkd:** gRPC test run in parallel w multiple curves

### Pull Requests
- Merge pull request [#126](https://github.com/consensys/gnark/issues/126) from ConsenSys/develop
- Merge pull request [#124](https://github.com/consensys/gnark/issues/124) from ConsenSys/groth16-stats-pk-vk
- Merge pull request [#113](https://github.com/consensys/gnark/issues/113) from ConsenSys/feat/gnarkd/plonk
- Merge pull request [#117](https://github.com/consensys/gnark/issues/117) from ConsenSys/perf/recursive-proof
- Merge pull request [#108](https://github.com/consensys/gnark/issues/108) from ConsenSys/feat/plonk/clean_verifier
- Merge pull request [#104](https://github.com/consensys/gnark/issues/104) from ConsenSys/bls24-315
- Merge pull request [#95](https://github.com/consensys/gnark/issues/95) from ConsenSys/fix/deterministic_r1cs


<a name="v0.4.0"></a>
## [v0.4.0] - 2021-04-29
### Build
- updated gnark-crypto in go.mod
- updated to latest bavard and gnark-crypto
- updated to latest gnark-crypto
- moved solidity integration tests in github.com/consensys/gnark-tests
- updated .gitignore
- added .gitlint file

### Ci
- added integration fuzz test in backend/groth16/fuzz_test.go

### Doc
- fixed typo ([#63](https://github.com/consensys/gnark/issues/63)) in README.md

### Docs
- preparing v0.4.0 release with new README.md and CHANGELOG.md
- added comments for the splitting of S in eddsa
- updated package level godoc
- fixed comments on wire ordering for sparse r1cs
- **plonk:** fixed doc for computeH

### Eddsa
- bw761 blinding factor and private key size are consistant with the field size

### Feat
- added funcitons for proving PLONK's claim 1 (bn256), not tested
- mock polynomial commitments for all curves + templates
- added code gen for placeholder feature
- added Fiat Shamir for plonk
- added intefaces for polynomial commitments
- updated go.mod (points to gnark-crypto[@hotfix](https://github.com/hotfix)/issue_36)
- support for batch proofs opening at single point
- **fft:** fft/fftInv now works on abitrary cosets (bounded by maxOrder)
- **gnarkd:** exposing gnark APIs through RPCs ([#54](https://github.com/consensys/gnark/issues/54))
- **gnarkd:** added optional TTL in CreateProveJobRequest
- **gnarkd:** added CancelProveJob method
- **gnarkd:** added ListProveJob method
- **plonk:** addition of placeholders to handle public inputs
- **plonk:** proof that Z starts at 1 done, tests ok (bn256)
- **plonk:** added commitments verification in plonk's verifier
- **plonk:** plonk tests are now executed via generic code in assert
- **plonk:** proof of permutation done (bn256), tests ok
- **plonk:** templates for plonk setup/prove/verify
- **plonk:** permutation proof part implemented, to be tested
- **plonk:** claim 1 of plonk prover works correctly(bn256)
- **plonk:** templates for testing prover (to be moved to backend/plonk)
- **plonk:** addition of templates + code gen for computing Z
- **plonk:** applying previous commit on all curves with code gen
- **plonk:** code gen for the previous fix
- **plonk:** addition of plonk generic code in backend/
- **plonk:** addition of the permutation in the setup
- **plonk:** H is split as h1+X**m*h2+X**2m*h3
- **plonk:** polynomial accumulating partial permutation OK (bn256)
- **plonk cs:** adding functionality to convert a constraint system to PLONK constraints ([#56](https://github.com/consensys/gnark/issues/56))

### Fix
- cs.Println doesn't trigger panic anymore
- fixed Groth16 snark circuit according to previous commit
- inverse and div in frontend had some variable ID offset issues ([#62](https://github.com/consensys/gnark/issues/62))
- fixed snark circuit for bls377 pairing
- removed dead function (getOneWire) in cs.go
- fixes [#88](https://github.com/consensys/gnark/issues/88)
- go mod update + fix bad import path with gofuzz build tag
- bn256 -> bn254
- updated go.mod to latest gnark-crypto on develop
- r1cs serialization test doesn't need to check logger io.Writer output
- ensure that L.id=M[0].id and R.id=M[1].id in a sparse_r1c
- added go.sum
- removed unreachable code piece
- typo in readme.md fixes [#60](https://github.com/consensys/gnark/issues/60)
- **fft:** fixed the ordering of cosets factor according to DIF/DIT
- **gnarkd:** ListProveJob test didn't account for other test adding jobs to the queue
- **plonk:** removed useless multiplication by L in the prover
- **plonk:** fixed size of permutation, it's now a power of 2
- **plonk:** fixed formula for Li->Li+1 in verify ... (bn256)
- **r1cs:** TestSerialization running sequentially
- **r1cs:** moved bytes.Buffer in t.Run (TestSerialization)
- **r1cs_sparse:** ensure that Solve never returns nil, err

### Groth16
- VerifyingKey data structure change to ensure compatibility with other impl and Solidity in Ethereum. Serialization format change.

### Integration_test
- added witness serialization tests

### Refactor
- gurvy -> gnark-crypto
- use gnark-crypto polynomial and accumulator packages
- bls381 -> bls12381
- bls377 -> bls12377
- templates for groth16 are in a dedicated folder
- moved crypto/utils in frontend/, for now
- bw761 -> bw6761
- bn256 -> bn254, bls377 -> bls12-377, etc. following gnark-crypto v0.4.0
- bn256 -> bn254
- first step for gurvy -> gnark-crypto
- removed the Curve field in the R part of eddsa signature
- verification of openings return an error instead of bool
- removed challenge for batch opening/verifying poly commit
- claimed value is inside an opening proof
- added method ClaimedValue on Proof interface
- **fft:** nbCosets --> Depth, easier for interpreting cosets
- **plonk:** lrozh (in the proof) are now in a single slice
- **plonk:** challenges are built in init() until Fiat Shamir
- **plonk:** suffix raw added to plonk API
- **plonk:** setup, prove, verify take frontend.Circuit as witness
- **plonk:** prove, verifiy now return error

### Style
- simplified findUnsolvedVariable in SparseR1CS
- untrack to_delete_bn256.go (used for printing stuff)
- removed comments of the previous fft in groth16 prove
- **plonk:** removed comments (used for testing) in setup

### Test
- added frontend and backend fuzz.go, go-fuzz compatible format
- added cs.Println must not panic base test

### Pull Requests
- Merge pull request [#94](https://github.com/consensys/gnark/issues/94) from ConsenSys/develop
- Merge pull request [#93](https://github.com/consensys/gnark/issues/93) from ConsenSys/hotfix/fft_groth16
- Merge pull request [#92](https://github.com/consensys/gnark/issues/92) from ConsenSys/feat/fiat_shamir
- Merge pull request [#89](https://github.com/consensys/gnark/issues/89) from ConsenSys/fix/eddsa
- Merge pull request [#86](https://github.com/consensys/gnark/issues/86) from ConsenSys/docs/godoc
- Merge pull request [#65](https://github.com/consensys/gnark/issues/65) from ConsenSys/refactor/gnark-crypto
- Merge pull request [#64](https://github.com/consensys/gnark/issues/64) from ConsenSys/feat/plonk_prover
- Merge pull request [#58](https://github.com/consensys/gnark/issues/58) from ConsenSys/feat/fft_cosets
- Merge pull request [#57](https://github.com/consensys/gnark/issues/57) from ConsenSys/feature/gnarkd
- Merge pull request [#53](https://github.com/consensys/gnark/issues/53) from ConsenSys/serialization/witness
- Merge pull request [#51](https://github.com/consensys/gnark/issues/51) from ConsenSys/eddsa_cleanup
- Merge pull request [#46](https://github.com/consensys/gnark/issues/46) from ConsenSys/experimental/solidity
- Merge pull request [#48](https://github.com/consensys/gnark/issues/48) from ConsenSys/issue_45


<a name="v0.3.8"></a>
## [v0.3.8] - 2020-12-23

<a name="v0.3.7"></a>
## [v0.3.7] - 2020-12-22

<a name="v0.3.6"></a>
## [v0.3.6] - 2020-12-22
### Features
- **profiling:** Adds a way to print the number of constraints in the circuit

### Fix
- **typo:** insertion -> assertion

### R1cs
- implemented serialization interfaces

### Serialization
- using gurvy.Encoder and gurvy.Decoder. Added benchmark and assert helpers
- added proving key
- added for fft domain
- replaced some int by uint64 to avoid ambiguity in serialization protocols
- gnark object implement io.ReaderFrom and io.WriterTo

### Wip
- updating to latest gurvy

### Pull Requests
- Merge pull request [#42](https://github.com/consensys/gnark/issues/42) from ConsenSys/linearexp
- Merge pull request [#41](https://github.com/consensys/gnark/issues/41) from AlexandreBelling/feature/cs-nb-constraints
- Merge pull request [#38](https://github.com/consensys/gnark/issues/38) from ConsenSys/hotfix/discard_secret


<a name="v0.3.5"></a>
## [v0.3.5] - 2020-10-19

<a name="v0.3.4"></a>
## [v0.3.4] - 2020-10-19

<a name="v0.3.3"></a>
## [v0.3.3] - 2020-09-23

<a name="v0.3.1"></a>
## [v0.3.1] - 2020-09-22

<a name="v0.3.0"></a>
## [v0.3.0] - 2020-09-22

<a name="v0.3.0-alpha"></a>
## [v0.3.0-alpha] - 2020-09-15
### Backend
- interface to big.Int added minimalist test
- moved  generated curve specific backends into internal to forbid library user to directly import it

### Backends
- restored bw761 groth16 code generation path

### Circleci
- added step to ensure no generated files are modified by hand
- change cache key
- new workflow with more insight on unit tests

### Encoding
- switch from gob to cbor

### Frontend
- remove Context object, mostly used as a curve.ID wrapper
- added frontend.Compile benchmark for reference in further modifications
- allocate slice capacity when known
- added Circuit and CircuitVariable interfaces. Can now assign values with compiler check (no more strings). Rollup tests OK

### Gnark
- fixing few staticcheck warnings
- input file is now json. accepts 0x hex and decimal repr for assignment to a circuit

### Groth16
- updated to latest gurvy and added go routines in prover workflow. WIP need to be benchmarked
- reorganize Setup to use gurvy.BatchScalarMultiplication api
- prover, removed appends in prover that resulted in array copies
- added test for reference circuit (non short path)
- restored reference circuit for benchmarking purposes
- fix possible starvation issue in Prover -- there existed a world were go routine may wait for ever for the tokenn causing a timeout in tests

### R1cs
- added GetNbCoefficients and GetNbWires to interface
- Solve takes typed arguments

### Refactor
- remove Gadget suffix from many structs
- checkpoint

### Wip
- investigating millerloop result in snark circuit
- frontend.NewConstraintSystem is now private. ToR1CS() is private. test circuits moved to internal. gadgets and test circuits use frontend.Compile()

### Pull Requests
- Merge pull request [#26](https://github.com/consensys/gnark/issues/26) from ConsenSys/gadget_cleanup
- Merge pull request [#23](https://github.com/consensys/gnark/issues/23) from ConsenSys/tagless_refactor


<a name="v0.2.1-alpha"></a>
## [v0.2.1-alpha] - 2020-06-18

<a name="v0.2.0-alpha"></a>
## [v0.2.0-alpha] - 2020-06-05
### Backend
- cleaned asserts
- generating backend/static/CURVE submodules from same template
- remove curve generated code, keep only build tag version

### Circleci
- added missing goimports indirect dependency
- run go generate to ensure repo consistency in CI
- test full repo with each build tag

### Cmd
- removed wip export command for now

### Examples
- back at root of repo

### Frontend
- fixed division-by-constant constraint

### Gnark
- integration test now uses test circuits defined in internal/tests/circuits

### Groth16
- fix reference to large reference test circuit in groth16
- remove assertion when checking number of inputs and fix missing curve in testdata path
- add point check infinity in assert
- VerifyingKey stores K in Affine, not Jacobian. Fixes [#18](https://github.com/consensys/gnark/issues/18)

### Integration_test
- be nice with circleci, do not add large circuit into integration test

### Refactor
- separated frontend and backend, code generate typed backend and tests for groth16. yes that's a big commit

### Templates
- minor code cleaning

### WIP
- using big.Int in frontend to avoid build tags

### Pull Requests
- Merge pull request [#22](https://github.com/consensys/gnark/issues/22) from ConsenSys/refactor
- Merge pull request [#21](https://github.com/consensys/gnark/issues/21) from Mikerah/patch-1


<a name="v0.1.0-alpha"></a>
## v0.1.0-alpha - 2020-03-06
### Pull Requests
- Merge pull request [#11](https://github.com/consensys/gnark/issues/11) from nkeywal/exGadget
- Merge pull request [#8](https://github.com/consensys/gnark/issues/8) from ConsenSys/internal-curve-tests
- Merge pull request [#7](https://github.com/consensys/gnark/issues/7) from ConsenSys/develop Fixed [#6](https://github.com/consensys/gnark/issues/6)
- Merge pull request [#5](https://github.com/consensys/gnark/issues/5) from ConsenSys/go1.14_deps


[Unreleased]: https://github.com/consensys/gnark/compare/v0.8.1...HEAD
[v0.8.1]: https://github.com/consensys/gnark/compare/v0.8.0...v0.8.1
[v0.8.0]: https://github.com/consensys/gnark/compare/v0.7.1...v0.8.0
[v0.7.1]: https://github.com/consensys/gnark/compare/v0.6.5...v0.7.1
[v0.6.5]: https://github.com/consensys/gnark/compare/v0.7.0...v0.6.5
[v0.7.0]: https://github.com/consensys/gnark/compare/v0.6.4...v0.7.0
[v0.6.4]: https://github.com/consensys/gnark/compare/v0.6.3...v0.6.4
[v0.6.3]: https://github.com/consensys/gnark/compare/v0.6.2...v0.6.3
[v0.6.2]: https://github.com/consensys/gnark/compare/v0.6.1...v0.6.2
[v0.6.1]: https://github.com/consensys/gnark/compare/v0.6.0...v0.6.1
[v0.6.0]: https://github.com/consensys/gnark/compare/v0.5.2...v0.6.0
[v0.5.2]: https://github.com/consensys/gnark/compare/v0.5.1...v0.5.2
[v0.5.1]: https://github.com/consensys/gnark/compare/v0.5.0...v0.5.1
[v0.5.0]: https://github.com/consensys/gnark/compare/v0.4.0...v0.5.0
[v0.4.0]: https://github.com/consensys/gnark/compare/v0.3.8...v0.4.0
[v0.3.8]: https://github.com/consensys/gnark/compare/v0.3.7...v0.3.8
[v0.3.7]: https://github.com/consensys/gnark/compare/v0.3.6...v0.3.7
[v0.3.6]: https://github.com/consensys/gnark/compare/v0.3.5...v0.3.6
[v0.3.5]: https://github.com/consensys/gnark/compare/v0.3.4...v0.3.5
[v0.3.4]: https://github.com/consensys/gnark/compare/v0.3.3...v0.3.4
[v0.3.3]: https://github.com/consensys/gnark/compare/v0.3.1...v0.3.3
[v0.3.1]: https://github.com/consensys/gnark/compare/v0.3.0...v0.3.1
[v0.3.0]: https://github.com/consensys/gnark/compare/v0.3.0-alpha...v0.3.0
[v0.3.0-alpha]: https://github.com/consensys/gnark/compare/v0.2.1-alpha...v0.3.0-alpha
[v0.2.1-alpha]: https://github.com/consensys/gnark/compare/v0.2.0-alpha...v0.2.1-alpha
[v0.2.0-alpha]: https://github.com/consensys/gnark/compare/v0.1.0-alpha...v0.2.0-alpha
