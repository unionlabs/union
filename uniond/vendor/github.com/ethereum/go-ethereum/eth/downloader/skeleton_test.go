// Copyright 2022 The go-ethereum Authors
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

package downloader

import (
	"encoding/json"
	"errors"
	"fmt"
	"math/big"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/eth/protocols/eth"
	"github.com/ethereum/go-ethereum/log"
)

// hookedBackfiller is a tester backfiller with all interface methods mocked and
// hooked so tests can implement only the things they need.
type hookedBackfiller struct {
	// suspendHook is an optional hook to be called when the filler is requested
	// to be suspended.
	suspendHook func() *types.Header

	// resumeHook is an optional hook to be called when the filler is requested
	// to be resumed.
	resumeHook func()
}

// newHookedBackfiller creates a hooked backfiller with all callbacks disabled,
// essentially acting as a noop.
func newHookedBackfiller() backfiller {
	return new(hookedBackfiller)
}

// suspend requests the backfiller to abort any running full or snap sync
// based on the skeleton chain as it might be invalid. The backfiller should
// gracefully handle multiple consecutive suspends without a resume, even
// on initial startup.
func (hf *hookedBackfiller) suspend() *types.Header {
	if hf.suspendHook != nil {
		return hf.suspendHook()
	}
	return nil // we don't really care about header cleanups for now
}

// resume requests the backfiller to start running fill or snap sync based on
// the skeleton chain as it has successfully been linked. Appending new heads
// to the end of the chain will not result in suspend/resume cycles.
func (hf *hookedBackfiller) resume() {
	if hf.resumeHook != nil {
		hf.resumeHook()
	}
}

// skeletonTestPeer is a mock peer that can only serve header requests from a
// pre-perated header chain (which may be arbitrarily wrong for testing).
//
// Requesting anything else from these peers will hard panic. Note, do *not*
// implement any other methods. We actually want to make sure that the skeleton
// syncer only depends on - and will only ever do so - on header requests.
type skeletonTestPeer struct {
	id      string          // Unique identifier of the mock peer
	headers []*types.Header // Headers to serve when requested

	serve func(origin uint64) []*types.Header // Hook to allow custom responses

	served  atomic.Uint64 // Number of headers served by this peer
	dropped atomic.Uint64 // Flag whether the peer was dropped (stop responding)
}

// newSkeletonTestPeer creates a new mock peer to test the skeleton sync with.
func newSkeletonTestPeer(id string, headers []*types.Header) *skeletonTestPeer {
	return &skeletonTestPeer{
		id:      id,
		headers: headers,
	}
}

// newSkeletonTestPeer creates a new mock peer to test the skeleton sync with,
// and sets an optional serve hook that can return headers for delivery instead
// of the predefined chain. Useful for emulating malicious behavior that would
// otherwise require dedicated peer types.
func newSkeletonTestPeerWithHook(id string, headers []*types.Header, serve func(origin uint64) []*types.Header) *skeletonTestPeer {
	return &skeletonTestPeer{
		id:      id,
		headers: headers,
		serve:   serve,
	}
}

