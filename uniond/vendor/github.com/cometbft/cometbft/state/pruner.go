package state

import (
	"errors"
	"sync"
	"time"

	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/service"
	"github.com/cometbft/cometbft/state/indexer"
	"github.com/cometbft/cometbft/state/txindex"
)

var (
	AppRetainHeightKey            = []byte("AppRetainHeightKey")
	CompanionBlockRetainHeightKey = []byte("DCBlockRetainHeightKey")
	ABCIResultsRetainHeightKey    = []byte("ABCIResRetainHeightKey")
)

// Pruner is a service that reads the retain heights for blocks, state and ABCI
// results from the database and prunes the corresponding data based on the
// minimum retain height set. The service sleeps between each run based on the
// configured pruner interval, and re-evaluates the retain height.
type Pruner struct {
	service.BaseService
	logger log.Logger

	mtx sync.Mutex
	// Must the pruner respect the retain heights set by the data companion?
	dcEnabled bool
	// DB to which we save the retain heights
	bs BlockStore
	// State store to prune state from
	stateStore   Store
	blockIndexer indexer.BlockIndexer
	txIndexer    txindex.TxIndexer
	interval     time.Duration
	observer     PrunerObserver
	metrics      *Metrics

	// Preserve the number of state entries pruned.
	// Used to calculated correctly when to trigger compactions
	prunedStates uint64
}

type prunerConfig struct {
	dcEnabled bool
	interval  time.Duration
	observer  PrunerObserver
	metrics   *Metrics
}

func defaultPrunerConfig() *prunerConfig {
	return &prunerConfig{
		dcEnabled: false,
		interval:  config.DefaultPruningInterval,
		observer:  &NoopPrunerObserver{},
		metrics:   NopMetrics(),
	}
}

type PrunerOption func(*prunerConfig)

// WithPrunerCompanionEnabled indicates to the pruner that it must respect the
// retain heights set by the data companion. By default, if this option is not
// supplied, the pruner will ignore any retain heights set by the data
// companion.
func WithPrunerCompanionEnabled() PrunerOption {
	return func(p *prunerConfig) {
		p.dcEnabled = true
	}
}

// WithPrunerInterval allows control over the interval between each run of the
// pruner.
func WithPrunerInterval(t time.Duration) PrunerOption {
	return func(p *prunerConfig) { p.interval = t }
}

func WithPrunerObserver(obs PrunerObserver) PrunerOption {
	return func(p *prunerConfig) { p.observer = obs }
}

func WithPrunerMetrics(metrics *Metrics) PrunerOption {
	return func(p *prunerConfig) {
		p.metrics = metrics
	}
}

// NewPruner creates a service that controls background pruning of node data.
//
// Assumes that the initial application and data companion retain heights have
// already been configured in the state store.
func NewPruner(
	stateStore Store,
	bs BlockStore,
	blockIndexer indexer.BlockIndexer,
	txIndexer txindex.TxIndexer,
	logger log.Logger,
	options ...PrunerOption,
) *Pruner {
	cfg := defaultPrunerConfig()
	for _, opt := range options {
		opt(cfg)
	}
	p := &Pruner{
		bs:           bs,
		txIndexer:    txIndexer,
		blockIndexer: blockIndexer,
		stateStore:   stateStore,
		logger:       logger,
		interval:     cfg.interval,
		observer:     cfg.observer,
		metrics:      cfg.metrics,
		dcEnabled:    cfg.dcEnabled,
	}
	p.BaseService = *service.NewBaseService(logger, "Pruner", p)
	return p
}

func (p *Pruner) SetObserver(obs PrunerObserver) {
	p.observer = obs
}

func (p *Pruner) OnStart() error {
	go p.pruneBlocks()
	// We only care about pruning ABCI results if the data companion has been
	// enabled.
	if p.dcEnabled {
		go p.pruneABCIResponses()
		go p.pruneIndexesRoutine()
	}
	p.observer.PrunerStarted(p.interval)
	return nil
}

// SetApplicationBlockRetainHeight sets the application block retain height
// with some basic checks on the requested height.
//
// If a higher retain height is already set, we cannot accept the requested
// height because the blocks might have been pruned.
func (p *Pruner) SetApplicationBlockRetainHeight(height int64) error {
	// Ensure that all requests to set retain heights via the application are
	// serialized.
	p.mtx.Lock()
	defer p.mtx.Unlock()

	if !p.heightWithinBounds(height) {
		return ErrInvalidHeightValue
	}
	curRetainHeight, err := p.stateStore.GetApplicationRetainHeight()
	if err != nil {
		return ErrPrunerFailedToGetRetainHeight{Which: "application block", Err: err}
	}
	if height < curRetainHeight {
		return ErrPrunerCannotLowerRetainHeight
	}
	if err := p.stateStore.SaveApplicationRetainHeight(height); err != nil {
		return err
	}
	p.metrics.ApplicationBlockRetainHeight.Set(float64(height))
	return nil
}

