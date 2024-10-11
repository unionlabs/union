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
	"bytes"
	"context"
	"errors"
	"fmt"
	"math/rand"
	"net"
	"strings"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/lru"
	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"golang.org/x/sync/singleflight"
	"golang.org/x/time/rate"
)

// Client discovers nodes by querying DNS servers.
type Client struct {
	cfg          Config
	clock        mclock.Clock
	entries      *lru.Cache[string, entry]
	ratelimit    *rate.Limiter
	singleflight singleflight.Group
}

// Config holds configuration options for the client.
type Config struct {
	Timeout         time.Duration      // timeout used for DNS lookups (default 5s)
	RecheckInterval time.Duration      // time between tree root update checks (default 30min)
	CacheLimit      int                // maximum number of cached records (default 1000)
	RateLimit       float64            // maximum DNS requests / second (default 3)
	ValidSchemes    enr.IdentityScheme // acceptable ENR identity schemes (default enode.ValidSchemes)
	Resolver        Resolver           // the DNS resolver to use (defaults to system DNS)
	Logger          log.Logger         // destination of client log messages (defaults to root logger)
}

// Resolver is a DNS resolver that can query TXT records.
type Resolver interface {
	LookupTXT(ctx context.Context, domain string) ([]string, error)
}

func (cfg Config) withDefaults() Config {
	const (
		defaultTimeout   = 5 * time.Second
		defaultRecheck   = 30 * time.Minute
		defaultRateLimit = 3
		defaultCache     = 1000
	)
	if cfg.Timeout == 0 {
		cfg.Timeout = defaultTimeout
	}
	if cfg.RecheckInterval == 0 {
		cfg.RecheckInterval = defaultRecheck
	}
	if cfg.CacheLimit == 0 {
		cfg.CacheLimit = defaultCache
	}
	if cfg.RateLimit == 0 {
		cfg.RateLimit = defaultRateLimit
	}
	if cfg.ValidSchemes == nil {
		cfg.ValidSchemes = enode.ValidSchemes
	}
	if cfg.Resolver == nil {
		cfg.Resolver = new(net.Resolver)
	}
	if cfg.Logger == nil {
		cfg.Logger = log.Root()
	}
	return cfg
}

// NewClient creates a client.
func NewClient(cfg Config) *Client {
	cfg = cfg.withDefaults()
	rlimit := rate.NewLimiter(rate.Limit(cfg.RateLimit), 10)
	return &Client{
		cfg:       cfg,
		entries:   lru.NewCache[string, entry](cfg.CacheLimit),
		clock:     mclock.System{},
		ratelimit: rlimit,
	}
}

// SyncTree downloads the entire node tree at the given URL.
func (c *Client) SyncTree(url string) (*Tree, error) {
	le, err := parseLink(url)
	if err != nil {
		return nil, fmt.Errorf("invalid enrtree URL: %v", err)
	}
	ct := newClientTree(c, new(linkCache), le)
	t := &Tree{entries: make(map[string]entry)}
	if err := ct.syncAll(t.entries); err != nil {
		return nil, err
	}
	t.root = ct.root
	return t, nil
}

// NewIterator creates an iterator that visits all nodes at the
// given tree URLs.
func (c *Client) NewIterator(urls ...string) (enode.Iterator, error) {
	it := c.newRandomIterator()
	for _, url := range urls {
		if err := it.addTree(url); err != nil {
			return nil, err
		}
	}
	return it, nil
}

// resolveRoot retrieves a root entry via DNS.
func (c *Client) resolveRoot(ctx context.Context, loc *linkEntry) (rootEntry, error) {
	e, err, _ := c.singleflight.Do(loc.str, func() (interface{}, error) {
		txts, err := c.cfg.Resolver.LookupTXT(ctx, loc.domain)
		c.cfg.Logger.Trace("Updating DNS discovery root", "tree", loc.domain, "err", err)
		if err != nil {
			return rootEntry{}, err
		}
		for _, txt := range txts {
			if strings.HasPrefix(txt, rootPrefix) {
				return parseAndVerifyRoot(txt, loc)
			}
		}
		return rootEntry{}, nameError{loc.domain, errNoRoot}
	})
	return e.(rootEntry), err
}

func parseAndVerifyRoot(txt string, loc *linkEntry) (rootEntry, error) {
	e, err := parseRoot(txt)
	if err != nil {
		return e, err
	}
	if !e.verifySignature(loc.pubkey) {
		return e, entryError{typ: "root", err: errInvalidSig}
	}
	return e, nil
}

// resolveEntry retrieves an entry from the cache or fetches it from the network
// if it isn't cached.
func (c *Client) resolveEntry(ctx context.Context, domain, hash string) (entry, error) {
	// The rate limit always applies, even when the result might be cached. This is
	// important because it avoids hot-spinning in consumers of node iterators created on
	// this client.
	if err := c.ratelimit.Wait(ctx); err != nil {
		return nil, err
	}
	cacheKey := truncateHash(hash)
	if e, ok := c.entries.Get(cacheKey); ok {
		return e, nil
	}

	ei, err, _ := c.singleflight.Do(cacheKey, func() (interface{}, error) {
		e, err := c.doResolveEntry(ctx, domain, hash)
		if err != nil {
			return nil, err
		}
		c.entries.Add(cacheKey, e)
		return e, nil
	})
	e, _ := ei.(entry)
	return e, err
}

