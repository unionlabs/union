// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package idtoken_test

import (
	"context"
	"net/http"
	"os"
	"strings"
	"testing"

	"golang.org/x/oauth2/google"
	"google.golang.org/api/idtoken"
	"google.golang.org/api/option"
)

const (
	envCredentialFile = "GOOGLE_APPLICATION_CREDENTIALS"

	aud = "http://example.com"
)

func TestNewTokenSource(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	ts, err := idtoken.NewTokenSource(context.Background(), "http://example.com", option.WithCredentialsFile(os.Getenv(envCredentialFile)))
	if err != nil {
		t.Fatalf("unable to create TokenSource: %v", err)
	}
	tok, err := ts.Token()
	if err != nil {
		t.Fatalf("unable to retrieve Token: %v", err)
	}
	req := &http.Request{Header: make(http.Header)}
	tok.SetAuthHeader(req)
	if !strings.HasPrefix(req.Header.Get("Authorization"), "Bearer ") {
		t.Fatalf("token should sign requests with Bearer Authorization header")
	}
	validTok, err := idtoken.Validate(context.Background(), tok.AccessToken, aud)
	if err != nil {
		t.Fatalf("token validation failed: %v", err)
	}
	if validTok.Audience != aud {
		t.Fatalf("got %q, want %q", validTok.Audience, aud)
	}
}

func TestNewTokenSource_WithCredentialJSON(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	ctx := context.Background()
	creds, err := google.FindDefaultCredentials(ctx)
	if err != nil {
		t.Fatalf("unable to find default creds: %v", err)
	}
	ts, err := idtoken.NewTokenSource(ctx, aud, option.WithCredentialsJSON(creds.JSON))
	if err != nil {
		t.Fatalf("unable to create Client: %v", err)
	}
	tok, err := ts.Token()
	if err != nil {
		t.Fatalf("unable to retrieve Token: %v", err)
	}
	req := &http.Request{Header: make(http.Header)}
	tok.SetAuthHeader(req)
	if !strings.HasPrefix(req.Header.Get("Authorization"), "Bearer ") {
		t.Fatalf("token should sign requests with Bearer Authorization header")
	}
	validTok, err := idtoken.Validate(context.Background(), tok.AccessToken, aud)
	if err != nil {
		t.Fatalf("token validation failed: %v", err)
	}
	if validTok.Audience != aud {
		t.Fatalf("got %q, want %q", validTok.Audience, aud)
	}
}
