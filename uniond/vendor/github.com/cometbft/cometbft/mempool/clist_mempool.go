package mempool

import (
	"bytes"
	"context"
	"fmt"
	"sync"
	"sync/atomic"
	"time"

	abcicli "github.com/cometbft/cometbft/abci/client"
	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/internal/clist"
	"github.com/cometbft/cometbft/libs/log"
	cmtmath "github.com/cometbft/cometbft/libs/math"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/proxy"
	"github.com/cometbft/cometbft/types"
)

// CListMempool is an ordered in-memory pool for transactions before they are
// proposed in a consensus round. Transaction validity is checked using the
// CheckTx abci message before the transaction is added to the pool. The
// mempool uses a concurrent list structure for storing transactions that can
// be efficiently accessed by multiple concurrent readers.
type CListMempool struct {
	height   atomic.Int64 // the last block Update()'d to
	txsBytes atomic.Int64 // total size of mempool, in bytes

	// notify listeners (ie. consensus) when txs are available
	notifiedTxsAvailable atomic.Bool
	txsAvailable         chan struct{} // fires once for each height, when the mempool is not empty

	config *config.MempoolConfig

	// Exclusive mutex for Update method to prevent concurrent execution of
	// CheckTx or ReapMaxBytesMaxGas(ReapMaxTxs) methods.
	updateMtx cmtsync.RWMutex
	preCheck  PreCheckFunc
	postCheck PostCheckFunc

	proxyAppConn proxy.AppConnMempool

	// Keeps track of the rechecking process.
	recheck *recheck

	// Concurrent linked-list of valid txs.
	// `txsMap`: txKey -> CElement is for quick access to txs.
	// Transactions in both `txs` and `txsMap` must to be kept in sync.
	txs    *clist.CList
	txsMap sync.Map

	// Keep a cache of already-seen txs.
	// This reduces the pressure on the proxyApp.
	cache TxCache

	logger  log.Logger
	metrics *Metrics
}

var _ Mempool = &CListMempool{}

// CListMempoolOption sets an optional parameter on the mempool.
type CListMempoolOption func(*CListMempool)

// NewCListMempool returns a new mempool with the given configuration and
// connection to an application.
func NewCListMempool(
	cfg *config.MempoolConfig,
	proxyAppConn proxy.AppConnMempool,
	height int64,
	options ...CListMempoolOption,
) *CListMempool {
	mp := &CListMempool{
		config:       cfg,
		proxyAppConn: proxyAppConn,
		txs:          clist.New(),
		recheck:      newRecheck(),
		logger:       log.NewNopLogger(),
		metrics:      NopMetrics(),
	}
	mp.height.Store(height)

	if cfg.CacheSize > 0 {
		mp.cache = NewLRUTxCache(cfg.CacheSize)
	} else {
		mp.cache = NopTxCache{}
	}

	for _, option := range options {
		option(mp)
	}

	return mp
}

func (mem *CListMempool) getCElement(txKey types.TxKey) (*clist.CElement, bool) {
	if e, ok := mem.txsMap.Load(txKey); ok {
		return e.(*clist.CElement), true
	}
	return nil, false
}

func (mem *CListMempool) InMempool(txKey types.TxKey) bool {
	_, ok := mem.getCElement(txKey)
	return ok
}

func (mem *CListMempool) addToCache(tx types.Tx) bool {
	return mem.cache.Push(tx)
}

func (mem *CListMempool) forceRemoveFromCache(tx types.Tx) {
	mem.cache.Remove(tx)
}

// tryRemoveFromCache removes a transaction from the cache in case it can be
// added to the mempool at a later stage (probably when the transaction becomes
// valid).
func (mem *CListMempool) tryRemoveFromCache(tx types.Tx) {
	if !mem.config.KeepInvalidTxsInCache {
		mem.forceRemoveFromCache(tx)
	}
}

func (mem *CListMempool) removeAllTxs() {
	for e := mem.txs.Front(); e != nil; e = e.Next() {
		mem.txs.Remove(e)
		e.DetachPrev()
	}

	mem.txsMap.Range(func(key, _ any) bool {
		mem.txsMap.Delete(key)
		return true
	})
}

// NOTE: not thread safe - should only be called once, on startup.
func (mem *CListMempool) EnableTxsAvailable() {
	mem.txsAvailable = make(chan struct{}, 1)
}

