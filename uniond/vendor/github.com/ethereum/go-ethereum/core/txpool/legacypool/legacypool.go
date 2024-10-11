// Copyright 2014 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

// Package legacypool implements the normal EVM execution transaction pool.
package legacypool

import (
	"errors"
	"math"
	"math/big"
	"sort"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/prque"
	"github.com/ethereum/go-ethereum/consensus/misc/eip1559"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/state"
	"github.com/ethereum/go-ethereum/core/txpool"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/params"
)

const (
	// txSlotSize is used to calculate how many data slots a single transaction
	// takes up based on its size. The slots are used as DoS protection, ensuring
	// that validating a new transaction remains a constant operation (in reality
	// O(maxslots), where max slots are 4 currently).
	txSlotSize = 32 * 1024

	// txMaxSize is the maximum size a single transaction can have. This field has
	// non-trivial consequences: larger transactions are significantly harder and
	// more expensive to propagate; larger transactions also take more resources
	// to validate whether they fit into the pool or not.
	txMaxSize = 4 * txSlotSize // 128KB
)

var (
	// ErrTxPoolOverflow is returned if the transaction pool is full and can't accept
	// another remote transaction.
	ErrTxPoolOverflow = errors.New("txpool is full")
)

var (
	evictionInterval    = time.Minute     // Time interval to check for evictable transactions
	statsReportInterval = 8 * time.Second // Time interval to report transaction pool stats
)

var (
	// Metrics for the pending pool
	pendingDiscardMeter   = metrics.NewRegisteredMeter("txpool/pending/discard", nil)
	pendingReplaceMeter   = metrics.NewRegisteredMeter("txpool/pending/replace", nil)
	pendingRateLimitMeter = metrics.NewRegisteredMeter("txpool/pending/ratelimit", nil) // Dropped due to rate limiting
	pendingNofundsMeter   = metrics.NewRegisteredMeter("txpool/pending/nofunds", nil)   // Dropped due to out-of-funds

	// Metrics for the queued pool
	queuedDiscardMeter   = metrics.NewRegisteredMeter("txpool/queued/discard", nil)
	queuedReplaceMeter   = metrics.NewRegisteredMeter("txpool/queued/replace", nil)
	queuedRateLimitMeter = metrics.NewRegisteredMeter("txpool/queued/ratelimit", nil) // Dropped due to rate limiting
	queuedNofundsMeter   = metrics.NewRegisteredMeter("txpool/queued/nofunds", nil)   // Dropped due to out-of-funds
	queuedEvictionMeter  = metrics.NewRegisteredMeter("txpool/queued/eviction", nil)  // Dropped due to lifetime

	// General tx metrics
	knownTxMeter       = metrics.NewRegisteredMeter("txpool/known", nil)
	validTxMeter       = metrics.NewRegisteredMeter("txpool/valid", nil)
	invalidTxMeter     = metrics.NewRegisteredMeter("txpool/invalid", nil)
	underpricedTxMeter = metrics.NewRegisteredMeter("txpool/underpriced", nil)
	overflowedTxMeter  = metrics.NewRegisteredMeter("txpool/overflowed", nil)

	// throttleTxMeter counts how many transactions are rejected due to too-many-changes between
	// txpool reorgs.
	throttleTxMeter = metrics.NewRegisteredMeter("txpool/throttle", nil)
	// reorgDurationTimer measures how long time a txpool reorg takes.
	reorgDurationTimer = metrics.NewRegisteredTimer("txpool/reorgtime", nil)
	// dropBetweenReorgHistogram counts how many drops we experience between two reorg runs. It is expected
	// that this number is pretty low, since txpool reorgs happen very frequently.
	dropBetweenReorgHistogram = metrics.NewRegisteredHistogram("txpool/dropbetweenreorg", nil, metrics.NewExpDecaySample(1028, 0.015))

	pendingGauge = metrics.NewRegisteredGauge("txpool/pending", nil)
	queuedGauge  = metrics.NewRegisteredGauge("txpool/queued", nil)
	localGauge   = metrics.NewRegisteredGauge("txpool/local", nil)
	slotsGauge   = metrics.NewRegisteredGauge("txpool/slots", nil)

	reheapTimer = metrics.NewRegisteredTimer("txpool/reheap", nil)
)

// BlockChain defines the minimal set of methods needed to back a tx pool with
// a chain. Exists to allow mocking the live chain out of tests.
type BlockChain interface {
	// Config retrieves the chain's fork configuration.
	Config() *params.ChainConfig

	// CurrentBlock returns the current head of the chain.
	CurrentBlock() *types.Header

	// GetBlock retrieves a specific block, used during pool resets.
	GetBlock(hash common.Hash, number uint64) *types.Block

	// StateAt returns a state database for a given root hash (generally the head).
	StateAt(root common.Hash) (*state.StateDB, error)
}

// Config are the configuration parameters of the transaction pool.
type Config struct {
	Locals    []common.Address // Addresses that should be treated by default as local
	NoLocals  bool             // Whether local transaction handling should be disabled
	Journal   string           // Journal of local transactions to survive node restarts
	Rejournal time.Duration    // Time interval to regenerate the local transaction journal

	PriceLimit uint64 // Minimum gas price to enforce for acceptance into the pool
	PriceBump  uint64 // Minimum price bump percentage to replace an already existing transaction (nonce)

	AccountSlots uint64 // Number of executable transaction slots guaranteed per account
	GlobalSlots  uint64 // Maximum number of executable transaction slots for all accounts
	AccountQueue uint64 // Maximum number of non-executable transaction slots permitted per account
	GlobalQueue  uint64 // Maximum number of non-executable transaction slots for all accounts

	Lifetime time.Duration // Maximum amount of time non-executable transaction are queued
}

// DefaultConfig contains the default configurations for the transaction pool.
var DefaultConfig = Config{
	Journal:   "transactions.rlp",
	Rejournal: time.Hour,

	PriceLimit: 1,
	PriceBump:  10,

	AccountSlots: 16,
	GlobalSlots:  4096 + 1024, // urgent + floating queue capacity with 4:1 ratio
	AccountQueue: 64,
	GlobalQueue:  1024,

	Lifetime: 3 * time.Hour,
}

// sanitize checks the provided user configurations and changes anything that's
// unreasonable or unworkable.
func (config *Config) sanitize() Config {
	conf := *config
	if conf.Rejournal < time.Second {
		log.Warn("Sanitizing invalid txpool journal time", "provided", conf.Rejournal, "updated", time.Second)
		conf.Rejournal = time.Second
	}
	if conf.PriceLimit < 1 {
		log.Warn("Sanitizing invalid txpool price limit", "provided", conf.PriceLimit, "updated", DefaultConfig.PriceLimit)
		conf.PriceLimit = DefaultConfig.PriceLimit
	}
	if conf.PriceBump < 1 {
		log.Warn("Sanitizing invalid txpool price bump", "provided", conf.PriceBump, "updated", DefaultConfig.PriceBump)
		conf.PriceBump = DefaultConfig.PriceBump
	}
	if conf.AccountSlots < 1 {
		log.Warn("Sanitizing invalid txpool account slots", "provided", conf.AccountSlots, "updated", DefaultConfig.AccountSlots)
		conf.AccountSlots = DefaultConfig.AccountSlots
	}
	if conf.GlobalSlots < 1 {
		log.Warn("Sanitizing invalid txpool global slots", "provided", conf.GlobalSlots, "updated", DefaultConfig.GlobalSlots)
		conf.GlobalSlots = DefaultConfig.GlobalSlots
	}
	if conf.AccountQueue < 1 {
		log.Warn("Sanitizing invalid txpool account queue", "provided", conf.AccountQueue, "updated", DefaultConfig.AccountQueue)
		conf.AccountQueue = DefaultConfig.AccountQueue
	}
	if conf.GlobalQueue < 1 {
		log.Warn("Sanitizing invalid txpool global queue", "provided", conf.GlobalQueue, "updated", DefaultConfig.GlobalQueue)
		conf.GlobalQueue = DefaultConfig.GlobalQueue
	}
	if conf.Lifetime < 1 {
		log.Warn("Sanitizing invalid txpool lifetime", "provided", conf.Lifetime, "updated", DefaultConfig.Lifetime)
		conf.Lifetime = DefaultConfig.Lifetime
	}
	return conf
}

