// Copyright 2019 The go-ethereum Authors
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

package enode

import (
	"sync"
	"time"
)

// Iterator represents a sequence of nodes. The Next method moves to the next node in the
// sequence. It returns false when the sequence has ended or the iterator is closed. Close
// may be called concurrently with Next and Node, and interrupts Next if it is blocked.
type Iterator interface {
	Next() bool  // moves to next node
	Node() *Node // returns current node
	Close()      // ends the iterator
}

// ReadNodes reads at most n nodes from the given iterator. The return value contains no
// duplicates and no nil values. To prevent looping indefinitely for small repeating node
// sequences, this function calls Next at most n times.
func ReadNodes(it Iterator, n int) []*Node {
	seen := make(map[ID]*Node, n)
	for i := 0; i < n && it.Next(); i++ {
		// Remove duplicates, keeping the node with higher seq.
		node := it.Node()
		prevNode, ok := seen[node.ID()]
		if ok && prevNode.Seq() > node.Seq() {
			continue
		}
		seen[node.ID()] = node
	}
	result := make([]*Node, 0, len(seen))
	for _, node := range seen {
		result = append(result, node)
	}
	return result
}

// IterNodes makes an iterator which runs through the given nodes once.
func IterNodes(nodes []*Node) Iterator {
	return &sliceIter{nodes: nodes, index: -1}
}

// CycleNodes makes an iterator which cycles through the given nodes indefinitely.
func CycleNodes(nodes []*Node) Iterator {
	return &sliceIter{nodes: nodes, index: -1, cycle: true}
}

type sliceIter struct {
	mu    sync.Mutex
	nodes []*Node
	index int
	cycle bool
}

func (it *sliceIter) Next() bool {
	it.mu.Lock()
	defer it.mu.Unlock()

	if len(it.nodes) == 0 {
		return false
	}
	it.index++
	if it.index == len(it.nodes) {
		if it.cycle {
			it.index = 0
		} else {
			it.nodes = nil
			return false
		}
	}
	return true
}

func (it *sliceIter) Node() *Node {
	it.mu.Lock()
	defer it.mu.Unlock()
	if len(it.nodes) == 0 {
		return nil
	}
	return it.nodes[it.index]
}

func (it *sliceIter) Close() {
	it.mu.Lock()
	defer it.mu.Unlock()

	it.nodes = nil
}

// Filter wraps an iterator such that Next only returns nodes for which
// the 'check' function returns true.
func Filter(it Iterator, check func(*Node) bool) Iterator {
	return &filterIter{it, check}
}

type filterIter struct {
	Iterator
	check func(*Node) bool
}

func (f *filterIter) Next() bool {
	for f.Iterator.Next() {
		if f.check(f.Node()) {
			return true
		}
	}
	return false
}

// FairMix aggregates multiple node iterators. The mixer itself is an iterator which ends
// only when Close is called. Source iterators added via AddSource are removed from the
// mix when they end.
//
// The distribution of nodes returned by Next is approximately fair, i.e. FairMix
// attempts to draw from all sources equally often. However, if a certain source is slow
// and doesn't return a node within the configured timeout, a node from any other source
// will be returned.
//
// It's safe to call AddSource and Close concurrently with Next.
type FairMix struct {
	wg      sync.WaitGroup
	fromAny chan *Node
	timeout time.Duration
	cur     *Node

	mu      sync.Mutex
	closed  chan struct{}
	sources []*mixSource
	last    int
}

type mixSource struct {
	it      Iterator
	next    chan *Node
	timeout time.Duration
}

// NewFairMix creates a mixer.
//
// The timeout specifies how long the mixer will wait for the next fairly-chosen source
// before giving up and taking a node from any other source. A good way to set the timeout
// is deciding how long you'd want to wait for a node on average. Passing a negative
// timeout makes the mixer completely fair.
func NewFairMix(timeout time.Duration) *FairMix {
	m := &FairMix{
		fromAny: make(chan *Node),
		closed:  make(chan struct{}),
		timeout: timeout,
	}
	return m
}

// AddSource adds a source of nodes.
func (m *FairMix) AddSource(it Iterator) {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.closed == nil {
		return
	}
	m.wg.Add(1)
	source := &mixSource{it, make(chan *Node), m.timeout}
	m.sources = append(m.sources, source)
	go m.runSource(m.closed, source)
}

// Close shuts down the mixer and all current sources.
// Calling this is required to release resources associated with the mixer.
func (m *FairMix) Close() {
	m.mu.Lock()
	defer m.mu.Unlock()

	if m.closed == nil {
		return
	}
	for _, s := range m.sources {
		s.it.Close()
	}
	close(m.closed)
	m.wg.Wait()
	close(m.fromAny)
	m.sources = nil
	m.closed = nil
}

// Next returns a node from a random source.
func (m *FairMix) Next() bool {
	m.cur = nil

	for {
		source := m.pickSource()
		if source == nil {
			return m.nextFromAny()
		}

		var timeout <-chan time.Time
		if source.timeout >= 0 {
			timer := time.NewTimer(source.timeout)
			timeout = timer.C
			defer timer.Stop()
		}

		select {
		case n, ok := <-source.next:
			if ok {
				// Here, the timeout is reset to the configured value
				// because the source delivered a node.
				source.timeout = m.timeout
				m.cur = n
				return true
			}
			// This source has ended.
			m.deleteSource(source)
		case <-timeout:
			// The selected source did not deliver a node within the timeout, so the
			// timeout duration is halved for next time. This is supposed to improve
			// latency with stuck sources.
			source.timeout /= 2
			return m.nextFromAny()
		}
	}
}

// Node returns the current node.
func (m *FairMix) Node() *Node {
	return m.cur
}

// nextFromAny is used when there are no sources or when the 'fair' choice
// doesn't turn up a node quickly enough.
func (m *FairMix) nextFromAny() bool {
	n, ok := <-m.fromAny
	if ok {
		m.cur = n
	}
	return ok
}

// pickSource chooses the next source to read from, cycling through them in order.
func (m *FairMix) pickSource() *mixSource {
	m.mu.Lock()
	defer m.mu.Unlock()

	if len(m.sources) == 0 {
		return nil
	}
	m.last = (m.last + 1) % len(m.sources)
	return m.sources[m.last]
}

// deleteSource deletes a source.
func (m *FairMix) deleteSource(s *mixSource) {
	m.mu.Lock()
	defer m.mu.Unlock()

	for i := range m.sources {
		if m.sources[i] == s {
			copy(m.sources[i:], m.sources[i+1:])
			m.sources[len(m.sources)-1] = nil
			m.sources = m.sources[:len(m.sources)-1]
			break
		}
	}
}

// runSource reads a single source in a loop.
func (m *FairMix) runSource(closed chan struct{}, s *mixSource) {
	defer m.wg.Done()
	defer close(s.next)
	for s.it.Next() {
		n := s.it.Node()
		select {
		case s.next <- n:
		case m.fromAny <- n:
		case <-closed:
			return
		}
	}
}
