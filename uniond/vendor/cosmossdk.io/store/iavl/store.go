package iavl

import (
	"errors"
	"fmt"
	"io"

	cmtprotocrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/cosmos/iavl"
	ics23 "github.com/cosmos/ics23/go"

	errorsmod "cosmossdk.io/errors"
	"cosmossdk.io/log"
	"cosmossdk.io/store/cachekv"
	"cosmossdk.io/store/internal/kv"
	"cosmossdk.io/store/metrics"
	pruningtypes "cosmossdk.io/store/pruning/types"
	"cosmossdk.io/store/tracekv"
	"cosmossdk.io/store/types"
)

const (
	DefaultIAVLCacheSize = 500000
)

var (
	_ types.KVStore                 = (*Store)(nil)
	_ types.CommitStore             = (*Store)(nil)
	_ types.CommitKVStore           = (*Store)(nil)
	_ types.Queryable               = (*Store)(nil)
	_ types.StoreWithInitialVersion = (*Store)(nil)
)

// Store Implements types.KVStore and CommitKVStore.
type Store struct {
	tree    Tree
	logger  log.Logger
	metrics metrics.StoreMetrics
}

// LoadStore returns an IAVL Store as a CommitKVStore. Internally, it will load the
// store's version (id) from the provided DB. An error is returned if the version
// fails to load, or if called with a positive version on an empty tree.
func LoadStore(db dbm.DB, logger log.Logger, key types.StoreKey, id types.CommitID, cacheSize int, disableFastNode bool, metrics metrics.StoreMetrics) (types.CommitKVStore, error) {
	return LoadStoreWithInitialVersion(db, logger, key, id, 0, cacheSize, disableFastNode, metrics)
}

// LoadStoreWithInitialVersion returns an IAVL Store as a CommitKVStore setting its initialVersion
// to the one given. Internally, it will load the store's version (id) from the
// provided DB. An error is returned if the version fails to load, or if called with a positive
// version on an empty tree.
func LoadStoreWithInitialVersion(db dbm.DB, logger log.Logger, key types.StoreKey, id types.CommitID, initialVersion uint64, cacheSize int, disableFastNode bool, metrics metrics.StoreMetrics) (types.CommitKVStore, error) {
	tree := iavl.NewMutableTree(db, cacheSize, disableFastNode, logger, iavl.InitialVersionOption(initialVersion))

	isUpgradeable, err := tree.IsUpgradeable()
	if err != nil {
		return nil, err
	}

	if isUpgradeable && logger != nil {
		logger.Info(
			"Upgrading IAVL storage for faster queries + execution on live state. This may take a while",
			"store_key", key.String(),
			"version", initialVersion,
			"commit", fmt.Sprintf("%X", id),
		)
	}

	_, err = tree.LoadVersion(id.Version)
	if err != nil {
		return nil, err
	}

	if logger != nil {
		logger.Debug("Finished loading IAVL tree")
	}

	return &Store{
		tree:    tree,
		logger:  logger,
		metrics: metrics,
	}, nil
}

// UnsafeNewStore returns a reference to a new IAVL Store with a given mutable
// IAVL tree reference. It should only be used for testing purposes.
//
// CONTRACT: The IAVL tree should be fully loaded.
// CONTRACT: PruningOptions passed in as argument must be the same as pruning options
// passed into iavl.MutableTree
func UnsafeNewStore(tree *iavl.MutableTree) *Store {
	return &Store{
		tree:    tree,
		metrics: metrics.NewNoOpMetrics(),
	}
}

// GetImmutable returns a reference to a new store backed by an immutable IAVL
// tree at a specific version (height) without any pruning options. This should
// be used for querying and iteration only. If the version does not exist or has
// been pruned, an empty immutable IAVL tree will be used.
// Any mutable operations executed will result in a panic.
func (st *Store) GetImmutable(version int64) (*Store, error) {
	if !st.VersionExists(version) {
		return nil, errors.New("version mismatch on immutable IAVL tree; version does not exist. Version has either been pruned, or is for a future block height")
	}

	iTree, err := st.tree.GetImmutable(version)
	if err != nil {
		return nil, err
	}

	return &Store{
		tree:    &immutableTree{iTree},
		metrics: st.metrics,
	}, nil
}

// Commit commits the current store state and returns a CommitID with the new
// version and hash.
func (st *Store) Commit() types.CommitID {
	defer st.metrics.MeasureSince("store", "iavl", "commit")

	hash, version, err := st.tree.SaveVersion()
	if err != nil {
		panic(err)
	}

	return types.CommitID{
		Version: version,
		Hash:    hash,
	}
}

