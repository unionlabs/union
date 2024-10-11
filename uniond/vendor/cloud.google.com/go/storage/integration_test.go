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
	"bytes"
	"compress/gzip"
	"context"
	"crypto"
	"crypto/md5"
	cryptorand "crypto/rand"
	"crypto/rsa"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"hash/crc32"
	"io"
	"log"
	"math"
	"math/rand"
	"mime/multipart"
	"net/http"
	"net/http/httputil"
	"os"
	"sort"
	"strconv"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/httpreplay"
	"cloud.google.com/go/iam"
	"cloud.google.com/go/iam/apiv1/iampb"
	"cloud.google.com/go/internal/testutil"
	"cloud.google.com/go/internal/uid"
	control "cloud.google.com/go/storage/control/apiv2"
	"cloud.google.com/go/storage/control/apiv2/controlpb"
	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/googleapis/gax-go/v2/apierror"
	"golang.org/x/oauth2/google"
	"google.golang.org/api/googleapi"
	"google.golang.org/api/iterator"
	itesting "google.golang.org/api/iterator/testing"
	"google.golang.org/api/option"
	"google.golang.org/api/transport"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type skipTransportTestKey string

const (
	testPrefix     = "go-integration-test"
	replayFilename = "storage.replay"
	// TODO(jba): move to testutil, factor out from firestore/integration_test.go.
	envFirestoreProjID     = "GCLOUD_TESTS_GOLANG_FIRESTORE_PROJECT_ID"
	envFirestorePrivateKey = "GCLOUD_TESTS_GOLANG_FIRESTORE_KEY"
	grpcTestPrefix         = "golang-grpc-test"
)

var (
	record = flag.Bool("record", false, "record RPCs")

	uidSpace        *uid.Space
	uidSpaceObjects *uid.Space
	bucketName      string
	grpcBucketName  string
	// Use our own random number generator to isolate the sequence of random numbers from
	// other packages. This makes it possible to use HTTP replay and draw the same sequence
	// of numbers as during recording.
	rng           *rand.Rand
	newTestClient func(ctx context.Context, opts ...option.ClientOption) (*Client, error)
	replaying     bool
	testTime      time.Time
	controlClient *control.StorageControlClient
)

func TestMain(m *testing.M) {
	grpc.EnableTracing = true
	cleanup := initIntegrationTest()
	cleanupEmulatorClients := initEmulatorClients()
	exit := m.Run()
	if err := cleanup(); err != nil {
		// Don't fail the test if cleanup fails.
		log.Printf("Post-test cleanup failed: %v", err)
	}
	if err := cleanupEmulatorClients(); err != nil {
		// Don't fail the test if cleanup fails.
		log.Printf("Post-test cleanup failed for emulator clients: %v", err)
	}

	os.Exit(exit)
}

// If integration tests will be run, create a unique bucket for them.
// Also, set newTestClient to handle record/replay.
// Return a cleanup function.
func initIntegrationTest() func() error {
	flag.Parse() // needed for testing.Short()
	switch {
	case testing.Short() && *record:
		log.Fatal("cannot combine -short and -record")
		return nil

	case testing.Short() && httpreplay.Supported() && testutil.CanReplay(replayFilename) && testutil.ProjID() != "":
		// go test -short with a replay file will replay the integration tests, if
		// the appropriate environment variables have been set.
		replaying = true
		httpreplay.DebugHeaders()
		replayer, err := httpreplay.NewReplayer(replayFilename)
		if err != nil {
			log.Fatal(err)
		}
		var t time.Time
		if err := json.Unmarshal(replayer.Initial(), &t); err != nil {
			log.Fatal(err)
		}
		initUIDsAndRand(t)
		newTestClient = func(ctx context.Context, _ ...option.ClientOption) (*Client, error) {
			hc, err := replayer.Client(ctx) // no creds needed
			if err != nil {
				return nil, err
			}
			return NewClient(ctx, option.WithHTTPClient(hc))
		}
		log.Printf("replaying from %s", replayFilename)
		return func() error { return replayer.Close() }

	case testing.Short():
		// go test -short without a replay file skips the integration tests.
		if testutil.CanReplay(replayFilename) && testutil.ProjID() != "" {
			log.Print("replay not supported for Go versions before 1.8")
		}
		newTestClient = nil
		return func() error { return nil }

	default: // Run integration tests against a real backend.
		now := time.Now().UTC()
		initUIDsAndRand(now)
		var cleanup func() error
		if *record && httpreplay.Supported() {
			// Remember the time for replay.
			nowBytes, err := json.Marshal(now)
			if err != nil {
				log.Fatal(err)
			}
			recorder, err := httpreplay.NewRecorder(replayFilename, nowBytes)
			if err != nil {
				log.Fatalf("could not record: %v", err)
			}
			newTestClient = func(ctx context.Context, opts ...option.ClientOption) (*Client, error) {
				hc, err := recorder.Client(ctx, opts...)
				if err != nil {
					return nil, err
				}
				return NewClient(ctx, option.WithHTTPClient(hc))
			}
			cleanup = func() error {
				err1 := cleanupBuckets()
				err2 := recorder.Close()
				if err1 != nil {
					return err1
				}
				return err2
			}
			log.Printf("recording to %s", replayFilename)
		} else {
			if *record {
				log.Print("record not supported for Go versions before 1.8")
			}
			newTestClient = NewClient
			cleanup = cleanupBuckets
		}
		ctx := context.Background()
		client, err := newTestClient(ctx)
		if err != nil {
			log.Fatalf("NewClient: %v", err)
		}
		if client == nil {
			return func() error { return nil }
		}
		defer client.Close()
		// Create storage control client; in some cases this is needed for test
		// setup and cleanup.
		controlClient, err = control.NewStorageControlClient(ctx)
		if err != nil {
			log.Fatalf("NewStorageControlClient: %v", err)
		}
		if err := client.Bucket(bucketName).Create(ctx, testutil.ProjID(), nil); err != nil {
			log.Fatalf("creating bucket %q: %v", bucketName, err)
		}
		if err := client.Bucket(grpcBucketName).Create(ctx, testutil.ProjID(), nil); err != nil {
			log.Fatalf("creating bucket %q: %v", grpcBucketName, err)
		}
		return cleanup
	}
}

func initUIDsAndRand(t time.Time) {
	uidSpace = uid.NewSpace("", &uid.Options{Time: t})
	bucketName = testPrefix + uidSpace.New()
	uidSpaceObjects = uid.NewSpace("obj", &uid.Options{Time: t})
	grpcBucketName = grpcTestPrefix + uidSpace.New()
	// Use our own random source, to avoid other parts of the program taking
	// random numbers from the global source and putting record and replay
	// out of sync.
	rng = testutil.NewRand(t)
	testTime = t
}

// testConfig returns the Client used to access GCS. testConfig skips
// the current test if credentials are not available or when being run
// in Short mode.
func testConfig(ctx context.Context, t *testing.T, opts ...option.ClientOption) *Client {
	if testing.Short() && !replaying {
		t.Skip("Integration tests skipped in short mode")
	}
	client, err := newTestClient(ctx, opts...)
	if err != nil {
		t.Fatalf("NewClient: %v", err)
	}
	if client == nil {
		t.Skip("Integration tests skipped. See CONTRIBUTING.md for details")
	}
	return client
}

// testConfigGPRC returns a gRPC-based client to access GCS. testConfigGRPC
// skips the curent test when being run in Short mode.
func testConfigGRPC(ctx context.Context, t *testing.T, opts ...option.ClientOption) (gc *Client) {
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	gc, err := NewGRPCClient(ctx, opts...)
	if err != nil {
		t.Fatalf("NewGRPCClient: %v", err)
	}

	return
}

// initTransportClients initializes Storage clients for each supported transport.
func initTransportClients(ctx context.Context, t *testing.T, opts ...option.ClientOption) map[string]*Client {
	withJSON := append(opts, WithJSONReads())
	return map[string]*Client{
		"http": testConfig(ctx, t, opts...),
		"grpc": testConfigGRPC(ctx, t, opts...),
		// TODO: remove jsonReads when support for XML reads is dropped
		"jsonReads": testConfig(ctx, t, withJSON...),
	}
}

// multiTransportTest initializes fresh clients for each transport, then runs
// given testing function using each transport-specific client, supplying the
// test function with the sub-test instance, the context it was given, the name
// of an existing bucket to use, a bucket name to use for bucket creation, and
// the client to use.
func multiTransportTest(ctx context.Context, t *testing.T,
	test func(*testing.T, context.Context, string, string, *Client),
	opts ...option.ClientOption) {
	for transport, client := range initTransportClients(ctx, t, opts...) {
		t.Run(transport, func(t *testing.T) {
			t.Cleanup(func() {
				client.Close()
			})

			if reason := ctx.Value(skipTransportTestKey(transport)); reason != nil {
				t.Skip("transport", fmt.Sprintf("%q", transport), "explicitly skipped:", reason)
			}

			bucket := bucketName
			prefix := testPrefix
			if transport == "grpc" {
				bucket = grpcBucketName
				prefix = grpcTestPrefix
			}

			// Don't let any individual test run more than 5 minutes. This eases debugging if
			// test runs get stalled out.
			ctx, cancel := context.WithTimeout(ctx, 5*time.Minute)
			t.Cleanup(func() {
				cancel()
			})

			test(t, ctx, bucket, prefix, client)
		})
	}
}

// Use two different reading funcs for some tests to cover both Read and WriteTo.
type readCase struct {
	desc     string
	readFunc (func(io.Reader) ([]byte, error))
}

var readCases = []readCase{
	{
		desc:     "Read",
		readFunc: io.ReadAll,
	},
	{
		desc: "WriteTo",
		readFunc: func(r io.Reader) ([]byte, error) {
			b := new(bytes.Buffer)
			_, err := io.Copy(b, r)
			return b.Bytes(), err
		},
	},
}

func TestIntegration_BucketCreateDelete(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		projectID := testutil.ProjID()

		labels := map[string]string{
			"l1":    "v1",
			"empty": "",
		}

		lifecycle := Lifecycle{
			Rules: []LifecycleRule{{
				Action: LifecycleAction{
					Type:         SetStorageClassAction,
					StorageClass: "NEARLINE",
				},
				Condition: LifecycleCondition{
					AgeInDays:             10,
					Liveness:              Archived,
					CreatedBefore:         time.Date(2017, 1, 1, 0, 0, 0, 0, time.UTC),
					MatchesStorageClasses: []string{"STANDARD"},
					NumNewerVersions:      3,
				},
			}, {
				Action: LifecycleAction{
					Type:         SetStorageClassAction,
					StorageClass: "ARCHIVE",
				},
				Condition: LifecycleCondition{
					CustomTimeBefore:      time.Date(2020, 1, 2, 0, 0, 0, 0, time.UTC),
					DaysSinceCustomTime:   20,
					Liveness:              Live,
					MatchesStorageClasses: []string{"STANDARD"},
				},
			}, {
				Action: LifecycleAction{
					Type: DeleteAction,
				},
				Condition: LifecycleCondition{
					DaysSinceNoncurrentTime: 30,
					Liveness:                Live,
					NoncurrentTimeBefore:    time.Date(2017, 1, 1, 0, 0, 0, 0, time.UTC),
					MatchesStorageClasses:   []string{"NEARLINE"},
					NumNewerVersions:        10,
				},
			}, {
				Action: LifecycleAction{
					Type: DeleteAction,
				},
				Condition: LifecycleCondition{
					AgeInDays:        10,
					MatchesPrefix:    []string{"testPrefix"},
					MatchesSuffix:    []string{"testSuffix"},
					NumNewerVersions: 3,
				},
			}, {
				Action: LifecycleAction{
					Type: DeleteAction,
				},
				Condition: LifecycleCondition{
					AllObjects: true,
				},
			}},
		}

		// testedAttrs are the bucket attrs directly compared in this test
		type testedAttrs struct {
			StorageClass          string
			VersioningEnabled     bool
			LocationType          string
			Labels                map[string]string
			Location              string
			Lifecycle             Lifecycle
			CustomPlacementConfig *CustomPlacementConfig
		}

		for _, test := range []struct {
			name      string
			attrs     *BucketAttrs
			wantAttrs testedAttrs
		}{
			{
				name:  "no attrs",
				attrs: nil,
				wantAttrs: testedAttrs{
					StorageClass:      "STANDARD",
					VersioningEnabled: false,
					LocationType:      "multi-region",
					Location:          "US",
				},
			},
			{
				name: "with attrs",
				attrs: &BucketAttrs{
					StorageClass:      "NEARLINE",
					VersioningEnabled: true,
					Labels:            labels,
					Lifecycle:         lifecycle,
					Location:          "SOUTHAMERICA-EAST1",
				},
				wantAttrs: testedAttrs{
					StorageClass:      "NEARLINE",
					VersioningEnabled: true,
					Labels:            labels,
					Location:          "SOUTHAMERICA-EAST1",
					LocationType:      "region",
					Lifecycle:         lifecycle,
				},
			},
			{
				name: "dual-region",
				attrs: &BucketAttrs{
					Location: "US",
					CustomPlacementConfig: &CustomPlacementConfig{
						DataLocations: []string{"US-EAST1", "US-WEST1"},
					},
				},
				wantAttrs: testedAttrs{
					Location:     "US",
					LocationType: "dual-region",
					StorageClass: "STANDARD",
					CustomPlacementConfig: &CustomPlacementConfig{
						DataLocations: []string{"US-EAST1", "US-WEST1"},
					},
				},
			},
		} {
			t.Run(test.name, func(t *testing.T) {
				newBucketName := prefix + uidSpace.New()
				b := client.Bucket(newBucketName)

				if err := b.Create(ctx, projectID, test.attrs); err != nil {
					t.Fatalf("BucketHandle.Create(%q): %v", newBucketName, err)
				}

				gotAttrs, err := b.Attrs(ctx)
				if err != nil {
					t.Fatalf("BucketHandle.Attrs(%q): %v", newBucketName, err)
				}

				// All newly created buckets should conform to the following:
				if gotAttrs.MetaGeneration != 1 {
					t.Errorf("metageneration: got %d, should be 1", gotAttrs.MetaGeneration)
				}
				if gotAttrs.ProjectNumber == 0 {
					t.Errorf("got a zero ProjectNumber")
				}

				// Test specific wanted bucket attrs
				if gotAttrs.VersioningEnabled != test.wantAttrs.VersioningEnabled {
					t.Errorf("versioning enabled: got %t, want %t", gotAttrs.VersioningEnabled, test.wantAttrs.VersioningEnabled)
				}
				if got, want := gotAttrs.Labels, test.wantAttrs.Labels; !testutil.Equal(got, want) {
					t.Errorf("labels: got %v, want %v", got, want)
				}
				if diff := cmp.Diff(gotAttrs.Lifecycle, test.wantAttrs.Lifecycle); diff != "" {
					t.Errorf("lifecycle: diff got vs. want: %v", diff)
				}
				if gotAttrs.LocationType != test.wantAttrs.LocationType {
					t.Errorf("location type: got %s, want %s", gotAttrs.LocationType, test.wantAttrs.LocationType)
				}
				if gotAttrs.StorageClass != test.wantAttrs.StorageClass {
					t.Errorf("storage class: got %s, want %s", gotAttrs.StorageClass, test.wantAttrs.StorageClass)
				}
				if gotAttrs.Location != test.wantAttrs.Location {
					t.Errorf("location: got %s, want %s", gotAttrs.Location, test.wantAttrs.Location)
				}
				if got, want := gotAttrs.CustomPlacementConfig, test.wantAttrs.CustomPlacementConfig; !testutil.Equal(got, want) {
					t.Errorf("customPlacementConfig: \ngot\t%v\nwant\t%v", got, want)
				}

				// Delete the bucket and check that the deletion was succesful
				if err := b.Delete(ctx); err != nil {
					t.Fatalf("BucketHandle.Delete(%q): %v", newBucketName, err)
				}
				_, err = b.Attrs(ctx)
				if err != ErrBucketNotExist {
					t.Fatalf("expected ErrBucketNotExist, got %v", err)
				}
			})
		}
	})
}

func TestIntegration_BucketLifecycle(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		wantLifecycle := Lifecycle{
			Rules: []LifecycleRule{
				{
					Action:    LifecycleAction{Type: AbortIncompleteMPUAction},
					Condition: LifecycleCondition{AgeInDays: 30},
				},
				{
					Action:    LifecycleAction{Type: DeleteAction},
					Condition: LifecycleCondition{AllObjects: true},
				},
			},
		}

		bucket := client.Bucket(prefix + uidSpace.New())

		// Create bucket with lifecycle rules
		h.mustCreate(bucket, testutil.ProjID(), &BucketAttrs{
			Lifecycle: wantLifecycle,
		})
		defer h.mustDeleteBucket(bucket)

		attrs := h.mustBucketAttrs(bucket)
		if !testutil.Equal(attrs.Lifecycle, wantLifecycle) {
			t.Fatalf("got %v, want %v", attrs.Lifecycle, wantLifecycle)
		}

		// Remove lifecycle rules
		ua := BucketAttrsToUpdate{Lifecycle: &Lifecycle{}}
		attrs = h.mustUpdateBucket(bucket, ua, attrs.MetaGeneration)
		if !testutil.Equal(attrs.Lifecycle, Lifecycle{}) {
			t.Fatalf("got %v, want %v", attrs.Lifecycle, Lifecycle{})
		}

		// Update bucket with a lifecycle rule
		ua = BucketAttrsToUpdate{Lifecycle: &wantLifecycle}
		attrs = h.mustUpdateBucket(bucket, ua, attrs.MetaGeneration)
		if !testutil.Equal(attrs.Lifecycle, wantLifecycle) {
			t.Fatalf("got %v, want %v", attrs.Lifecycle, wantLifecycle)
		}
	})
}

func TestIntegration_BucketUpdate(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		b := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(b, testutil.ProjID(), nil)
		defer h.mustDeleteBucket(b)

		attrs := h.mustBucketAttrs(b)
		if attrs.VersioningEnabled {
			t.Fatal("bucket should not have versioning by default")
		}
		if len(attrs.Labels) > 0 {
			t.Fatal("bucket should not have labels initially")
		}

		// Turn on versioning, add some labels.
		ua := BucketAttrsToUpdate{VersioningEnabled: true}
		ua.SetLabel("l1", "v1")
		ua.SetLabel("empty", "")
		attrs = h.mustUpdateBucket(b, ua, attrs.MetaGeneration)
		if !attrs.VersioningEnabled {
			t.Fatal("should have versioning now")
		}
		wantLabels := map[string]string{
			"l1":    "v1",
			"empty": "",
		}
		if !testutil.Equal(attrs.Labels, wantLabels) {
			t.Fatalf("add labels: got %v, want %v", attrs.Labels, wantLabels)
		}

		// Turn off versioning again; add and remove some more labels.
		ua = BucketAttrsToUpdate{VersioningEnabled: false}
		ua.SetLabel("l1", "v2")   // update
		ua.SetLabel("new", "new") // create
		ua.DeleteLabel("empty")   // delete
		ua.DeleteLabel("absent")  // delete non-existent
		attrs = h.mustUpdateBucket(b, ua, attrs.MetaGeneration)
		if attrs.VersioningEnabled {
			t.Fatal("should have versioning off")
		}
		wantLabels = map[string]string{
			"l1":  "v2",
			"new": "new",
		}
		if !testutil.Equal(attrs.Labels, wantLabels) {
			t.Fatalf("got %v, want %v", attrs.Labels, wantLabels)
		}

		// Configure a lifecycle
		wantLifecycle := Lifecycle{
			Rules: []LifecycleRule{
				{
					Action: LifecycleAction{Type: "Delete"},
					Condition: LifecycleCondition{
						AgeInDays:     30,
						MatchesPrefix: []string{"testPrefix"},
						MatchesSuffix: []string{"testSuffix"},
					},
				},
			},
		}
		ua = BucketAttrsToUpdate{Lifecycle: &wantLifecycle}
		attrs = h.mustUpdateBucket(b, ua, attrs.MetaGeneration)
		if !testutil.Equal(attrs.Lifecycle, wantLifecycle) {
			t.Fatalf("got %v, want %v", attrs.Lifecycle, wantLifecycle)
		}
		// Check that StorageClass has "STANDARD" value for unset field by default
		// before passing new value.
		wantStorageClass := "STANDARD"
		if !testutil.Equal(attrs.StorageClass, wantStorageClass) {
			t.Fatalf("got %v, want %v", attrs.StorageClass, wantStorageClass)
		}
		wantStorageClass = "NEARLINE"
		ua = BucketAttrsToUpdate{StorageClass: wantStorageClass}
		attrs = h.mustUpdateBucket(b, ua, attrs.MetaGeneration)
		if !testutil.Equal(attrs.StorageClass, wantStorageClass) {
			t.Fatalf("got %v, want %v", attrs.StorageClass, wantStorageClass)
		}

		// Empty update should succeed without changing the bucket.
		gotAttrs, err := b.Update(ctx, BucketAttrsToUpdate{})
		if err != nil {
			t.Fatalf("empty update: %v", err)
		}
		if !testutil.Equal(attrs, gotAttrs) {
			t.Fatalf("empty update: got %v, want %v", gotAttrs, attrs)
		}
	})
}

func TestIntegration_BucketPolicyOnly(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer h.mustDeleteBucket(bkt)

		// Insert an object with custom ACL.
		o := bkt.Object("bucketPolicyOnly")
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()
		wc := o.NewWriter(ctx)
		wc.ContentType = "text/plain"
		h.mustWrite(wc, []byte("test"))
		a := o.ACL()
		aclEntity := ACLEntity("user-test@example.com")
		err := a.Set(ctx, aclEntity, RoleReader)
		if err != nil {
			t.Fatalf("set ACL failed: %v", err)
		}

		// Enable BucketPolicyOnly.
		ua := BucketAttrsToUpdate{BucketPolicyOnly: &BucketPolicyOnly{Enabled: true}}
		attrs := h.mustUpdateBucket(bkt, ua, h.mustBucketAttrs(bkt).MetaGeneration)
		if got, want := attrs.BucketPolicyOnly.Enabled, true; got != want {
			t.Fatalf("got %v, want %v", got, want)
		}
		if got := attrs.BucketPolicyOnly.LockedTime; got.IsZero() {
			t.Fatal("got a zero time value, want a populated value")
		}

		// Confirm BucketAccessControl returns error, since we cannot get legacy ACL
		// for a bucket that has uniform bucket-level access.

		// Metadata updates may be delayed up to 10s. Since we expect an error from
		// this call, we retry on a nil error until we get the non-retryable error
		// that we are expecting.
		ctxWithTimeout, cancelCtx := context.WithTimeout(ctx, time.Second*10)
		b := bkt.Retryer(WithErrorFunc(retryOnNilAndTransientErrs))
		_, err = b.ACL().List(ctxWithTimeout)
		cancelCtx()
		if err == nil {
			t.Errorf("ACL.List: expected bucket ACL list to fail")
		}

		// Confirm ObjectAccessControl returns error, for same reason as above.
		ctxWithTimeout, cancelCtx = context.WithTimeout(ctx, time.Second*10)
		_, err = o.Retryer(WithErrorFunc(retryOnNilAndTransientErrs)).ACL().List(ctxWithTimeout)
		cancelCtx()
		if err == nil {
			t.Errorf("ACL.List: expected object ACL list to fail")
		}

		// Disable BucketPolicyOnly.
		ua = BucketAttrsToUpdate{BucketPolicyOnly: &BucketPolicyOnly{Enabled: false}}
		attrs = h.mustUpdateBucket(bkt, ua, attrs.MetaGeneration)
		if got, want := attrs.BucketPolicyOnly.Enabled, false; got != want {
			t.Fatalf("attrs.BucketPolicyOnly.Enabled: got %v, want %v", got, want)
		}

		// Check that the object ACL rules are the same.

		// Metadata updates may be delayed up to 10s. Before that, we can get a 400
		// indicating that uniform bucket-level access is still enabled in HTTP.
		// We need to retry manually as GRPC will not error but provide empty ACL.
		var acl []ACLRule
		err = retry(ctx, func() error {
			var err error
			acl, err = o.ACL().List(ctx)
			if err != nil {
				return fmt.Errorf("ACL.List: object ACL list failed: %v", err)
			}
			return nil
		}, func() error {
			if !containsACLRule(acl, entityRoleACL{aclEntity, RoleReader}) {
				return fmt.Errorf("containsACL: expected ACL %v to include custom ACL entity %v", acl, entityRoleACL{aclEntity, RoleReader})
			}
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}
	})
}

func TestIntegration_UniformBucketLevelAccess(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}
		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer h.mustDeleteBucket(bkt)

		// Insert an object with custom ACL.
		o := bkt.Object("uniformBucketLevelAccess")
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()
		wc := o.NewWriter(ctx)
		wc.ContentType = "text/plain"
		h.mustWrite(wc, []byte("test"))
		a := o.ACL()
		aclEntity := ACLEntity("user-test@example.com")
		err := a.Set(ctx, aclEntity, RoleReader)
		if err != nil {
			t.Fatalf("set ACL failed: %v", err)
		}

		// Enable UniformBucketLevelAccess.
		ua := BucketAttrsToUpdate{UniformBucketLevelAccess: &UniformBucketLevelAccess{Enabled: true}}
		attrs := h.mustUpdateBucket(bkt, ua, h.mustBucketAttrs(bkt).MetaGeneration)
		if got, want := attrs.UniformBucketLevelAccess.Enabled, true; got != want {
			t.Fatalf("got %v, want %v", got, want)
		}
		if got := attrs.UniformBucketLevelAccess.LockedTime; got.IsZero() {
			t.Fatal("got a zero time value, want a populated value")
		}

		// Confirm BucketAccessControl returns error.
		// We retry on nil to account for propagation delay in metadata update.
		ctxWithTimeout, cancelCtx := context.WithTimeout(ctx, time.Second*10)
		b := bkt.Retryer(WithErrorFunc(retryOnNilAndTransientErrs))
		_, err = b.ACL().List(ctxWithTimeout)
		cancelCtx()
		if err == nil {
			t.Errorf("ACL.List: expected bucket ACL list to fail")
		}

		// Confirm ObjectAccessControl returns error.
		ctxWithTimeout, cancelCtx = context.WithTimeout(ctx, time.Second*10)
		_, err = o.Retryer(WithErrorFunc(retryOnNilAndTransientErrs)).ACL().List(ctxWithTimeout)
		cancelCtx()
		if err == nil {
			t.Errorf("ACL.List: expected object ACL list to fail")
		}

		// Disable UniformBucketLevelAccess.
		ua = BucketAttrsToUpdate{UniformBucketLevelAccess: &UniformBucketLevelAccess{Enabled: false}}
		attrs = h.mustUpdateBucket(bkt, ua, attrs.MetaGeneration)
		if got, want := attrs.UniformBucketLevelAccess.Enabled, false; got != want {
			t.Fatalf("got %v, want %v", got, want)
		}

		// Metadata updates may be delayed up to 10s. Before that, we can get a 400
		// indicating that uniform bucket-level access is still enabled in HTTP.
		// We need to retry manually as GRPC will not error but provide empty ACL.
		var acl []ACLRule
		err = retry(ctx, func() error {
			var err error
			acl, err = o.ACL().List(ctx)
			if err != nil {
				return fmt.Errorf("ACL.List: object ACL list failed: %v", err)
			}
			return nil
		}, func() error {
			if !containsACLRule(acl, entityRoleACL{aclEntity, RoleReader}) {
				return fmt.Errorf("containsACL: expected ACL %v to include custom ACL entity %v", acl, entityRoleACL{aclEntity, RoleReader})
			}
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}
	})
}

