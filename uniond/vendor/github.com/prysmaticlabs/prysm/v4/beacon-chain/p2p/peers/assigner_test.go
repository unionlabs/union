package peers

import (
	"fmt"
	"testing"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestPickBest(t *testing.T) {
	best := testPeerIds(10)
	cases := []struct {
		name     string
		busy     map[peer.ID]bool
		n        int
		best     []peer.ID
		expected []peer.ID
	}{
		{
			name: "",
			n:    0,
		},
		{
			name:     "none busy",
			n:        1,
			expected: best[0:1],
		},
		{
			name:     "all busy except last",
			n:        1,
			busy:     testBusyMap(best[0 : len(best)-1]),
			expected: best[len(best)-1:],
		},
		{
			name:     "all busy except i=5",
			n:        1,
			busy:     testBusyMap(append(append([]peer.ID{}, best[0:5]...), best[6:]...)),
			expected: []peer.ID{best[5]},
		},
		{
			name: "all busy - 0 results",
			n:    1,
			busy: testBusyMap(best),
		},
		{
			name:     "first half busy",
			n:        5,
			busy:     testBusyMap(best[0:5]),
			expected: best[5:],
		},
		{
			name:     "back half busy",
			n:        5,
			busy:     testBusyMap(best[5:]),
			expected: best[0:5],
		},
		{
			name:     "pick all ",
			n:        10,
			expected: best,
		},
		{
			name: "none available",
			n:    10,
			best: []peer.ID{},
		},
		{
			name:     "not enough",
			n:        10,
			best:     best[0:1],
			expected: best[0:1],
		},
		{
			name:     "not enough, some busy",
			n:        10,
			best:     best[0:6],
			busy:     testBusyMap(best[0:5]),
			expected: best[5:6],
		},
	}
	for _, c := range cases {
		name := fmt.Sprintf("n=%d", c.n)
		if c.name != "" {
			name += " " + c.name
		}
		t.Run(name, func(t *testing.T) {
			if c.best == nil {
				c.best = best
			}
			pb := pickBest(c.busy, c.n, c.best)
			require.Equal(t, len(c.expected), len(pb))
			for i := range c.expected {
				require.Equal(t, c.expected[i], pb[i])
			}
		})
	}
}

func testBusyMap(b []peer.ID) map[peer.ID]bool {
	m := make(map[peer.ID]bool)
	for i := range b {
		m[b[i]] = true
	}
	return m
}

func testPeerIds(n int) []peer.ID {
	pids := make([]peer.ID, n)
	for i := range pids {
		pids[i] = peer.ID(fmt.Sprintf("%d", i))
	}
	return pids
}
