package util

import (
	"os"
	"path"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/pkg/errors"
)

// BazelDirectoryNonEmpty returns true if directory exists and is not empty.
func BazelDirectoryNonEmpty(filePath string) (bool, error) {
	fs, err := bazelReadDir(filePath)
	if err != nil {
		return false, err
	}
	return len(fs) > 0, nil
}

// BazelFileBytes returns the byte array of the bazel file path given.
func BazelFileBytes(filePaths ...string) ([]byte, error) {
	filepath, err := bazel.Runfile(path.Join(filePaths...))
	if err != nil {
		return nil, err
	}
	fileBytes, err := os.ReadFile(filepath) // #nosec G304
	if err != nil {
		return nil, err
	}
	if len(fileBytes) == 0 {
		return nil, errors.New("empty file")
	}
	return fileBytes, nil
}

// BazelListFiles lists all of the file names in a given directory. Excludes directories. Returns
// an error when no non-directory files exist.
func BazelListFiles(filepath string) ([]string, error) {
	d, err := bazelReadDir(filepath)
	if err != nil {
		return nil, err
	}

	ret := make([]string, 0, len(d))

	for _, f := range d {
		if f.IsDir() {
			continue
		}
		ret = append(ret, f.Name())
	}

	if len(ret) == 0 {
		return nil, errors.New("no files found")
	}

	return ret, nil
}

// BazelListDirectories lists all of the directories in the given directory. Excludes regular files.
// Returns error when no directories exist.
func BazelListDirectories(filepath string) ([]string, error) {
	d, err := bazelReadDir(filepath)
	if err != nil {
		return nil, err
	}
	ret := make([]string, 0)
	for _, f := range d {
		if f.IsDir() {
			ret = append(ret, f.Name())
		}
	}

	if len(ret) == 0 {
		return nil, errors.New("no directories found")
	}

	return ret, nil
}

func bazelReadDir(filepath string) ([]os.DirEntry, error) {
	p, err := bazel.Runfile(filepath)
	if err != nil {
		return nil, err
	}
	return os.ReadDir(p)
}
