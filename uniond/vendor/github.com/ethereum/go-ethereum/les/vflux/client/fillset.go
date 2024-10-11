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

package client

import (
	"sync"

	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
)

// FillSet tries to read nodes from an input iterator and add them to a node set by
// setting the specified node state flag(s) until the size of the set reaches the target.
// Note that other mechanisms (like other FillSet instances reading from different inputs)
// can also set the same flag(s) and FillSet will always care about the total number of
// nodes having those flags.
type FillSet struct {
	lock          sync.Mutex
	cond          *sync.Cond
	ns            *nodestate.NodeStateMachine
	input         enode.Iterator
	closed        bool
	flags         nodestate.Flags
	count, target int
}

// NewFillSet creates a new FillSet
func NewFillSet(ns *nodestate.NodeStateMachine, input enode.Iterator, flags nodestate.Flags) *FillSet {
	fs := &FillSet{
		ns:    ns,
		input: input,
		flags: flags,
	}
	fs.cond = sync.NewCond(&fs.lock)

	ns.SubscribeState(flags, func(n *enode.Node, oldState, newState nodestate.Flags) {
		fs.lock.Lock()
		if oldState.Equals(flags) {
			fs.count--
		}
		if newState.Equals(flags) {
			fs.count++
		}
		if fs.target > fs.count {
			fs.cond.Signal()
		}
		fs.lock.Unlock()
	})

	go fs.readLoop()
	return fs
}

// readLoop keeps reading nodes from the input and setting the specified flags for them
// whenever the node set size is under the current target
func (fs *FillSet) readLoop() {
	for {
		fs.lock.Lock()
		for fs.target <= fs.count && !fs.closed {
			fs.cond.Wait()
		}

		fs.lock.Unlock()
		if !fs.input.Next() {
			return
		}
		fs.ns.SetState(fs.input.Node(), fs.flags, nodestate.Flags{}, 0)
	}
}

// SetTarget sets the current target for node set size. If the previous target was not
// reached and FillSet was still waiting for the next node from the input then the next
// incoming node will be added to the set regardless of the target. This ensures that
// all nodes coming from the input are eventually added to the set.
func (fs *FillSet) SetTarget(target int) {
	fs.lock.Lock()
	defer fs.lock.Unlock()

	fs.target = target
	if fs.target > fs.count {
		fs.cond.Signal()
	}
}

// Close shuts FillSet down and closes the input iterator
func (fs *FillSet) Close() {
	fs.lock.Lock()
	defer fs.lock.Unlock()

	fs.closed = true
	fs.input.Close()
	fs.cond.Signal()
}
