// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// Package netconv provides OpenTelemetry network semantic conventions for
// tracing telemetry.
package netconv // import "go.opentelemetry.io/otel/semconv/v1.15.0/netconv"

import (
	"net"

	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/semconv/internal/v2"
	semconv "go.opentelemetry.io/otel/semconv/v1.15.0"
)

var nc = &internal.NetConv{
	NetHostNameKey:     semconv.NetHostNameKey,
	NetHostPortKey:     semconv.NetHostPortKey,
	NetPeerNameKey:     semconv.NetPeerNameKey,
	NetPeerPortKey:     semconv.NetPeerPortKey,
	NetSockFamilyKey:   semconv.NetSockFamilyKey,
	NetSockPeerAddrKey: semconv.NetSockPeerAddrKey,
	NetSockPeerPortKey: semconv.NetSockPeerPortKey,
	NetSockHostAddrKey: semconv.NetSockHostAddrKey,
	NetSockHostPortKey: semconv.NetSockHostPortKey,
	NetTransportOther:  semconv.NetTransportOther,
	NetTransportTCP:    semconv.NetTransportTCP,
	NetTransportUDP:    semconv.NetTransportUDP,
	NetTransportInProc: semconv.NetTransportInProc,
}

// Transport returns a trace attribute describing the transport protocol of the
// passed network. See the net.Dial for information about acceptable network
// values.
func Transport(network string) attribute.KeyValue {
	return nc.Transport(network)
}

// Client returns trace attributes for a client network connection to address.
// See net.Dial for information about acceptable address values, address should
// be the same as the one used to create conn. If conn is nil, only network
// peer attributes will be returned that describe address. Otherwise, the
// socket level information about conn will also be included.
func Client(address string, conn net.Conn) []attribute.KeyValue {
	return nc.Client(address, conn)
}

// Server returns trace attributes for a network listener listening at address.
// See net.Listen for information about acceptable address values, address
// should be the same as the one used to create ln. If ln is nil, only network
// host attributes will be returned that describe address. Otherwise, the
// socket level information about ln will also be included.
func Server(address string, ln net.Listener) []attribute.KeyValue {
	return nc.Server(address, ln)
}
