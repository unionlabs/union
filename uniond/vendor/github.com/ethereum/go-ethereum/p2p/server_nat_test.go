// Copyright 2023 The go-ethereum Authors
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

package p2p

import (
	"net"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/internal/testlog"
	"github.com/ethereum/go-ethereum/log"
)

func TestServerPortMapping(t *testing.T) {
	clock := new(mclock.Simulated)
	mockNAT := &mockNAT{mappedPort: 30000}
	srv := Server{
		Config: Config{
			PrivateKey: newkey(),
			NoDial:     true,
			ListenAddr: ":0",
			NAT:        mockNAT,
			Logger:     testlog.Logger(t, log.LvlTrace),
			clock:      clock,
		},
	}
	err := srv.Start()
	if err != nil {
		t.Fatal(err)
	}
	defer srv.Stop()

	// Wait for the port mapping to be registered. Synchronization with the port mapping
	// goroutine works like this: For each iteration, we allow other goroutines to run and
	// also advance the virtual clock by 1 second. Waiting stops when the NAT interface
	// has received some requests, or when the clock reaches a timeout.
	deadline := clock.Now().Add(portMapRefreshInterval)
	for clock.Now() < deadline && mockNAT.mapRequests.Load() < 2 {
		time.Sleep(10 * time.Millisecond)
		clock.Run(1 * time.Second)
	}

	if mockNAT.ipRequests.Load() == 0 {
		t.Fatal("external IP was never requested")
	}
	reqCount := mockNAT.mapRequests.Load()
	if reqCount != 2 {
		t.Error("wrong request count:", reqCount)
	}
	enr := srv.LocalNode().Node()
	if enr.IP().String() != "192.0.2.0" {
		t.Error("wrong IP in ENR:", enr.IP())
	}
	if enr.TCP() != 30000 {
		t.Error("wrong TCP port in ENR:", enr.TCP())
	}
	if enr.UDP() != 30000 {
		t.Error("wrong UDP port in ENR:", enr.UDP())
	}
}

type mockNAT struct {
	mappedPort    uint16
	mapRequests   atomic.Int32
	unmapRequests atomic.Int32
	ipRequests    atomic.Int32
}

func (m *mockNAT) AddMapping(protocol string, extport, intport int, name string, lifetime time.Duration) (uint16, error) {
	m.mapRequests.Add(1)
	return m.mappedPort, nil
}

func (m *mockNAT) DeleteMapping(protocol string, extport, intport int) error {
	m.unmapRequests.Add(1)
	return nil
}

func (m *mockNAT) ExternalIP() (net.IP, error) {
	m.ipRequests.Add(1)
	return net.ParseIP("192.0.2.0"), nil
}

func (m *mockNAT) String() string {
	return "mockNAT"
}
