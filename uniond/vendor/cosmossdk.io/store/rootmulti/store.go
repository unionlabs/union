package rootmulti

import (
	"crypto/sha256"
	"errors"
	"fmt"
	"io"
	"math"
	"sort"
	"strings"
	"sync"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	protoio "github.com/cosmos/gogoproto/io"
	gogotypes "github.com/cosmos/gogoproto/types"
	iavltree "github.com/cosmos/iavl"

	corestore "cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/store/cachemulti"
	dbm "cosmossdk.io/store/db"
	"cosmossdk.io/store/dbadapter"
	"cosmossdk.io/store/iavl"
	"cosmossdk.io/store/listenkv"
	"cosmossdk.io/store/mem"
	"cosmossdk.io/store/metrics"
	"cosmossdk.io/store/pruning"
	pruningtypes "cosmossdk.io/store/pruning/types"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	"cosmossdk.io/store/tracekv"
	"cosmossdk.io/store/transient"
	"cosmossdk.io/store/types"
)

const (
	latestVersionKey = "s/latest"
	commitInfoKeyFmt = "s/%d" // s/<version>
)

const iavlDisablefastNodeDefault = false

// keysFromStoreKeyMap returns a slice of keys for the provided map lexically sorted by StoreKey.Name()
func keysFromStoreKeyMap[V any](m map[types.StoreKey]V) []types.StoreKey {
	keys := make([]types.StoreKey, 0, len(m))
	for key := range m {
		keys = append(keys, key)
	}
	sort.Slice(keys, func(i, j int) bool {
		ki, kj := keys[i], keys[j]
		return ki.Name() < kj.Name()
	})
	return keys
}

// Store is composed of many CommitStores. Name contrasts with
// cacheMultiStore which is used for branching other MultiStores. It implements
// the CommitMultiStore interface.
type Store struct {
	db                  corestore.KVStoreWithBatch
	logger              iavltree.Logger
	lastCommitInfo      *types.CommitInfo
	pruningManager      *pruning.Manager
	iavlCacheSize       int
	iavlDisableFastNode bool
	storesParams        map[types.StoreKey]storeParams
	stores              map[types.StoreKey]types.CommitKVStore
	keysByName          map[string]types.StoreKey
	initialVersion      int64
	removalMap          map[types.StoreKey]bool
	traceWriter         io.Writer
	traceContext        types.TraceContext
	traceContextMutex   sync.Mutex
	interBlockCache     types.MultiStorePersistentCache
	listeners           map[types.StoreKey]*types.MemoryListener
	metrics             metrics.StoreMetrics
	commitHeader        cmtproto.Header
}

var (
	_ types.CommitMultiStore = (*Store)(nil)
	_ types.Queryable        = (*Store)(nil)
)

// NewStore returns a reference to a new Store object with the provided DB. The
// store will be created with a PruneNothing pruning strategy by default. After
// a store is created, KVStores must be mounted and finally LoadLatestVersion or
// LoadVersion must be called.
func NewStore(db corestore.KVStoreWithBatch, logger iavltree.Logger, metricGatherer metrics.StoreMetrics) *Store {
	return &Store{
		db:                  db,
		logger:              logger,
		iavlCacheSize:       iavl.DefaultIAVLCacheSize,
		iavlDisableFastNode: iavlDisablefastNodeDefault,
		storesParams:        make(map[types.StoreKey]storeParams),
		stores:              make(map[types.StoreKey]types.CommitKVStore),
		keysByName:          make(map[string]types.StoreKey),
		listeners:           make(map[types.StoreKey]*types.MemoryListener),
		removalMap:          make(map[types.StoreKey]bool),
		pruningManager:      pruning.NewManager(db, logger),
		metrics:             metricGatherer,
	}
}

// GetPruning fetches the pruning strategy from the root store.
func (rs *Store) GetPruning() pruningtypes.PruningOptions {
	return rs.pruningManager.GetOptions()
}

// SetPruning sets the pruning strategy on the root store and all the sub-stores.
// Note, calling SetPruning on the root store prior to LoadVersion or
// LoadLatestVersion performs a no-op as the stores aren't mounted yet.
func (rs *Store) SetPruning(pruningOpts pruningtypes.PruningOptions) {
	rs.pruningManager.SetOptions(pruningOpts)
}

// SetMetrics sets the metrics gatherer for the store package
func (rs *Store) SetMetrics(metrics metrics.StoreMetrics) {
	rs.metrics = metrics
}

// SetSnapshotInterval sets the interval at which the snapshots are taken.
// It is used by the store to determine which heights to retain until after the snapshot is complete.
func (rs *Store) SetSnapshotInterval(snapshotInterval uint64) {
	rs.pruningManager.SetSnapshotInterval(snapshotInterval)
}

func (rs *Store) SetIAVLCacheSize(cacheSize int) {
	rs.iavlCacheSize = cacheSize
}

func (rs *Store) SetIAVLDisableFastNode(disableFastNode bool) {
	rs.iavlDisableFastNode = disableFastNode
}

// GetStoreType implements Store.
func (rs *Store) GetStoreType() types.StoreType {
	return types.StoreTypeMulti
}

// MountStoreWithDB implements CommitMultiStore.
func (rs *Store) MountStoreWithDB(key types.StoreKey, typ types.StoreType, db corestore.KVStoreWithBatch) {
	if key == nil {
		panic("MountIAVLStore() key cannot be nil")
	}
	if _, ok := rs.storesParams[key]; ok {
		panic(fmt.Sprintf("store duplicate store key %v", key))
	}
	if _, ok := rs.keysByName[key.Name()]; ok {
		panic(fmt.Sprintf("store duplicate store key name %v", key))
	}
	rs.storesParams[key] = newStoreParams(key, db, typ, 0)
	rs.keysByName[key.Name()] = key
}

