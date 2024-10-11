# RFC 017: ABCI++ Vote Extension Propagation

## Changelog

- 11-Apr-2022: Initial draft (@sergio-mena).
- 15-Apr-2022: Addressed initial comments. First complete version (@sergio-mena).
- 09-May-2022: Addressed all outstanding comments.

## Abstract

According to the
[ABCI++ specification](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/abci%2B%2B/README.md)
(as of 11-Apr-2022), a validator MUST provide a signed vote extension for each non-`nil` precommit vote
of height *h* that it uses to propose a block in height *h+1*. When a validator is up to
date, this is easy to do, but when a validator needs to catch up this is far from trivial as this data
cannot be retrieved from the blockchain.

This RFC presents and compares the different options to address this problem, which have been proposed
in several discussions by the Tendermint Core team.

## Document Structure

The RFC is structured as follows. In the [Background](#background) section,
subsections [Problem Description](#problem-description) and [Cases to Address](#cases-to-address)
explain the problem at hand from a high level perspective, i.e., abstracting away from the current
Tendermint implementation. In contrast, subsection
[Current Catch-up Mechanisms](#current-catch-up-mechanisms) delves into the details of the current
Tendermint code.

In the [Discussion](#discussion) section, subsection [Solutions Proposed](#solutions-proposed) is also
worded abstracting away from implementation details, whilst subsections
[Feasibility of the Proposed Solutions](#feasibility-of-the-proposed-solutions) and
[Current Limitations and Possible Implementations](#current-limitations-and-possible-implementations)
analyze the viability of one of the proposed solutions in the context of Tendermint's architecture
based on reactors. Finally, [Formalization Work](#formalization-work) briefly discusses the work
still needed demonstrate the correctness of the chosen solution.

The high level subsections are aimed at readers who are familiar with consensus algorithms, in
particular with the one described in the Tendermint (white paper), but who are not necessarily
acquainted with the details of the Tendermint codebase. The other subsections, which go into
implementation details, are best understood by engineers with deep knowledge of the implementation of
Tendermint's blocksync and consensus reactors.

## Background

### Basic Definitions

This document assumes that all validators have equal voting power for the sake of simplicity. This is done
without loss of generality.

There are two types of votes in Tendermint: *prevotes* and *precommits*. Votes can be `nil` or refer to
a proposed block. This RFC focuses on precommits,
also known as *precommit votes*. In this document we sometimes call them simply *votes*.

Validators send precommit votes to their peer nodes in *precommit messages*. According to the
[ABCI++ specification](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/abci%2B%2B/README.md),
a precommit message MUST also contain a *vote extension*.
This mandatory vote extension can be empty, but MUST be signed with the same key as the precommit
vote (i.e., the sending validator's).
Nevertheless, the vote extension is signed independently from the vote, so a vote can be separated from
its extension.
The reason for vote extensions to be mandatory in precommit messages is that, otherwise, a (malicious)
node can omit a vote extension while still providing/forwarding/sending the corresponding precommit vote.

The validator set at height *h* is denoted *valset<sub>h</sub>*. A *commit* for height *h* consists of more
than *2n<sub>h</sub>/3* precommit votes voting for a block *b*, where *n<sub>h</sub>* denotes the size of
*valset<sub>h</sub>*. A commit does not contain `nil` precommit votes, and all votes in it refer to the
same block. An *extended commit* is a *commit* where every precommit vote has its respective vote extension
attached.

### Problem Description

In the version of [ABCI](https://github.com/tendermint/spec/blob/4fb99af/spec/abci/README.md) present up to
Tendermint v0.35, for any height *h*, a validator *v* MUST have the decided block *b* and a commit for
height *h* in order to decide at height *h*. Then, *v* just needs a commit for height *h* to propose at
height *h+1*, in the rounds of *h+1* where *v* is a proposer.

In [ABCI++](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/abci%2B%2B/README.md),
the information that a validator *v* MUST have to be able to decide in *h* does not change with
respect to pre-existing ABCI: the decided block *b* and a commit for *h*.
In contrast, for proposing in *h+1*, a commit for *h* is not enough: *v* MUST now have an extended
commit.

When a validator takes an active part in consensus at height *h*, it has all the data it needs in memory,
in its consensus state, to decide on *h* and propose in *h+1*. Things are not so easy in the cases when
*v* cannot take part in consensus because it is late (e.g., it falls behind, it crashes
and recovers, or it just starts after the others). If *v* does not take part, it cannot actively
gather precommit messages (which include vote extensions) in order to decide.
Before ABCI++, this was not a problem: full nodes are supposed to persist past blocks in the block store,
so other nodes would realise that *v* is late and send it the missing decided block at height *h* and
the corresponding commit (kept in block *h+1*) so that *v* can catch up.
However, we cannot apply this catch-up technique for ABCI++, as the vote extensions, which are part
of the needed *extended commit* are not part of the blockchain.

### Cases to Address

Before we tackle the description of the possible cases we need to address, let us describe the following
incremental improvement to the ABCI++ logic. Upon decision, a full node persists (e.g., in the block
store) the extended commit that allowed the node to decide. For the moment, let us assume the node only
needs to keep its *most recent* extended commit, and MAY remove any older extended commits from persistent
storage.
This improvement is so obvious that all solutions described in the [Discussion](#discussion) section use
it as a building block. Moreover, it completely addresses by itself some of the cases described in this
subsection.

We now describe the cases (i.e. possible *runs* of the system) that have been raised in different
discussions and need to be addressed. They are (roughly) ordered from easiest to hardest to deal with.

- **(a)** *Happy path: all validators advance together, no crash*.

    This case is included for completeness. All validators have taken part in height *h*.
    Even if some of them did not manage to send a precommit message for the decided block, they all
    receive enough precommit messages to be able to decide. As vote extensions are mandatory in
    precommit messages, every validator *v* trivially has all the information, namely the decided block
    and the extended commit, needed to propose in height *h+1* for the rounds in which *v* is the
    proposer.

    No problem to solve here.

- **(b)** *All validators advance together, then all crash at the same height*.

    This case has been raised in some discussions, the main concern being whether the vote extensions
    for the previous height would be lost across the network. With the improvement described above,
    namely persisting the latest extended commit at decision time, this case is solved.
    When a crashed validator recovers, it recovers the last extended commit from persistent storage
    and handshakes with the Application.
    If need be, it also reconstructs messages for the unfinished height
    (including all precommits received) from the WAL.
    Then, the validator can resume where it was at the time of the crash. Thus, as extensions are
    persisted, either in the WAL (in the form of received precommit messages), or in the latest
    extended commit, the only way that vote extensions needed to start the next height could be lost
    forever would be if all validators crashed and never recovered (e.g. disk corruption).
    Since a *correct* node MUST eventually recover, this violates Tendermint's assumption of more than
    *2n<sub>h</sub>/3* correct validators for every height *h*.

    No problem to solve here.

- **(c)** *Lagging majority*.

    Let us assume the validator set does not change between *h* and *h+1*.
    It is not possible by the nature of the Tendermint algorithm, which requires more
    than *2n<sub>h</sub>/3* precommit votes for some round of height *h* in order to make progress.
    So, only up to *n<sub>h</sub>/3* validators can lag behind.

    On the other hand, for the case where there are changes to the validator set between *h* and
    *h+1* please see case (d) below, where the extreme case is discussed.

- **(d)** *Validator set changes completely between* h *and* h+1.

    If sets *valset<sub>h</sub>* and *valset<sub>h+1</sub>* are disjoint,
    more than *2n<sub>h</sub>/3* of validators in height *h* should
    have actively participated in consensus in *h*. So, as of height *h*, only a minority of validators
    in *h* can be lagging behind, although they could all lag behind from *h+1* on, as they are no
    longer validators, only full nodes. This situation falls under the assumptions of case (h) below.

    As for validators in *valset<sub>h+1</sub>*, as they were not validators as of height *h*, they
    could all be lagging behind by that time. However, by the time *h* finishes and *h+1* begins, the
    chain will halt until more than *2n<sub>h+1</sub>/3* of them have caught up and started consensus
    at height *h+1*. If set *valset<sub>h+1</sub>* does not change in *h+2* and subsequent
    heights, only up to *n<sub>h+1</sub>/3* validators will be able to lag behind. Thus, we have
    converted this case into case (h) below.

- **(e)** *Enough validators crash to block the rest*.

    In this case, blockchain progress halts, i.e. surviving full nodes keep increasing rounds
    indefinitely, until some of the crashed validators are able to recover.
    Those validators that recover first will handshake with the Application and recover at the height
    they crashed, which is still the same the nodes that did not crash are stuck in, so they don't need
    to catch up.
    Further, they had persisted the extended commit for the previous height. Nothing to solve.

    For those validators recovering later, we are in case (h) below.

- **(f)** *Some validators crash, but not enough to block progress*.

    When the correct processes that crashed recover, they handshake with the Application and resume at
    the height they were at when they crashed. As the blockchain did not stop making progress, the
    recovered processes are likely to have fallen behind with respect to the progressing majority.

    At this point, the recovered processes are in case (h) below.

- **(g)** *A new full node starts*.

    The reasoning here also applies to the case when more than one full node are starting.
    When the full node starts from scratch, it has no state (its current height is 0). Ignoring
    statesync for the time being, the node just needs to catch up by applying past blocks one by one
    (after verifying them).

    Thus, the node is in case (h) below.

- **(h)** *Advancing majority, lagging minority*

    In this case, some nodes are late. More precisely, at the present time, a set of full nodes,
    denoted *L<sub>h<sub>p</sub></sub>*, are falling behind
    (e.g., temporary disconnection or network partition, memory thrashing, crashes, new nodes)
    an arbitrary
    number of heights:
    between *h<sub>s</sub>* and *h<sub>p</sub>*, where *h<sub>s</sub> < h<sub>p</sub>*, and
    *h<sub>p</sub>* is the highest height
    any correct full node has reached so far.

    The correct full nodes that reached *h<sub>p</sub>* were able to decide for *h<sub>p</sub>-1*.
    Therefore, less than *n<sub>h<sub>p</sub>-1</sub>/3* validators of *h<sub>p</sub>-1* can be part
    of *L<sub>h<sub>p</sub></sub>*, since enough up-to-date validators needed to actively participate
    in consensus for *h<sub>p</sub>-1*.

    Since, at the present time,
    no node in *L<sub>h<sub>p</sub></sub>* took part in any consensus between
    *h<sub>s</sub>* and *h<sub>p</sub>-1*,
    the reasoning above can be extended to validator set changes between *h<sub>s</sub>* and
    *h<sub>p</sub>-1*. This results in the following restriction on the full nodes that can be part of *L<sub>h<sub>p</sub></sub>*.

    - &forall; *h*, where *h<sub>s</sub> ≤ h < h<sub>p</sub>*,
    | *valset<sub>h</sub>* &cap; *L<sub>h<sub>p</sub></sub>*  | *< n<sub>h</sub>/3*

    If this property does not hold for a particular height *h*, where
    *h<sub>s</sub> ≤ h < h<sub>p</sub>*, Tendermint could not have progressed beyond *h* and
    therefore no full node could have reached *h<sub>p</sub>* (a contradiction).

    These lagging nodes in *L<sub>h<sub>p</sub></sub>* need to catch up. They have to obtain the
    information needed to make
    progress from other nodes. For each height *h* between *h<sub>s</sub>* and *h<sub>p</sub>-2*,
    this includes the decided block for *h*, and the
    precommit votes also for *deciding h* (which can be extracted from the block at height *h+1*).

    At a given height  *h<sub>c</sub>* (where possibly *h<sub>c</sub> << h<sub>p</sub>*),
    a full node in *L<sub>h<sub>p</sub></sub>* will consider itself *caught up*, based on the
    (maybe out of date) information it is getting from its peers. Then, the node needs to be ready to
    propose at height *h<sub>c</sub>+1*, which requires having received the vote extensions for
    *h<sub>c</sub>*.
    As the vote extensions are *not* stored in the blocks, and it is difficult to have strong
    guarantees on *when* a late node considers itself caught up, providing the late node with the right
    vote extensions for the right height poses a problem.

At this point, we have described and compared all cases raised in discussions leading up to this
RFC. The list above aims at being exhaustive. The analysis of each case included above makes all of
them converge into case (h).

### Current Catch-up Mechanisms

We now briefly describe the current catch-up mechanisms in the reactors concerned in Tendermint.

#### Statesync

Full nodes optionally run statesync just after starting, when they start from scratch.
If statesync succeeds, an Application snapshot is installed, and Tendermint jumps from height 0 directly
to the height the Application snapshop represents, without applying the block of any previous height.
Some light blocks are received and stored in the block store for running light-client verification of
all the skipped blocks. Light blocks are incomplete blocks, typically containing the header and the
canonical commit but, e.g., no transactions. They are stored in the block store as "signed headers".

The statesync reactor is not really relevant for solving the problem discussed in this RFC. We will
nevertheless mention it when needed; in particular, to understand some corner cases.

#### Blocksync

The blocksync reactor kicks in after start up or recovery (and, optionally, after statesync is done)
and sends the following messages to its peers:

- `StatusRequest` to query the height its peers are currently at, and
- `BlockRequest`, asking for blocks of heights the local node is missing.

Using `BlockResponse` messages received from peers, the blocksync reactor validates each received
block using the block of the following height, saves the block in the block store, and sends the
block to the Application for execution.

If blocksync has validated and applied the block for the height *previous* to the highest seen in
a `StatusResponse` message, or if no progress has been made after a timeout, the node considers
itself as caught up and switches to the consensus reactor.

#### Consensus Reactor

The consensus reactor runs the full Tendermint algorithm. For a validator this means it has to
propose blocks, and send/receive prevote/precommit messages, as mandated by Tendermint, before it can
decide and move on to the next height.

If a full node that is running the consensus reactor falls behind at height *h*, when a peer node
realises this it will retrieve the canonical commit of *h+1* from the block store, and *convert*
it into a set of precommit votes and will send those to the late node.

## Discussion

### Solutions Proposed

These are the solutions proposed in discussions leading up to this RFC.

- **Solution 0.** *Vote extensions are made **best effort** in the specification*.

    This is the simplest solution, considered as a way to provide vote extensions in a simple enough
    way so that it can be part of v0.36.
    It consists in changing the specification so as to not *require* that precommit votes used upon
    `PrepareProposal` contain their corresponding vote extensions. In other words, we render vote
    extensions optional.
    There are strong implications stemming from such a relaxation of the original specification.

    - As a vote extension is signed *separately* from the vote it is extending, an intermediate node
      can now remove (i.e., censor) vote extensions from precommit messages at will.
    - Further, there is no point anymore in the spec requiring the Application to accept a vote extension
      passed via `VerifyVoteExtension` to consider a precommit message valid in its entirety. Remember
      this behavior of `VerifyVoteExtension` is adding a constraint to Tendermint's conditions for
      liveness.
      In this situation, it is better and simpler to just drop the vote extension rejected by the
      Application via `VerifyVoteExtension`, but still consider the precommit vote itself valid as long
      as its signature verifies.

- **Solution 1.** *Include vote extensions in the blockchain*.

    Another obvious solution, which has somehow been considered in the past, is to include the vote
    extensions and their signatures in the blockchain.
    The blockchain would thus include the extended commit, rather than a regular commit, as the structure
    to be canonicalized in the next block.
    With this solution, the current mechanisms implemented both in the blocksync and consensus reactors
    would still be correct, as all the information a node needs to catch up, and to start proposing when
    it considers itself as caught-up, can now be recovered from past blocks saved in the block store.

    This solution has two main drawbacks.

    - As the block format must change, upgrading a chain requires a hard fork. Furthermore,
      all existing light client implementations will stop working until they are upgraded to deal with
      the new format (e.g., how certain hashes calculated and/or how certain signatures are checked).
      For instance, let us consider IBC, which relies on light clients. An IBC connection between
      two chains will be broken if only one chain upgrades.
    - The extra information (i.e., the vote extensions) that is now kept in the blockchain is not really
        needed *at every height* for a late node to catch up.
        - This information is only needed to be able to *propose* at the height the validator considers
          itself as caught-up. If a validator is indeed late for height *h*, it is useless (although
          correct) for it to call `PrepareProposal`, or `ExtendVote`, since the block is already decided.
        - Moreover, some use cases require pretty sizeable vote extensions, which would result in an
          important waste of space in the blockchain.

- **Solution 2.** *Skip* propose *step in Tendermint algorithm*.

    This solution consists in modifying the Tendermint algorithm to skip the *send proposal* step in
    heights where the node does not have the required vote extensions to populate the call to
    `PrepareProposal`. The main idea behind this is that it should only happen when the validator is late
    and, therefore, up-to-date validators have already proposed (and decided) for that height.
    A small variation of this solution is, rather than skipping the *send proposal* step, the validator
    sends a special *empty* or *bottom* (⊥) proposal to signal other nodes that it is not ready to propose
    at (any round of) the current height.

    The appeal of this solution is its simplicity. A possible implementation does not need to extend
    the data structures, or change the current catch-up mechanisms implemented in the blocksync or
    in the consensus reactor. When we lack the needed information (vote extensions), we simply rely
    on another correct validator to propose a valid block in other rounds of the current height.

    However, this solution can be attacked by a byzantine node in the network in the following way.
    Let us consider the following scenario:

    - all validators in *valset<sub>h</sub>* send out precommit messages, with vote extensions,
      for height *h*, round 0, roughly at the same time,
    - all those precommit messages contain non-`nil` precommit votes, which vote for block *b*
    - all those precommit messages sent in height *h*, round 0, and all messages sent in
      height *h*, round *r > 0* get delayed indefinitely, so,
    - all validators in *valset<sub>h</sub>* keep waiting for enough precommit
      messages for height *h*, round 0, needed for deciding in height *h*
    - an intermediate (malicious) full node *m* manages to receive block *b*, and gather more than
      *2n<sub>h</sub>/3* precommit messages for height *h*, round 0,
    - one way or another, the solution should have either (a) a mechanism for a full node to *tell*
      another full node it is late, or (b) a mechanism for a full node to conclude it is late based
      on other full nodes' messages; any of these mechanisms should, at the very least,
      require the late node receiving the decided block and a commit (not necessarily an extended
      commit) for *h*,
    - node *m* uses the gathered precommit messages to build a commit for height *h*, round 0,
    - in order to convince full nodes that they are late, node *m* either (a) *tells* them they
      are late, or (b) shows them it (i.e. *m*) is ahead, by sending them block *b*, along with the
      commit for height *h*, round 0,
    - all full nodes conclude they are late from *m*'s behavior, and use block *b* and the commit for
      height *h*, round 0, to decide on height *h*, and proceed to height *h+1*.

    At this point, *all* full nodes, including all validators in *valset<sub>h+1</sub>*, have advanced
    to height *h+1* believing they are late, and so, expecting the *hypothetical* leading majority of
    validators in *valset<sub>h+1</sub>* to propose for *h+1*. As a result, the blockchain
    grinds to a halt.
    A (rather complex) ad-hoc mechanism would need to be carried out by node operators to roll
    back all validators to the precommit step of height *h*, round *r*, so that they can regenerate
    vote extensions (remember vote extensions are non-deterministic) and continue execution.

- **Solution 3.** *Require extended commits to be available at switching time*.

    This one is more involved than all previous solutions, and builds on an idea present in Solution 2:
    vote extensions are actually not needed for Tendermint to make progress as long as the
    validator is *certain* it is late.

    We define two modes. The first is denoted *catch-up mode*, and Tendermint only calls
    `FinalizeBlock` for each height when in this mode. The second is denoted *consensus mode*, in
    which the validator considers itself up to date and fully participates in consensus and calls
    `PrepareProposal`/`ProcessProposal`, `ExtendVote`, and `VerifyVoteExtension`, before calling
    `FinalizeBlock`.

    The catch-up mode does not need vote extension information to make progress, as all it needs is the
    decided block at each height to call `FinalizeBlock` and keep the state-machine replication making
    progress. The consensus mode, on the other hand, does need vote extension information when
    starting every height.

    Validators are in consensus mode by default. When a validator in consensus mode falls behind
    for whatever reason, e.g. cases (b), (d), (e), (f), (g), or (h) above, we introduce the following
    key safety property:

    - for every height *h<sub>p</sub>*, a full node *f* in *h<sub>p</sub>* refuses to switch to catch-up
        mode **until** there exists a height *h'* such that:
        - *p* has received and (light-client) verified the blocks of
          all heights *h*, where *h<sub>p</sub> ≤ h ≤ h'*
        - it has received an extended commit for *h'* and has verified:
            - the precommit vote signatures in the extended commit
            - the vote extension signatures in the extended commit: each is signed with the same
              key as the precommit vote it extends

    If the condition above holds for *h<sub>p</sub>*, namely receiving a valid sequence of blocks in
    the *f*'s future, and an extended commit corresponding to the last block in the sequence, then
    node *f*:

    - switches to catch-up mode,
    - applies all blocks between *h<sub>p</sub>* and *h'* (calling `FinalizeBlock` only), and
    - switches back to consensus mode using the extended commit for *h'* to propose in the rounds of
      *h' + 1* where it is the proposer.

    This mechanism, together with the invariant it uses, ensures that the node cannot be attacked by
    being fed a block without extensions to make it believe it is late, in a similar way as explained
    for Solution 2.

### Feasibility of the Proposed Solutions

Solution 0, besides the drawbacks described in the previous section, provides guarantees that are
weaker than the rest. The Application does not have the assurance that more than *2n<sub>h</sub>/3* vote
extensions will *always* be available when calling `PrepareProposal` at height *h+1*.
This level of guarantees is probably not strong enough for vote extensions to be useful for some
important use cases that motivated them in the first place, e.g., encrypted mempool transactions.

Solution 1, while being simple in that the changes needed in the current Tendermint codebase would
be rather small, is changing the block format, and would therefore require all blockchains using
Tendermint v0.35 or earlier to hard-fork when upgrading to v0.36.

Since Solution 2 can be attacked, one might prefer Solution 3, even if it is more involved
to implement. Further, we must elaborate on how we can turn Solution 3, described in abstract
terms in the previous section, into a concrete implementation compatible with the current
Tendermint codebase.

### Current Limitations and Possible Implementations

The main limitations affecting the current version of Tendermint are the following.

- The current version of the blocksync reactor does not use the full
  [light client verification](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/light-client/README.md)
  algorithm to validate blocks coming from other peers.
- The code being structured into the blocksync and consensus reactors, only switching from the
  blocksync reactor to the consensus reactor is supported; switching in the opposite direction is
  not supported. Alternatively, the consensus reactor could have a mechanism allowing a late node
  to catch up by skipping calls to `PrepareProposal`/`ProcessProposal`, and
  `ExtendVote`/`VerifyVoteExtension` and only calling `FinalizeBlock` for each height.
  Such a mechanism does not exist at the time of writing this RFC.

The blocksync reactor featuring light client verification is being actively worked on (tentatively
for v0.37). So it is best if this RFC does not try to delve into that problem, but just makes sure
its outcomes are compatible with that effort.

In subsection [Cases to Address](#cases-to-address), we concluded that we can focus on
solving case (h) in theoretical terms.
However, as the current Tendermint version does not yet support switching back to blocksync once a
node has switched to consensus, we need to split case (h) into two cases. When a full node needs to
catch up...

- **(h.1)** ... it has not switched yet from the blocksync reactor to the consensus reactor, or

- **(h.2)** ... it has already switched to the consensus reactor.

This is important in order to discuss the different possible implementations.

#### Base Implementation: Persist and Propagate Extended Commit History

In order to circumvent the fact that we cannot switch from the consensus reactor back to blocksync,
rather than just keeping the few most recent extended commits, nodes will need to keep
and gossip a backlog of extended commits so that the consensus reactor can still propose and decide
in out-of-date heights (even if those proposals will be useless).

The base implementation － for which an experimental patch exists － consists in the conservative
approach of persisting in the block store *all* extended commits for which we have also stored
the full block. Currently, when statesync is run at startup, it saves light blocks.
This base implementation does not seek
to receive or persist extended commits for those light blocks as they would not be of any use.

Then, we modify the blocksync reactor so that peers *always* send requested full blocks together
with the corresponding extended commit in the `BlockResponse` messages. This guarantees that the
block store being reconstructed by blocksync has the same information as that of peers that are
up to date (at least starting from the latest snapshot applied by statesync before starting blocksync).
Thus, blocksync has all the data it requires to switch to the consensus reactor, as long as one of
the following exit conditions are met:

- The node is still at height 0 (where no commit or extended commit is needed)
- The node has processed at least 1 block in blocksync

The second condition is needed in case the node has installed an Application snapshot during statesync.
If that is the case, at the time blocksync starts, the block store only has the data statesync has saved:
light blocks, and no extended commits.
Hence we need to blocksync at least one block from another node, which will be sent with its corresponding extended commit, before we can switch to consensus.

As a side note, a chain might be started at a height *h<sub>i</sub> > 0*, all other heights
*h < h<sub>i</sub>* being non-existent. In this case, the chain is still considered to be at height 0 before
block *h<sub>i</sub>* is applied, so the first condition above allows the node to switch to consensus even
if blocksync has not processed any block (which is always the case if all nodes are starting from scratch).

When a validator falls behind while having already switched to the consensus reactor, a peer node can
simply retrieve the extended commit for the required height from the block store and reconstruct a set of
precommit votes together with their extensions and send them in the form of precommit messages to the
validator falling behind, regardless of whether the peer node holds the extended commit because it
actually participated in that consensus and thus received the precommit messages, or it received the extended commit via a `BlockResponse` message while running blocksync.

This solution requires a few changes to the consensus reactor:

- upon saving the block for a given height in the block store at decision time, save the
  corresponding extended commit as well
- in the catch-up mechanism, when a node realizes that another peer is more than 2 heights
  behind, it uses the extended commit (rather than the canonical commit as done previously) to
  reconstruct the precommit votes with their corresponding extensions

The changes to the blocksync reactor are more substantial:

- the `BlockResponse` message is extended to include the extended commit of the same height as
  the block included in the response (just as they are stored in the block store)
- structure `bpRequester` is likewise extended to hold the received extended commits coming in
  `BlockResponse` messages
- method `PeekTwoBlocks` is modified to also return the extended commit corresponding to the first block
- when successfully verifying a received block, the reactor saves its corresponding extended commit in
  the block store

The two main drawbacks of this base implementation are:

- the increased size taken by the block store, in particular with big extensions
- the increased bandwidth taken by the new format of `BlockResponse`

#### Possible Optimization: Pruning the Extended Commit History

If we cannot switch from the consensus reactor back to the blocksync reactor we cannot prune the extended commit backlog in the block store without sacrificing the implementation's correctness. The asynchronous
nature of our distributed system model allows a process to fall behind an arbitrary number of
heights, and thus all extended commits need to be kept *just in case* a node that late had
previously switched to the consensus reactor.

However, there is a possibility to optimize the base implementation. Every time we enter a new height,
we could prune from the block store all extended commits that are more than *d* heights in the past.
Then, we need to handle two new situations, roughly equivalent to cases (h.1) and (h.2) described above.

- (h.1) A node starts from scratch or recovers after a crash. In this case, we need to modify the
    blocksync reactor's base implementation.
    - when receiving a `BlockResponse` message, it MUST accept that the extended commit set to `nil`,
    - when sending a `BlockResponse` message, if the block store contains the extended commit for that
      height, it MUST set it in the message, otherwise it sets it to `nil`,
    - the exit conditions used for the base implementation are no longer valid; the only reliable exit
      condition now consists in making sure that the last block processed by blocksync was received with
      the corresponding commit, and not `nil`; this extended commit will allow the node to switch from
      the blocksync reactor to the consensus reactor and immediately act as a proposer if required.
- (h.2) A node already running the consensus reactor falls behind beyond *d* heights. In principle,
  the node will be stuck forever as no other node can provide the vote extensions it needs to make
  progress (they all have pruned the corresponding extended commit).
  However we can manually have the node crash and recover as a workaround. This effectively converts
  this case into (h.1).

### Formalization Work

A formalization work to show or prove the correctness of the different use cases and solutions
presented here (and any other that may be found) needs to be carried out.
A question that needs a precise answer is how many extended commits (one?, two?) a node needs
to keep in persistent memory when implementing Solution 3 described above without Tendermint's
current limitations.
Another important invariant we need to prove formally is that the set of vote extensions
required to make progress will always be held somewhere in the network.

## References

- [ABCI++ specification](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/abci%2B%2B/README.md)
- [ABCI as of v0.35](https://github.com/tendermint/spec/blob/4fb99af/spec/abci/README.md)
- [Vote extensions issue](https://github.com/tendermint/tendermint/issues/8174)
- [Light client verification](https://github.com/tendermint/tendermint/blob/4743a7ad0/spec/light-client/README.md)
