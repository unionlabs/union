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

package aeadcrypter

import (
	"fmt"
	"testing"

	"github.com/google/s2a-go/internal/record/internal/aeadcrypter/testutil"
)

// getChachaPolyCrypterPair outputs a sender/receiver pair of CHACHA-POLY AEAD crypters.
func getChachaPolyCrypterPair(key []byte, t *testing.T) (S2AAEADCrypter, S2AAEADCrypter) {
	sender, err := NewChachaPoly(key)
	if err != nil {
		t.Fatalf("NewChachaPoly(ClientSide, key) = %v", err)
	}
	receiver, err := NewChachaPoly(key)
	if err != nil {
		t.Fatalf("NewChachaPoly(ServerSide, key) = %v", err)
	}
	return sender, receiver
}

// wycheProofTestVectorFilter filters out unsupported wycheproof test vectors.
func wycheProofTestVectorFilterChachaPoly(testGroup testutil.TestGroup) bool {
	// Filter these test groups out, since they are not supported in our
	// implementation of Chacha-Poly.
	return testGroup.IVSize != 96 ||
		(testGroup.KeySize != 256) ||
		testGroup.TagSize != 128
}

func testChachaPolyEncryptionDecryption(sender S2AAEADCrypter, receiver S2AAEADCrypter, tc *testutil.CryptoTestVector, t *testing.T) {
	// Ciphertext is: encrypted text + tag.
	ciphertext := append(tc.Ciphertext, tc.Tag...)

	// Encrypt.
	var dst []byte
	if tc.AllocateDst {
		dst = make([]byte, len(tc.Plaintext)+sender.TagSize())
	}
	got, err := sender.Encrypt(dst[:0], tc.Plaintext, tc.Nonce, tc.Aad)
	if testutil.IsFailure(tc.Result, err, got, ciphertext) {
		t.Errorf("key=%v\nEncrypt(\n dst = %v\n plaintext = %v\n nonce = %v\n aad = %v\n) = (\n %v\n %v\n), want %v",
			tc.Key, dst[:0], tc.Plaintext, tc.Nonce, tc.Aad, got, err, ciphertext)
	}

	// Decrypt.
	got, err = receiver.Decrypt(nil, ciphertext, tc.Nonce, tc.Aad)
	if testutil.IsFailure(tc.Result, err, got, tc.Plaintext) {
		t.Errorf("key=%v\nDecrypt(\n dst = nil\n ciphertext = %v\n nonce = %v\n aad = %v\n) = (\n %v\n %v\n), want %v",
			tc.Key, ciphertext, tc.Nonce, tc.Aad, got, err, tc.Plaintext)
	}
}

func testChachaPolyEncryptRoundtrip(sender S2AAEADCrypter, receiver S2AAEADCrypter, t *testing.T) {
	// Construct a dummy nonce.
	nonce := make([]byte, NonceSize)

	// Encrypt.
	const plaintext = "This is plaintext."
	var err error
	// Reuse `buf` as both the input and output buffer. This is required to test
	// the case where the input and output buffers fully overlap.
	buf := []byte(plaintext)
	ciphertext, err := sender.Encrypt(buf[:0], buf, nonce, nil)
	if err != nil {
		t.Fatalf("Encrypt(%v, %v, %v, nil) failed: %v", buf[:0], buf, nonce, err)
	}

	// Decrypt first message.
	decryptedPlaintext, err := receiver.Decrypt(ciphertext[:0], ciphertext, nonce, nil)
	if err != nil {
		t.Fatalf("Decrypt(%v, %v, %v, nil) failed: %v", ciphertext[:0], ciphertext, nonce, err)
	}
	if string(decryptedPlaintext) != plaintext {
		t.Fatalf("Decrypt(%v, %v, %v, nil) = %v, want %v", ciphertext[:0], ciphertext, nonce, decryptedPlaintext, plaintext)
	}
}

// Test creating new Chacha-Poly AEAD crypter using an invalid key size.
func TestChachaPolyInvalidKeySize(t *testing.T) {
	// Use 1 + keySize, which is invalid.
	key := make([]byte, 1+Chacha20Poly1305KeySize)
	if _, err := NewChachaPoly(key); err == nil {
		t.Error("expected an error when using invalid key size")
	}
}

