// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"context"
	"io"
	"os"
	"sort"
	"sync"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/objiotracing"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/sharedcache"
	"github.com/cockroachdb/pebble/objstorage/remote"
	"github.com/cockroachdb/pebble/vfs"
)

// provider is the implementation of objstorage.Provider.
type provider struct {
	st Settings

	fsDir vfs.File

	tracer *objiotracing.Tracer

	remote remoteSubsystem

	mu struct {
		sync.RWMutex

		remote struct {
			// catalogBatch accumulates remote object creations and deletions until
			// Sync is called.
			catalogBatch remoteobjcat.Batch

			storageObjects map[remote.Locator]remote.Storage
		}

		// localObjectsChanged is set if non-remote objects were created or deleted
		// but Sync was not yet called.
		localObjectsChanged bool

		// knownObjects maintains information about objects that are known to the provider.
		// It is initialized with the list of files in the manifest when we open a DB.
		knownObjects map[base.DiskFileNum]objstorage.ObjectMetadata

		// protectedObjects are objects that cannot be unreferenced because they
		// have outstanding SharedObjectBackingHandles. The value is a count of outstanding handles
		protectedObjects map[base.DiskFileNum]int
	}
}

var _ objstorage.Provider = (*provider)(nil)

// Settings that must be specified when creating the provider.
type Settings struct {
	Logger base.Logger

	// Local filesystem configuration.
	FS        vfs.FS
	FSDirName string

	// FSDirInitialListing is a listing of FSDirName at the time of calling Open.
	//
	// This is an optional optimization to avoid double listing on Open when the
	// higher layer already has a listing. When nil, we obtain the listing on
	// Open.
	FSDirInitialListing []string

	// Cleaner cleans obsolete files from the local filesystem.
	//
	// The default cleaner uses the DeleteCleaner.
	FSCleaner base.Cleaner

	// NoSyncOnClose decides whether the implementation will enforce a
	// close-time synchronization (e.g., fdatasync() or sync_file_range())
	// on files it writes to. Setting this to true removes the guarantee for a
	// sync on close. Some implementations can still issue a non-blocking sync.
	NoSyncOnClose bool

	// BytesPerSync enables periodic syncing of files in order to smooth out
	// writes to disk. This option does not provide any persistence guarantee, but
	// is used to avoid latency spikes if the OS automatically decides to write
	// out a large chunk of dirty filesystem buffers.
	BytesPerSync int

	// Local contains fields that are only relevant for files stored on the local
	// filesystem.
	Local struct {
		// TODO(radu): move FSCleaner, NoSyncOnClose, BytesPerSync here.

		// ReadaheadConfigFn is a function used to retrieve the current readahead
		// mode. This function is run whenever a local object is open for reading.
		// If it is nil, DefaultReadaheadConfig is used.
		ReadaheadConfigFn func() ReadaheadConfig
	}

	// Fields here are set only if the provider is to support remote objects
	// (experimental).
	Remote struct {
		StorageFactory remote.StorageFactory

		// If CreateOnShared is non-zero, sstables are created on remote storage using
		// the CreateOnSharedLocator (when the PreferSharedStorage create option is
		// true).
		CreateOnShared        remote.CreateOnSharedStrategy
		CreateOnSharedLocator remote.Locator

		// CacheSizeBytes is the size of the on-disk block cache for objects
		// on remote storage. If it is 0, no cache is used.
		CacheSizeBytes int64

		// CacheBlockSize is the block size of the cache; if 0, the default of 32KB is used.
		CacheBlockSize int

		// ShardingBlockSize is the size of a shard block. The cache is split into contiguous
		// ShardingBlockSize units. The units are distributed across multiple independent shards
		// of the cache, via a hash(offset) modulo num shards operation. The cache replacement
		// policies operate at the level of shard, not whole cache. This is done to reduce lock
		// contention.
		//
		// If ShardingBlockSize is 0, the default of 1 MB is used.
		ShardingBlockSize int64

		// The number of independent shards the cache leverages. Each shard is the same size,
		// and a hash of filenum & offset map a read to a certain shard. If set to 0,
		// 2*runtime.GOMAXPROCS is used as the shard count.
		CacheShardCount int

		// TODO(radu): allow the cache to live on another FS/location (e.g. to use
		// instance-local SSD).
	}
}

// ReadaheadConfig controls the use of read-ahead.
type ReadaheadConfig struct {
	// Informed is the type of read-ahead for operations that are known to read a
	// large consecutive chunk of a file.
	Informed ReadaheadMode

	// Speculative is the type of read-ahead used automatically, when consecutive
	// reads are detected.
	Speculative ReadaheadMode
}

