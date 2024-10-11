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
	"fmt"
	"io"
	"log"
	"os"
	"path/filepath"
	"sort"
	"syscall"

	"github.com/googleapis/google-cloud-go-testing/storage/stiface"

	"cloud.google.com/go/storage"

	"google.golang.org/api/iterator"
)

// GcsFs is the Afero version adapted for GCS
type GcsFile struct {
	openFlags int
	fhOffset  int64 // File handle specific offset
	closed    bool
	ReadDirIt stiface.ObjectIterator
	resource  *gcsFileResource
}

func NewGcsFile(
	ctx context.Context,
	fs *Fs,
	obj stiface.ObjectHandle,
	openFlags int,
	// Unused: there is no use to the file mode in GCloud just yet - but we keep it here, just in case we need it
	fileMode os.FileMode,
	name string,
) *GcsFile {
	return &GcsFile{
		openFlags: openFlags,
		fhOffset:  0,
		closed:    false,
		ReadDirIt: nil,
		resource: &gcsFileResource{
			ctx: ctx,
			fs:  fs,

			obj:      obj,
			name:     name,
			fileMode: fileMode,

			currentGcsSize: 0,

			offset: 0,
			reader: nil,
			writer: nil,
		},
	}
}

func NewGcsFileFromOldFH(
	openFlags int,
	fileMode os.FileMode,
	oldFile *gcsFileResource,
) *GcsFile {
	res := &GcsFile{
		openFlags: openFlags,
		fhOffset:  0,
		closed:    false,
		ReadDirIt: nil,

		resource: oldFile,
	}
	res.resource.fileMode = fileMode

	return res
}

func (o *GcsFile) Close() error {
	if o.closed {
		// the afero spec expects the call to Close on a closed file to return an error
		return ErrFileClosed
	}
	o.closed = true
	return o.resource.Close()
}

func (o *GcsFile) Seek(newOffset int64, whence int) (int64, error) {
	if o.closed {
		return 0, ErrFileClosed
	}

	// Since this is an expensive operation; let's make sure we need it
	if (whence == 0 && newOffset == o.fhOffset) || (whence == 1 && newOffset == 0) {
		return o.fhOffset, nil
	}
	log.Printf("WARNING: Seek behavior triggered, highly inefficent. Offset before seek is at %d\n", o.fhOffset)

	// Fore the reader/writers to be reopened (at correct offset)
	err := o.Sync()
	if err != nil {
		return 0, err
	}
	stat, err := o.Stat()
	if err != nil {
		return 0, nil
	}

	switch whence {
	case 0:
		o.fhOffset = newOffset
	case 1:
		o.fhOffset += newOffset
	case 2:
		o.fhOffset = stat.Size() + newOffset
	}
	return o.fhOffset, nil
}

func (o *GcsFile) Read(p []byte) (n int, err error) {
	return o.ReadAt(p, o.fhOffset)
}

func (o *GcsFile) ReadAt(p []byte, off int64) (n int, err error) {
	if o.closed {
		return 0, ErrFileClosed
	}

	read, err := o.resource.ReadAt(p, off)
	o.fhOffset += int64(read)
	return read, err
}

func (o *GcsFile) Write(p []byte) (n int, err error) {
	return o.WriteAt(p, o.fhOffset)
}

func (o *GcsFile) WriteAt(b []byte, off int64) (n int, err error) {
	if o.closed {
		return 0, ErrFileClosed
	}

	if o.openFlags&os.O_RDONLY != 0 {
		return 0, fmt.Errorf("file is opend as read only")
	}

	_, err = o.resource.obj.Attrs(o.resource.ctx)
	if err != nil {
		if err == storage.ErrObjectNotExist {
			if o.openFlags&os.O_CREATE == 0 {
				return 0, ErrFileNotFound
			}
		} else {
			return 0, fmt.Errorf("error getting file attributes: %v", err)
		}
	}

	written, err := o.resource.WriteAt(b, off)
	o.fhOffset += int64(written)
	return written, err
}

func (o *GcsFile) Name() string {
	return filepath.FromSlash(o.resource.name)
}

func (o *GcsFile) readdirImpl(count int) ([]*FileInfo, error) {
	err := o.Sync()
	if err != nil {
		return nil, err
	}

	var ownInfo os.FileInfo
	ownInfo, err = o.Stat()
	if err != nil {
		return nil, err
	}

	if !ownInfo.IsDir() {
		return nil, syscall.ENOTDIR
	}

	path := o.resource.fs.ensureTrailingSeparator(o.resource.name)
	if o.ReadDirIt == nil {
		// log.Printf("Querying path : %s\n", path)
		bucketName, bucketPath := o.resource.fs.splitName(path)

		o.ReadDirIt = o.resource.fs.client.Bucket(bucketName).Objects(
			o.resource.ctx, &storage.Query{Delimiter: o.resource.fs.separator, Prefix: bucketPath, Versions: false})
	}
	var res []*FileInfo
	for {
		object, err := o.ReadDirIt.Next()
		if err == iterator.Done {
			// reset the iterator
			o.ReadDirIt = nil

			if len(res) > 0 || count <= 0 {
				return res, nil
			}

			return res, io.EOF
		}
		if err != nil {
			return res, err
		}

		tmp := newFileInfoFromAttrs(object, o.resource.fileMode)

		if tmp.Name() == "" {
			// neither object.Name, not object.Prefix were present - so let's skip this unknown thing
			continue
		}

		if object.Name == "" && object.Prefix == "" {
			continue
		}

		if tmp.Name() == ownInfo.Name() {
			// Hmmm
			continue
		}

		res = append(res, tmp)

		// This would interrupt the iteration, once we reach the count.
		// But it would then have files coming before folders - that's not what we want to have exactly,
		// since it makes the results unpredictable. Hence, we iterate all the objects and then do
		// the cut-off in a higher level method
		//if count > 0 && len(res) >= count {
		//	break
		//}
	}
	// return res, nil
}

func (o *GcsFile) Readdir(count int) ([]os.FileInfo, error) {
	fi, err := o.readdirImpl(count)
	if len(fi) > 0 {
		sort.Sort(ByName(fi))
	}

	if count > 0 {
		fi = fi[:count]
	}

	var res []os.FileInfo
	for _, f := range fi {
		res = append(res, f)
	}
	return res, err
}

func (o *GcsFile) Readdirnames(n int) ([]string, error) {
	fi, err := o.Readdir(n)
	if err != nil && err != io.EOF {
		return nil, err
	}
	names := make([]string, len(fi))

	for i, f := range fi {
		names[i] = f.Name()
	}
	return names, err
}

func (o *GcsFile) Stat() (os.FileInfo, error) {
	err := o.Sync()
	if err != nil {
		return nil, err
	}

	return newFileInfo(o.resource.name, o.resource.fs, o.resource.fileMode)
}

func (o *GcsFile) Sync() error {
	return o.resource.maybeCloseIo()
}

func (o *GcsFile) Truncate(wantedSize int64) error {
	if o.closed {
		return ErrFileClosed
	}
	if o.openFlags == os.O_RDONLY {
		return fmt.Errorf("file was opened as read only")
	}
	return o.resource.Truncate(wantedSize)
}

func (o *GcsFile) WriteString(s string) (ret int, err error) {
	return o.Write([]byte(s))
}