func TestIntegration_PublicAccessPrevention(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		// Create a bucket with PublicAccessPrevention enforced.
		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{PublicAccessPrevention: PublicAccessPreventionEnforced})
		defer h.mustDeleteBucket(bkt)

		// Making bucket public should fail.
		policy, err := bkt.IAM().V3().Policy(ctx)
		if err != nil {
			t.Fatalf("fetching bucket IAM policy: %v", err)
		}
		policy.Bindings = append(policy.Bindings, &iampb.Binding{
			Role:    "roles/storage.objectViewer",
			Members: []string{iam.AllUsers},
		})
		if err := bkt.IAM().V3().SetPolicy(ctx, policy); err == nil {
			t.Error("SetPolicy: expected adding AllUsers policy to bucket should fail")
		}

		// Making object public via ACL should fail.
		o := bkt.Object("publicAccessPrevention")
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()
		wc := o.NewWriter(ctx)
		wc.ContentType = "text/plain"
		h.mustWrite(wc, []byte("test"))
		a := o.ACL()
		if err := a.Set(ctx, AllUsers, RoleReader); err == nil {
			t.Error("ACL.Set: expected adding AllUsers ACL to object should fail")
		}

		// Update PAP setting to inherited should work and not affect UBLA setting.
		attrs, err := bkt.Update(ctx, BucketAttrsToUpdate{PublicAccessPrevention: PublicAccessPreventionInherited})
		if err != nil {
			t.Fatalf("updating PublicAccessPrevention failed: %v", err)
		}
		if attrs.PublicAccessPrevention != PublicAccessPreventionInherited {
			t.Errorf("updating PublicAccessPrevention: got %s, want %s", attrs.PublicAccessPrevention, PublicAccessPreventionInherited)
		}
		if attrs.UniformBucketLevelAccess.Enabled || attrs.BucketPolicyOnly.Enabled {
			t.Error("updating PublicAccessPrevention changed UBLA setting")
		}

		// Now, making object public or making bucket public should succeed. Run with
		// retry because ACL settings may take time to propagate.
		retrier := func(err error) bool {
			// Once ACL settings propagate, PAP should no longer be enforced and the call will succeed.
			// In the meantime, while PAP is enforced, trying to set ACL results in:
			// 	-	FailedPrecondition for gRPC
			// 	-	condition not met (412) for HTTP
			return ShouldRetry(err) || status.Code(err) == codes.FailedPrecondition || extractErrCode(err) == http.StatusPreconditionFailed
		}

		ctxWithTimeout, cancelCtx := context.WithTimeout(ctx, time.Second*10)
		a = o.Retryer(WithErrorFunc(retrier), WithPolicy(RetryAlways)).ACL()
		err = a.Set(ctxWithTimeout, AllUsers, RoleReader)
		cancelCtx()
		if err != nil {
			t.Errorf("ACL.Set: making object public failed: %v", err)
		}

		policy, err = bkt.IAM().V3().Policy(ctx)
		if err != nil {
			t.Fatalf("fetching bucket IAM policy: %v", err)
		}
		policy.Bindings = append(policy.Bindings, &iampb.Binding{
			Role:    "roles/storage.objectViewer",
			Members: []string{iam.AllUsers},
		})
		if err := bkt.IAM().V3().SetPolicy(ctx, policy); err != nil {
			t.Errorf("SetPolicy: making bucket public failed: %v", err)
		}

		// Updating UBLA should not affect PAP setting.
		attrs, err = bkt.Update(ctx, BucketAttrsToUpdate{UniformBucketLevelAccess: &UniformBucketLevelAccess{Enabled: true}})
		if err != nil {
			t.Fatalf("updating UBLA failed: %v", err)
		}
		if !attrs.UniformBucketLevelAccess.Enabled {
			t.Error("updating UBLA: got UBLA not enabled, want enabled")
		}
		if attrs.PublicAccessPrevention != PublicAccessPreventionInherited {
			t.Errorf("updating UBLA: got %s, want %s", attrs.PublicAccessPrevention, PublicAccessPreventionInherited)
		}
	})
}

func TestIntegration_Autoclass(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		// Create a bucket with Autoclass enabled.
		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{Autoclass: &Autoclass{Enabled: true}})
		defer h.mustDeleteBucket(bkt)

		// Get Autoclass configuration from bucket attrs.
		// Autoclass.TerminalStorageClass is defaulted to NEARLINE if not specified.
		attrs, err := bkt.Attrs(ctx)
		if err != nil {
			t.Fatalf("get bucket attrs failed: %v", err)
		}
		var toggleTime time.Time
		var tscUpdateTime time.Time
		if attrs != nil && attrs.Autoclass != nil {
			if got, want := attrs.Autoclass.Enabled, true; got != want {
				t.Errorf("attr.Autoclass.Enabled = %v, want %v", got, want)
			}
			if toggleTime = attrs.Autoclass.ToggleTime; toggleTime.IsZero() {
				t.Error("got a zero time value, want a populated value")
			}
			if got, want := attrs.Autoclass.TerminalStorageClass, "NEARLINE"; got != want {
				t.Errorf("attr.Autoclass.TerminalStorageClass = %v, want %v", got, want)
			}
			if tscUpdateTime := attrs.Autoclass.TerminalStorageClassUpdateTime; tscUpdateTime.IsZero() {
				t.Error("got a zero time value, want a populated value")
			}
		}

		// Update TerminalStorageClass on the bucket.
		ua := BucketAttrsToUpdate{Autoclass: &Autoclass{Enabled: true, TerminalStorageClass: "ARCHIVE"}}
		attrs = h.mustUpdateBucket(bkt, ua, attrs.MetaGeneration)
		if got, want := attrs.Autoclass.Enabled, true; got != want {
			t.Errorf("attr.Autoclass.Enabled = %v, want %v", got, want)
		}
		if got, want := attrs.Autoclass.TerminalStorageClass, "ARCHIVE"; got != want {
			t.Errorf("attr.Autoclass.TerminalStorageClass = %v, want %v", got, want)
		}
		latestTSCUpdateTime := attrs.Autoclass.TerminalStorageClassUpdateTime
		if latestTSCUpdateTime.IsZero() {
			t.Error("got a zero time value, want a populated value")
		}
		if !latestTSCUpdateTime.After(tscUpdateTime) {
			t.Error("latestTSCUpdateTime should be newer than bucket creation tscUpdateTime")
		}

		// Disable Autoclass on the bucket.
		ua = BucketAttrsToUpdate{Autoclass: &Autoclass{Enabled: false}}
		attrs = h.mustUpdateBucket(bkt, ua, attrs.MetaGeneration)
		if got, want := attrs.Autoclass.Enabled, false; got != want {
			t.Errorf("attr.Autoclass.Enabled = %v, want %v", got, want)
		}
		latestToggleTime := attrs.Autoclass.ToggleTime
		if latestToggleTime.IsZero() {
			t.Error("got a zero time value, want a populated value")
		}
		if !latestToggleTime.After(toggleTime) {
			t.Error("latestToggleTime should be newer than bucket creation toggleTime")
		}
	})
}

func TestIntegration_ConditionalDelete(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		o := client.Bucket(bucket).Object("conddel")

		wc := o.NewWriter(ctx)
		wc.ContentType = "text/plain"
		h.mustWrite(wc, []byte("foo"))

		gen := wc.Attrs().Generation
		metaGen := wc.Attrs().Metageneration

		if err := o.Generation(gen - 1).Delete(ctx); err == nil {
			t.Fatalf("Unexpected successful delete with Generation")
		}
		if err := o.If(Conditions{MetagenerationMatch: metaGen + 1}).Delete(ctx); err == nil {
			t.Fatalf("Unexpected successful delete with IfMetaGenerationMatch")
		}
		if err := o.If(Conditions{MetagenerationNotMatch: metaGen}).Delete(ctx); err == nil {
			t.Fatalf("Unexpected successful delete with IfMetaGenerationNotMatch")
		}
		if err := o.Generation(gen).Delete(ctx); err != nil {
			t.Fatalf("final delete failed: %v", err)
		}
	})
}

func TestIntegration_HierarchicalNamespace(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, prefix string, client *Client) {
		h := testHelper{t}

		// Create a bucket with HNS enabled.
		hnsBucketName := prefix + uidSpace.New()
		bkt := client.Bucket(hnsBucketName)
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{
			UniformBucketLevelAccess: UniformBucketLevelAccess{Enabled: true},
			HierarchicalNamespace:    &HierarchicalNamespace{Enabled: true},
		})
		defer h.mustDeleteBucket(bkt)

		attrs, err := bkt.Attrs(ctx)
		if err != nil {
			t.Fatalf("bkt(%q).Attrs: %v", hnsBucketName, err)
		}

		if got, want := (attrs.HierarchicalNamespace), (&HierarchicalNamespace{Enabled: true}); cmp.Diff(got, want) != "" {
			t.Errorf("HierarchicalNamespace: got %+v, want %+v", got, want)
		}

		// Folder creation should work on HNS bucket, but not on standard bucket.
		req := &controlpb.CreateFolderRequest{
			Parent:   fmt.Sprintf("projects/_/buckets/%v", hnsBucketName),
			FolderId: "foo/",
			Folder:   &controlpb.Folder{},
		}
		if _, err := controlClient.CreateFolder(ctx, req); err != nil {
			t.Errorf("creating folder in bucket %q: %v", hnsBucketName, err)
		}

		req2 := &controlpb.CreateFolderRequest{
			Parent:   fmt.Sprintf("projects/_/buckets/%v", bucket),
			FolderId: "foo/",
			Folder:   &controlpb.Folder{},
		}
		if _, err := controlClient.CreateFolder(ctx, req2); status.Code(err) != codes.FailedPrecondition {
			t.Errorf("creating folder in non-HNS bucket %q: got error %v, want FailedPrecondition", bucket, err)
		}
	})
}

func TestIntegration_ObjectsRangeReader(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		objName := uidSpaceObjects.New()
		obj := bkt.Object(objName)
		contents := []byte("Hello, world this is a range request")

		w := obj.If(Conditions{DoesNotExist: true}).NewWriter(ctx)
		if _, err := w.Write(contents); err != nil {
			t.Errorf("Failed to write contents: %v", err)
		}
		if err := w.Close(); err != nil {
			t.Errorf("Failed to close writer: %v", err)
		}

		last5s := []struct {
			name   string
			start  int64
			length int64
		}{
			{name: "negative offset", start: -5, length: -1},
			{name: "offset with specified length", start: int64(len(contents)) - 5, length: 5},
			{name: "offset and read till end", start: int64(len(contents)) - 5, length: -1},
		}

		for _, last5 := range last5s {
			t.Run(last5.name, func(t *testing.T) {
				// Test Read and WriteTo.
				for _, c := range readCases {
					t.Run(c.desc, func(t *testing.T) {
						wantBuf := contents[len(contents)-5:]
						r, err := obj.NewRangeReader(ctx, last5.start, last5.length)
						if err != nil {
							t.Fatalf("Failed to make range read: %v", err)
						}
						defer r.Close()

						if got, want := r.Attrs.StartOffset, int64(len(contents))-5; got != want {
							t.Errorf("StartOffset mismatch, got %d want %d", got, want)
						}

						gotBuf, err := c.readFunc(r)
						if err != nil {
							t.Fatalf("reading object: %v", err)
						}
						if got, want := len(gotBuf), 5; got != want {
							t.Errorf("Body length mismatch, got %d want %d", got, want)
						} else if diff := cmp.Diff(string(gotBuf), string(wantBuf)); diff != "" {
							t.Errorf("Content read does not match - got(-),want(+):\n%s", diff)
						}
					})
				}

			})
		}
	})
}

func TestIntegration_ObjectReadChunksGRPC(t *testing.T) {
	multiTransportTest(skipHTTP("gRPC implementation specific test"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}
		// Use a larger blob to test chunking logic. This is a little over 5MB.
		content := make([]byte, 5<<20)
		rand.New(rand.NewSource(0)).Read(content)

		// Upload test data.
		obj := client.Bucket(bucket).Object(uidSpaceObjects.New())
		if err := writeObject(ctx, obj, "text/plain", content); err != nil {
			t.Fatal(err)
		}
		defer h.mustDeleteObject(obj)

		r, err := obj.NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}
		defer r.Close()

		if size := r.Size(); size != int64(len(content)) {
			t.Errorf("got size = %v, want %v", size, len(content))
		}
		if rem := r.Remain(); rem != int64(len(content)) {
			t.Errorf("got %v bytes remaining, want %v", rem, len(content))
		}

		bufSize := len(content)
		buf := make([]byte, bufSize)

		// Read in smaller chunks, offset to provoke reading across a Recv boundary.
		chunk := 4<<10 + 1234
		offset := 0
		for {
			end := math.Min(float64(offset+chunk), float64(bufSize))
			n, err := r.Read(buf[offset:int(end)])
			if err == io.EOF {
				break
			}
			if err != nil {
				t.Fatal(err)
			}
			offset += n
		}

		if rem := r.Remain(); rem != 0 {
			t.Errorf("got %v bytes remaining, want 0", rem)
		}
		if !bytes.Equal(buf, content) {
			t.Errorf("content mismatch")
		}
	})
}

func TestIntegration_MultiMessageWriteGRPC(t *testing.T) {
	multiTransportTest(skipHTTP("gRPC implementation specific test"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		name := uidSpaceObjects.New()
		obj := client.Bucket(bucket).Object(name).Retryer(WithPolicy(RetryAlways))
		defer h.mustDeleteObject(obj)

		// Use a larger blob to test multi-message logic. This is a little over 5MB.
		content := bytes.Repeat([]byte("a"), 5<<20)

		crc32c := crc32.Checksum(content, crc32cTable)
		w := obj.NewWriter(ctx)
		w.ProgressFunc = func(p int64) {
			t.Logf("%s: committed %d\n", t.Name(), p)
		}
		w.SendCRC32C = true
		w.CRC32C = crc32c
		got, err := w.Write(content)
		if err != nil {
			t.Fatalf("Writer.Write: %v", err)
		}
		// Flush the buffer to finish the upload.
		if err := w.Close(); err != nil {
			t.Fatalf("Writer.Close: %v", err)
		}

		want := len(content)
		if got != want {
			t.Errorf("While writing got: %d want %d", got, want)
		}

		// Read back the Object for verification.
		reader, err := client.Bucket(bucket).Object(name).NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}
		defer reader.Close()

		buf := make([]byte, want+4<<10)
		b := bytes.NewBuffer(buf)
		gotr, err := io.Copy(b, reader)
		if err != nil {
			t.Fatal(err)
		}
		if gotr != int64(want) {
			t.Errorf("While reading got: %d want %d", gotr, want)
		}
	})
}

func TestIntegration_MultiChunkWrite(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}
		obj := client.Bucket(bucket).Object(uidSpaceObjects.New()).Retryer(WithPolicy(RetryAlways))
		defer h.mustDeleteObject(obj)

		// Use a larger blob to test multi-message logic. This is a little over 5MB.
		content := bytes.Repeat([]byte("a"), 5<<20)
		crc32c := crc32.Checksum(content, crc32cTable)

		w := obj.NewWriter(ctx)
		w.SendCRC32C = true
		w.CRC32C = crc32c
		// Use a 1 MB chunk size.
		w.ChunkSize = 1 << 20
		w.ProgressFunc = func(p int64) {
			t.Logf("%s: committed %d\n", t.Name(), p)
		}
		got, err := w.Write(content)
		if err != nil {
			t.Fatalf("Writer.Write: %v", err)
		}
		// Flush the buffer to finish the upload.
		if err := w.Close(); err != nil {
			t.Fatalf("Writer.Close: %v", err)
		}

		want := len(content)
		if got != want {
			t.Errorf("While writing got: %d want %d", got, want)
		}

		r, err := obj.NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}
		defer r.Close()

		buf := make([]byte, want+4<<10)
		b := bytes.NewBuffer(buf)
		gotr, err := io.Copy(b, r)
		if err != nil {
			t.Fatal(err)
		}
		if gotr != int64(want) {
			t.Errorf("While reading got: %d want %d", gotr, want)
		}
	})
}

func TestIntegration_ConditionalDownload(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		o := client.Bucket(bucket).Object("condread")
		defer o.Delete(ctx)

		wc := o.NewWriter(ctx)
		wc.ContentType = "text/plain"
		h.mustWrite(wc, []byte("foo"))

		gen := wc.Attrs().Generation
		metaGen := wc.Attrs().Metageneration

		if _, err := o.Generation(gen + 1).NewReader(ctx); err == nil {
			t.Fatalf("Unexpected successful download with nonexistent Generation")
		}
		if _, err := o.If(Conditions{MetagenerationMatch: metaGen + 1}).NewReader(ctx); err == nil {
			t.Fatalf("Unexpected successful download with failed preconditions IfMetaGenerationMatch")
		}
		if _, err := o.If(Conditions{GenerationMatch: gen + 1}).NewReader(ctx); err == nil {
			t.Fatalf("Unexpected successful download with failed preconditions IfGenerationMatch")
		}
		if _, err := o.If(Conditions{GenerationMatch: gen}).NewReader(ctx); err != nil {
			t.Fatalf("Download failed: %v", err)
		}
	})
}

func TestIntegration_ObjectIteration(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		// Reset testTime, 'cause object last modification time should be within 5 min
		// from test (test iteration if -count passed) start time.
		testTime = time.Now().UTC()
		newBucketName := prefix + uidSpace.New()
		h := testHelper{t}
		bkt := client.Bucket(newBucketName).Retryer(WithPolicy(RetryAlways))

		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer func() {
			if err := killBucket(ctx, client, newBucketName); err != nil {
				log.Printf("deleting %q: %v", newBucketName, err)
			}
		}()
		const defaultType = "text/plain"

		// Populate object names and make a map for their contents.
		objects := []string{
			"obj1",
			"obj2",
			"obj/with/slashes",
			"obj/",
		}
		contents := make(map[string][]byte)

		// Test Writer.
		for _, obj := range objects {
			c := randomContents()
			if err := writeObject(ctx, bkt.Object(obj), defaultType, c); err != nil {
				t.Errorf("Write for %v failed with %v", obj, err)
			}
			contents[obj] = c
		}

		testObjectIterator(t, bkt, objects)
		testObjectsIterateSelectedAttrs(t, bkt, objects)
		testObjectsIterateAllSelectedAttrs(t, bkt, objects)
		testObjectIteratorWithOffset(t, bkt, objects)
		testObjectsIterateWithProjection(t, bkt)
		t.Run("testObjectsIterateSelectedAttrsDelimiter", func(t *testing.T) {
			query := &Query{Prefix: "", Delimiter: "/"}
			if err := query.SetAttrSelection([]string{"Name"}); err != nil {
				t.Fatalf("selecting query attrs: %v", err)
			}

			var gotNames []string
			var gotPrefixes []string
			it := bkt.Objects(context.Background(), query)
			for {
				attrs, err := it.Next()
				if err == iterator.Done {
					break
				}
				if err != nil {
					t.Fatalf("iterator.Next: %v", err)
				}
				if attrs.Name != "" {
					gotNames = append(gotNames, attrs.Name)
				} else if attrs.Prefix != "" {
					gotPrefixes = append(gotPrefixes, attrs.Prefix)
				}

				if attrs.Bucket != "" {
					t.Errorf("Bucket field not selected, want empty, got = %v", attrs.Bucket)
				}
			}

			sortedNames := []string{"obj1", "obj2"}
			if !cmp.Equal(sortedNames, gotNames) {
				t.Errorf("names = %v, want %v", gotNames, sortedNames)
			}
			sortedPrefixes := []string{"obj/"}
			if !cmp.Equal(sortedPrefixes, gotPrefixes) {
				t.Errorf("prefixes = %v, want %v", gotPrefixes, sortedPrefixes)
			}
		})
		t.Run("testObjectsIterateSelectedAttrsDelimiterIncludeTrailingDelimiter", func(t *testing.T) {
			query := &Query{Prefix: "", Delimiter: "/", IncludeTrailingDelimiter: true}
			if err := query.SetAttrSelection([]string{"Name"}); err != nil {
				t.Fatalf("selecting query attrs: %v", err)
			}

			var gotNames []string
			var gotPrefixes []string
			it := bkt.Objects(context.Background(), query)
			for {
				attrs, err := it.Next()
				if err == iterator.Done {
					break
				}
				if err != nil {
					t.Fatalf("iterator.Next: %v", err)
				}
				if attrs.Name != "" {
					gotNames = append(gotNames, attrs.Name)
				} else if attrs.Prefix != "" {
					gotPrefixes = append(gotPrefixes, attrs.Prefix)
				}

				if attrs.Bucket != "" {
					t.Errorf("Bucket field not selected, want empty, got = %v", attrs.Bucket)
				}
			}

			sortedNames := []string{"obj/", "obj1", "obj2"}
			if !cmp.Equal(sortedNames, gotNames) {
				t.Errorf("names = %v, want %v", gotNames, sortedNames)
			}
			sortedPrefixes := []string{"obj/"}
			if !cmp.Equal(sortedPrefixes, gotPrefixes) {
				t.Errorf("prefixes = %v, want %v", gotPrefixes, sortedPrefixes)
			}
		})
	})
}

func TestIntegration_ObjectIterationMatchGlob(t *testing.T) {
	multiTransportTest(skipJSONReads(context.Background(), "no reads in test"), t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		// Reset testTime, 'cause object last modification time should be within 5 min
		// from test (test iteration if -count passed) start time.
		testTime = time.Now().UTC()
		newBucketName := prefix + uidSpace.New()
		h := testHelper{t}
		bkt := client.Bucket(newBucketName).Retryer(WithPolicy(RetryAlways))

		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer func() {
			if err := killBucket(ctx, client, newBucketName); err != nil {
				log.Printf("deleting %q: %v", newBucketName, err)
			}
		}()
		const defaultType = "text/plain"

		// Populate object names and make a map for their contents.
		objects := []string{
			"obj1",
			"obj2",
			"obj/with/slashes",
			"obj/",
			"other/obj1",
		}
		contents := make(map[string][]byte)

		// Test Writer.
		for _, obj := range objects {
			c := randomContents()
			if err := writeObject(ctx, bkt.Object(obj), defaultType, c); err != nil {
				t.Errorf("Write for %v failed with %v", obj, err)
			}
			contents[obj] = c
		}
		query := &Query{MatchGlob: "**obj1"}

		var gotNames []string
		it := bkt.Objects(context.Background(), query)
		for {
			attrs, err := it.Next()
			if err == iterator.Done {
				break
			}
			if err != nil {
				t.Fatalf("iterator.Next: %v", err)
			}
			if attrs.Name != "" {
				gotNames = append(gotNames, attrs.Name)
			}
		}

		sortedNames := []string{"obj1", "other/obj1"}
		if !cmp.Equal(sortedNames, gotNames) {
			t.Errorf("names = %v, want %v", gotNames, sortedNames)
		}
	})
}

func TestIntegration_ObjectIterationManagedFolder(t *testing.T) {
	ctx := skipGRPC("not yet implemented in gRPC")
	multiTransportTest(skipJSONReads(ctx, "no reads in test"), t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		newBucketName := prefix + uidSpace.New()
		h := testHelper{t}
		bkt := client.Bucket(newBucketName).Retryer(WithPolicy(RetryAlways))

		// Create bucket with UBLA enabled as this is necessary for managed folders.
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{
			UniformBucketLevelAccess: UniformBucketLevelAccess{
				Enabled: true,
			},
		})

		t.Cleanup(func() {
			if err := killBucket(ctx, client, newBucketName); err != nil {
				log.Printf("deleting %q: %v", newBucketName, err)
			}
		})
		const defaultType = "text/plain"

		// Populate object names and make a map for their contents.
		objects := []string{
			"obj1",
			"obj2",
			"obj/with/slashes",
			"obj/",
			"other/obj1",
		}
		contents := make(map[string][]byte)

		// Test Writer.
		for _, obj := range objects {
			c := randomContents()
			if err := writeObject(ctx, bkt.Object(obj), defaultType, c); err != nil {
				t.Errorf("Write for %v failed with %v", obj, err)
			}
			contents[obj] = c
		}

		req := &controlpb.CreateManagedFolderRequest{
			Parent:          fmt.Sprintf("projects/_/buckets/%s", newBucketName),
			ManagedFolder:   &controlpb.ManagedFolder{},
			ManagedFolderId: "mf",
		}
		mf, err := controlClient.CreateManagedFolder(ctx, req)
		if err != nil {
			t.Fatalf("creating managed folder: %v", err)
		}

		t.Cleanup(func() {
			req := &controlpb.DeleteManagedFolderRequest{
				Name: mf.Name,
			}
			controlClient.DeleteManagedFolder(ctx, req)
		})

		// Test that managed folders are only included when IncludeFoldersAsPrefixes is set.
		cases := []struct {
			name  string
			query *Query
			want  []string
		}{
			{
				name:  "include folders",
				query: &Query{Delimiter: "/", IncludeFoldersAsPrefixes: true},
				want:  []string{"mf/", "obj/", "other/"},
			},
			{
				name:  "no folders",
				query: &Query{Delimiter: "/"},
				want:  []string{"obj/", "other/"},
			},
		}

		for _, c := range cases {
			t.Run(c.name, func(t *testing.T) {
				var gotNames []string
				var gotPrefixes []string
				it := bkt.Objects(context.Background(), c.query)
				for {
					attrs, err := it.Next()
					if err == iterator.Done {
						break
					}
					if err != nil {
						t.Fatalf("iterator.Next: %v", err)
					}
					if attrs.Name != "" {
						gotNames = append(gotNames, attrs.Name)
					}
					if attrs.Prefix != "" {
						gotPrefixes = append(gotPrefixes, attrs.Prefix)
					}
				}

				sortedNames := []string{"obj1", "obj2"}
				if !cmp.Equal(sortedNames, gotNames) {
					t.Errorf("names = %v, want %v", gotNames, sortedNames)
				}

				if !cmp.Equal(c.want, gotPrefixes) {
					t.Errorf("prefixes = %v, want %v", gotPrefixes, c.want)
				}
			})
		}
	})
}

