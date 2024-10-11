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

package gdch

import (
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/http"
	"net/http/httptest"
	"os"
	"strings"
	"testing"

	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
	"cloud.google.com/go/auth/internal/jwt"
)

func TestTokenProvider(t *testing.T) {
	aud := "http://sampele-aud.com/"
	b, err := os.ReadFile("../../../internal/testdata/gdch.json")
	if err != nil {
		t.Fatal(err)
	}
	f, err := credsfile.ParseGDCHServiceAccount(b)
	if err != nil {
		t.Fatal(err)
	}
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			t.Errorf("unexpected request method: %v", r.Method)
		}
		if err := r.ParseForm(); err != nil {
			t.Error(err)
		}
		parts := strings.Split(r.FormValue("subject_token"), ".")
		var header jwt.Header
		var claims jwt.Claims
		b, err = base64.RawURLEncoding.DecodeString(parts[0])
		if err != nil {
			t.Fatal(err)
		}
		if err := json.Unmarshal(b, &header); err != nil {
			t.Fatal(err)
		}
		b, err = base64.RawURLEncoding.DecodeString(parts[1])
		if err != nil {
			t.Fatal(err)
		}
		if err := json.Unmarshal(b, &claims); err != nil {
			t.Fatal(err)
		}

		if got := r.FormValue("audience"); got != aud {
			t.Errorf("got audience %v, want %v", got, GrantType)
		}
		if want := jwt.HeaderAlgRSA256; header.Algorithm != want {
			t.Errorf("got alg %q, want %q", header.Algorithm, want)
		}
		if want := jwt.HeaderType; header.Type != want {
			t.Errorf("got typ %q, want %q", header.Type, want)
		}
		if want := "abcdef1234567890"; header.KeyID != want {
			t.Errorf("got kid %q, want %q", header.KeyID, want)
		}

		if want := "system:serviceaccount:fake_project:sa_name"; claims.Iss != want {
			t.Errorf("got iss %q, want %q", claims.Iss, want)
		}
		if want := "system:serviceaccount:fake_project:sa_name"; claims.Sub != want {
			t.Errorf("got sub %q, want %q", claims.Sub, want)
		}
		if want := fmt.Sprintf("http://%s", r.Host); claims.Aud != want {
			t.Errorf("got aud %q, want %q", claims.Aud, want)
		}
		w.Write([]byte(`{
			"access_token": "a_fake_token",
			"token_type": "Bearer",
			"expires_in": 60
		}`))
	}))
	f.TokenURL = ts.URL
	f.CertPath = "../../../internal/testdata/cert.pem"
	b, err = json.Marshal(&f)
	if err != nil {
		t.Fatal(err)
	}

	if _, err := NewTokenProvider(f, &Options{}); err == nil {
		t.Fatal("STSAudience should be required")
	}

	cred, err := NewTokenProvider(f, &Options{
		STSAudience: aud,
		Client:      internal.CloneDefaultClient(),
	})
	if err != nil {
		t.Fatal(err)
	}

	tok, err := cred.Token(context.Background())
	if err != nil {
		t.Fatal(err)
	}
	if want := "a_fake_token"; tok.Value != want {
		t.Fatalf("got AccessToken %q, want %q", tok.Value, want)
	}
	if want := internal.TokenTypeBearer; tok.Type != want {
		t.Fatalf("got TokenType %q, want %q", tok.Type, want)
	}
}
