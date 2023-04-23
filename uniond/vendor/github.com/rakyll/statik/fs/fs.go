// Copyright 2014 Google Inc. All Rights Reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package fs contains an HTTP file system that works with zip contents.
package fs

import (
	"archive/zip"
	"bytes"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"os"
	"path"
	"path/filepath"
	"sort"
	"strings"
	"time"
)

var zipData = map[string]string{}

// file holds unzipped read-only file contents and file metadata.
type file struct {
	os.FileInfo
	data []byte
	fs   *statikFS
}

type statikFS struct {
	files map[string]file
	dirs  map[string][]string
}

const defaultNamespace = "default"

// IsDefaultNamespace returns true if the assetNamespace is
// the default one
func IsDefaultNamespace(assetNamespace string) bool {
	return assetNamespace == defaultNamespace
}

// Register registers zip contents data, later used to initialize
// the statik file system.
func Register(data string) {
	RegisterWithNamespace(defaultNamespace, data)
}

// RegisterWithNamespace registers zip contents data and set asset namespace,
// later used to initialize the statik file system.
func RegisterWithNamespace(assetNamespace string, data string) {
	zipData[assetNamespace] = data
}

// New creates a new file system with the default registered zip contents data.
// It unzips all files and stores them in an in-memory map.
func New() (http.FileSystem, error) {
	return NewWithNamespace(defaultNamespace)
}

// NewWithNamespace creates a new file system with the registered zip contents data.
// It unzips all files and stores them in an in-memory map.
func NewWithNamespace(assetNamespace string) (http.FileSystem, error) {
	asset, ok := zipData[assetNamespace]
	if !ok {
		return nil, errors.New("statik/fs: no zip data registered")
	}
	zipReader, err := zip.NewReader(strings.NewReader(asset), int64(len(asset)))
	if err != nil {
		return nil, err
	}
	files := make(map[string]file, len(zipReader.File))
	dirs := make(map[string][]string)
	fs := &statikFS{files: files, dirs: dirs}
	for _, zipFile := range zipReader.File {
		fi := zipFile.FileInfo()
		f := file{FileInfo: fi, fs: fs}
		f.data, err = unzip(zipFile)
		if err != nil {
			return nil, fmt.Errorf("statik/fs: error unzipping file %q: %s", zipFile.Name, err)
		}
		files["/"+zipFile.Name] = f
	}
	for fn := range files {
		// go up directories recursively in order to care deep directory
		for dn := path.Dir(fn); dn != fn; {
			if _, ok := files[dn]; !ok {
				files[dn] = file{FileInfo: dirInfo{dn}, fs: fs}
			} else {
				break
			}
			fn, dn = dn, path.Dir(dn)
		}
	}
	for fn := range files {
		dn := path.Dir(fn)
		if fn != dn {
			fs.dirs[dn] = append(fs.dirs[dn], path.Base(fn))
		}
	}
	for _, s := range fs.dirs {
		sort.Strings(s)
	}
	return fs, nil
}

var _ = os.FileInfo(dirInfo{})

type dirInfo struct {
	name string
}

func (di dirInfo) Name() string       { return path.Base(di.name) }
func (di dirInfo) Size() int64        { return 0 }
func (di dirInfo) Mode() os.FileMode  { return 0755 | os.ModeDir }
func (di dirInfo) ModTime() time.Time { return time.Time{} }
func (di dirInfo) IsDir() bool        { return true }
func (di dirInfo) Sys() interface{}   { return nil }

// Open returns a file matching the given file name, or os.ErrNotExists if
// no file matching the given file name is found in the archive.
// If a directory is requested, Open returns the file named "index.html"
// in the requested directory, if that file exists.
func (fs *statikFS) Open(name string) (http.File, error) {
	name = filepath.ToSlash(filepath.Clean(name))
	if f, ok := fs.files[name]; ok {
		return newHTTPFile(f), nil
	}
	return nil, os.ErrNotExist
}

func newHTTPFile(file file) *httpFile {
	if file.IsDir() {
		return &httpFile{file: file, isDir: true}
	}
	return &httpFile{file: file, reader: bytes.NewReader(file.data)}
}

// httpFile represents an HTTP file and acts as a bridge
// between file and http.File.
type httpFile struct {
	file

	reader *bytes.Reader
	isDir  bool
	dirIdx int
}

// Read reads bytes into p, returns the number of read bytes.
func (f *httpFile) Read(p []byte) (n int, err error) {
	if f.reader == nil && f.isDir {
		return 0, io.EOF
	}
	return f.reader.Read(p)
}

// Seek seeks to the offset.
func (f *httpFile) Seek(offset int64, whence int) (ret int64, err error) {
	return f.reader.Seek(offset, whence)
}

// Stat stats the file.
func (f *httpFile) Stat() (os.FileInfo, error) {
	return f, nil
}

// IsDir returns true if the file location represents a directory.
func (f *httpFile) IsDir() bool {
	return f.isDir
}

// Readdir returns an empty slice of files, directory
// listing is disabled.
func (f *httpFile) Readdir(count int) ([]os.FileInfo, error) {
	var fis []os.FileInfo
	if !f.isDir {
		return fis, nil
	}
	di, ok := f.FileInfo.(dirInfo)
	if !ok {
		return nil, fmt.Errorf("failed to read directory: %q", f.Name())
	}

	// If count is positive, the specified number of files will be returned,
	// and if non-positive, all remaining files will be returned.
	// The reading position of which file is returned is held in dirIndex.
	fnames := f.file.fs.dirs[di.name]
	flen := len(fnames)

	// If dirIdx reaches the end and the count is a positive value,
	// an io.EOF error is returned.
	// In other cases, no error will be returned even if, for example,
	// you specified more counts than the number of remaining files.
	start := f.dirIdx
	if start >= flen && count > 0 {
		return fis, io.EOF
	}
	var end int
	if count <= 0 {
		end = flen
	} else {
		end = start + count
	}
	if end > flen {
		end = flen
	}
	for i := start; i < end; i++ {
		fis = append(fis, f.file.fs.files[path.Join(di.name, fnames[i])].FileInfo)
	}
	f.dirIdx += len(fis)
	return fis, nil
}

func (f *httpFile) Close() error {
	return nil
}

func unzip(zf *zip.File) ([]byte, error) {
	rc, err := zf.Open()
	if err != nil {
		return nil, err
	}
	defer rc.Close()
	return ioutil.ReadAll(rc)
}
