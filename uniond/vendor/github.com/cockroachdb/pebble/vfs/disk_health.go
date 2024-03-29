// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs

import (
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"
	"sync/atomic"
	"time"

	"github.com/cockroachdb/redact"
)

const (
	// preallocatedSlotCount is the default number of slots available for
	// concurrent filesystem operations. The slot count may be exceeded, but
	// each additional slot will incur an additional allocation. We choose 16
	// here with the expectation that it is significantly more than required in
	// practice. See the comment above the diskHealthCheckingFS type definition.
	preallocatedSlotCount = 16
	// deltaBits is the number of bits in the packed 64-bit integer used for
	// identifying a delta from the file creation time in milliseconds.
	deltaBits = 40
	// writeSizeBits is the number of bits in the packed 64-bit integer used for
	// identifying the size of the write operation, if the operation is sized. See
	// writeSizePrecision below for precision of size.
	writeSizeBits = 20
	// Track size of writes at kilobyte precision. See comment above lastWritePacked for more.
	writeSizePrecision = 1024
)

// Variables to enable testing.
var (
	// defaultTickInterval is the default interval between two ticks of each
	// diskHealthCheckingFile loop iteration.
	defaultTickInterval = 2 * time.Second
)

// OpType is the type of IO operation being monitored by a
// diskHealthCheckingFile.
type OpType uint8

// The following OpTypes is limited to the subset of file system operations that
// a diskHealthCheckingFile supports (namely writes and syncs).
const (
	OpTypeUnknown OpType = iota
	OpTypeWrite
	OpTypeSync
	OpTypeSyncData
	OpTypeSyncTo
	OpTypeCreate
	OpTypeLink
	OpTypeMkdirAll
	OpTypePreallocate
	OpTypeRemove
	OpTypeRemoveAll
	OpTypeRename
	OpTypeReuseForWrite
	// Note: opTypeMax is just used in tests. It must appear last in the list
	// of OpTypes.
	opTypeMax
)

// String implements fmt.Stringer.
func (o OpType) String() string {
	switch o {
	case OpTypeWrite:
		return "write"
	case OpTypeSync:
		return "sync"
	case OpTypeSyncData:
		return "syncdata"
	case OpTypeSyncTo:
		return "syncto"
	case OpTypeCreate:
		return "create"
	case OpTypeLink:
		return "link"
	case OpTypeMkdirAll:
		return "mkdirall"
	case OpTypePreallocate:
		return "preallocate"
	case OpTypeRemove:
		return "remove"
	case OpTypeRemoveAll:
		return "removall"
	case OpTypeRename:
		return "rename"
	case OpTypeReuseForWrite:
		return "reuseforwrite"
	case OpTypeUnknown:
		return "unknown"
	default:
		panic(fmt.Sprintf("vfs: unknown op type: %d", o))
	}
}

// diskHealthCheckingFile is a File wrapper to detect slow disk operations, and
// call onSlowDisk if a disk operation is seen to exceed diskSlowThreshold.
//
// This struct creates a goroutine (in startTicker()) that, at every tick
// interval, sees if there's a disk operation taking longer than the specified
// duration. This setup is preferable to creating a new timer at every disk
// operation, as it reduces overhead per disk operation.
type diskHealthCheckingFile struct {
	file              File
	onSlowDisk        func(opType OpType, writeSizeInBytes int, duration time.Duration)
	diskSlowThreshold time.Duration
	tickInterval      time.Duration

	stopper chan struct{}
	// lastWritePacked is a 64-bit unsigned int. The most significant
	// 40 bits represent an delta (in milliseconds) from the creation
	// time of the diskHealthCheckingFile. The next most significant 20 bits
	// represent the size of the write in KBs, if the write has a size. (If
	// it doesn't, the 20 bits are zeroed). The least significant four bits
	// contains the OpType.
	//
	// The use of 40 bits for an delta provides ~34 years of effective
	// monitoring time before the uint wraps around, at millisecond precision.
	// ~34 years of process uptime "ought to be enough for anybody". Millisecond
	// writeSizePrecision is sufficient, given that we are monitoring for writes that take
	// longer than one millisecond.
	//
	// The use of 20 bits for the size in KBs allows representing sizes up
	// to nearly one GB. If the write is larger than that, we round down to ~one GB.
	//
	// The use of four bits for OpType allows for 16 operation types.
	//
	// NB: this packing scheme is not persisted, and is therefore safe to adjust
	// across process boundaries.
	lastWritePacked atomic.Uint64
	createTime      time.Time
}