// RequestHeadersByNumber constructs a GetBlockHeaders function based on a numbered
// origin; associated with a particular peer in the download tester. The returned
// function can be used to retrieve batches of headers from the particular peer.
func (p *skeletonTestPeer) RequestHeadersByNumber(origin uint64, amount int, skip int, reverse bool, sink chan *eth.Response) (*eth.Request, error) {
	// Since skeleton test peer are in-memory mocks, dropping the does not make
	// them inaccessible. As such, check a local `dropped` field to see if the
	// peer has been dropped and should not respond any more.
	if p.dropped.Load() != 0 {
		return nil, errors.New("peer already dropped")
	}
	// Skeleton sync retrieves batches of headers going backward without gaps.
	// This ensures we can follow a clean parent progression without any reorg
	// hiccups. There is no need for any other type of header retrieval, so do
	// panic if there's such a request.
	if !reverse || skip != 0 {
		// Note, if other clients want to do these kinds of requests, it's their
		// problem, it will still work. We just don't want *us* making complicated
		// requests without a very strong reason to.
		panic(fmt.Sprintf("invalid header retrieval: reverse %v, want true; skip %d, want 0", reverse, skip))
	}
	// If the skeleton syncer requests the genesis block, panic. Whilst it could
	// be considered a valid request, our code specifically should not request it
	// ever since we want to link up headers to an existing local chain, which at
	// worse will be the genesis.
	if int64(origin)-int64(amount) < 0 {
		panic(fmt.Sprintf("headers requested before (or at) genesis: origin %d, amount %d", origin, amount))
	}
	// To make concurrency easier, the skeleton syncer always requests fixed size
	// batches of headers. Panic if the peer is requested an amount other than the
	// configured batch size (apart from the request leading to the genesis).
	if amount > requestHeaders || (amount < requestHeaders && origin > uint64(amount)) {
		panic(fmt.Sprintf("non-chunk size header batch requested: requested %d, want %d, origin %d", amount, requestHeaders, origin))
	}
	// Simple reverse header retrieval. Fill from the peer's chain and return.
	// If the tester has a serve hook set, try to use that before falling back
	// to the default behavior.
	var headers []*types.Header
	if p.serve != nil {
		headers = p.serve(origin)
	}
	if headers == nil {
		headers = make([]*types.Header, 0, amount)
		if len(p.headers) > int(origin) { // Don't serve headers if we're missing the origin
			for i := 0; i < amount; i++ {
				// Consider nil headers as a form of attack and withhold them. Nil
				// cannot be decoded from RLP, so it's not possible to produce an
				// attack by sending/receiving those over eth.
				header := p.headers[int(origin)-i]
				if header == nil {
					continue
				}
				headers = append(headers, header)
			}
		}
	}
	p.served.Add(uint64(len(headers)))

	hashes := make([]common.Hash, len(headers))
	for i, header := range headers {
		hashes[i] = header.Hash()
	}
	// Deliver the headers to the downloader
	req := &eth.Request{
		Peer: p.id,
	}
	res := &eth.Response{
		Req:  req,
		Res:  (*eth.BlockHeadersRequest)(&headers),
		Meta: hashes,
		Time: 1,
		Done: make(chan error),
	}
	go func() {
		sink <- res
		if err := <-res.Done; err != nil {
			log.Warn("Skeleton test peer response rejected", "err", err)
			p.dropped.Add(1)
		}
	}()
	return req, nil
}

func (p *skeletonTestPeer) Head() (common.Hash, *big.Int) {
	panic("skeleton sync must not request the remote head")
}

func (p *skeletonTestPeer) RequestHeadersByHash(common.Hash, int, int, bool, chan *eth.Response) (*eth.Request, error) {
	panic("skeleton sync must not request headers by hash")
}

func (p *skeletonTestPeer) RequestBodies([]common.Hash, chan *eth.Response) (*eth.Request, error) {
	panic("skeleton sync must not request block bodies")
}

func (p *skeletonTestPeer) RequestReceipts([]common.Hash, chan *eth.Response) (*eth.Request, error) {
	panic("skeleton sync must not request receipts")
}

