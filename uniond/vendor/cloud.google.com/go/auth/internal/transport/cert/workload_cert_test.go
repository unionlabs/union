// Copyright 2024 Google LLC
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

func TestWorkloadCertSource_ConfigMissing(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("missing.json")
	if got, want := err, errSourceUnavailable; !errors.Is(err, errSourceUnavailable) {
		t.Fatalf("got %v, want %v", got, want)
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

func TestWorkloadCertSource_EmptyConfig(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("testdata/certificate_config_workload_empty.json")
	if err == nil {
		t.Fatal("got nil, want non-nil error")
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

func TestWorkloadCertSource_MissingCert(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("testdata/certificate_config_workload_no_cert.json")
	if err == nil {
		t.Fatal("got nil, want non-nil error")
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

func TestWorkloadCertSource_MissingKey(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("testdata/certificate_config_workload_no_key.json")
	if err == nil {
		t.Fatal("got nil, want non-nil error")
	}
	if source != nil {
		t.Errorf("got %v, want nil source", source)
	}
}

func TestWorkloadCertSource_GetClientCertificateInvalidCert(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("testdata/certificate_config_workload_invalid_cert.json")
	if err != nil {
		t.Fatal(err)
	}
	_, err = source(nil)
	if err == nil {
		t.Fatal("got nil, want non-nil error")
	}
}

func TestWorkloadCertSource_GetClientCertificateSuccess(t *testing.T) {
	source, err := NewWorkloadX509CertProvider("testdata/certificate_config_workload.json")
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