// LegacyPool contains all currently known transactions. Transactions
// enter the pool when they are received from the network or submitted
// locally. They exit the pool when they are included in the blockchain.
//
// The pool separates processable transactions (which can be applied to the
// current state) and future transactions. Transactions move between those
// two states over time as they are received and processed.
type LegacyPool struct {
	config      Config
	chainconfig *params.ChainConfig
	chain       BlockChain
	gasTip      atomic.Pointer[big.Int]
	txFeed      event.Feed
	signer      types.Signer
	mu          sync.RWMutex

	currentHead   atomic.Pointer[types.Header] // Current head of the blockchain
	currentState  *state.StateDB               // Current state in the blockchain head
	pendingNonces *noncer                      // Pending state tracking virtual nonces

	locals  *accountSet // Set of local transaction to exempt from eviction rules
	journal *journal    // Journal of local transaction to back up to disk

	reserve txpool.AddressReserver       // Address reserver to ensure exclusivity across subpools
	pending map[common.Address]*list     // All currently processable transactions
	queue   map[common.Address]*list     // Queued but non-processable transactions
	beats   map[common.Address]time.Time // Last heartbeat from each known account
	all     *lookup                      // All transactions to allow lookups
	priced  *pricedList                  // All transactions sorted by price

	reqResetCh      chan *txpoolResetRequest
	reqPromoteCh    chan *accountSet
	queueTxEventCh  chan *types.Transaction
	reorgDoneCh     chan chan struct{}
	reorgShutdownCh chan struct{}  // requests shutdown of scheduleReorgLoop
	wg              sync.WaitGroup // tracks loop, scheduleReorgLoop
	initDoneCh      chan struct{}  // is closed once the pool is initialized (for tests)

	changesSinceReorg int // A counter for how many drops we've performed in-between reorg.
}

type txpoolResetRequest struct {
	oldHead, newHead *types.Header
}

// New creates a new transaction pool to gather, sort and filter inbound
// transactions from the network.
func New(config Config, chain BlockChain) *LegacyPool {
	// Sanitize the input to ensure no vulnerable gas prices are set
	config = (&config).sanitize()

	// Create the transaction pool with its initial settings
	pool := &LegacyPool{
		config:          config,
		chain:           chain,
		chainconfig:     chain.Config(),
		signer:          types.LatestSigner(chain.Config()),
		pending:         make(map[common.Address]*list),
		queue:           make(map[common.Address]*list),
		beats:           make(map[common.Address]time.Time),
		all:             newLookup(),
		reqResetCh:      make(chan *txpoolResetRequest),
		reqPromoteCh:    make(chan *accountSet),
		queueTxEventCh:  make(chan *types.Transaction),
		reorgDoneCh:     make(chan chan struct{}),
		reorgShutdownCh: make(chan struct{}),
		initDoneCh:      make(chan struct{}),
	}
	pool.locals = newAccountSet(pool.signer)
	for _, addr := range config.Locals {
		log.Info("Setting new local account", "address", addr)
		pool.locals.add(addr)
	}
	pool.priced = newPricedList(pool.all)

	if !config.NoLocals && config.Journal != "" {
		pool.journal = newTxJournal(config.Journal)
	}
	return pool
}

// Filter returns whether the given transaction can be consumed by the legacy
// pool, specifically, whether it is a Legacy, AccessList or Dynamic transaction.
func (pool *LegacyPool) Filter(tx *types.Transaction) bool {
	switch tx.Type() {
	case types.LegacyTxType, types.AccessListTxType, types.DynamicFeeTxType:
		return true
	default:
		return false
	}
}

// Init sets the gas price needed to keep a transaction in the pool and the chain
// head to allow balance / nonce checks. The transaction journal will be loaded
// from disk and filtered based on the provided starting settings. The internal
// goroutines will be spun up and the pool deemed operational afterwards.
func (pool *LegacyPool) Init(gasTip *big.Int, head *types.Header, reserve txpool.AddressReserver) error {
	// Set the address reserver to request exclusive access to pooled accounts
	pool.reserve = reserve

	// Set the basic pool parameters
	pool.gasTip.Store(gasTip)

	// Initialize the state with head block, or fallback to empty one in
	// case the head state is not available(might occur when node is not
	// fully synced).
	statedb, err := pool.chain.StateAt(head.Root)
	if err != nil {
		statedb, err = pool.chain.StateAt(types.EmptyRootHash)
	}
	if err != nil {
		return err
	}
	pool.currentHead.Store(head)
	pool.currentState = statedb
	pool.pendingNonces = newNoncer(statedb)

	// Start the reorg loop early, so it can handle requests generated during
	// journal loading.
	pool.wg.Add(1)
	go pool.scheduleReorgLoop()

	// If local transactions and journaling is enabled, load from disk
	if pool.journal != nil {
		if err := pool.journal.load(pool.addLocals); err != nil {
			log.Warn("Failed to load transaction journal", "err", err)
		}
		if err := pool.journal.rotate(pool.local()); err != nil {
			log.Warn("Failed to rotate transaction journal", "err", err)
		}
	}
	pool.wg.Add(1)
	go pool.loop()
	return nil
}

// loop is the transaction pool's main event loop, waiting for and reacting to
// outside blockchain events as well as for various reporting and transaction
// eviction events.
func (pool *LegacyPool) loop() {
	defer pool.wg.Done()

	var (
		prevPending, prevQueued, prevStales int

		// Start the stats reporting and transaction eviction tickers
		report  = time.NewTicker(statsReportInterval)
		evict   = time.NewTicker(evictionInterval)
		journal = time.NewTicker(pool.config.Rejournal)
	)
	defer report.Stop()
	defer evict.Stop()
	defer journal.Stop()

	// Notify tests that the init phase is done
	close(pool.initDoneCh)
	for {
		select {
		// Handle pool shutdown
		case <-pool.reorgShutdownCh:
			return

		// Handle stats reporting ticks
		case <-report.C:
			pool.mu.RLock()
			pending, queued := pool.stats()
			pool.mu.RUnlock()
			stales := int(pool.priced.stales.Load())

			if pending != prevPending || queued != prevQueued || stales != prevStales {
				log.Debug("Transaction pool status report", "executable", pending, "queued", queued, "stales", stales)
				prevPending, prevQueued, prevStales = pending, queued, stales
			}

		// Handle inactive account transaction eviction
		case <-evict.C:
			pool.mu.Lock()
			for addr := range pool.queue {
				// Skip local transactions from the eviction mechanism
				if pool.locals.contains(addr) {
					continue
				}
				// Any non-locals old enough should be removed
				if time.Since(pool.beats[addr]) > pool.config.Lifetime {
					list := pool.queue[addr].Flatten()
					for _, tx := range list {
						pool.removeTx(tx.Hash(), true, true)
					}
					queuedEvictionMeter.Mark(int64(len(list)))
				}
			}
			pool.mu.Unlock()

		// Handle local transaction journal rotation
		case <-journal.C:
			if pool.journal != nil {
				pool.mu.Lock()
				if err := pool.journal.rotate(pool.local()); err != nil {
					log.Warn("Failed to rotate local tx journal", "err", err)
				}
				pool.mu.Unlock()
			}
		}
	}
}

// Close terminates the transaction pool.
func (pool *LegacyPool) Close() error {
	// Terminate the pool reorger and return
	close(pool.reorgShutdownCh)
	pool.wg.Wait()

	if pool.journal != nil {
		pool.journal.close()
	}
	log.Info("Transaction pool stopped")
	return nil
}

// Reset implements txpool.SubPool, allowing the legacy pool's internal state to be
// kept in sync with the main transaction pool's internal state.
func (pool *LegacyPool) Reset(oldHead, newHead *types.Header) {
	wait := pool.requestReset(oldHead, newHead)
	<-wait
}

// SubscribeTransactions registers a subscription for new transaction events,
// supporting feeding only newly seen or also resurrected transactions.
func (pool *LegacyPool) SubscribeTransactions(ch chan<- core.NewTxsEvent, reorgs bool) event.Subscription {
	// The legacy pool has a very messed up internal shuffling, so it's kind of
	// hard to separate newly discovered transaction from resurrected ones. This
	// is because the new txs are added to the queue, resurrected ones too and
	// reorgs run lazily, so separating the two would need a marker.
	return pool.txFeed.Subscribe(ch)
}

// SetGasTip updates the minimum gas tip required by the transaction pool for a
// new transaction, and drops all transactions below this threshold.
func (pool *LegacyPool) SetGasTip(tip *big.Int) {
	pool.mu.Lock()
	defer pool.mu.Unlock()

	old := pool.gasTip.Load()
	pool.gasTip.Store(new(big.Int).Set(tip))

	// If the min miner fee increased, remove transactions below the new threshold
	if tip.Cmp(old) > 0 {
		// pool.priced is sorted by GasFeeCap, so we have to iterate through pool.all instead
		drop := pool.all.RemotesBelowTip(tip)
		for _, tx := range drop {
			pool.removeTx(tx.Hash(), false, true)
		}
		pool.priced.Removed(len(drop))
	}
	log.Info("Legacy pool tip threshold updated", "tip", tip)
}

