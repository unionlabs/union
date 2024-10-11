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
	"context"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"syscall"
	"time"

	"github.com/googleapis/google-cloud-go-testing/storage/stiface"
)

const (
	defaultFileMode = 0o755
	gsPrefix        = "gs://"
)

// Fs is a Fs implementation that uses functions provided by google cloud storage
type Fs struct {
	ctx       context.Context
	client    stiface.Client
	separator string

	buckets       map[string]stiface.BucketHandle
	rawGcsObjects map[string]*GcsFile

	autoRemoveEmptyFolders bool // trigger for creating "virtual folders" (not required by GCSs)
}

func NewGcsFs(ctx context.Context, client stiface.Client) *Fs {
	return NewGcsFsWithSeparator(ctx, client, "/")
}

func NewGcsFsWithSeparator(ctx context.Context, client stiface.Client, folderSep string) *Fs {
	return &Fs{
		ctx:           ctx,
		client:        client,
		separator:     folderSep,
		rawGcsObjects: make(map[string]*GcsFile),

		autoRemoveEmptyFolders: true,
	}
}

// normSeparators will normalize all "\\" and "/" to the provided separator
func (fs *Fs) normSeparators(s string) string {
	return strings.Replace(strings.Replace(s, "\\", fs.separator, -1), "/", fs.separator, -1)
}

func (fs *Fs) ensureTrailingSeparator(s string) string {
	if len(s) > 0 && !strings.HasSuffix(s, fs.separator) {
		return s + fs.separator
	}
	return s
}

func (fs *Fs) ensureNoLeadingSeparator(s string) string {
	if len(s) > 0 && strings.HasPrefix(s, fs.separator) {
		s = s[len(fs.separator):]
	}

	return s
}

func ensureNoPrefix(s string) string {
	if len(s) > 0 && strings.HasPrefix(s, gsPrefix) {
		return s[len(gsPrefix):]
	}
	return s
}

func validateName(s string) error {
	if len(s) == 0 {
		return ErrNoBucketInName
	}
	return nil
}

// Splits provided name into bucket name and path
func (fs *Fs) splitName(name string) (bucketName string, path string) {
	splitName := strings.Split(name, fs.separator)

	return splitName[0], strings.Join(splitName[1:], fs.separator)
}

func (fs *Fs) getBucket(name string) (stiface.BucketHandle, error) {
	bucket := fs.buckets[name]
	if bucket == nil {
		bucket = fs.client.Bucket(name)
		_, err := bucket.Attrs(fs.ctx)
		if err != nil {
			return nil, err
		}
	}
	return bucket, nil
}

func (fs *Fs) getObj(name string) (stiface.ObjectHandle, error) {
	bucketName, path := fs.splitName(name)

	bucket, err := fs.getBucket(bucketName)
	if err != nil {
		return nil, err
	}

	return bucket.Object(path), nil
}

func (fs *Fs) Name() string { return "GcsFs" }

func (fs *Fs) Create(name string) (*GcsFile, error) {
	name = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(name)))
	if err := validateName(name); err != nil {
		return nil, err
	}

	if !fs.autoRemoveEmptyFolders {
		baseDir := filepath.Base(name)
		if stat, err := fs.Stat(baseDir); err != nil || !stat.IsDir() {
			err = fs.MkdirAll(baseDir, 0)
			if err != nil {
				return nil, err
			}
		}
	}

	obj, err := fs.getObj(name)
	if err != nil {
		return nil, err
	}
	w := obj.NewWriter(fs.ctx)
	err = w.Close()
	if err != nil {
		return nil, err
	}
	file := NewGcsFile(fs.ctx, fs, obj, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0, name)

	fs.rawGcsObjects[name] = file
	return file, nil
}

func (fs *Fs) Mkdir(name string, _ os.FileMode) error {
	name = fs.ensureNoLeadingSeparator(fs.ensureTrailingSeparator(fs.normSeparators(ensureNoPrefix(name))))
	if err := validateName(name); err != nil {
		return err
	}
	// folder creation logic has to additionally check for folder name presence
	bucketName, path := fs.splitName(name)
	if bucketName == "" {
		return ErrNoBucketInName
	}
	if path == "" {
		// the API would throw "googleapi: Error 400: No object name, required", but this one is more consistent
		return ErrEmptyObjectName
	}

	obj, err := fs.getObj(name)
	if err != nil {
		return err
	}
	w := obj.NewWriter(fs.ctx)
	return w.Close()
}

func (fs *Fs) MkdirAll(path string, perm os.FileMode) error {
	path = fs.ensureNoLeadingSeparator(fs.ensureTrailingSeparator(fs.normSeparators(ensureNoPrefix(path))))
	if err := validateName(path); err != nil {
		return err
	}
	// folder creation logic has to additionally check for folder name presence
	bucketName, splitPath := fs.splitName(path)
	if bucketName == "" {
		return ErrNoBucketInName
	}
	if splitPath == "" {
		// the API would throw "googleapi: Error 400: No object name, required", but this one is more consistent
		return ErrEmptyObjectName
	}

	root := ""
	folders := strings.Split(path, fs.separator)
	for i, f := range folders {
		if f == "" && i != 0 {
			continue // it's the last item - it should be empty
		}
		// Don't force a delimiter prefix
		if root != "" {
			root = root + fs.separator + f
		} else {
			// we have to have at least bucket name + folder name to create successfully
			root = f
			continue
		}

		if err := fs.Mkdir(root, perm); err != nil {
			return err
		}
	}
	return nil
}

