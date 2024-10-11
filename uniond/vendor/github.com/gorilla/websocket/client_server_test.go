// Copyright 2013 The Gorilla WebSocket Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package websocket

import (
	"bytes"
	"context"
	"crypto/tls"
	"crypto/x509"
	"encoding/base64"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"net/http/cookiejar"
	"net/http/httptest"
	"net/http/httptrace"
	"net/url"
	"reflect"
	"strings"
	"testing"
	"time"
)

var cstUpgrader = Upgrader{
	Subprotocols:      []string{"p0", "p1"},
	ReadBufferSize:    1024,
	WriteBufferSize:   1024,
	EnableCompression: true,
	Error: func(w http.ResponseWriter, r *http.Request, status int, reason error) {
		http.Error(w, reason.Error(), status)
	},
}

var cstDialer = Dialer{
	Subprotocols:     []string{"p1", "p2"},
	ReadBufferSize:   1024,
	WriteBufferSize:  1024,
	HandshakeTimeout: 30 * time.Second,
}

type cstHandler struct{ *testing.T }

type cstServer struct {
	*httptest.Server
	URL string
	t   *testing.T
}

const (
	cstPath       = "/a/b"
	cstRawQuery   = "x=y"
	cstRequestURI = cstPath + "?" + cstRawQuery
)

func newServer(t *testing.T) *cstServer {
	var s cstServer
	s.Server = httptest.NewServer(cstHandler{t})
	s.Server.URL += cstRequestURI
	s.URL = makeWsProto(s.Server.URL)
	return &s
}

func newTLSServer(t *testing.T) *cstServer {
	var s cstServer
	s.Server = httptest.NewTLSServer(cstHandler{t})
	s.Server.URL += cstRequestURI
	s.URL = makeWsProto(s.Server.URL)
	return &s
}

func (t cstHandler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != cstPath {
		t.Logf("path=%v, want %v", r.URL.Path, cstPath)
		http.Error(w, "bad path", http.StatusBadRequest)
		return
	}
	if r.URL.RawQuery != cstRawQuery {
		t.Logf("query=%v, want %v", r.URL.RawQuery, cstRawQuery)
		http.Error(w, "bad path", http.StatusBadRequest)
		return
	}
	subprotos := Subprotocols(r)
	if !reflect.DeepEqual(subprotos, cstDialer.Subprotocols) {
		t.Logf("subprotols=%v, want %v", subprotos, cstDialer.Subprotocols)
		http.Error(w, "bad protocol", http.StatusBadRequest)
		return
	}
	ws, err := cstUpgrader.Upgrade(w, r, http.Header{"Set-Cookie": {"sessionID=1234"}})
	if err != nil {
		t.Logf("Upgrade: %v", err)
		return
	}
	defer ws.Close()

	if ws.Subprotocol() != "p1" {
		t.Logf("Subprotocol() = %s, want p1", ws.Subprotocol())
		ws.Close()
		return
	}
	op, rd, err := ws.NextReader()
	if err != nil {
		t.Logf("NextReader: %v", err)
		return
	}
	wr, err := ws.NextWriter(op)
	if err != nil {
		t.Logf("NextWriter: %v", err)
		return
	}
	if _, err = io.Copy(wr, rd); err != nil {
		t.Logf("NextWriter: %v", err)
		return
	}
	if err := wr.Close(); err != nil {
		t.Logf("Close: %v", err)
		return
	}
}

func makeWsProto(s string) string {
	return "ws" + strings.TrimPrefix(s, "http")
}