// GetCommitStore returns a mounted CommitStore for a given StoreKey. If the
// store is wrapped in an inter-block cache, it will be unwrapped before returning.
func (rs *Store) GetCommitStore(key types.StoreKey) types.CommitStore {
	return rs.GetCommitKVStore(key)
}

// GetCommitKVStore returns a mounted CommitKVStore for a given StoreKey. If the
// store is wrapped in an inter-block cache, it will be unwrapped before returning.
func (rs *Store) GetCommitKVStore(key types.StoreKey) types.CommitKVStore {
	// If the Store has an inter-block cache, first attempt to lookup and unwrap
	// the underlying CommitKVStore by StoreKey. If it does not exist, fallback to
	// the main mapping of CommitKVStores.
	if rs.interBlockCache != nil {
		if store := rs.interBlockCache.Unwrap(key); store != nil {
			return store
		}
	}

	return rs.stores[key]
}

// StoreKeysByName returns mapping storeNames -> StoreKeys
func (rs *Store) StoreKeysByName() map[string]types.StoreKey {
	return rs.keysByName
}

// LoadLatestVersionAndUpgrade implements CommitMultiStore
func (rs *Store) LoadLatestVersionAndUpgrade(upgrades *types.StoreUpgrades) error {
	ver := GetLatestVersion(rs.db)
	return rs.loadVersion(ver, upgrades)
}

// LoadVersionAndUpgrade allows us to rename substores while loading an older version
func (rs *Store) LoadVersionAndUpgrade(ver int64, upgrades *types.StoreUpgrades) error {
	return rs.loadVersion(ver, upgrades)
}

// LoadLatestVersion implements CommitMultiStore.
func (rs *Store) LoadLatestVersion() error {
	ver := GetLatestVersion(rs.db)
	return rs.loadVersion(ver, nil)
}

// LoadVersion implements CommitMultiStore.
func (rs *Store) LoadVersion(ver int64) error {
	return rs.loadVersion(ver, nil)
}

func (rs *Store) loadVersion(ver int64, upgrades *types.StoreUpgrades) error {
	infos := make(map[string]types.StoreInfo)

	rs.logger.Debug("loadVersion", "ver", ver)
	cInfo := &types.CommitInfo{}

	// load old data if we are not version 0
	if ver != 0 {
		var err error
		cInfo, err = rs.GetCommitInfo(ver)
		if err != nil {
			return err
		}

		// convert StoreInfos slice to map
		for _, storeInfo := range cInfo.StoreInfos {
			infos[storeInfo.Name] = storeInfo
		}
	}

	// load each Store (note this doesn't panic on unmounted keys now)
	newStores := make(map[types.StoreKey]types.CommitKVStore)

	storesKeys := make([]types.StoreKey, 0, len(rs.storesParams))

	for key := range rs.storesParams {
		storesKeys = append(storesKeys, key)
	}

	if upgrades != nil {
		// deterministic iteration order for upgrades
		// (as the underlying store may change and
		// upgrades make store changes where the execution order may matter)
		sort.Slice(storesKeys, func(i, j int) bool {
			return storesKeys[i].Name() < storesKeys[j].Name()
		})
	}

	for _, key := range storesKeys {
		storeParams := rs.storesParams[key]
		commitID := rs.getCommitID(infos, key.Name())
		rs.logger.Debug("loadVersion commitID", "key", key, "ver", ver, "hash", fmt.Sprintf("%x", commitID.Hash))

		// If it has been added, set the initial version
		if upgrades.IsAdded(key.Name()) || upgrades.RenamedFrom(key.Name()) != "" {
			storeParams.initialVersion = uint64(ver) + 1
		} else if commitID.Version != ver && storeParams.typ == types.StoreTypeIAVL {
			return fmt.Errorf("version of store %q mismatch root store's version; expected %d got %d; new stores should be added using StoreUpgrades", key.Name(), ver, commitID.Version)
		}

		store, err := rs.loadCommitStoreFromParams(key, commitID, storeParams)
		if err != nil {
			return errorsmod.Wrap(err, "failed to load store")
		}

		newStores[key] = store

		// If it was deleted, remove all data
		if upgrades.IsDeleted(key.Name()) {
			if err := deleteKVStore(store.(types.KVStore)); err != nil {
				return errorsmod.Wrapf(err, "failed to delete store %s", key.Name())
			}
			rs.removalMap[key] = true
		} else if oldName := upgrades.RenamedFrom(key.Name()); oldName != "" {
			// handle renames specially
			// make an unregistered key to satisfy loadCommitStore params
			oldKey := types.NewKVStoreKey(oldName)
			oldParams := newStoreParams(oldKey, storeParams.db, storeParams.typ, 0)

			// load from the old name
			oldStore, err := rs.loadCommitStoreFromParams(oldKey, rs.getCommitID(infos, oldName), oldParams)
			if err != nil {
				return errorsmod.Wrapf(err, "failed to load old store %s", oldName)
			}

			// move all data
			if err := moveKVStoreData(oldStore.(types.KVStore), store.(types.KVStore)); err != nil {
				return errorsmod.Wrapf(err, "failed to move store %s -> %s", oldName, key.Name())
			}

			// add the old key so its deletion is committed
			newStores[oldKey] = oldStore
			// this will ensure it's not perpetually stored in commitInfo
			rs.removalMap[oldKey] = true
		}
	}

	rs.lastCommitInfo = cInfo
	rs.stores = newStores

	// load any snapshot heights we missed from disk to be pruned on the next run
	if err := rs.pruningManager.LoadSnapshotHeights(rs.db); err != nil {
		return err
	}

	return nil
}

