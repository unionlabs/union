// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package internal

import (
	"net"
	"strconv"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"go.opentelemetry.io/otel/attribute"
)

const (
	addr = "127.0.0.1"
	port = 1834
)

var nc = &NetConv{
	NetHostNameKey:     attribute.Key("net.host.name"),
	NetHostPortKey:     attribute.Key("net.host.port"),
	NetPeerNameKey:     attribute.Key("net.peer.name"),
	NetPeerPortKey:     attribute.Key("net.peer.port"),
	NetSockPeerAddrKey: attribute.Key("net.sock.peer.addr"),
	NetSockPeerPortKey: attribute.Key("net.sock.peer.port"),
	NetTransportOther:  attribute.String("net.transport", "other"),
	NetTransportTCP:    attribute.String("net.transport", "ip_tcp"),
	NetTransportUDP:    attribute.String("net.transport", "ip_udp"),
	NetTransportInProc: attribute.String("net.transport", "inproc"),
}

func TestNetTransport(t *testing.T) {
	transports := map[string]attribute.KeyValue{
		"tcp":        attribute.String("net.transport", "ip_tcp"),
		"tcp4":       attribute.String("net.transport", "ip_tcp"),
		"tcp6":       attribute.String("net.transport", "ip_tcp"),
		"udp":        attribute.String("net.transport", "ip_udp"),
		"udp4":       attribute.String("net.transport", "ip_udp"),
		"udp6":       attribute.String("net.transport", "ip_udp"),
		"unix":       attribute.String("net.transport", "inproc"),
		"unixgram":   attribute.String("net.transport", "inproc"),
		"unixpacket": attribute.String("net.transport", "inproc"),
		"ip:1":       attribute.String("net.transport", "other"),
		"ip:icmp":    attribute.String("net.transport", "other"),
		"ip4:proto":  attribute.String("net.transport", "other"),
		"ip6:proto":  attribute.String("net.transport", "other"),
	}

	for network, want := range transports {
		assert.Equal(t, want, nc.Transport(network))
	}
}

