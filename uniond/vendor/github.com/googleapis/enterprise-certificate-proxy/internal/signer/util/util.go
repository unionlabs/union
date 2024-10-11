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

// Package util provides helper functions for the signer.
package util

import (
	"encoding/json"
	"io"
	"os"
)

// EnterpriseCertificateConfig contains parameters for initializing signer.
type EnterpriseCertificateConfig struct {
	CertConfigs CertConfigs `json:"cert_configs"`
}

// CertConfigs is a container for various OS-specific ECP Configs.
type CertConfigs struct {
	MacOSKeychain MacOSKeychain `json:"macos_keychain"`
	WindowsStore  WindowsStore  `json:"windows_store"`
	PKCS11        PKCS11        `json:"pkcs11"`
}

// MacOSKeychain contains keychain parameters describing the certificate to use.
type MacOSKeychain struct {
	Issuer string `json:"issuer"`
}

// WindowsStore contains Windows key store parameters describing the certificate to use.
type WindowsStore struct {
	Issuer   string `json:"issuer"`
	Store    string `json:"store"`
	Provider string `json:"provider"`
}

// PKCS11 contains PKCS#11 parameters describing the certificate to use.
type PKCS11 struct {
	Slot         string `json:"slot"`     // The hexadecimal representation of the uint36 slot ID. (ex:0x1739427)
	Label        string `json:"label"`    // The token label (ex: gecc)
	PKCS11Module string `json:"module"`   // The path to the pkcs11 module (shared lib)
	UserPin      string `json:"user_pin"` // Optional user pin to unlock the PKCS #11 module. If it is not defined or empty C_Login will not be called.
}

// LoadConfig retrieves the ECP config file.
func LoadConfig(configFilePath string) (config EnterpriseCertificateConfig, err error) {
	jsonFile, err := os.Open(configFilePath)
	if err != nil {
		return EnterpriseCertificateConfig{}, err
	}

	byteValue, err := io.ReadAll(jsonFile)
	if err != nil {
		return EnterpriseCertificateConfig{}, err
	}
	err = json.Unmarshal(byteValue, &config)
	if err != nil {
		return EnterpriseCertificateConfig{}, err
	}
	return config, nil
}
