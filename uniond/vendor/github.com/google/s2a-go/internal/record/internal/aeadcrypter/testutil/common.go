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

// Package testutil is a collection of test utilities for the AEAD crypter.
package testutil

import (
	"encoding/hex"
)

// Constants indicating whether the test vector is valid or not.
const (
	ValidResult   = "valid"
	InvalidResult = "invalid"
)

// CryptoTestVector is a struct representing a test vector for an S2AAEADCrypter
// instance.
type CryptoTestVector struct {
	Desc                                        string
	ID                                          int
	Key, Plaintext, Ciphertext, Tag, Nonce, Aad []byte
	Result                                      string
	AllocateDst                                 bool
}

// TestVector is a struct for a WycheProof test vector.
type TestVector struct {
	TcID    int    `json:"tcId"`
	Comment string `json:"comment"`
	Key     string `json:"key"`
	IV      string `json:"iv"`
	Aad     string `json:"aad"`
	Msg     string `json:"msg"`
	Ct      string `json:"ct"`
	Tag     string `json:"tag"`
	Result  string `json:"result"`
}

// TestGroup is a struct for a WycheProof test group.
type TestGroup struct {
	IVSize  int          `json:"ivSize"`
	KeySize int          `json:"keySize"`
	TagSize int          `json:"tagSize"`
	Tests   []TestVector `json:"tests"`
}

// TestFile is a struct for a WycheProof test file.
type TestFile struct {
	TestGroups []TestGroup `json:"testGroups"`
}

// Dehex converts a byte string into a byte array.
func Dehex(s string) []byte {
	if len(s) == 0 {
		return make([]byte, 0)
	}
	b, err := hex.DecodeString(s)
	if err != nil {
		panic(err)
	}
	return b
}
