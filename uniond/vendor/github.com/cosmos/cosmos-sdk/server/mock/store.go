package mock

import (
	"io"

	dbm "github.com/cometbft/cometbft-db"
	protoio "github.com/cosmos/gogoproto/io"

	snapshottypes "github.com/cosmos/cosmos-sdk/snapshots/types"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

var _ sdk.MultiStore = multiStore{}

type multiStore struct {
	kv map[storetypes.StoreKey]kvStore
}

func (ms multiStore) CacheMultiStore() sdk.CacheMultiStore {
	panic("not implemented")
}

func (ms multiStore) CacheMultiStoreWithVersion(_ int64) (sdk.CacheMultiStore, error) {
	panic("not implemented")
}

func (ms multiStore) CacheWrap() storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) CacheWrapWithTrace(_ io.Writer, _ sdk.TraceContext) storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) CacheWrapWithListeners(_ storetypes.StoreKey, _ []storetypes.WriteListener) storetypes.CacheWrap {
	panic("not implemented")
}

func (ms multiStore) TracingEnabled() bool {
	panic("not implemented")
}

func (ms multiStore) SetTracingContext(tc sdk.TraceContext) sdk.MultiStore {
	panic("not implemented")
}

func (ms multiStore) SetTracer(w io.Writer) sdk.MultiStore {
	panic("not implemented")
}

func (ms multiStore) AddListeners(key storetypes.StoreKey, listeners []storetypes.WriteListener) {
	panic("not implemented")
}

func (ms multiStore) ListeningEnabled(key storetypes.StoreKey) bool {
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

func (ms multiStore) MountStoreWithDB(key storetypes.StoreKey, typ storetypes.StoreType, db dbm.DB) {
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

func (ms multiStore) GetKVStore(key storetypes.StoreKey) sdk.KVStore {
	return ms.kv[key]
}

func (ms multiStore) GetStore(key storetypes.StoreKey) sdk.Store {
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

func (ms multiStore) SetInterBlockCache(_ sdk.MultiStorePersistentCache) {
	panic("not implemented")
}

func (ms multiStore) SetIAVLCacheSize(size int) {
	panic("not implemented")
}

func (ms multiStore) SetIAVLDisableFastNode(disable bool) {
	panic("not implemented")
}

func (ms multiStore) SetLazyLoading(bool) {
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

var _ sdk.KVStore = kvStore{}

type kvStore struct {
	store map[string][]byte
}

func (kv kvStore) CacheWrap() storetypes.CacheWrap {
	panic("not implemented")
}

func (kv kvStore) CacheWrapWithTrace(w io.Writer, tc sdk.TraceContext) storetypes.CacheWrap {
	panic("not implemented")
}

func (kv kvStore) CacheWrapWithListeners(_ storetypes.StoreKey, _ []storetypes.WriteListener) storetypes.CacheWrap {
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

func (kv kvStore) Prefix(prefix []byte) sdk.KVStore {
	panic("not implemented")
}

func (kv kvStore) Gas(meter sdk.GasMeter, config sdk.GasConfig) sdk.KVStore {
	panic("not implmeneted")
}

func (kv kvStore) Iterator(start, end []byte) sdk.Iterator {
	panic("not implemented")
}

func (kv kvStore) ReverseIterator(start, end []byte) sdk.Iterator {
	panic("not implemented")
}

func (kv kvStore) SubspaceIterator(prefix []byte) sdk.Iterator {
	panic("not implemented")
}

func (kv kvStore) ReverseSubspaceIterator(prefix []byte) sdk.Iterator {
	panic("not implemented")
}

func NewCommitMultiStore() sdk.CommitMultiStore {
	return multiStore{kv: make(map[storetypes.StoreKey]kvStore)}
}
