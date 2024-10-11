// Copyright 2013 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package ssh

import (
	"errors"
	"fmt"
	"io"
	"sync"
	"testing"
)

func muxPair() (*mux, *mux) {
	a, b := memPipe()

	s := newMux(a)
	c := newMux(b)

	return s, c
}

// Returns both ends of a channel, and the mux for the 2nd
// channel.
func channelPair(t *testing.T) (*channel, *channel, *mux) {
	c, s := muxPair()

	res := make(chan *channel, 1)
	go func() {
		newCh, ok := <-s.incomingChannels
		if !ok {
			t.Error("no incoming channel")
			close(res)
			return
		}
		if newCh.ChannelType() != "chan" {
			t.Errorf("got type %q want chan", newCh.ChannelType())
			newCh.Reject(Prohibited, fmt.Sprintf("got type %q want chan", newCh.ChannelType()))
			close(res)
			return
		}
		ch, _, err := newCh.Accept()
		if err != nil {
			t.Errorf("accept: %v", err)
			close(res)
			return
		}
		res <- ch.(*channel)
	}()

	ch, err := c.openChannel("chan", nil)
	if err != nil {
		t.Fatalf("OpenChannel: %v", err)
	}
	w := <-res
	if w == nil {
		t.Fatal("unable to get write channel")
	}

	return w, ch, c
}

// Test that stderr and stdout can be addressed from different
// goroutines. This is intended for use with the race detector.
func TestMuxChannelExtendedThreadSafety(t *testing.T) {
	writer, reader, mux := channelPair(t)
	defer writer.Close()
	defer reader.Close()
	defer mux.Close()

	var wr, rd sync.WaitGroup
	magic := "hello world"

	wr.Add(2)
	go func() {
		io.WriteString(writer, magic)
		wr.Done()
	}()
	go func() {
		io.WriteString(writer.Stderr(), magic)
		wr.Done()
	}()

	rd.Add(2)
	go func() {
		c, err := io.ReadAll(reader)
		if string(c) != magic {
			t.Errorf("stdout read got %q, want %q (error %s)", c, magic, err)
		}
		rd.Done()
	}()
	go func() {
		c, err := io.ReadAll(reader.Stderr())
		if string(c) != magic {
			t.Errorf("stderr read got %q, want %q (error %s)", c, magic, err)
		}
		rd.Done()
	}()

	wr.Wait()
	writer.CloseWrite()
	rd.Wait()
}

func TestMuxReadWrite(t *testing.T) {
	s, c, mux := channelPair(t)
	defer s.Close()
	defer c.Close()
	defer mux.Close()

	magic := "hello world"
	magicExt := "hello stderr"
	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()
		_, err := s.Write([]byte(magic))
		if err != nil {
			t.Errorf("Write: %v", err)
			return
		}
		_, err = s.Extended(1).Write([]byte(magicExt))
		if err != nil {
			t.Errorf("Write: %v", err)
			return
		}
	}()

	var buf [1024]byte
	n, err := c.Read(buf[:])
	if err != nil {
		t.Fatalf("server Read: %v", err)
	}
	got := string(buf[:n])
	if got != magic {
		t.Fatalf("server: got %q want %q", got, magic)
	}

	n, err = c.Extended(1).Read(buf[:])
	if err != nil {
		t.Fatalf("server Read: %v", err)
	}

	got = string(buf[:n])
	if got != magicExt {
		t.Fatalf("server: got %q want %q", got, magic)
	}
}

func TestMuxChannelOverflow(t *testing.T) {
	reader, writer, mux := channelPair(t)
	defer reader.Close()
	defer writer.Close()
	defer mux.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()
		if _, err := writer.Write(make([]byte, channelWindowSize)); err != nil {
			t.Errorf("could not fill window: %v", err)
		}
		writer.Write(make([]byte, 1))
	}()
	writer.remoteWin.waitWriterBlocked()

	// Send 1 byte.
	packet := make([]byte, 1+4+4+1)
	packet[0] = msgChannelData
	marshalUint32(packet[1:], writer.remoteId)
	marshalUint32(packet[5:], uint32(1))
	packet[9] = 42

	if err := writer.mux.conn.writePacket(packet); err != nil {
		t.Errorf("could not send packet")
	}
	if _, err := reader.SendRequest("hello", true, nil); err == nil {
		t.Errorf("SendRequest succeeded.")
	}
}

