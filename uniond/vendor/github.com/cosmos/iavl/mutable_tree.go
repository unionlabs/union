package iavl

import (
	"bytes"
	"errors"
	"fmt"
	"sort"
	"sync"

	log "cosmossdk.io/log"

	dbm "github.com/cosmos/iavl/db"
	"github.com/cosmos/iavl/fastnode"
	ibytes "github.com/cosmos/iavl/internal/bytes"
)

var (
	// ErrVersionDoesNotExist is returned if a requested version does not exist.
	ErrVersionDoesNotExist = errors.New("version does not exist")

	// ErrKeyDoesNotExist is returned if a key does not exist.
	ErrKeyDoesNotExist = errors.New("key does not exist")
)

type Option func(*Options)

// MutableTree is a persistent tree which keeps track of versions. It is not safe for concurrent
// use, and should be guarded by a Mutex or RWLock as appropriate. An immutable tree at a given
// version can be returned via GetImmutable, which is safe for concurrent access.
//
// Given and returned key/value byte slices must not be modified, since they may point to data
// located inside IAVL which would also be modified.
//
// The inner ImmutableTree should not be used directly by callers.
type MutableTree struct {
	logger log.Logger

	*ImmutableTree                          // The current, working tree.
	lastSaved                *ImmutableTree // The most recently saved tree.
	unsavedFastNodeAdditions *sync.Map      // map[string]*FastNode FastNodes that have not yet been saved to disk
	unsavedFastNodeRemovals  *sync.Map      // map[string]interface{} FastNodes that have not yet been removed from disk
	ndb                      *nodeDB
	skipFastStorageUpgrade   bool // If true, the tree will work like no fast storage and always not upgrade fast storage

	mtx sync.Mutex
}

// NewMutableTree returns a new tree with the specified optional options.
func NewMutableTree(db dbm.DB, cacheSize int, skipFastStorageUpgrade bool, lg log.Logger, options ...Option) *MutableTree {
	opts := DefaultOptions()
	for _, opt := range options {
		opt(&opts)
	}

	ndb := newNodeDB(db, cacheSize, opts, lg)
	head := &ImmutableTree{ndb: ndb, skipFastStorageUpgrade: skipFastStorageUpgrade}

	return &MutableTree{
		logger:                   lg,
		ImmutableTree:            head,
		lastSaved:                head.clone(),
		unsavedFastNodeAdditions: &sync.Map{},
		unsavedFastNodeRemovals:  &sync.Map{},
		ndb:                      ndb,
		skipFastStorageUpgrade:   skipFastStorageUpgrade,
	}
}

// IsEmpty returns whether or not the tree has any keys. Only trees that are
// not empty can be saved.
func (tree *MutableTree) IsEmpty() bool {
	return tree.ImmutableTree.Size() == 0
}

// VersionExists returns whether or not a version exists.
func (tree *MutableTree) VersionExists(version int64) bool {
	legacyLatestVersion, err := tree.ndb.getLegacyLatestVersion()
	if err != nil {
		return false
	}
	if version <= legacyLatestVersion {
		has, err := tree.ndb.hasLegacyVersion(version)
		return err == nil && has
	}
	firstVersion, err := tree.ndb.getFirstVersion()
	if err != nil {
		return false
	}
	latestVersion, err := tree.ndb.getLatestVersion()
	if err != nil {
		return false
	}

	return firstVersion <= version && version <= latestVersion
}

// AvailableVersions returns all available versions in ascending order
func (tree *MutableTree) AvailableVersions() []int {
	firstVersion, err := tree.ndb.getFirstVersion()
	if err != nil {
		return nil
	}
	latestVersion, err := tree.ndb.getLatestVersion()
	if err != nil {
		return nil
	}
	legacyLatestVersion, err := tree.ndb.getLegacyLatestVersion()
	if err != nil {
		return nil
	}

	res := make([]int, 0)
	if legacyLatestVersion > firstVersion {
		for version := firstVersion; version < legacyLatestVersion; version++ {
			has, err := tree.ndb.hasLegacyVersion(version)
			if err != nil {
				return nil
			}
			if has {
				res = append(res, int(version))
			}
		}
		firstVersion = legacyLatestVersion
	}

	for version := firstVersion; version <= latestVersion; version++ {
		res = append(res, int(version))
	}
	return res
}

