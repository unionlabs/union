// Deprecated: Priority mempool will be removed in the next major release.
package v1

import (
	"fmt"
	"runtime"
	"sort"
	"sync"
	"sync/atomic"
	"time"

	"github.com/creachadair/taskgroup"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/clist"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/mempool"
	"github.com/cometbft/cometbft/proxy"
	"github.com/cometbft/cometbft/types"
)

var _ mempool.Mempool = (*TxMempool)(nil)

// TxMempoolOption sets an optional parameter on the TxMempool.
type TxMempoolOption func(*TxMempool)

// TxMempool implemements the Mempool interface and allows the application to
// set priority values on transactions in the CheckTx response. When selecting
// transactions to include in a block, higher-priority transactions are chosen
// first.  When evicting transactions from the mempool for size constraints,
// lower-priority transactions are evicted sooner.
//
// Within the mempool, transactions are ordered by time of arrival, and are
// gossiped to the rest of the network based on that order (gossip order does
// not take priority into account).
type TxMempool struct {
	// Immutable fields
	logger       log.Logger
	config       *config.MempoolConfig
	proxyAppConn proxy.AppConnMempool
	metrics      *mempool.Metrics
	cache        mempool.TxCache // seen transactions

	// Atomically-updated fields
	txsBytes int64 // atomic: the total size of all transactions in the mempool, in bytes

	// Synchronized fields, protected by mtx.
	mtx                  *sync.RWMutex
	notifiedTxsAvailable bool
	txsAvailable         chan struct{} // one value sent per height when mempool is not empty
	preCheck             mempool.PreCheckFunc
	postCheck            mempool.PostCheckFunc
	height               int64 // the latest height passed to Update

	txs        *clist.CList // valid transactions (passed CheckTx)
	txByKey    map[types.TxKey]*clist.CElement
	txBySender map[string]*clist.CElement // for sender != ""
}

// NewTxMempool constructs a new, empty priority mempool at the specified
// initial height and using the given config and options.
func NewTxMempool(
	logger log.Logger,
	cfg *config.MempoolConfig,
	proxyAppConn proxy.AppConnMempool,
	height int64,
	options ...TxMempoolOption,
) *TxMempool {

	txmp := &TxMempool{
		logger:       logger,
		config:       cfg,
		proxyAppConn: proxyAppConn,
		metrics:      mempool.NopMetrics(),
		cache:        mempool.NopTxCache{},
		txs:          clist.New(),
		mtx:          new(sync.RWMutex),
		height:       height,
		txByKey:      make(map[types.TxKey]*clist.CElement),
		txBySender:   make(map[string]*clist.CElement),
	}
	if cfg.CacheSize > 0 {
		txmp.cache = mempool.NewLRUTxCache(cfg.CacheSize)
	}

	for _, opt := range options {
		opt(txmp)
	}

	return txmp
}

// WithPreCheck sets a filter for the mempool to reject a transaction if f(tx)
// returns an error. This is executed before CheckTx. It only applies to the
// first created block. After that, Update() overwrites the existing value.
func WithPreCheck(f mempool.PreCheckFunc) TxMempoolOption {
	return func(txmp *TxMempool) { txmp.preCheck = f }
}

// WithPostCheck sets a filter for the mempool to reject a transaction if
// f(tx, resp) returns an error. This is executed after CheckTx. It only applies
// to the first created block. After that, Update overwrites the existing value.
func WithPostCheck(f mempool.PostCheckFunc) TxMempoolOption {
	return func(txmp *TxMempool) { txmp.postCheck = f }
}

// WithMetrics sets the mempool's metrics collector.
func WithMetrics(metrics *mempool.Metrics) TxMempoolOption {
	return func(txmp *TxMempool) { txmp.metrics = metrics }
}

// Lock obtains a write-lock on the mempool. A caller must be sure to explicitly
// release the lock when finished.
func (txmp *TxMempool) Lock() { txmp.mtx.Lock() }

// Unlock releases a write-lock on the mempool.
func (txmp *TxMempool) Unlock() { txmp.mtx.Unlock() }