// SetLogger sets the Logger.
func (mem *CListMempool) SetLogger(l log.Logger) {
	mem.logger = l
}

// WithPreCheck sets a filter for the mempool to reject a tx if f(tx) returns
// false. This is ran before CheckTx. Only applies to the first created block.
// After that, Update overwrites the existing value.
func WithPreCheck(f PreCheckFunc) CListMempoolOption {
	return func(mem *CListMempool) { mem.preCheck = f }
}

// WithPostCheck sets a filter for the mempool to reject a tx if f(tx) returns
// false. This is ran after CheckTx. Only applies to the first created block.
// After that, Update overwrites the existing value.
func WithPostCheck(f PostCheckFunc) CListMempoolOption {
	return func(mem *CListMempool) { mem.postCheck = f }
}

// WithMetrics sets the metrics.
func WithMetrics(metrics *Metrics) CListMempoolOption {
	return func(mem *CListMempool) { mem.metrics = metrics }
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) Lock() {
	mem.updateMtx.Lock()
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) Unlock() {
	mem.updateMtx.Unlock()
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) PreUpdate() {
	if mem.recheck.setRecheckFull() {
		mem.logger.Debug("The state of recheckFull has flipped")
	}
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) Size() int {
	return mem.txs.Len()
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) SizeBytes() int64 {
	return mem.txsBytes.Load()
}

// Lock() must be help by the caller during execution.
func (mem *CListMempool) FlushAppConn() error {
	err := mem.proxyAppConn.Flush(context.TODO())
	if err != nil {
		return ErrFlushAppConn{Err: err}
	}

	return nil
}

// XXX: Unsafe! Calling Flush may leave mempool in inconsistent state.
func (mem *CListMempool) Flush() {
	mem.updateMtx.Lock()
	defer mem.updateMtx.Unlock()

	mem.txsBytes.Store(0)
	mem.cache.Reset()

	mem.removeAllTxs()
}

// TxsFront returns the first transaction in the ordered list for peer
// goroutines to call .NextWait() on.
// FIXME: leaking implementation details!
//
// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) TxsFront() *clist.CElement {
	return mem.txs.Front()
}

// TxsWaitChan returns a channel to wait on transactions. It will be closed
// once the mempool is not empty (ie. the internal `mem.txs` has at least one
// element)
//
// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) TxsWaitChan() <-chan struct{} {
	return mem.txs.WaitChan()
}

// It blocks if we're waiting on Update() or Reap().
// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) CheckTx(tx types.Tx, sender p2p.ID) (*abcicli.ReqRes, error) {
	mem.updateMtx.RLock()
	// use defer to unlock mutex because application (*local client*) might panic
	defer mem.updateMtx.RUnlock()

	txSize := len(tx)

	if err := mem.isFull(txSize); err != nil {
		return nil, err
	}

	if txSize > mem.config.MaxTxBytes {
		return nil, ErrTxTooLarge{
			Max:    mem.config.MaxTxBytes,
			Actual: txSize,
		}
	}

	if mem.preCheck != nil {
		if err := mem.preCheck(tx); err != nil {
			return nil, ErrPreCheck{Err: err}
		}
	}

	// NOTE: proxyAppConn may error if tx buffer is full
	if err := mem.proxyAppConn.Error(); err != nil {
		return nil, ErrAppConnMempool{Err: err}
	}

	if added := mem.addToCache(tx); !added {
		mem.metrics.AlreadyReceivedTxs.Add(1)
		if sender != "" {
			// Record a new sender for a tx we've already seen.
			// Note it's possible a tx is still in the cache but no longer in the mempool
			// (eg. after committing a block, txs are removed from mempool but not cache),
			// so we only record the sender for txs still in the mempool.
			if elem, ok := mem.getCElement(tx.Key()); ok {
				memTx := elem.Value.(*mempoolTx)
				if found := memTx.addSender(sender); found {
					// It should not be possible to receive twice a tx from the same sender.
					mem.logger.Error("Tx already received from peer", "tx", tx.Hash(), "sender", sender)
				}
			}
		}
		// TODO: consider punishing peer for dups,
		// its non-trivial since invalid txs can become valid,
		// but they can spam the same tx with little cost to them atm.
		return nil, ErrTxInCache
	}

	reqRes, err := mem.proxyAppConn.CheckTxAsync(context.TODO(), &abci.CheckTxRequest{
		Tx:   tx,
		Type: abci.CHECK_TX_TYPE_CHECK,
	})
	if err != nil {
		panic(fmt.Errorf("CheckTx request for tx %s failed: %w", log.NewLazySprintf("%X", tx.Hash()), err))
	}
	reqRes.SetCallback(mem.handleCheckTxResponse(tx, sender))

	return reqRes, nil
}

