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

package stsexchange

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/internal"
	"github.com/google/go-cmp/cmp"
)

var (
	clientAuth = ClientAuthentication{
		AuthStyle:    auth.StyleInHeader,
		ClientID:     clientID,
		ClientSecret: clientSecret,
	}
	tokReq = TokenRequest{
		ActingParty: struct {
			ActorToken     string
			ActorTokenType string
		}{},
		GrantType:          GrantType,
		Resource:           "",
		Audience:           "32555940559.apps.googleusercontent.com",
		Scope:              []string{"https://www.googleapis.com/auth/devstorage.full_control"},
		RequestedTokenType: TokenType,
		SubjectToken:       "Sample.Subject.Token",
		SubjectTokenType:   jwtTokenType,
	}

	responseBody = `{"access_token":"Sample.Access.Token","issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer","expires_in":3600,"scope":"https://www.googleapis.com/auth/cloud-platform"}`
)

func TestExchangeToken(t *testing.T) {
	requestbody := "audience=32555940559.apps.googleusercontent.com&grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fdevstorage.full_control&subject_token=Sample.Subject.Token&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Ajwt"
	wantToken := TokenResponse{
		AccessToken:     "Sample.Access.Token",
		IssuedTokenType: TokenType,
		TokenType:       internal.TokenTypeBearer,
		ExpiresIn:       3600,
		Scope:           "https://www.googleapis.com/auth/cloud-platform",
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != http.MethodPost {
			t.Errorf("got %v, want %v", r.Method, http.MethodPost)
		}
		if got, want := r.Header.Get("Authorization"), "Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		if got, want := r.Header.Get("Content-Type"), "application/x-www-form-urlencoded"; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Error(err)
		}
		if got, want := string(body), requestbody; got != want {
			t.Errorf("got %v, want %v", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(responseBody))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Set("Content-Type", "application/x-www-form-urlencoded")

	resp, err := ExchangeToken(context.Background(), &Options{
		Client:         internal.CloneDefaultClient(),
		Endpoint:       ts.URL,
		Request:        &tokReq,
		Authentication: clientAuth,
		Headers:        headers,
		ExtraOpts:      nil,
	})
	if err != nil {
		t.Fatalf("exchangeToken() = %v", err)
	}

	if *resp != wantToken {
		t.Errorf("got %v, want %v", *resp, wantToken)
	}
}

func TestExchangeToken_Err(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte("what's wrong with this response?"))
	}))
	defer ts.Close()

	headers := http.Header{}
	headers.Set("Content-Type", "application/x-www-form-urlencoded")
	if _, err := ExchangeToken(context.Background(), &Options{
		Client:         internal.CloneDefaultClient(),
		Endpoint:       ts.URL,
		Request:        &tokReq,
		Authentication: clientAuth,
		Headers:        headers,
		ExtraOpts:      nil,
	}); err == nil {
		t.Errorf("got nil, want an error")
	}
}