// Size returns the number of valid transactions in the mempool. It is
// thread-safe.
func (txmp *TxMempool) Size() int { return txmp.txs.Len() }

// SizeBytes return the total sum in bytes of all the valid transactions in the
// mempool. It is thread-safe.
func (txmp *TxMempool) SizeBytes() int64 { return atomic.LoadInt64(&txmp.txsBytes) }

// FlushAppConn executes FlushSync on the mempool's proxyAppConn.
//
// The caller must hold an exclusive mempool lock (by calling txmp.Lock) before
// calling FlushAppConn.
func (txmp *TxMempool) FlushAppConn() error {
	// N.B.: We have to issue the call outside the lock so that its callback can
	// fire.  It's safe to do this, the flush will block until complete.
	//
	// We could just not require the caller to hold the lock at all, but the
	// semantics of the Mempool interface require the caller to hold it, and we
	// can't change that without disrupting existing use.
	txmp.mtx.Unlock()
	defer txmp.mtx.Lock()

	return txmp.proxyAppConn.FlushSync()
}

// EnableTxsAvailable enables the mempool to trigger events when transactions
// are available on a block by block basis.
func (txmp *TxMempool) EnableTxsAvailable() {
	txmp.mtx.Lock()
	defer txmp.mtx.Unlock()

	txmp.txsAvailable = make(chan struct{}, 1)
}

// TxsAvailable returns a channel which fires once for every height, and only
// when transactions are available in the mempool. It is thread-safe.
func (txmp *TxMempool) TxsAvailable() <-chan struct{} { return txmp.txsAvailable }

// CheckTx adds the given transaction to the mempool if it fits and passes the
// application's ABCI CheckTx method.
//
// CheckTx reports an error without adding tx if:
//
// - The size of tx exceeds the configured maximum transaction size.
// - The pre-check hook is defined and reports an error for tx.
// - The transaction already exists in the cache.
// - The proxy connection to the application fails.
//
// If tx passes all of the above conditions, it is passed (asynchronously) to
// the application's ABCI CheckTx method and this CheckTx method returns nil.
// If cb != nil, it is called when the ABCI request completes to report the
// application response.
//
// If the application accepts the transaction and the mempool is full, the
// mempool evicts one or more of the lowest-priority transaction whose priority
// is (strictly) lower than the priority of tx and whose size together exceeds
// the size of tx, and adds tx instead. If no such transactions exist, tx is
// discarded.
func (txmp *TxMempool) CheckTx(tx types.Tx, cb func(*abci.Response), txInfo mempool.TxInfo) error {

	// During the initial phase of CheckTx, we do not need to modify any state.
	// A transaction will not actually be added to the mempool until it survives
	// a call to the ABCI CheckTx method and size constraint checks.
	height, err := func() (int64, error) {
		txmp.mtx.RLock()
		defer txmp.mtx.RUnlock()

		// Reject transactions in excess of the configured maximum transaction size.
		if len(tx) > txmp.config.MaxTxBytes {
			return 0, mempool.ErrTxTooLarge{Max: txmp.config.MaxTxBytes, Actual: len(tx)}
		}

		// If a precheck hook is defined, call it before invoking the application.
		if txmp.preCheck != nil {
			if err := txmp.preCheck(tx); err != nil {
				return 0, mempool.ErrPreCheck{Reason: err}
			}
		}

		// Early exit if the proxy connection has an error.
		if err := txmp.proxyAppConn.Error(); err != nil {
			return 0, err
		}

		txKey := tx.Key()

		// Check for the transaction in the cache.
		if !txmp.cache.Push(tx) {
			// If the cached transaction is also in the pool, record its sender.
			if elt, ok := txmp.txByKey[txKey]; ok {
				w := elt.Value.(*WrappedTx)
				w.SetPeer(txInfo.SenderID)
			}
			return 0, mempool.ErrTxInCache
		}
		return txmp.height, nil
	}()
	if err != nil {
		return err
	}

	// Invoke an ABCI CheckTx for this transaction.
	rsp, err := txmp.proxyAppConn.CheckTxSync(abci.RequestCheckTx{Tx: tx})
	if err != nil {
		txmp.cache.Remove(tx)
		return err
	}
	wtx := &WrappedTx{
		tx:        tx,
		hash:      tx.Key(),
		timestamp: time.Now().UTC(),
		height:    height,
	}
	wtx.SetPeer(txInfo.SenderID)
	txmp.addNewTransaction(wtx, rsp)
	if cb != nil {
		cb(&abci.Response{Value: &abci.Response_CheckTx{CheckTx: rsp}})
	}
	return nil
}

