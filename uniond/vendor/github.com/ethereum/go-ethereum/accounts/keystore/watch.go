// Copyright 2016 The go-ethereum Authors
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

//go:build (darwin && !ios && cgo) || freebsd || (linux && !arm64) || netbsd || solaris
// +build darwin,!ios,cgo freebsd linux,!arm64 netbsd solaris

package keystore

import (
	"os"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/fsnotify/fsnotify"
)

type watcher struct {
	ac       *accountCache
	running  bool // set to true when runloop begins
	runEnded bool // set to true when runloop ends
	starting bool // set to true prior to runloop starting
	quit     chan struct{}
}

func newWatcher(ac *accountCache) *watcher {
	return &watcher{
		ac:   ac,
		quit: make(chan struct{}),
	}
}

// enabled returns false on systems not supported.
func (*watcher) enabled() bool { return true }

// starts the watcher loop in the background.
// Start a watcher in the background if that's not already in progress.
// The caller must hold w.ac.mu.
func (w *watcher) start() {
	if w.starting || w.running {
		return
	}
	w.starting = true
	go w.loop()
}

func (w *watcher) close() {
	close(w.quit)
}

func (w *watcher) loop() {
	defer func() {
		w.ac.mu.Lock()
		w.running = false
		w.starting = false
		w.runEnded = true
		w.ac.mu.Unlock()
	}()
	logger := log.New("path", w.ac.keydir)

	// Create new watcher.
	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		log.Error("Failed to start filesystem watcher", "err", err)
		return
	}
	defer watcher.Close()
	if err := watcher.Add(w.ac.keydir); err != nil {
		if !os.IsNotExist(err) {
			logger.Warn("Failed to watch keystore folder", "err", err)
		}
		return
	}

	logger.Trace("Started watching keystore folder", "folder", w.ac.keydir)
	defer logger.Trace("Stopped watching keystore folder")

	w.ac.mu.Lock()
	w.running = true
	w.ac.mu.Unlock()

	// Wait for file system events and reload.
	// When an event occurs, the reload call is delayed a bit so that
	// multiple events arriving quickly only cause a single reload.
	var (
		debounceDuration = 500 * time.Millisecond
		rescanTriggered  = false
		debounce         = time.NewTimer(0)
	)
	// Ignore initial trigger
	if !debounce.Stop() {
		<-debounce.C
	}
	defer debounce.Stop()
	for {
		select {
		case <-w.quit:
			return
		case _, ok := <-watcher.Events:
			if !ok {
				return
			}
			// Trigger the scan (with delay), if not already triggered
			if !rescanTriggered {
				debounce.Reset(debounceDuration)
				rescanTriggered = true
			}
			// The fsnotify library does provide more granular event-info, it
			// would be possible to refresh individual affected files instead
			// of scheduling a full rescan. For most cases though, the
			// full rescan is quick and obviously simplest.
		case err, ok := <-watcher.Errors:
			if !ok {
				return
			}
			log.Info("Filsystem watcher error", "err", err)
		case <-debounce.C:
			w.ac.scanAccounts()
			rescanTriggered = false
		}
	}
}