func sendRecv(t *testing.T, ws *Conn) {
	const message = "Hello World!"
	if err := ws.SetWriteDeadline(time.Now().Add(time.Second)); err != nil {
		t.Fatalf("SetWriteDeadline: %v", err)
	}
	if err := ws.WriteMessage(TextMessage, []byte(message)); err != nil {
		t.Fatalf("WriteMessage: %v", err)
	}
	if err := ws.SetReadDeadline(time.Now().Add(time.Second)); err != nil {
		t.Fatalf("SetReadDeadline: %v", err)
	}
	_, p, err := ws.ReadMessage()
	if err != nil {
		t.Fatalf("ReadMessage: %v", err)
	}
	if string(p) != message {
		t.Fatalf("message=%s, want %s", p, message)
	}
}

func TestProxyDial(t *testing.T) {

	s := newServer(t)
	defer s.Close()

	surl, _ := url.Parse(s.Server.URL)

	cstDialer := cstDialer // make local copy for modification on next line.
	cstDialer.Proxy = http.ProxyURL(surl)

	connect := false
	origHandler := s.Server.Config.Handler

	// Capture the request Host header.
	s.Server.Config.Handler = http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			if r.Method == http.MethodConnect {
				connect = true
				w.WriteHeader(http.StatusOK)
				return
			}

			if !connect {
				t.Log("connect not received")
				http.Error(w, "connect not received", http.StatusMethodNotAllowed)
				return
			}
			origHandler.ServeHTTP(w, r)
		})

	ws, _, err := cstDialer.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestProxyAuthorizationDial(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	surl, _ := url.Parse(s.Server.URL)
	surl.User = url.UserPassword("username", "password")

	cstDialer := cstDialer // make local copy for modification on next line.
	cstDialer.Proxy = http.ProxyURL(surl)

	connect := false
	origHandler := s.Server.Config.Handler

	// Capture the request Host header.
	s.Server.Config.Handler = http.HandlerFunc(
		func(w http.ResponseWriter, r *http.Request) {
			proxyAuth := r.Header.Get("Proxy-Authorization")
			expectedProxyAuth := "Basic " + base64.StdEncoding.EncodeToString([]byte("username:password"))
			if r.Method == http.MethodConnect && proxyAuth == expectedProxyAuth {
				connect = true
				w.WriteHeader(http.StatusOK)
				return
			}

			if !connect {
				t.Log("connect with proxy authorization not received")
				http.Error(w, "connect with proxy authorization not received", http.StatusMethodNotAllowed)
				return
			}
			origHandler.ServeHTTP(w, r)
		})

	ws, _, err := cstDialer.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestDial(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	ws, _, err := cstDialer.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestDialCookieJar(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	jar, _ := cookiejar.New(nil)
	d := cstDialer
	d.Jar = jar

	u, _ := url.Parse(s.URL)

	switch u.Scheme {
	case "ws":
		u.Scheme = "http"
	case "wss":
		u.Scheme = "https"
	}

	cookies := []*http.Cookie{{Name: "gorilla", Value: "ws", Path: "/"}}
	d.Jar.SetCookies(u, cookies)

	ws, _, err := d.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()

	var gorilla string
	var sessionID string
	for _, c := range d.Jar.Cookies(u) {
		if c.Name == "gorilla" {
			gorilla = c.Value
		}

		if c.Name == "sessionID" {
			sessionID = c.Value
		}
	}
	if gorilla != "ws" {
		t.Error("Cookie not present in jar.")
	}

	if sessionID != "1234" {
		t.Error("Set-Cookie not received from the server.")
	}

	sendRecv(t, ws)
}

func rootCAs(t *testing.T, s *httptest.Server) *x509.CertPool {
	certs := x509.NewCertPool()
	for _, c := range s.TLS.Certificates {
		roots, err := x509.ParseCertificates(c.Certificate[len(c.Certificate)-1])
		if err != nil {
			t.Fatalf("error parsing server's root cert: %v", err)
		}
		for _, root := range roots {
			certs.AddCert(root)
		}
	}
	return certs
}

func TestDialTLS(t *testing.T) {
	s := newTLSServer(t)
	defer s.Close()

	d := cstDialer
	d.TLSClientConfig = &tls.Config{RootCAs: rootCAs(t, s.Server)}
	ws, _, err := d.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestDialTimeout(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	d := cstDialer
	d.HandshakeTimeout = -1
	ws, _, err := d.Dial(s.URL, nil)
	if err == nil {
		ws.Close()
		t.Fatalf("Dial: nil")
	}
}

// requireDeadlineNetConn fails the current test when Read or Write are called
// with no deadline.
type requireDeadlineNetConn struct {
	t                  *testing.T
	c                  net.Conn
	readDeadlineIsSet  bool
	writeDeadlineIsSet bool
}

func (c *requireDeadlineNetConn) SetDeadline(t time.Time) error {
	c.writeDeadlineIsSet = !t.Equal(time.Time{})
	c.readDeadlineIsSet = c.writeDeadlineIsSet
	return c.c.SetDeadline(t)
}

func (c *requireDeadlineNetConn) SetReadDeadline(t time.Time) error {
	c.readDeadlineIsSet = !t.Equal(time.Time{})
	return c.c.SetDeadline(t)
}

func (c *requireDeadlineNetConn) SetWriteDeadline(t time.Time) error {
	c.writeDeadlineIsSet = !t.Equal(time.Time{})
	return c.c.SetDeadline(t)
}

func (c *requireDeadlineNetConn) Write(p []byte) (int, error) {
	if !c.writeDeadlineIsSet {
		c.t.Fatalf("write with no deadline")
	}
	return c.c.Write(p)
}

func (c *requireDeadlineNetConn) Read(p []byte) (int, error) {
	if !c.readDeadlineIsSet {
		c.t.Fatalf("read with no deadline")
	}
	return c.c.Read(p)
}

func (c *requireDeadlineNetConn) Close() error         { return c.c.Close() }
func (c *requireDeadlineNetConn) LocalAddr() net.Addr  { return c.c.LocalAddr() }
func (c *requireDeadlineNetConn) RemoteAddr() net.Addr { return c.c.RemoteAddr() }

func TestHandshakeTimeout(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	d := cstDialer
	d.NetDial = func(n, a string) (net.Conn, error) {
		c, err := net.Dial(n, a)
		return &requireDeadlineNetConn{c: c, t: t}, err
	}
	ws, _, err := d.Dial(s.URL, nil)
	if err != nil {
		t.Fatal("Dial:", err)
	}
	ws.Close()
}

func TestHandshakeTimeoutInContext(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	d := cstDialer
	d.HandshakeTimeout = 0
	d.NetDialContext = func(ctx context.Context, n, a string) (net.Conn, error) {
		netDialer := &net.Dialer{}
		c, err := netDialer.DialContext(ctx, n, a)
		return &requireDeadlineNetConn{c: c, t: t}, err
	}

	ctx, cancel := context.WithDeadline(context.Background(), time.Now().Add(30*time.Second))
	defer cancel()
	ws, _, err := d.DialContext(ctx, s.URL, nil)
	if err != nil {
		t.Fatal("Dial:", err)
	}
	ws.Close()
}

func TestDialBadScheme(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	ws, _, err := cstDialer.Dial(s.Server.URL, nil)
	if err == nil {
		ws.Close()
		t.Fatalf("Dial: nil")
	}
}

func TestDialBadOrigin(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	ws, resp, err := cstDialer.Dial(s.URL, http.Header{"Origin": {"bad"}})
	if err == nil {
		ws.Close()
		t.Fatalf("Dial: nil")
	}
	if resp == nil {
		t.Fatalf("resp=nil, err=%v", err)
	}
	if resp.StatusCode != http.StatusForbidden {
		t.Fatalf("status=%d, want %d", resp.StatusCode, http.StatusForbidden)
	}
}

func TestDialBadHeader(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	for _, k := range []string{"Upgrade",
		"Connection",
		"Sec-Websocket-Key",
		"Sec-Websocket-Version",
		"Sec-Websocket-Protocol"} {
		h := http.Header{}
		h.Set(k, "bad")
		ws, _, err := cstDialer.Dial(s.URL, http.Header{"Origin": {"bad"}})
		if err == nil {
			ws.Close()
			t.Errorf("Dial with header %s returned nil", k)
		}
	}
}

func TestBadMethod(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		ws, err := cstUpgrader.Upgrade(w, r, nil)
		if err == nil {
			t.Errorf("handshake succeeded, expect fail")
			ws.Close()
		}
	}))
	defer s.Close()

	req, err := http.NewRequest(http.MethodPost, s.URL, strings.NewReader(""))
	if err != nil {
		t.Fatalf("NewRequest returned error %v", err)
	}
	req.Header.Set("Connection", "upgrade")
	req.Header.Set("Upgrade", "websocket")
	req.Header.Set("Sec-Websocket-Version", "13")

	resp, err := http.DefaultClient.Do(req)
	if err != nil {
		t.Fatalf("Do returned error %v", err)
	}
	resp.Body.Close()
	if resp.StatusCode != http.StatusMethodNotAllowed {
		t.Errorf("Status = %d, want %d", resp.StatusCode, http.StatusMethodNotAllowed)
	}
}