// Nonce returns the next nonce of an account, with all transactions executable
// by the pool already applied on top.
func (pool *LegacyPool) Nonce(addr common.Address) uint64 {
	pool.mu.RLock()
	defer pool.mu.RUnlock()

	return pool.pendingNonces.get(addr)
}

// Stats retrieves the current pool stats, namely the number of pending and the
// number of queued (non-executable) transactions.
func (pool *LegacyPool) Stats() (int, int) {
	pool.mu.RLock()
	defer pool.mu.RUnlock()

	return pool.stats()
}

// stats retrieves the current pool stats, namely the number of pending and the
// number of queued (non-executable) transactions.
func (pool *LegacyPool) stats() (int, int) {
	pending := 0
	for _, list := range pool.pending {
		pending += list.Len()
	}
	queued := 0
	for _, list := range pool.queue {
		queued += list.Len()
	}
	return pending, queued
}

// Content retrieves the data content of the transaction pool, returning all the
// pending as well as queued transactions, grouped by account and sorted by nonce.
func (pool *LegacyPool) Content() (map[common.Address][]*types.Transaction, map[common.Address][]*types.Transaction) {
	pool.mu.Lock()
	defer pool.mu.Unlock()

	pending := make(map[common.Address][]*types.Transaction, len(pool.pending))
	for addr, list := range pool.pending {
		pending[addr] = list.Flatten()
	}
	queued := make(map[common.Address][]*types.Transaction, len(pool.queue))
	for addr, list := range pool.queue {
		queued[addr] = list.Flatten()
	}
	return pending, queued
}

// ContentFrom retrieves the data content of the transaction pool, returning the
// pending as well as queued transactions of this address, grouped by nonce.
func (pool *LegacyPool) ContentFrom(addr common.Address) ([]*types.Transaction, []*types.Transaction) {
	pool.mu.RLock()
	defer pool.mu.RUnlock()

	var pending []*types.Transaction
	if list, ok := pool.pending[addr]; ok {
		pending = list.Flatten()
	}
	var queued []*types.Transaction
	if list, ok := pool.queue[addr]; ok {
		queued = list.Flatten()
	}
	return pending, queued
}

// Pending retrieves all currently processable transactions, grouped by origin
// account and sorted by nonce. The returned transaction set is a copy and can be
// freely modified by calling code.
//
// The enforceTips parameter can be used to do an extra filtering on the pending
// transactions and only return those whose **effective** tip is large enough in
// the next pending execution environment.
func (pool *LegacyPool) Pending(enforceTips bool) map[common.Address][]*txpool.LazyTransaction {
	pool.mu.Lock()
	defer pool.mu.Unlock()

	pending := make(map[common.Address][]*txpool.LazyTransaction, len(pool.pending))
	for addr, list := range pool.pending {
		txs := list.Flatten()

		// If the miner requests tip enforcement, cap the lists now
		if enforceTips && !pool.locals.contains(addr) {
			for i, tx := range txs {
				if tx.EffectiveGasTipIntCmp(pool.gasTip.Load(), pool.priced.urgent.baseFee) < 0 {
					txs = txs[:i]
					break
				}
			}
		}
		if len(txs) > 0 {
			lazies := make([]*txpool.LazyTransaction, len(txs))
			for i := 0; i < len(txs); i++ {
				lazies[i] = &txpool.LazyTransaction{
					Pool:      pool,
					Hash:      txs[i].Hash(),
					Tx:        txs[i],
					Time:      txs[i].Time(),
					GasFeeCap: txs[i].GasFeeCap(),
					GasTipCap: txs[i].GasTipCap(),
					Gas:       txs[i].Gas(),
					BlobGas:   txs[i].BlobGas(),
				}
			}
			pending[addr] = lazies
		}
	}
	return pending
}

// Locals retrieves the accounts currently considered local by the pool.
func (pool *LegacyPool) Locals() []common.Address {
	pool.mu.Lock()
	defer pool.mu.Unlock()

	return pool.locals.flatten()
}

// local retrieves all currently known local transactions, grouped by origin
// account and sorted by nonce. The returned transaction set is a copy and can be
// freely modified by calling code.
func (pool *LegacyPool) local() map[common.Address]types.Transactions {
	txs := make(map[common.Address]types.Transactions)
	for addr := range pool.locals.accounts {
		if pending := pool.pending[addr]; pending != nil {
			txs[addr] = append(txs[addr], pending.Flatten()...)
		}
		if queued := pool.queue[addr]; queued != nil {
			txs[addr] = append(txs[addr], queued.Flatten()...)
		}
	}
	return txs
}

// validateTxBasics checks whether a transaction is valid according to the consensus
// rules, but does not check state-dependent validation such as sufficient balance.
// This check is meant as an early check which only needs to be performed once,
// and does not require the pool mutex to be held.
func (pool *LegacyPool) validateTxBasics(tx *types.Transaction, local bool) error {
	opts := &txpool.ValidationOptions{
		Config: pool.chainconfig,
		Accept: 0 |
			1<<types.LegacyTxType |
			1<<types.AccessListTxType |
			1<<types.DynamicFeeTxType,
		MaxSize: txMaxSize,
		MinTip:  pool.gasTip.Load(),
	}
	if local {
		opts.MinTip = new(big.Int)
	}
	if err := txpool.ValidateTransaction(tx, pool.currentHead.Load(), pool.signer, opts); err != nil {
		return err
	}
	return nil
}

// validateTx checks whether a transaction is valid according to the consensus
// rules and adheres to some heuristic limits of the local node (price and size).
func (pool *LegacyPool) validateTx(tx *types.Transaction, local bool) error {
	opts := &txpool.ValidationOptionsWithState{
		State: pool.currentState,

		FirstNonceGap: nil, // Pool allows arbitrary arrival order, don't invalidate nonce gaps
		UsedAndLeftSlots: func(addr common.Address) (int, int) {
			var have int
			if list := pool.pending[addr]; list != nil {
				have += list.Len()
			}
			if list := pool.queue[addr]; list != nil {
				have += list.Len()
			}
			return have, math.MaxInt
		},
		ExistingExpenditure: func(addr common.Address) *big.Int {
			if list := pool.pending[addr]; list != nil {
				return list.totalcost
			}
			return new(big.Int)
		},
		ExistingCost: func(addr common.Address, nonce uint64) *big.Int {
			if list := pool.pending[addr]; list != nil {
				if tx := list.txs.Get(nonce); tx != nil {
					return tx.Cost()
				}
			}
			return nil
		},
	}
	if err := txpool.ValidateTransactionWithState(tx, pool.signer, opts); err != nil {
		return err
	}
	return nil
}

