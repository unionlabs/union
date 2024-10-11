// Copyright 2024 Google LLC
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

package transfermanager

import (
	"bytes"
	"context"
	crand "crypto/rand"
	"errors"
	"flag"
	"fmt"
	"hash/crc32"
	"io"
	"log"
	"math/rand"
	"sync"
	"testing"
	"time"

	"cloud.google.com/go/internal/testutil"
	"cloud.google.com/go/internal/uid"
	"cloud.google.com/go/storage"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/api/googleapi"
	"google.golang.org/api/iterator"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

const (
	testPrefix      = "go-integration-test-tm"
	grpcTestPrefix  = "golang-grpc-test-tm"
	bucketExpiryAge = 24 * time.Hour
)

var (
	uidSpace = uid.NewSpace("", &uid.Options{Short: true})
	//  These buckets are shared amongst download tests. They are created,
	// populated with objects and cleaned up in TestMain.
	httpTestBucket = downloadTestBucket{}
	grpcTestBucket = downloadTestBucket{}
)

func TestMain(m *testing.M) {
	flag.Parse()

	if err := httpTestBucket.Create(testPrefix); err != nil {
		log.Fatalf("test bucket creation failed: %v", err)
	}

	if err := grpcTestBucket.Create(grpcTestPrefix); err != nil {
		log.Fatalf("test bucket creation failed: %v", err)
	}

	m.Run()

	if err := httpTestBucket.Cleanup(); err != nil {
		log.Printf("test bucket cleanup failed: %v", err)
	}
	if err := grpcTestBucket.Cleanup(); err != nil {
		log.Printf("grpc test bucket cleanup failed: %v", err)
	}
	if err := deleteExpiredBuckets(testPrefix); err != nil {
		log.Printf("expired http bucket cleanup failed: %v", err)
	}
	if err := deleteExpiredBuckets(grpcTestPrefix); err != nil {
		log.Printf("expired grpc bucket cleanup failed: %v", err)
	}
}

func TestIntegration_DownloaderSynchronous(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, c *storage.Client, tb downloadTestBucket) {
		objects := tb.objects

		// Start a downloader. Give it a smaller amount of workers than objects,
		// to make sure we aren't blocking anywhere.
		d, err := NewDownloader(c, WithWorkers(2))
		if err != nil {
			t.Fatalf("NewDownloader: %v", err)
		}
		// Download several objects.
		writers := make([]*testWriter, len(objects))
		objToWriter := make(map[string]int) // so we can map the resulting content back to the correct object
		for i, obj := range objects {
			writers[i] = &testWriter{}
			objToWriter[obj] = i
			if err := d.DownloadObject(ctx, &DownloadObjectInput{
				Bucket:      tb.bucket,
				Object:      obj,
				Destination: writers[i],
			}); err != nil {
				t.Errorf("d.DownloadObject: %v", err)
			}
		}

		results, err := d.WaitAndClose()
		if err != nil {
			t.Fatalf("d.WaitAndClose: %v", err)
		}

		// Close the writers so we can check the contents. This should be fine,
		// since the downloads should all be complete after WaitAndClose.
		for i := range objects {
			if err := writers[i].Close(); err != nil {
				t.Fatalf("testWriter.Close: %v", err)
			}
		}

		// Check the results.
		for _, got := range results {
			writerIdx := objToWriter[got.Object]

			if got.Err != nil {
				t.Errorf("result.Err: %v", got.Err)
				continue
			}

			if want, got := tb.contentHashes[got.Object], writers[writerIdx].crc32c; got != want {
				t.Fatalf("content crc32c does not match; got: %v, expected: %v", got, want)
			}

			if got.Attrs.Size != tb.objectSize {
				t.Errorf("expected object size %d, got %d", tb.objectSize, got.Attrs.Size)
			}
		}

		if len(results) != len(objects) {
			t.Errorf("expected to receive %d results, got %d results", len(objects), len(results))
		}
	})
}

