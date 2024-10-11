// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package stsexchange

import (
	"context"
	"encoding/json"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"golang.org/x/oauth2"
)

var auth = ClientAuthentication{
	AuthStyle:    oauth2.AuthStyleInHeader,
	ClientID:     clientID,
	ClientSecret: clientSecret,
}

var exchangeTokenRequest = TokenExchangeRequest{
	ActingParty: struct {
		ActorToken     string
		ActorTokenType string
	}{},
	GrantType:          "urn:ietf:params:oauth:grant-type:token-exchange",
	Resource:           "",
	Audience:           "32555940559.apps.googleusercontent.com", //TODO: Make sure audience is correct in this test (might be mismatched)
	Scope:              []string{"https://www.googleapis.com/auth/devstorage.full_control"},
	RequestedTokenType: "urn:ietf:params:oauth:token-type:access_token",
	SubjectToken:       "Sample.Subject.Token",
	SubjectTokenType:   "urn:ietf:params:oauth:token-type:jwt",
}

var exchangeRequestBody = "audience=32555940559.apps.googleusercontent.com&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdevstorage.full_control&subject_token=Sample.Subject.Token&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Ajwt"
var exchangeResponseBody = `{"access_token":"Sample.Access.Token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"scope":"https://www.googleapis.com/auth/cloud-platform"}`
var expectedExchangeToken = Response{
	AccessToken:     "Sample.Access.Token",
	IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
	TokenType:       "Bearer",
	ExpiresIn:       3600,
	Scope:           "https://www.googleapis.com/auth/cloud-platform",
	RefreshToken:    "",
}

var refreshToken = "ReFrEsHtOkEn"
var refreshRequestBody = "grant_type=refresh_token&refresh_token=" + refreshToken
var refreshResponseBody = `{"access_token":"Sample.Access.Token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"scope":"https://www.googleapis.com/auth/cloud-platform","refresh_token":"REFRESHED_REFRESH"}`
var expectedRefreshResponse = Response{
	AccessToken:     "Sample.Access.Token",
	IssuedTokenType: "urn:ietf:params:oauth:token-type:access_token",
	TokenType:       "Bearer",
	ExpiresIn:       3600,
	Scope:           "https://www.googleapis.com/auth/cloud-platform",
	RefreshToken:    "REFRESHED_REFRESH",
}

func TestExchangeToken(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		if r.URL.String() != "/" {
			t.Errorf("Unexpected request URL, %v is found", r.URL)
		}
		if got, want := r.Header.Get("Authorization"), "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="; got != want {
			t.Errorf("Unexpected authorization header, got %v, want %v", got, want)
		}
		if got, want := r.Header.Get("Content-Type"), "application/x-www-form-urlencoded"; got != want {
			t.Errorf("Unexpected Content-Type header, got %v, want %v", got, want)
		}
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			t.Errorf("Failed reading request body: %v.", err)
		}
		if got, want := string(body), exchangeRequestBody; got != want {
			t.Errorf("Unexpected exchange payload, got %v but want %v", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(exchangeResponseBody))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Add("Content-Type", "application/x-www-form-urlencoded")

	resp, err := ExchangeToken(context.Background(), ts.URL, &exchangeTokenRequest, auth, headers, nil)
	if err != nil {
		t.Fatalf("exchangeToken failed with error: %v", err)
	}

	if expectedExchangeToken != *resp {
		t.Errorf("mismatched messages received by mock server.  \nWant: \n%v\n\nGot:\n%v", expectedExchangeToken, *resp)
	}

	resp, err = ExchangeToken(context.Background(), ts.URL, &exchangeTokenRequest, auth, nil, nil)
	if err != nil {
		t.Fatalf("exchangeToken failed with error: %v", err)
	}

	if expectedExchangeToken != *resp {
		t.Errorf("mismatched messages received by mock server.  \nWant: \n%v\n\nGot:\n%v", expectedExchangeToken, *resp)
	}
}

func TestExchangeToken_Err(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte("what's wrong with this response?"))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Add("Content-Type", "application/x-www-form-urlencoded")
	_, err := ExchangeToken(context.Background(), ts.URL, &exchangeTokenRequest, auth, headers, nil)
	if err == nil {
		t.Errorf("Expected handled error; instead got nil.")
	}
}

/* Lean test specifically for options, as the other features are tested earlier. */
type testOpts struct {
	First  string `json:"first"`
	Second string `json:"second"`
}

var optsValues = [][]string{{"foo", "bar"}, {"cat", "pan"}}

