# PBTS: Algorithm Specification

## Proposal Time

PBTS computes, for a proposed value `v`, the proposal time `v.time` with bounded difference to the actual real-time the proposed value was generated.
The proposal time is read from the clock of the process that proposes a value for the first time, which is its original proposer.

With PBTS, therefore, we assume that processes have access to **synchronized clocks**.
The proper definition of what it means can be found in the [system model][sysmodel],
but essentially we assume that two correct processes do not simultaneously read from their clocks
time values that differ by more than `PRECISION`, which is a system parameter.

### Proposal times are definitive

When a value `v` is produced by a process, it also assigns the associated proposal time `v.time`.
If the same value `v` is then re-proposed in a subsequent round of consensus,
it retains its original time, assigned by its original proposer.

A value `v` should re-proposed when it becomes locked by the network, i.e., when it receives `2f + 1 PREVOTES` in a round `r` of consensus.
This means that processes with `2f + 1`-equivalent voting power accepted both `v` and its associated time `v.time` in round `r`.
Since the originally proposed value and its associated time were considered valid, there is no reason for reassigning `v.time`.

> In the [first version][algorithm_v1] of this specification, proposals were defined as pairs `(v, time)`.
> In addition, the same value `v` could be proposed, in different rounds, but would be associated with distinct times each time it was re-proposed.
> Since this possibility does not exist in this second specification, the proposal time became part of the proposed value.
> With this simplification, several small changes to the [arXiv][arXiv] algorithm are no longer required.

## Time Monotonicity

Values decided in successive heights of consensus must have increasing times, so:

- Monotonicity: for any process `p` and any two decided heights `h` and `h'`, if `h > h'` then `decision_p[h].time > decision_p[h'].time`.

To ensure time monotonicity, it is enough to ensure that a value `v` proposed by process `p` at height `h_p` has `v.time > decision_p[h_p-1].time`.
So, if process `p` is the proposer of a round of height `h_p` and reads from its clock a time `now_p <= decision_p[h_p-1]`,
it should postpone the generation of its proposal until `now_p > decision_p[h_p-1]`.

> Although it should be considered, this scenario is unlikely during regular operation,
as from `decision_p[h_p-1].time` and the start of height `h_p`, a complete consensus instance need to be concluded.

Notice that monotonicity is not introduced by this proposal, as it is already ensured by  [`BFT Time`][bfttime].
In `BFT Time`, the `Timestamp` field of every `Precommit` message of height `h_p` sent by a correct process is required to be larger than `decision_p[h_p-1].time`.
As one of such `Timestamp` fields becomes the time assigned to a value proposed at height `h_p`, time monotonicity is observed.

The time monotonicity of values proposed in heights of consensus is verified by the `valid()` predicate, to which every proposed value is submitted.
A value rejected by `valid()` is not accepted by any correct process.

## Timely Predicate

PBTS introduces a new requirement for a process to accept a proposal: the proposal time must be `timely`.
It is a temporal requirement, associated with the following
[synchrony assumptions][sysmodel] regarding the behavior of processes and the network:

