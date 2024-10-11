// Copyright 2015 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.
package file_test

import (
	"bufio"
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"os"
	"os/user"
	"path/filepath"
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestPathExpansion(t *testing.T) {
	u, err := user.Current()
	require.NoError(t, err)
	tests := map[string]string{
		"/home/someuser/tmp": "/home/someuser/tmp",
		"~/tmp":              u.HomeDir + "/tmp",
		"$DDDXXX/a/b":        "/tmp/a/b",
		"/a/b/":              "/a/b",
	}
	require.NoError(t, os.Setenv("DDDXXX", "/tmp"))
	for test, expected := range tests {
		expanded, err := file.ExpandPath(test)
		require.NoError(t, err)
		assert.Equal(t, expected, expanded)
	}
}

func TestMkdirAll_AlreadyExists_Override(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, params.BeaconIoConfig().ReadWriteExecutePermissions)
	require.NoError(t, err)
	assert.NoError(t, file.MkdirAll(dirName))
}

func TestHandleBackupDir_AlreadyExists_Override(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	info, err := os.Stat(dirName)
	require.NoError(t, err)
	assert.Equal(t, "drwxr-xr-x", info.Mode().String())
	assert.NoError(t, file.HandleBackupDir(dirName, true))
	info, err = os.Stat(dirName)
	require.NoError(t, err)
	assert.Equal(t, "drwx------", info.Mode().String())
}

func TestHandleBackupDir_AlreadyExists_No_Override(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	info, err := os.Stat(dirName)
	require.NoError(t, err)
	assert.Equal(t, "drwxr-xr-x", info.Mode().String())
	err = file.HandleBackupDir(dirName, false)
	assert.ErrorContains(t, "dir already exists without proper 0700 permissions", err)
	info, err = os.Stat(dirName)
	require.NoError(t, err)
	assert.Equal(t, "drwxr-xr-x", info.Mode().String())
}

func TestHandleBackupDir_NewDir(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	require.NoError(t, file.HandleBackupDir(dirName, true))
	info, err := os.Stat(dirName)
	require.NoError(t, err)
	assert.Equal(t, "drwx------", info.Mode().String())
}

func TestMkdirAll_OK(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := file.MkdirAll(dirName)
	assert.NoError(t, err)
	exists, err := file.HasDir(dirName)
	require.NoError(t, err)
	assert.Equal(t, true, exists)
}

func TestWriteFile_AlreadyExists_WrongPermissions(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, []byte("hi"), os.ModePerm))
	err = file.WriteFile(someFileName, []byte("hi"))
	assert.ErrorContains(t, "already exists without proper 0600 permissions", err)
}

func TestWriteFile_AlreadyExists_OK(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, os.WriteFile(someFileName, []byte("hi"), params.BeaconIoConfig().ReadWritePermissions))
	assert.NoError(t, file.WriteFile(someFileName, []byte("hi")))
}

func TestWriteFile_OK(t *testing.T) {
	dirName := t.TempDir() + "somedir"
	err := os.MkdirAll(dirName, os.ModePerm)
	require.NoError(t, err)
	someFileName := filepath.Join(dirName, "somefile.txt")
	require.NoError(t, file.WriteFile(someFileName, []byte("hi")))
	exists := file.Exists(someFileName)
	assert.Equal(t, true, exists)
}

func TestCopyFile(t *testing.T) {
	fName := t.TempDir() + "testfile"
	err := os.WriteFile(fName, []byte{1, 2, 3}, params.BeaconIoConfig().ReadWritePermissions)
	require.NoError(t, err)

	err = file.CopyFile(fName, fName+"copy")
	assert.NoError(t, err)
	defer func() {
		assert.NoError(t, os.Remove(fName+"copy"))
	}()

	assert.Equal(t, true, deepCompare(t, fName, fName+"copy"))
}

