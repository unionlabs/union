---
order: 5
---

# Indexing Transactions

CometBFT allows you to index transactions and blocks and later query or
subscribe to their results. Transactions are indexed by `FinalizeBlockResponse.tx_results.events` and
blocks are indexed by `FinalizeBlockResponse.events`. However, transactions
are also indexed by a primary key which includes the transaction hash and maps
to and stores the corresponding transaction results. Blocks are indexed by a primary key
which includes the block height and maps to and stores the block height, i.e.
the block itself is never stored.

Each event contains a type and a list of attributes, which are key-value pairs
denoting something about what happened during the method's execution. For more
details on `Events`, see the [ABCI][abci-events] documentation.

An `Event` has a composite key associated with it. A `compositeKey` is
constructed by its type and key separated by a dot.

For example:

```json
"jack": [
  "account.number": 100
]
```

would be equal to the composite key of `jack.account.number`.

By default, CometBFT will index all transactions by their respective hashes
and height and blocks by their height.

CometBFT allows for different events within the same height to have
equal attributes.

## Configuration

Operators can configure indexing via the `[tx_index]` section. The `indexer`
field takes a series of supported indexers. If `null` is included, indexing will
be turned off regardless of other values provided.

```toml
[tx-index]

# The backend database to back the indexer.
# If indexer is "null", no indexer service will be used.
#
# The application will set which txs to index. In some cases a node operator will be able
# to decide which txs to index based on configuration set in the application.
#
# Options:
#   1) "null"
#   2) "kv" (default) - the simplest possible indexer, backed by key-value storage (defaults to levelDB; see DBBackend).
#     - When "kv" is chosen "tx.height" and "tx.hash" will always be indexed.
#   3) "psql" - the indexer services backed by PostgreSQL.
# indexer = "kv"
```

### Supported Indexers

#### KV

The `kv` indexer type is an embedded key-value store supported by the main
underlying CometBFT database. Using the `kv` indexer type allows you to query
for block and transaction events directly against CometBFT's RPC. However, the
query syntax is limited and so this indexer type might be deprecated or removed
entirely in the future.

**Implementation and data layout**

The kv indexer stores each attribute of an event individually, by creating a composite key
with
- event type,
- attribute key,
- attribute value,
- event generator (e.g. `FinalizeBlock`)
- the height, and
- event counter.
 For example the following events:

```
Type: "transfer",
  Attributes: []abci.EventAttribute{
   {Key: "sender", Value: "Bob", Index: true},
   {Key: "recipient", Value: "Alice", Index: true},
   {Key: "balance", Value: "100", Index: true},
   {Key: "note", Value: "nothing", Index: true},
   },

```

```
Type: "transfer",
  Attributes: []abci.EventAttribute{
   {Key: "sender", Value: "Tom", Index: true},
   {Key: "recipient", Value: "Alice", Index: true},
   {Key: "balance", Value: "200", Index: true},
   {Key: "note", Value: "nothing", Index: true},
   },
```

will be represented as follows in the store, assuming these events result from the `FinalizeBlock` call for height 1:

```
Key                                 value
---- event1 ------
transferSenderBobFinalizeBlock11           1
transferRecipientAliceFinalizeBlock11      1
transferBalance100FinalizeBlock11          1
transferNodeNothingFinalizeBlock11         1
---- event2 ------
transferSenderTomFinalizeBlock12           1
transferRecepientAliceFinalizeBlock12      1
transferBalance200FinalizeBlock12          1
transferNodeNothingFinalizeBlock12         1

```
The event number is a local variable kept by the indexer and incremented when a new event is processed.
It is an `int64` variable and has no other semantics besides being used to associate attributes belonging to the same events within a height.
This variable is not atomically incremented as event indexing is deterministic. **Should this ever change**, the event id generation
will be broken.

#### PostgreSQL

The `psql` indexer type allows an operator to enable block and transaction event
indexing by proxying it to an external PostgreSQL instance allowing for the events
to be stored in relational models. Since the events are stored in a RDBMS, operators
can leverage SQL to perform a series of rich and complex queries that are not
supported by the `kv` indexer type. Since operators can leverage SQL directly,
searching is not enabled for the `psql` indexer type via CometBFT's RPC -- any
such query will fail.

Note, the SQL schema is stored in `state/indexer/sink/psql/schema.sql` and operators
must explicitly create the relations prior to starting CometBFT and enabling
the `psql` indexer type.

Example:

```shell
psql ... -f state/indexer/sink/psql/schema.sql
```

The schema file adopts standard table names: `blocks`, `tx_results`, `events`, and `attributes`.
In order to adopt customizable table names, the user should adapt the schema file **and** configure CometBFT's indexer to employ the appropriate table names.

Example:
```toml
[tx-index]
psql-conn = "your connection string"
table_blocks = "cometbft_blocks"
table_tx_results = "cometbft_tx_results"
table_events = "cometbft_events"
table_attributes = "cometbft_attributes"
```

