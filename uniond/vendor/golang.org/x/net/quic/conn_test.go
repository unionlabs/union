// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

package quic

import (
	"bytes"
	"context"
	"crypto/tls"
	"errors"
	"flag"
	"fmt"
	"log/slog"
	"math"
	"net/netip"
	"reflect"
	"strings"
	"testing"
	"time"

	"golang.org/x/net/quic/qlog"
)

var (
	testVV  = flag.Bool("vv", false, "even more verbose test output")
	qlogdir = flag.String("qlog", "", "write qlog logs to directory")
)

func TestConnTestConn(t *testing.T) {
	tc := newTestConn(t, serverSide)
	tc.handshake()
	if got, want := tc.timeUntilEvent(), defaultMaxIdleTimeout; got != want {
		t.Errorf("new conn timeout=%v, want %v (max_idle_timeout)", got, want)
	}

	ranAt, _ := runAsync(tc, func(ctx context.Context) (when time.Time, _ error) {
		tc.conn.runOnLoop(ctx, func(now time.Time, c *Conn) {
			when = now
		})
		return
	}).result()
	if !ranAt.Equal(tc.endpoint.now) {
		t.Errorf("func ran on loop at %v, want %v", ranAt, tc.endpoint.now)
	}
	tc.wait()

	nextTime := tc.endpoint.now.Add(defaultMaxIdleTimeout / 2)
	tc.advanceTo(nextTime)
	ranAt, _ = runAsync(tc, func(ctx context.Context) (when time.Time, _ error) {
		tc.conn.runOnLoop(ctx, func(now time.Time, c *Conn) {
			when = now
		})
		return
	}).result()
	if !ranAt.Equal(nextTime) {
		t.Errorf("func ran on loop at %v, want %v", ranAt, nextTime)
	}
	tc.wait()

	tc.advanceToTimer()
	if got := tc.conn.lifetime.state; got != connStateDone {
		t.Errorf("after advancing to idle timeout, conn state = %v, want done", got)
	}
}

type testDatagram struct {
	packets    []*testPacket
	paddedSize int
	addr       netip.AddrPort
}

func (d testDatagram) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "datagram with %v packets", len(d.packets))
	if d.paddedSize > 0 {
		fmt.Fprintf(&b, " (padded to %v bytes)", d.paddedSize)
	}
	b.WriteString(":")
	for _, p := range d.packets {
		b.WriteString("\n")
		b.WriteString(p.String())
	}
	return b.String()
}

type testPacket struct {
	ptype             packetType
	header            byte
	version           uint32
	num               packetNumber
	keyPhaseBit       bool
	keyNumber         int
	dstConnID         []byte
	srcConnID         []byte
	token             []byte
	originalDstConnID []byte // used for encoding Retry packets
	frames            []debugFrame
}

func (p testPacket) String() string {
	var b strings.Builder
	fmt.Fprintf(&b, "  %v %v", p.ptype, p.num)
	if p.version != 0 {
		fmt.Fprintf(&b, " version=%v", p.version)
	}
	if p.srcConnID != nil {
		fmt.Fprintf(&b, " src={%x}", p.srcConnID)
	}
	if p.dstConnID != nil {
		fmt.Fprintf(&b, " dst={%x}", p.dstConnID)
	}
	if p.token != nil {
		fmt.Fprintf(&b, " token={%x}", p.token)
	}
	for _, f := range p.frames {
		fmt.Fprintf(&b, "\n    %v", f)
	}
	return b.String()
}

// maxTestKeyPhases is the maximum number of 1-RTT keys we'll generate in a test.
const maxTestKeyPhases = 3

// A testConn is a Conn whose external interactions (sending and receiving packets,
// setting timers) can be manipulated in tests.
type testConn struct {
	t              *testing.T
	conn           *Conn
	endpoint       *testEndpoint
	timer          time.Time
	timerLastFired time.Time
	idlec          chan struct{} // only accessed on the conn's loop

	// Keys are distinct from the conn's keys,
	// because the test may know about keys before the conn does.
	// For example, when sending a datagram with coalesced
	// Initial and Handshake packets to a client conn,
	// we use Handshake keys to encrypt the packet.
	// The client only acquires those keys when it processes
	// the Initial packet.
	keysInitial   fixedKeyPair
	keysHandshake fixedKeyPair
	rkeyAppData   test1RTTKeys
	wkeyAppData   test1RTTKeys
	rsecrets      [numberSpaceCount]keySecret
	wsecrets      [numberSpaceCount]keySecret

	// testConn uses a test hook to snoop on the conn's TLS events.
	// CRYPTO data produced by the conn's QUICConn is placed in
	// cryptoDataOut.
	//
	// The peerTLSConn is is a QUICConn representing the peer.
	// CRYPTO data produced by the conn is written to peerTLSConn,
	// and data produced by peerTLSConn is placed in cryptoDataIn.
	cryptoDataOut map[tls.QUICEncryptionLevel][]byte
	cryptoDataIn  map[tls.QUICEncryptionLevel][]byte
	peerTLSConn   *tls.QUICConn

	// Information about the conn's (fake) peer.
	peerConnID        []byte                         // source conn id of peer's packets
	peerNextPacketNum [numberSpaceCount]packetNumber // next packet number to use

	// Datagrams, packets, and frames sent by the conn,
	// but not yet processed by the test.
	sentDatagrams [][]byte
	sentPackets   []*testPacket
	sentFrames    []debugFrame
	lastDatagram  *testDatagram
	lastPacket    *testPacket

	recvDatagram chan *datagram

	// Transport parameters sent by the conn.
	sentTransportParameters *transportParameters

	// Frame types to ignore in tests.
	ignoreFrames map[byte]bool

	// Values to set in packets sent to the conn.
	sendKeyNumber   int
	sendKeyPhaseBit bool

	asyncTestState
}

