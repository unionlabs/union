// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"context"
	"fmt"
	"runtime"
	"sync"
	"sync/atomic"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/sharedcache"
	"github.com/cockroachdb/pebble/objstorage/remote"
)

// remoteSubsystem contains the provider fields related to remote storage.
// All fields remain unset if remote storage is not configured.
type remoteSubsystem struct {
	catalog *remoteobjcat.Catalog
	// catalogSyncMutex is used to correctly serialize two sharedSync operations.
	// It must be acquired before the provider mutex.
	catalogSyncMutex sync.Mutex

	cache *sharedcache.Cache

	// shared contains the fields relevant to shared objects, i.e. objects that
	// are created by Pebble and potentially shared between Pebble instances.
	shared struct {
		// initialized guards access to the creatorID field.
		initialized atomic.Bool
		creatorID   objstorage.CreatorID
		initOnce    sync.Once

		// checkRefsOnOpen controls whether we check the ref marker file when opening
		// an object. Normally this is true when invariants are enabled (but the provider
		// test tweaks this field).
		checkRefsOnOpen bool
	}
}

// remoteInit initializes the remote object subsystem (if configured) and finds
// any remote objects.
func (p *provider) remoteInit() error {
	if p.st.Remote.StorageFactory == nil {
		return nil
	}
	catalog, contents, err := remoteobjcat.Open(p.st.FS, p.st.FSDirName)
	if err != nil {
		return errors.Wrapf(err, "pebble: could not open remote object catalog")
	}
	p.remote.catalog = catalog
	p.remote.shared.checkRefsOnOpen = invariants.Enabled

	// The creator ID may or may not be initialized yet.
	if contents.CreatorID.IsSet() {
		p.remote.initShared(contents.CreatorID)
		p.st.Logger.Infof("remote storage configured; creatorID = %s", contents.CreatorID)
	} else {
		p.st.Logger.Infof("remote storage configured; no creatorID yet")
	}

	if p.st.Remote.CacheSizeBytes > 0 {
		const defaultBlockSize = 32 * 1024
		blockSize := p.st.Remote.CacheBlockSize
		if blockSize == 0 {
			blockSize = defaultBlockSize
		}

		const defaultShardingBlockSize = 1024 * 1024
		shardingBlockSize := p.st.Remote.ShardingBlockSize
		if shardingBlockSize == 0 {
			shardingBlockSize = defaultShardingBlockSize
		}

		numShards := p.st.Remote.CacheShardCount
		if numShards == 0 {
			numShards = 2 * runtime.GOMAXPROCS(0)
		}

		p.remote.cache, err = sharedcache.Open(
			p.st.FS, p.st.Logger, p.st.FSDirName, blockSize, shardingBlockSize, p.st.Remote.CacheSizeBytes, numShards)
		if err != nil {
			return errors.Wrapf(err, "pebble: could not open remote object cache")
		}
	}

	for _, meta := range contents.Objects {
		o := objstorage.ObjectMetadata{
			DiskFileNum: meta.FileNum,
			FileType:    meta.FileType,
		}
		o.Remote.CreatorID = meta.CreatorID
		o.Remote.CreatorFileNum = meta.CreatorFileNum
		o.Remote.CleanupMethod = meta.CleanupMethod
		o.Remote.Locator = meta.Locator
		o.Remote.CustomObjectName = meta.CustomObjectName
		o.Remote.Storage, err = p.ensureStorageLocked(o.Remote.Locator)
		if err != nil {
			return errors.Wrapf(err, "creating remote.Storage object for locator '%s'", o.Remote.Locator)
		}
		if invariants.Enabled {
			o.AssertValid()
		}
		p.mu.knownObjects[o.DiskFileNum] = o
	}
	return nil
}

// initShared initializes the creator ID, allowing use of shared objects.
func (ss *remoteSubsystem) initShared(creatorID objstorage.CreatorID) {
	ss.shared.initOnce.Do(func() {
		ss.shared.creatorID = creatorID
		ss.shared.initialized.Store(true)
	})
}

func (p *provider) sharedClose() error {
	if p.st.Remote.StorageFactory == nil {
		return nil
	}
	var err error
	if p.remote.cache != nil {
		err = p.remote.cache.Close()
		p.remote.cache = nil
	}
	if p.remote.catalog != nil {
		err = firstError(err, p.remote.catalog.Close())
		p.remote.catalog = nil
	}
	return err
}