// WorkingHash returns the hash of the current working tree.
func (st *Store) WorkingHash() []byte {
	return st.tree.WorkingHash()
}

// LastCommitID implements Committer.
func (st *Store) LastCommitID() types.CommitID {
	return types.CommitID{
		Version: st.tree.Version(),
		Hash:    st.tree.Hash(),
	}
}

// SetPruning panics as pruning options should be provided at initialization
// since IAVl accepts pruning options directly.
func (st *Store) SetPruning(_ pruningtypes.PruningOptions) {
	panic("cannot set pruning options on an initialized IAVL store")
}

// SetPruning panics as pruning options should be provided at initialization
// since IAVl accepts pruning options directly.
func (st *Store) GetPruning() pruningtypes.PruningOptions {
	panic("cannot get pruning options on an initialized IAVL store")
}

// VersionExists returns whether or not a given version is stored.
func (st *Store) VersionExists(version int64) bool {
	return st.tree.VersionExists(version)
}

// GetAllVersions returns all versions in the iavl tree
func (st *Store) GetAllVersions() []int {
	return st.tree.AvailableVersions()
}

// Implements Store.
func (st *Store) GetStoreType() types.StoreType {
	return types.StoreTypeIAVL
}

// Implements Store.
func (st *Store) CacheWrap() types.CacheWrap {
	return cachekv.NewStore(st)
}

// CacheWrapWithTrace implements the Store interface.
func (st *Store) CacheWrapWithTrace(w io.Writer, tc types.TraceContext) types.CacheWrap {
	return cachekv.NewStore(tracekv.NewStore(st, w, tc))
}

// Implements types.KVStore.
func (st *Store) Set(key, value []byte) {
	types.AssertValidKey(key)
	types.AssertValidValue(value)
	_, err := st.tree.Set(key, value)
	if err != nil && st.logger != nil {
		st.logger.Error("iavl set error", "error", err.Error())
	}
}

// Implements types.KVStore.
func (st *Store) Get(key []byte) []byte {
	defer st.metrics.MeasureSince("store", "iavl", "get")
	value, err := st.tree.Get(key)
	if err != nil {
		panic(err)
	}
	return value
}

// Implements types.KVStore.
func (st *Store) Has(key []byte) (exists bool) {
	defer st.metrics.MeasureSince("store", "iavl", "has")
	has, err := st.tree.Has(key)
	if err != nil {
		panic(err)
	}
	return has
}

// Implements types.KVStore.
func (st *Store) Delete(key []byte) {
	defer st.metrics.MeasureSince("store", "iavl", "delete")
	_, _, err := st.tree.Remove(key)
	if err != nil {
		panic(err)
	}
}

// DeleteVersionsTo deletes versions upto the given version from the MutableTree. An error
// is returned if any single version is invalid or the delete fails. All writes
// happen in a single batch with a single commit.
func (st *Store) DeleteVersionsTo(version int64) error {
	return st.tree.DeleteVersionsTo(version)
}

// LoadVersionForOverwriting attempts to load a tree at a previously committed
// version. Any versions greater than targetVersion will be deleted.
func (st *Store) LoadVersionForOverwriting(targetVersion int64) error {
	return st.tree.LoadVersionForOverwriting(targetVersion)
}

// Implements types.KVStore.
func (st *Store) Iterator(start, end []byte) types.Iterator {
	iterator, err := st.tree.Iterator(start, end, true)
	if err != nil {
		panic(err)
	}
	return iterator
}

// Implements types.KVStore.
func (st *Store) ReverseIterator(start, end []byte) types.Iterator {
	iterator, err := st.tree.Iterator(start, end, false)
	if err != nil {
		panic(err)
	}
	return iterator
}

// SetInitialVersion sets the initial version of the IAVL tree. It is used when
// starting a new chain at an arbitrary height.
func (st *Store) SetInitialVersion(version int64) {
	st.tree.SetInitialVersion(uint64(version))
}

// Exports the IAVL store at the given version, returning an iavl.Exporter for the tree.
func (st *Store) Export(version int64) (*iavl.Exporter, error) {
	istore, err := st.GetImmutable(version)
	if err != nil {
		return nil, errorsmod.Wrapf(err, "iavl export failed for version %v", version)
	}
	tree, ok := istore.tree.(*immutableTree)
	if !ok || tree == nil {
		return nil, fmt.Errorf("iavl export failed: unable to fetch tree for version %v", version)
	}
	return tree.Export()
}

// Import imports an IAVL tree at the given version, returning an iavl.Importer for importing.
func (st *Store) Import(version int64) (*iavl.Importer, error) {
	tree, ok := st.tree.(*iavl.MutableTree)
	if !ok {
		return nil, errors.New("iavl import failed: unable to find mutable tree")
	}
	return tree.Import(version)
}

