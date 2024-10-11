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

package transport

import (
	"net/http"
	"reflect"
	"testing"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
)

// TestCloneDetectOptions_FieldTest is meant to fail every time a new field is
// added to the detect.Options type. This tests exists to make sure the
// CloneDetectOptions function is updated to copy over any new fields in the
// future. To make the test pass simply bump the int, but please also clone the
// relevant fields.
func TestCloneDetectOptions_FieldTest(t *testing.T) {
	const WantNumberOfFields = 13
	o := credentials.DetectOptions{}
	got := reflect.TypeOf(o).NumField()
	if got != WantNumberOfFields {
		t.Errorf("if this fails please read comment above the test: got %v, want %v", got, WantNumberOfFields)
	}
}

func TestCloneDetectOptions(t *testing.T) {
	oldDo := &credentials.DetectOptions{
		Audience:          "aud",
		Subject:           "sub",
		EarlyTokenRefresh: 42,
		TokenURL:          "TokenURL",
		STSAudience:       "STSAudience",
		CredentialsFile:   "CredentialsFile",
		UseSelfSignedJWT:  true,
		CredentialsJSON:   []byte{1, 2, 3, 4, 5},
		Scopes:            []string{"a", "b"},
		Client:            &http.Client{},
		AuthHandlerOptions: &auth.AuthorizationHandlerOptions{
			Handler: func(authCodeURL string) (code string, state string, err error) {
				return "", "", nil
			},
			State: "state",
			PKCEOpts: &auth.PKCEOptions{
				Challenge:       "Challenge",
				ChallengeMethod: "ChallengeMethod",
				Verifier:        "Verifier",
			},
		},
	}
	newDo := CloneDetectOptions(oldDo)

	// Simple fields
	if got, want := newDo.Audience, oldDo.Audience; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.Subject, oldDo.Subject; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.EarlyTokenRefresh, oldDo.EarlyTokenRefresh; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.TokenURL, oldDo.TokenURL; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.STSAudience, oldDo.STSAudience; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.CredentialsFile, oldDo.CredentialsFile; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.UseSelfSignedJWT, oldDo.UseSelfSignedJWT; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}

	// Slices
	if got, want := len(newDo.CredentialsJSON), len(oldDo.CredentialsJSON); got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.CredentialsJSON, oldDo.CredentialsJSON; reflect.ValueOf(got).Pointer() == reflect.ValueOf(want).Pointer() {
		t.Fatalf("CredentialsJSON should not reference the same slice")
	}
	if got, want := len(newDo.Scopes), len(oldDo.Scopes); got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := newDo.Scopes, oldDo.Scopes; reflect.ValueOf(got).Pointer() == reflect.ValueOf(want).Pointer() {
		t.Fatalf("Scopes should not reference the same slice")
	}

	// Pointer types that should be the same memory
	if got, want := newDo.Client, oldDo.Client; reflect.ValueOf(got).Pointer() != reflect.ValueOf(want).Pointer() {
		t.Fatalf("Scopes should not reference the same slice")
	}
	if got, want := newDo.AuthHandlerOptions, oldDo.AuthHandlerOptions; reflect.ValueOf(got).Pointer() != reflect.ValueOf(want).Pointer() {
		t.Fatalf("Scopes should not reference the same slice")
	}
}
