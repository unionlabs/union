// Copyright 2018 Google LLC
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
	"bytes"
	"compress/gzip"
	"context"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"net/url"
	"strconv"
	"strings"
	"testing"

	"google.golang.org/api/option"
)

const readData = "0123456789"

func TestRangeReader(t *testing.T) {
	ctx := context.Background()
	hc, close := newTestServer(handleRangeRead)
	defer close()

	multiReaderTest(ctx, t, func(t *testing.T, c *Client) {
		obj := c.Bucket("b").Object("o")
		for _, test := range []struct {
			offset, length int64
			want           string
		}{
			{0, -1, readData},
			{0, 10, readData},
			{0, 5, readData[:5]},
			{1, 3, readData[1:4]},
			{6, -1, readData[6:]},
			{4, 20, readData[4:]},
			{-20, -1, readData},
			{-6, -1, readData[4:]},
		} {
			r, err := obj.NewRangeReader(ctx, test.offset, test.length)
			if err != nil {
				t.Errorf("%d/%d: %v", test.offset, test.length, err)
				continue
			}
			gotb, err := ioutil.ReadAll(r)
			if err != nil {
				t.Errorf("%d/%d: %v", test.offset, test.length, err)
				continue
			}
			if got := string(gotb); got != test.want {
				t.Errorf("%d/%d: got %q, want %q", test.offset, test.length, got, test.want)
			}
		}
	}, option.WithHTTPClient(hc))
}

func handleRangeRead(w http.ResponseWriter, r *http.Request) {
	rh := strings.TrimSpace(r.Header.Get("Range"))
	data := readData
	var from, to int
	if rh == "" {
		from = 0
		to = len(data)
	} else {
		// assume "bytes=N-", "bytes=-N" or "bytes=N-M"
		var err error
		i := strings.IndexRune(rh, '=')
		j := strings.IndexRune(rh, '-')
		hasPositiveStartOffset := i+1 != j
		if hasPositiveStartOffset { // The case of "bytes=N-"
			from, err = strconv.Atoi(rh[i+1 : j])
		} else { // The case of "bytes=-N"
			from, err = strconv.Atoi(rh[i+1:])
			from += len(data)
			if from < 0 {
				from = 0
			}
		}
		if err != nil {
			w.WriteHeader(500)
			return
		}
		to = len(data)
		if hasPositiveStartOffset && j+1 < len(rh) { // The case of "bytes=N-M"
			to, err = strconv.Atoi(rh[j+1:])
			if err != nil {
				w.WriteHeader(500)
				return
			}
			to++ // Range header is inclusive, Go slice is exclusive
		}
		if from >= len(data) && to != from {
			w.WriteHeader(416)
			return
		}
		if from > len(data) {
			from = len(data)
		}
		if to > len(data) {
			to = len(data)
		}
	}
	data = data[from:to]
	if data != readData {
		w.Header().Set("Content-Range", fmt.Sprintf("bytes %d-%d/%d", from, to-1, len(readData)))
		w.WriteHeader(http.StatusPartialContent)
	}
	if _, err := w.Write([]byte(data)); err != nil {
		panic(err)
	}
}

type http2Error string

func (h http2Error) Error() string {
	return string(h)
}