// Tests that a single error does not affect the rest of the downloads.
func TestIntegration_DownloaderErrorSync(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, c *storage.Client, tb downloadTestBucket) {
		// Make a copy of the objects slice.
		objects := make([]string, len(tb.objects))
		copy(objects, tb.objects)

		// Add another object to attempt to download; since it hasn't been written,
		// this one will fail. Append to the start so that it will (likely) be
		// attempted in the first 2 downloads.
		nonexistentObject := "not-written"
		objects = append([]string{nonexistentObject}, objects...)

		// Start a downloader.
		d, err := NewDownloader(c, WithWorkers(2))
		if err != nil {
			t.Fatalf("NewDownloader: %v", err)
		}

		// Download objects.
		writers := make([]*testWriter, len(objects))
		objToWriter := make(map[string]int) // so we can map the resulting content back to the correct object
		for i, obj := range objects {
			writers[i] = &testWriter{}
			objToWriter[obj] = i
			if err := d.DownloadObject(ctx, &DownloadObjectInput{
				Bucket:      tb.bucket,
				Object:      obj,
				Destination: writers[i],
			}); err != nil {
				t.Errorf("d.DownloadObject: %v", err)
			}
		}

		// WaitAndClose should return an error since one of our downloads should have failed.
		results, err := d.WaitAndClose()
		if err == nil {
			t.Error("d.WaitAndClose should return an error, instead got nil")
		}

		// Close the writers so we can check the contents. This should be fine,
		// since the downloads should all be complete after WaitAndClose.
		for i := range objects {
			if err := writers[i].Close(); err != nil {
				t.Fatalf("testWriter.Close: %v", err)
			}
		}

		// Check the results.
		for _, got := range results {
			writerIdx := objToWriter[got.Object]

			// Check that the nonexistent object returned an error.
			if got.Object == nonexistentObject {
				if got.Err != storage.ErrObjectNotExist {
					t.Errorf("Object(%q) should not exist, err found to be %v", got.Object, got.Err)
				}
				continue
			}

			// All other objects should complete correctly.
			if got.Err != nil {
				t.Errorf("result.Err: %v", got.Err)
				continue
			}

			if want, got := tb.contentHashes[got.Object], writers[writerIdx].crc32c; got != want {
				t.Fatalf("content crc32c does not match; got: %v, expected: %v", got, want)
			}

			if got.Attrs.Size != tb.objectSize {
				t.Errorf("expected object size %d, got %d", tb.objectSize, got.Attrs.Size)
			}
		}

		if len(results) != len(objects) {
			t.Errorf("expected to receive %d results, got %d results", len(objects), len(results))
		}
	})
}

func TestIntegration_DownloaderAsynchronous(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, c *storage.Client, tb downloadTestBucket) {
		objects := tb.objects

		d, err := NewDownloader(c, WithWorkers(2), WithCallbacks())
		if err != nil {
			t.Fatalf("NewDownloader: %v", err)
		}

		numCallbacks := 0
		callbackMu := sync.Mutex{}

		// Download objects.
		writers := make([]*testWriter, len(objects))
		for i, obj := range objects {
			i := i
			writers[i] = &testWriter{}
			if err := d.DownloadObject(ctx, &DownloadObjectInput{
				Bucket:      tb.bucket,
				Object:      obj,
				Destination: writers[i],
				Callback: func(got *DownloadOutput) {
					callbackMu.Lock()
					numCallbacks++
					callbackMu.Unlock()

					if got.Err != nil {
						t.Errorf("result.Err: %v", got.Err)
					}

					// Close the writer so we can check the contents.
					if err := writers[i].Close(); err != nil {
						t.Fatalf("testWriter.Close: %v", err)
					}

					if want, got := tb.contentHashes[got.Object], writers[i].crc32c; got != want {
						t.Fatalf("content crc32c does not match; got: %v, expected: %v", got, want)
					}

					if got.Attrs.Size != tb.objectSize {
						t.Errorf("expected object size %d, got %d", tb.objectSize, got.Attrs.Size)
					}
				},
			}); err != nil {
				t.Errorf("d.DownloadObject: %v", err)
			}
		}

		if _, err := d.WaitAndClose(); err != nil {
			t.Fatalf("d.WaitAndClose: %v", err)
		}

		if numCallbacks != len(objects) {
			t.Errorf("expected to receive %d results, got %d callbacks", len(objects), numCallbacks)
		}
	})
}

