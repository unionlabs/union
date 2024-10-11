# PBTS: System Model and Properties (first draft)

This specification is **OUTDATED**. Please refer to the [new version][sysmodel].

## System Model

### Time and Clocks

#### **[PBTS-CLOCK-NEWTON.0]**

There is a reference Newtonian real-time `t` (UTC).

Every correct validator `V` maintains a synchronized clock `C_V` that ensures:

#### **[PBTS-CLOCK-PRECISION.0]**

There exists a system parameter `PRECISION` such that for any two correct validators `V` and `W`, and at any real-time `t`,  
`|C_V(t) - C_W(t)| < PRECISION`


### Message Delays

We do not want to interfere with the timing assumptions of Tendermint consensus algorithm.
We will postulate a timing restriction, which, if satisfied, ensures that liveness is preserved.

In general the local clock may drift from the global time. (It may progress faster, e.g., one second of clock time might take 1.005 seconds of real-time). As a result the local clock and the global clock may be measured in different time units. Usually, the message delay is measured in global clock time units. To estimate the correct local timeout precisely, we would need to estimate the clock time duration of a message delay taking into account the clock drift. For simplicity we ignore this, and directly postulate the message delay assumption in terms of local time.


#### **[PBTS-MSG-D.0]**

There exists a system parameter `MSGDELAY` for message end-to-end delays **counted in clock-time**.

> Observe that [PBTS-MSG-D.0] imposes constraints on message delays as well as on the clock.

#### **[PBTS-MSG-FAIR.0]**

The message end-to-end delay between a correct proposer and a correct validator (for `PROPOSE` messages) is less than `MSGDELAY`.


## Problem Statement

In this section we define the properties of Tendermint consensus algorithm (cf. the [arXiv paper][arXiv]) in this new system model.

#### **[PBTS-PROPOSE.0]**

A proposer proposes a pair `(v,t)` of consensus value `v` and time `t`.

> We then restrict the allowed decisions along the following lines:

#### **[PBTS-INV-AGREEMENT.0]**

[Agreement] No two correct validators decide on different values `v`.

#### **[PBTS-INV-TIME-VAL.0]**

[Time-Validity] If a correct validator decides on `t` then `t` is "OK" (we will formalize this below), even if up to `2f` validators are faulty.

However, the properties of Tendermint consensus algorithm are of more interest with respect to the blocks, that is, what is written into a block and when. We therefore, in the following, will give the safety and liveness properties from this block-centric viewpoint.  
For this, observe that the time `t` decided at consensus height `k` will be written in the block of height `k+1`, and will be supported by `2f + 1` `PRECOMMIT` messages of the same consensus round `r`. The time written in the block, we will denote by `b.time` (to distinguish it from the term `bfttime` used for median-based time). For this, it is important to have the following consensus algorithm property:

#### **[PBTS-INV-TIME-AGR.0]**

[Time-Agreement] If two correct validators decide in the same round, then they decide on the same `t`.

#### **[PBTS-DECISION-ROUND.0]**

Note that the relation between consensus decisions, on the one hand, and blocks, on the other hand, is not immediate; in particular if we consider time: In the proposed solution,
as validators may decide in different rounds, they may decide on different times.
The proposer of the next block, may pick a commit (at least `2f + 1` `PRECOMMIT` messages from one round), and thus it picks a decision round that is going to become "canonic".
As a result, the proposer implicitly has a choice of one of the times that belong to rounds in which validators decided. Observe that this choice was implicitly the case already in the median-based `bfttime`.
However, as most consensus instances terminate within one round on the Cosmos hub, this is hardly ever observed in practice.



Finally, observe that the agreement ([Agreement] and [Time-Agreement]) properties are based on the Cosmos security model [CMBC-FM-2THIRDS.0][CMBC-FM-2THIRDS-link] of more than 2/3 correct validators, while [Time-Validity] is based on more than 1/3 correct validators.

### SAFETY

