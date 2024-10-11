// Copyright 2023 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.21

// The interop command is the client and server used by QUIC interoperability tests.
//
// https://github.com/marten-seemann/quic-interop-runner
package main

import (
	"bytes"
	"context"
	"crypto/tls"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"log/slog"
	"net"
	"net/url"
	"os"
	"path/filepath"
	"sync"

	"golang.org/x/net/quic"
	"golang.org/x/net/quic/qlog"
)

var (
	listen  = flag.String("listen", "", "listen address")
	cert    = flag.String("cert", "", "certificate")
	pkey    = flag.String("key", "", "private key")
	root    = flag.String("root", "", "serve files from this root")
	output  = flag.String("output", "", "directory to write files to")
	qlogdir = flag.String("qlog", "", "directory to write qlog output to")
)

func main() {
	ctx := context.Background()
	flag.Parse()
	urls := flag.Args()

	config := &quic.Config{
		TLSConfig: &tls.Config{
			InsecureSkipVerify: true,
			MinVersion:         tls.VersionTLS13,
			NextProtos:         []string{"hq-interop"},
		},
		MaxBidiRemoteStreams: -1,
		MaxUniRemoteStreams:  -1,
		QLogLogger: slog.New(qlog.NewJSONHandler(qlog.HandlerOptions{
			Level: quic.QLogLevelFrame,
			Dir:   *qlogdir,
		})),
	}
	if *cert != "" {
		c, err := tls.LoadX509KeyPair(*cert, *pkey)
		if err != nil {
			log.Fatal(err)
		}
		config.TLSConfig.Certificates = []tls.Certificate{c}
	}
	if *root != "" {
		config.MaxBidiRemoteStreams = 100
	}
	if keylog := os.Getenv("SSLKEYLOGFILE"); keylog != "" {
		f, err := os.Create(keylog)
		if err != nil {
			log.Fatal(err)
		}
		defer f.Close()
		config.TLSConfig.KeyLogWriter = f
	}

	testcase := os.Getenv("TESTCASE")
	switch testcase {
	case "handshake", "keyupdate":
		basicTest(ctx, config, urls)
		return
	case "chacha20":
		// "[...] offer only ChaCha20 as a ciphersuite."
		//
		// crypto/tls does not support configuring TLS 1.3 ciphersuites,
		// so we can't support this test.
	case "transfer":
		// "The client should use small initial flow control windows
		// for both stream- and connection-level flow control
		// such that the during the transfer of files on the order of 1 MB
		// the flow control window needs to be increased."
		config.MaxStreamReadBufferSize = 64 << 10
		config.MaxConnReadBufferSize = 64 << 10
		basicTest(ctx, config, urls)
		return
	case "http3":
		// TODO
	case "multiconnect":
		// TODO
	case "resumption":
		// TODO
	case "retry":
		// TODO
	case "versionnegotiation":
		// "The client should start a connection using
		// an unsupported version number [...]"
		//
		// We don't support setting the client's version,
		// so only run this test as a server.
		if *listen != "" && len(urls) == 0 {
			basicTest(ctx, config, urls)
			return
		}
	case "v2":
		// We do not support QUIC v2.
	case "zerortt":
		// TODO
	}
	fmt.Printf("unsupported test case %q\n", testcase)
	os.Exit(127)
}

// basicTest runs the standard test setup.
//
// As a server, it serves the contents of the -root directory.
// As a client, it downloads all the provided URLs in parallel,
// making one connection to each destination server.
func basicTest(ctx context.Context, config *quic.Config, urls []string) {
	l, err := quic.Listen("udp", *listen, config)
	if err != nil {
		log.Fatal(err)
	}
	log.Printf("listening on %v", l.LocalAddr())

	byAuthority := map[string][]*url.URL{}
	for _, s := range urls {
		u, addr, err := parseURL(s)
		if err != nil {
			log.Fatal(err)
		}
		byAuthority[addr] = append(byAuthority[addr], u)
	}
	var g sync.WaitGroup
	defer g.Wait()
	for addr, u := range byAuthority {
		addr, u := addr, u
		g.Add(1)
		go func() {
			defer g.Done()
			fetchFrom(ctx, config, l, addr, u)
		}()
	}

	if config.MaxBidiRemoteStreams >= 0 {
		serve(ctx, l)
	}
}

func serve(ctx context.Context, l *quic.Endpoint) error {
	for {
		c, err := l.Accept(ctx)
		if err != nil {
			return err
		}
		go serveConn(ctx, c)
	}
}

func serveConn(ctx context.Context, c *quic.Conn) {
	for {
		s, err := c.AcceptStream(ctx)
		if err != nil {
			return
		}
		go func() {
			if err := serveReq(ctx, s); err != nil {
				log.Print("serveReq:", err)
			}
		}()
	}
}

func serveReq(ctx context.Context, s *quic.Stream) error {
	defer s.Close()
	req, err := io.ReadAll(s)
	if err != nil {
		return err
	}
	if !bytes.HasSuffix(req, []byte("\r\n")) {
		return errors.New("invalid request")
	}
	req = bytes.TrimSuffix(req, []byte("\r\n"))
	if !bytes.HasPrefix(req, []byte("GET /")) {
		return errors.New("invalid request")
	}
	req = bytes.TrimPrefix(req, []byte("GET /"))
	if !filepath.IsLocal(string(req)) {
		return errors.New("invalid request")
	}
	f, err := os.Open(filepath.Join(*root, string(req)))
	if err != nil {
		return err
	}
	defer f.Close()
	_, err = io.Copy(s, f)
	return err
}

func parseURL(s string) (u *url.URL, authority string, err error) {
	u, err = url.Parse(s)
	if err != nil {
		return nil, "", err
	}
	host := u.Hostname()
	port := u.Port()
	if port == "" {
		port = "443"
	}
	authority = net.JoinHostPort(host, port)
	return u, authority, nil
}

func fetchFrom(ctx context.Context, config *quic.Config, l *quic.Endpoint, addr string, urls []*url.URL) {
	conn, err := l.Dial(ctx, "udp", addr, config)
	if err != nil {
		log.Printf("%v: %v", addr, err)
		return
	}
	log.Printf("connected to %v", addr)
	defer conn.Close()
	var g sync.WaitGroup
	for _, u := range urls {
		u := u
		g.Add(1)
		go func() {
			defer g.Done()
			if err := fetchOne(ctx, conn, u); err != nil {
				log.Printf("fetch %v: %v", u, err)
			} else {
				log.Printf("fetched %v", u)
			}
		}()
	}
	g.Wait()
}

func fetchOne(ctx context.Context, conn *quic.Conn, u *url.URL) error {
	if len(u.Path) == 0 || u.Path[0] != '/' || !filepath.IsLocal(u.Path[1:]) {
		return errors.New("invalid path")
	}
	file, err := os.Create(filepath.Join(*output, u.Path[1:]))
	if err != nil {
		return err
	}
	s, err := conn.NewStream(ctx)
	if err != nil {
		return err
	}
	defer s.Close()
	if _, err := s.Write([]byte("GET " + u.Path + "\r\n")); err != nil {
		return err
	}
	s.CloseWrite()
	if _, err := io.Copy(file, s); err != nil {
		return err
	}
	return nil
}
