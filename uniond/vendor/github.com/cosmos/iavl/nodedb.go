package iavl

import (
	"bytes"
	"crypto/sha256"
	"errors"
	"fmt"
	"math"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"

	"cosmossdk.io/log"
	dbm "github.com/cosmos/cosmos-db"

	"github.com/cosmos/iavl/cache"
	"github.com/cosmos/iavl/fastnode"
	"github.com/cosmos/iavl/keyformat"
)

const (
	int32Size         = 4
	int64Size         = 8
	hashSize          = sha256.Size
	genesisVersion    = 1
	storageVersionKey = "storage_version"
	// We store latest saved version together with storage version delimited by the constant below.
	// This delimiter is valid only if fast storage is enabled (i.e. storageVersion >= fastStorageVersionValue).
	// The latest saved version is needed for protection against downgrade and re-upgrade. In such a case, it would
	// be possible to observe mismatch between the latest version state and the fast nodes on disk.
	// Therefore, we would like to detect that and overwrite fast nodes on disk with the latest version state.
	fastStorageVersionDelimiter = "-"
	// Using semantic versioning: https://semver.org/
	defaultStorageVersionValue = "1.0.0"
	fastStorageVersionValue    = "1.1.0"
	fastNodeCacheSize          = 100000
)

var (
	// All new node keys are prefixed with the byte 's'. This ensures no collision is
	// possible with the legacy nodes, and makes them easier to traverse. They are indexed by the version and the local nonce.
	nodeKeyFormat = keyformat.NewFastPrefixFormatter('s', int64Size+int32Size) // s<version><nonce>

	// This is only used for the iteration purpose.
	nodeKeyPrefixFormat = keyformat.NewFastPrefixFormatter('s', int64Size) // s<version>

	// Key Format for making reads and iterates go through a data-locality preserving db.
	// The value at an entry will list what version it was written to.
	// Then to query values, you first query state via this fast method.
	// If its present, then check the tree version. If tree version >= result_version,
	// return result_version. Else, go through old (slow) IAVL get method that walks through tree.
	fastKeyFormat = keyformat.NewKeyFormat('f', 0) // f<keystring>

	// Key Format for storing metadata about the chain such as the version number.
	// The value at an entry will be in a variable format and up to the caller to
	// decide how to parse.
	metadataKeyFormat = keyformat.NewKeyFormat('m', 0) // m<keystring>

	// All legacy node keys are prefixed with the byte 'n'.
	legacyNodeKeyFormat = keyformat.NewFastPrefixFormatter('n', hashSize) // n<hash>

	// All legacy orphan keys are prefixed with the byte 'o'.
	legacyOrphanKeyFormat = keyformat.NewKeyFormat('o', int64Size, int64Size, hashSize) // o<last-version><first-version><hash>

	// All legacy root keys are prefixed with the byte 'r'.
	legacyRootKeyFormat = keyformat.NewKeyFormat('r', int64Size) // r<version>
)

var errInvalidFastStorageVersion = fmt.Errorf("fast storage version must be in the format <storage version>%s<latest fast cache version>", fastStorageVersionDelimiter)

type nodeDB struct {
	logger log.Logger

	mtx                 sync.Mutex       // Read/write lock.
	db                  dbm.DB           // Persistent node storage.
	batch               dbm.Batch        // Batched writing buffer.
	opts                Options          // Options to customize for pruning/writing
	versionReaders      map[int64]uint32 // Number of active version readers
	storageVersion      string           // Storage version
	firstVersion        int64            // First version of nodeDB.
	latestVersion       int64            // Latest version of nodeDB.
	legacyLatestVersion int64            // Latest version of nodeDB in legacy format.
	nodeCache           cache.Cache      // Cache for nodes in the regular tree that consists of key-value pairs at any version.
	fastNodeCache       cache.Cache      // Cache for nodes in the fast index that represents only key-value pairs at the latest version.
}

func newNodeDB(db dbm.DB, cacheSize int, opts Options, lg log.Logger) *nodeDB {
	storeVersion, err := db.Get(metadataKeyFormat.Key([]byte(storageVersionKey)))

	if err != nil || storeVersion == nil {
		storeVersion = []byte(defaultStorageVersionValue)
	}

	return &nodeDB{
		logger:              lg,
		db:                  db,
		batch:               NewBatchWithFlusher(db, opts.FlushThreshold),
		opts:                opts,
		firstVersion:        0,
		latestVersion:       0, // initially invalid
		legacyLatestVersion: 0,
		nodeCache:           cache.New(cacheSize),
		fastNodeCache:       cache.New(fastNodeCacheSize),
		versionReaders:      make(map[int64]uint32, 8),
		storageVersion:      string(storeVersion),
	}
}