func TestIntegration_DownloaderErrorAsync(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, c *storage.Client, tb downloadTestBucket) {
		d, err := NewDownloader(c, WithWorkers(2), WithCallbacks())
		if err != nil {
			t.Fatalf("NewDownloader: %v", err)
		}

		// Keep track of the number of callbacks. Since the callbacks may happen
		// in parallel, we sync access to this variable.
		numCallbacks := 0
		callbackMu := sync.Mutex{}

		// Download an object with incorrect generation.
		if err := d.DownloadObject(ctx, &DownloadObjectInput{
			Bucket:      tb.bucket,
			Object:      tb.objects[0],
			Destination: &testWriter{},
			Conditions: &storage.Conditions{
				GenerationMatch: -10,
			},
			Callback: func(got *DownloadOutput) {
				callbackMu.Lock()
				numCallbacks++
				callbackMu.Unlock()

				// This will match both the expected http and grpc errors.
				wantErr := errors.Join(&googleapi.Error{Code: 412}, status.Error(codes.FailedPrecondition, ""))

				if !errorIs(got.Err, wantErr) {
					t.Errorf("mismatching errors: got %v, want %v", got.Err, wantErr)
				}
			},
		}); err != nil {
			t.Errorf("d.DownloadObject: %v", err)
		}

		// Download existing objects.
		writers := make([]*testWriter, len(tb.objects))
		for i, obj := range tb.objects {
			i := i
			writers[i] = &testWriter{}
			if err := d.DownloadObject(ctx, &DownloadObjectInput{
				Bucket:      tb.bucket,
				Object:      obj,
				Destination: writers[i],
				Callback: func(got *DownloadOutput) {
					callbackMu.Lock()
					numCallbacks++
					callbackMu.Unlock()

					if got.Err != nil {
						t.Errorf("result.Err: %v", got.Err)
					}

					// Close the writer so we can check the contents.
					if err := writers[i].Close(); err != nil {
						t.Fatalf("testWriter.Close: %v", err)
					}

					if want, got := tb.contentHashes[got.Object], writers[i].crc32c; got != want {
						t.Fatalf("content crc32c does not match; got: %v, expected: %v", got, want)
					}

					if got.Attrs.Size != tb.objectSize {
						t.Errorf("expected object size %d, got %d", tb.objectSize, got.Attrs.Size)
					}
				},
			}); err != nil {
				t.Errorf("d.DownloadObject: %v", err)
			}
		}

		// Download a nonexistent object.
		if err := d.DownloadObject(ctx, &DownloadObjectInput{
			Bucket:      tb.bucket,
			Object:      "not-written",
			Destination: &testWriter{},
			Callback: func(got *DownloadOutput) {
				callbackMu.Lock()
				numCallbacks++
				callbackMu.Unlock()

				// Check that the nonexistent object returned an error.
				if got.Err != storage.ErrObjectNotExist {
					t.Errorf("Object(%q) should not exist, err found to be %v", got.Object, got.Err)
				}
			},
		}); err != nil {
			t.Errorf("d.DownloadObject: %v", err)
		}

		// WaitAndClose should return an error since 2 of our downloads should have failed.
		_, err = d.WaitAndClose()
		if err == nil {
			t.Error("d.WaitAndClose should return an error, instead got nil")
		}

		// Check that both errors were returned.
		wantErrs := []error{errors.Join(&googleapi.Error{Code: 412}, status.Error(codes.FailedPrecondition, "")),
			storage.ErrObjectNotExist}

		for _, want := range wantErrs {
			if !errorIs(err, want) {
				t.Errorf("got error does not wrap expected error %q, got:\n%v", want, err)
			}
		}

		// We expect num objects callbacks + 2 for the errored calls.
		if want, got := len(tb.objects)+2, numCallbacks; want != got {
			t.Errorf("expected to receive %d results, got %d callbacks", want, got)
		}
	})
}