// newDiskHealthCheckingFile instantiates a new diskHealthCheckingFile, with the
// specified time threshold and event listener.
func newDiskHealthCheckingFile(
	file File,
	diskSlowThreshold time.Duration,
	onSlowDisk func(OpType OpType, writeSizeInBytes int, duration time.Duration),
) *diskHealthCheckingFile {
	return &diskHealthCheckingFile{
		file:              file,
		onSlowDisk:        onSlowDisk,
		diskSlowThreshold: diskSlowThreshold,
		tickInterval:      defaultTickInterval,

		stopper:    make(chan struct{}),
		createTime: time.Now(),
	}
}

// startTicker starts a new goroutine with a ticker to monitor disk operations.
// Can only be called if the ticker goroutine isn't running already.
func (d *diskHealthCheckingFile) startTicker() {
	if d.diskSlowThreshold == 0 {
		return
	}

	go func() {
		ticker := time.NewTicker(d.tickInterval)
		defer ticker.Stop()

		for {
			select {
			case <-d.stopper:
				return

			case <-ticker.C:
				packed := d.lastWritePacked.Load()
				if packed == 0 {
					continue
				}
				delta, writeSize, op := unpack(packed)
				lastWrite := d.createTime.Add(delta)
				now := time.Now()
				if lastWrite.Add(d.diskSlowThreshold).Before(now) {
					// diskSlowThreshold was exceeded. Call the passed-in
					// listener.
					d.onSlowDisk(op, writeSize, now.Sub(lastWrite))
				}
			}
		}
	}()
}

// stopTicker stops the goroutine started in startTicker.
func (d *diskHealthCheckingFile) stopTicker() {
	close(d.stopper)
}

// Fd implements (vfs.File).Fd.
func (d *diskHealthCheckingFile) Fd() uintptr {
	return d.file.Fd()
}

// Read implements (vfs.File).Read
func (d *diskHealthCheckingFile) Read(p []byte) (int, error) {
	return d.file.Read(p)
}

// ReadAt implements (vfs.File).ReadAt
func (d *diskHealthCheckingFile) ReadAt(p []byte, off int64) (int, error) {
	return d.file.ReadAt(p, off)
}

// Write implements the io.Writer interface.
func (d *diskHealthCheckingFile) Write(p []byte) (n int, err error) {
	d.timeDiskOp(OpTypeWrite, int64(len(p)), func() {
		n, err = d.file.Write(p)
	})
	return n, err
}

// Write implements the io.WriterAt interface.
func (d *diskHealthCheckingFile) WriteAt(p []byte, ofs int64) (n int, err error) {
	d.timeDiskOp(OpTypeWrite, int64(len(p)), func() {
		n, err = d.file.WriteAt(p, ofs)
	})
	return n, err
}

// Close implements the io.Closer interface.
func (d *diskHealthCheckingFile) Close() error {
	d.stopTicker()
	return d.file.Close()
}

// Prefetch implements (vfs.File).Prefetch.
func (d *diskHealthCheckingFile) Prefetch(offset, length int64) error {
	return d.file.Prefetch(offset, length)
}

// Preallocate implements (vfs.File).Preallocate.
func (d *diskHealthCheckingFile) Preallocate(off, n int64) (err error) {
	d.timeDiskOp(OpTypePreallocate, n, func() {
		err = d.file.Preallocate(off, n)
	})
	return err
}

