package utils

import (
	"os"
	"path"
	"strings"
	"testing"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func RetrieveFiles(name string, t *testing.T) ([]string, [][]byte) {
	filepath, err := bazel.Runfile(name)
	require.NoError(t, err)
	testFiles, err := os.ReadDir(filepath)
	require.NoError(t, err)

	fileNames := make([]string, 0, len(testFiles))
	fileContent := make([][]byte, 0, len(testFiles))
	require.Equal(t, false, len(testFiles) == 0, "no files exist in directory")
	for _, f := range testFiles {
		// Remove .yml suffix
		fName := strings.TrimSuffix(f.Name(), ".yaml")
		fileNames = append(fileNames, fName)
		data, err := file.ReadFileAsBytes(path.Join(filepath, f.Name()))
		require.NoError(t, err)
		fileContent = append(fileContent, data)
	}
	return fileNames, fileContent
}
