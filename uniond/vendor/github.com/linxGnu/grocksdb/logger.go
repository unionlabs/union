package grocksdb

// #include "rocksdb/c.h"
// #include "grocksdb.h"
import "C"
import "unsafe"

// Logger struct.
type Logger struct {
	c *C.rocksdb_logger_t
}

func NewStderrLogger(level InfoLogLevel, prefix string) *Logger {
	prefix_ := C.CString(prefix)
	defer C.free(unsafe.Pointer(prefix_))

	return &Logger{
		c: C.rocksdb_logger_create_stderr_logger(C.int(level), prefix_),
	}
}

// Destroy Logger.
func (l *Logger) Destroy() {
	C.rocksdb_logger_destroy(l.c)
	l.c = nil
}