// Tests various sync initializations based on previous leftovers in the database
// and announced heads.
func TestSkeletonSyncInit(t *testing.T) {
	// Create a few key headers
	var (
		genesis  = &types.Header{Number: big.NewInt(0)}
		block49  = &types.Header{Number: big.NewInt(49)}
		block49B = &types.Header{Number: big.NewInt(49), Extra: []byte("B")}
		block50  = &types.Header{Number: big.NewInt(50), ParentHash: block49.Hash()}
	)
	tests := []struct {
		headers  []*types.Header // Database content (beside the genesis)
		oldstate []*subchain     // Old sync state with various interrupted subchains
		head     *types.Header   // New head header to announce to reorg to
		newstate []*subchain     // Expected sync state after the reorg
	}{
		// Completely empty database with only the genesis set. The sync is expected
		// to create a single subchain with the requested head.
		{
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 50}},
		},
		// Empty database with only the genesis set with a leftover empty sync
		// progress. This is a synthetic case, just for the sake of covering things.
		{
			oldstate: []*subchain{},
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 50}},
		},
		// A single leftover subchain is present, older than the new head. The
		// old subchain should be left as is and a new one appended to the sync
		// status.
		{
			oldstate: []*subchain{{Head: 10, Tail: 5}},
			head:     block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 10, Tail: 5},
			},
		},
		// Multiple leftover subchains are present, older than the new head. The
		// old subchains should be left as is and a new one appended to the sync
		// status.
		{
			oldstate: []*subchain{
				{Head: 20, Tail: 15},
				{Head: 10, Tail: 5},
			},
			head: block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 20, Tail: 15},
				{Head: 10, Tail: 5},
			},
		},
		// A single leftover subchain is present, newer than the new head. The
		// newer subchain should be deleted and a fresh one created for the head.
		{
			oldstate: []*subchain{{Head: 65, Tail: 60}},
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 50}},
		},
		// Multiple leftover subchain is present, newer than the new head. The
		// newer subchains should be deleted and a fresh one created for the head.
		{
			oldstate: []*subchain{
				{Head: 75, Tail: 70},
				{Head: 65, Tail: 60},
			},
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 50}},
		},

		// Two leftover subchains are present, one fully older and one fully
		// newer than the announced head. The head should delete the newer one,
		// keeping the older one.
		{
			oldstate: []*subchain{
				{Head: 65, Tail: 60},
				{Head: 10, Tail: 5},
			},
			head: block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 10, Tail: 5},
			},
		},
		// Multiple leftover subchains are present, some fully older and some
		// fully newer than the announced head. The head should delete the newer
		// ones, keeping the older ones.
		{
			oldstate: []*subchain{
				{Head: 75, Tail: 70},
				{Head: 65, Tail: 60},
				{Head: 20, Tail: 15},
				{Head: 10, Tail: 5},
			},
			head: block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 20, Tail: 15},
				{Head: 10, Tail: 5},
			},
		},
		// A single leftover subchain is present and the new head is extending
		// it with one more header. We expect the subchain head to be pushed
		// forward.
		{
			headers:  []*types.Header{block49},
			oldstate: []*subchain{{Head: 49, Tail: 5}},
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 5}},
		},
		// A single leftover subchain is present and although the new head does
		// extend it number wise, the hash chain does not link up. We expect a
		// new subchain to be created for the dangling head.
		{
			headers:  []*types.Header{block49B},
			oldstate: []*subchain{{Head: 49, Tail: 5}},
			head:     block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 49, Tail: 5},
			},
		},
		// A single leftover subchain is present. A new head is announced that
		// links into the middle of it, correctly anchoring into an existing
		// header. We expect the old subchain to be truncated and extended with
		// the new head.
		{
			headers:  []*types.Header{block49},
			oldstate: []*subchain{{Head: 100, Tail: 5}},
			head:     block50,
			newstate: []*subchain{{Head: 50, Tail: 5}},
		},
		// A single leftover subchain is present. A new head is announced that
		// links into the middle of it, but does not anchor into an existing
		// header. We expect the old subchain to be truncated and a new chain
		// be created for the dangling head.
		{
			headers:  []*types.Header{block49B},
			oldstate: []*subchain{{Head: 100, Tail: 5}},
			head:     block50,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
				{Head: 49, Tail: 5},
			},
		},
	}
	for i, tt := range tests {
		// Create a fresh database and initialize it with the starting state
		db := rawdb.NewMemoryDatabase()

		rawdb.WriteHeader(db, genesis)
		for _, header := range tt.headers {
			rawdb.WriteSkeletonHeader(db, header)
		}
		if tt.oldstate != nil {
			blob, _ := json.Marshal(&skeletonProgress{Subchains: tt.oldstate})
			rawdb.WriteSkeletonSyncStatus(db, blob)
		}
		// Create a skeleton sync and run a cycle
		wait := make(chan struct{})

		skeleton := newSkeleton(db, newPeerSet(), nil, newHookedBackfiller())
		skeleton.syncStarting = func() { close(wait) }
		skeleton.Sync(tt.head, nil, true)

		<-wait
		skeleton.Terminate()

		// Ensure the correct resulting sync status
		var progress skeletonProgress
		json.Unmarshal(rawdb.ReadSkeletonSyncStatus(db), &progress)

		if len(progress.Subchains) != len(tt.newstate) {
			t.Errorf("test %d: subchain count mismatch: have %d, want %d", i, len(progress.Subchains), len(tt.newstate))
			continue
		}
		for j := 0; j < len(progress.Subchains); j++ {
			if progress.Subchains[j].Head != tt.newstate[j].Head {
				t.Errorf("test %d: subchain %d head mismatch: have %d, want %d", i, j, progress.Subchains[j].Head, tt.newstate[j].Head)
			}
			if progress.Subchains[j].Tail != tt.newstate[j].Tail {
				t.Errorf("test %d: subchain %d tail mismatch: have %d, want %d", i, j, progress.Subchains[j].Tail, tt.newstate[j].Tail)
			}
		}
	}
}