// GetNode gets a node from memory or disk. If it is an inner node, it does not
// load its children.
// It is used for both formats of nodes: legacy and new.
// `legacy`: nk is the hash of the node. `new`: <version><nonce>.
func (ndb *nodeDB) GetNode(nk []byte) (*Node, error) {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()

	if nk == nil {
		return nil, ErrNodeMissingNodeKey
	}

	// Check the cache.
	if cachedNode := ndb.nodeCache.Get(nk); cachedNode != nil {
		ndb.opts.Stat.IncCacheHitCnt()
		return cachedNode.(*Node), nil
	}

	ndb.opts.Stat.IncCacheMissCnt()

	// Doesn't exist, load.
	isLegcyNode := len(nk) == hashSize
	var nodeKey []byte
	if isLegcyNode {
		nodeKey = ndb.legacyNodeKey(nk)
	} else {
		nodeKey = ndb.nodeKey(nk)
	}
	buf, err := ndb.db.Get(nodeKey)
	if err != nil {
		return nil, fmt.Errorf("can't get node %v: %v", nk, err)
	}
	if buf == nil {
		return nil, fmt.Errorf("Value missing for key %v corresponding to nodeKey %x", nk, nodeKey)
	}

	var node *Node
	if isLegcyNode {
		node, err = MakeLegacyNode(nk, buf)
		if err != nil {
			return nil, fmt.Errorf("error reading Legacy Node. bytes: %x, error: %v", buf, err)
		}
	} else {
		node, err = MakeNode(nk, buf)
		if err != nil {
			return nil, fmt.Errorf("error reading Node. bytes: %x, error: %v", buf, err)
		}
	}

	ndb.nodeCache.Add(node)

	return node, nil
}

func (ndb *nodeDB) GetFastNode(key []byte) (*fastnode.Node, error) {
	if !ndb.hasUpgradedToFastStorage() {
		return nil, errors.New("storage version is not fast")
	}

	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()

	if len(key) == 0 {
		return nil, fmt.Errorf("nodeDB.GetFastNode() requires key, len(key) equals 0")
	}

	if cachedFastNode := ndb.fastNodeCache.Get(key); cachedFastNode != nil {
		ndb.opts.Stat.IncFastCacheHitCnt()
		return cachedFastNode.(*fastnode.Node), nil
	}

	ndb.opts.Stat.IncFastCacheMissCnt()

	// Doesn't exist, load.
	buf, err := ndb.db.Get(ndb.fastNodeKey(key))
	if err != nil {
		return nil, fmt.Errorf("can't get FastNode %X: %w", key, err)
	}
	if buf == nil {
		return nil, nil
	}

	fastNode, err := fastnode.DeserializeNode(key, buf)
	if err != nil {
		return nil, fmt.Errorf("error reading FastNode. bytes: %x, error: %w", buf, err)
	}
	ndb.fastNodeCache.Add(fastNode)
	return fastNode, nil
}

// SaveNode saves a node to disk.
func (ndb *nodeDB) SaveNode(node *Node) error {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()

	if node.nodeKey == nil {
		return ErrNodeMissingNodeKey
	}

	// Save node bytes to db.
	var buf bytes.Buffer
	buf.Grow(node.encodedSize())

	if err := node.writeBytes(&buf); err != nil {
		return err
	}

	if err := ndb.batch.Set(ndb.nodeKey(node.GetKey()), buf.Bytes()); err != nil {
		return err
	}

	// resetBatch only working on generate a genesis block
	if node.nodeKey.version <= genesisVersion {
		if err := ndb.resetBatch(); err != nil {
			return err
		}
	}

	ndb.logger.Debug("BATCH SAVE", "node", node)
	ndb.nodeCache.Add(node)
	return nil
}

// SaveFastNode saves a FastNode to disk and add to cache.
func (ndb *nodeDB) SaveFastNode(node *fastnode.Node) error {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	return ndb.saveFastNodeUnlocked(node, true)
}

// SaveFastNodeNoCache saves a FastNode to disk without adding to cache.
func (ndb *nodeDB) SaveFastNodeNoCache(node *fastnode.Node) error {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	return ndb.saveFastNodeUnlocked(node, false)
}

