// Copyright 2015 Google LLC
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package gensupport

import (
	"bytes"
	cryptorand "crypto/rand"
	"io"
	mathrand "math/rand"
	"net/http"
	"strings"
	"testing"
	"time"

	"google.golang.org/api/googleapi"
)

func TestNewInfoFromMedia(t *testing.T) {
	const textType = "text/plain; charset=utf-8"
	for _, test := range []struct {
		desc                                   string
		r                                      io.Reader
		opts                                   []googleapi.MediaOption
		wantType                               string
		wantMedia, wantBuffer, wantSingleChunk bool
		wantDeadline                           time.Duration
	}{
		{
			desc:            "an empty reader results in a MediaBuffer with a single, empty chunk",
			r:               new(bytes.Buffer),
			opts:            nil,
			wantType:        textType,
			wantBuffer:      true,
			wantSingleChunk: true,
		},
		{
			desc:            "ContentType is observed",
			r:               new(bytes.Buffer),
			opts:            []googleapi.MediaOption{googleapi.ContentType("xyz")},
			wantType:        "xyz",
			wantBuffer:      true,
			wantSingleChunk: true,
		},
		{
			desc:            "ChunkRetryDeadline is observed",
			r:               new(bytes.Buffer),
			opts:            []googleapi.MediaOption{googleapi.ChunkRetryDeadline(time.Second)},
			wantType:        textType,
			wantBuffer:      true,
			wantSingleChunk: true,
			wantDeadline:    time.Second,
		},
		{
			desc:            "chunk size of zero: don't use a MediaBuffer; upload as a single chunk",
			r:               strings.NewReader("12345"),
			opts:            []googleapi.MediaOption{googleapi.ChunkSize(0)},
			wantType:        textType,
			wantMedia:       true,
			wantSingleChunk: true,
		},
		{
			desc:            "chunk size > data size: MediaBuffer with single chunk",
			r:               strings.NewReader("12345"),
			opts:            []googleapi.MediaOption{googleapi.ChunkSize(100)},
			wantType:        textType,
			wantBuffer:      true,
			wantSingleChunk: true,
		},
		{
			desc:            "chunk size == data size: MediaBuffer with single chunk",
			r:               &nullReader{googleapi.MinUploadChunkSize},
			opts:            []googleapi.MediaOption{googleapi.ChunkSize(1)},
			wantType:        "application/octet-stream",
			wantBuffer:      true,
			wantSingleChunk: true,
		},
		{
			desc: "chunk size < data size: MediaBuffer, not single chunk",
			// Note that ChunkSize = 1 is rounded up to googleapi.MinUploadChunkSize.
			r:               &nullReader{2 * googleapi.MinUploadChunkSize},
			opts:            []googleapi.MediaOption{googleapi.ChunkSize(1)},
			wantType:        "application/octet-stream",
			wantBuffer:      true,
			wantSingleChunk: false,
		},
	} {

		mi := NewInfoFromMedia(test.r, test.opts)
		if got, want := mi.mType, test.wantType; got != want {
			t.Errorf("%s: type: got %q, want %q", test.desc, got, want)
		}
		if got, want := (mi.media != nil), test.wantMedia; got != want {
			t.Errorf("%s: media non-nil: got %t, want %t", test.desc, got, want)
		}
		if got, want := (mi.buffer != nil), test.wantBuffer; got != want {
			t.Errorf("%s: buffer non-nil: got %t, want %t", test.desc, got, want)
		}
		if got, want := mi.singleChunk, test.wantSingleChunk; got != want {
			t.Errorf("%s: singleChunk: got %t, want %t", test.desc, got, want)
		}
		if got, want := mi.chunkRetryDeadline, test.wantDeadline; got != want {
			t.Errorf("%s: chunkRetryDeadline: got %v, want %v", test.desc, got, want)
		}
	}
}

func TestUploadRequest(t *testing.T) {
	for _, test := range []struct {
		desc            string
		r               io.Reader
		chunkSize       int
		wantContentType string
		wantUploadType  string
	}{
		{
			desc:            "chunk size of zero: don't use a MediaBuffer; upload as a single chunk",
			r:               strings.NewReader("12345"),
			chunkSize:       0,
			wantContentType: "multipart/related;",
		},
		{
			desc:            "chunk size > data size: MediaBuffer with single chunk",
			r:               strings.NewReader("12345"),
			chunkSize:       100,
			wantContentType: "multipart/related;",
		},
		{
			desc:            "chunk size == data size: MediaBuffer with single chunk",
			r:               &nullReader{googleapi.MinUploadChunkSize},
			chunkSize:       1,
			wantContentType: "multipart/related;",
		},
		{
			desc: "chunk size < data size: MediaBuffer, not single chunk",
			// Note that ChunkSize = 1 is rounded up to googleapi.MinUploadChunkSize.
			r:              &nullReader{2 * googleapi.MinUploadChunkSize},
			chunkSize:      1,
			wantUploadType: "application/octet-stream",
		},
	} {
		mi := NewInfoFromMedia(test.r, []googleapi.MediaOption{googleapi.ChunkSize(test.chunkSize)})
		h := http.Header{}
		mi.UploadRequest(h, new(bytes.Buffer))
		if got, want := h.Get("Content-Type"), test.wantContentType; !strings.HasPrefix(got, want) {
			t.Errorf("%s: Content-Type: got %q, want prefix %q", test.desc, got, want)
		}
		if got, want := h.Get("X-Upload-Content-Type"), test.wantUploadType; got != want {
			t.Errorf("%s: X-Upload-Content-Type: got %q, want %q", test.desc, got, want)
		}
	}
}