// Hash returns the hash of the latest saved version of the tree, as returned
// by SaveVersion. If no versions have been saved, Hash returns nil.
func (tree *MutableTree) Hash() []byte {
	return tree.lastSaved.Hash()
}

// WorkingHash returns the hash of the current working tree.
func (tree *MutableTree) WorkingHash() []byte {
	return tree.ImmutableTree.Hash()
}

func (tree *MutableTree) WorkingVersion() int64 {
	version := tree.version + 1
	if version == 1 && tree.ndb.opts.InitialVersion > 0 {
		version = int64(tree.ndb.opts.InitialVersion)
	}
	return version
}

// String returns a string representation of the tree.
func (tree *MutableTree) String() (string, error) {
	return tree.ndb.String()
}

// Set sets a key in the working tree. Nil values are invalid. The given
// key/value byte slices must not be modified after this call, since they point
// to slices stored within IAVL. It returns true when an existing value was
// updated, while false means it was a new key.
func (tree *MutableTree) Set(key, value []byte) (updated bool, err error) {
	updated, err = tree.set(key, value)
	if err != nil {
		return false, err
	}
	return updated, nil
}

// Get returns the value of the specified key if it exists, or nil otherwise.
// The returned value must not be modified, since it may point to data stored within IAVL.
func (tree *MutableTree) Get(key []byte) ([]byte, error) {
	if tree.root == nil {
		return nil, nil
	}

	if !tree.skipFastStorageUpgrade {
		if fastNode, ok := tree.unsavedFastNodeAdditions.Load(ibytes.UnsafeBytesToStr(key)); ok {
			return fastNode.(*fastnode.Node).GetValue(), nil
		}
		// check if node was deleted
		if _, ok := tree.unsavedFastNodeRemovals.Load(string(key)); ok {
			return nil, nil
		}
	}

	return tree.ImmutableTree.Get(key)
}

// Import returns an importer for tree nodes previously exported by ImmutableTree.Export(),
// producing an identical IAVL tree. The caller must call Close() on the importer when done.
//
// version should correspond to the version that was initially exported. It must be greater than
// or equal to the highest ExportNode version number given.
//
// Import can only be called on an empty tree. It is the callers responsibility that no other
// modifications are made to the tree while importing.
func (tree *MutableTree) Import(version int64) (*Importer, error) {
	return newImporter(tree, version)
}

// Iterate iterates over all keys of the tree. The keys and values must not be modified,
// since they may point to data stored within IAVL. Returns true if stopped by callnack, false otherwise
func (tree *MutableTree) Iterate(fn func(key []byte, value []byte) bool) (stopped bool, err error) {
	if tree.root == nil {
		return false, nil
	}

	if tree.skipFastStorageUpgrade {
		return tree.ImmutableTree.Iterate(fn)
	}

	isFastCacheEnabled, err := tree.IsFastCacheEnabled()
	if err != nil {
		return false, err
	}
	if !isFastCacheEnabled {
		return tree.ImmutableTree.Iterate(fn)
	}

	itr := NewUnsavedFastIterator(nil, nil, true, tree.ndb, tree.unsavedFastNodeAdditions, tree.unsavedFastNodeRemovals)
	defer itr.Close()
	for ; itr.Valid(); itr.Next() {
		if fn(itr.Key(), itr.Value()) {
			return true, nil
		}
	}
	return false, nil
}

// Iterator returns an iterator over the mutable tree.
// CONTRACT: no updates are made to the tree while an iterator is active.
func (tree *MutableTree) Iterator(start, end []byte, ascending bool) (dbm.Iterator, error) {
	if !tree.skipFastStorageUpgrade {
		isFastCacheEnabled, err := tree.IsFastCacheEnabled()
		if err != nil {
			return nil, err
		}

		if isFastCacheEnabled {
			return NewUnsavedFastIterator(start, end, ascending, tree.ndb, tree.unsavedFastNodeAdditions, tree.unsavedFastNodeRemovals), nil
		}
	}

	return tree.ImmutableTree.Iterator(start, end, ascending)
}

