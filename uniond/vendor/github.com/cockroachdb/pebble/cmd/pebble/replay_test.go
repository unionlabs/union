// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"fmt"
	"testing"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/stretchr/testify/require"
)

func TestParseOptionsStr(t *testing.T) {
	type testCase struct {
		c       replayConfig
		options *pebble.Options
	}

	testCases := []testCase{
		{
			c:       replayConfig{optionsString: `[Options] max_concurrent_compactions=9`},
			options: &pebble.Options{MaxConcurrentCompactions: func() int { return 9 }},
		},
		{
			c:       replayConfig{optionsString: `[Options] bytes_per_sync=90000`},
			options: &pebble.Options{BytesPerSync: 90000},
		},
		{
			c:       replayConfig{optionsString: fmt.Sprintf(`[Options] cache_size=%d`, 16<<20 /* 16MB */)},
			options: &pebble.Options{Cache: cache.New(16 << 20 /* 16 MB */)},
		},
		{
			c: replayConfig{
				maxCacheSize:  16 << 20, /* 16 MB */
				optionsString: fmt.Sprintf(`[Options] cache_size=%d`, int64(10<<30 /* 10 GB */)),
			},
			options: &pebble.Options{Cache: cache.New(16 << 20 /* 16 MB */)},
		},
		{
			c: replayConfig{optionsString: `[Options] [Level "0"] target_file_size=222`},
			options: &pebble.Options{Levels: []pebble.LevelOptions{
				{TargetFileSize: 222},
			}},
		},
		{
			c: replayConfig{optionsString: `[Options] lbase_max_bytes=10  max_open_files=20  [Level "0"] target_file_size=30 [Level "1"] index_block_size=40`},
			options: &pebble.Options{
				LBaseMaxBytes: 10,
				MaxOpenFiles:  20,
				Levels: []pebble.LevelOptions{
					{TargetFileSize: 30},
					{IndexBlockSize: 40},
				},
			},
		},
	}

	for _, tc := range testCases {
		o := new(pebble.Options)
		require.NoError(t, tc.c.parseCustomOptions(tc.c.optionsString, o))
		o.EnsureDefaults()
		got := o.String()

		tc.options.EnsureDefaults()
		want := tc.options.String()
		require.Equal(t, want, got)
		if o.Cache != nil {
			o.Cache.Unref()
		}
		if tc.options.Cache != nil {
			tc.options.Cache.Unref()
		}
	}
}
