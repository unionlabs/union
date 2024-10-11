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

package credentials

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

const computeMetadataEnvVar = "GCE_METADATA_HOST"

func TestComputeTokenProvider(t *testing.T) {
	scope := "https://www.googleapis.com/auth/bigquery"
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if strings.HasSuffix(r.URL.String(), computeTokenURI) {
			t.Errorf("got %q, want %q", r.URL.String(), computeTokenURI)
		}
		if r.URL.Query().Get("scopes") != scope {
			t.Errorf("got %q, want %q", r.URL.Query().Get("scopes"), scope)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token": "90d64460d14870c08c81352a05dedd3465940a7c", "token_type": "bearer", "expires_in": 86400}`))
	}))
	t.Setenv(computeMetadataEnvVar, strings.TrimPrefix(ts.URL, "http://"))
	tp := computeTokenProvider(&DetectOptions{
		EarlyTokenRefresh: 0,
		Scopes: []string{
			scope,
		},
	})
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if want := "90d64460d14870c08c81352a05dedd3465940a7c"; tok.Value != want {
		t.Errorf("got %q, want %q", tok.Value, want)
	}
	if want := "bearer"; tok.Type != want {
		t.Errorf("got %q, want %q", tok.Type, want)
	}
}