type test1RTTKeys struct {
	hdr headerKey
	pkt [maxTestKeyPhases]packetKey
}

type keySecret struct {
	suite  uint16
	secret []byte
}

// newTestConn creates a Conn for testing.
//
// The Conn's event loop is controlled by the test,
// allowing test code to access Conn state directly
// by first ensuring the loop goroutine is idle.
func newTestConn(t *testing.T, side connSide, opts ...any) *testConn {
	t.Helper()
	config := &Config{
		TLSConfig:         newTestTLSConfig(side),
		StatelessResetKey: testStatelessResetKey,
		QLogLogger: slog.New(qlog.NewJSONHandler(qlog.HandlerOptions{
			Level: QLogLevelFrame,
			Dir:   *qlogdir,
		})),
	}
	var cids newServerConnIDs
	if side == serverSide {
		// The initial connection ID for the server is chosen by the client.
		cids.srcConnID = testPeerConnID(0)
		cids.dstConnID = testPeerConnID(-1)
		cids.originalDstConnID = cids.dstConnID
	}
	var configTransportParams []func(*transportParameters)
	var configTestConn []func(*testConn)
	for _, o := range opts {
		switch o := o.(type) {
		case func(*Config):
			o(config)
		case func(*tls.Config):
			o(config.TLSConfig)
		case func(cids *newServerConnIDs):
			o(&cids)
		case func(p *transportParameters):
			configTransportParams = append(configTransportParams, o)
		case func(p *testConn):
			configTestConn = append(configTestConn, o)
		default:
			t.Fatalf("unknown newTestConn option %T", o)
		}
	}

	endpoint := newTestEndpoint(t, config)
	endpoint.configTransportParams = configTransportParams
	endpoint.configTestConn = configTestConn
	conn, err := endpoint.e.newConn(
		endpoint.now,
		config,
		side,
		cids,
		"",
		netip.MustParseAddrPort("127.0.0.1:443"))
	if err != nil {
		t.Fatal(err)
	}
	tc := endpoint.conns[conn]
	tc.wait()
	return tc
}

func newTestConnForConn(t *testing.T, endpoint *testEndpoint, conn *Conn) *testConn {
	t.Helper()
	tc := &testConn{
		t:          t,
		endpoint:   endpoint,
		conn:       conn,
		peerConnID: testPeerConnID(0),
		ignoreFrames: map[byte]bool{
			frameTypePadding: true, // ignore PADDING by default
		},
		cryptoDataOut: make(map[tls.QUICEncryptionLevel][]byte),
		cryptoDataIn:  make(map[tls.QUICEncryptionLevel][]byte),
		recvDatagram:  make(chan *datagram),
	}
	t.Cleanup(tc.cleanup)
	for _, f := range endpoint.configTestConn {
		f(tc)
	}
	conn.testHooks = (*testConnHooks)(tc)

	if endpoint.peerTLSConn != nil {
		tc.peerTLSConn = endpoint.peerTLSConn
		endpoint.peerTLSConn = nil
		return tc
	}

	peerProvidedParams := defaultTransportParameters()
	peerProvidedParams.initialSrcConnID = testPeerConnID(0)
	if conn.side == clientSide {
		peerProvidedParams.originalDstConnID = testLocalConnID(-1)
	}
	for _, f := range endpoint.configTransportParams {
		f(&peerProvidedParams)
	}

	peerQUICConfig := &tls.QUICConfig{TLSConfig: newTestTLSConfig(conn.side.peer())}
	if conn.side == clientSide {
		tc.peerTLSConn = tls.QUICServer(peerQUICConfig)
	} else {
		tc.peerTLSConn = tls.QUICClient(peerQUICConfig)
	}
	tc.peerTLSConn.SetTransportParameters(marshalTransportParameters(peerProvidedParams))
	tc.peerTLSConn.Start(context.Background())
	t.Cleanup(func() {
		tc.peerTLSConn.Close()
	})

	return tc
}

