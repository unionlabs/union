// Copyright © 2021 Vasily Ovchinnikov <vasily@remerge.io>.
//
// The code in this file is derived from afero fork github.com/Zatte/afero by Mikael Rapp
// licensed under Apache License 2.0.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package gcsfs

import (
	"os"
	"path/filepath"
	"strings"
	"time"

	"cloud.google.com/go/storage"
)

const (
	folderSize = 42
)

type FileInfo struct {
	name     string
	size     int64
	updated  time.Time
	isDir    bool
	fileMode os.FileMode
}

func newFileInfo(name string, fs *Fs, fileMode os.FileMode) (*FileInfo, error) {
	res := &FileInfo{
		name:     name,
		size:     folderSize,
		updated:  time.Time{},
		isDir:    false,
		fileMode: fileMode,
	}

	obj, err := fs.getObj(name)
	if err != nil {
		return nil, err
	}

	objAttrs, err := obj.Attrs(fs.ctx)
	if err != nil {
		if err.Error() == ErrEmptyObjectName.Error() {
			// It's a root folder here, we return right away
			res.name = fs.ensureTrailingSeparator(res.name)
			res.isDir = true
			return res, nil
		} else if err.Error() == ErrObjectDoesNotExist.Error() {
			// Folders do not actually "exist" in GCloud, so we have to check, if something exists with
			// such a prefix
			bucketName, bucketPath := fs.splitName(name)
			it := fs.client.Bucket(bucketName).Objects(
				fs.ctx, &storage.Query{Delimiter: fs.separator, Prefix: bucketPath, Versions: false})
			if _, err = it.Next(); err == nil {
				res.name = fs.ensureTrailingSeparator(res.name)
				res.isDir = true
				return res, nil
			}

			return nil, ErrFileNotFound
		}
		return nil, err
	}

	res.size = objAttrs.Size
	res.updated = objAttrs.Updated

	return res, nil
}

func newFileInfoFromAttrs(objAttrs *storage.ObjectAttrs, fileMode os.FileMode) *FileInfo {
	res := &FileInfo{
		name:     objAttrs.Name,
		size:     objAttrs.Size,
		updated:  objAttrs.Updated,
		isDir:    false,
		fileMode: fileMode,
	}

	if res.name == "" {
		if objAttrs.Prefix != "" {
			// It's a virtual folder! It does not have a name, but prefix - this is how GCS API
			// deals with them at the moment
			res.name = objAttrs.Prefix
			res.size = folderSize
			res.isDir = true
		}
	}

	return res
}

func (fi *FileInfo) Name() string {
	return filepath.Base(filepath.FromSlash(fi.name))
}

func (fi *FileInfo) Size() int64 {
	return fi.size
}

func (fi *FileInfo) Mode() os.FileMode {
	if fi.IsDir() {
		return os.ModeDir | fi.fileMode
	}
	return fi.fileMode
}

func (fi *FileInfo) ModTime() time.Time {
	return fi.updated
}

func (fi *FileInfo) IsDir() bool {
	return fi.isDir
}

func (fi *FileInfo) Sys() interface{} {
	return nil
}

type ByName []*FileInfo

func (a ByName) Len() int { return len(a) }
func (a ByName) Swap(i, j int) {
	a[i].name, a[j].name = a[j].name, a[i].name
	a[i].size, a[j].size = a[j].size, a[i].size
	a[i].updated, a[j].updated = a[j].updated, a[i].updated
	a[i].isDir, a[j].isDir = a[j].isDir, a[i].isDir
}
func (a ByName) Less(i, j int) bool { return strings.Compare(a[i].Name(), a[j].Name()) == -1 }
