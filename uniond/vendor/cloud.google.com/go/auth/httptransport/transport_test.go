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

package httptransport

import (
	"testing"

	"cloud.google.com/go/auth/internal"
)

func TestAuthTransport_GetClientUniverseDomain(t *testing.T) {
	nonDefault := "example.com"
	tests := []struct {
		name           string
		universeDomain string
		want           string
	}{
		{
			name:           "default",
			universeDomain: "",
			want:           internal.DefaultUniverseDomain,
		},
		{
			name:           "non-default",
			universeDomain: nonDefault,
			want:           nonDefault,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			at := &authTransport{clientUniverseDomain: tt.universeDomain}
			got := at.getClientUniverseDomain()
			if got != tt.want {
				t.Errorf("got %q, want %q", got, tt.want)
			}
		})
	}
}
