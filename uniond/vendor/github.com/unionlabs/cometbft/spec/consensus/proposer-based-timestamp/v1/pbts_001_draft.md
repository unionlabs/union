# Proposer-Based Time (first draft)

## Current BFTTime

### Description

In CometBFT, the first version of how time is computed and stored in a block works as follows:

- validators send their current local time as part of `precommit` messages
- upon collecting the `precommit` messages that the proposer uses to build a commit to be put in the next block, the proposer computes the `time` of the next block as the median (weighted over voting power) of the times in the `precommit` messages.

### Analysis

1. **Fault tolerance.** The computed median time is called [`bfttime`][bfttime] as it is indeed fault-tolerant: if **less than a third** of the validators is faulty (counted in voting power), it is guaranteed that the computed time lies between the minimum and the maximum times sent by correct validators.
1. **Effect of faulty validators.** If more than `1/2` of the voting power (which is in fact more than one third and less than two thirds of the voting power) is held by faulty validators, then the time is under total control of the faulty validators. (This is particularly challenging in the context of [lightclient][lcspec] security.)
1. **Proposer influence on block time.** The proposer of the next block has a degree of freedom in choosing the `bfttime`, since it computes the median time based on the timestamps from `precommit` messages sent by
   `2f + 1` correct validators.
   1. If there are `n` different timestamps in the  `precommit` messages, the proposer can use any subset of timestamps that add up to `2f + 1`
	  of the voting power in order to compute the median.
   1. If the validators decide in different rounds, the proposer can decide on which round the median computation is based.
1. **Liveness.** The liveness of the protocol:
   1. does not depend on clock synchronization,
   1. depends on bounded message delays.
1. **Relation to real time.** There is no clock synchronization, which implies that there is **no relation** between the computed block `time` and real time.
1. **Aggregate signatures.** As the `precommit` messages contain the local times, all these `precommit` messages typically differ in the time field, which **prevents** the use of aggregate signatures.

## Suggested Proposer-Based Time

### Outline

An alternative approach to time has been discussed: Rather than having the validators send the time in the `precommit` messages, the proposer in the consensus algorithm sends its time in the `propose` message, and the validators locally check whether the time is OK (by comparing to their local clock).

This proposed solution adds the requirement of having synchronized clocks, and other implicit assumptions.

### Comparison of the Suggested Method to the Old One

1. **Fault tolerance.** Maintained in the suggested protocol.
1. **Effect of faulty validators.** Eliminated in the suggested protocol,
   that is, the block `time` can be corrupted only in the extreme case when
   `>2/3` of the validators are faulty.
1. **Proposer influence on block time.** The proposer of the next block
   has less freedom when choosing the block time.
   1. This scenario is eliminated in the suggested protocol, provided that there are `<1/3` faulty validators.
   1. This scenario is still there.
1. **Liveness.** The liveness of the suggested protocol:
   1. depends on the introduced assumptions on synchronized clocks (see below),
   1. still depends on the message delays (unavoidable).
1. **Relation to real time.** We formalize clock synchronization, and obtain a **well-defined relation** between the block `time` and real time.
1. **Aggregate signatures.** The `precommit` messages free of time, which **allows** for aggregate signatures.

### Protocol Overview

#### Proposed Time

We assume that the field `proposal` in the `PROPOSE` message is a pair `(v, time)`, of the proposed consensus value `v` and the proposed time `time`.

#### Reception Step

In the reception step at node `p` at local time `now_p`, upon receiving a message `m`:

