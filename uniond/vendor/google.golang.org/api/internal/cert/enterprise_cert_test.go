// Copyright 2022 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.
package cert

import (
	"errors"
	"testing"
)

func TestEnterpriseCertificateProxySource_ConfigMissing(t *testing.T) {
	source, err := NewEnterpriseCertificateProxySource("missing.json")
	if got, want := err, errSourceUnavailable; !errors.Is(err, errSourceUnavailable) {
		t.Fatalf("NewEnterpriseCertificateProxySource: with missing config; got %v, want %v err", got, want)
	}
	if source != nil {
		t.Errorf("NewEnterpriseCertificateProxySource: with missing config; got %v, want nil source", source)
	}
}

// This test launches a mock signer binary "test_signer.go" that uses a valid pem file.
func TestEnterpriseCertificateProxySource_GetClientCertificateSuccess(t *testing.T) {
	source, err := NewEnterpriseCertificateProxySource("testdata/certificate_config.json")
	if err != nil {
		t.Fatal(err)
	}
	cert, err := source(nil)
	if err != nil {
		t.Fatal(err)
	}
	if cert.Certificate == nil {
		t.Error("getClientCertificate: got nil, want non-nil Certificate")
	}
	if cert.PrivateKey == nil {
		t.Error("getClientCertificate: got nil, want non-nil PrivateKey")
	}
}

// This test launches a mock signer binary "test_signer.go" that uses an invalid pem file.
func TestEnterpriseCertificateProxySource_InitializationFailure(t *testing.T) {
	_, err := NewEnterpriseCertificateProxySource("testdata/certificate_config_invalid_pem.json")
	if err == nil {
		t.Error("NewEnterpriseCertificateProxySource: got nil, want non-nil err")
	}
}
