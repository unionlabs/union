// Copyright 2022, Google Inc.
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

package gax

import (
	"bytes"
	"io"
	"io/ioutil"
	"reflect"
	"testing"
)

// errReader reads out of a buffer until it is empty, then returns the specified error.
type errReader struct {
	buf []byte
	err error
}

func (er *errReader) Read(p []byte) (int, error) {
	if len(er.buf) == 0 {
		if er.err == nil {
			return 0, io.EOF
		}
		return 0, er.err
	}
	n := copy(p, er.buf)
	er.buf = er.buf[n:]
	return n, nil
}

func TestContentSniffing(t *testing.T) {
	type testCase struct {
		data     []byte // the data to read from the Reader
		finalErr error  // error to return after data has been read

		wantContentType       string
		wantContentTypeResult bool
	}

	for _, tc := range []testCase{
		{
			data:                  []byte{0, 0, 0, 0},
			finalErr:              nil,
			wantContentType:       "application/octet-stream",
			wantContentTypeResult: true,
		},
		{
			data:                  []byte(""),
			finalErr:              nil,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: true,
		},
		{
			data:                  []byte(""),
			finalErr:              io.ErrUnexpectedEOF,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: false,
		},
		{
			data:                  []byte("abc"),
			finalErr:              nil,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: true,
		},
		{
			data:                  []byte("abc"),
			finalErr:              io.ErrUnexpectedEOF,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: false,
		},
		// The following examples contain more bytes than are buffered for sniffing.
		{
			data:                  bytes.Repeat([]byte("a"), 513),
			finalErr:              nil,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: true,
		},
		{
			data:                  bytes.Repeat([]byte("a"), 513),
			finalErr:              io.ErrUnexpectedEOF,
			wantContentType:       "text/plain; charset=utf-8",
			wantContentTypeResult: true, // true because error is after first 512 bytes.
		},
	} {
		er := &errReader{buf: tc.data, err: tc.finalErr}

		sct := newContentSniffer(er)

		// Even if was an error during the first 512 bytes, we should still be able to read those bytes.
		buf, err := ioutil.ReadAll(sct)

		if !reflect.DeepEqual(buf, tc.data) {
			t.Fatalf("Failed reading buffer: got: %q; want:%q", buf, tc.data)
		}

		if err != tc.finalErr {
			t.Fatalf("Reading buffer error: got: %v; want: %v", err, tc.finalErr)
		}

		ct, ok := sct.ContentType()
		if ok != tc.wantContentTypeResult {
			t.Fatalf("Content type result got: %v; want: %v", ok, tc.wantContentTypeResult)
		}
		if ok && ct != tc.wantContentType {
			t.Fatalf("Content type got: %q; want: %q", ct, tc.wantContentType)
		}
	}
}

type staticContentTyper struct {
	io.Reader
}

func (sct staticContentTyper) ContentType() string {
	return "static content type"
}

func TestDetermineContentType(t *testing.T) {
	data := []byte("abc")
	rdr := func() io.Reader {
		return bytes.NewBuffer(data)
	}

	type testCase struct {
		r               io.Reader
		wantContentType string
	}

	for _, tc := range []testCase{
		{
			r:               rdr(),
			wantContentType: "text/plain; charset=utf-8",
		},
		{
			r:               staticContentTyper{rdr()},
			wantContentType: "static content type",
		},
	} {
		r, ctype := DetermineContentType(tc.r)
		got, err := ioutil.ReadAll(r)
		if err != nil {
			t.Fatalf("Failed reading buffer: %v", err)
		}
		if !reflect.DeepEqual(got, data) {
			t.Fatalf("Failed reading buffer: got: %q; want:%q", got, data)
		}

		if ctype != tc.wantContentType {
			t.Fatalf("Content type got: %q; want: %q", ctype, tc.wantContentType)
		}
	}
}