func TestMuxChannelReadUnblock(t *testing.T) {
	reader, writer, mux := channelPair(t)
	defer reader.Close()
	defer writer.Close()
	defer mux.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()
		if _, err := writer.Write(make([]byte, channelWindowSize)); err != nil {
			t.Errorf("could not fill window: %v", err)
		}
		if _, err := writer.Write(make([]byte, 1)); err != nil {
			t.Errorf("Write: %v", err)
		}
		writer.Close()
	}()

	writer.remoteWin.waitWriterBlocked()

	buf := make([]byte, 32768)
	for {
		_, err := reader.Read(buf)
		if err == io.EOF {
			break
		}
		if err != nil {
			t.Fatalf("Read: %v", err)
		}
	}
}

func TestMuxChannelCloseWriteUnblock(t *testing.T) {
	reader, writer, mux := channelPair(t)
	defer reader.Close()
	defer writer.Close()
	defer mux.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()
		if _, err := writer.Write(make([]byte, channelWindowSize)); err != nil {
			t.Errorf("could not fill window: %v", err)
		}
		if _, err := writer.Write(make([]byte, 1)); err != io.EOF {
			t.Errorf("got %v, want EOF for unblock write", err)
		}
	}()

	writer.remoteWin.waitWriterBlocked()
	reader.Close()
}

func TestMuxConnectionCloseWriteUnblock(t *testing.T) {
	reader, writer, mux := channelPair(t)
	defer reader.Close()
	defer writer.Close()
	defer mux.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()
		if _, err := writer.Write(make([]byte, channelWindowSize)); err != nil {
			t.Errorf("could not fill window: %v", err)
		}
		if _, err := writer.Write(make([]byte, 1)); err != io.EOF {
			t.Errorf("got %v, want EOF for unblock write", err)
		}
	}()

	writer.remoteWin.waitWriterBlocked()
	mux.Close()
}

func TestMuxReject(t *testing.T) {
	client, server := muxPair()
	defer server.Close()
	defer client.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		defer wg.Done()

		ch, ok := <-server.incomingChannels
		if !ok {
			t.Error("cannot accept channel")
			return
		}
		if ch.ChannelType() != "ch" || string(ch.ExtraData()) != "extra" {
			t.Errorf("unexpected channel: %q, %q", ch.ChannelType(), ch.ExtraData())
			ch.Reject(RejectionReason(UnknownChannelType), UnknownChannelType.String())
			return
		}
		ch.Reject(RejectionReason(42), "message")
	}()

	ch, err := client.openChannel("ch", []byte("extra"))
	if ch != nil {
		t.Fatal("openChannel not rejected")
	}

	ocf, ok := err.(*OpenChannelError)
	if !ok {
		t.Errorf("got %#v want *OpenChannelError", err)
	} else if ocf.Reason != 42 || ocf.Message != "message" {
		t.Errorf("got %#v, want {Reason: 42, Message: %q}", ocf, "message")
	}

	want := "ssh: rejected: unknown reason 42 (message)"
	if err.Error() != want {
		t.Errorf("got %q, want %q", err.Error(), want)
	}
}

func TestMuxChannelRequest(t *testing.T) {
	client, server, mux := channelPair(t)
	defer server.Close()
	defer client.Close()
	defer mux.Close()

	var received int
	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		for r := range server.incomingRequests {
			received++
			r.Reply(r.Type == "yes", nil)
		}
		wg.Done()
	}()
	_, err := client.SendRequest("yes", false, nil)
	if err != nil {
		t.Fatalf("SendRequest: %v", err)
	}
	ok, err := client.SendRequest("yes", true, nil)
	if err != nil {
		t.Fatalf("SendRequest: %v", err)
	}

	if !ok {
		t.Errorf("SendRequest(yes): %v", ok)

	}

	ok, err = client.SendRequest("no", true, nil)
	if err != nil {
		t.Fatalf("SendRequest: %v", err)
	}
	if ok {
		t.Errorf("SendRequest(no): %v", ok)
	}

	client.Close()
	wg.Wait()

	if received != 3 {
		t.Errorf("got %d requests, want %d", received, 3)
	}
}