func (tree *MutableTree) set(key []byte, value []byte) (updated bool, err error) {
	if value == nil {
		return updated, fmt.Errorf("attempt to store nil value at key '%s'", key)
	}

	if tree.ImmutableTree.root == nil {
		if !tree.skipFastStorageUpgrade {
			tree.addUnsavedAddition(key, fastnode.NewNode(key, value, tree.version+1))
		}
		tree.ImmutableTree.root = NewNode(key, value)
		return updated, nil
	}

	tree.ImmutableTree.root, updated, err = tree.recursiveSet(tree.ImmutableTree.root, key, value)
	return updated, err
}

func (tree *MutableTree) recursiveSet(node *Node, key []byte, value []byte) (
	newSelf *Node, updated bool, err error,
) {
	if node.isLeaf() {
		return tree.recursiveSetLeaf(node, key, value)
	}
	node, err = node.clone(tree)
	if err != nil {
		return nil, false, err
	}

	if bytes.Compare(key, node.key) < 0 {
		node.leftNode, updated, err = tree.recursiveSet(node.leftNode, key, value)
		if err != nil {
			return nil, updated, err
		}
	} else {
		node.rightNode, updated, err = tree.recursiveSet(node.rightNode, key, value)
		if err != nil {
			return nil, updated, err
		}
	}

	if updated {
		return node, updated, nil
	}
	err = node.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, false, err
	}
	newNode, err := tree.balance(node)
	if err != nil {
		return nil, false, err
	}
	return newNode, updated, err
}

func (tree *MutableTree) recursiveSetLeaf(node *Node, key []byte, value []byte) (
	newSelf *Node, updated bool, err error,
) {
	version := tree.version + 1
	if !tree.skipFastStorageUpgrade {
		tree.addUnsavedAddition(key, fastnode.NewNode(key, value, version))
	}
	switch bytes.Compare(key, node.key) {
	case -1: // setKey < leafKey
		return &Node{
			key:           node.key,
			subtreeHeight: 1,
			size:          2,
			nodeKey:       nil,
			leftNode:      NewNode(key, value),
			rightNode:     node,
		}, false, nil
	case 1: // setKey > leafKey
		return &Node{
			key:           key,
			subtreeHeight: 1,
			size:          2,
			nodeKey:       nil,
			leftNode:      node,
			rightNode:     NewNode(key, value),
		}, false, nil
	default:
		return NewNode(key, value), true, nil
	}
}

// Remove removes a key from the working tree. The given key byte slice should not be modified
// after this call, since it may point to data stored inside IAVL.
func (tree *MutableTree) Remove(key []byte) ([]byte, bool, error) {
	if tree.root == nil {
		return nil, false, nil
	}
	newRoot, _, value, removed, err := tree.recursiveRemove(tree.root, key)
	if err != nil {
		return nil, false, err
	}
	if !removed {
		return nil, false, nil
	}

	if !tree.skipFastStorageUpgrade {
		tree.addUnsavedRemoval(key)
	}

	tree.root = newRoot
	return value, true, nil
}

// removes the node corresponding to the passed key and balances the tree.
// It returns:
// - the hash of the new node (or nil if the node is the one removed)
// - the node that replaces the orig. node after remove
// - new leftmost leaf key for tree after successfully removing 'key' if changed.
// - the removed value
func (tree *MutableTree) recursiveRemove(node *Node, key []byte) (newSelf *Node, newKey []byte, newValue []byte, removed bool, err error) {
	tree.logger.Debug("recursiveRemove", "node", node, "key", key)
	if node.isLeaf() {
		if bytes.Equal(key, node.key) {
			return nil, nil, node.value, true, nil
		}
		return node, nil, nil, false, nil
	}

	node, err = node.clone(tree)
	if err != nil {
		return nil, nil, nil, false, err
	}

	// node.key < key; we go to the left to find the key:
	if bytes.Compare(key, node.key) < 0 {
		newLeftNode, newKey, value, removed, err := tree.recursiveRemove(node.leftNode, key)
		if err != nil {
			return nil, nil, nil, false, err
		}

		if !removed {
			return node, nil, value, removed, nil
		}

		if newLeftNode == nil { // left node held value, was removed
			return node.rightNode, node.key, value, removed, nil
		}

		node.leftNode = newLeftNode
		err = node.calcHeightAndSize(tree.ImmutableTree)
		if err != nil {
			return nil, nil, nil, false, err
		}
		node, err = tree.balance(node)
		if err != nil {
			return nil, nil, nil, false, err
		}

		return node, newKey, value, removed, nil
	}
	// node.key >= key; either found or look to the right:
	newRightNode, newKey, value, removed, err := tree.recursiveRemove(node.rightNode, key)
	if err != nil {
		return nil, nil, nil, false, err
	}

	if !removed {
		return node, nil, value, removed, nil
	}

	if newRightNode == nil { // right node held value, was removed
		return node.leftNode, nil, value, removed, nil
	}

	node.rightNode = newRightNode
	if newKey != nil {
		node.key = newKey
	}
	err = node.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, nil, nil, false, err
	}

	node, err = tree.balance(node)
	if err != nil {
		return nil, nil, nil, false, err
	}

	return node, nil, value, removed, nil
}

