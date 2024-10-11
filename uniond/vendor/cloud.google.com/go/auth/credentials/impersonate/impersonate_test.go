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

	"github.com/google/go-cmp/cmp"
)

func TestNewCredentials_serviceAccount(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name    string
		config  CredentialsOptions
		wantErr error
	}{
		{
			name:    "missing targetPrincipal",
			wantErr: errMissingTargetPrincipal,
		},
		{
			name: "missing scopes",
			config: CredentialsOptions{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			},
			wantErr: errMissingScopes,
		},
		{
			name: "lifetime over max",
			config: CredentialsOptions{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
				Lifetime:        13 * time.Hour,
			},
			wantErr: errLifetimeOverMax,
		},
		{
			name: "works",
			config: CredentialsOptions{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
			},
			wantErr: nil,
		},
		{
			name: "universe domain",
			config: CredentialsOptions{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
				Subject:         "admin@example.com",
				UniverseDomain:  "example.com",
			},
			wantErr: errUniverseNotSupportedDomainWideDelegation,
		},
	}

	for _, tt := range tests {
		name := tt.name
		t.Run(name, func(t *testing.T) {
			saTok := "sa-token"
			client := &http.Client{
				Transport: RoundTripFn(func(req *http.Request) *http.Response {
					if strings.Contains(req.URL.Path, "generateAccessToken") {
						defer req.Body.Close()
						b, err := io.ReadAll(req.Body)
						if err != nil {
							t.Error(err)
						}
						var r generateAccessTokenRequest
						if err := json.Unmarshal(b, &r); err != nil {
							t.Error(err)
						}
						if !cmp.Equal(r.Scope, tt.config.Scopes) {
							t.Errorf("got %v, want %v", r.Scope, tt.config.Scopes)
						}
						if !strings.Contains(req.URL.Path, tt.config.TargetPrincipal) {
							t.Errorf("got %q, want %q", req.URL.Path, tt.config.TargetPrincipal)
						}

						resp := generateAccessTokenResponse{
							AccessToken: saTok,
							ExpireTime:  time.Now().Format(time.RFC3339),
						}
						b, err = json.Marshal(&resp)
						if err != nil {
							t.Fatalf("unable to marshal response: %v", err)
						}
						return &http.Response{
							StatusCode: 200,
							Body:       io.NopCloser(bytes.NewReader(b)),
							Header:     http.Header{},
						}
					}
					return nil
				}),
			}
			tt.config.Client = client
			ts, err := NewCredentials(&tt.config)
			if err != nil {
				if err != tt.wantErr {
					t.Fatalf("err: %v", err)
				}
			} else {
				tok, err := ts.Token(ctx)
				if err != nil {
					t.Fatal(err)
				}
				if tok.Value != saTok {
					t.Fatalf("got %q, want %q", tok.Value, saTok)
				}
			}
		})
	}
}

type RoundTripFn func(req *http.Request) *http.Response

func (f RoundTripFn) RoundTrip(req *http.Request) (*http.Response, error) { return f(req), nil }

func TestCredentialsOptions_UniverseDomain(t *testing.T) {
	testCases := []struct {
		name               string
		opts               *CredentialsOptions
		wantUniverseDomain string
		wantIsGDU          bool
	}{
		{
			name:               "empty",
			opts:               &CredentialsOptions{},
			wantUniverseDomain: "googleapis.com",
			wantIsGDU:          true,
		},
		{
			name: "defaults",
			opts: &CredentialsOptions{
				UniverseDomain: "googleapis.com",
			},
			wantUniverseDomain: "googleapis.com",
			wantIsGDU:          true,
		},
		{
			name: "non-GDU",
			opts: &CredentialsOptions{
				UniverseDomain: "example.com",
			},
			wantUniverseDomain: "example.com",
			wantIsGDU:          false,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if got := tc.opts.getUniverseDomain(); got != tc.wantUniverseDomain {
				t.Errorf("got %v, want %v", got, tc.wantUniverseDomain)
			}
			if got := tc.opts.isUniverseDomainGDU(); got != tc.wantIsGDU {
				t.Errorf("got %v, want %v", got, tc.wantIsGDU)
			}
		})
	}
}