func TestMuxUnknownChannelRequests(t *testing.T) {
	clientPipe, serverPipe := memPipe()
	client := newMux(clientPipe)
	defer serverPipe.Close()
	defer client.Close()

	kDone := make(chan error, 1)
	go func() {
		// Ignore unknown channel messages that don't want a reply.
		err := serverPipe.writePacket(Marshal(channelRequestMsg{
			PeersID:             1,
			Request:             "keepalive@openssh.com",
			WantReply:           false,
			RequestSpecificData: []byte{},
		}))
		if err != nil {
			kDone <- fmt.Errorf("send: %w", err)
			return
		}

		// Send a keepalive, which should get a channel failure message
		// in response.
		err = serverPipe.writePacket(Marshal(channelRequestMsg{
			PeersID:             2,
			Request:             "keepalive@openssh.com",
			WantReply:           true,
			RequestSpecificData: []byte{},
		}))
		if err != nil {
			kDone <- fmt.Errorf("send: %w", err)
			return
		}

		packet, err := serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		decoded, err := decode(packet)
		if err != nil {
			kDone <- fmt.Errorf("decode failed: %w", err)
			return
		}

		switch msg := decoded.(type) {
		case *channelRequestFailureMsg:
			if msg.PeersID != 2 {
				kDone <- fmt.Errorf("received response to wrong message: %v", msg)
				return

			}
		default:
			kDone <- fmt.Errorf("unexpected channel message: %v", msg)
			return
		}

		kDone <- nil

		// Receive and respond to the keepalive to confirm the mux is
		// still processing requests.
		packet, err = serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		if packet[0] != msgGlobalRequest {
			kDone <- errors.New("expected global request")
			return
		}

		err = serverPipe.writePacket(Marshal(globalRequestFailureMsg{
			Data: []byte{},
		}))
		if err != nil {
			kDone <- fmt.Errorf("failed to send failure msg: %w", err)
			return
		}

		close(kDone)
	}()

	// Wait for the server to send the keepalive message and receive back a
	// response.
	if err := <-kDone; err != nil {
		t.Fatal(err)
	}

	// Confirm client hasn't closed.
	if _, _, err := client.SendRequest("keepalive@golang.org", true, nil); err != nil {
		t.Fatalf("failed to send keepalive: %v", err)
	}

	// Wait for the server to shut down.
	if err := <-kDone; err != nil {
		t.Fatal(err)
	}
}

func TestMuxClosedChannel(t *testing.T) {
	clientPipe, serverPipe := memPipe()
	client := newMux(clientPipe)
	defer serverPipe.Close()
	defer client.Close()

	kDone := make(chan error, 1)
	go func() {
		// Open the channel.
		packet, err := serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		if packet[0] != msgChannelOpen {
			kDone <- errors.New("expected chan open")
			return
		}

		var openMsg channelOpenMsg
		if err := Unmarshal(packet, &openMsg); err != nil {
			kDone <- fmt.Errorf("unmarshal: %w", err)
			return
		}

		// Send back the opened channel confirmation.
		err = serverPipe.writePacket(Marshal(channelOpenConfirmMsg{
			PeersID:       openMsg.PeersID,
			MyID:          0,
			MyWindow:      0,
			MaxPacketSize: channelMaxPacket,
		}))
		if err != nil {
			kDone <- fmt.Errorf("send: %w", err)
			return
		}

		// Close the channel.
		err = serverPipe.writePacket(Marshal(channelCloseMsg{
			PeersID: openMsg.PeersID,
		}))
		if err != nil {
			kDone <- fmt.Errorf("send: %w", err)
			return
		}

		// Send a keepalive message on the channel we just closed.
		err = serverPipe.writePacket(Marshal(channelRequestMsg{
			PeersID:             openMsg.PeersID,
			Request:             "keepalive@openssh.com",
			WantReply:           true,
			RequestSpecificData: []byte{},
		}))
		if err != nil {
			kDone <- fmt.Errorf("send: %w", err)
			return
		}

		// Receive the channel closed response.
		packet, err = serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		if packet[0] != msgChannelClose {
			kDone <- errors.New("expected channel close")
			return
		}

		// Receive the keepalive response failure.
		packet, err = serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		if packet[0] != msgChannelFailure {
			kDone <- errors.New("expected channel failure")
			return
		}
		kDone <- nil

		// Receive and respond to the keepalive to confirm the mux is
		// still processing requests.
		packet, err = serverPipe.readPacket()
		if err != nil {
			kDone <- fmt.Errorf("read packet: %w", err)
			return
		}
		if packet[0] != msgGlobalRequest {
			kDone <- errors.New("expected global request")
			return
		}

		err = serverPipe.writePacket(Marshal(globalRequestFailureMsg{
			Data: []byte{},
		}))
		if err != nil {
			kDone <- fmt.Errorf("failed to send failure msg: %w", err)
			return
		}

		close(kDone)
	}()

	// Open a channel.
	ch, err := client.openChannel("chan", nil)
	if err != nil {
		t.Fatalf("OpenChannel: %v", err)
	}
	defer ch.Close()

	// Wait for the server to close the channel and send the keepalive.
	<-kDone

	// Make sure the channel closed.
	if _, ok := <-ch.incomingRequests; ok {
		t.Fatalf("channel not closed")
	}

	// Confirm client hasn't closed
	if _, _, err := client.SendRequest("keepalive@golang.org", true, nil); err != nil {
		t.Fatalf("failed to send keepalive: %v", err)
	}

	// Wait for the server to shut down.
	<-kDone
}