// advance causes time to pass.
func (tc *testConn) advance(d time.Duration) {
	tc.t.Helper()
	tc.endpoint.advance(d)
}

// advanceTo sets the current time.
func (tc *testConn) advanceTo(now time.Time) {
	tc.t.Helper()
	tc.endpoint.advanceTo(now)
}

// advanceToTimer sets the current time to the time of the Conn's next timer event.
func (tc *testConn) advanceToTimer() {
	if tc.timer.IsZero() {
		tc.t.Fatalf("advancing to timer, but timer is not set")
	}
	tc.advanceTo(tc.timer)
}

func (tc *testConn) timerDelay() time.Duration {
	if tc.timer.IsZero() {
		return math.MaxInt64 // infinite
	}
	if tc.timer.Before(tc.endpoint.now) {
		return 0
	}
	return tc.timer.Sub(tc.endpoint.now)
}

const infiniteDuration = time.Duration(math.MaxInt64)

// timeUntilEvent returns the amount of time until the next connection event.
func (tc *testConn) timeUntilEvent() time.Duration {
	if tc.timer.IsZero() {
		return infiniteDuration
	}
	if tc.timer.Before(tc.endpoint.now) {
		return 0
	}
	return tc.timer.Sub(tc.endpoint.now)
}

// wait blocks until the conn becomes idle.
// The conn is idle when it is blocked waiting for a packet to arrive or a timer to expire.
// Tests shouldn't need to call wait directly.
// testConn methods that wake the Conn event loop will call wait for them.
func (tc *testConn) wait() {
	tc.t.Helper()
	idlec := make(chan struct{})
	fail := false
	tc.conn.sendMsg(func(now time.Time, c *Conn) {
		if tc.idlec != nil {
			tc.t.Errorf("testConn.wait called concurrently")
			fail = true
			close(idlec)
		} else {
			// nextMessage will close idlec.
			tc.idlec = idlec
		}
	})
	select {
	case <-idlec:
	case <-tc.conn.donec:
		// We may have async ops that can proceed now that the conn is done.
		tc.wakeAsync()
	}
	if fail {
		panic(fail)
	}
}

func (tc *testConn) cleanup() {
	if tc.conn == nil {
		return
	}
	tc.conn.exit()
	<-tc.conn.donec
}

func (tc *testConn) acceptStream() *Stream {
	tc.t.Helper()
	s, err := tc.conn.AcceptStream(canceledContext())
	if err != nil {
		tc.t.Fatalf("conn.AcceptStream() = %v, want stream", err)
	}
	s.SetReadContext(canceledContext())
	s.SetWriteContext(canceledContext())
	return s
}

func logDatagram(t *testing.T, text string, d *testDatagram) {
	t.Helper()
	if !*testVV {
		return
	}
	pad := ""
	if d.paddedSize > 0 {
		pad = fmt.Sprintf(" (padded to %v)", d.paddedSize)
	}
	t.Logf("%v datagram%v", text, pad)
	for _, p := range d.packets {
		var s string
		switch p.ptype {
		case packetType1RTT:
			s = fmt.Sprintf("  %v pnum=%v", p.ptype, p.num)
		default:
			s = fmt.Sprintf("  %v pnum=%v ver=%v dst={%x} src={%x}", p.ptype, p.num, p.version, p.dstConnID, p.srcConnID)
		}
		if p.token != nil {
			s += fmt.Sprintf(" token={%x}", p.token)
		}
		if p.keyPhaseBit {
			s += fmt.Sprintf(" KeyPhase")
		}
		if p.keyNumber != 0 {
			s += fmt.Sprintf(" keynum=%v", p.keyNumber)
		}
		t.Log(s)
		for _, f := range p.frames {
			t.Logf("    %v", f)
		}
	}
}

// write sends the Conn a datagram.
func (tc *testConn) write(d *testDatagram) {
	tc.t.Helper()
	tc.endpoint.writeDatagram(d)
}

// writeFrame sends the Conn a datagram containing the given frames.
func (tc *testConn) writeFrames(ptype packetType, frames ...debugFrame) {
	tc.t.Helper()
	space := spaceForPacketType(ptype)
	dstConnID := tc.conn.connIDState.local[0].cid
	if tc.conn.connIDState.local[0].seq == -1 && ptype != packetTypeInitial {
		// Only use the transient connection ID in Initial packets.
		dstConnID = tc.conn.connIDState.local[1].cid
	}
	d := &testDatagram{
		packets: []*testPacket{{
			ptype:       ptype,
			num:         tc.peerNextPacketNum[space],
			keyNumber:   tc.sendKeyNumber,
			keyPhaseBit: tc.sendKeyPhaseBit,
			frames:      frames,
			version:     quicVersion1,
			dstConnID:   dstConnID,
			srcConnID:   tc.peerConnID,
		}},
		addr: tc.conn.peerAddr,
	}
	if ptype == packetTypeInitial && tc.conn.side == serverSide {
		d.paddedSize = 1200
	}
	tc.write(d)
}

