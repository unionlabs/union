// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/tls"
	"io"
	"log/slog"
	"net/netip"
	"runtime"
	"testing"
	"time"

	"golang.org/x/net/quic/qlog"
)

func TestConnect(t *testing.T) {
	newLocalConnPair(t, &Config{}, &Config{})
}

func TestConnectDefaultTLSConfig(t *testing.T) {
	serverConfig := newTestTLSConfigWithMoreDefaults(serverSide)
	clientConfig := newTestTLSConfigWithMoreDefaults(clientSide)
	newLocalConnPair(t, &Config{TLSConfig: serverConfig}, &Config{TLSConfig: clientConfig})
}

func TestStreamTransfer(t *testing.T) {
	ctx := context.Background()
	cli, srv := newLocalConnPair(t, &Config{}, &Config{})
	data := makeTestData(1 << 20)

	srvdone := make(chan struct{})
	go func() {
		defer close(srvdone)
		s, err := srv.AcceptStream(ctx)
		if err != nil {
			t.Errorf("AcceptStream: %v", err)
			return
		}
		b, err := io.ReadAll(s)
		if err != nil {
			t.Errorf("io.ReadAll(s): %v", err)
			return
		}
		if !bytes.Equal(b, data) {
			t.Errorf("read data mismatch (got %v bytes, want %v", len(b), len(data))
		}
		if err := s.Close(); err != nil {
			t.Errorf("s.Close() = %v", err)
		}
	}()

	s, err := cli.NewSendOnlyStream(ctx)
	if err != nil {
		t.Fatalf("NewStream: %v", err)
	}
	n, err := io.Copy(s, bytes.NewBuffer(data))
	if n != int64(len(data)) || err != nil {
		t.Fatalf("io.Copy(s, data) = %v, %v; want %v, nil", n, err, len(data))
	}
	if err := s.Close(); err != nil {
		t.Fatalf("s.Close() = %v", err)
	}
}

func newLocalConnPair(t testing.TB, conf1, conf2 *Config) (clientConn, serverConn *Conn) {
	switch runtime.GOOS {
	case "plan9":
		t.Skipf("ReadMsgUDP not supported on %s", runtime.GOOS)
	}
	t.Helper()
	ctx := context.Background()
	e1 := newLocalEndpoint(t, serverSide, conf1)
	e2 := newLocalEndpoint(t, clientSide, conf2)
	conf2 = makeTestConfig(conf2, clientSide)
	c2, err := e2.Dial(ctx, "udp", e1.LocalAddr().String(), conf2)
	if err != nil {
		t.Fatal(err)
	}
	c1, err := e1.Accept(ctx)
	if err != nil {
		t.Fatal(err)
	}
	return c2, c1
}

func newLocalEndpoint(t testing.TB, side connSide, conf *Config) *Endpoint {
	t.Helper()
	conf = makeTestConfig(conf, side)
	e, err := Listen("udp", "127.0.0.1:0", conf)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		e.Close(canceledContext())
	})
	return e
}

func makeTestConfig(conf *Config, side connSide) *Config {
	if conf == nil {
		return nil
	}
	newConf := *conf
	conf = &newConf
	if conf.TLSConfig == nil {
		conf.TLSConfig = newTestTLSConfig(side)
	}
	if conf.QLogLogger == nil {
		conf.QLogLogger = slog.New(qlog.NewJSONHandler(qlog.HandlerOptions{
			Level: QLogLevelFrame,
			Dir:   *qlogdir,
		}))
	}
	return conf
}

type testEndpoint struct {
	t                     *testing.T
	e                     *Endpoint
	now                   time.Time
	recvc                 chan *datagram
	idlec                 chan struct{}
	conns                 map[*Conn]*testConn
	acceptQueue           []*testConn
	configTransportParams []func(*transportParameters)
	configTestConn        []func(*testConn)
	sentDatagrams         [][]byte
	peerTLSConn           *tls.QUICConn
	lastInitialDstConnID  []byte // for parsing Retry packets
}

func newTestEndpoint(t *testing.T, config *Config) *testEndpoint {
	te := &testEndpoint{
		t:     t,
		now:   time.Date(2020, 1, 1, 0, 0, 0, 0, time.UTC),
		recvc: make(chan *datagram),
		idlec: make(chan struct{}),
		conns: make(map[*Conn]*testConn),
	}
	var err error
	te.e, err = newEndpoint((*testEndpointUDPConn)(te), config, (*testEndpointHooks)(te))
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(te.cleanup)
	return te
}

func (te *testEndpoint) cleanup() {
	te.e.Close(canceledContext())
}

func (te *testEndpoint) wait() {
	select {
	case te.idlec <- struct{}{}:
	case <-te.e.closec:
	}
	for _, tc := range te.conns {
		tc.wait()
	}
}

// accept returns a server connection from the endpoint.
// Unlike Endpoint.Accept, connections are available as soon as they are created.
func (te *testEndpoint) accept() *testConn {
	if len(te.acceptQueue) == 0 {
		te.t.Fatalf("accept: expected available conn, but found none")
	}
	tc := te.acceptQueue[0]
	te.acceptQueue = te.acceptQueue[1:]
	return tc
}

