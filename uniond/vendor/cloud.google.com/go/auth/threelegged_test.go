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

package auth

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"
	"time"
)

const day = 24 * time.Hour

func newOpts(url string) *Options3LO {
	return &Options3LO{
		ClientID:     "CLIENT_ID",
		ClientSecret: "CLIENT_SECRET",
		RedirectURL:  "REDIRECT_URL",
		Scopes:       []string{"scope1", "scope2"},
		AuthURL:      url + "/auth",
		TokenURL:     url + "/token",
		AuthStyle:    StyleInHeader,
		RefreshToken: "OLD_REFRESH_TOKEN",
	}
}

func Test3LO_URLUnsafe(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.Header.Get("Authorization"), "Basic Q0xJRU5UX0lEJTNGJTNGOkNMSUVOVF9TRUNSRVQlM0YlM0Y="; got != want {
			t.Errorf("Authorization header = %q; want %q", got, want)
		}

		w.Header().Set("Content-Type", "application/x-www-form-urlencoded")
		w.Write([]byte("access_token=90d64460d14870c08c81352a05dedd3465940a7c&scope=user&token_type=bearer"))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	conf.ClientID = "CLIENT_ID??"
	conf.ClientSecret = "CLIENT_SECRET??"
	_, _, err := conf.exchange(context.Background(), "exchange-code")
	if err != nil {
		t.Error(err)
	}
}

func Test3LO_StandardExchange(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.String() != "/token" {
			t.Errorf("Unexpected exchange request URL %q", r.URL)
		}
		headerAuth := r.Header.Get("Authorization")
		if want := "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ="; headerAuth != want {
			t.Errorf("Unexpected authorization header %q, want %q", headerAuth, want)
		}
		headerContentType := r.Header.Get("Content-Type")
		if headerContentType != "application/x-www-form-urlencoded" {
			t.Errorf("Unexpected Content-Type header %q", headerContentType)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Errorf("Failed reading request body: %s.", err)
		}
		if string(body) != "code=exchange-code&grant_type=authorization_code&redirect_uri=REDIRECT_URL" {
			t.Errorf("Unexpected exchange payload; got %q", body)
		}
		w.Header().Set("Content-Type", "application/x-www-form-urlencoded")
		w.Write([]byte("access_token=90d64460d14870c08c81352a05dedd3465940a7c&scope=user&token_type=bearer"))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	tok, _, err := conf.exchange(context.Background(), "exchange-code")
	if err != nil {
		t.Error(err)
	}
	if !tok.IsValid() {
		t.Fatalf("Token invalid. Got: %#v", tok)
	}
	if tok.Value != "90d64460d14870c08c81352a05dedd3465940a7c" {
		t.Errorf("Unexpected access token, %#v.", tok.Value)
	}
	if tok.Type != "bearer" {
		t.Errorf("Unexpected token type, %#v.", tok.Type)
	}
	scope := tok.Metadata["scope"].([]string)
	if scope[0] != "user" {
		t.Errorf("Unexpected value for scope: %v", scope)
	}
}

func Test3LO_ExchangeCustomParams(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.String() != "/token" {
			t.Errorf("Unexpected exchange request URL, %v is found.", r.URL)
		}
		headerAuth := r.Header.Get("Authorization")
		if headerAuth != "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=" {
			t.Errorf("Unexpected authorization header, %v is found.", headerAuth)
		}
		headerContentType := r.Header.Get("Content-Type")
		if headerContentType != "application/x-www-form-urlencoded" {
			t.Errorf("Unexpected Content-Type header, %v is found.", headerContentType)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Errorf("Failed reading request body: %s.", err)
		}
		if string(body) != "code=exchange-code&foo=bar&grant_type=authorization_code&redirect_uri=REDIRECT_URL" {
			t.Errorf("Unexpected exchange payload, %v is found.", string(body))
		}
		w.Header().Set("Content-Type", "application/x-www-form-urlencoded")
		w.Write([]byte("access_token=90d64460d14870c08c81352a05dedd3465940a7c&scope=user&token_type=bearer"))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	conf.URLParams = url.Values{}
	conf.URLParams.Set("foo", "bar")

	tok, _, err := conf.exchange(context.Background(), "exchange-code")
	if err != nil {
		t.Error(err)
	}
	if !tok.IsValid() {
		t.Fatalf("Token invalid. Got: %#v", tok)
	}
	if tok.Value != "90d64460d14870c08c81352a05dedd3465940a7c" {
		t.Errorf("Unexpected access token, %#v.", tok.Value)
	}
	if tok.Type != "bearer" {
		t.Errorf("Unexpected token type, %#v.", tok.Type)
	}
	scope := tok.Metadata["scope"].([]string)
	if scope[0] != "user" {
		t.Errorf("Unexpected value for scope: %v", scope)
	}
}