// add validates a transaction and inserts it into the non-executable queue for later
// pending promotion and execution. If the transaction is a replacement for an already
// pending or queued one, it overwrites the previous transaction if its price is higher.
//
// If a newly added transaction is marked as local, its sending account will be
// added to the allowlist, preventing any associated transaction from being dropped
// out of the pool due to pricing constraints.
func (pool *LegacyPool) add(tx *types.Transaction, local bool) (replaced bool, err error) {
	// If the transaction is already known, discard it
	hash := tx.Hash()
	if pool.all.Get(hash) != nil {
		log.Trace("Discarding already known transaction", "hash", hash)
		knownTxMeter.Mark(1)
		return false, txpool.ErrAlreadyKnown
	}
	// Make the local flag. If it's from local source or it's from the network but
	// the sender is marked as local previously, treat it as the local transaction.
	isLocal := local || pool.locals.containsTx(tx)

	// If the transaction fails basic validation, discard it
	if err := pool.validateTx(tx, isLocal); err != nil {
		log.Trace("Discarding invalid transaction", "hash", hash, "err", err)
		invalidTxMeter.Mark(1)
		return false, err
	}
	// already validated by this point
	from, _ := types.Sender(pool.signer, tx)

	// If the address is not yet known, request exclusivity to track the account
	// only by this subpool until all transactions are evicted
	var (
		_, hasPending = pool.pending[from]
		_, hasQueued  = pool.queue[from]
	)
	if !hasPending && !hasQueued {
		if err := pool.reserve(from, true); err != nil {
			return false, err
		}
		defer func() {
			// If the transaction is rejected by some post-validation check, remove
			// the lock on the reservation set.
			//
			// Note, `err` here is the named error return, which will be initialized
			// by a return statement before running deferred methods. Take care with
			// removing or subscoping err as it will break this clause.
			if err != nil {
				pool.reserve(from, false)
			}
		}()
	}
	// If the transaction pool is full, discard underpriced transactions
	if uint64(pool.all.Slots()+numSlots(tx)) > pool.config.GlobalSlots+pool.config.GlobalQueue {
		// If the new transaction is underpriced, don't accept it
		if !isLocal && pool.priced.Underpriced(tx) {
			log.Trace("Discarding underpriced transaction", "hash", hash, "gasTipCap", tx.GasTipCap(), "gasFeeCap", tx.GasFeeCap())
			underpricedTxMeter.Mark(1)
			return false, txpool.ErrUnderpriced
		}

		// We're about to replace a transaction. The reorg does a more thorough
		// analysis of what to remove and how, but it runs async. We don't want to
		// do too many replacements between reorg-runs, so we cap the number of
		// replacements to 25% of the slots
		if pool.changesSinceReorg > int(pool.config.GlobalSlots/4) {
			throttleTxMeter.Mark(1)
			return false, ErrTxPoolOverflow
		}

		// New transaction is better than our worse ones, make room for it.
		// If it's a local transaction, forcibly discard all available transactions.
		// Otherwise if we can't make enough room for new one, abort the operation.
		drop, success := pool.priced.Discard(pool.all.Slots()-int(pool.config.GlobalSlots+pool.config.GlobalQueue)+numSlots(tx), isLocal)

		// Special case, we still can't make the room for the new remote one.
		if !isLocal && !success {
			log.Trace("Discarding overflown transaction", "hash", hash)
			overflowedTxMeter.Mark(1)
			return false, ErrTxPoolOverflow
		}

		// If the new transaction is a future transaction it should never churn pending transactions
		if !isLocal && pool.isGapped(from, tx) {
			var replacesPending bool
			for _, dropTx := range drop {
				dropSender, _ := types.Sender(pool.signer, dropTx)
				if list := pool.pending[dropSender]; list != nil && list.Contains(dropTx.Nonce()) {
					replacesPending = true
					break
				}
			}
			// Add all transactions back to the priced queue
			if replacesPending {
				for _, dropTx := range drop {
					pool.priced.Put(dropTx, false)
				}
				log.Trace("Discarding future transaction replacing pending tx", "hash", hash)
				return false, txpool.ErrFutureReplacePending
			}
		}

		// Kick out the underpriced remote transactions.
		for _, tx := range drop {
			log.Trace("Discarding freshly underpriced transaction", "hash", tx.Hash(), "gasTipCap", tx.GasTipCap(), "gasFeeCap", tx.GasFeeCap())
			underpricedTxMeter.Mark(1)

			sender, _ := types.Sender(pool.signer, tx)
			dropped := pool.removeTx(tx.Hash(), false, sender != from) // Don't unreserve the sender of the tx being added if last from the acc

			pool.changesSinceReorg += dropped
		}
	}

	// Try to replace an existing transaction in the pending pool
	if list := pool.pending[from]; list != nil && list.Contains(tx.Nonce()) {
		// Nonce already pending, check if required price bump is met
		inserted, old := list.Add(tx, pool.config.PriceBump)
		if !inserted {
			pendingDiscardMeter.Mark(1)
			return false, txpool.ErrReplaceUnderpriced
		}
		// New transaction is better, replace old one
		if old != nil {
			pool.all.Remove(old.Hash())
			pool.priced.Removed(1)
			pendingReplaceMeter.Mark(1)
		}
		pool.all.Add(tx, isLocal)
		pool.priced.Put(tx, isLocal)
		pool.journalTx(from, tx)
		pool.queueTxEvent(tx)
		log.Trace("Pooled new executable transaction", "hash", hash, "from", from, "to", tx.To())

		// Successful promotion, bump the heartbeat
		pool.beats[from] = time.Now()
		return old != nil, nil
	}
	// New transaction isn't replacing a pending one, push into queue
	replaced, err = pool.enqueueTx(hash, tx, isLocal, true)
	if err != nil {
		return false, err
	}
	// Mark local addresses and journal local transactions
	if local && !pool.locals.contains(from) {
		log.Info("Setting new local account", "address", from)
		pool.locals.add(from)
		pool.priced.Removed(pool.all.RemoteToLocals(pool.locals)) // Migrate the remotes if it's marked as local first time.
	}
	if isLocal {
		localGauge.Inc(1)
	}
	pool.journalTx(from, tx)

	log.Trace("Pooled new future transaction", "hash", hash, "from", from, "to", tx.To())
	return replaced, nil
}

// isGapped reports whether the given transaction is immediately executable.
func (pool *LegacyPool) isGapped(from common.Address, tx *types.Transaction) bool {
	// Short circuit if transaction falls within the scope of the pending list
	// or matches the next pending nonce which can be promoted as an executable
	// transaction afterwards. Note, the tx staleness is already checked in
	// 'validateTx' function previously.
	next := pool.pendingNonces.get(from)
	if tx.Nonce() <= next {
		return false
	}
	// The transaction has a nonce gap with pending list, it's only considered
	// as executable if transactions in queue can fill up the nonce gap.
	queue, ok := pool.queue[from]
	if !ok {
		return true
	}
	for nonce := next; nonce < tx.Nonce(); nonce++ {
		if !queue.Contains(nonce) {
			return true // txs in queue can't fill up the nonce gap
		}
	}
	return false
}

// enqueueTx inserts a new transaction into the non-executable transaction queue.
//
// Note, this method assumes the pool lock is held!
func (pool *LegacyPool) enqueueTx(hash common.Hash, tx *types.Transaction, local bool, addAll bool) (bool, error) {
	// Try to insert the transaction into the future queue
	from, _ := types.Sender(pool.signer, tx) // already validated
	if pool.queue[from] == nil {
		pool.queue[from] = newList(false)
	}
	inserted, old := pool.queue[from].Add(tx, pool.config.PriceBump)
	if !inserted {
		// An older transaction was better, discard this
		queuedDiscardMeter.Mark(1)
		return false, txpool.ErrReplaceUnderpriced
	}
	// Discard any previous transaction and mark this
	if old != nil {
		pool.all.Remove(old.Hash())
		pool.priced.Removed(1)
		queuedReplaceMeter.Mark(1)
	} else {
		// Nothing was replaced, bump the queued counter
		queuedGauge.Inc(1)
	}
	// If the transaction isn't in lookup set but it's expected to be there,
	// show the error log.
	if pool.all.Get(hash) == nil && !addAll {
		log.Error("Missing transaction in lookup set, please report the issue", "hash", hash)
	}
	if addAll {
		pool.all.Add(tx, local)
		pool.priced.Put(tx, local)
	}
	// If we never record the heartbeat, do it right now.
	if _, exist := pool.beats[from]; !exist {
		pool.beats[from] = time.Now()
	}
	return old != nil, nil
}

// journalTx adds the specified transaction to the local disk journal if it is
// deemed to have been sent from a local account.
func (pool *LegacyPool) journalTx(from common.Address, tx *types.Transaction) {
	// Only journal if it's enabled and the transaction is local
	if pool.journal == nil || !pool.locals.contains(from) {
		return
	}
	if err := pool.journal.insert(tx); err != nil {
		log.Warn("Failed to journal local transaction", "err", err)
	}
}

// promoteTx adds a transaction to the pending (processable) list of transactions
// and returns whether it was inserted or an older was better.
//
// Note, this method assumes the pool lock is held!
func (pool *LegacyPool) promoteTx(addr common.Address, hash common.Hash, tx *types.Transaction) bool {
	// Try to insert the transaction into the pending queue
	if pool.pending[addr] == nil {
		pool.pending[addr] = newList(true)
	}
	list := pool.pending[addr]

	inserted, old := list.Add(tx, pool.config.PriceBump)
	if !inserted {
		// An older transaction was better, discard this
		pool.all.Remove(hash)
		pool.priced.Removed(1)
		pendingDiscardMeter.Mark(1)
		return false
	}
	// Otherwise discard any previous transaction and mark this
	if old != nil {
		pool.all.Remove(old.Hash())
		pool.priced.Removed(1)
		pendingReplaceMeter.Mark(1)
	} else {
		// Nothing was replaced, bump the pending counter
		pendingGauge.Inc(1)
	}
	// Set the potentially new pending nonce and notify any subsystems of the new tx
	pool.pendingNonces.set(addr, tx.Nonce()+1)

	// Successful promotion, bump the heartbeat
	pool.beats[addr] = time.Now()
	return true
}

// addLocals enqueues a batch of transactions into the pool if they are valid, marking the
// senders as local ones, ensuring they go around the local pricing constraints.
//
// This method is used to add transactions from the RPC API and performs synchronous pool
// reorganization and event propagation.
func (pool *LegacyPool) addLocals(txs []*types.Transaction) []error {
	return pool.Add(txs, !pool.config.NoLocals, true)
}

// addLocal enqueues a single local transaction into the pool if it is valid. This is
// a convenience wrapper around addLocals.
func (pool *LegacyPool) addLocal(tx *types.Transaction) error {
	errs := pool.addLocals([]*types.Transaction{tx})
	return errs[0]
}