// Tests that a running skeleton sync can be extended with properly linked up
// headers but not with side chains.
func TestSkeletonSyncExtend(t *testing.T) {
	// Create a few key headers
	var (
		genesis  = &types.Header{Number: big.NewInt(0)}
		block49  = &types.Header{Number: big.NewInt(49)}
		block49B = &types.Header{Number: big.NewInt(49), Extra: []byte("B")}
		block50  = &types.Header{Number: big.NewInt(50), ParentHash: block49.Hash()}
		block51  = &types.Header{Number: big.NewInt(51), ParentHash: block50.Hash()}
	)
	tests := []struct {
		head     *types.Header // New head header to announce to reorg to
		extend   *types.Header // New head header to announce to extend with
		newstate []*subchain   // Expected sync state after the reorg
		err      error         // Whether extension succeeds or not
	}{
		// Initialize a sync and try to extend it with a subsequent block.
		{
			head:   block49,
			extend: block50,
			newstate: []*subchain{
				{Head: 50, Tail: 49},
			},
		},
		// Initialize a sync and try to extend it with the existing head block.
		{
			head:   block49,
			extend: block49,
			newstate: []*subchain{
				{Head: 49, Tail: 49},
			},
		},
		// Initialize a sync and try to extend it with a sibling block.
		{
			head:   block49,
			extend: block49B,
			newstate: []*subchain{
				{Head: 49, Tail: 49},
			},
			err: errReorgDenied,
		},
		// Initialize a sync and try to extend it with a number-wise sequential
		// header, but a hash wise non-linking one.
		{
			head:   block49B,
			extend: block50,
			newstate: []*subchain{
				{Head: 49, Tail: 49},
			},
			err: errReorgDenied,
		},
		// Initialize a sync and try to extend it with a non-linking future block.
		{
			head:   block49,
			extend: block51,
			newstate: []*subchain{
				{Head: 49, Tail: 49},
			},
			err: errReorgDenied,
		},
		// Initialize a sync and try to extend it with a past canonical block.
		{
			head:   block50,
			extend: block49,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
			},
			err: errReorgDenied,
		},
		// Initialize a sync and try to extend it with a past sidechain block.
		{
			head:   block50,
			extend: block49B,
			newstate: []*subchain{
				{Head: 50, Tail: 50},
			},
			err: errReorgDenied,
		},
	}
	for i, tt := range tests {
		// Create a fresh database and initialize it with the starting state
		db := rawdb.NewMemoryDatabase()
		rawdb.WriteHeader(db, genesis)

		// Create a skeleton sync and run a cycle
		wait := make(chan struct{})

		skeleton := newSkeleton(db, newPeerSet(), nil, newHookedBackfiller())
		skeleton.syncStarting = func() { close(wait) }
		skeleton.Sync(tt.head, nil, true)

		<-wait
		if err := skeleton.Sync(tt.extend, nil, false); err != tt.err {
			t.Errorf("test %d: extension failure mismatch: have %v, want %v", i, err, tt.err)
		}
		skeleton.Terminate()

		// Ensure the correct resulting sync status
		var progress skeletonProgress
		json.Unmarshal(rawdb.ReadSkeletonSyncStatus(db), &progress)

		if len(progress.Subchains) != len(tt.newstate) {
			t.Errorf("test %d: subchain count mismatch: have %d, want %d", i, len(progress.Subchains), len(tt.newstate))
			continue
		}
		for j := 0; j < len(progress.Subchains); j++ {
			if progress.Subchains[j].Head != tt.newstate[j].Head {
				t.Errorf("test %d: subchain %d head mismatch: have %d, want %d", i, j, progress.Subchains[j].Head, tt.newstate[j].Head)
			}
			if progress.Subchains[j].Tail != tt.newstate[j].Tail {
				t.Errorf("test %d: subchain %d tail mismatch: have %d, want %d", i, j, progress.Subchains[j].Tail, tt.newstate[j].Tail)
			}
		}
	}
}