func (fs *Fs) Open(name string) (*GcsFile, error) {
	return fs.OpenFile(name, os.O_RDONLY, 0)
}

func (fs *Fs) OpenFile(name string, flag int, fileMode os.FileMode) (*GcsFile, error) {
	var file *GcsFile
	var err error

	name = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(name)))
	if err = validateName(name); err != nil {
		return nil, err
	}

	f, found := fs.rawGcsObjects[name]
	if found {
		file = NewGcsFileFromOldFH(flag, fileMode, f.resource)
	} else {
		var obj stiface.ObjectHandle
		obj, err = fs.getObj(name)
		if err != nil {
			return nil, err
		}
		file = NewGcsFile(fs.ctx, fs, obj, flag, fileMode, name)
	}

	if flag == os.O_RDONLY {
		_, err = file.Stat()
		if err != nil {
			return nil, err
		}
	}

	if flag&os.O_TRUNC != 0 {
		err = file.resource.obj.Delete(fs.ctx)
		if err != nil {
			return nil, err
		}
		return fs.Create(name)
	}

	if flag&os.O_APPEND != 0 {
		_, err = file.Seek(0, 2)
		if err != nil {
			return nil, err
		}
	}

	if flag&os.O_CREATE != 0 {
		_, err = file.Stat()
		if err == nil { // the file actually exists
			return nil, syscall.EPERM
		}

		_, err = file.WriteString("")
		if err != nil {
			return nil, err
		}
	}
	return file, nil
}

func (fs *Fs) Remove(name string) error {
	name = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(name)))
	if err := validateName(name); err != nil {
		return err
	}

	obj, err := fs.getObj(name)
	if err != nil {
		return err
	}
	info, err := fs.Stat(name)
	if err != nil {
		return err
	}
	delete(fs.rawGcsObjects, name)

	if info.IsDir() {
		// it's a folder, we ha to check its contents - it cannot be removed, if not empty
		var dir *GcsFile
		dir, err = fs.Open(name)
		if err != nil {
			return err
		}
		var infos []os.FileInfo
		infos, err = dir.Readdir(0)
		if err != nil {
			return err
		}
		if len(infos) > 0 {
			return syscall.ENOTEMPTY
		}

		// it's an empty folder, we can continue
		name = fs.ensureTrailingSeparator(name)
		obj, err = fs.getObj(name)
		if err != nil {
			return err
		}

		return obj.Delete(fs.ctx)
	}
	return obj.Delete(fs.ctx)
}

func (fs *Fs) RemoveAll(path string) error {
	path = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(path)))
	if err := validateName(path); err != nil {
		return err
	}

	pathInfo, err := fs.Stat(path)
	if errors.Is(err, ErrFileNotFound) {
		// return early if file doesn't exist
		return nil
	}
	if err != nil {
		return err
	}

	if !pathInfo.IsDir() {
		return fs.Remove(path)
	}

	var dir *GcsFile
	dir, err = fs.Open(path)
	if err != nil {
		return err
	}

	var infos []os.FileInfo
	infos, err = dir.Readdir(0)
	if err != nil {
		return err
	}
	for _, info := range infos {
		nameToRemove := fs.normSeparators(info.Name())
		err = fs.RemoveAll(path + fs.separator + nameToRemove)
		if err != nil {
			return err
		}
	}

	return fs.Remove(path)
}

func (fs *Fs) Rename(oldName, newName string) error {
	oldName = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(oldName)))
	if err := validateName(oldName); err != nil {
		return err
	}

	newName = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(newName)))
	if err := validateName(newName); err != nil {
		return err
	}

	src, err := fs.getObj(oldName)
	if err != nil {
		return err
	}
	dst, err := fs.getObj(newName)
	if err != nil {
		return err
	}

	if _, err = dst.CopierFrom(src).Run(fs.ctx); err != nil {
		return err
	}
	delete(fs.rawGcsObjects, oldName)
	return src.Delete(fs.ctx)
}

func (fs *Fs) Stat(name string) (os.FileInfo, error) {
	name = fs.ensureNoLeadingSeparator(fs.normSeparators(ensureNoPrefix(name)))
	if err := validateName(name); err != nil {
		return nil, err
	}

	return newFileInfo(name, fs, defaultFileMode)
}

func (fs *Fs) Chmod(_ string, _ os.FileMode) error {
	return errors.New("method Chmod is not implemented in GCS")
}

func (fs *Fs) Chtimes(_ string, _, _ time.Time) error {
	return errors.New("method Chtimes is not implemented. Create, Delete, Updated times are read only fields in GCS and set implicitly")
}

func (fs *Fs) Chown(_ string, _, _ int) error {
	return errors.New("method Chown is not implemented for GCS")
}
