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

//go:build windows
// +build windows

package rpc

import (
	"context"
	"net"
	"time"

	"github.com/Microsoft/go-winio"
)

// This is used if the dialing context has no deadline. It is much smaller than the
// defaultDialTimeout because named pipes are local and there is no need to wait so long.
const defaultPipeDialTimeout = 2 * time.Second

// ipcListen will create a named pipe on the given endpoint.
func ipcListen(endpoint string) (net.Listener, error) {
	return winio.ListenPipe(endpoint, nil)
}

// newIPCConnection will connect to a named pipe with the given endpoint as name.
func newIPCConnection(ctx context.Context, endpoint string) (net.Conn, error) {
	ctx, cancel := context.WithTimeout(ctx, defaultPipeDialTimeout)
	defer cancel()
	return winio.DialPipeContext(ctx, endpoint)
}