func TestIntegration_ObjectUpdate(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		b := client.Bucket(bucket)

		o := b.Object("update-obj" + uidSpaceObjects.New())
		w := o.NewWriter(ctx)
		_, err := io.Copy(w, bytes.NewReader(randomContents()))
		if err != nil {
			t.Fatalf("io.Copy: %v", err)
		}
		if err := w.Close(); err != nil {
			t.Fatalf("w.Close: %v", err)
		}
		defer func() {
			if err := o.Delete(ctx); err != nil {
				t.Errorf("o.Delete : %v", err)
			}
		}()

		attrs, err := o.Attrs(ctx)
		if err != nil {
			t.Fatalf("o.Attrs: %v", err)
		}

		// Test UpdateAttrs.
		metadata := map[string]string{"key": "value"}

		updated, err := o.If(Conditions{MetagenerationMatch: attrs.Metageneration}).Update(ctx, ObjectAttrsToUpdate{
			ContentType:     "text/html",
			ContentLanguage: "en",
			Metadata:        metadata,
			ACL:             []ACLRule{{Entity: "domain-google.com", Role: RoleReader}},
		})
		if err != nil {
			t.Fatalf("o.Update: %v", err)
		}

		if got, want := updated.ContentType, "text/html"; got != want {
			t.Errorf("updated.ContentType == %q; want %q", got, want)
		}
		if got, want := updated.ContentLanguage, "en"; got != want {
			t.Errorf("updated.ContentLanguage == %q; want %q", updated.ContentLanguage, want)
		}
		if got, want := updated.Metadata, metadata; !testutil.Equal(got, want) {
			t.Errorf("updated.Metadata == %+v; want %+v", updated.Metadata, want)
		}
		if got, want := updated.Created, attrs.Created; got != want {
			t.Errorf("updated.Created == %q; want %q", got, want)
		}
		if !updated.Created.Before(updated.Updated) {
			t.Errorf("updated.Updated should be newer than update.Created")
		}

		// Add another metadata key
		anotherKey := map[string]string{"key2": "value2"}
		metadata["key2"] = "value2"

		updated, err = o.Update(ctx, ObjectAttrsToUpdate{
			Metadata: anotherKey,
		})
		if err != nil {
			t.Fatalf("o.Update: %v", err)
		}

		if got, want := updated.Metadata, metadata; !testutil.Equal(got, want) {
			t.Errorf("updated.Metadata == %+v; want %+v", updated.Metadata, want)
		}

		// Delete ContentType and ContentLanguage and Metadata.
		updated, err = o.If(Conditions{MetagenerationMatch: updated.Metageneration}).Update(ctx, ObjectAttrsToUpdate{
			ContentType:     "",
			ContentLanguage: "",
			Metadata:        map[string]string{},
			ACL:             []ACLRule{{Entity: "domain-google.com", Role: RoleReader}},
		})
		if err != nil {
			t.Fatalf("o.Update: %v", err)
		}

		if got, want := updated.ContentType, ""; got != want {
			t.Errorf("updated.ContentType == %q; want %q", got, want)
		}
		if got, want := updated.ContentLanguage, ""; got != want {
			t.Errorf("updated.ContentLanguage == %q; want %q", updated.ContentLanguage, want)
		}
		if updated.Metadata != nil {
			t.Errorf("updated.Metadata == %+v; want nil", updated.Metadata)
		}
		if got, want := updated.Created, attrs.Created; got != want {
			t.Errorf("updated.Created == %q; want %q", got, want)
		}
		if !updated.Created.Before(updated.Updated) {
			t.Errorf("updated.Updated should be newer than update.Created")
		}

		// Test empty update. Most fields should be unchanged, but updating will
		// increase the metageneration and update time.
		wantAttrs := updated
		gotAttrs, err := o.Update(ctx, ObjectAttrsToUpdate{})
		if err != nil {
			t.Fatalf("empty update: %v", err)
		}
		if diff := testutil.Diff(gotAttrs, wantAttrs, cmpopts.IgnoreFields(ObjectAttrs{}, "Etag", "Metageneration", "Updated")); diff != "" {
			t.Errorf("empty update: got=-, want=+:\n%s", diff)
		}
	})
}

func TestIntegration_ObjectChecksums(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		b := client.Bucket(bucket)
		checksumCases := []struct {
			name     string
			contents [][]byte
			size     int64
			md5      string
			crc32c   uint32
		}{
			{
				name:     "checksum-object",
				contents: [][]byte{[]byte("hello"), []byte("world")},
				size:     10,
				md5:      "fc5e038d38a57032085441e7fe7010b0",
				crc32c:   1456190592,
			},
			{
				name:     "zero-object",
				contents: [][]byte{},
				size:     0,
				md5:      "d41d8cd98f00b204e9800998ecf8427e",
				crc32c:   0,
			},
		}
		for _, c := range checksumCases {
			wc := b.Object(c.name + uidSpaceObjects.New()).NewWriter(ctx)
			for _, data := range c.contents {
				if _, err := wc.Write(data); err != nil {
					t.Fatalf("Write(%q) failed with %q", data, err)
				}
			}
			if err := wc.Close(); err != nil {
				t.Fatalf("%q: close failed with %q", c.name, err)
			}
			obj := wc.Attrs()
			if got, want := obj.Size, c.size; got != want {
				t.Errorf("Object (%q) Size = %v; want %v", c.name, got, want)
			}
			if got, want := fmt.Sprintf("%x", obj.MD5), c.md5; got != want {
				t.Errorf("Object (%q) MD5 = %q; want %q", c.name, got, want)
			}
			if got, want := obj.CRC32C, c.crc32c; got != want {
				t.Errorf("Object (%q) CRC32C = %v; want %v", c.name, got, want)
			}
		}
	})
}

func TestIntegration_ObjectCompose(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		b := client.Bucket(bucket)

		objects := []*ObjectHandle{
			b.Object("obj1" + uidSpaceObjects.New()),
			b.Object("obj2" + uidSpaceObjects.New()),
			b.Object("obj/with/slashes" + uidSpaceObjects.New()),
			b.Object("obj/" + uidSpaceObjects.New()),
		}
		var compSrcs []*ObjectHandle
		wantContents := make([]byte, 0)

		// Write objects to compose
		for _, obj := range objects {
			c := randomContents()
			if err := writeObject(ctx, obj, "text/plain", c); err != nil {
				t.Errorf("Write for %v failed with %v", obj, err)
			}
			compSrcs = append(compSrcs, obj)
			wantContents = append(wantContents, c...)
			defer obj.Delete(ctx)
		}

		checkCompose := func(obj *ObjectHandle, contentTypeSet *string) {
			r, err := obj.NewReader(ctx)
			if err != nil {
				t.Fatalf("new reader: %v", err)
			}

			slurp, err := io.ReadAll(r)
			if err != nil {
				t.Fatalf("io.ReadAll: %v", err)
			}
			defer r.Close()
			if !bytes.Equal(slurp, wantContents) {
				t.Errorf("Composed object contents\ngot:  %q\nwant: %q", slurp, wantContents)
			}
			got := r.ContentType()
			// Accept both an empty string and octet-stream if the content type was not set;
			// HTTP will set the content type as octet-stream whilst GRPC will not set it all.
			if !(contentTypeSet == nil && (got == "" || got == "application/octet-stream")) && got != *contentTypeSet {
				t.Errorf("Composed object content-type = %q, want %q", got, *contentTypeSet)
			}
		}

		// Compose should work even if the user sets no destination attributes.
		compDst := b.Object("composed1")
		c := compDst.ComposerFrom(compSrcs...)
		attrs, err := c.Run(ctx)
		if err != nil {
			t.Fatalf("ComposeFrom error: %v", err)
		}
		if attrs.ComponentCount != int64(len(objects)) {
			t.Errorf("mismatching ComponentCount: got %v, want %v", attrs.ComponentCount, int64(len(objects)))
		}
		checkCompose(compDst, nil)

		// It should also work if we do.
		contentType := "text/json"
		compDst = b.Object("composed2")
		c = compDst.ComposerFrom(compSrcs...)
		c.ContentType = contentType
		attrs, err = c.Run(ctx)
		if err != nil {
			t.Fatalf("ComposeFrom error: %v", err)
		}
		if attrs.ComponentCount != int64(len(objects)) {
			t.Errorf("mismatching ComponentCount: got %v, want %v", attrs.ComponentCount, int64(len(objects)))
		}
		checkCompose(compDst, &contentType)
	})
}

func TestIntegration_Copy(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, prefix string, client *Client) {
		h := testHelper{t}

		bucketFrom := client.Bucket(bucket)
		bucketInSameRegion := client.Bucket(prefix + uidSpace.New())
		bucketInDifferentRegion := client.Bucket(prefix + uidSpace.New())

		// Create new bucket
		if err := bucketInSameRegion.Create(ctx, testutil.ProjID(), nil); err != nil {
			t.Fatalf("bucket.Create: %v", err)
		}
		t.Cleanup(func() {
			h.mustDeleteBucket(bucketInSameRegion)
		})

		// Create new bucket
		if err := bucketInDifferentRegion.Create(ctx, testutil.ProjID(), &BucketAttrs{Location: "NORTHAMERICA-NORTHEAST2"}); err != nil {
			t.Fatalf("bucket.Create: %v", err)
		}
		t.Cleanup(func() {
			h.mustDeleteBucket(bucketInDifferentRegion)
		})

		// We use a larger object size to be able to trigger multiple rewrite calls
		minObjectSize := 2500000 // 2.5 Mb
		obj := bucketFrom.Object("copy-object-original" + uidSpaceObjects.New())

		// Create an object to copy from
		w := obj.NewWriter(ctx)
		c := randomContents()
		for written := 0; written < minObjectSize; {
			n, err := w.Write(c)
			if err != nil {
				t.Fatalf("w.Write: %v", err)
			}
			written += n
		}
		if err := w.Close(); err != nil {
			t.Fatalf("w.Close: %v", err)
		}
		t.Cleanup(func() {
			h.mustDeleteObject(obj)
		})

		attrs, err := obj.Attrs(ctx)
		if err != nil {
			t.Fatalf("obj.Attrs: %v", err)
		}

		crc32c := attrs.CRC32C

		type copierAttrs struct {
			contentEncoding string
			maxBytesPerCall int64
		}

		for _, test := range []struct {
			desc                    string
			toObj                   string
			toBucket                *BucketHandle
			copierAttrs             *copierAttrs
			numExpectedRewriteCalls int
		}{
			{
				desc:                    "copy within bucket",
				toObj:                   "copy-within-bucket",
				toBucket:                bucketFrom,
				numExpectedRewriteCalls: 1,
			},
			{
				desc:                    "copy to new bucket",
				toObj:                   "copy-new-bucket",
				toBucket:                bucketInSameRegion,
				numExpectedRewriteCalls: 1,
			},
			{
				desc:                    "copy with attributes",
				toObj:                   "copy-with-attributes",
				toBucket:                bucketInSameRegion,
				copierAttrs:             &copierAttrs{contentEncoding: "identity"},
				numExpectedRewriteCalls: 1,
			},
			{
				// this test should trigger multiple re-write calls and may fail
				// with a rate limit error if those calls are stuck in an infinite loop
				desc:                    "copy to new region",
				toObj:                   "copy-new-region",
				toBucket:                bucketInDifferentRegion,
				copierAttrs:             &copierAttrs{maxBytesPerCall: 1048576},
				numExpectedRewriteCalls: 3,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				copyObj := test.toBucket.Object(test.toObj)
				copier := copyObj.CopierFrom(obj)

				if attrs := test.copierAttrs; attrs != nil {
					if attrs.contentEncoding != "" {
						copier.ContentEncoding = attrs.contentEncoding
					}
					if attrs.maxBytesPerCall != 0 {
						copier.maxBytesRewrittenPerCall = attrs.maxBytesPerCall
					}
				}

				rewriteCallsCount := 0
				copier.ProgressFunc = func(_, _ uint64) {
					rewriteCallsCount++
				}

				attrs, err = copier.Run(ctx)
				if err != nil {
					t.Fatalf("Copier.Run failed with %v", err)
				}
				t.Cleanup(func() {
					h.mustDeleteObject(copyObj)
				})

				// Check copied object is in the correct bucket with the correct name
				if attrs.Bucket != test.toBucket.name || attrs.Name != test.toObj {
					t.Errorf("unexpected copy behaviour: got: %s in bucket %s, want: %s in bucket %s", attrs.Name, attrs.Bucket, attrs.Name, test.toBucket.name)
				}

				// Check attrs
				if test.copierAttrs != nil {
					if attrs.ContentEncoding != test.copierAttrs.contentEncoding {
						t.Errorf("unexpected ContentEncoding; got: %s, want: %s", attrs.ContentEncoding, test.copierAttrs.contentEncoding)
					}
				}

				// Check the copied contents
				if attrs.CRC32C != crc32c {
					t.Errorf("mismatching checksum: got %v, want %v", attrs.CRC32C, crc32c)
				}

				// Check that the number of requests made is as expected
				if rewriteCallsCount != test.numExpectedRewriteCalls {
					t.Errorf("unexpected number of rewrite calls: got %v, want %v", rewriteCallsCount, test.numExpectedRewriteCalls)
				}
			})
		}
	})
}

func TestIntegration_Encoding(t *testing.T) {
	multiTransportTest(skipGRPC("gzip transcoding not supported"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		// Test content encoding
		const zeroCount = 20 << 1 // TODO: should be 20 << 20
		obj := bkt.Object("gzip-test")
		w := obj.NewWriter(ctx)
		w.ContentEncoding = "gzip"
		gw := gzip.NewWriter(w)
		if _, err := io.Copy(gw, io.LimitReader(zeros{}, zeroCount)); err != nil {
			t.Fatalf("io.Copy, upload: %v", err)
		}
		if err := gw.Close(); err != nil {
			t.Errorf("gzip.Close(): %v", err)
		}
		if err := w.Close(); err != nil {
			t.Errorf("w.Close(): %v", err)
		}
		r, err := obj.NewReader(ctx)
		if err != nil {
			t.Fatalf("NewReader(gzip-test): %v", err)
		}
		n, err := io.Copy(io.Discard, r)
		if err != nil {
			t.Errorf("io.Copy, download: %v", err)
		}
		if n != zeroCount {
			t.Errorf("downloaded bad data: got %d bytes, want %d", n, zeroCount)
		}

		// Test NotFound.
		_, err = bkt.Object("obj-not-exists").NewReader(ctx)
		if err != ErrObjectNotExist {
			t.Errorf("Object should not exist, err found to be %v", err)
		}
	})
}

func testObjectIterator(t *testing.T, bkt *BucketHandle, objects []string) {
	ctx := context.Background()
	h := testHelper{t}
	// Collect the list of items we expect: ObjectAttrs in lexical order by name.
	names := make([]string, len(objects))
	copy(names, objects)
	sort.Strings(names)
	var attrs []*ObjectAttrs
	for _, name := range names {
		attrs = append(attrs, h.mustObjectAttrs(bkt.Object(name)))
	}
	msg, ok := itesting.TestIterator(attrs,
		func() interface{} { return bkt.Objects(ctx, &Query{Prefix: "obj"}) },
		func(it interface{}) (interface{}, error) { return it.(*ObjectIterator).Next() })
	if !ok {
		t.Errorf("ObjectIterator.Next: %s", msg)
	}
	// TODO(jba): test query.Delimiter != ""
}

func testObjectIteratorWithOffset(t *testing.T, bkt *BucketHandle, objects []string) {
	ctx := context.Background()
	h := testHelper{t}
	// Collect the list of items we expect: ObjectAttrs in lexical order by name.
	names := make([]string, len(objects))
	copy(names, objects)
	sort.Strings(names)
	var attrs []*ObjectAttrs
	for _, name := range names {
		attrs = append(attrs, h.mustObjectAttrs(bkt.Object(name)))
	}
	m := make(map[string][]*ObjectAttrs)
	for i, name := range names {
		// StartOffset takes the value of object names, the result must be for:
		// ― obj/with/slashes: obj/with/slashes, obj1, obj2
		// ― obj1: obj1, obj2
		// ― obj2: obj2.
		m[name] = attrs[i:]
		msg, ok := itesting.TestIterator(m[name],
			func() interface{} { return bkt.Objects(ctx, &Query{StartOffset: name}) },
			func(it interface{}) (interface{}, error) { return it.(*ObjectIterator).Next() })
		if !ok {
			t.Errorf("ObjectIterator.Next: %s", msg)
		}
		// EndOffset takes the value of object names, the result must be for:
		// ― obj/with/slashes: ""
		// ― obj1: obj/with/slashes
		// ― obj2: obj/with/slashes, obj1.
		m[name] = attrs[:i]
		msg, ok = itesting.TestIterator(m[name],
			func() interface{} { return bkt.Objects(ctx, &Query{EndOffset: name}) },
			func(it interface{}) (interface{}, error) { return it.(*ObjectIterator).Next() })
		if !ok {
			t.Errorf("ObjectIterator.Next: %s", msg)
		}
	}
}

func testObjectsIterateSelectedAttrs(t *testing.T, bkt *BucketHandle, objects []string) {
	// Create a query that will only select the "Name" attr of objects, and
	// invoke object listing.
	query := &Query{Prefix: ""}
	query.SetAttrSelection([]string{"Name"})

	var gotNames []string
	it := bkt.Objects(context.Background(), query)
	for {
		attrs, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			t.Fatalf("iterator.Next: %v", err)
		}
		gotNames = append(gotNames, attrs.Name)

		if len(attrs.Bucket) > 0 {
			t.Errorf("Bucket field not selected, want empty, got = %v", attrs.Bucket)
		}
	}

	sortedNames := make([]string, len(objects))
	copy(sortedNames, objects)
	sort.Strings(sortedNames)
	sort.Strings(gotNames)

	if !cmp.Equal(sortedNames, gotNames) {
		t.Errorf("names = %v, want %v", gotNames, sortedNames)
	}
}

func testObjectsIterateAllSelectedAttrs(t *testing.T, bkt *BucketHandle, objects []string) {
	// Tests that all selected attributes work - query succeeds (without actually
	// verifying the returned results).
	query := &Query{
		Prefix:      "",
		StartOffset: "obj/",
		EndOffset:   "obj2",
	}
	var selectedAttrs []string
	for k := range attrToFieldMap {
		selectedAttrs = append(selectedAttrs, k)
	}
	query.SetAttrSelection(selectedAttrs)

	count := 0
	it := bkt.Objects(context.Background(), query)
	for {
		_, err := it.Next()
		if err == iterator.Done {
			break
		}
		if err != nil {
			t.Fatalf("iterator.Next: %v", err)
		}
		count++
	}

	if count != len(objects)-1 {
		t.Errorf("count = %v, want %v", count, len(objects)-1)
	}
}

func testObjectsIterateWithProjection(t *testing.T, bkt *BucketHandle) {
	projections := map[Projection]bool{
		ProjectionDefault: true,
		ProjectionFull:    true,
		ProjectionNoACL:   false,
	}

	for projection, expectACL := range projections {
		query := &Query{Projection: projection}
		it := bkt.Objects(context.Background(), query)
		attrs, err := it.Next()
		if err == iterator.Done {
			t.Fatalf("iterator: no objects")
		}
		if err != nil {
			t.Fatalf("iterator.Next: %v", err)
		}

		if expectACL {
			if attrs.Owner == "" {
				t.Errorf("projection %q: Owner is empty, want nonempty Owner", projection)
			}
			if len(attrs.ACL) == 0 {
				t.Errorf("projection %q: ACL is empty, want at least one ACL rule", projection)
			}
		} else {
			if attrs.Owner != "" {
				t.Errorf("projection %q: got Owner = %q, want empty Owner", projection, attrs.Owner)
			}
			if len(attrs.ACL) != 0 {
				t.Errorf("projection %q: got %d ACL rules, want empty ACL", projection, len(attrs.ACL))
			}
		}
	}
}

func TestIntegration_SignedURL(t *testing.T) {
	multiTransportTest(skipJSONReads(context.Background(), "no reads in test"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		// To test SignedURL, we need a real user email and private key. Extract them
		// from the JSON key file.
		jwtConf, err := testutil.JWTConfig()
		if err != nil {
			t.Fatal(err)
		}
		if jwtConf == nil {
			t.Skip("JSON key file is not present")
		}

		bkt := client.Bucket(bucket)
		obj := "signedURL"
		contents := []byte("This is a test of SignedURL.\n")
		md5 := "Jyxvgwm9n2MsrGTMPbMeYA==" // base64-encoded MD5 of contents
		if err := writeObject(ctx, bkt.Object(obj), "text/plain", contents); err != nil {
			t.Fatalf("writing: %v", err)
		}
		for _, test := range []struct {
			desc    string
			opts    SignedURLOptions
			headers map[string][]string
			fail    bool
		}{
			{
				desc: "basic v2",
			},
			{
				desc: "basic v4",
				opts: SignedURLOptions{Scheme: SigningSchemeV4},
			},
			{
				desc:    "MD5 sent and matches",
				opts:    SignedURLOptions{MD5: md5},
				headers: map[string][]string{"Content-MD5": {md5}},
			},
			{
				desc: "MD5 not sent",
				opts: SignedURLOptions{MD5: md5},
				fail: true,
			},
			{
				desc:    "Content-Type sent and matches",
				opts:    SignedURLOptions{ContentType: "text/plain"},
				headers: map[string][]string{"Content-Type": {"text/plain"}},
			},
			{
				desc:    "Content-Type sent but does not match",
				opts:    SignedURLOptions{ContentType: "text/plain"},
				headers: map[string][]string{"Content-Type": {"application/json"}},
				fail:    true,
			},
			{
				desc: "Canonical headers sent and match",
				opts: SignedURLOptions{Headers: []string{
					" X-Goog-Foo: Bar baz ",
					"X-Goog-Novalue", // ignored: no value
					"X-Google-Foo",   // ignored: wrong prefix
					"x-goog-meta-start-time: 2023-02-10T02:00:00Z", // with colons
				}},
				headers: map[string][]string{"X-Goog-foo": {"Bar baz  "}, "x-goog-meta-start-time": {"2023-02-10T02:00:00Z"}},
			},
			{
				desc: "Canonical headers sent and match using V4",
				opts: SignedURLOptions{Headers: []string{
					"x-goog-meta-start-time: 2023-02-10T02:", // with colons
					" X-Goog-Foo: Bar baz ",
					"X-Goog-Novalue", // ignored: no value
					"X-Google-Foo",   // ignored: wrong prefix
				},
					Scheme: SigningSchemeV4,
				},
				headers: map[string][]string{"x-goog-meta-start-time": {"2023-02-10T02:"}, "X-Goog-foo": {"Bar baz  "}},
			},
			{
				desc:    "Canonical headers sent but don't match",
				opts:    SignedURLOptions{Headers: []string{" X-Goog-Foo: Bar baz"}},
				headers: map[string][]string{"X-Goog-Foo": {"bar baz"}},
				fail:    true,
			},
			{
				desc: "Virtual hosted style with custom hostname",
				opts: SignedURLOptions{
					Style:    VirtualHostedStyle(),
					Hostname: "storage.googleapis.com:443",
				},
				fail: false,
			},
			{
				desc: "Hostname v4",
				opts: SignedURLOptions{
					Hostname: "storage.googleapis.com:443",
					Scheme:   SigningSchemeV4,
				},
				fail: false,
			},
		} {
			opts := test.opts
			opts.GoogleAccessID = jwtConf.Email
			opts.PrivateKey = jwtConf.PrivateKey
			opts.Method = "GET"
			opts.Expires = time.Now().Add(time.Hour)

			u, err := bkt.SignedURL(obj, &opts)
			if err != nil {
				t.Errorf("%s: SignedURL: %v", test.desc, err)
				continue
			}

			err = verifySignedURL(u, test.headers, contents)
			if err != nil && !test.fail {
				t.Errorf("%s: wanted success but got error:\n%v", test.desc, err)
			} else if err == nil && test.fail {
				t.Errorf("%s: wanted failure but test succeeded", test.desc)
			}
		}
	})
}

