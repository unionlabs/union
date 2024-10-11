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

package node_test

import (
	"fmt"
	"log"

	"github.com/ethereum/go-ethereum/node"
)

// SampleLifecycle is a trivial network service that can be attached to a node for
// life cycle management.
//
// The following methods are needed to implement a node.Lifecycle:
//   - Start() error              - method invoked when the node is ready to start the service
//   - Stop() error               - method invoked when the node terminates the service
type SampleLifecycle struct{}

func (s *SampleLifecycle) Start() error { fmt.Println("Service starting..."); return nil }
func (s *SampleLifecycle) Stop() error  { fmt.Println("Service stopping..."); return nil }

func ExampleLifecycle() {
	// Create a network node to run protocols with the default values.
	stack, err := node.New(&node.Config{})
	if err != nil {
		log.Fatalf("Failed to create network node: %v", err)
	}
	defer stack.Close()

	// Create and register a simple network Lifecycle.
	service := new(SampleLifecycle)
	stack.RegisterLifecycle(service)

	// Boot up the entire protocol stack, do a restart and terminate
	if err := stack.Start(); err != nil {
		log.Fatalf("Failed to start the protocol stack: %v", err)
	}
	if err := stack.Close(); err != nil {
		log.Fatalf("Failed to stop the protocol stack: %v", err)
	}
	// Output:
	// Service starting...
	// Service stopping...
}