func (rs *Store) getCommitID(infos map[string]types.StoreInfo, name string) types.CommitID {
	info, ok := infos[name]
	if !ok {
		return types.CommitID{}
	}

	return info.CommitId
}

func deleteKVStore(kv types.KVStore) error {
	// Note that we cannot write while iterating, so load all keys here, delete below
	var keys [][]byte
	itr := kv.Iterator(nil, nil)
	for itr.Valid() {
		keys = append(keys, itr.Key())
		itr.Next()
	}
	if err := itr.Close(); err != nil {
		return err
	}

	for _, k := range keys {
		kv.Delete(k)
	}
	return nil
}

// we simulate move by a copy and delete
func moveKVStoreData(oldDB, newDB types.KVStore) error {
	// we read from one and write to another
	itr := oldDB.Iterator(nil, nil)
	for itr.Valid() {
		newDB.Set(itr.Key(), itr.Value())
		itr.Next()
	}
	if err := itr.Close(); err != nil {
		return err
	}

	// then delete the old store
	return deleteKVStore(oldDB)
}

// PruneSnapshotHeight prunes the given height according to the prune strategy.
// If the strategy is PruneNothing, this is a no-op.
// For other strategies, this height is persisted until the snapshot is operated.
func (rs *Store) PruneSnapshotHeight(height int64) {
	rs.pruningManager.HandleSnapshotHeight(height)
}

// SetInterBlockCache sets the Store's internal inter-block (persistent) cache.
// When this is defined, all CommitKVStores will be wrapped with their respective
// inter-block cache.
func (rs *Store) SetInterBlockCache(c types.MultiStorePersistentCache) {
	rs.interBlockCache = c
}

// SetTracer sets the tracer for the MultiStore that the underlying
// stores will utilize to trace operations. A MultiStore is returned.
func (rs *Store) SetTracer(w io.Writer) types.MultiStore {
	rs.traceWriter = w
	return rs
}

// SetTracingContext updates the tracing context for the MultiStore by merging
// the given context with the existing context by key. Any existing keys will
// be overwritten. It is implied that the caller should update the context when
// necessary between tracing operations. It returns a modified MultiStore.
func (rs *Store) SetTracingContext(tc types.TraceContext) types.MultiStore {
	rs.traceContextMutex.Lock()
	defer rs.traceContextMutex.Unlock()
	rs.traceContext = rs.traceContext.Merge(tc)

	return rs
}

func (rs *Store) getTracingContext() types.TraceContext {
	rs.traceContextMutex.Lock()
	defer rs.traceContextMutex.Unlock()

	if rs.traceContext == nil {
		return nil
	}

	ctx := types.TraceContext{}
	for k, v := range rs.traceContext {
		ctx[k] = v
	}

	return ctx
}

// TracingEnabled returns if tracing is enabled for the MultiStore.
func (rs *Store) TracingEnabled() bool {
	return rs.traceWriter != nil
}

// AddListeners adds a listener for the KVStore belonging to the provided StoreKey
func (rs *Store) AddListeners(keys []types.StoreKey) {
	for i := range keys {
		listener := rs.listeners[keys[i]]
		if listener == nil {
			rs.listeners[keys[i]] = types.NewMemoryListener()
		}
	}
}

// ListeningEnabled returns if listening is enabled for a specific KVStore
func (rs *Store) ListeningEnabled(key types.StoreKey) bool {
	if ls, ok := rs.listeners[key]; ok {
		return ls != nil
	}
	return false
}

// PopStateCache returns the accumulated state change messages from the CommitMultiStore
// Calling PopStateCache destroys only the currently accumulated state in each listener
// not the state in the store itself. This is a mutating and destructive operation.
// This method has been synchronized.
func (rs *Store) PopStateCache() []*types.StoreKVPair {
	var cache []*types.StoreKVPair
	for key := range rs.listeners {
		ls := rs.listeners[key]
		if ls != nil {
			cache = append(cache, ls.PopStateCache()...)
		}
	}
	sort.SliceStable(cache, func(i, j int) bool {
		return cache[i].StoreKey < cache[j].StoreKey
	})
	return cache
}

// LatestVersion returns the latest version in the store
func (rs *Store) LatestVersion() int64 {
	return rs.LastCommitID().Version
}

// LastCommitID implements Committer/CommitStore.
func (rs *Store) LastCommitID() types.CommitID {
	if rs.lastCommitInfo == nil {
		emptyHash := sha256.Sum256([]byte{})
		appHash := emptyHash[:]
		return types.CommitID{
			Version: GetLatestVersion(rs.db),
			Hash:    appHash, // set empty apphash to sha256([]byte{}) if info is nil
		}
	}
	if len(rs.lastCommitInfo.CommitID().Hash) == 0 {
		emptyHash := sha256.Sum256([]byte{})
		appHash := emptyHash[:]
		return types.CommitID{
			Version: rs.lastCommitInfo.Version,
			Hash:    appHash, // set empty apphash to sha256([]byte{}) if hash is nil
		}
	}

	return rs.lastCommitInfo.CommitID()
}