// RemoveTxByKey removes the transaction with the specified key from the
// mempool. It reports an error if no such transaction exists.  This operation
// does not remove the transaction from the cache.
func (txmp *TxMempool) RemoveTxByKey(txKey types.TxKey) error {
	txmp.mtx.Lock()
	defer txmp.mtx.Unlock()
	return txmp.removeTxByKey(txKey)
}

// removeTxByKey removes the specified transaction key from the mempool.
// The caller must hold txmp.mtx excluxively.
func (txmp *TxMempool) removeTxByKey(key types.TxKey) error {
	if elt, ok := txmp.txByKey[key]; ok {
		w := elt.Value.(*WrappedTx)
		delete(txmp.txByKey, key)
		delete(txmp.txBySender, w.sender)
		txmp.txs.Remove(elt)
		elt.DetachPrev()
		elt.DetachNext()
		atomic.AddInt64(&txmp.txsBytes, -w.Size())
		return nil
	}
	return fmt.Errorf("transaction %x not found", key)
}

// removeTxByElement removes the specified transaction element from the mempool.
// The caller must hold txmp.mtx exclusively.
func (txmp *TxMempool) removeTxByElement(elt *clist.CElement) {
	w := elt.Value.(*WrappedTx)
	delete(txmp.txByKey, w.tx.Key())
	delete(txmp.txBySender, w.sender)
	txmp.txs.Remove(elt)
	elt.DetachPrev()
	elt.DetachNext()
	atomic.AddInt64(&txmp.txsBytes, -w.Size())
}

// Flush purges the contents of the mempool and the cache, leaving both empty.
// The current height is not modified by this operation.
func (txmp *TxMempool) Flush() {
	txmp.mtx.Lock()
	defer txmp.mtx.Unlock()

	// Remove all the transactions in the list explicitly, so that the sizes
	// and indexes get updated properly.
	cur := txmp.txs.Front()
	for cur != nil {
		next := cur.Next()
		txmp.removeTxByElement(cur)
		cur = next
	}
	txmp.cache.Reset()
}

// allEntriesSorted returns a slice of all the transactions currently in the
// mempool, sorted in nonincreasing order by priority with ties broken by
// increasing order of arrival time.
func (txmp *TxMempool) allEntriesSorted() []*WrappedTx {
	txmp.mtx.RLock()
	defer txmp.mtx.RUnlock()

	all := make([]*WrappedTx, 0, len(txmp.txByKey))
	for _, tx := range txmp.txByKey {
		all = append(all, tx.Value.(*WrappedTx))
	}
	sort.Slice(all, func(i, j int) bool {
		if all[i].priority == all[j].priority {
			return all[i].timestamp.Before(all[j].timestamp)
		}
		return all[i].priority > all[j].priority // N.B. higher priorities first
	})
	return all
}

// ReapMaxBytesMaxGas returns a slice of valid transactions that fit within the
// size and gas constraints. The results are ordered by nonincreasing priority,
// with ties broken by increasing order of arrival.  Reaping transactions does
// not remove them from the mempool.
//
// If maxBytes < 0, no limit is set on the total size in bytes.
// If maxGas < 0, no limit is set on the total gas cost.
//
// If the mempool is empty or has no transactions fitting within the given
// constraints, the result will also be empty.
func (txmp *TxMempool) ReapMaxBytesMaxGas(maxBytes, maxGas int64) types.Txs {
	var totalGas, totalBytes int64

	var keep []types.Tx //nolint:prealloc
	for _, w := range txmp.allEntriesSorted() {
		// N.B. When computing byte size, we need to include the overhead for
		// encoding as protobuf to send to the application.
		totalGas += w.gasWanted
		totalBytes += types.ComputeProtoSizeForTxs([]types.Tx{w.tx})
		if (maxGas >= 0 && totalGas > maxGas) || (maxBytes >= 0 && totalBytes > maxBytes) {
			break
		}
		keep = append(keep, w.tx)
	}
	return keep
}

