/*
 *
 * Copyright 2020 gRPC authors.
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

package local

import (
	"context"
	"fmt"
	"net"
	"runtime"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc/credentials"
	"google.golang.org/grpc/internal/grpctest"
)

const defaultTestTimeout = 10 * time.Second

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

func (s) TestGetSecurityLevel(t *testing.T) {
	testCases := []struct {
		testNetwork string
		testAddr    string
		want        credentials.SecurityLevel
	}{
		{
			testNetwork: "tcp",
			testAddr:    "127.0.0.1:10000",
			want:        credentials.NoSecurity,
		},
		{
			testNetwork: "tcp",
			testAddr:    "[::1]:10000",
			want:        credentials.NoSecurity,
		},
		{
			testNetwork: "unix",
			testAddr:    "/tmp/grpc_fullstack_test",
			want:        credentials.PrivacyAndIntegrity,
		},
		{
			testNetwork: "tcp",
			testAddr:    "192.168.0.1:10000",
			want:        credentials.InvalidSecurityLevel,
		},
	}
	for _, tc := range testCases {
		got, _ := getSecurityLevel(tc.testNetwork, tc.testAddr)
		if got != tc.want {
			t.Fatalf("GetSeurityLevel(%s, %s) returned %s but want %s", tc.testNetwork, tc.testAddr, got.String(), tc.want.String())
		}
	}
}

type serverHandshake func(net.Conn) (credentials.AuthInfo, error)

func getSecurityLevelFromAuthInfo(ai credentials.AuthInfo) credentials.SecurityLevel {
	if c, ok := ai.(interface {
		GetCommonAuthInfo() credentials.CommonAuthInfo
	}); ok {
		return c.GetCommonAuthInfo().SecurityLevel
	}
	return credentials.InvalidSecurityLevel
}

// Server local handshake implementation.
func serverLocalHandshake(conn net.Conn) (credentials.AuthInfo, error) {
	cred := NewCredentials()
	_, authInfo, err := cred.ServerHandshake(conn)
	if err != nil {
		return nil, err
	}
	return authInfo, nil
}

// Client local handshake implementation.
func clientLocalHandshake(conn net.Conn, lisAddr string) (credentials.AuthInfo, error) {
	cred := NewCredentials()
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	_, authInfo, err := cred.ClientHandshake(ctx, lisAddr, conn)
	if err != nil {
		return nil, err
	}
	return authInfo, nil
}

// Client connects to a server with local credentials.
func clientHandle(hs func(net.Conn, string) (credentials.AuthInfo, error), network, lisAddr string) (credentials.AuthInfo, error) {
	conn, _ := net.Dial(network, lisAddr)
	defer conn.Close()
	clientAuthInfo, err := hs(conn, lisAddr)
	if err != nil {
		return nil, fmt.Errorf("Error on client while handshake")
	}
	return clientAuthInfo, nil
}

type testServerHandleResult struct {
	authInfo credentials.AuthInfo
	err      error
}

// Server accepts a client's connection with local credentials.
func serverHandle(hs serverHandshake, done chan testServerHandleResult, lis net.Listener) {
	serverRawConn, err := lis.Accept()
	if err != nil {
		done <- testServerHandleResult{authInfo: nil, err: fmt.Errorf("Server failed to accept connection. Error: %v", err)}
		return
	}
	serverAuthInfo, err := hs(serverRawConn)
	if err != nil {
		serverRawConn.Close()
		done <- testServerHandleResult{authInfo: nil, err: fmt.Errorf("Server failed while handshake. Error: %v", err)}
		return
	}
	done <- testServerHandleResult{authInfo: serverAuthInfo, err: nil}
}

func serverAndClientHandshake(lis net.Listener) (credentials.SecurityLevel, error) {
	done := make(chan testServerHandleResult, 1)
	const timeout = 5 * time.Second
	timer := time.NewTimer(timeout)
	defer timer.Stop()
	go serverHandle(serverLocalHandshake, done, lis)
	defer lis.Close()
	clientAuthInfo, err := clientHandle(clientLocalHandshake, lis.Addr().Network(), lis.Addr().String())
	if err != nil {
		return credentials.InvalidSecurityLevel, fmt.Errorf("Error at client-side: %v", err)
	}
	select {
	case <-timer.C:
		return credentials.InvalidSecurityLevel, fmt.Errorf("Test didn't finish in time")
	case serverHandleResult := <-done:
		if serverHandleResult.err != nil {
			return credentials.InvalidSecurityLevel, fmt.Errorf("Error at server-side: %v", serverHandleResult.err)
		}
		clientSecLevel := getSecurityLevelFromAuthInfo(clientAuthInfo)
		serverSecLevel := getSecurityLevelFromAuthInfo(serverHandleResult.authInfo)

		if clientSecLevel == credentials.InvalidSecurityLevel {
			return credentials.InvalidSecurityLevel, fmt.Errorf("Error at client-side: client's AuthInfo does not implement GetCommonAuthInfo()")
		}
		if serverSecLevel == credentials.InvalidSecurityLevel {
			return credentials.InvalidSecurityLevel, fmt.Errorf("Error at server-side: server's AuthInfo does not implement GetCommonAuthInfo()")
		}
		if clientSecLevel != serverSecLevel {
			return credentials.InvalidSecurityLevel, fmt.Errorf("client's AuthInfo contains %s but server's AuthInfo contains %s", clientSecLevel.String(), serverSecLevel.String())
		}
		return clientSecLevel, nil
	}
}

func (s) TestServerAndClientHandshake(t *testing.T) {
	testCases := []struct {
		testNetwork string
		testAddr    string
		want        credentials.SecurityLevel
	}{
		{
			testNetwork: "tcp",
			testAddr:    "127.0.0.1:0",
			want:        credentials.NoSecurity,
		},
		{
			testNetwork: "tcp",
			testAddr:    "[::1]:0",
			want:        credentials.NoSecurity,
		},
		{
			testNetwork: "tcp",
			testAddr:    "localhost:0",
			want:        credentials.NoSecurity,
		},
		{
			testNetwork: "unix",
			testAddr:    fmt.Sprintf("/tmp/grpc_fullstck_test%d", time.Now().UnixNano()),
			want:        credentials.PrivacyAndIntegrity,
		},
	}
	for _, tc := range testCases {
		if runtime.GOOS == "windows" && tc.testNetwork == "unix" {
			t.Skip("skipping tests for unix connections on Windows")
		}
		t.Run("serverAndClientHandshakeResult", func(t *testing.T) {
			lis, err := net.Listen(tc.testNetwork, tc.testAddr)
			if err != nil {
				if strings.Contains(err.Error(), "bind: cannot assign requested address") ||
					strings.Contains(err.Error(), "socket: address family not supported by protocol") {
					t.Skipf("no support for address %v", tc.testAddr)
				}
				t.Fatalf("Failed to listen: %v", err)
			}
			got, err := serverAndClientHandshake(lis)
			if got != tc.want {
				t.Fatalf("serverAndClientHandshake(%s, %s) = %v, %v; want %v, nil", tc.testNetwork, tc.testAddr, got, err, tc.want)
			}
		})
	}
}