// Load the latest versioned tree from disk.
func (tree *MutableTree) Load() (int64, error) {
	return tree.LoadVersion(int64(0))
}

// Returns the version number of the specific version found
func (tree *MutableTree) LoadVersion(targetVersion int64) (int64, error) {
	firstVersion, err := tree.ndb.getFirstVersion()
	if err != nil {
		return 0, err
	}

	if firstVersion > 0 && firstVersion < int64(tree.ndb.opts.InitialVersion) {
		return firstVersion, fmt.Errorf("initial version set to %v, but found earlier version %v",
			tree.ndb.opts.InitialVersion, firstVersion)
	}

	latestVersion, err := tree.ndb.getLatestVersion()
	if err != nil {
		return 0, err
	}

	if firstVersion > 0 && firstVersion < int64(tree.ndb.opts.InitialVersion) {
		return latestVersion, fmt.Errorf("initial version set to %v, but found earlier version %v",
			tree.ndb.opts.InitialVersion, firstVersion)
	}

	if latestVersion < targetVersion {
		return latestVersion, fmt.Errorf("wanted to load target %d but only found up to %d", targetVersion, latestVersion)
	}

	if firstVersion == 0 {
		if targetVersion <= 0 {
			if !tree.skipFastStorageUpgrade {
				tree.mtx.Lock()
				defer tree.mtx.Unlock()
				_, err := tree.enableFastStorageAndCommitIfNotEnabled()
				return 0, err
			}
			return 0, nil
		}
		return 0, fmt.Errorf("no versions found while trying to load %v", targetVersion)
	}

	if targetVersion <= 0 {
		targetVersion = latestVersion
	}
	if !tree.VersionExists(targetVersion) {
		return 0, ErrVersionDoesNotExist
	}
	rootNodeKey, err := tree.ndb.GetRoot(targetVersion)
	if err != nil {
		return 0, err
	}

	iTree := &ImmutableTree{
		ndb:                    tree.ndb,
		version:                targetVersion,
		skipFastStorageUpgrade: tree.skipFastStorageUpgrade,
	}

	if rootNodeKey != nil {
		iTree.root, err = tree.ndb.GetNode(rootNodeKey)
		if err != nil {
			return 0, err
		}
	}

	tree.ImmutableTree = iTree
	tree.lastSaved = iTree.clone()

	if !tree.skipFastStorageUpgrade {
		// Attempt to upgrade
		if _, err := tree.enableFastStorageAndCommitIfNotEnabled(); err != nil {
			return 0, err
		}
	}

	return latestVersion, nil
}

// loadVersionForOverwriting attempts to load a tree at a previously committed
// version, or the latest version below it. Any versions greater than targetVersion will be deleted.
func (tree *MutableTree) LoadVersionForOverwriting(targetVersion int64) error {
	if _, err := tree.LoadVersion(targetVersion); err != nil {
		return err
	}

	if err := tree.ndb.DeleteVersionsFrom(targetVersion + 1); err != nil {
		return err
	}

	// Commit the tree rollback first
	// The fast storage rebuild don't have to be atomic with this,
	// because it's idempotent and will do again when `LoadVersion`.
	if err := tree.ndb.Commit(); err != nil {
		return err
	}

	if !tree.skipFastStorageUpgrade {
		// it'll repopulates the fast node index because of version mismatch.
		if _, err := tree.enableFastStorageAndCommitIfNotEnabled(); err != nil {
			return err
		}
	}

	return nil
}

