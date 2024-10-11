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
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials/internal/stsexchange"
	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
)

const (
	textBaseCredPath                              = "testdata/3pi_cred.txt"
	jsonBaseCredPath                              = "testdata/3pi_cred.json"
	baseCredsRequestBody                          = "audience=32555940559.apps.googleusercontent.com&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdevstorage.full_control&subject_token=street123&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aid_token"
	baseCredsResponseBody                         = `{"access_token":"Sample.Access.Token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"scope":"https://www.googleapis.com/auth/cloud-platform"}`
	workforcePoolRequestBodyWithClientID          = "audience=%2F%2Fiam.googleapis.com%2Flocations%2Feu%2FworkforcePools%2Fpool-id%2Fproviders%2Fprovider-id&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdevstorage.full_control&subject_token=street123&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aid_token"
	workforcePoolRequestBodyWithoutClientID       = "audience=%2F%2Fiam.googleapis.com%2Flocations%2Feu%2FworkforcePools%2Fpool-id%2Fproviders%2Fprovider-id&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&options=%7B%22userProject%22%3A%22myProject%22%7D&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdevstorage.full_control&subject_token=street123&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aid_token"
	correctAT                                     = "Sample.Access.Token"
	expiry                                  int64 = 234852
)

var (
	testOpts = &Options{
		Audience:         "32555940559.apps.googleusercontent.com",
		SubjectTokenType: jwtTokenType,
		TokenInfoURL:     "http://localhost:8080/v1/tokeninfo",
		ClientSecret:     "notsosecret",
		ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
		CredentialSource: testBaseCredSource,
		Scopes:           []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		Client:           internal.CloneDefaultClient(),
	}
	testBaseCredSource = &credsfile.CredentialSource{
		File:   textBaseCredPath,
		Format: &credsfile.Format{Type: fileTypeText},
	}
	testNow = func() time.Time { return time.Unix(expiry, 0) }
)

func TestToken(t *testing.T) {
	tests := []struct {
		name      string
		respBody  *stsexchange.TokenResponse
		wantError bool
	}{
		{
			name: "works",
			respBody: &stsexchange.TokenResponse{
				AccessToken:     correctAT,
				IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
				TokenType:       "Bearer",
				ExpiresIn:       3600,
				Scope:           "https://www.googleapis.com/auth/cloud-platform",
			},
		},
		{
			name: "no exp time on tok",
			respBody: &stsexchange.TokenResponse{
				AccessToken:     correctAT,
				IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
				TokenType:       "Bearer",
				Scope:           "https://www.googleapis.com/auth/cloud-platform",
			},
			wantError: true,
		},
		{
			name: "negative exp time",
			respBody: &stsexchange.TokenResponse{
				AccessToken:     correctAT,
				IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
				TokenType:       "Bearer",
				ExpiresIn:       -1,
				Scope:           "https://www.googleapis.com/auth/cloud-platform",
			},
			wantError: true,
		},
	}
	for _, tt := range tests {
		opts := &Options{
			Audience:         "32555940559.apps.googleusercontent.com",
			SubjectTokenType: idTokenType,
			ClientSecret:     "notsosecret",
			ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
			CredentialSource: testBaseCredSource,
			Scopes:           []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		}

		respBody, err := json.Marshal(tt.respBody)
		if err != nil {
			t.Fatal(err)
		}

		server := &testExchangeTokenServer{
			url:           "/",
			authorization: "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ=",
			contentType:   "application/x-www-form-urlencoded",
			body:          baseCredsRequestBody,
			response:      string(respBody),
			metricsHeader: expectedMetricsHeader("file", false, false),
		}

		tok, err := run(t, opts, server)
		if err != nil && !tt.wantError {
			t.Fatal(err)
		}
		if tt.wantError {
			if err == nil {
				t.Fatal("want err, got nil")
			}
			continue
		}
		validateToken(t, tok)
	}
	opts := &Options{
		Audience:         "32555940559.apps.googleusercontent.com",
		SubjectTokenType: idTokenType,
		ClientSecret:     "notsosecret",
		ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
		CredentialSource: testBaseCredSource,
		Scopes:           []string{"https://www.googleapis.com/auth/devstorage.full_control"},
	}

	server := &testExchangeTokenServer{
		url:           "/",
		authorization: "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ=",
		contentType:   "application/x-www-form-urlencoded",
		body:          baseCredsRequestBody,
		response:      baseCredsResponseBody,
		metricsHeader: expectedMetricsHeader("file", false, false),
	}

	tok, err := run(t, opts, server)
	if err != nil {
		t.Fatal(err)
	}
	validateToken(t, tok)
}