// setFastStorageVersionToBatch sets storage version to fast where the version is
// 1.1.0-<version of the current live state>. Returns error if storage version is incorrect or on
// db error, nil otherwise. Requires changes to be committed after to be persisted.
func (ndb *nodeDB) setFastStorageVersionToBatch(latestVersion int64) error {
	var newVersion string
	if ndb.storageVersion >= fastStorageVersionValue {
		// Storage version should be at index 0 and latest fast cache version at index 1
		versions := strings.Split(ndb.storageVersion, fastStorageVersionDelimiter)

		if len(versions) > 2 {
			return errInvalidFastStorageVersion
		}

		newVersion = versions[0]
	} else {
		newVersion = fastStorageVersionValue
	}

	newVersion += fastStorageVersionDelimiter + strconv.Itoa(int(latestVersion))

	if err := ndb.batch.Set(metadataKeyFormat.Key([]byte(storageVersionKey)), []byte(newVersion)); err != nil {
		return err
	}
	ndb.storageVersion = newVersion
	return nil
}

func (ndb *nodeDB) getStorageVersion() string {
	return ndb.storageVersion
}

// Returns true if the upgrade to latest storage version has been performed, false otherwise.
func (ndb *nodeDB) hasUpgradedToFastStorage() bool {
	return ndb.getStorageVersion() >= fastStorageVersionValue
}

// Returns true if the upgrade to fast storage has occurred but it does not match the live state, false otherwise.
// When the live state is not matched, we must force reupgrade.
// We determine this by checking the version of the live state and the version of the live state when
// latest storage was updated on disk the last time.
func (ndb *nodeDB) shouldForceFastStorageUpgrade() (bool, error) {
	versions := strings.Split(ndb.storageVersion, fastStorageVersionDelimiter)

	if len(versions) == 2 {
		latestVersion, err := ndb.getLatestVersion()
		if err != nil {
			// TODO: should be true or false as default? (removed panic here)
			return false, err
		}
		if versions[1] != strconv.Itoa(int(latestVersion)) {
			return true, nil
		}
	}
	return false, nil
}

// saveFastNodeUnlocked saves a FastNode to disk.
func (ndb *nodeDB) saveFastNodeUnlocked(node *fastnode.Node, shouldAddToCache bool) error {
	if node.GetKey() == nil {
		return fmt.Errorf("cannot have FastNode with a nil value for key")
	}

	// Save node bytes to db.
	var buf bytes.Buffer
	buf.Grow(node.EncodedSize())

	if err := node.WriteBytes(&buf); err != nil {
		return fmt.Errorf("error while writing fastnode bytes. Err: %w", err)
	}

	if err := ndb.batch.Set(ndb.fastNodeKey(node.GetKey()), buf.Bytes()); err != nil {
		return fmt.Errorf("error while writing key/val to nodedb batch. Err: %w", err)
	}
	if shouldAddToCache {
		ndb.fastNodeCache.Add(node)
	}
	return nil
}

// Has checks if a node key exists in the database.
func (ndb *nodeDB) Has(nk []byte) (bool, error) {
	key := ndb.nodeKey(nk)

	if ldb, ok := ndb.db.(*dbm.GoLevelDB); ok {
		exists, err := ldb.DB().Has(key, nil)
		if err != nil {
			return false, err
		}
		return exists, nil
	}
	value, err := ndb.db.Get(key)
	if err != nil {
		return false, err
	}

	return value != nil, nil
}

// resetBatch reset the db batch, keep low memory used
func (ndb *nodeDB) resetBatch() error {
	size, err := ndb.batch.GetByteSize()
	if err != nil {
		// just don't do an optimization here. write with batch size 1.
		return ndb.writeBatch()
	}
	// write in ~64kb chunks. if less than 64kb, continue.
	if size < 64*1024 {
		return nil
	}

	return ndb.writeBatch()
}

func (ndb *nodeDB) writeBatch() error {
	var err error
	if ndb.opts.Sync {
		err = ndb.batch.WriteSync()
	} else {
		err = ndb.batch.Write()
	}
	if err != nil {
		return err
	}
	err = ndb.batch.Close()
	if err != nil {
		return err
	}

	ndb.batch = NewBatchWithFlusher(ndb.db, ndb.opts.FlushThreshold)

	return nil
}

// deleteVersion deletes a tree version from disk.
// deletes orphans
func (ndb *nodeDB) deleteVersion(version int64) error {
	rootKey, err := ndb.GetRoot(version)
	if err != nil {
		return err
	}
	if rootKey == nil || GetNodeKey(rootKey).version < version {
		if err := ndb.batch.Delete(ndb.nodeKey(GetRootKey(version))); err != nil {
			return err
		}
	}

	return ndb.traverseOrphans(version, version+1, func(orphan *Node) error {
		return ndb.batch.Delete(ndb.nodeKey(orphan.GetKey()))
	})
}

