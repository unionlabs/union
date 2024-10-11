// Copyright 2024 Google LLC
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
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	iexacc "cloud.google.com/go/auth/credentials/internal/externalaccount"
)

const (
	accessKeyID     = "accessKeyID"
	secretAccessKey = "secret"
	sessionToken    = "sessionTok"
	subjectTok      = `%7B%22url%22%3A%22https%3A%2F%2Fsts.us-east-2.amazonaws.com%3FAction%3DGetCallerIdentity%5Cu0026Version%3D2011-06-15%22%2C%22method%22%3A%22POST%22%2C%22headers%22%3A%5B%7B%22key%22%3A%22Authorization%22%2C%22value%22%3A%22AWS4-HMAC-SHA256+Credential%3DaccessKeyID%2F20110909%2Fus-east-2%2Fsts%2Faws4_request%2C+SignedHeaders%3Dhost%3Bx-amz-date%3Bx-amz-security-token%3Bx-goog-cloud-target-resource%2C+Signature%3D19e8a661c61d39d19a9c82e272deef7784908176b82b0eb42f328d2c640f369b%22%7D%2C%7B%22key%22%3A%22Host%22%2C%22value%22%3A%22sts.us-east-2.amazonaws.com%22%7D%2C%7B%22key%22%3A%22X-Amz-Date%22%2C%22value%22%3A%2220110909T233600Z%22%7D%2C%7B%22key%22%3A%22X-Amz-Security-Token%22%2C%22value%22%3A%22sessionTok%22%7D%2C%7B%22key%22%3A%22X-Goog-Cloud-Target-Resource%22%2C%22value%22%3A%2232555940559.apps.googleusercontent.com%22%7D%5D%7D`
)

var (
	defaultTime = time.Date(2011, 9, 9, 23, 36, 0, 0, time.UTC)
)

