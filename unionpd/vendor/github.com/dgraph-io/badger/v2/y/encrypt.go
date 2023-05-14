/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package y

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
)

// XORBlock encrypts the given data with AES and XOR's with IV.
// Can be used for both encryption and decryption. IV is of
// AES block size.
func XORBlock(src, key, iv []byte) ([]byte, error) {
	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	stream := cipher.NewCTR(block, iv)
	dst := make([]byte, len(src))
	stream.XORKeyStream(dst, src)
	return dst, nil
}

// GenerateIV generates IV.
func GenerateIV() ([]byte, error) {
	iv := make([]byte, aes.BlockSize)
	_, err := rand.Read(iv)
	return iv, err
}