func Test3LO_ExchangeJSONResponse(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.String() != "/token" {
			t.Errorf("Unexpected exchange request URL, %v is found.", r.URL)
		}
		headerAuth := r.Header.Get("Authorization")
		if headerAuth != "Basic Q0xJRU5UX0lEOkNMSUVOVF9TRUNSRVQ=" {
			t.Errorf("Unexpected authorization header, %v is found.", headerAuth)
		}
		headerContentType := r.Header.Get("Content-Type")
		if headerContentType != "application/x-www-form-urlencoded" {
			t.Errorf("Unexpected Content-Type header, %v is found.", headerContentType)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Errorf("Failed reading request body: %s.", err)
		}
		if string(body) != "code=exchange-code&grant_type=authorization_code&redirect_uri=REDIRECT_URL" {
			t.Errorf("Unexpected exchange payload, %v is found.", string(body))
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token": "90d64460d14870c08c81352a05dedd3465940a7c", "scope": "user", "token_type": "bearer", "expires_in": 86400}`))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	tok, _, err := conf.exchange(context.Background(), "exchange-code")
	if err != nil {
		t.Error(err)
	}
	if !tok.IsValid() {
		t.Fatalf("Token invalid. Got: %#v", tok)
	}
	if tok.Value != "90d64460d14870c08c81352a05dedd3465940a7c" {
		t.Errorf("Unexpected access token, %#v.", tok.Value)
	}
	if tok.Type != "bearer" {
		t.Errorf("Unexpected token type, %#v.", tok.Type)
	}
	scope := tok.Metadata["scope"].(string)
	if scope != "user" {
		t.Errorf("Unexpected value for scope: %v", scope)
	}
	expiresIn := tok.Metadata["expires_in"]
	if expiresIn != float64(86400) {
		t.Errorf("Unexpected non-numeric value for expires_in: %v", expiresIn)
	}
}

func Test3LO_ExchangeJSONResponseExpiry(t *testing.T) {
	seconds := int32(day.Seconds())
	for _, c := range []struct {
		name        string
		expires     string
		want        bool
		nullExpires bool
	}{
		{"normal", fmt.Sprintf(`"expires_in": %d`, seconds), true, false},
		{"null", `"expires_in": null`, true, true},
		{"wrong_type", `"expires_in": false`, false, false},
		{"wrong_type2", `"expires_in": {}`, false, false},
		{"wrong_value", `"expires_in": "zzz"`, false, false},
	} {
		t.Run(c.name, func(t *testing.T) {
			test3LOExchangeJSONResponseExpiry(t, c.expires, c.want, c.nullExpires)
		})
	}
}

func test3LOExchangeJSONResponseExpiry(t *testing.T, exp string, want, nullExpires bool) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(fmt.Sprintf(`{"access_token": "90d", "scope": "user", "token_type": "bearer", %s}`, exp)))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	t1 := time.Now().Add(day)
	tok, _, err := conf.exchange(context.Background(), "exchange-code")
	t2 := t1.Add(day)

	if got := (err == nil); got != want {
		if want {
			t.Errorf("unexpected error: got %v", err)
		} else {
			t.Errorf("unexpected success")
		}
	}
	if !want {
		return
	}
	if !tok.IsValid() {
		t.Fatalf("Token invalid. Got: %#v", tok)
	}
	expiry := tok.Expiry

	if nullExpires && expiry.IsZero() {
		return
	}
	if expiry.Before(t1) || expiry.After(t2) {
		t.Errorf("Unexpected value for Expiry: %v (should be between %v and %v)", expiry, t1, t2)
	}
}

