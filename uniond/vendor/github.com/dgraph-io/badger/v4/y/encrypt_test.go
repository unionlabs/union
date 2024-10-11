/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
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
	"crypto/rand"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestXORBlock(t *testing.T) {
	key := make([]byte, 32)
	_, _ = rand.Read(key)

	var iv []byte
	{
		b, err := aes.NewCipher(key)
		require.NoError(t, err)
		iv = make([]byte, b.BlockSize())
		_, _ = rand.Read(iv)
		t.Logf("Using %d size IV\n", len(iv))
	}

	src := make([]byte, 1024)
	_, _ = rand.Read(src)

	dst := make([]byte, 1024)
	err := XORBlock(dst, src, key, iv)
	require.NoError(t, err)

	act := make([]byte, 1024)
	err = XORBlock(act, dst, key, iv)
	require.NoError(t, err)
	require.Equal(t, src, act)

	// Now check if we can use the same byte slice as src and dst. While this is useful to know that
	// we can use src and dst as the same slice, this isn't applicable to Badger because we're
	// reading data right off mmap. We should not modify that data, so we have to use a different
	// slice for dst anyway.
	cp := append([]byte{}, src...)
	err = XORBlock(cp, cp, key, iv)
	require.NoError(t, err)
	require.Equal(t, dst, cp)

	err = XORBlock(cp, cp, key, iv)
	require.NoError(t, err)
	require.Equal(t, src, cp)
}