// TxsWaitChan returns a channel that is closed when there is at least one
// transaction available to be gossiped.
func (txmp *TxMempool) TxsWaitChan() <-chan struct{} { return txmp.txs.WaitChan() }

// TxsFront returns the frontmost element of the pending transaction list.
// It will be nil if the mempool is empty.
func (txmp *TxMempool) TxsFront() *clist.CElement { return txmp.txs.Front() }

// ReapMaxTxs returns up to max transactions from the mempool. The results are
// ordered by nonincreasing priority with ties broken by increasing order of
// arrival. Reaping transactions does not remove them from the mempool.
//
// If max < 0, all transactions in the mempool are reaped.
//
// The result may have fewer than max elements (possibly zero) if the mempool
// does not have that many transactions available.
func (txmp *TxMempool) ReapMaxTxs(max int) types.Txs {
	var keep []types.Tx //nolint:prealloc

	for _, w := range txmp.allEntriesSorted() {
		if max >= 0 && len(keep) >= max {
			break
		}
		keep = append(keep, w.tx)
	}
	return keep
}

// Update removes all the given transactions from the mempool and the cache,
// and updates the current block height. The blockTxs and deliverTxResponses
// must have the same length with each response corresponding to the tx at the
// same offset.
//
// If the configuration enables recheck, Update sends each remaining
// transaction after removing blockTxs to the ABCI CheckTx method.  Any
// transactions marked as invalid during recheck are also removed.
//
// The caller must hold an exclusive mempool lock (by calling txmp.Lock) before
// calling Update.
func (txmp *TxMempool) Update(
	blockHeight int64,
	blockTxs types.Txs,
	deliverTxResponses []*abci.ResponseDeliverTx,
	newPreFn mempool.PreCheckFunc,
	newPostFn mempool.PostCheckFunc,
) error {
	// Safety check: Transactions and responses must match in number.
	if len(blockTxs) != len(deliverTxResponses) {
		panic(fmt.Sprintf("mempool: got %d transactions but %d DeliverTx responses",
			len(blockTxs), len(deliverTxResponses)))
	}

	txmp.height = blockHeight
	txmp.notifiedTxsAvailable = false

	if newPreFn != nil {
		txmp.preCheck = newPreFn
	}
	if newPostFn != nil {
		txmp.postCheck = newPostFn
	}

	for i, tx := range blockTxs {
		// Add successful committed transactions to the cache (if they are not
		// already present).  Transactions that failed to commit are removed from
		// the cache unless the operator has explicitly requested we keep them.
		if deliverTxResponses[i].Code == abci.CodeTypeOK {
			_ = txmp.cache.Push(tx)
		} else if !txmp.config.KeepInvalidTxsInCache {
			txmp.cache.Remove(tx)
		}

		// Regardless of success, remove the transaction from the mempool.
		_ = txmp.removeTxByKey(tx.Key())
	}

	txmp.purgeExpiredTxs(blockHeight)

	// If there any uncommitted transactions left in the mempool, we either
	// initiate re-CheckTx per remaining transaction or notify that remaining
	// transactions are left.
	size := txmp.Size()
	txmp.metrics.Size.Set(float64(size))
	if size > 0 {
		if txmp.config.Recheck {
			txmp.recheckTransactions()
		} else {
			txmp.notifyTxsAvailable()
		}
	}
	return nil
}

