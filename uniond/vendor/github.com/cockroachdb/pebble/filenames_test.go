// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

// TestSetCurrentFileCrash tests a crash that occurs during
// a MANIFEST roll, leaving the temporary CURRENT file on
// the filesystem. These temporary files should be cleaned
// up on Open.
func TestSetCurrentFileCrash(t *testing.T) {
	mem := vfs.NewMem()

	// Initialize a fresh database to write the initial MANIFEST.
	{
		d, err := Open("", &Options{FS: mem})
		require.NoError(t, err)
		require.NoError(t, d.Close())
	}

	// Open the database again, this time with a FS that
	// errors on Rename and a tiny max manifest file size
	// to force manifest rolls.
	{
		wantErr := errors.New("rename error")
		_, err := Open("", &Options{
			FS:                    renameErrorFS{FS: mem, err: wantErr},
			Logger:                noFatalLogger{t: t},
			MaxManifestFileSize:   1,
			L0CompactionThreshold: 10,
		})
		// Open should fail during a manifest roll,
		// leaving a temp dir on the filesystem.
		if !errors.Is(err, wantErr) {
			t.Fatal(err)
		}
	}

	// A temp file should be left on the filesystem
	// from the failed Rename of the CURRENT file.
	if temps := allTempFiles(t, mem); len(temps) == 0 {
		t.Fatal("no temp files on the filesystem")
	}

	// Open the database a third time with a normal
	// filesystem again. It should clean up any temp
	// files on Open.
	{
		d, err := Open("", &Options{
			FS:                    mem,
			MaxManifestFileSize:   1,
			L0CompactionThreshold: 10,
		})
		require.NoError(t, err)
		require.NoError(t, d.Close())
		if temps := allTempFiles(t, mem); len(temps) > 0 {
			t.Fatalf("temporary files still on disk: %#v\n", temps)
		}
	}
}

func allTempFiles(t *testing.T, fs vfs.FS) []string {
	var files []string
	ls, err := fs.List("")
	require.NoError(t, err)
	for _, f := range ls {
		ft, _, ok := base.ParseFilename(fs, f)
		if ok && ft == fileTypeTemp {
			files = append(files, f)
		}
	}
	return files
}

type renameErrorFS struct {
	vfs.FS
	err error
}

func (fs renameErrorFS) Rename(oldname string, newname string) error {
	return fs.err
}

// noFatalLogger implements Logger, logging to the contained
// *testing.T. Notably it does not panic on calls to Fatalf
// to enable unit tests of fatal logic.
type noFatalLogger struct {
	t *testing.T
}

func (l noFatalLogger) Infof(format string, args ...interface{}) {
	l.t.Logf(format, args...)
}

func (l noFatalLogger) Fatalf(format string, args ...interface{}) {
	l.t.Logf(format, args...)
}
