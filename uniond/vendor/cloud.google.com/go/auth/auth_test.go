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
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/auth/internal/jwt"
	"github.com/google/go-cmp/cmp"
)

var fakePrivateKey = []byte(`-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAx4fm7dngEmOULNmAs1IGZ9Apfzh+BkaQ1dzkmbUgpcoghucE
DZRnAGd2aPyB6skGMXUytWQvNYav0WTR00wFtX1ohWTfv68HGXJ8QXCpyoSKSSFY
fuP9X36wBSkSX9J5DVgiuzD5VBdzUISSmapjKm+DcbRALjz6OUIPEWi1Tjl6p5RK
1w41qdbmt7E5/kGhKLDuT7+M83g4VWhgIvaAXtnhklDAggilPPa8ZJ1IFe31lNlr
k4DRk38nc6sEutdf3RL7QoH7FBusI7uXV03DC6dwN1kP4GE7bjJhcRb/7jYt7CQ9
/E9Exz3c0yAp0yrTg0Fwh+qxfH9dKwN52S7SBwIDAQABAoIBAQCaCs26K07WY5Jt
3a2Cw3y2gPrIgTCqX6hJs7O5ByEhXZ8nBwsWANBUe4vrGaajQHdLj5OKfsIDrOvn
2NI1MqflqeAbu/kR32q3tq8/Rl+PPiwUsW3E6Pcf1orGMSNCXxeducF2iySySzh3
nSIhCG5uwJDWI7a4+9KiieFgK1pt/Iv30q1SQS8IEntTfXYwANQrfKUVMmVF9aIK
6/WZE2yd5+q3wVVIJ6jsmTzoDCX6QQkkJICIYwCkglmVy5AeTckOVwcXL0jqw5Kf
5/soZJQwLEyBoQq7Kbpa26QHq+CJONetPP8Ssy8MJJXBT+u/bSseMb3Zsr5cr43e
DJOhwsThAoGBAPY6rPKl2NT/K7XfRCGm1sbWjUQyDShscwuWJ5+kD0yudnT/ZEJ1
M3+KS/iOOAoHDdEDi9crRvMl0UfNa8MAcDKHflzxg2jg/QI+fTBjPP5GOX0lkZ9g
z6VePoVoQw2gpPFVNPPTxKfk27tEzbaffvOLGBEih0Kb7HTINkW8rIlzAoGBAM9y
1yr+jvfS1cGFtNU+Gotoihw2eMKtIqR03Yn3n0PK1nVCDKqwdUqCypz4+ml6cxRK
J8+Pfdh7D+ZJd4LEG6Y4QRDLuv5OA700tUoSHxMSNn3q9As4+T3MUyYxWKvTeu3U
f2NWP9ePU0lV8ttk7YlpVRaPQmc1qwooBA/z/8AdAoGAW9x0HWqmRICWTBnpjyxx
QGlW9rQ9mHEtUotIaRSJ6K/F3cxSGUEkX1a3FRnp6kPLcckC6NlqdNgNBd6rb2rA
cPl/uSkZP42Als+9YMoFPU/xrrDPbUhu72EDrj3Bllnyb168jKLa4VBOccUvggxr
Dm08I1hgYgdN5huzs7y6GeUCgYEAj+AZJSOJ6o1aXS6rfV3mMRve9bQ9yt8jcKXw
5HhOCEmMtaSKfnOF1Ziih34Sxsb7O2428DiX0mV/YHtBnPsAJidL0SdLWIapBzeg
KHArByIRkwE6IvJvwpGMdaex1PIGhx5i/3VZL9qiq/ElT05PhIb+UXgoWMabCp84
OgxDK20CgYAeaFo8BdQ7FmVX2+EEejF+8xSge6WVLtkaon8bqcn6P0O8lLypoOhd
mJAYH8WU+UAy9pecUnDZj14LAGNVmYcse8HFX71MoshnvCTFEPVo4rZxIAGwMpeJ
5jgQ3slYLpqrGlcbLgUXBUgzEO684Wk/UV9DFPlHALVqCfXQ9dpJPg==
-----END RSA PRIVATE KEY-----`)