// addNewTransaction handles the ABCI CheckTx response for the first time a
// transaction is added to the mempool.  A recheck after a block is committed
// goes to handleRecheckResult.
//
// If either the application rejected the transaction or a post-check hook is
// defined and rejects the transaction, it is discarded.
//
// Otherwise, if the mempool is full, check for lower-priority transactions
// that can be evicted to make room for the new one. If no such transactions
// exist, this transaction is logged and dropped; otherwise the selected
// transactions are evicted.
//
// Finally, the new transaction is added and size stats updated.
func (txmp *TxMempool) addNewTransaction(wtx *WrappedTx, checkTxRes *abci.ResponseCheckTx) {
	txmp.mtx.Lock()
	defer txmp.mtx.Unlock()

	var err error
	if txmp.postCheck != nil {
		err = txmp.postCheck(wtx.tx, checkTxRes)
	}

	if err != nil || checkTxRes.Code != abci.CodeTypeOK {
		txmp.logger.Debug(
			"rejected bad transaction",
			"priority", wtx.Priority(),
			"tx", fmt.Sprintf("%X", wtx.tx.Hash()),
			"peer_id", wtx.peers,
			"code", checkTxRes.Code,
			"post_check_err", err,
		)

		txmp.metrics.FailedTxs.Add(1)

		// Remove the invalid transaction from the cache, unless the operator has
		// instructed us to keep invalid transactions.
		if !txmp.config.KeepInvalidTxsInCache {
			txmp.cache.Remove(wtx.tx)
		}

		// If there was a post-check error, record its text in the result for
		// debugging purposes.
		if err != nil {
			checkTxRes.MempoolError = err.Error()
		}
		return
	}

	priority := checkTxRes.Priority
	sender := checkTxRes.Sender

	// Disallow multiple concurrent transactions from the same sender assigned
	// by the ABCI application. As a special case, an empty sender is not
	// restricted.
	if sender != "" {
		elt, ok := txmp.txBySender[sender]
		if ok {
			w := elt.Value.(*WrappedTx)
			txmp.logger.Debug(
				"rejected valid incoming transaction; tx already exists for sender",
				"tx", fmt.Sprintf("%X", w.tx.Hash()),
				"sender", sender,
			)
			checkTxRes.MempoolError =
				fmt.Sprintf("rejected valid incoming transaction; tx already exists for sender %q (%X)",
					sender, w.tx.Hash())
			txmp.metrics.RejectedTxs.Add(1)
			return
		}
	}

	// At this point the application has ruled the transaction valid, but the
	// mempool might be full. If so, find the lowest-priority items with lower
	// priority than the application assigned to this new one, and evict as many
	// of them as necessary to make room for tx. If no such items exist, we
	// discard tx.

	if err := txmp.canAddTx(wtx); err != nil {
		var victims []*clist.CElement // eligible transactions for eviction
		var victimBytes int64         // total size of victims
		for cur := txmp.txs.Front(); cur != nil; cur = cur.Next() {
			cw := cur.Value.(*WrappedTx)
			if cw.priority < priority {
				victims = append(victims, cur)
				victimBytes += cw.Size()
			}
		}

		// If there are no suitable eviction candidates, or the total size of
		// those candidates is not enough to make room for the new transaction,
		// drop the new one.
		if len(victims) == 0 || victimBytes < wtx.Size() {
			txmp.cache.Remove(wtx.tx)
			txmp.logger.Error(
				"rejected valid incoming transaction; mempool is full",
				"tx", fmt.Sprintf("%X", wtx.tx.Hash()),
				"err", err.Error(),
			)
			checkTxRes.MempoolError =
				fmt.Sprintf("rejected valid incoming transaction; mempool is full (%X)",
					wtx.tx.Hash())
			txmp.metrics.RejectedTxs.Add(1)
			return
		}

		txmp.logger.Debug("evicting lower-priority transactions",
			"new_tx", fmt.Sprintf("%X", wtx.tx.Hash()),
			"new_priority", priority,
		)

		// Sort lowest priority items first so they will be evicted first.  Break
		// ties in favor of newer items (to maintain FIFO semantics in a group).
		sort.Slice(victims, func(i, j int) bool {
			iw := victims[i].Value.(*WrappedTx)
			jw := victims[j].Value.(*WrappedTx)
			if iw.Priority() == jw.Priority() {
				return iw.timestamp.After(jw.timestamp)
			}
			return iw.Priority() < jw.Priority()
		})

		// Evict as many of the victims as necessary to make room.
		var evictedBytes int64
		for _, vic := range victims {
			w := vic.Value.(*WrappedTx)

			txmp.logger.Debug(
				"evicted valid existing transaction; mempool full",
				"old_tx", fmt.Sprintf("%X", w.tx.Hash()),
				"old_priority", w.priority,
			)
			txmp.removeTxByElement(vic)
			txmp.cache.Remove(w.tx)
			txmp.metrics.EvictedTxs.Add(1)

			// We may not need to evict all the eligible transactions.  Bail out
			// early if we have made enough room.
			evictedBytes += w.Size()
			if evictedBytes >= wtx.Size() {
				break
			}
		}
	}

	wtx.SetGasWanted(checkTxRes.GasWanted)
	wtx.SetPriority(priority)
	wtx.SetSender(sender)
	txmp.insertTx(wtx)

	txmp.metrics.TxSizeBytes.Observe(float64(wtx.Size()))
	txmp.metrics.Size.Set(float64(txmp.Size()))
	txmp.logger.Debug(
		"inserted new valid transaction",
		"priority", wtx.Priority(),
		"tx", fmt.Sprintf("%X", wtx.tx.Hash()),
		"height", txmp.height,
		"num_txs", txmp.Size(),
	)
	txmp.notifyTxsAvailable()
}

