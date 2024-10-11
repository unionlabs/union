// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package detect

import (
	"context"
	"testing"

	"golang.org/x/oauth2/google"
	"google.golang.org/api/option"
)

func TestIt(t *testing.T) {
	tests := []struct {
		name         string
		projectID    string
		env          map[string]string
		adcProjectID string
		want         string
	}{
		{
			name:      "noop",
			projectID: "noop",
			want:      "noop",
		},
		{
			name:      "environment project id",
			projectID: projectIDSentinel,
			env:       map[string]string{envProjectID: "environment-project-id"},
			want:      "environment-project-id",
		},
		{
			name:         "adc project id",
			projectID:    projectIDSentinel,
			adcProjectID: "adc-project-id",
			want:         "adc-project-id",
		},
		{
			name:      "emulator project id",
			projectID: projectIDSentinel,
			env:       map[string]string{"EMULATOR_HOST": "something"},
			want:      "emulated-project",
		},
	}

	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			envLookupFunc = func(k string) string {
				if tc.env == nil {
					return ""
				}
				return tc.env[k]
			}
			adcLookupFunc = func(context.Context, ...option.ClientOption) (*google.Credentials, error) {
				return &google.Credentials{ProjectID: tc.adcProjectID}, nil
			}

			got, err := ProjectID(context.Background(), tc.projectID, "EMULATOR_HOST")
			if err != nil {
				t.Fatalf("unexpected error from ProjectID(): %v", err)
			}
			if got != tc.want {
				t.Fatalf("ProjectID() = %q, want %q", got, tc.want)
			}
		})
	}

}