func TestError_Temporary(t *testing.T) {
	tests := []struct {
		name string
		code int
		want bool
	}{
		{
			name: "temporary with 500",
			code: http.StatusInternalServerError,
			want: true,
		},
		{
			name: "temporary with 503",
			code: http.StatusServiceUnavailable,
			want: true,
		},
		{
			name: "temporary with 408",
			code: http.StatusRequestTimeout,
			want: true,
		},
		{
			name: "temporary with 429",
			code: http.StatusTooManyRequests,
			want: true,
		},
		{
			name: "temporary with 418",
			code: http.StatusTeapot,
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ae := &Error{
				Response: &http.Response{
					StatusCode: tt.code,
				},
			}
			if got := ae.Temporary(); got != tt.want {
				t.Errorf("Temporary() = %v; want %v", got, tt.want)
			}
		})
	}
}

func TestToken_isValidWithEarlyExpiry(t *testing.T) {
	now := time.Now()
	timeNow = func() time.Time { return now }
	defer func() { timeNow = time.Now }()

	cases := []struct {
		name   string
		tok    *Token
		expiry time.Duration
		want   bool
	}{
		{name: "4 minutes", tok: &Token{Expiry: now.Add(4 * 60 * time.Second)}, expiry: defaultExpiryDelta, want: true},
		{name: "3 minutes and 45 seconds", tok: &Token{Expiry: now.Add(defaultExpiryDelta)}, expiry: defaultExpiryDelta, want: true},
		{name: "3 minutes and 45 seconds-1ns", tok: &Token{Expiry: now.Add(defaultExpiryDelta - 1*time.Nanosecond)}, expiry: defaultExpiryDelta, want: false},
		{name: "-1 hour", tok: &Token{Expiry: now.Add(-1 * time.Hour)}, expiry: defaultExpiryDelta, want: false},
		{name: "12 seconds, custom expiryDelta", tok: &Token{Expiry: now.Add(12 * time.Second)}, expiry: time.Second * 5, want: true},
		{name: "5 seconds, custom expiryDelta", tok: &Token{Expiry: now.Add(time.Second * 5)}, expiry: time.Second * 5, want: true},
		{name: "5 seconds-1ns, custom expiryDelta", tok: &Token{Expiry: now.Add(time.Second*5 - 1*time.Nanosecond)}, expiry: time.Second * 5, want: false},
		{name: "-1 hour, custom expiryDelta", tok: &Token{Expiry: now.Add(-1 * time.Hour)}, expiry: time.Second * 5, want: false},
	}
	for _, tc := range cases {
		tc.tok.Value = "tok"
		if got, want := tc.tok.isValidWithEarlyExpiry(tc.expiry), tc.want; got != want {
			t.Errorf("expired (%q) = %v; want %v", tc.name, got, want)
		}
	}
}

func TestError_Error(t *testing.T) {

	tests := []struct {
		name string

		Response    *http.Response
		Body        []byte
		Err         error
		code        string
		description string
		uri         string

		want string
	}{
		{
			name: "basic",
			Response: &http.Response{
				StatusCode: http.StatusTeapot,
			},
			Body: []byte("I'm a teapot"),
			want: "auth: cannot fetch token: 418\nResponse: I'm a teapot",
		},
		{
			name:        "from query",
			code:        fmt.Sprint(http.StatusTeapot),
			description: "I'm a teapot",
			uri:         "somewhere",
			want:        "auth: \"418\" \"I'm a teapot\" \"somewhere\"",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			r := &Error{
				Response:    tt.Response,
				Body:        tt.Body,
				Err:         tt.Err,
				code:        tt.code,
				description: tt.description,
				uri:         tt.uri,
			}
			if got := r.Error(); got != tt.want {
				t.Errorf("Error.Error() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestNew2LOTokenProvider_JSONResponse(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
			"scope": "user",
			"token_type": "bearer",
			"expires_in": 3600
		}`))
	}))
	defer ts.Close()

	opts := &Options2LO{
		Email:      "aaa@example.com",
		PrivateKey: fakePrivateKey,
		TokenURL:   ts.URL,
	}
	tp, err := New2LOTokenProvider(opts)
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
		t.Errorf("token expiry = %v, want none", got)
	}
	scope := tok.Metadata["scope"].(string)
	if got, want := scope, "user"; got != want {
		t.Errorf("scope = %q; want %q", got, want)
	}
}

func TestNew2LOTokenProvider_BadResponse(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"scope": "user", "token_type": "bearer"}`))
	}))
	defer ts.Close()

	opts := &Options2LO{
		Email:      "aaa@example.com",
		PrivateKey: fakePrivateKey,
		TokenURL:   ts.URL,
	}
	tp, err := New2LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if tok == nil {
		t.Fatalf("got nil token; want token")
	}
	if tok.IsValid() {
		t.Errorf("got invalid token: %v", tok)
	}
	if got, want := tok.Value, ""; got != want {
		t.Errorf("access token = %q; want %q", got, want)
	}
	if got, want := tok.Type, "bearer"; got != want {
		t.Errorf("token type = %q; want %q", got, want)
	}
	scope := tok.Metadata["scope"].(string)
	if got, want := scope, "user"; got != want {
		t.Errorf("token scope = %q; want %q", got, want)
	}
}