func TestDialExtraTokensInRespHeaders(t *testing.T) {
	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		challengeKey := r.Header.Get("Sec-Websocket-Key")
		w.Header().Set("Upgrade", "foo, websocket")
		w.Header().Set("Connection", "upgrade, keep-alive")
		w.Header().Set("Sec-Websocket-Accept", computeAcceptKey(challengeKey))
		w.WriteHeader(101)
	}))
	defer s.Close()

	ws, _, err := cstDialer.Dial(makeWsProto(s.URL), nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
}

func TestHandshake(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	ws, resp, err := cstDialer.Dial(s.URL, http.Header{"Origin": {s.URL}})
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()

	var sessionID string
	for _, c := range resp.Cookies() {
		if c.Name == "sessionID" {
			sessionID = c.Value
		}
	}
	if sessionID != "1234" {
		t.Error("Set-Cookie not received from the server.")
	}

	if ws.Subprotocol() != "p1" {
		t.Errorf("ws.Subprotocol() = %s, want p1", ws.Subprotocol())
	}
	sendRecv(t, ws)
}

func TestRespOnBadHandshake(t *testing.T) {
	const expectedStatus = http.StatusGone
	const expectedBody = "This is the response body."

	s := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(expectedStatus)
		io.WriteString(w, expectedBody)
	}))
	defer s.Close()

	ws, resp, err := cstDialer.Dial(makeWsProto(s.URL), nil)
	if err == nil {
		ws.Close()
		t.Fatalf("Dial: nil")
	}

	if resp == nil {
		t.Fatalf("resp=nil, err=%v", err)
	}

	if resp.StatusCode != expectedStatus {
		t.Errorf("resp.StatusCode=%d, want %d", resp.StatusCode, expectedStatus)
	}

	p, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		t.Fatalf("ReadFull(resp.Body) returned error %v", err)
	}

	if string(p) != expectedBody {
		t.Errorf("resp.Body=%s, want %s", p, expectedBody)
	}
}