func TestWorkforcePoolTokenWithClientID(t *testing.T) {
	opts := Options{
		Audience:                 "//iam.googleapis.com/locations/eu/workforcePools/pool-id/providers/provider-id",
		SubjectTokenType:         idTokenType,
		ClientSecret:             "notsosecret",
		ClientID:                 "rbrgnognrhongo3bi4gb9ghg9g",
		CredentialSource:         testBaseCredSource,
		Scopes:                   []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		WorkforcePoolUserProject: "myProject",
	}

	server := testExchangeTokenServer{
		url:           "/",
		authorization: "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ=",
		contentType:   "application/x-www-form-urlencoded",
		body:          workforcePoolRequestBodyWithClientID,
		response:      baseCredsResponseBody,
		metricsHeader: expectedMetricsHeader("file", false, false),
	}

	tok, err := run(t, &opts, &server)
	if err != nil {
		t.Fatal(err)
	}
	validateToken(t, tok)
}

func TestWorkforcePoolTokenWithoutClientID(t *testing.T) {
	opts := Options{
		Audience:                 "//iam.googleapis.com/locations/eu/workforcePools/pool-id/providers/provider-id",
		SubjectTokenType:         idTokenType,
		ClientSecret:             "notsosecret",
		CredentialSource:         testBaseCredSource,
		Scopes:                   []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		WorkforcePoolUserProject: "myProject",
	}

	server := testExchangeTokenServer{
		url:           "/",
		authorization: "",
		contentType:   "application/x-www-form-urlencoded",
		body:          workforcePoolRequestBodyWithoutClientID,
		response:      baseCredsResponseBody,
		metricsHeader: expectedMetricsHeader("file", false, false),
	}

	tok, err := run(t, &opts, &server)
	if err != nil {
		t.Fatal(err)
	}
	validateToken(t, tok)
}

func TestNonworkforceWithWorkforcePoolUserProject(t *testing.T) {
	opts := &Options{
		Audience:                 "32555940559.apps.googleusercontent.com",
		SubjectTokenType:         idTokenType,
		TokenURL:                 "https://sts.googleapis.com",
		ClientSecret:             "notsosecret",
		ClientID:                 "rbrgnognrhongo3bi4gb9ghg9g",
		CredentialSource:         testBaseCredSource,
		Scopes:                   []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		WorkforcePoolUserProject: "myProject",
		Client:                   internal.CloneDefaultClient(),
	}

	_, err := NewTokenProvider(opts)
	if err == nil {
		t.Fatalf("got nil, want an error")
	}
	if got, want := err.Error(), "externalaccount: workforce_pool_user_project should not be set for non-workforce pool credentials"; got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}

func TestWorkforcePoolCreation(t *testing.T) {
	var audienceValidatyTests = []struct {
		audience      string
		expectSuccess bool
	}{
		{"//iam.googleapis.com/locations/global/workforcePools/pool-id/providers/provider-id", true},
		{"//iam.googleapis.com/locations/eu/workforcePools/pool-id/providers/provider-id", true},
		{"//iam.googleapis.com/locations/eu/workforcePools/workloadIdentityPools/providers/provider-id", true},
		{"identitynamespace:1f12345:my_provider", false},
		{"//iam.googleapis.com/projects/123456/locations/global/workloadIdentityPools/pool-id/providers/provider-id", false},
		{"//iam.googleapis.com/projects/123456/locations/eu/workloadIdentityPools/pool-id/providers/provider-id", false},
		{"//iam.googleapis.com/projects/123456/locations/global/workloadIdentityPools/workforcePools/providers/provider-id", false},
		{"//iamgoogleapis.com/locations/eu/workforcePools/pool-id/providers/provider-id", false},
		{"//iam.googleapiscom/locations/eu/workforcePools/pool-id/providers/provider-id", false},
		{"//iam.googleapis.com/locations/workforcePools/pool-id/providers/provider-id", false},
		{"//iam.googleapis.com/locations/eu/workforcePool/pool-id/providers/provider-id", false},
		{"//iam.googleapis.com/locations//workforcePool/pool-id/providers/provider-id", false},
	}
	for _, tt := range audienceValidatyTests {
		t.Run(" "+tt.audience, func(t *testing.T) { // We prepend a space ahead of the test input when outputting for sake of readability.
			opts := testOpts
			opts.TokenURL = "https://sts.googleapis.com" // Setting the most basic acceptable tokenURL
			opts.ServiceAccountImpersonationURL = "https://iamcredentials.googleapis.com"
			opts.Audience = tt.audience
			opts.WorkforcePoolUserProject = "myProject"
			_, err := NewTokenProvider(opts)

			if tt.expectSuccess && err != nil {
				t.Errorf("got %v, want nil", err)
			} else if !tt.expectSuccess && err == nil {
				t.Errorf("got nil, want an error")
			}
		})
	}
}

type testExchangeTokenServer struct {
	url           string
	authorization string
	contentType   string
	body          string
	response      string
	metricsHeader string
}

func run(t *testing.T, opts *Options, tets *testExchangeTokenServer) (*auth.Token, error) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.URL.String(), tets.url; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		headerAuth := r.Header.Get("Authorization")
		if got, want := headerAuth, tets.authorization; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		headerContentType := r.Header.Get("Content-Type")
		if got, want := headerContentType, tets.contentType; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		headerMetrics := r.Header.Get("x-goog-api-client")
		if got, want := headerMetrics, tets.metricsHeader; got != want {
			t.Errorf("got %v but want %v", got, want)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("Failed reading request body: %s.", err)
		}
		if got, want := string(body), tets.body; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(tets.response))
	}))
	defer server.Close()
	opts.TokenURL = server.URL

	oldNow := Now
	defer func() { Now = oldNow }()
	Now = testNow

	stp, err := newSubjectTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	tp := &tokenProvider{
		opts:   opts,
		client: internal.CloneDefaultClient(),
		stp:    stp,
	}

	return tp.Token(context.Background())
}