func TestNew2LOTokenProvider_BadResponseType(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{"access_token":123, "scope": "user", "token_type": "bearer"}`))
	}))
	defer ts.Close()
	opts := &Options2LO{
		Email:      "aaa@example.com",
		PrivateKey: fakePrivateKey,
		TokenURL:   ts.URL,
	}
	tp, err := New2LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	tok, err := tp.Token(context.Background())
	if err == nil {
		t.Error("got a token; expected error")
		if got, want := tok.Value, ""; got != want {
			t.Errorf("access token = %q; want %q", got, want)
		}
	}
}

func TestNew2LOTokenProvider_Assertion(t *testing.T) {
	var assertion string
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.ParseForm()
		assertion = r.Form.Get("assertion")

		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
			"scope": "user",
			"token_type": "bearer",
			"expires_in": 3600
		}`))
	}))
	defer ts.Close()

	opts := &Options2LO{
		Email:        "aaa@example.com",
		PrivateKey:   fakePrivateKey,
		PrivateKeyID: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
		TokenURL:     ts.URL,
	}

	tp, err := New2LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	_, err = tp.Token(context.Background())
	if err != nil {
		t.Fatalf("Failed to fetch token: %v", err)
	}

	parts := strings.Split(assertion, ".")
	if len(parts) != 3 {
		t.Fatalf("assertion = %q; want 3 parts", assertion)
	}
	gotjson, err := base64.RawURLEncoding.DecodeString(parts[0])
	if err != nil {
		t.Fatalf("invalid token header; err = %v", err)
	}

	got := jwt.Header{}
	if err := json.Unmarshal(gotjson, &got); err != nil {
		t.Errorf("failed to unmarshal json token header = %q; err = %v", gotjson, err)
	}

	want := jwt.Header{
		Algorithm: "RS256",
		Type:      "JWT",
		KeyID:     "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
	}
	if got != want {
		t.Errorf("access token header = %q; want %q", got, want)
	}
}