type testLogWriter struct {
	t *testing.T
}

func (w testLogWriter) Write(p []byte) (int, error) {
	w.t.Logf("%s", p)
	return len(p), nil
}

// TestHost tests handling of host names and confirms that it matches net/http.
func TestHost(t *testing.T) {

	upgrader := Upgrader{}
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if IsWebSocketUpgrade(r) {
			c, err := upgrader.Upgrade(w, r, http.Header{"X-Test-Host": {r.Host}})
			if err != nil {
				t.Fatal(err)
			}
			c.Close()
		} else {
			w.Header().Set("X-Test-Host", r.Host)
		}
	})

	server := httptest.NewServer(handler)
	defer server.Close()

	tlsServer := httptest.NewTLSServer(handler)
	defer tlsServer.Close()

	addrs := map[*httptest.Server]string{server: server.Listener.Addr().String(), tlsServer: tlsServer.Listener.Addr().String()}
	wsProtos := map[*httptest.Server]string{server: "ws://", tlsServer: "wss://"}
	httpProtos := map[*httptest.Server]string{server: "http://", tlsServer: "https://"}

	// Avoid log noise from net/http server by logging to testing.T
	server.Config.ErrorLog = log.New(testLogWriter{t}, "", 0)
	tlsServer.Config.ErrorLog = server.Config.ErrorLog

	cas := rootCAs(t, tlsServer)

	tests := []struct {
		fail               bool             // true if dial / get should fail
		server             *httptest.Server // server to use
		url                string           // host for request URI
		header             string           // optional request host header
		tls                string           // optional host for tls ServerName
		wantAddr           string           // expected host for dial
		wantHeader         string           // expected request header on server
		insecureSkipVerify bool
	}{
		{
			server:     server,
			url:        addrs[server],
			wantAddr:   addrs[server],
			wantHeader: addrs[server],
		},
		{
			server:     tlsServer,
			url:        addrs[tlsServer],
			wantAddr:   addrs[tlsServer],
			wantHeader: addrs[tlsServer],
		},

		{
			server:     server,
			url:        addrs[server],
			header:     "badhost.com",
			wantAddr:   addrs[server],
			wantHeader: "badhost.com",
		},
		{
			server:     tlsServer,
			url:        addrs[tlsServer],
			header:     "badhost.com",
			wantAddr:   addrs[tlsServer],
			wantHeader: "badhost.com",
		},

		{
			server:     server,
			url:        "example.com",
			header:     "badhost.com",
			wantAddr:   "example.com:80",
			wantHeader: "badhost.com",
		},
		{
			server:     tlsServer,
			url:        "example.com",
			header:     "badhost.com",
			wantAddr:   "example.com:443",
			wantHeader: "badhost.com",
		},

		{
			server:     server,
			url:        "badhost.com",
			header:     "example.com",
			wantAddr:   "badhost.com:80",
			wantHeader: "example.com",
		},
		{
			fail:     true,
			server:   tlsServer,
			url:      "badhost.com",
			header:   "example.com",
			wantAddr: "badhost.com:443",
		},
		{
			server:             tlsServer,
			url:                "badhost.com",
			insecureSkipVerify: true,
			wantAddr:           "badhost.com:443",
			wantHeader:         "badhost.com",
		},
		{
			server:     tlsServer,
			url:        "badhost.com",
			tls:        "example.com",
			wantAddr:   "badhost.com:443",
			wantHeader: "badhost.com",
		},
	}

	for i, tt := range tests {

		tls := &tls.Config{
			RootCAs:            cas,
			ServerName:         tt.tls,
			InsecureSkipVerify: tt.insecureSkipVerify,
		}

		var gotAddr string
		dialer := Dialer{
			NetDial: func(network, addr string) (net.Conn, error) {
				gotAddr = addr
				return net.Dial(network, addrs[tt.server])
			},
			TLSClientConfig: tls,
		}

		// Test websocket dial

		h := http.Header{}
		if tt.header != "" {
			h.Set("Host", tt.header)
		}
		c, resp, err := dialer.Dial(wsProtos[tt.server]+tt.url+"/", h)
		if err == nil {
			c.Close()
		}

		check := func(protos map[*httptest.Server]string) {
			name := fmt.Sprintf("%d: %s%s/ header[Host]=%q, tls.ServerName=%q", i+1, protos[tt.server], tt.url, tt.header, tt.tls)
			if gotAddr != tt.wantAddr {
				t.Errorf("%s: got addr %s, want %s", name, gotAddr, tt.wantAddr)
			}
			switch {
			case tt.fail && err == nil:
				t.Errorf("%s: unexpected success", name)
			case !tt.fail && err != nil:
				t.Errorf("%s: unexpected error %v", name, err)
			case !tt.fail && err == nil:
				if gotHost := resp.Header.Get("X-Test-Host"); gotHost != tt.wantHeader {
					t.Errorf("%s: got host %s, want %s", name, gotHost, tt.wantHeader)
				}
			}
		}

		check(wsProtos)

		// Confirm that net/http has same result

		transport := &http.Transport{
			Dial:            dialer.NetDial,
			TLSClientConfig: dialer.TLSClientConfig,
		}
		req, _ := http.NewRequest(http.MethodGet, httpProtos[tt.server]+tt.url+"/", nil)
		if tt.header != "" {
			req.Host = tt.header
		}
		client := &http.Client{Transport: transport}
		resp, err = client.Do(req)
		if err == nil {
			resp.Body.Close()
		}
		transport.CloseIdleConnections()
		check(httpProtos)
	}
}

