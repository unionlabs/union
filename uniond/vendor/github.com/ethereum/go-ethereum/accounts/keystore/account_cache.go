// Copyright 2017 The go-ethereum Authors
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

package keystore

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"

	mapset "github.com/deckarep/golang-set/v2"
	"github.com/ethereum/go-ethereum/accounts"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/log"
	"golang.org/x/exp/slices"
)

// Minimum amount of time between cache reloads. This limit applies if the platform does
// not support change notifications. It also applies if the keystore directory does not
// exist yet, the code will attempt to create a watcher at most this often.
const minReloadInterval = 2 * time.Second

// byURL defines the sorting order for accounts.
func byURL(a, b accounts.Account) int {
	return a.URL.Cmp(b.URL)
}

// AmbiguousAddrError is returned when attempting to unlock
// an address for which more than one file exists.
type AmbiguousAddrError struct {
	Addr    common.Address
	Matches []accounts.Account
}

func (err *AmbiguousAddrError) Error() string {
	files := ""
	for i, a := range err.Matches {
		files += a.URL.Path
		if i < len(err.Matches)-1 {
			files += ", "
		}
	}
	return fmt.Sprintf("multiple keys match address (%s)", files)
}

// accountCache is a live index of all accounts in the keystore.
type accountCache struct {
	keydir   string
	watcher  *watcher
	mu       sync.Mutex
	all      []accounts.Account
	byAddr   map[common.Address][]accounts.Account
	throttle *time.Timer
	notify   chan struct{}
	fileC    fileCache
}

func newAccountCache(keydir string) (*accountCache, chan struct{}) {
	ac := &accountCache{
		keydir: keydir,
		byAddr: make(map[common.Address][]accounts.Account),
		notify: make(chan struct{}, 1),
		fileC:  fileCache{all: mapset.NewThreadUnsafeSet[string]()},
	}
	ac.watcher = newWatcher(ac)
	return ac, ac.notify
}

func (ac *accountCache) accounts() []accounts.Account {
	ac.maybeReload()
	ac.mu.Lock()
	defer ac.mu.Unlock()
	cpy := make([]accounts.Account, len(ac.all))
	copy(cpy, ac.all)
	return cpy
}

func (ac *accountCache) hasAddress(addr common.Address) bool {
	ac.maybeReload()
	ac.mu.Lock()
	defer ac.mu.Unlock()
	return len(ac.byAddr[addr]) > 0
}

func (ac *accountCache) add(newAccount accounts.Account) {
	ac.mu.Lock()
	defer ac.mu.Unlock()

	i := sort.Search(len(ac.all), func(i int) bool { return ac.all[i].URL.Cmp(newAccount.URL) >= 0 })
	if i < len(ac.all) && ac.all[i] == newAccount {
		return
	}
	// newAccount is not in the cache.
	ac.all = append(ac.all, accounts.Account{})
	copy(ac.all[i+1:], ac.all[i:])
	ac.all[i] = newAccount
	ac.byAddr[newAccount.Address] = append(ac.byAddr[newAccount.Address], newAccount)
}

// note: removed needs to be unique here (i.e. both File and Address must be set).
func (ac *accountCache) delete(removed accounts.Account) {
	ac.mu.Lock()
	defer ac.mu.Unlock()

	ac.all = removeAccount(ac.all, removed)
	if ba := removeAccount(ac.byAddr[removed.Address], removed); len(ba) == 0 {
		delete(ac.byAddr, removed.Address)
	} else {
		ac.byAddr[removed.Address] = ba
	}
}

// deleteByFile removes an account referenced by the given path.
func (ac *accountCache) deleteByFile(path string) {
	ac.mu.Lock()
	defer ac.mu.Unlock()
	i := sort.Search(len(ac.all), func(i int) bool { return ac.all[i].URL.Path >= path })

	if i < len(ac.all) && ac.all[i].URL.Path == path {
		removed := ac.all[i]
		ac.all = append(ac.all[:i], ac.all[i+1:]...)
		if ba := removeAccount(ac.byAddr[removed.Address], removed); len(ba) == 0 {
			delete(ac.byAddr, removed.Address)
		} else {
			ac.byAddr[removed.Address] = ba
		}
	}
}

// watcherStarted returns true if the watcher loop started running (even if it
// has since also ended).
func (ac *accountCache) watcherStarted() bool {
	ac.mu.Lock()
	defer ac.mu.Unlock()
	return ac.watcher.running || ac.watcher.runEnded
}

