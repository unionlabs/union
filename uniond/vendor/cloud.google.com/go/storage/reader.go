// Copyright 2016 Google LLC
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

package storage

import (
	"context"
	"fmt"
	"hash/crc32"
	"io"
	"io/ioutil"
	"net/http"
	"strings"
	"time"

	"cloud.google.com/go/internal/trace"
)

var crc32cTable = crc32.MakeTable(crc32.Castagnoli)

// ReaderObjectAttrs are attributes about the object being read. These are populated
// during the New call. This struct only holds a subset of object attributes: to
// get the full set of attributes, use ObjectHandle.Attrs.
//
// Each field is read-only.
type ReaderObjectAttrs struct {
	// Size is the length of the object's content.
	Size int64

	// StartOffset is the byte offset within the object
	// from which reading begins.
	// This value is only non-zero for range requests.
	StartOffset int64

	// ContentType is the MIME type of the object's content.
	ContentType string

	// ContentEncoding is the encoding of the object's content.
	ContentEncoding string

	// CacheControl specifies whether and for how long browser and Internet
	// caches are allowed to cache your objects.
	CacheControl string

	// LastModified is the time that the object was last modified.
	LastModified time.Time

	// Generation is the generation number of the object's content.
	Generation int64

	// Metageneration is the version of the metadata for this object at
	// this generation. This field is used for preconditions and for
	// detecting changes in metadata. A metageneration number is only
	// meaningful in the context of a particular generation of a
	// particular object.
	Metageneration int64
}

// NewReader creates a new Reader to read the contents of the
// object.
// ErrObjectNotExist will be returned if the object is not found.
//
// The caller must call Close on the returned Reader when done reading.
func (o *ObjectHandle) NewReader(ctx context.Context) (*Reader, error) {
	return o.NewRangeReader(ctx, 0, -1)
}

// NewRangeReader reads part of an object, reading at most length bytes
// starting at the given offset. If length is negative, the object is read
// until the end. If offset is negative, the object is read abs(offset) bytes
// from the end, and length must also be negative to indicate all remaining
// bytes will be read.
//
// If the object's metadata property "Content-Encoding" is set to "gzip" or satisfies
// decompressive transcoding per https://cloud.google.com/storage/docs/transcoding
// that file will be served back whole, regardless of the requested range as
// Google Cloud Storage dictates.
func (o *ObjectHandle) NewRangeReader(ctx context.Context, offset, length int64) (r *Reader, err error) {
	ctx = trace.StartSpan(ctx, "cloud.google.com/go/storage.Object.NewRangeReader")
	defer func() { trace.EndSpan(ctx, err) }()

	if err := o.validate(); err != nil {
		return nil, err
	}
	if offset < 0 && length >= 0 {
		return nil, fmt.Errorf("storage: invalid offset %d < 0 requires negative length", offset)
	}
	if o.conds != nil {
		if err := o.conds.validate("NewRangeReader"); err != nil {
			return nil, err
		}
	}

	opts := makeStorageOpts(true, o.retry, o.userProject)

	params := &newRangeReaderParams{
		bucket:         o.bucket,
		object:         o.object,
		gen:            o.gen,
		offset:         offset,
		length:         length,
		encryptionKey:  o.encryptionKey,
		conds:          o.conds,
		readCompressed: o.readCompressed,
	}

	r, err = o.c.tc.NewRangeReader(ctx, params, opts...)

	return r, err
}

// decompressiveTranscoding returns true if the request was served decompressed
// and different than its original storage form. This happens when the "Content-Encoding"
// header is "gzip".
// See:
//   - https://cloud.google.com/storage/docs/transcoding#transcoding_and_gzip
//   - https://github.com/googleapis/google-cloud-go/issues/1800
func decompressiveTranscoding(res *http.Response) bool {
	// Decompressive Transcoding.
	return res.Header.Get("Content-Encoding") == "gzip" ||
		res.Header.Get("X-Goog-Stored-Content-Encoding") == "gzip"
}

func uncompressedByServer(res *http.Response) bool {
	// If the data is stored as gzip but is not encoded as gzip, then it
	// was uncompressed by the server.
	return res.Header.Get("X-Goog-Stored-Content-Encoding") == "gzip" &&
		res.Header.Get("Content-Encoding") != "gzip"
}