// Test Encrypt/Decrypt using an invalid nonce size.
func TestChachaPolyEncryptDecryptInvalidNonce(t *testing.T) {
	key := make([]byte, Chacha20Poly1305KeySize)
	crypter, err := NewChachaPoly(key)
	if err != nil {
		t.Fatalf("NewChachaPoly(keySize=%v) failed, err: %v", Chacha20Poly1305KeySize, err)
	}
	// Construct nonce with invalid size.
	nonce := make([]byte, 1)
	if _, err = crypter.Encrypt(nil, nil, nonce, nil); err == nil {
		t.Errorf("Encrypt should fail due to invalid nonce size")
	}
	if _, err = crypter.Decrypt(nil, nil, nonce, nil); err == nil {
		t.Fatalf("Decrypt should fail due to invalid nonce size")
	}
}

// Test encrypt and decrypt on roundtrip messages for Chacha-Poly.
func TestChachaPolyEncryptRoundtrip(t *testing.T) {
	for _, keySize := range []int{Chacha20Poly1305KeySize} {
		t.Run(fmt.Sprintf("keySize=%d", keySize), func(t *testing.T) {
			key := make([]byte, keySize)
			sender, receiver := getChachaPolyCrypterPair(key, t)
			testChachaPolyEncryptRoundtrip(sender, receiver, t)
		})
	}
}

func TestWycheProofTestVectorsChachaPoly(t *testing.T) {
	for _, tc := range testutil.ParseWycheProofTestVectors("testdata/chacha_poly_wycheproof.json", wycheProofTestVectorFilter, t) {
		t.Run(fmt.Sprintf("%d/%s", tc.ID, tc.Desc), func(t *testing.T) {
			// Test encryption and decryption for CHACHA-POLY.
			sender, receiver := getChachaPolyCrypterPair(tc.Key, t)
			testChachaPolyEncryptionDecryption(sender, receiver, &tc, t)
		})
	}
}

// Test ChachaPoly with RFC test vectors.
func TestChachaPolyRFC(t *testing.T) {
	for _, tc := range []testutil.CryptoTestVector{
		{
			Desc:       "RFC test vector 1",
			Key:        testutil.Dehex("808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f"),
			Nonce:      testutil.Dehex("070000004041424344454647"),
			Aad:        testutil.Dehex("50515253c0c1c2c3c4c5c6c7"),
			Plaintext:  testutil.Dehex("4c616469657320616e642047656e746c656d656e206f662074686520636c617373206f66202739393a204966204920636f756c64206f6666657220796f75206f6e6c79206f6e652074697020666f7220746865206675747572652c2073756e73637265656e20776f756c642062652069742e"),
			Ciphertext: testutil.Dehex("d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b3692ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc3ff4def08e4b7a9de576d26586cec64b61161ae10b594f09e26a7e902ecbd0600691"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "RFC test vector 2",
			Key:        testutil.Dehex("1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0"),
			Nonce:      testutil.Dehex("000000000102030405060708"),
			Aad:        testutil.Dehex("f33388860000000000004e91"),
			Plaintext:  testutil.Dehex("496e7465726e65742d4472616674732061726520647261667420646f63756d656e74732076616c696420666f722061206d6178696d756d206f6620736978206d6f6e74687320616e64206d617920626520757064617465642c207265706c616365642c206f72206f62736f6c65746564206279206f7468657220646f63756d656e747320617420616e792074696d652e20497420697320696e617070726f70726961746520746f2075736520496e7465726e65742d447261667473206173207265666572656e6365206d6174657269616c206f7220746f2063697465207468656d206f74686572207468616e206173202fe2809c776f726b20696e2070726f67726573732e2fe2809d"),
			Ciphertext: testutil.Dehex("64a0861575861af460f062c79be643bd5e805cfd345cf389f108670ac76c8cb24c6cfc18755d43eea09ee94e382d26b0bdb7b73c321b0100d4f03b7f355894cf332f830e710b97ce98c8a84abd0b948114ad176e008d33bd60f982b1ff37c8559797a06ef4f0ef61c186324e2b3506383606907b6a7c02b0f9f6157b53c867e4b9166c767b804d46a59b5216cde7a4e99040c5a40433225ee282a1b0a06c523eaf4534d7f83fa1155b0047718cbc546a0d072b04b3564eea1b422273f548271a0bb2316053fa76991955ebd63159434ecebb4e466dae5a1073a6727627097a1049e617d91d361094fa68f0ff77987130305beaba2eda04df997b714d6c6f2c29a6ad5cb4022b02709beead9d67890cbb22392336fea1851f38"),
			Result:     testutil.ValidResult,
		},
	} {
		t.Run(fmt.Sprintf("%s", tc.Desc), func(t *testing.T) {
			// Test encryption and decryption for Chacha-Poly.
			sender, receiver := getChachaPolyCrypterPair(tc.Key, t)
			testChachaPolyEncryptionDecryption(sender, receiver, &tc, t)
		})
	}
}