// Stat implements (vfs.File).Stat.
func (d *diskHealthCheckingFile) Stat() (os.FileInfo, error) {
	return d.file.Stat()
}

// Sync implements the io.Syncer interface.
func (d *diskHealthCheckingFile) Sync() (err error) {
	d.timeDiskOp(OpTypeSync, 0, func() {
		err = d.file.Sync()
	})
	return err
}

// SyncData implements (vfs.File).SyncData.
func (d *diskHealthCheckingFile) SyncData() (err error) {
	d.timeDiskOp(OpTypeSyncData, 0, func() {
		err = d.file.SyncData()
	})
	return err
}

// SyncTo implements (vfs.File).SyncTo.
func (d *diskHealthCheckingFile) SyncTo(length int64) (fullSync bool, err error) {
	d.timeDiskOp(OpTypeSyncTo, length, func() {
		fullSync, err = d.file.SyncTo(length)
	})
	return fullSync, err
}

// timeDiskOp runs the specified closure and makes its timing visible to the
// monitoring goroutine, in case it exceeds one of the slow disk durations.
// opType should always be set. writeSizeInBytes should be set if the write
// operation is sized. If not, it should be set to zero.
func (d *diskHealthCheckingFile) timeDiskOp(opType OpType, writeSizeInBytes int64, op func()) {
	if d == nil {
		op()
		return
	}

	delta := time.Since(d.createTime)
	packed := pack(delta, writeSizeInBytes, opType)
	if d.lastWritePacked.Swap(packed) != 0 {
		panic("concurrent write operations detected on file")
	}
	defer func() {
		if d.lastWritePacked.Swap(0) != packed {
			panic("concurrent write operations detected on file")
		}
	}()
	op()
}

// Note the slight lack of symmetry between pack & unpack. pack takes an int64 for writeSizeInBytes, since
// callers of pack use an int64. This is dictated by the vfs interface. unpack OTOH returns an int. This is
// safe because the packing scheme implies we only actually need 32 bits.
func pack(delta time.Duration, writeSizeInBytes int64, opType OpType) uint64 {
	// We have no guarantee of clock monotonicity. If we have a small regression
	// in the clock, we set deltaMillis to zero, so we can still catch the operation
	// if happens to be slow.
	deltaMillis := delta.Milliseconds()
	if deltaMillis < 0 {
		deltaMillis = 0
	}
	// As of 3/7/2023, the use of 40 bits for an delta provides ~34 years
	// of effective monitoring time before the uint wraps around, at millisecond
	// precision.
	if deltaMillis > 1<<deltaBits-1 {
		panic("vfs: last write delta would result in integer wraparound")
	}

	// See writeSizePrecision to get the unit of writeSize. As of 1/26/2023, the unit is KBs.
	writeSize := writeSizeInBytes / writeSizePrecision
	// If the size of the write is larger than we can store in the packed int, store the max
	// value we can store in the packed int.
	const writeSizeCeiling = 1<<writeSizeBits - 1
	if writeSize > writeSizeCeiling {
		writeSize = writeSizeCeiling
	}

	return uint64(deltaMillis)<<(64-deltaBits) | uint64(writeSize)<<(64-deltaBits-writeSizeBits) | uint64(opType)
}

func unpack(packed uint64) (delta time.Duration, writeSizeInBytes int, opType OpType) {
	delta = time.Duration(packed>>(64-deltaBits)) * time.Millisecond
	wz := int64(packed>>(64-deltaBits-writeSizeBits)) & ((1 << writeSizeBits) - 1) * writeSizePrecision
	// Given the packing scheme, converting wz to an int will not truncate anything.
	writeSizeInBytes = int(wz)
	opType = OpType(packed & 0xf)
	return delta, writeSizeInBytes, opType
}

