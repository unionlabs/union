# RFC 024: Block Structure Consolidation

## Changelog

- 19-Apr-2022: Initial draft started (@williambanfield).
- 3-May-2022: Initial draft complete (@williambanfield).

## Abstract

The `Block` data structure is a very central structure within Tendermint. Because
of its centrality, it has gained several fields over the years through accretion.
Not all of these fields may be necessary any more. This document examines which
of these fields may no longer be necessary for inclusion in the block and makes
recommendations about how to proceed with each of them.

## Background

The current block structure contains multiple fields that are not required for
validation or execution of a Tendermint block. Some of these fields had vestigial
purposes that they no longer serve and some of these fields exist as a result of
internal Tendermint domain objects leaking out into the external data structure.

In so far as is possible, we should consolidate and prune these superfluous
fields before releasing a 1.0 version of Tendermint. All pruning of these
fields should be done with the aim of simplifying the structures to what
is needed while preserving information that aids with debugging and that also
allow external protocols to function more efficiently than if they were removed.

### Current Block Structure

The current block structures are included here to aid discussion.

```proto
message Block {
  Header                        header      = 1;
  Data                          data        = 2;
  tendermint.types.EvidenceList evidence    = 3;
  Commit                        last_commit = 4;
}
```

```proto
message Header {
  tendermint.version.Consensus version              = 1;
  string                       chain_id             = 2;
  int64                        height               = 3;
  google.protobuf.Timestamp    time                 = 4;
  BlockID                      last_block_id        = 5;
  bytes                        last_commit_hash     = 6;
  bytes                        data_hash            = 7;
  bytes                        validators_hash      = 8;
  bytes                        next_validators_hash = 9;
  bytes                        consensus_hash       = 10;
  bytes                        app_hash             = 11;
  bytes                        last_results_hash    = 12;
  bytes                        evidence_hash        = 13;
  bytes                        proposer_address     = 14;
}

```

```proto
message Data {
  repeated bytes txs = 1;
}
```

```proto
message EvidenceList {
  repeated Evidence evidence = 1;
}
```

```proto
message Commit {
  int64              height     = 1;
  int32              round      = 2;
  BlockID            block_id   = 3;
  repeated CommitSig signatures = 4;
}
```

```proto
message CommitSig {
  BlockIDFlag               block_id_flag     = 1;
  bytes                     validator_address = 2;
  google.protobuf.Timestamp timestamp         = 3;
  bytes                     signature         = 4;
}
```

```proto
message BlockID {
  bytes         hash            = 1;
  PartSetHeader part_set_header = 2;
}
```

### On Tendermint Blocks

#### What is a Tendermint 'Block'?

A block is the structure produced as the result of an instance of the Tendermint
consensus algorithm. At its simplest, the 'block' can be represented as a Merkle
root hash of all of the data used to construct and produce the hash. Our current
block proto structure includes _far_ from all of the data used to produce the
hashes included in the block.

It does not contain the full `AppState`, it does not contain the `ConsensusParams`,
nor the `LastResults`, nor the `ValidatorSet`. Additionally, the layout of
the block structure is not inherently tied to this Merkle root hash. Different
layouts of the same set of data could trivially be used to construct the
exact same hash. The thing we currently call the 'Block' is really just a view
into a subset of the data used to construct the root hash. Sections of the
structure can be modified as long as alternative methods exist to query and
retrieve the constituent values.

#### Why this digression?

This digression is aimed at informing what it means to consolidate 'fields' in the
'block'. The discussion of what should be included in the block can be teased
apart into a few different lines of inquiry.

1. What values need to be included as part of the Merkle tree so that the
consensus algorithm can use proof-of-stake consensus to validate all of the
properties of the chain that we would like?
2. How can we create views of the data that can be easily retrieved, stored, and
verified by the relevant protocols?

These two concerns are intertwined at the moment as a result of how we store
and propagate our data but they don't necessarily need to be. This document
focuses primarily on the first concern by suggesting fields that can be
completely removed without any loss in the function of our consensus algorithm.

This document also suggests ways that we may update our storage and propagation
mechanisms to better take advantage of Merkle tree nature of our data although
these are not its primary concern.

## Discussion

### Data to consider removing

This section proposes a list of data that could be completely removed from the
Merkle tree with no loss to the functionality of our consensus algorithm.

Where the change is possible but would hamper external protocols or make
debugging more difficult, that is noted in discussion.

#### CommitSig.Timestamp

This field contains the timestamp included in the precommit message that was
issued for the block. The field was once used to produce the timestamp of the block.
With the introduction of Proposer-Based Timestamps, This field is no longer used
in any Tendermint algorithms and can be completely removed.

#### CommitSig.ValidatorAddress