// handleCheckTxResponse handles CheckTx responses for transactions validated for the first time.
//
//   - sender optionally holds the ID of the peer that sent the transaction, if any.
func (mem *CListMempool) handleCheckTxResponse(tx types.Tx, sender p2p.ID) func(res *abci.Response) {
	return func(r *abci.Response) {
		res := r.GetCheckTx()
		if res == nil {
			panic(fmt.Sprintf("unexpected response value %v not of type CheckTx", r))
		}

		// Check that rechecking txs is not in process.
		if !mem.recheck.done() {
			panic(log.NewLazySprintf("rechecking has not finished; cannot check new tx %X", tx.Hash()))
		}

		var postCheckErr error
		if mem.postCheck != nil {
			postCheckErr = mem.postCheck(tx, res)
		}

		// If tx is invalid, remove it from the cache.
		if res.Code != abci.CodeTypeOK || postCheckErr != nil {
			mem.tryRemoveFromCache(tx)
			mem.logger.Debug(
				"Rejected invalid transaction",
				"tx", tx.Hash(),
				"res", res,
				"err", postCheckErr,
			)
			mem.metrics.FailedTxs.Add(1)
			return
		}

		// Check again that mempool isn't full, to reduce the chance of exceeding the limits.
		if err := mem.isFull(len(tx)); err != nil {
			mem.forceRemoveFromCache(tx) // mempool might have space later
			mem.logger.Error(err.Error())
			return
		}

		// Add tx to mempool and notify that new txs are available.
		memTx := mempoolTx{
			height:    mem.height.Load(),
			gasWanted: res.GasWanted,
			tx:        tx,
		}
		if mem.addTx(&memTx, sender) {
			mem.notifyTxsAvailable()

			// update metrics
			mem.metrics.Size.Set(float64(mem.Size()))
			mem.metrics.SizeBytes.Set(float64(mem.SizeBytes()))
		}
	}
}

// Called from:
//   - handleCheckTxResponse (lock not held) if tx is valid
func (mem *CListMempool) addTx(memTx *mempoolTx, sender p2p.ID) bool {
	tx := memTx.tx
	txKey := tx.Key()

	// Check if the transaction is already in the mempool.
	if elem, ok := mem.getCElement(txKey); ok {
		if sender != "" {
			// Update senders on existing entry.
			memTx := elem.Value.(*mempoolTx)
			if found := memTx.addSender(sender); found {
				// It should not be possible to receive twice a tx from the same sender.
				mem.logger.Error("Tx already received from peer", "tx", tx.Hash(), "sender", sender)
			}
		}

		mem.logger.Debug(
			"Transaction already in mempool, not adding it again",
			"tx", tx.Hash(),
			"height", mem.height.Load(),
			"total", mem.Size(),
		)
		return false
	}

	// Add new transaction.
	_ = memTx.addSender(sender)
	e := mem.txs.PushBack(memTx)
	mem.txsMap.Store(txKey, e)
	mem.txsBytes.Add(int64(len(tx)))
	mem.metrics.TxSizeBytes.Observe(float64(len(tx)))

	mem.logger.Debug(
		"Added transaction",
		"tx", tx.Hash(),
		"height", mem.height.Load(),
		"total", mem.Size(),
	)
	return true
}

// RemoveTxByKey removes a transaction from the mempool by its TxKey index.
// Called from:
//   - Update (lock held) if tx was committed
//   - handleRecheckTxResponse (lock not held) if tx was invalidated
func (mem *CListMempool) RemoveTxByKey(txKey types.TxKey) error {
	elem, ok := mem.getCElement(txKey)
	if !ok {
		return ErrTxNotFound
	}

	mem.txs.Remove(elem)
	elem.DetachPrev()
	mem.txsMap.Delete(txKey)
	tx := elem.Value.(*mempoolTx).tx
	mem.txsBytes.Add(int64(-len(tx)))
	mem.logger.Debug("Removed transaction", "tx", tx.Hash(), "height", mem.height.Load(), "total", mem.Size())
	return nil
}

