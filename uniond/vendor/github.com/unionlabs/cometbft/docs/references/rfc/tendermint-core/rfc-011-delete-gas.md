# RFC 011: Remove Gas From Tendermint

## Changelog

- 03-Feb-2022: Initial draft (@williambanfield).
- 10-Feb-2022: Update in response to feedback (@williambanfield).
- 11-Feb-2022: Add reflection on MaxGas during consensus (@williambanfield).

## Abstract

In the v0.25.0 release, Tendermint added a mechanism for tracking 'Gas' in the mempool.
At a high level, Gas allows applications to specify how much it will cost the network,
often in compute resources, to execute a given transaction. While such a mechanism is common
in blockchain applications, it is not generalizable enough to be a maintained as a part
of Tendermint. This RFC explores the possibility of removing the concept of Gas from
Tendermint while still allowing applications the power to control the contents of
blocks to achieve similar goals.

## Background

The notion of Gas was included in the original Ethereum whitepaper and exists as
an important feature of the Ethereum blockchain.

The [whitepaper describes Gas][eth-whitepaper-messages] as an Anti-DoS mechanism. The Ethereum Virtual Machine
provides a Turing complete execution platform. Without any limitations, malicious
actors could waste computation resources by directing the EVM to perform large
or even infinite computations. Gas serves as a metering mechanism to prevent this.

Gas appears to have been added to Tendermint multiple times, initially as part of
a now defunct `/vm` package, and in its most recent iteration [as part of v0.25.0][gas-add-pr]
as a mechanism to limit the transactions that will be included in the block by an additional
parameter.

Gas has gained adoption within the Cosmos ecosystem [as part of the Cosmos SDK][cosmos-sdk-gas].
The SDK provides facilities for tracking how much 'Gas' a transaction is expected to take
and a mechanism for tracking how much gas a transaction has already taken.

Non-SDK applications also make use of the concept of Gas. Anoma appears to implement
[a gas system][anoma-gas] to meter the transactions it executes.

While the notion of gas is present in projects that make use of Tendermint, it is
not a concern of Tendermint's. Tendermint's value and goal is producing blocks
via a distributed consensus algorithm. Tendermint relies on the application specific
code to decide how to handle the transactions Tendermint has produced (or if the
application wants to consider them at all). Gas is an application concern.

Our implementation of Gas is not currently enforced by consensus. Our current validation check that
occurs during block propagation does not verify that the block is under the configured `MaxGas`.
Ensuring that the transactions in a proposed block do not exceed `MaxGas` would require
input from the application during propagation. The `ProcessProposal` method introduced
as part of ABCI++ would enable such input but would further entwine Tendermint and
the application. The issue of checking `MaxGas` during block propagation is important
because it demonstrates that the feature as it currently exists is not implemented
as fully as it perhaps should be.

Our implementation of Gas is causing issues for node operators and relayers. At
the moment, transactions that overflow the configured 'MaxGas' can be silently rejected
from the mempool. Overflowing MaxGas is the _only_ way that a transaction can be considered
invalid that is not directly a result of failing the `CheckTx`. Operators, and the application,
do not know that a transaction was removed from the mempool for this reason. A stateless check
of this nature is exactly what `CheckTx` exists for and there is no reason for the mempool
to keep track of this data separately. A special [MempoolError][add-mempool-error] field
was added in v0.35 to communicate to clients that a transaction failed after `CheckTx`.
While this should alleviate the pain for operators wishing to understand if their
transaction was included in the mempool, it highlights that the abstraction of
what is included in the mempool is not currently well defined.

Removing Gas from Tendermint and the mempool would allow for the mempool to be a better
abstraction: any transaction that arrived at `CheckTx` and passed the check will either be
a candidate for a later block or evicted after a TTL is reached or to make room for
other, higher priority transactions. All other transactions are completely invalid and can be discarded forever.

Removing gas will not be completely straightforward. It will mean ensuring that
equivalent functionality can be implemented outside of the mempool using the mempool's API.

## Discussion

This section catalogs the functionality that will need to exist within the Tendermint
mempool to allow Gas to be removed and replaced by application-side bookkeeping.

### Requirement: Provide Mempool Tx Sorting Mechanism

Gas produces a market for inclusion in a block. On many networks, a [gas fee][cosmos-sdk-fees] is
included in pending transactions. This fee indicates how much a user is willing to
pay per unit of execution and the fees are distributed to validators.

