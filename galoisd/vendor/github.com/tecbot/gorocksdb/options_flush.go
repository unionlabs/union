package gorocksdb

// #include "rocksdb/c.h"
import "C"

// FlushOptions represent all of the available options when manual flushing the
// database.
type FlushOptions struct {
	c *C.rocksdb_flushoptions_t
}

// NewDefaultFlushOptions creates a default FlushOptions object.
func NewDefaultFlushOptions() *FlushOptions {
	return NewNativeFlushOptions(C.rocksdb_flushoptions_create())
}

// NewNativeFlushOptions creates a FlushOptions object.
func NewNativeFlushOptions(c *C.rocksdb_flushoptions_t) *FlushOptions {
	return &FlushOptions{c}
}

// SetWait specify if the flush will wait until the flush is done.
// Default: true
func (opts *FlushOptions) SetWait(value bool) {
	C.rocksdb_flushoptions_set_wait(opts.c, boolToChar(value))
}

// Destroy deallocates the FlushOptions object.
func (opts *FlushOptions) Destroy() {
	C.rocksdb_flushoptions_destroy(opts.c)
	opts.c = nil
}