func TestExchangeToken_Opts(t *testing.T) {
	optsValues := [][]string{{"foo", "bar"}, {"cat", "pan"}}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		body, err := io.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("io.ReadAll() = %v", err)
		}
		data, err := url.ParseQuery(string(body))
		if err != nil {
			t.Fatalf("url.ParseQuery() = %v", err)
		}
		strOpts, ok := data["options"]
		if !ok {
			t.Errorf(`server didn't receive an "options" field`)
		} else if len(strOpts) < 1 {
			t.Errorf(`"options" field has length 0`)
		}
		var opts map[string]interface{}
		err = json.Unmarshal([]byte(strOpts[0]), &opts)
		if err != nil {
			t.Fatalf(`couldn't parse received "options" field`)
		}
		if len(opts) < 2 {
			t.Errorf("too few options received")
		}

		val, ok := opts["one"]
		if !ok {
			t.Errorf("couldn't find first option parameter")
		} else {
			tOpts1, ok := val.(map[string]interface{})
			if !ok {
				t.Errorf("failed to assert the first option parameter as type testOpts")
			} else {
				if got, want := tOpts1["first"].(string), optsValues[0][0]; got != want {
					t.Errorf("got %v, want %v", got, want)
				}
				if got, want := tOpts1["second"].(string), optsValues[0][1]; got != want {
					t.Errorf("got %v, want %v", got, want)
				}
			}
		}

		val2, ok := opts["two"]
		if !ok {
			t.Errorf("couldn't find second option parameter")
		} else {
			tOpts2, ok := val2.(map[string]interface{})
			if !ok {
				t.Errorf("Failed to assert the second option parameter as type testOpts.")
			} else {
				if got, want := tOpts2["first"].(string), optsValues[1][0]; got != want {
					t.Errorf("got %v, want %v", got, want)
				}
				if got, want := tOpts2["second"].(string), optsValues[1][1]; got != want {
					t.Errorf("got %v, want %v", got, want)
				}
			}
		}
		// Send a proper reply so that no other errors crop up.
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(responseBody))

	}))
	defer ts.Close()
	headers := http.Header{}
	headers.Set("Content-Type", "application/x-www-form-urlencoded")

	type testOpts struct {
		First  string `json:"first"`
		Second string `json:"second"`
	}
	firstOption := testOpts{optsValues[0][0], optsValues[0][1]}
	secondOption := testOpts{optsValues[1][0], optsValues[1][1]}
	inputOpts := make(map[string]interface{})
	inputOpts["one"] = firstOption
	inputOpts["two"] = secondOption

	ExchangeToken(context.Background(), &Options{
		Client:         internal.CloneDefaultClient(),
		Endpoint:       ts.URL,
		Request:        &tokReq,
		Authentication: clientAuth,
		Headers:        headers,
		ExtraOpts:      inputOpts,
	})
}

var (
	clientID           = "rbrgnognrhongo3bi4gb9ghg9g"
	clientSecret       = "notsosecret"
	audience           = []string{"32555940559.apps.googleusercontent.com"}
	grantType          = []string{GrantType}
	requestedTokenType = []string{TokenType}
	subjectTokenType   = []string{jwtTokenType}
	subjectToken       = []string{"eyJhbGciOiJSUzI1NiIsImtpZCI6IjJjNmZhNmY1OTUwYTdjZTQ2NWZjZjI0N2FhMGIwOTQ4MjhhYzk1MmMiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJhenAiOiIzMjU1NTk0MDU1OS5hcHBzLmdvb2dsZXVzZXJjb250ZW50LmNvbSIsImF1ZCI6IjMyNTU1OTQwNTU5LmFwcHMuZ29vZ2xldXNlcmNvbnRlbnQuY29tIiwic3ViIjoiMTEzMzE4NTQxMDA5MDU3Mzc4MzI4IiwiaGQiOiJnb29nbGUuY29tIiwiZW1haWwiOiJpdGh1cmllbEBnb29nbGUuY29tIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsImF0X2hhc2giOiI5OVJVYVFrRHJsVDFZOUV0SzdiYXJnIiwiaWF0IjoxNjAxNTgxMzQ5LCJleHAiOjE2MDE1ODQ5NDl9.SZ-4DyDcogDh_CDUKHqPCiT8AKLg4zLMpPhGQzmcmHQ6cJiV0WRVMf5Lq911qsvuekgxfQpIdKNXlD6yk3FqvC2rjBbuEztMF-OD_2B8CEIYFlMLGuTQimJlUQksLKM-3B2ITRDCxnyEdaZik0OVssiy1CBTsllS5MgTFqic7w8w0Cd6diqNkfPFZRWyRYsrRDRlHHbH5_TUnv2wnLVHBHlNvU4wU2yyjDIoqOvTRp8jtXdq7K31CDhXd47-hXsVFQn2ZgzuUEAkH2Q6NIXACcVyZOrjBcZiOQI9IRWz-g03LzbzPSecO7I8dDrhqUSqMrdNUz_f8Kr8JFhuVMfVug"}
	scope              = []string{"https://www.googleapis.com/auth/devstorage.full_control"}
	ContentType        = []string{"application/x-www-form-urlencoded"}
)