Validators wishing to extract higher gas fees are incentivized to include transactions
with the highest listed gas fees into each block. This produces a natural ordering
of the pending transactions. Applications wishing to implement a gas mechanism need
to be able to order the transactions in the mempool. This can trivially be accomplished
by sorting transactions using the `priority` field available to applications as part of
v0.35's `ResponseCheckTx` message.

### Requirement: Allow Application-Defined Block Resizing

When creating a block proposal, Tendermint pulls a set of possible transactions out of
the mempool to include in the next block. Tendermint uses MaxGas to limit the set of transactions
it pulls out of the mempool fetching a set of transactions whose sum is less than MaxGas.

By removing gas tracking from Tendermint's mempool, Tendermint will need to provide a way for
applications to determine an acceptable set of transactions to include in the block.

This is what the new ABCI++ `PrepareProposal` method is useful for. Applications
that wish to limit the contents of a block by an application-defined limit may
do so by removing transactions from the proposal it is passed during `PrepareProposal`.
Applications wishing to reach parity with the current Gas implementation may do
so by creating an application-side limit: filtering out transactions from
`PrepareProposal` the cause the proposal the exceed the maximum gas. Additionally,
applications can currently opt to have all transactions in the mempool delivered
during `PrepareProposal` by passing `-1` for `MaxGas` and `MaxBytes` into
[ReapMaxBytesMaxGas][reap-max-bytes-max-gas].

### Requirement: Handle Transaction Metadata

Moving the gas mechanism into applications adds an additional piece of complexity
to applications. The application must now track how much gas it expects a transaction
to consume. The mempool currently handles this bookkeeping responsibility and uses the estimated
gas to determine the set of transactions to include in the block. In order to task
the application with keeping track of this metadata, we should make it easier for the
application to do so. In general, we'll want to keep only one copy of this type
of metadata in the program at a time, either in the application or in Tendermint.

The following sections are possible solutions to the problem of storing transaction
metadata without duplication.

#### Metadata Handling: EvictTx Callback

A possible approach to handling transaction metadata is by adding a new `EvictTx`
ABCI method. Whenever the mempool is removing a transaction, either because it has
reached its TTL or because it failed `RecheckTx`, `EvictTx` would be called with
the transaction hash. This would indicate to the application that it could free any
metadata it was storing about the transaction such as the computed gas fee.

Eviction callbacks are pretty common in caching systems, so this would be very
well-worn territory.

#### Metadata Handling: Application-Specific Metadata Field(s)

An alternative approach to handling transaction metadata would be would be the
addition of a new application-metadata field in the `ResponseCheckTx`. This field
would be a protocol buffer message whose contents were entirely opaque to Tendermint.
The application would be responsible for marshalling and unmarshalling whatever data
it stored in this field. During `PrepareProposal`, the application would be passed
this metadata along with the transaction, allowing the application to use it to perform
any necessary filtering.

If either of these proposed metadata handling techniques are selected, it's likely
useful to enable applications to gossip metadata along with the transaction it is
gossiping. This could easily take the form of an opaque proto message that is
gossiped along with the transaction.

## References

[eth-whitepaper-messages]: https://ethereum.org/en/whitepaper/#messages-and-transactions
[gas-add-pr]: https://github.com/tendermint/tendermint/pull/2360
[cosmos-sdk-gas]: https://github.com/cosmos/cosmos-sdk/blob/c00cedb1427240a730d6eb2be6f7cb01f43869d3/docs/basics/gas-fees.md
[cosmos-sdk-fees]: https://github.com/cosmos/cosmos-sdk/blob/c00cedb1427240a730d6eb2be6f7cb01f43869d3/docs/basics/tx-lifecycle.md#gas-and-fees
[anoma-gas]: https://github.com/anoma/anoma/blob/6974fe1532a59db3574fc02e7f7e65d1216c1eb2/docs/src/specs/ledger.md#transaction-execution
[reap-max-bytes-max-gas]: https://github.com/tendermint/tendermint/blob/1ac58469f32a98f1c0e2905ca1773d9eac7b7103/internal/mempool/types.go#L45
[add-mempool-error]: https://github.com/tendermint/tendermint/blob/205bfca66f6da1b2dded381efb9ad3792f9404cf/rpc/coretypes/responses.go#L239