func TestCopyDir(t *testing.T) {
	tmpDir1 := t.TempDir()
	tmpDir2 := filepath.Join(t.TempDir(), "copyfolder")
	type fileDesc struct {
		path    string
		content []byte
	}
	fds := []fileDesc{
		{
			path:    "testfile1",
			content: []byte{1, 2, 3},
		},
		{
			path:    "subfolder1/testfile1",
			content: []byte{4, 5, 6},
		},
		{
			path:    "subfolder1/testfile2",
			content: []byte{7, 8, 9},
		},
		{
			path:    "subfolder2/testfile1",
			content: []byte{10, 11, 12},
		},
		{
			path:    "testfile2",
			content: []byte{13, 14, 15},
		},
	}
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir1, "subfolder1"), 0777))
	require.NoError(t, os.MkdirAll(filepath.Join(tmpDir1, "subfolder2"), 0777))
	for _, fd := range fds {
		require.NoError(t, file.WriteFile(filepath.Join(tmpDir1, fd.path), fd.content))
		assert.Equal(t, true, file.Exists(filepath.Join(tmpDir1, fd.path)))
		assert.Equal(t, false, file.Exists(filepath.Join(tmpDir2, fd.path)))
	}

	// Make sure that files are copied into non-existent directory only. If directory exists function exits.
	assert.ErrorContains(t, "destination directory already exists", file.CopyDir(tmpDir1, t.TempDir()))
	require.NoError(t, file.CopyDir(tmpDir1, tmpDir2))

	// Now, all files should have been copied.
	for _, fd := range fds {
		assert.Equal(t, true, file.Exists(filepath.Join(tmpDir2, fd.path)))
		assert.Equal(t, true, deepCompare(t, filepath.Join(tmpDir1, fd.path), filepath.Join(tmpDir2, fd.path)))
	}
	assert.Equal(t, true, file.DirsEqual(tmpDir1, tmpDir2))
}

func TestDirsEqual(t *testing.T) {
	t.Run("non-existent source directory", func(t *testing.T) {
		assert.Equal(t, false, file.DirsEqual(filepath.Join(t.TempDir(), "nonexistent"), t.TempDir()))
	})

	t.Run("non-existent dest directory", func(t *testing.T) {
		assert.Equal(t, false, file.DirsEqual(t.TempDir(), filepath.Join(t.TempDir(), "nonexistent")))
	})

	t.Run("non-empty directory", func(t *testing.T) {
		// Start with directories that do not have the same contents.
		tmpDir1, tmpFileNames := tmpDirWithContents(t)
		tmpDir2 := filepath.Join(t.TempDir(), "newfolder")
		assert.Equal(t, false, file.DirsEqual(tmpDir1, tmpDir2))

		// Copy dir, and retest (hashes should match now).
		require.NoError(t, file.CopyDir(tmpDir1, tmpDir2))
		assert.Equal(t, true, file.DirsEqual(tmpDir1, tmpDir2))

		// Tamper the data, make sure that hashes do not match anymore.
		require.NoError(t, os.Remove(filepath.Join(tmpDir1, tmpFileNames[2])))
		assert.Equal(t, false, file.DirsEqual(tmpDir1, tmpDir2))
	})
}

func TestHashDir(t *testing.T) {
	t.Run("non-existent directory", func(t *testing.T) {
		hash, err := file.HashDir(filepath.Join(t.TempDir(), "nonexistent"))
		assert.ErrorContains(t, "no such file or directory", err)
		assert.Equal(t, "", hash)
	})

	t.Run("empty directory", func(t *testing.T) {
		hash, err := file.HashDir(t.TempDir())
		assert.NoError(t, err)
		assert.Equal(t, "hashdir:47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=", hash)
	})

	t.Run("non-empty directory", func(t *testing.T) {
		tmpDir, _ := tmpDirWithContents(t)
		hash, err := file.HashDir(tmpDir)
		assert.NoError(t, err)
		assert.Equal(t, "hashdir:oSp9wRacwTIrnbgJWcwTvihHfv4B2zRbLYa0GZ7DDk0=", hash)
	})
}

func TestHashFile(t *testing.T) {
	originalData := []byte("test data")
	originalChecksum := sha256.Sum256(originalData)

	tempDir := t.TempDir()
	tempfile, err := os.CreateTemp(tempDir, "testfile")
	require.NoError(t, err)
	_, err = tempfile.Write(originalData)
	require.NoError(t, err)
	err = tempfile.Close()
	require.NoError(t, err)

	// Calculate the checksum of the temporary file
	checksum, err := file.HashFile(tempfile.Name())
	require.NoError(t, err)

	// Ensure the calculated checksum matches the original checksum
	require.Equal(t, hex.EncodeToString(originalChecksum[:]), hex.EncodeToString(checksum))
}

func TestDirFiles(t *testing.T) {
	tmpDir, tmpDirFnames := tmpDirWithContents(t)
	tests := []struct {
		name     string
		path     string
		outFiles []string
	}{
		{
			name:     "dot path",
			path:     filepath.Join(tmpDir, "/./"),
			outFiles: tmpDirFnames,
		},
		{
			name:     "non-empty folder",
			path:     tmpDir,
			outFiles: tmpDirFnames,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			outFiles, err := file.DirFiles(tt.path)
			require.NoError(t, err)

			sort.Strings(outFiles)
			assert.DeepEqual(t, tt.outFiles, outFiles)
		})
	}
}