func validateToken(t *testing.T, tok *auth.Token) {
	if got, want := tok.Value, correctAT; got != want {
		t.Errorf("got %v, want %v", got, want)
	}
	if got, want := tok.Type, internal.TokenTypeBearer; got != want {
		t.Errorf("got %v, want %v", got, want)
	}

	if got, want := tok.Expiry, testNow().Add(time.Duration(3600)*time.Second); got != want {
		t.Errorf("got %v, want %v", got, want)
	}
}

func cloneTestOpts() *Options {
	return &Options{
		Audience:                       "32555940559.apps.googleusercontent.com",
		SubjectTokenType:               jwtTokenType,
		TokenURL:                       "http://localhost:8080/v1/token",
		TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
		ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
		ClientSecret:                   "notsosecret",
		ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
		Client:                         internal.CloneDefaultClient(),
	}
}

func expectedMetricsHeader(source string, saImpersonation bool, configLifetime bool) string {
	return fmt.Sprintf("gl-go/%s auth/unknown google-byoid-sdk source/%s sa-impersonation/%t config-lifetime/%t", goVersion(), source, saImpersonation, configLifetime)
}

func TestOptionsValidate(t *testing.T) {
	tests := []struct {
		name    string
		o       *Options
		wantErr bool
	}{
		{
			name: "works",
			o: &Options{
				Audience:                       "32555940559.apps.googleusercontent.com",
				SubjectTokenType:               jwtTokenType,
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
				CredentialSource:               testBaseCredSource,
			},
		},
		{
			name: "missing aud",
			o: &Options{
				SubjectTokenType:               jwtTokenType,
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
				CredentialSource:               testBaseCredSource,
			},
			wantErr: true,
		},
		{
			name: "missing subjectTokenType",
			o: &Options{
				Audience:                       "32555940559.apps.googleusercontent.com",
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
				CredentialSource:               testBaseCredSource,
			},
			wantErr: true,
		},
		{
			name: "invalid workforcepool",
			o: &Options{
				WorkforcePoolUserProject:       "blah",
				Audience:                       "32555940559.apps.googleusercontent.com",
				SubjectTokenType:               jwtTokenType,
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
				CredentialSource:               testBaseCredSource,
			},
			wantErr: true,
		},
		{
			name: "no creds",
			o: &Options{
				Audience:                       "32555940559.apps.googleusercontent.com",
				SubjectTokenType:               jwtTokenType,
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
			},
			wantErr: true,
		},
		{
			name: "too many creds",
			o: &Options{
				Audience:                       "32555940559.apps.googleusercontent.com",
				SubjectTokenType:               jwtTokenType,
				TokenURL:                       "http://localhost:8080/v1/token",
				TokenInfoURL:                   "http://localhost:8080/v1/tokeninfo",
				ServiceAccountImpersonationURL: "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/service-gcs-admin@$PROJECT_ID.iam.gserviceaccount.com:generateAccessToken",
				ClientSecret:                   "notsosecret",
				ClientID:                       "rbrgnognrhongo3bi4gb9ghg9g",
				Client:                         internal.CloneDefaultClient(),
				CredentialSource:               testBaseCredSource,
				SubjectTokenProvider:           fakeSubjectTokenProvider{},
			},
			wantErr: true,
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			err := tc.o.validate()
			if err == nil && tc.wantErr {
				t.Fatalf("o.validate() = nil, want error")
			}
			if err != nil && !tc.wantErr {
				t.Fatalf("o.validate() = non-nil error, want error")
			}
		})
	}
}

func TestOptionsResolveTokenURL(t *testing.T) {
	tests := []struct {
		name string
		o    *Options
		want string
	}{
		{
			name: "default",
			o:    &Options{},
			want: "https://sts.googleapis.com/v1/token",
		},
		{
			name: "Options TokenURL",
			o: &Options{
				TokenURL: "http://localhost:8080/v1/token",
			},
			want: "http://localhost:8080/v1/token",
		},
		{
			name: "Options UniverseDomain",
			o: &Options{
				UniverseDomain: "example.com",
			},
			want: "https://sts.example.com/v1/token",
		},
		{
			name: "Options TokenURL overrides UniverseDomain",
			o: &Options{
				TokenURL:       "http://localhost:8080/v1/token",
				UniverseDomain: "example.com",
			},
			want: "http://localhost:8080/v1/token",
		},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			tc.o.resolveTokenURL()
			if tc.o.TokenURL != tc.want {
				t.Errorf("got %s, want %s", tc.o.TokenURL, tc.want)
			}
		})
	}
}
