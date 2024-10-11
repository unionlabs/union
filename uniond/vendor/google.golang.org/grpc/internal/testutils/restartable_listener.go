/*
 *
 * Copyright 2019 gRPC authors.
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

package testutils

import (
	"net"
	"sync"
)

type tempError struct{}

func (*tempError) Error() string {
	return "restartable listener temporary error"
}
func (*tempError) Temporary() bool {
	return true
}

// RestartableListener wraps a net.Listener and supports stopping and restarting
// the latter.
type RestartableListener struct {
	lis net.Listener

	mu      sync.Mutex
	stopped bool
	conns   []net.Conn
}

// NewRestartableListener returns a new RestartableListener wrapping l.
func NewRestartableListener(l net.Listener) *RestartableListener {
	return &RestartableListener{lis: l}
}

// Accept waits for and returns the next connection to the listener.
//
// If the listener is currently not accepting new connections, because `Stop`
// was called on it, the connection is immediately closed after accepting
// without any bytes being sent on it.
func (l *RestartableListener) Accept() (net.Conn, error) {
	conn, err := l.lis.Accept()
	if err != nil {
		return nil, err
	}

	l.mu.Lock()
	defer l.mu.Unlock()
	if l.stopped {
		conn.Close()
		return nil, &tempError{}
	}
	l.conns = append(l.conns, conn)
	return conn, nil
}

// Close closes the listener.
func (l *RestartableListener) Close() error {
	return l.lis.Close()
}

// Addr returns the listener's network address.
func (l *RestartableListener) Addr() net.Addr {
	return l.lis.Addr()
}

// Stop closes existing connections on the listener and prevents new connections
// from being accepted.
func (l *RestartableListener) Stop() {
	l.mu.Lock()
	l.stopped = true
	for _, conn := range l.conns {
		conn.Close()
	}
	l.conns = nil
	l.mu.Unlock()
}

// Restart gets a previously stopped listener to start accepting connections.
func (l *RestartableListener) Restart() {
	l.mu.Lock()
	l.stopped = false
	l.mu.Unlock()
}
