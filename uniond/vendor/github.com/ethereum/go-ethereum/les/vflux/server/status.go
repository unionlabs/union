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
	"reflect"

	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

type peerWrapper struct{ clientPeer } // the NodeStateMachine type system needs this wrapper

// serverSetup is a wrapper of the node state machine setup, which contains
// all the created flags and fields used in the vflux server side.
type serverSetup struct {
	setup       *nodestate.Setup
	clientField nodestate.Field // Field contains the client peer handler

	// Flags and fields controlled by balance tracker. BalanceTracker
	// is responsible for setting/deleting these flags or fields.
	priorityFlag nodestate.Flags // Flag is set if the node has a positive balance
	updateFlag   nodestate.Flags // Flag is set whenever the node balance is changed(priority changed)
	balanceField nodestate.Field // Field contains the client balance for priority calculation

	// Flags and fields controlled by priority queue. Priority queue
	// is responsible for setting/deleting these flags or fields.
	activeFlag    nodestate.Flags // Flag is set if the node is active
	inactiveFlag  nodestate.Flags // Flag is set if the node is inactive
	capacityField nodestate.Field // Field contains the capacity of the node
	queueField    nodestate.Field // Field contains the information in the priority queue
}

// newServerSetup initializes the setup for state machine and returns the flags/fields group.
func newServerSetup() *serverSetup {
	setup := &serverSetup{setup: &nodestate.Setup{}}
	setup.clientField = setup.setup.NewField("client", reflect.TypeOf(peerWrapper{}))
	setup.priorityFlag = setup.setup.NewFlag("priority")
	setup.updateFlag = setup.setup.NewFlag("update")
	setup.balanceField = setup.setup.NewField("balance", reflect.TypeOf(&nodeBalance{}))
	setup.activeFlag = setup.setup.NewFlag("active")
	setup.inactiveFlag = setup.setup.NewFlag("inactive")
	setup.capacityField = setup.setup.NewField("capacity", reflect.TypeOf(uint64(0)))
	setup.queueField = setup.setup.NewField("queue", reflect.TypeOf(&ppNodeInfo{}))
	return setup
}