func TestIntegration_DownloaderTimeout(t *testing.T) {
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	ctx := context.Background()
	client, err := storage.NewClient(ctx)
	if err != nil {
		t.Fatalf("NewClient: %v", err)
	}
	t.Cleanup(func() { client.Close() })

	// Start a downloader.
	d, err := NewDownloader(client, WithPerOpTimeout(time.Nanosecond))
	if err != nil {
		t.Fatalf("NewDownloader: %v", err)
	}

	// Download an object.
	if err := d.DownloadObject(ctx, &DownloadObjectInput{
		Bucket:      httpTestBucket.bucket,
		Object:      httpTestBucket.objects[0],
		Destination: &testWriter{},
	}); err != nil {
		t.Errorf("d.DownloadObject: %v", err)
	}

	// WaitAndClose should return an error since the timeout is too short.
	results, err := d.WaitAndClose()
	if err == nil {
		t.Error("d.WaitAndClose should return an error, instead got nil")
	}

	// Check the result.
	got := results[0]

	// Check that the nonexistent object returned an error.
	if got.Err != context.DeadlineExceeded {
		t.Errorf("expected deadline exceeded error, got: %v", got.Err)
	}
}

func TestIntegration_DownloadShard(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, c *storage.Client, tb downloadTestBucket) {
		objectName := tb.objects[0]

		// Get expected Attrs.
		o := c.Bucket(tb.bucket).Object(objectName)
		r, err := o.NewReader(ctx)
		if err != nil {
			t.Fatalf("o.Attrs: %v", err)
		}

		incorrectGen := r.Attrs.Generation - 1

		cancelledCtx, cancel := context.WithCancel(ctx)
		cancel()

		for _, test := range []struct {
			desc    string
			timeout time.Duration
			in      *DownloadObjectInput
			want    *DownloadOutput
		}{
			{
				desc: "basic input",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
				},
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Attrs:  &r.Attrs,
				},
			},
			{
				desc: "range",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
					Range: &DownloadRange{
						Offset: tb.objectSize - 5,
						Length: -1,
					},
				},
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Range: &DownloadRange{
						Offset: tb.objectSize - 5,
						Length: -1,
					},
					Attrs: &storage.ReaderObjectAttrs{
						Size:            tb.objectSize,
						StartOffset:     tb.objectSize - 5,
						ContentType:     r.Attrs.ContentType,
						ContentEncoding: r.Attrs.ContentEncoding,
						CacheControl:    r.Attrs.CacheControl,
						LastModified:    r.Attrs.LastModified,
						Generation:      r.Attrs.Generation,
						Metageneration:  r.Attrs.Metageneration,
					},
				},
			},
			{
				desc: "incorrect generation",
				in: &DownloadObjectInput{
					Bucket:     tb.bucket,
					Object:     objectName,
					Generation: &incorrectGen,
				},
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Err:    storage.ErrObjectNotExist,
				},
			},
			{
				desc: "conditions: generationmatch",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
					Conditions: &storage.Conditions{
						GenerationMatch: r.Attrs.Generation,
					},
				},
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Attrs:  &r.Attrs,
				},
			},
			{
				desc: "conditions do not hold",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
					Conditions: &storage.Conditions{
						GenerationMatch: incorrectGen,
					},
				},
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Err:    errors.Join(&googleapi.Error{Code: 412}, status.Error(codes.FailedPrecondition, "")),
				},
			},
			{
				desc: "timeout",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
				},
				timeout: time.Nanosecond,
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Err:    context.DeadlineExceeded,
				},
			},
			{
				desc: "cancelled ctx",
				in: &DownloadObjectInput{
					Bucket: tb.bucket,
					Object: objectName,
					ctx:    cancelledCtx,
				},
				timeout: time.Nanosecond,
				want: &DownloadOutput{
					Bucket: tb.bucket,
					Object: objectName,
					Err:    context.Canceled,
				},
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				w := &testWriter{}

				test.in.Destination = w

				if test.in.ctx == nil {
					test.in.ctx = ctx
				}

				got := test.in.downloadShard(c, test.timeout)

				if got.Bucket != test.want.Bucket || got.Object != test.want.Object {
					t.Errorf("wanted bucket %q, object %q, got: %q, %q", test.want.Bucket, test.want.Object, got.Bucket, got.Object)
				}

				if diff := cmp.Diff(got.Range, test.want.Range); diff != "" {
					t.Errorf("DownloadOutput.Range: got(-) vs. want(+): %v", diff)
				}

				if diff := cmp.Diff(got.Attrs, test.want.Attrs); diff != "" {
					t.Errorf("DownloadOutput.Attrs: got(-) vs. want(+): %v", diff)
				}

				if !errorIs(got.Err, test.want.Err) {
					t.Errorf("mismatching errors: got %v, want %v", got.Err, test.want.Err)
				}
			})
		}
	})
}