// TestRangeReaderRetry tests Reader resumption logic. It ensures that offset
// and seen bytes are handled correctly so that data is not corrupted.
// This tests only works for the HTTP Reader.
// TODO: Design a similar test for gRPC.
func TestRangeReaderRetry(t *testing.T) {
	internalErr := http2Error("blah blah INTERNAL_ERROR")
	goawayErr := http2Error("http2: server sent GOAWAY and closed the connection; LastStreamID=15, ErrCode=NO_ERROR, debug=\"load_shed\"")
	readBytes := []byte(readData)
	hc, close := newTestServer(handleRangeRead)
	defer close()
	ctx := context.Background()

	multiReaderTest(ctx, t, func(t *testing.T, c *Client) {
		obj := c.Bucket("b").Object("o")
		for i, test := range []struct {
			offset, length int64
			bodies         []fakeReadCloser
			want           string
		}{
			{
				offset: 0,
				length: -1,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{10}, err: io.EOF},
				},
				want: readData,
			},
			{
				offset: 0,
				length: -1,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{3}, err: internalErr},
					{data: readBytes[3:], counts: []int{5, 2}, err: io.EOF},
				},
				want: readData,
			},
			{
				offset: 0,
				length: -1,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{5}, err: internalErr},
					{data: readBytes[5:], counts: []int{1, 3}, err: goawayErr},
					{data: readBytes[9:], counts: []int{1}, err: io.EOF},
				},
				want: readData,
			},
			{
				offset: 0,
				length: 5,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{3}, err: internalErr},
					{data: readBytes[3:], counts: []int{2}, err: io.EOF},
				},
				want: readData[:5],
			},
			{
				offset: 0,
				length: 5,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{3}, err: goawayErr},
					{data: readBytes[3:], counts: []int{2}, err: io.EOF},
				},
				want: readData[:5],
			},
			{
				offset: 1,
				length: 5,
				bodies: []fakeReadCloser{
					{data: readBytes, counts: []int{3}, err: internalErr},
					{data: readBytes[3:], counts: []int{2}, err: io.EOF},
				},
				want: readData[:5],
			},
			{
				offset: 1,
				length: 3,
				bodies: []fakeReadCloser{
					{data: readBytes[1:], counts: []int{1}, err: internalErr},
					{data: readBytes[2:], counts: []int{2}, err: io.EOF},
				},
				want: readData[1:4],
			},
			{
				offset: 4,
				length: -1,
				bodies: []fakeReadCloser{
					{data: readBytes[4:], counts: []int{1}, err: internalErr},
					{data: readBytes[5:], counts: []int{4}, err: internalErr},
					{data: readBytes[9:], counts: []int{1}, err: io.EOF},
				},
				want: readData[4:],
			},
			{
				offset: -4,
				length: -1,
				bodies: []fakeReadCloser{
					{data: readBytes[6:], counts: []int{1}, err: internalErr},
					{data: readBytes[7:], counts: []int{3}, err: io.EOF},
				},
				want: readData[6:],
			},
		} {
			r, err := obj.NewRangeReader(ctx, test.offset, test.length)
			if err != nil {
				t.Errorf("#%d: %v", i, err)
				continue
			}
			b := 0
			r.reader = &httpReader{
				body: &test.bodies[0],
				reopen: func(int64) (*http.Response, error) {
					b++
					return &http.Response{Body: &test.bodies[b]}, nil
				},
			}
			buf := make([]byte, len(readData)/2)
			var gotb []byte
			for {
				n, err := r.Read(buf)
				gotb = append(gotb, buf[:n]...)
				if err == io.EOF {
					break
				}
				if err != nil {
					t.Fatalf("#%d: %v", i, err)
				}
			}
			if err != nil {
				t.Errorf("#%d: %v", i, err)
				continue
			}
			if got := string(gotb); got != test.want {
				t.Errorf("#%d: got %q, want %q", i, got, test.want)
			}
			if r.Attrs.Size != int64(len(readData)) {
				t.Errorf("#%d: got Attrs.Size=%q, want %q", i, r.Attrs.Size, len(readData))
			}
			wantOffset := test.offset
			if wantOffset < 0 {
				wantOffset += int64(len(readData))
				if wantOffset < 0 {
					wantOffset = 0
				}
			}
			if got := r.Attrs.StartOffset; got != wantOffset {
				t.Errorf("#%d: got Attrs.Offset=%q, want %q", i, got, wantOffset)
			}
		}
		r, err := obj.NewRangeReader(ctx, -100, 10)
		if err == nil {
			t.Fatal("Expected a non-nil error with negative offset and positive length")
		} else if want := "storage: invalid offset"; !strings.HasPrefix(err.Error(), want) {
			t.Errorf("Error mismatch\nGot:  %q\nWant prefix: %q\n", err.Error(), want)
		}
		if r != nil {
			t.Errorf("Expected nil reader")
		}
	}, option.WithHTTPClient(hc))
}

