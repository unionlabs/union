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

	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/jwt"
)

func TestNewCredentials_user(t *testing.T) {
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
					defer req.Body.Close()
					if strings.Contains(req.URL.Path, "signJwt") {
						b, err := io.ReadAll(req.Body)
						if err != nil {
							t.Error(err)
						}
						var r signJWTRequest
						if err := json.Unmarshal(b, &r); err != nil {
							t.Error(err)
						}
						jwtPayload := map[string]interface{}{}
						if err := json.Unmarshal([]byte(r.Payload), &jwtPayload); err != nil {
							t.Error(err)
						}
						if got, want := jwtPayload["iss"].(string), tt.targetPrincipal; got != want {
							t.Errorf("got %q, want %q", got, want)
						}
						if got, want := jwtPayload["sub"].(string), tt.subject; got != want {
							t.Errorf("got %q, want %q", got, want)
						}
						if got, want := jwtPayload["scope"].(string), strings.Join(tt.scopes, ","); got != want {
							t.Errorf("got %q, want %q", got, want)
						}

						resp := signJWTResponse{
							KeyID:     "123",
							SignedJWT: jwt.HeaderType,
						}
						b, err = json.Marshal(&resp)
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
							TokenType:   internal.TokenTypeBearer,
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
			ts, err := NewCredentials(&CredentialsOptions{
				TargetPrincipal: tt.targetPrincipal,
				Scopes:          tt.scopes,
				Lifetime:        tt.lifetime,
				Subject:         tt.subject,
				Client:          client,
				UniverseDomain:  tt.universeDomain,
			})
			if tt.wantErr && err != nil {
				return
			}
			if err != nil {
				t.Fatal(err)
			}
			tok, err := ts.Token(ctx)
			if err != nil {
				t.Fatal(err)
			}
			if tok.Value != userTok {
				t.Fatalf("got %q, want %q", tok.Value, userTok)
			}
		})
	}
}