// diskHealthCheckingDir implements disk-health checking for directories. Unlike
// other files, we allow directories to receive concurrent write operations
// (Syncs are the only write operations supported by a directory.) Since the
// diskHealthCheckingFile's timeDiskOp can only track a single in-flight
// operation at a time, we time the operation using the filesystem-level
// timeFilesystemOp function instead.
type diskHealthCheckingDir struct {
	File
	name string
	fs   *diskHealthCheckingFS
}

// Sync implements the io.Syncer interface.
func (d *diskHealthCheckingDir) Sync() (err error) {
	d.fs.timeFilesystemOp(d.name, OpTypeSync, func() {
		err = d.File.Sync()
	})
	return err
}

// DiskSlowInfo captures info about detected slow operations on the vfs.
type DiskSlowInfo struct {
	// Path of file being written to.
	Path string
	// Operation being performed on the file.
	OpType OpType
	// Size of write in bytes, if the write is sized.
	WriteSize int
	// Duration that has elapsed since this disk operation started.
	Duration time.Duration
}

func (i DiskSlowInfo) String() string {
	return redact.StringWithoutMarkers(i)
}

// SafeFormat implements redact.SafeFormatter.
func (i DiskSlowInfo) SafeFormat(w redact.SafePrinter, _ rune) {
	switch i.OpType {
	// Operations for which i.WriteSize is meaningful.
	case OpTypeWrite, OpTypeSyncTo, OpTypePreallocate:
		w.Printf("disk slowness detected: %s on file %s (%d bytes) has been ongoing for %0.1fs",
			redact.Safe(i.OpType.String()), redact.Safe(filepath.Base(i.Path)),
			redact.Safe(i.WriteSize), redact.Safe(i.Duration.Seconds()))
	default:
		w.Printf("disk slowness detected: %s on file %s has been ongoing for %0.1fs",
			redact.Safe(i.OpType.String()), redact.Safe(filepath.Base(i.Path)),
			redact.Safe(i.Duration.Seconds()))
	}
}

// diskHealthCheckingFS adds disk-health checking facilities to a VFS.
// It times disk write operations in two ways:
//
// 1. Wrapping vfs.Files.
//
// The bulk of write I/O activity is file writing and syncing, invoked through
// the `vfs.File` interface. This VFS wraps all files open for writing with a
// special diskHealthCheckingFile implementation of the vfs.File interface. See
// above for the implementation.
//
// 2. Monitoring filesystem metadata operations.
//
// Filesystem metadata operations (create, link, remove, rename, etc) are also
// sources of disk writes. Unlike a vfs.File which requires Write and Sync calls
// to be sequential, a vfs.FS may receive these filesystem metadata operations
// in parallel. To accommodate this parallelism, the diskHealthCheckingFS's
// write-oriented filesystem operations record their start times into a 'slot'
// on the filesystem. A single long-running goroutine periodically scans the
// slots looking for slow operations.
//
// The number of slots on a diskHealthCheckingFS grows to a working set of the
// maximum concurrent filesystem operations. This is expected to be very few
// for these reasons:
//  1. Pebble has limited write concurrency. Flushes, compactions and WAL
//     rotations are the primary sources of filesystem metadata operations. With
//     the default max-compaction concurrency, these operations require at most 5
//     concurrent slots if all 5 perform a filesystem metadata operation
//     simultaneously.
//  2. Pebble's limited concurrent I/O writers spend most of their time
//     performing file I/O, not performing the filesystem metadata operations that
//     require recording a slot on the diskHealthCheckingFS.
//  3. In CockroachDB, each additional store/Pebble instance has its own vfs.FS
//     which provides a separate goroutine and set of slots.
//  4. In CockroachDB, many of the additional sources of filesystem metadata
//     operations (like encryption-at-rest) are sequential with respect to Pebble's
//     threads.
type diskHealthCheckingFS struct {
	tickInterval      time.Duration
	diskSlowThreshold time.Duration
	onSlowDisk        func(DiskSlowInfo)
	fs                FS
	mu                struct {
		sync.Mutex
		tickerRunning bool
		stopper       chan struct{}
		inflight      []*slot
	}
	// prealloc preallocates the memory for mu.inflight slots and the slice
	// itself. The contained fields are not accessed directly except by
	// WithDiskHealthChecks when initializing mu.inflight. The number of slots
	// in d.mu.inflight will grow to the maximum number of concurrent file
	// metadata operations (create, remove, link, etc). If the number of
	// concurrent operations never exceeds preallocatedSlotCount, we'll never
	// incur an additional allocation.
	prealloc struct {
		slots        [preallocatedSlotCount]slot
		slotPtrSlice [preallocatedSlotCount]*slot
	}
}

