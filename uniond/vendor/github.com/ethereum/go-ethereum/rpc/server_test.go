// Copyright 2015 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package rpc

import (
	"bufio"
	"bytes"
	"io"
	"net"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func TestServerRegisterName(t *testing.T) {
	server := NewServer()
	service := new(testService)

	svcName := "test"
	if err := server.RegisterName(svcName, service); err != nil {
		t.Fatalf("%v", err)
	}

	if len(server.services.services) != 2 {
		t.Fatalf("Expected 2 service entries, got %d", len(server.services.services))
	}

	svc, ok := server.services.services[svcName]
	if !ok {
		t.Fatalf("Expected service %s to be registered", svcName)
	}

	wantCallbacks := 14
	if len(svc.callbacks) != wantCallbacks {
		t.Errorf("Expected %d callbacks for service 'service', got %d", wantCallbacks, len(svc.callbacks))
	}
}

func TestServer(t *testing.T) {
	files, err := os.ReadDir("testdata")
	if err != nil {
		t.Fatal("where'd my testdata go?")
	}
	for _, f := range files {
		if f.IsDir() || strings.HasPrefix(f.Name(), ".") {
			continue
		}
		path := filepath.Join("testdata", f.Name())
		name := strings.TrimSuffix(f.Name(), filepath.Ext(f.Name()))
		t.Run(name, func(t *testing.T) {
			runTestScript(t, path)
		})
	}
}

func runTestScript(t *testing.T, file string) {
	server := newTestServer()
	server.SetBatchLimits(4, 100000)
	content, err := os.ReadFile(file)
	if err != nil {
		t.Fatal(err)
	}

	clientConn, serverConn := net.Pipe()
	defer clientConn.Close()
	go server.ServeCodec(NewCodec(serverConn), 0)
	readbuf := bufio.NewReader(clientConn)
	for _, line := range strings.Split(string(content), "\n") {
		line = strings.TrimSpace(line)
		switch {
		case len(line) == 0 || strings.HasPrefix(line, "//"):
			// skip comments, blank lines
			continue
		case strings.HasPrefix(line, "--> "):
			t.Log(line)
			// write to connection
			clientConn.SetWriteDeadline(time.Now().Add(5 * time.Second))
			if _, err := io.WriteString(clientConn, line[4:]+"\n"); err != nil {
				t.Fatalf("write error: %v", err)
			}
		case strings.HasPrefix(line, "<-- "):
			t.Log(line)
			want := line[4:]
			// read line from connection and compare text
			clientConn.SetReadDeadline(time.Now().Add(5 * time.Second))
			sent, err := readbuf.ReadString('\n')
			if err != nil {
				t.Fatalf("read error: %v", err)
			}
			sent = strings.TrimRight(sent, "\r\n")
			if sent != want {
				t.Errorf("wrong line from server\ngot:  %s\nwant: %s", sent, want)
			}
		default:
			panic("invalid line in test script: " + line)
		}
	}
}

// This test checks that responses are delivered for very short-lived connections that
// only carry a single request.
func TestServerShortLivedConn(t *testing.T) {
	server := newTestServer()
	defer server.Stop()

	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatal("can't listen:", err)
	}
	defer listener.Close()
	go server.ServeListener(listener)

	var (
		request  = `{"jsonrpc":"2.0","id":1,"method":"rpc_modules"}` + "\n"
		wantResp = `{"jsonrpc":"2.0","id":1,"result":{"nftest":"1.0","rpc":"1.0","test":"1.0"}}` + "\n"
		deadline = time.Now().Add(10 * time.Second)
	)
	for i := 0; i < 20; i++ {
		conn, err := net.Dial("tcp", listener.Addr().String())
		if err != nil {
			t.Fatal("can't dial:", err)
		}

		conn.SetDeadline(deadline)
		// Write the request, then half-close the connection so the server stops reading.
		conn.Write([]byte(request))
		conn.(*net.TCPConn).CloseWrite()
		// Now try to get the response.
		buf := make([]byte, 2000)
		n, err := conn.Read(buf)
		conn.Close()

		if err != nil {
			t.Fatal("read error:", err)
		}
		if !bytes.Equal(buf[:n], []byte(wantResp)) {
			t.Fatalf("wrong response: %s", buf[:n])
		}
	}
}

func TestServerBatchResponseSizeLimit(t *testing.T) {
	server := newTestServer()
	defer server.Stop()
	server.SetBatchLimits(100, 60)
	var (
		batch  []BatchElem
		client = DialInProc(server)
	)
	for i := 0; i < 5; i++ {
		batch = append(batch, BatchElem{
			Method: "test_echo",
			Args:   []any{"x", 1},
			Result: new(echoResult),
		})
	}
	if err := client.BatchCall(batch); err != nil {
		t.Fatal("error sending batch:", err)
	}
	for i := range batch {
		// We expect the first two queries to be ok, but after that the size limit takes effect.
		if i < 2 {
			if batch[i].Error != nil {
				t.Fatalf("batch elem %d has unexpected error: %v", i, batch[i].Error)
			}
			continue
		}
		// After two, we expect an error.
		re, ok := batch[i].Error.(Error)
		if !ok {
			t.Fatalf("batch elem %d has wrong error: %v", i, batch[i].Error)
		}
		wantedCode := errcodeResponseTooLarge
		if re.ErrorCode() != wantedCode {
			t.Errorf("batch elem %d wrong error code, have %d want %d", i, re.ErrorCode(), wantedCode)
		}
	}
}