// PausePruning temporarily pauses the pruning of all individual stores which implement
// the PausablePruner interface.
func (rs *Store) PausePruning(pause bool) {
	for _, store := range rs.stores {
		if pauseable, ok := store.(types.PausablePruner); ok {
			pauseable.PausePruning(pause)
		}
	}
}

// Commit implements Committer/CommitStore.
func (rs *Store) Commit() types.CommitID {
	var previousHeight, version int64
	if rs.lastCommitInfo.GetVersion() == 0 && rs.initialVersion > 1 {
		// This case means that no commit has been made in the store, we
		// start from initialVersion.
		version = rs.initialVersion
	} else {
		// This case can means two things:
		// - either there was already a previous commit in the store, in which
		// case we increment the version from there,
		// - or there was no previous commit, and initial version was not set,
		// in which case we start at version 1.
		previousHeight = rs.lastCommitInfo.GetVersion()
		version = previousHeight + 1
	}

	if rs.commitHeader.Height != version {
		rs.logger.Debug("commit header and version mismatch", "header_height", rs.commitHeader.Height, "version", version)
	}

	func() { // ensure unpause
		// set the committing flag on all stores to block the pruning
		rs.PausePruning(true)
		// unset the committing flag on all stores to continue the pruning
		defer rs.PausePruning(false)
		rs.lastCommitInfo = commitStores(version, rs.stores, rs.removalMap)
	}()

	rs.lastCommitInfo.Timestamp = rs.commitHeader.Time
	defer rs.flushMetadata(rs.db, version, rs.lastCommitInfo)

	// remove remnants of removed stores
	for sk := range rs.removalMap {
		if _, ok := rs.stores[sk]; ok {
			delete(rs.stores, sk)
			delete(rs.storesParams, sk)
			delete(rs.keysByName, sk.Name())
		}
	}

	// reset the removalMap
	rs.removalMap = make(map[types.StoreKey]bool)

	if err := rs.handlePruning(version); err != nil {
		rs.logger.Error(
			"failed to prune store, please check your pruning configuration",
			"err", err,
		)
	}

	return types.CommitID{
		Version: version,
		Hash:    rs.lastCommitInfo.Hash(),
	}
}

// WorkingHash returns the current hash of the store.
// it will be used to get the current app hash before commit.
func (rs *Store) WorkingHash() []byte {
	storeInfos := make([]types.StoreInfo, 0, len(rs.stores))
	storeKeys := keysFromStoreKeyMap(rs.stores)

	for _, key := range storeKeys {
		store := rs.stores[key]

		if store.GetStoreType() != types.StoreTypeIAVL {
			continue
		}

		if !rs.removalMap[key] {
			si := types.StoreInfo{
				Name: key.Name(),
				CommitId: types.CommitID{
					Hash: store.WorkingHash(),
				},
			}
			storeInfos = append(storeInfos, si)
		}
	}

	sort.SliceStable(storeInfos, func(i, j int) bool {
		return storeInfos[i].Name < storeInfos[j].Name
	})

	return types.CommitInfo{StoreInfos: storeInfos}.Hash()
}

// CacheWrap implements CacheWrapper/Store/CommitStore.
func (rs *Store) CacheWrap() types.CacheWrap {
	return rs.CacheMultiStore().(types.CacheWrap)
}

// CacheWrapWithTrace implements the CacheWrapper interface.
func (rs *Store) CacheWrapWithTrace(_ io.Writer, _ types.TraceContext) types.CacheWrap {
	return rs.CacheWrap()
}

// CacheMultiStore creates ephemeral branch of the multi-store and returns a CacheMultiStore.
// It implements the MultiStore interface.
func (rs *Store) CacheMultiStore() types.CacheMultiStore {
	stores := make(map[types.StoreKey]types.CacheWrapper)
	for k, v := range rs.stores {
		store := types.KVStore(v)
		// Wire the listenkv.Store to allow listeners to observe the writes from the cache store,
		// set same listeners on cache store will observe duplicated writes.
		if rs.ListeningEnabled(k) {
			store = listenkv.NewStore(store, k, rs.listeners[k])
		}
		stores[k] = store
	}
	return cachemulti.NewStore(rs.db, stores, rs.keysByName, rs.traceWriter, rs.getTracingContext())
}

// CacheMultiStoreWithVersion is analogous to CacheMultiStore except that it
// attempts to load stores at a given version (height). An error is returned if
// any store cannot be loaded. This should only be used for querying and
// iterating at past heights.
func (rs *Store) CacheMultiStoreWithVersion(version int64) (types.CacheMultiStore, error) {
	cachedStores := make(map[types.StoreKey]types.CacheWrapper)
	var commitInfo *types.CommitInfo
	storeInfos := map[string]bool{}
	for key, store := range rs.stores {
		var cacheStore types.KVStore
		switch store.GetStoreType() {
		case types.StoreTypeIAVL:
			// If the store is wrapped with an inter-block cache, we must first unwrap
			// it to get the underlying IAVL store.
			store = rs.GetCommitKVStore(key)

			// Attempt to lazy-load an already saved IAVL store version. If the
			// version does not exist or is pruned, an error should be returned.
			var err error
			cacheStore, err = store.(*iavl.Store).GetImmutable(version)
			// if we got error from loading a module store
			// we fetch commit info of this version
			// we use commit info to check if the store existed at this version or not
			if err != nil {
				if commitInfo == nil {
					var errCommitInfo error
					commitInfo, errCommitInfo = rs.GetCommitInfo(version)

					if errCommitInfo != nil {
						return nil, errCommitInfo
					}

					for _, storeInfo := range commitInfo.StoreInfos {
						storeInfos[storeInfo.Name] = true
					}
				}

				// If the store existed at this version, it means there's actually an error
				// getting the root store at this version.
				if storeInfos[key.Name()] {
					return nil, err
				}

				// If the store donesn't exist at this version, create a dummy one to prevent
				// nil pointer panic in newer query APIs.
				cacheStore = dbadapter.Store{DB: coretesting.NewMemDB()}
			}

		default:
			cacheStore = store
		}

		// Wire the listenkv.Store to allow listeners to observe the writes from the cache store,
		// set same listeners on cache store will observe duplicated writes.
		if rs.ListeningEnabled(key) {
			cacheStore = listenkv.NewStore(cacheStore, key, rs.listeners[key])
		}

		cachedStores[key] = cacheStore
	}

	return cachemulti.NewStore(rs.db, cachedStores, rs.keysByName, rs.traceWriter, rs.getTracingContext()), nil
}

