package eth1

import (
	"fmt"
	"strings"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/params"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestComputeDeposits(t *testing.T) {
	cases := []struct {
		nvals   int
		offset  int
		partial bool
		err     error
		len     int
		name    string
	}{
		{
			nvals:   100,
			offset:  0,
			partial: false,
			len:     100,
			name:    "offset 0",
		},
		{
			nvals:   100,
			offset:  50,
			partial: false,
			len:     100,
			name:    "offset 50",
		},
		{
			nvals:   100,
			offset:  0,
			partial: true,
			len:     150,
			name:    "offset 50, partial",
		},
		{
			nvals:   100,
			offset:  50,
			partial: true,
			len:     150,
			name:    "offset 50, partial",
		},
		{
			nvals:   100,
			offset:  23,
			partial: true,
			len:     150,
			name:    "offset 23, partial",
		},
		{
			nvals:   23,
			offset:  0,
			partial: true,
			len:     35, // half of 23 (rounding down) is 11
			name:    "offset 23, partial",
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			totals := make(map[string]uint64, c.nvals)
			d, err := computeDeposits(c.offset, c.nvals, c.partial)
			for _, dd := range d {
				require.Equal(t, 48, len(dd.Data.PublicKey))
				k := fmt.Sprintf("%#x", dd.Data.PublicKey)
				if _, ok := totals[k]; !ok {
					totals[k] = dd.Data.Amount
				} else {
					totals[k] += dd.Data.Amount
				}
			}
			complete := make([]string, 0)
			incomplete := make([]string, 0)
			for k, v := range totals {
				if params.BeaconConfig().MaxEffectiveBalance != v {
					incomplete = append(incomplete, fmt.Sprintf("%s=%d", k, v))
				} else {
					complete = append(complete, k)
				}
			}
			require.Equal(t, 0, len(incomplete), strings.Join(incomplete, ", "))
			require.Equal(t, c.nvals, len(complete), strings.Join(complete, ", "))
			if err != nil {
				require.ErrorIs(t, err, c.err)
			}
			require.Equal(t, c.len, len(d))
		})
	}
}
