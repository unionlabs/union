// Code created by gotmpl. DO NOT MODIFY.
// source: internal/shared/semconvutil/netconv_test.go.tmpl

// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package semconvutil

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"go.opentelemetry.io/otel/attribute"
)

const (
	addr = "127.0.0.1"
	port = 1834
)

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
		assert.Equal(t, want, NetTransport(network))
	}
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

func TestNetFamily(t *testing.T) {
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

func TestNetProtocol(t *testing.T) {
	type testCase struct {
		name, version string
	}
	tests := map[string]testCase{
		"HTTP/1.0":        {name: "http", version: "1.0"},
		"HTTP/1.1":        {name: "http", version: "1.1"},
		"HTTP/2":          {name: "http", version: "2"},
		"HTTP/3":          {name: "http", version: "3"},
		"SPDY":            {name: "spdy"},
		"SPDY/2":          {name: "spdy", version: "2"},
		"QUIC":            {name: "quic"},
		"unknown/proto/2": {name: "unknown", version: "proto/2"},
		"other":           {name: "other"},
	}

	for proto, want := range tests {
		name, version := netProtocol(proto)
		assert.Equal(t, want.name, name)
		assert.Equal(t, want.version, version)
	}
}