func (mem *CListMempool) isFull(txSize int) error {
	memSize := mem.Size()
	txsBytes := mem.SizeBytes()
	if memSize >= mem.config.Size || uint64(txSize)+uint64(txsBytes) > uint64(mem.config.MaxTxsBytes) {
		return ErrMempoolIsFull{
			NumTxs:      memSize,
			MaxTxs:      mem.config.Size,
			TxsBytes:    txsBytes,
			MaxTxsBytes: mem.config.MaxTxsBytes,
		}
	}

	if mem.recheck.consideredFull() {
		return ErrRecheckFull
	}

	return nil
}

// handleRecheckTxResponse handles CheckTx responses for transactions in the mempool that need to be
// revalidated after a mempool update.
func (mem *CListMempool) handleRecheckTxResponse(tx types.Tx) func(res *abci.Response) {
	return func(r *abci.Response) {
		res := r.GetCheckTx()
		if res == nil {
			panic(fmt.Sprintf("unexpected response value %v not of type CheckTx", r))
		}

		// Check whether the rechecking process has finished.
		if mem.recheck.done() {
			mem.logger.Error("Rechecking has finished; discard late recheck response",
				"tx", log.NewLazySprintf("%X", tx.Hash()))
			return
		}
		mem.metrics.RecheckTimes.Add(1)

		// Check whether tx is still in the list of transactions that can be rechecked.
		if !mem.recheck.findNextEntryMatching(&tx) {
			// Reached the end of the list and didn't find a matching tx; rechecking has finished.
			return
		}

		var postCheckErr error
		if mem.postCheck != nil {
			postCheckErr = mem.postCheck(tx, res)
		}

		// If tx is invalid, remove it from the mempool and the cache.
		if (res.Code != abci.CodeTypeOK) || postCheckErr != nil {
			// Tx became invalidated due to newly committed block.
			mem.logger.Debug("Tx is no longer valid", "tx", tx.Hash(), "res", res, "postCheckErr", postCheckErr)
			if err := mem.RemoveTxByKey(tx.Key()); err != nil {
				mem.logger.Debug("Transaction could not be removed from mempool", "err", err)
			} else {
				// update metrics
				mem.metrics.Size.Set(float64(mem.Size()))
				mem.metrics.SizeBytes.Set(float64(mem.SizeBytes()))
			}
			mem.tryRemoveFromCache(tx)
		}
	}
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) TxsAvailable() <-chan struct{} {
	return mem.txsAvailable
}

func (mem *CListMempool) notifyTxsAvailable() {
	if mem.Size() == 0 {
		panic("notified txs available but mempool is empty!")
	}
	if mem.txsAvailable != nil && mem.notifiedTxsAvailable.CompareAndSwap(false, true) {
		// channel cap is 1, so this will send once
		select {
		case mem.txsAvailable <- struct{}{}:
		default:
		}
	}
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) ReapMaxBytesMaxGas(maxBytes, maxGas int64) types.Txs {
	mem.updateMtx.RLock()
	defer mem.updateMtx.RUnlock()

	var (
		totalGas    int64
		runningSize int64
	)

	// TODO: we will get a performance boost if we have a good estimate of avg
	// size per tx, and set the initial capacity based off of that.
	// txs := make([]types.Tx, 0, cmtmath.MinInt(mem.txs.Len(), max/mem.avgTxSize))
	txs := make([]types.Tx, 0, mem.txs.Len())
	for e := mem.txs.Front(); e != nil; e = e.Next() {
		memTx := e.Value.(*mempoolTx)

		txs = append(txs, memTx.tx)

		dataSize := types.ComputeProtoSizeForTxs([]types.Tx{memTx.tx})

		// Check total size requirement
		if maxBytes > -1 && runningSize+dataSize > maxBytes {
			return txs[:len(txs)-1]
		}

		runningSize += dataSize

		// Check total gas requirement.
		// If maxGas is negative, skip this check.
		// Since newTotalGas < masGas, which
		// must be non-negative, it follows that this won't overflow.
		newTotalGas := totalGas + memTx.gasWanted
		if maxGas > -1 && newTotalGas > maxGas {
			return txs[:len(txs)-1]
		}
		totalGas = newTotalGas
	}
	return txs
}