// GetStore returns a mounted Store for a given StoreKey. If the StoreKey does
// not exist, it will panic. If the Store is wrapped in an inter-block cache, it
// will be unwrapped prior to being returned.
//
// TODO: This isn't used directly upstream. Consider returning the Store as-is
// instead of unwrapping.
func (rs *Store) GetStore(key types.StoreKey) types.Store {
	store := rs.GetCommitKVStore(key)
	if store == nil {
		panic(fmt.Sprintf("store does not exist for key: %s", key.Name()))
	}

	return store
}

// GetKVStore returns a mounted KVStore for a given StoreKey. If tracing is
// enabled on the KVStore, a wrapped TraceKVStore will be returned with the root
// store's tracer, otherwise, the original KVStore will be returned.
//
// NOTE: The returned KVStore may be wrapped in an inter-block cache if it is
// set on the root store.
func (rs *Store) GetKVStore(key types.StoreKey) types.KVStore {
	s := rs.stores[key]
	if s == nil {
		panic(fmt.Sprintf("store does not exist for key: %s", key.Name()))
	}
	store := types.KVStore(s)

	if rs.TracingEnabled() {
		store = tracekv.NewStore(store, rs.traceWriter, rs.getTracingContext())
	}
	if rs.ListeningEnabled(key) {
		store = listenkv.NewStore(store, key, rs.listeners[key])
	}

	return store
}

func (rs *Store) handlePruning(version int64) error {
	pruneHeight := rs.pruningManager.GetPruningHeight(version)
	rs.logger.Debug("prune start", "height", version)
	defer rs.logger.Debug("prune end", "height", version)
	return rs.PruneStores(pruneHeight)
}

// PruneStores prunes all history up to the specific height of the multi store.
func (rs *Store) PruneStores(pruningHeight int64) (err error) {
	if pruningHeight <= 0 {
		rs.logger.Debug("pruning skipped, height is less than or equal to 0")
		return nil
	}

	rs.logger.Debug("pruning store", "heights", pruningHeight)

	for key, store := range rs.stores {
		rs.logger.Debug("pruning store", "key", key) // Also log store.name (a private variable)?

		// If the store is wrapped with an inter-block cache, we must first unwrap
		// it to get the underlying IAVL store.
		if store.GetStoreType() != types.StoreTypeIAVL {
			continue
		}

		store = rs.GetCommitKVStore(key)

		err := store.(*iavl.Store).DeleteVersionsTo(pruningHeight)
		if err == nil {
			continue
		}

		if errors.Is(err, iavltree.ErrVersionDoesNotExist) {
			return err
		}

		rs.logger.Error("failed to prune store", "key", key, "err", err)
	}
	return nil
}

// GetStoreByName performs a lookup of a StoreKey given a store name typically
// provided in a path. The StoreKey is then used to perform a lookup and return
// a Store. If the Store is wrapped in an inter-block cache, it will be unwrapped
// prior to being returned. If the StoreKey does not exist, nil is returned.
func (rs *Store) GetStoreByName(name string) types.Store {
	key := rs.keysByName[name]
	if key == nil {
		return nil
	}

	return rs.GetCommitKVStore(key)
}

// Query calls substore.Query with the same `req` where `req.Path` is
// modified to remove the substore prefix.
// Ie. `req.Path` here is `/<substore>/<path>`, and trimmed to `/<path>` for the substore.
// TODO: add proof for `multistore -> substore`.
func (rs *Store) Query(req *types.RequestQuery) (*types.ResponseQuery, error) {
	path := req.Path
	storeName, subpath, err := parsePath(path)
	if err != nil {
		return &types.ResponseQuery{}, err
	}

	store := rs.GetStoreByName(storeName)
	if store == nil {
		return &types.ResponseQuery{}, errorsmod.Wrapf(types.ErrUnknownRequest, "no such store: %s", storeName)
	}

	queryable, ok := store.(types.Queryable)
	if !ok {
		return &types.ResponseQuery{}, errorsmod.Wrapf(types.ErrUnknownRequest, "store %s (type %T) doesn't support queries", storeName, store)
	}

	// trim the path and make the query
	req.Path = subpath
	res, err := queryable.Query(req)

	if !req.Prove || !RequireProof(subpath) {
		return res, err
	}

	if res.ProofOps == nil || len(res.ProofOps.Ops) == 0 {
		return &types.ResponseQuery{}, errorsmod.Wrap(types.ErrInvalidRequest, "proof is unexpectedly empty; ensure height has not been pruned")
	}

	// If the request's height is the latest height we've committed, then utilize
	// the store's lastCommitInfo as this commit info may not be flushed to disk.
	// Otherwise, we query for the commit info from disk.
	var commitInfo *types.CommitInfo

	if res.Height == rs.lastCommitInfo.Version {
		commitInfo = rs.lastCommitInfo
	} else {
		commitInfo, err = rs.GetCommitInfo(res.Height)
		if err != nil {
			return &types.ResponseQuery{}, err
		}
	}

	// Restore origin path and append proof op.
	res.ProofOps.Ops = append(res.ProofOps.Ops, commitInfo.ProofOp(storeName))

	return res, nil
}

