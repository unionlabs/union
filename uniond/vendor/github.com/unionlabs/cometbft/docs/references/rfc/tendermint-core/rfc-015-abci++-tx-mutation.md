# RFC 015: ABCI++ TX Mutation

## Changelog

- 23-Feb-2022: Initial draft (@williambanfield).
- 28-Feb-2022: Revised draft (@williambanfield).

## Abstract

A previous version of the ABCI++ specification detailed a mechanism for proposers to replace transactions
in the proposed block. This scheme required the proposer to construct new transactions
and mark these new transactions as replacing other removed transactions. The specification
was ambiguous as to how the replacement may be communicated to peer nodes.
This RFC discusses issues with this mechanism and possible solutions.

## Background

### What is the proposed change?

A previous version of the ABCI++ specification proposed mechanisms for adding, removing, and replacing
transactions in a proposed block. To replace a transaction, the application running
`ProcessProposal` could mark a transaction as replaced by other application-supplied
transactions by returning a new transaction marked with the `ADDED` flag setting
the `new_hashes` field of the removed transaction to contain the list of transaction hashes
that replace it. In that previous specification for ABCI++, the full use of the
`new_hashes` field is left somewhat ambiguous. At present, these hashes are not
gossiped and are not eventually included in the block to signal replacement to
other nodes. The specification did indicate that the transactions specified in
the `new_hashes` field will be removed from the mempool but it's not clear how
peer nodes will learn about them.

### What systems would be affected by adding transaction replacement?

The 'transaction' is a central building block of a Tendermint blockchain, so adding
a mechanism for transaction replacement would require changes to many aspects of Tendermint.

The following is a rough list of the functionality that this mechanism would affect:

#### Transaction indexing

Tendermint's indexer stores transactions and transaction results using the hash of the executed
transaction [as the key][tx-result-index] and the ABCI results and transaction bytes as the value.

To allow transaction replacement, the replaced transactions would need to stored as well in the
indexer, likely as a mapping of original transaction to list of transaction hashes that replaced
the original transaction.

#### Transaction inclusion proofs

The result of a transaction query includes a Merkle proof of the existence of the
transaction in the block chain. This [proof is built][inclusion-proof] as a merkle tree
of the hashes of all of the transactions in the block where the queried transaction was executed.

To allow transaction replacement, these proofs would need to be updated to prove
that a replaced transaction was included by replacement in the block.

#### RPC-based transaction query parameters and results

Tendermint's RPC allows clients to retrieve information about transactions via the
`/tx_search` and `/tx` RPC endpoints.

RPC query results containing replaced transactions would need to be updated to include
information on replaced transactions, either by returning results for all of the replaced
transactions, or by including a response with just the hashes of the replaced transactions
which clients could proceed to query individually.

#### Mempool transaction removal

Additional logic would need to be added to the Tendermint mempool to clear out replaced
transactions after each block is executed. Tendermint currently removes executed transactions
from the mempool, so this would be a pretty straightforward change.

## Discussion

### What value may be added to Tendermint by introducing transaction replacement?

Transaction replacement would would enable applications to aggregate or disaggregate transactions.

For aggregation, a set of transactions that all related work, such as transferring
tokens between the same two accounts, could be replaced with a single transaction,
i.e. one that transfers a single sum from one account to the other.
Applications that make frequent use of aggregation may be able to achieve a higher throughput.
Aggregation would decrease the space occupied by a single client-submitted transaction in the block, allowing
more client-submitted transactions to be executed per block.

For disaggregation, a very complex transaction could be split into multiple smaller transactions.
This may be useful if an application wishes to perform more fine-grained indexing on intermediate parts
of a multi-part transaction.

### Drawbacks to transaction replacement

Transaction replacement would require updating and shimming many of the places that
Tendermint records and exposes information about executed transactions. While
systems within Tendermint could be updated to account for transaction replacement,
such a system would leave new issues and rough edges.

#### No way of guaranteeing correct replacement

If a user issues a transaction to the network and the transaction is replaced, the
user has no guarantee that the replacement was correct. For example, suppose a set of users issue
transactions A, B, and C and they are all aggregated into a new transaction, D.
There is nothing guaranteeing that D was constructed correctly from the inputs.
The only way for users to ensure D is correct would be if D contained all of the
information of its constituent transactions, in which case, nothing is really gained by the replacement.

#### Replacement transactions not signed by submitter

Abstractly, Tendermint simply views transactions as a ball of bytes and therefore
should be fine with replacing one for another. However, many applications require
that transactions submitted to the chain be signed by some private key to authenticate
and authorize the transaction. Replaced transactions could not be signed by the
submitter, only by the application node. Therefore, any use of transaction replacement
could not contain authorization from the submitter and would either need to grant
application-submitted transactions power to perform application logic on behalf
of a user without their consent.

Granting this power to application-submitted transactions would be very dangerous
and therefore might not be of much value to application developers.
Transaction replacement might only be really safe in the case of application-submitted
transactions or for transactions that require no authorization. For such transactions,
it's quite not quite clear what the utility of replacement is: the application can already
generate any transactions that it wants. The fact that such a transaction was a replacement
is not particularly relevant to participants in the chain since the application is
merely replacing its own transactions.

#### New vector for censorship

Depending on the implementation, transaction replacement may allow a node signal
to the rest of the chain that some transaction should no longer be considered for execution.
Honest nodes will use the replacement mechanism to signal that a transaction has been aggregated.
Malicious nodes will be granted a new vector for censoring transactions.
There is no guarantee that a replaced transactions is actually executed at all.
A malicious node could censor a transaction by simply listing it as replaced.
Honest nodes seeing the replacement would flush the transaction from their mempool
and not execute or propose it it in later blocks.

