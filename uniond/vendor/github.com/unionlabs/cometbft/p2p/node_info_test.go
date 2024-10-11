package p2p

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/ed25519"
)

func TestNodeInfoValidate(t *testing.T) {
	// empty fails
	ni := DefaultNodeInfo{}
	require.Error(t, ni.Validate())

	channels := make([]byte, maxNumChannels)
	for i := 0; i < maxNumChannels; i++ {
		channels[i] = byte(i)
	}
	dupChannels := make([]byte, 5)
	copy(dupChannels, channels[:5])
	dupChannels = append(dupChannels, testCh) //nolint:makezero // huge errors when we don't do it the "wrong" way

	nonASCII := "¢§µ"
	emptyTab := "\t"
	emptySpace := "  "

	testCases := []struct {
		testName         string
		malleateNodeInfo func(*DefaultNodeInfo)
		expectErr        bool
	}{
		{
			"Too Many Channels",
			func(ni *DefaultNodeInfo) { ni.Channels = append(channels, byte(maxNumChannels)) }, //nolint: makezero
			true,
		},
		{"Duplicate Channel", func(ni *DefaultNodeInfo) { ni.Channels = dupChannels }, true},
		{"Good Channels", func(ni *DefaultNodeInfo) { ni.Channels = ni.Channels[:5] }, false},

		{"Invalid NetAddress", func(ni *DefaultNodeInfo) { ni.ListenAddr = "not-an-address" }, true},
		{"Good NetAddress", func(ni *DefaultNodeInfo) { ni.ListenAddr = "0.0.0.0:26656" }, false},

		{"Non-ASCII Version", func(ni *DefaultNodeInfo) { ni.Version = nonASCII }, true},
		{"Empty tab Version", func(ni *DefaultNodeInfo) { ni.Version = emptyTab }, true},
		{"Empty space Version", func(ni *DefaultNodeInfo) { ni.Version = emptySpace }, true},
		{"Empty Version", func(ni *DefaultNodeInfo) { ni.Version = "" }, false},

		{"Non-ASCII Moniker", func(ni *DefaultNodeInfo) { ni.Moniker = nonASCII }, true},
		{"Empty tab Moniker", func(ni *DefaultNodeInfo) { ni.Moniker = emptyTab }, true},
		{"Empty space Moniker", func(ni *DefaultNodeInfo) { ni.Moniker = emptySpace }, true},
		{"Empty Moniker", func(ni *DefaultNodeInfo) { ni.Moniker = "" }, true},
		{"Good Moniker", func(ni *DefaultNodeInfo) { ni.Moniker = "hey its me" }, false},

		{"Non-ASCII TxIndex", func(ni *DefaultNodeInfo) { ni.Other.TxIndex = nonASCII }, true},
		{"Empty tab TxIndex", func(ni *DefaultNodeInfo) { ni.Other.TxIndex = emptyTab }, true},
		{"Empty space TxIndex", func(ni *DefaultNodeInfo) { ni.Other.TxIndex = emptySpace }, true},
		{"Empty TxIndex", func(ni *DefaultNodeInfo) { ni.Other.TxIndex = "" }, false},
		{"Off TxIndex", func(ni *DefaultNodeInfo) { ni.Other.TxIndex = "off" }, false},

		{"Non-ASCII RPCAddress", func(ni *DefaultNodeInfo) { ni.Other.RPCAddress = nonASCII }, true},
		{"Empty tab RPCAddress", func(ni *DefaultNodeInfo) { ni.Other.RPCAddress = emptyTab }, true},
		{"Empty space RPCAddress", func(ni *DefaultNodeInfo) { ni.Other.RPCAddress = emptySpace }, true},
		{"Empty RPCAddress", func(ni *DefaultNodeInfo) { ni.Other.RPCAddress = "" }, false},
		{"Good RPCAddress", func(ni *DefaultNodeInfo) { ni.Other.RPCAddress = "0.0.0.0:26657" }, false},
	}

	nodeKey := NodeKey{PrivKey: ed25519.GenPrivKey()}
	name := "testing"

	// test case passes
	ni = testNodeInfo(nodeKey.ID(), name).(DefaultNodeInfo)
	ni.Channels = channels
	require.NoError(t, ni.Validate())

	for _, tc := range testCases {
		ni := testNodeInfo(nodeKey.ID(), name).(DefaultNodeInfo)
		ni.Channels = channels
		tc.malleateNodeInfo(&ni)
		err := ni.Validate()
		if tc.expectErr {
			require.Error(t, err, tc.testName)
		} else {
			require.NoError(t, err, tc.testName)
		}
	}
}

func TestNodeInfoCompatible(t *testing.T) {
	nodeKey1 := NodeKey{PrivKey: ed25519.GenPrivKey()}
	nodeKey2 := NodeKey{PrivKey: ed25519.GenPrivKey()}
	name := "testing"

	var newTestChannel byte = 0x2

	// test NodeInfo is compatible
	ni1 := testNodeInfo(nodeKey1.ID(), name).(DefaultNodeInfo)
	ni2 := testNodeInfo(nodeKey2.ID(), name).(DefaultNodeInfo)
	require.NoError(t, ni1.CompatibleWith(ni2))

	// add another channel; still compatible
	ni2.Channels = append(ni2.Channels, newTestChannel)
	assert.True(t, ni2.HasChannel(newTestChannel))
	require.NoError(t, ni1.CompatibleWith(ni2))

	// wrong NodeInfo type is not compatible
	_, netAddr := CreateRoutableAddr()
	ni3 := mockNodeInfo{netAddr}
	require.Error(t, ni1.CompatibleWith(ni3))

	testCases := []struct {
		testName         string
		malleateNodeInfo func(*DefaultNodeInfo)
	}{
		{"Wrong block version", func(ni *DefaultNodeInfo) { ni.ProtocolVersion.Block++ }},
		{"Wrong network", func(ni *DefaultNodeInfo) { ni.Network += "-wrong" }},
		{"No common channels", func(ni *DefaultNodeInfo) { ni.Channels = []byte{newTestChannel} }},
	}

	for _, tc := range testCases {
		ni := testNodeInfo(nodeKey2.ID(), name).(DefaultNodeInfo)
		tc.malleateNodeInfo(&ni)
		require.Error(t, ni1.CompatibleWith(ni))
	}
}