type fakeReadCloser struct {
	data   []byte
	counts []int // how much of data to deliver on each read
	err    error // error to return with last count

	d int // current position in data
	c int // current position in counts
}

func (f *fakeReadCloser) Close() error {
	return nil
}

func (f *fakeReadCloser) Read(buf []byte) (int, error) {
	i := f.c
	n := 0
	if i < len(f.counts) {
		n = f.counts[i]
	}
	var err error
	if i >= len(f.counts)-1 {
		err = f.err
	}
	copy(buf, f.data[f.d:f.d+n])
	if len(buf) < n {
		n = len(buf)
		f.counts[i] -= n
		err = nil
	} else {
		f.c++
	}
	f.d += n
	return n, err
}

func TestFakeReadCloser(t *testing.T) {
	e := errors.New("")
	f := &fakeReadCloser{
		data:   []byte(readData),
		counts: []int{1, 2, 3},
		err:    e,
	}
	wants := []string{"0", "12", "345"}
	buf := make([]byte, 10)
	for i := 0; i < 3; i++ {
		n, err := f.Read(buf)
		if got, want := n, f.counts[i]; got != want {
			t.Fatalf("i=%d: got %d, want %d", i, got, want)
		}
		var wantErr error
		if i == 2 {
			wantErr = e
		}
		if err != wantErr {
			t.Fatalf("i=%d: got error %v, want %v", i, err, wantErr)
		}
		if got, want := string(buf[:n]), wants[i]; got != want {
			t.Fatalf("i=%d: got %q, want %q", i, got, want)
		}
	}
}

func TestContentEncodingGzipWithReader(t *testing.T) {
	bucketName := "my-bucket"
	objectName := "gzip-test"
	getAttrsURL := fmt.Sprintf("/b/%s/o/%s?alt=json&prettyPrint=false&projection=full", bucketName, objectName)
	downloadObjectXMLurl := fmt.Sprintf("/%s/%s", bucketName, objectName)
	downloadObjectJSONurl := fmt.Sprintf("/b/%s/o/%s?alt=media&prettyPrint=false&projection=full", bucketName, objectName)

	original := bytes.Repeat([]byte("a"), 4<<10)
	mockGCS := httptest.NewUnstartedServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.String() {
		case getAttrsURL:
			fmt.Fprintf(w, `{
                            "bucket": "bucket", "name": "name", "contentEncoding": "gzip",
                            "contentLength": 43,
                            "contentType": "text/plain","timeCreated": "2020-04-10T16:08:58-07:00",
                            "updated": "2020-04-14T16:08:58-07:00"
                        }`)
			return
		case downloadObjectXMLurl, downloadObjectJSONurl:
			// Serve back the file.
			w.Header().Set("Content-Type", "text/plain")
			w.Header().Set("Content-Encoding", "gzip")
			w.Header().Set("Etag", `"c50e3e41c9bc9df34e84c94ce073f928"`)
			w.Header().Set("X-Goog-Generation", "1587012235914578")
			w.Header().Set("X-Goog-MetaGeneration", "2")
			w.Header().Set("X-Goog-Stored-Content-Encoding", "gzip")
			w.Header().Set("vary", "Accept-Encoding")
			w.Header().Set("x-goog-stored-content-length", "43")
			w.Header().Set("x-goog-hash", "crc32c=pYIWwQ==")
			w.Header().Set("x-goog-hash", "md5=xQ4+Qcm8nfNOhMlM4HP5KA==")
			w.Header().Set("x-goog-storage-class", "STANDARD")
			gz := gzip.NewWriter(w)
			gz.Write(original)
			gz.Close()
		default:
			fmt.Fprintf(w, "unrecognized URL %s", r.URL)
		}
	}))
	mockGCS.EnableHTTP2 = true
	mockGCS.StartTLS()
	defer mockGCS.Close()

	ctx := context.Background()
	hc := mockGCS.Client()
	ux, _ := url.Parse(mockGCS.URL)
	hc.Transport.(*http.Transport).TLSClientConfig.InsecureSkipVerify = true
	wrt := &alwaysToTargetURLRoundTripper{
		destURL: ux,
		hc:      hc,
	}

	whc := &http.Client{Transport: wrt}

	// 2. Different flavours of the read should all return the body.
	readerCreators := []struct {
		name   string
		create func(ctx context.Context, obj *ObjectHandle) (*Reader, error)
	}{
		{
			"NewReader", func(cxt context.Context, obj *ObjectHandle) (*Reader, error) {
				return obj.NewReader(ctx)
			},
		},
		{
			"NewRangeReader(0, -1)",
			func(ctx context.Context, obj *ObjectHandle) (*Reader, error) {
				return obj.NewRangeReader(ctx, 0, -1)
			},
		},
		{
			"NewRangeReader(1kB, 2kB)",
			func(ctx context.Context, obj *ObjectHandle) (*Reader, error) {
				return obj.NewRangeReader(ctx, 1<<10, 2<<10)
			},
		},
		{
			"NewRangeReader(2kB, -1)",
			func(ctx context.Context, obj *ObjectHandle) (*Reader, error) {
				return obj.NewRangeReader(ctx, 2<<10, -1)
			},
		},
		{
			"NewRangeReader(2kB, 3kB)",
			func(ctx context.Context, obj *ObjectHandle) (*Reader, error) {
				return obj.NewRangeReader(ctx, 2<<10, 3<<10)
			},
		},
	}

	multiReaderTest(ctx, t, func(t *testing.T, c *Client) {
		for _, tt := range readerCreators {
			t.Run(tt.name, func(t *testing.T) {
				obj := c.Bucket(bucketName).Object(objectName)
				_, err := obj.Attrs(ctx)
				if err != nil {
					t.Fatal(err)
				}
				rd, err := tt.create(ctx, obj)
				if err != nil {
					t.Fatal(err)
				}
				defer rd.Close()

				got, err := ioutil.ReadAll(rd)
				if err != nil {
					t.Fatal(err)
				}
				if g, w := got, original; !bytes.Equal(g, w) {
					t.Fatalf("Response mismatch\nGot:\n%q\n\nWant:\n%q", g, w)
				}
			})
		}
	}, option.WithEndpoint(mockGCS.URL), option.WithoutAuthentication(), option.WithHTTPClient(whc))
}