func TestIntegration_SignedURL_WithEncryptionKeys(t *testing.T) {
	multiTransportTest(skipJSONReads(context.Background(), "no reads in test"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {

		// To test SignedURL, we need a real user email and private key. Extract
		// them from the JSON key file.
		jwtConf, err := testutil.JWTConfig()
		if err != nil {
			t.Fatal(err)
		}
		if jwtConf == nil {
			t.Skip("JSON key file is not present")
		}

		bkt := client.Bucket(bucket)

		// TODO(deklerk): document how these were generated and their significance
		encryptionKey := "AAryxNglNkXQY0Wa+h9+7BLSFMhCzPo22MtXUWjOBbI="
		encryptionKeySha256 := "QlCdVONb17U1aCTAjrFvMbnxW/Oul8VAvnG1875WJ3k="
		headers := map[string][]string{
			"x-goog-encryption-algorithm":  {"AES256"},
			"x-goog-encryption-key":        {encryptionKey},
			"x-goog-encryption-key-sha256": {encryptionKeySha256},
		}
		contents := []byte(`{"message":"encryption with csek works"}`)
		tests := []struct {
			desc string
			opts *SignedURLOptions
		}{
			{
				desc: "v4 URL with customer supplied encryption keys for PUT",
				opts: &SignedURLOptions{
					Method: "PUT",
					Headers: []string{
						"x-goog-encryption-algorithm:AES256",
						"x-goog-encryption-key:AAryxNglNkXQY0Wa+h9+7BLSFMhCzPo22MtXUWjOBbI=",
						"x-goog-encryption-key-sha256:QlCdVONb17U1aCTAjrFvMbnxW/Oul8VAvnG1875WJ3k=",
					},
					Scheme: SigningSchemeV4,
				},
			},
			{
				desc: "v4 URL with customer supplied encryption keys for GET",
				opts: &SignedURLOptions{
					Method: "GET",
					Headers: []string{
						"x-goog-encryption-algorithm:AES256",
						fmt.Sprintf("x-goog-encryption-key:%s", encryptionKey),
						fmt.Sprintf("x-goog-encryption-key-sha256:%s", encryptionKeySha256),
					},
					Scheme: SigningSchemeV4,
				},
			},
		}
		defer func() {
			// Delete encrypted object.
			err := bkt.Object("csek.json").Delete(ctx)
			if err != nil {
				log.Printf("failed to deleted encrypted file: %v", err)
			}
		}()

		for _, test := range tests {
			opts := test.opts
			opts.GoogleAccessID = jwtConf.Email
			opts.PrivateKey = jwtConf.PrivateKey
			opts.Expires = time.Now().Add(time.Hour)

			u, err := bkt.SignedURL("csek.json", test.opts)
			if err != nil {
				t.Fatalf("%s: %v", test.desc, err)
			}

			if test.opts.Method == "PUT" {
				if _, err := putURL(u, headers, bytes.NewReader(contents)); err != nil {
					t.Fatalf("%s: %v", test.desc, err)
				}
			}

			if test.opts.Method == "GET" {
				if err := verifySignedURL(u, headers, contents); err != nil {
					t.Fatalf("%s: %v", test.desc, err)
				}
			}
		}
	})
}

func TestIntegration_SignedURL_EmptyStringObjectName(t *testing.T) {
	multiTransportTest(skipJSONReads(context.Background(), "no reads in test"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {

		// To test SignedURL, we need a real user email and private key. Extract them
		// from the JSON key file.
		jwtConf, err := testutil.JWTConfig()
		if err != nil {
			t.Fatal(err)
		}
		if jwtConf == nil {
			t.Skip("JSON key file is not present")
		}

		opts := &SignedURLOptions{
			Scheme:         SigningSchemeV4,
			Method:         "GET",
			GoogleAccessID: jwtConf.Email,
			PrivateKey:     jwtConf.PrivateKey,
			Expires:        time.Now().Add(time.Hour),
		}

		bkt := client.Bucket(bucket)
		u, err := bkt.SignedURL("", opts)
		if err != nil {
			t.Fatal(err)
		}

		// Should be some ListBucketResult response.
		_, err = getURL(u, nil)
		if err != nil {
			t.Fatal(err)
		}
	})

}

func TestIntegration_BucketACL(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bucket := prefix + uidSpace.New()
		bkt := client.Bucket(bucket)
		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer h.mustDeleteBucket(bkt)

		entity := ACLEntity("domain-google.com")
		rule := ACLRule{Entity: entity, Role: RoleReader, Domain: "google.com"}

		if err := bkt.DefaultObjectACL().Set(ctx, entity, RoleReader); err != nil {
			t.Errorf("Can't put default ACL rule for the bucket, errored with %v", err)
		}

		acl, err := bkt.DefaultObjectACL().List(ctx)
		if err != nil {
			t.Errorf("DefaultObjectACL.List for bucket %q: %v", bucket, err)
		}
		if !containsACLRule(acl, testACLRule(rule)) {
			t.Fatalf("default ACL rule missing; want: %#v, got rules: %+v", rule, acl)
		}

		o := bkt.Object("acl1")
		defer h.mustDeleteObject(o)

		// Retry to account for propagation delay in metadata update.
		err = retry(ctx, func() error {
			if err := writeObject(ctx, o, "", randomContents()); err != nil {
				return fmt.Errorf("Write for %v failed with %v", o.ObjectName(), err)
			}
			acl, err = o.ACL().List(ctx)
			return err
		}, func() error {
			if !containsACLRule(acl, testACLRule(rule)) {
				return fmt.Errorf("object ACL rule missing %+v from ACL \n%+v", rule, acl)
			}
			return nil
		})
		if err != nil {
			t.Error(err)
		}

		if err := o.ACL().Delete(ctx, entity); err != nil {
			t.Errorf("object ACL: could not delete entity %s", entity)
		}
		// Delete the default ACL rule. We can't move this code earlier in the
		// test, because the test depends on the fact that the object ACL inherits
		// it.
		if err := bkt.DefaultObjectACL().Delete(ctx, entity); err != nil {
			t.Errorf("default ACL: could not delete entity %s", entity)
		}

		entity2 := AllAuthenticatedUsers
		rule2 := ACLRule{Entity: entity2, Role: RoleReader}
		if err := bkt.ACL().Set(ctx, entity2, RoleReader); err != nil {
			t.Errorf("Error while putting bucket ACL rule: %v", err)
		}

		var bACL []ACLRule

		// Retry to account for propagation delay in metadata update.
		err = retry(ctx, func() error {
			bACL, err = bkt.ACL().List(ctx)
			return err
		}, func() error {
			if !containsACLRule(bACL, testACLRule(rule2)) {
				return fmt.Errorf("bucket ACL missing %+v", rule2)
			}
			return nil
		})
		if err != nil {
			t.Error(err)
		}

		if err := bkt.ACL().Delete(ctx, entity2); err != nil {
			t.Errorf("Error while deleting bucket ACL rule: %v", err)
		}
	})
}

func TestIntegration_ValidObjectNames(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		validNames := []string{
			"gopher",
			"Гоферови",
			"a",
			strings.Repeat("a", 1024),
		}
		for _, name := range validNames {
			if err := writeObject(ctx, bkt.Object(name), "", []byte("data")); err != nil {
				t.Errorf("Object %q write failed: %v. Want success", name, err)
				continue
			}
			defer bkt.Object(name).Delete(ctx)
		}

		invalidNames := []string{
			"",                        // Too short.
			strings.Repeat("a", 1025), // Too long.
			"new\nlines",
			"bad\xffunicode",
		}
		for _, name := range invalidNames {
			// Invalid object names will either cause failure during Write or Close.
			if err := writeObject(ctx, bkt.Object(name), "", []byte("data")); err != nil {
				continue
			}
			defer bkt.Object(name).Delete(ctx)
			t.Errorf("%q should have failed. Didn't", name)
		}
	})
}

func TestIntegration_WriterContentType(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		obj := client.Bucket(bucket).Object("content")
		testCases := []struct {
			content               string
			setType, wantType     string
			forceEmptyContentType bool
		}{
			{
				// Sniffed content type.
				content:  "It was the best of times, it was the worst of times.",
				wantType: "text/plain; charset=utf-8",
			},
			{
				// Sniffed content type.
				content:  "<html><head><title>My first page</title></head></html>",
				wantType: "text/html; charset=utf-8",
			},
			{
				content:  "<html><head><title>My first page</title></head></html>",
				setType:  "text/html",
				wantType: "text/html",
			},
			{
				content:  "<html><head><title>My first page</title></head></html>",
				setType:  "image/jpeg",
				wantType: "image/jpeg",
			},
			{
				// Content type sniffing disabled.
				content:               "<html><head><title>My first page</title></head></html>",
				setType:               "",
				wantType:              "",
				forceEmptyContentType: true,
			},
		}
		for i, tt := range testCases {
			writer := newWriter(ctx, obj, tt.setType, tt.forceEmptyContentType)
			if err := writeContents(writer, []byte(tt.content)); err != nil {
				t.Errorf("writing #%d: %v", i, err)
			}
			attrs, err := obj.Attrs(ctx)
			if err != nil {
				t.Errorf("obj.Attrs: %v", err)
				continue
			}
			if got := attrs.ContentType; got != tt.wantType {
				t.Errorf("Content-Type = %q; want %q\nContent: %q\nSet Content-Type: %q", got, tt.wantType, tt.content, tt.setType)
			}
		}
	})
}

func TestIntegration_WriterChunksize(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		obj := client.Bucket(bucket).Object("writer-chunksize-test" + uidSpaceObjects.New())
		objSize := 1<<10<<10 + 1 // 1 Mib + 1 byte
		contents := bytes.Repeat([]byte("a"), objSize)

		for _, test := range []struct {
			desc             string
			chunksize        int
			wantBytesPerCall int64
			wantCallbacks    int
		}{
			{
				desc:             "default chunksize",
				chunksize:        16 << 10 << 10,
				wantBytesPerCall: 16 << 10 << 10,
				wantCallbacks:    0,
			},
			{
				desc:             "small chunksize rounds up to 256kib",
				chunksize:        1,
				wantBytesPerCall: 256 << 10,
				wantCallbacks:    5,
			},
			{
				desc:             "chunksize of 256kib",
				chunksize:        256 << 10,
				wantBytesPerCall: 256 << 10,
				wantCallbacks:    5,
			},
			{
				desc:             "chunksize of just over 256kib rounds up",
				chunksize:        256<<10 + 1,
				wantBytesPerCall: 256 * 2 << 10,
				wantCallbacks:    3,
			},
			{
				desc:             "multiple of 256kib",
				chunksize:        256 * 3 << 10,
				wantBytesPerCall: 256 * 3 << 10,
				wantCallbacks:    2,
			},
			{
				desc:             "chunksize 0 uploads everything",
				chunksize:        0,
				wantBytesPerCall: int64(objSize),
				wantCallbacks:    0,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				t.Cleanup(func() { obj.Delete(ctx) })

				w := obj.Retryer(WithPolicy(RetryAlways)).NewWriter(ctx)
				w.ChunkSize = test.chunksize

				bytesWrittenSoFar := int64(0)
				callbacks := 0

				w.ProgressFunc = func(i int64) {
					bytesWrittenByCall := i - bytesWrittenSoFar

					// Error if this is not the last call and we don't write exactly wantBytesPerCall
					if i != int64(objSize) && bytesWrittenByCall != test.wantBytesPerCall {
						t.Errorf("unexpected number of bytes written by call; wanted: %d, written: %d", test.wantBytesPerCall, bytesWrittenByCall)
					}

					bytesWrittenSoFar = i
					callbacks++
				}

				if _, err := w.Write(contents); err != nil {
					_ = w.Close()
					t.Fatalf("writer.Write: %v", err)
				}
				if err := w.Close(); err != nil {
					t.Fatalf("writer.Close: %v", err)
				}

				if callbacks != test.wantCallbacks {
					t.Errorf("ProgressFunc was called %d times, expected %d", callbacks, test.wantCallbacks)
				}

				// Confirm all bytes were uploaded.
				attrs, err := obj.Attrs(ctx)
				if err != nil {
					t.Fatalf("obj.Attrs: %v", err)
				}
				if attrs.Size != int64(objSize) {
					t.Errorf("incorrect number of bytes written; got %v, want %v", attrs.Size, objSize)
				}
			})
		}
	})
}

func TestIntegration_ZeroSizedObject(t *testing.T) {
	t.Parallel()
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		obj := client.Bucket(bucket).Object("zero")

		// Check writing it works as expected.
		w := obj.NewWriter(ctx)
		if err := w.Close(); err != nil {
			t.Fatalf("Writer.Close: %v", err)
		}
		defer obj.Delete(ctx)

		// Check we can read it too. Test both with WriteTo and Read.
		for _, c := range readCases {
			t.Run(c.desc, func(t *testing.T) {
				r, err := obj.NewReader(ctx)
				if err != nil {
					t.Fatalf("NewReader: %v", err)
				}
				body, err := c.readFunc(r)
				if err != nil {
					t.Fatalf("reading object: %v", err)
				}
				if len(body) != 0 {
					t.Errorf("Body is %v, want empty []byte{}", body)
				}
			})
		}
	})
}

func TestIntegration_Encryption(t *testing.T) {
	// This function tests customer-supplied encryption keys for all operations
	// involving objects. Bucket and ACL operations aren't tested because they
	// aren't affected by customer encryption. Neither is deletion.
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		h := testHelper{t}

		obj := client.Bucket(bucket).Object("customer-encryption")
		key := []byte("my-secret-AES-256-encryption-key")
		keyHash := sha256.Sum256(key)
		keyHashB64 := base64.StdEncoding.EncodeToString(keyHash[:])
		key2 := []byte("My-Secret-AES-256-Encryption-Key")
		contents := "top secret."

		checkMetadataCall := func(msg string, f func(o *ObjectHandle) (*ObjectAttrs, error)) {
			// Performing a metadata operation without the key should succeed.
			attrs, err := f(obj)
			if err != nil {
				t.Fatalf("%s: %v", msg, err)
			}
			// The key hash should match...
			if got, want := attrs.CustomerKeySHA256, keyHashB64; got != want {
				t.Errorf("%s: key hash: got %q, want %q", msg, got, want)
			}
			// ...but CRC and MD5 should not be present.
			if attrs.CRC32C != 0 {
				t.Errorf("%s: CRC: got %v, want 0", msg, attrs.CRC32C)
			}
			if len(attrs.MD5) > 0 {
				t.Errorf("%s: MD5: got %v, want len == 0", msg, attrs.MD5)
			}

			// Performing a metadata operation with the key should succeed.
			attrs, err = f(obj.Key(key))
			if err != nil {
				t.Fatalf("%s: %v", msg, err)
			}
			// Check the key and content hashes.
			if got, want := attrs.CustomerKeySHA256, keyHashB64; got != want {
				t.Errorf("%s: key hash: got %q, want %q", msg, got, want)
			}
			if attrs.CRC32C == 0 {
				t.Errorf("%s: CRC: got 0, want non-zero", msg)
			}
			if len(attrs.MD5) == 0 {
				t.Errorf("%s: MD5: got len == 0, want len > 0", msg)
			}
		}

		checkRead := func(msg string, o *ObjectHandle, k []byte, wantContents string) {
			// Reading the object without the key should fail.
			if _, err := readObject(ctx, o); err == nil {
				t.Errorf("%s: reading without key: want error, got nil", msg)
			}
			// Reading the object with the key should succeed.
			got := h.mustRead(o.Key(k))
			gotContents := string(got)
			// And the contents should match what we wrote.
			if gotContents != wantContents {
				t.Errorf("%s: contents: got %q, want %q", msg, gotContents, wantContents)
			}
		}

		checkReadUnencrypted := func(msg string, obj *ObjectHandle, wantContents string) {
			got := h.mustRead(obj)
			gotContents := string(got)
			if gotContents != wantContents {
				t.Errorf("%s: got %q, want %q", msg, gotContents, wantContents)
			}
		}

		// Write to obj using our own encryption key, which is a valid 32-byte
		// AES-256 key.
		h.mustWrite(obj.Key(key).NewWriter(ctx), []byte(contents))

		checkMetadataCall("Attrs", func(o *ObjectHandle) (*ObjectAttrs, error) {
			return o.Attrs(ctx)
		})

		checkMetadataCall("Update", func(o *ObjectHandle) (*ObjectAttrs, error) {
			return o.Update(ctx, ObjectAttrsToUpdate{ContentLanguage: "en"})
		})

		checkRead("first object", obj, key, contents)

		// We create 2 objects here and we can interleave operations to get around
		// the rate limit for object mutation operations (create, update, and delete).
		obj2 := client.Bucket(bucket).Object("customer-encryption-2")
		obj4 := client.Bucket(bucket).Object("customer-encryption-4")

		// Copying an object without the key should fail.
		if _, err := obj4.CopierFrom(obj).Run(ctx); err == nil {
			t.Fatal("want error, got nil")
		}
		// Copying an object with the key should succeed.
		if _, err := obj2.CopierFrom(obj.Key(key)).Run(ctx); err != nil {
			t.Fatal(err)
		}
		// The destination object is not encrypted; we can read it without a key.
		checkReadUnencrypted("copy dest", obj2, contents)

		// Providing a key on the destination but not the source should fail,
		// since the source is encrypted.
		if _, err := obj2.Key(key2).CopierFrom(obj).Run(ctx); err == nil {
			t.Fatal("want error, got nil")
		}

		// But copying with keys for both source and destination should succeed.
		if _, err := obj2.Key(key2).CopierFrom(obj.Key(key)).Run(ctx); err != nil {
			t.Fatal(err)
		}
		// And the destination should be encrypted, meaning we can only read it
		// with a key.
		checkRead("copy destination", obj2, key2, contents)

		// Change obj2's key to prepare for compose, where all objects must have
		// the same key. Also illustrates key rotation: copy an object to itself
		// with a different key.
		if _, err := obj2.Key(key).CopierFrom(obj2.Key(key2)).Run(ctx); err != nil {
			t.Fatal(err)
		}
		obj3 := client.Bucket(bucket).Object("customer-encryption-3")
		// Composing without keys should fail.
		if _, err := obj3.ComposerFrom(obj, obj2).Run(ctx); err == nil {
			t.Fatal("want error, got nil")
		}
		// Keys on the source objects result in an error.
		if _, err := obj3.ComposerFrom(obj.Key(key), obj2).Run(ctx); err == nil {
			t.Fatal("want error, got nil")
		}
		// A key on the destination object both decrypts the source objects
		// and encrypts the destination.
		if _, err := obj3.Key(key).ComposerFrom(obj, obj2).Run(ctx); err != nil {
			t.Fatalf("got %v, want nil", err)
		}
		// Check that the destination in encrypted.
		checkRead("compose destination", obj3, key, contents+contents)

		// You can't compose one or more unencrypted source objects into an
		// encrypted destination object.
		_, err := obj4.CopierFrom(obj2.Key(key)).Run(ctx) // unencrypt obj2
		if err != nil {
			t.Fatal(err)
		}
		if _, err := obj3.Key(key).ComposerFrom(obj4).Run(ctx); err == nil {
			t.Fatal("got nil, want error")
		}
	})
}

func TestIntegration_NonexistentObjectRead(t *testing.T) {
	t.Parallel()
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		_, err := client.Bucket(bucket).Object("object-does-not-exist").NewReader(ctx)
		if !errors.Is(err, ErrObjectNotExist) {
			t.Errorf("Objects: got %v, want ErrObjectNotExist", err)
		}
	})
}

func TestIntegration_NonexistentBucket(t *testing.T) {
	t.Parallel()
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {
		bkt := client.Bucket(prefix + uidSpace.New())
		if _, err := bkt.Attrs(ctx); err != ErrBucketNotExist {
			t.Errorf("Attrs: got %v, want ErrBucketNotExist", err)
		}
		it := bkt.Objects(ctx, nil)
		if _, err := it.Next(); err != ErrBucketNotExist {
			t.Errorf("Objects: got %v, want ErrBucketNotExist", err)
		}
	})
}

func TestIntegration_PerObjectStorageClass(t *testing.T) {
	const (
		defaultStorageClass = "STANDARD"
		newStorageClass     = "NEARLINE"
	)
	ctx := skipJSONReads(context.Background(), "no reads in test")

	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(bucket)

		// The bucket should have the default storage class.
		battrs := h.mustBucketAttrs(bkt)
		if battrs.StorageClass != defaultStorageClass {
			t.Fatalf("bucket storage class: got %q, want %q",
				battrs.StorageClass, defaultStorageClass)
		}
		// Write an object; it should start with the bucket's storage class.
		obj := bkt.Object("posc")
		h.mustWrite(obj.NewWriter(ctx), []byte("foo"))
		oattrs, err := obj.Attrs(ctx)
		if err != nil {
			t.Fatal(err)
		}
		if oattrs.StorageClass != defaultStorageClass {
			t.Fatalf("object storage class: got %q, want %q",
				oattrs.StorageClass, defaultStorageClass)
		}
		// Now use Copy to change the storage class.
		copier := obj.CopierFrom(obj)
		copier.StorageClass = newStorageClass
		oattrs2, err := copier.Run(ctx)
		if err != nil {
			log.Fatal(err)
		}
		if oattrs2.StorageClass != newStorageClass {
			t.Fatalf("new object storage class: got %q, want %q",
				oattrs2.StorageClass, newStorageClass)
		}

		// We can also write a new object using a non-default storage class.
		obj2 := bkt.Object("posc2")
		w := obj2.NewWriter(ctx)
		w.StorageClass = newStorageClass
		h.mustWrite(w, []byte("xxx"))
		if w.Attrs().StorageClass != newStorageClass {
			t.Fatalf("new object storage class: got %q, want %q",
				w.Attrs().StorageClass, newStorageClass)
		}
	})
}

func TestIntegration_NoUnicodeNormalization(t *testing.T) {
	t.Parallel()
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)
		h := testHelper{t}

		for _, tst := range []struct {
			nameQuoted, content string
		}{
			{`"Caf\u00e9"`, "Normalization Form C"},
			{`"Cafe\u0301"`, "Normalization Form D"},
		} {
			name, err := strconv.Unquote(tst.nameQuoted)
			w := bkt.Object(name).NewWriter(ctx)
			h.mustWrite(w, []byte(tst.content))
			if err != nil {
				t.Fatalf("invalid name: %s: %v", tst.nameQuoted, err)
			}
			if got := string(h.mustRead(bkt.Object(name))); got != tst.content {
				t.Errorf("content of %s is %q, want %q", tst.nameQuoted, got, tst.content)
			}
		}
	})
}

func TestIntegration_HashesOnUpload(t *testing.T) {
	// Check that the user can provide hashes on upload, and that these are checked.
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		obj := client.Bucket(bucket).Object("hashesOnUpload-1")
		data := []byte("I can't wait to be verified")

		write := func(w *Writer) error {
			if _, err := w.Write(data); err != nil {
				_ = w.Close()
				return err
			}
			return w.Close()
		}

		crc32c := crc32.Checksum(data, crc32cTable)
		// The correct CRC should succeed.
		w := obj.NewWriter(ctx)
		w.CRC32C = crc32c
		w.SendCRC32C = true
		if err := write(w); err != nil {
			t.Error(err)
		}

		// If we change the CRC, validation should fail.
		w = obj.NewWriter(ctx)
		w.CRC32C = crc32c + 1
		w.SendCRC32C = true
		if err := write(w); err == nil {
			t.Error("write with bad CRC32c: want error, got nil")
		}

		// If we have the wrong CRC but forget to send it, we succeed.
		w = obj.NewWriter(ctx)
		w.CRC32C = crc32c + 1
		if err := write(w); err != nil {
			t.Error(err)
		}

		// MD5
		md5 := md5.Sum(data)
		// The correct MD5 should succeed.
		w = obj.NewWriter(ctx)
		w.MD5 = md5[:]
		if err := write(w); err != nil {
			t.Error(err)
		}

		// If we change the MD5, validation should fail.
		w = obj.NewWriter(ctx)
		w.MD5 = append([]byte(nil), md5[:]...)
		w.MD5[0]++
		if err := write(w); err == nil {
			t.Error("write with bad MD5: want error, got nil")
		}
	})
}

func TestIntegration_BucketIAM(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {
		h := testHelper{t}
		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), nil)
		defer h.mustDeleteBucket(bkt)
		// This bucket is unique to this test run. So we don't have
		// to worry about other runs interfering with our IAM policy
		// changes.

		member := "projectViewer:" + testutil.ProjID()
		role := iam.RoleName("roles/storage.objectViewer")
		// Get the bucket's IAM policy.
		policy, err := bkt.IAM().Policy(ctx)
		if err != nil {
			t.Fatalf("Getting policy: %v", err)
		}
		// The member should not have the role.
		if policy.HasRole(member, role) {
			t.Errorf("member %q has role %q", member, role)
		}
		// Change the policy.
		policy.Add(member, role)
		if err := bkt.IAM().SetPolicy(ctx, policy); err != nil {
			t.Fatalf("SetPolicy: %v", err)
		}
		// Confirm that the binding was added.
		policy, err = bkt.IAM().Policy(ctx)
		if err != nil {
			t.Fatalf("Getting policy: %v", err)
		}
		if !policy.HasRole(member, role) {
			t.Errorf("member %q does not have role %q", member, role)
		}

		// Check TestPermissions.
		// This client should have all these permissions (and more).
		perms := []string{"storage.buckets.get", "storage.buckets.delete"}
		got, err := bkt.IAM().TestPermissions(ctx, perms)
		if err != nil {
			t.Fatalf("TestPermissions: %v", err)
		}
		sort.Strings(perms)
		sort.Strings(got)
		if !testutil.Equal(got, perms) {
			t.Errorf("got %v, want %v", got, perms)
		}
	})
}

// This test tests only possibilities where the user making the request is an
// owner on the project that owns the requester pays bucket. Therefore, we don't
// need a second project for this test.
//
// There are up to three entities involved in a requester-pays call:
//
//  1. The user making the request. Here, we use the account used as credentials
//     for most of our integration tests. The following must hold for this test:
//     - this user must have resourcemanager.projects.createBillingAssignment
//     permission (Owner role) on (2) (the project, not the bucket)
//     - this user must NOT have that permission on (3b).
//  2. The project that owns the requester-pays bucket. Here, that
//     is the test project ID (see testutil.ProjID).
//  3. The project provided as the userProject parameter of the request;
//     the project to be billed. This test uses:
//     a. The project that owns the requester-pays bucket (same as (2))
//     b. Another project (the Firestore project).
func TestIntegration_RequesterPaysOwner(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {
		jwt, err := testutil.JWTConfig()
		if err != nil {
			t.Fatalf("testutil.JWTConfig: %v", err)
		}
		// an account that has permissions on the project that owns the bucket
		mainUserEmail := jwt.Email

		// the project that owns the requester-pays bucket
		mainProjectID := testutil.ProjID()

		client.SetRetry(WithPolicy(RetryAlways))

		// Secondary project: a project that does not own the bucket.
		// The "main" user should not have permission on this.
		secondaryProject := os.Getenv(envFirestoreProjID)
		if secondaryProject == "" {
			t.Fatalf("need a second project (env var %s)", envFirestoreProjID)
		}

		for _, test := range []struct {
			desc          string
			userProject   *string // to set on bucket, nil if it should not be set
			expectSuccess bool
		}{
			{
				desc:          "user is Owner on the project that owns the bucket",
				userProject:   nil,
				expectSuccess: true, // by the rule permitting access by owners of the containing bucket
			},
			{
				desc:          "userProject is unnecessary but allowed",
				userProject:   &mainProjectID,
				expectSuccess: true, // by the rule permitting access by owners of the containing bucket
			},
			{
				desc:          "cannot use someone else's project for billing",
				userProject:   &secondaryProject,
				expectSuccess: false, // we cannot use a project we don't have access to for billing
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				h := testHelper{t}
				ctx, cancel := context.WithTimeout(ctx, time.Minute)
				defer cancel()

				printTestCase := func() string {
					userProject := "none"
					if test.userProject != nil {
						userProject = *test.userProject
					}
					return fmt.Sprintf("user: %s\n\t\tcontaining project: %s\n\t\tUserProject: %s", mainUserEmail, mainProjectID, userProject)
				}

				checkforErrors := func(desc string, err error) {
					if err != nil && test.expectSuccess {
						t.Errorf("%s: got unexpected error:%v\n\t\t%s", desc, err, printTestCase())
					} else if err == nil && !test.expectSuccess {
						t.Errorf("%s: got unexpected success\n\t\t%s", desc, printTestCase())
					}
				}

				bucketName := prefix + uidSpace.New()
				requesterPaysBucket := client.Bucket(bucketName)

				// Create a requester-pays bucket
				h.mustCreate(requesterPaysBucket, mainProjectID, &BucketAttrs{RequesterPays: true})
				t.Cleanup(func() { h.mustDeleteBucket(requesterPaysBucket) })

				// Make sure the object exists, so we don't get confused by ErrObjectNotExist.
				// The later write we perform may fail so we always write to the object as the user
				// with permissions on the containing bucket (mainUser).
				// The storage service may perform validation in any order (perhaps in parallel),
				// so if we delete or update an object that doesn't exist and for which we lack permission,
				// we could see either of those two errors. (See Google-internal bug 78341001.)
				objectName := "acl-go-test" + uidSpaceObjects.New()
				h.mustWrite(requesterPaysBucket.Object(objectName).NewWriter(ctx), []byte("hello"))

				// Set up the bucket to use depending on the test case
				bucket := client.Bucket(bucketName)
				if test.userProject != nil {
					bucket = bucket.UserProject(*test.userProject)
				}

				// Get bucket attrs
				attrs, err := bucket.Attrs(ctx)
				checkforErrors("get bucket attrs", err)
				if attrs != nil {
					if got, want := attrs.RequesterPays, true; got != want {
						t.Fatalf("attr.RequesterPays = %t, want %t", got, want)
					}
				}

				// Bucket ACL operations
				entity := ACLEntity("domain-google.com")

				checkforErrors("bucket acl set", bucket.ACL().Set(ctx, entity, RoleReader))
				_, err = bucket.ACL().List(ctx)
				checkforErrors("bucket acl list", err)
				checkforErrors("bucket acl delete", bucket.ACL().Delete(ctx, entity))

				// Object operations (except for delete)
				// Retry to account for propagation delay to objects in metadata update
				// (we updated the metadata to add the otherUserEmail as owner on the bucket)
				o := bucket.Object(objectName)
				ctxWithTimeout, cancel := context.WithTimeout(ctx, time.Second*10)
				defer cancel()
				// Only retry when we expect success to avoid retrying for 10 seconds
				// when we know it will fail
				if test.expectSuccess {
					o = o.Retryer(WithErrorFunc(retryOnTransient400and403))
				}
				checkforErrors("write object", writeObject(ctxWithTimeout, o, "text/plain", []byte("hello")))
				_, err = readObject(ctx, bucket.Object(objectName))
				checkforErrors("read object", err)
				_, err = bucket.Object(objectName).Attrs(ctx)
				checkforErrors("get object attrs", err)
				_, err = bucket.Object(objectName).Update(ctx, ObjectAttrsToUpdate{ContentLanguage: "en"})
				checkforErrors("update object", err)

				// Object ACL operations
				checkforErrors("object acl set", bucket.Object(objectName).ACL().Set(ctx, entity, RoleReader))
				_, err = bucket.Object(objectName).ACL().List(ctx)
				checkforErrors("object acl list", err)
				checkforErrors("object acl list", bucket.Object(objectName).ACL().Delete(ctx, entity))

				// Default object ACL operations
				// Once again, we interleave buckets to avoid rate limits
				checkforErrors("default object acl set", bucket.DefaultObjectACL().Set(ctx, entity, RoleReader))
				_, err = bucket.DefaultObjectACL().List(ctx)
				checkforErrors("default object acl list", err)
				checkforErrors("default object acl delete", bucket.DefaultObjectACL().Delete(ctx, entity))

				// Copy
				_, err = bucket.Object("copy").CopierFrom(bucket.Object(objectName)).Run(ctx)
				checkforErrors("copy", err)
				// Delete "copy" object, if created
				if err == nil {
					t.Cleanup(func() {
						h.mustDeleteObject(bucket.Object("copy"))
					})
				}

				// Compose
				_, err = bucket.Object("compose").ComposerFrom(bucket.Object(objectName), bucket.Object("copy")).Run(ctx)
				checkforErrors("compose", err)
				// Delete "compose" object, if created
				if err == nil {
					t.Cleanup(func() {
						h.mustDeleteObject(bucket.Object("compose"))
					})
				}

				// Delete object
				if err = bucket.Object(objectName).Delete(ctx); err != nil {
					// We still want to delete object if the test errors
					h.mustDeleteObject(requesterPaysBucket.Object(objectName))
				}
				checkforErrors("delete object", err)
			})
		}
	})
}

