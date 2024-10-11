// Copyright 2023 Google LLC.
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

// Package windows contains a windows-specific client for accessing the ncrypt APIs directly,
// bypassing the RPC-mechanism of the universal client.
package windows

import (
	"crypto"
	"io"

	"github.com/googleapis/enterprise-certificate-proxy/internal/signer/windows/ncrypt"
)

// SecureKey is a public wrapper for the internal ncrypt implementation.
type SecureKey struct {
	key *ncrypt.Key
}

// CertificateChain returns the SecureKey's raw X509 cert chain. This contains the public key.
func (sk *SecureKey) CertificateChain() [][]byte {
	return sk.key.CertificateChain()
}

// Public returns the public key for this SecureKey.
func (sk *SecureKey) Public() crypto.PublicKey {
	return sk.key.Public()
}

// Sign signs a message digest, using the specified signer options.
func (sk *SecureKey) Sign(_ io.Reader, digest []byte, opts crypto.SignerOpts) (signed []byte, err error) {
	return sk.key.Sign(nil, digest, opts)
}

// Close frees up resources associated with the underlying key.
func (sk *SecureKey) Close() {
	sk.key.Close()
}

// NewSecureKey returns a handle to the first available certificate and private key pair in
// the specified Windows key store matching the filters.
func NewSecureKey(issuer string, store string, provider string) (*SecureKey, error) {
	k, err := ncrypt.Cred(issuer, store, provider)
	if err != nil {
		return nil, err
	}
	return &SecureKey{key: k}, nil
}
