package encoding

import (
	"encoding/binary"
	"math"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestDecodeBytes(t *testing.T) {
	bz := []byte{0, 1, 2, 3, 4, 5, 6, 7}
	testcases := map[string]struct {
		bz           []byte
		lengthPrefix uint64
		expect       []byte
		expectErr    bool
	}{
		"full":                      {bz, 8, bz, false},
		"empty":                     {bz, 0, []byte{}, false},
		"partial":                   {bz, 3, []byte{0, 1, 2}, false},
		"out of bounds":             {bz, 9, nil, true},
		"empty input":               {[]byte{}, 0, []byte{}, false},
		"empty input out of bounds": {[]byte{}, 1, nil, true},

		// The following will always fail, since the byte slice is only 8 bytes,
		// but we're making sure they don't panic due to overflow issues. See:
		// https://github.com/cosmos/iavl/issues/339
		"max int32":     {bz, uint64(math.MaxInt32), nil, true},
		"max int32 -1":  {bz, uint64(math.MaxInt32) - 1, nil, true},
		"max int32 -10": {bz, uint64(math.MaxInt32) - 10, nil, true},
		"max int32 +1":  {bz, uint64(math.MaxInt32) + 1, nil, true},
		"max int32 +10": {bz, uint64(math.MaxInt32) + 10, nil, true},

		"max int32*2":     {bz, uint64(math.MaxInt32) * 2, nil, true},
		"max int32*2 -1":  {bz, uint64(math.MaxInt32)*2 - 1, nil, true},
		"max int32*2 -10": {bz, uint64(math.MaxInt32)*2 - 10, nil, true},
		"max int32*2 +1":  {bz, uint64(math.MaxInt32)*2 + 1, nil, true},
		"max int32*2 +10": {bz, uint64(math.MaxInt32)*2 + 10, nil, true},

		"max uint32":     {bz, uint64(math.MaxUint32), nil, true},
		"max uint32 -1":  {bz, uint64(math.MaxUint32) - 1, nil, true},
		"max uint32 -10": {bz, uint64(math.MaxUint32) - 10, nil, true},
		"max uint32 +1":  {bz, uint64(math.MaxUint32) + 1, nil, true},
		"max uint32 +10": {bz, uint64(math.MaxUint32) + 10, nil, true},

		"max uint32*2":     {bz, uint64(math.MaxUint32) * 2, nil, true},
		"max uint32*2 -1":  {bz, uint64(math.MaxUint32)*2 - 1, nil, true},
		"max uint32*2 -10": {bz, uint64(math.MaxUint32)*2 - 10, nil, true},
		"max uint32*2 +1":  {bz, uint64(math.MaxUint32)*2 + 1, nil, true},
		"max uint32*2 +10": {bz, uint64(math.MaxUint32)*2 + 10, nil, true},

		"max int64":     {bz, uint64(math.MaxInt64), nil, true},
		"max int64 -1":  {bz, uint64(math.MaxInt64) - 1, nil, true},
		"max int64 -10": {bz, uint64(math.MaxInt64) - 10, nil, true},
		"max int64 +1":  {bz, uint64(math.MaxInt64) + 1, nil, true},
		"max int64 +10": {bz, uint64(math.MaxInt64) + 10, nil, true},

		"max uint64":     {bz, uint64(math.MaxUint64), nil, true},
		"max uint64 -1":  {bz, uint64(math.MaxUint64) - 1, nil, true},
		"max uint64 -10": {bz, uint64(math.MaxUint64) - 10, nil, true},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			// Generate an input slice.
			buf := make([]byte, binary.MaxVarintLen64)
			varintBytes := binary.PutUvarint(buf, tc.lengthPrefix)
			buf = append(buf[:varintBytes], tc.bz...)

			// Attempt to decode it.
			b, n, err := DecodeBytes(buf)
			if tc.expectErr {
				require.Error(t, err)
				require.Equal(t, varintBytes, n)
			} else {
				require.NoError(t, err)
				require.Equal(t, uint64(n), uint64(varintBytes)+tc.lengthPrefix)
				require.Equal(t, tc.bz[:tc.lengthPrefix], b)
			}
		})
	}
}

func TestDecodeBytes_invalidVarint(t *testing.T) {
	_, _, err := DecodeBytes([]byte{0xff})
	require.Error(t, err)
}