func TestDialCompression(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	dialer := cstDialer
	dialer.EnableCompression = true
	ws, _, err := dialer.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestSocksProxyDial(t *testing.T) {
	s := newServer(t)
	defer s.Close()

	proxyListener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Fatalf("listen failed: %v", err)
	}
	defer proxyListener.Close()
	go func() {
		c1, err := proxyListener.Accept()
		if err != nil {
			t.Errorf("proxy accept failed: %v", err)
			return
		}
		defer c1.Close()

		c1.SetDeadline(time.Now().Add(30 * time.Second))

		buf := make([]byte, 32)
		if _, err := io.ReadFull(c1, buf[:3]); err != nil {
			t.Errorf("read failed: %v", err)
			return
		}
		if want := []byte{5, 1, 0}; !bytes.Equal(want, buf[:len(want)]) {
			t.Errorf("read %x, want %x", buf[:len(want)], want)
		}
		if _, err := c1.Write([]byte{5, 0}); err != nil {
			t.Errorf("write failed: %v", err)
			return
		}
		if _, err := io.ReadFull(c1, buf[:10]); err != nil {
			t.Errorf("read failed: %v", err)
			return
		}
		if want := []byte{5, 1, 0, 1}; !bytes.Equal(want, buf[:len(want)]) {
			t.Errorf("read %x, want %x", buf[:len(want)], want)
			return
		}
		buf[1] = 0
		if _, err := c1.Write(buf[:10]); err != nil {
			t.Errorf("write failed: %v", err)
			return
		}

		ip := net.IP(buf[4:8])
		port := binary.BigEndian.Uint16(buf[8:10])

		c2, err := net.DialTCP("tcp", nil, &net.TCPAddr{IP: ip, Port: int(port)})
		if err != nil {
			t.Errorf("dial failed; %v", err)
			return
		}
		defer c2.Close()
		done := make(chan struct{})
		go func() {
			io.Copy(c1, c2)
			close(done)
		}()
		io.Copy(c2, c1)
		<-done
	}()

	purl, err := url.Parse("socks5://" + proxyListener.Addr().String())
	if err != nil {
		t.Fatalf("parse failed: %v", err)
	}

	cstDialer := cstDialer // make local copy for modification on next line.
	cstDialer.Proxy = http.ProxyURL(purl)

	ws, _, err := cstDialer.Dial(s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}
	defer ws.Close()
	sendRecv(t, ws)
}