// DefaultReadaheadConfig is the readahead config used when ReadaheadConfigFn is
// not specified.
var DefaultReadaheadConfig = ReadaheadConfig{
	Informed:    FadviseSequential,
	Speculative: FadviseSequential,
}

// ReadaheadMode indicates the type of read-ahead to use, either for informed
// read-ahead (e.g. compactions) or speculative read-ahead.
type ReadaheadMode uint8

const (
	// NoReadahead disables readahead altogether.
	NoReadahead ReadaheadMode = iota

	// SysReadahead enables the use of SYS_READAHEAD call to prefetch data.
	// The prefetch window grows dynamically as consecutive writes are detected.
	SysReadahead

	// FadviseSequential enables to use of FADV_SEQUENTIAL. For informed
	// read-ahead, FADV_SEQUENTIAL is used from the beginning. For speculative
	// read-ahead SYS_READAHEAD is first used until the window reaches the maximum
	// size, then we siwtch to FADV_SEQUENTIAL.
	FadviseSequential
)

// DefaultSettings initializes default settings (with no remote storage),
// suitable for tests and tools.
func DefaultSettings(fs vfs.FS, dirName string) Settings {
	return Settings{
		Logger:        base.DefaultLogger,
		FS:            fs,
		FSDirName:     dirName,
		FSCleaner:     base.DeleteCleaner{},
		NoSyncOnClose: false,
		BytesPerSync:  512 * 1024, // 512KB
	}
}

// Open creates the provider.
func Open(settings Settings) (objstorage.Provider, error) {
	// Note: we can't just `return open(settings)` because in an error case we
	// would return (*provider)(nil) which is not objstorage.Provider(nil).
	p, err := open(settings)
	if err != nil {
		return nil, err
	}
	return p, nil
}

func open(settings Settings) (p *provider, _ error) {
	fsDir, err := settings.FS.OpenDir(settings.FSDirName)
	if err != nil {
		return nil, err
	}

	defer func() {
		if p == nil {
			fsDir.Close()
		}
	}()

	p = &provider{
		st:    settings,
		fsDir: fsDir,
	}
	p.mu.knownObjects = make(map[base.DiskFileNum]objstorage.ObjectMetadata)
	p.mu.protectedObjects = make(map[base.DiskFileNum]int)

	if objiotracing.Enabled {
		p.tracer = objiotracing.Open(settings.FS, settings.FSDirName)
	}

	// Add local FS objects.
	if err := p.vfsInit(); err != nil {
		return nil, err
	}

	// Initialize remote subsystem (if configured) and add remote objects.
	if err := p.remoteInit(); err != nil {
		return nil, err
	}

	return p, nil
}

// Close is part of the objstorage.Provider interface.
func (p *provider) Close() error {
	err := p.sharedClose()
	if p.fsDir != nil {
		err = firstError(err, p.fsDir.Close())
		p.fsDir = nil
	}
	if objiotracing.Enabled {
		if p.tracer != nil {
			p.tracer.Close()
			p.tracer = nil
		}
	}
	return err
}

// OpenForReading opens an existing object.
func (p *provider) OpenForReading(
	ctx context.Context,
	fileType base.FileType,
	fileNum base.DiskFileNum,
	opts objstorage.OpenOptions,
) (objstorage.Readable, error) {
	meta, err := p.Lookup(fileType, fileNum)
	if err != nil {
		if opts.MustExist {
			p.st.Logger.Fatalf("%v", err)
		}
		return nil, err
	}

	var r objstorage.Readable
	if !meta.IsRemote() {
		r, err = p.vfsOpenForReading(ctx, fileType, fileNum, opts)
	} else {
		r, err = p.remoteOpenForReading(ctx, meta, opts)
		if err != nil && p.isNotExistError(meta, err) {
			// Wrap the error so that IsNotExistError functions properly.
			err = errors.Mark(err, os.ErrNotExist)
		}
	}
	if err != nil {
		return nil, err
	}
	if objiotracing.Enabled {
		r = p.tracer.WrapReadable(ctx, r, fileNum)
	}
	return r, nil
}

// Create creates a new object and opens it for writing.
//
// The object is not guaranteed to be durable (accessible in case of crashes)
// until Sync is called.
func (p *provider) Create(
	ctx context.Context,
	fileType base.FileType,
	fileNum base.DiskFileNum,
	opts objstorage.CreateOptions,
) (w objstorage.Writable, meta objstorage.ObjectMetadata, err error) {
	if opts.PreferSharedStorage && p.st.Remote.CreateOnShared != remote.CreateOnSharedNone {
		w, meta, err = p.sharedCreate(ctx, fileType, fileNum, p.st.Remote.CreateOnSharedLocator, opts)
	} else {
		w, meta, err = p.vfsCreate(ctx, fileType, fileNum)
	}
	if err != nil {
		err = errors.Wrapf(err, "creating object %s", errors.Safe(fileNum))
		return nil, objstorage.ObjectMetadata{}, err
	}
	p.addMetadata(meta)
	if objiotracing.Enabled {
		w = p.tracer.WrapWritable(ctx, w, fileNum)
	}
	return w, meta, nil
}