func Test3LO_ExchangeBadResponse(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"scope": "user", "token_type": "bearer"}`))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	_, _, err := conf.exchange(context.Background(), "code")
	if err == nil {
		t.Error("expected error from missing access_token")
	}
}

func Test3LO_ExchangeBadResponseType(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token":123,  "scope": "user", "token_type": "bearer"}`))
	}))
	defer ts.Close()
	conf := newOpts(ts.URL)
	_, _, err := conf.exchange(context.Background(), "exchange-code")
	if err == nil {
		t.Error("expected error from non-string access_token")
	}
}

func Test3LO_RefreshTokenReplacement(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token":"ACCESS_TOKEN",  "scope": "user", "token_type": "bearer", "refresh_token": "NEW_REFRESH_TOKEN"}`))
	}))
	defer ts.Close()
	opts := newOpts(ts.URL)
	tp, err := New3LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tp.Token(context.Background()); err != nil {
		t.Errorf("got err = %v; want none", err)
		return
	}
	innerTP := tp.(*cachedTokenProvider).tp.(*tokenProvider3LO)
	if want := "NEW_REFRESH_TOKEN"; innerTP.refreshToken != want {
		t.Errorf("RefreshToken = %q; want %q", innerTP.refreshToken, want)
	}
}

func Test3LO_RefreshTokenPreservation(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token":"ACCESS_TOKEN",  "scope": "user", "token_type": "bearer"}`))
	}))
	defer ts.Close()
	opts := newOpts(ts.URL)
	const oldRefreshToken = "OLD_REFRESH_TOKEN"
	tp, err := New3LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := tp.Token(context.Background()); err != nil {
		t.Errorf("got err = %v; want none", err)
		return
	}
	innerTP := tp.(*cachedTokenProvider).tp.(*tokenProvider3LO)
	if innerTP.refreshToken != oldRefreshToken {
		t.Errorf("RefreshToken = %q; want %q", innerTP.refreshToken, oldRefreshToken)
	}
}

func Test3LO_AuthHandlerExchangeSuccess(t *testing.T) {
	authhandler := func(authCodeURL string) (string, string, error) {
		if authCodeURL == "testAuthCodeURL?client_id=testClientID&response_type=code&scope=pubsub&state=testState" {
			return "testCode", "testState", nil
		}
		return "", "", fmt.Errorf("invalid authCodeURL: %q", authCodeURL)
	}

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.ParseForm()
		if r.Form.Get("code") == "testCode" {
			w.Header().Set("Content-Type", "application/json")
			w.Write([]byte(`{
				"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
				"scope": "pubsub",
				"token_type": "bearer",
				"expires_in": 3600
			}`))
		}
	}))
	defer ts.Close()

	opts := &Options3LO{
		ClientID:  "testClientID",
		Scopes:    []string{"pubsub"},
		AuthURL:   "testAuthCodeURL",
		TokenURL:  ts.URL,
		AuthStyle: StyleInHeader,
		AuthHandlerOpts: &AuthorizationHandlerOptions{
			State:   "testState",
			Handler: authhandler,
		},
	}

	tp, err := New3LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if !tok.IsValid() {
		t.Errorf("got invalid token: %v", tok)
	}
	if got, want := tok.Value, "90d64460d14870c08c81352a05dedd3465940a7c"; got != want {
		t.Errorf("access token = %q; want %q", got, want)
	}
	if got, want := tok.Type, "bearer"; got != want {
		t.Errorf("token type = %q; want %q", got, want)
	}
	if got := tok.Expiry.IsZero(); got {
		t.Errorf("token expiry is zero = %v, want false", got)
	}
	scope := tok.Metadata["scope"].(string)
	if got, want := scope, "pubsub"; got != want {
		t.Errorf("scope = %q; want %q", got, want)
	}
}