type slot struct {
	name       string
	opType     OpType
	startNanos atomic.Int64
}

// diskHealthCheckingFS implements FS.
var _ FS = (*diskHealthCheckingFS)(nil)

// WithDiskHealthChecks wraps an FS and ensures that all write-oriented
// operations on the FS are wrapped with disk health detection checks. Disk
// operations that are observed to take longer than diskSlowThreshold trigger an
// onSlowDisk call.
//
// A threshold of zero disables disk-health checking.
func WithDiskHealthChecks(
	innerFS FS, diskSlowThreshold time.Duration, onSlowDisk func(info DiskSlowInfo),
) (FS, io.Closer) {
	if diskSlowThreshold == 0 {
		return innerFS, noopCloser{}
	}

	fs := &diskHealthCheckingFS{
		fs:                innerFS,
		tickInterval:      defaultTickInterval,
		diskSlowThreshold: diskSlowThreshold,
		onSlowDisk:        onSlowDisk,
	}
	fs.mu.stopper = make(chan struct{})
	// The fs holds preallocated slots and a preallocated array of slot pointers
	// with equal length. Initialize the inflight slice to use a slice backed by
	// the preallocated array with each slot initialized to a preallocated slot.
	fs.mu.inflight = fs.prealloc.slotPtrSlice[:]
	for i := range fs.mu.inflight {
		fs.mu.inflight[i] = &fs.prealloc.slots[i]
	}
	return fs, fs
}

func (d *diskHealthCheckingFS) timeFilesystemOp(name string, opType OpType, op func()) {
	if d == nil {
		op()
		return
	}

	// Record this operation's start time on the FS, so that the long-running
	// goroutine can monitor the filesystem operation.
	//
	// The diskHealthCheckingFile implementation uses a single field that is
	// atomically updated, taking advantage of the fact that writes to a single
	// vfs.File handle are not performed in parallel. The vfs.FS however may
	// receive write filesystem operations in parallel. To accommodate this
	// parallelism, writing goroutines append their start time to a
	// mutex-protected vector. On ticks, the long-running goroutine scans the
	// vector searching for start times older than the slow-disk threshold. When
	// a writing goroutine completes its operation, it atomically overwrites its
	// slot to signal completion.
	var s *slot
	func() {
		d.mu.Lock()
		defer d.mu.Unlock()

		// If there's no long-running goroutine to monitor this filesystem
		// operation, start one.
		if !d.mu.tickerRunning {
			d.startTickerLocked()
		}

		startNanos := time.Now().UnixNano()
		for i := 0; i < len(d.mu.inflight); i++ {
			if d.mu.inflight[i].startNanos.Load() == 0 {
				// This slot is not in use. Claim it.
				s = d.mu.inflight[i]
				s.name = name
				s.opType = opType
				s.startNanos.Store(startNanos)
				break
			}
		}
		// If we didn't find any unused slots, create a new slot and append it.
		// This slot will exist forever. The number of slots will grow to the
		// maximum number of concurrent filesystem operations over the lifetime
		// of the process. Only operations that grow the number of slots must
		// incur an allocation.
		if s == nil {
			s = &slot{
				name:   name,
				opType: opType,
			}
			s.startNanos.Store(startNanos)
			d.mu.inflight = append(d.mu.inflight, s)
		}
	}()

	op()

	// Signal completion by zeroing the start time.
	s.startNanos.Store(0)
}