// Remove removes an object.
//
// Note that if the object is remote, the object is only (conceptually) removed
// from this provider. If other providers have references on the remote object,
// it will not be removed.
//
// The object is not guaranteed to be durably removed until Sync is called.
func (p *provider) Remove(fileType base.FileType, fileNum base.DiskFileNum) error {
	meta, err := p.Lookup(fileType, fileNum)
	if err != nil {
		return err
	}

	if !meta.IsRemote() {
		err = p.vfsRemove(fileType, fileNum)
	} else {
		// TODO(radu): implement remote object removal (i.e. deref).
		err = p.sharedUnref(meta)
		if err != nil && p.isNotExistError(meta, err) {
			// Wrap the error so that IsNotExistError functions properly.
			err = errors.Mark(err, os.ErrNotExist)
		}
	}
	if err != nil && !p.IsNotExistError(err) {
		// We want to be able to retry a Remove, so we keep the object in our list.
		// TODO(radu): we should mark the object as "zombie" and not allow any other
		// operations.
		return errors.Wrapf(err, "removing object %s", errors.Safe(fileNum))
	}

	p.removeMetadata(fileNum)
	return err
}

func (p *provider) isNotExistError(meta objstorage.ObjectMetadata, err error) bool {
	if meta.Remote.Storage != nil {
		return meta.Remote.Storage.IsNotExistError(err)
	}
	return oserror.IsNotExist(err)
}

// IsNotExistError is part of the objstorage.Provider interface.
func (p *provider) IsNotExistError(err error) bool {
	// We use errors.Mark(err, os.ErrNotExist) for not-exist errors coming from
	// remote.Storage.
	return oserror.IsNotExist(err)
}

// Sync flushes the metadata from creation or removal of objects since the last Sync.
func (p *provider) Sync() error {
	if err := p.vfsSync(); err != nil {
		return err
	}
	if err := p.sharedSync(); err != nil {
		return err
	}
	return nil
}

// LinkOrCopyFromLocal creates a new object that is either a copy of a given
// local file or a hard link (if the new object is created on the same FS, and
// if the FS supports it).
//
// The object is not guaranteed to be durable (accessible in case of crashes)
// until Sync is called.
func (p *provider) LinkOrCopyFromLocal(
	ctx context.Context,
	srcFS vfs.FS,
	srcFilePath string,
	dstFileType base.FileType,
	dstFileNum base.DiskFileNum,
	opts objstorage.CreateOptions,
) (objstorage.ObjectMetadata, error) {
	shared := opts.PreferSharedStorage && p.st.Remote.CreateOnShared != remote.CreateOnSharedNone
	if !shared && srcFS == p.st.FS {
		// Wrap the normal filesystem with one which wraps newly created files with
		// vfs.NewSyncingFile.
		fs := vfs.NewSyncingFS(p.st.FS, vfs.SyncingFileOptions{
			NoSyncOnClose: p.st.NoSyncOnClose,
			BytesPerSync:  p.st.BytesPerSync,
		})
		dstPath := p.vfsPath(dstFileType, dstFileNum)
		if err := vfs.LinkOrCopy(fs, srcFilePath, dstPath); err != nil {
			return objstorage.ObjectMetadata{}, err
		}

		meta := objstorage.ObjectMetadata{
			DiskFileNum: dstFileNum,
			FileType:    dstFileType,
		}
		p.addMetadata(meta)
		return meta, nil
	}
	// Create the object and copy the data.
	w, meta, err := p.Create(ctx, dstFileType, dstFileNum, opts)
	if err != nil {
		return objstorage.ObjectMetadata{}, err
	}
	f, err := srcFS.Open(srcFilePath, vfs.SequentialReadsOption)
	if err != nil {
		return objstorage.ObjectMetadata{}, err
	}
	defer f.Close()
	buf := make([]byte, 64*1024)
	for {
		n, readErr := f.Read(buf)
		if readErr != nil && readErr != io.EOF {
			w.Abort()
			return objstorage.ObjectMetadata{}, readErr
		}

		if n > 0 {
			if err := w.Write(buf[:n]); err != nil {
				w.Abort()
				return objstorage.ObjectMetadata{}, err
			}
		}

		if readErr == io.EOF {
			break
		}
	}
	if err := w.Finish(); err != nil {
		return objstorage.ObjectMetadata{}, err
	}
	return meta, nil
}