// deleteLegacyNodes deletes all legacy nodes with the given version from disk.
// NOTE: This is only used for DeleteVersionsFrom.
func (ndb *nodeDB) deleteLegacyNodes(version int64, nk []byte) error {
	node, err := ndb.GetNode(nk)
	if err != nil {
		return err
	}
	if node.nodeKey.version < version {
		// it will skip the whole subtree.
		return nil
	}
	if node.leftNodeKey != nil {
		if err := ndb.deleteLegacyNodes(version, node.leftNodeKey); err != nil {
			return err
		}
	}
	if node.rightNodeKey != nil {
		if err := ndb.deleteLegacyNodes(version, node.rightNodeKey); err != nil {
			return err
		}
	}
	return ndb.batch.Delete(ndb.legacyNodeKey(nk))
}

var (
	isDeletingLegacyVersionsMutex = &sync.Mutex{}
	isDeletingLegacyVersions      = false
)

// deleteLegacyVersions deletes all legacy versions from disk.
func (ndb *nodeDB) deleteLegacyVersions() error {
	isDeletingLegacyVersionsMutex.Lock()
	if isDeletingLegacyVersions {
		isDeletingLegacyVersionsMutex.Unlock()
		return nil
	}
	isDeletingLegacyVersions = true
	isDeletingLegacyVersionsMutex.Unlock()

	go func() {
		defer func() {
			isDeletingLegacyVersionsMutex.Lock()
			isDeletingLegacyVersions = false
			isDeletingLegacyVersionsMutex.Unlock()
		}()

		// Check if we have a legacy version
		itr, err := dbm.IteratePrefix(ndb.db, legacyRootKeyFormat.Key())
		if err != nil {
			ndb.logger.Error(err.Error())
			return
		}
		defer itr.Close()

		// Delete orphans for all legacy versions
		var prevVersion, curVersion int64
		var rootKeys [][]byte
		counter := 0
		for ; itr.Valid(); itr.Next() {
			legacyRootKeyFormat.Scan(itr.Key(), &curVersion)
			rootKeys = append(rootKeys, itr.Key())
			if prevVersion > 0 {
				if err := ndb.traverseOrphans(prevVersion, curVersion, func(orphan *Node) error {
					counter++
					if counter == 1000 {
						counter = 0
						time.Sleep(1000 * time.Millisecond)
						fmt.Println("IAVL sleep happening")
					}
					return ndb.batch.Delete(ndb.nodeKey(orphan.GetKey()))
				}); err != nil {
					ndb.logger.Error(err.Error())
					return
				}
			}
			prevVersion = curVersion
		}
		// Delete the last version for the legacyLastVersion
		if curVersion > 0 {
			legacyLatestVersion, err := ndb.getLegacyLatestVersion()
			if err != nil {
				ndb.logger.Error(err.Error())
				return
			}
			if curVersion != legacyLatestVersion {
				ndb.logger.Error("expected legacyLatestVersion to be %d, got %d", legacyLatestVersion, curVersion)
				return
			}
			if err := ndb.traverseOrphans(curVersion, curVersion+1, func(orphan *Node) error {
				return ndb.batch.Delete(ndb.nodeKey(orphan.GetKey()))
			}); err != nil {
				ndb.logger.Error("failed to clean legacy orphans between versions", "err", err)
				return
			}
		}

		// Delete all roots of the legacy versions
		for _, rootKey := range rootKeys {
			if err := ndb.batch.Delete(rootKey); err != nil {
				ndb.logger.Error("failed to clean legacy orphans root keys", "err", err)
				return
			}
		}

		// Initialize the legacy latest version to -1 to demonstrate that all legacy versions have been deleted
		ndb.legacyLatestVersion = -1

		// Delete all orphan nodes of the legacy versions
		// TODO: Is this just deadcode?????
		if err := ndb.deleteOrphans(); err != nil {
			ndb.logger.Error("failed to clean legacy orphans", "err", err)
			return
		}
	}()

	return nil
}

// deleteOrphans cleans all legacy orphans from the nodeDB.
func (ndb *nodeDB) deleteOrphans() error {
	itr, err := dbm.IteratePrefix(ndb.db, legacyOrphanKeyFormat.Key())
	if err != nil {
		return err
	}
	defer itr.Close()

	count := 0
	for ; itr.Valid(); itr.Next() {
		if err := ndb.batch.Delete(itr.Key()); err != nil {
			return err
		}

		// Sleep for a while to avoid blocking the main thread i/o.
		count++
		if count > 1000 {
			count = 0
			time.Sleep(100 * time.Millisecond)
		}

	}

	return nil
}

