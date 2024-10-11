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

// Package local implements local transport credentials.
// Local credentials reports the security level based on the type
// of connection. If the connection is local TCP, NoSecurity will be
// reported, and if the connection is UDS, PrivacyAndIntegrity will be
// reported. If local credentials is not used in local connections
// (local TCP or UDS), it will fail.
//
// # Experimental
//
// Notice: This package is EXPERIMENTAL and may be changed or removed in a
// later release.
package local

import (
	"context"
	"fmt"
	"net"
	"strings"

	"google.golang.org/grpc/credentials"
)

// info contains the auth information for a local connection.
// It implements the AuthInfo interface.
type info struct {
	credentials.CommonAuthInfo
}

// AuthType returns the type of info as a string.
func (info) AuthType() string {
	return "local"
}

// localTC is the credentials required to establish a local connection.
type localTC struct {
	info credentials.ProtocolInfo
}

func (c *localTC) Info() credentials.ProtocolInfo {
	return c.info
}

// getSecurityLevel returns the security level for a local connection.
// It returns an error if a connection is not local.
func getSecurityLevel(network, addr string) (credentials.SecurityLevel, error) {
	switch {
	// Local TCP connection
	case strings.HasPrefix(addr, "127."), strings.HasPrefix(addr, "[::1]:"):
		return credentials.NoSecurity, nil
	// Windows named pipe connection
	case network == "pipe" && strings.HasPrefix(addr, `\\.\pipe\`):
		return credentials.NoSecurity, nil
	// UDS connection
	case network == "unix":
		return credentials.PrivacyAndIntegrity, nil
	// Not a local connection and should fail
	default:
		return credentials.InvalidSecurityLevel, fmt.Errorf("local credentials rejected connection to non-local address %q", addr)
	}
}

func (*localTC) ClientHandshake(_ context.Context, _ string, conn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	secLevel, err := getSecurityLevel(conn.RemoteAddr().Network(), conn.RemoteAddr().String())
	if err != nil {
		return nil, nil, err
	}
	return conn, info{credentials.CommonAuthInfo{SecurityLevel: secLevel}}, nil
}

func (*localTC) ServerHandshake(conn net.Conn) (net.Conn, credentials.AuthInfo, error) {
	secLevel, err := getSecurityLevel(conn.RemoteAddr().Network(), conn.RemoteAddr().String())
	if err != nil {
		return nil, nil, err
	}
	return conn, info{credentials.CommonAuthInfo{SecurityLevel: secLevel}}, nil
}

// NewCredentials returns a local credential implementing credentials.TransportCredentials.
func NewCredentials() credentials.TransportCredentials {
	return &localTC{
		info: credentials.ProtocolInfo{
			SecurityProtocol: "local",
		},
	}
}

// Clone makes a copy of Local credentials.
func (c *localTC) Clone() credentials.TransportCredentials {
	return &localTC{info: c.info}
}

// OverrideServerName overrides the server name used to verify the hostname on the returned certificates from the server.
// Since this feature is specific to TLS (SNI + hostname verification check), it does not take any effect for local credentials.
func (c *localTC) OverrideServerName(serverNameOverride string) error {
	c.info.ServerName = serverNameOverride
	return nil
}
