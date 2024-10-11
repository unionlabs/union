// Copyright 2022 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package google

import (
	"net/http"
	"testing"

	"golang.org/x/oauth2"
)

func TestAuthenticationError_Temporary(t *testing.T) {
	tests := []struct {
		name string
		code int
		want bool
	}{
		{
			name: "temporary with 500",
			code: 500,
			want: true,
		},
		{
			name: "temporary with 503",
			code: 503,
			want: true,
		},
		{
			name: "temporary with 408",
			code: 408,
			want: true,
		},
		{
			name: "temporary with 429",
			code: 429,
			want: true,
		},
		{
			name: "temporary with 418",
			code: 418,
			want: false,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ae := &AuthenticationError{
				err: &oauth2.RetrieveError{
					Response: &http.Response{
						StatusCode: tt.code,
					},
				},
			}
			if got := ae.Temporary(); got != tt.want {
				t.Errorf("Temporary() = %v; want %v", got, tt.want)
			}
		})
	}
}

func TestErrWrappingTokenSource_Token(t *testing.T) {
	tok := oauth2.Token{AccessToken: "MyAccessToken"}
	ts := errWrappingTokenSource{
		src: oauth2.StaticTokenSource(&tok),
	}
	got, err := ts.Token()
	if *got != tok {
		t.Errorf("Token() = %v; want %v", got, tok)
	}
	if err != nil {
		t.Error(err)
	}
}

type errTokenSource struct {
	err error
}

func (s *errTokenSource) Token() (*oauth2.Token, error) {
	return nil, s.err
}

func TestErrWrappingTokenSource_TokenError(t *testing.T) {
	re := &oauth2.RetrieveError{
		Response: &http.Response{
			StatusCode: 500,
		},
	}
	ts := errWrappingTokenSource{
		src: &errTokenSource{
			err: re,
		},
	}
	_, err := ts.Token()
	if err == nil {
		t.Fatalf("errWrappingTokenSource.Token() err = nil, want *AuthenticationError")
	}
	ae, ok := err.(*AuthenticationError)
	if !ok {
		t.Fatalf("errWrappingTokenSource.Token() err = %T, want *AuthenticationError", err)
	}
	wrappedErr := ae.Unwrap()
	if wrappedErr == nil {
		t.Fatalf("AuthenticationError.Unwrap() err = nil, want *oauth2.RetrieveError")
	}
	_, ok = wrappedErr.(*oauth2.RetrieveError)
	if !ok {
		t.Errorf("AuthenticationError.Unwrap() err = %T, want *oauth2.RetrieveError", err)
	}
}
