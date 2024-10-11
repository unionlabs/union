//go:build !race
// +build !race

/*
 *
 * Copyright 2017 gRPC authors.
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
 *
 */

package transport

import (
	"bufio"
	"bytes"
	"context"
	"encoding/base64"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"testing"
	"time"
)

const (
	envTestAddr  = "1.2.3.4:8080"
	envProxyAddr = "2.3.4.5:7687"
)

// overwriteAndRestore overwrite function httpProxyFromEnvironment and
// returns a function to restore the default values.
func overwrite(hpfe func(req *http.Request) (*url.URL, error)) func() {
	backHPFE := httpProxyFromEnvironment
	httpProxyFromEnvironment = hpfe
	return func() {
		httpProxyFromEnvironment = backHPFE
	}
}

type proxyServer struct {
	t   *testing.T
	lis net.Listener
	in  net.Conn
	out net.Conn

	requestCheck func(*http.Request) error
}

func (p *proxyServer) run(waitForServerHello bool) {
	in, err := p.lis.Accept()
	if err != nil {
		return
	}
	p.in = in

	req, err := http.ReadRequest(bufio.NewReader(in))
	if err != nil {
		p.t.Errorf("failed to read CONNECT req: %v", err)
		return
	}
	if err := p.requestCheck(req); err != nil {
		resp := http.Response{StatusCode: http.StatusMethodNotAllowed}
		resp.Write(p.in)
		p.in.Close()
		p.t.Errorf("get wrong CONNECT req: %+v, error: %v", req, err)
		return
	}

	out, err := net.Dial("tcp", req.URL.Host)
	if err != nil {
		p.t.Errorf("failed to dial to server: %v", err)
		return
	}
	out.SetDeadline(time.Now().Add(defaultTestTimeout))
	resp := http.Response{StatusCode: http.StatusOK, Proto: "HTTP/1.0"}
	var buf bytes.Buffer
	resp.Write(&buf)
	if waitForServerHello {
		// Batch the first message from the server with the http connect
		// response. This is done to test the cases in which the grpc client has
		// the response to the connect request and proxied packets from the
		// destination server when it reads the transport.
		b := make([]byte, 50)
		bytesRead, err := out.Read(b)
		if err != nil {
			p.t.Errorf("Got error while reading server hello: %v", err)
			in.Close()
			out.Close()
			return
		}
		buf.Write(b[0:bytesRead])
	}
	p.in.Write(buf.Bytes())
	p.out = out
	go io.Copy(p.in, p.out)
	go io.Copy(p.out, p.in)
}

func (p *proxyServer) stop() {
	p.lis.Close()
	if p.in != nil {
		p.in.Close()
	}
	if p.out != nil {
		p.out.Close()
	}
}

type testArgs struct {
	proxyURLModify func(*url.URL) *url.URL
	proxyReqCheck  func(*http.Request) error
	serverMessage  []byte
}

func testHTTPConnect(t *testing.T, args testArgs) {
	plis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to listen: %v", err)
	}
	p := &proxyServer{
		t:            t,
		lis:          plis,
		requestCheck: args.proxyReqCheck,
	}
	go p.run(len(args.serverMessage) > 0)
	defer p.stop()

	blis, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatalf("failed to listen: %v", err)
	}

	msg := []byte{4, 3, 5, 2}
	recvBuf := make([]byte, len(msg))
	done := make(chan error, 1)
	go func() {
		in, err := blis.Accept()
		if err != nil {
			done <- err
			return
		}
		defer in.Close()
		in.Write(args.serverMessage)
		in.Read(recvBuf)
		done <- nil
	}()

	// Overwrite the function in the test and restore them in defer.
	hpfe := func(*http.Request) (*url.URL, error) {
		return args.proxyURLModify(&url.URL{Host: plis.Addr().String()}), nil
	}
	defer overwrite(hpfe)()

	// Dial to proxy server.
	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	c, err := proxyDial(ctx, blis.Addr().String(), "test")
	if err != nil {
		t.Fatalf("HTTP connect Dial failed: %v", err)
	}
	defer c.Close()
	c.SetDeadline(time.Now().Add(defaultTestTimeout))

	// Send msg on the connection.
	c.Write(msg)
	if err := <-done; err != nil {
		t.Fatalf("Failed to accept: %v", err)
	}

	// Check received msg.
	if string(recvBuf) != string(msg) {
		t.Fatalf("Received msg: %v, want %v", recvBuf, msg)
	}

	if len(args.serverMessage) > 0 {
		gotServerMessage := make([]byte, len(args.serverMessage))
		if _, err := c.Read(gotServerMessage); err != nil {
			t.Errorf("Got error while reading message from server: %v", err)
			return
		}
		if string(gotServerMessage) != string(args.serverMessage) {
			t.Errorf("Message from server: %v, want %v", gotServerMessage, args.serverMessage)
		}
	}
}

func (s) TestHTTPConnect(t *testing.T) {
	args := testArgs{
		proxyURLModify: func(in *url.URL) *url.URL {
			return in
		},
		proxyReqCheck: func(req *http.Request) error {
			if req.Method != http.MethodConnect {
				return fmt.Errorf("unexpected Method %q, want %q", req.Method, http.MethodConnect)
			}
			return nil
		},
	}
	testHTTPConnect(t, args)
}

func (s) TestHTTPConnectWithServerHello(t *testing.T) {
	args := testArgs{
		proxyURLModify: func(in *url.URL) *url.URL {
			return in
		},
		proxyReqCheck: func(req *http.Request) error {
			if req.Method != http.MethodConnect {
				return fmt.Errorf("unexpected Method %q, want %q", req.Method, http.MethodConnect)
			}
			return nil
		},
		serverMessage: []byte("server-hello"),
	}
	testHTTPConnect(t, args)
}

func (s) TestHTTPConnectBasicAuth(t *testing.T) {
	const (
		user     = "notAUser"
		password = "notAPassword"
	)
	args := testArgs{
		proxyURLModify: func(in *url.URL) *url.URL {
			in.User = url.UserPassword(user, password)
			return in
		},
		proxyReqCheck: func(req *http.Request) error {
			if req.Method != http.MethodConnect {
				return fmt.Errorf("unexpected Method %q, want %q", req.Method, http.MethodConnect)
			}
			wantProxyAuthStr := "Basic " + base64.StdEncoding.EncodeToString([]byte(user+":"+password))
			if got := req.Header.Get(proxyAuthHeaderKey); got != wantProxyAuthStr {
				gotDecoded, _ := base64.StdEncoding.DecodeString(got)
				wantDecoded, _ := base64.StdEncoding.DecodeString(wantProxyAuthStr)
				return fmt.Errorf("unexpected auth %q (%q), want %q (%q)", got, gotDecoded, wantProxyAuthStr, wantDecoded)
			}
			return nil
		},
	}
	testHTTPConnect(t, args)
}

func (s) TestMapAddressEnv(t *testing.T) {
	// Overwrite the function in the test and restore them in defer.
	hpfe := func(req *http.Request) (*url.URL, error) {
		if req.URL.Host == envTestAddr {
			return &url.URL{
				Scheme: "https",
				Host:   envProxyAddr,
			}, nil
		}
		return nil, nil
	}
	defer overwrite(hpfe)()

	// envTestAddr should be handled by ProxyFromEnvironment.
	got, err := mapAddress(envTestAddr)
	if err != nil {
		t.Error(err)
	}
	if got.Host != envProxyAddr {
		t.Errorf("want %v, got %v", envProxyAddr, got)
	}
}
