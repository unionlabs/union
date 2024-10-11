// Copyright 2020 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package externalaccount

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

var myURLToken = "testTokenValue"

func TestRetrieveURLSubjectToken_Text(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		if r.Header.Get("Metadata") != "True" {
			t.Errorf("Metadata header not properly included.")
		}
		w.Write([]byte("testTokenValue"))
	}))
	heads := make(map[string]string)
	heads["Metadata"] = "True"
	cs := CredentialSource{
		URL:     ts.URL,
		Format:  Format{Type: fileTypeText},
		Headers: heads,
	}
	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	out, err := base.subjectToken()
	if err != nil {
		t.Fatalf("retrieveSubjectToken() failed: %v", err)
	}
	if out != myURLToken {
		t.Errorf("got %v but want %v", out, myURLToken)
	}
}

// Checking that retrieveSubjectToken properly defaults to type text
func TestRetrieveURLSubjectToken_Untyped(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		w.Write([]byte("testTokenValue"))
	}))
	cs := CredentialSource{
		URL: ts.URL,
	}
	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	out, err := base.subjectToken()
	if err != nil {
		t.Fatalf("Failed to retrieve URL subject token: %v", err)
	}
	if out != myURLToken {
		t.Errorf("got %v but want %v", out, myURLToken)
	}
}

func TestRetrieveURLSubjectToken_JSON(t *testing.T) {
	type tokenResponse struct {
		TestToken string `json:"SubjToken"`
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if got, want := r.Method, "GET"; got != want {
			t.Errorf("got %v, but want %v", r.Method, want)
		}
		resp := tokenResponse{TestToken: "testTokenValue"}
		jsonResp, err := json.Marshal(resp)
		if err != nil {
			t.Errorf("Failed to marshal values: %v", err)
		}
		w.Write(jsonResp)
	}))
	cs := CredentialSource{
		URL:    ts.URL,
		Format: Format{Type: fileTypeJSON, SubjectTokenFieldName: "SubjToken"},
	}
	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	out, err := base.subjectToken()
	if err != nil {
		t.Fatalf("%v", err)
	}
	if out != myURLToken {
		t.Errorf("got %v but want %v", out, myURLToken)
	}
}

func TestURLCredential_CredentialSourceType(t *testing.T) {
	cs := CredentialSource{
		URL:    "http://example.com",
		Format: Format{Type: fileTypeText},
	}
	tfc := testFileConfig
	tfc.CredentialSource = &cs

	base, err := tfc.parse(context.Background())
	if err != nil {
		t.Fatalf("parse() failed %v", err)
	}

	if got, want := base.credentialSourceType(), "url"; got != want {
		t.Errorf("got %v but want %v", got, want)
	}
}
