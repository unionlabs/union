// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//go:build linux
// +build linux

package grpctransport

import (
	"context"
	"errors"
	"fmt"
	"net"
	"syscall"
	"testing"
	"time"

	"cloud.google.com/go/auth"
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
	tcpConn, ok := conn.(*net.TCPConn)
	if !ok {
		return 0, fmt.Errorf("conn is not *net.TCPConn. got %T", conn)
	}
	rawConn, err := tcpConn.SyscallConn()
	if err != nil {
		return 0, err
	}
	var timeout int
	var syscallErr error
	controlErr := rawConn.Control(func(fd uintptr) {
		timeout, syscallErr = syscall.GetsockoptInt(int(fd), syscall.IPPROTO_TCP, tcpUserTimeoutOp)
	})
	if syscallErr != nil {
		return 0, syscallErr
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

	pool, err := Dial(ctx, true, &Options{
		Credentials: auth.NewCredentials(&auth.CredentialsOptions{
			TokenProvider: &staticTP{tok: &auth.Token{Value: "hey"}},
		}),
		GRPCDialOpts: []grpc.DialOption{userDialer},
		Endpoint:     "example.google.com:443",
		InternalOptions: &InternalOptions{
			EnableDirectPath: true,
		},
	})
	if err != nil {
		t.Errorf("DialGRPC: error %v, want nil", err)
	}
	defer pool.Close()

	// gRPC doesn't connect before the first call.
	grpc.Invoke(ctx, "foo", nil, nil, pool.Connection())
}