// Returns true if the tree may be auto-upgraded, false otherwise
// An example of when an upgrade may be performed is when we are enaling fast storage for the first time or
// need to overwrite fast nodes due to mismatch with live state.
func (tree *MutableTree) IsUpgradeable() (bool, error) {
	shouldForce, err := tree.ndb.shouldForceFastStorageUpgrade()
	if err != nil {
		return false, err
	}
	return !tree.skipFastStorageUpgrade && (!tree.ndb.hasUpgradedToFastStorage() || shouldForce), nil
}

// enableFastStorageAndCommitIfNotEnabled if nodeDB doesn't mark fast storage as enabled, enable it, and commit the update.
// Checks whether the fast cache on disk matches latest live state. If not, deletes all existing fast nodes and repopulates them
// from latest tree.

func (tree *MutableTree) enableFastStorageAndCommitIfNotEnabled() (bool, error) {
	isUpgradeable, err := tree.IsUpgradeable()
	if err != nil {
		return false, err
	}

	if !isUpgradeable {
		return false, nil
	}

	// If there is a mismatch between which fast nodes are on disk and the live state due to temporary
	// downgrade and subsequent re-upgrade, we cannot know for sure which fast nodes have been removed while downgraded,
	// Therefore, there might exist stale fast nodes on disk. As a result, to avoid persisting the stale state, it might
	// be worth to delete the fast nodes from disk.
	fastItr := NewFastIterator(nil, nil, true, tree.ndb)
	defer fastItr.Close()
	var deletedFastNodes uint64
	for ; fastItr.Valid(); fastItr.Next() {
		deletedFastNodes++
		if err := tree.ndb.DeleteFastNode(fastItr.Key()); err != nil {
			return false, err
		}
	}

	if err := tree.enableFastStorageAndCommit(); err != nil {
		tree.ndb.storageVersion = defaultStorageVersionValue
		return false, err
	}
	return true, nil
}

func (tree *MutableTree) enableFastStorageAndCommit() error {
	var err error

	itr := NewIterator(nil, nil, true, tree.ImmutableTree)
	defer itr.Close()
	var upgradedFastNodes uint64
	for ; itr.Valid(); itr.Next() {
		upgradedFastNodes++
		if err = tree.ndb.SaveFastNodeNoCache(fastnode.NewNode(itr.Key(), itr.Value(), tree.version)); err != nil {
			return err
		}
	}

	if err = itr.Error(); err != nil {
		return err
	}

	latestVersion, err := tree.ndb.getLatestVersion()
	if err != nil {
		return err
	}

	if err = tree.ndb.SetFastStorageVersionToBatch(latestVersion); err != nil {
		return err
	}

	return tree.ndb.Commit()
}

// GetImmutable loads an ImmutableTree at a given version for querying. The returned tree is
// safe for concurrent access, provided the version is not deleted, e.g. via `DeleteVersion()`.
func (tree *MutableTree) GetImmutable(version int64) (*ImmutableTree, error) {
	rootNodeKey, err := tree.ndb.GetRoot(version)
	if err != nil {
		return nil, err
	}

	var root *Node
	if rootNodeKey != nil {
		root, err = tree.ndb.GetNode(rootNodeKey)
		if err != nil {
			return nil, err
		}
	}

	return &ImmutableTree{
		root:                   root,
		ndb:                    tree.ndb,
		version:                version,
		skipFastStorageUpgrade: tree.skipFastStorageUpgrade,
	}, nil
}

// Rollback resets the working tree to the latest saved version, discarding
// any unsaved modifications.
func (tree *MutableTree) Rollback() {
	if tree.version > 0 {
		tree.ImmutableTree = tree.lastSaved.clone()
	} else {
		tree.ImmutableTree = &ImmutableTree{
			ndb:                    tree.ndb,
			version:                0,
			skipFastStorageUpgrade: tree.skipFastStorageUpgrade,
		}
	}
	if !tree.skipFastStorageUpgrade {
		tree.unsavedFastNodeAdditions = &sync.Map{}
		tree.unsavedFastNodeRemovals = &sync.Map{}
	}
}

