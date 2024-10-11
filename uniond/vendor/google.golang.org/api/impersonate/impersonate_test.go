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

func TestTokenSource_serviceAccount(t *testing.T) {
	ctx := context.Background()
	tests := []struct {
		name    string
		config  CredentialsConfig
		opts    option.ClientOption
		wantErr error
	}{
		{
			name:    "missing targetPrincipal",
			wantErr: errMissingTargetPrincipal,
		},
		{
			name: "missing scopes",
			config: CredentialsConfig{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
			},
			wantErr: errMissingScopes,
		},
		{
			name: "lifetime over max",
			config: CredentialsConfig{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
				Lifetime:        13 * time.Hour,
			},
			wantErr: errLifetimeOverMax,
		},
		{
			name: "works",
			config: CredentialsConfig{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
			},
			wantErr: nil,
		},
		{
			name: "universe domain",
			config: CredentialsConfig{
				TargetPrincipal: "foo@project-id.iam.gserviceaccount.com",
				Scopes:          []string{"scope"},
				Subject:         "admin@example.com",
			},
			opts:    option.WithUniverseDomain("example.com"),
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
						resp := generateAccessTokenResp{
							AccessToken: saTok,
							ExpireTime:  time.Now().Format(time.RFC3339),
						}
						b, err := json.Marshal(&resp)
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
			opts := []option.ClientOption{
				option.WithHTTPClient(client),
			}
			if tt.opts != nil {
				opts = append(opts, tt.opts)
			}
			ts, err := CredentialsTokenSource(ctx, tt.config, opts...)

			if err != nil {
				if err != tt.wantErr {
					t.Fatalf("%s: err: %v", tt.name, err)
				}
			} else {
				tok, err := ts.Token()
				if err != nil {
					t.Fatal(err)
				}
				if tok.AccessToken != saTok {
					t.Fatalf("got %q, want %q", tok.AccessToken, saTok)
				}
			}
		})
	}
}

type RoundTripFn func(req *http.Request) *http.Response

func (f RoundTripFn) RoundTrip(req *http.Request) (*http.Response, error) { return f(req), nil }
