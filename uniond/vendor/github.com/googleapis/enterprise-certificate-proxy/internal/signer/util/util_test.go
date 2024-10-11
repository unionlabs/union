// Copyright 2022 Google LLC.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package util

import (
	"testing"
)

func TestLoadConfig(t *testing.T) {
	config, err := LoadConfig("./test_data/certificate_config.json")
	// darwin
	if err != nil {
		t.Fatalf("LoadConfig error: %q", err)
	}
	want := "Google Endpoint Verification"
	if config.CertConfigs.MacOSKeychain.Issuer != want {
		t.Errorf("Expected issuer is %q, got: %q", want, config.CertConfigs.MacOSKeychain.Issuer)
	}

	// windows
	want = "enterprise_v1_corp_client"
	if config.CertConfigs.WindowsStore.Issuer != want {
		t.Errorf("Expected issuer is %q, got: %q", want, config.CertConfigs.WindowsStore.Issuer)
	}
	want = "MY"
	if config.CertConfigs.WindowsStore.Store != want {
		t.Errorf("Expected store is %q, got: %q", want, config.CertConfigs.WindowsStore.Store)
	}
	want = "current_user"
	if config.CertConfigs.WindowsStore.Provider != want {
		t.Errorf("Expected provider is %q, got: %q", want, config.CertConfigs.WindowsStore.Provider)
	}

	// pkcs11
	want = "0x1739427"
	if config.CertConfigs.PKCS11.Slot != want {
		t.Errorf("Expected slot is %v, got: %v", want, config.CertConfigs.PKCS11.Slot)
	}
	want = "gecc"
	if config.CertConfigs.PKCS11.Label != want {
		t.Errorf("Expected label is %v, got: %v", want, config.CertConfigs.PKCS11.Label)
	}
	want = "pkcs11_module.so"
	if config.CertConfigs.PKCS11.PKCS11Module != want {
		t.Errorf("Expected pkcs11_module is %v, got: %v", want, config.CertConfigs.PKCS11.PKCS11Module)
	}
	want = "0000"
	if config.CertConfigs.PKCS11.UserPin != want {
		t.Errorf("Expected user pin is %v, got: %v", want, config.CertConfigs.PKCS11.UserPin)
	}
}

func TestLoadConfigMissing(t *testing.T) {
	_, err := LoadConfig("./test_data/certificate_config_missing.json")
	if err == nil {
		t.Error("Expected error but got nil")
	}
}
