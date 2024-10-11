
# Synopsis

 A TLA+ specification of a simplified Tendermint consensus algorithm, tuned for
 fork accountability. The simplifications are as follows:

- the protocol runs for one height, that is, one-shot consensus

- this specification focuses on safety, so timeouts are modelled with
   with non-determinism

- the proposer function is non-determinstic, no fairness is assumed

- the messages by the faulty processes are injected right in the initial states

- every process has the voting power of 1

- hashes are modelled as identity

 Having the above assumptions in mind, the specification follows the pseudo-code
 of the Tendermint paper: <https://arxiv.org/abs/1807.04938>

 Byzantine processes can demonstrate arbitrary behavior, including
 no communication. However, we have to show that under the collective evidence
 collected by the correct processes, at least `f+1` Byzantine processes demonstrate
 one of the following behaviors:

- Equivocation: a Byzantine process sends two different values
     in the same round.

- Amnesia: a Byzantine process locks a value, although it has locked
     another value in the past.

# TLA+ modules

- [TendermintAcc_004_draft](TendermintAcc_004_draft.tla) is the protocol
   specification,

- [TendermintAccInv_004_draft](TendermintAccInv_004_draft.tla) contains an
   inductive invariant for establishing the protocol safety as well as the
   forking cases,

- `MC_n<n>_f<f>`, e.g., [MC_n4_f1](MC_n4_f1.tla), contains fixed constants for
   model checking with the [Apalache model
   checker](https://github.com/informalsystems/apalache),

- [TendermintAccTrace_004_draft](TendermintAccTrace_004_draft.tla) shows how
   to restrict the execution space to a fixed sequence of actions (e.g., to
   instantiate a counterexample),

- [TendermintAccDebug_004_draft](TendermintAccDebug_004_draft.tla) contains
   the useful definitions for debugging the protocol specification with TLC and
   Apalache.

# Reasoning about fork scenarios

The theorem statements can be found in
[TendermintAccInv_004_draft.tla](TendermintAccInv_004_draft.tla).

First, we would like to show that `TypedInv` is an inductive invariant.
Formally, the statement looks as follows:

```tla
THEOREM TypedInvIsInductive ==
    \/ FaultyQuorum
    \//\ Init => TypedInv
      /\ TypedInv /\ [Next]_vars => TypedInv'
```

When over two-thirds of processes are faulty, `TypedInv` is not inductive.
However, there is no hope to repair the protocol in this case. We run
[Apalache](https://github.com/informalsystems/apalache) to prove this theorem
only for fixed instances of 4 to 5 validators.  Apalache does not parse theorem
statements at the moment, so we ran Apalache using a shell script. To find a
parameterized argument, one has to use a theorem prover, e.g., TLAPS.

Second, we would like to show that the invariant implies `Agreement`, that is,
no fork, provided that less than one third of processes is faulty. By combining
this theorem with the previous theorem, we conclude that the protocol indeed
satisfies Agreement under the condition `LessThanThirdFaulty`.

```tla
THEOREM AgreementWhenLessThanThirdFaulty ==
    LessThanThirdFaulty /\ TypedInv => Agreement
```

Third, in the general case, we either have no fork, or two fork scenarios:

```tla
THEOREM AgreementOrFork ==
    ~FaultyQuorum /\ TypedInv => Accountability
```

# Model checking results

Check the report on [model checking with Apalache](./results/001indinv-apalache-report.md).

To run the model checking experiments, use the script:

```console
./run.sh
```

This script assumes that the apalache build is available in
`~/devl/apalache-unstable`.