func TestMuxGlobalRequest(t *testing.T) {
	var sawPeek bool
	var wg sync.WaitGroup
	defer func() {
		wg.Wait()
		if !sawPeek {
			t.Errorf("never saw 'peek' request")
		}
	}()

	clientMux, serverMux := muxPair()
	defer serverMux.Close()
	defer clientMux.Close()

	wg.Add(1)
	go func() {
		defer wg.Done()
		for r := range serverMux.incomingRequests {
			sawPeek = sawPeek || r.Type == "peek"
			if r.WantReply {
				err := r.Reply(r.Type == "yes",
					append([]byte(r.Type), r.Payload...))
				if err != nil {
					t.Errorf("AckRequest: %v", err)
				}
			}
		}
	}()

	_, _, err := clientMux.SendRequest("peek", false, nil)
	if err != nil {
		t.Errorf("SendRequest: %v", err)
	}

	ok, data, err := clientMux.SendRequest("yes", true, []byte("a"))
	if !ok || string(data) != "yesa" || err != nil {
		t.Errorf("SendRequest(\"yes\", true, \"a\"): %v %v %v",
			ok, data, err)
	}
	if ok, data, err := clientMux.SendRequest("yes", true, []byte("a")); !ok || string(data) != "yesa" || err != nil {
		t.Errorf("SendRequest(\"yes\", true, \"a\"): %v %v %v",
			ok, data, err)
	}

	if ok, data, err := clientMux.SendRequest("no", true, []byte("a")); ok || string(data) != "noa" || err != nil {
		t.Errorf("SendRequest(\"no\", true, \"a\"): %v %v %v",
			ok, data, err)
	}
}

func TestMuxGlobalRequestUnblock(t *testing.T) {
	clientMux, serverMux := muxPair()
	defer serverMux.Close()
	defer clientMux.Close()

	result := make(chan error, 1)
	go func() {
		_, _, err := clientMux.SendRequest("hello", true, nil)
		result <- err
	}()

	<-serverMux.incomingRequests
	serverMux.conn.Close()
	err := <-result

	if err != io.EOF {
		t.Errorf("want EOF, got %v", io.EOF)
	}
}

func TestMuxChannelRequestUnblock(t *testing.T) {
	a, b, connB := channelPair(t)
	defer a.Close()
	defer b.Close()
	defer connB.Close()

	result := make(chan error, 1)
	go func() {
		_, err := a.SendRequest("hello", true, nil)
		result <- err
	}()

	<-b.incomingRequests
	connB.conn.Close()
	err := <-result

	if err != io.EOF {
		t.Errorf("want EOF, got %v", err)
	}
}

func TestMuxCloseChannel(t *testing.T) {
	r, w, mux := channelPair(t)
	defer mux.Close()
	defer r.Close()
	defer w.Close()

	result := make(chan error, 1)
	go func() {
		var b [1024]byte
		_, err := r.Read(b[:])
		result <- err
	}()
	if err := w.Close(); err != nil {
		t.Errorf("w.Close: %v", err)
	}

	if _, err := w.Write([]byte("hello")); err != io.EOF {
		t.Errorf("got err %v, want io.EOF after Close", err)
	}

	if err := <-result; err != io.EOF {
		t.Errorf("got %v (%T), want io.EOF", err, err)
	}
}

