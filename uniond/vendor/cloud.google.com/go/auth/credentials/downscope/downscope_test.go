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

package downscope

import (
	"context"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"cloud.google.com/go/auth"
)

var (
	standardReqBody  = "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&options=%7B%22accessBoundary%22%3A%7B%22accessBoundaryRules%22%3A%5B%7B%22availableResource%22%3A%22test1%22%2C%22availablePermissions%22%3A%5B%22Perm1%22%2C%22Perm2%22%5D%7D%5D%7D%7D&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&subject_token=token_base&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token"
	standardRespBody = `{"access_token":"fake_token","expires_in":42,"token_type":"Bearer"}`
)

func staticCredentials(tok string) *auth.Credentials {
	return auth.NewCredentials(&auth.CredentialsOptions{
		TokenProvider: staticTokenProvider(tok),
	})
}

type staticTokenProvider string

func (s staticTokenProvider) Token(context.Context) (*auth.Token, error) {
	return &auth.Token{Value: string(s)}, nil
}

func TestNewTokenProvider(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		if r.URL.String() != "/" {
			t.Errorf("Unexpected request URL, %v is found", r.URL)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("Failed to read request body: %v", err)
		}
		if got, want := string(body), standardReqBody; got != want {
			t.Errorf("Unexpected exchange payload: got %v but want %v,", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(standardRespBody))

	}))
	defer ts.Close()
	creds, err := NewCredentials(&Options{
		Credentials: staticCredentials("token_base"),
		Rules: []AccessBoundaryRule{
			{
				AvailableResource:    "test1",
				AvailablePermissions: []string{"Perm1", "Perm2"},
			},
		},
	})
	if err != nil {
		t.Fatalf("NewTokenProvider() = %v", err)
	}
	// Replace the default STS endpoint on the TokenProvider with the test server URL.
	creds.TokenProvider.(*downscopedTokenProvider).identityBindingEndpoint = ts.URL

	tok, err := creds.Token(context.Background())
	if err != nil {
		t.Fatalf("Token failed with error: %v", err)
	}
	if want := "fake_token"; tok.Value != want {
		t.Fatalf("got %v, want %v", tok.Value, want)
	}
}

func TestNewCredentials_Validations(t *testing.T) {
	tests := []struct {
		name string
		opts *Options
	}{
		{
			name: "no opts",
			opts: nil,
		},
		{
			name: "no provider",
			opts: &Options{},
		},
		{
			name: "no rules",
			opts: &Options{
				Credentials: staticCredentials("token_base"),
			},
		},
		{
			name: "too many rules",
			opts: &Options{
				Credentials: staticCredentials("token_base"),
				Rules:       []AccessBoundaryRule{{}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}},
			},
		},
		{
			name: "no resource",
			opts: &Options{
				Credentials: staticCredentials("token_base"),
				Rules:       []AccessBoundaryRule{{}},
			},
		},
		{
			name: "no perm",
			opts: &Options{
				Credentials: staticCredentials("token_base"),
				Rules: []AccessBoundaryRule{{
					AvailableResource: "resource",
				}},
			},
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if _, err := NewCredentials(test.opts); err == nil {
				t.Fatal("want non-nil err")
			}
		})
	}
}

func TestOptions_UniverseDomain(t *testing.T) {
	tests := []struct {
		universeDomain string
		want           string
	}{
		{"", "https://sts.googleapis.com/v1/token"},
		{"googleapis.com", "https://sts.googleapis.com/v1/token"},
		{"example.com", "https://sts.example.com/v1/token"},
	}
	for _, tt := range tests {
		c := Options{
			UniverseDomain: tt.universeDomain,
		}
		if got := c.identityBindingEndpoint(); got != tt.want {
			t.Errorf("got %q, want %q", got, tt.want)
		}
	}
}
