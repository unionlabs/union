# ADR 070: Unordered Transactions

## Changelog

* Dec 4, 2023: Initial Draft (@yihuang, @tac0turtle, @alexanderbez)
* Jan 30, 2024: Include section on deterministic transaction encoding

## Status

ACCEPTED

## Abstract

We propose a way to do replay-attack protection without enforcing the order of
transactions, without requiring the use of nonces. In this way, we can support
un-ordered transaction inclusion.

## Context

As of today, the nonce value (account sequence number) prevents replay-attack and
ensures the transactions from the same sender are included into blocks and executed
in sequential order. However it makes it tricky to send many transactions from the
same sender concurrently in a reliable way. IBC relayer and crypto exchanges are
typical examples of such use cases.

## Decision

We propose to add a boolean field `unordered` to transaction body to mark "un-ordered"
transactions.

Un-ordered transactions will bypass the nonce rules and follow the rules described
below instead, in contrary, the default ordered transactions are not impacted by
this proposal, they'll follow the nonce rules the same as before.

When an un-ordered transaction is included into a block, the transaction hash is
recorded in a dictionary. New transactions are checked against this dictionary for
duplicates, and to prevent the dictionary grow indefinitely, the transaction must
specify `timeout_timestamp` for expiration, so it's safe to removed it from the
dictionary after it's expired.

The dictionary can be simply implemented as an in-memory golang map, a preliminary
analysis shows that the memory consumption won't be too big, for example `32M = 32 * 1024 * 1024`
can support 1024 blocks where each block contains 1024 unordered transactions. For
safety, we should limit the range of `timeout_timestamp` to prevent very long expiration,
and limit the size of the dictionary.

### Transaction Format

```protobuf
message TxBody {
  ...

  bool unordered = 4;
}
```

### Replay Protection

In order to provide replay protection, a user should ensure that the transaction's
TTL value is relatively short-lived but long enough to provide enough time to be
included in a block, e.g. ~10 minutes.

We facilitate this by storing the transaction's hash in a durable map, `UnorderedTxManager`,
to prevent duplicates, i.e. replay attacks. Upon transaction ingress during `CheckTx`,
we check if the transaction's hash exists in this map or if the TTL value is stale,
i.e. before the current block time. If so, we reject it. Upon inclusion in a block
during `DeliverTx`, the transaction's hash is set in the map along with it's TTL
value.

This map is evaluated at the end of each block, e.g. ABCI `Commit`, and all stale
transactions, i.e. transactions's TTL value who's now beyond the committed block,
are purged from the map.

An important point to note is that in theory, it may be possible to submit an unordered
transaction twice, or multiple times, before the transaction is included in a block.
However, we'll note a few important layers of protection and mitigation:

* Assuming CometBFT is used as the underlying consensus engine and a non-noop mempool
  is used, CometBFT will reject the duplicate for you.
* For applications that leverage ABCI++, `ProcessProposal` should evaluate and reject
  malicious proposals with duplicate transactions.
* For applications that leverage their own application mempool, their mempool should
  reject the duplicate for you.
* Finally, worst case if the duplicate transaction is somehow selected for a block
  proposal, 2nd and all further attempts to evaluate it, will fail during `DeliverTx`,
  so worst case you just end up filling up block space with a duplicate transaction.

```golang
type TxHash [32]byte

const PurgeLoopSleepMS = 500

// UnorderedTxManager contains the tx hash dictionary for duplicates checking,
// and expire them when block production progresses.
type UnorderedTxManager struct {
  // blockCh defines a channel to receive newly committed block time
  blockCh chan time.Time

  mu sync.RWMutex
	// txHashes defines a map from tx hash -> TTL value, which is used for duplicate
	// checking and replay protection, as well as purging the map when the TTL is
	// expired.
	txHashes map[TxHash]time.Time
}

func NewUnorderedTxManager() *UnorderedTxManager {
  m := &UnorderedTxManager{
		blockCh:  make(chan time.Time, 16),
		txHashes: make(map[TxHash]time.Time),
  }

 return m
}

func (m *UnorderedTxManager) Start() {
  go m.purgeLoop()
}

func (m *UnorderedTxManager) Close() error {
  close(m.blockCh)
  m.blockCh = nil
  return nil
}

func (m *UnorderedTxManager) Contains(hash TxHash)  bool{
  m.mu.RLock()
  defer m.mu.RUnlock()

  _, ok := m.txHashes[hash]
  return ok
}

func (m *UnorderedTxManager) Size() int {
  m.mu.RLock()
  defer m.mu.RUnlock()

  return len(m.txHashes)
}

func (m *UnorderedTxManager) Add(hash TxHash, expire time.Time) {
  m.mu.Lock()
  defer m.mu.Unlock()

  m.txHashes[hash] = expire
}

// OnNewBlock send the latest block time to the background purge loop, which
// should be called in ABCI Commit event.
func (m *UnorderedTxManager) OnNewBlock(blockTime time.Time) {
  m.blockCh <- blockTime
}

// expiredTxs returns expired tx hashes based on the provided block time.
func (m *UnorderedTxManager) expiredTxs(blockTime time.Time) []TxHash {
  m.mu.RLock()
  defer m.mu.RUnlock()

  var result []TxHash
  for txHash, expire := range m.txHashes {
    if blockTime.After(expire) {
      result = append(result, txHash)
    }
  }

  return result
}

func (m *UnorderedTxManager) purge(txHashes []TxHash) {
  m.mu.Lock()
  defer m.mu.Unlock()

  for _, txHash := range txHashes {
    delete(m.txHashes, txHash)
  }
}


// purgeLoop removes expired tx hashes in the background
func (m *UnorderedTxManager) purgeLoop() error {
  for {
		latestTime, ok := m.batchReceive()
		if !ok {
			// channel closed
			return
		}

		hashes := m.expiredTxs(latestTime)
		if len(hashes) > 0 {
			m.purge(hashes)
		}
	}
}


// channelBatchRecv try to exhaust the channel buffer when it's not empty,
// and block when it's empty.
func channelBatchRecv[T any](ch <-chan *T) []*T {
	item := <-ch  // block if channel is empty
	if item == nil {
		// channel is closed
		return nil
	}

	remaining := len(ch)
	result := make([]*T, 0, remaining+1)
	result = append(result, item)
	for i := 0; i < remaining; i++ {
		result = append(result, <-ch)
	}

	return result
}
```