// DeleteVersionsFrom permanently deletes all tree versions from the given version upwards.
func (ndb *nodeDB) DeleteVersionsFrom(fromVersion int64) error {
	latest, err := ndb.getLatestVersion()
	if err != nil {
		return err
	}
	if latest < fromVersion {
		return nil
	}

	ndb.mtx.Lock()
	for v, r := range ndb.versionReaders {
		if v >= fromVersion && r != 0 {
			ndb.mtx.Unlock() // Unlock before exiting
			return fmt.Errorf("unable to delete version %v with %v active readers", v, r)
		}
	}
	ndb.mtx.Unlock()

	// Delete the legacy versions
	legacyLatestVersion, err := ndb.getLegacyLatestVersion()
	if err != nil {
		return err
	}
	dumpFromVersion := fromVersion
	if legacyLatestVersion >= fromVersion {
		if err := ndb.traverseRange(legacyRootKeyFormat.Key(fromVersion), legacyRootKeyFormat.Key(legacyLatestVersion+1), func(k, v []byte) error {
			var version int64
			legacyRootKeyFormat.Scan(k, &version)
			// delete the legacy nodes
			if err := ndb.deleteLegacyNodes(version, v); err != nil {
				return err
			}
			// delete the legacy root
			// it will skip the orphans because orphans will be removed at once in `deleteLegacyVersions`
			return ndb.batch.Delete(k)
		}); err != nil {
			return err
		}
		// Update the legacy latest version forcibly
		ndb.legacyLatestVersion = 0
		fromVersion = legacyLatestVersion + 1
	}

	// Delete the nodes for new format
	err = ndb.traverseRange(nodeKeyPrefixFormat.KeyInt64(fromVersion), nodeKeyPrefixFormat.KeyInt64(latest+1), func(k, v []byte) error {
		return ndb.batch.Delete(k)
	})

	if err != nil {
		return err
	}

	// NOTICE: we don't touch fast node indexes here, because it'll be rebuilt later because of version mismatch.

	ndb.resetLatestVersion(dumpFromVersion - 1)

	return nil
}

// DeleteVersionsTo deletes the oldest versions up to the given version from disk.
func (ndb *nodeDB) DeleteVersionsTo(toVersion int64) error {
	legacyLatestVersion, err := ndb.getLegacyLatestVersion()
	if err != nil {
		return err
	}
	// If the legacy version is greater than the toVersion, we don't need to delete anything.
	// It will delete the legacy versions at once.
	if legacyLatestVersion > toVersion {
		return nil
	}

	first, err := ndb.getFirstVersion()
	if err != nil {
		return err
	}

	latest, err := ndb.getLatestVersion()
	if err != nil {
		return err
	}

	if latest <= toVersion {
		return fmt.Errorf("the version should be smaller than the latest version %d", latest)
	}

	for v, r := range ndb.versionReaders {
		if v >= first && v <= toVersion && r != 0 {
			return fmt.Errorf("unable to delete version %v with %v active readers", v, r)
		}
	}

	// Delete the legacy versions
	if legacyLatestVersion >= first {
		if err := ndb.deleteLegacyVersions(); err != nil {
			return err
		}
		first = legacyLatestVersion + 1
	}

	for version := first; version <= toVersion; version++ {
		if err := ndb.deleteVersion(version); err != nil {
			return err
		}
		ndb.resetFirstVersion(version + 1)
	}

	return nil
}

func (ndb *nodeDB) DeleteFastNode(key []byte) error {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	if err := ndb.batch.Delete(ndb.fastNodeKey(key)); err != nil {
		return err
	}
	ndb.fastNodeCache.Remove(key)
	return nil
}

func (ndb *nodeDB) nodeKey(nk []byte) []byte {
	return nodeKeyFormat.Key(nk)
}

func (ndb *nodeDB) nodeKeyPrefix(version int64) []byte {
	return nodeKeyPrefixFormat.KeyInt64(version)
}

func (ndb *nodeDB) fastNodeKey(key []byte) []byte {
	return fastKeyFormat.KeyBytes(key)
}

func (ndb *nodeDB) legacyNodeKey(nk []byte) []byte {
	return legacyNodeKeyFormat.Key(nk)
}

func (ndb *nodeDB) legacyRootKey(version int64) []byte {
	return legacyRootKeyFormat.Key(version)
}