// writeAckForAll sends the Conn a datagram containing an ack for all packets up to the
// last one received.
func (tc *testConn) writeAckForAll() {
	tc.t.Helper()
	if tc.lastPacket == nil {
		return
	}
	tc.writeFrames(tc.lastPacket.ptype, debugFrameAck{
		ranges: []i64range[packetNumber]{{0, tc.lastPacket.num + 1}},
	})
}

// writeAckForLatest sends the Conn a datagram containing an ack for the
// most recent packet received.
func (tc *testConn) writeAckForLatest() {
	tc.t.Helper()
	if tc.lastPacket == nil {
		return
	}
	tc.writeFrames(tc.lastPacket.ptype, debugFrameAck{
		ranges: []i64range[packetNumber]{{tc.lastPacket.num, tc.lastPacket.num + 1}},
	})
}

// ignoreFrame hides frames of the given type sent by the Conn.
func (tc *testConn) ignoreFrame(frameType byte) {
	tc.ignoreFrames[frameType] = true
}

// readDatagram reads the next datagram sent by the Conn.
// It returns nil if the Conn has no more datagrams to send at this time.
func (tc *testConn) readDatagram() *testDatagram {
	tc.t.Helper()
	tc.wait()
	tc.sentPackets = nil
	tc.sentFrames = nil
	buf := tc.endpoint.read()
	if buf == nil {
		return nil
	}
	d := parseTestDatagram(tc.t, tc.endpoint, tc, buf)
	// Log the datagram before removing ignored frames.
	// When things go wrong, it's useful to see all the frames.
	logDatagram(tc.t, "-> conn under test sends", d)
	typeForFrame := func(f debugFrame) byte {
		// This is very clunky, and points at a problem
		// in how we specify what frames to ignore in tests.
		//
		// We mark frames to ignore using the frame type,
		// but we've got a debugFrame data structure here.
		// Perhaps we should be ignoring frames by debugFrame
		// type instead: tc.ignoreFrame[debugFrameAck]().
		switch f := f.(type) {
		case debugFramePadding:
			return frameTypePadding
		case debugFramePing:
			return frameTypePing
		case debugFrameAck:
			return frameTypeAck
		case debugFrameResetStream:
			return frameTypeResetStream
		case debugFrameStopSending:
			return frameTypeStopSending
		case debugFrameCrypto:
			return frameTypeCrypto
		case debugFrameNewToken:
			return frameTypeNewToken
		case debugFrameStream:
			return frameTypeStreamBase
		case debugFrameMaxData:
			return frameTypeMaxData
		case debugFrameMaxStreamData:
			return frameTypeMaxStreamData
		case debugFrameMaxStreams:
			if f.streamType == bidiStream {
				return frameTypeMaxStreamsBidi
			} else {
				return frameTypeMaxStreamsUni
			}
		case debugFrameDataBlocked:
			return frameTypeDataBlocked
		case debugFrameStreamDataBlocked:
			return frameTypeStreamDataBlocked
		case debugFrameStreamsBlocked:
			if f.streamType == bidiStream {
				return frameTypeStreamsBlockedBidi
			} else {
				return frameTypeStreamsBlockedUni
			}
		case debugFrameNewConnectionID:
			return frameTypeNewConnectionID
		case debugFrameRetireConnectionID:
			return frameTypeRetireConnectionID
		case debugFramePathChallenge:
			return frameTypePathChallenge
		case debugFramePathResponse:
			return frameTypePathResponse
		case debugFrameConnectionCloseTransport:
			return frameTypeConnectionCloseTransport
		case debugFrameConnectionCloseApplication:
			return frameTypeConnectionCloseApplication
		case debugFrameHandshakeDone:
			return frameTypeHandshakeDone
		}
		panic(fmt.Errorf("unhandled frame type %T", f))
	}
	for _, p := range d.packets {
		var frames []debugFrame
		for _, f := range p.frames {
			if !tc.ignoreFrames[typeForFrame(f)] {
				frames = append(frames, f)
			}
		}
		p.frames = frames
	}
	tc.lastDatagram = d
	return d
}

// readPacket reads the next packet sent by the Conn.
// It returns nil if the Conn has no more packets to send at this time.
func (tc *testConn) readPacket() *testPacket {
	tc.t.Helper()
	for len(tc.sentPackets) == 0 {
		d := tc.readDatagram()
		if d == nil {
			return nil
		}
		for _, p := range d.packets {
			if len(p.frames) == 0 {
				tc.lastPacket = p
				continue
			}
			tc.sentPackets = append(tc.sentPackets, p)
		}
	}
	p := tc.sentPackets[0]
	tc.sentPackets = tc.sentPackets[1:]
	tc.lastPacket = p
	return p
}