func (txmp *TxMempool) insertTx(wtx *WrappedTx) {
	elt := txmp.txs.PushBack(wtx)
	txmp.txByKey[wtx.tx.Key()] = elt
	if s := wtx.Sender(); s != "" {
		txmp.txBySender[s] = elt
	}

	atomic.AddInt64(&txmp.txsBytes, wtx.Size())
}

// handleRecheckResult handles the responses from ABCI CheckTx calls issued
// during the recheck phase of a block Update.  It removes any transactions
// invalidated by the application.
//
// This method is NOT executed for the initial CheckTx on a new transaction;
// that case is handled by addNewTransaction instead.
func (txmp *TxMempool) handleRecheckResult(tx types.Tx, checkTxRes *abci.ResponseCheckTx) {
	txmp.metrics.RecheckTimes.Add(1)
	txmp.mtx.Lock()
	defer txmp.mtx.Unlock()

	// Find the transaction reported by the ABCI callback. It is possible the
	// transaction was evicted during the recheck, in which case the transaction
	// will be gone.
	elt, ok := txmp.txByKey[tx.Key()]
	if !ok {
		return
	}
	wtx := elt.Value.(*WrappedTx)

	// If a postcheck hook is defined, call it before checking the result.
	var err error
	if txmp.postCheck != nil {
		err = txmp.postCheck(tx, checkTxRes)
	}

	if checkTxRes.Code == abci.CodeTypeOK && err == nil {
		wtx.SetPriority(checkTxRes.Priority)
		return // N.B. Size of mempool did not change
	}

	txmp.logger.Debug(
		"existing transaction no longer valid; failed re-CheckTx callback",
		"priority", wtx.Priority(),
		"tx", fmt.Sprintf("%X", wtx.tx.Hash()),
		"err", err,
		"code", checkTxRes.Code,
	)
	txmp.removeTxByElement(elt)
	txmp.metrics.FailedTxs.Add(1)
	if !txmp.config.KeepInvalidTxsInCache {
		txmp.cache.Remove(wtx.tx)
	}
	txmp.metrics.Size.Set(float64(txmp.Size()))
}

