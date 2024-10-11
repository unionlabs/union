// Copyright 2022 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
package cert

import (
	"bytes"
	"errors"
	"testing"
)

func TestSecureConnectSource_ConfigMissing(t *testing.T) {
	source, err := NewSecureConnectSource("missing.json")
	if got, want := err, errSourceUnavailable; !errors.Is(err, errSourceUnavailable) {
		t.Fatalf("NewSecureConnectSource: with missing config; got %v, want %v err", got, want)
	}
	if source != nil {
		t.Errorf("NewSecureConnectSource: with missing config; got %v, want nil source", source)
	}
}

func TestSecureConnectSource_GetClientCertificateSuccess(t *testing.T) {
	source, err := NewSecureConnectSource("testdata/context_aware_metadata.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Error(err)
	}
	if cert.Certificate == nil {
		t.Error("getClientCertificate: got nil, want non-nil Certificate")
	}
	if cert.PrivateKey == nil {
		t.Error("getClientCertificate: got nil, want non-nil PrivateKey")
	}
}

func TestSecureConnectSource_GetClientCertificateFailure(t *testing.T) {
	source, err := NewSecureConnectSource("testdata/context_aware_metadata_invalid_pem.json")
	if err != nil {
		t.Fatal(err)
	}
	_, err = source(nil)
	if err == nil {
		t.Error("getClientCertificate: got nil, want non-nil err")
	}
}

func TestSecureConnectSource_ValidateMetadataSuccess(t *testing.T) {
	metadata := secureConnectMetadata{Cmd: []string{"cat", "testdata/testcert.pem"}}
	err := validateMetadata(metadata)
	if err != nil {
		t.Error(err)
	}
}

func TestSecureConnectSource_ValidateMetadataFailure(t *testing.T) {
	metadata := secureConnectMetadata{Cmd: []string{}}
	err := validateMetadata(metadata)
	if err == nil {
		t.Error("validateMetadata: got nil, want non-nil err")
	}
	if got, want := err.Error(), "empty cert_provider_command"; got != want {
		t.Errorf("validateMetadata: got %v, want %v err", got, want)
	}
}

func TestSecureConnectSource_IsCertificateExpiredTrue(t *testing.T) {
	source, err := NewSecureConnectSource("testdata/context_aware_metadata.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Error(err)
	}
	if !isCertificateExpired(cert) {
		t.Error("isCertificateExpired: got false, want true")
	}
}

func TestSecureConnectSource_IsCertificateExpiredFalse(t *testing.T) {
	source, err := NewSecureConnectSource("testdata/context_aware_metadata_nonexpiring_pem.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Error(err)
	}
	if isCertificateExpired(cert) {
		t.Error("isCertificateExpired: got true, want false")
	}
}

func TestCertificateCaching(t *testing.T) {
	source := secureConnectSource{metadata: secureConnectMetadata{Cmd: []string{"cat", "testdata/nonexpiring.pem"}}}
	cert, err := source.getClientCertificate(nil)
	if err != nil {
		t.Fatal(err)
	}
	if cert == nil {
		t.Fatal("getClientCertificate: got nil, want non-nil cert")
	}
	if source.cachedCert == nil {
		t.Fatal("getClientCertificate: got nil, want non-nil cachedCert")
	}
	if got, want := source.cachedCert.Certificate[0], cert.Certificate[0]; !bytes.Equal(got, want) {
		t.Fatalf("getClientCertificate: got %v, want %v cached Certificate", got, want)
	}
	if got, want := source.cachedCert.PrivateKey, cert.PrivateKey; got != want {
		t.Fatalf("getClientCertificate: got %v, want %v cached PrivateKey", got, want)
	}
}