## Default Indexes

The CometBFT tx and block event indexer indexes a few select reserved events
by default.

### Transactions

The following indexes are indexed by default:

- `tx.height`
- `tx.hash`

### Blocks

The following indexes are indexed by default:

- `block.height`

## Adding Events

Applications are free to define which events to index. CometBFT does not
expose functionality to define which events to index and which to ignore. In
your application's `FinalizeBlock` method, add the `Events` field with pairs of
UTF-8 encoded strings (e.g. "transfer.sender": "Bob", "transfer.recipient":
"Alice", "transfer.balance": "100").

Example:

```go
func (app *Application) FinalizeBlock(_ context.Context, req *types.FinalizeBlockRequest) (*types.FinalizeBlockResponse, error) {

    //...
  tx_results[0] := &types.ExecTxResult{
			Code: CodeTypeOK,
			// With every transaction we can emit a series of events. To make it simple, we just emit the same events.
			Events: []types.Event{
				{
					Type: "app",
					Attributes: []types.EventAttribute{
						{Key: "creator", Value: "Cosmoshi Netowoko", Index: true},
						{Key: "key", Value: key, Index: true},
						{Key: "index_key", Value: "index is working", Index: true},
						{Key: "noindex_key", Value: "index is working", Index: false},
					},
				},
				{
					Type: "app",
					Attributes: []types.EventAttribute{
						{Key: "creator", Value: "Cosmoshi", Index: true},
						{Key: "key", Value: value, Index: true},
						{Key: "index_key", Value: "index is working", Index: true},
						{Key: "noindex_key", Value: "index is working", Index: false},
					},
				},
			},
		}

    block_events = []types.Event{
			{
				Type: "loan",
				Attributes: []types.EventAttribute{
					{	Key:   "account_no", Value: "1", Index: true},
					{ Key:   "amount", Value: "200", Index: true },
				},
			},
			{
				Type: "loan",
				Attributes: []types.EventAttribute{
					{ Key:   "account_no", Value: "2",	Index: true },
					{ Key:   "amount", Value: "300", Index: true},
				},
			},
		}
    return &types.FinalizeBlockResponse{TxResults: tx_results, Events: block_events}
}
```

If the indexer is not `null`, the transaction will be indexed. Each event is
indexed using a composite key in the form of `{eventType}.{eventAttribute}={eventValue}`,
e.g. `transfer.sender=bob`.

## Querying Transactions Events

You can query for a paginated set of transaction by their events by calling the
`/tx_search` RPC endpoint:

```bash
curl "localhost:26657/tx_search?query=\"message.sender='cosmos1...'\"&prove=true"
```

Check out [API docs](https://docs.cometbft.com/main/rpc/#/Info/tx_search)
for more information on query syntax and other options.

## Subscribing to Transactions

Clients can subscribe to transactions with the given tags via WebSocket by providing
a query to `/subscribe` RPC endpoint.

```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "id": "0",
  "params": {
    "query": "message.sender='cosmos1...'"
  }
}
```

Check out [API docs](https://docs.cometbft.com/main/rpc/#subscribe) for more information
on query syntax and other options.

## Querying Block Events

You can query for a paginated set of blocks by their events by calling the
`/block_search` RPC endpoint:

```bash
curl "localhost:26657/block_search?query=\"block.height > 10\""
```

Storing the event sequence was introduced in CometBFT 0.34.26. Before that, up
until Tendermint Core 0.34.26, the event sequence was not stored in the kvstore
and events were stored only by height. That means that queries returned blocks
and transactions whose event attributes match within the height but can match
across different events on that height.

This behavior was fixed with CometBFT 0.34.26+. However, if the data was
indexed with earlier versions of Tendermint Core and not re-indexed, that data
will be queried as if all the attributes within a height occurred within the
same event.

## Event attribute value types

Users can use anything as an event value. However, if the event attribute value
is a number, the following needs to be taken into account:

- Negative numbers will not be properly retrieved when querying the indexer.
- Event values are converted to big floats (from the `big/math` package). The
  precision of the floating point number is set to the bit length of the
  integer it is supposed to represent, so that there is no loss of information
  due to insufficient precision. This was not present before CometBFT v0.38.x
  and all float values were ignored.
- As of CometBFT v0.38.x, queries can contain floating point numbers as well.
- Note that comparing to floats can be imprecise with a high number of decimals.

## Event type and attribute key format

An event type/attribute key is a string that can contain any Unicode letter or
digit, as well as the following characters: `.` (dot), `-` (dash), `_`
(underscore). The event type/attribute key must not start with `-` (dash) or
`.` (dot).

```
^[\w]+[\.-\w]?$
```

[abci-events]: https://github.com/cometbft/cometbft/blob/main/spec/abci/abci++_basic_concepts.md#events