func TestMuxCloseWriteChannel(t *testing.T) {
	r, w, mux := channelPair(t)
	defer mux.Close()

	result := make(chan error, 1)
	go func() {
		var b [1024]byte
		_, err := r.Read(b[:])
		result <- err
	}()
	if err := w.CloseWrite(); err != nil {
		t.Errorf("w.CloseWrite: %v", err)
	}

	if _, err := w.Write([]byte("hello")); err != io.EOF {
		t.Errorf("got err %v, want io.EOF after CloseWrite", err)
	}

	if err := <-result; err != io.EOF {
		t.Errorf("got %v (%T), want io.EOF", err, err)
	}
}

func TestMuxInvalidRecord(t *testing.T) {
	a, b := muxPair()
	defer a.Close()
	defer b.Close()

	packet := make([]byte, 1+4+4+1)
	packet[0] = msgChannelData
	marshalUint32(packet[1:], 29348723 /* invalid channel id */)
	marshalUint32(packet[5:], 1)
	packet[9] = 42

	a.conn.writePacket(packet)
	go a.SendRequest("hello", false, nil)
	// 'a' wrote an invalid packet, so 'b' has exited.
	req, ok := <-b.incomingRequests
	if ok {
		t.Errorf("got request %#v after receiving invalid packet", req)
	}
}

func TestZeroWindowAdjust(t *testing.T) {
	a, b, mux := channelPair(t)
	defer a.Close()
	defer b.Close()
	defer mux.Close()

	go func() {
		io.WriteString(a, "hello")
		// bogus adjust.
		a.sendMessage(windowAdjustMsg{})
		io.WriteString(a, "world")
		a.Close()
	}()

	want := "helloworld"
	c, _ := io.ReadAll(b)
	if string(c) != want {
		t.Errorf("got %q want %q", c, want)
	}
}

func TestMuxMaxPacketSize(t *testing.T) {
	a, b, mux := channelPair(t)
	defer a.Close()
	defer b.Close()
	defer mux.Close()

	large := make([]byte, a.maxRemotePayload+1)
	packet := make([]byte, 1+4+4+1+len(large))
	packet[0] = msgChannelData
	marshalUint32(packet[1:], a.remoteId)
	marshalUint32(packet[5:], uint32(len(large)))
	packet[9] = 42

	if err := a.mux.conn.writePacket(packet); err != nil {
		t.Errorf("could not send packet")
	}

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)
	wg.Add(1)
	go func() {
		a.SendRequest("hello", false, nil)
		wg.Done()
	}()

	_, ok := <-b.incomingRequests
	if ok {
		t.Errorf("connection still alive after receiving large packet.")
	}
}

func TestMuxChannelWindowDeferredUpdates(t *testing.T) {
	s, c, mux := channelPair(t)
	cTransport := mux.conn.(*memTransport)
	defer s.Close()
	defer c.Close()
	defer mux.Close()

	var wg sync.WaitGroup
	t.Cleanup(wg.Wait)

	data := make([]byte, 1024)

	wg.Add(1)
	go func() {
		defer wg.Done()
		_, err := s.Write(data)
		if err != nil {
			t.Errorf("Write: %v", err)
			return
		}
	}()
	cWritesInit := cTransport.getWriteCount()
	buf := make([]byte, 1)
	for i := 0; i < len(data); i++ {
		n, err := c.Read(buf)
		if n != len(buf) || err != nil {
			t.Fatalf("Read: %v, %v", n, err)
		}
	}
	cWrites := cTransport.getWriteCount() - cWritesInit
	// reading 1 KiB should not cause any window updates to be sent, but allow
	// for some unexpected writes
	if cWrites > 30 {
		t.Fatalf("reading 1 KiB from channel caused %v writes", cWrites)
	}
}

// Don't ship code with debug=true.
func TestDebug(t *testing.T) {
	if debugMux {
		t.Error("mux debug switched on")
	}
	if debugHandshake {
		t.Error("handshake debug switched on")
	}
	if debugTransport {
		t.Error("transport debug switched on")
	}
}
