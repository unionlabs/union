// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"fmt"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
)

func TestManualFlush(t *testing.T) {
	getOptions := func() *Options {
		opts := &Options{
			FS:                    vfs.NewMem(),
			L0CompactionThreshold: 10,
		}
		opts.DisableAutomaticCompactions = true
		return opts
	}
	d, err := Open("", getOptions())
	require.NoError(t, err)
	defer func() {
		require.NoError(t, d.Close())
	}()

	datadriven.RunTest(t, "testdata/manual_flush", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "batch":
			b := d.NewBatch()
			if err := runBatchDefineCmd(td, b); err != nil {
				return err.Error()
			}
			b.Commit(nil)
			return ""

		case "flush":
			if err := d.Flush(); err != nil {
				return err.Error()
			}

			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "async-flush":
			d.mu.Lock()
			cur := d.mu.versions.currentVersion()
			d.mu.Unlock()

			if _, err := d.AsyncFlush(); err != nil {
				return err.Error()
			}

			err := try(100*time.Microsecond, 20*time.Second, func() error {
				d.mu.Lock()
				defer d.mu.Unlock()
				if cur == d.mu.versions.currentVersion() {
					return errors.New("flush has not occurred")
				}
				return nil
			})
			if err != nil {
				return err.Error()
			}

			d.mu.Lock()
			s := d.mu.versions.currentVersion().String()
			d.mu.Unlock()
			return s

		case "reset":
			if err := d.Close(); err != nil {
				return err.Error()
			}
			d, err = Open("", getOptions())
			if err != nil {
				return err.Error()
			}
			return ""

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

// TestFlushDelRangeEmptyKey tests flushing a range tombstone that begins with
// an empty key. The empty key is a valid key but can be confused with nil.
func TestFlushDelRangeEmptyKey(t *testing.T) {
	d, err := Open("", &Options{FS: vfs.NewMem()})
	require.NoError(t, err)
	require.NoError(t, d.DeleteRange([]byte{}, []byte("z"), nil))
	require.NoError(t, d.Flush())
	require.NoError(t, d.Close())
}

// TestFlushEmptyKey tests that flushing an empty key does not trigger that key
// order invariant assertions.
func TestFlushEmptyKey(t *testing.T) {
	d, err := Open("", &Options{FS: vfs.NewMem()})
	require.NoError(t, err)
	require.NoError(t, d.Set(nil, []byte("hello"), nil))
	require.NoError(t, d.Flush())
	val, closer, err := d.Get(nil)
	require.NoError(t, err)
	require.Equal(t, val, []byte("hello"))
	require.NoError(t, closer.Close())
	require.NoError(t, d.Close())
}