func (ndb *nodeDB) getFirstVersion() (int64, error) {
	if ndb.firstVersion != 0 {
		return ndb.firstVersion, nil
	}

	// Check if we have a legacy version
	itr, err := dbm.IteratePrefix(ndb.db, legacyRootKeyFormat.Key())
	if err != nil {
		return 0, err
	}
	defer itr.Close()
	if itr.Valid() {
		var version int64
		legacyRootKeyFormat.Scan(itr.Key(), &version)
		return version, nil
	}
	// Find the first version
	latestVersion, err := ndb.getLatestVersion()
	if err != nil {
		return 0, err
	}
	firstVersion := int64(0)
	for firstVersion < latestVersion {
		version := (latestVersion + firstVersion) >> 1
		has, err := ndb.hasVersion(version)
		if err != nil {
			return 0, err
		}
		if has {
			latestVersion = version
		} else {
			firstVersion = version + 1
		}
	}
	ndb.firstVersion = latestVersion

	return ndb.firstVersion, nil
}

func (ndb *nodeDB) resetFirstVersion(version int64) {
	ndb.firstVersion = version
}

func (ndb *nodeDB) getLegacyLatestVersion() (int64, error) {
	if ndb.legacyLatestVersion != 0 {
		return ndb.legacyLatestVersion, nil
	}

	itr, err := ndb.db.ReverseIterator(
		legacyRootKeyFormat.Key(int64(1)),
		legacyRootKeyFormat.Key(int64(math.MaxInt64)),
	)
	if err != nil {
		return 0, err
	}
	defer itr.Close()

	if itr.Valid() {
		k := itr.Key()
		var version int64
		legacyRootKeyFormat.Scan(k, &version)
		ndb.legacyLatestVersion = version
		return version, nil
	}

	if err := itr.Error(); err != nil {
		return 0, err
	}

	// If there are no legacy versions, set -1
	ndb.legacyLatestVersion = -1

	return ndb.legacyLatestVersion, nil
}

func (ndb *nodeDB) getLatestVersion() (int64, error) {
	if ndb.latestVersion != 0 {
		return ndb.latestVersion, nil
	}

	itr, err := ndb.db.ReverseIterator(
		nodeKeyPrefixFormat.KeyInt64(int64(1)),
		nodeKeyPrefixFormat.KeyInt64(int64(math.MaxInt64)),
	)
	if err != nil {
		return 0, err
	}
	defer itr.Close()

	if itr.Valid() {
		k := itr.Key()
		var nk []byte
		nodeKeyFormat.Scan(k, &nk)
		ndb.latestVersion = GetNodeKey(nk).version
		return ndb.latestVersion, nil
	}

	if err := itr.Error(); err != nil {
		return 0, err
	}

	// If there are no versions, try to get the latest version from the legacy format.
	version, err := ndb.getLegacyLatestVersion()
	if err != nil {
		return 0, err
	}
	if version > 0 {
		ndb.latestVersion = version
	}

	return ndb.latestVersion, nil
}

func (ndb *nodeDB) resetLatestVersion(version int64) {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	ndb.latestVersion = version
}

// hasVersion checks if the given version exists.
func (ndb *nodeDB) hasVersion(version int64) (bool, error) {
	return ndb.db.Has(nodeKeyFormat.Key(GetRootKey(version)))
}

// hasLegacyVersion checks if the given version exists in the legacy format.
func (ndb *nodeDB) hasLegacyVersion(version int64) (bool, error) {
	return ndb.db.Has(ndb.legacyRootKey(version))
}

// GetRoot gets the nodeKey of the root for the specific version.
func (ndb *nodeDB) GetRoot(version int64) ([]byte, error) {
	rootKey := GetRootKey(version)
	val, err := ndb.db.Get(nodeKeyFormat.Key(rootKey))
	if err != nil {
		return nil, err
	}
	if val == nil {
		// try the legacy root key
		val, err := ndb.db.Get(ndb.legacyRootKey(version))
		if err != nil {
			return nil, err
		}
		if val == nil {
			return nil, ErrVersionDoesNotExist
		}
		if len(val) == 0 { // empty root
			return nil, nil
		}
		return val, nil
	}
	if len(val) == 0 { // empty root
		return nil, nil
	}
	if isReferenceToRoot(val) { // point to the prev root
		return append(val[1:], 0, 0, 0, 1), nil
	}

	return rootKey, nil
}

// SaveEmptyRoot saves the empty root.
func (ndb *nodeDB) SaveEmptyRoot(version int64) error {
	return ndb.batch.Set(nodeKeyFormat.Key(GetRootKey(version)), []byte{})
}

// SaveRoot saves the root when no updates.
func (ndb *nodeDB) SaveRoot(version, prevVersion int64) error {
	return ndb.batch.Set(nodeKeyFormat.Key(GetRootKey(version)), ndb.nodeKeyPrefix(prevVersion))
}