func TestNew2LOTokenProvider_AssertionPayload(t *testing.T) {
	var assertion string
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		r.ParseForm()
		assertion = r.Form.Get("assertion")

		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(`{
			"access_token": "90d64460d14870c08c81352a05dedd3465940a7c",
			"scope": "user",
			"token_type": "bearer",
			"expires_in": 3600
		}`))
	}))
	defer ts.Close()

	for _, opts := range []*Options2LO{
		{
			Email:        "aaa1@example.com",
			PrivateKey:   fakePrivateKey,
			PrivateKeyID: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
			TokenURL:     ts.URL,
		},
		{
			Email:        "aaa2@example.com",
			PrivateKey:   fakePrivateKey,
			PrivateKeyID: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
			TokenURL:     ts.URL,
			Audience:     "https://example.com",
		},
		{
			Email:        "aaa2@example.com",
			PrivateKey:   fakePrivateKey,
			PrivateKeyID: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
			TokenURL:     ts.URL,
			PrivateClaims: map[string]interface{}{
				"private0": "claim0",
				"private1": "claim1",
			},
		},
	} {
		t.Run(opts.Email, func(t *testing.T) {
			tp, err := New2LOTokenProvider(opts)
			if err != nil {
				t.Fatal(err)
			}
			_, err = tp.Token(context.Background())
			if err != nil {
				t.Fatalf("Failed to fetch token: %v", err)
			}

			parts := strings.Split(assertion, ".")
			if len(parts) != 3 {
				t.Fatalf("assertion = %q; want 3 parts", assertion)
			}
			gotjson, err := base64.RawURLEncoding.DecodeString(parts[1])
			if err != nil {
				t.Fatalf("invalid token payload; err = %v", err)
			}

			claimSet := jwt.Claims{}
			if err := json.Unmarshal(gotjson, &claimSet); err != nil {
				t.Errorf("failed to unmarshal json token payload = %q; err = %v", gotjson, err)
			}

			if got, want := claimSet.Iss, opts.Email; got != want {
				t.Errorf("payload email = %q; want %q", got, want)
			}
			if got, want := claimSet.Scope, strings.Join(opts.Scopes, " "); got != want {
				t.Errorf("payload scope = %q; want %q", got, want)
			}
			aud := opts.TokenURL
			if opts.Audience != "" {
				aud = opts.Audience
			}
			if got, want := claimSet.Aud, aud; got != want {
				t.Errorf("payload audience = %q; want %q", got, want)
			}
			if got, want := claimSet.Sub, opts.Subject; got != want {
				t.Errorf("payload subject = %q; want %q", got, want)
			}
			if len(opts.PrivateClaims) > 0 {
				var got interface{}
				if err := json.Unmarshal(gotjson, &got); err != nil {
					t.Errorf("failed to parse payload; err = %q", err)
				}
				m := got.(map[string]interface{})
				for v, k := range opts.PrivateClaims {
					if !cmp.Equal(m[v], k) {
						t.Errorf("payload private claims key = %q: got %#v; want %#v", v, m[v], k)
					}
				}
			}
		})
	}
}

func TestNew2LOTokenProvider_TokenError(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-type", "application/json")
		w.WriteHeader(http.StatusBadRequest)
		w.Write([]byte(`{"error": "invalid_grant"}`))
	}))
	defer ts.Close()

	opts := &Options2LO{
		Email:      "aaa@example.com",
		PrivateKey: fakePrivateKey,
		TokenURL:   ts.URL,
	}

	tp, err := New2LOTokenProvider(opts)
	if err != nil {
		t.Fatal(err)
	}
	_, err = tp.Token(context.Background())
	if err == nil {
		t.Fatalf("got no error, expected one")
	}
	_, ok := err.(*Error)
	if !ok {
		t.Fatalf("got %T error, expected *Error", err)
	}
	expected := fmt.Sprintf("auth: cannot fetch token: %v\nResponse: %s", "400", `{"error": "invalid_grant"}`)
	if errStr := err.Error(); errStr != expected {
		t.Fatalf("got %#v, expected %#v", errStr, expected)
	}
}

func TestNew2LOTokenProvider_Validate(t *testing.T) {
	tests := []struct {
		name string
		opts *Options2LO
	}{
		{
			name: "missing options",
		},
		{
			name: "missing email",
			opts: &Options2LO{
				PrivateKey: []byte("key"),
				TokenURL:   "url",
			},
		},
		{
			name: "missing key",
			opts: &Options2LO{
				Email:    "email",
				TokenURL: "url",
			},
		},
		{
			name: "missing URL",
			opts: &Options2LO{
				Email:      "email",
				PrivateKey: []byte("key"),
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if _, err := New2LOTokenProvider(tt.opts); err == nil {
				t.Error("got nil, want an error")
			}
		})
	}
}

type countingTestProvider struct {
	count int
}

func (tp *countingTestProvider) Token(ctx context.Context) (*Token, error) {
	tok := &Token{
		Value: fmt.Sprint(tp.count),
		// Set expiry to count times seconds from now, so that as count increases
		// to 2, token state changes from stale to fresh.
		Expiry: time.Now().Add(time.Duration(tp.count) * time.Second),
	}
	tp.count++
	return tok, nil
}