func (p *Pruner) heightWithinBounds(height int64) bool {
	if height < p.bs.Base() || height > p.bs.Height() {
		return false
	}
	return true
}

// SetCompanionBlockRetainHeight sets the application block retain height with
// some basic checks on the requested height.
//
// If a higher retain height is already set, we cannot accept the requested
// height because the blocks might have been pruned.
func (p *Pruner) SetCompanionBlockRetainHeight(height int64) error {
	// Ensure that all requests to set retain heights via the pruner are
	// serialized.
	p.mtx.Lock()
	defer p.mtx.Unlock()

	if !p.heightWithinBounds(height) {
		return ErrInvalidHeightValue
	}
	curRetainHeight, err := p.stateStore.GetCompanionBlockRetainHeight()
	if err != nil {
		return ErrPrunerFailedToGetRetainHeight{Which: "companion block", Err: err}
	}
	if height < curRetainHeight {
		return ErrPrunerCannotLowerRetainHeight
	}
	if err := p.stateStore.SaveCompanionBlockRetainHeight(height); err != nil {
		return err
	}
	p.metrics.PruningServiceBlockRetainHeight.Set(float64(height))
	return nil
}

// SetABCIResRetainHeight sets the retain height for ABCI responses.
//
// If the application has set the DiscardABCIResponses flag to true, nothing
// will be pruned.
func (p *Pruner) SetABCIResRetainHeight(height int64) error {
	// Ensure that all requests to set retain heights via the pruner are
	// serialized.
	p.mtx.Lock()
	defer p.mtx.Unlock()

	if !p.heightWithinBounds(height) {
		return ErrInvalidHeightValue
	}
	curRetainHeight, err := p.stateStore.GetABCIResRetainHeight()
	if err != nil {
		return ErrPrunerFailedToGetRetainHeight{Which: "ABCI results", Err: err}
	}
	if height < curRetainHeight {
		return ErrPrunerCannotLowerRetainHeight
	}
	if err := p.stateStore.SaveABCIResRetainHeight(height); err != nil {
		return err
	}
	p.metrics.PruningServiceBlockResultsRetainHeight.Set(float64(height))
	return nil
}

func (p *Pruner) SetTxIndexerRetainHeight(height int64) error {
	// Ensure that all requests to set retain heights via the application are
	// serialized.
	p.mtx.Lock()
	defer p.mtx.Unlock()
	if height <= 0 {
		return ErrInvalidHeightValue
	}

	currentRetainHeight, err := p.txIndexer.GetRetainHeight()
	if err != nil {
		if !errors.Is(err, ErrKeyNotFound) {
			return err
		}
		return p.txIndexer.SetRetainHeight(height)
	}
	if currentRetainHeight > height {
		return ErrPrunerCannotLowerRetainHeight
	}
	if err := p.txIndexer.SetRetainHeight(height); err != nil {
		return err
	}
	p.metrics.PruningServiceTxIndexerRetainHeight.Set(float64(height))
	return nil
}

func (p *Pruner) SetBlockIndexerRetainHeight(height int64) error {
	// Ensure that all requests to set retain heights via the application are
	// serialized.
	p.mtx.Lock()
	defer p.mtx.Unlock()
	if height <= 0 {
		return ErrInvalidHeightValue
	}

	currentRetainHeight, err := p.blockIndexer.GetRetainHeight()
	if err != nil {
		if !errors.Is(err, ErrKeyNotFound) {
			return err
		}
		return p.blockIndexer.SetRetainHeight(height)
	}
	if currentRetainHeight > height {
		return ErrPrunerCannotLowerRetainHeight
	}
	if err := p.blockIndexer.SetRetainHeight(height); err != nil {
		return err
	}
	p.metrics.PruningServiceBlockIndexerRetainHeight.Set(float64(height))
	return nil
}

// GetApplicationRetainHeight is a convenience method for accessing the
// GetApplicationRetainHeight method of the underlying state store.
func (p *Pruner) GetApplicationRetainHeight() (int64, error) {
	return p.stateStore.GetApplicationRetainHeight()
}

// GetCompanionBlockRetainHeight is a convenience method for accessing the
// GetCompanionBlockRetainHeight method of the underlying state store.
func (p *Pruner) GetCompanionBlockRetainHeight() (int64, error) {
	return p.stateStore.GetCompanionBlockRetainHeight()
}