// readFrame reads the next frame sent by the Conn.
// It returns nil if the Conn has no more frames to send at this time.
func (tc *testConn) readFrame() (debugFrame, packetType) {
	tc.t.Helper()
	for len(tc.sentFrames) == 0 {
		p := tc.readPacket()
		if p == nil {
			return nil, packetTypeInvalid
		}
		tc.sentFrames = p.frames
	}
	f := tc.sentFrames[0]
	tc.sentFrames = tc.sentFrames[1:]
	return f, tc.lastPacket.ptype
}

// wantDatagram indicates that we expect the Conn to send a datagram.
func (tc *testConn) wantDatagram(expectation string, want *testDatagram) {
	tc.t.Helper()
	got := tc.readDatagram()
	if !datagramEqual(got, want) {
		tc.t.Fatalf("%v:\ngot datagram:  %v\nwant datagram: %v", expectation, got, want)
	}
}

func datagramEqual(a, b *testDatagram) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	if a.paddedSize != b.paddedSize ||
		a.addr != b.addr ||
		len(a.packets) != len(b.packets) {
		return false
	}
	for i := range a.packets {
		if !packetEqual(a.packets[i], b.packets[i]) {
			return false
		}
	}
	return true
}

// wantPacket indicates that we expect the Conn to send a packet.
func (tc *testConn) wantPacket(expectation string, want *testPacket) {
	tc.t.Helper()
	got := tc.readPacket()
	if !packetEqual(got, want) {
		tc.t.Fatalf("%v:\ngot packet:  %v\nwant packet: %v", expectation, got, want)
	}
}

func packetEqual(a, b *testPacket) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	ac := *a
	ac.frames = nil
	ac.header = 0
	bc := *b
	bc.frames = nil
	bc.header = 0
	if !reflect.DeepEqual(ac, bc) {
		return false
	}
	if len(a.frames) != len(b.frames) {
		return false
	}
	for i := range a.frames {
		if !frameEqual(a.frames[i], b.frames[i]) {
			return false
		}
	}
	return true
}

// wantFrame indicates that we expect the Conn to send a frame.
func (tc *testConn) wantFrame(expectation string, wantType packetType, want debugFrame) {
	tc.t.Helper()
	got, gotType := tc.readFrame()
	if got == nil {
		tc.t.Fatalf("%v:\nconnection is idle\nwant %v frame: %v", expectation, wantType, want)
	}
	if gotType != wantType {
		tc.t.Fatalf("%v:\ngot %v packet, want %v\ngot frame:  %v", expectation, gotType, wantType, got)
	}
	if !frameEqual(got, want) {
		tc.t.Fatalf("%v:\ngot frame:  %v\nwant frame: %v", expectation, got, want)
	}
}

func frameEqual(a, b debugFrame) bool {
	switch af := a.(type) {
	case debugFrameConnectionCloseTransport:
		bf, ok := b.(debugFrameConnectionCloseTransport)
		return ok && af.code == bf.code
	}
	return reflect.DeepEqual(a, b)
}

// wantFrameType indicates that we expect the Conn to send a frame,
// although we don't care about the contents.
func (tc *testConn) wantFrameType(expectation string, wantType packetType, want debugFrame) {
	tc.t.Helper()
	got, gotType := tc.readFrame()
	if got == nil {
		tc.t.Fatalf("%v:\nconnection is idle\nwant %v frame: %v", expectation, wantType, want)
	}
	if gotType != wantType {
		tc.t.Fatalf("%v:\ngot %v packet, want %v\ngot frame:  %v", expectation, gotType, wantType, got)
	}
	if reflect.TypeOf(got) != reflect.TypeOf(want) {
		tc.t.Fatalf("%v:\ngot frame:  %v\nwant frame of type: %v", expectation, got, want)
	}
}

// wantIdle indicates that we expect the Conn to not send any more frames.
func (tc *testConn) wantIdle(expectation string) {
	tc.t.Helper()
	switch {
	case len(tc.sentFrames) > 0:
		tc.t.Fatalf("expect: %v\nunexpectedly got: %v", expectation, tc.sentFrames[0])
	case len(tc.sentPackets) > 0:
		tc.t.Fatalf("expect: %v\nunexpectedly got: %v", expectation, tc.sentPackets[0])
	}
	if f, _ := tc.readFrame(); f != nil {
		tc.t.Fatalf("expect: %v\nunexpectedly got: %v", expectation, f)
	}
}