// errorIs is equivalent to errors.Is, except that it additionally will return
// true if err and targetErr are googleapi.Errors with identical error codes,
// or if both errors have the same gRPC status code.
func errorIs(err error, targetErr error) bool {
	if errors.Is(err, targetErr) {
		return true
	}

	// Check http
	var e, targetE *googleapi.Error
	if errors.As(err, &e) && errors.As(targetErr, &targetE) {
		return e.Code == targetE.Code
	}

	// Check grpc errors
	if status.Code(err) != codes.Unknown {
		return status.Code(err) == status.Code(targetErr)
	}

	return false
}

// generateRandomFileInGCS uploads a file with random contents to GCS and returns
// the crc32c hash of the contents.
func generateFileInGCS(ctx context.Context, o *storage.ObjectHandle, size int64) (uint32, error) {
	w := o.Retryer(storage.WithPolicy(storage.RetryAlways)).NewWriter(ctx)

	crc32cHash := crc32.New(crc32.MakeTable(crc32.Castagnoli))
	mw := io.MultiWriter(w, crc32cHash)

	if _, err := io.CopyN(mw, crand.Reader, size); err != nil {
		w.Close()
		return 0, err
	}
	return crc32cHash.Sum32(), w.Close()
}

// randomInt64 returns a value in the closed interval [min, max].
// That is, the endpoints are possible return values.
func randomInt64(min, max int64) int64 {
	if min > max {
		log.Fatalf("min cannot be larger than max; min: %d max: %d", min, max)
	}
	return rand.Int63n(max-min+1) + min
}

// TODO: once we provide a DownloaderBuffer that implements WriterAt in the
// library, we can use that instead.
type testWriter struct {
	b      []byte
	crc32c uint32
	bufs   [][]byte // temp bufs that will be joined on Close()
}

// Close must be called to finalize the buffer
func (tw *testWriter) Close() error {
	tw.b = bytes.Join(tw.bufs, nil)
	crc := crc32.New(crc32.MakeTable(crc32.Castagnoli))

	_, err := io.Copy(crc, bytes.NewReader(tw.b))
	tw.crc32c = crc.Sum32()
	return err
}

func (tw *testWriter) WriteAt(b []byte, offset int64) (n int, err error) {
	// TODO: use the offset. This is fine for now since reads are not yet sharded.
	copiedB := make([]byte, len(b))
	copy(copiedB, b)
	tw.bufs = append(tw.bufs, copiedB)

	return len(b), nil
}

func deleteExpiredBuckets(prefix string) error {
	if testing.Short() {
		return nil
	}

	ctx := context.Background()
	client, err := storage.NewClient(ctx)
	if err != nil {
		return fmt.Errorf("NewClient: %v", err)
	}

	projectID := testutil.ProjID()
	it := client.Buckets(ctx, projectID)
	it.Prefix = prefix
	for {
		bktAttrs, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			return err
		}
		if time.Since(bktAttrs.Created) > bucketExpiryAge {
			log.Printf("deleting bucket %q, which is more than %s old", bktAttrs.Name, bucketExpiryAge)
			if err := killBucket(ctx, client, bktAttrs.Name); err != nil {
				return err
			}
		}
	}
	return nil
}