func TestComputeTokenProvider_NonBlockingRefresh(t *testing.T) {
	// Freeze now for consistent results.
	now := time.Now()
	timeNow = func() time.Time { return now }
	defer func() { timeNow = time.Now }()
	tp := NewCachedTokenProvider(&countingTestProvider{count: 1}, &CachedTokenProviderOptions{
		// EarlyTokenRefresh ensures that token with early expiry just less than 2 seconds before now is already stale.
		ExpireEarly: 1990 * time.Millisecond,
	})
	if state := tp.(*cachedTokenProvider).tokenState(); state != invalid {
		t.Errorf("got %d, want %d", state, invalid)
	}
	freshToken, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if state := tp.(*cachedTokenProvider).tokenState(); state != stale {
		t.Errorf("got %d, want %d", state, stale)
	}
	if want := "1"; freshToken.Value != want {
		t.Errorf("got %q, want %q", freshToken.Value, want)
	}
	staleToken, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if state := tp.(*cachedTokenProvider).tokenState(); state != stale {
		t.Errorf("got %d, want %d", state, stale)
	}
	if want := "1"; staleToken.Value != want {
		t.Errorf("got %q, want %q", staleToken.Value, want)
	}
	// Allow time for async refresh.
	time.Sleep(100 * time.Millisecond)
	freshToken2, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if state := tp.(*cachedTokenProvider).tokenState(); state != fresh {
		t.Errorf("got %d, want %d", state, fresh)
	}
	if want := "2"; freshToken2.Value != want {
		t.Errorf("got %q, want %q", freshToken2.Value, want)
	}
	// Allow time for 2nd async refresh.
	time.Sleep(100 * time.Millisecond)
	freshToken3, err := tp.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if state := tp.(*cachedTokenProvider).tokenState(); state != fresh {
		t.Errorf("got %d, want %d", state, fresh)
	}
	if want := "2"; freshToken3.Value != want {
		t.Errorf("got %q, want %q", freshToken3.Value, want)
	}
}

func TestComputeTokenProvider_BlockingRefresh(t *testing.T) {
	tests := []struct {
		name               string
		disableAutoRefresh bool
		want1              string
		want2              string
		wantState2         tokenState
	}{
		{
			name:               "disableAutoRefresh",
			disableAutoRefresh: true,
			want1:              "1",
			want2:              "1",
			// Because token "count" does not increase, it will always be stale.
			wantState2: stale,
		},
		{
			name:               "autoRefresh",
			disableAutoRefresh: false,
			want1:              "1",
			want2:              "2",
			// As token "count" increases to 2, it transitions to fresh.
			wantState2: fresh,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// Freeze now for consistent results.
			now := time.Now()
			timeNow = func() time.Time { return now }
			defer func() { timeNow = time.Now }()
			tp := NewCachedTokenProvider(&countingTestProvider{count: 1}, &CachedTokenProviderOptions{
				DisableAsyncRefresh: true,
				DisableAutoRefresh:  tt.disableAutoRefresh,
				// EarlyTokenRefresh ensures that token with early expiry just less than 2 seconds before now is already stale.
				ExpireEarly: 1990 * time.Millisecond,
			})
			if state := tp.(*cachedTokenProvider).tokenState(); state != invalid {
				t.Errorf("got %d, want %d", state, invalid)
			}
			freshToken, err := tp.Token(context.Background())
			if err != nil {
				t.Fatal(err)
			}
			if freshToken == nil {
				t.Fatal("freshToken is nil")
			}
			if state := tp.(*cachedTokenProvider).tokenState(); state != stale {
				t.Errorf("got %d, want %d", state, stale)
			}
			if freshToken.Value != tt.want1 {
				t.Errorf("got %q, want %q", freshToken.Value, tt.want1)
			}
			time.Sleep(100 * time.Millisecond)
			freshToken2, err := tp.Token(context.Background())
			if err != nil {
				t.Fatal(err)
			}
			if state := tp.(*cachedTokenProvider).tokenState(); state != tt.wantState2 {
				t.Errorf("got %d, want %d", state, tt.wantState2)
			}
			if freshToken2.Value != tt.want2 {
				t.Errorf("got %q, want %q", freshToken2.Value, tt.want2)
			}
		})
	}
}
