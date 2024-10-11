---
order: 1
---

# Data Structures

Here we describe the data structures in the CometBFT blockchain and the rules for validating them.

The CometBFT blockchain consists of a short list of data types:

- [Data Structures](#data-structures)
  - [Block](#block)
  - [Execution](#execution)
  - [Header](#header)
  - [Version](#version)
  - [BlockID](#blockid)
  - [PartSetHeader](#partsetheader)
  - [Part](#part)
  - [Time](#time)
  - [Data](#data)
  - [Commit](#commit)
  - [ExtendedCommit](#extendedcommit)
  - [CommitSig](#commitsig)
  - [ExtendedCommitSig](#extendedcommitsig)
  - [BlockIDFlag](#blockidflag)
  - [Vote](#vote)
  - [CanonicalVote](#canonicalvote)
    - [CanonicalVoteExtension](#canonicalvoteextension)
  - [Proposal](#proposal)
  - [SignedMsgType](#signedmsgtype)
  - [Signature](#signature)
  - [EvidenceList](#evidencelist)
  - [Evidence](#evidence)
    - [DuplicateVoteEvidence](#duplicatevoteevidence)
    - [LightClientAttackEvidence](#lightclientattackevidence)
  - [LightBlock](#lightblock)
  - [SignedHeader](#signedheader)
  - [ValidatorSet](#validatorset)
  - [Validator](#validator)
  - [Address](#address)
  - [Proof](#proof)
  - [ConsensusParams](#consensusparams)
    - [BlockParams](#blockparams)
    - [EvidenceParams](#evidenceparams)
    - [ValidatorParams](#validatorparams)
    - [VersionParams](#versionparams)
    - [ABCIParams](#abciparams)
    - [FeatureParams](#featureparams)
    - [SynchronyParams](#synchronyparams)


## Block

A block consists of a header, transactions, votes (the commit),
and a list of evidence of misbehavior (ie. signing conflicting votes).

| Name   | Type              | Description                                                                                                                                                                                                                                                                                                                                                                                                                                           | Validation                                               |
|--------|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------|
| Header | [Header](#header) | Header corresponding to the block. This field contains information used throughout consensus and other areas of the protocol. To find out what it contains, visit [header](#header)                                                                                                                                                                                                                                                                   | Must adhere to the validation rules of [header](#header) |
| Data       | [Data](#data)                  | Data contains a list of transactions. The contents of the transaction is unknown to CometBFT.                                                                                                                                                                                                                                                                                                                                                         | This field can be empty or populated, but no validation is performed. Applications can perform validation on individual transactions prior to block creation using [checkTx](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci%2B%2B_methods.md#checktx).
| Evidence   | [EvidenceList](#evidencelist) | Evidence contains a list of evidence of misbehavior committed by validators.                                                                                                                                                                                                                                                                                                                                                                           | Can be empty, but when populated the validations rules from [evidenceList](#evidencelist) apply |
| LastCommit | [Commit](#commit)              | `LastCommit` includes one vote for every validator.  All votes must either be for the previous block, nil or absent. If a vote is for the previous block it must have a valid signature from the corresponding validator. The sum of the voting power of the validators that voted must be greater than 2/3 of the total voting power of the complete validator set. The number of votes in a commit is limited to 10000 (see `types.MaxVotesCount`). | Must be empty for the initial height and must adhere to the validation rules of [commit](#commit).  |

## Execution

Once a block is validated, it can be executed against the state.

The state follows this recursive equation:

```go
state(initialHeight) = InitialState
state(h+1) <- Execute(state(h), ABCIApp, block(h))
```

where `InitialState` includes the initial consensus parameters and validator set,
and `ABCIApp` is an ABCI application that can return results and changes to the validator
set (TODO). Execute is defined as:

```go
func Execute(state State, app ABCIApp, block Block) State {
 // Function ApplyBlock executes block of transactions against the app and returns the new root hash of the app state,
 // modifications to the validator set and the changes of the consensus parameters.
 AppHash, ValidatorChanges, ConsensusParamChanges := app.ApplyBlock(block)

 nextConsensusParams := UpdateConsensusParams(state.ConsensusParams, ConsensusParamChanges)
 return State{
  ChainID:         state.ChainID,
  InitialHeight:   state.InitialHeight,
  LastResults:     abciResponses.DeliverTxResults,
  AppHash:         AppHash,
  LastValidators:  state.Validators,
  Validators:      state.NextValidators,
  NextValidators:  UpdateValidators(state.NextValidators, ValidatorChanges),
  ConsensusParams: nextConsensusParams,
  Version: {
   Consensus: {
    AppVersion: nextConsensusParams.Version.AppVersion,
   },
  },
 }
}
```

Validating a new block is first done prior to the `prevote`, `precommit` & `finalizeCommit` stages.

The steps to validate a new block are:

- Check the validity rules of the block and its fields.
- Check the versions (Block & App) are the same as in local state.
- Check the chainID's match.
- Check the height is correct.
- Check the `LastBlockID` corresponds to BlockID currently in state.
- Check the hashes in the header match those in state.
- Verify the LastCommit against state, this step is skipped for the initial height.
    - This is where checking the signatures correspond to the correct block will be made.
- Make sure the proposer is part of the validator set.
- Validate bock time.
    - Make sure the new blocks time is after the previous blocks time.
    - Calculate the medianTime and check it against the blocks time.
    - If the blocks height is the initial height then check if it matches the genesis time.
- Validate the evidence in the block. Note: Evidence can be empty

## Header

A block header contains metadata about the block and about the consensus, as well as commitments to
the data in the current block, the previous block, and the results returned by the application:

| Name              | Type                      | Description                                                                                                                                                                                                                                                                                                                                                                            | Validation                                                                                                                                                                                                                                                 |
|-------------------|---------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Version           | [Version](#version)       | Version defines the application and block versions being used.                                                                                                                                                                                                                                                                                                                       | Must adhere to the validation rules of [Version](#version)                                                                                                                                                                                                 |
| ChainID           | String                    | ChainID is the ID of the chain. This must be unique to your chain.                                                                                                                                                                                                                                                                                                                     | ChainID must be less than 50 bytes.                                                                                                                                                                                                                        |
| Height            | uint64                    | Height is the height for this header.                                                                                                                                                                                                                                                                                                                                                  | Must be > 0, >= initialHeight, and == previous Height+1                                                                                                                                                                                                    |
| Time              | [Time](#time)             | The timestamp can be computed using [PBTS][pbts] or [BFT Time][bfttime] algorithms. In case of PBTS, it is the time at which the proposer has produced the block (the value of its local clock). In case of BFT Time, it is equal to the weighted median of timestamps present in the previous commit.                                                                                 | Time must be larger than the Time of the previous block header. The timestamp of the first block should not be smaller than the genesis time. When BFT Time is used, it should match the genesis time (since there's no votes to compute the median with). |
| LastBlockID       | [BlockID](#blockid)       | BlockID of the previous block.                                                                                                                                                                                                                                                                                                                                                         | Must adhere to the validation rules of [blockID](#blockid). The first block has `block.Header.LastBlockID == BlockID{}`.                                                                                                                                   |
| LastCommitHash    | slice of bytes (`[]byte`) | MerkleRoot of the lastCommit's signatures. The signatures represent the validators that committed to the last block. The first block has an empty slices of bytes for the hash.                                                                                                                                                                                                        | Must  be of length 32                                                                                                                                                                                                                                      |
| DataHash          | slice of bytes (`[]byte`) | MerkleRoot of the hash of transactions. **Note**: The transactions are hashed before being included in the merkle tree, the leaves of the Merkle tree are the hashes, not the transactions themselves.                                                                                                                                                                                 | Must  be of length 32                                                                                                                                                                                                                                      |
| ValidatorHash     | slice of bytes (`[]byte`) | MerkleRoot of the current validator set. The validators are first sorted by voting power (descending), then by address (ascending) prior to computing the MerkleRoot.                                                                                                                                                                                                                  | Must  be of length 32                                                                                                                                                                                                                                      |
| NextValidatorHash | slice of bytes (`[]byte`) | MerkleRoot of the next validator set. The validators are first sorted by voting power (descending), then by address (ascending) prior to computing the MerkleRoot.                                                                                                                                                                                                                     | Must  be of length 32                                                                                                                                                                                                                                      |
| ConsensusHash     | slice of bytes (`[]byte`) | Hash of the protobuf encoded consensus parameters.                                                                                                                                                                                                                                                                                                                                     | Must  be of length 32                                                                                                                                                                                                                                      |
| AppHash           | slice of bytes (`[]byte`) | Arbitrary byte array returned by the application after executing and committing the previous block. It serves as the basis for validating any merkle proofs that comes from the ABCI application and represents the state of the actual application rather than the state of the blockchain itself. The first block's `block.Header.AppHash` is given by `InitChainResponse.app_hash`. | This hash is determined by the application, CometBFT can not perform validation on it.                                                                                                                                                                     |
| LastResultHash    | slice of bytes (`[]byte`) | `LastResultsHash` is the root hash of a Merkle tree built from `DeliverTxResponse` responses (`Log`,`Info`, `Codespace` and `Events` fields are ignored).                                                                                                                                                                                                                              | Must  be of length 32. The first block has `block.Header.ResultsHash == MerkleRoot(nil)`, i.e. the hash of an empty input, for RFC-6962 conformance.                                                                                                       |
| EvidenceHash      | slice of bytes (`[]byte`) | MerkleRoot of the evidence of Byzantine behavior included in this block.                                                                                                                                                                                                                                                                                                               | Must  be of length 32                                                                                                                                                                                                                                      |
| ProposerAddress   | slice of bytes (`[]byte`) | Address of the original proposer of the block. Validator must be in the current validatorSet.                                                                                                                                                                                                                                                                                          | Must  be of length 20                                                                                                                                                                                                                                      |

## Version

NOTE: that this is more specifically the consensus version and doesn't include information like the
P2P Version. (TODO: we should write a comprehensive document about
versioning that this can refer to)

| Name  | type   | Description                                                                                                                                    | Validation                                                                                                      |
|-------|--------|------------------------------------------------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------|
| Block | uint64 | This number represents the block version and must be the same throughout an operational network                                                | Must be equal to block version being used in a network (`block.Version.Block == state.Version.Consensus.Block`) |
| App   | uint64 | App version is decided on by the application. Read [here](https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_app_requirements.md) | `block.Version.App == state.Version.Consensus.App`                                                              |

## BlockID

The `BlockID` contains two distinct Merkle roots of the block. The `BlockID` includes these two hashes, as well as the number of parts (ie. `len(MakeParts(block))`)

| Name          | Type                            | Description                                                                                                                                                      | Validation                                                             |
|---------------|---------------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------|
| Hash          | slice of bytes (`[]byte`)       | MerkleRoot of all the fields in the header (ie. `MerkleRoot(header)`.                                                                                            | hash must be of length 32                                              |
| PartSetHeader | [PartSetHeader](#partsetheader) | Used for secure gossiping of the block during consensus, is the MerkleRoot of the complete serialized block cut into parts (ie. `MerkleRoot(MakeParts(block))`). | Must adhere to the validation rules of [PartSetHeader](#partsetheader) |

See [MerkleRoot](./encoding.md#MerkleRoot) for details.

## PartSetHeader

| Name  | Type                      | Description                       | Validation           |
|-------|---------------------------|-----------------------------------|----------------------|
| Total | int32                     | Total amount of parts for a block | Must be > 0          |
| Hash  | slice of bytes (`[]byte`) | MerkleRoot of a serialized block  | Must be of length 32 |

## Part

Part defines a part of a block. In CometBFT blocks are broken into `parts` for gossip.

| Name  | Type            | Description                       | Validation           |
|-------|-----------------|-----------------------------------|----------------------|
| index | int32           | Total amount of parts for a block | Must be >= 0         |
| bytes | bytes           | MerkleRoot of a serialized block  | Must be of length 32 |
| proof | [Proof](#proof) | MerkleRoot of a serialized block  | Must be of length 32 |

## Time

CometBFT uses the [Google.Protobuf.Timestamp](https://protobuf.dev/reference/protobuf/google.protobuf/#timestamp)
format, which uses two integers, one 64 bit integer for Seconds and a 32 bit integer for Nanoseconds.
Time is aligned with the Coordinated Universal Time (UTC).

## Data

Data is just a wrapper for a list of transactions, where transactions are arbitrary byte arrays:

| Name | Type                       | Description            | Validation                                                                  |
|------|----------------------------|------------------------|-----------------------------------------------------------------------------|
| Txs  | Matrix of bytes ([][]byte) | Slice of transactions. | Validation does not occur on this field, this data is unknown to CometBFT |

## Commit

Commit is a simple wrapper for a list of signatures, with one for each validator. It also contains the relevant BlockID, height and round:

| Name       | Type                             | Description                                                          | Validation                                                                                                                         |
|------------|----------------------------------|----------------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------------|
| Height     | int64                            | Height at which this commit was created.                             | Must be >= 0.                                                                                                                      |
| Round      | int32                            | Round that the commit corresponds to.                                | Must be >= 0.                                                                                                                      |
| BlockID    | [BlockID](#blockid)              | The blockID of the corresponding block.                              | If Height > 0, then it cannot be the [BlockID](#blockid) of a nil block.                                                           |
| Signatures | Array of [CommitSig](#commitsig) | Array of commit signatures that correspond to current validator set. | If Height > 0, then the length of signatures must be > 0 and adhere to the validation of each individual [Commitsig](#commitsig).  |



## ExtendedCommit

`ExtendedCommit`, similarly to Commit, wraps a list of votes with signatures together with other data needed to verify them.
In addition, it contains the verified vote extensions, one for each non-`nil` vote, along with the extension signatures.

| Name               | Type                                     | Description                                                                         | Validation                                                                                                               |
|--------------------|------------------------------------------|-------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------|
| Height             | int64                                    | Height at which this commit was created.                                            | Must be >= 0                                                                                                             |
| Round              | int32                                    | Round that the commit corresponds to.                                               | Must be >= 0                                                                                                             |
| BlockID            | [BlockID](#blockid)                      | The blockID of the corresponding block.                                             | Must adhere to the validation rules of [BlockID](#blockid).                                                              |
| ExtendedSignatures | Array of [ExtendedCommitSig](#commitsig) | The current validator set's commit signatures, extension, and extension signatures. | Length of signatures must be > 0 and adhere to the validation of each individual [ExtendedCommitSig](#extendedcommitsig) |

## CommitSig

`CommitSig` represents a signature of a validator, who has voted either for nil,
a particular `BlockID` or was absent. It's a part of the `Commit` and can be used
to reconstruct the vote set given the validator set.

| Name             | Type                        | Description                                                                                                                                       | Validation                                                        |
|------------------|-----------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------|
| BlockIDFlag      | [BlockIDFlag](#blockidflag) | Represents the validators participation in consensus: its vote was not received, voted for the block that received the majority, or voted for nil | Must be one of the fields in the [BlockIDFlag](#blockidflag) enum |
| ValidatorAddress | [Address](#address)         | Address of the validator                                                                                                                          | Must be of length 20                                              |
| Timestamp        | [Time](#time)               | This field will vary from `CommitSig` to `CommitSig`. It represents the timestamp of the validator.                                               | [Time](#time)                                                     |
| Signature        | [Signature](#signature)     | Signature corresponding to the validators participation in consensus.                                                                             | The length of the signature must be > 0 and < than  64            |

NOTE: `ValidatorAddress` and `Timestamp` fields may be removed in the future
(see [ADR-25](https://github.com/cometbft/cometbft/blob/main/docs/architecture/adr-025-commit.md)).

## ExtendedCommitSig

`ExtendedCommitSig` represents a signature of a validator that has voted either for `nil`,
a particular `BlockID` or was absent. It is part of the `ExtendedCommit` and can be used
to reconstruct the vote set given the validator set.
Additionally it contains the vote extensions that were attached to each non-`nil` precommit vote.
All these extensions have been verified by the application operating at the signing validator's node.

| Name               | Type                        | Description                                                                                                                                       | Validation                                                          |
|--------------------|-----------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------|
| BlockIDFlag        | [BlockIDFlag](#blockidflag) | Represents the validators participation in consensus: its vote was not received, voted for the block that received the majority, or voted for nil | Must be one of the fields in the [BlockIDFlag](#blockidflag) enum   |
| ValidatorAddress   | [Address](#address)         | Address of the validator                                                                                                                          | Must be of length 20                                                |
| Timestamp          | [Time](#time)               | This field will vary from `CommitSig` to `CommitSig`. It represents the timestamp of the validator.                                               |                                                                     |
| Signature          | [Signature](#signature)     | Signature corresponding to the validators participation in consensus.                                                                             | Length must be > 0 and < 64                                         |
| Extension          | bytes                       | Vote extension provided by the Application running on the sender of the precommit vote, and verified by the local application.                    | Length must be zero if BlockIDFlag is not `Commit`                  |
| ExtensionSignature | [Signature](#signature)     | Signature of the vote extension.                                                                                                                  | Length must be > 0 and < than 64 if BlockIDFlag is `Commit`, else 0 |

## BlockIDFlag

BlockIDFlag represents which BlockID the [signature](#commitsig) is for.

```go
enum BlockIDFlag {
  BLOCK_ID_FLAG_UNKNOWN = 0; // indicates an error condition
  BLOCK_ID_FLAG_ABSENT  = 1; // the vote was not received
  BLOCK_ID_FLAG_COMMIT  = 2; // voted for the block that received the majority
  BLOCK_ID_FLAG_NIL     = 3; // voted for nil
}
```

## Vote

A vote is a signed message from a validator for a particular block.
The vote includes information about the validator signing it. When stored in the blockchain or propagated over the network, votes are encoded in Protobuf.

| Name               | Type                            | Description                                                                              | Validation                               |
|--------------------|---------------------------------|------------------------------------------------------------------------------------------|------------------------------------------|
| Type               | [SignedMsgType](#signedmsgtype) | The type of message the vote refers to                                                   | Must be `PrevoteType` or `PrecommitType` |
| Height             | int64                           | Height for which this vote was created for                                               | Must be > 0                             |
| Round              | int32                           | Round that the commit corresponds to.                                                    | Must be >= 0                             |
| BlockID            | [BlockID](#blockid)             | The blockID of the corresponding block.                                                  |                                          |
| Timestamp          | [Time](#time)                   | Timestamp represents the time at which a validator signed.                               |                                          |
| ValidatorAddress   | bytes                           | Address of the validator                                                                 | Length must be equal to 20               |
| ValidatorIndex     | int32                           | Index at a specific block height corresponding to the Index of the validator in the set. | Must be > 0                              |
| Signature          | bytes                           | Signature by the validator if they participated in consensus for the associated block.   | Length must be > 0 and < 64              |
| Extension          | bytes                           | Vote extension provided by the Application running at the validator's node.              | Length can be 0                          |
| ExtensionSignature | bytes                           | Signature for the extension                                                              | Length must be > 0 and < 64              |

## CanonicalVote

CanonicalVote is for validator signing. This type will not be present in a block.
Votes are represented via `CanonicalVote` and also encoded using protobuf via `type.SignBytes` which includes the `ChainID`,
and uses a different ordering of the fields.

| Name      | Type                            | Description                             | Validation                               |
|-----------|---------------------------------|-----------------------------------------|------------------------------------------|
| Type      | [SignedMsgType](#signedmsgtype) | The type of message the vote refers to  | Must be `PrevoteType` or `PrecommitType` |
| Height    | int64                           | Height in which the vote was provided.  | Must be > 0                             |
| Round     | int64                           | Round in which the vote was provided.   | Must be >= 0                             |
| BlockID   | string                          | ID of the block the vote refers to.     |                                          |
| Timestamp | string                          | Time of the vote.                       |                                          |
| ChainID   | string                          | ID of the blockchain running consensus. |                                          |

For signing, votes are represented via [`CanonicalVote`](#canonicalvote) and also encoded using protobuf via
`type.SignBytes` which includes the `ChainID`, and uses a different ordering of
the fields.

We define a method `Verify` that returns `true` if the signature verifies against the pubkey for the `SignBytes`
using the given ChainID:

```go
func (vote *Vote) Verify(chainID string, pubKey crypto.PubKey) error {
 if !bytes.Equal(pubKey.Address(), vote.ValidatorAddress) {
  return ErrVoteInvalidValidatorAddress
 }
 v := vote.ToProto()
 if !pubKey.VerifyBytes(types.VoteSignBytes(chainID, v), vote.Signature) {
  return ErrVoteInvalidSignature
 }
 return nil
}
```

### CanonicalVoteExtension

Vote extensions are signed using a representation similar to votes.
This is the structure to marshall in order to obtain the bytes to sign or verify the signature.

| Name      | Type   | Description                                 | Validation           |
|-----------|--------|---------------------------------------------|----------------------|
| Extension | bytes  | Vote extension provided by the Application. | Can have zero length |
| Height    | int64  | Height in which the extension was provided. | Must be >= 0         |
| Round     | int64  | Round in which the extension was provided.  | Must be >= 0         |
| ChainID   | string | ID of the blockchain running consensus.     |                      |

## Proposal

Proposal contains height and round for which this proposal is made, BlockID as a unique identifier
of proposed block, timestamp, and POLRound (a so-called Proof-of-Lock (POL) round) that is needed for
termination of the consensus. If POLRound >= 0, then BlockID corresponds to the block that was
or could have been locked in POLRound. The message is signed by the validator private key.

| Name      | Type                            | Description                                                                           | Validation                                              |
|-----------|---------------------------------|---------------------------------------------------------------------------------------|---------------------------------------------------------|
| Type      | [SignedMsgType](#signedmsgtype) | Represents a Proposal [SignedMsgType](#signedmsgtype).                                | Must be `ProposalType`                                  |
| Height    | uint64                          | Height for which this vote was created for                                            | Must be >= 0                                            |
| Round     | int32                           | Round that the commit corresponds to.                                                 | Must be >= 0                                            |
| POLRound  | int64                           | Proof of lock round.                                                                  | Must be >= -1                                           |
| BlockID   | [BlockID](#blockid)             | The blockID of the corresponding block.                                               | [BlockID](#blockid)                                     |
| Timestamp | [Time](#time)                   | Timestamp represents the time at which the block was produced.               | [Time](#time)                                           |
| Signature | slice of bytes (`[]byte`)       | Signature by the validator if they participated in consensus for the associated bock. | Length of signature must be > 0 and < 64                |

## SignedMsgType

Signed message type represents a signed messages in consensus.

```proto
enum SignedMsgType {

  SIGNED_MSG_TYPE_UNKNOWN = 0;
  // Votes
  SIGNED_MSG_TYPE_PREVOTE   = 1;
  SIGNED_MSG_TYPE_PRECOMMIT = 2;

  // Proposal
  SIGNED_MSG_TYPE_PROPOSAL = 32;
}
```

## Signature

Signatures in CometBFT are raw bytes representing the underlying signature.

See the [signature spec](./encoding.md#key-types) for more.

## EvidenceList

EvidenceList is a simple wrapper for a list of evidence:

| Name     | Type                           | Description                            | Validation                                                      |
|----------|--------------------------------|----------------------------------------|-----------------------------------------------------------------|
| Evidence | Array of [Evidence](#evidence) | List of verified [evidence](#evidence) | Validation adheres to individual types of [Evidence](#evidence) |

## Evidence

Evidence in CometBFT is used to indicate breaches in the consensus by a validator.

More information on how evidence works in CometBFT can be found [here](../consensus/evidence.md)

### DuplicateVoteEvidence

`DuplicateVoteEvidence` represents a validator that has voted for two different blocks
in the same round of the same height. Votes are lexicographically sorted on `BlockID`.

| Name             | Type          | Description                                                        | Validation                                          |
|------------------|---------------|--------------------------------------------------------------------|-----------------------------------------------------|
| VoteA            | [Vote](#vote) | One of the votes submitted by a validator when they equivocated    | VoteA must adhere to [Vote](#vote) validation rules |
| VoteB            | [Vote](#vote) | The second vote submitted by a validator when they equivocated     | VoteB must adhere to [Vote](#vote) validation rules |
| TotalVotingPower | int64         | The total power of the validator set at the height of equivocation | Must be equal to nodes own copy of the data         |
| ValidatorPower   | int64         | Power of the equivocating validator at the height                  | Must be equal to the nodes own copy of the data     |
| Timestamp        | [Time](#time) | Time of the block where the equivocation occurred                  | Must be equal to the nodes own copy of the data     |

### LightClientAttackEvidence

`LightClientAttackEvidence` is a generalized evidence that captures all forms of known attacks on
a light client such that a full node can verify, propose and commit the evidence on-chain for
punishment of the malicious validators. There are three forms of attacks: Lunatic, Equivocation
and Amnesia. These attacks are exhaustive. You can find a more detailed overview of this [here](../light-client/accountability#the_misbehavior_of_faulty_validators)

| Name                 | Type                               | Description                                                          | Validation                                                       |
|----------------------|------------------------------------|----------------------------------------------------------------------|------------------------------------------------------------------|
| ConflictingBlock     | [LightBlock](#lightblock)          | Read Below                                                           | Must adhere to the validation rules of [lightBlock](#lightblock) |
| CommonHeight         | int64                              | Read Below                                                           | must be > 0                                                      |
| Byzantine Validators | Array of [Validators](#validator) | validators that acted maliciously                                    | Read Below                                                       |
| TotalVotingPower     | int64                              | The total power of the validator set at the height of the infraction | Must be equal to the nodes own copy of the data                  |
| Timestamp            | [Time](#time)                      | Time of the block where the infraction occurred                      | Must be equal to the nodes own copy of the data                  |

## LightBlock

LightBlock is the core data structure of the [light client](../light-client/README.md). It combines two data structures needed for verification ([signedHeader](#signedheader) & [validatorSet](#validatorset)).

| Name         | Type                          | Description                                                                                                                            | Validation                                                                          |
|--------------|-------------------------------|----------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| SignedHeader | [SignedHeader](#signedheader) | The header and commit, these are used for verification purposes. To find out more visit [light client docs](../light-client/README.md) | Must not be nil and adhere to the validation rules of [signedHeader](#signedheader) |
| ValidatorSet | [ValidatorSet](#validatorset) | The validatorSet is used to help with verify that the validators in that committed the infraction were truly in the validator set.     | Must not be nil and adhere to the validation rules of [validatorSet](#validatorset) |

The `SignedHeader` and `ValidatorSet` are linked by the hash of the validator set(`SignedHeader.ValidatorsHash == ValidatorSet.Hash()`.

## SignedHeader

The SignedhHeader is the [header](#header) accompanied by the commit to prove it.

| Name   | Type              | Description       | Validation                                                                        |
|--------|-------------------|-------------------|-----------------------------------------------------------------------------------|
| Header | [Header](#header) | [Header](#header) | Header cannot be nil and must adhere to the [Header](#header) validation criteria |
| Commit | [Commit](#commit) | [Commit](#commit) | Commit cannot be nil and must adhere to the [Commit](#commit) criteria            |

## ValidatorSet

| Name       | Type                             | Description                                        | Validation                                                                                                        |
|------------|----------------------------------|----------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|
| Validators | Array of [validator](#validator) | List of the active validators at a specific height | The list of validators can not be empty or nil and must adhere to the validation rules of [validator](#validator) |
| Proposer   | [validator](#validator)          | The block proposer for the corresponding block     | The proposer cannot be nil and must adhere to the validation rules of  [validator](#validator)                    |

## Validator

| Name             | Type                      | Description                                                                                       | Validation                                        |
|------------------|---------------------------|---------------------------------------------------------------------------------------------------|---------------------------------------------------|
| Address          | [Address](#address)       | Validators Address                                                                                | Length must be of size 20                         |
| Pubkey           | slice of bytes (`[]byte`) | Validators Public Key                                                                             | must be a length greater than 0                   |
| VotingPower      | int64                     | Validators voting power                                                                           | cannot be < 0                                     |
| ProposerPriority | int64                     | Validators proposer priority. This is used to gauge when a validator is up next to propose blocks | No validation, value can be negative and positive |

## Address

Address is a type alias of a slice of bytes. The address is calculated by hashing the public key using sha256 and truncating it to only use the first 20 bytes of the slice.

```go
const (
  TruncatedSize = 20
)

func SumTruncated(bz []byte) []byte {
  hash := sha256.Sum256(bz)
  return hash[:TruncatedSize]
}
```

## Proof

| Name      | Type           | Description                                   | Field Number |
|-----------|----------------|-----------------------------------------------|:------------:|
| total     | int64          | Total number of items.                        | 1            |
| index     | int64          | Index item to prove.                          | 2            |
| leaf_hash | bytes          | Hash of item value.                           | 3            |
| aunts     | repeated bytes | Hashes from leaf's sibling to a root's child. | 4            |

## ConsensusParams

| Name      | Type                                | Description                                                             | Field Number |
|-----------|-------------------------------------|-------------------------------------------------------------------------|:------------:|
| block     | [BlockParams](#blockparams)         | Parameters limiting the block and gas.                                  | 1            |
| evidence  | [EvidenceParams](#evidenceparams)   | Parameters determining the validity of evidences of Byzantine behavior. | 2            |
| validator | [ValidatorParams](#validatorparams) | Parameters limiting the types of public keys validators can use.        | 3            |
| version   | [VersionParams](#versionparams)     | The version of specific components of CometBFT.                         | 4            |
| synchrony | [SynchronyParams](#synchronyparams) | Parameters determining the validity of block timestamps.                | 6            |
| feature   | [FeatureParams](#featureparms)      | Parameters for configuring the height from which features are enabled.  | 7            |

### BlockParams

| Name      | Type  | Description                                             | Field Number |
|-----------|-------|---------------------------------------------------------|:------------:|
| max_bytes | int64 | Maximum size of a block, in bytes.                      | 1            |
| max_gas   | int64 | Maximum gas wanted by transactions included in a block. | 2            |

The `max_bytes` parameter must be greater or equal to -1, and cannot be greater
than the hard-coded maximum block size, which is 100MB.
If set to -1, the limit is the hard-coded maximum block size.

The `max_gas` parameter must be greater or equal to -1.
If set to -1, no limit is enforced.

Blocks that violate `max_gas` were potentially proposed by Byzantine validators.
CometBFT does not enforce the maximum wanted gas for committed blocks.
It is responsibility of the application handling blocks whose wanted gas exceeds
the configured `max_gas` when processing the block.

### EvidenceParams

| Name               | Type                                       | Description                                                          | Field Number |
|--------------------|--------------------------------------------|----------------------------------------------------------------------|:------------:|
| max_age_num_blocks | int64                                      | Max age of evidence, in blocks.                                      | 1            |
| max_age_duration   | [google.protobuf.Duration][proto-duration] | Max age of evidence, in time.                                        | 2            |
| max_bytes          | int64                                      | Maximum size in bytes of evidence allowed to be included in a block. | 3            |

The recommended value of `max_age_duration` parameter should correspond to
the application's "unbonding period" or other similar mechanism for handling
[Nothing-At-Stake attacks](https://github.com/ethereum/wiki/wiki/Proof-of-Stake-FAQ#what-is-the-nothing-at-stake-problem-and-how-can-it-be-fixed).

The recommended formula for calculating `max_age_num_blocks` is `max_age_duration / {average block time}`.

### ValidatorParams

| Name          | Type            | Description                                                           | Field Number |
|---------------|-----------------|-----------------------------------------------------------------------|:------------:|
| pub_key_types | repeated string | List of accepted public key types. Uses same naming as `PubKey.Type`. | 1            |

The `pub_key_types` parameter uses ABCI public keys naming, not Amino names.

### VersionParams

| Name | Type   | Description                   | Field Number |
|------|--------|-------------------------------|:------------:|
| app  | uint64 | The ABCI application version. | 1            |

The `app` parameter was named `app_version` in CometBFT 0.34.

### ABCIParams

| Name                          | Type  | Description                                       | Field Number |
|-------------------------------|-------|---------------------------------------------------|:------------:|
| vote_extensions_enable_height | int64 | The height where vote extensions will be enabled. | 1            |

The `ABCIParams` type has been **deprecated** from CometBFT `v1.0`.

### FeatureParams

| Name                          | Type  | Description                                                       | Field Number |
|-------------------------------|-------|-------------------------------------------------------------------|:------------:|
| vote_extensions_enable_height | int64 | First height during which vote extensions will be enabled.        | 1            |
| pbts_enable_height            | int64 | Height at which Proposer-Based Timestamps (PBTS) will be enabled. | 2            |

From the configured height, and for all subsequent heights, the corresponding
feature will be enabled.
Cannot be set to heights lower or equal to the current blockchain height.
A value of 0 (the default) indicates that the feature is disabled.

### SynchronyParams

| Name          | Type                                       | Description                                                                                                             | Field Number |
|---------------|--------------------------------------------|-------------------------------------------------------------------------------------------------------------------------|:------------:|
| precision     | [google.protobuf.Duration][proto-duration] | Bound for how skewed a proposer's clock may be from any validator on the network while still producing valid proposals. | 1            |
| message_delay | [google.protobuf.Duration][proto-duration] | Bound for how long a proposal message may take to reach all validators on a network and still be considered valid.      | 2            |

These parameters are part of the Proposer-Based Timestamps (PBTS) algorithm.
For more information on the relationship of the synchrony parameters to
block timestamps validity, refer to the [PBTS specification][pbts].

[pbts]: ../consensus/proposer-based-timestamp/README.md
[bfttime]: ../consensus/bft-time.md
[proto-duration]: https://protobuf.dev/reference/protobuf/google.protobuf/#duration
