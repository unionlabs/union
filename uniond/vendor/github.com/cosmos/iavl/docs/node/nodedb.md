# NodeDB

## Structure

The nodeDB is responsible for persisting nodes and fastNodes correctly in persistent storage.

### Saving Versions

It marshals and saves any new node that has been created under: `n|node.nodeKey.version|node.nodeKey.nonce`. For more details on how the node gets marshaled, see [node documentation](./node.md). The root of each version is saved under `n|version|1`.

(For more details on key formats see the [keyformat docs](./key_format.md))

### Deleting Versions

When a version `v` is deleted, all nodes which removed in the current version will be safely deleted and uncached from the storage. `nodeDB` will keep the range of versions [`fromVersion`, `toVersion`]. There are two apis to delete versions:

#### DeleteVersionsFrom

```golang
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
			return fmt.Errorf("unable to delete version %v with %v active readers", v, r)
		}
	}
	ndb.mtx.Unlock()

	// Delete the nodes
	err = ndb.traverseRange(nodeKeyFormat.Key(fromVersion), nodeKeyFormat.Key(latest+1), func(k, v []byte) error {
		if err = ndb.batch.Delete(k); err != nil {
			return err
		}
		return nil
	})

	if err != nil {
		return err
	}

	// NOTICE: we don't touch fast node indexes here, because it'll be rebuilt later because of version mismatch.

	ndb.resetLatestVersion(fromVersion - 1)

	return nil
}
```

#### DeleteVersionsTo

```golang
// DeleteVersionsTo deletes the oldest versions up to the given version from disk.
func (ndb *nodeDB) DeleteVersionsTo(toVersion int64) error {
	first, err := ndb.getFirstVersion()
	if err != nil {
		return err
	}

	latest, err := ndb.getLatestVersion()
	if err != nil {
		return err
	}

	if toVersion < first || latest <= toVersion {
		return fmt.Errorf("the version should be in the range of [%d, %d)", first, latest)
	}

	for v, r := range ndb.versionReaders {
		if v >= first && v <= toVersion && r != 0 {
			return fmt.Errorf("unable to delete version %v with %v active readers", v, r)
		}
	}

	for version := first; version <= toVersion; version++ {
		if err := ndb.deleteVersion(version); err != nil {
			return err
		}
		ndb.resetFirstVersion(version + 1)
	}

	return nil
}

// deleteVersion deletes a tree version from disk.
// deletes orphans
func (ndb *nodeDB) deleteVersion(version int64) error {
	rootKey, err := ndb.GetRoot(version)
	if err != nil {
		return err
	}
	if rootKey == nil || rootKey.version < version {
		if err := ndb.batch.Delete(ndb.nodeKey(&NodeKey{version: version, nonce: 1})); err != nil {
			return err
		}
	}

	return ndb.traverseOrphans(version, func(orphan *Node) error {
		return ndb.batch.Delete(ndb.nodeKey(orphan.nodeKey))
	})
}
```

##### Traversing Orphans

The traverseOrphans algorithm is shown below:

```golang
// traverseOrphans traverses orphans which removed by the updates of the version (n+1).
func (ndb *nodeDB) traverseOrphans(version int64, fn func(*Node) error) error {
	curKey, err := ndb.GetRoot(version + 1)
	if err != nil {
		return err
	}

	curIter, err := NewNodeIterator(curKey, ndb)
	if err != nil {
		return err
	}

	prevKey, err := ndb.GetRoot(version)
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
			if node.nodeKey.version <= version {
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
```