func encodeTestPacket(t *testing.T, tc *testConn, p *testPacket, pad int) []byte {
	t.Helper()
	var w packetWriter
	w.reset(1200)
	var pnumMaxAcked packetNumber
	switch p.ptype {
	case packetTypeRetry:
		return encodeRetryPacket(p.originalDstConnID, retryPacket{
			srcConnID: p.srcConnID,
			dstConnID: p.dstConnID,
			token:     p.token,
		})
	case packetType1RTT:
		w.start1RTTPacket(p.num, pnumMaxAcked, p.dstConnID)
	default:
		w.startProtectedLongHeaderPacket(pnumMaxAcked, longPacket{
			ptype:     p.ptype,
			version:   p.version,
			num:       p.num,
			dstConnID: p.dstConnID,
			srcConnID: p.srcConnID,
			extra:     p.token,
		})
	}
	for _, f := range p.frames {
		f.write(&w)
	}
	w.appendPaddingTo(pad)
	if p.ptype != packetType1RTT {
		var k fixedKeys
		if tc == nil {
			if p.ptype == packetTypeInitial {
				k = initialKeys(p.dstConnID, serverSide).r
			} else {
				t.Fatalf("sending %v packet with no conn", p.ptype)
			}
		} else {
			switch p.ptype {
			case packetTypeInitial:
				k = tc.keysInitial.w
			case packetTypeHandshake:
				k = tc.keysHandshake.w
			}
		}
		if !k.isSet() {
			t.Fatalf("sending %v packet with no write key", p.ptype)
		}
		w.finishProtectedLongHeaderPacket(pnumMaxAcked, k, longPacket{
			ptype:     p.ptype,
			version:   p.version,
			num:       p.num,
			dstConnID: p.dstConnID,
			srcConnID: p.srcConnID,
			extra:     p.token,
		})
	} else {
		if tc == nil || !tc.wkeyAppData.hdr.isSet() {
			t.Fatalf("sending 1-RTT packet with no write key")
		}
		// Somewhat hackish: Generate a temporary updatingKeyPair that will
		// always use our desired key phase.
		k := &updatingKeyPair{
			w: updatingKeys{
				hdr: tc.wkeyAppData.hdr,
				pkt: [2]packetKey{
					tc.wkeyAppData.pkt[p.keyNumber],
					tc.wkeyAppData.pkt[p.keyNumber],
				},
			},
			updateAfter: maxPacketNumber,
		}
		if p.keyPhaseBit {
			k.phase |= keyPhaseBit
		}
		w.finish1RTTPacket(p.num, pnumMaxAcked, p.dstConnID, k)
	}
	return w.datagram()
}

func parseTestDatagram(t *testing.T, te *testEndpoint, tc *testConn, buf []byte) *testDatagram {
	t.Helper()
	bufSize := len(buf)
	d := &testDatagram{}
	size := len(buf)
	for len(buf) > 0 {
		if buf[0] == 0 {
			d.paddedSize = bufSize
			break
		}
		ptype := getPacketType(buf)
		switch ptype {
		case packetTypeRetry:
			retry, ok := parseRetryPacket(buf, te.lastInitialDstConnID)
			if !ok {
				t.Fatalf("could not parse %v packet", ptype)
			}
			return &testDatagram{
				packets: []*testPacket{{
					ptype:     packetTypeRetry,
					dstConnID: retry.dstConnID,
					srcConnID: retry.srcConnID,
					token:     retry.token,
				}},
			}
		case packetTypeInitial, packetTypeHandshake:
			var k fixedKeys
			if tc == nil {
				if ptype == packetTypeInitial {
					p, _ := parseGenericLongHeaderPacket(buf)
					k = initialKeys(p.srcConnID, serverSide).w
				} else {
					t.Fatalf("reading %v packet with no conn", ptype)
				}
			} else {
				switch ptype {
				case packetTypeInitial:
					k = tc.keysInitial.r
				case packetTypeHandshake:
					k = tc.keysHandshake.r
				}
			}
			if !k.isSet() {
				t.Fatalf("reading %v packet with no read key", ptype)
			}
			var pnumMax packetNumber // TODO: Track packet numbers.
			p, n := parseLongHeaderPacket(buf, k, pnumMax)
			if n < 0 {
				t.Fatalf("packet parse error")
			}
			frames, err := parseTestFrames(t, p.payload)
			if err != nil {
				t.Fatal(err)
			}
			var token []byte
			if ptype == packetTypeInitial && len(p.extra) > 0 {
				token = p.extra
			}
			d.packets = append(d.packets, &testPacket{
				ptype:     p.ptype,
				header:    buf[0],
				version:   p.version,
				num:       p.num,
				dstConnID: p.dstConnID,
				srcConnID: p.srcConnID,
				token:     token,
				frames:    frames,
			})
			buf = buf[n:]
		case packetType1RTT:
			if tc == nil || !tc.rkeyAppData.hdr.isSet() {
				t.Fatalf("reading 1-RTT packet with no read key")
			}
			var pnumMax packetNumber // TODO: Track packet numbers.
			pnumOff := 1 + len(tc.peerConnID)
			// Try unprotecting the packet with the first maxTestKeyPhases keys.
			var phase int
			var pnum packetNumber
			var hdr []byte
			var pay []byte
			var err error
			for phase = 0; phase < maxTestKeyPhases; phase++ {
				b := append([]byte{}, buf...)
				hdr, pay, pnum, err = tc.rkeyAppData.hdr.unprotect(b, pnumOff, pnumMax)
				if err != nil {
					t.Fatalf("1-RTT packet header parse error")
				}
				k := tc.rkeyAppData.pkt[phase]
				pay, err = k.unprotect(hdr, pay, pnum)
				if err == nil {
					break
				}
			}
			if err != nil {
				t.Fatalf("1-RTT packet payload parse error")
			}
			frames, err := parseTestFrames(t, pay)
			if err != nil {
				t.Fatal(err)
			}
			d.packets = append(d.packets, &testPacket{
				ptype:       packetType1RTT,
				header:      hdr[0],
				num:         pnum,
				dstConnID:   hdr[1:][:len(tc.peerConnID)],
				keyPhaseBit: hdr[0]&keyPhaseBit != 0,
				keyNumber:   phase,
				frames:      frames,
			})
			buf = buf[len(buf):]
		default:
			t.Fatalf("unhandled packet type %v", ptype)
		}
	}
	// This is rather hackish: If the last frame in the last packet
	// in the datagram is PADDING, then remove it and record
	// the padded size in the testDatagram.paddedSize.
	//
	// This makes it easier to write a test that expects a datagram
	// padded to 1200 bytes.
	if len(d.packets) > 0 && len(d.packets[len(d.packets)-1].frames) > 0 {
		p := d.packets[len(d.packets)-1]
		f := p.frames[len(p.frames)-1]
		if _, ok := f.(debugFramePadding); ok {
			p.frames = p.frames[:len(p.frames)-1]
			d.paddedSize = size
		}
	}
	return d
}

