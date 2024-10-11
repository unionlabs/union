# ADR 023: ABCI `ProposeTx` Method

## Changelog

25-06-2018: Initial draft based on [#1776](https://github.com/tendermint/tendermint/issues/1776)

## Context

[#1776](https://github.com/tendermint/tendermint/issues/1776) was
opened in relation to implementation of a Plasma child chain using Tendermint
Core as consensus/replication engine.

Due to the requirements of [Minimal Viable Plasma (MVP)](https://ethresear.ch/t/minimal-viable-plasma/426) and [Plasma Cash](https://ethresear.ch/t/plasma-cash-plasma-with-much-less-per-user-data-checking/1298), it is necessary for ABCI apps to have a mechanism to handle the following cases (more may emerge in the near future):

1. `deposit` transactions on the Root Chain, which must consist of a block
   with a single transaction, where there are no inputs and only one output
   made in favour of the depositor. In this case, a `block` consists of
   a transaction with the following shape:

   ```
   [0, 0, 0, 0, #input1 - zeroed out
    0, 0, 0, 0, #input2 - zeroed out
    <depositor_address>, <amount>, #output1 - in favour of depositor
    0, 0, #output2 - zeroed out
    <fee>,
   ]
   ```

   `exit` transactions may also be treated in a similar manner, wherein the
   input is the UTXO being exited on the Root Chain, and the output belongs to
   a reserved "burn" address, e.g., `0x0`. In such cases, it is favorable for
   the containing block to only hold a single transaction that may receive
   special treatment.

2. Other "internal" transactions on the child chain, which may be initiated
   unilaterally. The most basic example of is a coinbase transaction
   implementing validator node incentives, but may also be app-specific. In
   these cases, it may be favorable for such transactions to
   be ordered in a specific manner, e.g., coinbase transactions will always be
   at index 0. In general, such strategies increase the determinism and
   predictability of blockchain applications.

While it is possible to deal with the cases enumerated above using the
existing ABCI, currently available result in suboptimal workarounds. Two are
explained in greater detail below.

### Solution 1: App state-based Plasma chain

In this work around, the app maintains a `PlasmaStore` with a corresponding
`Keeper`. The PlasmaStore is responsible for maintaining a second, separate
blockchain that complies with the MVP specification, including `deposit`
blocks and other "internal" transactions. These "virtual" blocks are then broadcasted
to the Root Chain.

This naive approach is, however, fundamentally flawed, as it by definition
diverges from the canonical chain maintained by Tendermint. This is further
exacerbated if the business logic for generating such transactions is
potentially non-deterministic, as this should not even be done in
`Begin/EndBlock`, which may, as a result, break consensus guarantees.

Additinoally, this has serious implications for "watchers" - independent third parties,
or even an auxiliary blockchain, responsible for ensuring that blocks recorded
on the Root Chain are consistent with the Plasma chain's. Since, in this case,
the Plasma chain is inconsistent with the canonical one maintained by Tendermint
Core, it seems that there exists no compact means of verifying the legitimacy of
the Plasma chain without replaying every state transition from genesis (!).

### Solution 2: Broadcast to Tendermint Core from ABCI app

This approach is inspired by `tendermint`, in which Ethereum transactions are
relayed to Tendermint Core. It requires the app to maintain a client connection
to the consensus engine.

Whenever an "internal" transaction needs to be created, the proposer of the
current block broadcasts the transaction or transactions to Tendermint as
needed in order to ensure that the Tendermint chain and Plasma chain are
completely consistent.

This allows "internal" transactions to pass through the full consensus
process, and can be validated in methods like `CheckTx`, i.e., signed by the
proposer, is the semantically correct, etc. Note that this involves informing
the ABCI app of the block proposer, which was temporarily hacked in as a means
of conducting this experiment, although this should not be necessary when the
current proposer is passed to `BeginBlock`.

It is much easier to relay these transactions directly to the Root
Chain smart contract and/or maintain a "compressed" auxiliary chain comprised
of Plasma-friendly blocks that 100% reflect the canonical (Tendermint)
blockchain. Unfortunately, this approach not idiomatic (i.e., utilises the
Tendermint consensus engine in unintended ways). Additionally, it does not
allow the application developer to:

- Control the _ordering_ of transactions in the proposed block (e.g., index 0,
  or 0 to `n` for coinbase transactions)
- Control the _number_ of transactions in the block (e.g., when a `deposit`
  block is required)

Since determinism is of utmost importance in blockchain engineering, this approach,
while more viable, should also not be considered as fit for production.

## Decision

### `ProposeTx`

In order to address the difficulties described above, the ABCI interface must
expose an additional method, tentatively named `ProposeTx`.

It should have the following signature:

```
ProposeTx(RequestProposeTx) ResponseProposeTx
```

Where `RequestProposeTx` and `ResponseProposeTx` are `message`s with the
following shapes:

```
message RequestProposeTx {
  int64 next_block_height = 1; // height of the block the proposed tx would be part of
  Validator proposer = 2; // the proposer details
}

message ResponseProposeTx {
  int64 num_tx = 1; // the number of tx to include in proposed block
  repeated bytes txs = 2; // ordered transaction data to include in block
  bool exclusive = 3; // whether the block should include other transactions (from `mempool`)
}
```

`ProposeTx` would be called by before `mempool.Reap` at this
[line](https://github.com/tendermint/tendermint/blob/9cd9f3338bc80a12590631632c23c8dbe3ff5c34/consensus/state.go#L935).
Depending on whether `exclusive` is `true` or `false`, the proposed
transactions are then pushed on top of the transactions received from
`mempool.Reap`.

### `DeliverTx`

Since the list of `tx` received from `ProposeTx` are _not_ passed through `CheckTx`,
it is probably a good idea to provide a means of differentiatiating "internal" transactions
from user-generated ones, in case the app developer needs/wants to take extra measures to
ensure validity of the proposed transactions.

Therefore, the `RequestDeliverTx` message should be changed to provide an additional flag, like so:

```
message RequestDeliverTx {
	bytes tx = 1;
	bool internal = 2;
}
```

Alternatively, an additional method `DeliverProposeTx` may be added as an accompanient to
`ProposeTx`. However, it is not clear at this stage if this additional overhead is necessary
to preserve consensus guarantees given that a simple flag may suffice for now.

## Status

Pending

## Consequences

### Positive

- Tendermint ABCI apps will be able to function as minimally viable Plasma chains.
- It will thereby become possible to add an extension to `cosmos-sdk` to enable
  ABCI apps to support both IBC and Plasma, maximising interop.
- ABCI apps will have great control and flexibility in managing blockchain state,
  without having to resort to non-deterministic hacks and/or unsafe workarounds

### Negative

- Maintenance overhead of exposing additional ABCI method
- Potential security issues that may have been overlooked and must now be tested extensively

### Neutral

- ABCI developers must deal with increased (albeit nominal) API surface area.

## References

- [#1776 Plasma and "Internal" Transactions in ABCI Apps](https://github.com/tendermint/tendermint/issues/1776)
- [Minimal Viable Plasma](https://ethresear.ch/t/minimal-viable-plasma/426)
- [Plasma Cash: Plasma with much less per-user data checking](https://ethresear.ch/t/plasma-cash-plasma-with-much-less-per-user-data-checking/1298)