The `ValidatorAddress` is included in each `CommitSig` structure. This field
is hashed along with all of the other fields of the `CommitSig`s in the block
to form the `LastCommitHash` field in the `Header`. The `ValidatorAddress` is
somewhat redundant in the hash. Each validator has a unique position in the
`CommitSig` and the hash is built preserving this order. Therefore, the
information of which signature corresponds to which validator is included in
the root hash, even if the address is absent.

It's worth noting that the validator address could still be included in the
_hash_ even if it is absent from the `CommitSig` structure in the block by
simply hashing it locally at each validator but not including it in the block.
The reverse is also true. It would be perfectly possible to not include the
`ValidatorAddress` data in the `LastCommitHash` but still include the field in
the block.

#### BlockID.PartSetHeader

The [BlockID][block-id] field comprises the [PartSetHeader][part-set-header] and the hash of the block.
The `PartSetHeader` is used by nodes to gossip the block by dividing it into
parts. Nodes receive the `PartSetHeader` from their peers, informing them of
what pieces of the block to gather. There is no strong reason to include this
value in the block. Validators will still be able to gossip and validate the
blocks that they received from their peers using this mechanism even if it is
not written into the block. The `BlockID` can therefore be consolidated into
just the hash of the block. This is by far the most uncontroversial change
and there appears to be no good reason _not_ to do it. Further evidence that
the field is not meaningful can be found in the fact that the field is not
actually validated to ensure it is correct during block validation. Validation
only checks that the [field is well formed][psh-check].

#### ChainID

The `ChainID` is a string selected by the chain operators, usually a
human-readable name for the network. This value is immutable for the lifetime
of the chain and is defined in the genesis file. It is therefore hashed into the
original block and therefore transitively included as in the Merkle root hash of
every block. The redundant field is a candidate for removal from the root hash
of each block. However, aesthetically, it's somewhat nice to include in each
block, as if the block was 'stamped' with the ID. Additionally, re-validating
the value from genesis would be painful and require reconstituting potentially
large chains. I'm therefore mildly in favor of maintaining this redundant
piece of information. We pay almost no storage cost for maintaining this
identical data, so the only cost is in the time required to hash it into the
structure.

#### LastResultsHash

`LastResultsHash` is a hash covering the result of executing the transactions
from the previous block. It covers the response `Code`, `Data`, `GasWanted`,
and `GasUsed` with the aim of ensuring that execution of all of the transactions
was performed identically on each node. The data covered by this field _should_
be also reflected in the `AppHash`. The `AppHash` is provided by the application
and should be deterministically calculated by each node. This field could
therefore be removed on the grounds that its data is already reflected elsewhere.

I would advocate for keeping this field. This field provides an additional check
for determinism across nodes. Logic to update the application hash is more
complicated for developers to implement because it relies either on building a
complete view of the state of the application data. The `Results` returned by
the application contain simple response codes and deterministic data bytes.
Leaving the field will allow for transaction execution issues that are not
correctly reflected in the `AppHash` to be more completely diagnosed.

Take the case of mismatch of `LastResultsHash` between two nodes, A and B, where both
nodes report divergent values. If `A` and `B` both report
the same `AppHash`, then some non-deterministic behavior occurred that was not
accurately reflected in the `AppHash`. The issue caused by this
non-determinism may not show itself for several blocks, but catching the divergent
state earlier will improve the chances that a chain is able to recover.

#### ValidatorsHash

Both `ValidatorsHash` and `NextValidatorsHash` are included in the block
header. `Validatorshash` contains the hash of the [public key and voting power][val-hash]
of each validator in the active set for the current block and `NextValidatorsHash`
contains the same data but for the next height.

This data is effectively redundant. Having both values present in the block
structure is helpful for light client verification. The light client is able to
easily determine if two sequential blocks used the same validator set by querying
only one header.

`ValidatorsHash` is also important to the light client algorithm for performing block
validation. The light client uses this field to ensure that the validator set
it fetched from a full node is correct. It can be sure of the correctness of
the retrieved structure by hashing it and checking the hash against the `ValidatorsHash`
of the block it is verifying. Because a validator that the light client trusts
signed over the `ValidatorsHash`, it can be certain of the validity of the
structure. Without this check, phony validator sets could be handed to the light
client and the code tricked into believing a different validator set was present
at a height, opening up a major hole in the light client security model.

This creates a recursive problem. To verify the validator set that signed the
block at height `H`, what information do we need? We could fetch the
`NextValidatorsHash` from height `H-1`, but how do we verify that that hash is correct?

#### ProposerAddress

The section below details a change to allow the `ProposerAddress` to be calculated
from a field added to the block. This would allow the `Address` to be dropped
from the block. Consumers of the chain could run the proposer selection [algorithm][proposer-selection]
to determine who proposed each block.

I would advocate against this. Any consumer of the chain that wanted to
know which validator proposed a block would have to run the proposer selection
algorithm. This algorithm is not widely implemented, meaning that consumers
in other languages would need to implement the algorithm to determine a piece
of basic information about the chain.

