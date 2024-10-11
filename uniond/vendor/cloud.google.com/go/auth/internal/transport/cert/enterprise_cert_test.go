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
	"errors"
	"testing"
)

func TestEnterpriseCertificateProxySource_ConfigMissing(t *testing.T) {
	source, err := NewEnterpriseCertificateProxyProvider("missing.json")
	if got, want := err, errSourceUnavailable; !errors.Is(err, errSourceUnavailable) {
		t.Fatalf("got %v, want %v err", got, want)
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

// This test launches a mock signer binary "test_signer.go" that uses a valid pem file.
func TestEnterpriseCertificateProxySource_GetClientCertificateSuccess(t *testing.T) {
	source, err := NewEnterpriseCertificateProxyProvider("testdata/certificate_config.json")
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

// This test launches a mock signer binary "test_signer.go" that uses an invalid pem file.
func TestEnterpriseCertificateProxySource_InitializationFailure(t *testing.T) {
	_, err := NewEnterpriseCertificateProxyProvider("testdata/certificate_config_invalid_pem.json")
	if err == nil {
		t.Fatal("got nil, want non-nil err")
	}
}
