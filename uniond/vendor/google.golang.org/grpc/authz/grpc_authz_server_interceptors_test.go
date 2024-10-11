/*
 *
 * Copyright 2021 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package authz_test

import (
	"fmt"
	"os"
	"path"
	"testing"
	"time"

	"google.golang.org/grpc/authz"
)

func createTmpPolicyFile(t *testing.T, dirSuffix string, policy []byte) string {
	t.Helper()

	// Create a temp directory. Passing an empty string for the first argument
	// uses the system temp directory.
	dir, err := os.MkdirTemp("", dirSuffix)
	if err != nil {
		t.Fatalf("os.MkdirTemp() failed: %v", err)
	}
	t.Logf("Using tmpdir: %s", dir)
	// Write policy into file.
	filename := path.Join(dir, "policy.json")
	if err := os.WriteFile(filename, policy, os.ModePerm); err != nil {
		t.Fatalf("os.WriteFile(%q) failed: %v", filename, err)
	}
	t.Logf("Wrote policy %s to file at %s", string(policy), filename)
	return filename
}

func (s) TestNewStatic(t *testing.T) {
	tests := map[string]struct {
		authzPolicy string
		wantErr     error
	}{
		"InvalidPolicyFailsToCreateInterceptor": {
			authzPolicy: `{}`,
			wantErr:     fmt.Errorf(`"name" is not present`),
		},
		"ValidPolicyCreatesInterceptor": {
			authzPolicy: `{		
				"name": "authz",
				"allow_rules": 
				[
					{
						"name": "allow_all"
					}
				]
			}`,
		},
	}
	for name, test := range tests {
		t.Run(name, func(t *testing.T) {
			if _, err := authz.NewStatic(test.authzPolicy); fmt.Sprint(err) != fmt.Sprint(test.wantErr) {
				t.Fatalf("NewStatic(%v) returned err: %v, want err: %v", test.authzPolicy, err, test.wantErr)
			}
		})
	}
}

func (s) TestNewFileWatcher(t *testing.T) {
	tests := map[string]struct {
		authzPolicy     string
		refreshDuration time.Duration
		wantErr         error
	}{
		"InvalidRefreshDurationFailsToCreateInterceptor": {
			refreshDuration: time.Duration(0),
			wantErr:         fmt.Errorf("requires refresh interval(0s) greater than 0s"),
		},
		"InvalidPolicyFailsToCreateInterceptor": {
			authzPolicy:     `{}`,
			refreshDuration: time.Duration(1),
			wantErr:         fmt.Errorf(`"name" is not present`),
		},
		"ValidPolicyCreatesInterceptor": {
			authzPolicy: `{
				"name": "authz",
				"allow_rules":
				[
					{
						"name": "allow_all"
					}
				]
			}`,
			refreshDuration: time.Duration(1),
		},
	}
	for name, test := range tests {
		t.Run(name, func(t *testing.T) {
			file := createTmpPolicyFile(t, name, []byte(test.authzPolicy))
			i, err := authz.NewFileWatcher(file, test.refreshDuration)
			if fmt.Sprint(err) != fmt.Sprint(test.wantErr) {
				t.Fatalf("NewFileWatcher(%v) returned err: %v, want err: %v", test.authzPolicy, err, test.wantErr)
			}
			if i != nil {
				i.Close()
			}
		})
	}
}
