// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"sync"

	"github.com/cockroachdb/errors"
)

type logRecycler struct {
	// The maximum number of log files to maintain for recycling.
	limit int

	// The minimum log number that is allowed to be recycled. Log numbers smaller
	// than this will be subject to immediate deletion. This is used to prevent
	// recycling a log written by a previous instance of the DB which may not
	// have had log recycling enabled. If that previous instance of the DB was
	// RocksDB, the old non-recyclable log record headers will be present.
	minRecycleLogNum FileNum

	mu struct {
		sync.Mutex
		logs      []fileInfo
		maxLogNum FileNum
	}
}

// add attempts to recycle the log file specified by logInfo. Returns true if
// the log file should not be deleted (i.e. the log is being recycled), and
// false otherwise.
func (r *logRecycler) add(logInfo fileInfo) bool {
	if logInfo.fileNum.FileNum() < r.minRecycleLogNum {
		return false
	}

	r.mu.Lock()
	defer r.mu.Unlock()

	if logInfo.fileNum.FileNum() <= r.mu.maxLogNum {
		// The log file number was already considered for recycling. Don't consider
		// it again. This avoids a race between adding the same log file for
		// recycling multiple times, and removing the log file for actual
		// reuse. Note that we return true because the log was already considered
		// for recycling and either it was deleted on the previous attempt (which
		// means we shouldn't get here) or it was recycled and thus the file
		// shouldn't be deleted.
		return true
	}
	r.mu.maxLogNum = logInfo.fileNum.FileNum()
	if len(r.mu.logs) >= r.limit {
		return false
	}
	r.mu.logs = append(r.mu.logs, logInfo)
	return true
}

// peek returns the log at the head of the recycling queue, or the zero value
// fileInfo and false if the queue is empty.
func (r *logRecycler) peek() (fileInfo, bool) {
	r.mu.Lock()
	defer r.mu.Unlock()

	if len(r.mu.logs) == 0 {
		return fileInfo{}, false
	}
	return r.mu.logs[0], true
}

func (r *logRecycler) stats() (count int, size uint64) {
	r.mu.Lock()
	defer r.mu.Unlock()
	count = len(r.mu.logs)
	for i := 0; i < count; i++ {
		size += r.mu.logs[i].fileSize
	}
	return count, size
}

// pop removes the log number at the head of the recycling queue, enforcing
// that it matches the specified logNum. An error is returned of the recycling
// queue is empty or the head log number does not match the specified one.
func (r *logRecycler) pop(logNum FileNum) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	if len(r.mu.logs) == 0 {
		return errors.New("pebble: log recycler empty")
	}
	if r.mu.logs[0].fileNum.FileNum() != logNum {
		return errors.Errorf("pebble: log recycler invalid %d vs %d", errors.Safe(logNum), errors.Safe(fileInfoNums(r.mu.logs)))
	}
	r.mu.logs = r.mu.logs[1:]
	return nil
}

func fileInfoNums(finfos []fileInfo) []FileNum {
	if len(finfos) == 0 {
		return nil
	}
	nums := make([]FileNum, len(finfos))
	for i := range finfos {
		nums[i] = finfos[i].fileNum.FileNum()
	}
	return nums
}