// recheckTransactions initiates re-CheckTx ABCI calls for all the transactions
// currently in the mempool. It reports the number of recheck calls that were
// successfully initiated.
//
// Precondition: The mempool is not empty.
// The caller must hold txmp.mtx exclusively.
func (txmp *TxMempool) recheckTransactions() {
	if txmp.Size() == 0 {
		panic("mempool: cannot run recheck on an empty mempool")
	}
	txmp.logger.Debug(
		"executing re-CheckTx for all remaining transactions",
		"num_txs", txmp.Size(),
		"height", txmp.height,
	)

	// Collect transactions currently in the mempool requiring recheck.
	wtxs := make([]*WrappedTx, 0, txmp.txs.Len())
	for e := txmp.txs.Front(); e != nil; e = e.Next() {
		wtxs = append(wtxs, e.Value.(*WrappedTx))
	}

	// Issue CheckTx calls for each remaining transaction, and when all the
	// rechecks are complete signal watchers that transactions may be available.
	go func() {
		g, start := taskgroup.New(nil).Limit(2 * runtime.NumCPU())

		for _, wtx := range wtxs {
			wtx := wtx
			start(func() error {
				// The response for this CheckTx is handled by the default recheckTxCallback.
				rsp, err := txmp.proxyAppConn.CheckTxSync(abci.RequestCheckTx{
					Tx:   wtx.tx,
					Type: abci.CheckTxType_Recheck,
				})
				if err != nil {
					txmp.logger.Error("failed to execute CheckTx during recheck",
						"err", err, "hash", fmt.Sprintf("%x", wtx.tx.Hash()))
				} else {
					txmp.handleRecheckResult(wtx.tx, rsp)
				}
				return nil
			})
		}
		_ = txmp.proxyAppConn.FlushAsync()

		// When recheck is complete, trigger a notification for more transactions.
		_ = g.Wait()
		txmp.mtx.Lock()
		defer txmp.mtx.Unlock()
		txmp.notifyTxsAvailable()
	}()
}

// canAddTx returns an error if we cannot insert the provided *WrappedTx into
// the mempool due to mempool configured constraints. Otherwise, nil is
// returned and the transaction can be inserted into the mempool.
func (txmp *TxMempool) canAddTx(wtx *WrappedTx) error {
	numTxs := txmp.Size()
	txBytes := txmp.SizeBytes()

	if numTxs >= txmp.config.Size || wtx.Size()+txBytes > txmp.config.MaxTxsBytes {
		return mempool.ErrMempoolIsFull{
			NumTxs:      numTxs,
			MaxTxs:      txmp.config.Size,
			TxsBytes:    txBytes,
			MaxTxsBytes: txmp.config.MaxTxsBytes,
		}
	}

	return nil
}

// purgeExpiredTxs removes all transactions from the mempool that have exceeded
// their respective height or time-based limits as of the given blockHeight.
// Transactions removed by this operation are not removed from the cache.
//
// The caller must hold txmp.mtx exclusively.
func (txmp *TxMempool) purgeExpiredTxs(blockHeight int64) {
	if txmp.config.TTLNumBlocks == 0 && txmp.config.TTLDuration == 0 { //nolint:staticcheck // SA1019 Priority mempool deprecated but still supported in this release.
		return // nothing to do
	}

	now := time.Now()
	cur := txmp.txs.Front()
	for cur != nil {
		// N.B. Grab the next element first, since if we remove cur its successor
		// will be invalidated.
		next := cur.Next()

		w := cur.Value.(*WrappedTx)
		if txmp.config.TTLNumBlocks > 0 && (blockHeight-w.height) > txmp.config.TTLNumBlocks { //nolint:staticcheck // SA1019 Priority mempool deprecated but still supported in this release.
			txmp.removeTxByElement(cur)
			txmp.cache.Remove(w.tx)
			txmp.metrics.EvictedTxs.Add(1)
		} else if txmp.config.TTLDuration > 0 && now.Sub(w.timestamp) > txmp.config.TTLDuration { //nolint:staticcheck // SA1019 Priority mempool deprecated but still supported in this release.
			txmp.removeTxByElement(cur)
			txmp.cache.Remove(w.tx)
			txmp.metrics.EvictedTxs.Add(1)
		}
		cur = next
	}
}

func (txmp *TxMempool) notifyTxsAvailable() {
	if txmp.Size() == 0 {
		return // nothing to do
	}

	if txmp.txsAvailable != nil && !txmp.notifiedTxsAvailable {
		// channel cap is 1, so this will send once
		txmp.notifiedTxsAvailable = true

		select {
		case txmp.txsAvailable <- struct{}{}:
		default:
		}
	}
}