// alwaysToTargetURLRoundTripper ensures that every single request
// is routed to a target destination. Some requests within the storage
// client by-pass using the provided HTTP client, hence this enforcemenet.
type alwaysToTargetURLRoundTripper struct {
	destURL *url.URL
	hc      *http.Client
}

func (adrt *alwaysToTargetURLRoundTripper) RoundTrip(req *http.Request) (*http.Response, error) {
	req.URL.Host = adrt.destURL.Host
	// Cloud Storage has full control over the response headers for their
	// HTTP server but unfortunately we don't, so we have to prune
	// the Range header to mimick GCS ignoring Range header:
	// https://cloud.google.com/storage/docs/transcoding#range
	delete(req.Header, "Range")
	return adrt.hc.Do(req)
}

// multiTransportTest initializes fresh clients for each transport, then runs
// given testing function using each transport-specific client, supplying the
// test function with the sub-test instance, the context it was given, the name
// of an existing bucket to use, a bucket name to use for bucket creation, and
// the client to use.
func multiReaderTest(ctx context.Context, t *testing.T, test func(*testing.T, *Client), opts ...option.ClientOption) {
	jsonOpts := append(opts, WithJSONReads())
	xmlOpts := append(opts, WithXMLReads())
	jsonClient, err := NewClient(ctx, jsonOpts...)
	if err != nil {
		t.Fatal(err)
	}
	xmlClient, err := NewClient(ctx, xmlOpts...)
	if err != nil {
		t.Fatal(err)
	}

	clients := map[string]*Client{
		"xmlReads":  xmlClient,
		"jsonReads": jsonClient,
	}

	for transport, client := range clients {
		t.Run(transport, func(t *testing.T) {
			defer client.Close()

			test(t, client)
		})
	}
}
