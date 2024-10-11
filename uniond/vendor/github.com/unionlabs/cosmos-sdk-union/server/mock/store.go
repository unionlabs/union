package mock

import (
	"io"

	protoio "github.com/cosmos/gogoproto/io"

	corestore "cosmossdk.io/core/store"
	"cosmossdk.io/store/metrics"
	pruningtypes "cosmossdk.io/store/pruning/types"
	snapshottypes "cosmossdk.io/store/snapshots/types"
	storetypes "cosmossdk.io/store/types"
)

var _ storetypes.MultiStore = multiStore{}

type multiStore struct {
	kv map[storetypes.StoreKey]kvStore
}

func (ms multiStore) CacheMultiStore() storetypes.CacheMultiStore {
	panic("not implemented")
}

func (ms multiStore) CacheMultiStoreWithVersion(_ int64) (storetypes.CacheMultiStore, error) {
	panic("not implemented")
}

func (ms multiStore) CacheWrap() storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) CacheWrapWithTrace(_ io.Writer, _ storetypes.TraceContext) storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) CacheWrapWithListeners(_ storetypes.StoreKey, _ []storetypes.MemoryListener) storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) TracingEnabled() bool {
	panic("not implemented")
}

func (ms multiStore) SetTracingContext(tc storetypes.TraceContext) storetypes.MultiStore {
	panic("not implemented")
}

func (ms multiStore) SetTracer(w io.Writer) storetypes.MultiStore {
	panic("not implemented")
}

func (ms multiStore) AddListeners(keys []storetypes.StoreKey) {
	panic("not implemented")
}

func (ms multiStore) SetMetrics(metrics.StoreMetrics) {
	panic("not implemented")
}

func (ms multiStore) ListeningEnabled(key storetypes.StoreKey) bool {
	panic("not implemented")
}

func (ms multiStore) PopStateCache() []*storetypes.StoreKVPair {
	panic("not implemented")
}

func (ms multiStore) Commit() storetypes.CommitID {
	panic("not implemented")
}

func (ms multiStore) LastCommitID() storetypes.CommitID {
	panic("not implemented")
}

func (ms multiStore) SetPruning(opts pruningtypes.PruningOptions) {
	panic("not implemented")
}

func (ms multiStore) GetPruning() pruningtypes.PruningOptions {
	panic("not implemented")
}

func (ms multiStore) GetCommitKVStore(key storetypes.StoreKey) storetypes.CommitKVStore {
	panic("not implemented")
}

func (ms multiStore) GetCommitStore(key storetypes.StoreKey) storetypes.CommitStore {
	panic("not implemented")
}

func (ms multiStore) MountStoreWithDB(key storetypes.StoreKey, typ storetypes.StoreType, db corestore.KVStoreWithBatch) {
	ms.kv[key] = kvStore{store: make(map[string][]byte)}
}

func (ms multiStore) LoadLatestVersion() error {
	return nil
}

func (ms multiStore) LoadLatestVersionAndUpgrade(upgrades *storetypes.StoreUpgrades) error {
	return nil
}

func (ms multiStore) LoadVersionAndUpgrade(ver int64, upgrades *storetypes.StoreUpgrades) error {
	panic("not implemented")
}

func (ms multiStore) LoadVersion(ver int64) error {
	panic("not implemented")
}

func (ms multiStore) GetKVStore(key storetypes.StoreKey) storetypes.KVStore {
	return ms.kv[key]
}

func (ms multiStore) GetStore(key storetypes.StoreKey) storetypes.Store {
	panic("not implemented")
}

func (ms multiStore) GetStoreType() storetypes.StoreType {
	panic("not implemented")
}

func (ms multiStore) PruneSnapshotHeight(height int64) {
	panic("not implemented")
}

func (ms multiStore) SetSnapshotInterval(snapshotInterval uint64) {
	panic("not implemented")
}

func (ms multiStore) SetInterBlockCache(_ storetypes.MultiStorePersistentCache) {
	panic("not implemented")
}

func (ms multiStore) SetIAVLCacheSize(size int) {
	panic("not implemented")
}

func (ms multiStore) SetIAVLDisableFastNode(disable bool) {
	panic("not implemented")
}

func (ms multiStore) SetInitialVersion(version int64) error {
	panic("not implemented")
}

func (ms multiStore) Snapshot(height uint64, protoWriter protoio.Writer) error {
	panic("not implemented")
}

func (ms multiStore) Restore(
	height uint64, format uint32, protoReader protoio.Reader,
) (snapshottypes.SnapshotItem, error) {
	panic("not implemented")
}

func (ms multiStore) RollbackToVersion(version int64) error {
	panic("not implemented")
}

func (ms multiStore) LatestVersion() int64 {
	panic("not implemented")
}

func (ms multiStore) WorkingHash() []byte {
	panic("not implemented")
}

var _ storetypes.KVStore = kvStore{}

type kvStore struct {
	store map[string][]byte
}

func (kv kvStore) CacheWrap() storetypes.CacheWrap {
	panic("not implemented")
}

func (kv kvStore) CacheWrapWithTrace(w io.Writer, tc storetypes.TraceContext) storetypes.CacheWrap {
	panic("not implemented")
}

func (kv kvStore) CacheWrapWithListeners(_ storetypes.StoreKey, _ []storetypes.MemoryListener) storetypes.CacheWrap {
	panic("not implemented")
}

func (kv kvStore) GetStoreType() storetypes.StoreType {
	panic("not implemented")
}

func (kv kvStore) Get(key []byte) []byte {
	v, ok := kv.store[string(key)]
	if !ok {
		return nil
	}
	return v
}

func (kv kvStore) Has(key []byte) bool {
	_, ok := kv.store[string(key)]
	return ok
}

func (kv kvStore) Set(key, value []byte) {
	storetypes.AssertValidKey(key)
	kv.store[string(key)] = value
}

func (kv kvStore) Delete(key []byte) {
	delete(kv.store, string(key))
}

func (kv kvStore) Prefix(prefix []byte) storetypes.KVStore {
	panic("not implemented")
}

func (kv kvStore) Gas(meter storetypes.GasMeter, config storetypes.GasConfig) storetypes.KVStore {
	panic("not implemented")
}

func (kv kvStore) Iterator(start, end []byte) storetypes.Iterator {
	panic("not implemented")
}

func (kv kvStore) ReverseIterator(start, end []byte) storetypes.Iterator {
	panic("not implemented")
}

func (kv kvStore) SubspaceIterator(prefix []byte) storetypes.Iterator {
	panic("not implemented")
}

func (kv kvStore) ReverseSubspaceIterator(prefix []byte) storetypes.Iterator {
	panic("not implemented")
}

func NewCommitMultiStore() storetypes.CommitMultiStore {
	return multiStore{kv: make(map[storetypes.StoreKey]kvStore)}
}