// startTickerLocked starts a new goroutine with a ticker to monitor disk
// filesystem operations. Requires d.mu and !d.mu.tickerRunning.
func (d *diskHealthCheckingFS) startTickerLocked() {
	d.mu.tickerRunning = true
	stopper := d.mu.stopper
	go func() {
		ticker := time.NewTicker(d.tickInterval)
		defer ticker.Stop()
		type exceededSlot struct {
			name       string
			opType     OpType
			startNanos int64
		}
		var exceededSlots []exceededSlot

		for {
			select {
			case <-ticker.C:
				// Scan the inflight slots for any slots recording a start
				// time older than the diskSlowThreshold.
				exceededSlots = exceededSlots[:0]
				d.mu.Lock()
				now := time.Now()
				for i := range d.mu.inflight {
					nanos := d.mu.inflight[i].startNanos.Load()
					if nanos != 0 && time.Unix(0, nanos).Add(d.diskSlowThreshold).Before(now) {
						// diskSlowThreshold was exceeded. Copy this inflightOp into
						// exceededSlots and call d.onSlowDisk after dropping the mutex.
						inflightOp := exceededSlot{
							name:       d.mu.inflight[i].name,
							opType:     d.mu.inflight[i].opType,
							startNanos: nanos,
						}
						exceededSlots = append(exceededSlots, inflightOp)
					}
				}
				d.mu.Unlock()
				for i := range exceededSlots {
					d.onSlowDisk(
						DiskSlowInfo{
							Path:      exceededSlots[i].name,
							OpType:    exceededSlots[i].opType,
							WriteSize: 0, // writes at the fs level are not sized
							Duration:  now.Sub(time.Unix(0, exceededSlots[i].startNanos)),
						})
				}
			case <-stopper:
				return
			}
		}
	}()
}

// Close implements io.Closer. Close stops the long-running goroutine that
// monitors for slow filesystem metadata operations. Close may be called
// multiple times. If the filesystem is used after Close has been called, a new
// long-running goroutine will be created.
func (d *diskHealthCheckingFS) Close() error {
	d.mu.Lock()
	if !d.mu.tickerRunning {
		// Nothing to stop.
		d.mu.Unlock()
		return nil
	}

	// Grab the stopper so we can request the long-running goroutine to stop.
	// Replace the stopper in case this FS is reused. It's possible to Close and
	// reuse a disk-health checking FS. This is to accommodate the on-by-default
	// behavior in Pebble, and the possibility that users may continue to use
	// the Pebble default FS beyond the lifetime of a single DB.
	stopper := d.mu.stopper
	d.mu.stopper = make(chan struct{})
	d.mu.tickerRunning = false
	d.mu.Unlock()

	// Ask the long-running goroutine to stop. This is a synchronous channel
	// send.
	stopper <- struct{}{}
	close(stopper)
	return nil
}

// Create implements the FS interface.
func (d *diskHealthCheckingFS) Create(name string) (File, error) {
	var f File
	var err error
	d.timeFilesystemOp(name, OpTypeCreate, func() {
		f, err = d.fs.Create(name)
	})
	if err != nil {
		return f, err
	}
	if d.diskSlowThreshold == 0 {
		return f, nil
	}
	checkingFile := newDiskHealthCheckingFile(f, d.diskSlowThreshold, func(opType OpType, writeSizeInBytes int, duration time.Duration) {
		d.onSlowDisk(
			DiskSlowInfo{
				Path:      name,
				OpType:    opType,
				WriteSize: writeSizeInBytes,
				Duration:  duration,
			})
	})
	checkingFile.startTicker()
	return checkingFile, nil
}

// GetDiskUsage implements the FS interface.
func (d *diskHealthCheckingFS) GetDiskUsage(path string) (DiskUsage, error) {
	return d.fs.GetDiskUsage(path)
}

