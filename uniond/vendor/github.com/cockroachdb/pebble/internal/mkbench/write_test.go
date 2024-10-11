package main

import (
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

const testdataSummaryDir = "./testdata/write-throughput/"

var (
	testdataSummaryFile            = filepath.Join(testdataSummaryDir, summaryFilename)
	testdataPerRunSummaryFilenames = []string{
		"20211027-pebble-write-size=1024-run_1-summary.json",
		"20211028-pebble-write-size=1024-run_1-summary.json",
	}
)

func TestParseWrite_FromScratch(t *testing.T) {
	maybeSkip(t)

	testFn := func(t *testing.T, dataDir string) {
		// Set up the output directory for the test.
		testDir := t.TempDir()
		summaryDir := filepath.Join(testDir, "write-throughput")
		err := os.Mkdir(summaryDir, 0700)
		require.NoError(t, err)

		// Write out the new summary file.
		err = parseWrite(dataDir, summaryDir)
		require.NoError(t, err)

		// Confirm new summary.json file matches what we expect.
		err = filesEqual(testdataSummaryFile, filepath.Join(summaryDir, summaryFilename))
		require.NoError(t, err)

		// The individual per-run *summary.json files are equal.
		for _, p := range testdataPerRunSummaryFilenames {
			err = filesEqual(filepath.Join(testdataSummaryDir, p), filepath.Join(summaryDir, p))
			require.NoError(t, err)
		}
	}

	for _, dir := range dataDirPaths {
		t.Run(dir, func(t *testing.T) {
			testFn(t, dir)
		})
	}
}

func TestParseWrite_Existing(t *testing.T) {
	maybeSkip(t)

	testFn := func(t *testing.T, dataDir string) {
		// Set up the output directory for the test.
		testDir := t.TempDir()
		summaryDir := filepath.Join(testDir, "write-throughput")
		err := os.Mkdir(summaryDir, 0700)
		require.NoError(t, err)

		// Copy all files into the test dir excluding one day.
		newDataDir := filepath.Join(testDir, "data")
		err = copyDir(dataDir, newDataDir)
		require.NoError(t, err)
		err = os.RemoveAll(filepath.Join(newDataDir, "20211027"))
		require.NoError(t, err)

		// Write out the new summary file.
		err = parseWrite(newDataDir, summaryDir)
		require.NoError(t, err)

		// Confirm new summary.json files are NOT equal.
		err = filesEqual(testdataSummaryFile, filepath.Join(summaryDir, summaryFilename))
		require.Error(t, err)

		// The only per-run *summary.json files are for the days we did not remove
		// (i.e. 20211028/**-summary.json)
		var perRunFiles []string
		err = filepath.Walk(summaryDir, func(path string, info os.FileInfo, err error) error {
			basename := filepath.Base(path)
			if strings.HasSuffix(basename, "-summary.json") {
				perRunFiles = append(perRunFiles, basename)
			}
			return nil
		})
		require.NoError(t, err)
		require.Equal(t, perRunFiles, testdataPerRunSummaryFilenames[1:])

		// Re-construct the summary.json file with the full set of data (i.e. the
		// original data directory).
		err = parseWrite(dataDir, summaryDir)
		require.NoError(t, err)

		// Confirm the two summary.json files are now equal.
		err = filesEqual(testdataSummaryFile, filepath.Join(summaryDir, summaryFilename))
		require.NoError(t, err)

		// The individual per-run *summary.json files are equal.
		for _, p := range testdataPerRunSummaryFilenames {
			err = filesEqual(filepath.Join(testdataSummaryDir, p), filepath.Join(summaryDir, p))
			require.NoError(t, err)
		}
	}

	for _, dir := range dataDirPaths {
		t.Run(dir, func(t *testing.T) {
			testFn(t, dir)
		})
	}
}
