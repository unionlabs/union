package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// RestoreOptions captures the options to be used during
// restoration of a backup.
type RestoreOptions struct {
	c *C.rocksdb_restore_options_t
}

// NewRestoreOptions creates a RestoreOptions instance.
func NewRestoreOptions() *RestoreOptions {
	return &RestoreOptions{
		c: C.rocksdb_restore_options_create(),
	}
}

// SetKeepLogFiles is used to set or unset the keep_log_files option
// If true, restore won't overwrite the existing log files in wal_dir. It will
// also move all log files from archive directory to wal_dir.
// By default, this is false.
func (ro *RestoreOptions) SetKeepLogFiles(v int) {
	C.rocksdb_restore_options_set_keep_log_files(ro.c, C.int(v))
}

// Destroy destroys this RestoreOptions instance.
func (ro *RestoreOptions) Destroy() {
	C.rocksdb_restore_options_destroy(ro.c)
	ro.c = nil
}