// Link implements the FS interface.
func (d *diskHealthCheckingFS) Link(oldname, newname string) error {
	var err error
	d.timeFilesystemOp(newname, OpTypeLink, func() {
		err = d.fs.Link(oldname, newname)
	})
	return err
}

// List implements the FS interface.
func (d *diskHealthCheckingFS) List(dir string) ([]string, error) {
	return d.fs.List(dir)
}

// Lock implements the FS interface.
func (d *diskHealthCheckingFS) Lock(name string) (io.Closer, error) {
	return d.fs.Lock(name)
}

// MkdirAll implements the FS interface.
func (d *diskHealthCheckingFS) MkdirAll(dir string, perm os.FileMode) error {
	var err error
	d.timeFilesystemOp(dir, OpTypeMkdirAll, func() {
		err = d.fs.MkdirAll(dir, perm)
	})
	return err
}

// Open implements the FS interface.
func (d *diskHealthCheckingFS) Open(name string, opts ...OpenOption) (File, error) {
	return d.fs.Open(name, opts...)
}

// OpenReadWrite implements the FS interface.
func (d *diskHealthCheckingFS) OpenReadWrite(name string, opts ...OpenOption) (File, error) {
	return d.fs.OpenReadWrite(name, opts...)
}

// OpenDir implements the FS interface.
func (d *diskHealthCheckingFS) OpenDir(name string) (File, error) {
	f, err := d.fs.OpenDir(name)
	if err != nil {
		return f, err
	}
	// Directories opened with OpenDir must be opened with health checking,
	// because they may be explicitly synced.
	return &diskHealthCheckingDir{
		File: f,
		name: name,
		fs:   d,
	}, nil
}

// PathBase implements the FS interface.
func (d *diskHealthCheckingFS) PathBase(path string) string {
	return d.fs.PathBase(path)
}

// PathJoin implements the FS interface.
func (d *diskHealthCheckingFS) PathJoin(elem ...string) string {
	return d.fs.PathJoin(elem...)
}

// PathDir implements the FS interface.
func (d *diskHealthCheckingFS) PathDir(path string) string {
	return d.fs.PathDir(path)
}

// Remove implements the FS interface.
func (d *diskHealthCheckingFS) Remove(name string) error {
	var err error
	d.timeFilesystemOp(name, OpTypeRemove, func() {
		err = d.fs.Remove(name)
	})
	return err
}

// RemoveAll implements the FS interface.
func (d *diskHealthCheckingFS) RemoveAll(name string) error {
	var err error
	d.timeFilesystemOp(name, OpTypeRemoveAll, func() {
		err = d.fs.RemoveAll(name)
	})
	return err
}

// Rename implements the FS interface.
func (d *diskHealthCheckingFS) Rename(oldname, newname string) error {
	var err error
	d.timeFilesystemOp(newname, OpTypeRename, func() {
		err = d.fs.Rename(oldname, newname)
	})
	return err
}

// ReuseForWrite implements the FS interface.
func (d *diskHealthCheckingFS) ReuseForWrite(oldname, newname string) (File, error) {
	var f File
	var err error
	d.timeFilesystemOp(newname, OpTypeReuseForWrite, func() {
		f, err = d.fs.ReuseForWrite(oldname, newname)
	})
	if err != nil {
		return f, err
	}
	if d.diskSlowThreshold == 0 {
		return f, nil
	}
	checkingFile := newDiskHealthCheckingFile(f, d.diskSlowThreshold, func(opType OpType, writeSizeInBytes int, duration time.Duration) {
		d.onSlowDisk(
			DiskSlowInfo{
				Path:      newname,
				OpType:    opType,
				WriteSize: writeSizeInBytes,
				Duration:  duration,
			})
	})
	checkingFile.startTicker()
	return checkingFile, nil
}

// Stat implements the FS interface.
func (d *diskHealthCheckingFS) Stat(name string) (os.FileInfo, error) {
	return d.fs.Stat(name)
}

type noopCloser struct{}

func (noopCloser) Close() error { return nil }