// addRemotes enqueues a batch of transactions into the pool if they are valid. If the
// senders are not among the locally tracked ones, full pricing constraints will apply.
//
// This method is used to add transactions from the p2p network and does not wait for pool
// reorganization and internal event propagation.
func (pool *LegacyPool) addRemotes(txs []*types.Transaction) []error {
	return pool.Add(txs, false, false)
}

// addRemote enqueues a single transaction into the pool if it is valid. This is a convenience
// wrapper around addRemotes.
func (pool *LegacyPool) addRemote(tx *types.Transaction) error {
	errs := pool.addRemotes([]*types.Transaction{tx})
	return errs[0]
}

// addRemotesSync is like addRemotes, but waits for pool reorganization. Tests use this method.
func (pool *LegacyPool) addRemotesSync(txs []*types.Transaction) []error {
	return pool.Add(txs, false, true)
}

// This is like addRemotes with a single transaction, but waits for pool reorganization. Tests use this method.
func (pool *LegacyPool) addRemoteSync(tx *types.Transaction) error {
	return pool.Add([]*types.Transaction{tx}, false, true)[0]
}

// Add enqueues a batch of transactions into the pool if they are valid. Depending
// on the local flag, full pricing constraints will or will not be applied.
//
// If sync is set, the method will block until all internal maintenance related
// to the add is finished. Only use this during tests for determinism!
func (pool *LegacyPool) Add(txs []*types.Transaction, local, sync bool) []error {
	// Filter out known ones without obtaining the pool lock or recovering signatures
	var (
		errs = make([]error, len(txs))
		news = make([]*types.Transaction, 0, len(txs))
	)
	for i, tx := range txs {
		// If the transaction is known, pre-set the error slot
		if pool.all.Get(tx.Hash()) != nil {
			errs[i] = txpool.ErrAlreadyKnown
			knownTxMeter.Mark(1)
			continue
		}
		// Exclude transactions with basic errors, e.g invalid signatures and
		// insufficient intrinsic gas as soon as possible and cache senders
		// in transactions before obtaining lock
		if err := pool.validateTxBasics(tx, local); err != nil {
			errs[i] = err
			invalidTxMeter.Mark(1)
			continue
		}
		// Accumulate all unknown transactions for deeper processing
		news = append(news, tx)
	}
	if len(news) == 0 {
		return errs
	}

	// Process all the new transaction and merge any errors into the original slice
	pool.mu.Lock()
	newErrs, dirtyAddrs := pool.addTxsLocked(news, local)
	pool.mu.Unlock()

	var nilSlot = 0
	for _, err := range newErrs {
		for errs[nilSlot] != nil {
			nilSlot++
		}
		errs[nilSlot] = err
		nilSlot++
	}
	// Reorg the pool internals if needed and return
	done := pool.requestPromoteExecutables(dirtyAddrs)
	if sync {
		<-done
	}
	return errs
}

// addTxsLocked attempts to queue a batch of transactions if they are valid.
// The transaction pool lock must be held.
func (pool *LegacyPool) addTxsLocked(txs []*types.Transaction, local bool) ([]error, *accountSet) {
	dirty := newAccountSet(pool.signer)
	errs := make([]error, len(txs))
	for i, tx := range txs {
		replaced, err := pool.add(tx, local)
		errs[i] = err
		if err == nil && !replaced {
			dirty.addTx(tx)
		}
	}
	validTxMeter.Mark(int64(len(dirty.accounts)))
	return errs, dirty
}

// Status returns the status (unknown/pending/queued) of a batch of transactions
// identified by their hashes.
func (pool *LegacyPool) Status(hash common.Hash) txpool.TxStatus {
	tx := pool.get(hash)
	if tx == nil {
		return txpool.TxStatusUnknown
	}
	from, _ := types.Sender(pool.signer, tx) // already validated

	pool.mu.RLock()
	defer pool.mu.RUnlock()

	if txList := pool.pending[from]; txList != nil && txList.txs.items[tx.Nonce()] != nil {
		return txpool.TxStatusPending
	} else if txList := pool.queue[from]; txList != nil && txList.txs.items[tx.Nonce()] != nil {
		return txpool.TxStatusQueued
	}
	return txpool.TxStatusUnknown
}

// Get returns a transaction if it is contained in the pool and nil otherwise.
func (pool *LegacyPool) Get(hash common.Hash) *types.Transaction {
	tx := pool.get(hash)
	if tx == nil {
		return nil
	}
	return tx
}

// get returns a transaction if it is contained in the pool and nil otherwise.
func (pool *LegacyPool) get(hash common.Hash) *types.Transaction {
	return pool.all.Get(hash)
}

// Has returns an indicator whether txpool has a transaction cached with the
// given hash.
func (pool *LegacyPool) Has(hash common.Hash) bool {
	return pool.all.Get(hash) != nil
}

// removeTx removes a single transaction from the queue, moving all subsequent
// transactions back to the future queue.
//
// In unreserve is false, the account will not be relinquished to the main txpool
// even if there are no more references to it. This is used to handle a race when
// a tx being added, and it evicts a previously scheduled tx from the same account,
// which could lead to a premature release of the lock.
//
// Returns the number of transactions removed from the pending queue.
func (pool *LegacyPool) removeTx(hash common.Hash, outofbound bool, unreserve bool) int {
	// Fetch the transaction we wish to delete
	tx := pool.all.Get(hash)
	if tx == nil {
		return 0
	}
	addr, _ := types.Sender(pool.signer, tx) // already validated during insertion

	// If after deletion there are no more transactions belonging to this account,
	// relinquish the address reservation. It's a bit convoluted do this, via a
	// defer, but it's safer vs. the many return pathways.
	if unreserve {
		defer func() {
			var (
				_, hasPending = pool.pending[addr]
				_, hasQueued  = pool.queue[addr]
			)
			if !hasPending && !hasQueued {
				pool.reserve(addr, false)
			}
		}()
	}
	// Remove it from the list of known transactions
	pool.all.Remove(hash)
	if outofbound {
		pool.priced.Removed(1)
	}
	if pool.locals.contains(addr) {
		localGauge.Dec(1)
	}
	// Remove the transaction from the pending lists and reset the account nonce
	if pending := pool.pending[addr]; pending != nil {
		if removed, invalids := pending.Remove(tx); removed {
			// If no more pending transactions are left, remove the list
			if pending.Empty() {
				delete(pool.pending, addr)
			}
			// Postpone any invalidated transactions
			for _, tx := range invalids {
				// Internal shuffle shouldn't touch the lookup set.
				pool.enqueueTx(tx.Hash(), tx, false, false)
			}
			// Update the account nonce if needed
			pool.pendingNonces.setIfLower(addr, tx.Nonce())
			// Reduce the pending counter
			pendingGauge.Dec(int64(1 + len(invalids)))
			return 1 + len(invalids)
		}
	}
	// Transaction is in the future queue
	if future := pool.queue[addr]; future != nil {
		if removed, _ := future.Remove(tx); removed {
			// Reduce the queued counter
			queuedGauge.Dec(1)
		}
		if future.Empty() {
			delete(pool.queue, addr)
			delete(pool.beats, addr)
		}
	}
	return 0
}

// requestReset requests a pool reset to the new head block.
// The returned channel is closed when the reset has occurred.
func (pool *LegacyPool) requestReset(oldHead *types.Header, newHead *types.Header) chan struct{} {
	select {
	case pool.reqResetCh <- &txpoolResetRequest{oldHead, newHead}:
		return <-pool.reorgDoneCh
	case <-pool.reorgShutdownCh:
		return pool.reorgShutdownCh
	}
}

// requestPromoteExecutables requests transaction promotion checks for the given addresses.
// The returned channel is closed when the promotion checks have occurred.
func (pool *LegacyPool) requestPromoteExecutables(set *accountSet) chan struct{} {
	select {
	case pool.reqPromoteCh <- set:
		return <-pool.reorgDoneCh
	case <-pool.reorgShutdownCh:
		return pool.reorgShutdownCh
	}
}

// queueTxEvent enqueues a transaction event to be sent in the next reorg run.
func (pool *LegacyPool) queueTxEvent(tx *types.Transaction) {
	select {
	case pool.queueTxEventCh <- tx:
	case <-pool.reorgShutdownCh:
	}
}