// Tests that the skeleton sync correctly retrieves headers from one or more
// peers without duplicates or other strange side effects.
func TestSkeletonSyncRetrievals(t *testing.T) {
	//log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(true))))

	// Since skeleton headers don't need to be meaningful, beyond a parent hash
	// progression, create a long fake chain to test with.
	chain := []*types.Header{{Number: big.NewInt(0)}}
	for i := 1; i < 10000; i++ {
		chain = append(chain, &types.Header{
			ParentHash: chain[i-1].Hash(),
			Number:     big.NewInt(int64(i)),
		})
	}
	// Some tests require a forking side chain to trigger cornercases.
	var sidechain []*types.Header
	for i := 0; i < len(chain)/2; i++ { // Fork at block #5000
		sidechain = append(sidechain, chain[i])
	}
	for i := len(chain) / 2; i < len(chain); i++ {
		sidechain = append(sidechain, &types.Header{
			ParentHash: sidechain[i-1].Hash(),
			Number:     big.NewInt(int64(i)),
			Extra:      []byte("B"), // force a different hash
		})
	}
	tests := []struct {
		fill          bool // Whether to run a real backfiller in this test case
		unpredictable bool // Whether to ignore drops/serves due to uncertain packet assignments

		head     *types.Header       // New head header to announce to reorg to
		peers    []*skeletonTestPeer // Initial peer set to start the sync with
		midstate []*subchain         // Expected sync state after initial cycle
		midserve uint64              // Expected number of header retrievals after initial cycle
		middrop  uint64              // Expected number of peers dropped after initial cycle

		newHead  *types.Header     // New header to anoint on top of the old one
		newPeer  *skeletonTestPeer // New peer to join the skeleton syncer
		endstate []*subchain       // Expected sync state after the post-init event
		endserve uint64            // Expected number of header retrievals after the post-init event
		enddrop  uint64            // Expected number of peers dropped after the post-init event
	}{
		// Completely empty database with only the genesis set. The sync is expected
		// to create a single subchain with the requested head. No peers however, so
		// the sync should be stuck without any progression.
		//
		// When a new peer is added, it should detect the join and fill the headers
		// to the genesis block.
		{
			head:     chain[len(chain)-1],
			midstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: uint64(len(chain) - 1)}},

			newPeer:  newSkeletonTestPeer("test-peer", chain),
			endstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: 1}},
			endserve: uint64(len(chain) - 2), // len - head - genesis
		},
		// Completely empty database with only the genesis set. The sync is expected
		// to create a single subchain with the requested head. With one valid peer,
		// the sync is expected to complete already in the initial round.
		//
		// Adding a second peer should not have any effect.
		{
			head:     chain[len(chain)-1],
			peers:    []*skeletonTestPeer{newSkeletonTestPeer("test-peer-1", chain)},
			midstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: 1}},
			midserve: uint64(len(chain) - 2), // len - head - genesis

			newPeer:  newSkeletonTestPeer("test-peer-2", chain),
			endstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: 1}},
			endserve: uint64(len(chain) - 2), // len - head - genesis
		},
		// Completely empty database with only the genesis set. The sync is expected
		// to create a single subchain with the requested head. With many valid peers,
		// the sync is expected to complete already in the initial round.
		//
		// Adding a new peer should not have any effect.
		{
			head: chain[len(chain)-1],
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("test-peer-1", chain),
				newSkeletonTestPeer("test-peer-2", chain),
				newSkeletonTestPeer("test-peer-3", chain),
			},
			midstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: 1}},
			midserve: uint64(len(chain) - 2), // len - head - genesis

			newPeer:  newSkeletonTestPeer("test-peer-4", chain),
			endstate: []*subchain{{Head: uint64(len(chain) - 1), Tail: 1}},
			endserve: uint64(len(chain) - 2), // len - head - genesis
		},
		// This test checks if a peer tries to withhold a header - *on* the sync
		// boundary - instead of sending the requested amount. The malicious short
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100],
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-skipper", append(append(append([]*types.Header{}, chain[:99]...), nil), chain[100:]...)),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 3, // len - head - genesis - missing
			middrop:  1,                        // penalize shortened header deliveries

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 3) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test checks if a peer tries to withhold a header - *off* the sync
		// boundary - instead of sending the requested amount. The malicious short
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100],
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-skipper", append(append(append([]*types.Header{}, chain[:50]...), nil), chain[51:]...)),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 3, // len - head - genesis - missing
			middrop:  1,                        // penalize shortened header deliveries

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 3) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test checks if a peer tries to duplicate a header - *on* the sync
		// boundary - instead of sending the correct sequence. The malicious duped
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100], // We want to force the 100th header to be a request boundary
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-duper", append(append(append([]*types.Header{}, chain[:99]...), chain[98]), chain[100:]...)),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 2, // len - head - genesis
			middrop:  1,                        // penalize invalid header sequences

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 2) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test checks if a peer tries to duplicate a header - *off* the sync
		// boundary - instead of sending the correct sequence. The malicious duped
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100], // We want to force the 100th header to be a request boundary
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-duper", append(append(append([]*types.Header{}, chain[:50]...), chain[49]), chain[51:]...)),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 2, // len - head - genesis
			middrop:  1,                        // penalize invalid header sequences

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 2) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test checks if a peer tries to inject a different header - *on*
		// the sync boundary - instead of sending the correct sequence. The bad
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100], // We want to force the 100th header to be a request boundary
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-changer",
					append(
						append(
							append([]*types.Header{}, chain[:99]...),
							&types.Header{
								ParentHash: chain[98].Hash(),
								Number:     big.NewInt(int64(99)),
								GasLimit:   1,
							},
						), chain[100:]...,
					),
				),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 2, // len - head - genesis
			middrop:  1,                        // different set of headers, drop // TODO(karalabe): maybe just diff sync?

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 2) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test checks if a peer tries to inject a different header - *off*
		// the sync boundary - instead of sending the correct sequence. The bad
		// package should not be accepted.
		//
		// Joining with a new peer should however unblock the sync.
		{
			head: chain[requestHeaders+100], // We want to force the 100th header to be a request boundary
			peers: []*skeletonTestPeer{
				newSkeletonTestPeer("header-changer",
					append(
						append(
							append([]*types.Header{}, chain[:50]...),
							&types.Header{
								ParentHash: chain[49].Hash(),
								Number:     big.NewInt(int64(50)),
								GasLimit:   1,
							},
						), chain[51:]...,
					),
				),
			},
			midstate: []*subchain{{Head: requestHeaders + 100, Tail: 100}},
			midserve: requestHeaders + 101 - 2, // len - head - genesis
			middrop:  1,                        // different set of headers, drop

			newPeer:  newSkeletonTestPeer("good-peer", chain),
			endstate: []*subchain{{Head: requestHeaders + 100, Tail: 1}},
			endserve: (requestHeaders + 101 - 2) + (100 - 1), // midserve + lenrest - genesis
			enddrop:  1,                                      // no new drops
		},
		// This test reproduces a bug caught during review (kudos to @holiman)
		// where a subchain is merged with a previously interrupted one, causing
		// pending data in the scratch space to become "invalid" (since we jump
		// ahead during subchain merge). In that case it is expected to ignore
		// the queued up data instead of trying to process on top of a shifted
		// task set.
		//
		// The test is a bit convoluted since it needs to trigger a concurrency
		// issue. First we sync up an initial chain of 2x512 items. Then announce
		// 2x512+2 as head and delay delivering the head batch to fill the scratch
		// space first. The delivery head should merge with the previous download
		// and the scratch space must not be consumed further.
		{
			head: chain[2*requestHeaders],
			peers: []*skeletonTestPeer{
				newSkeletonTestPeerWithHook("peer-1", chain, func(origin uint64) []*types.Header {
					if origin == chain[2*requestHeaders+1].Number.Uint64() {
						time.Sleep(100 * time.Millisecond)
					}
					return nil // Fallback to default behavior, just delayed
				}),
				newSkeletonTestPeerWithHook("peer-2", chain, func(origin uint64) []*types.Header {
					if origin == chain[2*requestHeaders+1].Number.Uint64() {
						time.Sleep(100 * time.Millisecond)
					}
					return nil // Fallback to default behavior, just delayed
				}),
			},
			midstate: []*subchain{{Head: 2 * requestHeaders, Tail: 1}},
			midserve: 2*requestHeaders - 1, // len - head - genesis

			newHead:  chain[2*requestHeaders+2],
			endstate: []*subchain{{Head: 2*requestHeaders + 2, Tail: 1}},
			endserve: 4 * requestHeaders,
		},
		// This test reproduces a bug caught by (@rjl493456442) where a skeleton
		// header goes missing, causing the sync to get stuck and/or panic.
		//
		// The setup requires a previously successfully synced chain up to a block
		// height N. That results is a single skeleton header (block N) and a single
		// subchain (head N, Tail N) being stored on disk.
		//
		// The following step requires a new sync cycle to a new side chain of a
		// height higher than N, and an ancestor lower than N (e.g. N-2, N+2).
		// In this scenario, when processing a batch of headers, a link point of
		// N-2 will be found, meaning that N-1 and N have been overwritten.
		//
		// The link event triggers an early exit, noticing that the previous sub-
		// chain is a leftover and deletes it (with it's skeleton header N). But
		// since skeleton header N has been overwritten to the new side chain, we
		// end up losing it and creating a gap.
		{
			fill:          true,
			unpredictable: true, // We have good and bad peer too, bad may be dropped, test too short for certainty

			head:     chain[len(chain)/2+1], // Sync up until the sidechain common ancestor + 2
			peers:    []*skeletonTestPeer{newSkeletonTestPeer("test-peer-oldchain", chain)},
			midstate: []*subchain{{Head: uint64(len(chain)/2 + 1), Tail: 1}},

			newHead:  sidechain[len(sidechain)/2+3], // Sync up until the sidechain common ancestor + 4
			newPeer:  newSkeletonTestPeer("test-peer-newchain", sidechain),
			endstate: []*subchain{{Head: uint64(len(sidechain)/2 + 3), Tail: uint64(len(chain) / 2)}},
		},
	}
	for i, tt := range tests {
		// Create a fresh database and initialize it with the starting state
		db := rawdb.NewMemoryDatabase()

		rawdb.WriteBlock(db, types.NewBlockWithHeader(chain[0]))
		rawdb.WriteReceipts(db, chain[0].Hash(), chain[0].Number.Uint64(), types.Receipts{})

		// Create a peer set to feed headers through
		peerset := newPeerSet()
		for _, peer := range tt.peers {
			peerset.Register(newPeerConnection(peer.id, eth.ETH67, peer, log.New("id", peer.id)))
		}
		// Create a peer dropper to track malicious peers
		dropped := make(map[string]int)
		drop := func(peer string) {
			if p := peerset.Peer(peer); p != nil {
				p.peer.(*skeletonTestPeer).dropped.Add(1)
			}
			peerset.Unregister(peer)
			dropped[peer]++
		}
		// Create a backfiller if we need to run more advanced tests
		filler := newHookedBackfiller()
		if tt.fill {
			var filled *types.Header

			filler = &hookedBackfiller{
				resumeHook: func() {
					var progress skeletonProgress
					json.Unmarshal(rawdb.ReadSkeletonSyncStatus(db), &progress)

					for progress.Subchains[0].Tail < progress.Subchains[0].Head {
						header := rawdb.ReadSkeletonHeader(db, progress.Subchains[0].Tail)

						rawdb.WriteBlock(db, types.NewBlockWithHeader(header))
						rawdb.WriteReceipts(db, header.Hash(), header.Number.Uint64(), types.Receipts{})

						rawdb.DeleteSkeletonHeader(db, header.Number.Uint64())

						progress.Subchains[0].Tail++
						progress.Subchains[0].Next = header.Hash()
					}
					filled = rawdb.ReadSkeletonHeader(db, progress.Subchains[0].Tail)

					rawdb.WriteBlock(db, types.NewBlockWithHeader(filled))
					rawdb.WriteReceipts(db, filled.Hash(), filled.Number.Uint64(), types.Receipts{})
				},

				suspendHook: func() *types.Header {
					prev := filled
					filled = nil

					return prev
				},
			}
		}
		// Create a skeleton sync and run a cycle
		skeleton := newSkeleton(db, peerset, drop, filler)
		skeleton.Sync(tt.head, nil, true)

		var progress skeletonProgress
		// Wait a bit (bleah) for the initial sync loop to go to idle. This might
		// be either a finish or a never-start hence why there's no event to hook.
		check := func() error {
			if len(progress.Subchains) != len(tt.midstate) {
				return fmt.Errorf("test %d, mid state: subchain count mismatch: have %d, want %d", i, len(progress.Subchains), len(tt.midstate))
			}
			for j := 0; j < len(progress.Subchains); j++ {
				if progress.Subchains[j].Head != tt.midstate[j].Head {
					return fmt.Errorf("test %d, mid state: subchain %d head mismatch: have %d, want %d", i, j, progress.Subchains[j].Head, tt.midstate[j].Head)
				}
				if progress.Subchains[j].Tail != tt.midstate[j].Tail {
					return fmt.Errorf("test %d, mid state: subchain %d tail mismatch: have %d, want %d", i, j, progress.Subchains[j].Tail, tt.midstate[j].Tail)
				}
			}
			return nil
		}

		waitStart := time.Now()
		for waitTime := 20 * time.Millisecond; time.Since(waitStart) < 2*time.Second; waitTime = waitTime * 2 {
			time.Sleep(waitTime)
			// Check the post-init end state if it matches the required results
			json.Unmarshal(rawdb.ReadSkeletonSyncStatus(db), &progress)
			if err := check(); err == nil {
				break
			}
		}
		if err := check(); err != nil {
			t.Error(err)
			continue
		}
		if !tt.unpredictable {
			var served uint64
			for _, peer := range tt.peers {
				served += peer.served.Load()
			}
			if served != tt.midserve {
				t.Errorf("test %d, mid state: served headers mismatch: have %d, want %d", i, served, tt.midserve)
			}
			var drops uint64
			for _, peer := range tt.peers {
				drops += peer.dropped.Load()
			}
			if drops != tt.middrop {
				t.Errorf("test %d, mid state: dropped peers mismatch: have %d, want %d", i, drops, tt.middrop)
			}
		}
		// Apply the post-init events if there's any
		if tt.newHead != nil {
			skeleton.Sync(tt.newHead, nil, true)
		}
		if tt.newPeer != nil {
			if err := peerset.Register(newPeerConnection(tt.newPeer.id, eth.ETH67, tt.newPeer, log.New("id", tt.newPeer.id))); err != nil {
				t.Errorf("test %d: failed to register new peer: %v", i, err)
			}
		}
		// Wait a bit (bleah) for the second sync loop to go to idle. This might
		// be either a finish or a never-start hence why there's no event to hook.
		check = func() error {
			if len(progress.Subchains) != len(tt.endstate) {
				return fmt.Errorf("test %d, end state: subchain count mismatch: have %d, want %d", i, len(progress.Subchains), len(tt.endstate))
			}
			for j := 0; j < len(progress.Subchains); j++ {
				if progress.Subchains[j].Head != tt.endstate[j].Head {
					return fmt.Errorf("test %d, end state: subchain %d head mismatch: have %d, want %d", i, j, progress.Subchains[j].Head, tt.endstate[j].Head)
				}
				if progress.Subchains[j].Tail != tt.endstate[j].Tail {
					return fmt.Errorf("test %d, end state: subchain %d tail mismatch: have %d, want %d", i, j, progress.Subchains[j].Tail, tt.endstate[j].Tail)
				}
			}
			return nil
		}
		waitStart = time.Now()
		for waitTime := 20 * time.Millisecond; time.Since(waitStart) < 2*time.Second; waitTime = waitTime * 2 {
			time.Sleep(waitTime)
			// Check the post-init end state if it matches the required results
			json.Unmarshal(rawdb.ReadSkeletonSyncStatus(db), &progress)
			if err := check(); err == nil {
				break
			}
		}
		if err := check(); err != nil {
			t.Error(err)
			continue
		}
		// Check that the peers served no more headers than we actually needed
		if !tt.unpredictable {
			served := uint64(0)
			for _, peer := range tt.peers {
				served += peer.served.Load()
			}
			if tt.newPeer != nil {
				served += tt.newPeer.served.Load()
			}
			if served != tt.endserve {
				t.Errorf("test %d, end state: served headers mismatch: have %d, want %d", i, served, tt.endserve)
			}
			drops := uint64(0)
			for _, peer := range tt.peers {
				drops += peer.dropped.Load()
			}
			if tt.newPeer != nil {
				drops += tt.newPeer.dropped.Load()
			}
			if drops != tt.enddrop {
				t.Errorf("test %d, end state: dropped peers mismatch: have %d, want %d", i, drops, tt.middrop)
			}
		}
		// Clean up any leftover skeleton sync resources
		skeleton.Terminate()
	}
}
