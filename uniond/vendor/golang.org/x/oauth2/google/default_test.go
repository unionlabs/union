// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package google

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"

	"cloud.google.com/go/compute/metadata"
)

var saJSONJWT = []byte(`{
  "type": "service_account",
  "project_id": "fake_project",
  "private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
  "private_key": "super secret key",
  "client_email": "gopher@developer.gserviceaccount.com",
  "client_id": "gopher.apps.googleusercontent.com",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/gopher%40fake_project.iam.gserviceaccount.com"
}`)

var saJSONJWTUniverseDomain = []byte(`{
  "type": "service_account",
  "project_id": "fake_project",
  "universe_domain": "example.com",
  "private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
  "private_key": "super secret key",
  "client_email": "gopher@developer.gserviceaccount.com",
  "client_id": "gopher.apps.googleusercontent.com",
  "auth_uri": "https://accounts.google.com/o/oauth2/auth",
  "token_uri": "https://oauth2.googleapis.com/token",
  "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
  "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/gopher%40fake_project.iam.gserviceaccount.com"
}`)

var userJSON = []byte(`{
  "client_id": "abc123.apps.googleusercontent.com",
  "client_secret": "shh",
  "refresh_token": "refreshing",
  "type": "authorized_user",
  "quota_project_id": "fake_project2"
}`)

var userJSONUniverseDomain = []byte(`{
  "client_id": "abc123.apps.googleusercontent.com",
  "client_secret": "shh",
  "refresh_token": "refreshing",
  "type": "authorized_user",
  "quota_project_id": "fake_project2",
  "universe_domain": "example.com"
}`)

var universeDomain = "example.com"

var universeDomain2 = "apis-tpclp.goog"

func TestCredentialsFromJSONWithParams_SA(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes: []string{scope},
	}
	creds, err := CredentialsFromJSONWithParams(ctx, saJSONJWT, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "fake_project"; creds.ProjectID != want {
		t.Fatalf("got %q, want %q", creds.ProjectID, want)
	}
	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
}

func TestCredentialsFromJSONWithParams_SA_Params_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes:         []string{scope},
		UniverseDomain: universeDomain2,
	}
	creds, err := CredentialsFromJSONWithParams(ctx, saJSONJWT, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "fake_project"; creds.ProjectID != want {
		t.Fatalf("got %q, want %q", creds.ProjectID, want)
	}
	if creds.UniverseDomain() != universeDomain2 {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), universeDomain2)
	}
	if creds.UniverseDomain() != universeDomain2 {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), universeDomain2)
	}
}

func TestCredentialsFromJSONWithParams_SA_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes: []string{scope},
	}
	creds, err := CredentialsFromJSONWithParams(ctx, saJSONJWTUniverseDomain, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "fake_project"; creds.ProjectID != want {
		t.Fatalf("got %q, want %q", creds.ProjectID, want)
	}
	if creds.UniverseDomain() != universeDomain {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), universeDomain)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if got != universeDomain {
		t.Fatalf("got %q, want %q", got, universeDomain)
	}
}

func TestCredentialsFromJSONWithParams_SA_UniverseDomain_Params_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes:         []string{scope},
		UniverseDomain: universeDomain2,
	}
	creds, err := CredentialsFromJSONWithParams(ctx, saJSONJWTUniverseDomain, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "fake_project"; creds.ProjectID != want {
		t.Fatalf("got %q, want %q", creds.ProjectID, want)
	}
	if creds.UniverseDomain() != universeDomain2 {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), universeDomain2)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if got != universeDomain2 {
		t.Fatalf("got %q, want %q", got, universeDomain2)
	}
}

func TestCredentialsFromJSONWithParams_User(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes: []string{scope},
	}
	creds, err := CredentialsFromJSONWithParams(ctx, userJSON, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if want := "googleapis.com"; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestCredentialsFromJSONWithParams_User_Params_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes:         []string{scope},
		UniverseDomain: universeDomain2,
	}
	creds, err := CredentialsFromJSONWithParams(ctx, userJSON, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if want := "googleapis.com"; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestCredentialsFromJSONWithParams_User_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes: []string{scope},
	}
	creds, err := CredentialsFromJSONWithParams(ctx, userJSONUniverseDomain, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if want := "googleapis.com"; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestCredentialsFromJSONWithParams_User_UniverseDomain_Params_UniverseDomain(t *testing.T) {
	ctx := context.Background()
	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes:         []string{scope},
		UniverseDomain: universeDomain2,
	}
	creds, err := CredentialsFromJSONWithParams(ctx, userJSONUniverseDomain, params)
	if err != nil {
		t.Fatal(err)
	}

	if want := "googleapis.com"; creds.UniverseDomain() != want {
		t.Fatalf("got %q, want %q", creds.UniverseDomain(), want)
	}
	got, err := creds.GetUniverseDomain()
	if err != nil {
		t.Fatal(err)
	}
	if want := "googleapis.com"; got != want {
		t.Fatalf("got %q, want %q", got, want)
	}
}

func TestComputeUniverseDomain(t *testing.T) {
	universeDomainPath := "/computeMetadata/v1/universe/universe_domain"
	universeDomainResponseBody := "example.com"
	var requests int
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		requests++
		if r.URL.Path != universeDomainPath {
			t.Errorf("bad path, got %s, want %s", r.URL.Path, universeDomainPath)
		}
		if requests > 1 {
			t.Errorf("too many requests, got %d, want 1", requests)
		}
		w.Write([]byte(universeDomainResponseBody))
	}))
	defer s.Close()
	t.Setenv("GCE_METADATA_HOST", strings.TrimPrefix(s.URL, "http://"))

	scope := "https://www.googleapis.com/auth/cloud-platform"
	params := CredentialsParams{
		Scopes: []string{scope},
	}
	universeDomainProvider := func() (string, error) {
		universeDomain, err := metadata.Get("universe/universe_domain")
		if err != nil {
			return "", err
		}
		return universeDomain, nil
	}
	// Copied from FindDefaultCredentialsWithParams, metadata.OnGCE() = true block
	creds := &Credentials{
		ProjectID:              "fake_project",
		TokenSource:            computeTokenSource("", params.EarlyTokenRefresh, params.Scopes...),
		UniverseDomainProvider: universeDomainProvider,
		universeDomain:         params.UniverseDomain, // empty
	}
	c := make(chan bool)
	go func() {
		got, err := creds.GetUniverseDomain() // First conflicting access.
		if err != nil {
			t.Error(err)
		}
		if want := universeDomainResponseBody; got != want {
			t.Errorf("got %q, want %q", got, want)
		}
		c <- true
	}()
	got, err := creds.GetUniverseDomain() // Second conflicting (and potentially uncached) access.
	<-c
	if err != nil {
		t.Error(err)
	}
	if want := universeDomainResponseBody; got != want {
		t.Errorf("got %q, want %q", got, want)
	}

}
