/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package halfconn

import (
	"crypto/sha256"
	"crypto/sha512"
	"hash"
	"reflect"
	"testing"

	s2apb "github.com/google/s2a-go/internal/proto/common_go_proto"
	"github.com/google/s2a-go/internal/record/internal/aeadcrypter"
	"github.com/google/s2a-go/internal/record/internal/aeadcrypter/testutil"
)

func TestCiphersuites(t *testing.T) {
	for _, tc := range []struct {
		s2aProtoCiphersuite                   s2apb.Ciphersuite
		expectedCiphersuite                   ciphersuite
		key                                   []byte
		keySize, NonceSize, trafficSecretSize int
		hashFunction                          func() hash.Hash
		aeadCrypterConstructor                func([]byte) (aeadcrypter.S2AAEADCrypter, error)
	}{
		{
			s2aProtoCiphersuite:    s2apb.Ciphersuite_AES_128_GCM_SHA256,
			expectedCiphersuite:    &aesgcm128sha256{},
			key:                    testutil.Dehex("88ee087fd95da9fbf6725aa9d757b0cd"),
			keySize:                aeadcrypter.AES128GCMKeySize,
			NonceSize:              aeadcrypter.NonceSize,
			trafficSecretSize:      aeadcrypter.SHA256DigestSize,
			hashFunction:           sha256.New,
			aeadCrypterConstructor: aeadcrypter.NewAESGCM,
		},
		{
			s2aProtoCiphersuite:    s2apb.Ciphersuite_AES_256_GCM_SHA384,
			expectedCiphersuite:    &aesgcm256sha384{},
			key:                    testutil.Dehex("83c093b58de7ffe1c0da926ac43fb3609ac1c80fee1b624497ef942e2f79a823"),
			keySize:                aeadcrypter.AES256GCMKeySize,
			NonceSize:              aeadcrypter.NonceSize,
			trafficSecretSize:      aeadcrypter.SHA384DigestSize,
			hashFunction:           sha512.New384,
			aeadCrypterConstructor: aeadcrypter.NewAESGCM,
		},
		{
			s2aProtoCiphersuite:    s2apb.Ciphersuite_CHACHA20_POLY1305_SHA256,
			expectedCiphersuite:    &chachapolysha256{},
			key:                    testutil.Dehex("83c093b58de7ffe1c0da926ac43fb3609ac1c80fee1b624497ef942e2f79a823"),
			keySize:                aeadcrypter.Chacha20Poly1305KeySize,
			NonceSize:              aeadcrypter.NonceSize,
			trafficSecretSize:      aeadcrypter.SHA256DigestSize,
			hashFunction:           sha256.New,
			aeadCrypterConstructor: aeadcrypter.NewChachaPoly,
		},
	} {
		t.Run(tc.s2aProtoCiphersuite.String(), func(t *testing.T) {
			hc, err := newCiphersuite(tc.s2aProtoCiphersuite)
			if err != nil {
				t.Fatalf("newCiphersuite(%v) failed: %v", tc.s2aProtoCiphersuite, err)
			}
			if got, want := reflect.TypeOf(hc), reflect.TypeOf(tc.expectedCiphersuite); got != want {
				t.Fatalf("newCiphersuite(%v) = %v, want %v", tc.s2aProtoCiphersuite, got, want)
			}
			if got, want := hc.keySize(), tc.keySize; got != want {
				t.Errorf("hc.keySize() = %v, want %v", got, want)
			}
			if got, want := hc.nonceSize(), tc.NonceSize; got != want {
				t.Errorf("hc.nonceSize() = %v, want %v", got, want)
			}
			if got, want := hc.trafficSecretSize(), tc.trafficSecretSize; got != want {
				t.Errorf("hc.trafficSecretSize() = %v, want %v", got, want)
			}
			if got, want := reflect.TypeOf(hc.hashFunction()), reflect.TypeOf(tc.hashFunction); got != want {
				t.Errorf("hc.hashFunction() = %v, want %v", got, want)
			}
			aeadCrypter, err := hc.aeadCrypter(tc.key)
			if err != nil {
				t.Fatalf("hc.aeadCrypter(%v) failed: %v", tc.key, err)
			}
			tcAEADCrypter, err := tc.aeadCrypterConstructor(make([]byte, tc.keySize))
			if err != nil {
				t.Fatalf("tc.aeadCrypterConstructor(make([]byte, %v)) failed: %v", tc.keySize, err)
			}
			if got, want := reflect.TypeOf(aeadCrypter), reflect.TypeOf(tcAEADCrypter); got != want {
				t.Errorf("hc.aeadCrypter(%v) = %v, want %v", tc.key, got, want)
			}
		})
	}
}