// SetCreatorID is part of the objstorage.Provider interface.
func (p *provider) SetCreatorID(creatorID objstorage.CreatorID) error {
	if p.st.Remote.StorageFactory == nil {
		return errors.AssertionFailedf("attempt to set CreatorID but remote storage not enabled")
	}
	// Note: this call is a cheap no-op if the creator ID was already set. This
	// call also checks if we are trying to change the ID.
	if err := p.remote.catalog.SetCreatorID(creatorID); err != nil {
		return err
	}
	if !p.remote.shared.initialized.Load() {
		p.st.Logger.Infof("remote storage creatorID set to %s", creatorID)
		p.remote.initShared(creatorID)
	}
	return nil
}

// IsSharedForeign is part of the objstorage.Provider interface.
func (p *provider) IsSharedForeign(meta objstorage.ObjectMetadata) bool {
	if !p.remote.shared.initialized.Load() {
		return false
	}
	return meta.IsShared() && (meta.Remote.CreatorID != p.remote.shared.creatorID)
}

func (p *provider) remoteCheckInitialized() error {
	if p.st.Remote.StorageFactory == nil {
		return errors.Errorf("remote object support not configured")
	}
	return nil
}

func (p *provider) sharedCheckInitialized() error {
	if err := p.remoteCheckInitialized(); err != nil {
		return err
	}
	if !p.remote.shared.initialized.Load() {
		return errors.Errorf("remote object support not available: remote creator ID not yet set")
	}
	return nil
}

func (p *provider) sharedSync() error {
	// Serialize parallel sync operations. Note that ApplyBatch is already
	// serialized internally, but we want to make sure they get called with
	// batches in the right order.
	p.remote.catalogSyncMutex.Lock()
	defer p.remote.catalogSyncMutex.Unlock()

	batch := func() remoteobjcat.Batch {
		p.mu.Lock()
		defer p.mu.Unlock()
		res := p.mu.remote.catalogBatch.Copy()
		p.mu.remote.catalogBatch.Reset()
		return res
	}()

	if batch.IsEmpty() {
		return nil
	}

	if err := p.remote.catalog.ApplyBatch(batch); err != nil {
		// Put back the batch (for the next Sync), appending any operations that
		// happened in the meantime.
		p.mu.Lock()
		defer p.mu.Unlock()
		batch.Append(p.mu.remote.catalogBatch)
		p.mu.remote.catalogBatch = batch
		return err
	}

	return nil
}

func (p *provider) remotePath(meta objstorage.ObjectMetadata) string {
	if meta.Remote.Locator != "" {
		return fmt.Sprintf("remote-%s://%s", meta.Remote.Locator, remoteObjectName(meta))
	}
	return "remote://" + remoteObjectName(meta)
}

// sharedCreateRef creates a reference marker object.
func (p *provider) sharedCreateRef(meta objstorage.ObjectMetadata) error {
	if err := p.sharedCheckInitialized(); err != nil {
		return err
	}
	if meta.Remote.CleanupMethod != objstorage.SharedRefTracking {
		return nil
	}
	refName := p.sharedObjectRefName(meta)
	writer, err := meta.Remote.Storage.CreateObject(refName)
	if err == nil {
		// The object is empty, just close the writer.
		err = writer.Close()
	}
	if err != nil {
		return errors.Wrapf(err, "creating marker object %q", refName)
	}
	return nil
}

func (p *provider) sharedCreate(
	_ context.Context,
	fileType base.FileType,
	fileNum base.DiskFileNum,
	locator remote.Locator,
	opts objstorage.CreateOptions,
) (objstorage.Writable, objstorage.ObjectMetadata, error) {
	if err := p.sharedCheckInitialized(); err != nil {
		return nil, objstorage.ObjectMetadata{}, err
	}
	storage, err := p.ensureStorage(locator)
	if err != nil {
		return nil, objstorage.ObjectMetadata{}, err
	}
	meta := objstorage.ObjectMetadata{
		DiskFileNum: fileNum,
		FileType:    fileType,
	}
	meta.Remote.CreatorID = p.remote.shared.creatorID
	meta.Remote.CreatorFileNum = fileNum
	meta.Remote.CleanupMethod = opts.SharedCleanupMethod
	meta.Remote.Locator = locator
	meta.Remote.Storage = storage

	objName := remoteObjectName(meta)
	writer, err := storage.CreateObject(objName)
	if err != nil {
		return nil, objstorage.ObjectMetadata{}, errors.Wrapf(err, "creating object %q", objName)
	}
	return &sharedWritable{
		p:             p,
		meta:          meta,
		storageWriter: writer,
	}, meta, nil
}