func (te *testEndpoint) write(d *datagram) {
	te.recvc <- d
	te.wait()
}

var testClientAddr = netip.MustParseAddrPort("10.0.0.1:8000")

func (te *testEndpoint) writeDatagram(d *testDatagram) {
	te.t.Helper()
	logDatagram(te.t, "<- endpoint under test receives", d)
	var buf []byte
	for _, p := range d.packets {
		tc := te.connForDestination(p.dstConnID)
		if p.ptype != packetTypeRetry && tc != nil {
			space := spaceForPacketType(p.ptype)
			if p.num >= tc.peerNextPacketNum[space] {
				tc.peerNextPacketNum[space] = p.num + 1
			}
		}
		if p.ptype == packetTypeInitial {
			te.lastInitialDstConnID = p.dstConnID
		}
		pad := 0
		if p.ptype == packetType1RTT {
			pad = d.paddedSize - len(buf)
		}
		buf = append(buf, encodeTestPacket(te.t, tc, p, pad)...)
	}
	for len(buf) < d.paddedSize {
		buf = append(buf, 0)
	}
	te.write(&datagram{
		b:        buf,
		peerAddr: d.addr,
	})
}

func (te *testEndpoint) connForDestination(dstConnID []byte) *testConn {
	for _, tc := range te.conns {
		for _, loc := range tc.conn.connIDState.local {
			if bytes.Equal(loc.cid, dstConnID) {
				return tc
			}
		}
	}
	return nil
}

func (te *testEndpoint) connForSource(srcConnID []byte) *testConn {
	for _, tc := range te.conns {
		for _, loc := range tc.conn.connIDState.remote {
			if bytes.Equal(loc.cid, srcConnID) {
				return tc
			}
		}
	}
	return nil
}

func (te *testEndpoint) read() []byte {
	te.t.Helper()
	te.wait()
	if len(te.sentDatagrams) == 0 {
		return nil
	}
	d := te.sentDatagrams[0]
	te.sentDatagrams = te.sentDatagrams[1:]
	return d
}

func (te *testEndpoint) readDatagram() *testDatagram {
	te.t.Helper()
	buf := te.read()
	if buf == nil {
		return nil
	}
	p, _ := parseGenericLongHeaderPacket(buf)
	tc := te.connForSource(p.dstConnID)
	d := parseTestDatagram(te.t, te, tc, buf)
	logDatagram(te.t, "-> endpoint under test sends", d)
	return d
}

// wantDatagram indicates that we expect the Endpoint to send a datagram.
func (te *testEndpoint) wantDatagram(expectation string, want *testDatagram) {
	te.t.Helper()
	got := te.readDatagram()
	if !datagramEqual(got, want) {
		te.t.Fatalf("%v:\ngot datagram:  %v\nwant datagram: %v", expectation, got, want)
	}
}

// wantIdle indicates that we expect the Endpoint to not send any more datagrams.
func (te *testEndpoint) wantIdle(expectation string) {
	if got := te.readDatagram(); got != nil {
		te.t.Fatalf("expect: %v\nunexpectedly got: %v", expectation, got)
	}
}

// advance causes time to pass.
func (te *testEndpoint) advance(d time.Duration) {
	te.t.Helper()
	te.advanceTo(te.now.Add(d))
}

// advanceTo sets the current time.
func (te *testEndpoint) advanceTo(now time.Time) {
	te.t.Helper()
	if te.now.After(now) {
		te.t.Fatalf("time moved backwards: %v -> %v", te.now, now)
	}
	te.now = now
	for _, tc := range te.conns {
		if !tc.timer.After(te.now) {
			tc.conn.sendMsg(timerEvent{})
			tc.wait()
		}
	}
}

// testEndpointHooks implements endpointTestHooks.
type testEndpointHooks testEndpoint

func (te *testEndpointHooks) timeNow() time.Time {
	return te.now
}

func (te *testEndpointHooks) newConn(c *Conn) {
	tc := newTestConnForConn(te.t, (*testEndpoint)(te), c)
	te.conns[c] = tc
}

// testEndpointUDPConn implements UDPConn.
type testEndpointUDPConn testEndpoint

func (te *testEndpointUDPConn) Close() error {
	close(te.recvc)
	return nil
}

func (te *testEndpointUDPConn) LocalAddr() netip.AddrPort {
	return netip.MustParseAddrPort("127.0.0.1:443")
}

func (te *testEndpointUDPConn) Read(f func(*datagram)) {
	for {
		select {
		case d, ok := <-te.recvc:
			if !ok {
				return
			}
			f(d)
		case <-te.idlec:
		}
	}
}

func (te *testEndpointUDPConn) Write(dgram datagram) error {
	te.sentDatagrams = append(te.sentDatagrams, append([]byte(nil), dgram.b...))
	return nil
}
