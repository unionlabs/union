package blocks

import (
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestROBlockSorting(t *testing.T) {
	one := bytesutil.ToBytes32(bytesutil.PadTo([]byte{1}, 32))
	two := bytesutil.ToBytes32(bytesutil.PadTo([]byte{2}, 32))
	cases := []struct {
		name   string
		ros    []ROBlock
		sorted []ROBlock
	}{
		{
			name:   "1 item",
			ros:    []ROBlock{testROBlock(t, 1, [32]byte{})},
			sorted: []ROBlock{testROBlock(t, 1, [32]byte{})},
		},
		{
			name:   "2 items, sorted",
			ros:    []ROBlock{testROBlock(t, 1, [32]byte{}), testROBlock(t, 2, [32]byte{})},
			sorted: []ROBlock{testROBlock(t, 1, [32]byte{}), testROBlock(t, 2, [32]byte{})},
		},
		{
			name:   "2 items, reversed",
			ros:    []ROBlock{testROBlock(t, 2, [32]byte{}), testROBlock(t, 1, [32]byte{})},
			sorted: []ROBlock{testROBlock(t, 1, [32]byte{}), testROBlock(t, 2, [32]byte{})},
		},
		{
			name: "3 items, reversed, with tie breaker",
			ros: []ROBlock{
				testROBlock(t, 2, two),
				testROBlock(t, 2, one),
				testROBlock(t, 1, [32]byte{}),
			},
			sorted: []ROBlock{
				testROBlock(t, 1, [32]byte{}),
				testROBlock(t, 2, one),
				testROBlock(t, 2, two),
			},
		},
		{
			name: "5 items, reversed, with double root tie",
			ros: []ROBlock{
				testROBlock(t, 0, one),
				testROBlock(t, 2, two),
				testROBlock(t, 2, one),
				testROBlock(t, 2, two),
				testROBlock(t, 2, one),
				testROBlock(t, 1, [32]byte{}),
			},
			sorted: []ROBlock{
				testROBlock(t, 0, one),
				testROBlock(t, 1, [32]byte{}),
				testROBlock(t, 2, one),
				testROBlock(t, 2, one),
				testROBlock(t, 2, two),
				testROBlock(t, 2, two),
			},
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			sort.Sort(ROBlockSlice(c.ros))
			for i := 0; i < len(c.sorted); i++ {
				require.Equal(t, c.sorted[i].Block().Slot(), c.ros[i].Block().Slot())
				require.Equal(t, c.sorted[i].Root(), c.ros[i].Root())
			}
		})
	}
}

func testROBlock(t *testing.T, slot primitives.Slot, root [32]byte) ROBlock {
	b, err := NewSignedBeaconBlock(&eth.SignedBeaconBlock{Block: &eth.BeaconBlock{
		Body: &eth.BeaconBlockBody{},
		Slot: slot,
	}})
	require.NoError(t, err)
	return ROBlock{
		ReadOnlySignedBeaconBlock: b,
		root:                      root,
	}
}