// This test needs a second project and user to test all possibilities. Since we
// need these things for Firestore already, we use them here.
//
// There are up to three entities involved in a requester-pays call:
//  1. The user making the request. Here, we use the account used for the
//     Firestore tests. The following must hold for this test to work:
//     - this user must NOT have resourcemanager.projects.createBillingAssignment
//     on the project that owns the bucket (2).
//     - this user must have serviceusage.services.use permission on the Firestore
//     project (3b).
//     - this user must NOT have that serviceusage.services.use permission on
//     the project that owns the bucket (3a).
//  2. The project that owns the requester-pays bucket. Here, that
//     is the test project ID (see testutil.ProjID).
//  3. The project provided as the userProject parameter of the request;
//     the project to be billed. This test uses:
//     a. The project that owns the requester-pays bucket (same as (2))
//     b. Another project (the Firestore project).
func TestIntegration_RequesterPaysNonOwner(t *testing.T) {
	if testing.Short() && !replaying {
		t.Skip("Integration tests skipped in short mode")
	}
	ctx := context.Background()

	// Main project: the project that owns the requester-pays bucket.
	mainProject := testutil.ProjID()

	// Secondary project: a project that does not own the bucket.
	// The "main" user does not have permission on this.
	// This project should have billing enabled.
	secondaryProject := os.Getenv(envFirestoreProjID)
	if secondaryProject == "" {
		t.Fatalf("need a second project (env var %s)", envFirestoreProjID)
	}

	// Secondary email: an account with permissions on the secondary project,
	// but not on the main project.
	// We will grant this email permissions to the bucket created under the main
	// project, but it must provide a user project to make requests
	// against that bucket (since it's a requester-pays bucket).
	secondaryUserEmail, err := keyFileEmail(os.Getenv(envFirestorePrivateKey))
	if err != nil {
		t.Fatalf("keyFileEmail error getting second account (env var %s): %v", envFirestorePrivateKey, err)
	}

	// Token source from secondary email to authenticate to client
	ts := testutil.TokenSourceEnv(ctx, envFirestorePrivateKey, ScopeFullControl)
	if ts == nil {
		t.Fatalf("need a second account (env var %s)", envFirestorePrivateKey)
	}

	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {
		client.SetRetry(WithPolicy(RetryAlways))

		for _, test := range []struct {
			desc              string
			userProject       *string // to set on bucket, nil if it should not be set
			expectSuccess     bool
			wantErrorCode     int
			wantErrorCodeGRPC codes.Code
		}{
			{
				desc:          "no UserProject",
				userProject:   nil,
				expectSuccess: false, // by the standard requester-pays rule
			},
			{
				desc:          "user is an Editor on UserProject",
				userProject:   &secondaryProject,
				expectSuccess: true, // by the standard requester-pays rule
			},
			{
				desc:              "user is not an Editor on UserProject",
				userProject:       &mainProject,
				expectSuccess:     false, // we cannot use a project we don't have access to for billing
				wantErrorCode:     403,
				wantErrorCodeGRPC: codes.PermissionDenied,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				ctx, cancel := context.WithTimeout(ctx, 30*time.Second)
				t.Cleanup(cancel)

				printTestCase := func() string {
					userProject := "none"
					if test.userProject != nil {
						userProject = *test.userProject
					}
					return fmt.Sprintf("user: %s\n\t\tcontaining project: %s\n\t\tUserProject: %s", secondaryUserEmail, mainProject, userProject)
				}

				checkforErrors := func(desc string, err error) {
					errCode := extractErrCode(err)
					if err != nil && test.expectSuccess {
						t.Errorf("%s: got unexpected error:%v\n\t\t%s", desc, err, printTestCase())
					} else if err == nil && !test.expectSuccess {
						t.Errorf("%s: got unexpected success\n\t\t%s", desc, printTestCase())
					} else if !test.expectSuccess && test.wantErrorCode != 0 {
						if (status.Code(err) != codes.OK && status.Code(err) != codes.Unknown && status.Code(err) != test.wantErrorCodeGRPC) || (errCode > 0 && errCode != test.wantErrorCode) {
							fmt.Println(status.Code(err), "   ", status.Code(err) != test.wantErrorCodeGRPC)
							t.Errorf("%s: mismatched errors; want error code: %d or grpc error: %s, got error: %v \n\t\t%s\n",
								desc, test.wantErrorCode, test.wantErrorCodeGRPC, err, printTestCase())
						}
					}
				}

				bucketName := prefix + uidSpace.New()
				objectName := "acl-go-test" + uidSpaceObjects.New()

				setUpRequesterPaysBucket(ctx, t, bucketName, objectName, secondaryUserEmail)

				// Set up the bucket to use depending on the test case
				bucket := client.Bucket(bucketName)
				if test.userProject != nil {
					bucket = bucket.UserProject(*test.userProject)
				}

				// Get bucket attrs
				attrs, err := bucket.Attrs(ctx)
				checkforErrors("get bucket attrs", err)
				if attrs != nil {
					if got, want := attrs.RequesterPays, true; got != want {
						t.Fatalf("attr.RequesterPays = %t, want %t", got, want)
					}
				}

				// Bucket ACL operations
				entity := ACLEntity("domain-google.com")

				checkforErrors("bucket acl set", bucket.ACL().Set(ctx, entity, RoleReader))
				_, err = bucket.ACL().List(ctx)
				checkforErrors("bucket acl list", err)
				checkforErrors("bucket acl delete", bucket.ACL().Delete(ctx, entity))

				// Object operations (except for delete)
				// Retry to account for propagation delay to objects in metadata update
				// (we updated the metadata to add the otherUserEmail as owner on the bucket)
				o := bucket.Object(objectName)
				ctxWithTimeout, cancel := context.WithTimeout(ctx, time.Second*15)
				defer cancel()
				// Only retry when we expect success to avoid retrying
				// when we know it will fail
				if test.expectSuccess {
					o = o.Retryer(WithErrorFunc(retryOnTransient400and403))
				}
				checkforErrors("write object", writeObject(ctxWithTimeout, o, "text/plain", []byte("hello")))
				_, err = readObject(ctx, bucket.Object(objectName))
				checkforErrors("read object", err)
				_, err = bucket.Object(objectName).Attrs(ctx)
				checkforErrors("get object attrs", err)
				_, err = bucket.Object(objectName).Update(ctx, ObjectAttrsToUpdate{ContentLanguage: "en"})
				checkforErrors("update object", err)

				// Object ACL operations
				checkforErrors("object acl set", bucket.Object(objectName).ACL().Set(ctx, entity, RoleReader))
				_, err = bucket.Object(objectName).ACL().List(ctx)
				checkforErrors("object acl list", err)
				checkforErrors("object acl list", bucket.Object(objectName).ACL().Delete(ctx, entity))

				// Default object ACL operations
				// Once again, we interleave buckets to avoid rate limits
				checkforErrors("default object acl set", bucket.DefaultObjectACL().Set(ctx, entity, RoleReader))
				_, err = bucket.DefaultObjectACL().List(ctx)
				checkforErrors("default object acl list", err)
				checkforErrors("default object acl delete", bucket.DefaultObjectACL().Delete(ctx, entity))

				// Copy
				copyObj := bucket.Object("copy")
				_, err = copyObj.CopierFrom(bucket.Object(objectName)).Run(ctx)
				checkforErrors("copy", err)
				// Delete "copy" object, if created
				if err == nil {
					t.Cleanup(func() {
						if err := deleteObjectIfExists(copyObj, WithErrorFunc(retryOnTransient400and403)); err != nil {
							t.Error(err)
						}
					})
				}

				// Compose
				composeObj := bucket.Object("compose")
				_, err = composeObj.ComposerFrom(bucket.Object(objectName), bucket.Object("copy")).Run(ctx)
				checkforErrors("compose", err)
				// Delete "compose" object, if created
				if err == nil {
					t.Cleanup(func() {
						if err := deleteObjectIfExists(composeObj, WithErrorFunc(retryOnTransient400and403)); err != nil {
							t.Error(err)
						}
					})
				}

				// Delete object
				checkforErrors("delete object", bucket.Object(objectName).Delete(ctx))
			})
		}
	}, option.WithTokenSource(ts))
}

func TestIntegration_Notifications(t *testing.T) {
	multiTransportTest(skipGRPC("notifications not implemented"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		checkNotifications := func(msg string, want map[string]*Notification) {
			got, err := bkt.Notifications(ctx)
			if err != nil {
				t.Fatal(err)
			}
			if diff := testutil.Diff(got, want); diff != "" {
				t.Errorf("%s: got=-, want=+:\n%s", msg, diff)
			}
		}
		checkNotifications("initial", map[string]*Notification{})

		nArg := &Notification{
			TopicProjectID: testutil.ProjID(),
			TopicID:        "go-storage-notification-test",
			PayloadFormat:  NoPayload,
		}
		n, err := bkt.AddNotification(ctx, nArg)
		if err != nil {
			t.Fatal(err)
		}
		if n.ID == "" {
			t.Fatal("expected created Notification to have non-empty ID")
		}
		nArg.ID = n.ID
		if !testutil.Equal(n, nArg) {
			t.Errorf("got %+v, want %+v", n, nArg)
		}
		checkNotifications("after add", map[string]*Notification{n.ID: n})

		if err := bkt.DeleteNotification(ctx, n.ID); err != nil {
			t.Fatal(err)
		}
		checkNotifications("after delete", map[string]*Notification{})
	})
}

func TestIntegration_PublicBucket(t *testing.T) {
	// Confirm that an unauthenticated client can access a public bucket.
	// See https://cloud.google.com/storage/docs/public-datasets/landsat

	multiTransportTest(skipGRPC("no public buckets for gRPC"), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		const landsatBucket = "gcp-public-data-landsat"
		const landsatPrefix = "LC08/01/001/002/LC08_L1GT_001002_20160817_20170322_01_T2/"
		const landsatObject = landsatPrefix + "LC08_L1GT_001002_20160817_20170322_01_T2_ANG.txt"

		h := testHelper{t}
		bkt := client.Bucket(landsatBucket)
		obj := bkt.Object(landsatObject)

		// Read a public object.
		bytes := h.mustRead(obj)
		if got, want := len(bytes), 117255; got != want {
			t.Errorf("len(bytes) = %d, want %d", got, want)
		}

		// List objects in a public bucket.
		iter := bkt.Objects(ctx, &Query{Prefix: landsatPrefix})
		gotCount := 0
		for {
			_, err := iter.Next()
			if err == iterator.Done {
				break
			}
			if err != nil {
				t.Fatal(err)
			}
			gotCount++
		}
		if wantCount := 14; gotCount != wantCount {
			t.Errorf("object count: got %d, want %d", gotCount, wantCount)
		}

		errCode := func(err error) int {
			var err2 *googleapi.Error
			if ok := errors.As(err, &err2); !ok {
				return -1
			}
			return err2.Code
		}

		// Reading from or writing to a non-public bucket fails.
		c := testConfig(ctx, t)
		defer c.Close()
		nonPublicObj := client.Bucket(bucket).Object("noauth")
		// XML API calls return 403 but the JSON API returns 401. Either is
		// acceptable for reads.
		_, err := readObject(ctx, nonPublicObj)
		if got := errCode(err); got != 403 && got != 401 {
			t.Errorf("got code %d; want %v\nerror: %v", got, "401 or 403", err)
		}
		err = writeObject(ctx, nonPublicObj, "text/plain", []byte("b"))
		if got, want := errCode(err), 401; got != want {
			t.Errorf("got code %d; want %d\nerror: %v", got, want, err)
		}
	}, option.WithoutAuthentication())
}

func TestIntegration_PublicObject(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		publicObj := client.Bucket(bucket).Object("public-obj" + uidSpaceObjects.New())
		contents := randomContents()

		w := publicObj.Retryer(WithPolicy(RetryAlways)).NewWriter(ctx)
		if _, err := w.Write(contents); err != nil {
			t.Fatalf("writer.Write: %v", err)
		}
		if err := w.Close(); err != nil {
			t.Errorf("writer.Close: %v", err)
		}

		// Set object ACL to public read.
		if err := publicObj.ACL().Set(ctx, AllUsers, RoleReader); err != nil {
			t.Fatalf("PutACLEntry failed with %v", err)
		}

		// Create unauthenticated client.
		publicClient, err := newTestClient(ctx, option.WithoutAuthentication())
		if err != nil {
			t.Fatalf("newTestClient: %v", err)
		}

		// Test can read public object.
		publicObjUnauthenticated := publicClient.Bucket(bucket).Object(publicObj.ObjectName())
		data, err := readObject(context.Background(), publicObjUnauthenticated)
		if err != nil {
			t.Fatalf("readObject: %v", err)
		}

		if !bytes.Equal(data, contents) {
			t.Errorf("Public object's content: got %q, want %q", data, contents)
		}

		// Test cannot write to read-only object without authentication.
		wc := publicObjUnauthenticated.NewWriter(ctx)
		if _, err := wc.Write([]byte("hello")); err != nil {
			t.Errorf("Write unexpectedly failed with %v", err)
		}
		if err = wc.Close(); err == nil {
			t.Error("Close expected an error, found none")
		}
	})
}

func TestIntegration_ReadCRC(t *testing.T) {
	// Test that the checksum is handled correctly when reading files.
	// For gzipped files, see https://github.com/GoogleCloudPlatform/google-cloud-dotnet/issues/1641.
	ctx := skipJSONReads(skipGRPC("transcoding not supported"), "https://github.com/googleapis/google-cloud-go/issues/7786")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		const (
			// This is an uncompressed file.
			// See https://cloud.google.com/storage/docs/public-datasets/landsat
			uncompressedBucket = "gcp-public-data-landsat"
			uncompressedObject = "LC08/01/001/002/LC08_L1GT_001002_20160817_20170322_01_T2/LC08_L1GT_001002_20160817_20170322_01_T2_ANG.txt"

			gzippedObject = "gzipped-text.txt"
		)

		h := testHelper{t}

		// Create gzipped object.
		var buf bytes.Buffer
		zw := gzip.NewWriter(&buf)
		zw.Name = gzippedObject
		if _, err := zw.Write([]byte("gzipped object data")); err != nil {
			t.Fatalf("creating gzip: %v", err)
		}
		if err := zw.Close(); err != nil {
			t.Fatalf("closing gzip writer: %v", err)
		}
		w := client.Bucket(bucket).Object(gzippedObject).NewWriter(ctx)
		w.ContentEncoding = "gzip"
		w.ContentType = "text/plain"
		h.mustWrite(w, buf.Bytes())

		for _, test := range []struct {
			desc           string
			obj            *ObjectHandle
			offset, length int64
			readCompressed bool // don't decompress a gzipped file

			wantErr   bool
			wantCheck bool // Should Reader try to check the CRC?
		}{
			{
				desc:           "uncompressed, entire file",
				obj:            client.Bucket(uncompressedBucket).Object(uncompressedObject),
				offset:         0,
				length:         -1,
				readCompressed: false,
				wantCheck:      true,
			},
			{
				desc:           "uncompressed, entire file, don't decompress",
				obj:            client.Bucket(uncompressedBucket).Object(uncompressedObject),
				offset:         0,
				length:         -1,
				readCompressed: true,
				wantCheck:      true,
			},
			{
				desc:           "uncompressed, suffix",
				obj:            client.Bucket(uncompressedBucket).Object(uncompressedObject),
				offset:         1,
				length:         -1,
				readCompressed: false,
				wantCheck:      false,
			},
			{
				desc:           "uncompressed, prefix",
				obj:            client.Bucket(uncompressedBucket).Object(uncompressedObject),
				offset:         0,
				length:         18,
				readCompressed: false,
				wantCheck:      false,
			},
			{
				// When a gzipped file is unzipped on read, we can't verify the checksum
				// because it was computed against the zipped contents. We can detect
				// this case using http.Response.Uncompressed.
				desc:           "compressed, entire file, unzipped",
				obj:            client.Bucket(bucket).Object(gzippedObject),
				offset:         0,
				length:         -1,
				readCompressed: false,
				wantCheck:      false,
			},
			{
				// When we read a gzipped file uncompressed, it's like reading a regular file:
				// the served content and the CRC match.
				desc:           "compressed, entire file, read compressed",
				obj:            client.Bucket(bucket).Object(gzippedObject),
				offset:         0,
				length:         -1,
				readCompressed: true,
				wantCheck:      true,
			},
			{
				desc:           "compressed, partial, server unzips",
				obj:            client.Bucket(bucket).Object(gzippedObject),
				offset:         1,
				length:         8,
				readCompressed: false,
				wantErr:        true, // GCS can't serve part of a gzipped object
				wantCheck:      false,
			},
			{
				desc:           "compressed, partial, read compressed",
				obj:            client.Bucket(bucket).Object(gzippedObject),
				offset:         1,
				length:         8,
				readCompressed: true,
				wantCheck:      false,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				// Test both Read and WriteTo.
				for _, c := range readCases {
					t.Run(c.desc, func(t *testing.T) {
						obj := test.obj.ReadCompressed(test.readCompressed)
						r, err := obj.NewRangeReader(ctx, test.offset, test.length)
						if err != nil {
							if test.wantErr {
								return
							}
							t.Fatalf("%s: %v", test.desc, err)
						}
						if got, want := r.checkCRC, test.wantCheck; got != want {
							t.Errorf("%s, checkCRC: got %t, want %t", test.desc, got, want)
						}
						_, err = c.readFunc(r)
						_ = r.Close()
						if err != nil {
							t.Fatalf("%s: %v", test.desc, err)
						}
					})

				}
			})
		}
	})
}

func TestIntegration_CancelWrite(t *testing.T) {
	// Verify that canceling the writer's context immediately stops uploading an object
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		cctx, cancel := context.WithCancel(ctx)
		defer cancel()
		obj := bkt.Object("cancel-write")
		w := obj.NewWriter(cctx)
		w.ChunkSize = googleapi.MinUploadChunkSize
		buf := make([]byte, w.ChunkSize)
		// Write the first chunk. This is read in its entirety before sending the request
		// (see google.golang.org/api/gensupport.PrepareUpload), so we expect it to return
		// without error.
		_, err := w.Write(buf)
		if err != nil {
			t.Fatal(err)
		}
		// Now cancel the context.
		cancel()
		// The next Write should return context.Canceled.
		_, err = w.Write(buf)
		if !errors.Is(err, context.Canceled) {
			t.Fatalf("got %v, wanted context.Canceled", err)
		}
		// The Close should too.
		err = w.Close()
		if !errors.Is(err, context.Canceled) {
			t.Fatalf("got %v, wanted context.Canceled", err)
		}
	})
}

func TestIntegration_UpdateCORS(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		initialSettings := []CORS{
			{
				MaxAge:          time.Hour,
				Methods:         []string{"POST"},
				Origins:         []string{"some-origin.com"},
				ResponseHeaders: []string{"foo-bar"},
			},
		}

		for _, test := range []struct {
			desc  string
			input []CORS
			want  []CORS
		}{
			{
				desc: "set new values",
				input: []CORS{
					{
						MaxAge:          time.Hour,
						Methods:         []string{"GET"},
						Origins:         []string{"*"},
						ResponseHeaders: []string{"some-header"},
					},
				},
				want: []CORS{
					{
						MaxAge:          time.Hour,
						Methods:         []string{"GET"},
						Origins:         []string{"*"},
						ResponseHeaders: []string{"some-header"},
					},
				},
			},
			{
				desc:  "set to empty to remove existing policies",
				input: []CORS{},
				want:  nil,
			},
			{
				desc:  "do not set to keep existing policies",
				input: nil,
				want: []CORS{
					{
						MaxAge:          time.Hour,
						Methods:         []string{"POST"},
						Origins:         []string{"some-origin.com"},
						ResponseHeaders: []string{"foo-bar"},
					},
				},
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				h := testHelper{t}

				bkt := client.Bucket(prefix + uidSpace.New())
				h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{CORS: initialSettings})
				defer h.mustDeleteBucket(bkt)
				h.mustUpdateBucket(bkt, BucketAttrsToUpdate{CORS: test.input}, h.mustBucketAttrs(bkt).MetaGeneration)
				attrs := h.mustBucketAttrs(bkt)
				if diff := testutil.Diff(attrs.CORS, test.want); diff != "" {
					t.Errorf("input: %v\ngot=-, want=+:\n%s", test.input, diff)
				}
			})
		}
	})
}

func TestIntegration_UpdateDefaultEventBasedHold(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{})
		defer h.mustDeleteBucket(bkt)
		attrs := h.mustBucketAttrs(bkt)
		if attrs.DefaultEventBasedHold != false {
			t.Errorf("got=%v, want=%v", attrs.DefaultEventBasedHold, false)
		}

		h.mustUpdateBucket(bkt, BucketAttrsToUpdate{DefaultEventBasedHold: true}, attrs.MetaGeneration)
		attrs = h.mustBucketAttrs(bkt)
		if attrs.DefaultEventBasedHold != true {
			t.Errorf("got=%v, want=%v", attrs.DefaultEventBasedHold, true)
		}

		// Omitting it should leave the value unchanged.
		h.mustUpdateBucket(bkt, BucketAttrsToUpdate{RequesterPays: true}, attrs.MetaGeneration)
		attrs = h.mustBucketAttrs(bkt)
		if attrs.DefaultEventBasedHold != true {
			t.Errorf("got=%v, want=%v", attrs.DefaultEventBasedHold, true)
		}
	})
}

func TestIntegration_UpdateEventBasedHold(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		obj := client.Bucket(bucket).Object("some-obj")
		h.mustWrite(obj.NewWriter(ctx), randomContents())

		defer func() {
			h.mustUpdateObject(obj, ObjectAttrsToUpdate{EventBasedHold: false}, h.mustObjectAttrs(obj).Metageneration)
			h.mustDeleteObject(obj)
		}()

		attrs := h.mustObjectAttrs(obj)
		if attrs.EventBasedHold != false {
			t.Fatalf("got=%v, want=%v", attrs.EventBasedHold, false)
		}

		h.mustUpdateObject(obj, ObjectAttrsToUpdate{EventBasedHold: true}, attrs.Metageneration)
		attrs = h.mustObjectAttrs(obj)
		if attrs.EventBasedHold != true {
			t.Fatalf("got=%v, want=%v", attrs.EventBasedHold, true)
		}

		// Omitting it should leave the value unchanged.
		h.mustUpdateObject(obj, ObjectAttrsToUpdate{ContentType: "foo"}, attrs.Metageneration)
		attrs = h.mustObjectAttrs(obj)
		if attrs.EventBasedHold != true {
			t.Fatalf("got=%v, want=%v", attrs.EventBasedHold, true)
		}
	})
}

func TestIntegration_UpdateTemporaryHold(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		obj := client.Bucket(bucket).Object("updatetemporaryhold-obj")
		h.mustWrite(obj.NewWriter(ctx), randomContents())

		defer func() {
			h.mustUpdateObject(obj, ObjectAttrsToUpdate{TemporaryHold: false}, h.mustObjectAttrs(obj).Metageneration)
			h.mustDeleteObject(obj)
		}()

		attrs := h.mustObjectAttrs(obj)
		if attrs.TemporaryHold != false {
			t.Fatalf("got=%v, want=%v", attrs.TemporaryHold, false)
		}

		h.mustUpdateObject(obj, ObjectAttrsToUpdate{TemporaryHold: true}, attrs.Metageneration)
		attrs = h.mustObjectAttrs(obj)
		if attrs.TemporaryHold != true {
			t.Fatalf("got=%v, want=%v", attrs.TemporaryHold, true)
		}

		// Omitting it should leave the value unchanged.
		h.mustUpdateObject(obj, ObjectAttrsToUpdate{ContentType: "foo"}, attrs.Metageneration)
		attrs = h.mustObjectAttrs(obj)
		if attrs.TemporaryHold != true {
			t.Fatalf("got=%v, want=%v", attrs.TemporaryHold, true)
		}
	})
}

