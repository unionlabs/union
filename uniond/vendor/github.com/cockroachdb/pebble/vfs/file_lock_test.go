// Copyright 2014 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package vfs_test

import (
	"bytes"
	"flag"
	"os"
	"os/exec"
	"testing"

	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

var lockFilename = flag.String("lockfile", "", "File to lock. A non-empty value implies a child process.")

func spawn(prog, filename string) ([]byte, error) {
	return exec.Command(prog, "-lockfile", filename, "-test.v",
		"-test.run=TestLock$").CombinedOutput()
}

// TestLock locks a file, spawns a second process that attempts to grab the
// lock to verify it fails.
// Then it closes the lock, and spawns a third copy to verify it can be
// relocked.
func TestLock(t *testing.T) {
	child := *lockFilename != ""
	var filename string
	if child {
		filename = *lockFilename
	} else {
		f, err := os.CreateTemp("", "golang-pebble-db-testlock-")
		require.NoError(t, err)

		filename = f.Name()
		// NB: On Windows, locking will fail if the file is already open by the
		// current process, so we close the lockfile here.
		require.NoError(t, f.Close())
		defer os.Remove(filename)
	}

	// Avoid truncating an existing, non-empty file.
	fi, err := os.Stat(filename)
	if err == nil && fi.Size() != 0 {
		t.Fatalf("The file %s is not empty", filename)
	}

	t.Logf("Locking: %s", filename)
	lock, err := vfs.Default.Lock(filename)
	if err != nil {
		t.Fatalf("Could not lock %s: %v", filename, err)
	}

	if !child {
		t.Logf("Spawning child, should fail to grab lock.")
		out, err := spawn(os.Args[0], filename)
		if err == nil {
			t.Fatalf("Attempt to grab open lock should have failed.\n%s", out)
		}
		if !bytes.Contains(out, []byte("Could not lock")) {
			t.Fatalf("Child failed with unexpected output: %s", out)
		}
		t.Logf("Child failed to grab lock as expected.")
	}

	t.Logf("Unlocking %s", filename)
	if err := lock.Close(); err != nil {
		t.Fatalf("Could not unlock %s: %v", filename, err)
	}

	if !child {
		t.Logf("Spawning child, should successfully grab lock.")
		if out, err := spawn(os.Args[0], filename); err != nil {
			t.Fatalf("Attempt to re-open lock should have succeeded: %v\n%s",
				err, out)
		}
		t.Logf("Child grabbed lock.")
	}
}

func TestLockSameProcess(t *testing.T) {
	f, err := os.CreateTemp("", "pebble-testlocksameprocess-")
	require.NoError(t, err)
	filename := f.Name()

	// NB: On Windows, locking will fail if the file is already open by the
	// current process, so we close the lockfile here.
	require.NoError(t, f.Close())
	defer os.Remove(filename)

	lock1, err := vfs.Default.Lock(filename)
	require.NoError(t, err)

	// Locking the file again from within the same process should fail.
	// On Unix, Lock should detect the file in the global map of
	// process-locked files.
	// On Windows, locking will fail since the file is already open by the
	// current process.
	_, err = vfs.Default.Lock(filename)
	require.Error(t, err)

	require.NoError(t, lock1.Close())
}
