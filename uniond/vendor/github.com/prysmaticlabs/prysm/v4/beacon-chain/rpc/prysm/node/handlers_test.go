package node

import (
	"bytes"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"strconv"
	"testing"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	corenet "github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	libp2ptest "github.com/libp2p/go-libp2p/p2p/host/peerstore/test"
	ma "github.com/multiformats/go-multiaddr"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

type testIdentity enode.ID

func (_ testIdentity) Verify(_ *enr.Record, _ []byte) error { return nil }
func (id testIdentity) NodeAddr(_ *enr.Record) []byte       { return id[:] }

func TestListTrustedPeer(t *testing.T) {
	ids := libp2ptest.GeneratePeerIDs(9)
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	peerStatus := peerFetcher.Peers()

	for i, id := range ids {
		if i == len(ids)-1 {
			var p2pAddr = "/ip4/127.0.0." + strconv.Itoa(i) + "/udp/12000/p2p/16Uiu2HAm7yD5fhhw1Kihg5pffaGbvKV3k7sqxRGHMZzkb7u9UUxQ"
			p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
			require.NoError(t, err)
			peerStatus.Add(nil, id, p2pMultiAddr, corenet.DirUnknown)
			continue
		}
		enrRecord := &enr.Record{}
		err := enrRecord.SetSig(testIdentity{1}, []byte{42})
		require.NoError(t, err)
		enrRecord.Set(enr.IPv4{127, 0, 0, byte(i)})
		err = enrRecord.SetSig(testIdentity{}, []byte{})
		require.NoError(t, err)
		var p2pAddr = "/ip4/127.0.0." + strconv.Itoa(i) + "/udp/12000/p2p/16Uiu2HAm7yD5fhhw1Kihg5pffaGbvKV3k7sqxRGHMZzkb7u9UUxQ"
		p2pMultiAddr, err := ma.NewMultiaddr(p2pAddr)
		require.NoError(t, err)

		var direction corenet.Direction
		if i%2 == 0 {
			direction = corenet.DirInbound
		} else {
			direction = corenet.DirOutbound
		}
		peerStatus.Add(enrRecord, id, p2pMultiAddr, direction)

		switch i {
		case 0, 1:
			peerStatus.SetConnectionState(id, peers.PeerConnecting)
		case 2, 3:
			peerStatus.SetConnectionState(id, peers.PeerConnected)
		case 4, 5:
			peerStatus.SetConnectionState(id, peers.PeerDisconnecting)
		case 6, 7:
			peerStatus.SetConnectionState(id, peers.PeerDisconnected)
		default:
			t.Fatalf("Failed to set connection state for peer")
		}
	}

	s := Server{PeersFetcher: peerFetcher}
	// set all peers as trusted peers
	s.PeersFetcher.Peers().SetTrustedPeers(ids)

	t.Run("Peer data OK", func(t *testing.T) {
		url := "http://anything.is.fine"
		request := httptest.NewRequest("GET", url, nil)
		writer := httptest.NewRecorder()
		writer.Body = &bytes.Buffer{}
		s.ListTrustedPeer(writer, request)
		assert.Equal(t, http.StatusOK, writer.Code)
		resp := &PeersResponse{}
		require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
		peers := resp.Peers
		// assert number of trusted peer is right
		assert.Equal(t, 9, len(peers))

		for i := 0; i < 9; i++ {
			pid, err := peer.Decode(peers[i].PeerID)
			require.NoError(t, err)
			if pid == ids[8] {
				assert.Equal(t, "", peers[i].Enr)
				assert.Equal(t, "", peers[i].LastSeenP2PAddress)
				assert.Equal(t, "DISCONNECTED", peers[i].State)
				assert.Equal(t, "UNKNOWN", peers[i].Direction)
				continue
			}
			expectedEnr, err := peerStatus.ENR(pid)
			require.NoError(t, err)
			serializeENR, err := p2p.SerializeENR(expectedEnr)
			require.NoError(t, err)
			assert.Equal(t, "enr:"+serializeENR, peers[i].Enr)
			expectedP2PAddr, err := peerStatus.Address(pid)
			require.NoError(t, err)
			assert.Equal(t, expectedP2PAddr.String(), peers[i].LastSeenP2PAddress)
			switch pid {
			case ids[0]:
				assert.Equal(t, "CONNECTING", peers[i].State)
				assert.Equal(t, "INBOUND", peers[i].Direction)
			case ids[1]:
				assert.Equal(t, "CONNECTING", peers[i].State)
				assert.Equal(t, "OUTBOUND", peers[i].Direction)
			case ids[2]:
				assert.Equal(t, "CONNECTED", peers[i].State)
				assert.Equal(t, "INBOUND", peers[i].Direction)
			case ids[3]:
				assert.Equal(t, "CONNECTED", peers[i].State)
				assert.Equal(t, "OUTBOUND", peers[i].Direction)
			case ids[4]:
				assert.Equal(t, "DISCONNECTING", peers[i].State)
				assert.Equal(t, "INBOUND", peers[i].Direction)
			case ids[5]:
				assert.Equal(t, "DISCONNECTING", peers[i].State)
				assert.Equal(t, "OUTBOUND", peers[i].Direction)
			case ids[6]:
				assert.Equal(t, "DISCONNECTED", peers[i].State)
				assert.Equal(t, "INBOUND", peers[i].Direction)
			case ids[7]:
				assert.Equal(t, "DISCONNECTED", peers[i].State)
				assert.Equal(t, "OUTBOUND", peers[i].Direction)
			default:
				t.Fatalf("Failed to get connection state and direction for peer")
			}
		}
	})
}

func TestListTrustedPeers_NoPeersReturnsEmptyArray(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine"
	request := httptest.NewRequest("GET", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.ListTrustedPeer(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
	resp := &PeersResponse{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), resp))
	peers := resp.Peers
	assert.Equal(t, 0, len(peers))
}

func TestAddTrustedPeer(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine"
	addr := &AddrRequest{
		Addr: "/ip4/127.0.0.1/tcp/30303/p2p/16Uiu2HAm1n583t4huDMMqEUUBuQs6bLts21mxCfX3tiqu9JfHvRJ",
	}
	addrJson, err := json.Marshal(addr)
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(addrJson)
	require.NoError(t, err)
	request := httptest.NewRequest("POST", url, &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.AddTrustedPeer(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)
}

func TestAddTrustedPeer_EmptyBody(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine"
	request := httptest.NewRequest("POST", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.AddTrustedPeer(writer, request)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, writer.Code)
	assert.Equal(t, "Could not decode request body into peer address: unexpected end of JSON input", e.Message)

}

func TestAddTrustedPeer_BadAddress(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine"
	addr := &AddrRequest{
		Addr: "anything/but/not/an/address",
	}
	addrJson, err := json.Marshal(addr)
	require.NoError(t, err)
	var body bytes.Buffer
	_, err = body.Write(addrJson)
	require.NoError(t, err)
	request := httptest.NewRequest("POST", url, &body)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.AddTrustedPeer(writer, request)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, writer.Code)
	assert.StringContains(t, "Could not derive peer info from multiaddress", e.Message)
}

func TestRemoveTrustedPeer(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine.but.last.is.important/16Uiu2HAm1n583t4huDMMqEUUBuQs6bLts21mxCfX3tiqu9JfHvRJ"
	request := httptest.NewRequest("DELETE", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.RemoveTrustedPeer(writer, request)
	assert.Equal(t, http.StatusOK, writer.Code)

}

func TestRemoveTrustedPeer_EmptyParameter(t *testing.T) {
	peerFetcher := &mockp2p.MockPeersProvider{}
	peerFetcher.ClearPeers()
	s := Server{PeersFetcher: peerFetcher}

	url := "http://anything.is.fine"
	request := httptest.NewRequest("DELETE", url, nil)
	writer := httptest.NewRecorder()
	writer.Body = &bytes.Buffer{}
	s.RemoveTrustedPeer(writer, request)
	e := &httputil.DefaultJsonError{}
	require.NoError(t, json.Unmarshal(writer.Body.Bytes(), e))
	assert.Equal(t, http.StatusBadRequest, writer.Code)
	assert.Equal(t, "Could not decode peer id: failed to parse peer ID: invalid cid: cid too short", e.Message)
}
