// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package main

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/require"
)

const (
	dataDirPath          = "./testdata/data"
	dataSymlinkedDirPath = "./testdata/data-symlink"
	dataJSPath           = "./testdata/data.js"
)

var dataDirPaths = []string{dataDirPath, dataSymlinkedDirPath}

func TestParseYCSB_FromScratch(t *testing.T) {
	maybeSkip(t)

	testFn := func(t *testing.T, dataDir string) {
		// Write out a new data.js file from the input data.
		fPath := filepath.Join(t.TempDir(), "data.js")
		parseYCSB(dataDir, fPath, fPath)

		// Confirm the two data.js files are now equal.
		err := filesEqual(dataJSPath, fPath)
		require.NoError(t, err)
	}

	for _, dir := range dataDirPaths {
		t.Run(dir, func(t *testing.T) {
			testFn(t, dir)
		})
	}
}

func TestYCSB_Existing(t *testing.T) {
	maybeSkip(t)

	testFn := func(t *testing.T, dataDir string) {
		// Set up the test directory.
		testDir := t.TempDir()
		newDataDir := filepath.Join(testDir, "data")
		newDataJS := filepath.Join(testDir, "data.js")

		// Copy all files into the test dir excluding one day.
		err := copyDir(dataDir, newDataDir)
		require.NoError(t, err)
		err = os.RemoveAll(filepath.Join(newDataDir, "20211027"))
		require.NoError(t, err)

		// Construct the data.js file on the test data with a single day removed.
		parseYCSB(newDataDir, newDataJS, newDataJS)

		// Confirm the two data.js files are not equal.
		err = filesEqual(dataJSPath, newDataJS)
		require.Error(t, err)

		// Re-construct the data.js file with the full set of data.
		parseYCSB(dataDir, dataJSPath, newDataJS)

		// Confirm the two data.js files are now equal.
		err = filesEqual(dataJSPath, newDataJS)
		require.NoError(t, err)
	}

	for _, dir := range dataDirPaths {
		t.Run(dir, func(t *testing.T) {
			testFn(t, dir)
		})
	}
}
