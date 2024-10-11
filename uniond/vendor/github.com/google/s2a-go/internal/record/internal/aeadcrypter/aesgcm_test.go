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

// getGCMCryptoPair outputs a sender/receiver pair on AES-GCM.
func getGCMCryptoPair(key []byte, t *testing.T) (S2AAEADCrypter, S2AAEADCrypter) {
	sender, err := NewAESGCM(key)
	if err != nil {
		t.Fatalf("NewAESGCM(ClientSide, key) = %v", err)
	}
	receiver, err := NewAESGCM(key)
	if err != nil {
		t.Fatalf("NewAESGCM(ServerSide, key) = %v", err)
	}
	return sender, receiver
}

// wycheProofTestVectorFilter filters out unsupported wycheproof test vectors.
func wycheProofTestVectorFilter(testGroup testutil.TestGroup) bool {
	// Filter these test groups out, since they are not supported in our
	// implementation of AES-GCM.
	return testGroup.IVSize != 96 ||
		(testGroup.KeySize != 128 && testGroup.KeySize != 256) ||
		testGroup.TagSize != 128
}

func testGCMEncryptionDecryption(sender S2AAEADCrypter, receiver S2AAEADCrypter, tc *testutil.CryptoTestVector, t *testing.T) {
	// ciphertext is: encrypted text + tag.
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

func testGCMEncryptRoundtrip(sender S2AAEADCrypter, receiver S2AAEADCrypter, t *testing.T) {
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

// Test encrypt and decrypt using an invalid key size.
func TestAESGCMInvalidKeySize(t *testing.T) {
	// Use 17 bytes, which is invalid
	key := make([]byte, 17)
	if _, err := NewAESGCM(key); err == nil {
		t.Error("expected an error when using invalid key size")
	}
}

// Test encrypt and decrypt on roundtrip messages for AES-GCM.
func TestAESGCMEncryptRoundtrip(t *testing.T) {
	for _, keySize := range []int{AES128GCMKeySize, AES256GCMKeySize} {
		t.Run(fmt.Sprintf("keySize=%d", keySize), func(t *testing.T) {
			key := make([]byte, keySize)
			sender, receiver := getGCMCryptoPair(key, t)
			testGCMEncryptRoundtrip(sender, receiver, t)
		})
	}
}

// Test encrypt and decrypt using test vectors for aes128gcm.
func TestAESGCMEncrypt(t *testing.T) {
	for _, tc := range []testutil.CryptoTestVector{
		{
			Desc:   "nil plaintext and ciphertext",
			Key:    testutil.Dehex("11754cd72aec309bf52f7687212e8957"),
			Tag:    testutil.Dehex("250327c674aaf477aef2675748cf6971"),
			Nonce:  testutil.Dehex("3c819d9a9bed087615030b65"),
			Result: testutil.ValidResult,
		},
		{
			Desc:       "invalid nonce size",
			Key:        testutil.Dehex("ab72c77b97cb5fe9a382d9fe81ffdbed"),
			Plaintext:  testutil.Dehex("007c5e5b3e59df24a7c355584fc1518d"),
			Ciphertext: testutil.Dehex("0e1bde206a07a9c2c1b65300f8c64997"),
			Tag:        testutil.Dehex("2b4401346697138c7a4891ee59867d0c"),
			Nonce:      testutil.Dehex("00"),
			Result:     testutil.InvalidResult,
		},
		{
			Desc:        "nil plaintext and ciphertext with dst allocation",
			Key:         testutil.Dehex("11754cd72aec309bf52f7687212e8957"),
			Tag:         testutil.Dehex("250327c674aaf477aef2675748cf6971"),
			Nonce:       testutil.Dehex("3c819d9a9bed087615030b65"),
			Result:      testutil.ValidResult,
			AllocateDst: true,
		},
		{
			Desc:       "basic test 1",
			Key:        testutil.Dehex("7fddb57453c241d03efbed3ac44e371c"),
			Plaintext:  testutil.Dehex("d5de42b461646c255c87bd2962d3b9a2"),
			Ciphertext: testutil.Dehex("2ccda4a5415cb91e135c2a0f78c9b2fd"),
			Tag:        testutil.Dehex("b36d1df9b9d5e596f83e8b7f52971cb3"),
			Nonce:      testutil.Dehex("ee283a3fc75575e33efd4887"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "basic test 2",
			Key:        testutil.Dehex("ab72c77b97cb5fe9a382d9fe81ffdbed"),
			Plaintext:  testutil.Dehex("007c5e5b3e59df24a7c355584fc1518d"),
			Ciphertext: testutil.Dehex("0e1bde206a07a9c2c1b65300f8c64997"),
			Tag:        testutil.Dehex("2b4401346697138c7a4891ee59867d0c"),
			Nonce:      testutil.Dehex("54cc7dc2c37ec006bcc6d1da"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:        "basic dst allocation 1",
			Key:         testutil.Dehex("7fddb57453c241d03efbed3ac44e371c"),
			Plaintext:   testutil.Dehex("d5de42b461646c255c87bd2962d3b9a2"),
			Ciphertext:  testutil.Dehex("2ccda4a5415cb91e135c2a0f78c9b2fd"),
			Tag:         testutil.Dehex("b36d1df9b9d5e596f83e8b7f52971cb3"),
			Nonce:       testutil.Dehex("ee283a3fc75575e33efd4887"),
			Result:      testutil.ValidResult,
			AllocateDst: true,
		},
		{
			Desc:        "basic dst allocation 2",
			Key:         testutil.Dehex("ab72c77b97cb5fe9a382d9fe81ffdbed"),
			Plaintext:   testutil.Dehex("007c5e5b3e59df24a7c355584fc1518d"),
			Ciphertext:  testutil.Dehex("0e1bde206a07a9c2c1b65300f8c64997"),
			Tag:         testutil.Dehex("2b4401346697138c7a4891ee59867d0c"),
			Nonce:       testutil.Dehex("54cc7dc2c37ec006bcc6d1da"),
			Result:      testutil.ValidResult,
			AllocateDst: true,
		},
		{
			Desc:        "basic dst allocation 3",
			Key:         testutil.Dehex("5b9604fe14eadba931b0ccf34843dab9"),
			Plaintext:   testutil.Dehex("001d0c231287c1182784554ca3a21908"),
			Ciphertext:  testutil.Dehex("26073cc1d851beff176384dc9896d5ff"),
			Tag:         testutil.Dehex("0a3ea7a5487cb5f7d70fb6c58d038554"),
			Nonce:       testutil.Dehex("028318abc1824029138141a2"),
			Result:      testutil.ValidResult,
			AllocateDst: true,
		},
	} {
		t.Run(fmt.Sprintf("%s", tc.Desc), func(t *testing.T) {
			sender, receiver := getGCMCryptoPair(tc.Key, t)
			testGCMEncryptionDecryption(sender, receiver, &tc, t)
		})
	}
}

func TestWycheProofTestVectors(t *testing.T) {
	for _, tc := range testutil.ParseWycheProofTestVectors("testdata/aes_gcm_wycheproof.json", wycheProofTestVectorFilter, t) {
		t.Run(fmt.Sprintf("%d/%s", tc.ID, tc.Desc), func(t *testing.T) {
			// Test encryption and decryption for AES-GCM.
			sender, receiver := getGCMCryptoPair(tc.Key, t)
			testGCMEncryptionDecryption(sender, receiver, &tc, t)
		})
	}
}

// Test AES-GCM with NIST and IEEE test vectors.
func TestAESGCMNISTAndIEEE(t *testing.T) {
	// NIST vectors from:
	// http://csrc.nist.gov/groups/ST/toolkit/BCM/documents/proposedmodes/gcm/gcm-revised-spec.pdf
	// IEEE vectors from:
	// http://www.ieee802.org/1/files/public/docs2011/bn-randall-test-vectors-0511-v1.pdf
	for _, tc := range []testutil.CryptoTestVector{
		{
			Desc:       "NIST test vector 1",
			Key:        testutil.Dehex("00000000000000000000000000000000"),
			Nonce:      testutil.Dehex("000000000000000000000000"),
			Aad:        testutil.Dehex(""),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("58e2fccefa7e3061367f1d57a4e7455a"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "NIST test vector 2",
			Key:        testutil.Dehex("00000000000000000000000000000000"),
			Nonce:      testutil.Dehex("000000000000000000000000"),
			Aad:        testutil.Dehex(""),
			Plaintext:  testutil.Dehex("00000000000000000000000000000000"),
			Ciphertext: testutil.Dehex("0388dace60b6a392f328c2b971b2fe78ab6e47d42cec13bdf53a67b21257bddf"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "NIST test vector 3",
			Key:        testutil.Dehex("feffe9928665731c6d6a8f9467308308"),
			Nonce:      testutil.Dehex("cafebabefacedbaddecaf888"),
			Aad:        testutil.Dehex(""),
			Plaintext:  testutil.Dehex("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255"),
			Ciphertext: testutil.Dehex("42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f59854d5c2af327cd64a62cf35abd2ba6fab4"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "NIST test vector 4",
			Key:        testutil.Dehex("feffe9928665731c6d6a8f9467308308"),
			Nonce:      testutil.Dehex("cafebabefacedbaddecaf888"),
			Aad:        testutil.Dehex("feedfacedeadbeeffeedfacedeadbeefabaddad2"),
			Plaintext:  testutil.Dehex("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39"),
			Ciphertext: testutil.Dehex("42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e0915bc94fbc3221a5db94fae95ae7121a47"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.1.1 54-byte auth",
			Key:        testutil.Dehex("ad7a2bd03eac835a6f620fdcb506b345"),
			Nonce:      testutil.Dehex("12153524c0895e81b2c28465"),
			Aad:        testutil.Dehex("d609b1f056637a0d46df998d88e5222ab2c2846512153524c0895e8108000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233340001"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("f09478a9b09007d06f46e9b6a1da25dd"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.1.2 54-byte auth",
			Key:        testutil.Dehex("e3c08a8f06c6e3ad95a70557b23f75483ce33021a9c72b7025666204c69c0b72"),
			Nonce:      testutil.Dehex("12153524c0895e81b2c28465"),
			Aad:        testutil.Dehex("d609b1f056637a0d46df998d88e5222ab2c2846512153524c0895e8108000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233340001"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("2f0bc5af409e06d609ea8b7d0fa5ea50"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.2.1 60-byte crypt",
			Key:        testutil.Dehex("ad7a2bd03eac835a6f620fdcb506b345"),
			Nonce:      testutil.Dehex("12153524c0895e81b2c28465"),
			Aad:        testutil.Dehex("d609b1f056637a0d46df998d88e52e00b2c2846512153524c0895e81"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a0002"),
			Ciphertext: testutil.Dehex("701afa1cc039c0d765128a665dab69243899bf7318ccdc81c9931da17fbe8edd7d17cb8b4c26fc81e3284f2b7fba713d4f8d55e7d3f06fd5a13c0c29b9d5b880"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.2.2 60-byte crypt",
			Key:        testutil.Dehex("e3c08a8f06c6e3ad95a70557b23f75483ce33021a9c72b7025666204c69c0b72"),
			Nonce:      testutil.Dehex("12153524c0895e81b2c28465"),
			Aad:        testutil.Dehex("d609b1f056637a0d46df998d88e52e00b2c2846512153524c0895e81"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a0002"),
			Ciphertext: testutil.Dehex("e2006eb42f5277022d9b19925bc419d7a592666c925fe2ef718eb4e308efeaa7c5273b394118860a5be2a97f56ab78365ca597cdbb3edb8d1a1151ea0af7b436"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.3.1 60-byte auth",
			Key:        testutil.Dehex("071b113b0ca743fecccf3d051f737382"),
			Nonce:      testutil.Dehex("f0761e8dcd3d000176d457ed"),
			Aad:        testutil.Dehex("e20106d7cd0df0761e8dcd3d88e5400076d457ed08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a0003"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("0c017bc73b227dfcc9bafa1c41acc353"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.3.2 60-byte auth",
			Key:        testutil.Dehex("691d3ee909d7f54167fd1ca0b5d769081f2bde1aee655fdbab80bd5295ae6be7"),
			Nonce:      testutil.Dehex("f0761e8dcd3d000176d457ed"),
			Aad:        testutil.Dehex("e20106d7cd0df0761e8dcd3d88e5400076d457ed08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a0003"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("35217c774bbc31b63166bcf9d4abed07"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.4.1 54-byte crypt",
			Key:        testutil.Dehex("071b113b0ca743fecccf3d051f737382"),
			Nonce:      testutil.Dehex("f0761e8dcd3d000176d457ed"),
			Aad:        testutil.Dehex("e20106d7cd0df0761e8dcd3d88e54c2a76d457ed"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233340004"),
			Ciphertext: testutil.Dehex("13b4c72b389dc5018e72a171dd85a5d3752274d3a019fbcaed09a425cd9b2e1c9b72eee7c9de7d52b3f3d6a5284f4a6d3fe22a5d6c2b960494c3"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.4.2 54-byte crypt",
			Key:        testutil.Dehex("691d3ee909d7f54167fd1ca0b5d769081f2bde1aee655fdbab80bd5295ae6be7"),
			Nonce:      testutil.Dehex("f0761e8dcd3d000176d457ed"),
			Aad:        testutil.Dehex("e20106d7cd0df0761e8dcd3d88e54c2a76d457ed"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f30313233340004"),
			Ciphertext: testutil.Dehex("c1623f55730c93533097addad25664966125352b43adacbd61c5ef3ac90b5bee929ce4630ea79f6ce51912af39c2d1fdc2051f8b7b3c9d397ef2"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.5.1 65-byte auth",
			Key:        testutil.Dehex("013fe00b5f11be7f866d0cbbc55a7a90"),
			Nonce:      testutil.Dehex("7cfde9f9e33724c68932d612"),
			Aad:        testutil.Dehex("84c5d513d2aaf6e5bbd2727788e523008932d6127cfde9f9e33724c608000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f0005"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("217867e50c2dad74c28c3b50abdf695a"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.5.2 65-byte auth",
			Key:        testutil.Dehex("83c093b58de7ffe1c0da926ac43fb3609ac1c80fee1b624497ef942e2f79a823"),
			Nonce:      testutil.Dehex("7cfde9f9e33724c68932d612"),
			Aad:        testutil.Dehex("84c5d513d2aaf6e5bbd2727788e523008932d6127cfde9f9e33724c608000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f0005"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("6ee160e8faeca4b36c86b234920ca975"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE  2.6.1 61-byte crypt",
			Key:        testutil.Dehex("013fe00b5f11be7f866d0cbbc55a7a90"),
			Nonce:      testutil.Dehex("7cfde9f9e33724c68932d612"),
			Aad:        testutil.Dehex("84c5d513d2aaf6e5bbd2727788e52f008932d6127cfde9f9e33724c6"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b0006"),
			Ciphertext: testutil.Dehex("3a4de6fa32191014dbb303d92ee3a9e8a1b599c14d22fb080096e13811816a3c9c9bcf7c1b9b96da809204e29d0e2a7642bfd310a4837c816ccfa5ac23ab003988"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.6.2 61-byte crypt",
			Key:        testutil.Dehex("83c093b58de7ffe1c0da926ac43fb3609ac1c80fee1b624497ef942e2f79a823"),
			Nonce:      testutil.Dehex("7cfde9f9e33724c68932d612"),
			Aad:        testutil.Dehex("84c5d513d2aaf6e5bbd2727788e52f008932d6127cfde9f9e33724c6"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b0006"),
			Ciphertext: testutil.Dehex("110222ff8050cbece66a813ad09a73ed7a9a089c106b959389168ed6e8698ea902eb1277dbec2e68e473155a15a7daeed4a10f4e05139c23df00b3aadc71f0596a"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.7.1 79-byte crypt",
			Key:        testutil.Dehex("88ee087fd95da9fbf6725aa9d757b0cd"),
			Nonce:      testutil.Dehex("7ae8e2ca4ec500012e58495c"),
			Aad:        testutil.Dehex("68f2e77696ce7ae8e2ca4ec588e541002e58495c08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d0007"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("07922b8ebcf10bb2297588ca4c614523"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.7.2 79-byte crypt",
			Key:        testutil.Dehex("4c973dbc7364621674f8b5b89e5c15511fced9216490fb1c1a2caa0ffe0407e5"),
			Nonce:      testutil.Dehex("7ae8e2ca4ec500012e58495c"),
			Aad:        testutil.Dehex("68f2e77696ce7ae8e2ca4ec588e541002e58495c08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d0007"),
			Plaintext:  testutil.Dehex(""),
			Ciphertext: testutil.Dehex("00bda1b7e87608bcbf470f12157f4c07"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.8.1 61-byte crypt",
			Key:        testutil.Dehex("88ee087fd95da9fbf6725aa9d757b0cd"),
			Nonce:      testutil.Dehex("7ae8e2ca4ec500012e58495c"),
			Aad:        testutil.Dehex("68f2e77696ce7ae8e2ca4ec588e54d002e58495c"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748490008"),
			Ciphertext: testutil.Dehex("c31f53d99e5687f7365119b832d2aae70741d593f1f9e2ab3455779b078eb8feacdfec1f8e3e5277f8180b43361f6512adb16d2e38548a2c719dba7228d84088f8757adb8aa788d8f65ad668be70e7"),
			Result:     testutil.ValidResult,
		},
		{
			Desc:       "IEEE 2.8.2 61-byte crypt",
			Key:        testutil.Dehex("4c973dbc7364621674f8b5b89e5c15511fced9216490fb1c1a2caa0ffe0407e5"),
			Nonce:      testutil.Dehex("7ae8e2ca4ec500012e58495c"),
			Aad:        testutil.Dehex("68f2e77696ce7ae8e2ca4ec588e54d002e58495c"),
			Plaintext:  testutil.Dehex("08000f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748490008"),
			Ciphertext: testutil.Dehex("ba8ae31bc506486d6873e4fce460e7dc57591ff00611f31c3834fe1c04ad80b66803afcf5b27e6333fa67c99da47c2f0ced68d531bd741a943cff7a6713bd02611cd7daa01d61c5c886dc1a8170107"),
			Result:     testutil.ValidResult,
		},
	} {
		t.Run(tc.Desc, func(t *testing.T) {
			// Test encryption and decryption for AES-GCM.
			sender, receiver := getGCMCryptoPair(tc.Key, t)
			testGCMEncryptionDecryption(sender, receiver, &tc, t)
		})
	}
}
