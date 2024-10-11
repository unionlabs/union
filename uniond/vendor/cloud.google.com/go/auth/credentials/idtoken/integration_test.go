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

package idtoken_test

import (
	"context"
	"log"
	"net/http"
	"os"
	"strings"
	"testing"

	"cloud.google.com/go/auth/credentials/idtoken"
	"cloud.google.com/go/auth/httptransport"
	"cloud.google.com/go/auth/internal/credsfile"
	"cloud.google.com/go/auth/internal/testutil"
)

const (
	aud = "http://example.com"
)

func TestNewCredentials_CredentialsFile(t *testing.T) {
	testutil.IntegrationTestCheck(t)
	ctx := context.Background()
	ts, err := idtoken.NewCredentials(&idtoken.Options{
		Audience:        "http://example.com",
		CredentialsFile: os.Getenv(credsfile.GoogleAppCredsEnvVar),
	})
	if err != nil {
		t.Fatalf("unable to create credentials: %v", err)
	}
	tok, err := ts.Token(ctx)
	if err != nil {
		t.Fatalf("unable to retrieve Token: %v", err)
	}
	req := &http.Request{Header: make(http.Header)}
	httptransport.SetAuthHeader(tok, req)
	if !strings.HasPrefix(req.Header.Get("Authorization"), "Bearer ") {
		t.Fatalf("token should sign requests with Bearer Authorization header")
	}
	validTok, err := idtoken.Validate(context.Background(), tok.Value, aud)
	if err != nil {
		t.Fatalf("token validation failed: %v", err)
	}
	if validTok.Audience != aud {
		t.Fatalf("got %q, want %q", validTok.Audience, aud)
	}
}

func TestNewCredentials_CredentialsJSON(t *testing.T) {
	testutil.IntegrationTestCheck(t)
	ctx := context.Background()
	b, err := os.ReadFile(os.Getenv(credsfile.GoogleAppCredsEnvVar))
	if err != nil {
		log.Fatal(err)
	}
	creds, err := idtoken.NewCredentials(&idtoken.Options{
		Audience:        aud,
		CredentialsJSON: b,
	})
	if err != nil {
		t.Fatalf("unable to create Client: %v", err)
	}
	tok, err := creds.Token(ctx)
	if err != nil {
		t.Fatalf("unable to retrieve Token: %v", err)
	}
	validTok, err := idtoken.Validate(context.Background(), tok.Value, aud)
	if err != nil {
		t.Fatalf("token validation failed: %v", err)
	}
	if validTok.Audience != aud {
		t.Fatalf("got %q, want %q", validTok.Audience, aud)
	}
}
