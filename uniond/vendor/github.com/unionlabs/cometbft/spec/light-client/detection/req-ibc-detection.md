# Requirements for Fork Detection in the IBC Context

## What you need to know about IBC

In the following, I distilled what I considered relevant from

<https://github.com/cosmos/ibc>

### Components and their interface

#### Cosmos Blockchains

> I assume you know what that is.

#### An IBC/Cosmos correspondence

| IBC Term | Cosmos Spec Term | Comment |
|----------|-------------------------| --------|
| `CommitmentRoot` | AppState | app hash |
| `ConsensusState` | Lightblock | not all fields are there. NextValidator is definitely needed |
| `ClientState` | latest light block + configuration parameters (e.g., trusting period + `frozenHeight` |  NextValidators missing; what is `proofSpecs`?|
| `frozenHeight` | height of fork | set when a fork is detected |
| "would-have-been-fooled" | light node fork detection | light node may submit proof of fork to IBC component to halt it |
| `Height` | (no epochs) | (epoch,height) pair in lexicographical order (`compare`) |
| `Header` | ~signed header | validatorSet explicit (no hash); nextValidators missing |
| `Evidence` | t.b.d. | definition unclear "which the light client would have considered valid". Data structure will need to change |
| `verify` | `ValidAndVerified` | signature does not match perfectly (ClientState vs. LightBlock) + in `checkMisbehaviorAndUpdateState` it is unclear whether it uses traces or goes to h1 and h2 in one step |

#### Some IBC links

- [QueryConsensusState](https://github.com/cosmos/cosmos-sdk/blob/2651427ab4c6ea9f81d26afa0211757fc76cf747/x/ibc/02-client/client/utils/utils.go#L68)
  
#### Required Changes in ICS 007

- `assert(height > 0)` in definition of `initialize` doesn't match
  definition of `Height` as *(epoch,height)* pair.
  
- `initialize` needs to be updated to new data structures

- `clientState.frozenHeight` semantics seem not totally consistent in
  document. E.g., `min` needs to be defined over optional value in
  `checkMisbehaviorAndUpdateState`. Also, if you are frozen, why do
  you accept more evidence.

- `checkValidityAndUpdateState`
    - `verify`: it needs to be clarified that checkValidityAndUpdateState
      does not perform "bisection" (as currently hinted in the text) but
   performs a single step of "skipping verification", called,
      `ValidAndVerified`
    - `assert (header.height > clientState.latestHeight)`: no old
      headers can be installed. This might be OK, but we need to check
      interplay with misbehavior
    - clienstState needs to be updated according to complete data
      structure

- `checkMisbehaviorAndUpdateState`: as evidence will contain a trace
  (or two), the assertion that uses verify will need to change.

- ICS 002 states w.r.t. `queryChainConsensusState` that "Note that
  retrieval of past consensus states by height (as opposed to just the
  current consensus state) is convenient but not required." For
  Cosmos fork detection, this seems to be a necessity.
  
- `Header` should become a lightblock

- `Evidence` should become `LightNodeProofOfFork` [LCV-DATA-POF.1]

- `upgradeClientState` what is the semantics (in particular what is
  `height` doing?).
  
- `checkMisbehaviorAndUpdateState(cs: ClientState, PoF:
  LightNodeProofOfFork)` needs to be adapted

#### Handler

A blockchain runs a **handler** that passively collects information about
  other blockchains. It can be thought of a state machine that takes
  input events.
  
- the state includes a lightstore (I guess called `ConsensusState`
  in IBC)

- The following function is used to pass a header to a handler
  
```go
type checkValidityAndUpdateState = (Header) => Void
```

  For Cosmos, it will perform
  `ValidandVerified`, that is, it does the trusting period check and the
  +1/3 check (+2/3 for sequential headers).
  If it verifies a header, it adds it to its lightstore,
  if it does not pass verification it drops it.
  Right now it only accepts a header more recent then the latest
  header,
  and drops older
  ones or ones that could not be verified.

> The above paragraph captures what I believe what is the current
  logic of `checkValidityAndUpdateState`. It may be subject to
  change. E.g., maintain a lightstore with state (unverified, verified)

- The following function is used to pass  "evidence" (this we
  will need to make precise eventually) to a handler
  
```go
type checkMisbehaviorAndUpdateState = (bytes) => Void
```

  We have to design this, and the data that the handler can use to
  check that there was some misbehavior (fork) in order react on
  it, e.g., flagging a situation and
  stop the protocol.

- The following function is used to query the light store (`ConsensusState`)

```go
type queryChainConsensusState = (height: uint64) => ConsensusState
```

#### Relayer

- The active components are called **relayer**.

- a relayer contains light clients to two (or more?) blockchains

- the relayer send headers and data to the handler to invoke
  `checkValidityAndUpdateState` and
  `checkMisbehaviorAndUpdateState`. It may also query
  `queryChainConsensusState`.
  
- multiple relayers may talk to one handler. Some relayers might be
  faulty. We assume existence of at least single correct relayer.

## Informal Problem Statement: Fork detection in IBC
  
### Relayer requirement: Evidence for Handler

- The relayer should provide the handler with
  "evidence" that there was a fork.
  
- The relayer can read the handler's consensus state. Thus the relayer can
  feed the handler precisely the information the handler needs to detect a
  fork.
  What is this
  information needs to be specified.
  
- The information depends on the verification the handler does. It
  might be necessary to provide a bisection proof (list of
  lightblocks) so that the handler can verify based on its local
  lightstore a header *h* that is conflicting with a header *h'* in the
  local lightstore, that is, *h != h'* and *h.Height = h'.Height*
  
### Relayer requirement: Fork detection

Let's assume there is a fork at chain A. There are two ways the
relayer can figure that out:

1. as the relayer contains a light client for A, it also includes a fork
   detector that can detect a fork.

2. the relayer may also detect a fork by observing that the
   handler for chain A (on chain B)
   is on a different branch than the relayer

- in both detection scenarios, the relayer should submit evidence to
  full nodes of chain A where there is a fork. As we assume a fullnode
  has a complete list of blocks, it is sufficient to send "Bucky's
  evidence" (<https://github.com/tendermint/tendermint/issues/5083>),
  that is,
    - two lightblocks from different branches +
    - a lightblock (perhaps just a height) from which both blocks
    can be verified.
  
- in the scenario 2., the relayer must feed the A-handler (on chain B)
  a proof of a fork on A so that chain B can react accordingly
  
### Handler requirement
  
- there are potentially many relayers, some correct some faulty

- a handler cannot trust the information provided by the relayer,
  but must verify
  (Доверя́й, но проверя́й)

- in case of a fork, we accept that the handler temporarily stores
  headers (tagged as verified).
  
- eventually, a handler should be informed
 (`checkMisbehaviorAndUpdateState`)
 by some relayer that it has
  verified a header from a fork. Then the handler should do what is
 required by IBC in this case (stop?)

### Challenges in the handler requirement

- handlers and relayers work on different lightstores. In principle
  the lightstore need not intersect in any heights a priori

- if a relayer  sees a header *h* it doesn't know at a handler (`queryChainConsensusState`), the
  relayer needs to
  verify that header. If it cannot do it locally based on downloaded
  and verified (trusted?) light blocks, it might need to use
  `VerifyToTarget` (bisection). To call `VerifyToTarget` we might keep
  *h* in the lightstore. If verification fails, we need to download the
  "alternative" header of height *h.Height* to generate evidence for
  the handler.
  
- we have to specify what precisely `queryChainConsensusState`
  returns. It cannot be the complete lightstore. Is the last header enough?

- we would like to assume that every now and then (smaller than the
  trusting period) a correct relayer checks whether the handler is on a
  different branch than the relayer.
  And we would like that this is enough to achieve
  the Handler requirement.
  
    - here the correctness argument would be easy if a correct relayer is
     based on a light client with a *trusted* state, that is, a light
     client who never changes its opinion about trusted. Then if such a
     correct relayer checks-in with a handler, it will detect a fork, and
     act in time.

    - if the light client does not provide this interface, in the case of
     a fork, we need some assumption about a correct relayer being on a
     different branch than the handler, and we need such a relayer to
  check-in not too late. Also
     what happens if the relayer's light client is forced to roll-back
     its lightstore?
     Does it have to re-check all handlers?

## On the interconnectedness of things

In the broader discussion of so-called "fork accountability" there are
several subproblems

- Fork detection

- Evidence creation and submission

- Isolating misbehaving nodes (and report them for punishment over abci)

### Fork detection

The preliminary specification ./detection.md formalizes the notion of
a fork. Roughly, a fork exists if there are two conflicting headers
for the same height, where both are supported by bonded full nodes
(that have been validators in the near past, that is, within the
trusting period). We distinguish between *fork on the chain* where two
conflicting blocks are signed by +2/3 of the validators of that
height, and a *light client fork* where one of the conflicting headers
is not signed by  +2/3 of the current height, but by +1/3 of the
validators of some smaller height.

In principle everyone can detect a fork

- ./detection talks about the Cosmos light client with a focus on
  light nodes. A relayer runs such light clients and may detect
  forks in this way

- in IBC, a relayer can see that a handler is on a conflicting branch
    - the relayer should feed the handler the necessary information so
      that it can halt
    - the relayer should report the fork to a full node

### Evidence creation and submission

- the information sent from the relayer to the handler could be called
  evidence, but this is perhaps a bad idea because the information sent to a
  full node can also be called evidence. But this evidence might still
  not be enough as the full node might need to run the "fork
  accountability" protocol to generate evidence in the form of
  consensus messages. So perhaps we should
  introduce different terms for:
  
    - proof of fork for the handler (basically consisting of lightblocks)
    - proof of fork for a full node (basically consisting of (fewer) lightblocks)
    - proof of misbehavior (consensus messages)
  
### Isolating misbehaving nodes

- this is the job of a full node.

- might be subjective in the future: the protocol depends on what the
  full node believes is the "correct" chain. Right now we postulate
  that every full node is on the correct chain, that is, there is no
  fork on the chain.
  
- The full node figures out which nodes are
    - lunatic
    - double signing
    - amnesic; **using the challenge response protocol**

- We do not punish "phantom" validators
    - currently we understand a phantom validator as a node that
        - signs a block for a height in which it is not in the
          validator set
        - the node is not part of the +1/3 of previous validators that
          are used to support the header. Whether we call a validator
          phantom might be subjective and depend on the header we
          check against. Their formalization actually seems not so
          clear.
    - they can only do something if there are +1/3 faulty validators
      that are either lunatic, double signing, or amnesic.
    - abci requires that we only report bonded validators. So if a
      node is a "phantom", we would need the check whether the node is
      bonded, which currently is expensive, as it requires checking
      blocks from the last three weeks.
    - in the future, with state sync, a correct node might be
      convinced by faulty nodes that it is in the validator set. Then
      it might appear to be "phantom" although it behaves correctly

## Next steps

> The following points are subject to my limited knowledge of the
> state of the work on IBC. Some/most of it might already exist and we
> will just need to bring everything together.

- "proof of fork for a full node" defines a clean interface between
  fork detection and misbehavior isolation. So it should be produced
  by protocols (light client, the relayer). So we should fix that
  first.
  
- Given the problems of not having a light client architecture spec,
  for the relayer we should start with this. E.g.
  
    - the relayer runs light clients for two chains
    - the relayer regularly queries consensus state of a handler
    - the relayer needs to check the consensus state
        - this involves local checks
        - this involves calling the light client
    - the relayer uses the light client to do IBC business (channels,
      packets, connections, etc.)
    - the relayer submits proof of fork to handlers and full nodes

> the list is definitely not complete. I think part of this
> (perhaps all)  is
> covered by what Anca presented recently.

We will need to define what we expect from these components

- for the parts where the relayer talks to the handler, we need to fix
  the interface, and what the handler does

- we write specs for these components.
