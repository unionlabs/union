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

package shutdowncheck

import (
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/log"
)

// ShutdownTracker is a service that reports previous unclean shutdowns
// upon start. It needs to be started after a successful start-up and stopped
// after a successful shutdown, just before the db is closed.
type ShutdownTracker struct {
	db     ethdb.Database
	stopCh chan struct{}
}

// NewShutdownTracker creates a new ShutdownTracker instance and has
// no other side-effect.
func NewShutdownTracker(db ethdb.Database) *ShutdownTracker {
	return &ShutdownTracker{
		db:     db,
		stopCh: make(chan struct{}),
	}
}

// MarkStartup is to be called in the beginning when the node starts. It will:
// - Push a new startup marker to the db
// - Report previous unclean shutdowns
func (t *ShutdownTracker) MarkStartup() {
	if uncleanShutdowns, discards, err := rawdb.PushUncleanShutdownMarker(t.db); err != nil {
		log.Error("Could not update unclean-shutdown-marker list", "error", err)
	} else {
		if discards > 0 {
			log.Warn("Old unclean shutdowns found", "count", discards)
		}
		for _, tstamp := range uncleanShutdowns {
			t := time.Unix(int64(tstamp), 0)
			log.Warn("Unclean shutdown detected", "booted", t,
				"age", common.PrettyAge(t))
		}
	}
}

// Start runs an event loop that updates the current marker's timestamp every 5 minutes.
func (t *ShutdownTracker) Start() {
	go func() {
		ticker := time.NewTicker(5 * time.Minute)
		defer ticker.Stop()
		for {
			select {
			case <-ticker.C:
				rawdb.UpdateUncleanShutdownMarker(t.db)
			case <-t.stopCh:
				return
			}
		}
	}()
}

// Stop will stop the update loop and clear the current marker.
func (t *ShutdownTracker) Stop() {
	// Stop update loop.
	t.stopCh <- struct{}{}
	// Clear last marker.
	rawdb.PopUncleanShutdownMarker(t.db)
}
