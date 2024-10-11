---
order: 1
---
# Byzantine Consensus Algorithm

## Terms

- The network is composed of optionally connected _nodes_. Nodes
  directly connected to a particular node are called _peers_.
- The consensus process in deciding the next block (at some _height_
  `H`) is composed of one or many _rounds_.
- `NewHeight`, `Propose`, `Prevote`, `Precommit`, and `Commit`
  represent state machine states of a round. (aka `RoundStep` or
  just "step").
- A node is said to be _at_ a given height, round, and step, or at
  `(H,R,S)`, or at `(H,R)` in short to omit the step.
- To _prevote_ or _precommit_ something means to broadcast a prevote
  or precommit [vote](https://github.com/cometbft/cometbft/blob/af3bc47df982e271d4d340a3c5e0d773e440466d/types/vote.go#L50)
  for something.
- A vote _at_ `(H,R)` is a vote signed with the bytes for `H` and `R`
  included in its [sign-bytes](../core/data_structures.md#vote).
- _+2/3_ is short for "more than 2/3"
- _1/3+_ is short for "1/3 or more"
- A set of +2/3 of prevotes for a particular block or `<nil>` at
  `(H,R)` is called a _proof-of-lock-change_ or _PoLC_ for short.

## State Machine Overview

At each height of the blockchain a round-based protocol is run to
determine the next block. Each round is composed of three _steps_
(`Propose`, `Prevote`, and `Precommit`), along with two special steps
`Commit` and `NewHeight`.

In the optimal scenario, the order of steps is:

```md
NewHeight -> (Propose -> Prevote -> Precommit)+ -> Commit -> NewHeight ->...
```

The sequence `(Propose -> Prevote -> Precommit)` is called a _round_.
There may be more than one round required to commit a block at a given
height. Examples for why more rounds may be required include:

- The designated proposer was not online.
- The block proposed by the designated proposer was not valid.
- The block proposed by the designated proposer did not propagate
  in time.
- The block proposed was valid, but +2/3 of prevotes for the proposed
  block were not received in time for enough validator nodes by the
  time they reached the `Precommit` step. Even though +2/3 of prevotes
  are necessary to progress to the next step, at least one validator
  may have voted `<nil>` or maliciously voted for something else.
- The block proposed was valid, and +2/3 of prevotes were received for
  enough nodes, but +2/3 of precommits for the proposed block were not
  received for enough validator nodes.

Some of these problems are resolved by moving onto the next round &
proposer. Others are resolved by increasing certain round timeout
parameters over each successive round.

## State Machine Diagram

```md
                         +-------------------------------------+
                         v                                     |(Wait til `CommmitTime+timeoutCommit`)
                   +-----------+                         +-----+-----+
      +----------> |  Propose  +--------------+          | NewHeight |
      |            +-----------+              |          +-----------+
      |                                       |                ^
      |(Else, after timeoutPrecommit)         v                |
+-----+-----+                           +-----------+          |
| Precommit |  <------------------------+  Prevote  |          |
+-----+-----+                           +-----------+          |
      |(When +2/3 Precommits for block found)                  |
      v                                                        |
+--------------------------------------------------------------------+
|  Commit                                                            |
|                                                                    |
|  * Set CommitTime = now;                                           |
|  * Wait for block, then stage/save/commit block;                   |
+--------------------------------------------------------------------+
```

# Background Gossip

A node may not have a corresponding validator private key, but it
nevertheless plays an active role in the consensus process by relaying
relevant meta-data, proposals, blocks, and votes to its peers. A node
that has the private keys of an active validator and is engaged in
signing votes is called a _validator-node_. All nodes (not just
validator-nodes) have an associated state (the current height, round,
and step) and work to make progress.

Between two nodes there exists a `Connection`, and multiplexed on top of
this connection are fairly throttled `Channel`s of information. An
epidemic gossip protocol is implemented among some of these channels to
bring peers up to speed on the most recent state of consensus. For
example,

- Nodes gossip `PartSet` parts of the current round's proposer's
  proposed block. A LibSwift inspired algorithm is used to quickly
  broadcast blocks across the gossip network.
- Nodes gossip prevote/precommit votes. A node `NODE_A` that is ahead
  of `NODE_B` can send `NODE_B` prevotes or precommits for `NODE_B`'s
  current (or future) round to enable it to progress forward.
- Nodes gossip prevotes for the proposed PoLC (proof-of-lock-change)
  round if one is proposed.
- Nodes gossip to nodes lagging in blockchain height with block
  [commits](https://github.com/cometbft/cometbft/blob/af3bc47df982e271d4d340a3c5e0d773e440466d/types/block.go#L738)
  for older blocks.
- Nodes opportunistically gossip `ReceivedVote` messages to hint peers what
  votes it already has.
- Nodes broadcast their current state to all neighboring peers. (but
  is not gossiped further)

There's more, but let's not get ahead of ourselves here.

## Proposals

A proposal is signed and published by the designated proposer at each
round. The proposer is chosen by a deterministic and non-choking round
robin selection algorithm that selects proposers in proportion to their
voting power (see
[implementation](https://github.com/cometbft/cometbft/blob/af3bc47df982e271d4d340a3c5e0d773e440466d/types/validator_set.go#L51)).

A proposal at `(H,R)` is composed of a block and an optional latest
`PoLC-Round < R` which is included iff the proposer knows of one. This
hints the network to allow nodes to unlock (when safe) to ensure the
liveness property.

## State Machine Spec

### Propose Step (height:H,round:R)

Upon entering `Propose`:

- The designated proposer proposes a block at `(H,R)`.

The `Propose` step ends:

- After `timeoutProposeR` after entering `Propose`. --> goto
  `Prevote(H,R)`
- After receiving proposal block and all prevotes at `PoLC-Round`. -->
  goto `Prevote(H,R)`
- After [common exit conditions](#common-exit-conditions)

### Prevote Step (height:H,round:R)

Upon entering `Prevote`, each validator broadcasts its prevote vote.

- First, if the validator is locked on a block since `LastLockRound`
  but now has a PoLC for something else at round `PoLC-Round` where
  `LastLockRound < PoLC-Round < R`, then it unlocks.
- If the validator is still locked on a block, it prevotes that.
- Else, if the proposed block from `Propose(H,R)` is good, it
  prevotes that.
- Else, if the proposal is invalid or wasn't received on time, it
  prevotes `<nil>`.

The `Prevote` step ends:

- After +2/3 prevotes for a particular block or `<nil>`. -->; goto
  `Precommit(H,R)`
- After `timeoutPrevote` after receiving any +2/3 prevotes. --> goto
  `Precommit(H,R)`
- After [common exit conditions](#common-exit-conditions)

### Precommit Step (height:H,round:R)

Upon entering `Precommit`, each validator broadcasts its precommit vote.

- If the validator has a PoLC at `(H,R)` for a particular block `B`, it
  (re)locks (or changes lock to) and precommits `B` and sets
  `LastLockRound = R`.
- Else, if the validator has a PoLC at `(H,R)` for `<nil>`, it unlocks
  and precommits `<nil>`.
- Else, it keeps the lock unchanged and precommits `<nil>`.

A precommit for `<nil>` means "I didn’t see a PoLC for this round, but I
did get +2/3 prevotes and waited a bit".

The Precommit step ends:

- After +2/3 precommits for `<nil>`. --> goto `Propose(H,R+1)`
- After `timeoutPrecommit` after receiving any +2/3 precommits. --> goto
  `Propose(H,R+1)`
- After [common exit conditions](#common-exit-conditions)

### Common exit conditions

- After +2/3 precommits for a particular block. --> goto
  `Commit(H)`
- After any +2/3 prevotes received at `(H,R+x)`. --> goto
  `Prevote(H,R+x)`
- After any +2/3 precommits received at `(H,R+x)`. --> goto
  `Precommit(H,R+x)`

### Commit Step (height:H)

- Set `CommitTime = now()`
- Wait until block is received. --> goto `NewHeight(H+1)`

### NewHeight Step (height:H)

- Move `Precommits` to `LastCommit` and increment height.
- Set `StartTime = CommitTime+timeoutCommit`
- Wait until `StartTime` to receive straggler commits. --> goto
  `Propose(H,0)`

## Proofs

### Proof of Safety

Assume that at most -1/3 of the voting power of validators is byzantine.
If a validator commits block `B` at round `R`, it's because it saw +2/3
of precommits at round `R`. This implies that 1/3+ of honest nodes are
still locked at round `R' > R`. These locked validators will remain
locked until they see a PoLC at `R' > R`, but this won't happen because
1/3+ are locked and honest, so at most -2/3 are available to vote for
anything other than `B`.

### Proof of Liveness

If 1/3+ honest validators are locked on two different blocks from
different rounds, a proposers' `PoLC-Round` will eventually cause nodes
locked from the earlier round to unlock. Eventually, the designated
proposer will be one that is aware of a PoLC at the later round. Also,
`timeoutProposalR` increments with round `R`, while the size of a
proposal are capped, so eventually the network is able to "fully gossip"
the whole proposal (e.g. the block & PoLC).

### Proof of Fork Accountability

Define the JSet (justification-vote-set) at height `H` of a validator
`V1` to be all the votes signed by the validator at `H` along with
justification PoLC prevotes for each lock change. For example, if `V1`
signed the following precommits: `Precommit(B1 @ round 0)`,
`Precommit(<nil> @ round 1)`, `Precommit(B2 @ round 4)` (note that no
precommits were signed for rounds 2 and 3, and that's ok),
`Precommit(B1 @ round 0)` must be justified by a PoLC at round 0, and
`Precommit(B2 @ round 4)` must be justified by a PoLC at round 4; but
the precommit for `<nil>` at round 1 is not a lock-change by definition
so the JSet for `V1` need not include any prevotes at round 1, 2, or 3
(unless `V1` happened to have prevoted for those rounds).

Further, define the JSet at height `H` of a set of validators `VSet` to
be the union of the JSets for each validator in `VSet`. For a given
commit by honest validators at round `R` for block `B` we can construct
a JSet to justify the commit for `B` at `R`. We say that a JSet
_justifies_ a commit at `(H,R)` if all the committers (validators in the
commit-set) are each justified in the JSet with no duplicitous vote
signatures (by the committers).

- **Lemma**: When a fork is detected by the existence of two
  conflicting [commits](../core/data_structures.md#commit), the
  union of the JSets for both commits (if they can be compiled) must
  include double-signing by at least 1/3+ of the validator set.
  **Proof**: The commit cannot be at the same round, because that
  would immediately imply double-signing by 1/3+. Take the union of
  the JSets of both commits. If there is no double-signing by at least
  1/3+ of the validator set in the union, then no honest validator
  could have precommitted any different block after the first commit.
  Yet, +2/3 did. Reductio ad absurdum.

As a corollary, when there is a fork, an external process can determine
the blame by requiring each validator to justify all of its round votes.
Either we will find 1/3+ who cannot justify at least one of their votes,
and/or, we will find 1/3+ who had double-signed.

### Alternative algorithm

Alternatively, we can take the JSet of a commit to be the "full commit".
That is, if light clients and validators do not consider a block to be
committed unless the JSet of the commit is also known, then we get the
desirable property that if there ever is a fork (e.g. there are two
conflicting "full commits"), then 1/3+ of the validators are immediately
punishable for double-signing.

There are many ways to ensure that the gossip network efficiently share
the JSet of a commit. One solution is to add a new message type that
tells peers that this node has (or does not have) a +2/3 majority for B
(or) at (H,R), and a bitarray of which votes contributed towards that
majority. Peers can react by responding with appropriate votes.

We will implement such an algorithm for the next iteration of the
consensus protocol.

Other potential improvements include adding more data in votes such as
the last known PoLC round that caused a lock change, and the last voted
round/step (or, we may require that validators not skip any votes). This
may make JSet verification/gossip logic easier to implement.

### Censorship Attacks

Due to the definition of a block
[commit](https://github.com/cometbft/cometbft/blob/main/docs/core/validators.md), any 1/3+ coalition of
validators can halt the blockchain by not broadcasting their votes. Such
a coalition can also censor particular transactions by rejecting blocks
that include these transactions, though this would result in a
significant proportion of block proposals to be rejected, which would
slow down the rate of block commits of the blockchain, reducing its
utility and value. The malicious coalition might also broadcast votes in
a trickle so as to grind blockchain block commits to a near halt, or
engage in any combination of these attacks.

If a global active adversary were also involved, it can partition the
network in such a way that it may appear that the wrong subset of
validators were responsible for the slowdown. This is not just a
limitation of Tendermint, but rather a limitation of all consensus
protocols whose network is potentially controlled by an active
adversary.

### Overcoming Forks and Censorship Attacks

For these types of attacks, a subset of the validators through external
means should coordinate to sign a reorg-proposal that chooses a fork
(and any evidence thereof) and the initial subset of validators with
their signatures. Validators who sign such a reorg-proposal forego its
collateral on all other forks. Clients should verify the signatures on
the reorg-proposal, verify any evidence, and make a judgement or prompt
the end-user for a decision. For example, a phone wallet app may prompt
the user with a security warning, while a refrigerator may accept any
reorg-proposal signed by +1/2 of the original validators.

No non-synchronous Byzantine fault-tolerant algorithm can come to
consensus when 1/3+ of validators are dishonest, yet a fork assumes that
1/3+ of validators have already been dishonest by double-signing or
lock-changing without justification. So, signing the reorg-proposal is a
coordination problem that cannot be solved by any non-synchronous
protocol (i.e. automatically, and without making assumptions about the
reliability of the underlying network). It must be provided by means
external to the weakly-synchronous Tendermint consensus algorithm. For
now, we leave the problem of reorg-proposal coordination to human
coordination via internet media. Validators must take care to ensure
that there are no significant network partitions, to avoid situations
where two conflicting reorg-proposals are signed.

Assuming that the external coordination medium and protocol is robust,
it follows that forks are less of a concern than [censorship
attacks](#censorship-attacks).

### Canonical vs subjective commit

We distinguish between "canonical" and "subjective" commits. A subjective commit is what
each validator sees locally when they decide to commit a block. The canonical commit is
what is included by the proposer of the next block in the `LastCommit` field of
the block. This is what makes it canonical and ensures every validator agrees on the canonical commit,
even if it is different from the +2/3 votes a validator has seen, which caused the validator to
commit the respective block. Each block contains a canonical +2/3 commit for the previous
block.