func TestExchangeToken_Opts(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("Failed reading request body: %v.", err)
		}
		data, err := url.ParseQuery(string(body))
		if err != nil {
			t.Fatalf("Failed to parse request body: %v", err)
		}
		strOpts, ok := data["options"]
		if !ok {
			t.Errorf("Server didn't recieve an \"options\" field.")
		} else if len(strOpts) < 1 {
			t.Errorf("\"options\" field has length 0.")
		}
		var opts map[string]interface{}
		err = json.Unmarshal([]byte(strOpts[0]), &opts)
		if err != nil {
			t.Fatalf("Couldn't parse received \"options\" field.")
		}
		if len(opts) < 2 {
			t.Errorf("Too few options received.")
		}

		val, ok := opts["one"]
		if !ok {
			t.Errorf("Couldn't find first option parameter.")
		} else {
			tOpts1, ok := val.(map[string]interface{})
			if !ok {
				t.Errorf("Failed to assert the first option parameter as type testOpts.")
			} else {
				if got, want := tOpts1["first"].(string), optsValues[0][0]; got != want {
					t.Errorf("First value in first options field is incorrect; got %v but want %v", got, want)
				}
				if got, want := tOpts1["second"].(string), optsValues[0][1]; got != want {
					t.Errorf("Second value in first options field is incorrect; got %v but want %v", got, want)
				}
			}
		}

		val2, ok := opts["two"]
		if !ok {
			t.Errorf("Couldn't find second option parameter.")
		} else {
			tOpts2, ok := val2.(map[string]interface{})
			if !ok {
				t.Errorf("Failed to assert the second option parameter as type testOpts.")
			} else {
				if got, want := tOpts2["first"].(string), optsValues[1][0]; got != want {
					t.Errorf("First value in second options field is incorrect; got %v but want %v", got, want)
				}
				if got, want := tOpts2["second"].(string), optsValues[1][1]; got != want {
					t.Errorf("Second value in second options field is incorrect; got %v but want %v", got, want)
				}
			}
		}

		// Send a proper reply so that no other errors crop up.
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(exchangeResponseBody))

	}))
	defer ts.Close()
	headers := http.Header{}
	headers.Add("Content-Type", "application/x-www-form-urlencoded")

	firstOption := testOpts{optsValues[0][0], optsValues[0][1]}
	secondOption := testOpts{optsValues[1][0], optsValues[1][1]}
	inputOpts := make(map[string]interface{})
	inputOpts["one"] = firstOption
	inputOpts["two"] = secondOption
	ExchangeToken(context.Background(), ts.URL, &exchangeTokenRequest, auth, headers, inputOpts)
}

func TestRefreshToken(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		if r.URL.String() != "/" {
			t.Errorf("Unexpected request URL, %v is found", r.URL)
		}
		if got, want := r.Header.Get("Authorization"), "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="; got != want {
			t.Errorf("Unexpected authorization header, got %v, want %v", got, want)
		}
		if got, want := r.Header.Get("Content-Type"), "application/x-www-form-urlencoded"; got != want {
			t.Errorf("Unexpected Content-Type header, got %v, want %v", got, want)
		}
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			t.Errorf("Failed reading request body: %v.", err)
		}
		if got, want := string(body), refreshRequestBody; got != want {
			t.Errorf("Unexpected exchange payload, got %v but want %v", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(refreshResponseBody))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Add("Content-Type", "application/x-www-form-urlencoded")

	resp, err := RefreshAccessToken(context.Background(), ts.URL, refreshToken, auth, headers)
	if err != nil {
		t.Fatalf("exchangeToken failed with error: %v", err)
	}

	if expectedRefreshResponse != *resp {
		t.Errorf("mismatched messages received by mock server.  \nWant: \n%v\n\nGot:\n%v", expectedRefreshResponse, *resp)
	}

	resp, err = RefreshAccessToken(context.Background(), ts.URL, refreshToken, auth, nil)
	if err != nil {
		t.Fatalf("exchangeToken failed with error: %v", err)
	}

	if expectedRefreshResponse != *resp {
		t.Errorf("mismatched messages received by mock server.  \nWant: \n%v\n\nGot:\n%v", expectedRefreshResponse, *resp)
	}
}

func TestRefreshToken_Err(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte("what's wrong with this response?"))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Add("Content-Type", "application/x-www-form-urlencoded")

	_, err := RefreshAccessToken(context.Background(), ts.URL, refreshToken, auth, headers)
	if err == nil {
		t.Errorf("Expected handled error; instead got nil.")
	}
}