// GetABCIResRetainHeight is a convenience method for accessing the
// GetABCIResRetainHeight method of the underlying state store.
func (p *Pruner) GetABCIResRetainHeight() (int64, error) {
	return p.stateStore.GetABCIResRetainHeight()
}

// GetTxIndexerRetainHeight is a convenience method for accessing the
// GetTxIndexerRetainHeight method of the underlying indexer.
func (p *Pruner) GetTxIndexerRetainHeight() (int64, error) {
	return p.txIndexer.GetRetainHeight()
}

// GetBlockIndexerRetainHeight is a convenience method for accessing the
// GetBlockIndexerRetainHeight method of the underlying state store.
func (p *Pruner) GetBlockIndexerRetainHeight() (int64, error) {
	return p.blockIndexer.GetRetainHeight()
}

func (p *Pruner) pruneABCIResponses() {
	p.logger.Info("Started pruning ABCI responses", "interval", p.interval.String())
	lastRetainHeight := int64(0)
	for {
		select {
		case <-p.Quit():
			return
		default:
			newRetainHeight := p.pruneABCIResToRetainHeight(lastRetainHeight)
			if newRetainHeight != lastRetainHeight {
				p.observer.PrunerPrunedABCIRes(&ABCIResponsesPrunedInfo{
					FromHeight: lastRetainHeight,
					ToHeight:   newRetainHeight - 1,
				})
			}
			lastRetainHeight = newRetainHeight
			time.Sleep(p.interval)
		}
	}
}

func (p *Pruner) pruneBlocks() {
	p.logger.Info("Started pruning blocks", "interval", p.interval.String())
	lastRetainHeight := int64(0)
	for {
		select {
		case <-p.Quit():
			return
		default:
			newRetainHeight := p.pruneBlocksToRetainHeight(lastRetainHeight)
			if newRetainHeight != lastRetainHeight {
				p.observer.PrunerPrunedBlocks(&BlocksPrunedInfo{
					FromHeight: lastRetainHeight,
					ToHeight:   newRetainHeight - 1,
				})
			}
			lastRetainHeight = newRetainHeight
			time.Sleep(p.interval)
		}
	}
}

func (p *Pruner) pruneIndexesRoutine() {
	p.logger.Info("Index pruner started", "interval", p.interval.String())
	lastTxIndexerRetainHeight := int64(0)
	lastBlockIndexerRetainHeight := int64(0)
	for {
		select {
		case <-p.Quit():
			return
		default:
			lastTxIndexerRetainHeight = p.pruneTxIndexerToRetainHeight(lastTxIndexerRetainHeight)
			lastBlockIndexerRetainHeight = p.pruneBlockIndexerToRetainHeight(lastBlockIndexerRetainHeight)
			// TODO call observer
			time.Sleep(p.interval)
		}
	}
}

func (p *Pruner) pruneTxIndexerToRetainHeight(lastRetainHeight int64) int64 {
	targetRetainHeight, err := p.GetTxIndexerRetainHeight()
	if err != nil {
		// Indexer retain height has not yet been set - do not log any
		// errors at this time.
		if errors.Is(err, ErrKeyNotFound) {
			return 0
		}
		p.logger.Error("Failed to get Indexer retain height", "err", err)
		return lastRetainHeight
	}

	if lastRetainHeight >= targetRetainHeight {
		return lastRetainHeight
	}

	numPrunedTxIndexer, newTxIndexerRetainHeight, err := p.txIndexer.Prune(targetRetainHeight)
	if err != nil {
		p.logger.Error("Failed to prune tx indexer", "err", err, "targetRetainHeight", targetRetainHeight, "newTxIndexerRetainHeight", newTxIndexerRetainHeight)
	} else if numPrunedTxIndexer > 0 {
		p.metrics.TxIndexerBaseHeight.Set(float64(newTxIndexerRetainHeight))
		p.logger.Debug("Pruned tx indexer", "count", numPrunedTxIndexer, "newTxIndexerRetainHeight", newTxIndexerRetainHeight)
	}
	return newTxIndexerRetainHeight
}

func (p *Pruner) pruneBlockIndexerToRetainHeight(lastRetainHeight int64) int64 {
	targetRetainHeight, err := p.GetBlockIndexerRetainHeight()
	if err != nil {
		// Indexer retain height has not yet been set - do not log any
		// errors at this time.
		if errors.Is(err, ErrKeyNotFound) {
			return 0
		}
		p.logger.Error("Failed to get Indexer retain height", "err", err)
		return lastRetainHeight
	}

	if lastRetainHeight >= targetRetainHeight {
		return lastRetainHeight
	}

	numPrunedBlockIndexer, newBlockIndexerRetainHeight, err := p.blockIndexer.Prune(targetRetainHeight)
	if err != nil {
		p.logger.Error("Failed to prune block indexer", "err", err, "targetRetainHeight", targetRetainHeight, "newBlockIndexerRetainHeight", newBlockIndexerRetainHeight)
	} else if numPrunedBlockIndexer > 0 {
		p.metrics.BlockIndexerBaseHeight.Set(float64(newBlockIndexerRetainHeight))
		p.logger.Debug("Pruned block indexer", "count", numPrunedBlockIndexer, "newBlockIndexerRetainHeight", newBlockIndexerRetainHeight)
	}
	return newBlockIndexerRetainHeight
}