func TestRecursiveFileFind(t *testing.T) {
	tmpDir, _ := tmpDirWithContentsForRecursiveFind(t)
	tests := []struct {
		name  string
		root  string
		path  string
		found bool
	}{
		{
			name:  "file1",
			root:  tmpDir,
			path:  "subfolder1/subfolder11/file1",
			found: true,
		},
		{
			name:  "file2",
			root:  tmpDir,
			path:  "subfolder2/file2",
			found: true,
		},
		{
			name:  "file1",
			root:  tmpDir + "/subfolder1",
			path:  "subfolder11/file1",
			found: true,
		},
		{
			name:  "file3",
			root:  tmpDir,
			path:  "file3",
			found: true,
		},
		{
			name:  "file4",
			root:  tmpDir,
			path:  "",
			found: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			found, _, err := file.RecursiveFileFind(tt.name, tt.root)
			require.NoError(t, err)

			assert.DeepEqual(t, tt.found, found)
		})
	}
}

func deepCompare(t *testing.T, file1, file2 string) bool {
	sf, err := os.Open(file1)
	assert.NoError(t, err)
	df, err := os.Open(file2)
	assert.NoError(t, err)
	sscan := bufio.NewScanner(sf)
	dscan := bufio.NewScanner(df)

	for sscan.Scan() && dscan.Scan() {
		if !bytes.Equal(sscan.Bytes(), dscan.Bytes()) {
			return false
		}
	}
	return true
}

// tmpDirWithContents returns path to temporary directory having some folders/files in it.
// Directory is automatically removed by internal testing cleanup methods.
func tmpDirWithContents(t *testing.T) (string, []string) {
	dir := t.TempDir()
	fnames := []string{
		"file1",
		"file2",
		"subfolder1/file1",
		"subfolder1/file2",
		"subfolder1/subfolder11/file1",
		"subfolder1/subfolder11/file2",
		"subfolder1/subfolder12/file1",
		"subfolder1/subfolder12/file2",
		"subfolder2/file1",
	}
	require.NoError(t, os.MkdirAll(filepath.Join(dir, "subfolder1", "subfolder11"), 0777))
	require.NoError(t, os.MkdirAll(filepath.Join(dir, "subfolder1", "subfolder12"), 0777))
	require.NoError(t, os.MkdirAll(filepath.Join(dir, "subfolder2"), 0777))
	for _, fname := range fnames {
		require.NoError(t, os.WriteFile(filepath.Join(dir, fname), []byte(fname), 0777))
	}
	sort.Strings(fnames)
	return dir, fnames
}

// tmpDirWithContentsForRecursiveFind returns path to temporary directory having some folders/files in it.
// Directory is automatically removed by internal testing cleanup methods.
func tmpDirWithContentsForRecursiveFind(t *testing.T) (string, []string) {
	dir := t.TempDir()
	fnames := []string{
		"subfolder1/subfolder11/file1",
		"subfolder2/file2",
		"file3",
	}
	require.NoError(t, os.MkdirAll(filepath.Join(dir, "subfolder1", "subfolder11"), 0777))
	require.NoError(t, os.MkdirAll(filepath.Join(dir, "subfolder2"), 0777))
	for _, fname := range fnames {
		require.NoError(t, os.WriteFile(filepath.Join(dir, fname), []byte(fname), 0777))
	}
	sort.Strings(fnames)
	return dir, fnames
}

func TestHasReadWritePermissions(t *testing.T) {
	type args struct {
		itemPath string
		perms    os.FileMode
	}
	tests := []struct {
		name    string
		args    args
		want    bool
		wantErr bool
	}{
		{
			name: "0600 permissions returns true",
			args: args{
				itemPath: "somefile",
				perms:    params.BeaconIoConfig().ReadWritePermissions,
			},
			want: true,
		},
		{
			name: "other permissions returns false",
			args: args{
				itemPath: "somefile2",
				perms:    params.BeaconIoConfig().ReadWriteExecutePermissions,
			},
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fullPath := filepath.Join(t.TempDir(), tt.args.itemPath)
			require.NoError(t, os.WriteFile(fullPath, []byte("foo"), tt.args.perms))
			got, err := file.HasReadWritePermissions(fullPath)
			if (err != nil) != tt.wantErr {
				t.Errorf("HasReadWritePermissions() error = %v, wantErr %v", err, tt.wantErr)
				return
			}
			if got != tt.want {
				t.Errorf("HasReadWritePermissions() got = %v, want %v", got, tt.want)
			}
		})
	}
}