func TestNewCredentials_AwsSecurityCredentials(t *testing.T) {
	opts := &Options{
		Audience:         "32555940559.apps.googleusercontent.com",
		SubjectTokenType: "urn:ietf:params:oauth:token-type:jwt",
		ClientSecret:     "notsosecret",
		ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
	}
	opts.AwsSecurityCredentialsProvider = &fakeAwsCredsProvider{
		awsRegion: "us-east-2",
		creds: &AwsSecurityCredentials{
			AccessKeyID:     accessKeyID,
			SecretAccessKey: secretAccessKey,
			SessionToken:    sessionToken,
		},
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer r.Body.Close()
		if r.URL.Path == "/sts" {
			r.ParseForm()
			if got, want := r.Form.Get("subject_token"), subjectTok; got != want {
				t.Errorf("got %q, want %q", got, want)
			}

			resp := &struct {
				AccessToken string `json:"access_token"`
				ExpiresIn   int    `json:"expires_in"`
			}{
				AccessToken: "a_fake_token_sts",
				ExpiresIn:   60,
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else if r.URL.Path == "/impersonate" {
			if want := "a_fake_token_sts"; !strings.Contains(r.Header.Get("Authorization"), want) {
				t.Errorf("missing sts token: got %q, want %q", r.Header.Get("Authorization"), want)
			}

			resp := &struct {
				AccessToken string `json:"accessToken"`
				ExpireTime  string `json:"expireTime"`
			}{
				AccessToken: "a_fake_token",
				ExpireTime:  "2006-01-02T15:04:05Z",
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else {
			t.Errorf("unexpected call to %q", r.URL.Path)
		}
	}))
	opts.ServiceAccountImpersonationURL = ts.URL + "/impersonate"
	opts.TokenURL = ts.URL + "/sts"

	oldNow := iexacc.Now
	defer func() {
		iexacc.Now = oldNow
	}()
	iexacc.Now = func() time.Time {
		return defaultTime
	}

	creds, err := NewCredentials(opts)
	if err != nil {
		t.Fatalf("NewCredentials() = %v", err)
	}
	if _, err := creds.Token(context.Background()); err != nil {
		t.Fatalf("creds.Token() = %v", err)
	}
}

func TestNewCredentials_SubjectTokenProvider(t *testing.T) {
	opts := &Options{
		Audience:         "32555940559.apps.googleusercontent.com",
		SubjectTokenType: "urn:ietf:params:oauth:token-type:jwt",
		ClientSecret:     "notsosecret",
		ClientID:         "rbrgnognrhongo3bi4gb9ghg9g",
	}
	opts.SubjectTokenProvider = &fakeSubjectTokenProvider{
		subjectToken: "fake_token",
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer r.Body.Close()
		if r.URL.Path == "/sts" {
			r.ParseForm()
			if got, want := r.Form.Get("subject_token"), "fake_token"; got != want {
				t.Errorf("got %q, want %q", got, want)
			}

			resp := &struct {
				AccessToken string `json:"access_token"`
				ExpiresIn   int    `json:"expires_in"`
			}{
				AccessToken: "a_fake_token_sts",
				ExpiresIn:   60,
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else if r.URL.Path == "/impersonate" {
			if want := "a_fake_token_sts"; !strings.Contains(r.Header.Get("Authorization"), want) {
				t.Errorf("missing sts token: got %q, want %q", r.Header.Get("Authorization"), want)
			}

			resp := &struct {
				AccessToken string `json:"accessToken"`
				ExpireTime  string `json:"expireTime"`
			}{
				AccessToken: "a_fake_token",
				ExpireTime:  "2006-01-02T15:04:05Z",
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else {
			t.Errorf("unexpected call to %q", r.URL.Path)
		}
	}))
	opts.ServiceAccountImpersonationURL = ts.URL + "/impersonate"
	opts.TokenURL = ts.URL + "/sts"

	oldNow := iexacc.Now
	defer func() {
		iexacc.Now = oldNow
	}()
	iexacc.Now = func() time.Time {
		return defaultTime
	}

	creds, err := NewCredentials(opts)
	if err != nil {
		t.Fatalf("NewCredentials() = %v", err)
	}
	if _, err := creds.Token(context.Background()); err != nil {
		t.Fatalf("creds.Token() = %v", err)
	}
}

func TestNewCredentials_CredentialSourceURL(t *testing.T) {
	opts := &Options{
		Audience:         "//iam.googleapis.com/projects/$PROJECT_NUMBER/locations/global/workloadIdentityPools/$POOL_ID/providers/$PROVIDER_ID",
		SubjectTokenType: "urn:ietf:params:oauth:token-type:jwt",
		CredentialSource: &CredentialSource{
			Format: &Format{
				Type:                  "json",
				SubjectTokenFieldName: "id_token",
			},
		},
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer r.Body.Close()
		if r.URL.Path == "/token" {
			resp := &struct {
				Token string `json:"id_token"`
			}{
				Token: "a_fake_token_base",
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else if r.URL.Path == "/sts" {
			r.ParseForm()
			if got, want := r.Form.Get("subject_token"), "a_fake_token_base"; got != want {
				t.Errorf("got %q, want %q", got, want)
			}

			resp := &struct {
				AccessToken string `json:"access_token"`
				ExpiresIn   int    `json:"expires_in"`
			}{
				AccessToken: "a_fake_token_sts",
				ExpiresIn:   60,
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else if r.URL.Path == "/impersonate" {
			if want := "a_fake_token_sts"; !strings.Contains(r.Header.Get("Authorization"), want) {
				t.Errorf("missing sts token: got %q, want %q", r.Header.Get("Authorization"), want)
			}

			resp := &struct {
				AccessToken string `json:"accessToken"`
				ExpireTime  string `json:"expireTime"`
			}{
				AccessToken: "a_fake_token",
				ExpireTime:  "2006-01-02T15:04:05Z",
			}
			if err := json.NewEncoder(w).Encode(&resp); err != nil {
				t.Error(err)
			}
		} else {
			t.Errorf("unexpected call to %q", r.URL.Path)
		}
	}))
	opts.ServiceAccountImpersonationURL = ts.URL + "/impersonate"
	opts.TokenURL = ts.URL + "/sts"
	opts.CredentialSource.URL = ts.URL + "/token"

	creds, err := NewCredentials(opts)
	if err != nil {
		t.Fatalf("NewCredentials() = %v", err)
	}
	if _, err := creds.Token(context.Background()); err != nil {
		t.Fatalf("creds.Token() = %v", err)
	}
}

type fakeAwsCredsProvider struct {
	credsErr  error
	regionErr error
	awsRegion string
	creds     *AwsSecurityCredentials
}

func (acp fakeAwsCredsProvider) AwsRegion(ctx context.Context, opts *RequestOptions) (string, error) {
	if acp.regionErr != nil {
		return "", acp.regionErr
	}
	return acp.awsRegion, nil
}

func (acp fakeAwsCredsProvider) AwsSecurityCredentials(ctx context.Context, opts *RequestOptions) (*AwsSecurityCredentials, error) {
	if acp.credsErr != nil {
		return nil, acp.credsErr
	}
	return acp.creds, nil
}

type fakeSubjectTokenProvider struct {
	err          error
	subjectToken string
}

func (p fakeSubjectTokenProvider) SubjectToken(ctx context.Context, options *RequestOptions) (string, error) {
	if p.err != nil {
		return "", p.err
	}
	return p.subjectToken, nil
}