func (p *provider) remoteOpenForReading(
	ctx context.Context, meta objstorage.ObjectMetadata, opts objstorage.OpenOptions,
) (objstorage.Readable, error) {
	if err := p.remoteCheckInitialized(); err != nil {
		return nil, err
	}
	// Verify we have a reference on this object; for performance reasons, we only
	// do this in testing scenarios.
	if p.remote.shared.checkRefsOnOpen && meta.Remote.CleanupMethod == objstorage.SharedRefTracking {
		if err := p.sharedCheckInitialized(); err != nil {
			return nil, err
		}
		refName := p.sharedObjectRefName(meta)
		if _, err := meta.Remote.Storage.Size(refName); err != nil {
			if meta.Remote.Storage.IsNotExistError(err) {
				if opts.MustExist {
					p.st.Logger.Fatalf("marker object %q does not exist", refName)
					// TODO(radu): maybe list references for the object.
				}
				return nil, errors.Errorf("marker object %q does not exist", refName)
			}
			return nil, errors.Wrapf(err, "checking marker object %q", refName)
		}
	}
	objName := remoteObjectName(meta)
	reader, size, err := meta.Remote.Storage.ReadObject(ctx, objName)
	if err != nil {
		if opts.MustExist && meta.Remote.Storage.IsNotExistError(err) {
			p.st.Logger.Fatalf("object %q does not exist", objName)
			// TODO(radu): maybe list references for the object.
		}
		return nil, err
	}
	return p.newRemoteReadable(reader, size, meta.DiskFileNum), nil
}

func (p *provider) remoteSize(meta objstorage.ObjectMetadata) (int64, error) {
	if err := p.remoteCheckInitialized(); err != nil {
		return 0, err
	}
	objName := remoteObjectName(meta)
	return meta.Remote.Storage.Size(objName)
}

// sharedUnref implements object "removal" with the remote backend. The ref
// marker object is removed and the backing object is removed only if there are
// no other ref markers.
func (p *provider) sharedUnref(meta objstorage.ObjectMetadata) error {
	if meta.Remote.CleanupMethod == objstorage.SharedNoCleanup {
		// Never delete objects in this mode.
		return nil
	}
	if p.isProtected(meta.DiskFileNum) {
		// TODO(radu): we need a mechanism to unref the object when it becomes
		// unprotected.
		return nil
	}

	refName := p.sharedObjectRefName(meta)
	// Tolerate a not-exists error.
	if err := meta.Remote.Storage.Delete(refName); err != nil && !meta.Remote.Storage.IsNotExistError(err) {
		return err
	}
	otherRefs, err := meta.Remote.Storage.List(sharedObjectRefPrefix(meta), "" /* delimiter */)
	if err != nil {
		return err
	}
	if len(otherRefs) == 0 {
		objName := remoteObjectName(meta)
		if err := meta.Remote.Storage.Delete(objName); err != nil && !meta.Remote.Storage.IsNotExistError(err) {
			return err
		}
	}
	return nil
}

// ensureStorageLocked populates the remote.Storage object for the given
// locator, if necessary. p.mu must be held.
func (p *provider) ensureStorageLocked(locator remote.Locator) (remote.Storage, error) {
	if p.mu.remote.storageObjects == nil {
		p.mu.remote.storageObjects = make(map[remote.Locator]remote.Storage)
	}
	if res, ok := p.mu.remote.storageObjects[locator]; ok {
		return res, nil
	}
	res, err := p.st.Remote.StorageFactory.CreateStorage(locator)
	if err != nil {
		return nil, err
	}

	p.mu.remote.storageObjects[locator] = res
	return res, nil
}

// ensureStorage populates the remote.Storage object for the given locator, if necessary.
func (p *provider) ensureStorage(locator remote.Locator) (remote.Storage, error) {
	p.mu.Lock()
	defer p.mu.Unlock()
	return p.ensureStorageLocked(locator)
}
