/*
 * Copyright 2022 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package test

import (
	"bytes"
	"io"
	"net"
	"testing"

	"golang.org/x/net/http2"
)

var (
	clientPreface = []byte(http2.ClientPreface)
)

func newClientTester(t *testing.T, conn net.Conn) *clientTester {
	ct := &clientTester{
		t:    t,
		conn: conn,
	}
	ct.fr = http2.NewFramer(conn, conn)
	ct.greet()
	return ct
}

type clientTester struct {
	t    *testing.T
	conn net.Conn
	fr   *http2.Framer
}

// greet() performs the necessary steps for http2 connection establishment on
// the server side.
func (ct *clientTester) greet() {
	ct.wantClientPreface()
	ct.wantSettingsFrame()
	ct.writeSettingsFrame()
	ct.writeSettingsAck()

	for {
		f, err := ct.fr.ReadFrame()
		if err != nil {
			ct.t.Errorf("error reading frame from client side: %v", err)
		}
		switch f := f.(type) {
		case *http2.SettingsFrame:
			if f.IsAck() { // HTTP/2 handshake completed.
				return
			}
		default:
			ct.t.Errorf("during greet, unexpected frame type %T", f)
		}
	}
}

func (ct *clientTester) wantClientPreface() {
	preface := make([]byte, len(clientPreface))
	if _, err := io.ReadFull(ct.conn, preface); err != nil {
		ct.t.Errorf("Error at server-side while reading preface from client. Err: %v", err)
	}
	if !bytes.Equal(preface, clientPreface) {
		ct.t.Errorf("received bogus greeting from client %q", preface)
	}
}

func (ct *clientTester) wantSettingsFrame() {
	frame, err := ct.fr.ReadFrame()
	if err != nil {
		ct.t.Errorf("error reading initial settings frame from client: %v", err)
	}
	_, ok := frame.(*http2.SettingsFrame)
	if !ok {
		ct.t.Errorf("initial frame sent from client is not a settings frame, type %T", frame)
	}
}

func (ct *clientTester) writeSettingsFrame() {
	if err := ct.fr.WriteSettings(); err != nil {
		ct.t.Fatalf("Error writing initial SETTINGS frame from client to server: %v", err)
	}
}

func (ct *clientTester) writeSettingsAck() {
	if err := ct.fr.WriteSettingsAck(); err != nil {
		ct.t.Fatalf("Error writing ACK of client's SETTINGS: %v", err)
	}
}

func (ct *clientTester) writeGoAway(maxStreamID uint32, code http2.ErrCode, debugData []byte) {
	if err := ct.fr.WriteGoAway(maxStreamID, code, debugData); err != nil {
		ct.t.Fatalf("Error writing GOAWAY: %v", err)
	}
}