// SetInitialVersion sets the initial version of the IAVL tree. It is used when
// starting a new chain at an arbitrary height.
func (rs *Store) SetInitialVersion(version int64) error {
	rs.initialVersion = version

	// Loop through all the stores, if it's an IAVL store, then set initial
	// version on it.
	for key, store := range rs.stores {
		if store.GetStoreType() == types.StoreTypeIAVL {
			// If the store is wrapped with an inter-block cache, we must first unwrap
			// it to get the underlying IAVL store.
			store = rs.GetCommitKVStore(key)
			store.(types.StoreWithInitialVersion).SetInitialVersion(version)
		}
	}

	return nil
}

// parsePath expects a format like /<storeName>[/<subpath>]
// Must start with /, subpath may be empty
// Returns error if it doesn't start with /
func parsePath(path string) (storeName, subpath string, err error) {
	if !strings.HasPrefix(path, "/") {
		return storeName, subpath, errorsmod.Wrapf(types.ErrUnknownRequest, "invalid path: %s", path)
	}

	paths := strings.SplitN(path[1:], "/", 2)
	storeName = paths[0]

	if len(paths) == 2 {
		subpath = "/" + paths[1]
	}

	return storeName, subpath, nil
}

//---------------------- Snapshotting ------------------

// Snapshot implements snapshottypes.Snapshotter. The snapshot output for a given format must be
// identical across nodes such that chunks from different sources fit together. If the output for a
// given format changes (at the byte level), the snapshot format must be bumped - see
// TestMultistoreSnapshot_Checksum test.
func (rs *Store) Snapshot(height uint64, protoWriter protoio.Writer) error {
	if height == 0 {
		return errorsmod.Wrap(types.ErrLogic, "cannot snapshot height 0")
	}
	if height > uint64(GetLatestVersion(rs.db)) {
		return errorsmod.Wrapf(types.ErrLogic, "cannot snapshot future height %v", height)
	}

	// Collect stores to snapshot (only IAVL stores are supported)
	type namedStore struct {
		*iavl.Store
		name string
	}
	stores := []namedStore{}
	keys := keysFromStoreKeyMap(rs.stores)
	for _, key := range keys {
		switch store := rs.GetCommitKVStore(key).(type) {
		case *iavl.Store:
			stores = append(stores, namedStore{name: key.Name(), Store: store})
		case *transient.Store, *mem.Store:
			// Non-persisted stores shouldn't be snapshotted
			continue
		default:
			return errorsmod.Wrapf(types.ErrLogic,
				"don't know how to snapshot store %q of type %T", key.Name(), store)
		}
	}
	sort.Slice(stores, func(i, j int) bool {
		return strings.Compare(stores[i].name, stores[j].name) == -1
	})

	// Export each IAVL store. Stores are serialized as a stream of SnapshotItem Protobuf
	// messages. The first item contains a SnapshotStore with store metadata (i.e. name),
	// and the following messages contain a SnapshotNode (i.e. an ExportNode). Store changes
	// are demarcated by new SnapshotStore items.
	for _, store := range stores {
		rs.logger.Debug("starting snapshot", "store", store.name, "height", height)
		exporter, err := store.Export(int64(height))
		if err != nil {
			rs.logger.Error("snapshot failed; exporter error", "store", store.name, "err", err)
			return err
		}

		err = func() error {
			defer exporter.Close()

			err := protoWriter.WriteMsg(&snapshottypes.SnapshotItem{
				Item: &snapshottypes.SnapshotItem_Store{
					Store: &snapshottypes.SnapshotStoreItem{
						Name: store.name,
					},
				},
			})
			if err != nil {
				rs.logger.Error("snapshot failed; item store write failed", "store", store.name, "err", err)
				return err
			}

			nodeCount := 0
			for {
				node, err := exporter.Next()
				if errors.Is(err, iavltree.ErrorExportDone) {
					rs.logger.Debug("snapshot Done", "store", store.name, "nodeCount", nodeCount)
					break
				} else if err != nil {
					return err
				}
				err = protoWriter.WriteMsg(&snapshottypes.SnapshotItem{
					Item: &snapshottypes.SnapshotItem_IAVL{
						IAVL: &snapshottypes.SnapshotIAVLItem{
							Key:     node.Key,
							Value:   node.Value,
							Height:  int32(node.Height),
							Version: node.Version,
						},
					},
				})
				if err != nil {
					return err
				}
				nodeCount++
			}

			return nil
		}()
		if err != nil {
			return err
		}
	}

	return nil
}

