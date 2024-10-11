package main

import (
	"crypto/ecdsa"
	"crypto/rand"
	"fmt"
	"io"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/p2p/discover"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/libp2p/go-libp2p/core/crypto"
	ecdsaprysm "github.com/prysmaticlabs/prysm/v4/crypto/ecdsa"
	"github.com/prysmaticlabs/prysm/v4/network"
	_ "github.com/prysmaticlabs/prysm/v4/runtime/maxprocs"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/sirupsen/logrus"
)

func TestMain(m *testing.M) {
	logrus.SetLevel(logrus.DebugLevel)
	logrus.SetOutput(io.Discard)

	m.Run()
}

func TestBootnode_OK(t *testing.T) {
	ipAddr, err := network.ExternalIPv4()
	require.NoError(t, err)
	privKey := extractPrivateKey()
	cfg := discover.Config{
		PrivateKey: privKey,
	}
	listener := createListener(ipAddr, 4000, cfg)
	defer listener.Close()

	cfg.PrivateKey = extractPrivateKey()
	bootNode, err := enode.Parse(enode.ValidSchemes, listener.Self().String())
	require.NoError(t, err)
	cfg.Bootnodes = []*enode.Node{bootNode}
	listener2 := createListener(ipAddr, 4001, cfg)
	defer listener2.Close()

	// test that both the nodes have the other peer stored in their local table.
	listenerNode := listener.Self()
	listenerNode2 := listener2.Self()

	time.Sleep(1 * time.Second)

	nodes := listener.Lookup(listenerNode2.ID())
	assert.NotEqual(t, 0, len(nodes), "Length of nodes stored in table is not expected")
	assert.Equal(t, listenerNode2.ID(), nodes[0].ID())

	nodes = listener2.Lookup(listenerNode.ID())
	assert.NotEqual(t, 0, len(nodes), "Length of nodes stored in table is not expected")
	assert.Equal(t, listenerNode.ID(), nodes[0].ID())
}

func TestPrivateKey_ParsesCorrectly(t *testing.T) {
	privKey, _, err := crypto.GenerateSecp256k1Key(rand.Reader)
	require.NoError(t, err)

	pk, err := privKey.Raw()
	require.NoError(t, err)
	*privateKey = fmt.Sprintf("%x", pk)

	extractedKey := extractPrivateKey()

	rawKey, err := ecdsaprysm.ConvertFromInterfacePrivKey(privKey)
	require.NoError(t, err)

	r, s, err := ecdsa.Sign(rand.Reader, extractedKey, []byte{'t', 'e', 's', 't'})
	require.NoError(t, err)

	isVerified := ecdsa.Verify(&rawKey.PublicKey, []byte{'t', 'e', 's', 't'}, r, s)
	assert.Equal(t, true, isVerified, "Unmarshalled key is not the same as the key that was given to the function")
	*privateKey = ""
}
