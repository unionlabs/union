// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package atomicfs

import (
	"fmt"
	"strconv"
	"strings"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/errors/oserror"
	"github.com/cockroachdb/pebble/vfs"
)

// ReadMarker looks up the current state of a marker returning just the
// current value of the marker. Callers that may need to move the marker
// to a new value should use LocateMarker.
func ReadMarker(fs vfs.FS, dir, markerName string) (string, error) {
	state, err := scanForMarker(fs, dir, markerName)
	if err != nil {
		return "", err
	}
	return state.value, nil
}

// LocateMarker loads the current state of a marker. It returns a handle
// to the Marker that may be used to move the marker and the
// current value of the marker.
func LocateMarker(fs vfs.FS, dir, markerName string) (*Marker, string, error) {
	state, err := scanForMarker(fs, dir, markerName)
	if err != nil {
		return nil, "", err
	}
	dirFD, err := fs.OpenDir(dir)
	if err != nil {
		return nil, "", err
	}
	return &Marker{
		fs:            fs,
		dir:           dir,
		dirFD:         dirFD,
		name:          markerName,
		filename:      state.filename,
		iter:          state.iter,
		obsoleteFiles: state.obsolete,
	}, state.value, nil
}

type scannedState struct {
	// filename is the latest marker file found (the one with the highest iter value).
	filename string
	iter     uint64
	value    string
	// obsolete is a list of earlier markers that were found.
	obsolete []string
}

func scanForMarker(fs vfs.FS, dir, markerName string) (scannedState, error) {
	ls, err := fs.List(dir)
	if err != nil {
		return scannedState{}, err
	}
	var state scannedState
	for _, filename := range ls {
		if !strings.HasPrefix(filename, `marker.`) {
			continue
		}
		// Any filenames with the `marker.` prefix are required to be
		// well-formed and parse as markers.
		name, iter, value, err := parseMarkerFilename(filename)
		if err != nil {
			return scannedState{}, err
		}
		if name != markerName {
			continue
		}

		if state.filename == "" || state.iter < iter {
			if state.filename != "" {
				state.obsolete = append(state.obsolete, state.filename)
			}
			state.filename = filename
			state.iter = iter
			state.value = value
		} else {
			state.obsolete = append(state.obsolete, filename)
		}
	}
	return state, nil
}

// A Marker provides an interface for maintaining a single string value on the
// filesystem. The marker may be atomically moved from value to value.
//
// The implementation creates a new marker file for each new value, embedding
// the value in the marker filename.
//
// Marker is not safe for concurrent use. Multiple processes may not read or
// move the same marker simultaneously. A Marker may only be constructed through
// LocateMarker.
//
// Marker names must be unique within the directory.
type Marker struct {
	fs    vfs.FS
	dir   string
	dirFD vfs.File
	// name identifies the marker.
	name string
	// filename contains the entire filename of the current marker. It
	// has a format of `marker.<name>.<iter>.<value>`. It's not
	// necessarily in sync with iter, since filename is only updated
	// when the marker is successfully moved.
	filename string
	// iter holds the current iteration value. It matches the iteration
	// value encoded within filename, if filename is non-empty. Iter is
	// monotonically increasing over the lifetime of a marker. Actual
	// marker files will always have a positive iter value.
	iter uint64
	// obsoleteFiles holds a list of marker files discovered by LocateMarker that
	// are old values for this marker. These files may exist in certain error
	// cases or crashes (e.g. if the deletion of the previous marker file failed
	// during Move).
	obsoleteFiles []string
}

func markerFilename(name string, iter uint64, value string) string {
	return fmt.Sprintf("marker.%s.%06d.%s", name, iter, value)
}

func parseMarkerFilename(s string) (name string, iter uint64, value string, err error) {
	// Check for and remove the `marker.` prefix.
	if !strings.HasPrefix(s, `marker.`) {
		return "", 0, "", errors.Newf("invalid marker filename: %q", s)
	}
	s = s[len(`marker.`):]

	// Extract the marker's name.
	i := strings.IndexByte(s, '.')
	if i == -1 {
		return "", 0, "", errors.Newf("invalid marker filename: %q", s)
	}
	name = s[:i]
	s = s[i+1:]

	// Extract the marker's iteration number.
	i = strings.IndexByte(s, '.')
	if i == -1 {
		return "", 0, "", errors.Newf("invalid marker filename: %q", s)
	}
	iter, err = strconv.ParseUint(s[:i], 10, 64)
	if err != nil {
		return "", 0, "", errors.Newf("invalid marker filename: %q", s)
	}

	// Everything after the iteration's `.` delimiter is the value.
	s = s[i+1:]

	return name, iter, s, nil
}

// Close releases all resources in use by the marker.
func (a *Marker) Close() error {
	return a.dirFD.Close()
}

// Move atomically moves the marker to a new value.
//
// If Move returns a nil error, the new marker value is guaranteed to be
// persisted to stable storage. If Move returns an error, the current
// value of the marker may be the old value or the new value. Callers
// may retry a Move error.
//
// If an error occurs while syncing the directory, Move panics.
func (a *Marker) Move(newValue string) error {
	a.iter++
	dstFilename := markerFilename(a.name, a.iter, newValue)
	dstPath := a.fs.PathJoin(a.dir, dstFilename)
	oldFilename := a.filename

	// Create the new marker.
	f, err := a.fs.Create(dstPath)
	if err != nil {
		// On a distributed filesystem, an error doesn't guarantee that
		// the file wasn't created. A retry of the same Move call will
		// use a new iteration value, and try to a create a new file. If
		// the errored invocation was actually successful in creating
		// the file, we'll leak a file. That's okay, because the next
		// time the marker is located we'll add it to the obsolete files
		// list.
		//
		// Note that the unconditional increment of `a.iter` means that
		// `a.iter` and `a.filename` are not necessarily in sync,
		// because `a.filename` is only updated on success.
		return err
	}
	a.filename = dstFilename
	if err := f.Close(); err != nil {
		return err
	}

	// Remove the now defunct file. If an error is surfaced, we record
	// the file as an obsolete file.  The file's presence does not
	// affect correctness, and it will be cleaned up the next time
	// RemoveObsolete is called, either by this process or the next.
	if oldFilename != "" {
		if err := a.fs.Remove(a.fs.PathJoin(a.dir, oldFilename)); err != nil && !oserror.IsNotExist(err) {
			a.obsoleteFiles = append(a.obsoleteFiles, oldFilename)
		}
	}

	// Sync the directory to ensure marker movement is synced.
	if err := a.dirFD.Sync(); err != nil {
		// Fsync errors are unrecoverable.
		// See https://wiki.postgresql.org/wiki/Fsync_Errors and
		// https://danluu.com/fsyncgate.
		panic(errors.WithStack(err))
	}
	return nil
}

// NextIter returns the next iteration number that the marker will use.
// Clients may use this number for formulating new values that are
// unused.
func (a *Marker) NextIter() uint64 {
	return a.iter + 1
}

// RemoveObsolete removes any obsolete files discovered while locating
// the marker or files unable to be removed during Move.
func (a *Marker) RemoveObsolete() error {
	for i, filename := range a.obsoleteFiles {
		if err := a.fs.Remove(a.fs.PathJoin(a.dir, filename)); err != nil && !oserror.IsNotExist(err) {
			a.obsoleteFiles = a.obsoleteFiles[i:]
			return err
		}
	}
	a.obsoleteFiles = nil
	return nil
}
