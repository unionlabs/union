// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package downscope

import (
	"context"
	"flag"
	"fmt"
	"io"
	"log"
	"os"
	"testing"
	"time"

	"google.golang.org/api/option"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google"
	"golang.org/x/oauth2/google/downscope"
	storage "google.golang.org/api/storage/v1"
	"google.golang.org/api/transport"
)

const (
	rootTokenScope        = "https://www.googleapis.com/auth/cloud-platform"
	envServiceAccountFile = "GCLOUD_TESTS_GOLANG_KEY"
	object1               = "cab-first-c45wknuy.txt"
	object2               = "cab-second-c45wknuy.txt"
	bucket                = "dulcet-port-762"
)

var (
	rootCredential *google.Credentials
)

// TestMain contains all of the setup code that needs to be run once before any of the tests are run
func TestMain(m *testing.M) {
	flag.Parse()
	if testing.Short() {
		// This line runs all of our individual tests
		os.Exit(m.Run())
	}
	ctx := context.Background()
	credentialFileName := os.Getenv(envServiceAccountFile)

	var err error
	rootCredential, err = transport.Creds(ctx, option.WithCredentialsFile(credentialFileName), option.WithScopes(rootTokenScope))

	if err != nil {
		log.Fatalf("failed to construct root credential: %v", err)
	}

	// This line runs all of our individual tests
	os.Exit(m.Run())

}

// downscopeTest holds the parameters necessary for running a test of the token downscoping capabilities implemented in `oauth2/google/downscope`
type downscopeTest struct {
	name                 string
	availableResource    string
	availablePermissions []string
	condition            downscope.AvailabilityCondition
	objectName           string
	rootSource           oauth2.TokenSource
	expectError          bool
}

func TestDownscopedToken(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}

	var downscopeTests = []downscopeTest{
		{
			name:                 "successfulDownscopedRead",
			availableResource:    "//storage.googleapis.com/projects/_/buckets/" + bucket,
			availablePermissions: []string{"inRole:roles/storage.objectViewer"},
			condition: downscope.AvailabilityCondition{
				Expression: "resource.name.startsWith('projects/_/buckets/" + bucket + "/objects/" + object1 + "')",
			},
			rootSource:  rootCredential.TokenSource,
			objectName:  object1,
			expectError: false,
		},
		{
			name:                 "readWithoutPermission",
			availableResource:    "//storage.googleapis.com/projects/_/buckets/" + bucket,
			availablePermissions: []string{"inRole:roles/storage.objectViewer"},
			condition: downscope.AvailabilityCondition{
				Expression: "resource.name.startsWith('projects/_/buckets/" + bucket + "/objects/" + object1 + "')",
			},
			rootSource:  rootCredential.TokenSource,
			objectName:  object2,
			expectError: true,
		},
	}

	for _, tt := range downscopeTests {
		t.Run(tt.name, func(t *testing.T) {
			err := downscopeQuery(t, tt)
			// If a test isn't supposed to fail, it shouldn't fail.
			if !tt.expectError && err != nil {
				t.Errorf("test case %v should have succeeded, but instead returned %v", tt.name, err)
			} else if tt.expectError && err == nil { // If a test is supposed to fail, it should return a non-nil error.
				t.Errorf(" test case %v should have returned an error, but instead returned nil", tt.name)
			}
		})
	}
}

// I'm not sure what I should name this according to convention.
func downscopeQuery(t *testing.T, tt downscopeTest) error {
	t.Helper()
	ctx := context.Background()

	// Initializes an accessBoundary
	var AccessBoundaryRules []downscope.AccessBoundaryRule
	AccessBoundaryRules = append(AccessBoundaryRules, downscope.AccessBoundaryRule{AvailableResource: tt.availableResource, AvailablePermissions: tt.availablePermissions, Condition: &tt.condition})

	downscopedTokenSource, err := downscope.NewTokenSource(context.Background(), downscope.DownscopingConfig{RootSource: tt.rootSource, Rules: AccessBoundaryRules})
	if err != nil {
		return fmt.Errorf("failed to create the initial token source: %v", err)
	}
	downscopedTokenSource = oauth2.ReuseTokenSource(nil, downscopedTokenSource)

	ctx, cancel := context.WithTimeout(ctx, time.Second*30)
	defer cancel()
	storageService, err := storage.NewService(ctx, option.WithTokenSource(downscopedTokenSource))
	if err != nil {
		return fmt.Errorf("failed to create the storage service: %v", err)
	}
	resp, err := storageService.Objects.Get(bucket, tt.objectName).Download()
	if err != nil {
		return fmt.Errorf("failed to retrieve object from GCP project with error: %v", err)
	}
	defer resp.Body.Close()
	_, err = io.ReadAll(resp.Body)
	if err != nil {
		return fmt.Errorf("io.ReadAll: %v", err)
	}
	return nil
}