// Safe for concurrent use by multiple goroutines.
func (mem *CListMempool) ReapMaxTxs(max int) types.Txs {
	mem.updateMtx.RLock()
	defer mem.updateMtx.RUnlock()

	if max < 0 {
		max = mem.txs.Len()
	}

	txs := make([]types.Tx, 0, cmtmath.MinInt(mem.txs.Len(), max))
	for e := mem.txs.Front(); e != nil && len(txs) <= max; e = e.Next() {
		memTx := e.Value.(*mempoolTx)
		txs = append(txs, memTx.tx)
	}
	return txs
}

// Lock() must be help by the caller during execution.
// TODO: this function always returns nil; remove the return value.
func (mem *CListMempool) Update(
	height int64,
	txs types.Txs,
	txResults []*abci.ExecTxResult,
	preCheck PreCheckFunc,
	postCheck PostCheckFunc,
) error {
	mem.logger.Debug("Update", "height", height, "len(txs)", len(txs))

	// Set height
	mem.height.Store(height)
	mem.notifiedTxsAvailable.Store(false)

	if preCheck != nil {
		mem.preCheck = preCheck
	}
	if postCheck != nil {
		mem.postCheck = postCheck
	}

	for i, tx := range txs {
		if txResults[i].Code == abci.CodeTypeOK {
			// Add valid committed tx to the cache (if missing).
			_ = mem.addToCache(tx)
		} else {
			mem.tryRemoveFromCache(tx)
		}

		// Remove committed tx from the mempool.
		//
		// Note an evil proposer can drop valid txs!
		// Mempool before:
		//   100 -> 101 -> 102
		// Block, proposed by an evil proposer:
		//   101 -> 102
		// Mempool after:
		//   100
		// https://github.com/tendermint/tendermint/issues/3322.
		if err := mem.RemoveTxByKey(tx.Key()); err != nil {
			mem.logger.Debug("Committed transaction not in local mempool (not an error)",
				"tx", tx.Hash(),
				"error", err.Error())
		}
	}

	// Recheck txs left in the mempool to remove them if they became invalid in the new state.
	if mem.config.Recheck {
		mem.recheckTxs()
	}

	// Notify if there are still txs left in the mempool.
	if mem.Size() > 0 {
		mem.notifyTxsAvailable()
	}

	// Update metrics
	mem.metrics.Size.Set(float64(mem.Size()))
	mem.metrics.SizeBytes.Set(float64(mem.SizeBytes()))

	return nil
}

// recheckTxs sends all transactions in the mempool to the app for re-validation. When the function
// returns, all recheck responses from the app have been processed.
func (mem *CListMempool) recheckTxs() {
	mem.logger.Debug("Recheck txs", "height", mem.height.Load(), "num-txs", mem.Size())

	if mem.Size() <= 0 {
		return
	}

	mem.recheck.init(mem.txs.Front(), mem.txs.Back())

	// NOTE: CheckTx for new transactions cannot be executed concurrently
	// because this function has the lock (via Update and Lock).
	for e := mem.txs.Front(); e != nil; e = e.Next() {
		tx := e.Value.(*mempoolTx).tx
		mem.recheck.numPendingTxs.Add(1)

		// Send CheckTx request to the app to re-validate transaction.
		resReq, err := mem.proxyAppConn.CheckTxAsync(context.TODO(), &abci.CheckTxRequest{
			Tx:   tx,
			Type: abci.CHECK_TX_TYPE_RECHECK,
		})
		if err != nil {
			panic(fmt.Errorf("(re-)CheckTx request for tx %s failed: %w", log.NewLazySprintf("%X", tx.Hash()), err))
		}
		resReq.SetCallback(mem.handleRecheckTxResponse(tx))
	}

	// Flush any pending asynchronous recheck requests to process.
	mem.proxyAppConn.Flush(context.TODO())

	// Give some time to finish processing the responses; then finish the rechecking process, even
	// if not all txs were rechecked.
	select {
	case <-time.After(mem.config.RecheckTimeout):
		mem.recheck.setDone()
		mem.logger.Error("Timed out waiting for recheck responses")
	case <-mem.recheck.doneRechecking():
	}

	if n := mem.recheck.numPendingTxs.Load(); n > 0 {
		mem.logger.Error("Not all txs were rechecked", "not-rechecked", n)
	}
	mem.logger.Debug("Done rechecking txs", "height", mem.height.Load(), "num-txs", mem.Size())
}

