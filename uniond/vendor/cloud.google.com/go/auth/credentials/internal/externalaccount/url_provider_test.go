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

package externalaccount

import (
	"context"
	"net/http"
	"net/http/httptest"
	"testing"

	"cloud.google.com/go/auth/internal/credsfile"
)

func TestRetrieveURLSubjectToken_Text(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if want := http.MethodGet; r.Method != want {
			t.Errorf("got %v, want %v", r.Method, want)
		}
		if got, want := r.Header.Get("Metadata"), "True"; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		w.Write([]byte("testTokenValue"))
	}))
	defer ts.Close()

	opts := cloneTestOpts()
	opts.CredentialSource = &credsfile.CredentialSource{
		URL:    ts.URL,
		Format: &credsfile.Format{Type: fileTypeText},
		Headers: map[string]string{
			"Metadata": "True",
		},
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("base.subjectToken() = %v", err)
	}
	if want := "testTokenValue"; got != want {
		t.Errorf("got %q, want %q", got, want)
	}
	if got, want := base.providerType(), urlProviderType; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestRetrieveURLSubjectToken_Untyped(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if want := http.MethodGet; r.Method != want {
			t.Errorf("got %v, want %v", r.Method, want)
		}
		w.Write([]byte("testTokenValue"))
	}))
	defer ts.Close()

	opts := cloneTestOpts()
	opts.CredentialSource = &credsfile.CredentialSource{
		URL: ts.URL,
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() failed %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("base.subjectToken() = %v", err)
	}
	if want := "testTokenValue"; got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestRetrieveURLSubjectToken_JSON(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.Method, "GET"; got != want {
			t.Errorf("got %v, want %v", r.Method, want)
		}
		w.Write([]byte(`{"SubjToken":"testTokenValue"}`))
	}))
	defer ts.Close()

	opts := cloneTestOpts()
	opts.CredentialSource = &credsfile.CredentialSource{
		URL:    ts.URL,
		Format: &credsfile.Format{Type: fileTypeJSON, SubjectTokenFieldName: "SubjToken"},
	}

	base, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatalf("newSubjectTokenProvider() = %v", err)
	}

	got, err := base.subjectToken(context.Background())
	if err != nil {
		t.Fatalf("base.subjectToken() = %v", err)
	}
	if want := "testTokenValue"; got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}
