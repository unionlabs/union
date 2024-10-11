// Copyright 2015 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package google

import (
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
)

var webJSONKey = []byte(`
{
    "web": {
        "auth_uri": "https://google.com/o/oauth2/auth",
        "client_secret": "3Oknc4jS_wA2r9i",
        "token_uri": "https://google.com/o/oauth2/token",
        "client_email": "222-nprqovg5k43uum874cs9osjt2koe97g8@developer.gserviceaccount.com",
        "redirect_uris": ["https://www.example.com/oauth2callback"],
        "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/222-nprqovg5k43uum874cs9osjt2koe97g8@developer.gserviceaccount.com",
        "client_id": "222-nprqovg5k43uum874cs9osjt2koe97g8.apps.googleusercontent.com",
        "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
        "javascript_origins": ["https://www.example.com"]
    }
}`)

var installedJSONKey = []byte(`{
  "installed": {
      "client_id": "222-installed.apps.googleusercontent.com",
      "redirect_uris": ["https://www.example.com/oauth2callback"]
    }
}`)

var jwtJSONKey = []byte(`{
  "private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
  "private_key": "super secret key",
  "client_email": "gopher@developer.gserviceaccount.com",
  "client_id": "gopher.apps.googleusercontent.com",
  "token_uri": "https://accounts.google.com/o/gophers/token",
  "type": "service_account",
  "audience": "https://testservice.googleapis.com/"
}`)

var jwtJSONKeyNoTokenURL = []byte(`{
  "private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
  "private_key": "super secret key",
  "client_email": "gopher@developer.gserviceaccount.com",
  "client_id": "gopher.apps.googleusercontent.com",
  "type": "service_account"
}`)

var jwtJSONKeyNoAudience = []byte(`{
  "private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
  "private_key": "super secret key",
  "client_email": "gopher@developer.gserviceaccount.com",
  "client_id": "gopher.apps.googleusercontent.com",
  "token_uri": "https://accounts.google.com/o/gophers/token",
  "type": "service_account"
}`)

func TestConfigFromJSON(t *testing.T) {
	conf, err := ConfigFromJSON(webJSONKey, "scope1", "scope2")
	if err != nil {
		t.Error(err)
	}
	if got, want := conf.ClientID, "222-nprqovg5k43uum874cs9osjt2koe97g8.apps.googleusercontent.com"; got != want {
		t.Errorf("ClientID = %q; want %q", got, want)
	}
	if got, want := conf.ClientSecret, "3Oknc4jS_wA2r9i"; got != want {
		t.Errorf("ClientSecret = %q; want %q", got, want)
	}
	if got, want := conf.RedirectURL, "https://www.example.com/oauth2callback"; got != want {
		t.Errorf("RedictURL = %q; want %q", got, want)
	}
	if got, want := strings.Join(conf.Scopes, ","), "scope1,scope2"; got != want {
		t.Errorf("Scopes = %q; want %q", got, want)
	}
	if got, want := conf.Endpoint.AuthURL, "https://google.com/o/oauth2/auth"; got != want {
		t.Errorf("AuthURL = %q; want %q", got, want)
	}
	if got, want := conf.Endpoint.TokenURL, "https://google.com/o/oauth2/token"; got != want {
		t.Errorf("TokenURL = %q; want %q", got, want)
	}
}

func TestConfigFromJSON_Installed(t *testing.T) {
	conf, err := ConfigFromJSON(installedJSONKey)
	if err != nil {
		t.Error(err)
	}
	if got, want := conf.ClientID, "222-installed.apps.googleusercontent.com"; got != want {
		t.Errorf("ClientID = %q; want %q", got, want)
	}
}

func TestJWTConfigFromJSON(t *testing.T) {
	conf, err := JWTConfigFromJSON(jwtJSONKey, "scope1", "scope2")
	if err != nil {
		t.Fatal(err)
	}
	if got, want := conf.Email, "gopher@developer.gserviceaccount.com"; got != want {
		t.Errorf("Email = %q, want %q", got, want)
	}
	if got, want := string(conf.PrivateKey), "super secret key"; got != want {
		t.Errorf("PrivateKey = %q, want %q", got, want)
	}
	if got, want := conf.PrivateKeyID, "268f54e43a1af97cfc71731688434f45aca15c8b"; got != want {
		t.Errorf("PrivateKeyID = %q, want %q", got, want)
	}
	if got, want := strings.Join(conf.Scopes, ","), "scope1,scope2"; got != want {
		t.Errorf("Scopes = %q; want %q", got, want)
	}
	if got, want := conf.TokenURL, "https://accounts.google.com/o/gophers/token"; got != want {
		t.Errorf("TokenURL = %q; want %q", got, want)
	}
	if got, want := conf.Audience, "https://testservice.googleapis.com/"; got != want {
		t.Errorf("Audience = %q; want %q", got, want)
	}
}

func TestJWTConfigFromJSONNoTokenURL(t *testing.T) {
	conf, err := JWTConfigFromJSON(jwtJSONKeyNoTokenURL, "scope1", "scope2")
	if err != nil {
		t.Fatal(err)
	}
	if got, want := conf.TokenURL, "https://oauth2.googleapis.com/token"; got != want {
		t.Errorf("TokenURL = %q; want %q", got, want)
	}
}

func TestJWTConfigFromJSONNoAudience(t *testing.T) {
	conf, err := JWTConfigFromJSON(jwtJSONKeyNoAudience, "scope1", "scope2")
	if err != nil {
		t.Fatal(err)
	}
	if got, want := conf.Audience, ""; got != want {
		t.Errorf("Audience = %q; want %q", got, want)
	}
}

func TestComputeTokenSource(t *testing.T) {
	tokenPath := "/computeMetadata/v1/instance/service-accounts/default/token"
	tokenResponseBody := `{"access_token":"Sample.Access.Token","token_type":"Bearer","expires_in":3600}`
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != tokenPath {
			t.Errorf("got %s, want %s", r.URL.Path, tokenPath)
		}
		w.Write([]byte(tokenResponseBody))
	}))
	defer s.Close()
	t.Setenv("GCE_METADATA_HOST", strings.TrimPrefix(s.URL, "http://"))
	ts := ComputeTokenSource("")
	_, err := ts.Token()
	if err != nil {
		t.Errorf("ts.Token() = %v", err)
	}
}