func TestIntegration_UpdateRetentionExpirationTime(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{RetentionPolicy: &RetentionPolicy{RetentionPeriod: time.Hour}})
		obj := bkt.Object("some-obj")
		h.mustWrite(obj.NewWriter(ctx), randomContents())

		defer func() {
			t.Helper()
			h.mustUpdateBucket(bkt, BucketAttrsToUpdate{RetentionPolicy: &RetentionPolicy{RetentionPeriod: 0}}, h.mustBucketAttrs(bkt).MetaGeneration)

			// RetentionPeriod of less than a day is explicitly called out
			// as best effort and not guaranteed, so let's log problems deleting
			// objects instead of failing.
			if err := obj.Delete(context.Background()); err != nil {
				t.Logf("object delete: %v", err)
			}
			if err := bkt.Delete(context.Background()); err != nil {
				t.Logf("bucket delete: %v", err)
			}
		}()

		attrs := h.mustObjectAttrs(obj)
		if attrs.RetentionExpirationTime == (time.Time{}) {
			t.Fatalf("got=%v, wanted a non-zero value", attrs.RetentionExpirationTime)
		}
	})
}

func TestIntegration_CustomTime(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		h := testHelper{t}

		// Create object with CustomTime.
		bkt := client.Bucket(bucket)
		obj := bkt.Object("custom-time-obj")
		w := obj.NewWriter(ctx)
		ct := time.Date(2020, 8, 25, 12, 12, 12, 0, time.UTC)
		w.ObjectAttrs.CustomTime = ct
		h.mustWrite(w, randomContents())

		// Validate that CustomTime has been set
		checkCustomTime := func(want time.Time) error {
			attrs, err := obj.Attrs(ctx)
			if err != nil {
				return fmt.Errorf("failed to get object attrs: %v", err)
			}
			if got := attrs.CustomTime; got != want {
				return fmt.Errorf("CustomTime not set correctly: got %+v, want %+v", got, ct)
			}
			return nil
		}

		if err := checkCustomTime(ct); err != nil {
			t.Fatalf("checking CustomTime: %v", err)
		}

		// Update CustomTime to the future should succeed.
		laterTime := ct.Add(10 * time.Hour)
		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{CustomTime: laterTime}); err != nil {
			t.Fatalf("updating CustomTime: %v", err)
		}

		// Update CustomTime to the past should give error.
		earlierTime := ct.Add(5 * time.Hour)
		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{CustomTime: earlierTime}); err == nil {
			t.Fatalf("backdating CustomTime: expected error, got none")
		}

		// Zero value for CustomTime should be ignored. Set TemporaryHold so that
		// we don't send an empty update request, which is invalid for gRPC.
		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{TemporaryHold: false}); err != nil {
			t.Fatalf("empty update: %v", err)
		}
		if err := checkCustomTime(laterTime); err != nil {
			t.Fatalf("after sending zero value: %v", err)
		}
	})
}

func TestIntegration_UpdateRetentionPolicy(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		initial := &RetentionPolicy{RetentionPeriod: time.Minute}

		for _, test := range []struct {
			desc  string
			input *RetentionPolicy
			want  *RetentionPolicy
		}{
			{
				desc:  "update",
				input: &RetentionPolicy{RetentionPeriod: time.Hour},
				want:  &RetentionPolicy{RetentionPeriod: time.Hour},
			},
			{
				desc:  "update even with timestamp (EffectiveTime should be ignored)",
				input: &RetentionPolicy{RetentionPeriod: time.Hour, EffectiveTime: time.Now()},
				want:  &RetentionPolicy{RetentionPeriod: time.Hour},
			},
			{
				desc:  "remove",
				input: &RetentionPolicy{},
				want:  nil,
			},
			{
				desc:  "remove even with timestamp (EffectiveTime should be ignored)",
				input: &RetentionPolicy{EffectiveTime: time.Now().Add(time.Hour)},
				want:  nil,
			},
			{
				desc:  "ignore",
				input: nil,
				want:  initial,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				h := testHelper{t}
				bkt := client.Bucket(prefix + uidSpace.New())
				h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{RetentionPolicy: initial})
				defer h.mustDeleteBucket(bkt)
				h.mustUpdateBucket(bkt, BucketAttrsToUpdate{RetentionPolicy: test.input}, h.mustBucketAttrs(bkt).MetaGeneration)

				attrs := h.mustBucketAttrs(bkt)
				if attrs.RetentionPolicy != nil && attrs.RetentionPolicy.EffectiveTime.Unix() == 0 {
					// Should be set by the server and parsed by the client
					t.Fatal("EffectiveTime should be set, but it was not")
				}
				if diff := testutil.Diff(attrs.RetentionPolicy, test.want, cmpopts.IgnoreTypes(time.Time{})); diff != "" {
					t.Errorf("input: %v\ngot=-, want=+:\n%s", test.input, diff)
				}
			})
		}
	})
}

func TestIntegration_DeleteObjectInBucketWithRetentionPolicy(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{RetentionPolicy: &RetentionPolicy{RetentionPeriod: 25 * time.Hour}})
		defer h.mustDeleteBucket(bkt)

		o := bkt.Object("some-object")
		if err := writeObject(ctx, o, "text/plain", []byte("hello world")); err != nil {
			t.Fatal(err)
		}

		if err := o.Delete(ctx); err == nil {
			t.Fatal("expected to err deleting an object in a bucket with retention period, but got nil")
		}

		// Remove the retention period
		h.mustUpdateBucket(bkt, BucketAttrsToUpdate{RetentionPolicy: &RetentionPolicy{}}, h.mustBucketAttrs(bkt).MetaGeneration)

		// Delete with retry, as bucket metadata changes
		// can take some time to propagate.
		retry := func(err error) bool { return err != nil }
		ctx, cancel := context.WithTimeout(ctx, time.Second*10)
		defer cancel()

		o = o.Retryer(WithErrorFunc(retry), WithPolicy(RetryAlways))
		if err := o.Delete(ctx); err != nil {
			t.Fatalf("object delete: %v", err)
		}
	})
}

func TestIntegration_LockBucket(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{RetentionPolicy: &RetentionPolicy{RetentionPeriod: time.Hour * 25}})
		attrs := h.mustBucketAttrs(bkt)
		if attrs.RetentionPolicy.IsLocked {
			t.Fatal("Expected bucket to begin unlocked, but it was not")
		}
		err := bkt.If(BucketConditions{MetagenerationMatch: attrs.MetaGeneration}).LockRetentionPolicy(ctx)
		if err != nil {
			t.Fatal("could not lock", err)
		}

		attrs = h.mustBucketAttrs(bkt)
		if !attrs.RetentionPolicy.IsLocked {
			t.Fatal("Expected bucket to be locked, but it was not")
		}

		_, err = bkt.Update(ctx, BucketAttrsToUpdate{RetentionPolicy: &RetentionPolicy{RetentionPeriod: time.Hour}})
		if err == nil {
			t.Fatal("Expected error updating locked bucket, got nil")
		}
	})
}

func TestIntegration_LockBucket_MetagenerationRequired(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{
			RetentionPolicy: &RetentionPolicy{RetentionPeriod: time.Hour * 25},
		})
		err := bkt.LockRetentionPolicy(ctx)
		if err == nil {
			t.Fatal("expected error locking bucket without metageneration condition, got nil")
		}
	})
}

func TestIntegration_BucketObjectRetention(t *testing.T) {
	ctx := skipJSONReads(skipGRPC("not yet available in gRPC - b/308194853"), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		setTrue, setFalse := true, false

		for _, test := range []struct {
			desc              string
			enable            *bool
			wantRetentionMode string
		}{
			{
				desc:              "ObjectRetentionMode is not enabled by default",
				wantRetentionMode: "",
			},
			{
				desc:              "Enable retention",
				enable:            &setTrue,
				wantRetentionMode: "Enabled",
			},
			{
				desc:              "Set object retention to false",
				enable:            &setFalse,
				wantRetentionMode: "",
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				b := client.Bucket(prefix + uidSpace.New())
				if test.enable != nil {
					b = b.SetObjectRetention(*test.enable)
				}

				err := b.Create(ctx, testutil.ProjID(), nil)
				if err != nil {
					t.Fatalf("error creating bucket: %v", err)
				}
				t.Cleanup(func() { b.Delete(ctx) })

				attrs, err := b.Attrs(ctx)
				if err != nil {
					t.Fatalf("b.Attrs: %v", err)
				}
				if got, want := attrs.ObjectRetentionMode, test.wantRetentionMode; got != want {
					t.Errorf("expected ObjectRetentionMode to be %q, got %q", want, got)
				}
			})
		}
	})
}

func TestIntegration_ObjectRetention(t *testing.T) {
	ctx := skipJSONReads(skipGRPC("not yet available in gRPC - b/308194853"), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		b := client.Bucket(prefix + uidSpace.New()).SetObjectRetention(true)

		if err := b.Create(ctx, testutil.ProjID(), nil); err != nil {
			t.Fatalf("error creating bucket: %v", err)
		}
		t.Cleanup(func() { h.mustDeleteBucket(b) })

		retentionUnlocked := &ObjectRetention{
			Mode:        "Unlocked",
			RetainUntil: time.Now().Add(time.Minute * 20).Truncate(time.Second),
		}
		retentionUnlockedExtended := &ObjectRetention{
			Mode:        "Unlocked",
			RetainUntil: time.Now().Add(time.Hour).Truncate(time.Second),
		}

		// Create an object with future retain until time
		o := b.Object("retention-on-create" + uidSpaceObjects.New())
		w := o.NewWriter(ctx)
		w.Retention = retentionUnlocked
		h.mustWrite(w, []byte("contents"))
		t.Cleanup(func() {
			if _, err := o.OverrideUnlockedRetention(true).Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{}}); err != nil {
				t.Fatalf("failed to remove retention from object: %v", err)
			}
			h.mustDeleteObject(o)
		})

		if got, want := w.Attrs().Retention, retentionUnlocked; got.Mode != want.Mode || !got.RetainUntil.Equal(want.RetainUntil) {
			t.Errorf("mismatching retention config, got: %+v, want:%+v", got, want)
		}

		// Delete object under retention returns 403
		if err := o.Delete(ctx); err == nil || extractErrCode(err) != http.StatusForbidden {
			t.Fatalf("delete should have failed with: %v, instead got:%v", http.StatusForbidden, err)
		}

		// Extend retain until time of Unlocked object is possible
		attrs, err := o.Update(ctx, ObjectAttrsToUpdate{Retention: retentionUnlockedExtended})
		if err != nil {
			t.Fatalf("failed to add retention to object: %v", err)
		}

		if got, want := attrs.Retention, retentionUnlockedExtended; got.Mode != want.Mode || !got.RetainUntil.Equal(want.RetainUntil) {
			t.Errorf("mismatching retention config, got: %+v, want:%+v", got, want)
		}

		// Reduce retain until time of Unlocked object without
		// override_unlocked_retention=True returns 403
		_, err = o.Update(ctx, ObjectAttrsToUpdate{Retention: retentionUnlocked})
		if err == nil || extractErrCode(err) != http.StatusForbidden {
			t.Fatalf("o.Update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		// Remove retention of Unlocked object without
		// override_unlocked_retention=True returns 403
		_, err = o.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{}})
		if err == nil || extractErrCode(err) != http.StatusForbidden {
			t.Fatalf("o.Update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		// Reduce retain until time of Unlocked object with override_unlocked_retention=True
		attrs, err = o.OverrideUnlockedRetention(true).Update(ctx, ObjectAttrsToUpdate{
			Retention: retentionUnlocked,
		})
		if err != nil {
			t.Fatalf("failed to add retention to object: %v", err)
		}

		if got, want := attrs.Retention, retentionUnlocked; got.Mode != want.Mode || !got.RetainUntil.Equal(want.RetainUntil) {
			t.Errorf("mismatching retention config, got: %+v, want:%+v", got, want)
		}

		// Create a new object
		objectWithRetentionOnUpdate := b.Object("retention-on-update" + uidSpaceObjects.New())
		w = objectWithRetentionOnUpdate.NewWriter(ctx)
		h.mustWrite(w, []byte("contents"))

		// Retention should not be set
		if got := w.Attrs().Retention; got != nil {
			t.Errorf("expected no ObjectRetention, got: %+v", got)
		}

		// Update object with only one of (retain until time, retention mode) returns 400
		_, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{Mode: "Locked"}})
		if err == nil || extractErrCode(err) != http.StatusBadRequest {
			t.Errorf("update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		_, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{RetainUntil: time.Now().Add(time.Second)}})
		if err == nil || extractErrCode(err) != http.StatusBadRequest {
			t.Errorf("update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		// Update object with future retain until time
		attrs, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: retentionUnlocked})
		if err != nil {
			t.Errorf("o.Update: %v", err)
		}

		if got, want := attrs.Retention, retentionUnlocked; got.Mode != want.Mode || !got.RetainUntil.Equal(want.RetainUntil) {
			t.Errorf("mismatching retention config, got: %+v, want:%+v", got, want)
		}

		// Update/Patch object with retain until time in the past returns 400
		_, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{RetainUntil: time.Now().Add(-time.Second)}})
		if err == nil || extractErrCode(err) != http.StatusBadRequest {
			t.Errorf("update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		// Update object with only one of (retain until time, retention mode) returns 400
		_, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{Mode: "Locked"}})
		if err == nil || extractErrCode(err) != http.StatusBadRequest {
			t.Errorf("update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		_, err = objectWithRetentionOnUpdate.Update(ctx, ObjectAttrsToUpdate{Retention: &ObjectRetention{RetainUntil: time.Now().Add(time.Second)}})
		if err == nil || extractErrCode(err) != http.StatusBadRequest {
			t.Errorf("update should have failed with: %v, instead got:%v", http.StatusBadRequest, err)
		}

		// Remove retention of Unlocked object with override_unlocked_retention=True
		attrs, err = objectWithRetentionOnUpdate.OverrideUnlockedRetention(true).Update(ctx, ObjectAttrsToUpdate{
			Retention: &ObjectRetention{},
		})
		if err != nil {
			t.Fatalf("failed to remove retention from object: %v", err)
		}

		if got := attrs.Retention; got != nil {
			t.Errorf("mismatching retention config, got: %+v, wanted nil", got)
		}

		// We should be able to delete the object as normal since retention was removed
		if err := objectWithRetentionOnUpdate.Delete(ctx); err != nil {
			t.Errorf("object.Delete:%v", err)
		}
	})
}

func TestIntegration_SoftDelete(t *testing.T) {
	multiTransportTest(skipJSONReads(context.Background(), "does not test reads"), t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}
		testStart := time.Now()

		policy := &SoftDeletePolicy{
			RetentionDuration: time.Hour * 24 * 8,
		}

		b := client.Bucket(prefix + uidSpace.New())

		// Create bucket with soft delete policy.
		if err := b.Create(ctx, testutil.ProjID(), &BucketAttrs{SoftDeletePolicy: policy}); err != nil {
			t.Fatalf("error creating bucket with soft delete policy set: %v", err)
		}
		t.Cleanup(func() { h.mustDeleteBucket(b) })

		// Get bucket's soft delete policy and confirm accuracy.
		attrs, err := b.Attrs(ctx)
		if err != nil {
			t.Fatalf("b.Attrs(%q): %v", b.name, err)
		}

		got := attrs.SoftDeletePolicy
		if got == nil {
			t.Fatal("got nil soft delete policy")
		}
		if got.RetentionDuration != policy.RetentionDuration {
			t.Fatalf("mismatching retention duration; got soft delete policy: %+v, expected: %+v", got, policy)
		}
		if got.EffectiveTime.Before(testStart) {
			t.Fatalf("effective time of soft delete policy should not be in the past, got: %v, test start: %v", got.EffectiveTime, testStart.UTC())
		}

		// Update the soft delete policy.
		policy.RetentionDuration = time.Hour * 24 * 9

		attrs, err = b.Update(ctx, BucketAttrsToUpdate{SoftDeletePolicy: policy})
		if err != nil {
			t.Fatalf("b.Update: %v", err)
		}

		if got, expect := attrs.SoftDeletePolicy.RetentionDuration, policy.RetentionDuration; got != expect {
			t.Fatalf("mismatching retention duration; got: %+v, expected: %+v", got, expect)
		}

		// Create 2 objects and delete one of them.
		deletedObject := b.Object("soft-delete" + uidSpaceObjects.New())
		liveObject := b.Object("not-soft-delete" + uidSpaceObjects.New())

		h.mustWrite(deletedObject.NewWriter(ctx), []byte("soft-deleted"))
		h.mustWrite(liveObject.NewWriter(ctx), []byte("soft-delete"))
		t.Cleanup(func() {
			h.mustDeleteObject(liveObject)
			h.mustDeleteObject(deletedObject)
		})

		h.mustDeleteObject(deletedObject)

		var gen int64
		// List soft deleted objects.
		it := b.Objects(ctx, &Query{SoftDeleted: true})
		var gotNames []string
		for {
			attrs, err := it.Next()
			if err == iterator.Done {
				break
			}
			if err != nil {
				t.Fatalf("iterator.Next: %v", err)
			}
			gotNames = append(gotNames, attrs.Name)

			// Get the generation here as the test will fail if there is more than one object
			gen = attrs.Generation
		}
		if len(gotNames) != 1 || gotNames[0] != deletedObject.ObjectName() {
			t.Fatalf("list soft deleted objects; got: %v, expected only one object named: %s", gotNames, deletedObject.ObjectName())
		}

		// List live objects.
		gotNames = []string{}
		it = b.Objects(ctx, nil)
		for {
			attrs, err := it.Next()
			if err == iterator.Done {
				break
			}
			if err != nil {
				t.Fatalf("iterator.Next: %v", err)
			}
			gotNames = append(gotNames, attrs.Name)
		}
		if len(gotNames) != 1 || gotNames[0] != liveObject.ObjectName() {
			t.Fatalf("list objects that are not soft deleted; got: %v, expected only one object named: %s", gotNames, liveObject.ObjectName())
		}

		// Get a soft deleted object and check soft and hard delete times.
		oAttrs, err := deletedObject.Generation(gen).SoftDeleted().Attrs(ctx)
		if err != nil {
			t.Fatalf("deletedObject.SoftDeleted().Attrs: %v", err)
		}
		if oAttrs.SoftDeleteTime.Before(testStart) {
			t.Fatalf("SoftDeleteTime of soft deleted object should not be in the past, got: %v, test start: %v", oAttrs.SoftDeleteTime, testStart.UTC())
		}
		if got, expected := oAttrs.HardDeleteTime, oAttrs.SoftDeleteTime.Add(policy.RetentionDuration); !expected.Equal(got) {
			t.Fatalf("HardDeleteTime of soft deleted object should be equal to SoftDeleteTime+RetentionDuration, got: %v, expected: %v", got, expected)
		}

		// Restore a soft deleted object.
		_, err = deletedObject.Generation(gen).Restore(ctx, &RestoreOptions{CopySourceACL: true})
		if err != nil {
			t.Fatalf("Object(deletedObject).Restore: %v", err)
		}

		// Update the soft delete policy to remove it.
		attrs, err = b.Update(ctx, BucketAttrsToUpdate{SoftDeletePolicy: &SoftDeletePolicy{}})
		if err != nil {
			t.Fatalf("b.Update: %v", err)
		}

		if got, expect := attrs.SoftDeletePolicy.RetentionDuration, time.Duration(0); got != expect {
			t.Fatalf("mismatching retention duration; got: %+v, expected: %+v", got, expect)
		}
	})
}

func TestIntegration_KMS(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, prefix string, client *Client) {
		h := testHelper{t}

		keyRingName := os.Getenv("GCLOUD_TESTS_GOLANG_KEYRING")
		if keyRingName == "" {
			t.Fatal("GCLOUD_TESTS_GOLANG_KEYRING must be set. See CONTRIBUTING.md for details")
		}
		keyName1 := keyRingName + "/cryptoKeys/key1"
		keyName2 := keyRingName + "/cryptoKeys/key2"
		contents := []byte("my secret")

		write := func(obj *ObjectHandle, setKey bool) {
			w := obj.NewWriter(ctx)
			if setKey {
				w.KMSKeyName = keyName1
			}
			h.mustWrite(w, contents)
		}

		checkRead := func(obj *ObjectHandle) {
			got := h.mustRead(obj)
			if !bytes.Equal(got, contents) {
				t.Errorf("got %v, want %v", got, contents)
			}
			attrs := h.mustObjectAttrs(obj)
			if len(attrs.KMSKeyName) < len(keyName1) || attrs.KMSKeyName[:len(keyName1)] != keyName1 {
				t.Errorf("got %q, want %q", attrs.KMSKeyName, keyName1)
			}
		}

		// Write an object with a key, then read it to verify its contents and the presence of the key name.
		bkt := client.Bucket(bucket)
		obj := bkt.Object("kms")
		write(obj, true)
		checkRead(obj)
		h.mustDeleteObject(obj)

		// Encrypt an object with a CSEK, then copy it using a CMEK.
		src := bkt.Object("csek").Key(testEncryptionKey)
		if err := writeObject(ctx, src, "text/plain", contents); err != nil {
			t.Fatal(err)
		}
		dest := bkt.Object("cmek")
		c := dest.CopierFrom(src)
		c.DestinationKMSKeyName = keyName1
		if _, err := c.Run(ctx); err != nil {
			t.Fatal(err)
		}
		checkRead(dest)
		src.Delete(ctx)
		dest.Delete(ctx)

		// Create a bucket with a default key, then write and read an object.
		bkt = client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{
			Location:   "US",
			Encryption: &BucketEncryption{DefaultKMSKeyName: keyName1},
		})
		defer h.mustDeleteBucket(bkt)

		attrs := h.mustBucketAttrs(bkt)
		if got, want := attrs.Encryption.DefaultKMSKeyName, keyName1; got != want {
			t.Fatalf("got %q, want %q", got, want)
		}
		obj = bkt.Object("kms")
		write(obj, false)
		checkRead(obj)
		h.mustDeleteObject(obj)

		// Update the bucket's default key to a different name.
		// (This key doesn't have to exist.)
		attrs = h.mustUpdateBucket(bkt, BucketAttrsToUpdate{Encryption: &BucketEncryption{DefaultKMSKeyName: keyName2}}, attrs.MetaGeneration)
		if got, want := attrs.Encryption.DefaultKMSKeyName, keyName2; got != want {
			t.Fatalf("got %q, want %q", got, want)
		}
		attrs = h.mustBucketAttrs(bkt)
		if got, want := attrs.Encryption.DefaultKMSKeyName, keyName2; got != want {
			t.Fatalf("got %q, want %q", got, want)
		}

		// Remove the default KMS key.
		attrs = h.mustUpdateBucket(bkt, BucketAttrsToUpdate{Encryption: &BucketEncryption{DefaultKMSKeyName: ""}}, attrs.MetaGeneration)
		if attrs.Encryption != nil {
			t.Fatalf("got %#v, want nil", attrs.Encryption)
		}
	})
}