// Handle gatest the latest height, if height is 0
func getHeight(tree Tree, req *types.RequestQuery) int64 {
	height := req.Height
	if height == 0 {
		latest := tree.Version()
		if tree.VersionExists(latest - 1) {
			height = latest - 1
		} else {
			height = latest
		}
	}
	return height
}

// Query implements ABCI interface, allows queries
//
// by default we will return from (latest height -1),
// as we will have merkle proofs immediately (header height = data height + 1)
// If latest-1 is not present, use latest (which must be present)
// if you care to have the latest data to see a tx results, you must
// explicitly set the height you want to see
func (st *Store) Query(req *types.RequestQuery) (res *types.ResponseQuery, err error) {
	defer st.metrics.MeasureSince("store", "iavl", "query")

	if len(req.Data) == 0 {
		return &types.ResponseQuery{}, errorsmod.Wrap(types.ErrTxDecode, "query cannot be zero length")
	}

	tree := st.tree

	// store the height we chose in the response, with 0 being changed to the
	// latest height
	res = &types.ResponseQuery{
		Height: getHeight(tree, req),
	}

	switch req.Path {
	case "/key": // get by key
		key := req.Data // data holds the key bytes

		res.Key = key
		if !st.VersionExists(res.Height) {
			res.Log = iavl.ErrVersionDoesNotExist.Error()
			break
		}

		value, err := tree.GetVersioned(key, res.Height)
		if err != nil {
			panic(err)
		}
		res.Value = value

		if !req.Prove {
			break
		}

		// Continue to prove existence/absence of value
		// Must convert store.Tree to iavl.MutableTree with given version to use in CreateProof
		iTree, err := tree.GetImmutable(res.Height)
		if err != nil {
			// sanity check: If value for given version was retrieved, immutable tree must also be retrievable
			panic(fmt.Sprintf("version exists in store but could not retrieve corresponding versioned tree in store, %s", err.Error()))
		}
		mtree := &iavl.MutableTree{
			ImmutableTree: iTree,
		}

		// get proof from tree and convert to merkle.Proof before adding to result
		res.ProofOps = getProofFromTree(mtree, req.Data, res.Value != nil)

	case "/subspace":
		pairs := kv.Pairs{
			Pairs: make([]kv.Pair, 0),
		}

		subspace := req.Data
		res.Key = subspace

		iterator := types.KVStorePrefixIterator(st, subspace)
		for ; iterator.Valid(); iterator.Next() {
			pairs.Pairs = append(pairs.Pairs, kv.Pair{Key: iterator.Key(), Value: iterator.Value()})
		}
		if err := iterator.Close(); err != nil {
			panic(fmt.Errorf("failed to close iterator: %w", err))
		}

		bz, err := pairs.Marshal()
		if err != nil {
			panic(fmt.Errorf("failed to marshal KV pairs: %w", err))
		}

		res.Value = bz

	default:
		return &types.ResponseQuery{}, errorsmod.Wrapf(types.ErrUnknownRequest, "unexpected query path: %v", req.Path)
	}

	return res, err
}

// TraverseStateChanges traverses the state changes between two versions and calls the given function.
func (st *Store) TraverseStateChanges(startVersion, endVersion int64, fn func(version int64, changeSet *iavl.ChangeSet) error) error {
	return st.tree.TraverseStateChanges(startVersion, endVersion, fn)
}

// Takes a MutableTree, a key, and a flag for creating existence or absence proof and returns the
// appropriate merkle.Proof. Since this must be called after querying for the value, this function should never error
// Thus, it will panic on error rather than returning it
func getProofFromTree(tree *iavl.MutableTree, key []byte, exists bool) *cmtprotocrypto.ProofOps {
	var (
		commitmentProof *ics23.CommitmentProof
		err             error
	)

	if exists {
		// value was found
		commitmentProof, err = tree.GetMembershipProof(key)
		if err != nil {
			// sanity check: If value was found, membership proof must be creatable
			panic(fmt.Sprintf("unexpected value for empty proof: %s", err.Error()))
		}
	} else {
		// value wasn't found
		commitmentProof, err = tree.GetNonMembershipProof(key)
		if err != nil {
			// sanity check: If value wasn't found, nonmembership proof must be creatable
			panic(fmt.Sprintf("unexpected error for nonexistence proof: %s", err.Error()))
		}
	}

	op := types.NewIavlCommitmentOp(key, commitmentProof)
	return &cmtprotocrypto.ProofOps{Ops: []cmtprotocrypto.ProofOp{op.ProofOp()}}
}