// parseCRC32c parses the crc32c hash from the X-Goog-Hash header.
// It can parse headers in the form [crc32c=xxx md5=xxx] (XML responses) or the
// form [crc32c=xxx,md5=xxx] (JSON responses). The md5 hash is ignored.
func parseCRC32c(res *http.Response) (uint32, bool) {
	const prefix = "crc32c="
	for _, spec := range res.Header["X-Goog-Hash"] {
		values := strings.Split(spec, ",")

		for _, v := range values {
			if strings.HasPrefix(v, prefix) {
				c, err := decodeUint32(v[len(prefix):])
				if err == nil {
					return c, true
				}
			}
		}

	}
	return 0, false
}

// setConditionsHeaders sets precondition request headers for downloads
// using the XML API. It assumes that the conditions have been validated.
func setConditionsHeaders(headers http.Header, conds *Conditions) error {
	if conds == nil {
		return nil
	}
	if conds.MetagenerationMatch != 0 {
		headers.Set("x-goog-if-metageneration-match", fmt.Sprint(conds.MetagenerationMatch))
	}
	switch {
	case conds.GenerationMatch != 0:
		headers.Set("x-goog-if-generation-match", fmt.Sprint(conds.GenerationMatch))
	case conds.DoesNotExist:
		headers.Set("x-goog-if-generation-match", "0")
	}
	return nil
}

// Wrap a request to look similar to an apiary library request, in order to
// be used by run().
type readerRequestWrapper struct {
	req *http.Request
}

func (w *readerRequestWrapper) Header() http.Header {
	return w.req.Header
}

var emptyBody = ioutil.NopCloser(strings.NewReader(""))

// Reader reads a Cloud Storage object.
// It implements io.Reader.
//
// Typically, a Reader computes the CRC of the downloaded content and compares it to
// the stored CRC, returning an error from Read if there is a mismatch. This integrity check
// is skipped if transcoding occurs. See https://cloud.google.com/storage/docs/transcoding.
type Reader struct {
	Attrs              ReaderObjectAttrs
	seen, remain, size int64
	checkCRC           bool   // should we check the CRC?
	wantCRC            uint32 // the CRC32c value the server sent in the header
	gotCRC             uint32 // running crc

	reader io.ReadCloser
}

// Close closes the Reader. It must be called when done reading.
func (r *Reader) Close() error {
	return r.reader.Close()
}

func (r *Reader) Read(p []byte) (int, error) {
	n, err := r.reader.Read(p)
	if r.remain != -1 {
		r.remain -= int64(n)
	}
	if r.checkCRC {
		r.gotCRC = crc32.Update(r.gotCRC, crc32cTable, p[:n])
		// Check CRC here. It would be natural to check it in Close, but
		// everybody defers Close on the assumption that it doesn't return
		// anything worth looking at.
		if err == io.EOF {
			if r.gotCRC != r.wantCRC {
				return n, fmt.Errorf("storage: bad CRC on read: got %d, want %d",
					r.gotCRC, r.wantCRC)
			}
		}
	}
	return n, err
}

// Size returns the size of the object in bytes.
// The returned value is always the same and is not affected by
// calls to Read or Close.
//
// Deprecated: use Reader.Attrs.Size.
func (r *Reader) Size() int64 {
	return r.Attrs.Size
}

// Remain returns the number of bytes left to read, or -1 if unknown.
func (r *Reader) Remain() int64 {
	return r.remain
}

// ContentType returns the content type of the object.
//
// Deprecated: use Reader.Attrs.ContentType.
func (r *Reader) ContentType() string {
	return r.Attrs.ContentType
}

// ContentEncoding returns the content encoding of the object.
//
// Deprecated: use Reader.Attrs.ContentEncoding.
func (r *Reader) ContentEncoding() string {
	return r.Attrs.ContentEncoding
}

// CacheControl returns the cache control of the object.
//
// Deprecated: use Reader.Attrs.CacheControl.
func (r *Reader) CacheControl() string {
	return r.Attrs.CacheControl
}

// LastModified returns the value of the Last-Modified header.
//
// Deprecated: use Reader.Attrs.LastModified.
func (r *Reader) LastModified() (time.Time, error) {
	return r.Attrs.LastModified, nil
}