### Data to consider adding

#### ProofOfLockRound

The _proof of lock round_ is the round of consensus for a height in which the
Tendermint algorithm observed a super majority of voting power on the network for
a block.

Including this value in the block will allow validation of currently
un-validated metadata. Specifically, including this value will allow Tendermint
to validate that the `ProposerAddress` in the block is correct. Without knowing
the locked round number, Tendermint cannot calculate which validator was supposed
to propose a height. Because of this, our [validation logic][proposer-check] does not check that
the `ProposerAddress` included in the block corresponds to the validator that
proposed the height. Instead, the validation logic simply checks that the value
is an address of one of the known validators.

Currently, we maintain the _committed round_ in the `Commit` for height `H`, which is
written into the block at height `H+1`. This value corresponds to the round in
which the proposer of height `H+1` received the commit for height `H`. The proof
of lock round would not subsume this value.

### Additional possible updates

#### Updates to storage

Currently we store the [every piece of each block][save-block] in the `BlockStore`.
I suspect that this has lead to some mistakes in reasoning around the merits of
consolidating fields in the block. We could update the storage scheme we use to
store only some pieces of each block and still achieve a space savings without having
to change the block structure at all.

The main way to achieve this would be by _no longer saving data that does not change_.
At each height we save a set of data that is unlikely to have changed from the
previous height in the block structure, this includes the `ValidatorAddress`es,
the `ValidatorsHash`, the `ChainID`. These do not need to be saved along with
_each_ block. We could easily save the value and the height at which the value
was updated and construct each block using the data that existed at the time.

This document does not make any specific recommendations around storage since
that is likely to change with upcoming improvements to to the database infrastructure.
However, it's important to note that removing fields from the block for the
purposes of 'saving space' may not be that meaningful. We should instead focus
our attention of removing fields from the block that are no longer needed
for correct functioning of the protocol.

#### Updates to propagation

Block propagation suffers from the same issue that plagues block storage, we
propagate all of the contents of each block proto _even when these contents are redundant
or unchanged from previous blocks_. For example, we propagate the `ValidatorAddress`es
for each block in the `CommitSig` structure even when it never changed from a
previous height. We could achieve a speed-up in many cases by communicating the
hashes _first_ and letting peers request additional information when they do not
recognize the communicated hash.

For example, in the case of the `ValidatorAddress`es, the node would first
communicate the `ValidatorsHash` of the block to its peers. The peers would
check their storage for a validator set matching the provided hash. If the peer
has a matching set, it would populate its local block structure with the
appropriate values from its store. If peer did not have a matching set, it would
issue a request to its peers, either via P2P or RPC for the data it did not have.

Conceptually, this is very similar to how content addressing works in protocols
such as git where pushing a commit does not require pushing the entire contents
of the tree referenced by the commit.

### Impact on light clients

As outlined in the section [On Tendermint Blocks](#on-tendermint-blocks), there
is a distinction between what data is referenced in the Merkle root hash and the
contents of the proto structure we currently call the `Block`.

Any changes to the Merkle root hash will necessarily be breaking for legacy light clients.
Either a soft-upgrades scheme will need to be implemented or a hard fork will
be required for chains and light clients to function with the new hashes.
This means that all of the additions and deletions from the Merkle root hash
proposed by this document will be light client breaking.

Changes to the block structure alone are not necessarily light client breaking if the
data being hashed are identical and legacy views into the data are provided
for old light clients during transitions. For example, a newer version of the
block structure could move the `ValidatorAddress` field to a different field
in the block while still including it in the hashed data of the `LastCommitHash`.
As long as old light clients could still fetch the old data structure, then
this would not be light client breaking.

## References

[part-set-header]: https://github.com/tendermint/tendermint/blob/208a15dadf01e4e493c187d8c04a55a61758c3cc/types/part_set.go#L94
[block-id]: https://github.com/tendermint/tendermint/blob/208a15dadf01e4e493c187d8c04a55a61758c3cc/types/block.go#L1090
[psh-check]: https://github.com/tendermint/tendermint/blob/208a15dadf01e4e493c187d8c04a55a61758c3cc/types/part_set.go#L116
[proposer-selection]: https://github.com/tendermint/tendermint/blob/208a15dadf01e4e493c187d8c04a55a61758c3cc/spec/consensus/proposer-selection.md
[val-hash]: https://github.com/tendermint/tendermint/blob/29e5fbcc648510e4763bd0af0b461aed92c21f30/types/validator.go#L160
[proposer-check]: https://github.com/tendermint/tendermint/blob/29e5fbcc648510e4763bd0af0b461aed92c21f30/internal/state/validation.go#L102
[save-block]: https://github.com/tendermint/tendermint/blob/59f0236b845c83009bffa62ed44053b04370b8a9/internal/store/store.go#L490
