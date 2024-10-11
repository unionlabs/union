# ADR 065: Custom Event Indexing

- [ADR 065: Custom Event Indexing](#adr-065-custom-event-indexing)
  - [Changelog](#changelog)
  - [Status](#status)
  - [Context](#context)
  - [Alternative Approaches](#alternative-approaches)
  - [Decision](#decision)
  - [Detailed Design](#detailed-design)
    - [EventSink](#eventsink)
    - [Supported Sinks](#supported-sinks)
      - [`KVEventSink`](#kveventsink)
      - [`PSQLEventSink`](#psqleventsink)
    - [Configuration](#configuration)
  - [Future Improvements](#future-improvements)
  - [Consequences](#consequences)
    - [Positive](#positive)
    - [Negative](#negative)
    - [Neutral](#neutral)
  - [References](#references)

## Changelog

- April 1, 2021: Initial Draft (@alexanderbez)
- April 28, 2021: Specify search capabilities are only supported through the KV indexer (@marbar3778)
- May 19, 2021: Update the SQL schema and the eventsink interface (@jayt106)
- Aug 30, 2021: Update the SQL schema and the psql implementation (@creachadair)
- Oct 5, 2021: Clarify goals and implementation changes (@creachadair)

## Status

Implemented

## Context

Currently, Tendermint Core supports block and transaction event indexing through
the `tx_index.indexer` configuration. Events are captured in transactions and
are indexed via a `TxIndexer` type. Events are captured in blocks, specifically
from `BeginBlock` and `EndBlock` application responses, and are indexed via a
`BlockIndexer` type. Both of these types are managed by a single `IndexerService`
which is responsible for consuming events and sending those events off to be
indexed by the respective type.

In addition to indexing, Tendermint Core also supports the ability to query for
both indexed transaction and block events via Tendermint's RPC layer. The ability
to query for these indexed events facilitates a great multitude of upstream client
and application capabilities, e.g. block explorers, IBC relayers, and auxiliary
data availability and indexing services.

Currently, Tendermint only supports indexing via a `kv` indexer, which is supported
by an underlying embedded key/value store database. The `kv` indexer implements
its own indexing and query mechanisms. While the former is somewhat trivial,
providing a rich and flexible query layer is not as trivial and has caused many
issues and UX concerns for upstream clients and applications.

The fragile nature of the proprietary `kv` query engine and the potential
performance and scaling issues that arise when a large number of consumers are
introduced, motivate the need for a more robust and flexible indexing and query
solution.

## Alternative Approaches

With regards to alternative approaches to a more robust solution, the only serious
contender that was considered was to transition to using [SQLite](https://www.sqlite.org/index.html).

While the approach would work, it locks us into a specific query language and
storage layer, so in some ways it's only a bit better than our current approach.
In addition, the implementation would require the introduction of CGO into the
Tendermint Core stack, whereas right now CGO is only introduced depending on
the database used.

## Decision

We will adopt a similar approach to that of the Cosmos SDK's `KVStore` state
listening described in [ADR-038](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-038-state-listening.md).

We will implement the following changes:

- Introduce a new interface, `EventSink`, that all data sinks must implement.
- Augment the existing `tx_index.indexer` configuration to now accept a series
  of one or more indexer types, i.e., sinks.
- Combine the current `TxIndexer` and `BlockIndexer` into a single `KVEventSink`
  that implements the `EventSink` interface.
- Introduce an additional `EventSink` implementation that is backed by
  [PostgreSQL](https://www.postgresql.org/).
  - Implement the necessary schemas to support both block and transaction event indexing.
- Update `IndexerService` to use a series of `EventSinks`.

In addition:

- The Postgres indexer implementation will _not_ implement the proprietary `kv`
  query language. Users wishing to write queries against the Postgres indexer
  will connect to the underlying DBMS directly and use SQL queries based on the
  indexing schema.

  Future custom indexer implementations will not be required to support the
  proprietary query language either.

- For now, the existing `kv` indexer will be left in place with its current
  query support, but will be marked as deprecated in a subsequent release, and
  the documentation will be updated to encourage users who need to query the
  event index to migrate to the Postgres indexer.

- In the future we may remove the `kv` indexer entirely, or replace it with a
  different implementation; that decision is deferred as future work.

- In the future, we may remove the index query endpoints from the RPC service
  entirely; that decision is deferred as future work, but recommended.


## Detailed Design

### EventSink

We introduce the `EventSink` interface type that all supported sinks must implement.
The interface is defined as follows:

```go
type EventSink interface {
  IndexBlockEvents(types.EventDataNewBlockHeader) error
  IndexTxEvents([]*abci.TxResult) error

  SearchBlockEvents(context.Context, *query.Query) ([]int64, error)
  SearchTxEvents(context.Context, *query.Query) ([]*abci.TxResult, error)

  GetTxByHash([]byte) (*abci.TxResult, error)
  HasBlock(int64) (bool, error)

  Type() EventSinkType
  Stop() error
}
```

The `IndexerService`  will accept a list of one or more `EventSink` types. During
the `OnStart` method it will call the appropriate APIs on each `EventSink` to
index both block and transaction events.

### Supported Sinks

We will initially support two `EventSink` types out of the box.

#### `KVEventSink`

This type of `EventSink` is a combination of the  `TxIndexer` and `BlockIndexer`
indexers, both of which are backed by a single embedded key/value database.

A bulk of the existing business logic will remain the same, but the existing APIs
mapped to the new `EventSink` API. Both types will be removed in favor of a single
`KVEventSink` type.

The `KVEventSink` will be the only `EventSink` enabled by default, so from a UX
perspective, operators should not notice a difference apart from a configuration
change.

We omit `EventSink` implementation details as it should be fairly straightforward
to map the existing business logic to the new APIs.

#### `PSQLEventSink`

This type of `EventSink` indexes block and transaction events into a [PostgreSQL](https://www.postgresql.org/).
database. We define and automatically migrate the following schema when the
`IndexerService` starts.

The postgres eventsink will not support `tx_search`, `block_search`, `GetTxByHash` and `HasBlock`.

```sql
-- Table Definition ----------------------------------------------

-- The blocks table records metadata about each block.
-- The block record does not include its events or transactions (see tx_results).
CREATE TABLE blocks (
  rowid      BIGSERIAL PRIMARY KEY,

  height     BIGINT NOT NULL,
  chain_id   VARCHAR NOT NULL,

  -- When this block header was logged into the sink, in UTC.
  created_at TIMESTAMPTZ NOT NULL,

  UNIQUE (height, chain_id)
);

-- Index blocks by height and chain, since we need to resolve block IDs when
-- indexing transaction records and transaction events.
CREATE INDEX idx_blocks_height_chain ON blocks(height, chain_id);

-- The tx_results table records metadata about transaction results.  Note that
-- the events from a transaction are stored separately.
CREATE TABLE tx_results (
  rowid BIGSERIAL PRIMARY KEY,

  -- The block to which this transaction belongs.
  block_id BIGINT NOT NULL REFERENCES blocks(rowid),
  -- The sequential index of the transaction within the block.
  index INTEGER NOT NULL,
  -- When this result record was logged into the sink, in UTC.
  created_at TIMESTAMPTZ NOT NULL,
  -- The hex-encoded hash of the transaction.
  tx_hash VARCHAR NOT NULL,
  -- The protobuf wire encoding of the TxResult message.
  tx_result BYTEA NOT NULL,

  UNIQUE (block_id, index)
);

-- The events table records events. All events (both block and transaction) are
-- associated with a block ID; transaction events also have a transaction ID.
CREATE TABLE events (
  rowid BIGSERIAL PRIMARY KEY,

  -- The block and transaction this event belongs to.
  -- If tx_id is NULL, this is a block event.
  block_id BIGINT NOT NULL REFERENCES blocks(rowid),
  tx_id    BIGINT NULL REFERENCES tx_results(rowid),

  -- The application-defined type label for the event.
  type VARCHAR NOT NULL
);

-- The attributes table records event attributes.
CREATE TABLE attributes (
   event_id      BIGINT NOT NULL REFERENCES events(rowid),
   key           VARCHAR NOT NULL, -- bare key
   composite_key VARCHAR NOT NULL, -- composed type.key
   value         VARCHAR NULL,

   UNIQUE (event_id, key)
);

-- A joined view of events and their attributes. Events that do not have any
-- attributes are represented as a single row with empty key and value fields.
CREATE VIEW event_attributes AS
  SELECT block_id, tx_id, type, key, composite_key, value
  FROM events LEFT JOIN attributes ON (events.rowid = attributes.event_id);

-- A joined view of all block events (those having tx_id NULL).
CREATE VIEW block_events AS
  SELECT blocks.rowid as block_id, height, chain_id, type, key, composite_key, value
  FROM blocks JOIN event_attributes ON (blocks.rowid = event_attributes.block_id)
  WHERE event_attributes.tx_id IS NULL;

-- A joined view of all transaction events.
CREATE VIEW tx_events AS
  SELECT height, index, chain_id, type, key, composite_key, value, tx_results.created_at
  FROM blocks JOIN tx_results ON (blocks.rowid = tx_results.block_id)
  JOIN event_attributes ON (tx_results.rowid = event_attributes.tx_id)
  WHERE event_attributes.tx_id IS NOT NULL;
```

The `PSQLEventSink` will implement the `EventSink` interface as follows
(some details omitted for brevity):

```go
func NewEventSink(connStr, chainID string) (*EventSink, error) {
	db, err := sql.Open(driverName, connStr)
	// ...

	return &EventSink{
		store:   db,
		chainID: chainID,
	}, nil
}

func (es *EventSink) IndexBlockEvents(h types.EventDataNewBlockHeader) error {
	ts := time.Now().UTC()

	return runInTransaction(es.store, func(tx *sql.Tx) error {
		// Add the block to the blocks table and report back its row ID for use
		// in indexing the events for the block.
		blockID, err := queryWithID(tx, `
INSERT INTO blocks (height, chain_id, created_at)
  VALUES ($1, $2, $3)
  ON CONFLICT DO NOTHING
  RETURNING rowid;
`, h.Header.Height, es.chainID, ts)
		// ...

		// Insert the special block meta-event for height.
		if err := insertEvents(tx, blockID, 0, []abci.Event{
			makeIndexedEvent(types.BlockHeightKey, fmt.Sprint(h.Header.Height)),
		}); err != nil {
			return fmt.Errorf("block meta-events: %w", err)
		}
		// Insert all the block events. Order is important here,
		if err := insertEvents(tx, blockID, 0, h.ResultBeginBlock.Events); err != nil {
			return fmt.Errorf("begin-block events: %w", err)
		}
		if err := insertEvents(tx, blockID, 0, h.ResultEndBlock.Events); err != nil {
			return fmt.Errorf("end-block events: %w", err)
		}
		return nil
	})
}

func (es *EventSink) IndexTxEvents(txrs []*abci.TxResult) error {
	ts := time.Now().UTC()

	for _, txr := range txrs {
		// Encode the result message in protobuf wire format for indexing.
		resultData, err := proto.Marshal(txr)
		// ...

		// Index the hash of the underlying transaction as a hex string.
		txHash := fmt.Sprintf("%X", types.Tx(txr.Tx).Hash())

		if err := runInTransaction(es.store, func(tx *sql.Tx) error {
			// Find the block associated with this transaction.
			blockID, err := queryWithID(tx, `
SELECT rowid FROM blocks WHERE height = $1 AND chain_id = $2;
`, txr.Height, es.chainID)
			// ...

			// Insert a record for this tx_result and capture its ID for indexing events.
			txID, err := queryWithID(tx, `
INSERT INTO tx_results (block_id, index, created_at, tx_hash, tx_result)
  VALUES ($1, $2, $3, $4, $5)
  ON CONFLICT DO NOTHING
  RETURNING rowid;
`, blockID, txr.Index, ts, txHash, resultData)
			// ...

			// Insert the special transaction meta-events for hash and height.
			if err := insertEvents(tx, blockID, txID, []abci.Event{
				makeIndexedEvent(types.TxHashKey, txHash),
				makeIndexedEvent(types.TxHeightKey, fmt.Sprint(txr.Height)),
			}); err != nil {
				return fmt.Errorf("indexing transaction meta-events: %w", err)
			}
			// Index any events packaged with the transaction.
			if err := insertEvents(tx, blockID, txID, txr.Result.Events); err != nil {
				return fmt.Errorf("indexing transaction events: %w", err)
			}
			return nil

		}); err != nil {
			return err
		}
	}
	return nil
}

// SearchBlockEvents is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) SearchBlockEvents(ctx context.Context, q *query.Query) ([]int64, error)

// SearchTxEvents is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) SearchTxEvents(ctx context.Context, q *query.Query) ([]*abci.TxResult, error)

// GetTxByHash is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) GetTxByHash(hash []byte) (*abci.TxResult, error)

// HasBlock is not implemented by this sink, and reports an error for all queries.
func (es *EventSink) HasBlock(h int64) (bool, error)
```

### Configuration

The current `tx_index.indexer` configuration would be changed to accept a list
of supported `EventSink` types instead of a single value.

Example:

```toml
[tx_index]

indexer = [
  "kv",
  "psql"
]
```

If the `indexer` list contains the `null` indexer, then no indexers will be used
regardless of what other values may exist.

Additional configuration parameters might be required depending on what event
sinks are supplied to `tx_index.indexer`. The `psql` will require an additional
connection configuration.

```toml
[tx_index]

indexer = [
  "kv",
  "psql"
]

pqsql_conn = "postgresql://<user>:<password>@<host>:<port>/<db>?<opts>"
```

Any invalid or misconfigured `tx_index` configuration should yield an error as
early as possible.

## Future Improvements

Although not technically required to maintain feature parity with the current
existing Tendermint indexer, it would be beneficial for operators to have a method
of performing a "re-index". Specifically, Tendermint operators could invoke an
RPC method that allows the Tendermint node to perform a re-indexing of all block
and transaction events between two given heights, H<sub>1</sub> and H<sub>2</sub>,
so long as the block store contains the blocks and transaction results for all
the heights specified in a given range.

## Consequences

### Positive

- A more robust and flexible indexing and query engine for indexing and search
  block and transaction events.
- The ability to not have to support a custom indexing and query engine beyond
  the legacy `kv` type.
- The ability to offload/proxy indexing and querying to the underling sink.
- Scalability and reliability that essentially comes "for free" from the underlying
  sink, if it supports it.

### Negative

- The need to support multiple and potentially a growing set of custom `EventSink`
  types.

### Neutral

## References

- [Cosmos SDK ADR-038](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-038-state-listening.md)
- [PostgreSQL](https://www.postgresql.org/)
- [SQLite](https://www.sqlite.org/index.html)
