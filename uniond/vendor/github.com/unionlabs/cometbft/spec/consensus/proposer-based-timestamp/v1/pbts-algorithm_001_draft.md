# PBTS: Protocol Specification (first draft)

This specification is **OUTDATED**. Please refer to the [new version][algorithm].

## Updated Consensus Algorithm

### Outline

The algorithm in the [arXiv paper][arXiv] evaluates rules of the received messages without making explicit how these messages are received. In our solution, we will make some message filtering explicit. We will assume that there are message reception steps (where messages are received and possibly stored locally for later evaluation of rules) and processing steps (the latter roughly as described in a way similar to the pseudo code of the arXiv paper).

In contrast to the original algorithm the field `proposal` in the `PROPOSE` message is a pair `(v, time)`, of the proposed consensus value `v` and the proposed time `time`.

#### **[PBTS-RECEPTION-STEP.0]**

In the reception step at process `p` at local time `now_p`, upon receiving a message `m`:

- if the message `m` is of type `PROPOSE` and satisfies `now_p - PRECISION <  m.time < now_p + PRECISION + MSGDELAY`, then mark the message as `timely`

> if `m` does not satisfy the constraint consider it `untimely`


#### **[PBTS-PROCESSING-STEP.0]**

In the processing step, based on the messages stored, the rules of the algorithms are
executed. Note that the processing step only operates on messages
for the current height. The consensus algorithm rules are defined by the following updates to arXiv paper.

#### New `StartRound`

There are two additions

- in case the proposer's local time is smaller than the time of the previous block, the proposer waits until this is not the case anymore (to ensure the block time is monotonically increasing)
- the proposer sends its time `now_p` as part of its proposal

We update the timeout for the `PROPOSE` step according to the following reasoning:

- If a correct proposer needs to wait to make sure its proposed time is larger than the `blockTime` of the previous block, then it sends by realtime `blockTime + ACCURACY` (By this time, its local clock must exceed `blockTime`)
- the receiver will receive a `PROPOSE` message by `blockTime + ACCURACY + MSGDELAY`
- the receiver's local clock will be `<= blockTime + 2 * ACCURACY + MSGDELAY`
- thus when the receiver `p` enters this round it can set its timeout to a value `waitingTime => blockTime + 2 * ACCURACY + MSGDELAY - now_p`

So we should set the timeout to `max(timeoutPropose(round_p), waitingTime)`.

> If, in the future, a block delay parameter `BLOCKDELAY` is introduced, this means
that the proposer should wait for `now_p > blockTime + BLOCKDELAY` before sending a `PROPOSE` message.
Also, `BLOCKDELAY` needs to be added to `waitingTime`.

#### **[PBTS-ALG-STARTROUND.0]**

```go
function StartRound(round) {
  blockTime ← block time of block h_p - 1
  waitingTime ← blockTime + 2 * ACCURACY + MSGDELAY - now_p
  round_p ← round
  step_p ← propose
  if proposer(h_p, round_p) = p {
    wait until now_p > blockTime // new wait condition
    if validValue_p != nil {
      proposal ← (validValue_p, now_p) // added "now_p"
    }
    else {
      proposal ← (getValue(), now_p)   // added "now_p"
    }
    broadcast ⟨PROPOSAL, h_p, round_p, proposal, validRound_p⟩
  }
  else {
    schedule OnTimeoutPropose(h_p,round_p) to be executed after max(timeoutPropose(round_p), waitingTime)
  }
}
```

#### New Rule Replacing Lines 22 - 27

- a validator prevotes for the consensus value `v` **and** the time `t`
- the code changes as the `PROPOSAL` message carries time (while `lockedValue` does not)

#### **[PBTS-ALG-UPON-PROP.0]**

```go
upon timely(⟨PROPOSAL, h_p, round_p, (v,t), −1⟩) from proposer(h_p, round_p) while step_p = propose do {
  if valid(v) ∧ (lockedRound_p = −1 ∨ lockedValue_p = v) {
    broadcast ⟨PREVOTE, h_p, round_p, id(v,t)⟩ 
  }
  else {
    broadcast ⟨PREVOTE, h_p, round_p, nil⟩ 
  }
  step_p ← prevote
}
```

#### New Rule Replacing Lines 28 - 33

In case consensus is not reached in round 1, in `StartRound` the proposer of future rounds may propose the same value but with a different time.
Thus, the time `tprop` in the `PROPOSAL` message need not match the time `tvote` in the (old) `PREVOTE` messages.
A validator may send `PREVOTE` for the current round as long as the value `v` matches.
This gives the following rule:

#### **[PBTS-ALG-OLD-PREVOTE.0]**

```go
upon timely(⟨PROPOSAL, h_p, round_p, (v, tprop), vr⟩) from proposer(h_p, round_p) AND 2f + 1 ⟨PREVOTE, h_p, vr, id((v, tvote)⟩ 
while step_p = propose ∧ (vr ≥ 0 ∧ vr < round_p) do {
  if valid(v) ∧ (lockedRound_p ≤ vr ∨ lockedValue_p = v) {
    broadcast ⟨PREVOTE, h_p, roundp, id(v, tprop)⟩
  }
  else {
    broadcast ⟨PREVOTE, hp, roundp, nil⟩
  }
  step_p ← prevote
}
```

#### New Rule Replacing Lines 36 - 43

- As above, in the following `(v,t)` is part of the message rather than `v`
- the stored values (i.e., `lockedValue`, `validValue`) do not contain the time

#### **[PBTS-ALG-NEW-PREVOTE.0]**

```go
upon timely(⟨PROPOSAL, h_p, round_p, (v,t), ∗⟩) from proposer(h_p, round_p) AND 2f + 1 ⟨PREVOTE, h_p, round_p, id(v,t)⟩ while valid(v) ∧ step_p ≥ prevote for the first time do {
  if step_p = prevote {
    lockedValue_p ← v
    lockedRound_p ← round_p
    broadcast ⟨PRECOMMIT, h_p, round_p, id(v,t))⟩ 
    step_p ← precommit
  }
  validValue_p ← v 
  validRound_p ← round_p
}
```

#### New Rule Replacing Lines 49 - 54

- we decide on `v` as well as on the time from the proposal message
- here we do not care whether the proposal was received timely.

> In particular we need to take care of the case where the proposer is untimely to one correct validator only. We need to ensure that this validator decides if all decide.

#### **[PBTS-ALG-DECIDE.0]**

```go
upon ⟨PROPOSAL, h_p, r, (v,t), ∗⟩ from proposer(h_p, r) AND 2f + 1 ⟨PRECOMMIT, h_p, r, id(v,t)⟩ while decisionp[h_p] = nil do {
  if valid(v) {
    decision_p [h_p] = (v,t) // decide on time too
    h_p ← h_p + 1
    reset lockedRound_p , lockedValue_p, validRound_p and validValue_p to initial values and empty message log 
    StartRound(0)
  }
}
```

**All other rules remains unchanged.**

Back to [main document][main_v1].

[main_v1]: ./pbts_001_draft.md

[algorithm]: ../pbts-algorithm.md
[algorithm_v1]: ./pbts-algorithm_001_draft.md

[arXiv]: https://arxiv.org/abs/1807.04938
