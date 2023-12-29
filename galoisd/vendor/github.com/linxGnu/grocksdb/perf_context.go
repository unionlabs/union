package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import (
	"unsafe"
)

// PerfContext a thread local context for gathering performance counter efficiently
// and transparently.
type PerfContext struct {
	c *C.rocksdb_perfcontext_t
}

// NewPerfContext returns new perf context.
func NewPerfContext() *PerfContext {
	return &PerfContext{
		c: C.rocksdb_perfcontext_create(),
	}
}

// Destroy perf context object.
func (ctx *PerfContext) Destroy() {
	C.rocksdb_perfcontext_destroy(ctx.c)
	ctx.c = nil
}

// Reset context.
func (ctx *PerfContext) Reset() {
	C.rocksdb_perfcontext_reset(ctx.c)
}

// Report with exclusion of zero counter.
func (ctx *PerfContext) Report(excludeZeroCounters bool) (value string) {
	cValue := C.rocksdb_perfcontext_report(ctx.c, boolToChar(excludeZeroCounters))
	value = C.GoString(cValue)
	C.free(unsafe.Pointer(cValue))
	return
}

// Metric returns value of a metric by its id.
//
// Id is one of:
//
//	enum {
//		rocksdb_user_key_comparison_count = 0,
//		rocksdb_block_cache_hit_count,
//		rocksdb_block_read_count,
//		rocksdb_block_read_byte,
//		rocksdb_block_read_time,
//		rocksdb_block_checksum_time,
//		rocksdb_block_decompress_time,
//		rocksdb_get_read_bytes,
//		rocksdb_multiget_read_bytes,
//		rocksdb_iter_read_bytes,
//		rocksdb_internal_key_skipped_count,
//		rocksdb_internal_delete_skipped_count,
//		rocksdb_internal_recent_skipped_count,
//		rocksdb_internal_merge_count,
//		rocksdb_get_snapshot_time,
//		rocksdb_get_from_memtable_time,
//		rocksdb_get_from_memtable_count,
//		rocksdb_get_post_process_time,
//		rocksdb_get_from_output_files_time,
//		rocksdb_seek_on_memtable_time,
//		rocksdb_seek_on_memtable_count,
//		rocksdb_next_on_memtable_count,
//		rocksdb_prev_on_memtable_count,
//		rocksdb_seek_child_seek_time,
//		rocksdb_seek_child_seek_count,
//		rocksdb_seek_min_heap_time,
//		rocksdb_seek_max_heap_time,
//		rocksdb_seek_internal_seek_time,
//		rocksdb_find_next_user_entry_time,
//		rocksdb_write_wal_time,
//		rocksdb_write_memtable_time,
//		rocksdb_write_delay_time,
//		rocksdb_write_pre_and_post_process_time,
//		rocksdb_db_mutex_lock_nanos,
//		rocksdb_db_condition_wait_nanos,
//		rocksdb_merge_operator_time_nanos,
//		rocksdb_read_index_block_nanos,
//		rocksdb_read_filter_block_nanos,
//		rocksdb_new_table_block_iter_nanos,
//		rocksdb_new_table_iterator_nanos,
//		rocksdb_block_seek_nanos,
//		rocksdb_find_table_nanos,
//		rocksdb_bloom_memtable_hit_count,
//		rocksdb_bloom_memtable_miss_count,
//		rocksdb_bloom_sst_hit_count,
//		rocksdb_bloom_sst_miss_count,
//		rocksdb_key_lock_wait_time,
//		rocksdb_key_lock_wait_count,
//		rocksdb_env_new_sequential_file_nanos,
//		rocksdb_env_new_random_access_file_nanos,
//		rocksdb_env_new_writable_file_nanos,
//		rocksdb_env_reuse_writable_file_nanos,
//		rocksdb_env_new_random_rw_file_nanos,
//		rocksdb_env_new_directory_nanos,
//		rocksdb_env_file_exists_nanos,
//		rocksdb_env_get_children_nanos,
//		rocksdb_env_get_children_file_attributes_nanos,
//		rocksdb_env_delete_file_nanos,
//		rocksdb_env_create_dir_nanos,
//		rocksdb_env_create_dir_if_missing_nanos,
//		rocksdb_env_delete_dir_nanos,
//		rocksdb_env_get_file_size_nanos,
//		rocksdb_env_get_file_modification_time_nanos,
//		rocksdb_env_rename_file_nanos,
//		rocksdb_env_link_file_nanos,
//		rocksdb_env_lock_file_nanos,
//		rocksdb_env_unlock_file_nanos,
//		rocksdb_env_new_logger_nanos,
//		rocksdb_number_async_seek,
//		rocksdb_blob_cache_hit_count,
//		rocksdb_blob_read_count,
//		rocksdb_blob_read_byte,
//		rocksdb_blob_read_time,
//		rocksdb_blob_checksum_time,
//		rocksdb_blob_decompress_time,
//		rocksdb_total_metric_count = 77
//	  };
func (ctx *PerfContext) Metric(id int) uint64 {
	value := C.rocksdb_perfcontext_metric(ctx.c, C.int(id))
	return uint64(value)
}