### AnteHandler Decorator

In order to facilitate bypassing nonce verification, we have to modify the existing
`IncrementSequenceDecorator` AnteHandler decorator to skip the nonce verification
when the transaction is marked as un-ordered.

```golang
func (isd IncrementSequenceDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
  if tx.UnOrdered() {
    return next(ctx, tx, simulate)
  }

  // ...
}
```

In addition, we need to introduce a new decorator to perform the un-ordered transaction
verification and map lookup.

```golang
const (
	// DefaultMaxTimeoutDuration defines the default maximum duration an un-ordered transaction
	// can set.
	DefaultMaxTimeoutDuration = time.Minute * 40
)

type DedupTxDecorator struct {
  m *UnorderedTxManager
  maxTimeoutDuration time.Time
}

func (d *DedupTxDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
  // only apply to un-ordered transactions
  if !tx.UnOrdered() {
    return next(ctx, tx, simulate)
  }

  headerInfo := d.env.HeaderService.HeaderInfo(ctx)
	timeoutTimestamp := unorderedTx.GetTimeoutTimeStamp()
	if timeoutTimestamp.IsZero() || timeoutTimestamp.Unix() == 0 {
		return ctx, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unordered transaction must have timeout_timestamp set")
	}
	if timeoutTimestamp.Before(headerInfo.Time) {
		return ctx, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "unordered transaction has a timeout_timestamp that has already passed")
	}
	if timeoutTimestamp.After(headerInfo.Time.Add(d.maxTimeoutDuration)) {
		return ctx, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "unordered tx ttl exceeds %s", d.maxTimeoutDuration.String())
	}

  	// in order to create a deterministic hash based on the tx, we need to hash the contents of the tx with signature
	// Get a Buffer from the pool
	buf := bufPool.Get().(*bytes.Buffer)
	// Make sure to reset the buffer
	buf.Reset()

	// Use the buffer
	for _, msg := range tx.GetMsgs() {
		// loop through the messages and write them to the buffer
		// encoding the msg to bytes makes it deterministic within the state machine.
		// Malleability is not a concern here because the state machine will encode the transaction deterministically.
		bz, err := proto.Marshal(msg)
		if err != nil {
			return ctx, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "failed to marshal message")
		}

		buf.Write(bz)
	}

  // check for duplicates
 	// check for duplicates
	if d.txManager.Contains(txHash) {
		return ctx, errorsmod.Wrap(sdkerrors.ErrInvalidRequest, "tx %X is duplicated")
	}

	if d.env.TransactionService.ExecMode(ctx) == transaction.ExecModeFinalize {
		// a new tx included in the block, add the hash to the unordered tx manager
		d.txManager.Add(txHash, ttl)
	}

  return next(ctx, tx, simulate)
}
```

### Transaction Hashes

It is absolutely vital that transaction hashes are deterministic, i.e. transaction
encoding is not malleable. If a given transaction, which is otherwise valid, can
be encoded to produce different hashes, which reflect the same valid transaction,
then a duplicate unordered transaction can be submitted and included in a block.

In order to prevent this, the decoded transaction contents is taken. Starting with the content of the transaction we marshal the transaction in order to prevent a client reordering the transaction. Next we include the gas and timeout timestamp as part of the identifier. All these fields are signed over in the transaction payload. If one of them changes the signature will not match the transaction. 

### State Management

On start up, the node needs to ensure the TxManager's state contains all un-expired
transactions that have been committed to the chain. This is critical since if the
state is not properly initialized, the node will not reject duplicate transactions
and thus will not provide replay protection, and will likely get an app hash mismatch error.

We propose to write all un-expired unordered transactions from the TxManager's to
file on disk. On start up, the node will read this file and re-populate the TxManager's
map. The write to file will happen when the node gracefully shuts down on `Close()`.

Note, this is not a perfect solution, in the context of store v1. With store v2,
we can omit explicit file handling altogether and simply write the all the transactions
to non-consensus state, i.e State Storage (SS).

Alternatively, we can write all the transactions to consensus state.

## Consequences

### Positive

* Support un-ordered and concurrent transaction inclusion.

### Negative

* Requires additional storage overhead and management of processed unordered
  transactions that exist outside of consensus state.

## References

* https://github.com/cosmos/cosmos-sdk/issues/13009