Here we will provide specifications that relate local time to block time. However, since we do not assume (by now) that local time is linked to real-time, these specifications also do not provide a relation between block time and real-time. Such properties are given [later](#real-time-safety).

For a correct validator `V`, let `beginConsensus(V,k)` be the local time when it sets its height to `k`, and let `endConsensus(V,k)` be the time when it sets its height to `k + 1`.

Let

- `beginConsensus(k)` be the minimum over `beginConsensus(V,k)`, and
- `last-beginConsensus(k)` be the maximum over `beginConsensus(V,k)`, and
- `endConsensus(k)` the maximum over `endConsensus(V,k)`

for all correct validators `V`.

> Observe that `beginConsensus(k) <= last-beginConsensus(k)` and if local clocks are monotonic, then `last-beginConsensus(k) <= endConsensus(k)`.

#### **[PBTS-CLOCK-GROW.0]**

We assume that during one consensus instance, local clocks are not set back, in particular for each correct validator `V` and each height `k`, we have `beginConsensus(V,k) < endConsensus(V,k)`.


#### **[PBTS-CONSENSUS-TIME-VALID.0]**

If

- there is a valid commit `c` for height `k`, and
- `c` contains a `PRECOMMIT` message by at least one correct validator,

then the time `b.time` in the block `b` that is signed by `c` satisfies

- `beginConsensus(k) - PRECISION <= b.time < endConsensus(k) + PRECISION + MSGDELAY`.


> [PBTS-CONSENSUS-TIME-VALID.0] is based on an analysis where the proposer is faulty (and does does not count towards `beginConsensus(k)` and `endConsensus(k)`), and we estimate the times at which correct validators receive and `accept` the `propose` message. If the proposer is correct we obtain

#### **[PBTS-CONSENSUS-LIVE-VALID-CORR-PROP.0]**

If the proposer of round 1 is correct, and

- [CMBC-FM-2THIRDS.0] holds for a block of height `k - 1`, and
- [PBTS-MSG-FAIR.0], and
- [PBTS-CLOCK-PRECISION.0], and
- [PBTS-CLOCK-GROW.0] (**TODO:** is that enough?)

then eventually (within bounded time) every correct validator decides in round 1.

#### **[PBTS-CONSENSUS-SAFE-VALID-CORR-PROP.0]**

If the proposer of round 1 is correct, and

- [CMBC-FM-2THIRDS.0] holds for a block of height `k - 1`, and
- [PBTS-MSG-FAIR.0], and
- [PBTS-CLOCK-PRECISION.0], and
- [PBTS-CLOCK-GROW.0] (**TODO:** is that enough?)

then `beginConsensus_k <= b.time <= last-beginConsensus_k`.


> For the above two properties we will assume that a correct proposer `v` sends its `PROPOSAL` at its local time `beginConsensus(v,k)`.

### LIVENESS

If

- [CMBC-FM-2THIRDS.0] holds for a block of height `k - 1`, and
- [PBTS-MSG-FAIR.0],
- [PBTS-CLOCK.0], and
- [PBTS-CLOCK-GROW.0] (**TODO:** is that enough?)

then eventually there is a valid commit `c` for height `k`.


### REAL-TIME SAFETY

> We want to give a property that can be exploited from the outside, that is, given a block with some time stored in it, what is the estimate at which real-time the block was generated. To do so, we need to link clock-time to real-time; which is not the case with [PBTS-CLOCK.0]. For this, we introduce the following assumption on the clocks:

#### **[PBTS-CLOCKSYNC-EXTERNAL.0]**

There is a system parameter `ACCURACY`, such that for all real-times `t` and all correct validators `V`,

- `| C_V(t) - t | < ACCURACY`.

> `ACCURACY` is not necessarily visible at the code level. The properties below just show that the smaller
its value, the closer the block time will be to real-time

#### **[PBTS-CONSENSUS-PTIME.0]**

LET `m` be a propose message. We consider the following two real-times `proposalTime(m)` and `propRecvTime(m)`:

- if the proposer is correct and sends `m` at time `t`, we write `proposalTime(m)` for real-time `t`.
- if first correct validator receives `m` at time `t`, we write `propRecvTime(m)` for real-time `t`.


#### **[PBTS-CONSENSUS-REALTIME-VALID.0]**

Let `b` be a block with a valid commit that contains at least one `precommit` message by a correct validator (and `proposalTime` is the time for the height/round `propose` message `m` that triggered the `precommit`). Then:

`propRecvTime(m) - ACCURACY - PRECISION < b.time < propRecvTime(m) + ACCURACY + PRECISION + MSGDELAY`


#### **[PBTS-CONSENSUS-REALTIME-VALID-CORR.0]**

Let `b` be a block with a valid commit that contains at least one `precommit` message by a correct validator (and `proposalTime` is the time for the height/round `propose` message `m` that triggered the `precommit`). Then, if the proposer is correct:

`proposalTime(m) - ACCURACY < b.time < proposalTime(m) + ACCURACY`

> by the algorithm at time `proposalTime(m)` the proposer fixes `m.time <- now_p(proposalTime(m))`

> "triggered the `PRECOMMIT`" implies that the data in `m` and `b` are "matching", that is, `m` proposed the values that are actually stored in `b`.

Back to [main document][main_v1].

[main_v1]: ./pbts_001_draft.md

[algorithm_v1]: ./pbts-algorithm_001_draft.md

[sysmodel]: ../pbts-sysmodel.md

[arXiv]: https://arxiv.org/abs/1807.04938

[CMBC-FM-2THIRDS-link]: ../../../light-client/verification/verification_002_draft.md#cmbc-fm-2thirds1
