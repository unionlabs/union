# PBTS: System Model and Properties

## Outline

 - [System model](#system-model)
   - [Synchronized clocks](#synchronized-clocks)
   - [Message delays](#message-delays)
 - [Problem Statement](#problem-statement)
 - [Timely Predicate](#timely-predicate)
    - [Timely Proof-of-Locks](#timely-proof-of-locks)
    - [Derived Proof-of-Locks](#derived-proof-of-locks)
 - [Temporal Analysis](#temporal-analysis)
    - [Safety](#safety)
    - [Liveness](#liveness)

## System Model

#### **[PBTS-CLOCK-NEWTON.0]**

There is a reference Newtonian real-time `t`.

No process has direct access to this reference time, used only for specification purposes.
The reference real-time is assumed to be aligned with the Coordinated Universal Time (UTC).

### Synchronized clocks

Processes are assumed to be equipped with synchronized clocks,
aligned with the Coordinated Universal Time (UTC).

This requires processes to periodically synchronize their local clocks with an
external and trusted source of the time (e.g. NTP servers).
Each synchronization cycle aligns the process local clock with the external
source of time, making it a *fairly accurate* source of real time.
The periodic (re)synchronization aims to correct the *drift* of local clocks,
which tend to pace slightly faster or slower than the real time.

To avoid an excessive level detail in the parameters and guarantees of
synchronized clocks, we adopt a single system parameter `PRECISION` to
encapsulate the potential inaccuracy of the synchronization mechanisms,
and drifts of local clocks from real time.

#### **[PBTS-CLOCK-PRECISION.0]**

There exists a system parameter `PRECISION`, such that
for any two processes `p` and `q`, with local clocks `C_p` and `C_q`:

- If `p` and `q` are equipped with synchronized clocks,
 then for any real-time `t` we have `|C_p(t) - C_q(t)| <= PRECISION`.

`PRECISION` thus bounds the difference on the times simultaneously read by processes
from their local clocks, so that their clocks can be considered synchronized.

### Message Delays

To properly evaluate whether the time assigned to a proposal is consistent with the real time,
we need some information regarding the time it takes for a message carrying a proposal
to reach all its (correct) destinations.
More precisely, the *maximum delay* for delivering a proposal to its destinations allows
defining a lower bound, a *minimum time* that a correct process assigns to proposal.

#### **[PBTS-MSG-DELAY.0]**

There exists a system parameter `MSGDELAY` for end-to-end delays of proposal messages,
such for any two correct processes `p` and `q`:

- If `p` sends a proposal message `m` at real time `t` and `q` receives `m` at
  real time `t'`, then `t <= t' <= t + MSGDELAY`.

Notice that, as a system parameter, `MSGDELAY` should be observed for any
proposal message broadcast by correct processes: it is a *worst-case* parameter.
As message delays depends on the message size, the above requirement implicitly
indicates that the size of proposal messages is either fixed or upper bounded.

#### **[PBTS-MSG-DELAY-ADAPTIVE.0]**

This specification is written assuming that there exists an end-to-end maximum
delay `maxMsgDelay` observed in the network, possibly unknown, and
that the chosen value for `MSGDELAY` is such that `MSGDELAY >= maxMsgDelay`.
Under this assumption, all properties described in this specification are satisfied.

However, it is possible that in some networks the `MSGDELAY` parameters
selected by operators is too small, i.e., `MSGDELAY < maxMsgDelay`.
In order to tolerate this possibility, we propose the adoption of adaptive
end-to-end delays, namely a relaxation of [PBTS-MSG-DELAY.0] where the
`MSGDELAY` value increases each time consensus requires a new round.
In this way, after a number of rounds, the adopted `MSGDELAY` should match the
actual, but possibly unknown, end-to-end `maxMsgDelay`. 
This is a typical approach in partial synchronous models.

The adaptive system parameter `MSGDELAY(r)` is defined as follows.
Lets `p` and `q` be any correct processes:

- If `p` sends a proposal message `m` from round `r` at real time `t` and `q` receives `m` at
  real time `t'`, then `t < t' <= t + MSGDELAY(r)`.

The adaptiveness is represented by the assumption that the value of the
parameter increases over rounds, i.e., `MSGDELAY(r+1) > MSGDELAY(r)`.
The initial value `MSGDELAY(0)` is equal to `MSGDELAY` as in [PBTS-MSG-DELAY.0].

For the sake of correctness and formal verification, if `MSGDELAY` is
chosen sufficiently large, then the fact that it increments in later rounds
(i) in practice will never be experienced,
and (ii) also has no theoretical implications.
The adaptation (increment) of `MSGDELAY` is only introduced here to handle
potential misconfiguration.

## Problem Statement

This section defines the properties of Tendermint consensus algorithm
(cf. the [arXiv paper][arXiv]) in this system model.

#### **[PBTS-PROPOSE.0]**

A proposer proposes a consensus value `v` that includes a proposal time
`v.time`.

#### **[PBTS-INV-AGREEMENT.0]**

- [Agreement] No two correct processes decide different values.

This implies that no two correct processes decide, in particular, different
proposal times.

#### **[PBTS-INV-VALID.0]**

- [Validity] If a correct process decides on value `v`, then `v` satisfies a
  predefined `valid` predicate.

With respect to PBTS, the `valid` predicate requires proposal times to be
[monotonic][time-monotonicity] over heights of
consensus.

#### **[PBTS-INV-MONOTONICITY.0]**

- If a correct process decides on value `v` at the height `h` of consensus,
  thus setting `decision[h] = v`, then `v.time > decision[h'].time` for all
  previous heights `h' < h`.

The monotonicity of proposal times
implicitly assumes that heights of consensus are executed in order.

#### **[PBTS-INV-TIMELY.0]**

- [Time-Validity] If a correct process decides on value `v`, then the proposal
  time `v.time` was considered `timely` by at least one correct process.

The following section defines the `timely` predicate
that restricts the allowed decisions based
on the proposal time `v.time` associated with a proposed value `v`.

## Timely Predicate

For PBTS, a `proposal` is a tuple `(v, v.time, v.round)`, where:

- `v` is the proposed value;
- `v.time` is the associated proposal time;
- `v.round` is the round at which `v` was first proposed.

We include the proposal round `v.round` in the proposal definition because a
value `v` can be proposed in multiple rounds of consensus,
but the evaluation of the `timely` predicate is only relevant at round `v.round`.

> Considering the algorithm in the [arXiv paper][arXiv], a new proposal is
> produced by the `getValue()` method (line 18), invoked by the proposer `p` of round
> `round_p` when starting the round with `validValue_p = nil`.
> In this case, the proposed value is broadcast in a `PROPOSAL` message with
> `vr = validRound_p = -1`.

#### **[PBTS-PROPOSAL-RECEPTION.0]**

The `timely` predicate is evaluated when a process receives a proposal.
More precisely, let `p` be a correct process:

- `proposalReceptionTime(p,r)` is the time `p` reads from its local clock when
  it receives the proposal of round `r`.

#### **[PBTS-TIMELY.0]**

Lets `(v, v.time, v.round)` be a proposal, then `v.time` is considered `timely` by a correct process
`p` if:

1. `proposalReceptionTime(p,v.round)` is set, and
1. `proposalReceptionTime(p,v.round) >= v.time - PRECISION`, and
1. `proposalReceptionTime(p,v.round) <= v.time + MSGDELAY(v.round) + PRECISION`.

A correct process only sends a `PREVOTE` for `v` at round `v.round` if the
associated proposal time `v.time` is considered `timely`.

> Considering the algorithm in the [arXiv paper][arXiv], the `timely` predicate
> is evaluated by a process `p` when it receives a valid `PROPOSAL` message
> from the proposer of the current round `round_p` with `vr = -1` (line 22).

### Timely Proof-of-Locks

A *Proof-of-Lock* is a set of `PREVOTE` messages of round of consensus for the
same value from processes whose cumulative voting power is at least `2f + 1`.
We denote as `POL(v,r)` a proof-of-lock of value `v` at round `r`.

For PBTS, we are particularly interested in the `POL(v,v.round)` produced in
the round `v.round` at which a value `v` was first proposed.
We call it a *timely* proof-of-lock for `v` because it can only be observed
if at least one correct process considered it `timely`:

#### **[PBTS-TIMELY-POL.0]**

If

- there is a valid `POL(v,r)` with `r = v.round`, and
- `POL(v,v.round)` contains a `PREVOTE` message from at least one correct process,

Then, let `p` is a such correct process:

- `p` received a `PROPOSAL` message of round `v.round`, and
- the `PROPOSAL` message contained a proposal `(v, v.time, v.round)`, and
- `p` was in round `v.round` and evaluated the proposal time `v.time` as `timely`.

The existence of a such correct process `p` is guaranteed provided that the
voting power of Byzantine processes is bounded by `2f`.

### Derived Proof-of-Locks

The existence of `POL(v,r)` is a requirement for the decision of `v` at round
`r` of consensus.

At the same time, the Time-Validity property establishes that if `v` is decided
then a timely proof-of-lock `POL(v,v.round)` must have been produced.

So, we need to demonstrate here that any valid `POL(v,r)` is either a timely
proof-of-lock or it is derived from a timely proof-of-lock:

#### **[PBTS-DERIVED-POL.0]**

If

- there is a valid `POL(v,r)`, and
- `POL(v,r)` contains a `PREVOTE` message from at least one correct process,

Then

- there is a valid `POL(v,v.round)` with `v.round <= r` which is a timely proof-of-lock.

The above relation is trivially observed when `r = v.round`, as `POL(v,r)` must
be a timely proof-of-lock.
Notice that we cannot have `r < v.round`, as `v.round` is defined as the first
round at which `v` was proposed.

For `r > v.round` we need to demonstrate that if there is a valid `POL(v,r)`,
then a timely `POL(v,v.round)` was previously obtained.
We observe that a condition for observing a `POL(v,r)` is that the proposer of
round `r` has broadcast a `PROPOSAL` message for `v`.
As `r > v.round`, we can affirm that `v` was not produced in round `r`.
Instead, by the protocol operation, `v` was a *valid value* for the proposer of
round `r`, which means that if the proposer has observed a `POL(v,vr)` with `vr
< r`.
The above operation considers a *correct* proposer, but since a `POL(v,r)` was
produced (by hypothesis) we can affirm that at least one correct process (also)
observed a `POL(v,vr)`.

> Considering the algorithm in the [arXiv paper][arXiv], `v` was proposed by
> the proposer `p` of round `round_p` because its `validValue_p` variable was
> set to `v`.
> The `PROPOSAL` message broadcast by the proposer, in this case, had `vr = validRound_p > -1`,
> and it could only be accepted by processes that also observed a `POL(v,vr)`.

Thus, if there is a `POL(v,r)` with `r > v.round`, then there is a valid
`POL(v,vr)` with `v.round <= vr < r`.
If `vr = v.round` then `POL(vr,v)` is a timely proof-of-lock and we are done.
Otherwise, there is another valid `POL(v,vr')` with `v.round <= vr' < vr`,
and the above reasoning can be recursively applied until we get `vr' = v.round`
and observe a timely proof-of-lock.

## Temporal analysis

In this section we present invariants that need be observed for ensuring that
PBTS is both safe and live.

In addition to the variables and system parameters already defined, we use
`beginRound(p,r)` as the value of process `p`'s local clock
when it starts round `r` of consensus.

### Safety

The safety of PBTS requires that if a value `v` is decided, then at least one
correct process `p` considered the associated proposal time `v.time` timely.
Following the definition of [timely proposal times](#pbts-timely0) and
proof-of-locks, we require this condition to be asserted at a specific round of
consensus, defined as `v.round`:

#### **[PBTS-SAFETY.0]**

If

- there is a valid commit `C` for a value `v`
- `C` contains a `PRECOMMIT` message from at least one correct process

then there is a correct process `p` (not necessarily the same above considered) such that:

- `beginRound(p,v.round) <= proposalReceptionTime(p,v.round) <= beginRound(p,v.round+1)` and
- `v.time <= proposalReceptionTime(p,v.round) + PRECISION` and
- `v.time >= proposalReceptionTime(p,v.round) - MSGDELAY(v.round) - PRECISION` 

That is, a correct process `p` started round `v.round` and, while still at
round `v.round`, received a `PROPOSAL` message from round `v.round` proposing
`v`.
Moreover, the reception time of the original proposal for `v`, according with
`p`'s local clock, enabled `p` to consider the proposal time `v.time` as
`timely`.
This is the requirement established by PBTS for issuing a `PREVOTE` for the
proposal `(v, v.time, v.round)`, so for the eventual decision of `v`.

### Liveness

The liveness of PBTS relies on correct processes accepting proposal times
assigned by correct proposers.
We thus present a set of conditions for assigning a proposal time `v.time` so
that every correct process should be able to issue a `PREVOTE` for `v`.

#### **[PBTS-LIVENESS.0]**

If

- the proposer of a round `r` of consensus is correct
- and it proposes a value `v` for the first time, with associated proposal time `v.time`

then the proposal `(v, v.time, r)` is accepted by every correct process provided that:

- `min{p is correct : beginRound(p,r)} <= v.time <= max{p is correct : beginRound(p,r)}` and
- `max{p is correct : beginRound(p,r)} <= v.time + MSGDELAY(r) + PRECISION <= min{p is correct : beginRound(p,r+1)}`

The first condition establishes a range of safe proposal times `v.time` for round `r`.
This condition is trivially observed if a correct proposer `p` sets `v.time` to the time it
reads from its clock when starting round `r` and proposing `v`.
A `PROPOSAL` message sent by `p` at local time `v.time` should not be received
by any correct process before its local clock reads `v.time - PRECISION`, so
that condition 2 of [PBTS-TIMELY.0] is observed.

The second condition establishes that every correct process should start round
`v.round` at a local time that allows `v.time` to still be considered timely,
according to condition 3. of [PBTS-TIMELY.0].
In addition, it requires correct processes to stay long enough in round
`v.round` so that they can receive the `PROPOSAL` message of round `v.round`.
It assumed here that the proposer of `v` broadcasts a `PROPOSAL` message at
time `v.time`, according to its local clock, so that every correct process
should receive this message by time `v.time + MSGDELAY(v.round) + PRECISION`, according
to their local clocks.

Back to [main document][main].

[main]: ./README.md

[algorithm]: ./pbts-algorithm.md
[time-monotonicity]: ./pbts-algorithm.md#time-monotonicity

[sysmodel]: ./pbts-sysmodel.md
[sysmodel_v1]: ./v1/pbts-sysmodel_001_draft.md

[arXiv]: https://arxiv.org/pdf/1807.04938.pdf
