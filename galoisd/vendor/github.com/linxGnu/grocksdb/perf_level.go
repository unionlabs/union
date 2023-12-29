package grocksdb

// #include "rocksdb/c.h"
import "C"

// PerfLevel indicates how much perf stats to collect. Affects perf_context and iostats_context.
type PerfLevel int

const (
	// KUninitialized indicates unknown setting
	KUninitialized PerfLevel = 0
	// KDisable disables perf stats
	KDisable PerfLevel = 1
	// KEnableCount enables only count stats
	KEnableCount PerfLevel = 2
	// KEnableTimeExceptForMutex other than count stats,
	// also enable time stats except for mutexes
	KEnableTimeExceptForMutex PerfLevel = 3
	// KEnableTimeAndCPUTimeExceptForMutex other than time,
	// also measure CPU time counters. Still don't measure
	// time (neither wall time nor CPU time) for mutexes.
	KEnableTimeAndCPUTimeExceptForMutex PerfLevel = 4
	// KEnableTime enables count and time stats
	KEnableTime PerfLevel = 5
	// KOutOfBounds N.B. Must always be the last value!
	KOutOfBounds PerfLevel = 6
)

// SetPerfLevel sets the perf stats level for current thread.
func SetPerfLevel(level PerfLevel) {
	C.rocksdb_set_perf_level(C.int(level))
}