// scheduleReorgLoop schedules runs of reset and promoteExecutables. Code above should not
// call those methods directly, but request them being run using requestReset and
// requestPromoteExecutables instead.
func (pool *LegacyPool) scheduleReorgLoop() {
	defer pool.wg.Done()

	var (
		curDone       chan struct{} // non-nil while runReorg is active
		nextDone      = make(chan struct{})
		launchNextRun bool
		reset         *txpoolResetRequest
		dirtyAccounts *accountSet
		queuedEvents  = make(map[common.Address]*sortedMap)
	)
	for {
		// Launch next background reorg if needed
		if curDone == nil && launchNextRun {
			// Run the background reorg and announcements
			go pool.runReorg(nextDone, reset, dirtyAccounts, queuedEvents)

			// Prepare everything for the next round of reorg
			curDone, nextDone = nextDone, make(chan struct{})
			launchNextRun = false

			reset, dirtyAccounts = nil, nil
			queuedEvents = make(map[common.Address]*sortedMap)
		}

		select {
		case req := <-pool.reqResetCh:
			// Reset request: update head if request is already pending.
			if reset == nil {
				reset = req
			} else {
				reset.newHead = req.newHead
			}
			launchNextRun = true
			pool.reorgDoneCh <- nextDone

		case req := <-pool.reqPromoteCh:
			// Promote request: update address set if request is already pending.
			if dirtyAccounts == nil {
				dirtyAccounts = req
			} else {
				dirtyAccounts.merge(req)
			}
			launchNextRun = true
			pool.reorgDoneCh <- nextDone

		case tx := <-pool.queueTxEventCh:
			// Queue up the event, but don't schedule a reorg. It's up to the caller to
			// request one later if they want the events sent.
			addr, _ := types.Sender(pool.signer, tx)
			if _, ok := queuedEvents[addr]; !ok {
				queuedEvents[addr] = newSortedMap()
			}
			queuedEvents[addr].Put(tx)

		case <-curDone:
			curDone = nil

		case <-pool.reorgShutdownCh:
			// Wait for current run to finish.
			if curDone != nil {
				<-curDone
			}
			close(nextDone)
			return
		}
	}
}

// runReorg runs reset and promoteExecutables on behalf of scheduleReorgLoop.
func (pool *LegacyPool) runReorg(done chan struct{}, reset *txpoolResetRequest, dirtyAccounts *accountSet, events map[common.Address]*sortedMap) {
	defer func(t0 time.Time) {
		reorgDurationTimer.Update(time.Since(t0))
	}(time.Now())
	defer close(done)

	var promoteAddrs []common.Address
	if dirtyAccounts != nil && reset == nil {
		// Only dirty accounts need to be promoted, unless we're resetting.
		// For resets, all addresses in the tx queue will be promoted and
		// the flatten operation can be avoided.
		promoteAddrs = dirtyAccounts.flatten()
	}
	pool.mu.Lock()
	if reset != nil {
		// Reset from the old head to the new, rescheduling any reorged transactions
		pool.reset(reset.oldHead, reset.newHead)

		// Nonces were reset, discard any events that became stale
		for addr := range events {
			events[addr].Forward(pool.pendingNonces.get(addr))
			if events[addr].Len() == 0 {
				delete(events, addr)
			}
		}
		// Reset needs promote for all addresses
		promoteAddrs = make([]common.Address, 0, len(pool.queue))
		for addr := range pool.queue {
			promoteAddrs = append(promoteAddrs, addr)
		}
	}
	// Check for pending transactions for every account that sent new ones
	promoted := pool.promoteExecutables(promoteAddrs)

	// If a new block appeared, validate the pool of pending transactions. This will
	// remove any transaction that has been included in the block or was invalidated
	// because of another transaction (e.g. higher gas price).
	if reset != nil {
		pool.demoteUnexecutables()
		if reset.newHead != nil {
			if pool.chainconfig.IsLondon(new(big.Int).Add(reset.newHead.Number, big.NewInt(1))) {
				pendingBaseFee := eip1559.CalcBaseFee(pool.chainconfig, reset.newHead)
				pool.priced.SetBaseFee(pendingBaseFee)
			} else {
				pool.priced.Reheap()
			}
		}
		// Update all accounts to the latest known pending nonce
		nonces := make(map[common.Address]uint64, len(pool.pending))
		for addr, list := range pool.pending {
			highestPending := list.LastElement()
			nonces[addr] = highestPending.Nonce() + 1
		}
		pool.pendingNonces.setAll(nonces)
	}
	// Ensure pool.queue and pool.pending sizes stay within the configured limits.
	pool.truncatePending()
	pool.truncateQueue()

	dropBetweenReorgHistogram.Update(int64(pool.changesSinceReorg))
	pool.changesSinceReorg = 0 // Reset change counter
	pool.mu.Unlock()

	// Notify subsystems for newly added transactions
	for _, tx := range promoted {
		addr, _ := types.Sender(pool.signer, tx)
		if _, ok := events[addr]; !ok {
			events[addr] = newSortedMap()
		}
		events[addr].Put(tx)
	}
	if len(events) > 0 {
		var txs []*types.Transaction
		for _, set := range events {
			txs = append(txs, set.Flatten()...)
		}
		pool.txFeed.Send(core.NewTxsEvent{Txs: txs})
	}
}

// reset retrieves the current state of the blockchain and ensures the content
// of the transaction pool is valid with regard to the chain state.
func (pool *LegacyPool) reset(oldHead, newHead *types.Header) {
	// If we're reorging an old state, reinject all dropped transactions
	var reinject types.Transactions

	if oldHead != nil && oldHead.Hash() != newHead.ParentHash {
		// If the reorg is too deep, avoid doing it (will happen during fast sync)
		oldNum := oldHead.Number.Uint64()
		newNum := newHead.Number.Uint64()

		if depth := uint64(math.Abs(float64(oldNum) - float64(newNum))); depth > 64 {
			log.Debug("Skipping deep transaction reorg", "depth", depth)
		} else {
			// Reorg seems shallow enough to pull in all transactions into memory
			var (
				rem = pool.chain.GetBlock(oldHead.Hash(), oldHead.Number.Uint64())
				add = pool.chain.GetBlock(newHead.Hash(), newHead.Number.Uint64())
			)
			if rem == nil {
				// This can happen if a setHead is performed, where we simply discard the old
				// head from the chain.
				// If that is the case, we don't have the lost transactions anymore, and
				// there's nothing to add
				if newNum >= oldNum {
					// If we reorged to a same or higher number, then it's not a case of setHead
					log.Warn("Transaction pool reset with missing old head",
						"old", oldHead.Hash(), "oldnum", oldNum, "new", newHead.Hash(), "newnum", newNum)
					return
				}
				// If the reorg ended up on a lower number, it's indicative of setHead being the cause
				log.Debug("Skipping transaction reset caused by setHead",
					"old", oldHead.Hash(), "oldnum", oldNum, "new", newHead.Hash(), "newnum", newNum)
				// We still need to update the current state s.th. the lost transactions can be readded by the user
			} else {
				if add == nil {
					// if the new head is nil, it means that something happened between
					// the firing of newhead-event and _now_: most likely a
					// reorg caused by sync-reversion or explicit sethead back to an
					// earlier block.
					log.Warn("Transaction pool reset with missing new head", "number", newHead.Number, "hash", newHead.Hash())
					return
				}
				var discarded, included types.Transactions
				for rem.NumberU64() > add.NumberU64() {
					discarded = append(discarded, rem.Transactions()...)
					if rem = pool.chain.GetBlock(rem.ParentHash(), rem.NumberU64()-1); rem == nil {
						log.Error("Unrooted old chain seen by tx pool", "block", oldHead.Number, "hash", oldHead.Hash())
						return
					}
				}
				for add.NumberU64() > rem.NumberU64() {
					included = append(included, add.Transactions()...)
					if add = pool.chain.GetBlock(add.ParentHash(), add.NumberU64()-1); add == nil {
						log.Error("Unrooted new chain seen by tx pool", "block", newHead.Number, "hash", newHead.Hash())
						return
					}
				}
				for rem.Hash() != add.Hash() {
					discarded = append(discarded, rem.Transactions()...)
					if rem = pool.chain.GetBlock(rem.ParentHash(), rem.NumberU64()-1); rem == nil {
						log.Error("Unrooted old chain seen by tx pool", "block", oldHead.Number, "hash", oldHead.Hash())
						return
					}
					included = append(included, add.Transactions()...)
					if add = pool.chain.GetBlock(add.ParentHash(), add.NumberU64()-1); add == nil {
						log.Error("Unrooted new chain seen by tx pool", "block", newHead.Number, "hash", newHead.Hash())
						return
					}
				}
				lost := make([]*types.Transaction, 0, len(discarded))
				for _, tx := range types.TxDifference(discarded, included) {
					if pool.Filter(tx) {
						lost = append(lost, tx)
					}
				}
				reinject = lost
			}
		}
	}
	// Initialize the internal state to the current head
	if newHead == nil {
		newHead = pool.chain.CurrentBlock() // Special case during testing
	}
	statedb, err := pool.chain.StateAt(newHead.Root)
	if err != nil {
		log.Error("Failed to reset txpool state", "err", err)
		return
	}
	pool.currentHead.Store(newHead)
	pool.currentState = statedb
	pool.pendingNonces = newNoncer(statedb)

	// Inject any transactions discarded due to reorgs
	log.Debug("Reinjecting stale transactions", "count", len(reinject))
	core.SenderCacher.Recover(pool.signer, reinject)
	pool.addTxsLocked(reinject, false)
}