// The cursor and end pointers define a dynamic list of transactions that could be rechecked. The
// end pointer is fixed. When a recheck response for a transaction is received, cursor will point to
// the entry in the mempool corresponding to that transaction, thus narrowing the list. Transactions
// corresponding to entries between the old and current positions of cursor will be ignored for
// rechecking. This is to guarantee that recheck responses are processed in the same sequential
// order as they appear in the mempool.
type recheck struct {
	cursor        *clist.CElement // next expected recheck response
	end           *clist.CElement // last entry in the mempool to recheck
	doneCh        chan struct{}   // to signal that rechecking has finished successfully (for async app connections)
	numPendingTxs atomic.Int32    // number of transactions still pending to recheck
	isRechecking  atomic.Bool     // true iff the rechecking process has begun and is not yet finished
	recheckFull   atomic.Bool     // whether rechecking TXs cannot be completed before a new block is decided
}

func newRecheck() *recheck {
	return &recheck{
		doneCh: make(chan struct{}, 1),
	}
}

func (rc *recheck) init(first, last *clist.CElement) {
	if !rc.done() {
		panic("Having more than one rechecking process at a time is not possible.")
	}
	rc.cursor = first
	rc.end = last
	rc.numPendingTxs.Store(0)
	rc.isRechecking.Store(true)
}

// done returns true when there is no recheck response to process.
// Safe for concurrent use by multiple goroutines.
func (rc *recheck) done() bool {
	return !rc.isRechecking.Load()
}

// setDone registers that rechecking has finished.
func (rc *recheck) setDone() {
	rc.cursor = nil
	rc.recheckFull.Store(false)
	rc.isRechecking.Store(false)
}

// setNextEntry sets cursor to the next entry in the list. If there is no next, cursor will be nil.
func (rc *recheck) setNextEntry() {
	rc.cursor = rc.cursor.Next()
}

// tryFinish will check if the cursor is at the end of the list and notify the channel that
// rechecking has finished. It returns true iff it's done rechecking.
func (rc *recheck) tryFinish() bool {
	if rc.cursor == rc.end {
		// Reached end of the list without finding a matching tx.
		rc.setDone()
	}
	if rc.done() {
		// Notify that recheck has finished.
		select {
		case rc.doneCh <- struct{}{}:
		default:
		}
		return true
	}
	return false
}

// findNextEntryMatching searches for the next transaction matching the given transaction, which
// corresponds to the recheck response to be processed next. Then it checks if it has reached the
// end of the list, so it can finish rechecking.
//
// The goal is to guarantee that transactions are rechecked in the order in which they are in the
// mempool. Transactions whose recheck response arrive late or don't arrive at all are skipped and
// not rechecked.
func (rc *recheck) findNextEntryMatching(tx *types.Tx) bool {
	found := false
	for ; !rc.done(); rc.setNextEntry() {
		expectedTx := rc.cursor.Value.(*mempoolTx).tx
		if bytes.Equal(*tx, expectedTx) {
			// Found an entry in the list of txs to recheck that matches tx.
			found = true
			rc.numPendingTxs.Add(-1)
			break
		}
	}

	if !rc.tryFinish() {
		// Not finished yet; set the cursor for processing the next recheck response.
		rc.setNextEntry()
	}
	return found
}

// doneRechecking returns the channel used to signal that rechecking has finished.
func (rc *recheck) doneRechecking() <-chan struct{} {
	return rc.doneCh
}

// setRecheckFull sets recheckFull to true if rechecking is still in progress. It returns true iff
// the value of recheckFull has changed.
func (rc *recheck) setRecheckFull() bool {
	rechecking := !rc.done()
	recheckFull := rc.recheckFull.Swap(rechecking)
	return rechecking != recheckFull
}

// consideredFull returns true iff the mempool should be considered as full while rechecking is in
// progress.
func (rc *recheck) consideredFull() bool {
	return rc.recheckFull.Load()
}
