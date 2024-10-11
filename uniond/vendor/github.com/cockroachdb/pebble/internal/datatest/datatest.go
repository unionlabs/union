// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package datatest provides common datadriven test commands for use outside of
// the root Pebble package.
package datatest

import (
	"strings"
	"sync"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
)

// TODO(jackson): Consider a refactoring that can consolidate this package and
// the datadriven commands defined in pebble/data_test.go.

// DefineBatch interprets the provided datadriven command as a sequence of write
// operations, one-per-line, to apply to the provided batch.
func DefineBatch(d *datadriven.TestData, b *pebble.Batch) error {
	for _, line := range strings.Split(d.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		if parts[1] == `<nil>` {
			parts[1] = ""
		}
		var err error
		switch parts[0] {
		case "set":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.Set([]byte(parts[1]), []byte(parts[2]), nil)
		case "del":
			if len(parts) != 2 {
				return errors.Errorf("%s expects 1 argument", parts[0])
			}
			err = b.Delete([]byte(parts[1]), nil)
		case "singledel":
			if len(parts) != 2 {
				return errors.Errorf("%s expects 1 argument", parts[0])
			}
			err = b.SingleDelete([]byte(parts[1]), nil)
		case "del-range":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.DeleteRange([]byte(parts[1]), []byte(parts[2]), nil)
		case "merge":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.Merge([]byte(parts[1]), []byte(parts[2]), nil)
		case "range-key-set":
			if len(parts) != 5 {
				return errors.Errorf("%s expects 4 arguments", parts[0])
			}
			err = b.RangeKeySet(
				[]byte(parts[1]),
				[]byte(parts[2]),
				[]byte(parts[3]),
				[]byte(parts[4]),
				nil)
		case "range-key-unset":
			if len(parts) != 4 {
				return errors.Errorf("%s expects 3 arguments", parts[0])
			}
			err = b.RangeKeyUnset(
				[]byte(parts[1]),
				[]byte(parts[2]),
				[]byte(parts[3]),
				nil)
		case "range-key-del":
			if len(parts) != 3 {
				return errors.Errorf("%s expects 2 arguments", parts[0])
			}
			err = b.RangeKeyDelete(
				[]byte(parts[1]),
				[]byte(parts[2]),
				nil)
		default:
			return errors.Errorf("unknown op: %s", parts[0])
		}
		if err != nil {
			return err
		}
	}
	return nil
}

// CompactionTracker is a listener that tracks the number of compactions.
type CompactionTracker struct {
	sync.Cond
	count    int
	attached bool
}

// NewCompactionTracker setups the necessary options to keep track of the
// compactions that are in flight.
func NewCompactionTracker(options *pebble.Options) *CompactionTracker {
	ct := CompactionTracker{}
	ct.Cond = sync.Cond{
		L: &sync.Mutex{},
	}
	ct.attached = true
	el := pebble.EventListener{
		CompactionEnd: func(info pebble.CompactionInfo) {
			ct.L.Lock()
			ct.count--
			ct.Broadcast()
			ct.L.Unlock()
		},
		CompactionBegin: func(info pebble.CompactionInfo) {
			ct.L.Lock()
			ct.count++
			ct.Broadcast()
			ct.L.Unlock()
		},
	}

	options.AddEventListener(el)
	return &ct
}

// WaitForInflightCompactionsToEqual waits until compactions meet the specified target.
func (cql *CompactionTracker) WaitForInflightCompactionsToEqual(target int) {
	cql.L.Lock()
	if !cql.attached {
		panic("Cannot wait for compactions if listener has not been attached")
	}
	for cql.count != target {
		cql.Wait()
	}
	cql.L.Unlock()
}