// doResolveEntry fetches an entry via DNS.
func (c *Client) doResolveEntry(ctx context.Context, domain, hash string) (entry, error) {
	wantHash, err := b32format.DecodeString(hash)
	if err != nil {
		return nil, fmt.Errorf("invalid base32 hash")
	}
	name := hash + "." + domain
	txts, err := c.cfg.Resolver.LookupTXT(ctx, hash+"."+domain)
	c.cfg.Logger.Trace("DNS discovery lookup", "name", name, "err", err)
	if err != nil {
		return nil, err
	}
	for _, txt := range txts {
		e, err := parseEntry(txt, c.cfg.ValidSchemes)
		if errors.Is(err, errUnknownEntry) {
			continue
		}
		if !bytes.HasPrefix(crypto.Keccak256([]byte(txt)), wantHash) {
			err = nameError{name, errHashMismatch}
		} else if err != nil {
			err = nameError{name, err}
		}
		return e, err
	}
	return nil, nameError{name, errNoEntry}
}

// randomIterator traverses a set of trees and returns nodes found in them.
type randomIterator struct {
	cur      *enode.Node
	ctx      context.Context
	cancelFn context.CancelFunc
	c        *Client

	mu    sync.Mutex
	lc    linkCache              // tracks tree dependencies
	trees map[string]*clientTree // all trees
	// buffers for syncableTrees
	syncableList []*clientTree
	disabledList []*clientTree
}

func (c *Client) newRandomIterator() *randomIterator {
	ctx, cancel := context.WithCancel(context.Background())
	return &randomIterator{
		c:        c,
		ctx:      ctx,
		cancelFn: cancel,
		trees:    make(map[string]*clientTree),
	}
}

// Node returns the current node.
func (it *randomIterator) Node() *enode.Node {
	return it.cur
}

// Close closes the iterator.
func (it *randomIterator) Close() {
	it.cancelFn()

	it.mu.Lock()
	defer it.mu.Unlock()
	it.trees = nil
}

// Next moves the iterator to the next node.
func (it *randomIterator) Next() bool {
	it.cur = it.nextNode()
	return it.cur != nil
}

// addTree adds an enrtree:// URL to the iterator.
func (it *randomIterator) addTree(url string) error {
	le, err := parseLink(url)
	if err != nil {
		return fmt.Errorf("invalid enrtree URL: %v", err)
	}
	it.lc.addLink("", le.str)
	return nil
}

// nextNode syncs random tree entries until it finds a node.
func (it *randomIterator) nextNode() *enode.Node {
	for {
		ct := it.pickTree()
		if ct == nil {
			return nil
		}
		n, err := ct.syncRandom(it.ctx)
		if err != nil {
			if errors.Is(err, it.ctx.Err()) {
				return nil // context canceled.
			}
			it.c.cfg.Logger.Debug("Error in DNS random node sync", "tree", ct.loc.domain, "err", err)
			continue
		}
		if n != nil {
			return n
		}
	}
}

// pickTree returns a random tree to sync from.
func (it *randomIterator) pickTree() *clientTree {
	it.mu.Lock()
	defer it.mu.Unlock()

	// First check if iterator was closed.
	// Need to do this here to avoid nil map access in rebuildTrees.
	if it.trees == nil {
		return nil
	}

	// Rebuild the trees map if any links have changed.
	if it.lc.changed {
		it.rebuildTrees()
		it.lc.changed = false
	}

	for {
		canSync, trees := it.syncableTrees()
		switch {
		case canSync:
			// Pick a random tree.
			return trees[rand.Intn(len(trees))]
		case len(trees) > 0:
			// No sync action can be performed on any tree right now. The only meaningful
			// thing to do is waiting for any root record to get updated.
			if !it.waitForRootUpdates(trees) {
				// Iterator was closed while waiting.
				return nil
			}
		default:
			// There are no trees left, the iterator was closed.
			return nil
		}
	}
}

// syncableTrees finds trees on which any meaningful sync action can be performed.
func (it *randomIterator) syncableTrees() (canSync bool, trees []*clientTree) {
	// Resize tree lists.
	it.syncableList = it.syncableList[:0]
	it.disabledList = it.disabledList[:0]

	// Partition them into the two lists.
	for _, ct := range it.trees {
		if ct.canSyncRandom() {
			it.syncableList = append(it.syncableList, ct)
		} else {
			it.disabledList = append(it.disabledList, ct)
		}
	}
	if len(it.syncableList) > 0 {
		return true, it.syncableList
	}
	return false, it.disabledList
}

// waitForRootUpdates waits for the closest scheduled root check time on the given trees.
func (it *randomIterator) waitForRootUpdates(trees []*clientTree) bool {
	var minTree *clientTree
	var nextCheck mclock.AbsTime
	for _, ct := range trees {
		check := ct.nextScheduledRootCheck()
		if minTree == nil || check < nextCheck {
			minTree = ct
			nextCheck = check
		}
	}

	sleep := nextCheck.Sub(it.c.clock.Now())
	it.c.cfg.Logger.Debug("DNS iterator waiting for root updates", "sleep", sleep, "tree", minTree.loc.domain)
	timeout := it.c.clock.NewTimer(sleep)
	defer timeout.Stop()
	select {
	case <-timeout.C():
		return true
	case <-it.ctx.Done():
		return false // Iterator was closed.
	}
}

// rebuildTrees rebuilds the 'trees' map.
func (it *randomIterator) rebuildTrees() {
	// Delete removed trees.
	for loc := range it.trees {
		if !it.lc.isReferenced(loc) {
			delete(it.trees, loc)
		}
	}
	// Add new trees.
	for loc := range it.lc.backrefs {
		if it.trees[loc] == nil {
			link, _ := parseLink(linkPrefix + loc)
			it.trees[loc] = newClientTree(it.c, &it.lc, link)
		}
	}
}
