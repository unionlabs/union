/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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
 */

package badger

import (
	"sync"

	"github.com/dgraph-io/badger/v2/pb"
	"github.com/dgraph-io/badger/v2/trie"
	"github.com/dgraph-io/badger/v2/y"
)

type subscriber struct {
	prefixes  [][]byte
	sendCh    chan<- *pb.KVList
	subCloser *y.Closer
}

type publisher struct {
	sync.Mutex
	pubCh       chan requests
	subscribers map[uint64]subscriber
	nextID      uint64
	indexer     *trie.Trie
}

func newPublisher() *publisher {
	return &publisher{
		pubCh:       make(chan requests, 1000),
		subscribers: make(map[uint64]subscriber),
		nextID:      0,
		indexer:     trie.NewTrie(),
	}
}

func (p *publisher) listenForUpdates(c *y.Closer) {
	defer func() {
		p.cleanSubscribers()
		c.Done()
	}()
	slurp := func(batch requests) {
		for {
			select {
			case reqs := <-p.pubCh:
				batch = append(batch, reqs...)
			default:
				p.publishUpdates(batch)
				return
			}
		}
	}
	for {
		select {
		case <-c.HasBeenClosed():
			return
		case reqs := <-p.pubCh:
			slurp(reqs)
		}
	}
}

func (p *publisher) publishUpdates(reqs requests) {
	p.Lock()
	defer func() {
		p.Unlock()
		// Release all the request.
		reqs.DecrRef()
	}()
	batchedUpdates := make(map[uint64]*pb.KVList)
	for _, req := range reqs {
		for _, e := range req.Entries {
			ids := p.indexer.Get(e.Key)
			if len(ids) > 0 {
				k := y.SafeCopy(nil, e.Key)
				kv := &pb.KV{
					Key:       y.ParseKey(k),
					Value:     y.SafeCopy(nil, e.Value),
					Meta:      []byte{e.UserMeta},
					ExpiresAt: e.ExpiresAt,
					Version:   y.ParseTs(k),
				}
				for id := range ids {
					if _, ok := batchedUpdates[id]; !ok {
						batchedUpdates[id] = &pb.KVList{}
					}
					batchedUpdates[id].Kv = append(batchedUpdates[id].Kv, kv)
				}
			}
		}
	}

	for id, kvs := range batchedUpdates {
		p.subscribers[id].sendCh <- kvs
	}
}

func (p *publisher) newSubscriber(c *y.Closer, prefixes ...[]byte) (<-chan *pb.KVList, uint64) {
	p.Lock()
	defer p.Unlock()
	ch := make(chan *pb.KVList, 1000)
	id := p.nextID
	// Increment next ID.
	p.nextID++
	p.subscribers[id] = subscriber{
		prefixes:  prefixes,
		sendCh:    ch,
		subCloser: c,
	}
	for _, prefix := range prefixes {
		p.indexer.Add(prefix, id)
	}
	return ch, id
}

// cleanSubscribers stops all the subscribers. Ideally, It should be called while closing DB.
func (p *publisher) cleanSubscribers() {
	p.Lock()
	defer p.Unlock()
	for id, s := range p.subscribers {
		for _, prefix := range s.prefixes {
			p.indexer.Delete(prefix, id)
		}
		delete(p.subscribers, id)
		s.subCloser.SignalAndWait()
	}
}

func (p *publisher) deleteSubscriber(id uint64) {
	p.Lock()
	defer p.Unlock()
	if s, ok := p.subscribers[id]; ok {
		for _, prefix := range s.prefixes {
			p.indexer.Delete(prefix, id)
		}
	}
	delete(p.subscribers, id)
}

func (p *publisher) sendUpdates(reqs requests) {
	if p.noOfSubscribers() != 0 {
		reqs.IncrRef()
		p.pubCh <- reqs
	}
}

func (p *publisher) noOfSubscribers() int {
	p.Lock()
	defer p.Unlock()
	return len(p.subscribers)
}