func TestClientAuthentication_InjectHeaderAuthentication(t *testing.T) {
	valuesH := url.Values{
		"audience":             audience,
		"grant_type":           grantType,
		"requested_token_type": requestedTokenType,
		"subject_token_type":   subjectTokenType,
		"subject_token":        subjectToken,
		"scope":                scope,
	}
	headerH := http.Header{
		"Content-Type": ContentType,
	}

	headerAuthentication := ClientAuthentication{
		AuthStyle:    auth.StyleInHeader,
		ClientID:     clientID,
		ClientSecret: clientSecret,
	}
	headerAuthentication.InjectAuthentication(valuesH, headerH)

	if got, want := valuesH["audience"], audience; !cmp.Equal(got, want) {
		t.Errorf("audience = %q, want %q", got, want)
	}
	if got, want := valuesH["grant_type"], grantType; !cmp.Equal(got, want) {
		t.Errorf("grant_type = %q, want %q", got, want)
	}
	if got, want := valuesH["requested_token_type"], requestedTokenType; !cmp.Equal(got, want) {
		t.Errorf("requested_token_type = %q, want %q", got, want)
	}
	if got, want := valuesH["subject_token_type"], subjectTokenType; !cmp.Equal(got, want) {
		t.Errorf("subject_token_type = %q, want %q", got, want)
	}
	if got, want := valuesH["subject_token"], subjectToken; !cmp.Equal(got, want) {
		t.Errorf("subject_token = %q, want %q", got, want)
	}
	if got, want := valuesH["scope"], scope; !cmp.Equal(got, want) {
		t.Errorf("scope = %q, want %q", got, want)
	}
	if got, want := headerH["Authorization"], []string{"Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="}; !cmp.Equal(got, want) {
		t.Errorf("Authorization in header = %q, want %q", got, want)
	}
}

func TestClientAuthentication_ParamsAuthentication(t *testing.T) {
	valuesP := url.Values{
		"audience":             audience,
		"grant_type":           grantType,
		"requested_token_type": requestedTokenType,
		"subject_token_type":   subjectTokenType,
		"subject_token":        subjectToken,
		"scope":                scope,
	}
	headerP := http.Header{
		"Content-Type": ContentType,
	}
	paramsAuthentication := ClientAuthentication{
		AuthStyle:    auth.StyleInParams,
		ClientID:     clientID,
		ClientSecret: clientSecret,
	}
	paramsAuthentication.InjectAuthentication(valuesP, headerP)

	if got, want := valuesP["audience"], audience; !cmp.Equal(got, want) {
		t.Errorf("audience = %q, want %q", got, want)
	}
	if got, want := valuesP["grant_type"], grantType; !cmp.Equal(got, want) {
		t.Errorf("grant_type = %q, want %q", got, want)
	}
	if got, want := valuesP["requested_token_type"], requestedTokenType; !cmp.Equal(got, want) {
		t.Errorf("requested_token_type = %q, want %q", got, want)
	}
	if got, want := valuesP["subject_token_type"], subjectTokenType; !cmp.Equal(got, want) {
		t.Errorf("subject_token_type = %q, want %q", got, want)
	}
	if got, want := valuesP["subject_token"], subjectToken; !cmp.Equal(got, want) {
		t.Errorf("subject_token = %q, want %q", got, want)
	}
	if got, want := valuesP["scope"], scope; !cmp.Equal(got, want) {
		t.Errorf("scope = %q, want %q", got, want)
	}
	if got, want := valuesP["client_id"], []string{clientID}; !cmp.Equal(got, want) {
		t.Errorf("client_id = %q, want %q", got, want)
	}
	if got, want := valuesP["client_secret"], []string{clientSecret}; !cmp.Equal(got, want) {
		t.Errorf("client_secret = %q, want %q", got, want)
	}
}