// Traverse fast nodes and return error if any, nil otherwise
func (ndb *nodeDB) traverseFastNodes(fn func(k, v []byte) error) error {
	return ndb.traversePrefix(fastKeyFormat.Key(), fn)
}

// Traverse all keys and return error if any, nil otherwise

func (ndb *nodeDB) traverse(fn func(key, value []byte) error) error {
	return ndb.traverseRange(nil, nil, fn)
}

// Traverse all keys between a given range (excluding end) and return error if any, nil otherwise
func (ndb *nodeDB) traverseRange(start []byte, end []byte, fn func(k, v []byte) error) error {
	itr, err := ndb.db.Iterator(start, end)
	if err != nil {
		return err
	}
	defer itr.Close()

	for ; itr.Valid(); itr.Next() {
		if err := fn(itr.Key(), itr.Value()); err != nil {
			return err
		}
	}

	return itr.Error()
}

// Traverse all keys with a certain prefix. Return error if any, nil otherwise
func (ndb *nodeDB) traversePrefix(prefix []byte, fn func(k, v []byte) error) error {
	itr, err := dbm.IteratePrefix(ndb.db, prefix)
	if err != nil {
		return err
	}
	defer itr.Close()

	for ; itr.Valid(); itr.Next() {
		if err := fn(itr.Key(), itr.Value()); err != nil {
			return err
		}
	}

	return nil
}

// Get iterator for fast prefix and error, if any
func (ndb *nodeDB) getFastIterator(start, end []byte, ascending bool) (dbm.Iterator, error) {
	var startFormatted, endFormatted []byte

	if start != nil {
		startFormatted = fastKeyFormat.KeyBytes(start)
	} else {
		startFormatted = fastKeyFormat.Key()
	}

	if end != nil {
		endFormatted = fastKeyFormat.KeyBytes(end)
	} else {
		endFormatted = fastKeyFormat.Key()
		endFormatted[0]++
	}

	if ascending {
		return ndb.db.Iterator(startFormatted, endFormatted)
	}

	return ndb.db.ReverseIterator(startFormatted, endFormatted)
}

// Write to disk.
func (ndb *nodeDB) Commit() error {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()

	var err error
	if ndb.opts.Sync {
		err = ndb.batch.WriteSync()
	} else {
		err = ndb.batch.Write()
	}
	if err != nil {
		return fmt.Errorf("failed to write batch, %w", err)
	}

	ndb.batch.Close()
	ndb.batch = NewBatchWithFlusher(ndb.db, ndb.opts.FlushThreshold)

	return nil
}

func (ndb *nodeDB) incrVersionReaders(version int64) {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	ndb.versionReaders[version]++
}

func (ndb *nodeDB) decrVersionReaders(version int64) {
	ndb.mtx.Lock()
	defer ndb.mtx.Unlock()
	if ndb.versionReaders[version] > 0 {
		ndb.versionReaders[version]--
	}
}

// traverseOrphans traverses orphans which removed by the updates of the curVersion in the prevVersion.
// NOTE: it is used for both legacy and new nodes.
func (ndb *nodeDB) traverseOrphans(prevVersion, curVersion int64, fn func(*Node) error) error {
	curKey, err := ndb.GetRoot(curVersion)
	if err != nil {
		return err
	}

	curIter, err := NewNodeIterator(curKey, ndb)
	if err != nil {
		return err
	}

	prevKey, err := ndb.GetRoot(prevVersion)
	if err != nil {
		return err
	}
	prevIter, err := NewNodeIterator(prevKey, ndb)
	if err != nil {
		return err
	}

	var orgNode *Node
	for prevIter.Valid() {
		for orgNode == nil && curIter.Valid() {
			node := curIter.GetNode()
			if node.nodeKey.version <= prevVersion {
				curIter.Next(true)
				orgNode = node
			} else {
				curIter.Next(false)
			}
		}
		pNode := prevIter.GetNode()

		if orgNode != nil && bytes.Equal(pNode.hash, orgNode.hash) {
			prevIter.Next(true)
			orgNode = nil
		} else {
			err = fn(pNode)
			if err != nil {
				return err
			}
			prevIter.Next(false)
		}
	}

	return nil
}

// Utility and test functions

func (ndb *nodeDB) leafNodes() ([]*Node, error) {
	leaves := []*Node{}

	err := ndb.traverseNodes(func(node *Node) error {
		if node.isLeaf() {
			leaves = append(leaves, node)
		}
		return nil
	})
	if err != nil {
		return nil, err
	}

	return leaves, nil
}

func (ndb *nodeDB) nodes() ([]*Node, error) {
	nodes := []*Node{}

	err := ndb.traverseNodes(func(node *Node) error {
		nodes = append(nodes, node)
		return nil
	})
	if err != nil {
		return nil, err
	}

	return nodes, nil
}