func TestTracingDialWithContext(t *testing.T) {

	var headersWrote, requestWrote, getConn, gotConn, connectDone, gotFirstResponseByte bool
	trace := &httptrace.ClientTrace{
		WroteHeaders: func() {
			headersWrote = true
		},
		WroteRequest: func(httptrace.WroteRequestInfo) {
			requestWrote = true
		},
		GetConn: func(hostPort string) {
			getConn = true
		},
		GotConn: func(info httptrace.GotConnInfo) {
			gotConn = true
		},
		ConnectDone: func(network, addr string, err error) {
			connectDone = true
		},
		GotFirstResponseByte: func() {
			gotFirstResponseByte = true
		},
	}
	ctx := httptrace.WithClientTrace(context.Background(), trace)

	s := newTLSServer(t)
	defer s.Close()

	d := cstDialer
	d.TLSClientConfig = &tls.Config{RootCAs: rootCAs(t, s.Server)}

	ws, _, err := d.DialContext(ctx, s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}

	if !headersWrote {
		t.Fatal("Headers was not written")
	}
	if !requestWrote {
		t.Fatal("Request was not written")
	}
	if !getConn {
		t.Fatal("getConn was not called")
	}
	if !gotConn {
		t.Fatal("gotConn was not called")
	}
	if !connectDone {
		t.Fatal("connectDone was not called")
	}
	if !gotFirstResponseByte {
		t.Fatal("GotFirstResponseByte was not called")
	}

	defer ws.Close()
	sendRecv(t, ws)
}