// promoteExecutables moves transactions that have become processable from the
// future queue to the set of pending transactions. During this process, all
// invalidated transactions (low nonce, low balance) are deleted.
func (pool *LegacyPool) promoteExecutables(accounts []common.Address) []*types.Transaction {
	// Track the promoted transactions to broadcast them at once
	var promoted []*types.Transaction

	// Iterate over all accounts and promote any executable transactions
	gasLimit := pool.currentHead.Load().GasLimit
	for _, addr := range accounts {
		list := pool.queue[addr]
		if list == nil {
			continue // Just in case someone calls with a non existing account
		}
		// Drop all transactions that are deemed too old (low nonce)
		forwards := list.Forward(pool.currentState.GetNonce(addr))
		for _, tx := range forwards {
			hash := tx.Hash()
			pool.all.Remove(hash)
		}
		log.Trace("Removed old queued transactions", "count", len(forwards))
		// Drop all transactions that are too costly (low balance or out of gas)
		drops, _ := list.Filter(pool.currentState.GetBalance(addr), gasLimit)
		for _, tx := range drops {
			hash := tx.Hash()
			pool.all.Remove(hash)
		}
		log.Trace("Removed unpayable queued transactions", "count", len(drops))
		queuedNofundsMeter.Mark(int64(len(drops)))

		// Gather all executable transactions and promote them
		readies := list.Ready(pool.pendingNonces.get(addr))
		for _, tx := range readies {
			hash := tx.Hash()
			if pool.promoteTx(addr, hash, tx) {
				promoted = append(promoted, tx)
			}
		}
		log.Trace("Promoted queued transactions", "count", len(promoted))
		queuedGauge.Dec(int64(len(readies)))

		// Drop all transactions over the allowed limit
		var caps types.Transactions
		if !pool.locals.contains(addr) {
			caps = list.Cap(int(pool.config.AccountQueue))
			for _, tx := range caps {
				hash := tx.Hash()
				pool.all.Remove(hash)
				log.Trace("Removed cap-exceeding queued transaction", "hash", hash)
			}
			queuedRateLimitMeter.Mark(int64(len(caps)))
		}
		// Mark all the items dropped as removed
		pool.priced.Removed(len(forwards) + len(drops) + len(caps))
		queuedGauge.Dec(int64(len(forwards) + len(drops) + len(caps)))
		if pool.locals.contains(addr) {
			localGauge.Dec(int64(len(forwards) + len(drops) + len(caps)))
		}
		// Delete the entire queue entry if it became empty.
		if list.Empty() {
			delete(pool.queue, addr)
			delete(pool.beats, addr)
			if _, ok := pool.pending[addr]; !ok {
				pool.reserve(addr, false)
			}
		}
	}
	return promoted
}

// truncatePending removes transactions from the pending queue if the pool is above the
// pending limit. The algorithm tries to reduce transaction counts by an approximately
// equal number for all for accounts with many pending transactions.
func (pool *LegacyPool) truncatePending() {
	pending := uint64(0)
	for _, list := range pool.pending {
		pending += uint64(list.Len())
	}
	if pending <= pool.config.GlobalSlots {
		return
	}

	pendingBeforeCap := pending
	// Assemble a spam order to penalize large transactors first
	spammers := prque.New[int64, common.Address](nil)
	for addr, list := range pool.pending {
		// Only evict transactions from high rollers
		if !pool.locals.contains(addr) && uint64(list.Len()) > pool.config.AccountSlots {
			spammers.Push(addr, int64(list.Len()))
		}
	}
	// Gradually drop transactions from offenders
	offenders := []common.Address{}
	for pending > pool.config.GlobalSlots && !spammers.Empty() {
		// Retrieve the next offender if not local address
		offender, _ := spammers.Pop()
		offenders = append(offenders, offender)

		// Equalize balances until all the same or below threshold
		if len(offenders) > 1 {
			// Calculate the equalization threshold for all current offenders
			threshold := pool.pending[offender].Len()

			// Iteratively reduce all offenders until below limit or threshold reached
			for pending > pool.config.GlobalSlots && pool.pending[offenders[len(offenders)-2]].Len() > threshold {
				for i := 0; i < len(offenders)-1; i++ {
					list := pool.pending[offenders[i]]

					caps := list.Cap(list.Len() - 1)
					for _, tx := range caps {
						// Drop the transaction from the global pools too
						hash := tx.Hash()
						pool.all.Remove(hash)

						// Update the account nonce to the dropped transaction
						pool.pendingNonces.setIfLower(offenders[i], tx.Nonce())
						log.Trace("Removed fairness-exceeding pending transaction", "hash", hash)
					}
					pool.priced.Removed(len(caps))
					pendingGauge.Dec(int64(len(caps)))
					if pool.locals.contains(offenders[i]) {
						localGauge.Dec(int64(len(caps)))
					}
					pending--
				}
			}
		}
	}

	// If still above threshold, reduce to limit or min allowance
	if pending > pool.config.GlobalSlots && len(offenders) > 0 {
		for pending > pool.config.GlobalSlots && uint64(pool.pending[offenders[len(offenders)-1]].Len()) > pool.config.AccountSlots {
			for _, addr := range offenders {
				list := pool.pending[addr]

				caps := list.Cap(list.Len() - 1)
				for _, tx := range caps {
					// Drop the transaction from the global pools too
					hash := tx.Hash()
					pool.all.Remove(hash)

					// Update the account nonce to the dropped transaction
					pool.pendingNonces.setIfLower(addr, tx.Nonce())
					log.Trace("Removed fairness-exceeding pending transaction", "hash", hash)
				}
				pool.priced.Removed(len(caps))
				pendingGauge.Dec(int64(len(caps)))
				if pool.locals.contains(addr) {
					localGauge.Dec(int64(len(caps)))
				}
				pending--
			}
		}
	}
	pendingRateLimitMeter.Mark(int64(pendingBeforeCap - pending))
}

// truncateQueue drops the oldest transactions in the queue if the pool is above the global queue limit.
func (pool *LegacyPool) truncateQueue() {
	queued := uint64(0)
	for _, list := range pool.queue {
		queued += uint64(list.Len())
	}
	if queued <= pool.config.GlobalQueue {
		return
	}

	// Sort all accounts with queued transactions by heartbeat
	addresses := make(addressesByHeartbeat, 0, len(pool.queue))
	for addr := range pool.queue {
		if !pool.locals.contains(addr) { // don't drop locals
			addresses = append(addresses, addressByHeartbeat{addr, pool.beats[addr]})
		}
	}
	sort.Sort(sort.Reverse(addresses))

	// Drop transactions until the total is below the limit or only locals remain
	for drop := queued - pool.config.GlobalQueue; drop > 0 && len(addresses) > 0; {
		addr := addresses[len(addresses)-1]
		list := pool.queue[addr.address]

		addresses = addresses[:len(addresses)-1]

		// Drop all transactions if they are less than the overflow
		if size := uint64(list.Len()); size <= drop {
			for _, tx := range list.Flatten() {
				pool.removeTx(tx.Hash(), true, true)
			}
			drop -= size
			queuedRateLimitMeter.Mark(int64(size))
			continue
		}
		// Otherwise drop only last few transactions
		txs := list.Flatten()
		for i := len(txs) - 1; i >= 0 && drop > 0; i-- {
			pool.removeTx(txs[i].Hash(), true, true)
			drop--
			queuedRateLimitMeter.Mark(1)
		}
	}
}