func parseTestFrames(t *testing.T, payload []byte) ([]debugFrame, error) {
	t.Helper()
	var frames []debugFrame
	for len(payload) > 0 {
		f, n := parseDebugFrame(payload)
		if n < 0 {
			return nil, errors.New("error parsing frames")
		}
		frames = append(frames, f)
		payload = payload[n:]
	}
	return frames, nil
}

func spaceForPacketType(ptype packetType) numberSpace {
	switch ptype {
	case packetTypeInitial:
		return initialSpace
	case packetType0RTT:
		panic("TODO: packetType0RTT")
	case packetTypeHandshake:
		return handshakeSpace
	case packetTypeRetry:
		panic("retry packets have no number space")
	case packetType1RTT:
		return appDataSpace
	}
	panic("unknown packet type")
}

// testConnHooks implements connTestHooks.
type testConnHooks testConn

func (tc *testConnHooks) init() {
	tc.conn.keysAppData.updateAfter = maxPacketNumber // disable key updates
	tc.keysInitial.r = tc.conn.keysInitial.w
	tc.keysInitial.w = tc.conn.keysInitial.r
	if tc.conn.side == serverSide {
		tc.endpoint.acceptQueue = append(tc.endpoint.acceptQueue, (*testConn)(tc))
	}
}