// killBucket deletes a bucket and all its objects.
func killBucket(ctx context.Context, client *storage.Client, bucketName string) error {
	bkt := client.Bucket(bucketName)
	// Bucket must be empty to delete.
	it := bkt.Objects(ctx, nil)
	for {
		objAttrs, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			return err
		}
		if err := bkt.Object(objAttrs.Name).Delete(ctx); err != nil {
			return fmt.Errorf("deleting %q: %v", bucketName+"/"+objAttrs.Name, err)
		}
	}
	// GCS is eventually consistent, so this delete may fail because the
	// replica still sees an object in the bucket. We log the error and expect
	// a later test run to delete the bucket.
	if err := bkt.Delete(ctx); err != nil {
		log.Printf("deleting %q: %v", bucketName, err)
	}
	return nil
}

// downloadTestBucket provides a bucket that can be reused for tests that only
// download from the bucket.
type downloadTestBucket struct {
	bucket        string
	objects       []string
	contentHashes map[string]uint32
	objectSize    int64
}

// Create initializes the downloadTestBucket, creating a bucket and populating
// objects in it. All objects are of the same size but with different contents
// and can be mapped to their respective crc32c hash in contentHashes.
func (tb *downloadTestBucket) Create(prefix string) error {
	if testing.Short() {
		return nil
	}
	ctx := context.Background()

	tb.bucket = prefix + uidSpace.New()
	tb.objectSize = randomInt64(200, 1024*1024)
	tb.objects = []string{
		"obj1",
		"obj2",
		"obj/with/slashes",
		"obj/",
		"./obj",
		"!#$&'()*+,/:;=,?@,[] and spaces",
	}
	tb.contentHashes = make(map[string]uint32)

	client, err := storage.NewClient(ctx)
	if err != nil {
		return fmt.Errorf("NewClient: %v", err)
	}
	defer client.Close()

	b := client.Bucket(tb.bucket)
	if err := b.Create(ctx, testutil.ProjID(), nil); err != nil {
		return fmt.Errorf("bucket(%q).Create: %v", tb.bucket, err)
	}

	// Write objects.
	for _, obj := range tb.objects {
		crc, err := generateFileInGCS(ctx, b.Object(obj), tb.objectSize)
		if err != nil {
			return fmt.Errorf("generateFileInGCS: %v", err)
		}
		tb.contentHashes[obj] = crc

	}
	return nil
}

// Cleanup deletes the objects and bucket created in Create.
func (tb *downloadTestBucket) Cleanup() error {
	if testing.Short() {
		return nil
	}
	ctx := context.Background()

	client, err := storage.NewClient(ctx)
	if err != nil {
		return fmt.Errorf("NewClient: %v", err)
	}
	defer client.Close()

	b := client.Bucket(tb.bucket)

	for _, obj := range tb.objects {
		if err := b.Object(obj).Delete(ctx); err != nil {
			return fmt.Errorf("object.Delete: %v", err)
		}
	}

	return b.Delete(ctx)
}

// multiTransportTest initializes fresh clients for each transport, then runs
// given testing function using each transport-specific client, supplying the
// test function with the sub-test instance, the context it was given, a test
// bucket and the client to use.
func multiTransportTest(ctx context.Context, t *testing.T, test func(*testing.T, context.Context, *storage.Client, downloadTestBucket)) {
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	clients, err := initTransportClients(ctx)
	if err != nil {
		t.Fatalf("init clients: %v", err)
	}

	for transport, client := range clients {
		t.Run(transport, func(t *testing.T) {
			t.Cleanup(func() {
				client.Close()
			})

			testBucket := httpTestBucket

			if transport == "grpc" {
				testBucket = grpcTestBucket
			}

			test(t, ctx, client, testBucket)
		})
	}
}

func initTransportClients(ctx context.Context) (map[string]*storage.Client, error) {
	c, err := storage.NewClient(ctx)
	if err != nil {
		return nil, err
	}

	gc, err := storage.NewGRPCClient(ctx)
	if err != nil {
		return nil, err
	}

	return map[string]*storage.Client{
		"http": c,
		"grpc": gc,
	}, nil
}
