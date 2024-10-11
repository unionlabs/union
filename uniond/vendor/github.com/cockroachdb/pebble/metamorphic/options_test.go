// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/kr/pretty"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestSetupInitialState(t *testing.T) {
	// Construct a small database in the test's TempDir.
	initialStatePath := t.TempDir()
	initialDataPath := vfs.Default.PathJoin(initialStatePath, "data")
	{
		d, err := pebble.Open(initialDataPath, &pebble.Options{})
		require.NoError(t, err)
		const maxKeyLen = 2
		ks := testkeys.Alpha(maxKeyLen)
		var key [maxKeyLen]byte
		for i := int64(0); i < ks.Count(); i++ {
			n := testkeys.WriteKey(key[:], ks, i)
			require.NoError(t, d.Set(key[:n], key[:n], pebble.NoSync))
			if i%100 == 0 {
				require.NoError(t, d.Flush())
			}
		}
		require.NoError(t, d.Close())
	}
	ls, err := vfs.Default.List(initialStatePath)
	require.NoError(t, err)

	// setupInitialState with an initial state path set to the test's TempDir
	// should populate opts.opts.FS with the directory's contents.
	opts := &TestOptions{
		Opts:             defaultOptions(),
		initialStatePath: initialStatePath,
		initialStateDesc: "test",
	}
	require.NoError(t, setupInitialState("data", opts))
	copied, err := opts.Opts.FS.List("")
	require.NoError(t, err)
	require.ElementsMatch(t, ls, copied)
}

func TestOptionsRoundtrip(t *testing.T) {
	// Some fields must be ignored to avoid spurious diffs.
	ignorePrefixes := []string{
		// Pointers
		"Cache:",
		"Cache.",
		"FS:",
		"TableCache:",
		// Function pointers
		"BlockPropertyCollectors:",
		"EventListener:",
		"MaxConcurrentCompactions:",
		"Experimental.DisableIngestAsFlushable:",
		"Experimental.EnableValueBlocks:",
		"Experimental.IneffectualSingleDeleteCallback:",
		"Experimental.IngestSplit:",
		"Experimental.RemoteStorage:",
		"Experimental.SingleDeleteInvariantViolationCallback:",
		// Floating points
		"Experimental.PointTombstoneWeight:",
	}

	// Ensure that we unref any caches created, so invariants builds don't
	// complain about the leaked ref counts.
	maybeUnref := func(o *TestOptions) {
		if o.Opts.Cache != nil {
			o.Opts.Cache.Unref()
		}
	}

	checkOptions := func(t *testing.T, o *TestOptions) {
		s := optionsToString(o)
		t.Logf("Serialized options:\n%s\n", s)

		parsed := defaultTestOptions()
		require.NoError(t, parseOptions(parsed, s, nil))
		maybeUnref(parsed)
		got := optionsToString(parsed)
		require.Equal(t, s, got)
		t.Logf("Re-serialized options:\n%s\n", got)

		// In some options, the closure obscures the underlying value. Check
		// that the return values are equal.
		require.Equal(t, o.Opts.Experimental.EnableValueBlocks == nil, parsed.Opts.Experimental.EnableValueBlocks == nil)
		if o.Opts.Experimental.EnableValueBlocks != nil {
			require.Equal(t, o.Opts.Experimental.EnableValueBlocks(), parsed.Opts.Experimental.EnableValueBlocks())
		}
		require.Equal(t, o.Opts.Experimental.DisableIngestAsFlushable == nil, parsed.Opts.Experimental.DisableIngestAsFlushable == nil)
		if o.Opts.Experimental.DisableIngestAsFlushable != nil {
			require.Equal(t, o.Opts.Experimental.DisableIngestAsFlushable(), parsed.Opts.Experimental.DisableIngestAsFlushable())
		}
		if o.Opts.Experimental.IngestSplit != nil && o.Opts.Experimental.IngestSplit() {
			require.Equal(t, o.Opts.Experimental.IngestSplit(), parsed.Opts.Experimental.IngestSplit())
		}
		require.Equal(t, o.Opts.MaxConcurrentCompactions(), parsed.Opts.MaxConcurrentCompactions())
		require.Equal(t, len(o.Opts.BlockPropertyCollectors), len(parsed.Opts.BlockPropertyCollectors))

		diff := pretty.Diff(o.Opts, parsed.Opts)
		cleaned := diff[:0]
		for _, d := range diff {
			var ignored bool
			for _, prefix := range ignorePrefixes {
				if strings.HasPrefix(d, prefix) {
					ignored = true
					break
				}
			}
			if !ignored {
				cleaned = append(cleaned, d)
			}
		}
		require.Equal(t, diff[:0], cleaned)
	}

	standard := standardOptions()
	for i := range standard {
		t.Run(fmt.Sprintf("standard-%03d", i), func(t *testing.T) {
			defer maybeUnref(standard[i])
			checkOptions(t, standard[i])
		})
	}
	rng := rand.New(rand.NewSource(uint64(time.Now().UnixNano())))
	for i := 0; i < 100; i++ {
		t.Run(fmt.Sprintf("random-%03d", i), func(t *testing.T) {
			o := randomOptions(rng, nil)
			defer maybeUnref(o)
			checkOptions(t, o)
		})
	}
}

