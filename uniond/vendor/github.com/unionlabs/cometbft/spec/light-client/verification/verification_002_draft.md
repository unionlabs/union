# Light Client Verification

The light client implements a read operation of a
[header][CMBC-HEADER-link] from the [blockchain][CMBC-SEQ-link], by
communicating with full nodes.  As some full nodes may be faulty, this
functionality must be implemented in a fault-tolerant way.

In a Cosmos blockchain, the validator set may change with every
new block.  The staking and unbonding mechanism induces a [security
model][CMBC-FM-2THIRDS-link]: starting at time *Time* of the
[header][CMBC-HEADER-link],
more than two-thirds of the next validators of a new block are correct
for the duration of *TrustedPeriod*. The fault-tolerant read
operation is designed for this security model.

The challenge addressed here is that the light client might have a
block of height *h1* and needs to read the block of height *h2*
greater than *h1*.  Checking all headers of heights from *h1* to *h2*
might be too costly (e.g., in terms of energy for mobile devices).
This specification tries to reduce the number of intermediate blocks
that need to be checked, by exploiting the guarantees provided by the
[security model][CMBC-FM-2THIRDS-link].

# Status

## Previous Versions

- [[001_published]](./verification_001_published.md)
 is thoroughly reviewed, and the protocol has been
formalized in TLA+ and model checked.

## Issues that are addressed in this revision

As it is part of the larger light node, its data structures and
functions interact with the attack detection functionality of the light
client. As a result of the work on

