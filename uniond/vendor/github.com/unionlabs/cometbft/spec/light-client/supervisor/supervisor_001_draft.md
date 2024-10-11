# Draft of Light Client Supervisor for discussion

## TODOs

This specification in done in parallel with updates on the
verification specification. So some hyperlinks have to be placed to
the correct files eventually.

# Light Client Sequential Supervisor
<!-- markdown-link-check-disable -->
The light client implements a read operation of a
[header](CMBC-HEADER-link) from the [blockchain](CMBC-SEQ-link), by
communicating with full nodes, a so-called primary and several
so-called witnesses. As some full nodes may be faulty, this
functionality must be implemented in a fault-tolerant way.

In a Cosmos blockchain, the validator set may change with every
new block.  The staking and unbonding mechanism induces a [security
model](CMBC-FM-2THIRDS-link): starting at time *Time* of the
[header](CMBC-HEADER-link),
more than two-thirds of the next validators of a new block are correct
for the duration of *TrustedPeriod*.

[Light Client Verification](https://informal.systems) implements the fault-tolerant read
operation designed for this security model. That is, it is safe if the
model assumptions are satisfied and makes progress if it communicates
to a correct primary.

However, if the [security model](CMBC-FM-2THIRDS-link) is violated,
faulty peers (that have been validators at some point in the past) may
launch attacks on the Cosmos network, and on the light
client. These attacks as well as an axiomatization of blocks in
general are defined in [a document that contains the definitions that
are currently in detection.md](https://informal.systems).

If there is a light client attack (but no
successful attack on the network), the safety of the verification step
may be violated (as we operate outside its basic assumption).
The light client also
contains a defense mechanism against light clients attacks, called detection.

[Light Client Detection](https://informal.systems) implements a cross check of the result
of the verification step. If there is a light client attack, and the
light client is connected to a correct peer, the light client as a
whole is safe, that is, it will not operate on invalid
blocks. However, in this case it cannot successfully read, as
inconsistent blocks are in the system. However, in this case the
detection performs a distributed computation that results in so-called
evidence. Evidence can be used to prove
to a correct full node that there has been a
light client attack.

[Light Client Evidence Accountability](https://informal.systems) is a protocol run on a
full node to check whether submitted evidence indeed proves the
existence of a light client attack. Further, from the evidence and its
own knowledge about the blockchain, the full node computes a set of
bonded full nodes (that at some point had more than one third of the
voting power) that participated in the attack that will be reported
via ABCI to the application.

In this document we specify

- Initialization of the Light Client
- The interaction of [verification](https://informal.systems) and [detection](https://informal.systems)

The details of these two protocols are captured in their own
documents, as is the [accountability](https://informal.systems) protocol.

> Another related line is IBC attack detection and submission at the
> relayer, as well as attack verification at the IBC handler. This
> will call for yet another spec.

# Status

This document is work in progress. In order to develop the
specification step-by-step,
it assumes certain details of [verification](https://informal.systems) and
[detection](https://informal.systems) that are not specified in the respective current
versions yet. This inconsistencies will be addresses over several
upcoming PRs.

# Part I - Cosmos Blockchain

See [verification spec](addLinksWhenDone)

# Part II - Sequential Problem Definition

#### **[LC-SEQ-INIT-LIVE.1]**

Upon initialization, the light client gets as input a header of the
blockchain, or the genesis file of the blockchain, and eventually
stores a header of the blockchain.

#### **[LC-SEQ-LIVE.1]**

The light client gets a sequence of heights as inputs. For each input
height *targetHeight*, it eventually stores the header of height
*targetHeight*.

#### **[LC-SEQ-SAFE.1]**

The light client never stores a header which is not in the blockchain.

# Part III - Light Client as Distributed System

## Computational Model

The light client communicates with remote processes only via the
[verification](TODO) and the [detection](TODO) protocols. The
respective assumptions are given there.

## Distributed Problem Statement

### Two Kinds of Liveness

In case of light client attacks, the sequential problem statement
cannot always be satisfied. The lightclient cannot decide which block
is from the chain and which is not. As a result, the light client just
creates evidence, submits it, and terminates.
For the liveness property, we thus add the
possibility that instead of adding a lightblock, we also might terminate
in case there is an attack.

#### **[LC-DIST-TERM.1]**

The light client either runs forever or it *terminates on attack*.

### Design choices

#### [LC-DIST-STORE.1]

The light client has a local data structure called LightStore
that contains light blocks (that contain a header).

> The light store exposes functions to query and update it. They are
> specified [here](TODO:onceVerificationIsMerged).

**TODO:** reference light store invariant [LCV-INV-LS-ROOT.2] once
verification is merged

#### **[LC-DIST-SAFE.1]**

It is always the case that every header in *LightStore* was
generated by an instance of Tendermint consensus.

#### **[LC-DIST-LIVE.1]**

Whenever the light client gets a new height *h* as input,

- and there is
no light client attack up to height *h*, then the lightclient
eventually puts the lightblock of height *h* in the lightstore and
wait for another input.
- otherwise, that is, if there
is a light client attack on height *h*, then the light client
must perform one of the following:
    - it terminates on attack.
    - it eventually puts the lightblock of height *h* in the lightstore and
wait for another input.

> Observe that the "existence of a lightclient attack" just means that some node has generated a conflicting block. It does not necessarily mean that a (faulty) peer sends such a block to "our" lightclient. Thus, even if there is an attack somewhere in the system, our lightclient might still continue to operate normally.

### Solving the sequential specification

[LC-DIST-SAFE.1] is guaranteed by the detector; in particular it
follows from
[[LCD-DIST-INV-STORE.1]](TODO)
[[LCD-DIST-LIVE.1]](TODO)

# Part IV - Light Client Supervisor Protocol

We provide a specification for a sequential Light Client Supervisor.
The local code for verification is presented by a sequential function
`Sequential-Supervisor` to highlight the control flow of this
functionality. Each lightblock is first verified with a primary, and then
cross-checked with secondaries, and if all goes well, the lightblock
is
added (with the attribute "trusted") to the
lightstore. Intermiate lightblocks that were used to verify the target
block but were not cross-checked are stored as "verified"

> We note that if a different concurrency model is considered
> for an implementation, the semantics of the lightstore might change:
> In a concurrent implementation, we might do verification for some
> height *h*, add the
> lightblock to the lightstore, and start concurrent threads that
>
> - do verification for the next height *h' != h*
> - do cross-checking for height *h*. If we find an attack, we remove
>   *h* from the lightstore.
> - the user might already start to use *h*
>
> Thus, this concurrency model changes the semantics of the
> lightstore (not all lightblocks that are read by the user are
> trusted; they may be removed if
> we find a problem). Whether this is desirable, and whether the gain in
> performance is worth it, we keep for future versions/discussion of
> lightclient protocols.

## Definitions

### Peers

#### **[LC-DATA-PEERS.1]:**

A fixed set of full nodes is provided in the configuration upon
initialization. Initially this set is partitioned into

- one full node that is the *primary* (singleton set),
- a set *Secondaries* (of fixed size, e.g., 3),
- a set *FullNodes*; it excludes *primary* and *Secondaries* nodes.
- A set *FaultyNodes* of nodes that the light client suspects of
    being faulty; it is initially empty

#### **[LC-INV-NODES.1]:**

The detector shall maintain the following invariants:

- *FullNodes \intersect Secondaries = {}*
- *FullNodes \intersect FaultyNodes = {}*
- *Secondaries \intersect FaultyNodes = {}*

and the following transition invariant

- *FullNodes' \union Secondaries' \union FaultyNodes' = FullNodes
   \union Secondaries \union FaultyNodes*

#### **[LC-FUNC-REPLACE-PRIMARY.1]:**

```go
Replace_Primary(root-of-trust LightBlock)
```

- Implementation remark
    - the primary is replaced by a secondary
    - to maintain a constant size of secondaries, need to
        - pick a new secondary *nsec* while ensuring [LC-INV-ROOT-AGREED.1]
        - that is, we need to ensure that root-of-trust = FetchLightBlock(nsec, root-of-trust.Header.Height)
- Expected precondition
    - *FullNodes* is nonempty
- Expected postcondition
    - *primary* is moved to *FaultyNodes*
    - a secondary *s* is moved from *Secondaries* to primary
- Error condition
    - if precondition is violated

#### **[LC-FUNC-REPLACE-SECONDARY.1]:**

```go
Replace_Secondary(addr Address, root-of-trust LightBlock)
```

- Implementation remark
    - maintain [LC-INV-ROOT-AGREED.1], that is,
    ensure root-of-trust = FetchLightBlock(nsec, root-of-trust.Header.Height)
- Expected precondition
    - *FullNodes* is nonempty
- Expected postcondition
    - addr is moved from *Secondaries* to *FaultyNodes*
    - an address *nsec* is moved from *FullNodes* to *Secondaries*
- Error condition
    - if precondition is violated

### Data Types

The core data structure of the protocol is the LightBlock.

#### **[LC-DATA-LIGHTBLOCK.1]**

```go
type LightBlock struct {
                Header          Header
                Commit          Commit
                Validators      ValidatorSet
                NextValidators  ValidatorSet
                Provider        PeerID
}
```

#### **[LC-DATA-LIGHTSTORE.1]**

LightBlocks are stored in a structure which stores all LightBlock from
initialization or received from peers.

```go
type LightStore struct {
        ...
}

```

We use the functions that the LightStore exposes, which
are defined in the [verification specification](TODO).

### Inputs

The lightclient is initialized with LCInitData

#### **[LC-DATA-INIT.1]**

```go
type LCInitData struct {
    lightBlock     LightBlock
    genesisDoc     GenesisDoc
}
```

where only one of the components must be provided. `GenesisDoc` is
defined in the [CometBFT
Types](https://github.com/cometbft/cometbft/blob/main/types/genesis.go).

#### **[LC-DATA-GENESIS.1]**

```go
type GenesisDoc struct {
    GenesisTime     time.Time                `json:"genesis_time"`
    ChainID         string                   `json:"chain_id"`
    InitialHeight   int64                    `json:"initial_height"`
    ConsensusParams *tmproto.ConsensusParams `json:"consensus_params,omitempty"`
    Validators      []GenesisValidator       `json:"validators,omitempty"`
    AppHash         tmbytes.HexBytes         `json:"app_hash"`
    AppState        json.RawMessage          `json:"app_state,omitempty"`
}
```

We use the following function
`makeblock` so that we create a lightblock from the genesis
file in order to do verification based on the data from the genesis
file using the same verification function we use in normal operation.

#### **[LC-FUNC-MAKEBLOCK.1]**

```go
func makeblock (genesisDoc GenesisDoc) (lightBlock LightBlock))
```

- Implementation remark
    - none
- Expected precondition
    - none
- Expected postcondition
    - lightBlock.Header.Height =  genesisDoc.InitialHeight
    - lightBlock.Header.Time = genesisDoc.GenesisTime
    - lightBlock.Header.LastBlockID = nil
    - lightBlock.Header.LastCommit = nil
    - lightBlock.Header.Validators = genesisDoc.Validators
    - lightBlock.Header.NextValidators = genesisDoc.Validators
    - lightBlock.Header.Data = nil
    - lightBlock.Header.AppState =  genesisDoc.AppState
    - lightBlock.Header.LastResult = nil
    - lightBlock.Commit = nil
    - lightBlock.Validators = genesisDoc.Validators
    - lightBlock.NextValidators = genesisDoc.Validators
    - lightBlock.Provider = nil
- Error condition
    - none

----

### Configuration Parameters

#### **[LC-INV-ROOT-AGREED.1]**

In the Sequential-Supervisor, it is always the case that the primary
and all secondaries agree on lightStore.Latest().

### Assumptions

We have to assume that the initialization data (the lightblock or the
genesis file) are consistent with the blockchain. This is subjective
initialization and it cannot be checked locally.

### Invariants

#### **[LC-INV-PEERLIST.1]:**

The peer list contains a primary and a secondary.

> If the invariant is violated, the light client does not have enough
> peers to download headers from. As a result, the light client
> needs to terminate in case this invariant is violated.

## Supervisor

### Outline

The supervisor implements the functionality of the lightclient. It is
initialized with a genesis file or with a lightblock the user
trusts. This initialization is subjective, that is, the security of
the lightclient is based on the validity of the input. If the genesis
file or the lightblock deviate from the actual ones on the blockchain,
the lightclient provides no guarantees.

After initialization, the supervisor awaits an input, that is, the
height of the next lightblock that should be obtained. Then it
downloads, verifies, and cross-checks a lightblock, and if all tests
go through, the light block (and possibly other lightblocks) are added
to the lightstore, which is returned in an output event to the user.

The following main loop does the interaction with the user (input,
output) and calls the following two functions:

- `InitLightClient`: it initializes the lightstore either with the
  provided lightblock or with the lightblock that corresponds to the
  first block generated by the blockchain (by the validators defined
  by the genesis file)
- `VerifyAndDetect`: takes as input a lightstore and a height and
  returns the updated lightstore.

#### **[LC-FUNC-SUPERVISOR.1]:**

```go
func Sequential-Supervisor (initdata LCInitData) (Error) {

    lightStore,result := InitLightClient(initData);
    if result != OK {
        return result;
    }

    loop {
        // get the next height
        nextHeight := input();
  
        lightStore,result := VerifyAndDetect(lightStore, nextHeight);
  
        if result == OK {
            output(LightStore.Get(targetHeight));
   // we only output a trusted lightblock
        }
        else {
            return result
        }
        // QUESTION: is it OK to generate output event in normal case,
        // and terminate with failure in the (light client) attack case?
    }
}
```

- Implementation remark
    - infinite loop unless a light client attack is detected
    - In typical implementations (e.g., the one in Rust),
   there are multiple input actions:
      `VerifytoLatest`, `LatestTrusted`, and `GetStatus`. The
      information can be easily obtained from the lightstore, so that
      we do not treat these requests explicitly here but just consider
   the request for a block of a given height which requires more
   involved computation and communication.
- Expected precondition
    - *LCInitData* contains a genesis file or a lightblock.
- Expected postcondition
    - if a light client attack is detected: it stops and submits
      evidence (in `InitLightClient` or `VerifyAndDetect`)
    - otherwise: non. It runs forever.
- Invariant: *lightStore* contains trusted lightblocks only.
- Error condition
    - if `InitLightClient` or `VerifyAndDetect` fails (if a attack is
 detected, or if [LCV-INV-TP.1] is violated)

----

### Details of the Functions

#### Initialization

The light client is based on subjective initialization. It has to
trust the initial data given to it by the user. It cannot do any
detection of attack. So either upon initialization we obtain a
lightblock and just initialize the lightstore with it. Or in case of a
genesis file, we download, verify, and cross-check the first block, to
initialize the lightstore with this first block. The reason is that
we want to maintain [LCV-INV-TP.1] from the beginning.

> If the lightclient is initialized with a lightblock, one might think
> it may increase trust, when one cross-checks the initial light
> block. However, if a peer provides a conflicting
> lightblock, the question is to distinguish the case of a
> [bogus](https://informal.systems) block (upon which operation should proceed) from a
> [light client attack](https://informal.systems) (upon which operation should stop). In
> case of a bogus block, the lightclient might be forced to do
> backwards verification until the blocks are out of the trusting
> period, to make sure no previous validator set could have generated
> the bogus block, which effectively opens up a DoS attack on the lightclient
> without adding effective robustness.

#### **[LC-FUNC-INIT.1]:**

```go
func InitLightClient (initData LCInitData) (LightStore, Error) {

    if LCInitData.LightBlock != nil {
        // we trust the provided initial block.
        newblock := LCInitData.LightBlock
    }
    else {
        genesisBlock := makeblock(initData.genesisDoc);

        result := NoResult;
        while result != ResultSuccess {
            current = FetchLightBlock(PeerList.primary(), genesisBlock.Header.Height + 1)
            // QUESTION: is the height with "+1" OK?

            if CANNOT_VERIFY = ValidAndVerify(genesisBlock, current) {
                Replace_Primary();
            }
            else {
                result = ResultSuccess
            }
        }
  
        // cross-check
  auxLS := new LightStore
  auxLS.Add(current)
        Evidences := AttackDetector(genesisBlock, auxLS)
        if Evidences.Empty {
            newBlock := current
        }
        else {
            // [LC-SUMBIT-EVIDENCE.1]
            submitEvidence(Evidences);
            return(nil, ErrorAttack);
        }
    }

    lightStore := new LightStore;
    lightStore.Add(newBlock);
    return (lightStore, OK);
}

```

- Implementation remark
    - none
- Expected precondition
    - *LCInitData* contains either a genesis file of a lightblock
    - if genesis it passes `ValidateAndComplete()` see [CometBFT](https://informal.systems)
- Expected postcondition
    - *lightStore* initialized with trusted lightblock. It has either been
      cross-checked (from genesis) or it has initial trust from the
      user.
- Error condition
    - if precondition is violated
    - empty peerList

----

#### Main verification and detection logic

#### **[LC-FUNC-MAIN-VERIF-DETECT.1]:**

```go
func VerifyAndDetect (lightStore LightStore, targetHeight Height)
                     (LightStore, Result) {

    b1, r1 = lightStore.Get(targetHeight)
    if r1 == true {
        if b1.State == StateTrusted {
            // block already there and trusted
            return (lightStore, ResultSuccess)
  }
  else {
            // We have a lightblock in the store, but it has not been 
            // cross-checked by now. We do that now.
            root_of_trust, auxLS := lightstore.TraceTo(b1);
   
            // Cross-check
            Evidences := AttackDetector(root_of_trust, auxLS);
            if Evidences.Empty {
                // no attack detected, we trust the new lightblock
                lightStore.Update(auxLS.Latest(), 
                                  StateTrusted, 
                                  verfiedLS.Latest().verification-root);
                return (lightStore, OK);
            }
            else {
                // there is an attack, we exit
  submitEvidence(Evidences);
                return(lightStore, ErrorAttack);
            }
        }
    }

    // get the lightblock with maximum height smaller than targetHeight
    // would typically be the heights, if we always move forward
    root_of_trust, r2 = lightStore.LatestPrevious(targetHeight);

    if r2 = false {
        // there is no lightblock from which we can do forward
        // (skipping) verification. Thus we have to go backwards.
        // No cross-check needed. We trust hashes. Therefore, we
        // directly return the result
        return Backwards(primary, lightStore.Lowest(), targetHeight)
    }
    else {
        // Forward verification + detection
        result := NoResult;
        while result != ResultSuccess {
            verifiedLS,result := VerifyToTarget(primary,
                                                root_of_trust,
                                                nextHeight);
            if result == ResultFailure {
                // pick new primary (promote a secondary to primary)
                Replace_Primary(root_of_trust);
            }
            else if result == ResultExpired {
                return (lightStore, result)
            }
        }

        // Cross-check
        Evidences := AttackDetector(root_of_trust, verifiedLS);
        if Evidences.Empty {
            // no attack detected, we trust the new lightblock
            verifiedLS.Update(verfiedLS.Latest(), 
                              StateTrusted, 
                              verfiedLS.Latest().verification-root);
            lightStore.store_chain(verifidLS);
            return (lightStore, OK);
        }
        else {
            // there is an attack, we exit
            return(lightStore, ErrorAttack);
        }
    }
}
```

- Implementation remark
    - none
- Expected precondition
    - none
- Expected postcondition
    - lightblock of height *targetHeight* (and possibly additional blocks) added to *lightStore*
- Error condition
    - an attack is detected
    - [LC-DATA-PEERLIST-INV.1] is violated

----
