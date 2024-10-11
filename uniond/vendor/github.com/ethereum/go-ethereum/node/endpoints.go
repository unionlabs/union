// Copyright 2020 The go-ethereum Authors
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

package node

import (
	"net"
	"net/http"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rpc"
)

// StartHTTPEndpoint starts the HTTP RPC endpoint.
func StartHTTPEndpoint(endpoint string, timeouts rpc.HTTPTimeouts, handler http.Handler) (*http.Server, net.Addr, error) {
	// start the HTTP listener
	var (
		listener net.Listener
		err      error
	)
	if listener, err = net.Listen("tcp", endpoint); err != nil {
		return nil, nil, err
	}
	// make sure timeout values are meaningful
	CheckTimeouts(&timeouts)
	// Bundle and start the HTTP server
	httpSrv := &http.Server{
		Handler:           handler,
		ReadTimeout:       timeouts.ReadTimeout,
		ReadHeaderTimeout: timeouts.ReadHeaderTimeout,
		WriteTimeout:      timeouts.WriteTimeout,
		IdleTimeout:       timeouts.IdleTimeout,
	}
	go httpSrv.Serve(listener)
	return httpSrv, listener.Addr(), err
}

// checkModuleAvailability checks that all names given in modules are actually
// available API services. It assumes that the MetadataApi module ("rpc") is always available;
// the registration of this "rpc" module happens in NewServer() and is thus common to all endpoints.
func checkModuleAvailability(modules []string, apis []rpc.API) (bad, available []string) {
	availableSet := make(map[string]struct{})
	for _, api := range apis {
		if _, ok := availableSet[api.Namespace]; !ok {
			availableSet[api.Namespace] = struct{}{}
			available = append(available, api.Namespace)
		}
	}
	for _, name := range modules {
		if _, ok := availableSet[name]; !ok {
			if name != rpc.MetadataApi && name != rpc.EngineApi {
				bad = append(bad, name)
			}
		}
	}
	return bad, available
}

// CheckTimeouts ensures that timeout values are meaningful
func CheckTimeouts(timeouts *rpc.HTTPTimeouts) {
	if timeouts.ReadTimeout < time.Second {
		log.Warn("Sanitizing invalid HTTP read timeout", "provided", timeouts.ReadTimeout, "updated", rpc.DefaultHTTPTimeouts.ReadTimeout)
		timeouts.ReadTimeout = rpc.DefaultHTTPTimeouts.ReadTimeout
	}
	if timeouts.ReadHeaderTimeout < time.Second {
		log.Warn("Sanitizing invalid HTTP read header timeout", "provided", timeouts.ReadHeaderTimeout, "updated", rpc.DefaultHTTPTimeouts.ReadHeaderTimeout)
		timeouts.ReadHeaderTimeout = rpc.DefaultHTTPTimeouts.ReadHeaderTimeout
	}
	if timeouts.WriteTimeout < time.Second {
		log.Warn("Sanitizing invalid HTTP write timeout", "provided", timeouts.WriteTimeout, "updated", rpc.DefaultHTTPTimeouts.WriteTimeout)
		timeouts.WriteTimeout = rpc.DefaultHTTPTimeouts.WriteTimeout
	}
	if timeouts.IdleTimeout < time.Second {
		log.Warn("Sanitizing invalid HTTP idle timeout", "provided", timeouts.IdleTimeout, "updated", rpc.DefaultHTTPTimeouts.IdleTimeout)
		timeouts.IdleTimeout = rpc.DefaultHTTPTimeouts.IdleTimeout
	}
}