// Restore implements snapshottypes.Snapshotter.
// returns next snapshot item and error.
func (rs *Store) Restore(
	height uint64, format uint32, protoReader protoio.Reader,
) (snapshottypes.SnapshotItem, error) {
	// Import nodes into stores. The first item is expected to be a SnapshotItem containing
	// a SnapshotStoreItem, telling us which store to import into. The following items will contain
	// SnapshotNodeItem (i.e. ExportNode) until we reach the next SnapshotStoreItem or EOF.
	var importer *iavltree.Importer
	var snapshotItem snapshottypes.SnapshotItem
loop:
	for {
		snapshotItem = snapshottypes.SnapshotItem{}
		err := protoReader.ReadMsg(&snapshotItem)
		if errors.Is(err, io.EOF) {
			break
		} else if err != nil {
			return snapshottypes.SnapshotItem{}, errorsmod.Wrap(err, "invalid protobuf message")
		}

		switch item := snapshotItem.Item.(type) {
		case *snapshottypes.SnapshotItem_Store:
			if importer != nil {
				err = importer.Commit()
				if err != nil {
					return snapshottypes.SnapshotItem{}, errorsmod.Wrap(err, "IAVL commit failed")
				}
				importer.Close()
			}
			store, ok := rs.GetStoreByName(item.Store.Name).(*iavl.Store)
			if !ok || store == nil {
				return snapshottypes.SnapshotItem{}, errorsmod.Wrapf(types.ErrLogic, "cannot import into non-IAVL store %q", item.Store.Name)
			}
			importer, err = store.Import(int64(height))
			if err != nil {
				return snapshottypes.SnapshotItem{}, errorsmod.Wrap(err, "import failed")
			}
			defer importer.Close()
			// Importer height must reflect the node height (which usually matches the block height, but not always)
			rs.logger.Debug("restoring snapshot", "store", item.Store.Name)

		case *snapshottypes.SnapshotItem_IAVL:
			if importer == nil {
				rs.logger.Error("failed to restore; received IAVL node item before store item")
				return snapshottypes.SnapshotItem{}, errorsmod.Wrap(types.ErrLogic, "received IAVL node item before store item")
			}
			if item.IAVL.Height > math.MaxInt8 {
				return snapshottypes.SnapshotItem{}, errorsmod.Wrapf(types.ErrLogic, "node height %v cannot exceed %v",
					item.IAVL.Height, math.MaxInt8)
			}
			node := &iavltree.ExportNode{
				Key:     item.IAVL.Key,
				Value:   item.IAVL.Value,
				Height:  int8(item.IAVL.Height),
				Version: item.IAVL.Version,
			}
			// Protobuf does not differentiate between []byte{} as nil, but fortunately IAVL does
			// not allow nil keys nor nil values for leaf nodes, so we can always set them to empty.
			if node.Key == nil {
				node.Key = []byte{}
			}
			if node.Height == 0 && node.Value == nil {
				node.Value = []byte{}
			}
			err := importer.Add(node)
			if err != nil {
				return snapshottypes.SnapshotItem{}, errorsmod.Wrap(err, "IAVL node import failed")
			}

		default:
			break loop
		}
	}

	if importer != nil {
		err := importer.Commit()
		if err != nil {
			return snapshottypes.SnapshotItem{}, errorsmod.Wrap(err, "IAVL commit failed")
		}
		importer.Close()
	}

	rs.flushMetadata(rs.db, int64(height), rs.buildCommitInfo(int64(height)))
	return snapshotItem, rs.LoadLatestVersion()
}

func (rs *Store) loadCommitStoreFromParams(key types.StoreKey, id types.CommitID, params storeParams) (types.CommitKVStore, error) {
	var db corestore.KVStoreWithBatch

	if params.db != nil {
		db = dbm.NewPrefixDB(params.db, []byte("s/_/"))
	} else {
		prefix := "s/k:" + params.key.Name() + "/"
		db = dbm.NewPrefixDB(rs.db, []byte(prefix))
	}

	switch params.typ {
	case types.StoreTypeMulti:
		panic("recursive MultiStores not yet supported")

	case types.StoreTypeIAVL:
		var store types.CommitKVStore
		var err error

		if params.initialVersion == 0 {
			store, err = iavl.LoadStore(db, rs.logger, key, id, rs.iavlCacheSize, rs.iavlDisableFastNode, rs.metrics)
		} else {
			store, err = iavl.LoadStoreWithInitialVersion(db, rs.logger, key, id, params.initialVersion, rs.iavlCacheSize, rs.iavlDisableFastNode, rs.metrics)
		}

		if err != nil {
			return nil, err
		}

		if rs.interBlockCache != nil {
			// Wrap and get a CommitKVStore with inter-block caching. Note, this should
			// only wrap the primary CommitKVStore, not any store that is already
			// branched as that will create unexpected behavior.
			store = rs.interBlockCache.GetStoreCache(key, store)
		}

		return store, err

	case types.StoreTypeDB:
		return commitDBStoreAdapter{Store: dbadapter.Store{DB: db}}, nil

	case types.StoreTypeTransient:
		_, ok := key.(*types.TransientStoreKey)
		if !ok {
			return nil, fmt.Errorf("invalid StoreKey for StoreTypeTransient: %s", key.String())
		}

		return transient.NewStore(), nil

	case types.StoreTypeMemory:
		if _, ok := key.(*types.MemoryStoreKey); !ok {
			return nil, fmt.Errorf("unexpected key type for a MemoryStoreKey; got: %s", key.String())
		}

		return mem.NewStore(), nil

	default:
		panic(fmt.Sprintf("unrecognized store type %v", params.typ))
	}
}

func (rs *Store) buildCommitInfo(version int64) *types.CommitInfo {
	keys := keysFromStoreKeyMap(rs.stores)
	storeInfos := []types.StoreInfo{}
	for _, key := range keys {
		store := rs.stores[key]
		storeType := store.GetStoreType()
		if storeType == types.StoreTypeTransient || storeType == types.StoreTypeMemory {
			continue
		}
		storeInfos = append(storeInfos, types.StoreInfo{
			Name:     key.Name(),
			CommitId: store.LastCommitID(),
		})
	}
	return &types.CommitInfo{
		Version:    version,
		StoreInfos: storeInfos,
	}
}