func TestIntegration_PredefinedACLs(t *testing.T) {
	projectOwners := prefixRoleACL{prefix: "project-owners", role: RoleOwner}
	userOwner := prefixRoleACL{prefix: "user", role: RoleOwner}
	authenticatedRead := entityRoleACL{entity: AllAuthenticatedUsers, role: RoleReader}

	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, testutil.ProjID(), &BucketAttrs{
			PredefinedACL:              "authenticatedRead",
			PredefinedDefaultObjectACL: "publicRead",
		})
		defer h.mustDeleteBucket(bkt)
		attrs := h.mustBucketAttrs(bkt)

		if acl, want := attrs.ACL, projectOwners; !containsACLRule(acl, want) {
			t.Fatalf("Bucket.ACL: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if acl, want := attrs.ACL, authenticatedRead; !containsACLRule(acl, want) {
			t.Fatalf("Bucket.ACL: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if acl := attrs.DefaultObjectACL; !containsACLRule(acl, entityRoleACL{AllUsers, RoleReader}) {
			t.Fatalf("DefaultObjectACL: expected acl to contain: %+v, got acl: %+v", entityRoleACL{AllUsers, RoleReader}, acl)
		}

		// Bucket update
		attrs = h.mustUpdateBucket(bkt, BucketAttrsToUpdate{
			PredefinedACL:              "private",
			PredefinedDefaultObjectACL: "authenticatedRead",
		}, attrs.MetaGeneration)
		if acl, want := attrs.ACL, projectOwners; !containsACLRule(acl, want) {
			t.Fatalf("Bucket.ACL update: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if acl, want := attrs.DefaultObjectACL, authenticatedRead; !containsACLRule(acl, want) {
			t.Fatalf("DefaultObjectACL update: expected acl to contain: %+v, got acl: %+v", want, acl)
		}

		// Object creation
		obj := bkt.Object("private")
		w := obj.NewWriter(ctx)
		w.PredefinedACL = "authenticatedRead"
		h.mustWrite(w, []byte("hello"))
		defer h.mustDeleteObject(obj)
		var acl []ACLRule
		err := retry(ctx, func() error {
			attrs, err := obj.Attrs(ctx)
			if err != nil {
				return fmt.Errorf("Object.Attrs: object metadata get failed: %v", err)
			}
			acl = attrs.ACL
			return nil
		}, func() error {
			if want := userOwner; !containsACLRule(acl, want) {
				return fmt.Errorf("Object.ACL: expected acl to contain: %+v, got acl: %+v", want, acl)
			}
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}
		err = retry(ctx, func() error {
			attrs, err := obj.Attrs(ctx)
			if err != nil {
				return fmt.Errorf("Object.Attrs: object metadata get failed: %v", err)
			}
			acl = attrs.ACL
			return nil
		}, func() error {
			if want := authenticatedRead; !containsACLRule(acl, want) {
				return fmt.Errorf("Object.ACL: expected acl to contain: %+v, got acl: %+v", want, acl)
			}
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}

		// Object update
		oattrs := h.mustUpdateObject(obj, ObjectAttrsToUpdate{PredefinedACL: "private"}, h.mustObjectAttrs(obj).Metageneration)
		if acl, want := oattrs.ACL, userOwner; !containsACLRule(acl, want) {
			t.Fatalf("Object.ACL update: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if got := len(oattrs.ACL); got != 1 {
			t.Errorf("got %d ACL rules, want 1", got)
		}

		// Copy
		dst := bkt.Object("dst")
		copier := dst.CopierFrom(obj)
		copier.PredefinedACL = "publicRead"
		oattrs, err = copier.Run(ctx)
		if err != nil {
			t.Fatal(err)
		}
		defer h.mustDeleteObject(dst)
		// The copied object still retains the "private" ACL of the source object.
		if acl, want := oattrs.ACL, userOwner; !containsACLRule(acl, want) {
			t.Fatalf("copy dest: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if !containsACLRule(oattrs.ACL, entityRoleACL{AllUsers, RoleReader}) {
			t.Fatalf("copy dest: expected acl to contain: %+v, got acl: %+v", entityRoleACL{AllUsers, RoleReader}, oattrs.ACL)
		}

		// Compose
		comp := bkt.Object("comp")

		composer := comp.ComposerFrom(obj, dst)
		composer.PredefinedACL = "authenticatedRead"
		oattrs, err = composer.Run(ctx)
		if err != nil {
			t.Fatal(err)
		}
		defer h.mustDeleteObject(comp)
		// The composed object still retains the "private" ACL.
		if acl, want := oattrs.ACL, userOwner; !containsACLRule(acl, want) {
			t.Fatalf("compose: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
		if acl, want := oattrs.ACL, authenticatedRead; !containsACLRule(acl, want) {
			t.Fatalf("compose: expected acl to contain: %+v, got acl: %+v", want, acl)
		}
	})
}

func TestIntegration_ServiceAccount(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _, _ string, client *Client) {
		s, err := client.ServiceAccount(ctx, testutil.ProjID())
		if err != nil {
			t.Fatal(err)
		}
		want := "@gs-project-accounts.iam.gserviceaccount.com"
		if !strings.Contains(s, want) {
			t.Fatalf("got %v, want to contain %v", s, want)
		}
	})
}

func TestIntegration_Reader(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		b := client.Bucket(bucket)
		const defaultType = "text/plain"

		// Populate object names and make a map for their contents.
		objects := []string{
			"obj1",
			"obj2",
			"obj/with/slashes",
			"obj/",
			"./obj",
			"!#$&'()*+,/:;=,?@,[] and spaces",
		}
		contents := make(map[string][]byte)

		// Write objects.
		for _, obj := range objects {
			c := randomContents()
			if err := writeObject(ctx, b.Object(obj), defaultType, c); err != nil {
				t.Errorf("Write for %v failed with %v", obj, err)
			}
			contents[obj] = c
		}

		// Test Reader. Cache control and last-modified are tested separately, as
		// the JSON and XML APIs return different values for these.
		for _, obj := range objects {
			t.Run(obj, func(t *testing.T) {
				// Test both Read and WriteTo.
				for _, c := range readCases {
					t.Run(c.desc, func(t *testing.T) {
						rc, err := b.Object(obj).NewReader(ctx)
						if err != nil {
							t.Fatalf("Can't create a reader for %v, errored with %v", obj, err)
						}
						if !rc.checkCRC {
							t.Errorf("%v: not checking CRC", obj)
						}

						slurp, err := c.readFunc(rc)
						if err != nil {
							t.Errorf("Can't read object %v, errored with %v", obj, err)
						}
						if got, want := slurp, contents[obj]; !bytes.Equal(got, want) {
							t.Errorf("Contents (%q) = %q; want %q", obj, got, want)
						}
						if got, want := rc.Size(), len(contents[obj]); got != int64(want) {
							t.Errorf("Size (%q) = %d; want %d", obj, got, want)
						}
						if got, want := rc.ContentType(), "text/plain"; got != want {
							t.Errorf("ContentType (%q) = %q; want %q", obj, got, want)
						}
						rc.Close()

						// Check early close.
						buf := make([]byte, 1)
						rc, err = b.Object(obj).NewReader(ctx)
						if err != nil {
							t.Fatalf("%v: %v", obj, err)
						}
						_, err = rc.Read(buf)
						if err != nil {
							t.Fatalf("%v: %v", obj, err)
						}
						if got, want := buf, contents[obj][:1]; !bytes.Equal(got, want) {
							t.Errorf("Contents[0] (%q) = %q; want %q", obj, got, want)
						}
						if err := rc.Close(); err != nil {
							t.Errorf("%v Close: %v", obj, err)
						}
					})
				}

			})
		}

		obj := objects[0]
		objlen := int64(len(contents[obj]))

		// Test Range Reader.
		for _, r := range []struct {
			desc                 string
			offset, length, want int64
		}{
			{"entire object", 0, objlen, objlen},
			{"first half of object", 0, objlen / 2, objlen / 2},
			{"second half of object", objlen / 2, objlen, objlen / 2},
			{"no bytes - start at beginning", 0, 0, 0},
			{"no bytes - start halfway through", objlen / 2, 0, 0},
			{"start halfway through - use negative to get rest of obj", objlen / 2, -1, objlen / 2},
			{"2 times object length", 0, objlen * 2, objlen},
			{"-2 offset", -2, -1, 2},
			{"-object length offset", -objlen, -1, objlen},
			{"-half of object length offset", -(objlen / 2), -1, objlen / 2},
		} {
			t.Run(r.desc, func(t *testing.T) {

				for _, c := range readCases {
					t.Run(c.desc, func(t *testing.T) {
						rc, err := b.Object(obj).NewRangeReader(ctx, r.offset, r.length)
						if err != nil {
							t.Fatalf("%+v: Can't create a range reader for %v, errored with %v", r.desc, obj, err)
						}
						if rc.Size() != objlen {
							t.Errorf("%+v: Reader has a content-size of %d, want %d", r.desc, rc.Size(), objlen)
						}
						if rc.Remain() != r.want {
							t.Errorf("%+v: Reader's available bytes reported as %d, want %d", r.desc, rc.Remain(), r.want)
						}
						slurp, err := c.readFunc(rc)
						if err != nil {
							t.Fatalf("%+v: can't read object %v, errored with %v", r, obj, err)
						}
						if len(slurp) != int(r.want) {
							t.Fatalf("%+v: RangeReader (%d, %d): Read %d bytes, wanted %d bytes", r.desc, r.offset, r.length, len(slurp), r.want)
						}

						switch {
						case r.offset < 0: // The case of reading the last N bytes.
							start := objlen + r.offset
							if got, want := slurp, contents[obj][start:]; !bytes.Equal(got, want) {
								t.Errorf("RangeReader (%d, %d) = %q; want %q", r.offset, r.length, got, want)
							}

						default:
							if got, want := slurp, contents[obj][r.offset:r.offset+r.want]; !bytes.Equal(got, want) {
								t.Errorf("RangeReader (%d, %d) = %q; want %q", r.offset, r.length, got, want)
							}
						}
						rc.Close()
					})
				}
			})
		}

		objName := objects[0]

		// Test NewReader googleapi.Error.
		// Since a 429 or 5xx is hard to cause, we trigger a 416 (InvalidRange).
		realLen := len(contents[objName])
		_, err := b.Object(objName).NewRangeReader(ctx, int64(realLen*2), 10)

		var e *googleapi.Error
		if !errors.As(err, &e) {
			// Check if it is the correct GRPC error
			if !(status.Code(err) == codes.OutOfRange) {
				t.Errorf("NewRangeReader did not return a googleapi.Error nor GRPC OutOfRange error; got: %v", err)
			}
		} else {
			if e.Code != 416 {
				t.Errorf("Code = %d; want %d", e.Code, 416)
			}
			if len(e.Header) == 0 {
				t.Error("Missing googleapi.Error.Header")
			}
			if len(e.Body) == 0 {
				t.Error("Missing googleapi.Error.Body")
			}
		}
	})
}

func TestIntegration_ReaderAttrs(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)

		const defaultType = "text/plain"
		o := bkt.Object("reader-attrs-obj")
		c := randomContents()
		if err := writeObject(ctx, o, defaultType, c); err != nil {
			t.Errorf("Write for %v failed with %v", o.ObjectName(), err)
		}
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()

		rc, err := o.NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}

		attrs, err := o.Attrs(ctx)
		if err != nil {
			t.Fatal(err)
		}

		got := rc.Attrs
		want := ReaderObjectAttrs{
			Size:            attrs.Size,
			ContentType:     attrs.ContentType,
			ContentEncoding: attrs.ContentEncoding,
			CacheControl:    got.CacheControl, // ignored, tested separately
			LastModified:    got.LastModified, // ignored, tested separately
			Generation:      attrs.Generation,
			Metageneration:  attrs.Metageneration,
		}
		if got != want {
			t.Fatalf("got\t%v,\nwanted\t%v", got, want)
		}
	})
}

func TestIntegration_ReaderLastModified(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "LastModified not populated by json response")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		testStart := time.Now()
		b := client.Bucket(bucket)
		o := b.Object("reader-lm-obj" + uidSpaceObjects.New())

		if err := writeObject(ctx, o, "text/plain", randomContents()); err != nil {
			t.Errorf("Write for %v failed with %v", o.ObjectName(), err)
		}
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()

		r, err := o.NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}

		lm := r.Attrs.LastModified
		if lm.IsZero() {
			t.Fatal("LastModified is 0, should be >0")
		}

		// We just wrote this object, so it should have a recent last-modified time.
		// Accept a time within the start + variance of the test, to account for natural
		// variation.
		expectedVariance := time.Minute

		if lm.After(testStart.Add(expectedVariance)) {
			t.Errorf("LastModified (%q): got %s, which is not within %v from test start (%v)", o.ObjectName(), lm, expectedVariance, testStart)
		}
	})
}

func TestIntegration_ReaderCacheControl(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "Cache control header is populated differently by the json api")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		b := client.Bucket(bucket)
		o := b.Object("reader-cc" + uidSpaceObjects.New())

		cacheControl := "public, max-age=60"

		// Write object.
		w := o.Retryer(WithPolicy(RetryAlways)).NewWriter(ctx)
		w.CacheControl = cacheControl
		if _, err := w.Write(randomContents()); err != nil {
			t.Fatalf("Write for %v failed with %v", o.ObjectName(), err)
		}
		if err := w.Close(); err != nil {
			t.Fatalf("Write close for %v failed with %v", o.ObjectName(), err)
		}
		defer func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()

		// Check cache control on reader attrs.
		r, err := o.NewReader(ctx)
		if err != nil {
			t.Fatal(err)
		}

		if got, want := r.Attrs.CacheControl, cacheControl; got != want {
			t.Fatalf("cache control; got: %s, want: %s", got, want)
		}
	})
}

func TestIntegration_ReaderErrObjectNotExist(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		o := client.Bucket(bucket).Object("non-existing")

		_, err := o.NewReader(ctx)
		if !errors.Is(err, ErrObjectNotExist) {
			t.Fatalf("expected ErrObjectNotExist, got %v", err)
		}
	})
}

// TestIntegration_JSONReaderConditions tests only JSON reads as some conditions
// do not work with XML.
func TestIntegration_JSONReaderConditions(t *testing.T) {
	ctx := skipXMLReads(skipGRPC("json-only test"), "json-only test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		b := client.Bucket(bucket)
		o := b.Object("reader-conditions" + uidSpaceObjects.New())

		// Write object.
		w := o.Retryer(WithPolicy(RetryAlways)).NewWriter(ctx)
		if _, err := w.Write(randomContents()); err != nil {
			t.Fatalf("Write for %v failed with %v", o.ObjectName(), err)
		}
		if err := w.Close(); err != nil {
			t.Fatalf("Write close for %v failed with %v", o.ObjectName(), err)
		}

		t.Cleanup(func() {
			if err := o.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		})

		// Get current gens.
		attrs, err := o.Attrs(ctx)
		if err != nil {
			t.Fatalf("o.Attrs(%s): %v", o.ObjectName(), err)
		}
		currGen := attrs.Generation
		currMetagen := attrs.Metageneration

		// Test each condition to make sure it is passed through correctly.
		for _, test := range []struct {
			desc        string
			conds       Conditions
			wantErrCode int
		}{
			{
				desc:        "GenerationMatch incorrect gen",
				conds:       Conditions{GenerationMatch: currGen + 2},
				wantErrCode: 412,
			},
			{
				desc:        "GenerationNotMatch current gen",
				conds:       Conditions{GenerationNotMatch: currGen},
				wantErrCode: 304,
			},
			{
				desc:        "DoesNotExist set to true",
				conds:       Conditions{DoesNotExist: true},
				wantErrCode: 412,
			},
			{
				desc:        "MetagenerationMatch incorrect gen",
				conds:       Conditions{MetagenerationMatch: currMetagen + 1},
				wantErrCode: 412,
			},
			{
				desc:        "MetagenerationNotMatch current gen",
				conds:       Conditions{MetagenerationNotMatch: currMetagen},
				wantErrCode: 304,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				o := o.If(test.conds)
				_, err := o.NewReader(ctx)

				got := extractErrCode(err)
				if test.wantErrCode != got {
					t.Errorf("want err code: %v, got err: %v", test.wantErrCode, err)
				}
			})
		}
	})
}

// Test that context cancellation correctly stops a download before completion.
func TestIntegration_ReaderCancel(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		ctx, close := context.WithDeadline(ctx, time.Now().Add(time.Second*30))
		defer close()

		bkt := client.Bucket(bucket)
		obj := bkt.Object("reader-cancel-obj")

		minObjectSize := 5000000 // 5 Mb

		w := obj.NewWriter(ctx)
		c := randomContents()
		for written := 0; written < minObjectSize; {
			n, err := w.Write(c)
			if err != nil {
				t.Fatalf("w.Write: %v", err)
			}
			written += n
		}

		if err := w.Close(); err != nil {
			t.Fatalf("writer close: %v", err)
		}
		defer func() {
			if err := obj.Delete(ctx); err != nil {
				log.Printf("failed to delete test object: %v", err)
			}
		}()

		// Create a reader (which makes a GET request to GCS and opens the body to
		// read the object) and then cancel the context before reading.
		readerCtx, cancel := context.WithCancel(ctx)
		r, err := obj.NewReader(readerCtx)
		if err != nil {
			t.Fatalf("obj.NewReader: %v", err)
		}
		defer func() {
			if err := r.Close(); err != nil {
				log.Printf("r.Close(): %v", err)
			}
		}()

		cancel()

		_, err = io.Copy(io.Discard, r)
		if err == nil || !errors.Is(err, context.Canceled) && !(status.Code(err) == codes.Canceled) {
			t.Fatalf("r.Read: got error %v, want context.Canceled", err)
		}
	})
}

// Ensures that a file stored with a:
// * Content-Encoding of "gzip"
// * Content-Type of "text/plain"
// will be properly served back.
// See:
//   - https://cloud.google.com/storage/docs/transcoding#transcoding_and_gzip
//   - https://github.com/googleapis/google-cloud-go/issues/1800
func TestIntegration_NewReaderWithContentEncodingGzip(t *testing.T) {
	multiTransportTest(skipGRPC("gzip transcoding not supported"), t, func(t *testing.T, ctx context.Context, _ string, prefix string, client *Client) {
		h := testHelper{t}

		projectID := testutil.ProjID()
		bkt := client.Bucket(prefix + uidSpace.New())
		h.mustCreate(bkt, projectID, nil)
		defer h.mustDeleteBucket(bkt)
		obj := bkt.Object("decompressive-transcoding")
		original := bytes.Repeat([]byte("a"), 4<<10)

		// Firstly upload the gzip compressed file.
		w := obj.If(Conditions{DoesNotExist: true}).NewWriter(ctx)
		// Compress and upload the content.
		gzw := gzip.NewWriter(w)
		if _, err := gzw.Write(original); err != nil {
			t.Fatalf("Failed to compress content: %v", err)
		}
		if err := gzw.Close(); err != nil {
			t.Errorf("Failed to compress content: %v", err)
		}
		if err := w.Close(); err != nil {
			t.Errorf("Failed to finish uploading the file: %v", err)
		}

		defer h.mustDeleteObject(obj)

		// Now update the Content-Encoding and Content-Type to enable
		// decompressive transcoding.
		updatedAttrs, err := obj.Update(ctx, ObjectAttrsToUpdate{
			ContentEncoding: "gzip",
			ContentType:     "text/plain",
		})
		if err != nil {
			t.Fatalf("Attribute update failure: %v", err)
		}
		if g, w := updatedAttrs.ContentEncoding, "gzip"; g != w {
			t.Fatalf("ContentEncoding mismtach:\nGot:  %q\nWant: %q", g, w)
		}
		if g, w := updatedAttrs.ContentType, "text/plain"; g != w {
			t.Fatalf("ContentType mismtach:\nGot:  %q\nWant: %q", g, w)
		}

		// Test both Read and WriteTo.
		for _, c := range readCases {
			t.Run(c.desc, func(t *testing.T) {
				rWhole, err := obj.NewReader(ctx)
				if err != nil {
					t.Fatalf("Failed to create wholesome reader: %v", err)
				}
				blobWhole, err := c.readFunc(rWhole)
				rWhole.Close()
				if err != nil {
					t.Fatalf("Failed to read the whole body: %v", err)
				}
				if g, w := blobWhole, original; !bytes.Equal(g, w) {
					t.Fatalf("Body mismatch\nGot:\n%s\n\nWant:\n%s", g, w)
				}

				// Now try a range read, which should return the whole body anyways since
				// for decompressive transcoding, range requests ARE IGNORED by Cloud Storage.
				r2kBTo3kB, err := obj.NewRangeReader(ctx, 2<<10, 3<<10)
				if err != nil {
					t.Fatalf("Failed to create range reader: %v", err)
				}
				blob2kBTo3kB, err := c.readFunc(r2kBTo3kB)
				r2kBTo3kB.Close()
				if err != nil {
					t.Fatalf("Failed to read with the 2kB to 3kB range request: %v", err)
				}
				// The ENTIRE body MUST be served back regardless of the requested range.
				if g, w := blob2kBTo3kB, original; !bytes.Equal(g, w) {
					t.Fatalf("Body mismatch\nGot:\n%s\n\nWant:\n%s", g, w)
				}
			})
		}
	})
}

func TestIntegration_HMACKey(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _, _ string, client *Client) {
		client.SetRetry(WithPolicy(RetryAlways))

		projectID := testutil.ProjID()

		// Use the service account email from the user's credentials. Requires that the
		// credentials are set via a JSON credentials file.
		// Note that a service account may only have up to 5 active HMAC keys at once; if
		// we see flakes because of this, we should consider switching to using a project
		// pool.
		credentials := testutil.CredentialsEnv(ctx, "GCLOUD_TESTS_GOLANG_KEY")
		if credentials == nil {
			t.Fatal("credentials could not be determined, is GCLOUD_TESTS_GOLANG_KEY set correctly?")
		}
		if credentials.JSON == nil {
			t.Fatal("could not read the JSON key file, is GCLOUD_TESTS_GOLANG_KEY set correctly?")
		}
		conf, err := google.JWTConfigFromJSON(credentials.JSON)
		if err != nil {
			t.Fatal(err)
		}
		serviceAccountEmail := conf.Email

		hmacKey, err := client.CreateHMACKey(ctx, projectID, serviceAccountEmail)
		if err != nil {
			t.Fatalf("Failed to create HMACKey: %v", err)
		}
		if hmacKey == nil {
			t.Fatal("Unexpectedly got back a nil HMAC key")
		}

		if hmacKey.State != Active {
			t.Fatalf("Unexpected state %q, expected %q", hmacKey.State, Active)
		}

		hkh := client.HMACKeyHandle(projectID, hmacKey.AccessID)
		// 1. Ensure that we CANNOT delete an ACTIVE key.
		if err := hkh.Delete(ctx); err == nil {
			t.Fatal("Unexpectedly deleted key whose state is ACTIVE: No error from Delete.")
		}

		invalidStates := []HMACState{"", Deleted, "active", "inactive", "foo_bar"}
		for _, invalidState := range invalidStates {
			t.Run("invalid-"+string(invalidState), func(t *testing.T) {
				_, err := hkh.Update(ctx, HMACKeyAttrsToUpdate{
					State: invalidState,
				})
				if err == nil {
					t.Fatal("Unexpectedly succeeded")
				}
				invalidStateMsg := fmt.Sprintf(`storage: invalid state %q for update, must be either "ACTIVE" or "INACTIVE"`, invalidState)
				if err.Error() != invalidStateMsg {
					t.Fatalf("Mismatched error: got:  %q\nwant: %q", err, invalidStateMsg)
				}
			})
		}

		// 2.1. Setting the State to Inactive should succeed.
		hu, err := hkh.Update(ctx, HMACKeyAttrsToUpdate{
			State: Inactive,
		})
		if err != nil {
			t.Fatalf("Unexpected Update failure: %v", err)
		}
		if got, want := hu.State, Inactive; got != want {
			t.Fatalf("Unexpected updated state %q, expected %q", got, want)
		}

		// 2.2. Setting the State back to Active should succeed.
		hu, err = hkh.Update(ctx, HMACKeyAttrsToUpdate{
			State: Active,
		})
		if err != nil {
			t.Fatalf("Unexpected Update failure: %v", err)
		}
		if got, want := hu.State, Active; got != want {
			t.Fatalf("Unexpected updated state %q, expected %q", got, want)
		}

		// 3. Verify that keys are listed as expected.
		iter := client.ListHMACKeys(ctx, projectID)
		count := 0
		for ; ; count++ {
			_, err := iter.Next()
			if err == iterator.Done {
				break
			}
			if err != nil {
				t.Fatalf("Failed to ListHMACKeys: %v", err)
			}
		}
		if count == 0 {
			t.Fatal("Failed to list any HMACKeys")
		}

		// 4. Finally set it to back to Inactive and
		// then retry the deletion which should now succeed.
		_, _ = hkh.Update(ctx, HMACKeyAttrsToUpdate{
			State: Inactive,
		})
		if err := hkh.Delete(ctx); err != nil {
			t.Fatalf("Unexpected deletion failure: %v", err)
		}

		_, err = hkh.Get(ctx)
		if err != nil && !strings.Contains(err.Error(), "404") {
			// If the deleted key has already been garbage collected, a 404 is expected.
			// Other errors should cause a failure and are not expected.
			t.Fatalf("Unexpected error: %v", err)
		}
	})
}

func TestIntegration_PostPolicyV4(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {
		jwtConf, err := testutil.JWTConfig()
		if err != nil {
			t.Fatal(err)
		}
		if jwtConf == nil {
			t.Skip("JSON key file is not present")
		}

		projectID := testutil.ProjID()
		newBucketName := prefix + uidSpace.New()
		b := client.Bucket(newBucketName)
		h := testHelper{t}
		h.mustCreate(b, projectID, nil)
		defer h.mustDeleteBucket(b)

		statusCodeToRespond := 200
		opts := &PostPolicyV4Options{
			GoogleAccessID: jwtConf.Email,
			PrivateKey:     jwtConf.PrivateKey,

			Expires: time.Now().Add(30 * time.Minute),

			Fields: &PolicyV4Fields{
				StatusCodeOnSuccess: statusCodeToRespond,
				ContentType:         "text/plain",
				ACL:                 "public-read",
			},

			// The conditions that the uploaded file will be expected to conform to.
			Conditions: []PostPolicyV4Condition{
				// Make the file a maximum of 10mB.
				ConditionContentLengthRange(0, 10<<20),
				ConditionStartsWith("$acl", "public"),
			},
		}

		objectName := uidSpaceObjects.New()
		object := b.Object(objectName)
		defer h.mustDeleteObject(object)

		pv4, err := b.GenerateSignedPostPolicyV4(objectName, opts)
		if err != nil {
			t.Fatal(err)
		}

		if err := verifyPostPolicy(pv4, object, bytes.Repeat([]byte("a"), 25), statusCodeToRespond); err != nil {
			t.Fatal(err)
		}
	})
}

// Verify that custom scopes passed in by the user are applied correctly.
func TestIntegration_Scopes(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "no reads in test")

	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)
		obj := bkt.Object("test-scopes")
		contents := []byte("This object should not be written.\n")

		// A client with ReadOnly scope should be able to read bucket successfully.
		if _, err := bkt.Attrs(ctx); err != nil {
			t.Errorf("client with ScopeReadOnly was not able to read attrs: %v", err)
		}

		// Should not be able to write successfully.
		if err := writeObject(ctx, obj, "text/plain", contents); err == nil {
			if err := obj.Delete(ctx); err != nil {
				t.Logf("obj.Delete: %v", err)
			}
			t.Error("client with ScopeReadOnly was able to write an object unexpectedly.")
		}

		// Should not be able to change permissions.
		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{ACL: []ACLRule{{Entity: "domain-google.com", Role: RoleReader}}}); err == nil {
			t.Error("client with ScopeReadWrite was able to change unexpectedly.")
		}
	}, option.WithScopes(ScopeReadOnly))

	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)
		obj := bkt.Object("test-scopes")
		contents := []byte("This object should be written.\n")

		// A client with ReadWrite scope should be able to read bucket successfully.
		if _, err := bkt.Attrs(ctx); err != nil {
			t.Errorf("client with ScopeReadOnly was not able to read attrs: %v", err)
		}

		// Should be able to write to an object.
		if err := writeObject(ctx, obj, "text/plain", contents); err != nil {
			t.Errorf("client with ScopeReadWrite was not able to write: %v", err)
		}
		defer func() {
			if err := obj.Delete(ctx); err != nil {
				t.Logf("obj.Delete: %v", err)
			}
		}()

		// Should not be able to change permissions.
		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{ACL: []ACLRule{{Entity: "domain-google.com", Role: RoleReader}}}); err == nil {
			t.Error("client with ScopeReadWrite was able to change permissions unexpectedly")
		}
	}, option.WithScopes(ScopeReadWrite))

	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		bkt := client.Bucket(bucket)
		obj := bkt.Object("test-scopes")
		contents := []byte("This object should be written.\n")

		// A client without any scopes should not be able to perform ops.
		if _, err := bkt.Attrs(ctx); err == nil {
			t.Errorf("client with no scopes was able to read attrs unexpectedly")
		}

		if err := writeObject(ctx, obj, "text/plain", contents); err == nil {
			if err := obj.Delete(ctx); err != nil {
				t.Logf("obj.Delete: %v", err)
			}
			t.Error("client with no scopes was able to write an object unexpectedly.")
		}

		if _, err := obj.Update(ctx, ObjectAttrsToUpdate{ACL: []ACLRule{{Entity: "domain-google.com", Role: RoleReader}}}); err == nil {
			t.Error("client with no scopes was able to change permissions unexpectedly")
		}
	}, option.WithScopes(""))
}

func TestIntegration_SignedURL_WithCreds(t *testing.T) {
	// Skip before getting creds if running with -short
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	ctx := context.Background()

	creds, err := findTestCredentials(ctx, "GCLOUD_TESTS_GOLANG_KEY", ScopeFullControl, "https://www.googleapis.com/auth/cloud-platform")
	if err != nil {
		t.Fatalf("unable to find test credentials: %v", err)
	}

	multiTransportTest(skipGRPC("creds capture logic must be implemented for gRPC constructor"), t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		// We can use any client to create the object
		obj := "testBucketSignedURL"
		contents := []byte("test")
		if err := writeObject(ctx, client.Bucket(bucket).Object(obj), "text/plain", contents); err != nil {
			t.Fatalf("writing: %v", err)
		}
		opts := SignedURLOptions{
			Method:  "GET",
			Expires: time.Now().Add(30 * time.Second),
		}
		bkt := client.Bucket(bucket)
		url, err := bkt.SignedURL(obj, &opts)
		if err != nil {
			t.Fatalf("unable to create signed URL: %v", err)
		}

		if err := verifySignedURL(url, nil, contents); err != nil {
			t.Fatalf("problem with the signed URL: %v", err)
		}
	}, option.WithCredentials(creds))
}