// GetVersioned gets the value at the specified key and version. The returned value must not be
// modified, since it may point to data stored within IAVL.
func (tree *MutableTree) GetVersioned(key []byte, version int64) ([]byte, error) {
	if tree.VersionExists(version) {
		if !tree.skipFastStorageUpgrade {
			isFastCacheEnabled, err := tree.IsFastCacheEnabled()
			if err != nil {
				return nil, err
			}

			if isFastCacheEnabled {
				fastNode, _ := tree.ndb.GetFastNode(key)
				if fastNode == nil && version == tree.ndb.latestVersion {
					return nil, nil
				}

				if fastNode != nil && fastNode.GetVersionLastUpdatedAt() <= version {
					return fastNode.GetValue(), nil
				}
			}
		}
		t, err := tree.GetImmutable(version)
		if err != nil {
			return nil, nil
		}
		value, err := t.Get(key)
		if err != nil {
			return nil, err
		}
		return value, nil
	}
	return nil, nil
}

// SaveVersion saves a new tree version to disk, based on the current state of
// the tree. Returns the hash and new version number.
func (tree *MutableTree) SaveVersion() ([]byte, int64, error) {
	version := tree.WorkingVersion()

	if tree.VersionExists(version) {
		// If the version already exists, return an error as we're attempting to overwrite.
		// However, the same hash means idempotent (i.e. no-op).
		existingNodeKey, err := tree.ndb.GetRoot(version)
		if err != nil {
			return nil, version, err
		}
		var existingRoot *Node
		if existingNodeKey != nil {
			existingRoot, err = tree.ndb.GetNode(existingNodeKey)
			if err != nil {
				return nil, version, err
			}
		}

		newHash := tree.WorkingHash()

		if (existingRoot == nil && tree.root == nil) || (existingRoot != nil && bytes.Equal(existingRoot.hash, newHash)) { // TODO with WorkingHash
			tree.version = version
			tree.root = existingRoot
			tree.ImmutableTree = tree.ImmutableTree.clone()
			tree.lastSaved = tree.ImmutableTree.clone()
			return newHash, version, nil
		}

		return nil, version, fmt.Errorf("version %d was already saved to different hash from %X (existing nodeKey %d)", version, newHash, existingNodeKey)
	}

	tree.logger.Debug("SAVE TREE", "version", version)

	// save new fast nodes
	if !tree.skipFastStorageUpgrade {
		if err := tree.saveFastNodeVersion(version); err != nil {
			return nil, version, err
		}
	}
	// save new nodes
	if tree.root == nil {
		if err := tree.ndb.SaveEmptyRoot(version); err != nil {
			return nil, 0, err
		}
	} else {
		if tree.root.nodeKey != nil {
			// it means there are no updated nodes
			if err := tree.ndb.SaveRoot(version, tree.root.nodeKey); err != nil {
				return nil, 0, err
			}
			// it means the reference node is a legacy node
			if tree.root.isLegacy {
				// it will update the legacy node to the new format
				// which ensures the reference node is not a legacy node
				tree.root.isLegacy = false
				if err := tree.ndb.SaveNode(tree.root); err != nil {
					return nil, 0, fmt.Errorf("failed to save the reference legacy node: %w", err)
				}
			}
		} else {
			if err := tree.saveNewNodes(version); err != nil {
				return nil, 0, err
			}
		}
	}

	if err := tree.ndb.Commit(); err != nil {
		return nil, version, err
	}

	tree.ndb.resetLatestVersion(version)
	tree.version = version

	// set new working tree
	tree.ImmutableTree = tree.ImmutableTree.clone()
	tree.lastSaved = tree.ImmutableTree.clone()
	if !tree.skipFastStorageUpgrade {
		tree.unsavedFastNodeAdditions = &sync.Map{}
		tree.unsavedFastNodeRemovals = &sync.Map{}
	}

	return tree.Hash(), version, nil
}

func (tree *MutableTree) saveFastNodeVersion(latestVersion int64) error {
	if err := tree.saveFastNodeAdditions(); err != nil {
		return err
	}
	if err := tree.saveFastNodeRemovals(); err != nil {
		return err
	}
	return tree.ndb.SetFastStorageVersionToBatch(latestVersion)
}

