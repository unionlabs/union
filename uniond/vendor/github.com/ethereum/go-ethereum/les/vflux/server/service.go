// Copyright 2021 The go-ethereum Authors
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

package server

import (
	"net"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/les/vflux"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/rlp"
)

type (
	// Server serves vflux requests
	Server struct {
		limiter         *utils.Limiter
		lock            sync.Mutex
		services        map[string]*serviceEntry
		delayPerRequest time.Duration
	}

	// Service is a service registered at the Server and identified by a string id
	Service interface {
		Handle(id enode.ID, address string, name string, data []byte) []byte // never called concurrently
	}

	serviceEntry struct {
		id, desc string
		backend  Service
	}
)

// NewServer creates a new Server
func NewServer(delayPerRequest time.Duration) *Server {
	return &Server{
		limiter:         utils.NewLimiter(1000),
		delayPerRequest: delayPerRequest,
		services:        make(map[string]*serviceEntry),
	}
}

// Register registers a Service
func (s *Server) Register(b Service, id, desc string) {
	srv := &serviceEntry{backend: b, id: id, desc: desc}
	if strings.Contains(srv.id, ":") {
		// srv.id + ":" will be used as a service database prefix
		log.Error("Service ID contains ':'", "id", srv.id)
		return
	}
	s.lock.Lock()
	s.services[srv.id] = srv
	s.lock.Unlock()
}

// Serve serves a vflux request batch
// Note: requests are served by the Handle functions of the registered services. Serve
// may be called concurrently but the Handle functions are called sequentially and
// therefore thread safety is guaranteed.
func (s *Server) Serve(id enode.ID, address string, requests vflux.Requests) vflux.Replies {
	reqLen := uint(len(requests))
	if reqLen == 0 || reqLen > vflux.MaxRequestLength {
		return nil
	}
	// Note: the value parameter will be supplied by the token sale module (total amount paid)
	ch := <-s.limiter.Add(id, address, 0, reqLen)
	if ch == nil {
		return nil
	}
	// Note: the limiter ensures that the following section is not running concurrently,
	// the lock only protects against contention caused by new service registration
	s.lock.Lock()
	results := make(vflux.Replies, len(requests))
	for i, req := range requests {
		if service := s.services[req.Service]; service != nil {
			results[i] = service.backend.Handle(id, address, req.Name, req.Params)
		}
	}
	s.lock.Unlock()
	time.Sleep(s.delayPerRequest * time.Duration(reqLen))
	close(ch)
	return results
}

// ServeEncoded serves an encoded vflux request batch and returns the encoded replies
func (s *Server) ServeEncoded(id enode.ID, addr *net.UDPAddr, req []byte) []byte {
	var requests vflux.Requests
	if err := rlp.DecodeBytes(req, &requests); err != nil {
		return nil
	}
	results := s.Serve(id, addr.String(), requests)
	if results == nil {
		return nil
	}
	res, _ := rlp.EncodeToBytes(&results)
	return res
}

// Stop shuts down the server
func (s *Server) Stop() {
	s.limiter.Stop()
}