func TestIntegration_SignedURL_DefaultSignBytes(t *testing.T) {
	// Skip before getting creds if running with -short
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	ctx := context.Background()

	// Create another client to test the sign byte function as well
	scopes := []string{ScopeFullControl, "https://www.googleapis.com/auth/cloud-platform"}
	ts := testutil.TokenSource(ctx, scopes...)
	if ts == nil {
		t.Fatalf("Cannot get token source to create client")
	}

	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, client *Client) {
		jwt, err := testutil.JWTConfig()
		if err != nil {
			t.Fatalf("unable to find test credentials: %v", err)
		}

		obj := "testBucketSignedURL"
		contents := []byte("test")
		if err := writeObject(ctx, client.Bucket(bucket).Object(obj), "text/plain", contents); err != nil {
			t.Fatalf("writing: %v", err)
		}

		opts := SignedURLOptions{
			Method:         "GET",
			Expires:        time.Now().Add(30 * time.Second),
			GoogleAccessID: jwt.Email,
		}
		bkt := client.Bucket(bucket)
		url, err := bkt.SignedURL(obj, &opts)
		if err != nil {
			t.Fatalf("unable to create signed URL: %v", err)
		}

		if err := verifySignedURL(url, nil, contents); err != nil {
			t.Fatalf("problem with the signed URL: %v", err)
		}
	}, option.WithTokenSource(ts))

}

func TestIntegration_PostPolicyV4_WithCreds(t *testing.T) {
	// Skip before getting creds if running with -short
	if testing.Short() {
		t.Skip("Integration tests skipped in short mode")
	}

	// By default we are authed with a token source, so don't have the context to
	// read some of the fields from the keyfile.
	// Here we explictly send the key to the client.
	creds, err := findTestCredentials(context.Background(), "GCLOUD_TESTS_GOLANG_KEY", ScopeFullControl, "https://www.googleapis.com/auth/cloud-platform")
	if err != nil {
		t.Fatalf("unable to find test credentials: %v", err)
	}

	ctx := skipJSONReads(skipGRPC("creds capture logic must be implemented for gRPC constructor"), "test is not testing the read behaviour")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, clientWithCredentials *Client) {
		h := testHelper{t}

		statusCodeToRespond := 200

		for _, test := range []struct {
			desc   string
			opts   PostPolicyV4Options
			client *Client
		}{
			{
				desc: "signing with the private key",
				opts: PostPolicyV4Options{
					Expires: time.Now().Add(30 * time.Minute),

					Fields: &PolicyV4Fields{
						StatusCodeOnSuccess: statusCodeToRespond,
						ContentType:         "text/plain",
						ACL:                 "public-read",
					},
				},
				client: clientWithCredentials,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				objectName := uidSpace.New()
				object := test.client.Bucket(bucket).Object(objectName)
				defer h.mustDeleteObject(object)

				pv4, err := test.client.Bucket(bucket).GenerateSignedPostPolicyV4(objectName, &test.opts)
				if err != nil {
					t.Fatal(err)
				}

				if err := verifyPostPolicy(pv4, object, bytes.Repeat([]byte("a"), 25), statusCodeToRespond); err != nil {
					t.Fatal(err)
				}
			})
		}
	}, option.WithCredentials(creds))

}

func TestIntegration_PostPolicyV4_BucketDefault(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "test is not testing the read behaviour")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, bucket, _ string, clientWithoutPrivateKey *Client) {
		h := testHelper{t}

		jwt, err := testutil.JWTConfig()
		if err != nil {
			t.Fatalf("unable to find test credentials: %v", err)
		}

		statusCodeToRespond := 200

		for _, test := range []struct {
			desc   string
			opts   PostPolicyV4Options
			client *Client
		}{
			{
				desc: "signing with the default sign bytes func",
				opts: PostPolicyV4Options{
					Expires:        time.Now().Add(30 * time.Minute),
					GoogleAccessID: jwt.Email,
					Fields: &PolicyV4Fields{
						StatusCodeOnSuccess: statusCodeToRespond,
						ContentType:         "text/plain",
						ACL:                 "public-read",
					},
				},
				client: clientWithoutPrivateKey,
			},
		} {
			t.Run(test.desc, func(t *testing.T) {
				objectName := uidSpaceObjects.New()
				object := test.client.Bucket(bucket).Object(objectName)
				defer h.mustDeleteObject(object)

				pv4, err := test.client.Bucket(bucket).GenerateSignedPostPolicyV4(object.ObjectName(), &test.opts)
				if err != nil {
					t.Fatal(err)
				}

				if err := verifyPostPolicy(pv4, object, bytes.Repeat([]byte("a"), 25), statusCodeToRespond); err != nil {
					t.Fatal(err)
				}
			})
		}
	})

}

// Tests that the same SignBytes function works for both
// SignRawBytes on GeneratePostPolicyV4 and SignBytes on SignedURL
func TestIntegration_PostPolicyV4_SignedURL_WithSignBytes(t *testing.T) {
	ctx := skipJSONReads(context.Background(), "test is not testing the read behaviour")
	multiTransportTest(ctx, t, func(t *testing.T, ctx context.Context, _, prefix string, client *Client) {

		h := testHelper{t}
		projectID := testutil.ProjID()
		bucketName := prefix + uidSpace.New()
		objectName := uidSpaceObjects.New()
		fileBody := bytes.Repeat([]byte("b"), 25)
		bucket := client.Bucket(bucketName)

		h.mustCreate(bucket, projectID, nil)
		defer h.mustDeleteBucket(bucket)

		object := bucket.Object(objectName)
		defer h.mustDeleteObject(object)

		jwtConf, err := testutil.JWTConfig()
		if err != nil {
			t.Fatal(err)
		}
		if jwtConf == nil {
			t.Skip("JSON key file is not present")
		}

		signingFunc := func(b []byte) ([]byte, error) {
			parsedRSAPrivKey, err := parseKey(jwtConf.PrivateKey)
			if err != nil {
				return nil, err
			}
			sum := sha256.Sum256(b)
			return rsa.SignPKCS1v15(cryptorand.Reader, parsedRSAPrivKey, crypto.SHA256, sum[:])
		}

		// Test Post Policy
		successStatusCode := 200
		ppv4Opts := &PostPolicyV4Options{
			GoogleAccessID: jwtConf.Email,
			SignRawBytes:   signingFunc,
			Expires:        time.Now().Add(30 * time.Minute),
			Fields: &PolicyV4Fields{
				StatusCodeOnSuccess: successStatusCode,
				ContentType:         "text/plain",
				ACL:                 "public-read",
			},
		}

		pv4, err := GenerateSignedPostPolicyV4(bucketName, objectName, ppv4Opts)
		if err != nil {
			t.Fatal(err)
		}

		if err := verifyPostPolicy(pv4, object, fileBody, successStatusCode); err != nil {
			t.Fatal(err)
		}

		// Test Signed URL
		signURLOpts := &SignedURLOptions{
			GoogleAccessID: jwtConf.Email,
			SignBytes:      signingFunc,
			Method:         "GET",
			Expires:        time.Now().Add(30 * time.Second),
		}

		url, err := bucket.SignedURL(objectName, signURLOpts)
		if err != nil {
			t.Fatalf("unable to create signed URL: %v", err)
		}

		if err := verifySignedURL(url, nil, fileBody); err != nil {
			t.Fatal(err)
		}
	})
}

func TestIntegration_OCTracing(t *testing.T) {
	multiTransportTest(context.Background(), t, func(t *testing.T, ctx context.Context, bucket string, _ string, client *Client) {
		te := testutil.NewTestExporter()
		defer te.Unregister()

		bkt := client.Bucket(bucket)
		bkt.Attrs(ctx)

		if len(te.Spans()) == 0 {
			t.Fatalf("Expected some spans to be created, but got %d", 0)
		}
	})
}

// verifySignedURL gets the bytes at the provided url and verifies them against the
// expectedFileBody. Make sure the SignedURLOptions set the method as "GET".
func verifySignedURL(url string, headers map[string][]string, expectedFileBody []byte) error {
	got, err := getURL(url, headers)
	if err != nil {
		return fmt.Errorf("getURL %q: %v", url, err)
	}
	if !bytes.Equal(got, expectedFileBody) {
		return fmt.Errorf("got %q, want %q", got, expectedFileBody)
	}
	return nil
}

// verifyPostPolicy uploads a file to the obj using the provided post policy and
// verifies that it was uploaded correctly
func verifyPostPolicy(pv4 *PostPolicyV4, obj *ObjectHandle, bytesToWrite []byte, statusCodeOnSuccess int) error {
	ctx := context.Background()
	var res *http.Response

	// Request is sent using a vanilla net/http client, so there are no built-in
	// retries. We must wrap with a retry to prevent flakes.
	return retry(ctx,
		func() error {
			formBuf := new(bytes.Buffer)
			mw := multipart.NewWriter(formBuf)
			for fieldName, value := range pv4.Fields {
				if err := mw.WriteField(fieldName, value); err != nil {
					return fmt.Errorf("Failed to write form field: %q: %v", fieldName, err)
				}
			}

			// Now let's perform the upload
			mf, err := mw.CreateFormFile("file", "myfile.txt")
			if err != nil {
				return err
			}
			if _, err := mf.Write(bytesToWrite); err != nil {
				return err
			}
			if err := mw.Close(); err != nil {
				return err
			}

			// Compose the HTTP request
			req, err := http.NewRequest("POST", pv4.URL, formBuf)
			if err != nil {
				return fmt.Errorf("Failed to compose HTTP request: %v", err)
			}

			// Ensure the Content-Type is derived from the writer
			req.Header.Set("Content-Type", mw.FormDataContentType())

			// Send request
			res, err = http.DefaultClient.Do(req)
			if err != nil {
				return err
			}
			return nil
		},
		func() error {
			// Check response
			if g, w := res.StatusCode, statusCodeOnSuccess; g != w {
				blob, _ := httputil.DumpResponse(res, true)
				return fmt.Errorf("Status code in response mismatch: got %d want %d\nBody: %s", g, w, blob)
			}
			io.Copy(io.Discard, res.Body)

			// Verify that the file was properly uploaded
			// by reading back its attributes and content
			attrs, err := obj.Attrs(ctx)
			if err != nil {
				return fmt.Errorf("Failed to retrieve attributes: %v", err)
			}
			if g, w := attrs.Size, int64(len(bytesToWrite)); g != w {
				return fmt.Errorf("ContentLength mismatch: got %d want %d", g, w)
			}
			if g, w := attrs.MD5, md5.Sum(bytesToWrite); !bytes.Equal(g, w[:]) {
				return fmt.Errorf("MD5Checksum mismatch\nGot:  %x\nWant: %x", g, w)
			}

			// Compare the uploaded body with the expected
			rd, err := obj.NewReader(ctx)
			if err != nil {
				return fmt.Errorf("Failed to create a reader: %v", err)
			}
			gotBody, err := io.ReadAll(rd)
			if err != nil {
				return fmt.Errorf("Failed to read the body: %v", err)
			}
			if diff := testutil.Diff(string(gotBody), string(bytesToWrite)); diff != "" {
				return fmt.Errorf("Body mismatch: got - want +\n%s", diff)
			}
			return nil
		})
}

func findTestCredentials(ctx context.Context, envVar string, scopes ...string) (*google.Credentials, error) {
	key := os.Getenv(envVar)
	var opts []option.ClientOption
	if len(scopes) > 0 {
		opts = append(opts, option.WithScopes(scopes...))
	}
	if key != "" {
		opts = append(opts, option.WithCredentialsFile(key))
	}
	return transport.Creds(ctx, opts...)
}

type testHelper struct {
	t *testing.T
}

func (h testHelper) mustCreate(b *BucketHandle, projID string, attrs *BucketAttrs) {
	h.t.Helper()
	if err := b.Create(context.Background(), projID, attrs); err != nil {
		h.t.Fatalf("BucketHandle(%q).Create: %v", b.BucketName(), err)
	}
}

func (h testHelper) mustDeleteBucket(b *BucketHandle) {
	h.t.Helper()
	if err := b.Delete(context.Background()); err != nil {
		h.t.Fatalf("BucketHandle(%q).Delete: %v", b.BucketName(), err)
	}
}

func (h testHelper) mustBucketAttrs(b *BucketHandle) *BucketAttrs {
	h.t.Helper()
	attrs, err := b.Attrs(context.Background())
	if err != nil {
		h.t.Fatalf("BucketHandle(%q).Attrs: %v", b.BucketName(), err)
	}
	return attrs
}

// updating a bucket is conditionally idempotent on metageneration, so we pass that in to enable retries
func (h testHelper) mustUpdateBucket(b *BucketHandle, ua BucketAttrsToUpdate, metageneration int64) *BucketAttrs {
	h.t.Helper()
	attrs, err := b.If(BucketConditions{MetagenerationMatch: metageneration}).Update(context.Background(), ua)
	if err != nil {
		var apiErr *apierror.APIError
		if ok := errors.As(err, &apiErr); ok {
			// Update may already have succeeded with retry; if so, log and grab attrs.
			if apiErr.HTTPCode() == http.StatusPreconditionFailed || apiErr.GRPCStatus().Code() == codes.FailedPrecondition {
				log.Println("bucket update failed due to precondition; likely succeeded but retried - grabbing attrs instead")
				return h.mustBucketAttrs(b)
			}
		}
		h.t.Fatalf("BucketHandle(%q).Update: %v", b.BucketName(), err)
	}
	return attrs
}

func (h testHelper) mustObjectAttrs(o *ObjectHandle) *ObjectAttrs {
	h.t.Helper()
	attrs, err := o.Attrs(context.Background())
	if err != nil {
		h.t.Fatalf("get object %q from bucket %q: %v", o.ObjectName(), o.BucketName(), err)
	}
	return attrs
}

func (h testHelper) mustDeleteObject(o *ObjectHandle) {
	h.t.Helper()
	if err := o.Retryer(WithPolicy(RetryAlways)).Delete(context.Background()); err != nil {
		var apiErr *apierror.APIError
		if ok := errors.As(err, &apiErr); ok {
			// Object may already be deleted with retry; if so skip.
			if apiErr.HTTPCode() == 404 || apiErr.GRPCStatus().Code() == codes.NotFound {
				return
			}
		}
		h.t.Fatalf("delete object %q from bucket %q: %v", o.ObjectName(), o.BucketName(), err)
	}
}

// updating an object is conditionally idempotent on metageneration, so we pass that in to enable retries
func (h testHelper) mustUpdateObject(o *ObjectHandle, ua ObjectAttrsToUpdate, metageneration int64) *ObjectAttrs {
	h.t.Helper()
	attrs, err := o.If(Conditions{MetagenerationMatch: metageneration}).Update(context.Background(), ua)
	if err != nil {
		h.t.Fatalf("update object %q from bucket %q: %v", o.ObjectName(), o.BucketName(), err)
	}
	return attrs
}

func (h testHelper) mustWrite(w *Writer, data []byte) {
	h.t.Helper()
	if _, err := w.Write(data); err != nil {
		w.Close()
		h.t.Fatalf("write object %q to bucket %q: %v", w.o.ObjectName(), w.o.BucketName(), err)
	}
	if err := w.Close(); err != nil {
		h.t.Fatalf("close write object %q to bucket %q: %v", w.o.ObjectName(), w.o.BucketName(), err)
	}
}

func (h testHelper) mustRead(obj *ObjectHandle) []byte {
	h.t.Helper()
	data, err := readObject(context.Background(), obj)
	if err != nil {
		h.t.Fatalf("read: %v", err)
	}
	return data
}

// deleteObjectIfExists deletes an object with a RetryAlways policy (unless another
// policy is supplied in the options). It will not return an error if the object
// is already deleted/doesn't exist. It will time out after 15 seconds.
func deleteObjectIfExists(o *ObjectHandle, retryOpts ...RetryOption) error {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second*15)
	defer cancel()
	retryOpts = append([]RetryOption{WithPolicy(RetryAlways)}, retryOpts...)

	if err := o.Retryer(retryOpts...).Delete(ctx); err != nil {
		var apiErr *apierror.APIError
		if ok := errors.As(err, &apiErr); ok {
			// Object may already be deleted with retry; if so, return no error.
			if apiErr.HTTPCode() == 404 || apiErr.GRPCStatus().Code() == codes.NotFound {
				return nil
			}
		}
		return fmt.Errorf("delete object %q from bucket %q: %v", o.ObjectName(), o.BucketName(), err)
	}
	return nil
}

func writeContents(w *Writer, contents []byte) error {
	if contents != nil {
		if _, err := w.Write(contents); err != nil {
			_ = w.Close()
			return err
		}
	}
	return w.Close()
}

func writeObject(ctx context.Context, obj *ObjectHandle, contentType string, contents []byte) error {
	w := newWriter(ctx, obj, contentType, false)

	return writeContents(w, contents)
}

func newWriter(ctx context.Context, obj *ObjectHandle, contentType string, forceEmptyContentType bool) *Writer {
	w := obj.Retryer(WithPolicy(RetryAlways)).NewWriter(ctx)
	w.ContentType = contentType
	w.ForceEmptyContentType = forceEmptyContentType

	return w
}

func readObject(ctx context.Context, obj *ObjectHandle) ([]byte, error) {
	r, err := obj.NewReader(ctx)
	if err != nil {
		return nil, err
	}
	defer r.Close()
	return io.ReadAll(r)
}

// cleanupBuckets deletes the bucket used for testing, as well as old
// testing buckets that weren't cleaned previously.
func cleanupBuckets() error {
	if testing.Short() {
		return nil // Don't clean up in short mode.
	}
	ctx := context.Background()
	client, err := newTestClient(ctx)
	if err != nil {
		log.Fatalf("NewClient: %v", err)
	}
	if client == nil {
		return nil // Don't cleanup if we're not configured correctly.
	}
	defer client.Close()
	if err := killBucket(ctx, client, bucketName); err != nil {
		return err
	}
	if err := killBucket(ctx, client, grpcBucketName); err != nil {
		return err
	}

	// Delete buckets whose name begins with our test prefix, and which were
	// created a while ago. (Unfortunately GCS doesn't provide last-modified
	// time, which would be a better way to check for staleness.)
	if err := deleteExpiredBuckets(ctx, client, testPrefix); err != nil {
		return err
	}
	return deleteExpiredBuckets(ctx, client, grpcTestPrefix)
}

func deleteExpiredBuckets(ctx context.Context, client *Client, prefix string) error {
	const expireAge = 24 * time.Hour
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
		if time.Since(bktAttrs.Created) > expireAge {
			log.Printf("deleting bucket %q, which is more than %s old", bktAttrs.Name, expireAge)
			if err := killBucket(ctx, client, bktAttrs.Name); err != nil {
				return err
			}
		}
	}
	return nil
}

// killBucket deletes a bucket and all its objects.
func killBucket(ctx context.Context, client *Client, bucketName string) error {
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
		// Objects with a hold must have the hold released.
		if objAttrs.EventBasedHold || objAttrs.TemporaryHold {
			obj := bkt.Object(objAttrs.Name)
			if _, err := obj.Update(ctx, ObjectAttrsToUpdate{EventBasedHold: false, TemporaryHold: false}); err != nil {
				return fmt.Errorf("removing hold from %q: %v", bucketName+"/"+objAttrs.Name, err)
			}
		}
		if err := bkt.Object(objAttrs.Name).Delete(ctx); err != nil {
			return fmt.Errorf("deleting %q: %v", bucketName+"/"+objAttrs.Name, err)
		}
	}
	// Delete any managed folders.
	req := &controlpb.ListManagedFoldersRequest{
		Parent: fmt.Sprintf("projects/_/buckets/%s", bucketName),
	}
	mfIt := controlClient.ListManagedFolders(ctx, req)
	for {
		resp, err := mfIt.Next()
		if err == iterator.Done {
			break
		}
		// Buckets without UBLA will return this error for MF ops; skip.
		if status.Code(err) == codes.FailedPrecondition {
			break
		}
		if err != nil {
			return err
		}
		deleteReq := &controlpb.DeleteManagedFolderRequest{
			Name:          resp.Name,
			AllowNonEmpty: true,
		}
		err = controlClient.DeleteManagedFolder(ctx, deleteReq)
		if err != nil {
			return err
		}
	}

	// Delete any folders.
	listFoldersReq := &controlpb.ListFoldersRequest{
		Parent: fmt.Sprintf("projects/_/buckets/%s", bucketName),
	}
	folderIt := controlClient.ListFolders(ctx, listFoldersReq)
	for {
		resp, err := folderIt.Next()
		if err == iterator.Done {
			break
		}
		// Buckets without UBLA will return this error for Folder ops; skip.
		if status.Code(err) == codes.FailedPrecondition {
			break
		}
		if err != nil {
			return err
		}
		deleteFolderReq := &controlpb.DeleteFolderRequest{
			Name: resp.Name,
		}
		err = controlClient.DeleteFolder(ctx, deleteFolderReq)
		if err != nil {
			return err
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

func randomContents() []byte {
	h := md5.New()
	io.WriteString(h, fmt.Sprintf("hello world%d", rng.Intn(100000)))
	return h.Sum(nil)
}

type zeros struct{}

func (zeros) Read(p []byte) (int, error) { return len(p), nil }

// Make a GET request to a URL using an unauthenticated client, and return its contents.
func getURL(url string, headers map[string][]string) ([]byte, error) {
	req, err := http.NewRequest("GET", url, nil)
	if err != nil {
		return nil, err
	}
	req.Header = headers
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()
	bytes, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}
	if res.StatusCode != 200 {
		return nil, fmt.Errorf("code=%d, body=%s", res.StatusCode, string(bytes))
	}
	return bytes, nil
}

// Make a PUT request to a URL using an unauthenticated client, and return its contents.
func putURL(url string, headers map[string][]string, payload io.Reader) ([]byte, error) {
	req, err := http.NewRequest("PUT", url, payload)
	if err != nil {
		return nil, err
	}
	req.Header = headers
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return nil, err
	}
	defer res.Body.Close()
	bytes, err := io.ReadAll(res.Body)
	if err != nil {
		return nil, err
	}
	if res.StatusCode != 200 {
		return nil, fmt.Errorf("code=%d, body=%s", res.StatusCode, string(bytes))
	}
	return bytes, nil
}

func keyFileEmail(filename string) (string, error) {
	bytes, err := os.ReadFile(filename)
	if err != nil {
		return "", err
	}
	var v struct {
		ClientEmail string `json:"client_email"`
	}
	if err := json.Unmarshal(bytes, &v); err != nil {
		return "", err
	}
	return v.ClientEmail, nil
}

type comparableACL interface {
	equals(ACLRule) bool
}

type testACLRule ACLRule

func (acl testACLRule) equals(a ACLRule) bool {
	return cmp.Equal(a, ACLRule(acl))
}

type entityRoleACL struct {
	entity ACLEntity
	role   ACLRole
}

func (er entityRoleACL) equals(a ACLRule) bool {
	return a.Entity == er.entity && a.Role == er.role
}

type prefixRoleACL struct {
	prefix string
	role   ACLRole
}

func (pr prefixRoleACL) equals(a ACLRule) bool {
	return strings.HasPrefix(string(a.Entity), pr.prefix) && a.Role == pr.role
}

func containsACLRule(acl []ACLRule, want comparableACL) bool {
	for _, acl := range acl {
		if want.equals(acl) {
			return true
		}
	}
	return false
}

// retry retries a function call as well as an (optional) correctness check for up
// to 60 seconds. Both call and check must run without error in order to succeed.
// If the timeout is hit, the most recent error from call or check will be returned.
// This function should be used to wrap calls that might cause integration test
// flakes due to delays in propagation (for example, metadata updates).
func retry(ctx context.Context, call func() error, check func() error) error {
	timeout := time.After(60 * time.Second)
	var err error
	for {
		select {
		case <-timeout:
			return err
		default:
		}
		err = call()
		if err == nil {
			if check == nil || check() == nil {
				return nil
			}
			err = check()
		}
		time.Sleep(200 * time.Millisecond)
	}
}

func retryOnNilAndTransientErrs(err error) bool {
	return err == nil || ShouldRetry(err)
}
func retryOnTransient400and403(err error) bool {
	var e *googleapi.Error
	var ae *apierror.APIError
	return ShouldRetry(err) ||
		/* http */ errors.As(err, &e) && (e.Code == 400 || e.Code == 403) ||
		/* grpc */ errors.As(err, &ae) && (ae.GRPCStatus().Code() == codes.InvalidArgument || ae.GRPCStatus().Code() == codes.PermissionDenied)
}

func skipGRPC(reason string) context.Context {
	return context.WithValue(context.Background(), skipTransportTestKey("grpc"), reason)
}

func skipHTTP(reason string) context.Context {
	ctx := context.WithValue(context.Background(), skipTransportTestKey("http"), reason)
	return context.WithValue(ctx, skipTransportTestKey("jsonReads"), reason)
}

func skipJSONReads(ctx context.Context, reason string) context.Context {
	return context.WithValue(ctx, skipTransportTestKey("jsonReads"), reason)
}

func skipXMLReads(ctx context.Context, reason string) context.Context {
	return context.WithValue(ctx, skipTransportTestKey("http"), reason)
}

// Extract the error code if it's a googleapi.Error
func extractErrCode(err error) int {
	if err == nil {
		return 0
	}
	var e *googleapi.Error
	if errors.As(err, &e) {
		return e.Code
	}

	return -1
}

func setUpRequesterPaysBucket(ctx context.Context, t *testing.T, bucket, object string, addOwnerEmail string) {
	t.Helper()
	client := testConfig(ctx, t)
	h := testHelper{t}

	requesterPaysBucket := client.Bucket(bucket)

	// Create a requester-pays bucket.
	h.mustCreate(requesterPaysBucket, testutil.ProjID(), &BucketAttrs{RequesterPays: true})
	t.Cleanup(func() { h.mustDeleteBucket(requesterPaysBucket) })

	// Grant ownership
	if err := requesterPaysBucket.ACL().Set(ctx, ACLEntity("user-"+addOwnerEmail), RoleOwner); err != nil {
		t.Fatalf("set ACL: %v", err)
	}

	h.mustWrite(requesterPaysBucket.Object(object).NewWriter(ctx), []byte("hello"))
	t.Cleanup(func() {
		err := requesterPaysBucket.Object(object).Delete(ctx)
		if err != nil {
			// only log because object may be deleted by test
			t.Logf("could not delete object: %v", err)
		}
	})
}
