// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package sstable

import (
	"bytes"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
)

const (
	noPrefixFilter = false
	prefixFilter   = true

	noFullKeyBloom = false
	fullKeyBloom   = true

	defaultIndexBlockSize = math.MaxInt32
	smallIndexBlockSize   = 128
)

type keyCountPropertyCollector struct {
	count int
}

func (c *keyCountPropertyCollector) Add(key InternalKey, value []byte) error {
	c.count++
	return nil
}

func (c *keyCountPropertyCollector) Finish(userProps map[string]string) error {
	userProps["test.key-count"] = fmt.Sprint(c.count)
	return nil
}

func (c *keyCountPropertyCollector) Name() string {
	return "KeyCountPropertyCollector"
}

var fixtureComparer = func() *Comparer {
	c := *base.DefaultComparer
	// NB: this is named as such only to match the built-in RocksDB comparer.
	c.Name = "leveldb.BytewiseComparator"
	c.Split = func(a []byte) int {
		// TODO(tbg): this matches logic in testdata/make-table.cc. It's
		// difficult to provide a more meaningful prefix extractor on the given
		// dataset since it's not MVCC, and so it's impossible to come up with a
		// sensible one. We need to add a better dataset and use that instead to
		// get confidence that prefix extractors are working as intended.
		return len(a)
	}
	return &c
}()

type fixtureOpts struct {
	compression    Compression
	fullKeyFilter  bool
	prefixFilter   bool
	indexBlockSize int
}

func (o fixtureOpts) String() string {
	return fmt.Sprintf(
		"compression=%s,fullKeyFilter=%t,prefixFilter=%t",
		o.compression, o.fullKeyFilter, o.prefixFilter,
	)
}

var fixtures = map[fixtureOpts]struct {
	filename      string
	comparer      *Comparer
	propCollector func() TablePropertyCollector
}{
	{SnappyCompression, noFullKeyBloom, noPrefixFilter, defaultIndexBlockSize}: {
		"testdata/h.sst", nil,
		func() TablePropertyCollector {
			return &keyCountPropertyCollector{}
		},
	},
	{SnappyCompression, fullKeyBloom, noPrefixFilter, defaultIndexBlockSize}: {
		"testdata/h.table-bloom.sst", nil, nil,
	},
	{NoCompression, noFullKeyBloom, noPrefixFilter, defaultIndexBlockSize}: {
		"testdata/h.no-compression.sst", nil,
		func() TablePropertyCollector {
			return &keyCountPropertyCollector{}
		},
	},
	{NoCompression, fullKeyBloom, noPrefixFilter, defaultIndexBlockSize}: {
		"testdata/h.table-bloom.no-compression.sst", nil, nil,
	},
	{NoCompression, noFullKeyBloom, prefixFilter, defaultIndexBlockSize}: {
		"testdata/h.table-bloom.no-compression.prefix_extractor.no_whole_key_filter.sst",
		fixtureComparer, nil,
	},
	{NoCompression, noFullKeyBloom, noPrefixFilter, smallIndexBlockSize}: {
		"testdata/h.no-compression.two_level_index.sst", nil,
		func() TablePropertyCollector {
			return &keyCountPropertyCollector{}
		},
	},
	{ZstdCompression, noFullKeyBloom, noPrefixFilter, defaultIndexBlockSize}: {
		"testdata/h.zstd-compression.sst", nil,
		func() TablePropertyCollector {
			return &keyCountPropertyCollector{}
		},
	},
}

func runTestFixtureOutput(opts fixtureOpts) error {
	fixture, ok := fixtures[opts]
	if !ok {
		return errors.Errorf("fixture missing: %+v", opts)
	}

	compression := opts.compression

	var fp base.FilterPolicy
	if opts.fullKeyFilter || opts.prefixFilter {
		fp = bloom.FilterPolicy(10)
	}
	ftype := base.TableFilter

	// Check that a freshly made table is byte-for-byte equal to a pre-made
	// table.
	want, err := os.ReadFile(filepath.FromSlash(fixture.filename))
	if err != nil {
		return err
	}

	f, err := build(compression, fp, ftype, fixture.comparer, fixture.propCollector, 2048, opts.indexBlockSize)
	if err != nil {
		return err
	}
	stat, err := f.Stat()
	if err != nil {
		return err
	}
	got := make([]byte, stat.Size())
	_, err = f.ReadAt(got, 0)
	if err != nil {
		return err
	}

	if !bytes.Equal(got, want) {
		i := 0
		for ; i < len(got) && i < len(want) && got[i] == want[i]; i++ {
		}
		os.WriteFile("fail.txt", got, 0644)
		return errors.Errorf("built table %s does not match pre-made table. From byte %d onwards,\ngot:\n% x\nwant:\n% x",
			fixture.filename, i, got[i:], want[i:])
	}
	return nil
}

func TestFixtureOutput(t *testing.T) {
	for opt := range fixtures {
		// Note: we disabled the zstd fixture test when CGO_ENABLED=0, because the
		// implementation between DataDog/zstd and klauspost/compress are
		// different, which leads to different compression output
		// <https://github.com/klauspost/compress/issues/109#issuecomment-498763233>.
		// Since the fixture test requires bit-to-bit reproducibility, we cannot
		// run the zstd test when the implementation is not based on facebook/zstd.
		if !useStandardZstdLib && opt.compression == ZstdCompression {
			continue
		}
		t.Run(opt.String(), func(t *testing.T) {
			require.NoError(t, runTestFixtureOutput(opt))
		})
	}
}
