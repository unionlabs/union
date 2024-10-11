// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package externalaccountauthorizeduser

import (
	"context"
	"encoding/json"
	"errors"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"golang.org/x/oauth2"
	"golang.org/x/oauth2/google/internal/stsexchange"
)

const expiryDelta = 10 * time.Second

var (
	expiry    = time.Unix(234852, 0)
	testNow   = func() time.Time { return expiry }
	testValid = func(t oauth2.Token) bool {
		return t.AccessToken != "" && !t.Expiry.Round(0).Add(-expiryDelta).Before(testNow())
	}
)

type testRefreshTokenServer struct {
	URL             string
	Authorization   string
	ContentType     string
	Body            string
	ResponsePayload *stsexchange.Response
	Response        string
	server          *httptest.Server
}

func TestExernalAccountAuthorizedUser_JustToken(t *testing.T) {
	config := &Config{
		Token:  "AAAAAAA",
		Expiry: now().Add(time.Hour),
	}
	ts, err := config.TokenSource(context.Background())
	if err != nil {
		t.Fatalf("Error getting token source: %v", err)
	}

	token, err := ts.Token()
	if err != nil {
		t.Fatalf("Error retrieving Token: %v", err)
	}
	if got, want := token.AccessToken, "AAAAAAA"; got != want {
		t.Fatalf("Unexpected access token, got %v, want %v", got, want)
	}
}

func TestExernalAccountAuthorizedUser_TokenRefreshWithRefreshTokenInRespondse(t *testing.T) {
	server := &testRefreshTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.Response{
			ExpiresIn:    3600,
			AccessToken:  "AAAAAAA",
			RefreshToken: "CCCCCCC",
		},
	}

	url, err := server.run(t)
	if err != nil {
		t.Fatalf("Error starting server")
	}
	defer server.close(t)

	config := &Config{
		RefreshToken: "BBBBBBBBB",
		TokenURL:     url,
		ClientID:     "CLIENT_ID",
		ClientSecret: "CLIENT_SECRET",
	}
	ts, err := config.TokenSource(context.Background())
	if err != nil {
		t.Fatalf("Error getting token source: %v", err)
	}

	token, err := ts.Token()
	if err != nil {
		t.Fatalf("Error retrieving Token: %v", err)
	}
	if got, want := token.AccessToken, "AAAAAAA"; got != want {
		t.Fatalf("Unexpected access token, got %v, want %v", got, want)
	}
	if config.RefreshToken != "CCCCCCC" {
		t.Fatalf("Refresh token not updated")
	}
}

func TestExernalAccountAuthorizedUser_MinimumFieldsRequiredForRefresh(t *testing.T) {
	server := &testRefreshTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.Response{
			ExpiresIn:   3600,
			AccessToken: "AAAAAAA",
		},
	}

	url, err := server.run(t)
	if err != nil {
		t.Fatalf("Error starting server")
	}
	defer server.close(t)

	config := &Config{
		RefreshToken: "BBBBBBBBB",
		TokenURL:     url,
		ClientID:     "CLIENT_ID",
		ClientSecret: "CLIENT_SECRET",
	}
	ts, err := config.TokenSource(context.Background())
	if err != nil {
		t.Fatalf("Error getting token source: %v", err)
	}

	token, err := ts.Token()
	if err != nil {
		t.Fatalf("Error retrieving Token: %v", err)
	}
	if got, want := token.AccessToken, "AAAAAAA"; got != want {
		t.Fatalf("Unexpected access token, got %v, want %v", got, want)
	}
}

func TestExternalAccountAuthorizedUser_MissingRefreshFields(t *testing.T) {
	server := &testRefreshTokenServer{
		URL:           "/",
		Authorization: "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=",
		ContentType:   "application/x-www-form-urlencoded",
		Body:          "grant_type=refresh_token&refresh_token=BBBBBBBBB",
		ResponsePayload: &stsexchange.Response{
			ExpiresIn:   3600,
			AccessToken: "AAAAAAA",
		},
	}

	url, err := server.run(t)
	if err != nil {
		t.Fatalf("Error starting server")
	}
	defer server.close(t)
	testCases := []struct {
		name   string
		config Config
	}{
		{
			name:   "empty config",
			config: Config{},
		},
		{
			name: "missing refresh token",
			config: Config{
				TokenURL:     url,
				ClientID:     "CLIENT_ID",
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing token url",
			config: Config{
				RefreshToken: "BBBBBBBBB",
				ClientID:     "CLIENT_ID",
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing client id",
			config: Config{
				RefreshToken: "BBBBBBBBB",
				TokenURL:     url,
				ClientSecret: "CLIENT_SECRET",
			},
		},
		{
			name: "missing client secrect",
			config: Config{
				RefreshToken: "BBBBBBBBB",
				TokenURL:     url,
				ClientID:     "CLIENT_ID",
			},
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {

			expectErrMsg := "oauth2/google: Token should be created with fields to make it valid (`token` and `expiry`), or fields to allow it to refresh (`refresh_token`, `token_url`, `client_id`, `client_secret`)."
			_, err := tc.config.TokenSource((context.Background()))
			if err == nil {
				t.Fatalf("Expected error, but received none")
			}
			if got := err.Error(); got != expectErrMsg {
				t.Fatalf("Unexpected error, got %v, want %v", got, expectErrMsg)
			}
		})
	}
}

func (trts *testRefreshTokenServer) run(t *testing.T) (string, error) {
	t.Helper()
	if trts.server != nil {
		return "", errors.New("Server is already running")
	}
	trts.server = httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.URL.String(), trts.URL; got != want {
			t.Errorf("URL.String(): got %v but want %v", got, want)
		}
		headerAuth := r.Header.Get("Authorization")
		if got, want := headerAuth, trts.Authorization; got != want {
			t.Errorf("got %v but want %v", got, want)
		}
		headerContentType := r.Header.Get("Content-Type")
		if got, want := headerContentType, trts.ContentType; got != want {
			t.Errorf("got %v but want %v", got, want)
		}
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("Failed reading request body: %s.", err)
		}
		if got, want := string(body), trts.Body; got != want {
			t.Errorf("Unexpected exchange payload: got %v but want %v", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		if trts.ResponsePayload != nil {
			content, err := json.Marshal(trts.ResponsePayload)
			if err != nil {
				t.Fatalf("unable to marshall response JSON")
			}
			w.Write(content)
		} else {
			w.Write([]byte(trts.Response))
		}
	}))
	return trts.server.URL, nil
}

func (trts *testRefreshTokenServer) close(t *testing.T) error {
	t.Helper()
	if trts.server == nil {
		return errors.New("No server is running")
	}
	trts.server.Close()
	trts.server = nil
	return nil
}
