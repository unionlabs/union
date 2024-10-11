// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package stsexchange

import (
	"net/http"
	"net/url"
	"reflect"
	"testing"

	"golang.org/x/oauth2"
)

var clientID = "rbrgnognrhongo3bi4gb9ghg9g"
var clientSecret = "notsosecret"

var audience = []string{"32555940559.apps.googleusercontent.com"}
var grantType = []string{"urn:ietf:params:oauth:grant-type:token-exchange"}
var requestedTokenType = []string{"urn:ietf:params:oauth:token-type:access_token"}
var subjectTokenType = []string{"urn:ietf:params:oauth:token-type:jwt"}
var subjectToken = []string{"eyJhbGciOiJSUzI1NiIsImtpZCI6IjJjNmZhNmY1OTUwYTdjZTQ2NWZjZjI0N2FhMGIwOTQ4MjhhYzk1MmMiLCJ0eXAiOiJKV1QifQ.eyJpc3MiOiJodHRwczovL2FjY291bnRzLmdvb2dsZS5jb20iLCJhenAiOiIzMjU1NTk0MDU1OS5hcHBzLmdvb2dsZXVzZXJjb250ZW50LmNvbSIsImF1ZCI6IjMyNTU1OTQwNTU5LmFwcHMuZ29vZ2xldXNlcmNvbnRlbnQuY29tIiwic3ViIjoiMTEzMzE4NTQxMDA5MDU3Mzc4MzI4IiwiaGQiOiJnb29nbGUuY29tIiwiZW1haWwiOiJpdGh1cmllbEBnb29nbGUuY29tIiwiZW1haWxfdmVyaWZpZWQiOnRydWUsImF0X2hhc2giOiI5OVJVYVFrRHJsVDFZOUV0SzdiYXJnIiwiaWF0IjoxNjAxNTgxMzQ5LCJleHAiOjE2MDE1ODQ5NDl9.SZ-4DyDcogDh_CDUKHqPCiT8AKLg4zLMpPhGQzmcmHQ6cJiV0WRVMf5Lq911qsvuekgxfQpIdKNXlD6yk3FqvC2rjBbuEztMF-OD_2B8CEIYFlMLGuTQimJlUQksLKM-3B2ITRDCxnyEdaZik0OVssiy1CBTsllS5MgTFqic7w8w0Cd6diqNkfPFZRWyRYsrRDRlHHbH5_TUnv2wnLVHBHlNvU4wU2yyjDIoqOvTRp8jtXdq7K31CDhXd47-hXsVFQn2ZgzuUEAkH2Q6NIXACcVyZOrjBcZiOQI9IRWz-g03LzbzPSecO7I8dDrhqUSqMrdNUz_f8Kr8JFhuVMfVug"}
var scope = []string{"https://www.googleapis.com/auth/devstorage.full_control"}

var ContentType = []string{"application/x-www-form-urlencoded"}

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
		AuthStyle:    oauth2.AuthStyleInHeader,
		ClientID:     clientID,
		ClientSecret: clientSecret,
	}
	headerAuthentication.InjectAuthentication(valuesH, headerH)

	if got, want := valuesH["audience"], audience; !reflect.DeepEqual(got, want) {
		t.Errorf("audience = %q, want %q", got, want)
	}
	if got, want := valuesH["grant_type"], grantType; !reflect.DeepEqual(got, want) {
		t.Errorf("grant_type = %q, want %q", got, want)
	}
	if got, want := valuesH["requested_token_type"], requestedTokenType; !reflect.DeepEqual(got, want) {
		t.Errorf("requested_token_type = %q, want %q", got, want)
	}
	if got, want := valuesH["subject_token_type"], subjectTokenType; !reflect.DeepEqual(got, want) {
		t.Errorf("subject_token_type = %q, want %q", got, want)
	}
	if got, want := valuesH["subject_token"], subjectToken; !reflect.DeepEqual(got, want) {
		t.Errorf("subject_token = %q, want %q", got, want)
	}
	if got, want := valuesH["scope"], scope; !reflect.DeepEqual(got, want) {
		t.Errorf("scope = %q, want %q", got, want)
	}
	if got, want := headerH["Authorization"], []string{"Basic cmJyZ25vZ25yaG9uZ28zYmk0Z2I5Z2hnOWc6bm90c29zZWNyZXQ="}; !reflect.DeepEqual(got, want) {
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
		AuthStyle:    oauth2.AuthStyleInParams,
		ClientID:     clientID,
		ClientSecret: clientSecret,
	}
	paramsAuthentication.InjectAuthentication(valuesP, headerP)

	if got, want := valuesP["audience"], audience; !reflect.DeepEqual(got, want) {
		t.Errorf("audience = %q, want %q", got, want)
	}
	if got, want := valuesP["grant_type"], grantType; !reflect.DeepEqual(got, want) {
		t.Errorf("grant_type = %q, want %q", got, want)
	}
	if got, want := valuesP["requested_token_type"], requestedTokenType; !reflect.DeepEqual(got, want) {
		t.Errorf("requested_token_type = %q, want %q", got, want)
	}
	if got, want := valuesP["subject_token_type"], subjectTokenType; !reflect.DeepEqual(got, want) {
		t.Errorf("subject_token_type = %q, want %q", got, want)
	}
	if got, want := valuesP["subject_token"], subjectToken; !reflect.DeepEqual(got, want) {
		t.Errorf("subject_token = %q, want %q", got, want)
	}
	if got, want := valuesP["scope"], scope; !reflect.DeepEqual(got, want) {
		t.Errorf("scope = %q, want %q", got, want)
	}
	if got, want := valuesP["client_id"], []string{clientID}; !reflect.DeepEqual(got, want) {
		t.Errorf("client_id = %q, want %q", got, want)
	}
	if got, want := valuesP["client_secret"], []string{clientSecret}; !reflect.DeepEqual(got, want) {
		t.Errorf("client_secret = %q, want %q", got, want)
	}
}
