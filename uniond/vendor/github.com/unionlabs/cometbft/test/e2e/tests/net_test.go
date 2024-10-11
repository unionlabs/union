package e2e_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
)

// Tests that all nodes have peered with each other, regardless of discovery method.
func TestNet_Peers(t *testing.T) {
	// FIXME Skip test since nodes aren't always able to fully mesh
	t.SkipNow()

	testNode(t, func(t *testing.T, node e2e.Node) {
		t.Helper()
		// Seed nodes shouldn't necessarily mesh with the entire network.
		if node.Mode == e2e.ModeSeed {
			return
		}

		client, err := node.Client()
		require.NoError(t, err)
		netInfo, err := client.NetInfo(ctx)
		require.NoError(t, err)

		require.Equal(t, len(node.Testnet.Nodes)-1, netInfo.NPeers,
			"node is not fully meshed with peers")

		seen := map[string]bool{}
		for _, n := range node.Testnet.Nodes {
			seen[n.Name] = (n.Name == node.Name) // we've clearly seen ourself
		}
		for _, peerInfo := range netInfo.Peers {
			peer := node.Testnet.LookupNode(peerInfo.NodeInfo.Moniker)
			require.NotNil(t, peer, "unknown node %v", peerInfo.NodeInfo.Moniker)
			require.Equal(t, peer.InternalIP.String(), peerInfo.RemoteIP,
				"unexpected IP address for peer %v", peer.Name)
			seen[peerInfo.NodeInfo.Moniker] = true
		}

		for name := range seen {
			require.True(t, seen[name], "node %v not peered with %v", node.Name, name)
		}
	})
}