func (tree *MutableTree) getUnsavedFastNodeAdditions() map[string]*fastnode.Node {
	additions := make(map[string]*fastnode.Node)
	tree.unsavedFastNodeAdditions.Range(func(key, value interface{}) bool {
		additions[key.(string)] = value.(*fastnode.Node)
		return true
	})
	return additions
}

// getUnsavedFastNodeRemovals returns unsaved FastNodes to remove

func (tree *MutableTree) getUnsavedFastNodeRemovals() map[string]interface{} {
	removals := make(map[string]interface{})
	tree.unsavedFastNodeRemovals.Range(func(key, value interface{}) bool {
		removals[key.(string)] = value
		return true
	})
	return removals
}

// addUnsavedAddition stores an addition into the unsaved additions map
func (tree *MutableTree) addUnsavedAddition(key []byte, node *fastnode.Node) {
	skey := ibytes.UnsafeBytesToStr(key)
	tree.unsavedFastNodeRemovals.Delete(skey)
	tree.unsavedFastNodeAdditions.Store(skey, node)
}

func (tree *MutableTree) saveFastNodeAdditions() error {
	keysToSort := make([]string, 0)
	tree.unsavedFastNodeAdditions.Range(func(k, v interface{}) bool {
		keysToSort = append(keysToSort, k.(string))
		return true
	})
	sort.Strings(keysToSort)

	for _, key := range keysToSort {
		val, _ := tree.unsavedFastNodeAdditions.Load(key)
		if err := tree.ndb.SaveFastNode(val.(*fastnode.Node)); err != nil {
			return err
		}
	}
	return nil
}

// addUnsavedRemoval adds a removal to the unsaved removals map
func (tree *MutableTree) addUnsavedRemoval(key []byte) {
	skey := ibytes.UnsafeBytesToStr(key)
	tree.unsavedFastNodeAdditions.Delete(skey)
	tree.unsavedFastNodeRemovals.Store(skey, true)
}

func (tree *MutableTree) saveFastNodeRemovals() error {
	keysToSort := make([]string, 0)
	tree.unsavedFastNodeRemovals.Range(func(k, v interface{}) bool {
		keysToSort = append(keysToSort, k.(string))
		return true
	})
	sort.Strings(keysToSort)

	for _, key := range keysToSort {
		if err := tree.ndb.DeleteFastNode(ibytes.UnsafeStrToBytes(key)); err != nil {
			return err
		}
	}
	return nil
}

// SetInitialVersion sets the initial version of the tree, replacing Options.InitialVersion.
// It is only used during the initial SaveVersion() call for a tree with no other versions,
// and is otherwise ignored.
func (tree *MutableTree) SetInitialVersion(version uint64) {
	tree.ndb.opts.InitialVersion = version
}

// DeleteVersionsTo removes versions upto the given version from the MutableTree.
// It will not block the SaveVersion() call, instead it will be queued and executed deferred.
func (tree *MutableTree) DeleteVersionsTo(toVersion int64) error {
	if err := tree.ndb.DeleteVersionsTo(toVersion); err != nil {
		return err
	}

	return tree.ndb.Commit()
}

// Rotate right and return the new node and orphan.
func (tree *MutableTree) rotateRight(node *Node) (*Node, error) {
	var err error
	// TODO: optimize balance & rotate.
	node, err = node.clone(tree)
	if err != nil {
		return nil, err
	}

	newNode, err := node.leftNode.clone(tree)
	if err != nil {
		return nil, err
	}

	node.leftNode = newNode.rightNode
	newNode.rightNode = node

	err = node.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, err
	}

	err = newNode.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, err
	}

	return newNode, nil
}

// Rotate left and return the new node and orphan.
func (tree *MutableTree) rotateLeft(node *Node) (*Node, error) {
	var err error
	// TODO: optimize balance & rotate.
	node, err = node.clone(tree)
	if err != nil {
		return nil, err
	}

	newNode, err := node.rightNode.clone(tree)
	if err != nil {
		return nil, err
	}

	node.rightNode = newNode.leftNode
	newNode.leftNode = node

	err = node.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, err
	}

	err = newNode.calcHeightAndSize(tree.ImmutableTree)
	if err != nil {
		return nil, err
	}

	return newNode, nil
}

