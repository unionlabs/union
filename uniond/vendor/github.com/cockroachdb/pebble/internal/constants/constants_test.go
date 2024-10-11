// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package constants

import (
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestConstants(t *testing.T) {
	if math.MaxInt == math.MaxInt64 {
		require.Equal(t, uint64(math.MaxUint32), uint64(MaxUint32OrInt))
	} else {
		require.Equal(t, uint64(math.MaxInt), uint64(MaxUint32OrInt))
	}
}