func Test3LO_AuthHandlerExchangeStateMismatch(t *testing.T) {
	authhandler := func(authCodeURL string) (string, string, error) {
		return "testCode", "testStateMismatch", nil
	}

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
			"scope": "pubsub",
			"token_type": "bearer",
			"expires_in": 3600
		}`))
	}))
	defer ts.Close()

	opts := &Options3LO{
		ClientID:  "testClientID",
		Scopes:    []string{"pubsub"},
		AuthURL:   "testAuthCodeURL",
		TokenURL:  ts.URL,
		AuthStyle: StyleInParams,
		AuthHandlerOpts: &AuthorizationHandlerOptions{
			State:   "testState",
			Handler: authhandler,
		},
	}
	tp, err := New3LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	_, err = tp.Token(context.Background())
	if wantErr := "auth: state mismatch in 3-legged-OAuth flow"; err == nil || err.Error() != wantErr {
		t.Errorf("err = %q; want %q", err, wantErr)
	}
}

func Test3LO_PKCEExchangeWithSuccess(t *testing.T) {
	authhandler := func(authCodeURL string) (string, string, error) {
		if authCodeURL == "testAuthCodeURL?client_id=testClientID&code_challenge=codeChallenge&code_challenge_method=plain&response_type=code&scope=pubsub&state=testState" {
			return "testCode", "testState", nil
		}
		return "", "", fmt.Errorf("invalid authCodeURL: %q", authCodeURL)
	}

	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.ParseForm()
		if r.Form.Get("code") == "testCode" && r.Form.Get("code_verifier") == "codeChallenge" {
			w.Header().Set("Content-Type", "application/json")
			w.Write([]byte(`{
				"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
				"scope": "pubsub",
				"token_type": "bearer",
				"expires_in": 3600
			}`))
		}
	}))
	defer ts.Close()

	opts := &Options3LO{
		ClientID:  "testClientID",
		Scopes:    []string{"pubsub"},
		AuthURL:   "testAuthCodeURL",
		TokenURL:  ts.URL,
		AuthStyle: StyleInParams,
		AuthHandlerOpts: &AuthorizationHandlerOptions{
			State:   "testState",
			Handler: authhandler,
			PKCEOpts: &PKCEOptions{
				Challenge:       "codeChallenge",
				ChallengeMethod: "plain",
				Verifier:        "codeChallenge",
			},
		},
	}

	tp, err := New3LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if !tok.IsValid() {
		t.Errorf("got invalid token: %v", tok)
	}
	if got, want := tok.Value, "90d64460d14870c08c81352a05dedd3465940a7c"; got != want {
		t.Errorf("access token = %q; want %q", got, want)
	}
	if got, want := tok.Type, "bearer"; got != want {
		t.Errorf("token type = %q; want %q", got, want)
	}
	if got := tok.Expiry.IsZero(); got {
		t.Errorf("token expiry is zero = %v, want false", got)
	}
	scope := tok.Metadata["scope"].(string)
	if got, want := scope, "pubsub"; got != want {
		t.Errorf("scope = %q; want %q", got, want)
	}
}

func Test3LO_Validate(t *testing.T) {
	tests := []struct {
		name string
		opts *Options3LO
	}{
		{
			name: "missing options",
		},
		{
			name: "missing client ID",
			opts: &Options3LO{
				ClientSecret: "client_secret",
				AuthURL:      "auth_url",
				TokenURL:     "token_url",
				AuthStyle:    StyleInHeader,
				RefreshToken: "refreshing",
			},
		},
		{
			name: "missing client secret",
			opts: &Options3LO{
				ClientID:     "client_id",
				AuthURL:      "auth_url",
				TokenURL:     "token_url",
				AuthStyle:    StyleInHeader,
				RefreshToken: "refreshing",
			},
		},
		{
			name: "missing auth URL",
			opts: &Options3LO{
				ClientID:     "client_id",
				ClientSecret: "client_secret",
				TokenURL:     "token_url",
				AuthStyle:    StyleInHeader,
				RefreshToken: "refreshing",
			},
		},
		{
			name: "missing token URL",
			opts: &Options3LO{
				ClientID:     "client_id",
				ClientSecret: "client_secret",
				AuthURL:      "auth_url",
				AuthStyle:    StyleInHeader,
				RefreshToken: "refreshing",
			},
		},
		{
			name: "missing auth style",
			opts: &Options3LO{
				ClientID:     "client_id",
				ClientSecret: "client_secret",
				AuthURL:      "auth_url",
				TokenURL:     "token_url",
				RefreshToken: "refreshing",
			},
		},
		{
			name: "missing refresh token",
			opts: &Options3LO{
				ClientID:     "client_id",
				ClientSecret: "client_secret",
				AuthURL:      "auth_url",
				TokenURL:     "token_url",
				AuthStyle:    StyleInHeader,
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if _, err := New3LOTokenProvider(tt.opts); err == nil {
				t.Error("got nil, want an error")
			}
		})
	}
}