// NOTE: assumes that node can be modified
// TODO: optimize balance & rotate
func (tree *MutableTree) balance(node *Node) (newSelf *Node, err error) {
	if node.nodeKey != nil {
		return nil, fmt.Errorf("unexpected balance() call on persisted node")
	}
	balance, err := node.calcBalance(tree.ImmutableTree)
	if err != nil {
		return nil, err
	}

	if balance > 1 {
		lftBalance, err := node.leftNode.calcBalance(tree.ImmutableTree)
		if err != nil {
			return nil, err
		}

		if lftBalance >= 0 {
			// Left Left Case
			newNode, err := tree.rotateRight(node)
			if err != nil {
				return nil, err
			}
			return newNode, nil
		}
		// Left Right Case
		node.leftNodeKey = nil
		node.leftNode, err = tree.rotateLeft(node.leftNode)
		if err != nil {
			return nil, err
		}

		newNode, err := tree.rotateRight(node)
		if err != nil {
			return nil, err
		}

		return newNode, nil
	}
	if balance < -1 {
		rightNode, err := node.getRightNode(tree.ImmutableTree)
		if err != nil {
			return nil, err
		}

		rightBalance, err := rightNode.calcBalance(tree.ImmutableTree)
		if err != nil {
			return nil, err
		}
		if rightBalance <= 0 {
			// Right Right Case
			newNode, err := tree.rotateLeft(node)
			if err != nil {
				return nil, err
			}
			return newNode, nil
		}
		// Right Left Case
		node.rightNodeKey = nil
		node.rightNode, err = tree.rotateRight(rightNode)
		if err != nil {
			return nil, err
		}
		newNode, err := tree.rotateLeft(node)
		if err != nil {
			return nil, err
		}
		return newNode, nil
	}
	// Nothing changed
	return node, nil
}

// saveNewNodes save new created nodes by the changes of the working tree.
// NOTE: This function clears leftNode/rigthNode recursively and
// calls _hash() on the given node.
func (tree *MutableTree) saveNewNodes(version int64) error {
	nonce := uint32(0)
	newNodes := make([]*Node, 0)
	var recursiveAssignKey func(*Node) ([]byte, error)
	recursiveAssignKey = func(node *Node) ([]byte, error) {
		if node.nodeKey != nil {
			return node.GetKey(), nil
		}
		nonce++
		node.nodeKey = &NodeKey{
			version: version,
			nonce:   nonce,
		}

		var err error
		// the inner nodes should have two children.
		if node.subtreeHeight > 0 {
			node.leftNodeKey, err = recursiveAssignKey(node.leftNode)
			if err != nil {
				return nil, err
			}
			node.rightNodeKey, err = recursiveAssignKey(node.rightNode)
			if err != nil {
				return nil, err
			}
		}

		node._hash(version)
		newNodes = append(newNodes, node)

		return node.nodeKey.GetKey(), nil
	}

	if _, err := recursiveAssignKey(tree.root); err != nil {
		return err
	}

	for _, node := range newNodes {
		if err := tree.ndb.SaveNode(node); err != nil {
			return err
		}
		node.leftNode, node.rightNode = nil, nil
	}

	return nil
}

// SaveChangeSet saves a ChangeSet to the tree.
// It is used to replay a ChangeSet as a new version.
func (tree *MutableTree) SaveChangeSet(cs *ChangeSet) (int64, error) {
	// if the tree has uncommitted changes, return error
	if tree.root != nil && tree.root.nodeKey == nil {
		return 0, fmt.Errorf("cannot save changeset with uncommitted changes")
	}
	for _, pair := range cs.Pairs {
		if pair.Delete {
			_, removed, err := tree.Remove(pair.Key)
			if !removed {
				return 0, fmt.Errorf("attempted to remove non-existent key %s", pair.Key)
			}
			if err != nil {
				return 0, err
			}
		} else {
			if _, err := tree.Set(pair.Key, pair.Value); err != nil {
				return 0, err
			}
		}
	}
	_, version, err := tree.SaveVersion()
	return version, err
}

// Close closes the tree.
func (tree *MutableTree) Close() error {
	tree.mtx.Lock()
	defer tree.mtx.Unlock()

	tree.ImmutableTree = nil
	tree.lastSaved = nil
	return tree.ndb.Close()
}
