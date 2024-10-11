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

// Contains a batch of utility type declarations used by the tests. As the node
// operates on unique types, a lot of them are needed to check various features.

package node

import (
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/rpc"
)

// NoopLifecycle is a trivial implementation of the Service interface.
type NoopLifecycle struct{}

func (s *NoopLifecycle) Start() error { return nil }
func (s *NoopLifecycle) Stop() error  { return nil }

func NewNoop() *Noop {
	noop := new(Noop)
	return noop
}

// Set of services all wrapping the base NoopLifecycle resulting in the same method
// signatures but different outer types.
type Noop struct{ NoopLifecycle }

// InstrumentedService is an implementation of Lifecycle for which all interface
// methods can be instrumented both return value as well as event hook wise.
type InstrumentedService struct {
	start error
	stop  error

	startHook func()
	stopHook  func()
}

func (s *InstrumentedService) Start() error {
	if s.startHook != nil {
		s.startHook()
	}
	return s.start
}

func (s *InstrumentedService) Stop() error {
	if s.stopHook != nil {
		s.stopHook()
	}
	return s.stop
}

type FullService struct{}

func NewFullService(stack *Node) (*FullService, error) {
	fs := new(FullService)

	stack.RegisterProtocols(fs.Protocols())
	stack.RegisterAPIs(fs.APIs())
	stack.RegisterLifecycle(fs)
	return fs, nil
}

func (f *FullService) Start() error { return nil }

func (f *FullService) Stop() error { return nil }

func (f *FullService) Protocols() []p2p.Protocol {
	return []p2p.Protocol{
		{
			Name:    "test1",
			Version: uint(1),
		},
		{
			Name:    "test2",
			Version: uint(2),
		},
	}
}

func (f *FullService) APIs() []rpc.API {
	return []rpc.API{
		{
			Namespace: "admin",
		},
		{
			Namespace: "debug",
		},
		{
			Namespace: "net",
		},
	}
}