// RollbackToVersion delete the versions after `target` and update the latest version.
func (rs *Store) RollbackToVersion(target int64) error {
	if target <= 0 {
		return fmt.Errorf("invalid rollback height target: %d", target)
	}

	for key, store := range rs.stores {
		if store.GetStoreType() == types.StoreTypeIAVL {
			// If the store is wrapped with an inter-block cache, we must first unwrap
			// it to get the underlying IAVL store.
			store = rs.GetCommitKVStore(key)
			err := store.(*iavl.Store).LoadVersionForOverwriting(target)
			if err != nil {
				return err
			}
		}
	}

	rs.flushMetadata(rs.db, target, rs.buildCommitInfo(target))

	return rs.LoadLatestVersion()
}

// SetCommitHeader sets the commit block header of the store.
func (rs *Store) SetCommitHeader(h cmtproto.Header) {
	rs.commitHeader = h
}

// GetCommitInfo attempts to retrieve CommitInfo for a given version/height. It
// will return an error if no CommitInfo exists, we fail to unmarshal the record
// or if we cannot retrieve the object from the DB.
func (rs *Store) GetCommitInfo(ver int64) (*types.CommitInfo, error) {
	cInfoKey := fmt.Sprintf(commitInfoKeyFmt, ver)

	bz, err := rs.db.Get([]byte(cInfoKey))
	if err != nil {
		return nil, errorsmod.Wrap(err, "failed to get commit info")
	} else if bz == nil {
		return nil, errors.New("no commit info found")
	}

	cInfo := &types.CommitInfo{}
	if err = cInfo.Unmarshal(bz); err != nil {
		return nil, errorsmod.Wrap(err, "failed unmarshal commit info")
	}

	return cInfo, nil
}

func (rs *Store) flushMetadata(db corestore.KVStoreWithBatch, version int64, cInfo *types.CommitInfo) {
	rs.logger.Debug("flushing metadata", "height", version)
	batch := db.NewBatch()
	defer func() {
		_ = batch.Close()
	}()

	if cInfo != nil {
		flushCommitInfo(batch, version, cInfo)
	} else {
		rs.logger.Debug("commitInfo is nil, not flushed", "height", version)
	}

	flushLatestVersion(batch, version)

	if err := batch.WriteSync(); err != nil {
		panic(fmt.Errorf("error on batch write %w", err))
	}
	rs.logger.Debug("flushing metadata finished", "height", version)
}

type storeParams struct {
	key            types.StoreKey
	db             corestore.KVStoreWithBatch
	typ            types.StoreType
	initialVersion uint64
}

func newStoreParams(key types.StoreKey, db corestore.KVStoreWithBatch, typ types.StoreType, initialVersion uint64) storeParams {
	return storeParams{
		key:            key,
		db:             db,
		typ:            typ,
		initialVersion: initialVersion,
	}
}

func GetLatestVersion(db corestore.KVStoreWithBatch) int64 {
	bz, err := db.Get([]byte(latestVersionKey))
	if err != nil {
		panic(err)
	} else if bz == nil {
		return 0
	}

	var latestVersion int64

	if err := gogotypes.StdInt64Unmarshal(&latestVersion, bz); err != nil {
		panic(err)
	}

	return latestVersion
}

// Commits each store and returns a new commitInfo.
func commitStores(version int64, storeMap map[types.StoreKey]types.CommitKVStore, removalMap map[types.StoreKey]bool) *types.CommitInfo {
	storeInfos := make([]types.StoreInfo, 0, len(storeMap))
	storeKeys := keysFromStoreKeyMap(storeMap)

	for _, key := range storeKeys {
		store := storeMap[key]
		last := store.LastCommitID()

		// If a commit event execution is interrupted, a new iavl store's version
		// will be larger than the RMS's metadata, when the block is replayed, we
		// should avoid committing that iavl store again.
		var commitID types.CommitID
		if last.Version >= version {
			last.Version = version
			commitID = last
		} else {
			commitID = store.Commit()
		}

		storeType := store.GetStoreType()
		if storeType == types.StoreTypeTransient || storeType == types.StoreTypeMemory {
			continue
		}

		if !removalMap[key] {
			si := types.StoreInfo{}
			si.Name = key.Name()
			si.CommitId = commitID
			storeInfos = append(storeInfos, si)
		}
	}

	sort.SliceStable(storeInfos, func(i, j int) bool {
		return strings.Compare(storeInfos[i].Name, storeInfos[j].Name) < 0
	})

	return &types.CommitInfo{
		Version:    version,
		StoreInfos: storeInfos,
	}
}

func flushCommitInfo(batch corestore.Batch, version int64, cInfo *types.CommitInfo) {
	bz, err := cInfo.Marshal()
	if err != nil {
		panic(err)
	}

	cInfoKey := fmt.Sprintf(commitInfoKeyFmt, version)
	err = batch.Set([]byte(cInfoKey), bz)
	if err != nil {
		panic(err)
	}
}

func flushLatestVersion(batch corestore.Batch, version int64) {
	bz, err := gogotypes.StdInt64Marshal(version)
	if err != nil {
		panic(err)
	}

	err = batch.Set([]byte(latestVersionKey), bz)
	if err != nil {
		panic(err)
	}
}
