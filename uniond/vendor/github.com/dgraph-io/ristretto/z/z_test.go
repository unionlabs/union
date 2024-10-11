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

package z

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func verifyHashProduct(t *testing.T, wantKey, wantConflict, key, conflict uint64) {
	require.Equal(t, wantKey, key)
	require.Equal(t, wantConflict, conflict)
}

func TestKeyToHash(t *testing.T) {
	var key uint64
	var conflict uint64

	key, conflict = KeyToHash(uint64(1))
	verifyHashProduct(t, 1, 0, key, conflict)

	key, conflict = KeyToHash(1)
	verifyHashProduct(t, 1, 0, key, conflict)

	key, conflict = KeyToHash(int32(2))
	verifyHashProduct(t, 2, 0, key, conflict)

	key, conflict = KeyToHash(int32(-2))
	verifyHashProduct(t, math.MaxUint64-1, 0, key, conflict)

	key, conflict = KeyToHash(int64(-2))
	verifyHashProduct(t, math.MaxUint64-1, 0, key, conflict)

	key, conflict = KeyToHash(uint32(3))
	verifyHashProduct(t, 3, 0, key, conflict)

	key, conflict = KeyToHash(int64(3))
	verifyHashProduct(t, 3, 0, key, conflict)
}

func TestMulipleSignals(t *testing.T) {
	closer := NewCloser(0)
	require.NotPanics(t, func() { closer.Signal() })
	// Should not panic.
	require.NotPanics(t, func() { closer.Signal() })
	require.NotPanics(t, func() { closer.SignalAndWait() })

	// Attempt 2.
	closer = NewCloser(1)
	require.NotPanics(t, func() { closer.Done() })

	require.NotPanics(t, func() { closer.SignalAndWait() })
	// Should not panic.
	require.NotPanics(t, func() { closer.SignalAndWait() })
	require.NotPanics(t, func() { closer.Signal() })
}

func TestCloser(t *testing.T) {
	closer := NewCloser(1)
	go func() {
		defer closer.Done()
		<-closer.Ctx().Done()
	}()
	closer.SignalAndWait()
}

func TestZeroOut(t *testing.T) {
	dst := make([]byte, 4*1024)
	fill := func() {
		for i := 0; i < len(dst); i++ {
			dst[i] = 0xFF
		}
	}
	check := func(buf []byte, b byte) {
		for i := 0; i < len(buf); i++ {
			require.Equalf(t, b, buf[i], "idx: %d", i)
		}
	}
	fill()

	ZeroOut(dst, 0, 1)
	check(dst[:1], 0x00)
	check(dst[1:], 0xFF)

	ZeroOut(dst, 0, 1024)
	check(dst[:1024], 0x00)
	check(dst[1024:], 0xFF)

	ZeroOut(dst, 0, len(dst))
	check(dst, 0x00)
}
