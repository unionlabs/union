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
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"cloud.google.com/go/auth/internal"
)

var (
	baseImpersonateCredsReqBody  = "audience=32555940559.apps.googleusercontent.com&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcloud-platform&subject_token=street123&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Ajwt"
	baseImpersonateCredsRespBody = `{"accessToken":"Second.Access.Token","expireTime":"2020-12-28T15:01:23Z"}`
)

func TestImpersonation(t *testing.T) {
	var impersonationTests = []struct {
		name          string
		opts          *Options
		wantBody      string
		metricsHeader string
	}{
		{
			name: "Base Impersonation",
			opts: &Options{
				Audience:         "32555940559.apps.googleusercontent.com",
				SubjectTokenType: jwtTokenType,
				TokenInfoURL:     "http://localhost:8080/v1/tokeninfo",
				ClientSecret:     "notsosecret",
				ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
				CredentialSource: testBaseCredSource,
				Scopes:           []string{"https://www.googleapis.com/auth/devstorage.full_control"},
			},
			wantBody:      "{\"lifetime\":\"3600s\",\"scope\":[\"https://www.googleapis.com/auth/devstorage.full_control\"]}",
			metricsHeader: expectedMetricsHeader("file", true, false),
		},
		{
			name: "With TokenLifetime Set",
			opts: &Options{
				Audience:         "32555940559.apps.googleusercontent.com",
				SubjectTokenType: jwtTokenType,
				TokenInfoURL:     "http://localhost:8080/v1/tokeninfo",
				ClientSecret:     "notsosecret",
				ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
				CredentialSource: testBaseCredSource,
				Scopes:           []string{"https://www.googleapis.com/auth/devstorage.full_control"},
				ServiceAccountImpersonationLifetimeSeconds: 10000,
			},
			wantBody:      "{\"lifetime\":\"10000s\",\"scope\":[\"https://www.googleapis.com/auth/devstorage.full_control\"]}",
			metricsHeader: expectedMetricsHeader("file", true, false),
		},
	}
	for _, tt := range impersonationTests {
		t.Run(tt.name, func(t *testing.T) {
			ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
				if r.URL.Path == "/target" {
					headerAuth := r.Header.Get("Authorization")
					if got, want := headerAuth, "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					headerContentType := r.Header.Get("Content-Type")
					if got, want := headerContentType, "application/x-www-form-urlencoded"; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					body, err := io.ReadAll(r.Body)
					if err != nil {
						t.Fatalf("Failed reading request body: %v.", err)
					}
					if got, want := string(body), baseImpersonateCredsReqBody; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					w.Header().Set("Content-Type", "application/json")
					w.Write([]byte(baseCredsResponseBody))
				} else if r.URL.Path == "/impersonate" {
					headerAuth := r.Header.Get("Authorization")
					if got, want := headerAuth, "Bearer Sample.Access.Token"; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					headerContentType := r.Header.Get("Content-Type")
					if got, want := headerContentType, "application/json"; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					body, err := io.ReadAll(r.Body)
					if err != nil {
						t.Fatalf("Failed reading request body: %v.", err)
					}
					if got, want := string(body), tt.wantBody; got != want {
						t.Errorf("got %v, want %v", got, want)
					}
					w.Header().Set("Content-Type", "application/json")
					w.Write([]byte(baseImpersonateCredsRespBody))
				} else {
					t.Error("unmapped request")
				}
			}))
			defer ts.Close()

			testImpersonateOpts := tt.opts
			testImpersonateOpts.ServiceAccountImpersonationURL = ts.URL + "/impersonate"
			testImpersonateOpts.TokenURL = ts.URL + "/target"
			testImpersonateOpts.Client = internal.CloneDefaultClient()

			tp, err := NewTokenProvider(testImpersonateOpts)
			if err != nil {
				t.Fatalf("Failed to create Provider: %v", err)
			}

			oldNow := Now
			defer func() { Now = oldNow }()
			Now = testNow

			tok, err := tp.Token(context.Background())
			if err != nil {
				t.Fatalf("tp.Token() = %v", err)
			}
			if got, want := tok.Value, "Second.Access.Token"; got != want {
				t.Errorf("got %v, want %v", got, want)
			}
			if got, want := tok.Type, internal.TokenTypeBearer; got != want {
				t.Errorf("got %v, want %v", got, want)
			}
		})
	}
}