### Transaction tracking implementations

This section discusses possible ways to flesh out the implementation of transaction replacement.
Specifically, this section proposes a few alternative ways that Tendermint blockchains could
track and store transaction replacements.

#### Include transaction replacements in the block

One option to track transaction replacement is to include information on the
transaction replacement within the block. An additional structure may be added
the block of the following form:

```proto
message Block {
...
  repeated Replacement replacements = 5;
}

message Replacement {
  bytes          included_tx_key   = 1;
  repeated bytes replaced_txs_keys = 2;
}
```

Applications executing `PrepareProposal` would return the list of replacements and
Tendermint would include an encoding of these replacements in the block that is gossiped
and committed.

Tendermint's transaction indexing would include a new mapping for each replaced transaction
key to the committed transaction.
Transaction inclusion proofs would be updated to include these additional new transaction
keys in the Merkle tree and queries for transaction hashes that were replaced would return
information indicating that the transaction was replaced along with the hash of the
transaction that replaced it.

Block validation of gossiped blocks would be updated to check that each of the
`included_txs_key` matches the hash of some transaction in the proposed block.

Implementing the changes described in this section would allow Tendermint to gossip
and index transaction replacements as part of block propagation. These changes would
still require the application to certify that the replacements were valid. This
validation may be performed in one of two ways:

1. **Applications optimistically trust that the proposer performed a legitimate replacement.**

In this validation scheme, applications would not verify that the substitution
is valid during consensus and instead simply trust that the proposer is correct.
This would have the drawback of allowing a malicious proposer to remove transactions
it did not want executed.

2. **Applications completely validate transaction replacement.**

In this validation scheme, applications that allow replacement would check that
each listed replaced transaction was correctly reflected in the replacement transaction.
In order to perform such validation, the node would need to have the replaced transactions
locally. This could be accomplished one of a few ways: by querying the mempool,
by adding an additional p2p gossip channel for transaction replacements, or by including the replaced transactions
in the block. Replacement validation via mempool querying would require the node
to have received all of the replaced transactions in the mempool which is far from
guaranteed. Adding an additional gossip channel would make gossiping replaced transactions
a requirement for consensus to proceed, since all nodes would need to receive all replacement
messages before considering a block valid. Finally, including replaced transactions in
the block seems to obviate any benefit gained from performing a transaction replacement
since the replaced transaction and the original transactions would now both appear in the block.

#### Application defined transaction replacement

An additional option for allowing transaction replacement is to leave it entirely as a responsibility
of the application. The `PrepareProposal` ABCI++ call allows for applications to add
new transactions to a proposed block. Applications that wished to implement a transaction
replacement mechanism would be free to do so without the newly defined `new_hashes` field.
Applications wishing to implement transaction replacement would add the aggregated
transactions in the `PrepareProposal` response, and include one additional bookkeeping
transaction that listed all of the replacements, with a similar scheme to the `new_hashes`
field described in ABCI++. This new bookkeeping transaction could be used by the
application to determine which transactions to clear from the mempool in future calls
to `CheckTx`.

The meaning of any transaction in the block is completely opaque to Tendermint,
so applications performing this style of replacement would not be able to have the replacement
reflected in any most of Tendermint's transaction tracking mechanisms, such as transaction indexing
and the `/tx` endpoint.

#### Application defined Tx Keys

Tendermint currently uses cryptographic hashes, SHA256, as a key for each transaction.
As noted in the section on systems that would require changing, this key is used
to identify the transaction in the mempool, in the indexer, and within the RPC system.

An alternative approach to allowing `ProcessProposal` to specify a set of transaction
replacements would be instead to allow the application to specify an additional key or set
of keys for each transaction during `ProcessProposal`. This new `secondary_keys` set
would be included in the block and therefore gossiped during block propagation.
Additional RPC endpoints could be exposed to query by the application-defined keys.

Applications wishing to implement replacement would leverage this new field by providing the
replaced transaction hashes as the `secondary_keys` and checking their validity during
`ProcessProposal`. During `RecheckTx` the application would then be responsible for
clearing out transactions that matched the `secondary_keys`.

It is worth noting that something like this would be possible without `secondary_keys`.
An application wishing to implement a system like this one could define a replacement
transaction, as discussed in the section on application-defined transaction replacement,
and use a custom [ABCI event type][abci-event-type] to communicate that the replacement should
be indexed within Tendermint's ABCI event indexing.

### Complexity to value-add tradeoff

It is worth remarking that adding a system like this may introduce a decent amount
of new complexity into Tendermint. An approach that leaves much of the replacement
logic to Tendermint would require altering the core transaction indexing and querying
data. In many of the cases listed, a system for transaction replacement is possible
without explicitly defining it as part of `PrepareProposal`. Since applications
can now add transactions during `PrepareProposal` they can and should leverage this
functionality to include additional bookkeeping transactions in the block. It may
be worth encouraging applications to discover new and interesting ways to leverage this
power instead of immediately solving the problem for them.

### References

[inclusion-proof]: https://github.com/tendermint/tendermint/blob/0fcfaa4568cb700e27c954389c1fcd0b9e786332/types/tx.go#L67
[tx-result-index]: https://github.com/tendermint/tendermint/blob/0fcfaa4568cb700e27c954389c1fcd0b9e786332/internal/state/indexer/tx/kv/kv.go#L90
[abci-event-type]: https://github.com/tendermint/tendermint/blob/0fcfaa4568cb700e27c954389c1fcd0b9e786332/abci/types/types.pb.go#L3168