func (ndb *nodeDB) orphans() ([][]byte, error) {
	orphans := [][]byte{}

	for version := ndb.firstVersion; version < ndb.latestVersion; version++ {
		err := ndb.traverseOrphans(version, version+1, func(orphan *Node) error {
			orphans = append(orphans, orphan.hash)
			return nil
		})
		if err != nil {
			return nil, err
		}
	}

	return orphans, nil
}

// Not efficient.
// NOTE: DB cannot implement Size() because
// mutations are not always synchronous.
//

func (ndb *nodeDB) size() int {
	size := 0
	err := ndb.traverse(func(k, v []byte) error {
		size++
		return nil
	})
	if err != nil {
		return -1
	}
	return size
}

func isReferenceToRoot(bz []byte) bool {
	if bz[0] == nodeKeyPrefixFormat.Prefix()[0] {
		if len(bz) == nodeKeyPrefixFormat.Length() {
			return true
		}
	}
	return false
}

func (ndb *nodeDB) traverseNodes(fn func(node *Node) error) error {
	nodes := []*Node{}

	if err := ndb.traversePrefix(nodeKeyFormat.Prefix(), func(key, value []byte) error {
		if isReferenceToRoot(value) {
			return nil
		}
		node, err := MakeNode(key[1:], value)
		if err != nil {
			return err
		}
		nodes = append(nodes, node)
		return nil
	}); err != nil {
		return err
	}

	sort.Slice(nodes, func(i, j int) bool {
		return bytes.Compare(nodes[i].key, nodes[j].key) < 0
	})

	for _, n := range nodes {
		if err := fn(n); err != nil {
			return err
		}
	}
	return nil
}

// traverseStateChanges iterate the range of versions, compare each version to it's predecessor to extract the state changes of it.
// endVersion is exclusive, set to `math.MaxInt64` to cover the latest version.
func (ndb *nodeDB) traverseStateChanges(startVersion, endVersion int64, fn func(version int64, changeSet *ChangeSet) error) error {
	firstVersion, err := ndb.getFirstVersion()
	if err != nil {
		return err
	}
	if startVersion < firstVersion {
		startVersion = firstVersion
	}
	latestVersion, err := ndb.getLatestVersion()
	if err != nil {
		return err
	}
	if endVersion > latestVersion {
		endVersion = latestVersion
	}

	prevVersion := startVersion - 1
	prevRoot, err := ndb.GetRoot(prevVersion)
	if err != nil && err != ErrVersionDoesNotExist {
		return err
	}

	for version := startVersion; version <= endVersion; version++ {
		root, err := ndb.GetRoot(version)
		if err != nil {
			return err
		}

		var changeSet ChangeSet
		receiveKVPair := func(pair *KVPair) error {
			changeSet.Pairs = append(changeSet.Pairs, pair)
			return nil
		}

		if err := ndb.extractStateChanges(prevVersion, prevRoot, root, receiveKVPair); err != nil {
			return err
		}

		if err := fn(version, &changeSet); err != nil {
			return err
		}
		prevVersion = version
		prevRoot = root
	}

	return nil
}

func (ndb *nodeDB) String() (string, error) {
	buf := bufPool.Get().(*bytes.Buffer)
	defer bufPool.Put(buf)
	buf.Reset()

	index := 0

	err := ndb.traversePrefix(nodeKeyFormat.Prefix(), func(key, value []byte) error {
		fmt.Fprintf(buf, "%s: %x\n", key, value)
		return nil
	})
	if err != nil {
		return "", err
	}

	buf.WriteByte('\n')

	err = ndb.traverseNodes(func(node *Node) error {
		switch {
		case node == nil:
			fmt.Fprintf(buf, "%s: <nil>\n", nodeKeyFormat.Prefix())
		case node.value == nil && node.subtreeHeight > 0:
			fmt.Fprintf(buf, "%s: %s   %-16s h=%d nodeKey=%v\n",
				nodeKeyFormat.Prefix(), node.key, "", node.subtreeHeight, node.nodeKey)
		default:
			fmt.Fprintf(buf, "%s: %s = %-16s h=%d nodeKey=%v\n",
				nodeKeyFormat.Prefix(), node.key, node.value, node.subtreeHeight, node.nodeKey)
		}
		index++
		return nil
	})

	if err != nil {
		return "", err
	}

	return "-" + "\n" + buf.String() + "-", nil
}

var ErrNodeMissingNodeKey = fmt.Errorf("node does not have a nodeKey")
