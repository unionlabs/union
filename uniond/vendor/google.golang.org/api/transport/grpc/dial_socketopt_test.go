// Copyright 2019 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.11 && linux
// +build go1.11,linux

package grpc

import (
	"context"
	"errors"
	"fmt"
	"net"
	"syscall"
	"testing"
	"time"

	"golang.org/x/oauth2"
	"google.golang.org/api/option"
	"google.golang.org/api/option/internaloption"
	"google.golang.org/grpc"
)

func TestDialTCPUserTimeout(t *testing.T) {
	l, err := net.Listen("tcp", ":3000")
	if err != nil {
		t.Fatal(err)
	}
	defer l.Close()

	acceptErrCh := make(chan error, 1)

	go func() {
		conn, err := l.Accept()
		if err != nil {
			acceptErrCh <- err
			return
		}
		defer conn.Close()

		if err := conn.Close(); err != nil {
			acceptErrCh <- err
		}
	}()

	conn, err := dialTCPUserTimeout(context.Background(), ":3000")
	if err != nil {
		t.Fatal(err)
	}
	defer conn.Close()

	timeout, err := getTCPUserTimeout(conn)
	if err != nil {
		t.Fatal(err)
	}
	if timeout != tcpUserTimeoutMilliseconds {
		t.Fatalf("expected %v, got %v", tcpUserTimeoutMilliseconds, timeout)
	}

	select {
	case err := <-acceptErrCh:
		t.Fatalf("Accept failed with: %v", err)
	default:
	}
}

func getTCPUserTimeout(conn net.Conn) (int, error) {
	tcpconn, ok := conn.(*net.TCPConn)
	if !ok {
		return 0, fmt.Errorf("conn is not *net.TCPConn. got %T", conn)
	}
	rawConn, err := tcpconn.SyscallConn()
	if err != nil {
		return 0, err
	}
	var timeout int
	var syscalErr error
	controlErr := rawConn.Control(func(fd uintptr) {
		timeout, syscalErr = syscall.GetsockoptInt(int(fd), syscall.IPPROTO_TCP, tcpUserTimeoutOp)
	})
	if syscalErr != nil {
		return 0, syscalErr
	}
	if controlErr != nil {
		return 0, controlErr
	}
	return timeout, nil
}

// Check that tcp timeout dialer overwrites user defined dialer.
func TestDialWithDirectPathEnabled(t *testing.T) {
	t.Skip("https://github.com/googleapis/google-api-go-client/issues/790")
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Millisecond)

	userDialer := grpc.WithDialer(func(addr string, timeout time.Duration) (net.Conn, error) {
		t.Error("did not expect a call to user dialer, got one")
		cancel()
		return nil, errors.New("not expected")
	})

	conn, err := Dial(ctx,
		option.WithTokenSource(oauth2.StaticTokenSource(nil)), // No creds.
		option.WithGRPCDialOption(userDialer),
		option.WithEndpoint("example.google.com:443"),
		internaloption.EnableDirectPath(true))
	if err != nil {
		t.Errorf("DialGRPC: error %v, want nil", err)
	}
	defer conn.Close()

	// gRPC doesn't connect before the first call.
	grpc.Invoke(ctx, "foo", nil, nil, conn)
}
