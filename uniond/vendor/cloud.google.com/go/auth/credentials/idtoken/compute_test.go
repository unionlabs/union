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

package idtoken

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

const metadataHostEnv = "GCE_METADATA_HOST"

func TestComputeCredentials(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.Contains(r.URL.Path, identitySuffix) {
			t.Errorf("got %q, want contains %q", r.URL.Path, identitySuffix)
		}
		if got, want := r.URL.Query().Get("audience"), "aud"; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		if got, want := r.URL.Query().Get("format"), "full"; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		if got, want := r.URL.Query().Get("licenses"), "TRUE"; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		w.Write([]byte(`fake_token`))
	}))
	defer ts.Close()
	t.Setenv(metadataHostEnv, strings.TrimPrefix(ts.URL, "http://"))
	tp, err := computeCredentials(&Options{
		Audience:           "aud",
		ComputeTokenFormat: ComputeTokenFormatFullWithLicense,
	})
	if err != nil {
		t.Fatalf("computeCredentials() = %v", err)
	}
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatalf("tp.Token() = %v", err)
	}
	if want := "fake_token"; tok.Value != want {
		t.Errorf("got %q, want %q", tok.Value, want)
	}
}

func TestComputeCredentials_Standard(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.Contains(r.URL.Path, identitySuffix) {
			t.Errorf("got %q, want contains %q", r.URL.Path, identitySuffix)
		}
		if got, want := r.URL.Query().Get("audience"), "aud"; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		if got, want := r.URL.Query().Get("format"), ""; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		if got, want := r.URL.Query().Get("licenses"), ""; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		w.Write([]byte(`fake_token`))
	}))
	defer ts.Close()
	t.Setenv(metadataHostEnv, strings.TrimPrefix(ts.URL, "http://"))
	tp, err := computeCredentials(&Options{
		Audience:           "aud",
		ComputeTokenFormat: ComputeTokenFormatStandard,
	})
	if err != nil {
		t.Fatalf("computeCredentials() = %v", err)
	}
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatalf("tp.Token() = %v", err)
	}
	if want := "fake_token"; tok.Value != want {
		t.Errorf("got %q, want %q", tok.Value, want)
	}
}

func TestComputeCredentials_Invalid(t *testing.T) {
	if _, err := computeCredentials(&Options{
		Audience:     "aud",
		CustomClaims: map[string]interface{}{"foo": "bar"},
	}); err == nil {
		t.Fatal("computeCredentials() = nil, expected non-nil error", err)
	}
}