- [attack detection](https://github.com/tendermint/spec/pull/164) for light nodes

- attack detection for IBC and [relayer requirements](https://github.com/informalsystems/tendermint-rs/issues/497)

- light client
  [supervisor](https://github.com/tendermint/spec/pull/159) (also in
  [Rust proposal](https://github.com/informalsystems/tendermint-rs/pull/509))
  
adaptations to the semantics and functions exposed by the LightStore
needed to be made. In contrast to [version
001](./verification_001_published.md) we specify the following:

- `VerifyToTarget` and `Backwards` are called with a single lightblock
  as root of trust in contrast to passing the complete lightstore.

- During verification, we record for each lightblock which other
  lightblock can be used to verify it in one step. This is needed to
  generate verification traces that are needed for IBC.
  
# Outline

- [Part I](#part-i---cosmos-blockchain): Introduction of
 relevant terms of the Cosmos
blockchain.

- [Part II](#part-ii---sequential-definition-of-the-verification-problem): Introduction
of the problem addressed by the Lightclient Verification protocol.
    - [Verification Informal Problem
      statement](#verification-informal-problem-statement): For the general
      audience, that is, engineers who want to get an overview over what
      the component is doing from a bird's eye view.
    - [Sequential Problem statement](#sequential-problem-statement):
      Provides a mathematical definition of the problem statement in
      its sequential form, that is, ignoring the distributed aspect of
      the implementation of the blockchain.

- [Part III](#part-iii---light-client-as-distributed-system): Distributed
  aspects of the light client, system assumptions and temporal
  logic specifications.

    - [Incentives](#incentives): how faulty full nodes may benefit from
    misbehaving and how correct full nodes benefit from cooperating.
  
    - [Computational Model](#computational-model):
      timing and correctness assumptions.

    - [Distributed Problem Statement](#distributed-problem-statement):
      temporal properties that formalize safety and liveness
      properties in the distributed setting.

- [Part IV](#part-iv---light-client-verification-protocol):
  Specification of the protocols.

    - [Definitions](#definitions): Describes inputs, outputs,
       variables used by the protocol, auxiliary functions

    - [Core Verification](#core-verification): gives an outline of the solution,
       and details of the functions used (with preconditions,
       postconditions, error conditions).

    - [Liveness Scenarios](#liveness-scenarios): when the light
       client makes progress depends heavily on the changes in the
       validator sets of the blockchain. We discuss some typical scenarios.

- [Part V](#part-v---supporting-the-ibc-relayer): The above parts
  focus on a common case where the last verified block has height *h1*
  and the
  requested height *h2* satisfies *h2 > h1*. For IBC, there are
  scenarios where this might not be the case. In this part, we provide
  some preliminaries for supporting this. As not all details of the
  IBC requirements are clear by now, we do not provide a complete
  specification at this point. We mark with "Open Question" points
  that need to be addressed in order to finalize this specification.
  It should be noted that the technically
  most challenging case is the one specified in Part IV.

In this document we quite extensively use tags in order to be able to
reference assumptions, invariants, etc. in future communication. In
these tags we frequently use the following short forms:

- CMBC: Cosmos blockchain
- SEQ: for sequential specifications
- LCV: Lightclient Verification
- LIVE: liveness
- SAFE: safety
- FUNC: function
- INV: invariant
- A: assumption

# Part I - Cosmos Blockchain

## Header Fields necessary for the Light Client

#### **[CMBC-HEADER.1]**

A set of blockchain transactions is stored in a data structure called
*block*, which contains a field called *header*. (The data structure
*block* is defined [here][block]).  As the header contains hashes to
the relevant fields of the block, for the purpose of this
specification, we will assume that the blockchain is a list of
headers, rather than a list of blocks.

#### **[CMBC-HASH-UNIQUENESS.1]**

We assume that every hash in the header identifies the data it hashes.
Therefore, in this specification, we do not distinguish between hashes and the
data they represent.

#### **[CMBC-HEADER-FIELDS.2]**

A header contains the following fields:

- `Height`: non-negative integer
- `Time`: time (non-negative integer)
- `LastBlockID`: Hashvalue
- `LastCommit` DomainCommit
- `Validators`: DomainVal
- `NextValidators`: DomainVal
- `Data`: DomainTX
- `AppState`: DomainApp
- `LastResults`: DomainRes

#### **[CMBC-SEQ.1]**

The Cosmos blockchain is a list *chain* of headers.

#### **[CMBC-VALIDATOR-PAIR.1]**

Given a full node, a
*validator pair* is a pair *(peerID, voting_power)*, where

- *peerID* is the PeerID (public key) of a full node,
- *voting_power* is an integer (representing the full node's
  voting power in a certain consensus instance).
  
> In the Golang implementation the data type for *validator
pair* is called `Validator`

#### **[CMBC-VALIDATOR-SET.1]**

A *validator set* is a set of validator pairs. For a validator set
*vs*, we write *TotalVotingPower(vs)* for the sum of the voting powers
of its validator pairs.

#### **[CMBC-VOTE.1]**

A *vote* contains a `prevote` or `precommit` message sent and signed by
a validator node during the execution of [consensus][arXiv]. Each
message contains the following fields

- `Type`: prevote or precommit
- `Height`: positive integer
- `Round` a positive integer
- `BlockID` a Hashvalue of a block (not necessarily a block of the chain)

#### **[CMBC-COMMIT.1]**

A commit is a set of `precommit` message.

## Cosmos Failure Model

#### **[CMBC-AUTH-BYZ.1]**

We assume the authenticated Byzantine fault model in which no node (faulty or
correct) may break digital signatures, but otherwise, no additional
assumption is made about the internal behavior of faulty
nodes. That is, faulty nodes are only limited in that they cannot forge
messages.

#### **[CMBC-TIME-PARAMS.1]**

A Cosmos blockchain has the following configuration parameters:

- *unbondingPeriod*: a time duration.
- *trustingPeriod*: a time duration smaller than *unbondingPeriod*.

#### **[CMBC-CORRECT.1]**

We define a predicate *correctUntil(n, t)*, where *n* is a node and *t* is a
time point.
The predicate *correctUntil(n, t)* is true if and only if the node *n*
follows all the protocols (at least) until time *t*.

#### **[CMBC-FM-2THIRDS.1]**

If a block *h* is in the chain,
then there exists a subset *CorrV*
of *h.NextValidators*, such that:

- *TotalVotingPower(CorrV) > 2/3
    TotalVotingPower(h.NextValidators)*; cf. [CMBC-VALIDATOR-SET.1]
- For every validator pair *(n,p)* in *CorrV*, it holds *correctUntil(n,
    h.Time + trustingPeriod)*; cf. [CMBC-CORRECT.1]

> The definition of correct
> [**[CMBC-CORRECT.1]**][CMBC-CORRECT-link] refers to realtime, while it
> is used here with *Time* and *trustingPeriod*, which are "hardware
> times".  We do not make a distinction here.

#### **[CMBC-CORR-FULL.1]**

Every correct full node locally stores a prefix of the
current list of headers from [**[CMBC-SEQ.1]**][CMBC-SEQ-link].

## What the Light Client Checks

> From [CMBC-FM-2THIRDS.1] we directly derive the following observation:

#### **[CMBC-VAL-CONTAINS-CORR.1]**

Given a (trusted) block *tb* of the blockchain, a given set of full nodes
*N* contains a correct node at a real-time *t*, if

- *t - trustingPeriod < tb.Time < t*
- the voting power in tb.NextValidators of nodes in *N* is more
     than 1/3 of *TotalVotingPower(tb.NextValidators)*

> The following describes how a commit for a given block *b* must look
> like.

#### **[CMBC-SOUND-DISTR-POSS-COMMIT.1]**

For a block *b*, each element *pc* of *PossibleCommit(b)* satisfies:

- *pc* contains only votes (cf. [CMBC-VOTE.1])
  by validators from *b.Validators*
- the sum of the voting powers in *pc* is greater than 2/3
  *TotalVotingPower(b.Validators)*
- and there is an *r* such that  each vote *v* in *pc* satisfies
    - v.Type = precommit
    - v.Height = b.Height
    - v.Round = r
    - v.blockID = hash(b)

> The following property comes from the validity of the [consensus][arXiv]: A
> correct validator node only sends `prevote` or `precommit`, if
> `BlockID` of the new (to-be-decided) block is equal to the hash of
> the last block.

#### **[CMBC-VAL-COMMIT.1]**

If for a block *b*,  a commit *c*

- contains at least one validator pair *(v,p)* such that *v* is a
    **correct** validator node, and
- is contained in *PossibleCommit(b)*
  
then the block *b* is on the blockchain.

## Context of this document

In this document we specify the light client verification component,
called *Core Verification*.  The *Core Verification* communicates with
a full node.  As full nodes may be faulty, it cannot trust the
received information, but the light client has to check whether the
header it receives coincides with the one generated by Tendermint
consensus.

The two
 properties [[CMBC-VAL-CONTAINS-CORR.1]][CMBC-VAL-CONTAINS-CORR-link] and
[[CMBC-VAL-COMMIT]][CMBC-VAL-COMMIT-link]  formalize the checks done
 by this specification:
Given a trusted block *tb* and an untrusted block *ub* with a commit *cub*,
one has to check that *cub* is in *PossibleCommit(ub)*, and that *cub*
contains a correct node using *tb*.

# Part II - Sequential Definition of the Verification Problem

## Verification Informal Problem statement

Given a height *targetHeight* as an input, the *Verifier* eventually
stores a header *h* of height *targetHeight* locally.  This header *h*
is generated by the Cosmos [blockchain][block]. In
particular, a header that was not generated by the blockchain should
never be stored.

## Sequential Problem statement

#### **[LCV-SEQ-LIVE.1]**

The *Verifier* gets as input a height *targetHeight*, and eventually stores the
header of height *targetHeight* of the blockchain.

#### **[LCV-SEQ-SAFE.1]**

The *Verifier* never stores a header which is not in the blockchain.

# Part III - Light Client as Distributed System

## Incentives

Faulty full nodes may benefit from lying to the light client, by making the
light client accept a block that deviates (e.g., contains additional
transactions) from the one generated by Tendermint consensus.
Users using the light client might be harmed by accepting a forged header.

The [attack detector][attack-detector] of the light client may help the
correct full nodes to understand whether their header is a good one.
Hence, in combination with the light client detector, the correct full
nodes have the incentive to respond.  We can thus base liveness
arguments on the assumption that correct full nodes reliably talk to
the light client.

## Computational Model

#### **[LCV-A-PEER.1]**

The verifier communicates with a full node called *primary*. No assumption is made about the full node (it may be correct or faulty).

#### **[LCV-A-COMM.1]**

Communication between the light client and a correct full node is
reliable and bounded in time. Reliable communication means that
messages are not lost, not duplicated, and eventually delivered. There
is a (known) end-to-end delay *Delta*, such that if a message is sent
at time *t* then it is received and processes by time *t + Delta*.
This implies that we need a timeout of at least *2 Delta* for remote
procedure calls to ensure that the response of a correct peer arrives
before the timeout expires.

#### **[LCV-A-TFM.1]**

The Cosmos blockchain satisfies the Cosmos failure model [**[CMBC-FM-2THIRDS.1]**][CMBC-FM-2THIRDS-link].

#### **[LCV-A-VAL.1]**

The system satisfies [**[CMBC-AUTH-BYZ.1]**][CMBC-Auth-Byz-link] and
[**[CMBC-FM-2THIRDS.1]**][CMBC-FM-2THIRDS-link]. Thus, there is a
blockchain that satisfies the soundness requirements (that is, the
validation rules in [[block]]).

## Distributed Problem Statement

### Two Kinds of Termination

We do not assume that *primary* is correct. Under this assumption no
protocol can guarantee the combination of the sequential
properties. Thus, in the (unreliable) distributed setting, we consider
two kinds of termination (successful and failure) and we will specify
below under what (favorable) conditions *Core Verification* ensures to
terminate successfully, and satisfy the requirements of the sequential
problem statement:

#### **[LCV-DIST-TERM.1]**

*Core Verification* either *terminates
successfully* or it *terminates with failure*.

### Design choices

#### **[LCV-DIST-STORE.2]**

*Core Verification* returns a data structure called *LightStore* that
contains light blocks (that contain a header).

#### **[LCV-DIST-INIT.2]**

*Core Verification* is called with

- *primary*: the PeerID of a full node (with verification communicates)
- *root*: a light block (the root of trust)
- *targetHeight*: a height (the height of a header that should be obtained)

### Temporal Properties

#### **[LCV-DIST-SAFE.2]**

It is always the case that every header in *LightStore* was
generated by an instance of Tendermint consensus.

#### **[LCV-DIST-LIVE.2]**

If a new instance of *Core Verification* is called with a
height *targetHeight* greater than root.Header.Height it must
must eventually terminate.

- If
    - the  *primary* is correct (and locally has the block of
       *targetHeight*), and
    - the age of root is always less than the trusting period,  
    then *Core Verification* adds a verified header *hd* with height
    *targetHeight* to *LightStore* and it **terminates successfully**

> These definitions imply that if the primary is faulty, a header may or
> may not be added to *LightStore*. In any case,
> [**[LCV-DIST-SAFE.2]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-safe2) must hold.
> The invariant [**[LCV-DIST-SAFE.2]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-safe2) and the liveness
> requirement [**[LCV-DIST-LIVE.2]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-life)
> allow that verified headers are added to *LightStore* whose
> height was not passed
> to the verifier (e.g., intermediate headers used in bisection; see below).
> Note that for liveness, initially having a *root* within
> the *trustinPeriod* is not sufficient. However, as this
> specification will leave some freedom with respect to the strategy
> in which order to download intermediate headers, we do not give a
> more precise liveness specification here. After giving the
> specification of the protocol, we will discuss some liveness
> scenarios [below](#liveness-scenarios).

### Solving the sequential specification

This specification provides a partial solution to the sequential specification.
The *Verifier* solves the invariant of the sequential part

[**[LCV-DIST-SAFE.2]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-safe2) => [**[LCV-SEQ-SAFE.1]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-safe1)

In the case the primary is correct, and *root*  is a recent header in *LightStore*, the verifier satisfies the liveness requirements.

⋀ *primary is correct*  
⋀ *root.header.Time* > *now* - *trustingPeriod*  
⋀ [**[LCV-A-Comm.1]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-a-comm) ⋀ (
       ( [**[CMBC-CorrFull.1]**][CMBC-CorrFull-link] ⋀
         [**[LCV-DIST-LIVE.2]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-live2) )
       ⟹ [**[LCV-SEQ-LIVE.1]**](https://github.com/cometbft/cometbft/blob/main/spec/light-client/verification/verification_001_published.md#lcv-dist-live1)
)

# Part IV - Light Client Verification Protocol

We provide a specification for Light Client Verification. The local
code for verification is presented by a sequential function
`VerifyToTarget` to highlight the control flow of this functionality.
We note that if a different concurrency model is considered for
an implementation, the sequential flow of the function may be
implemented with mutexes, etc. However, the light client verification
is partitioned into three blocks that can be implemented and tested
independently:

- `FetchLightBlock` is called to download a light block (header) of a
  given height from a peer.
- `ValidAndVerified` is a local code that checks the header.
- `Schedule` decides which height to try to verify next. We keep this
  underspecified as different implementations (currently in Goland and
  Rust) may implement different optimizations here. We just provide
  necessary conditions on how the height may evolve.
  
<!-- > `ValidAndVerified` is the function that is sometimes called "Light -->
<!-- > Client" in the IBC context. -->

## Definitions

### Data Types

The core data structure of the protocol is the LightBlock.

#### **[LCV-DATA-LIGHTBLOCK.1]**

```go
type LightBlock struct {
  Header          Header
  Commit          Commit
  Validators      ValidatorSet
}
```

#### **[LCV-DATA-LIGHTSTORE.2]**

LightBlocks are stored in a structure which stores all LightBlock from
initialization or received from peers.

```go
type LightStore struct {
 ...
}

```

#### **[LCV-DATA-LS-ROOT.2]**

For each lightblock in a lightstore we record in a field `verification-root` of
type Height.

> `verification-root` records the height of a lightblock that can be used to verify
> the lightblock in one step

#### **[LCV-INV-LS-ROOT.2]**

At all times, if a lightblock *b* in a lightstore has *b.verification-root = h*,
then

- the lightstore contains a lightblock with height *h*, or
- *b* has the minimal height of all lightblocks in lightstore, then
  b.verification-root should be nil.

The LightStore exposes the following functions to query stored LightBlocks.

#### **[LCV-DATA-LS-STATE.1]**

Each LightBlock is in one of the following states:

```go
type VerifiedState int

const (
 StateUnverified = iota + 1
 StateVerified
 StateFailed
 StateTrusted
)
```

#### **[LCV-FUNC-GET.1]**

```go
func (ls LightStore) Get(height Height) (LightBlock, bool)
```

- Expected postcondition
    - returns a LightBlock at a given height or false in the second argument if
    the LightStore does not contain the specified LightBlock.

#### **[LCV-FUNC-LATEST.1]**

```go
func (ls LightStore) Latest() LightBlock
```

- Expected postcondition
    - returns the highest light block

#### **[LCV-FUNC-ADD.1]**

```go
func (ls LightStore) Add(newBlock)
```

- Expected precondition
    - the lightstore is empty
- Expected postcondition
    - adds newBlock into light store

#### **[LCV-FUNC-STORE.1]**

```go
func (ls LightStore) store_chain(newLS LightStore)
```

- Expected postcondition
    - adds `newLS` to the lightStore.

#### **[LCV-FUNC-LATEST-VERIF.2]**

```go
func (ls LightStore) LatestVerified() LightBlock
```

- Expected postcondition
    - returns the highest light block whose state is `StateVerified`

#### **[LCV-FUNC-FILTER.1]**

```go
func (ls LightStore) FilterVerified() LightStore
```

- Expected postcondition
    - returns all the lightblocks of the lightstore with state `StateVerified`

#### **[LCV-FUNC-UPDATE.2]**

```go
func (ls LightStore) Update(lightBlock LightBlock, verfiedState
VerifiedState, root-height Height)
```

- Expected postcondition
    - the lightblock is part of the lightstore
    - The state of the LightBlock is set to *verifiedState*.
    - The verification-root of the LightBlock is set to *root-height*

```go
func (ls LightStore) TraceTo(lightBlock LightBlock) (LightBlock, LightStore)
```

- Expected postcondition
    - returns a **trusted** lightblock `root` from the lightstore with a height
      less than `lightBlock`
    - returns a lightstore that contains lightblocks that constitute a
      [verification trace](https://github.com/cometbft/cometbft/tree/main/spec/light-client/detection) from
      `root` to `lightBlock` (including `lightBlock`)

### Inputs

- *root*: A light block that is trusted
- *primary*: peerID
- *targetHeight*: the height of the needed header

### Configuration Parameters

- *trustThreshold*: a float. Can be used if correctness should not be based on more voting power and 1/3.
- *trustingPeriod*: a time duration [**[CMBC-TIME_PARAMS.1]**][CMBC-TIME_PARAMS-link].
- *clockDrift*: a time duration. Correction parameter dealing with only approximately synchronized clocks.

### Variables

- *nextHeight*: initially *targetHeight*
  > *nextHeight* should be thought of the "height of the next header we need
  > to download and verify"

### Assumptions

#### **[LCV-A-INIT.2]**

- *root* is from the blockchain

- *targetHeight > root.Header.Height*

### Invariants

#### **[LCV-INV-TP.1]**

It is always the case that *LightStore.LatestTrusted.Header.Time > now - trustingPeriod*.

> If the invariant is violated, the light client does not have a
> header it can trust. A trusted header must be obtained externally,
> its trust can only be based on social consensus.  
> We use the convention that root is assumed to be verified.

### Used Remote Functions

We use the functions `commit` and `validators` that are provided
by the [RPC client][RPC].

```go
func Commit(height int64) (SignedHeader, error)
```

- Implementation remark
    - RPC to full node *n*
    - JSON sent:

```javascript
// POST /commit
{
 "jsonrpc": "2.0",
 "id": "ccc84631-dfdb-4adc-b88c-5291ea3c2cfb", // UUID v4, unique per request
 "method": "commit",
 "params": {
  "height": 1234
 }
}
```

- Expected precondition
    - header of `height` exists on blockchain
- Expected postcondition
    - if *n* is correct: Returns the signed header of height `height`
  from the blockchain if communication is timely (no timeout)
    - if *n* is faulty: Returns a signed header with arbitrary content
- Error condition
    - if *n* is correct: precondition violated or timeout
    - if *n* is faulty: arbitrary error

----;

```go
func Validators(height int64) (ValidatorSet, error)
```

- Implementation remark
    - RPC to full node *n*
    - JSON sent:

```javascript
// POST /validators
{
 "jsonrpc": "2.0",
 "id": "ccc84631-dfdb-4adc-b88c-5291ea3c2cfb", // UUID v4, unique per request
 "method": "validators",
 "params": {
  "height": 1234
 }
}
```

- Expected precondition
    - header of `height` exists on blockchain
- Expected postcondition
    - if *n* is correct: Returns the validator set of height `height`
  from the blockchain if communication is timely (no timeout)
    - if *n* is faulty: Returns arbitrary validator set
- Error condition
    - if *n* is correct: precondition violated or timeout
    - if *n* is faulty: arbitrary error

----;

### Communicating Function

#### **[LCV-FUNC-FETCH.1]**

  ```go
func FetchLightBlock(peer PeerID, height Height) LightBlock
```

- Implementation remark
    - RPC to peer at *PeerID*
    - calls `Commit` for *height* and `Validators` for *height* and *height+1*
- Expected precondition
    - `height` is less than or equal to height of the peer **[LCV-IO-PRE-HEIGHT.1]**
- Expected postcondition:
    - if *node* is correct:
        - Returns the LightBlock *lb* of height `height`
      that is consistent with the blockchain
        - *lb.provider = peer* **[LCV-IO-POST-PROVIDER.1]**
        - *lb.Header* is a header consistent with the blockchain
        - *lb.Validators* is the validator set of the blockchain at height *nextHeight*
        - *lb.NextValidators* is the validator set of the blockchain at height *nextHeight + 1*
    - if *node* is faulty: Returns a LightBlock with arbitrary content
    [**[CMBC-AUTH-BYZ.1]**][CMBC-Auth-Byz-link]
- Error condition
    - if *n* is correct: precondition violated
    - if *n* is faulty: arbitrary error
    - if *lb.provider != peer*
    - times out after 2 Delta (by assumption *n* is faulty)

----;

## Core Verification

### Outline

The `VerifyToTarget` is the main function and uses the following functions.

- `FetchLightBlock` is called to download the next light block. It is
  the only function that communicates with other nodes
- `ValidAndVerified` checks whether header is valid and checks if a
  new lightBlock should be trusted
  based on a previously verified lightBlock.
- `Schedule` decides which height to try to verify next

In the following description of `VerifyToTarget` we do not deal with error
handling. If any of the above function returns an error, VerifyToTarget just
passes the error on.

#### **[LCV-FUNC-MAIN.2]**

```go
func VerifyToTarget(primary PeerID, root LightBlock,
                    targetHeight Height) (LightStore, Result) {

    lightStore = new LightStore;
    lightStore.Update(root, StateVerified, root.verifiedBy);
    nextHeight := targetHeight;

    for lightStore.LatestVerified.height < targetHeight {

        // Get next LightBlock for verification
        current, found := lightStore.Get(nextHeight)
        if !found {
            current = FetchLightBlock(primary, nextHeight)
            lightStore.Update(current, StateUnverified, nil)
        }

        // Verify
        verdict = ValidAndVerified(lightStore.LatestVerified, current)

        // Decide whether/how to continue
        if verdict == SUCCESS {
            lightStore.Update(current, StateVerified, lightStore.LatestVerified.Height)
        }
        else if verdict == NOT_ENOUGH_TRUST {
            // do nothing
            // the light block current passed validation, but the validator
            // set is too different to verify it. We keep the state of
            // current at StateUnverified. For a later iteration, Schedule
            // might decide to try verification of that light block again.
        }
        else {
            // verdict is some error code
            lightStore.Update(current, StateFailed, nil)
            return (nil, ResultFailure)
        }
        nextHeight = Schedule(lightStore, nextHeight, targetHeight)
    }
    return (lightStore.FilterVerified, ResultSuccess)
}
```

- Expected precondition
    - *root* is within the *trustingPeriod*  **[LCV-PRE-TP.1]**
    - *targetHeight* is greater than the height of *root*
- Expected postcondition:
    - returns *lightStore* that contains a LightBlock that corresponds to a block
     of the blockchain of height *targetHeight*
     (that is, the LightBlock has been added to *lightStore*) **[LCV-POST-LS.1]**
- Error conditions
    - if the precondition is violated
    - if `ValidAndVerified` or `FetchLightBlock` report an error
    - if [**[LCV-INV-TP.1]**](#lcv-inv-tp1) is violated
  
### Details of the Functions

#### **[LCV-FUNC-VALID.2]**

```go
func ValidAndVerified(trusted LightBlock, untrusted LightBlock) Result
```

- Expected precondition:
    - *untrusted* is valid, that is, satisfies the soundness [checks][block]
    - *untrusted* is **well-formed**, that is,
        - *untrusted.Header.Time < now + clockDrift*
        - *untrusted.Validators = hash(untrusted.Header.Validators)*
        - *untrusted.NextValidators = hash(untrusted.Header.NextValidators)*
    - *trusted.Header.Time > now - trustingPeriod*
    - the `Height` and `Time` of `trusted` are smaller than the Height and
  `Time` of `untrusted`, respectively
    - the *untrusted.Header* is well-formed (passes the tests from
     [[block]]), and in particular
        - if the untrusted header `unstrusted.Header` is the immediate
   successor  of  `trusted.Header`, then it holds that
            - *trusted.Header.NextValidators =
    untrusted.Header.Validators*, and
    moreover,
            - *untrusted.Header.Commit*
                - contains signatures by more than two-thirds of the validators
                - contains no signature from nodes that are not in *trusted.Header.NextValidators*
- Expected postcondition:
    - Returns `SUCCESS`:
        - if *untrusted* is the immediate successor of *trusted*, or otherwise,
        - if the signatures of a set of validators that have more than
             *max(1/3,trustThreshold)* of voting power in
             *trusted.Nextvalidators* is contained in
             *untrusted.Commit* (that is, header passes the tests
             [**[CMBC-VAL-CONTAINS-CORR.1]**][CMBC-VAL-CONTAINS-CORR-link]
             and [**[CMBC-VAL-COMMIT.1]**][CMBC-VAL-COMMIT-link])
    - Returns `NOT_ENOUGH_TRUST` if:
        - *untrusted* is *not* the immediate successor of
           *trusted*
     and the  *max(1/3,trustThreshold)* threshold is not reached
           (that is, if
      [**[CMBC-VAL-CONTAINS-CORR.1]**][CMBC-VAL-CONTAINS-CORR-link]
      fails and header is does not violate the soundness
         checks [[block]]).
- Error condition:
    - if precondition violated

----;

#### **[LCV-FUNC-SCHEDULE.1]**

```go
func Schedule(lightStore, nextHeight, targetHeight) Height
```

- Implementation remark: If picks the next height to be verified.
  We keep the precise choice of the next header under-specified. It is
  subject to performance optimizations that do not influence the correctness
- Expected postcondition: **[LCV-SCHEDULE-POST.1]**
   Return *H* s.t.
   1. if *lightStore.LatestVerified.Height = nextHeight* and
      *lightStore.LatestVerified < targetHeight* then  
   *nextHeight < H <= targetHeight*
   2. if *lightStore.LatestVerified.Height < nextHeight* and
      *lightStore.LatestVerified.Height < targetHeight* then  
   *lightStore.LatestVerified.Height < H < nextHeight*
   3. if *lightStore.LatestVerified.Height = targetHeight* then  
     *H =  targetHeight*

> Case i. captures the case where the light block at height *nextHeight*
> has been verified, and we can choose a height closer to the *targetHeight*.
> As we get the *lightStore* as parameter, the choice of the next height can
> depend on the *lightStore*, e.g., we can pick a height for which we have
> already downloaded a light block.
> In Case ii. the header of *nextHeight* could not be verified, and we need to pick a smaller height.
> In Case iii. is a special case when we have verified the *targetHeight*.

### Solving the distributed specification

Analogous to [[001_published]](./verification_001_published.md#solving-the-distributed-specification)

## Liveness Scenarios

Analogous to [[001_published]](./verification_001_published.md#liveness-scenarios)

# Part V - Supporting the IBC Relayer

The above specification focuses on the most common case, which also
constitutes the most challenging task: using the Cosmos [security
model][CMBC-FM-2THIRDS-link] to verify light blocks without
downloading all intermediate blocks. To focus on this challenge, above
we have restricted ourselves to the case where  *targetHeight* is
greater than the height of any trusted header. This simplified
presentation of the algorithm as initially
`lightStore.LatestVerified()` is less than *targetHeight*, and in the
process of verification `lightStore.LatestVerified()` increases until
*targetHeight* is reached.

For [IBC][ibc-rs] there are two additional challenges:

1. it might be that some "older" header is needed, that is,
*targetHeight < lightStore.LatestVerified()*.  The
[supervisor](../supervisor/supervisor_002_draft.md) checks whether it is in this
case by calling `LatestPrevious` and `MinVerified` and if so it calls
`Backwards`. All these functions are specified below.

2. In order to submit proof of a light client attack, a relayer may
   need to submit a verification trace. This it is important to
   compute such a trace efficiently. That it can be done is based on
   the invariant [[LCV-INV-LS-ROOT.2]](#lcv-inv-ls-root2) that needs
   to be maintained by the light client. In particular
   `VerifyToTarget` and `Backwards` need to take care of setting
   `verification-root`.

#### **[LCV-FUNC-LATEST-PREV.2]**

```go
func (ls LightStore) LatestPrevious(height Height) (LightBlock, bool)
```

- Expected postcondition
    - returns a light block *lb* that satisfies:
        - *lb* is in lightStore
        - *lb* is in StateTrusted
        - *lb* is not expired
        - *lb.Header.Height < height*
        - for all *b* in lightStore s.t. *b* is trusted and not expired it
          holds *lb.Header.Height >= b.Header.Height*
    - *false* in the second argument if
      the LightStore does not contain such an *lb*.

----;

#### **[LCV-FUNC-LOWEST.2]**

```go
func (ls LightStore) Lowest() (LightBlock)
```

- Expected postcondition
    - returns the lowest trusted light block within trusting period

----;

#### **[LCV-FUNC-MIN.2]**

```go
func (ls LightStore) MinVerified() (LightBlock, bool)
```

- Expected postcondition
    - returns a light block *lb* that satisfies:
        - *lb* is in lightStore
        - *lb.Header.Height* is minimal in the lightStore
    - *false* in the second argument if
      the LightStore does not contain such an *lb*.

If a height that is smaller than the smallest height in the lightstore
is required, we check the hashes backwards. This is done with the
following function:

#### **[LCV-FUNC-BACKWARDS.2]**

```go
func Backwards (primary PeerID, root LightBlock, targetHeight Height)
               (LightStore, Result) {
  
    lb := root;
    lightStore := new LightStore;
    lightStore.Update(lb, StateTrusted, lb.verifiedBy)

    latest := lb.Header
    for i := lb.Header.height - 1; i >= targetHeight; i-- {
        // here we download height-by-height. We might first download all
        // headers down to targetHeight and then check them.
        current := FetchLightBlock(primary,i)
        if (hash(current) != latest.Header.LastBlockId) {
            return (nil, ResultFailure)
        }
        else {
            // latest and current are linked together by LastBlockId
            // therefore it is not relevant which we verified first
            // for consistency, we store latest was veried using
            // current so that the verifiedBy is always pointing down
            // the chain
            lightStore.Update(current, StateTrusted, nil)
            lightStore.Update(latest, StateTrusted, current.Header.Height)
        }
        latest = current
    }
    return (lightStore, ResultSuccess)
}
```

# References

[[block]] Specification of the block data structure.

[[RPC]] RPC client

[[attack-detector]] The specification of the light client attack detector.

[[fullnode]] Specification of the full node API

[[ibc-rs]] Rust implementation of IBC modules and relayer.

[[lightclient]] The light client ADR [77d2651 on Dec 27, 2019].

[RPC]: https://docs.cometbft.com/v0.34/rpc/

[block]: https://github.com/cometbft/cometbft/blob/main/spec/core/data_structures.md

[CMBC-HEADER-link]: #cmbc-header1
[CMBC-SEQ-link]: #cmbc-seq1
[CMBC-CorrFull-link]: #cmbc-corr-full1
[CMBC-Auth-Byz-link]: #cmbc-auth-byz1
[CMBC-TIME_PARAMS-link]: #cmbc-time-params1
[CMBC-FM-2THIRDS-link]: #cmbc-fm-2thirds1
[CMBC-VAL-CONTAINS-CORR-link]: #cmbc-val-contains-corr1
[CMBC-VAL-COMMIT-link]: #cmbc-val-commit1

[lightclient]: https://github.com/interchainio/tendermint-rs/blob/e2cb9aca0b95430fca2eac154edddc9588038982/docs/architecture/adr-002-lite-client.md
[attack-detector]: https://github.com/cometbft/cometbft/blob/main/spec/light-client/detection/detection_001_reviewed.md
[fullnode]: https://github.com/cometbft/cometbft/tree/main/spec/core

[ibc-rs]:https://github.com/informalsystems/ibc-rs


[arXiv]: https://arxiv.org/abs/1807.04938