// Lookup is part of the objstorage.Provider interface.
func (p *provider) Lookup(
	fileType base.FileType, fileNum base.DiskFileNum,
) (objstorage.ObjectMetadata, error) {
	p.mu.RLock()
	defer p.mu.RUnlock()
	meta, ok := p.mu.knownObjects[fileNum]
	if !ok {
		return objstorage.ObjectMetadata{}, errors.Wrapf(
			os.ErrNotExist,
			"file %s (type %d) unknown to the objstorage provider",
			errors.Safe(fileNum), errors.Safe(fileType),
		)
	}
	if meta.FileType != fileType {
		return objstorage.ObjectMetadata{}, errors.AssertionFailedf(
			"file %s type mismatch (known type %d, expected type %d)",
			errors.Safe(fileNum), errors.Safe(meta.FileType), errors.Safe(fileType),
		)
	}
	return meta, nil
}

// Path is part of the objstorage.Provider interface.
func (p *provider) Path(meta objstorage.ObjectMetadata) string {
	if !meta.IsRemote() {
		return p.vfsPath(meta.FileType, meta.DiskFileNum)
	}
	return p.remotePath(meta)
}

// Size returns the size of the object.
func (p *provider) Size(meta objstorage.ObjectMetadata) (int64, error) {
	if !meta.IsRemote() {
		return p.vfsSize(meta.FileType, meta.DiskFileNum)
	}
	return p.remoteSize(meta)
}

// List is part of the objstorage.Provider interface.
func (p *provider) List() []objstorage.ObjectMetadata {
	p.mu.RLock()
	defer p.mu.RUnlock()
	res := make([]objstorage.ObjectMetadata, 0, len(p.mu.knownObjects))
	for _, meta := range p.mu.knownObjects {
		res = append(res, meta)
	}
	sort.Slice(res, func(i, j int) bool {
		return res[i].DiskFileNum.FileNum() < res[j].DiskFileNum.FileNum()
	})
	return res
}

// Metrics is part of the objstorage.Provider interface.
func (p *provider) Metrics() sharedcache.Metrics {
	if p.remote.cache != nil {
		return p.remote.cache.Metrics()
	}
	return sharedcache.Metrics{}
}

func (p *provider) addMetadata(meta objstorage.ObjectMetadata) {
	if invariants.Enabled {
		meta.AssertValid()
	}
	p.mu.Lock()
	defer p.mu.Unlock()
	p.mu.knownObjects[meta.DiskFileNum] = meta
	if meta.IsRemote() {
		p.mu.remote.catalogBatch.AddObject(remoteobjcat.RemoteObjectMetadata{
			FileNum:        meta.DiskFileNum,
			FileType:       meta.FileType,
			CreatorID:      meta.Remote.CreatorID,
			CreatorFileNum: meta.Remote.CreatorFileNum,
			Locator:        meta.Remote.Locator,
			CleanupMethod:  meta.Remote.CleanupMethod,
		})
	} else {
		p.mu.localObjectsChanged = true
	}
}

func (p *provider) removeMetadata(fileNum base.DiskFileNum) {
	p.mu.Lock()
	defer p.mu.Unlock()

	meta, ok := p.mu.knownObjects[fileNum]
	if !ok {
		return
	}
	delete(p.mu.knownObjects, fileNum)
	if meta.IsRemote() {
		p.mu.remote.catalogBatch.DeleteObject(fileNum)
	} else {
		p.mu.localObjectsChanged = true
	}
}

// protectObject prevents the unreferencing of a remote object until
// unprotectObject is called.
func (p *provider) protectObject(fileNum base.DiskFileNum) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.mu.protectedObjects[fileNum] = p.mu.protectedObjects[fileNum] + 1
}

func (p *provider) unprotectObject(fileNum base.DiskFileNum) {
	p.mu.Lock()
	defer p.mu.Unlock()
	v := p.mu.protectedObjects[fileNum]
	if invariants.Enabled && v == 0 {
		panic("invalid protection count")
	}
	if v > 1 {
		p.mu.protectedObjects[fileNum] = v - 1
	} else {
		delete(p.mu.protectedObjects, fileNum)
		// TODO(radu): check if the object is still in knownObject; if not, unref it
		// now.
	}
}

func (p *provider) isProtected(fileNum base.DiskFileNum) bool {
	p.mu.Lock()
	defer p.mu.Unlock()
	return p.mu.protectedObjects[fileNum] > 0
}
