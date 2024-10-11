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
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/internal"
)

type mockProvider string

func (tp mockProvider) Token(context.Context) (*auth.Token, error) {
	return &auth.Token{
		Value: string(tp),
	}, nil
}

func TestNewImpersonatedTokenProvider_Validation(t *testing.T) {
	tests := []struct {
		name string
		opt  *Options
	}{
		{
			name: "missing source creds",
			opt: &Options{
				URL: "some-url",
			},
		},
		{
			name: "missing url",
			opt: &Options{
				Tp: &Options{},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			_, err := NewTokenProvider(tt.opt)
			if err == nil {
				t.Errorf("got nil, want an error")
			}
		})
	}
}

func TestNewImpersonatedTokenProvider(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.Header.Get("Authorization"), "Bearer fake_token_base"; got != want {
			t.Errorf("got %q; want %q", got, want)
		}
		resp := &struct {
			AccessToken string `json:"accessToken"`
			ExpireTime  string `json:"expireTime"`
		}{
			AccessToken: "a_fake_token",
			ExpireTime:  "2006-01-02T15:04:05Z",
		}
		if err := json.NewEncoder(w).Encode(&resp); err != nil {
			t.Fatal(err)
		}
	}))

	creds, err := NewTokenProvider(&Options{
		Tp:        mockProvider("fake_token_base"),
		URL:       ts.URL,
		Delegates: []string{"sa1@developer.gserviceaccount.com", "sa2@developer.gserviceaccount.com"},
		Scopes:    []string{"https://www.googleapis.com/auth/cloud-platform"},
		Client:    internal.CloneDefaultClient(),
	})
	if err != nil {
		t.Fatal(err)
	}
	tok, err := creds.Token(context.Background())
	if err != nil {
		t.Fatalf("creds.Token() = %v", err)
	}
	if want := "a_fake_token"; tok.Value != want {
		t.Fatalf("got %q, want %q", tok.Value, want)
	}
	if want := internal.TokenTypeBearer; tok.Type != want {
		t.Fatalf("got %q, want %q", tok.Type, want)
	}
}
