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
package badger

import (
	"math/rand"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
)

func getRegistryTestOptions(dir string, key []byte) KeyRegistryOptions {
	return KeyRegistryOptions{
		Dir:           dir,
		EncryptionKey: key,
		ReadOnly:      false,
	}
}
func TestBuildRegistry(t *testing.T) {
	encryptionKey := make([]byte, 32)
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)

	_, err = rand.Read(encryptionKey)
	require.NoError(t, err)
	opt := getRegistryTestOptions(dir, encryptionKey)

	kr, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	dk, err := kr.LatestDataKey()
	require.NoError(t, err)
	// We're resetting the last created timestamp. So, it creates
	// new datakey.
	kr.lastCreated = 0
	dk1, err := kr.LatestDataKey()
	// We generated two key. So, checking the length.
	require.Equal(t, 2, len(kr.dataKeys))
	require.NoError(t, err)
	require.NoError(t, kr.Close())

	kr2, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	require.Equal(t, 2, len(kr2.dataKeys))
	// Asserting the correctness of the datakey after opening the registry.
	require.Equal(t, dk.Data, kr.dataKeys[dk.KeyId].Data)
	require.Equal(t, dk1.Data, kr.dataKeys[dk1.KeyId].Data)
	require.NoError(t, kr2.Close())
}

func TestRewriteRegistry(t *testing.T) {
	encryptionKey := make([]byte, 32)
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	_, err = rand.Read(encryptionKey)
	require.NoError(t, err)
	opt := getRegistryTestOptions(dir, encryptionKey)
	kr, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	_, err = kr.LatestDataKey()
	require.NoError(t, err)
	// We're resetting the last created timestamp. So, it creates
	// new datakey.
	kr.lastCreated = 0
	_, err = kr.LatestDataKey()
	require.NoError(t, err)
	require.NoError(t, kr.Close())
	delete(kr.dataKeys, 1)
	require.NoError(t, WriteKeyRegistry(kr, opt))
	kr2, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	require.Equal(t, 1, len(kr2.dataKeys))
	require.NoError(t, kr2.Close())
}

func TestMismatch(t *testing.T) {
	encryptionKey := make([]byte, 32)
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	_, err = rand.Read(encryptionKey)
	require.NoError(t, err)
	opt := getRegistryTestOptions(dir, encryptionKey)
	kr, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	require.NoError(t, kr.Close())
	// Opening with the same key and asserting.
	kr, err = OpenKeyRegistry(opt)
	require.NoError(t, err)
	require.NoError(t, kr.Close())
	// Opening with the invalid key and asserting.
	encryptionKey = make([]byte, 32)
	_, err = rand.Read(encryptionKey)
	require.NoError(t, err)
	opt.EncryptionKey = encryptionKey
	_, err = OpenKeyRegistry(opt)
	require.Error(t, err)
	require.EqualError(t, err, ErrEncryptionKeyMismatch.Error())
}

func TestEncryptionAndDecryption(t *testing.T) {
	encryptionKey := make([]byte, 32)
	dir, err := os.MkdirTemp("", "badger-test")
	require.NoError(t, err)
	defer removeDir(dir)
	_, err = rand.Read(encryptionKey)
	require.NoError(t, err)
	opt := getRegistryTestOptions(dir, encryptionKey)
	kr, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	dk, err := kr.LatestDataKey()
	require.NoError(t, err)
	require.NoError(t, kr.Close())
	// Checking the correctness of the datakey after closing and
	// opening the key registry.
	kr, err = OpenKeyRegistry(opt)
	require.NoError(t, err)
	dk1, err := kr.DataKey(dk.GetKeyId())
	require.NoError(t, err)
	require.Equal(t, dk.Data, dk1.Data)
	require.NoError(t, kr.Close())
}

func TestKeyRegistryInMemory(t *testing.T) {
	encryptionKey := make([]byte, 32)
	_, err := rand.Read(encryptionKey)
	require.NoError(t, err)

	opt := getRegistryTestOptions("", encryptionKey)
	opt.InMemory = true

	kr, err := OpenKeyRegistry(opt)
	require.NoError(t, err)
	_, err = kr.LatestDataKey()
	require.NoError(t, err)
	// We're resetting the last created timestamp. So, it creates
	// new datakey.
	kr.lastCreated = 0
	_, err = kr.LatestDataKey()
	// We generated two key. So, checking the length.
	require.Equal(t, 2, len(kr.dataKeys))
	require.NoError(t, err)
	require.NoError(t, kr.Close())
}
