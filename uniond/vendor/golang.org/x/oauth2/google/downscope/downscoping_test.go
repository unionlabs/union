// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package downscope

import (
	"context"
	"io/ioutil"
	"net/http"
	"net/http/httptest"
	"testing"

	"golang.org/x/oauth2"
)

var (
	standardReqBody  = "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Atoken-exchange&options=%7B%22accessBoundary%22%3A%7B%22accessBoundaryRules%22%3A%5B%7B%22availableResource%22%3A%22test1%22%2C%22availablePermissions%22%3A%5B%22Perm1%22%2C%22Perm2%22%5D%7D%5D%7D%7D&requested_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token&subject_token=Mellon&subject_token_type=urn%3Aietf%3Aparams%3Aoauth%3Atoken-type%3Aaccess_token"
	standardRespBody = `{"access_token":"Open Sesame","expires_in":432,"issued_token_type":"urn:ietf:params:oauth:token-type:access_token","token_type":"Bearer"}`
)

func Test_DownscopedTokenSource(t *testing.T) {
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("Unexpected request method, %v is found", r.Method)
		}
		if r.URL.String() != "/" {
			t.Errorf("Unexpected request URL, %v is found", r.URL)
		}
		body, err := ioutil.ReadAll(r.Body)
		if err != nil {
			t.Fatalf("Failed to read request body: %v", err)
		}
		if got, want := string(body), standardReqBody; got != want {
			t.Errorf("Unexpected exchange payload: got %v but want %v,", got, want)
		}
		w.Header().Set("Content-Type", "application/json")
		w.Write([]byte(standardRespBody))

	}))
	myTok := oauth2.Token{AccessToken: "Mellon"}
	tmpSrc := oauth2.StaticTokenSource(&myTok)
	rules := []AccessBoundaryRule{
		{
			AvailableResource:    "test1",
			AvailablePermissions: []string{"Perm1", "Perm2"},
		},
	}
	dts := downscopingTokenSource{
		ctx: context.Background(),
		config: DownscopingConfig{
			RootSource: tmpSrc,
			Rules:      rules,
		},
		identityBindingEndpoint: ts.URL,
	}
	_, err := dts.Token()
	if err != nil {
		t.Fatalf("NewDownscopedTokenSource failed with error: %v", err)
	}
}

func Test_DownscopingConfig(t *testing.T) {
	tests := []struct {
		universeDomain string
		want           string
	}{
		{"", "https://sts.googleapis.com/v1/token"},
		{"googleapis.com", "https://sts.googleapis.com/v1/token"},
		{"example.com", "https://sts.example.com/v1/token"},
	}
	for _, tt := range tests {
		c := DownscopingConfig{
			UniverseDomain: tt.universeDomain,
		}
		if got := c.identityBindingEndpoint(); got != tt.want {
			t.Errorf("got %q, want %q", got, tt.want)
		}
	}
}
