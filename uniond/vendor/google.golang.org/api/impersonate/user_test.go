// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impersonate

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"net/http"
	"strings"
	"testing"
	"time"

	"google.golang.org/api/option"
)

func TestTokenSource_user(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name            string
		targetPrincipal string
		scopes          []string
		lifetime        time.Duration
		subject         string
		wantErr         bool
		universeDomain  string
	}{
		{
			name:    "missing targetPrincipal",
			wantErr: true,
		},
		{
			name:            "missing scopes",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			wantErr:         true,
		},
		{
			name:            "lifetime over max",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			scopes:          []string{"scope"},
			lifetime:        13 * time.Hour,
			wantErr:         true,
		},
		{
			name:            "works",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			scopes:          []string{"scope"},
			subject:         "admin@example.com",
			wantErr:         false,
		},
		{
			name:            "universeDomain",
			targetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			scopes:          []string{"scope"},
			subject:         "admin@example.com",
			wantErr:         true,
			// Non-GDU Universe Domain should result in error if
			// CredentialsConfig.Subject is present for domain-wide delegation.
			universeDomain: "example.com",
		},
	}

	for _, tt := range tests {
		userTok := "user-token"
		name := tt.name
		t.Run(name, func(t *testing.T) {
			client := &http.Client{
				Transport: RoundTripFn(func(req *http.Request) *http.Response {
					if strings.Contains(req.URL.Path, "signJwt") {
						resp := signJWTResponse{
							KeyID:     "123",
							SignedJWT: "jwt",
						}
						b, err := json.Marshal(&resp)
						if err != nil {
							t.Fatalf("unable to marshal response: %v", err)
						}
						return &http.Response{
							StatusCode: 200,
							Body:       io.NopCloser(bytes.NewReader(b)),
							Header:     make(http.Header),
						}
					}
					if strings.Contains(req.URL.Path, "/token") {
						resp := exchangeTokenResponse{
							AccessToken: userTok,
							TokenType:   "Bearer",
							ExpiresIn:   int64(time.Hour.Seconds()),
						}
						b, err := json.Marshal(&resp)
						if err != nil {
							t.Fatalf("unable to marshal response: %v", err)
						}
						return &http.Response{
							StatusCode: 200,
							Body:       io.NopCloser(bytes.NewReader(b)),
							Header:     make(http.Header),
						}
					}
					return nil
				}),
			}
			ts, err := CredentialsTokenSource(ctx,
				CredentialsConfig{
					TargetPrincipal: tt.targetPrincipal,
					Scopes:          tt.scopes,
					Lifetime:        tt.lifetime,
					Subject:         tt.subject,
				},
				option.WithHTTPClient(client),
				option.WithUniverseDomain(tt.universeDomain))
			if tt.wantErr && err != nil {
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			tok, err := ts.Token()
			if err != nil {
				t.Fatal(err)
			}
			if tok.AccessToken != userTok {
				t.Fatalf("got %q, want %q", tok.AccessToken, userTok)
			}
		})
	}
}
