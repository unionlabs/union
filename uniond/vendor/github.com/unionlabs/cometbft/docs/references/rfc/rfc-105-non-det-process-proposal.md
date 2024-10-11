# RFC 105: Allowing Non-Determinism in `ProcessProposal`

## Changelog

- 2023-07-27: Initial proposal as a GitHub issue ([#1174][1174]) (@BrendanChou)
- 2023-09-22: First version of this text, draft coming from knowledge base (@sergio-mena)

## Abstract

The new methods `PrepareProposal` and `ProcessProposal` offered by ABCI 1.0
are a powerful tool that enables new use cases for applications,
but they require application developers to use them with care.
Depending on how those methods are implemented, consensus liveness properties
might be compromised, thus risking hard-to-diagnose chain halts.

The [ABCI 1.0 specification][abci-spec] defines a set of requirements on the application.
If application developers fulfill these requirements
in their implementation of `PrepareProposal` and `ProcessProposal`,
consensus liveness will be guaranteed.
These requirements are good enough for most applications:
they are simple to understand and relatively easy to check whether an application fulfills them.

However, there may be applications that are unable to fulfill those requirements.
In this document we discuss such applications and propose weaker requirements that may be fulfilled instead.

## Background

### Consensus Properties

Let us first refresh some theoretical concepts needed to understand the rest of this text.
We define $valid(v, s)$ a function
whose output depends exclusively on its inputs, i.e., a mathematical function.
The inputs are the value proposed $v$ (i.e., a block)
and a state $s$ of the blockchain.
Byzantine Fault Tolerant (BFT) Consensus is usually specified by the following properties.
For every height $h$ and $s_{h-1}$, where $s_{h-1}$ is the state of the blockchain
after applying the block decided in height $h-1$:

- _agreement_: no two correct processes decide differently in $h$.
- _validity_: function $valid(v, s_{h-1})$, where $v$ is the block decided in $h$, always returns _true_.
- _termination_: all correct processes eventually decide in $h$.

The consensus algorithm implemented in CometBFT (Tendermint) fulfills these properties.

### ABCI Interface

In the version of ABCI (v0.17.0) that existed before ABCI 1.0 and 2.0 (a.k.a. ABCI++),
the implementation of function $valid(v, s)$ was totally internal to CometBFT.
Technically, the application's part of the state $s$ was not considered by function $valid(v, s)$.
Thus, the application had no direct say on the validity of a block,
although it could (and still can) indirectly influence the contents of blocks via the (best-effort) ABCI method `CheckTx`
(by rejecting transactions, so that they are not included in blocks produced by correct proposers).

With the evolution of ABCI to ABCI 1.0 and 2.0, CometBFT's implementation of
function $valid(v, s)$ has now two components:

- the validity checks performed directly by CometBFT on blocks
  (block format, hashes, etc; the same as in ABCI v0.17.0)
- the validity checks that now the application can perform as part of `ProcessProposal`;
  i.e., `ProcessProposal` is now part of $valid(v, s)$
  and may use the application's part of state $s$ in validating $v$.

With the new structure of the implementation of function $valid(v, s)$:

- consensus _agreement_ is not affected and all processes are still required to agree on the same value
- the consensus _validity_ property is not affected since we are changing the
  internals of function $valid(v, s)$; consensus _validity_ just requires this function to be true

However, the new structure of the implementation of function $valid(v, s)$
may affect consensus _termination_, as some implementations of `ProcessProposal` might reject values
that CometBFT's internal validity checks would otherwise accept.
In short, $valid(v, s)$ is more restrictive with ABCI++.

This document focuses on how consensus _termination_ is affected
by the new structure of function $valid(v, s)$,
in particular, the different implementations of `ProcessProposal`.

### ABCI 1.0 (and 2.0) Specification

The [ABCI 1.0 specification][abci-spec] imposes a set of new requirements on the application
so that its implementation of `PrepareProposal` and `ProcessProposal` does not compromise consensus _termination_,
given the current consensus algorithm implemented in CometBFT (called Tendermint, and described in the [arXiv paper][arxiv]).
In contrast to $valid(v, s)$, which is defined as a mathematical function used for consensus's formal specification,
`PrepareProposal` and `ProcessProposal` are understood as _software functions_ (namely, Go function callbacks) in CometBFT.
We reproduce here the requirements in the ABCI 1.0 (and 2.0) specification that are relevant for this discussion.

Let $p$ and $q$ be two correct processes.
Let $r_p$ be a round of consensus at height $h$ where $p$ is the proposer.
Let $s_{p,h-1}$ be $p$'s application's state committed for height $h-1$.
In other words, $s_{p,h-1}$ is $p$'s view of $s_{h-1}$.
Let $v_p$ be the block that $p$'s CometBFT passes
on to the application
via `RequestPrepareProposal` as proposer of round $r_p$, height $h$,
known as the _raw proposal_.
Let $u_p$ be the (possibly modified) block that $p$'s application
returns via `ResponsePrepareProposal` to CometBFT in round $r_p$, height $h$,
known as the _prepared proposal_.

* Requirement 3 [`PrepareProposal`, `ProcessProposal`, coherence]: For any two correct processes $p$ and $q$
  if $q$'s CometBFT calls `RequestProcessProposal` on $u_p$,
  $q$'s application returns _Accept_ in `ResponseProcessProposal`.

* Requirement 4 [`ProcessProposal`, determinism-1]:
  `ProcessProposal` is a (deterministic) function of the current
  state and the block being processed.
  In other words, for any correct process $p$, and any arbitrary block $u$,
  if $p$'s CometBFT calls `RequestProcessProposal` on $u$ at height $h$,
  then $p$'s application's acceptance or rejection **exclusively** depends on $u$ and $s_{p,h-1}$.

* Requirement 5 [`ProcessProposal`, determinism-2]:
  For any two correct processes *p* and *q*, and any arbitrary
  block $u$,
  if CometBFT instances at $p$ and $q$ call `RequestProcessProposal` on $u$ at height $h$,
  then $p$'s application accepts $u$ if and only if $q$'s application accepts $u$.
  Note that this requirement follows from the previous one and consensus _agreement_.

The requirements expressed above are good enough for most applications using ABCI 1.0 or 2.0.
They are simple to understand and it is relatively easy to check whether an application's
implementation of `PrepareProposal` and `ProcessProposal` fulfills them.
All applications that are able to enforce these properties do not need to reason about
the internals of the consensus implementation: they can consider it as a black box.
This is the most desirable situation in terms of modularity between CometBFT and the application.

The easiest (and thus canonical) way to ensure these requirements is to make sure
that `PrepareProposal` only prepares blocks $v$ that satisfy (mathematical) function $valid(v, s)$,
including the validation performed by correct processes in `ProcessProposal`.
However, `PrepareProposal` and `ProcessProposal` MAY also use other input in their
implementation and CometBFT will still guarantee consensus termination, _as long as
these implementations still ensure the requirements_.

## Discussion

### Breaking Determinism/Coherence Requirements

This document is dealing with the case when an application cannot guarantee
the coherence and/or determinism requirements as stated in the ABCI 1.0 specification.

An example of this is when `ProcessProposal` needs to take inputs from third-party entities
(e.g. price oracles) that are not guaranteed to provide exactly the same values to
different processes during the same height.
Another example is when `ProcessProposal` needs to read the system clock in order to perform its checks
(e.g. Proposer-Based Timestamp when expressed as an ABCI 1.0 application).

### Problem Statement

In principle, if an application's implementation of `PrepareProposal` and `ProcessProposal`
is not able to fulfill coherence and determinism requirements,
CometBFT cannot guarantee consensus _termination_ in all runs of the system.
For instance, think of an application whose implementation of `ProcessProposal`
always rejects values, which violates coherence.

> ⚠️ Warning ⚠️ 

If application designers decide to follow this road, they must consider **both**
CometBFT and their application as one monolithic block, in order to reason about termination.
They thus lose the modularity provided when fulfilling the ABCI 1.0 requirements.
Remember that CometBFT's consensus algorithm (Tendermint) is a well-known algorithm that
has been studied, reviewed, formally analyzed, model-checked, etc.
The combination of CometBFT and an arbitrary application as one single algorithm cannot
leverage that extensive body of research applied to the Tendermint algorithm.
This situation is risky and undesirable.

So, the questions that arise are the following.
Can we come up with a set of weaker requirements
that applications unable to fulfill the current ABCI 1.0 requirements
can still fulfill?
With this new set of requirements, can we still keep the reasoning on termination modular?
Is this set of weaker requirements still strong enough to guarantee consensus _termination_?

### Solution Proposed

#### Modified Consensus _validity_

Function $valid(v, s)$, as explained above, exclusively depends on its inputs
(a block, and the blockchain state at the previous height: $s_{h-1}$).
So it is always supposed to provide the same result when called at the same height for the same inputs,
no matter at which process.
This was the main reason for introducing the determinism requirements on `ProcessProposal`
in the ABCI 1.0 specification.

If we are to relax the determinism requirements on the application,
we first need to modify function $valid(...)$ to be of the form $valid(v, s, x_p)$,
where $x_p$ is local to process $p$.
Parameter $x_p$, unknown to Comet, represents non-deterministic input used by the application.
As $x_p$ may be different from $x_q$ for two processes $p$ and $q$, allowing `ProcessProposal`
to use $x_p$ may break Requirements 3 to 5.

Consensus _validity_ property is then modified as follows:

- _weak validity_: function $valid(v, s_{h-1}, x_p)$ has returned _true_ at least once
  by a correct process for the decided block $v$.

#### Eventual Requirements

We now relax the relevant ABCI 1.0 requirements in the following way.

* Requirement 3b [`PrepareProposal`, `ProcessProposal`, eventual coherence]:
  There exists a time $ts_{h}$ for every height $h$ such that,
  for any two correct processes $p$ and $q$ and any round $r_p$ in height $h$ starting after $ts_{h}$,
  if $q$'s CometBFT calls `RequestProcessProposal` on $u_p$,
  $q$'s application returns _Accept_ in `ResponseProcessProposal`.

* The determinism-related requirements, namely requirements 4 and 5, are removed.

We call $ts_{h}$ the coherence-stabilization time for height $h$.

If we think in terms of $valid(v, s, x_p)$, notice that it is the application's responsibility
to ensure 3b, that is, the application designers need to prove that the $x_p$ values at correct processes
are evolving in a way that eventually `ResponseProcessProposal` returns _Accept_ at all correct processes
that call `RequestProcessProposal` for height $h$ after $ts_{h}$.

For instance, in Proposer-Based Timestamp, $x_p$ can be considered to be process $p$'s local clock,
and having clocks synchronized is the mechanism ensuring eventual acceptance of a proposal.
Finally, it is worth noting that _weak validity_ requires just one correct processes while requirement 3b
refers to all correct processes.

#### Modifications to the Consensus Algorithm

The Tendermint algorithm as described in the [arXiv paper][arxiv],
and as implemented in CometBFT up to version `v0.38.0`,
cannot guarantee consensus _termination_ for applications
that just fulfill requirement 3b (eventual coherence), but do not fulfill requirements 3, 4, and 5.

For the sake of simplicity, and without loss of generality, we assume all validators have equal voting power.
We need the following modifications (in terms of the algorithm as described in page 6 of the arXiv paper):

- remove the evaluation of `valid(v)` in lines 29, 36 and 50 (i.e. replace `valid(v)` by `true`)
- modify line 23 as follows

> _\[Original\]_ &nbsp; 23: **if** $valid(v) \land (lockedRound_p = −1 \lor lockedValue_p = v)$ **then**

> _\[Modified\]_
>
> &nbsp; 23a: $validValMatch := (validRound_p \neq -1 \land validValue_p = v)$
>
> &nbsp; 23b: **if** $[lockedRound_p = −1 \land (validValMatch \lor valid(v))] \lor lockedValue_p=v$ **then**

- If we consider the new _weak validity_ property,
  the occurrences of `valid(v)` that we removed had become redundant,
  so removing them does not affect the ability of the algorithm to fulfill consensus properties
  (replacing _validity_ by _weak validity_).
- Regarding line 23, the changes have the following goals:
  - If `v` matches the block we have as $validValue_p$ or $lockedValue_p$, we skip the call
    to `valid(v)`.
    The rationale is that, by the algorithm, if we have `v` as $validValue_p$ or $lockedValue_p$
    we have received it from at least $2f + 1$ prevotes for $v$ (line 36).
  - If the previous condition is not met, then we call `valid(v)`.
  - The modification to the conditions must _only_ affect the decision whether to call `valid(v)` or not.
    It must not modify which part of the "if" should be taken.

Notice we have kept the original `valid(v)` notation, but it stands for the more general $valid(v, s, x_p)$.
These algorithmic modifications have also been made to CometBFT (on branch `main`)
as part of issues [#1171][1171], and [#1230][1230].

#### Going further in the modifications

The previous section describes the modifications to the algorithm we made to CometBFT.
However, we can go further in theory: these modifications are sufficient but not necessary.
The minimal necessary condition for the Tendermint algorithm to ensure consensus _weak validity_
(as mentioned [here][valid-further1] and [here][valid-further2]) while skipping `valid(v)`
is having received valid prevote messages for the proposed block from $f + 1$ validators.
These prevotes don't even need to belong to the same round, although they need to be for the current height.
We decided not to go this far in the modifications to CometBFT for two reasons:

* It is uncertain what practical advantages it would bring.
* It would require new custom data structures to keep track of prevotes for a block across rounds,
  adding complexity to our implementation.

### On Crash-Recovery

Applications that opt for eventual coherence because their `ProcessProposal` implementation can be non-deterministic,
may encounter an additional problem for recovering nodes.
When replaying messages for the unfinished consensus, they may run `ProcessProposal` twice for the same height and round:
one before the crash and one after the crash.
Since `ProcessProposal` can now be non-deterministic, it is possible that both executions produce different outputs.

This problem is analogous to the one already identified for `PrepareProposal`, which can always be non-deterministic,
and is captured in [#1035][1035].
So, similarly to `PrepareProposal`, the current CometBFT implementation of the `PrivValidator`
includes a double-signing protection mechanism that will prevent the recovering node from sending conflicting messages
for the same height, round, and step.
This mechanism will drop any prevote message resulting from `ProcessProposal` that doesn't match a previously sent
prevote message for the same height and round.

Until [#1035][1035] is addressed, this is considered a good enough solution in our implementation,
both for `PrepareProposal`, and for `ProcessProposal` implementations with non-determinism.

## Conclusion

This document has explored the possibility of relaxing the coherence and determinism properties
of the ABCI 1.0 (and 2.0) specification affecting `PrepareProposal` and `ProcessProposal`
for a class of applications that cannot guarantee them.

We first weakened the _validity_ property of the consensus specification
in a way that keeps the overall consensus specification strong enough to be relevant.
We then proposed a weaker coherence property for ABCI 1.0 (and 2.0) that can replace the original
coherence and determinism properties related to `PrepareProposal` and `ProcessProposal`.
The new property is useful for applications that cannot fulfill the original properties
but can fulfill the new one.
Finally, we explained how to modify the Tendermint consensus algorithm, implemented in CometBFT,
to guarantee the consensus _termination_ property for applications that fulfill the new property.

In this document, we have not formally proven that the changes we made to the algorithm,
combined with an application fulfilling weak validity, guarantee consensus properties
replacing _validity_ by _weak validity_. We plan to do this as an extension to this RFC.

Additionally, we have not tackled the problem of applications that cannot fulfill coherence and determinism properties
that refer to vote extensions in the ABCI 2.0 specification.
We leave this as future work.

## References

* [ABCI 1.0 specification][abci-spec]
* [Tendermint algorithm][arxiv]
* [Issue #1171][1171]
* [Issue #1174][1174]
* [Issue #1230][1230]
* [Issue #1035][1035]
* Going further in removing `valid(v)` check: [first comment][valid-further1], [second comment][valid-further2]

[abci-spec]: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_app_requirements.md#formal-requirements
[arxiv]: https://arxiv.org/abs/1807.04938
[1171]: https://github.com/cometbft/cometbft/issues/1171
[1174]: https://github.com/cometbft/cometbft/issues/1174
[1230]: https://github.com/cometbft/cometbft/issues/1230
[1035]: https://github.com/cometbft/cometbft/issues/1035
[valid-further1]: https://github.com/cometbft/cometbft/issues/1230#issuecomment-1671233308
[valid-further2]: https://github.com/cometbft/cometbft/pull/1391#pullrequestreview-1641521760