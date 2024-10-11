// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package authhandler

import (
	"context"
	"fmt"
	"net/http"
	"net/http/httptest"
	"testing"

	"golang.org/x/oauth2"
)

func TestTokenExchange_Success(t *testing.T) {
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

	conf := &oauth2.Config{
		ClientID: "testClientID",
		Scopes:   []string{"pubsub"},
		Endpoint: oauth2.Endpoint{
			AuthURL:  "testAuthCodeURL",
			TokenURL: ts.URL,
		},
	}

	tok, err := TokenSource(context.Background(), conf, "testState", authhandler).Token()
	if err != nil {
		t.Fatal(err)
	}
	if !tok.Valid() {
		t.Errorf("got invalid token: %v", tok)
	}
	if got, want := tok.AccessToken, "90d64460d14870c08c81352a05dedd3465940a7c"; got != want {
		t.Errorf("access token = %q; want %q", got, want)
	}
	if got, want := tok.TokenType, "bearer"; got != want {
		t.Errorf("token type = %q; want %q", got, want)
	}
	if got := tok.Expiry.IsZero(); got {
		t.Errorf("token expiry is zero = %v, want false", got)
	}
	scope := tok.Extra("scope")
	if got, want := scope, "pubsub"; got != want {
		t.Errorf("scope = %q; want %q", got, want)
	}
}

func TestTokenExchange_StateMismatch(t *testing.T) {
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

	conf := &oauth2.Config{
		ClientID: "testClientID",
		Scopes:   []string{"pubsub"},
		Endpoint: oauth2.Endpoint{
			AuthURL:  "testAuthCodeURL",
			TokenURL: ts.URL,
		},
	}

	_, err := TokenSource(context.Background(), conf, "testState", authhandler).Token()
	if want_err := "state mismatch in 3-legged-OAuth flow"; err == nil || err.Error() != want_err {
		t.Errorf("err = %q; want %q", err, want_err)
	}
}

func TestTokenExchangeWithPKCE_Success(t *testing.T) {
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

	conf := &oauth2.Config{
		ClientID: "testClientID",
		Scopes:   []string{"pubsub"},
		Endpoint: oauth2.Endpoint{
			AuthURL:  "testAuthCodeURL",
			TokenURL: ts.URL,
		},
	}
	pkce := PKCEParams{
		Challenge:       "codeChallenge",
		ChallengeMethod: "plain",
		Verifier:        "codeChallenge",
	}

	tok, err := TokenSourceWithPKCE(context.Background(), conf, "testState", authhandler, &pkce).Token()
	if err != nil {
		t.Fatal(err)
	}
	if !tok.Valid() {
		t.Errorf("got invalid token: %v", tok)
	}
	if got, want := tok.AccessToken, "90d64460d14870c08c81352a05dedd3465940a7c"; got != want {
		t.Errorf("access token = %q; want %q", got, want)
	}
	if got, want := tok.TokenType, "bearer"; got != want {
		t.Errorf("token type = %q; want %q", got, want)
	}
	if got := tok.Expiry.IsZero(); got {
		t.Errorf("token expiry is zero = %v, want false", got)
	}
	scope := tok.Extra("scope")
	if got, want := scope, "pubsub"; got != want {
		t.Errorf("scope = %q; want %q", got, want)
	}
}