func removeAccount(slice []accounts.Account, elem accounts.Account) []accounts.Account {
	for i := range slice {
		if slice[i] == elem {
			return append(slice[:i], slice[i+1:]...)
		}
	}
	return slice
}

// find returns the cached account for address if there is a unique match.
// The exact matching rules are explained by the documentation of accounts.Account.
// Callers must hold ac.mu.
func (ac *accountCache) find(a accounts.Account) (accounts.Account, error) {
	// Limit search to address candidates if possible.
	matches := ac.all
	if (a.Address != common.Address{}) {
		matches = ac.byAddr[a.Address]
	}
	if a.URL.Path != "" {
		// If only the basename is specified, complete the path.
		if !strings.ContainsRune(a.URL.Path, filepath.Separator) {
			a.URL.Path = filepath.Join(ac.keydir, a.URL.Path)
		}
		for i := range matches {
			if matches[i].URL == a.URL {
				return matches[i], nil
			}
		}
		if (a.Address == common.Address{}) {
			return accounts.Account{}, ErrNoMatch
		}
	}
	switch len(matches) {
	case 1:
		return matches[0], nil
	case 0:
		return accounts.Account{}, ErrNoMatch
	default:
		err := &AmbiguousAddrError{Addr: a.Address, Matches: make([]accounts.Account, len(matches))}
		copy(err.Matches, matches)
		slices.SortFunc(err.Matches, byURL)
		return accounts.Account{}, err
	}
}

func (ac *accountCache) maybeReload() {
	ac.mu.Lock()

	if ac.watcher.running {
		ac.mu.Unlock()
		return // A watcher is running and will keep the cache up-to-date.
	}
	if ac.throttle == nil {
		ac.throttle = time.NewTimer(0)
	} else {
		select {
		case <-ac.throttle.C:
		default:
			ac.mu.Unlock()
			return // The cache was reloaded recently.
		}
	}
	// No watcher running, start it.
	ac.watcher.start()
	ac.throttle.Reset(minReloadInterval)
	ac.mu.Unlock()
	ac.scanAccounts()
}

func (ac *accountCache) close() {
	ac.mu.Lock()
	ac.watcher.close()
	if ac.throttle != nil {
		ac.throttle.Stop()
	}
	if ac.notify != nil {
		close(ac.notify)
		ac.notify = nil
	}
	ac.mu.Unlock()
}

// scanAccounts checks if any changes have occurred on the filesystem, and
// updates the account cache accordingly
func (ac *accountCache) scanAccounts() error {
	// Scan the entire folder metadata for file changes
	creates, deletes, updates, err := ac.fileC.scan(ac.keydir)
	if err != nil {
		log.Debug("Failed to reload keystore contents", "err", err)
		return err
	}
	if creates.Cardinality() == 0 && deletes.Cardinality() == 0 && updates.Cardinality() == 0 {
		return nil
	}
	// Create a helper method to scan the contents of the key files
	var (
		buf = new(bufio.Reader)
		key struct {
			Address string `json:"address"`
		}
	)
	readAccount := func(path string) *accounts.Account {
		fd, err := os.Open(path)
		if err != nil {
			log.Trace("Failed to open keystore file", "path", path, "err", err)
			return nil
		}
		defer fd.Close()
		buf.Reset(fd)
		// Parse the address.
		key.Address = ""
		err = json.NewDecoder(buf).Decode(&key)
		addr := common.HexToAddress(key.Address)
		switch {
		case err != nil:
			log.Debug("Failed to decode keystore key", "path", path, "err", err)
		case addr == common.Address{}:
			log.Debug("Failed to decode keystore key", "path", path, "err", "missing or zero address")
		default:
			return &accounts.Account{
				Address: addr,
				URL:     accounts.URL{Scheme: KeyStoreScheme, Path: path},
			}
		}
		return nil
	}
	// Process all the file diffs
	start := time.Now()

	for _, path := range creates.ToSlice() {
		if a := readAccount(path); a != nil {
			ac.add(*a)
		}
	}
	for _, path := range deletes.ToSlice() {
		ac.deleteByFile(path)
	}
	for _, path := range updates.ToSlice() {
		ac.deleteByFile(path)
		if a := readAccount(path); a != nil {
			ac.add(*a)
		}
	}
	end := time.Now()

	select {
	case ac.notify <- struct{}{}:
	default:
	}
	log.Trace("Handled keystore changes", "time", end.Sub(start))
	return nil
}
