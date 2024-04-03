package grocksdb

// #include "rocksdb/c.h"
import "C"

type WaitForCompactOptions struct {
	p *C.rocksdb_wait_for_compact_options_t
}

func NewWaitForCompactOptions() *WaitForCompactOptions {
	return &WaitForCompactOptions{
		p: C.rocksdb_wait_for_compact_options_create(),
	}
}

// Destroy the object.
func (w *WaitForCompactOptions) Destroy() {
	C.rocksdb_wait_for_compact_options_destroy(w.p)
	w.p = nil
}

// SetAbortOnPause toggles the abort_on_pause flag, to abort waiting in case of
// a pause (PauseBackgroundWork() called).
//
// - If true, Status::Aborted will be returned immediately.
// - If false, ContinueBackgroundWork() must be called to resume the background jobs.
//
// Otherwise, jobs that were queued, but not scheduled yet may never finish
// and WaitForCompact() may wait indefinitely (if timeout is set, it will
// expire and return Status::TimedOut).
func (w *WaitForCompactOptions) SetAbortOnPause(v bool) {
	C.rocksdb_wait_for_compact_options_set_abort_on_pause(w.p, boolToChar(v))
}

// IsAbortOnPause checks if abort_on_pause flag is on.
func (w *WaitForCompactOptions) AbortOnPause() bool {
	return charToBool(C.rocksdb_wait_for_compact_options_get_abort_on_pause(w.p))
}

// SetFlush toggles the "flush" flag to flush all column families before starting to wait.
func (w *WaitForCompactOptions) SetFlush(v bool) {
	C.rocksdb_wait_for_compact_options_set_flush(w.p, boolToChar(v))
}

// IsFlush checks if "flush" flag is on.
func (w *WaitForCompactOptions) Flush() bool {
	return charToBool(C.rocksdb_wait_for_compact_options_get_flush(w.p))
}

// SetCloseDB toggles the "close_db" flag to call Close() after waiting is done.
// By the time Close() is called here, there should be no background jobs in progress
// and no new background jobs should be added.
//
// DB may not have been closed if Close() returned Aborted status due to unreleased snapshots
// in the system.
func (w *WaitForCompactOptions) SetCloseDB(v bool) {
	C.rocksdb_wait_for_compact_options_set_close_db(w.p, boolToChar(v))
}

// CloseDB checks if "close_db" flag is on.
func (w *WaitForCompactOptions) CloseDB() bool {
	return charToBool(C.rocksdb_wait_for_compact_options_get_close_db(w.p))
}

// SetTimeout in microseconds for waiting for compaction to complete.
// Status::TimedOut will be returned if timeout expires.
// when timeout == 0, WaitForCompact() will wait as long as there's background
// work to finish.
func (w *WaitForCompactOptions) SetTimeout(microseconds uint64) {
	C.rocksdb_wait_for_compact_options_set_timeout(w.p, C.uint64_t(microseconds))
}

// GetTimeout in microseconds.
func (w *WaitForCompactOptions) GetTimeout() uint64 {
	return uint64(C.rocksdb_wait_for_compact_options_get_timeout(w.p))
}
