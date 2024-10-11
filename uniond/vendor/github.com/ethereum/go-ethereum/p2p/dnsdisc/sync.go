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

package dnsdisc

import (
	"context"
	"math/rand"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

// This is the number of consecutive leaf requests that may fail before
// we consider re-resolving the tree root.
const rootRecheckFailCount = 5

// clientTree is a full tree being synced.
type clientTree struct {
	c   *Client
	loc *linkEntry // link to this tree

	lastRootCheck mclock.AbsTime // last revalidation of root
	leafFailCount int
	rootFailCount int

	root  *rootEntry
	enrs  *subtreeSync
	links *subtreeSync

	lc         *linkCache          // tracks all links between all trees
	curLinks   map[string]struct{} // links contained in this tree
	linkGCRoot string              // root on which last link GC has run
}

func newClientTree(c *Client, lc *linkCache, loc *linkEntry) *clientTree {
	return &clientTree{c: c, lc: lc, loc: loc}
}

// syncAll retrieves all entries of the tree.
func (ct *clientTree) syncAll(dest map[string]entry) error {
	if err := ct.updateRoot(context.Background()); err != nil {
		return err
	}
	if err := ct.links.resolveAll(dest); err != nil {
		return err
	}
	if err := ct.enrs.resolveAll(dest); err != nil {
		return err
	}
	return nil
}

// syncRandom retrieves a single entry of the tree. The Node return value
// is non-nil if the entry was a node.
func (ct *clientTree) syncRandom(ctx context.Context) (n *enode.Node, err error) {
	if ct.rootUpdateDue() {
		if err := ct.updateRoot(ctx); err != nil {
			return nil, err
		}
	}

	// Update fail counter for leaf request errors.
	defer func() {
		if err != nil {
			ct.leafFailCount++
		}
	}()

	// Link tree sync has priority, run it to completion before syncing ENRs.
	if !ct.links.done() {
		err := ct.syncNextLink(ctx)
		return nil, err
	}
	ct.gcLinks()

	// Sync next random entry in ENR tree. Once every node has been visited, we simply
	// start over. This is fine because entries are cached internally by the client LRU
	// also by DNS resolvers.
	if ct.enrs.done() {
		ct.enrs = newSubtreeSync(ct.c, ct.loc, ct.root.eroot, false)
	}
	return ct.syncNextRandomENR(ctx)
}

// canSyncRandom checks if any meaningful action can be performed by syncRandom.
func (ct *clientTree) canSyncRandom() bool {
	// Note: the check for non-zero leaf count is very important here.
	// If we're done syncing all nodes, and no leaves were found, the tree
	// is empty and we can't use it for sync.
	return ct.rootUpdateDue() || !ct.links.done() || !ct.enrs.done() || ct.enrs.leaves != 0
}

// gcLinks removes outdated links from the global link cache. GC runs once
// when the link sync finishes.
func (ct *clientTree) gcLinks() {
	if !ct.links.done() || ct.root.lroot == ct.linkGCRoot {
		return
	}
	ct.lc.resetLinks(ct.loc.str, ct.curLinks)
	ct.linkGCRoot = ct.root.lroot
}

func (ct *clientTree) syncNextLink(ctx context.Context) error {
	hash := ct.links.missing[0]
	e, err := ct.links.resolveNext(ctx, hash)
	if err != nil {
		return err
	}
	ct.links.missing = ct.links.missing[1:]

	if dest, ok := e.(*linkEntry); ok {
		ct.lc.addLink(ct.loc.str, dest.str)
		ct.curLinks[dest.str] = struct{}{}
	}
	return nil
}

func (ct *clientTree) syncNextRandomENR(ctx context.Context) (*enode.Node, error) {
	index := rand.Intn(len(ct.enrs.missing))
	hash := ct.enrs.missing[index]
	e, err := ct.enrs.resolveNext(ctx, hash)
	if err != nil {
		return nil, err
	}
	ct.enrs.missing = removeHash(ct.enrs.missing, index)
	if ee, ok := e.(*enrEntry); ok {
		return ee.node, nil
	}
	return nil, nil
}

func (ct *clientTree) String() string {
	return ct.loc.String()
}

// removeHash removes the element at index from h.
func removeHash(h []string, index int) []string {
	if len(h) == 1 {
		return nil
	}
	last := len(h) - 1
	if index < last {
		h[index] = h[last]
		h[last] = ""
	}
	return h[:last]
}

// updateRoot ensures that the given tree has an up-to-date root.
func (ct *clientTree) updateRoot(ctx context.Context) error {
	if !ct.slowdownRootUpdate(ctx) {
		return ctx.Err()
	}

	ct.lastRootCheck = ct.c.clock.Now()
	ctx, cancel := context.WithTimeout(ctx, ct.c.cfg.Timeout)
	defer cancel()
	root, err := ct.c.resolveRoot(ctx, ct.loc)
	if err != nil {
		ct.rootFailCount++
		return err
	}
	ct.root = &root
	ct.rootFailCount = 0
	ct.leafFailCount = 0

	// Invalidate subtrees if changed.
	if ct.links == nil || root.lroot != ct.links.root {
		ct.links = newSubtreeSync(ct.c, ct.loc, root.lroot, true)
		ct.curLinks = make(map[string]struct{})
	}
	if ct.enrs == nil || root.eroot != ct.enrs.root {
		ct.enrs = newSubtreeSync(ct.c, ct.loc, root.eroot, false)
	}
	return nil
}

// rootUpdateDue returns true when a root update is needed.
func (ct *clientTree) rootUpdateDue() bool {
	tooManyFailures := ct.leafFailCount > rootRecheckFailCount
	scheduledCheck := ct.c.clock.Now() >= ct.nextScheduledRootCheck()
	return ct.root == nil || tooManyFailures || scheduledCheck
}

func (ct *clientTree) nextScheduledRootCheck() mclock.AbsTime {
	return ct.lastRootCheck.Add(ct.c.cfg.RecheckInterval)
}

// slowdownRootUpdate applies a delay to root resolution if is tried
// too frequently. This avoids busy polling when the client is offline.
// Returns true if the timeout passed, false if sync was canceled.
func (ct *clientTree) slowdownRootUpdate(ctx context.Context) bool {
	var delay time.Duration
	switch {
	case ct.rootFailCount > 20:
		delay = 10 * time.Second
	case ct.rootFailCount > 5:
		delay = 5 * time.Second
	default:
		return true
	}
	timeout := ct.c.clock.NewTimer(delay)
	defer timeout.Stop()
	select {
	case <-timeout.C():
		return true
	case <-ctx.Done():
		return false
	}
}

// subtreeSync is the sync of an ENR or link subtree.
type subtreeSync struct {
	c       *Client
	loc     *linkEntry
	root    string
	missing []string // missing tree node hashes
	link    bool     // true if this sync is for the link tree
	leaves  int      // counter of synced leaves
}

func newSubtreeSync(c *Client, loc *linkEntry, root string, link bool) *subtreeSync {
	return &subtreeSync{c, loc, root, []string{root}, link, 0}
}

func (ts *subtreeSync) done() bool {
	return len(ts.missing) == 0
}

func (ts *subtreeSync) resolveAll(dest map[string]entry) error {
	for !ts.done() {
		hash := ts.missing[0]
		ctx, cancel := context.WithTimeout(context.Background(), ts.c.cfg.Timeout)
		e, err := ts.resolveNext(ctx, hash)
		cancel()
		if err != nil {
			return err
		}
		dest[hash] = e
		ts.missing = ts.missing[1:]
	}
	return nil
}

func (ts *subtreeSync) resolveNext(ctx context.Context, hash string) (entry, error) {
	e, err := ts.c.resolveEntry(ctx, ts.loc.domain, hash)
	if err != nil {
		return nil, err
	}
	switch e := e.(type) {
	case *enrEntry:
		if ts.link {
			return nil, errENRInLinkTree
		}
		ts.leaves++
	case *linkEntry:
		if !ts.link {
			return nil, errLinkInENRTree
		}
		ts.leaves++
	case *branchEntry:
		ts.missing = append(ts.missing, e.children...)
	}
	return e, nil
}

// linkCache tracks links between trees.
type linkCache struct {
	backrefs map[string]map[string]struct{}
	changed  bool
}

func (lc *linkCache) isReferenced(r string) bool {
	return len(lc.backrefs[r]) != 0
}

func (lc *linkCache) addLink(from, to string) {
	if _, ok := lc.backrefs[to][from]; ok {
		return
	}

	if lc.backrefs == nil {
		lc.backrefs = make(map[string]map[string]struct{})
	}
	if _, ok := lc.backrefs[to]; !ok {
		lc.backrefs[to] = make(map[string]struct{})
	}
	lc.backrefs[to][from] = struct{}{}
	lc.changed = true
}

// resetLinks clears all links of the given tree.
func (lc *linkCache) resetLinks(from string, keep map[string]struct{}) {
	stk := []string{from}
	for len(stk) > 0 {
		item := stk[len(stk)-1]
		stk = stk[:len(stk)-1]

		for r, refs := range lc.backrefs {
			if _, ok := keep[r]; ok {
				continue
			}
			if _, ok := refs[item]; !ok {
				continue
			}
			lc.changed = true
			delete(refs, item)
			if len(refs) == 0 {
				delete(lc.backrefs, r)
				stk = append(stk, r)
			}
		}
	}
}
