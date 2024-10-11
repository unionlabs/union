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

package cert

import (
	"bytes"
	"errors"
	"testing"
)

func TestSecureConnectSource_ConfigMissing(t *testing.T) {
	source, err := NewSecureConnectProvider("missing.json")
	if got, want := err, errSourceUnavailable; !errors.Is(err, errSourceUnavailable) {
		t.Fatalf("got %v, want %v", got, want)
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

func TestSecureConnectSource_GetClientCertificateSuccess(t *testing.T) {
	source, err := NewSecureConnectProvider("testdata/context_aware_metadata.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Fatal(err)
	}
	if cert.Certificate == nil {
		t.Fatal("got nil, want non-nil Certificate")
	}
	if cert.PrivateKey == nil {
		t.Fatal("got nil, want non-nil PrivateKey")
	}
}

func TestSecureConnectSource_GetClientCertificateFailure(t *testing.T) {
	source, err := NewSecureConnectProvider("testdata/context_aware_metadata_invalid_pem.json")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := source(nil); err == nil {
		t.Error("got nil, want non-nil err")
	}
}

func TestSecureConnectSource_ValidateMetadataSuccess(t *testing.T) {
	metadata := secureConnectMetadata{Cmd: []string{"cat", "testdata/testcert.pem"}}
	if err := validateMetadata(metadata); err != nil {
		t.Fatal(err)
	}
}

func TestSecureConnectSource_ValidateMetadataFailure(t *testing.T) {
	metadata := secureConnectMetadata{Cmd: []string{}}
	err := validateMetadata(metadata)
	if err == nil {
		t.Fatal("got  nil, want non-nil err")
	}
	if got, want := err.Error(), "empty cert_provider_command"; got != want {
		t.Errorf("got %v, want %v err", got, want)
	}
}

func TestSecureConnectSource_IsCertificateExpiredTrue(t *testing.T) {
	source, err := NewSecureConnectProvider("testdata/context_aware_metadata.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Fatal(err)
	}
	if !isCertificateExpired(cert) {
		t.Error("isCertificateExpired() = false, want true")
	}
}

func TestSecureConnectSource_IsCertificateExpiredFalse(t *testing.T) {
	source, err := NewSecureConnectProvider("testdata/context_aware_metadata_nonexpiring_pem.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Fatal(err)
	}
	if isCertificateExpired(cert) {
		t.Error("isCertificateExpired() = true, want false")
	}
}

func TestCertificateCaching(t *testing.T) {
	source := secureConnectSource{metadata: secureConnectMetadata{Cmd: []string{"cat", "testdata/nonexpiring.pem"}}}
	cert, err := source.getClientCertificate(nil)
	if err != nil {
		t.Fatal(err)
	}
	if cert == nil {
		t.Fatal("getClientCertificate() = nil, want non-nil cert")
	}
	if source.cachedCert == nil {
		t.Fatal("got nil, want non-nil cachedCert")
	}
	if got, want := source.cachedCert.Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Fatalf("got %v, want %v cached Certificate", got, want)
	}
	if got, want := source.cachedCert.PrivateKey, cert.PrivateKey; got != want {
		t.Fatalf("got %v, want %v cached PrivateKey", got, want)
	}
}