- Synchronized clocks: the values simultaneously read from clocks of any two correct processes differ by at most `PRECISION`;
- Bounded transmission delays: the real time interval between the sending of a proposal at a correct process and the reception of the proposal at any correct process is upper bounded by `MSGDELAY`.
  - With the introduction of [adaptive message delays](./pbts-sysmodel.md#pbts-msg-delay-adaptive0),
    the `MSGDELAY` parameter should be interpreted as `MSGDELAY(r)`, where `r` is the current round,
    where it is expected `MSGDELAY(r+1) > MSGDELAY(r)`.

#### **[PBTS-RECEPTION-STEP.1]**

Let `now_p` be the time, read from the clock of process `p`, at which `p` receives the proposed value `v` of round `r`.
The proposal time is considered `timely` by `p` when:

1. `now_p >= v.time - PRECISION`
1. `now_p <= v.time + MSGDELAY + PRECISION`

The first condition derives from the fact that the generation and sending of `v` precedes its reception.
The minimum receiving time `now_p` for `v.time` be considered `timely` by `p` is derived from the extreme scenario when
the clock of `p` is `PRECISION` *behind* of the clock of the proposer of `v`, and the proposal's transmission delay is `0` (minimum).

The second condition derives from the assumption of an upper bound for the transmission delay of a proposal.
The maximum receiving time `now_p` for `v.time` be considered `timely` by `p` is derived from the extreme scenario when
the clock of `p` is `PRECISION` *ahead* of the clock of the proposer of `v`, and the proposal's transmission delay is `MSGDELAY` (maximum).

## Updated Consensus Algorithm

The following changes are proposed for the algorithm in the [arXiv paper][arXiv].

### Updated `StartRound` function

There are two additions to operation of the **proposer** of a round:

1. To ensure time monotonicity, the proposer does not propose a value until its
current local time, represented by `now_p` in the algorithm, 
becomes greater than the previously decided value's time
2. When the proposer produce a new proposal it sets the proposal's time to its current local time
   - No changes are made to the logic when a proposer has a non-nil `validValue_p`, which retains its original proposal time.

#### **[PBTS-ALG-STARTROUND.1]**

```go
function StartRound(round) {
   round_p ← round
   step_p ← propose
   if proposer(h_p, round_p) = p {
      if validValue_p != nil {
         proposal ← validValue_p // proposal.time unchanged
      } else {
         wait until now_p > decision_p[h_p-1].time // time monotonicity
         proposal ← getValue()
         proposal.time ← now_p // proposal time set to current local time
      }
      broadcast ⟨PROPOSAL, h_p, round_p, proposal, validRound_p⟩
   } else {
      schedule OnTimeoutPropose(h_p,round_p) to be executed after timeoutPropose(round_p)
   }
}
```

### Updated upon clause of line 22

The rule on line 22 applies to values `v` proposed for the first time, i.e.,
for proposals not backed by `2f + 1 PREVOTE`s for `v` in a previous round.
The `PROPOSAL` message, in this case, has a valid round `vr = -1`.

The new rule for issuing a `PREVOTE` for a proposed value `v`
requires the proposal time to be `timely`:

#### **[PBTS-ALG-UPON-PROP.1]**

```go
upon ⟨PROPOSAL, h_p, round_p, v, −1⟩ from proposer(h_p, round_p) while step_p = propose do {
   if timely(v.time) ∧ valid(v) ∧ (lockedRound_p = −1 ∨ lockedValue_p = v) {
      broadcast ⟨PREVOTE, h_p, round_p, id(v)⟩ 
   }
   else {
      broadcast ⟨PREVOTE, h_p, round_p, nil⟩ 
   }
   step_p ← prevote
}
```

The `timely` predicate considers the time at which the `PROPOSAL` message is received.
Although not represented above, for the sake of simplicity, it is assumed that the
`PROPOSAL` receive time is registered and provided to the `timely` predicate.

### Unchanged upon clause of line 28

The rule on line 28 applies to values `v` re-proposed in the current round
because its proposer received `2f + 1 PREVOTE`s for `v` in a previous round
`vr`, therefore updating its `validValue_p` to `v` and `validRound_p` to `vr`. 

This means that there was a round `r <= vr` in which `2f + 1` processes
accepted a `PROPOSAL` for `v`, proposed at round `r` for the first time, with
`vr = -1`.
These processes executed the line 22 of the algorithm and broadcast a
`PREVOTE` for `v`, indicating in particular, that they have judged `v.time`
as `timely` in round `r`.

Since `v.time` was considered `timely` by `2f + 1` processes in a previous
round, and provided that `v.time` cannot be updated when `v` is re-proposed,
the evaluation of `timely` predicate is not necessary in this case.

For a more formal explanation and a proof of this assertion, refer to the
[properties][sysmodel-pol] section.

**All other rules remain unchanged.**

Back to [main document][main].

[main]: ./README.md

[algorithm_v1]: ./v1/pbts-algorithm_001_draft.md

[sysmodel]: ./pbts-sysmodel.md
[sysmodel-pol]: ./pbts-sysmodel.md#derived-proof-of-locks

[bfttime]: ../bft-time.md
[arXiv]: https://arxiv.org/pdf/1807.04938.pdf