// handleTLSEvent processes TLS events generated by
// the connection under test's tls.QUICConn.
//
// We maintain a second tls.QUICConn representing the peer,
// and feed the TLS handshake data into it.
//
// We stash TLS handshake data from both sides in the testConn,
// where it can be used by tests.
//
// We snoop packet protection keys out of the tls.QUICConns,
// and verify that both sides of the connection are getting
// matching keys.
func (tc *testConnHooks) handleTLSEvent(e tls.QUICEvent) {
	checkKey := func(typ string, secrets *[numberSpaceCount]keySecret, e tls.QUICEvent) {
		var space numberSpace
		switch {
		case e.Level == tls.QUICEncryptionLevelHandshake:
			space = handshakeSpace
		case e.Level == tls.QUICEncryptionLevelApplication:
			space = appDataSpace
		default:
			tc.t.Errorf("unexpected encryption level %v", e.Level)
			return
		}
		if secrets[space].secret == nil {
			secrets[space].suite = e.Suite
			secrets[space].secret = append([]byte{}, e.Data...)
		} else if secrets[space].suite != e.Suite || !bytes.Equal(secrets[space].secret, e.Data) {
			tc.t.Errorf("%v key mismatch for level for level %v", typ, e.Level)
		}
	}
	setAppDataKey := func(suite uint16, secret []byte, k *test1RTTKeys) {
		k.hdr.init(suite, secret)
		for i := 0; i < len(k.pkt); i++ {
			k.pkt[i].init(suite, secret)
			secret = updateSecret(suite, secret)
		}
	}
	switch e.Kind {
	case tls.QUICSetReadSecret:
		checkKey("write", &tc.wsecrets, e)
		switch e.Level {
		case tls.QUICEncryptionLevelHandshake:
			tc.keysHandshake.w.init(e.Suite, e.Data)
		case tls.QUICEncryptionLevelApplication:
			setAppDataKey(e.Suite, e.Data, &tc.wkeyAppData)
		}
	case tls.QUICSetWriteSecret:
		checkKey("read", &tc.rsecrets, e)
		switch e.Level {
		case tls.QUICEncryptionLevelHandshake:
			tc.keysHandshake.r.init(e.Suite, e.Data)
		case tls.QUICEncryptionLevelApplication:
			setAppDataKey(e.Suite, e.Data, &tc.rkeyAppData)
		}
	case tls.QUICWriteData:
		tc.cryptoDataOut[e.Level] = append(tc.cryptoDataOut[e.Level], e.Data...)
		tc.peerTLSConn.HandleData(e.Level, e.Data)
	}
	for {
		e := tc.peerTLSConn.NextEvent()
		switch e.Kind {
		case tls.QUICNoEvent:
			return
		case tls.QUICSetReadSecret:
			checkKey("write", &tc.rsecrets, e)
			switch e.Level {
			case tls.QUICEncryptionLevelHandshake:
				tc.keysHandshake.r.init(e.Suite, e.Data)
			case tls.QUICEncryptionLevelApplication:
				setAppDataKey(e.Suite, e.Data, &tc.rkeyAppData)
			}
		case tls.QUICSetWriteSecret:
			checkKey("read", &tc.wsecrets, e)
			switch e.Level {
			case tls.QUICEncryptionLevelHandshake:
				tc.keysHandshake.w.init(e.Suite, e.Data)
			case tls.QUICEncryptionLevelApplication:
				setAppDataKey(e.Suite, e.Data, &tc.wkeyAppData)
			}
		case tls.QUICWriteData:
			tc.cryptoDataIn[e.Level] = append(tc.cryptoDataIn[e.Level], e.Data...)
		case tls.QUICTransportParameters:
			p, err := unmarshalTransportParams(e.Data)
			if err != nil {
				tc.t.Logf("sent unparseable transport parameters %x %v", e.Data, err)
			} else {
				tc.sentTransportParameters = &p
			}
		}
	}
}

// nextMessage is called by the Conn's event loop to request its next event.
func (tc *testConnHooks) nextMessage(msgc chan any, timer time.Time) (now time.Time, m any) {
	tc.timer = timer
	for {
		if !timer.IsZero() && !timer.After(tc.endpoint.now) {
			if timer.Equal(tc.timerLastFired) {
				// If the connection timer fires at time T, the Conn should take some
				// action to advance the timer into the future. If the Conn reschedules
				// the timer for the same time, it isn't making progress and we have a bug.
				tc.t.Errorf("connection timer spinning; now=%v timer=%v", tc.endpoint.now, timer)
			} else {
				tc.timerLastFired = timer
				return tc.endpoint.now, timerEvent{}
			}
		}
		select {
		case m := <-msgc:
			return tc.endpoint.now, m
		default:
		}
		if !tc.wakeAsync() {
			break
		}
	}
	// If the message queue is empty, then the conn is idle.
	if tc.idlec != nil {
		idlec := tc.idlec
		tc.idlec = nil
		close(idlec)
	}
	m = <-msgc
	return tc.endpoint.now, m
}

func (tc *testConnHooks) newConnID(seq int64) ([]byte, error) {
	return testLocalConnID(seq), nil
}

func (tc *testConnHooks) timeNow() time.Time {
	return tc.endpoint.now
}

// testLocalConnID returns the connection ID with a given sequence number
// used by a Conn under test.
func testLocalConnID(seq int64) []byte {
	cid := make([]byte, connIDLen)
	copy(cid, []byte{0xc0, 0xff, 0xee})
	cid[len(cid)-1] = byte(seq)
	return cid
}

// testPeerConnID returns the connection ID with a given sequence number
// used by the fake peer of a Conn under test.
func testPeerConnID(seq int64) []byte {
	// Use a different length than we choose for our own conn ids,
	// to help catch any bad assumptions.
	return []byte{0xbe, 0xee, 0xff, byte(seq)}
}

func testPeerStatelessResetToken(seq int64) statelessResetToken {
	return statelessResetToken{
		0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee,
		0xee, 0xee, 0xee, 0xee, 0xee, 0xee, 0xee, byte(seq),
	}
}

// canceledContext returns a canceled Context.
//
// Functions which take a context preference progress over cancelation.
// For example, a read with a canceled context will return data if any is available.
// Tests use canceled contexts to perform non-blocking operations.
func canceledContext() context.Context {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	return ctx
}
