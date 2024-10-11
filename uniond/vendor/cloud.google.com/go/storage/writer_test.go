// Copyright 2014 Google LLC
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
	"crypto/sha256"
	"encoding/base64"
	"errors"
	"net/http"
	"strings"
	"testing"

	"cloud.google.com/go/internal/testutil"
	"google.golang.org/api/googleapi"
	"google.golang.org/api/option"
)

var testEncryptionKey = []byte("secret-key-that-is-32-bytes-long")

func TestErrorOnObjectsInsertCall(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	const contents = "hello world"

	doWrite := func(mt *mockTransport) *Writer {
		client := mockClient(t, mt)
		wc := client.Bucket("bucketname").Object("filename1").If(Conditions{DoesNotExist: true}).NewWriter(ctx)
		wc.ContentType = "text/plain"

		// We can't check that the Write fails, since it depends on the write to the
		// underling mockTransport failing which is racy.
		wc.Write([]byte(contents))
		return wc
	}

	wc := doWrite(&mockTransport{})
	// Close must always return an error though since it waits for the transport to
	// have closed.
	if err := wc.Close(); err == nil {
		t.Errorf("expected error on close, got nil")
	}

	// Retry on 5xx
	mt := &mockTransport{}
	mt.addResult(&http.Response{StatusCode: 503, Body: bodyReader("")}, nil)
	mt.addResult(&http.Response{StatusCode: 200, Body: bodyReader("{}")}, nil)

	wc = doWrite(mt)
	if err := wc.Close(); err != nil {
		t.Errorf("got %v, want nil", err)
	}
	got := string(mt.gotBody)
	if !strings.Contains(got, contents) {
		t.Errorf("got body %q, which does not contain %q", got, contents)
	}
}

func TestEncryption(t *testing.T) {
	t.Parallel()
	ctx := context.Background()
	mt := &mockTransport{}
	mt.addResult(&http.Response{StatusCode: 200, Body: bodyReader("{}")}, nil)
	client := mockClient(t, mt)
	obj := client.Bucket("bucketname").Object("filename1")
	wc := obj.Key(testEncryptionKey).NewWriter(ctx)
	if _, err := wc.Write([]byte("hello world")); err != nil {
		t.Fatal(err)
	}
	if err := wc.Close(); err != nil {
		t.Fatal(err)
	}
	if got, want := mt.gotReq.Header.Get("x-goog-encryption-algorithm"), "AES256"; got != want {
		t.Errorf("algorithm: got %q, want %q", got, want)
	}
	gotKey, err := base64.StdEncoding.DecodeString(mt.gotReq.Header.Get("x-goog-encryption-key"))
	if err != nil {
		t.Fatalf("decoding key: %v", err)
	}
	if !testutil.Equal(gotKey, testEncryptionKey) {
		t.Errorf("key: got %v, want %v", gotKey, testEncryptionKey)
	}
	wantHash := sha256.Sum256(testEncryptionKey)
	gotHash, err := base64.StdEncoding.DecodeString(mt.gotReq.Header.Get("x-goog-encryption-key-sha256"))
	if err != nil {
		t.Fatalf("decoding hash: %v", err)
	}
	if !testutil.Equal(gotHash, wantHash[:]) { // wantHash is an array
		t.Errorf("hash: got\n%v, want\n%v", gotHash, wantHash)
	}

	// Using a customer-supplied encryption key and a KMS key together is an error.
	checkKMSError := func(msg string, err error) {
		if err == nil {
			t.Errorf("%s: got nil, want error", msg)
		} else if !strings.Contains(err.Error(), "KMS") {
			t.Errorf(`%s: got %q, want it to contain "KMS"`, msg, err)
		}
	}

	wc = obj.Key(testEncryptionKey).NewWriter(ctx)
	wc.KMSKeyName = "key"
	_, err = wc.Write([]byte{})
	checkKMSError("Write", err)
	checkKMSError("Close", wc.Close())
}

// This test demonstrates the data race on Writer.err that can happen when the
// Writer's context is cancelled. To see the race, comment out the w.mu.Lock/Unlock
// lines in writer.go and run this test with -race.
func TestRaceOnCancelAfterWrite(t *testing.T) {
	client := mockClient(t, &mockTransport{})
	cctx, cancel := context.WithCancel(context.Background())
	w := client.Bucket("b").Object("o").NewWriter(cctx)
	w.ChunkSize = googleapi.MinUploadChunkSize
	buf := make([]byte, w.ChunkSize)
	// This Write starts the goroutine in Writer.open. That reads the first chunk in its entirety
	// before sending the request (see google.golang.org/api/gensupport.PrepareUpload),
	// so to exhibit the race we must provide ChunkSize bytes.  The goroutine then makes the RPC (L137).
	w.Write(buf)
	// Canceling the context causes the call to return context.Canceled, which makes the open goroutine
	// write to w.err (L151).
	cancel()
	// This call to Write concurrently reads w.err (L169).
	w.Write([]byte(nil))
}

// This test checks for the read-write race condition that can occur if
// monitorCancel is launched before the pipe is opened. See
// https://github.com/googleapis/google-cloud-go/issues/6816
func TestRaceOnCancelBeforeWrite(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	client := mockClient(t, &mockTransport{})
	cancel()
	writer := client.Bucket("foo").Object("bar").NewWriter(ctx)
	writer.ChunkSize = googleapi.MinUploadChunkSize
	writer.Write([]byte("data"))

	// We expect Close to return a context cancelled error, and the Writer should
	// not open the pipe and avoid triggering a race condition.
	if err := writer.Close(); !errors.Is(err, context.Canceled) {
		t.Errorf("Writer.Close: got %v, want %v", err, context.Canceled)
	}
}

func TestCancelDoesNotLeak(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	const contents = "hello world"
	mt := mockTransport{}

	client, err := NewClient(ctx, option.WithHTTPClient(&http.Client{Transport: &mt}))
	if err != nil {
		t.Fatal(err)
	}

	wc := client.Bucket("bucketname").Object("filename1").NewWriter(ctx)
	wc.ContentType = "text/plain"

	// We can't check that the Write fails, since it depends on the write to the
	// underling mockTransport failing which is racy.
	wc.Write([]byte(contents))

	cancel()
}

func TestCloseDoesNotLeak(t *testing.T) {
	ctx := context.Background()
	const contents = "hello world"
	mt := mockTransport{}

	client, err := NewClient(ctx, option.WithHTTPClient(&http.Client{Transport: &mt}))
	if err != nil {
		t.Fatal(err)
	}

	wc := client.Bucket("bucketname").Object("filename1").NewWriter(ctx)
	wc.ContentType = "text/plain"

	// We can't check that the Write fails, since it depends on the write to the
	// underling mockTransport failing which is racy.
	wc.Write([]byte(contents))

	wc.Close()
}