// demoteUnexecutables removes invalid and processed transactions from the pools
// executable/pending queue and any subsequent transactions that become unexecutable
// are moved back into the future queue.
//
// Note: transactions are not marked as removed in the priced list because re-heaping
// is always explicitly triggered by SetBaseFee and it would be unnecessary and wasteful
// to trigger a re-heap is this function
func (pool *LegacyPool) demoteUnexecutables() {
	// Iterate over all accounts and demote any non-executable transactions
	gasLimit := pool.currentHead.Load().GasLimit
	for addr, list := range pool.pending {
		nonce := pool.currentState.GetNonce(addr)

		// Drop all transactions that are deemed too old (low nonce)
		olds := list.Forward(nonce)
		for _, tx := range olds {
			hash := tx.Hash()
			pool.all.Remove(hash)
			log.Trace("Removed old pending transaction", "hash", hash)
		}
		// Drop all transactions that are too costly (low balance or out of gas), and queue any invalids back for later
		drops, invalids := list.Filter(pool.currentState.GetBalance(addr), gasLimit)
		for _, tx := range drops {
			hash := tx.Hash()
			log.Trace("Removed unpayable pending transaction", "hash", hash)
			pool.all.Remove(hash)
		}
		pendingNofundsMeter.Mark(int64(len(drops)))

		for _, tx := range invalids {
			hash := tx.Hash()
			log.Trace("Demoting pending transaction", "hash", hash)

			// Internal shuffle shouldn't touch the lookup set.
			pool.enqueueTx(hash, tx, false, false)
		}
		pendingGauge.Dec(int64(len(olds) + len(drops) + len(invalids)))
		if pool.locals.contains(addr) {
			localGauge.Dec(int64(len(olds) + len(drops) + len(invalids)))
		}
		// If there's a gap in front, alert (should never happen) and postpone all transactions
		if list.Len() > 0 && list.txs.Get(nonce) == nil {
			gapped := list.Cap(0)
			for _, tx := range gapped {
				hash := tx.Hash()
				log.Error("Demoting invalidated transaction", "hash", hash)

				// Internal shuffle shouldn't touch the lookup set.
				pool.enqueueTx(hash, tx, false, false)
			}
			pendingGauge.Dec(int64(len(gapped)))
		}
		// Delete the entire pending entry if it became empty.
		if list.Empty() {
			delete(pool.pending, addr)
			if _, ok := pool.queue[addr]; !ok {
				pool.reserve(addr, false)
			}
		}
	}
}

// addressByHeartbeat is an account address tagged with its last activity timestamp.
type addressByHeartbeat struct {
	address   common.Address
	heartbeat time.Time
}

type addressesByHeartbeat []addressByHeartbeat

func (a addressesByHeartbeat) Len() int           { return len(a) }
func (a addressesByHeartbeat) Less(i, j int) bool { return a[i].heartbeat.Before(a[j].heartbeat) }
func (a addressesByHeartbeat) Swap(i, j int)      { a[i], a[j] = a[j], a[i] }

// accountSet is simply a set of addresses to check for existence, and a signer
// capable of deriving addresses from transactions.
type accountSet struct {
	accounts map[common.Address]struct{}
	signer   types.Signer
	cache    *[]common.Address
}

// newAccountSet creates a new address set with an associated signer for sender
// derivations.
func newAccountSet(signer types.Signer, addrs ...common.Address) *accountSet {
	as := &accountSet{
		accounts: make(map[common.Address]struct{}, len(addrs)),
		signer:   signer,
	}
	for _, addr := range addrs {
		as.add(addr)
	}
	return as
}

// contains checks if a given address is contained within the set.
func (as *accountSet) contains(addr common.Address) bool {
	_, exist := as.accounts[addr]
	return exist
}

// containsTx checks if the sender of a given tx is within the set. If the sender
// cannot be derived, this method returns false.
func (as *accountSet) containsTx(tx *types.Transaction) bool {
	if addr, err := types.Sender(as.signer, tx); err == nil {
		return as.contains(addr)
	}
	return false
}

// add inserts a new address into the set to track.
func (as *accountSet) add(addr common.Address) {
	as.accounts[addr] = struct{}{}
	as.cache = nil
}

// addTx adds the sender of tx into the set.
func (as *accountSet) addTx(tx *types.Transaction) {
	if addr, err := types.Sender(as.signer, tx); err == nil {
		as.add(addr)
	}
}

// flatten returns the list of addresses within this set, also caching it for later
// reuse. The returned slice should not be changed!
func (as *accountSet) flatten() []common.Address {
	if as.cache == nil {
		accounts := make([]common.Address, 0, len(as.accounts))
		for account := range as.accounts {
			accounts = append(accounts, account)
		}
		as.cache = &accounts
	}
	return *as.cache
}

// merge adds all addresses from the 'other' set into 'as'.
func (as *accountSet) merge(other *accountSet) {
	for addr := range other.accounts {
		as.accounts[addr] = struct{}{}
	}
	as.cache = nil
}

// lookup is used internally by LegacyPool to track transactions while allowing
// lookup without mutex contention.
//
// Note, although this type is properly protected against concurrent access, it
// is **not** a type that should ever be mutated or even exposed outside of the
// transaction pool, since its internal state is tightly coupled with the pools
// internal mechanisms. The sole purpose of the type is to permit out-of-bound
// peeking into the pool in LegacyPool.Get without having to acquire the widely scoped
// LegacyPool.mu mutex.
//
// This lookup set combines the notion of "local transactions", which is useful
// to build upper-level structure.
type lookup struct {
	slots   int
	lock    sync.RWMutex
	locals  map[common.Hash]*types.Transaction
	remotes map[common.Hash]*types.Transaction
}

// newLookup returns a new lookup structure.
func newLookup() *lookup {
	return &lookup{
		locals:  make(map[common.Hash]*types.Transaction),
		remotes: make(map[common.Hash]*types.Transaction),
	}
}

// Range calls f on each key and value present in the map. The callback passed
// should return the indicator whether the iteration needs to be continued.
// Callers need to specify which set (or both) to be iterated.
func (t *lookup) Range(f func(hash common.Hash, tx *types.Transaction, local bool) bool, local bool, remote bool) {
	t.lock.RLock()
	defer t.lock.RUnlock()

	if local {
		for key, value := range t.locals {
			if !f(key, value, true) {
				return
			}
		}
	}
	if remote {
		for key, value := range t.remotes {
			if !f(key, value, false) {
				return
			}
		}
	}
}

// Get returns a transaction if it exists in the lookup, or nil if not found.
func (t *lookup) Get(hash common.Hash) *types.Transaction {
	t.lock.RLock()
	defer t.lock.RUnlock()

	if tx := t.locals[hash]; tx != nil {
		return tx
	}
	return t.remotes[hash]
}

// GetLocal returns a transaction if it exists in the lookup, or nil if not found.
func (t *lookup) GetLocal(hash common.Hash) *types.Transaction {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.locals[hash]
}

// GetRemote returns a transaction if it exists in the lookup, or nil if not found.
func (t *lookup) GetRemote(hash common.Hash) *types.Transaction {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.remotes[hash]
}

// Count returns the current number of transactions in the lookup.
func (t *lookup) Count() int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return len(t.locals) + len(t.remotes)
}

// LocalCount returns the current number of local transactions in the lookup.
func (t *lookup) LocalCount() int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return len(t.locals)
}

// RemoteCount returns the current number of remote transactions in the lookup.
func (t *lookup) RemoteCount() int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return len(t.remotes)
}

// Slots returns the current number of slots used in the lookup.
func (t *lookup) Slots() int {
	t.lock.RLock()
	defer t.lock.RUnlock()

	return t.slots
}

// Add adds a transaction to the lookup.
func (t *lookup) Add(tx *types.Transaction, local bool) {
	t.lock.Lock()
	defer t.lock.Unlock()

	t.slots += numSlots(tx)
	slotsGauge.Update(int64(t.slots))

	if local {
		t.locals[tx.Hash()] = tx
	} else {
		t.remotes[tx.Hash()] = tx
	}
}

// Remove removes a transaction from the lookup.
func (t *lookup) Remove(hash common.Hash) {
	t.lock.Lock()
	defer t.lock.Unlock()

	tx, ok := t.locals[hash]
	if !ok {
		tx, ok = t.remotes[hash]
	}
	if !ok {
		log.Error("No transaction found to be deleted", "hash", hash)
		return
	}
	t.slots -= numSlots(tx)
	slotsGauge.Update(int64(t.slots))

	delete(t.locals, hash)
	delete(t.remotes, hash)
}

// RemoteToLocals migrates the transactions belongs to the given locals to locals
// set. The assumption is held the locals set is thread-safe to be used.
func (t *lookup) RemoteToLocals(locals *accountSet) int {
	t.lock.Lock()
	defer t.lock.Unlock()

	var migrated int
	for hash, tx := range t.remotes {
		if locals.containsTx(tx) {
			t.locals[hash] = tx
			delete(t.remotes, hash)
			migrated += 1
		}
	}
	return migrated
}

// RemotesBelowTip finds all remote transactions below the given tip threshold.
func (t *lookup) RemotesBelowTip(threshold *big.Int) types.Transactions {
	found := make(types.Transactions, 0, 128)
	t.Range(func(hash common.Hash, tx *types.Transaction, local bool) bool {
		if tx.GasTipCapIntCmp(threshold) < 0 {
			found = append(found, tx)
		}
		return true
	}, false, true) // Only iterate remotes
	return found
}

// numSlots calculates the number of slots needed for a single transaction.
func numSlots(tx *types.Transaction) int {
	return int((tx.Size() + txSlotSize - 1) / txSlotSize)
}
