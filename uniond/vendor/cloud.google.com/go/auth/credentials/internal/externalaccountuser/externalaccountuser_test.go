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

package externalaccountuser

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"

	"cloud.google.com/go/auth/credentials/internal/stsexchange"
	"cloud.google.com/go/auth/internal"
)

type testTokenServer struct {
	URL             string
	Authorization   string
	ContentType     string
	Body            string
	ResponsePayload *stsexchange.TokenResponse
	Response        string
	server          *httptest.Server
}

func TestExernalAccountAuthorizedUser_TokenRefreshWithRefreshTokenInResponse(t *testing.T) {
	s := &testTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.TokenResponse{
			ExpiresIn:    3600,
			AccessToken:  "AAAAAAA",
			RefreshToken: "CCCCCCC",
		},
	}

	s.startTestServer(t)
	defer s.server.Close()

	opts := &Options{
		RefreshToken: "BBBBBBBBB",
		TokenURL:     s.server.URL,
		ClientID:     "CLIENT_ID",
		ClientSecret: "CLIENT_SECRET",
		Client:       internal.CloneDefaultClient(),
	}
	tp, err := NewTokenProvider(opts)
	if err != nil {
		t.Fatalf("NewTokenProvider() =  %v", err)
	}

	token, err := tp.Token(context.Background())
	if err != nil {
		t.Fatalf("Token() = %v", err)
	}
	if got, want := token.Value, "AAAAAAA"; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
	if got, want := opts.RefreshToken, "CCCCCCC"; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
}

func TestExernalAccountAuthorizedUser_MinimumFieldsRequiredForRefresh(t *testing.T) {
	s := &testTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.TokenResponse{
			ExpiresIn:   3600,
			AccessToken: "AAAAAAA",
		},
	}

	s.startTestServer(t)
	defer s.server.Close()

	opts := &Options{
		RefreshToken: "BBBBBBBBB",
		TokenURL:     s.server.URL,
		ClientID:     "CLIENT_ID",
		ClientSecret: "CLIENT_SECRET",
		Client:       internal.CloneDefaultClient(),
	}
	ts, err := NewTokenProvider(opts)
	if err != nil {
		t.Fatalf("NewTokenProvider() = %v", err)
	}

	token, err := ts.Token(context.Background())
	if err != nil {
		t.Fatalf("Token() = %v", err)
	}
	if got, want := token.Value, "AAAAAAA"; got != want {
		t.Fatalf("got %v, want %v", got, want)
	}
}

func TestExternalAccountAuthorizedUser_MissingRefreshFields(t *testing.T) {
	s := &testTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.TokenResponse{
			ExpiresIn:   3600,
			AccessToken: "AAAAAAA",
		},
	}

	s.startTestServer(t)
	defer s.server.Close()
	testCases := []struct {
		name string
		opts *Options
	}{
		{
			name: "empty config",
			opts: &Options{},
		},
		{
			name: "missing refresh token",
			opts: &Options{
				TokenURL:     s.server.URL,
				ClientID:     "CLIENT_ID",
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing token url",
			opts: &Options{
				RefreshToken: "BBBBBBBBB",
				ClientID:     "CLIENT_ID",
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing client id",
			opts: &Options{
				RefreshToken: "BBBBBBBBB",
				TokenURL:     s.server.URL,
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing client secrect",
			opts: &Options{
				RefreshToken: "BBBBBBBBB",
				TokenURL:     s.server.URL,
				ClientID:     "CLIENT_ID",
			},
		},
	}
	for _, tt := range testCases {
		t.Run(tt.name, func(t *testing.T) {
			if _, err := NewTokenProvider(tt.opts); err == nil {
				t.Fatalf("got nil, want an error")
			}
		})
	}
}

func (s *testTokenServer) startTestServer(t *testing.T) {
	t.Helper()
	s.server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.URL.String(), s.URL; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		headerAuth := r.Header.Get("Authorization")
		if got, want := headerAuth, s.Authorization; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		headerContentType := r.Header.Get("Content-Type")
		if got, want := headerContentType, s.ContentType; got != want {
			t.Errorf("got %v. want %v", got, want)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Error(err)
		}
		if got, want := string(body), s.Body; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		if s.ResponsePayload != nil {
			content, err := json.Marshal(s.ResponsePayload)
			if err != nil {
				t.Error(err)
			}
			w.Write(content)
		} else {
			w.Write([]byte(s.Response))
		}
	}))
}
