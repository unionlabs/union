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

package credentials

import (
	"bytes"
	"context"
	"crypto/rand"
	"crypto/rsa"
	"crypto/x509"
	"encoding/base64"
	"encoding/json"
	"encoding/pem"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/auth/internal/jwt"
)

var jwtJSONKey = []byte(`{
	"private_key_id": "268f54e43a1af97cfc71731688434f45aca15c8b",
	"private_key": "super secret key",
	"client_email": "gopher@developer.gserviceaccount.com",
	"client_id": "gopher.apps.googleusercontent.com",
	"token_uri": "https://accountp.google.com/o/gophers/token",
	"type": "service_account",
	"audience": "https://testpervice.googleapis.com/"
  }`)

func TestDefaultCredentials_SelfSignedJSON(t *testing.T) {
	privateKey, jsonKey, err := setupFakeKey()
	if err != nil {
		t.Fatal(err)
	}
	tp, err := DetectDefault(&DetectOptions{
		CredentialsJSON:  jsonKey,
		Audience:         "audience",
		UseSelfSignedJWT: true,
	})
	if err != nil {
		t.Fatalf("DefaultCredentials(%s): %v", jsonKey, err)
	}

	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatalf("Token(): %v", err)
	}

	if got, want := tok.Type, "Bearer"; got != want {
		t.Errorf("Type = %q, want %q", got, want)
	}
	if got := tok.Expiry; tok.Expiry.Before(time.Now()) {
		t.Errorf("Expiry = %v, should not be expired", got)
	}

	err = jwt.VerifyJWS(tok.Value, &privateKey.PublicKey)
	if err != nil {
		t.Errorf("jwt.Verify(%q): %v", tok.Value, err)
	}

	claim, err := jwt.DecodeJWS(tok.Value)
	if err != nil {
		t.Fatalf("jwt.Decode(%q): %v", tok.Value, err)
	}

	if got, want := claim.Iss, "gopher@developer.gserviceaccount.com"; got != want {
		t.Errorf("Iss = %q, want %q", got, want)
	}
	if got, want := claim.Sub, "gopher@developer.gserviceaccount.com"; got != want {
		t.Errorf("Sub = %q, want %q", got, want)
	}
	if got, want := claim.Aud, "audience"; got != want {
		t.Errorf("Aud = %q, want %q", got, want)
	}

	// Finally, check the header private key.
	tokParts := strings.Split(tok.Value, ".")
	hdrJSON, err := base64.RawURLEncoding.DecodeString(tokParts[0])
	if err != nil {
		t.Fatalf("DecodeString(%q): %v", tokParts[0], err)
	}
	var hdr jwt.Header
	if err := json.Unmarshal(hdrJSON, &hdr); err != nil {
		t.Fatalf("json.Unmarshal(%q): %v", hdrJSON, err)
	}

	if got, want := hdr.KeyID, "268f54e43a1af97cfc71731688434f45aca15c8b"; got != want {
		t.Errorf("KeyID = %q, want %q", got, want)
	}
}

func TestDefaultCredentials_SelfSignedWithScope(t *testing.T) {
	privateKey, jsonKey, err := setupFakeKey()
	if err != nil {
		t.Fatal(err)
	}
	tp, err := DetectDefault(&DetectOptions{
		CredentialsJSON:  jsonKey,
		Scopes:           []string{"scope1", "scope2"},
		UseSelfSignedJWT: true,
	})
	if err != nil {
		t.Fatalf("DefaultCredentials(%s): %v", jsonKey, err)
	}

	tok, err := tp.Token(context.Background())
	if err != nil {
		t.Fatalf("Token(): %v", err)
	}

	if got, want := tok.Type, "Bearer"; got != want {
		t.Errorf("TokenType = %q, want %q", got, want)
	}
	if got := tok.Expiry; tok.Expiry.Before(time.Now()) {
		t.Errorf("Expiry = %v, should not be expired", got)
	}

	err = jwt.VerifyJWS(tok.Value, &privateKey.PublicKey)
	if err != nil {
		t.Errorf("jwt.Verify(%q): %v", tok.Value, err)
	}

	claim, err := jwt.DecodeJWS(tok.Value)
	if err != nil {
		t.Fatalf("jwt.Decode(%q): %v", tok.Value, err)
	}

	if got, want := claim.Iss, "gopher@developer.gserviceaccount.com"; got != want {
		t.Errorf("Iss = %q, want %q", got, want)
	}
	if got, want := claim.Sub, "gopher@developer.gserviceaccount.com"; got != want {
		t.Errorf("Sub = %q, want %q", got, want)
	}
	if got, want := claim.Scope, "scope1 scope2"; got != want {
		t.Errorf("Aud = %q, want %q", got, want)
	}

	// Finally, check the header private key.
	tokParts := strings.Split(tok.Value, ".")
	hdrJSON, err := base64.RawURLEncoding.DecodeString(tokParts[0])
	if err != nil {
		t.Fatalf("DecodeString(%q): %v", tokParts[0], err)
	}
	var hdr jwt.Header
	if err := json.Unmarshal(hdrJSON, &hdr); err != nil {
		t.Fatalf("json.Unmarshal(%q): %v", hdrJSON, err)
	}

	if got, want := hdr.KeyID, "268f54e43a1af97cfc71731688434f45aca15c8b"; got != want {
		t.Errorf("KeyID = %q, want %q", got, want)
	}
}

// setupFakeKey generates a key we can use in the test data.
func setupFakeKey() (*rsa.PrivateKey, []byte, error) {
	// Generate a key we can use in the test data.
	pk, err := rsa.GenerateKey(rand.Reader, 2048)
	if err != nil {
		return nil, nil, err
	}
	// Encode the key and substitute into our example JSON.
	enc := pem.EncodeToMemory(&pem.Block{
		Type:  "PRIVATE KEY",
		Bytes: x509.MarshalPKCS1PrivateKey(pk),
	})
	enc, err = json.Marshal(string(enc))
	if err != nil {
		return nil, nil, err
	}
	return pk, bytes.Replace(jwtJSONKey, []byte(`"super secret key"`), enc, 1), nil
}
