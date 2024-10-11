// Copyright 2023 Google LLC
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

package downscope_test

import (
	"context"
	"fmt"
	"io"
	"os"
	"testing"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/credentials/downscope"
	"cloud.google.com/go/auth/internal/credsfile"
	"cloud.google.com/go/auth/internal/testutil"
	"cloud.google.com/go/auth/internal/testutil/testgcs"
)

const (
	rootTokenScope = "https://www.googleapis.com/auth/cloud-platform"
	object1        = "cab-first-c45wknuy.txt"
	object2        = "cab-second-c45wknuy.txt"
	bucket         = "dulcet-port-762"
)

func TestDownscopedToken(t *testing.T) {
	testutil.IntegrationTestCheck(t)
	creds, err := credentials.DetectDefault(&credentials.DetectOptions{
		CredentialsFile: os.Getenv(credsfile.GoogleAppCredsEnvVar),
		Scopes:          []string{rootTokenScope},
	})
	if err != nil {
		t.Fatalf("DefaultCredentials() = %v", err)
	}

	var downscopeTests = []struct {
		name        string
		rule        downscope.AccessBoundaryRule
		objectName  string
		expectError bool
	}{
		{
			name: "successfulDownscopedRead",
			rule: downscope.AccessBoundaryRule{
				AvailableResource:    "//storage.googleapis.com/projects/_/buckets/" + bucket,
				AvailablePermissions: []string{"inRole:roles/storage.objectViewer"},
				Condition: &downscope.AvailabilityCondition{
					Expression: "resource.name.startsWith('projects/_/buckets/" + bucket + "/objects/" + object1 + "')",
				},
			},
			objectName:  object1,
			expectError: false,
		},
		{
			name: "readWithoutPermission",
			rule: downscope.AccessBoundaryRule{
				AvailableResource:    "//storage.googleapis.com/projects/_/buckets/" + bucket,
				AvailablePermissions: []string{"inRole:roles/storage.objectViewer"},
				Condition: &downscope.AvailabilityCondition{
					Expression: "resource.name.startsWith('projects/_/buckets/" + bucket + "/objects/" + object1 + "')",
				},
			},
			objectName:  object2,
			expectError: true,
		},
	}

	for _, tt := range downscopeTests {
		t.Run(tt.name, func(t *testing.T) {
			err := testDownscopedToken(t, tt.rule, tt.objectName, creds)
			if !tt.expectError && err != nil {
				t.Errorf("test case %v should have succeeded, but instead returned %v", tt.name, err)
			} else if tt.expectError && err == nil {
				t.Errorf(" test case %v should have returned an error, but instead returned nil", tt.name)
			}
		})
	}
}

func testDownscopedToken(t *testing.T, rule downscope.AccessBoundaryRule, objectName string, creds *auth.Credentials) error {
	t.Helper()
	ctx := context.Background()
	creds, err := downscope.NewCredentials(&downscope.Options{Credentials: creds, Rules: []downscope.AccessBoundaryRule{rule}})
	if err != nil {
		return fmt.Errorf("downscope.NewCredentials() = %v", err)
	}

	ctx, cancel := context.WithTimeout(ctx, time.Second*30)
	defer cancel()
	client := testgcs.NewClient(creds)
	resp, err := client.DownloadObject(ctx, bucket, objectName)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	_, err = io.ReadAll(resp.Body)
	if err != nil {
		return err
	}
	return nil
}