- **if** the message `m` is of type `PROPOSE` and satisfies `now_p - PRECISION <  m.time < now_p + PRECISION + MSGDELAY`, then mark the message as `timely`.  
(`PRECISION` and `MSGDELAY` being system parameters, see [below](#safety-and-liveness))

> after the presentation in the dev session, we realized that different semantics for the reception step is closer aligned to the implementation. Instead of dropping propose messages, we keep all of them, and mark timely ones.

#### Processing Step

- Start round

<table>
<tr>
<th>arXiv paper</th>
<th>Proposer-based time</th>
</tr>

<tr>
<td>

```go
function StartRound(round) {
 round_p ← round
 step_p ← propose
 if proposer(h_p, round_p) = p {

 
  if validValue_p != nil {

   proposal ← validValue_p
  } else {

   proposal ← getValue()
  }
   broadcast ⟨PROPOSAL, h_p, round_p, proposal, validRound_p⟩
 } else {
  schedule OnTimeoutPropose(h_p,round_p) to 
   be executed after timeoutPropose(round_p)
 }
}
```

</td>

<td>

```go
function StartRound(round) {
 round_p ← round
 step_p ← propose
 if proposer(h_p, round_p) = p {
  // new wait condition
  wait until now_p > block time of block h_p - 1
  if validValue_p != nil {
   // add "now_p"
   proposal ← (validValue_p, now_p) 
  } else {
   // add "now_p"
   proposal ← (getValue(), now_p) 
  }
  broadcast ⟨PROPOSAL, h_p, round_p, proposal, validRound_p⟩
 } else {
  schedule OnTimeoutPropose(h_p,round_p) to 
   be executed after timeoutPropose(round_p)
 }
}
```

</td>
</tr>
</table>

- Rule on lines 28-35

<table>
<tr>
<th>arXiv paper</th>
<th>Proposer-based time</th>
</tr>

<tr>
<td>

```go
upon timely(⟨PROPOSAL, h_p, round_p, v, vr⟩) 
 from proposer(h_p, round_p)
 AND 2f + 1 ⟨PREVOTE, h_p, vr, id(v)⟩ 
while step_p = propose ∧ (vr ≥ 0 ∧ vr < round_p) do {
 if valid(v) ∧ (lockedRound_p ≤ vr ∨ lockedValue_p = v) {
  
  broadcast ⟨PREVOTE, h_p, round_p, id(v)⟩
 } else {
  broadcast ⟨PREVOTE, hp, round_p, nil⟩
 }
}
```

</td>

<td>

```go
upon timely(⟨PROPOSAL, h_p, round_p, (v, tprop), vr⟩) 
 from proposer(h_p, round_p) 
 AND 2f + 1 ⟨PREVOTE, h_p, vr, id(v, tvote)⟩ 
 while step_p = propose ∧ (vr ≥ 0 ∧ vr < round_p) do {
  if valid(v) ∧ (lockedRound_p ≤ vr ∨ lockedValue_p = v) {
   // send hash of v and tprop in PREVOTE message
   broadcast ⟨PREVOTE, h_p, round_p, id(v, tprop)⟩
  } else {
   broadcast ⟨PREVOTE, hp, round_p, nil⟩
  }
 }
```

</td>
</tr>
</table>

- Rule on lines 49-54

<table>
<tr>
<th>arXiv paper</th>
<th>Proposer-based time</th>
</tr>

<tr>
<td>

```go
upon ⟨PROPOSAL, h_p, r, v, ∗⟩ from proposer(h_p, r) 
 AND 2f + 1 ⟨PRECOMMIT, h_p, r, id(v)⟩ 
 while decisionp[h_p] = nil do {
  if valid(v) {

   decision_p [h_p] = v
   h_p ← h_p + 1
   reset lockedRound_p , lockedValue_p, validRound_p and 
    validValue_p to initial values and empty message log 
   StartRound(0)
  }
 }
```

</td>

<td>

```go
upon ⟨PROPOSAL, h_p, r, (v,t), ∗⟩ from proposer(h_p, r) 
 AND 2f + 1 ⟨PRECOMMIT, h_p, r, id(v,t)⟩
 while decisionp[h_p] = nil do {
  if valid(v) {
   // decide on time too
   decision_p [h_p] = (v,t) 
   h_p ← h_p + 1
   reset lockedRound_p , lockedValue_p, validRound_p and 
    validValue_p to initial values and empty message log 
   StartRound(0)
  }
 }
```

</td>
</tr>
</table>

- Other rules are extended in a similar way, or remain unchanged

### Property Overview

#### Safety and Liveness

For safety (Point 1, Point 2, Point 3i) and liveness (Point 4) we need
the following assumptions:

- There exists a system parameter `PRECISION` such that for any two correct validators `V` and `W`, and at any real-time `t`, their local times `C_V(t)` and `C_W(t)` differ by less than `PRECISION` time units,
i.e., `|C_V(t) - C_W(t)| < PRECISION`
- The message end-to-end delay between a correct proposer and a correct validator (for `PROPOSE` messages) is less than `MSGDELAY`.

#### Relation to Real-Time

For analyzing real-time safety (Point 5), we use a system parameter `ACCURACY`, such that for all real-times `t` and all correct validators `V`, we have `| C_V(t) - t | < ACCURACY`.

> `ACCURACY` is not necessarily visible at the code level.  We might even view `ACCURACY` as variable over time. The smaller it is during a consensus instance, the closer the block time will be to real-time.
>
> Note that `PRECISION` and `MSGDELAY` show up in the code.

### Detailed Specification

This specification describes the changes needed to be done to the Tendermint consensus algorithm as described in the [arXiv paper][arXiv] and the simplified specification in [TLA+][tlatender], and makes precise the underlying assumptions and the required properties.

- [Part I - System Model and Properties][sysmodel_v1]
- [Part II - Protocol specification][algorithm_v1]
- [TLA+ Specification][proposertla]

[algorithm_v1]: ./pbts-algorithm_001_draft.md

[tlatender]: ../../../light-client/accountability/README.md

[bfttime]: ../bft-time.md

[lcspec]: ../../light-client/README.md

[sysmodel_v1]: ./pbts-sysmodel_001_draft.md

[proposertla]: ../tla/TendermintPBT_001_draft.tla

[arXiv]: https://arxiv.org/abs/1807.04938