func TestNetServerNilListener(t *testing.T) {
	const addr = "127.0.0.1:8080"
	got := nc.Server(addr, nil)
	expected := nc.Host(addr)
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

type listener struct{ net.Listener }

func (listener) Addr() net.Addr { return nil }

func TestNetServerNilAddr(t *testing.T) {
	const addr = "127.0.0.1:8080"
	got := nc.Server(addr, listener{})
	expected := nc.Host(addr)
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

func newTCPListener() (net.Listener, error) {
	return net.Listen("tcp4", "127.0.0.1:0")
}

func TestNetServerTCP(t *testing.T) {
	ln, err := newTCPListener()
	require.NoError(t, err)
	defer func() { require.NoError(t, ln.Close()) }()

	host, pStr, err := net.SplitHostPort(ln.Addr().String())
	require.NoError(t, err)
	port, err := strconv.Atoi(pStr)
	require.NoError(t, err)

	got := nc.Server("example.com:8080", ln)
	expected := []attribute.KeyValue{
		nc.HostName("example.com"),
		nc.HostPort(8080),
		nc.NetTransportTCP,
		nc.NetSockFamilyKey.String("inet"),
		nc.NetSockHostAddrKey.String(host),
		nc.NetSockHostPortKey.Int(port),
	}
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

func TestNetHost(t *testing.T) {
	testAddrs(t, []addrTest{
		{address: "", expected: nil},
		{address: "192.0.0.1", expected: []attribute.KeyValue{
			nc.HostName("192.0.0.1"),
		}},
		{address: "192.0.0.1:9090", expected: []attribute.KeyValue{
			nc.HostName("192.0.0.1"),
			nc.HostPort(9090),
		}},
	}, nc.Host)
}

func TestNetHostName(t *testing.T) {
	expected := attribute.Key("net.host.name").String(addr)
	assert.Equal(t, expected, nc.HostName(addr))
}

func TestNetHostPort(t *testing.T) {
	expected := attribute.Key("net.host.port").Int(port)
	assert.Equal(t, expected, nc.HostPort(port))
}

func TestNetClientNilConn(t *testing.T) {
	const addr = "127.0.0.1:8080"
	got := nc.Client(addr, nil)
	expected := nc.Peer(addr)
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

type conn struct{ net.Conn }

func (conn) LocalAddr() net.Addr  { return nil }
func (conn) RemoteAddr() net.Addr { return nil }

func TestNetClientNilAddr(t *testing.T) {
	const addr = "127.0.0.1:8080"
	got := nc.Client(addr, conn{})
	expected := nc.Peer(addr)
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

func newTCPConn() (net.Conn, net.Listener, error) {
	ln, err := newTCPListener()
	if err != nil {
		return nil, nil, err
	}

	conn, err := net.Dial("tcp4", ln.Addr().String())
	if err != nil {
		_ = ln.Close()
		return nil, nil, err
	}

	return conn, ln, nil
}

func TestNetClientTCP(t *testing.T) {
	conn, ln, err := newTCPConn()
	require.NoError(t, err)
	defer func() { require.NoError(t, ln.Close()) }()
	defer func() { require.NoError(t, conn.Close()) }()

	lHost, pStr, err := net.SplitHostPort(conn.LocalAddr().String())
	require.NoError(t, err)
	lPort, err := strconv.Atoi(pStr)
	require.NoError(t, err)

	rHost, pStr, err := net.SplitHostPort(conn.RemoteAddr().String())
	require.NoError(t, err)
	rPort, err := strconv.Atoi(pStr)
	require.NoError(t, err)

	got := nc.Client("example.com:8080", conn)
	expected := []attribute.KeyValue{
		nc.PeerName("example.com"),
		nc.PeerPort(8080),
		nc.NetTransportTCP,
		nc.NetSockFamilyKey.String("inet"),
		nc.NetSockPeerAddrKey.String(rHost),
		nc.NetSockPeerPortKey.Int(rPort),
		nc.NetSockHostAddrKey.String(lHost),
		nc.NetSockHostPortKey.Int(lPort),
	}
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

type remoteOnlyConn struct{ net.Conn }

func (remoteOnlyConn) LocalAddr() net.Addr { return nil }

func TestNetClientTCPNilLocal(t *testing.T) {
	conn, ln, err := newTCPConn()
	require.NoError(t, err)
	defer func() { require.NoError(t, ln.Close()) }()
	defer func() { require.NoError(t, conn.Close()) }()

	conn = remoteOnlyConn{conn}

	rHost, pStr, err := net.SplitHostPort(conn.RemoteAddr().String())
	require.NoError(t, err)
	rPort, err := strconv.Atoi(pStr)
	require.NoError(t, err)

	got := nc.Client("example.com:8080", conn)
	expected := []attribute.KeyValue{
		nc.PeerName("example.com"),
		nc.PeerPort(8080),
		nc.NetTransportTCP,
		nc.NetSockFamilyKey.String("inet"),
		nc.NetSockPeerAddrKey.String(rHost),
		nc.NetSockPeerPortKey.Int(rPort),
	}
	assert.Equal(t, cap(expected), cap(got), "slice capacity")
	assert.ElementsMatch(t, expected, got)
}

func TestNetPeer(t *testing.T) {
	testAddrs(t, []addrTest{
		{address: "", expected: nil},
		{address: "example.com", expected: []attribute.KeyValue{
			nc.PeerName("example.com"),
		}},
		{address: "/tmp/file", expected: []attribute.KeyValue{
			nc.PeerName("/tmp/file"),
		}},
		{address: "192.0.0.1", expected: []attribute.KeyValue{
			nc.PeerName("192.0.0.1"),
		}},
		{address: ":9090", expected: nil},
		{address: "192.0.0.1:9090", expected: []attribute.KeyValue{
			nc.PeerName("192.0.0.1"),
			nc.PeerPort(9090),
		}},
	}, nc.Peer)
}

func TestNetPeerName(t *testing.T) {
	expected := attribute.Key("net.peer.name").String(addr)
	assert.Equal(t, expected, nc.PeerName(addr))
}

func TestNetPeerPort(t *testing.T) {
	expected := attribute.Key("net.peer.port").Int(port)
	assert.Equal(t, expected, nc.PeerPort(port))
}

func TestNetSockPeerName(t *testing.T) {
	expected := attribute.Key("net.sock.peer.addr").String(addr)
	assert.Equal(t, expected, nc.SockPeerAddr(addr))
}

func TestNetSockPeerPort(t *testing.T) {
	expected := attribute.Key("net.sock.peer.port").Int(port)
	assert.Equal(t, expected, nc.SockPeerPort(port))
}

func TestFamily(t *testing.T) {
	tests := []struct {
		network string
		address string
		expect  string
	}{
		{"", "", ""},
		{"unix", "", "unix"},
		{"unix", "gibberish", "unix"},
		{"unixgram", "", "unix"},
		{"unixgram", "gibberish", "unix"},
		{"unixpacket", "gibberish", "unix"},
		{"tcp", "123.0.2.8", "inet"},
		{"tcp", "gibberish", ""},
		{"", "123.0.2.8", "inet"},
		{"", "gibberish", ""},
		{"tcp", "fe80::1", "inet6"},
		{"", "fe80::1", "inet6"},
	}

	for _, test := range tests {
		got := family(test.network, test.address)
		assert.Equal(t, test.expect, got, test.network+"/"+test.address)
	}
}

func TestSplitHostPort(t *testing.T) {
	tests := []struct {
		hostport string
		host     string
		port     int
	}{
		{"", "", -1},
		{":8080", "", 8080},
		{"127.0.0.1", "127.0.0.1", -1},
		{"www.example.com", "www.example.com", -1},
		{"127.0.0.1%25en0", "127.0.0.1%25en0", -1},
		{"[]", "", -1}, // Ensure this doesn't panic.
		{"[fe80::1", "", -1},
		{"[fe80::1]", "fe80::1", -1},
		{"[fe80::1%25en0]", "fe80::1%25en0", -1},
		{"[fe80::1]:8080", "fe80::1", 8080},
		{"[fe80::1]::", "", -1}, // Too many colons.
		{"127.0.0.1:", "127.0.0.1", -1},
		{"127.0.0.1:port", "127.0.0.1", -1},
		{"127.0.0.1:8080", "127.0.0.1", 8080},
		{"www.example.com:8080", "www.example.com", 8080},
		{"127.0.0.1%25en0:8080", "127.0.0.1%25en0", 8080},
	}

	for _, test := range tests {
		h, p := splitHostPort(test.hostport)
		assert.Equal(t, test.host, h, test.hostport)
		assert.Equal(t, test.port, p, test.hostport)
	}
}

type addrTest struct {
	address  string
	expected []attribute.KeyValue
}

func testAddrs(t *testing.T, tests []addrTest, f func(string) []attribute.KeyValue) {
	t.Helper()

	for _, test := range tests {
		got := f(test.address)
		assert.Equal(t, cap(test.expected), cap(got), "slice capacity")
		assert.ElementsMatch(t, test.expected, got, test.address)
	}
}