func TestUploadRequestGetBody(t *testing.T) {
	// Test that a single chunk results in a getBody function that is non-nil, and
	// that produces the same content as the original body.

	// Restore the crypto/rand.Reader mocked out below.
	defer func(old io.Reader) { cryptorand.Reader = old }(cryptorand.Reader)

	for i, test := range []struct {
		desc        string
		r           io.Reader
		chunkSize   int
		wantGetBody bool
	}{
		{
			desc:        "chunk size of zero: no getBody",
			r:           &nullReader{10},
			chunkSize:   0,
			wantGetBody: false,
		},
		{
			desc:        "chunk size == data size: 1 chunk, getBody",
			r:           &nullReader{googleapi.MinUploadChunkSize},
			chunkSize:   1,
			wantGetBody: true,
		},
		{
			desc: "chunk size < data size: MediaBuffer, >1 chunk, getBody",
			// Note that ChunkSize = 1 is rounded up to googleapi.MinUploadChunkSize.
			r:           &nullReader{2 * googleapi.MinUploadChunkSize},
			chunkSize:   1,
			wantGetBody: true,
		},
	} {
		cryptorand.Reader = mathrand.New(mathrand.NewSource(int64(i)))

		mi := NewInfoFromMedia(test.r, []googleapi.MediaOption{googleapi.ChunkSize(test.chunkSize)})
		r, getBody, _ := mi.UploadRequest(http.Header{}, bytes.NewBuffer([]byte("body")))
		if got, want := (getBody != nil), test.wantGetBody; got != want {
			t.Errorf("%s: getBody: got %t, want %t", test.desc, got, want)
			continue
		}
		if getBody == nil {
			continue
		}
		want, err := io.ReadAll(r)
		if err != nil {
			t.Fatal(err)
		}
		for i := 0; i < 3; i++ {
			rc, err := getBody()
			if err != nil {
				t.Fatal(err)
			}
			got, err := io.ReadAll(rc)
			if err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(got, want) {
				t.Errorf("%s, %d:\ngot:\n%s\nwant:\n%s", test.desc, i, string(got), string(want))
			}
		}
	}
}

func TestResumableUpload(t *testing.T) {
	for _, test := range []struct {
		desc                string
		r                   io.Reader
		chunkSize           int
		wantUploadType      string
		wantResumableUpload bool
		chunkRetryDeadline  time.Duration
	}{
		{
			desc:                "chunk size of zero: don't use a MediaBuffer; upload as a single chunk",
			r:                   strings.NewReader("12345"),
			chunkSize:           0,
			wantUploadType:      "multipart",
			wantResumableUpload: false,
		},
		{
			desc:                "chunk size > data size: MediaBuffer with single chunk",
			r:                   strings.NewReader("12345"),
			chunkSize:           100,
			wantUploadType:      "multipart",
			wantResumableUpload: false,
		},
		{
			desc: "chunk size == data size: MediaBuffer with single chunk",
			// (Because nullReader returns EOF with the last bytes.)
			r:                   &nullReader{googleapi.MinUploadChunkSize},
			chunkSize:           googleapi.MinUploadChunkSize,
			wantUploadType:      "multipart",
			wantResumableUpload: false,
		},
		{
			desc: "chunk size < data size: MediaBuffer, not single chunk",
			// Note that ChunkSize = 1 is rounded up to googleapi.MinUploadChunkSize.
			r:                   &nullReader{2 * googleapi.MinUploadChunkSize},
			chunkSize:           1,
			wantUploadType:      "resumable",
			wantResumableUpload: true,
		},
		{
			desc:                "confirm that ChunkRetryDeadline is carried to ResumableUpload",
			r:                   &nullReader{2 * googleapi.MinUploadChunkSize},
			chunkSize:           1,
			wantUploadType:      "resumable",
			wantResumableUpload: true,
			chunkRetryDeadline:  1 * time.Second,
		},
	} {
		opts := []googleapi.MediaOption{googleapi.ChunkSize(test.chunkSize)}
		if test.chunkRetryDeadline != 0 {
			opts = append(opts, googleapi.ChunkRetryDeadline(test.chunkRetryDeadline))
		}
		mi := NewInfoFromMedia(test.r, opts)
		if got, want := mi.UploadType(), test.wantUploadType; got != want {
			t.Errorf("%s: upload type: got %q, want %q", test.desc, got, want)
		}
		if got, want := mi.ResumableUpload("") != nil, test.wantResumableUpload; got != want {
			t.Errorf("%s: resumable upload non-nil: got %t, want %t", test.desc, got, want)
		}
		if test.chunkRetryDeadline != 0 {
			if got := mi.ResumableUpload(""); got != nil {
				if got.ChunkRetryDeadline != test.chunkRetryDeadline {
					t.Errorf("%s: ChunkRetryDeadline: got %v, want %v", test.desc, got.ChunkRetryDeadline, test.chunkRetryDeadline)
				}
			} else {
				t.Errorf("%s: test case invalid; resumable upload is nil", test.desc)
			}
		}
	}
}

// A nullReader simulates reading a fixed number of bytes.
type nullReader struct {
	remain int
}

// Read doesn't touch buf, but it does reduce the amount of bytes remaining
// by len(buf).
func (r *nullReader) Read(buf []byte) (int, error) {
	n := len(buf)
	if r.remain < n {
		n = r.remain
	}
	r.remain -= n
	var err error
	if r.remain == 0 {
		err = io.EOF
	}
	return n, err
}