// TestBlockPropertiesParse ensures that the testkeys block property collector
// is in use by default. It runs a single OPTIONS run of the metamorphic tests
// and scans the resulting data directory to ensure there's at least one sstable
// with the property. It runs the test with the archive cleaner to avoid any
// flakiness from small working sets of keys.
func TestBlockPropertiesParse(t *testing.T) {
	const fixedSeed = 1
	const numOps = 10_000
	metaDir := t.TempDir()

	rng := rand.New(rand.NewSource(fixedSeed))
	ops := generate(rng, numOps, presetConfigs[0], newKeyManager())
	opsPath := filepath.Join(metaDir, "ops")
	formattedOps := formatOps(ops)
	require.NoError(t, os.WriteFile(opsPath, []byte(formattedOps), 0644))

	runDir := filepath.Join(metaDir, "run")
	require.NoError(t, os.MkdirAll(runDir, os.ModePerm))
	optionsPath := filepath.Join(runDir, "OPTIONS")
	opts := defaultTestOptions()
	opts.Opts.EnsureDefaults()
	opts.Opts.Cleaner = pebble.ArchiveCleaner{}
	optionsStr := optionsToString(opts)
	require.NoError(t, os.WriteFile(optionsPath, []byte(optionsStr), 0644))

	RunOnce(t, runDir, fixedSeed, filepath.Join(runDir, "history"), KeepData{})
	var foundTableBlockProperty bool
	require.NoError(t, filepath.Walk(filepath.Join(runDir, "data"),
		func(path string, info fs.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if filepath.Ext(path) != ".sst" {
				return nil
			}
			f, err := vfs.Default.Open(path)
			if err != nil {
				return err
			}
			readable, err := sstable.NewSimpleReadable(f)
			if err != nil {
				return err
			}
			r, err := sstable.NewReader(readable, opts.Opts.MakeReaderOptions())
			if err != nil {
				return err
			}
			_, ok := r.Properties.UserProperties[opts.Opts.BlockPropertyCollectors[0]().Name()]
			foundTableBlockProperty = foundTableBlockProperty || ok
			return r.Close()
		}))
	require.True(t, foundTableBlockProperty)
}

func TestCustomOptionParser(t *testing.T) {
	customOptionParsers := map[string]func(string) (CustomOption, bool){
		"foo": func(value string) (CustomOption, bool) {
			return testCustomOption{name: "foo", value: value}, true
		},
	}

	o1 := defaultTestOptions()
	o2 := defaultTestOptions()

	require.NoError(t, parseOptions(o1, `
[TestOptions]
  foo=bar
`, customOptionParsers))
	require.NoError(t, parseOptions(o2, optionsToString(o1), customOptionParsers))
	defer o2.Opts.Cache.Unref()

	for _, o := range []*TestOptions{o1, o2} {
		require.Equal(t, 1, len(o.CustomOpts))
		require.Equal(t, "foo", o.CustomOpts[0].Name())
		require.Equal(t, "bar", o.CustomOpts[0].Value())
	}
}

type testCustomOption struct {
	name, value string
}

func (o testCustomOption) Name() string                { return o.name }
func (o testCustomOption) Value() string               { return o.value }
func (o testCustomOption) Close(*pebble.Options) error { return nil }
func (o testCustomOption) Open(*pebble.Options) error  { return nil }
