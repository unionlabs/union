// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWalkDir(t *testing.T) {
	maybeSkip(t)

	// Number of files and directories in the data fixture directory (and
	// therefore the symlinked directory). Generated via:
	//   find . testdata | wc -l
	const wantCount = 97
	for _, path := range dataDirPaths {
		t.Run(path, func(t *testing.T) {
			var paths []string
			err := walkDir(path, func(_, pathRel string, info os.FileInfo) error {
				paths = append(paths, pathRel)
				return nil
			})
			require.NoError(t, err)
			require.Equal(t, wantCount, len(paths))
		})
	}
}
