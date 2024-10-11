// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rawdb

import (
	"os"
	"path/filepath"
	"sync"

	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
)

const tmpSuffix = ".tmp"

// freezerOpenFunc is the function used to open/create a freezer.
type freezerOpenFunc = func() (*Freezer, error)

// ResettableFreezer is a wrapper of the freezer which makes the
// freezer resettable.
type ResettableFreezer struct {
	freezer *Freezer
	opener  freezerOpenFunc
	datadir string
	lock    sync.RWMutex
}

// NewResettableFreezer creates a resettable freezer, note freezer is
// only resettable if the passed file directory is exclusively occupied
// by the freezer. And also the user-configurable ancient root directory
// is **not** supported for reset since it might be a mount and rename
// will cause a copy of hundreds of gigabyte into local directory. It
// needs some other file based solutions.
//
// The reset function will delete directory atomically and re-create the
// freezer from scratch.
func NewResettableFreezer(datadir string, namespace string, readonly bool, maxTableSize uint32, tables map[string]bool) (*ResettableFreezer, error) {
	if err := cleanup(datadir); err != nil {
		return nil, err
	}
	opener := func() (*Freezer, error) {
		return NewFreezer(datadir, namespace, readonly, maxTableSize, tables)
	}
	freezer, err := opener()
	if err != nil {
		return nil, err
	}
	return &ResettableFreezer{
		freezer: freezer,
		opener:  opener,
		datadir: datadir,
	}, nil
}

// Reset deletes the file directory exclusively occupied by the freezer and
// recreate the freezer from scratch. The atomicity of directory deletion
// is guaranteed by the rename operation, the leftover directory will be
// cleaned up in next startup in case crash happens after rename.
func (f *ResettableFreezer) Reset() error {
	f.lock.Lock()
	defer f.lock.Unlock()

	if err := f.freezer.Close(); err != nil {
		return err
	}
	tmp := tmpName(f.datadir)
	if err := os.Rename(f.datadir, tmp); err != nil {
		return err
	}
	if err := os.RemoveAll(tmp); err != nil {
		return err
	}
	freezer, err := f.opener()
	if err != nil {
		return err
	}
	f.freezer = freezer
	return nil
}

// Close terminates the chain freezer, unmapping all the data files.
func (f *ResettableFreezer) Close() error {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.Close()
}

// HasAncient returns an indicator whether the specified ancient data exists
// in the freezer
func (f *ResettableFreezer) HasAncient(kind string, number uint64) (bool, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.HasAncient(kind, number)
}

// Ancient retrieves an ancient binary blob from the append-only immutable files.
func (f *ResettableFreezer) Ancient(kind string, number uint64) ([]byte, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.Ancient(kind, number)
}

// AncientRange retrieves multiple items in sequence, starting from the index 'start'.
// It will return
//   - at most 'max' items,
//   - at least 1 item (even if exceeding the maxByteSize), but will otherwise
//     return as many items as fit into maxByteSize
func (f *ResettableFreezer) AncientRange(kind string, start, count, maxBytes uint64) ([][]byte, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.AncientRange(kind, start, count, maxBytes)
}

// Ancients returns the length of the frozen items.
func (f *ResettableFreezer) Ancients() (uint64, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.Ancients()
}

// Tail returns the number of first stored item in the freezer.
func (f *ResettableFreezer) Tail() (uint64, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.Tail()
}

// AncientSize returns the ancient size of the specified category.
func (f *ResettableFreezer) AncientSize(kind string) (uint64, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.AncientSize(kind)
}

// ReadAncients runs the given read operation while ensuring that no writes take place
// on the underlying freezer.
func (f *ResettableFreezer) ReadAncients(fn func(ethdb.AncientReaderOp) error) (err error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.ReadAncients(fn)
}

// ModifyAncients runs the given write operation.
func (f *ResettableFreezer) ModifyAncients(fn func(ethdb.AncientWriteOp) error) (writeSize int64, err error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.ModifyAncients(fn)
}

// TruncateHead discards any recent data above the provided threshold number.
// It returns the previous head number.
func (f *ResettableFreezer) TruncateHead(items uint64) (uint64, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.TruncateHead(items)
}

// TruncateTail discards any recent data below the provided threshold number.
// It returns the previous value
func (f *ResettableFreezer) TruncateTail(tail uint64) (uint64, error) {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.TruncateTail(tail)
}

// Sync flushes all data tables to disk.
func (f *ResettableFreezer) Sync() error {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.Sync()
}

// MigrateTable processes the entries in a given table in sequence
// converting them to a new format if they're of an old format.
func (f *ResettableFreezer) MigrateTable(kind string, convert convertLegacyFn) error {
	f.lock.RLock()
	defer f.lock.RUnlock()

	return f.freezer.MigrateTable(kind, convert)
}

// cleanup removes the directory located in the specified path
// has the name with deletion marker suffix.
func cleanup(path string) error {
	parent := filepath.Dir(path)
	if _, err := os.Lstat(parent); os.IsNotExist(err) {
		return nil
	}
	dir, err := os.Open(parent)
	if err != nil {
		return err
	}
	names, err := dir.Readdirnames(0)
	if err != nil {
		return err
	}
	if cerr := dir.Close(); cerr != nil {
		return cerr
	}
	for _, name := range names {
		if name == filepath.Base(path)+tmpSuffix {
			log.Info("Removed leftover freezer directory", "name", name)
			return os.RemoveAll(filepath.Join(parent, name))
		}
	}
	return nil
}

func tmpName(path string) string {
	return filepath.Join(filepath.Dir(path), filepath.Base(path)+tmpSuffix)
}