func (p *Pruner) pruneBlocksToRetainHeight(lastRetainHeight int64) int64 {
	targetRetainHeight := p.findMinBlockRetainHeight()
	if targetRetainHeight == lastRetainHeight {
		return lastRetainHeight
	}
	pruned, evRetainHeight, err := p.pruneBlocksToHeight(targetRetainHeight)
	// The new retain height is the current lowest point of the block store
	// indicated by Base()
	newRetainHeight := p.bs.Base()
	if err != nil {
		p.logger.Error("Failed to prune blocks", "err", err, "targetRetainHeight", targetRetainHeight, "newRetainHeight", newRetainHeight)
	} else if pruned > 0 {
		p.metrics.BlockStoreBaseHeight.Set(float64(newRetainHeight))
		p.logger.Debug("Pruned blocks", "count", pruned, "evidenceRetainHeight", evRetainHeight, "newRetainHeight", newRetainHeight)
	}
	return newRetainHeight
}

func (p *Pruner) pruneABCIResToRetainHeight(lastRetainHeight int64) int64 {
	targetRetainHeight, err := p.stateStore.GetABCIResRetainHeight()
	if err != nil {
		p.logger.Error("Failed to get ABCI response retain height", "err", err)
		if errors.Is(err, ErrKeyNotFound) {
			return 0
		}
		return lastRetainHeight
	}

	if lastRetainHeight == targetRetainHeight {
		return lastRetainHeight
	}

	// If the block retain height is 0, pruning of the block and state stores might be disabled
	// This should not prevent Comet from pruning ABCI results if needed.
	// We could by default always compact when pruning the responses, but in case the state store
	// is being compacted we introduce an overhead that might cause performance penalties.
	forceCompact := p.findMinBlockRetainHeight() == 0
	// newRetainHeight is the height just after that which we have successfully
	// pruned. In case of an error it will be 0, but then it will also be
	// ignored.
	numPruned, newRetainHeight, err := p.stateStore.PruneABCIResponses(targetRetainHeight, forceCompact)
	if err != nil {
		p.logger.Error("Failed to prune ABCI responses", "err", err, "targetRetainHeight", targetRetainHeight)
		return lastRetainHeight
	}
	if numPruned > 0 {
		p.logger.Info("Pruned ABCI responses", "heights", numPruned, "newRetainHeight", newRetainHeight)
		p.metrics.ABCIResultsBaseHeight.Set(float64(newRetainHeight))
	}
	return newRetainHeight
}

func (p *Pruner) findMinBlockRetainHeight() int64 {
	appRetainHeight, err := p.stateStore.GetApplicationRetainHeight()
	if err != nil {
		p.logger.Error("Unexpected error fetching application retain height", "err", err)
		return 0
	}
	// We only care about the companion retain height if pruning is configured
	// to respect the companion's retain height.
	if !p.dcEnabled {
		return appRetainHeight
	}
	dcRetainHeight, err := p.stateStore.GetCompanionBlockRetainHeight()
	if err != nil {
		p.logger.Error("Unexpected error fetching data companion retain height", "err", err)
		return 0
	}
	// If we are here, both heights were set and the companion is enabled, so
	// we pick the minimum.
	if appRetainHeight < dcRetainHeight {
		return appRetainHeight
	}
	return dcRetainHeight
}

func (p *Pruner) pruneBlocksToHeight(height int64) (uint64, int64, error) {
	if height <= 0 {
		return 0, 0, ErrInvalidRetainHeight
	}

	base := p.bs.Base()

	state, err := p.stateStore.Load()
	if err != nil {
		return 0, 0, ErrPrunerFailedToLoadState{Err: err}
	}
	pruned, evRetainHeight, err := p.bs.PruneBlocks(height, state)
	if err != nil {
		return 0, 0, ErrFailedToPruneBlocks{Height: height, Err: err}
	}
	if pruned > 0 {
		prunedStates, err := p.stateStore.PruneStates(base, height, evRetainHeight, p.prunedStates)
		p.prunedStates += prunedStates
		if err != nil {
			return 0, 0, ErrFailedToPruneStates{Height: height, Err: err}
		}
	}
	return pruned, evRetainHeight, err
}