func TestEmptyTracingDialWithContext(t *testing.T) {

	trace := &httptrace.ClientTrace{}
	ctx := httptrace.WithClientTrace(context.Background(), trace)

	s := newTLSServer(t)
	defer s.Close()

	d := cstDialer
	d.TLSClientConfig = &tls.Config{RootCAs: rootCAs(t, s.Server)}

	ws, _, err := d.DialContext(ctx, s.URL, nil)
	if err != nil {
		t.Fatalf("Dial: %v", err)
	}

	defer ws.Close()
	sendRecv(t, ws)
}

// TestNetDialConnect tests selection of dial method between NetDial, NetDialContext, NetDialTLS or NetDialTLSContext
func TestNetDialConnect(t *testing.T) {

	upgrader := Upgrader{}
	handler := http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if IsWebSocketUpgrade(r) {
			c, err := upgrader.Upgrade(w, r, http.Header{"X-Test-Host": {r.Host}})
			if err != nil {
				t.Fatal(err)
			}
			c.Close()
		} else {
			w.Header().Set("X-Test-Host", r.Host)
		}
	})

	server := httptest.NewServer(handler)
	defer server.Close()

	tlsServer := httptest.NewTLSServer(handler)
	defer tlsServer.Close()

	testUrls := map[*httptest.Server]string{
		server:    "ws://" + server.Listener.Addr().String() + "/",
		tlsServer: "wss://" + tlsServer.Listener.Addr().String() + "/",
	}

	cas := rootCAs(t, tlsServer)
	tlsConfig := &tls.Config{
		RootCAs:            cas,
		ServerName:         "example.com",
		InsecureSkipVerify: false,
	}

	tests := []struct {
		name              string
		server            *httptest.Server // server to use
		netDial           func(network, addr string) (net.Conn, error)
		netDialContext    func(ctx context.Context, network, addr string) (net.Conn, error)
		netDialTLSContext func(ctx context.Context, network, addr string) (net.Conn, error)
		tlsClientConfig   *tls.Config
	}{

		{
			name:   "HTTP server, all NetDial* defined, shall use NetDialContext",
			server: server,
			netDial: func(network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDial should not be called")
			},
			netDialContext: func(_ context.Context, network, addr string) (net.Conn, error) {
				return net.Dial(network, addr)
			},
			netDialTLSContext: func(_ context.Context, network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDialTLSContext should not be called")
			},
			tlsClientConfig: nil,
		},
		{
			name:              "HTTP server, all NetDial* undefined",
			server:            server,
			netDial:           nil,
			netDialContext:    nil,
			netDialTLSContext: nil,
			tlsClientConfig:   nil,
		},
		{
			name:   "HTTP server, NetDialContext undefined, shall fallback to NetDial",
			server: server,
			netDial: func(network, addr string) (net.Conn, error) {
				return net.Dial(network, addr)
			},
			netDialContext: nil,
			netDialTLSContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDialTLSContext should not be called")
			},
			tlsClientConfig: nil,
		},
		{
			name:   "HTTPS server, all NetDial* defined, shall use NetDialTLSContext",
			server: tlsServer,
			netDial: func(network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDial should not be called")
			},
			netDialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDialContext should not be called")
			},
			netDialTLSContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				netConn, err := net.Dial(network, addr)
				if err != nil {
					return nil, err
				}
				tlsConn := tls.Client(netConn, tlsConfig)
				err = tlsConn.Handshake()
				if err != nil {
					return nil, err
				}
				return tlsConn, nil
			},
			tlsClientConfig: nil,
		},
		{
			name:   "HTTPS server, NetDialTLSContext undefined, shall fallback to NetDialContext and do handshake",
			server: tlsServer,
			netDial: func(network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDial should not be called")
			},
			netDialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				return net.Dial(network, addr)
			},
			netDialTLSContext: nil,
			tlsClientConfig:   tlsConfig,
		},
		{
			name:   "HTTPS server, NetDialTLSContext and NetDialContext undefined, shall fallback to NetDial and do handshake",
			server: tlsServer,
			netDial: func(network, addr string) (net.Conn, error) {
				return net.Dial(network, addr)
			},
			netDialContext:    nil,
			netDialTLSContext: nil,
			tlsClientConfig:   tlsConfig,
		},
		{
			name:              "HTTPS server, all NetDial* undefined",
			server:            tlsServer,
			netDial:           nil,
			netDialContext:    nil,
			netDialTLSContext: nil,
			tlsClientConfig:   tlsConfig,
		},
		{
			name:   "HTTPS server, all NetDialTLSContext defined, dummy TlsClientConfig defined, shall not do handshake",
			server: tlsServer,
			netDial: func(network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDial should not be called")
			},
			netDialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				return nil, errors.New("NetDialContext should not be called")
			},
			netDialTLSContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				netConn, err := net.Dial(network, addr)
				if err != nil {
					return nil, err
				}
				tlsConn := tls.Client(netConn, tlsConfig)
				err = tlsConn.Handshake()
				if err != nil {
					return nil, err
				}
				return tlsConn, nil
			},
			tlsClientConfig: &tls.Config{
				RootCAs:            nil,
				ServerName:         "badserver.com",
				InsecureSkipVerify: false,
			},
		},
	}

	for _, tc := range tests {
		dialer := Dialer{
			NetDial:           tc.netDial,
			NetDialContext:    tc.netDialContext,
			NetDialTLSContext: tc.netDialTLSContext,
			TLSClientConfig:   tc.tlsClientConfig,
		}

		// Test websocket dial
		c, _, err := dialer.Dial(testUrls[tc.server], nil)
		if err != nil {
			t.Errorf("FAILED %s, err: %s", tc.name, err.Error())
		} else {
			c.Close()
		}
	}
}
func TestNextProtos(t *testing.T) {
	ts := httptest.NewUnstartedServer(
		http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {}),
	)
	ts.EnableHTTP2 = true
	ts.StartTLS()
	defer ts.Close()

	d := Dialer{
		TLSClientConfig: ts.Client().Transport.(*http.Transport).TLSClientConfig,
	}

	r, err := ts.Client().Get(ts.URL)
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	r.Body.Close()

	// Asserts that Dialer.TLSClientConfig.NextProtos contains "h2"
	// after the Client.Get call from net/http above.
	var containsHTTP2 bool = false
	for _, proto := range d.TLSClientConfig.NextProtos {
		if proto == "h2" {
			containsHTTP2 = true
		}
	}
	if !containsHTTP2 {
		t.Fatalf("Dialer.TLSClientConfig.NextProtos does not contain \"h2\"")
	}

	_, _, err = d.Dial(makeWsProto(ts.URL), nil)
	if err == nil {
		t.Fatalf("Dial succeeded, expect fail ")
	}
}
