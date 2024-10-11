---
---

# Evidence

Evidence is an important component of CometBFT's security model. Whilst the core
consensus protocol provides correctness guarantees for state machine replication
that can tolerate less than 1/3 failures, the evidence system looks to detect and
gossip byzantine faults whose combined power is greater than  or equal to 1/3. It is worth noting that
the evidence system is designed purely to detect possible attacks, gossip them,
commit them on chain and inform the application running on top of CometBFT.
Evidence in itself does not punish "bad actors", this is left to the discretion
of the application. A common form of punishment is slashing where the validators
that were caught violating the protocol have all or a portion of their voting
power removed. Evidence, given the assumption that 1/3+ of the network is still
byzantine, is susceptible to censorship and should therefore be considered added
security on a "best effort" basis.

This document walks through the various forms of evidence, how they are detected,
gossiped, verified and committed.

> NOTE: Evidence here is internal to CometBFT and should not be confused with
> application evidence

## Detection

### Equivocation

Equivocation is the most fundamental of byzantine faults. Simply put, to prevent
replication of state across all nodes, a validator tries to convince some subset
of nodes to commit one block whilst convincing another subset to commit a
different block. This is achieved by double voting (hence
`DuplicateVoteEvidence`). A successful duplicate vote attack requires greater
than 1/3 voting power and a (temporary) network partition between the aforementioned
subsets. This is because in consensus, votes are gossiped around. When a node
observes two conflicting votes from the same peer, it will use the two votes of
evidence and begin gossiping this evidence to other nodes. [Verification](#duplicatevoteevidence) is addressed further down.

```go
type DuplicateVoteEvidence struct {
    VoteA *Vote
    VoteB *Vote

    // and abci specific fields
}
```

### Light Client Attacks

Light clients also comply with the 1/3+ security model, however, by using a
different, more lightweight verification method they are subject to a
different kind of 1/3+ attack whereby the byzantine validators could sign an
alternative light block that the light client will think is valid. Detection,
explained in greater detail
[here](../light-client/detection/detection_003_reviewed.md), involves comparison
with multiple other nodes in the hope that at least one is "honest". An "honest"
node will return a challenging light block for the light client to validate. If
this challenging light block also meets the
[validation criteria](../light-client/verification/verification_001_published.md)
then the light client sends the "forged" light block to the node.
[Verification](#lightclientattackevidence) is addressed further down.

```go
type LightClientAttackEvidence struct {
    ConflictingBlock *LightBlock
    CommonHeight int64

      // and abci specific fields
}
```

## Verification

If a node receives evidence, it will first try to verify it, then persist it.
Evidence of byzantine behavior should only be committed once (uniqueness) and
should be committed within a certain period from the point that it occurred
(timely). Timelines is defined by the `EvidenceParams`: `MaxAgeNumBlocks` and
`MaxAgeDuration`. In Proof of Stake chains where validators are bonded, evidence
age should be less than the unbonding period so validators still can be
punished. Given these two properties the following initial checks are made.

1. Has the evidence expired? This is done by taking the height of the `Vote`
   within `DuplicateVoteEvidence` or `CommonHeight` within
   `LightClientAttakEvidence`. The evidence height is then used to retrieve the
   header and thus the time of the block that corresponds to the evidence. If
   `CurrentHeight - MaxAgeNumBlocks > EvidenceHeight` && `CurrentTime -
   MaxAgeDuration > EvidenceTime`, the evidence is considered expired and
   ignored.

2. Has the evidence already been committed? The evidence pool tracks the hash of
   all committed evidence and uses this to determine uniqueness. If a new
   evidence has the same hash as a committed one, the new evidence will be
   ignored.

### DuplicateVoteEvidence

Valid `DuplicateVoteEvidence` must adhere to the following rules:

- Validator Address, Height, Round and Type must be the same for both votes

- BlockID must be different for both votes (BlockID can be for a nil block)

- Validator must have been in the validator set at that height

- Vote signature must be correctly signed. This also uses `ChainID` so we know
  that the fault occurred on this chain

### LightClientAttackEvidence

Valid Light Client Attack Evidence must adhere to the following rules:

- If the header of the light block is invalid, thus indicating a lunatic attack,
  the node must check that they can use `verifySkipping` from their header at
  the common height to the conflicting header

- If the header is valid, then the validator sets are the same and this is
  either a form of equivocation or amnesia. We therefore check that 2/3 of the
  validator set also signed the conflicting header.

- The nodes own header at the same height as the conflicting header must have a
  different hash to the conflicting header.

- If the nodes latest header is less in height to the conflicting header, then
  the node must check that the conflicting block has a time that is less than
  this latest header (This is a forward lunatic attack).

## Gossiping

If a node verifies evidence it then broadcasts it to all peers, continuously sending
the same evidence once every 10 seconds until the evidence is seen on chain or
expires.

## Committing on Chain

Evidence takes strict priority over regular transactions, thus a block is filled
with evidence first and transactions take up the remainder of the space. To
mitigate the threat of an already punished node from spamming the network with
more evidence, the size of the evidence in a block can be capped by
`EvidenceParams.MaxBytes`. Nodes receiving blocks with evidence will validate
the evidence before sending `Prevote` and `Precommit` votes. The evidence pool
will usually cache verifications so that this process is much quicker.

## Sending Evidence to the Application

After evidence is committed, the block is then processed by the block executor
which delivers the list of misbehavior (`[]abci.Misbehavior`) to the application via the `FinalizeBlock`.

```proto
// The type of misbehavior committed by a validator.
enum MisbehaviorType {
  option (gogoproto.goproto_enum_prefix) = false;

  // Unknown
  MISBEHAVIOR_TYPE_UNKNOWN = 0;
  // Duplicate vote
  MISBEHAVIOR_TYPE_DUPLICATE_VOTE = 1;
  // Light client attack
  MISBEHAVIOR_TYPE_LIGHT_CLIENT_ATTACK = 2;
}

// Misbehavior is a type of misbehavior committed by a validator.
message Misbehavior {
  MisbehaviorType type = 1;
  // The offending validator
  Validator validator = 2 [(gogoproto.nullable) = false];
  // The height when the offense occurred
  int64 height = 3;
  // The corresponding time where the offense occurred
  google.protobuf.Timestamp time = 4 [
    (gogoproto.nullable) = false,
    (gogoproto.stdtime)  = true
  ];
  // Total voting power of the validator set in case the ABCI application does
  // not store historical validators.
  // https://github.com/tendermint/tendermint/issues/4581
  int64 total_voting_power = 5;
}
```

`DuplicateVoteEvidence` and `LightClientAttackEvidence` are can be used to derive the list of `abci.Misbehavior` for
each byzantine validator that is sent to the application in the `FinalizeBlockRequest`.

Because of this, extra fields are necessary:

```go
type DuplicateVoteEvidence struct {
  VoteA *Vote
  VoteB *Vote

  // abci specific information
  TotalVotingPower int64
  ValidatorPower   int64
  Timestamp        time.Time
}

type LightClientAttackEvidence struct {
  ConflictingBlock *LightBlock
  CommonHeight     int64

  // abci specific information
  ByzantineValidators []*Validator
  TotalVotingPower    int64
  Timestamp           time.Time
}
```

These ABCI specific fields don't affect validity of the evidence itself but must
be consistent amongst nodes and agreed upon on chain. If evidence with the
incorrect abci information is sent, a node will create new evidence from it and
replace the ABCI fields with the correct information.
