//  Copyright (c) 2011-present, Facebook, Inc.  All rights reserved.
//  This source code is licensed under both the GPLv2 (found in the
//  COPYING file in the root directory) and Apache 2.0 License
//  (found in the LICENSE.Apache file in the root directory).

/* Copyright (c) 2011 The LevelDB Authors. All rights reserved.
  Use of this source code is governed by a BSD-style license that can be
  found in the LICENSE file. See the AUTHORS file for names of contributors.

  C bindings for rocksdb.  May be useful as a stable ABI that can be
  used by programs that keep rocksdb in a shared library, or for
  a JNI api.

  Does not support:
  . getters for the option types
  . custom comparators that implement key shortening
  . capturing post-write-snapshot
  . custom iter, db, env, cache implementations using just the C bindings

  Some conventions:

  (1) We expose just opaque struct pointers and functions to clients.
  This allows us to change internal representations without having to
  recompile clients.

  (2) For simplicity, there is no equivalent to the Slice type.  Instead,
  the caller has to pass the pointer and length as separate
  arguments.

  (3) Errors are represented by a null-terminated c string.  NULL
  means no error.  All operations that can raise an error are passed
  a "char** errptr" as the last argument.  One of the following must
  be true on entry:
     *errptr == NULL
     *errptr points to a malloc()ed null-terminated error message
  On success, a leveldb routine leaves *errptr unchanged.
  On failure, leveldb frees the old value of *errptr and
  set *errptr to a malloc()ed error message.

  (4) Bools have the type unsigned char (0 == false; rest == true)

  (5) All of the pointer arguments must be non-NULL.
*/

#pragma once

#ifdef _WIN32
#ifdef ROCKSDB_DLL
#ifdef ROCKSDB_LIBRARY_EXPORTS
#define ROCKSDB_LIBRARY_API __declspec(dllexport)
#else
#define ROCKSDB_LIBRARY_API __declspec(dllimport)
#endif
#else
#define ROCKSDB_LIBRARY_API
#endif
#else
#define ROCKSDB_LIBRARY_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

/* Exported types */

typedef struct rocksdb_t rocksdb_t;
typedef struct rocksdb_backup_engine_t rocksdb_backup_engine_t;
typedef struct rocksdb_backup_engine_info_t rocksdb_backup_engine_info_t;
typedef struct rocksdb_backup_engine_options_t rocksdb_backup_engine_options_t;
typedef struct rocksdb_restore_options_t rocksdb_restore_options_t;
typedef struct rocksdb_memory_allocator_t rocksdb_memory_allocator_t;
typedef struct rocksdb_lru_cache_options_t rocksdb_lru_cache_options_t;
typedef struct rocksdb_cache_t rocksdb_cache_t;
typedef struct rocksdb_compactionfilter_t rocksdb_compactionfilter_t;
typedef struct rocksdb_compactionfiltercontext_t
    rocksdb_compactionfiltercontext_t;
typedef struct rocksdb_compactionfilterfactory_t
    rocksdb_compactionfilterfactory_t;
typedef struct rocksdb_comparator_t rocksdb_comparator_t;
typedef struct rocksdb_dbpath_t rocksdb_dbpath_t;
typedef struct rocksdb_env_t rocksdb_env_t;
typedef struct rocksdb_fifo_compaction_options_t
    rocksdb_fifo_compaction_options_t;
typedef struct rocksdb_filelock_t rocksdb_filelock_t;
typedef struct rocksdb_filterpolicy_t rocksdb_filterpolicy_t;
typedef struct rocksdb_flushoptions_t rocksdb_flushoptions_t;
typedef struct rocksdb_iterator_t rocksdb_iterator_t;
typedef struct rocksdb_logger_t rocksdb_logger_t;
typedef struct rocksdb_mergeoperator_t rocksdb_mergeoperator_t;
typedef struct rocksdb_options_t rocksdb_options_t;
typedef struct rocksdb_compactoptions_t rocksdb_compactoptions_t;
typedef struct rocksdb_block_based_table_options_t
    rocksdb_block_based_table_options_t;
typedef struct rocksdb_cuckoo_table_options_t rocksdb_cuckoo_table_options_t;
typedef struct rocksdb_randomfile_t rocksdb_randomfile_t;
typedef struct rocksdb_readoptions_t rocksdb_readoptions_t;
typedef struct rocksdb_seqfile_t rocksdb_seqfile_t;
typedef struct rocksdb_slicetransform_t rocksdb_slicetransform_t;
typedef struct rocksdb_snapshot_t rocksdb_snapshot_t;
typedef struct rocksdb_writablefile_t rocksdb_writablefile_t;
typedef struct rocksdb_writebatch_t rocksdb_writebatch_t;
typedef struct rocksdb_writebatch_wi_t rocksdb_writebatch_wi_t;
typedef struct rocksdb_writeoptions_t rocksdb_writeoptions_t;
typedef struct rocksdb_universal_compaction_options_t
    rocksdb_universal_compaction_options_t;
typedef struct rocksdb_livefiles_t rocksdb_livefiles_t;
typedef struct rocksdb_column_family_handle_t rocksdb_column_family_handle_t;
typedef struct rocksdb_column_family_metadata_t
    rocksdb_column_family_metadata_t;
typedef struct rocksdb_level_metadata_t rocksdb_level_metadata_t;
typedef struct rocksdb_sst_file_metadata_t rocksdb_sst_file_metadata_t;
typedef struct rocksdb_envoptions_t rocksdb_envoptions_t;
typedef struct rocksdb_ingestexternalfileoptions_t
    rocksdb_ingestexternalfileoptions_t;
typedef struct rocksdb_sstfilewriter_t rocksdb_sstfilewriter_t;
typedef struct rocksdb_ratelimiter_t rocksdb_ratelimiter_t;
typedef struct rocksdb_perfcontext_t rocksdb_perfcontext_t;
typedef struct rocksdb_pinnableslice_t rocksdb_pinnableslice_t;
typedef struct rocksdb_transactiondb_options_t rocksdb_transactiondb_options_t;
typedef struct rocksdb_transactiondb_t rocksdb_transactiondb_t;
typedef struct rocksdb_transaction_options_t rocksdb_transaction_options_t;
typedef struct rocksdb_optimistictransactiondb_t
    rocksdb_optimistictransactiondb_t;
typedef struct rocksdb_optimistictransaction_options_t
    rocksdb_optimistictransaction_options_t;
typedef struct rocksdb_transaction_t rocksdb_transaction_t;
typedef struct rocksdb_checkpoint_t rocksdb_checkpoint_t;
typedef struct rocksdb_wal_iterator_t rocksdb_wal_iterator_t;
typedef struct rocksdb_wal_readoptions_t rocksdb_wal_readoptions_t;
typedef struct rocksdb_memory_consumers_t rocksdb_memory_consumers_t;
typedef struct rocksdb_memory_usage_t rocksdb_memory_usage_t;

/* DB operations */

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open(
    const rocksdb_options_t* options, const char* name, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_with_ttl(
    const rocksdb_options_t* options, const char* name, int ttl, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_for_read_only(
    const rocksdb_options_t* options, const char* name,
    unsigned char error_if_wal_file_exists, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_as_secondary(
    const rocksdb_options_t* options, const char* name,
    const char* secondary_path, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_backup_engine_t* rocksdb_backup_engine_open(
    const rocksdb_options_t* options, const char* path, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_backup_engine_t*
rocksdb_backup_engine_open_opts(const rocksdb_backup_engine_options_t* options,
                                rocksdb_env_t* env, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_create_new_backup(
    rocksdb_backup_engine_t* be, rocksdb_t* db, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_create_new_backup_flush(
    rocksdb_backup_engine_t* be, rocksdb_t* db,
    unsigned char flush_before_backup, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_purge_old_backups(
    rocksdb_backup_engine_t* be, uint32_t num_backups_to_keep, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_restore_options_t*
rocksdb_restore_options_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_restore_options_destroy(
    rocksdb_restore_options_t* opt);
extern ROCKSDB_LIBRARY_API void rocksdb_restore_options_set_keep_log_files(
    rocksdb_restore_options_t* opt, int v);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_verify_backup(
    rocksdb_backup_engine_t* be, uint32_t backup_id, char** errptr);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_restore_db_from_latest_backup(
    rocksdb_backup_engine_t* be, const char* db_dir, const char* wal_dir,
    const rocksdb_restore_options_t* restore_options, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_restore_db_from_backup(
    rocksdb_backup_engine_t* be, const char* db_dir, const char* wal_dir,
    const rocksdb_restore_options_t* restore_options, const uint32_t backup_id,
    char** errptr);

extern ROCKSDB_LIBRARY_API const rocksdb_backup_engine_info_t*
rocksdb_backup_engine_get_backup_info(rocksdb_backup_engine_t* be);

extern ROCKSDB_LIBRARY_API int rocksdb_backup_engine_info_count(
    const rocksdb_backup_engine_info_t* info);

extern ROCKSDB_LIBRARY_API int64_t rocksdb_backup_engine_info_timestamp(
    const rocksdb_backup_engine_info_t* info, int index);

extern ROCKSDB_LIBRARY_API uint32_t rocksdb_backup_engine_info_backup_id(
    const rocksdb_backup_engine_info_t* info, int index);

extern ROCKSDB_LIBRARY_API uint64_t rocksdb_backup_engine_info_size(
    const rocksdb_backup_engine_info_t* info, int index);

extern ROCKSDB_LIBRARY_API uint32_t rocksdb_backup_engine_info_number_files(
    const rocksdb_backup_engine_info_t* info, int index);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_info_destroy(
    const rocksdb_backup_engine_info_t* info);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_close(
    rocksdb_backup_engine_t* be);

extern ROCKSDB_LIBRARY_API void rocksdb_put_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, const char* ts, size_t tslen, const char* val, size_t vallen,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_put_cf_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* ts, size_t tslen, const char* val, size_t vallen,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, const char* ts, size_t tslen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_cf_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* ts, size_t tslen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_singledelete(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_singledelete_cf(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_singledelete_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, const char* ts, size_t tslen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_singledelete_cf_with_ts(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* ts, size_t tslen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_increase_full_history_ts_low(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    const char* ts_low, size_t ts_lowlen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_get_full_history_ts_low(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    size_t* ts_lowlen, char** errptr);

/* BackupEngineOptions */

extern ROCKSDB_LIBRARY_API rocksdb_backup_engine_options_t*
rocksdb_backup_engine_options_create(const char* backup_dir);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_options_set_backup_dir(
    rocksdb_backup_engine_options_t* options, const char* backup_dir);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_options_set_env(
    rocksdb_backup_engine_options_t* options, rocksdb_env_t* env);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_share_table_files(
    rocksdb_backup_engine_options_t* options, unsigned char val);

extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_backup_engine_options_get_share_table_files(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_options_set_sync(
    rocksdb_backup_engine_options_t* options, unsigned char val);

extern ROCKSDB_LIBRARY_API unsigned char rocksdb_backup_engine_options_get_sync(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_destroy_old_data(
    rocksdb_backup_engine_options_t* options, unsigned char val);

extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_backup_engine_options_get_destroy_old_data(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_backup_log_files(
    rocksdb_backup_engine_options_t* options, unsigned char val);

extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_backup_engine_options_get_backup_log_files(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_backup_rate_limit(
    rocksdb_backup_engine_options_t* options, uint64_t limit);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_backup_engine_options_get_backup_rate_limit(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_restore_rate_limit(
    rocksdb_backup_engine_options_t* options, uint64_t limit);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_backup_engine_options_get_restore_rate_limit(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_max_background_operations(
    rocksdb_backup_engine_options_t* options, int val);

extern ROCKSDB_LIBRARY_API int
rocksdb_backup_engine_options_get_max_background_operations(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_callback_trigger_interval_size(
    rocksdb_backup_engine_options_t* options, uint64_t size);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_backup_engine_options_get_callback_trigger_interval_size(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_max_valid_backups_to_open(
    rocksdb_backup_engine_options_t* options, int val);

extern ROCKSDB_LIBRARY_API int
rocksdb_backup_engine_options_get_max_valid_backups_to_open(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void
rocksdb_backup_engine_options_set_share_files_with_checksum_naming(
    rocksdb_backup_engine_options_t* options, int val);

extern ROCKSDB_LIBRARY_API int
rocksdb_backup_engine_options_get_share_files_with_checksum_naming(
    rocksdb_backup_engine_options_t* options);

extern ROCKSDB_LIBRARY_API void rocksdb_backup_engine_options_destroy(
    rocksdb_backup_engine_options_t*);

/* Checkpoint */

extern ROCKSDB_LIBRARY_API rocksdb_checkpoint_t*
rocksdb_checkpoint_object_create(rocksdb_t* db, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_checkpoint_create(
    rocksdb_checkpoint_t* checkpoint, const char* checkpoint_dir,
    uint64_t log_size_for_flush, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_checkpoint_object_destroy(
    rocksdb_checkpoint_t* checkpoint);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_and_trim_history(
    const rocksdb_options_t* options, const char* name, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, char* trim_ts,
    size_t trim_tslen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_column_families(
    const rocksdb_options_t* options, const char* name, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_column_families_with_ttl(
    const rocksdb_options_t* options, const char* name, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, const int* ttls,
    char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t*
rocksdb_open_for_read_only_column_families(
    const rocksdb_options_t* options, const char* name, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles,
    unsigned char error_if_wal_file_exists, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t* rocksdb_open_as_secondary_column_families(
    const rocksdb_options_t* options, const char* name,
    const char* secondary_path, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, char** errptr);

extern ROCKSDB_LIBRARY_API char** rocksdb_list_column_families(
    const rocksdb_options_t* options, const char* name, size_t* lencf,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_list_column_families_destroy(
    char** list, size_t len);

extern ROCKSDB_LIBRARY_API rocksdb_column_family_handle_t*
rocksdb_create_column_family(rocksdb_t* db,
                             const rocksdb_options_t* column_family_options,
                             const char* column_family_name, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_column_family_handle_t*
rocksdb_create_column_family_with_ttl(
    rocksdb_t* db, const rocksdb_options_t* column_family_options,
    const char* column_family_name, int ttl, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_drop_column_family(
    rocksdb_t* db, rocksdb_column_family_handle_t* handle, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_column_family_handle_destroy(
    rocksdb_column_family_handle_t*);

extern ROCKSDB_LIBRARY_API uint32_t
rocksdb_column_family_handle_get_id(rocksdb_column_family_handle_t* handle);

extern ROCKSDB_LIBRARY_API char* rocksdb_column_family_handle_get_name(
    rocksdb_column_family_handle_t* handle, size_t* name_len);

extern ROCKSDB_LIBRARY_API void rocksdb_close(rocksdb_t* db);

extern ROCKSDB_LIBRARY_API void rocksdb_put(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, const char* val, size_t vallen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_put_cf(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* val, size_t vallen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_cf(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_range_cf(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* start_key,
    size_t start_key_len, const char* end_key, size_t end_key_len,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_merge(
    rocksdb_t* db, const rocksdb_writeoptions_t* options, const char* key,
    size_t keylen, const char* val, size_t vallen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_merge_cf(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* val, size_t vallen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_write(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_writebatch_t* batch, char** errptr);

/* Returns NULL if not found.  A malloc()ed array otherwise.
   Stores the length of the array in *vallen. */
extern ROCKSDB_LIBRARY_API char* rocksdb_get(
    rocksdb_t* db, const rocksdb_readoptions_t* options, const char* key,
    size_t keylen, size_t* vallen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_get_with_ts(
    rocksdb_t* db, const rocksdb_readoptions_t* options, const char* key,
    size_t keylen, size_t* vallen, char** ts, size_t* tslen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_get_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_get_cf_with_ts(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, char** ts, size_t* tslen, char** errptr);

// if values_list[i] == NULL and errs[i] == NULL,
// then we got status.IsNotFound(), which we will not return.
// all errors except status status.ok() and status.IsNotFound() are returned.
//
// errs, values_list and values_list_sizes must be num_keys in length,
// allocated by the caller.
// errs is a list of strings as opposed to the conventional one error,
// where errs[i] is the status for retrieval of keys_list[i].
// each non-NULL errs entry is a malloc()ed, null terminated string.
// each non-NULL values_list entry is a malloc()ed array, with
// the length for each stored in values_list_sizes[i].
extern ROCKSDB_LIBRARY_API void rocksdb_multi_get(
    rocksdb_t* db, const rocksdb_readoptions_t* options, size_t num_keys,
    const char* const* keys_list, const size_t* keys_list_sizes,
    char** values_list, size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_multi_get_with_ts(
    rocksdb_t* db, const rocksdb_readoptions_t* options, size_t num_keys,
    const char* const* keys_list, const size_t* keys_list_sizes,
    char** values_list, size_t* values_list_sizes, char** timestamp_list,
    size_t* timestamp_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_multi_get_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    const rocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_multi_get_cf_with_ts(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    const rocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** timestamps_list,
    size_t* timestamps_list_sizes, char** errs);

// The MultiGet API that improves performance by batching operations
// in the read path for greater efficiency. Currently, only the block based
// table format with full filters are supported. Other table formats such
// as plain table, block based table with block based filters and
// partitioned indexes will still work, but will not get any performance
// benefits.
//
// Note that all the keys passed to this API are restricted to a single
// column family.
//
// Parameters -
// db - the RocksDB instance.
// options - ReadOptions
// column_family - ColumnFamilyHandle* that the keys belong to. All the keys
//                 passed to the API are restricted to a single column family
// num_keys - Number of keys to lookup
// keys_list - Pointer to C style array of keys with num_keys elements
// keys_list_sizes - Pointer to C style array of the size of corresponding key
//   in key_list with num_keys elements.
// values - Pointer to C style array of PinnableSlices with num_keys elements
// statuses - Pointer to C style array of Status with num_keys elements
// sorted_input - If true, it means the input keys are already sorted by key
//                order, so the MultiGet() API doesn't have to sort them
//                again. If false, the keys will be copied and sorted
//                internally by the API - the input array will not be
//                modified
extern ROCKSDB_LIBRARY_API void rocksdb_batched_multi_get_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, size_t num_keys,
    const char* const* keys_list, const size_t* keys_list_sizes,
    rocksdb_pinnableslice_t** values, char** errs, const bool sorted_input);

// The value is only allocated (using malloc) and returned if it is found and
// value_found isn't NULL. In that case the user is responsible for freeing it.
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_key_may_exist(
    rocksdb_t* db, const rocksdb_readoptions_t* options, const char* key,
    size_t key_len, char** value, size_t* val_len, const char* timestamp,
    size_t timestamp_len, unsigned char* value_found);

// The value is only allocated (using malloc) and returned if it is found and
// value_found isn't NULL. In that case the user is responsible for freeing it.
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_key_may_exist_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t key_len, char** value, size_t* val_len, const char* timestamp,
    size_t timestamp_len, unsigned char* value_found);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t* rocksdb_create_iterator(
    rocksdb_t* db, const rocksdb_readoptions_t* options);

extern ROCKSDB_LIBRARY_API rocksdb_wal_iterator_t* rocksdb_get_updates_since(
    rocksdb_t* db, uint64_t seq_number,
    const rocksdb_wal_readoptions_t* options, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t* rocksdb_create_iterator_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family);

extern ROCKSDB_LIBRARY_API void rocksdb_create_iterators(
    rocksdb_t* db, rocksdb_readoptions_t* opts,
    rocksdb_column_family_handle_t** column_families,
    rocksdb_iterator_t** iterators, size_t size, char** errptr);

extern ROCKSDB_LIBRARY_API const rocksdb_snapshot_t* rocksdb_create_snapshot(
    rocksdb_t* db);

extern ROCKSDB_LIBRARY_API void rocksdb_release_snapshot(
    rocksdb_t* db, const rocksdb_snapshot_t* snapshot);

/* Returns NULL if property name is unknown.
   Else returns a pointer to a malloc()-ed null-terminated value. */
extern ROCKSDB_LIBRARY_API char* rocksdb_property_value(rocksdb_t* db,
                                                        const char* propname);
/* returns 0 on success, -1 otherwise */
int rocksdb_property_int(rocksdb_t* db, const char* propname,
                         uint64_t* out_val);

/* returns 0 on success, -1 otherwise */
int rocksdb_property_int_cf(rocksdb_t* db,
                            rocksdb_column_family_handle_t* column_family,
                            const char* propname, uint64_t* out_val);

extern ROCKSDB_LIBRARY_API char* rocksdb_property_value_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    const char* propname);

extern ROCKSDB_LIBRARY_API void rocksdb_approximate_sizes(
    rocksdb_t* db, int num_ranges, const char* const* range_start_key,
    const size_t* range_start_key_len, const char* const* range_limit_key,
    const size_t* range_limit_key_len, uint64_t* sizes, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_approximate_sizes_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    int num_ranges, const char* const* range_start_key,
    const size_t* range_start_key_len, const char* const* range_limit_key,
    const size_t* range_limit_key_len, uint64_t* sizes, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_compact_range(rocksdb_t* db,
                                                      const char* start_key,
                                                      size_t start_key_len,
                                                      const char* limit_key,
                                                      size_t limit_key_len);

extern ROCKSDB_LIBRARY_API void rocksdb_compact_range_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len);

extern ROCKSDB_LIBRARY_API void rocksdb_suggest_compact_range(
    rocksdb_t* db, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_suggest_compact_range_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_compact_range_opt(
    rocksdb_t* db, rocksdb_compactoptions_t* opt, const char* start_key,
    size_t start_key_len, const char* limit_key, size_t limit_key_len);

extern ROCKSDB_LIBRARY_API void rocksdb_compact_range_cf_opt(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    rocksdb_compactoptions_t* opt, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_file(rocksdb_t* db,
                                                    const char* name);

extern ROCKSDB_LIBRARY_API const rocksdb_livefiles_t* rocksdb_livefiles(
    rocksdb_t* db);

extern ROCKSDB_LIBRARY_API void rocksdb_flush(
    rocksdb_t* db, const rocksdb_flushoptions_t* options, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_flush_cf(
    rocksdb_t* db, const rocksdb_flushoptions_t* options,
    rocksdb_column_family_handle_t* column_family, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_flush_wal(rocksdb_t* db,
                                                  unsigned char sync,
                                                  char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_disable_file_deletions(rocksdb_t* db,
                                                               char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_enable_file_deletions(
    rocksdb_t* db, unsigned char force, char** errptr);

/* Management operations */

extern ROCKSDB_LIBRARY_API void rocksdb_destroy_db(
    const rocksdb_options_t* options, const char* name, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_repair_db(
    const rocksdb_options_t* options, const char* name, char** errptr);

/* Iterator */

extern ROCKSDB_LIBRARY_API void rocksdb_iter_destroy(rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_iter_valid(
    const rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_seek_to_first(rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_seek_to_last(rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_seek(rocksdb_iterator_t*,
                                                  const char* k, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_seek_for_prev(rocksdb_iterator_t*,
                                                           const char* k,
                                                           size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_next(rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_prev(rocksdb_iterator_t*);
extern ROCKSDB_LIBRARY_API const char* rocksdb_iter_key(
    const rocksdb_iterator_t*, size_t* klen);
extern ROCKSDB_LIBRARY_API const char* rocksdb_iter_value(
    const rocksdb_iterator_t*, size_t* vlen);
extern ROCKSDB_LIBRARY_API const char* rocksdb_iter_timestamp(
    const rocksdb_iterator_t*, size_t* tslen);
extern ROCKSDB_LIBRARY_API void rocksdb_iter_get_error(
    const rocksdb_iterator_t*, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_wal_iter_next(
    rocksdb_wal_iterator_t* iter);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_wal_iter_valid(
    const rocksdb_wal_iterator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_wal_iter_status(
    const rocksdb_wal_iterator_t* iter, char** errptr);
extern ROCKSDB_LIBRARY_API rocksdb_writebatch_t* rocksdb_wal_iter_get_batch(
    const rocksdb_wal_iterator_t* iter, uint64_t* seq);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_get_latest_sequence_number(rocksdb_t* db);
extern ROCKSDB_LIBRARY_API void rocksdb_wal_iter_destroy(
    const rocksdb_wal_iterator_t* iter);

/* Write batch */

extern ROCKSDB_LIBRARY_API rocksdb_writebatch_t* rocksdb_writebatch_create(
    void);
extern ROCKSDB_LIBRARY_API rocksdb_writebatch_t* rocksdb_writebatch_create_from(
    const char* rep, size_t size);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_destroy(
    rocksdb_writebatch_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_clear(rocksdb_writebatch_t*);
extern ROCKSDB_LIBRARY_API int rocksdb_writebatch_count(rocksdb_writebatch_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_put(rocksdb_writebatch_t*,
                                                       const char* key,
                                                       size_t klen,
                                                       const char* val,
                                                       size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_put_cf(
    rocksdb_writebatch_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_put_cf_with_ts(
    rocksdb_writebatch_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* ts, size_t tslen, const char* val,
    size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_putv(
    rocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_putv_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_merge(rocksdb_writebatch_t*,
                                                         const char* key,
                                                         size_t klen,
                                                         const char* val,
                                                         size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_merge_cf(
    rocksdb_writebatch_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_mergev(
    rocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_mergev_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete(rocksdb_writebatch_t*,
                                                          const char* key,
                                                          size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_singledelete(
    rocksdb_writebatch_t* b, const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_cf(
    rocksdb_writebatch_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_cf_with_ts(
    rocksdb_writebatch_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* ts, size_t tslen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_singledelete_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_singledelete_cf_with_ts(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* ts, size_t tslen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_deletev(
    rocksdb_writebatch_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_deletev_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_range(
    rocksdb_writebatch_t* b, const char* start_key, size_t start_key_len,
    const char* end_key, size_t end_key_len);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_range_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* end_key,
    size_t end_key_len);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_rangev(
    rocksdb_writebatch_t* b, int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_delete_rangev_cf(
    rocksdb_writebatch_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_put_log_data(
    rocksdb_writebatch_t*, const char* blob, size_t len);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_iterate(
    rocksdb_writebatch_t*, void* state,
    void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
    void (*deleted)(void*, const char* k, size_t klen));
extern ROCKSDB_LIBRARY_API const char* rocksdb_writebatch_data(
    rocksdb_writebatch_t*, size_t* size);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_set_save_point(
    rocksdb_writebatch_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_rollback_to_save_point(
    rocksdb_writebatch_t*, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_pop_save_point(
    rocksdb_writebatch_t*, char** errptr);

/* Write batch with index */

extern ROCKSDB_LIBRARY_API rocksdb_writebatch_wi_t*
rocksdb_writebatch_wi_create(size_t reserved_bytes,
                             unsigned char overwrite_keys);
extern ROCKSDB_LIBRARY_API rocksdb_writebatch_wi_t*
rocksdb_writebatch_wi_create_from(const char* rep, size_t size);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_destroy(
    rocksdb_writebatch_wi_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_clear(
    rocksdb_writebatch_wi_t*);
extern ROCKSDB_LIBRARY_API int rocksdb_writebatch_wi_count(
    rocksdb_writebatch_wi_t* b);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_put(
    rocksdb_writebatch_wi_t*, const char* key, size_t klen, const char* val,
    size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_put_cf(
    rocksdb_writebatch_wi_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_putv(
    rocksdb_writebatch_wi_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_putv_cf(
    rocksdb_writebatch_wi_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_merge(
    rocksdb_writebatch_wi_t*, const char* key, size_t klen, const char* val,
    size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_merge_cf(
    rocksdb_writebatch_wi_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_mergev(
    rocksdb_writebatch_wi_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, int num_values,
    const char* const* values_list, const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_mergev_cf(
    rocksdb_writebatch_wi_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes,
    int num_values, const char* const* values_list,
    const size_t* values_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete(
    rocksdb_writebatch_wi_t*, const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_singledelete(
    rocksdb_writebatch_wi_t*, const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete_cf(
    rocksdb_writebatch_wi_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_singledelete_cf(
    rocksdb_writebatch_wi_t*, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_deletev(
    rocksdb_writebatch_wi_t* b, int num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_deletev_cf(
    rocksdb_writebatch_wi_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* keys_list, const size_t* keys_list_sizes);
// DO NOT USE - rocksdb_writebatch_wi_delete_range is not yet supported
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete_range(
    rocksdb_writebatch_wi_t* b, const char* start_key, size_t start_key_len,
    const char* end_key, size_t end_key_len);
// DO NOT USE - rocksdb_writebatch_wi_delete_range_cf is not yet supported
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete_range_cf(
    rocksdb_writebatch_wi_t* b, rocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* end_key,
    size_t end_key_len);
// DO NOT USE - rocksdb_writebatch_wi_delete_rangev is not yet supported
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete_rangev(
    rocksdb_writebatch_wi_t* b, int num_keys,
    const char* const* start_keys_list, const size_t* start_keys_list_sizes,
    const char* const* end_keys_list, const size_t* end_keys_list_sizes);
// DO NOT USE - rocksdb_writebatch_wi_delete_rangev_cf is not yet supported
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_delete_rangev_cf(
    rocksdb_writebatch_wi_t* b, rocksdb_column_family_handle_t* column_family,
    int num_keys, const char* const* start_keys_list,
    const size_t* start_keys_list_sizes, const char* const* end_keys_list,
    const size_t* end_keys_list_sizes);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_put_log_data(
    rocksdb_writebatch_wi_t*, const char* blob, size_t len);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_iterate(
    rocksdb_writebatch_wi_t* b, void* state,
    void (*put)(void*, const char* k, size_t klen, const char* v, size_t vlen),
    void (*deleted)(void*, const char* k, size_t klen));
extern ROCKSDB_LIBRARY_API const char* rocksdb_writebatch_wi_data(
    rocksdb_writebatch_wi_t* b, size_t* size);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_set_save_point(
    rocksdb_writebatch_wi_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writebatch_wi_rollback_to_save_point(
    rocksdb_writebatch_wi_t*, char** errptr);
extern ROCKSDB_LIBRARY_API char* rocksdb_writebatch_wi_get_from_batch(
    rocksdb_writebatch_wi_t* wbwi, const rocksdb_options_t* options,
    const char* key, size_t keylen, size_t* vallen, char** errptr);
extern ROCKSDB_LIBRARY_API char* rocksdb_writebatch_wi_get_from_batch_cf(
    rocksdb_writebatch_wi_t* wbwi, const rocksdb_options_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, char** errptr);
extern ROCKSDB_LIBRARY_API char* rocksdb_writebatch_wi_get_from_batch_and_db(
    rocksdb_writebatch_wi_t* wbwi, rocksdb_t* db,
    const rocksdb_readoptions_t* options, const char* key, size_t keylen,
    size_t* vallen, char** errptr);
extern ROCKSDB_LIBRARY_API char* rocksdb_writebatch_wi_get_from_batch_and_db_cf(
    rocksdb_writebatch_wi_t* wbwi, rocksdb_t* db,
    const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_write_writebatch_wi(
    rocksdb_t* db, const rocksdb_writeoptions_t* options,
    rocksdb_writebatch_wi_t* wbwi, char** errptr);
extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_writebatch_wi_create_iterator_with_base(
    rocksdb_writebatch_wi_t* wbwi, rocksdb_iterator_t* base_iterator);
extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_writebatch_wi_create_iterator_with_base_cf(
    rocksdb_writebatch_wi_t* wbwi, rocksdb_iterator_t* base_iterator,
    rocksdb_column_family_handle_t* cf);

/* Options utils */

// Load the latest rocksdb options from the specified db_path.
//
// On success, num_column_families will be updated with a non-zero
// number indicating the number of column families.
// The returned db_options, column_family_names, and column_family_options
// should be released via rocksdb_load_latest_options_destroy().
//
// On error, a non-null errptr that includes the error message will be
// returned.  db_options, column_family_names, and column_family_options
// will be set to NULL.
extern ROCKSDB_LIBRARY_API void rocksdb_load_latest_options(
    const char* db_path, rocksdb_env_t* env, bool ignore_unknown_options,
    rocksdb_cache_t* cache, rocksdb_options_t** db_options,
    size_t* num_column_families, char*** column_family_names,
    rocksdb_options_t*** column_family_options, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_load_latest_options_destroy(
    rocksdb_options_t* db_options, char** list_column_family_names,
    rocksdb_options_t** list_column_family_options, size_t len);

/* Block based table options */

extern ROCKSDB_LIBRARY_API rocksdb_block_based_table_options_t*
rocksdb_block_based_options_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_destroy(
    rocksdb_block_based_table_options_t* options);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_checksum(
    rocksdb_block_based_table_options_t*, char);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_block_size(
    rocksdb_block_based_table_options_t* options, size_t block_size);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_block_size_deviation(
    rocksdb_block_based_table_options_t* options, int block_size_deviation);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_block_restart_interval(
    rocksdb_block_based_table_options_t* options, int block_restart_interval);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_index_block_restart_interval(
    rocksdb_block_based_table_options_t* options,
    int index_block_restart_interval);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_metadata_block_size(
    rocksdb_block_based_table_options_t* options, uint64_t metadata_block_size);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_partition_filters(
    rocksdb_block_based_table_options_t* options,
    unsigned char partition_filters);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_optimize_filters_for_memory(
    rocksdb_block_based_table_options_t* options,
    unsigned char optimize_filters_for_memory);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_use_delta_encoding(
    rocksdb_block_based_table_options_t* options,
    unsigned char use_delta_encoding);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_filter_policy(
    rocksdb_block_based_table_options_t* options,
    rocksdb_filterpolicy_t* filter_policy);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_no_block_cache(
    rocksdb_block_based_table_options_t* options, unsigned char no_block_cache);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_block_cache(
    rocksdb_block_based_table_options_t* options, rocksdb_cache_t* block_cache);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_block_cache_compressed(
    rocksdb_block_based_table_options_t* options,
    rocksdb_cache_t* block_cache_compressed);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_whole_key_filtering(
    rocksdb_block_based_table_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_format_version(
    rocksdb_block_based_table_options_t*, int);
enum {
  rocksdb_block_based_table_index_type_binary_search = 0,
  rocksdb_block_based_table_index_type_hash_search = 1,
  rocksdb_block_based_table_index_type_two_level_index_search = 2,
};
extern ROCKSDB_LIBRARY_API void rocksdb_block_based_options_set_index_type(
    rocksdb_block_based_table_options_t*, int);  // uses one of the above enums
enum {
  rocksdb_block_based_table_data_block_index_type_binary_search = 0,
  rocksdb_block_based_table_data_block_index_type_binary_search_and_hash = 1,
};
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_data_block_index_type(
    rocksdb_block_based_table_options_t*, int);  // uses one of the above enums
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_data_block_hash_ratio(
    rocksdb_block_based_table_options_t* options, double v);
// rocksdb_block_based_options_set_hash_index_allow_collision()
// is removed since BlockBasedTableOptions.hash_index_allow_collision()
// is removed
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_cache_index_and_filter_blocks(
    rocksdb_block_based_table_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_cache_index_and_filter_blocks_with_high_priority(
    rocksdb_block_based_table_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_pin_l0_filter_and_index_blocks_in_cache(
    rocksdb_block_based_table_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void
rocksdb_block_based_options_set_pin_top_level_index_and_filter(
    rocksdb_block_based_table_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_block_based_table_factory(
    rocksdb_options_t* opt, rocksdb_block_based_table_options_t* table_options);

/* Cuckoo table options */

extern ROCKSDB_LIBRARY_API rocksdb_cuckoo_table_options_t*
rocksdb_cuckoo_options_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_cuckoo_options_destroy(
    rocksdb_cuckoo_table_options_t* options);
extern ROCKSDB_LIBRARY_API void rocksdb_cuckoo_options_set_hash_ratio(
    rocksdb_cuckoo_table_options_t* options, double v);
extern ROCKSDB_LIBRARY_API void rocksdb_cuckoo_options_set_max_search_depth(
    rocksdb_cuckoo_table_options_t* options, uint32_t v);
extern ROCKSDB_LIBRARY_API void rocksdb_cuckoo_options_set_cuckoo_block_size(
    rocksdb_cuckoo_table_options_t* options, uint32_t v);
extern ROCKSDB_LIBRARY_API void
rocksdb_cuckoo_options_set_identity_as_first_hash(
    rocksdb_cuckoo_table_options_t* options, unsigned char v);
extern ROCKSDB_LIBRARY_API void rocksdb_cuckoo_options_set_use_module_hash(
    rocksdb_cuckoo_table_options_t* options, unsigned char v);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_cuckoo_table_factory(
    rocksdb_options_t* opt, rocksdb_cuckoo_table_options_t* table_options);

/* Options */
extern ROCKSDB_LIBRARY_API void rocksdb_set_options(rocksdb_t* db, int count,
                                                    const char* const keys[],
                                                    const char* const values[],
                                                    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_set_options_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* handle, int count,
    const char* const keys[], const char* const values[], char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_options_t* rocksdb_options_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_options_destroy(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API rocksdb_options_t* rocksdb_options_create_copy(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_increase_parallelism(
    rocksdb_options_t* opt, int total_threads);
extern ROCKSDB_LIBRARY_API void rocksdb_options_optimize_for_point_lookup(
    rocksdb_options_t* opt, uint64_t block_cache_size_mb);
extern ROCKSDB_LIBRARY_API void rocksdb_options_optimize_level_style_compaction(
    rocksdb_options_t* opt, uint64_t memtable_memory_budget);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_optimize_universal_style_compaction(
    rocksdb_options_t* opt, uint64_t memtable_memory_budget);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_allow_ingest_behind(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_allow_ingest_behind(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compaction_filter(
    rocksdb_options_t*, rocksdb_compactionfilter_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compaction_filter_factory(
    rocksdb_options_t*, rocksdb_compactionfilterfactory_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_compaction_readahead_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_compaction_readahead_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_comparator(
    rocksdb_options_t*, rocksdb_comparator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_merge_operator(
    rocksdb_options_t*, rocksdb_mergeoperator_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_uint64add_merge_operator(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compression_per_level(
    rocksdb_options_t* opt, const int* level_values, size_t num_levels);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_create_if_missing(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_create_if_missing(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_create_missing_column_families(rocksdb_options_t*,
                                                   unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_create_missing_column_families(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_error_if_exists(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_error_if_exists(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_paranoid_checks(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_paranoid_checks(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_db_paths(
    rocksdb_options_t*, const rocksdb_dbpath_t** path_values, size_t num_paths);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_env(rocksdb_options_t*,
                                                        rocksdb_env_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_info_log(rocksdb_options_t*,
                                                             rocksdb_logger_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_info_log_level(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_info_log_level(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_write_buffer_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_write_buffer_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_db_write_buffer_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_db_write_buffer_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_open_files(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_open_files(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_file_opening_threads(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_file_opening_threads(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_total_wal_size(
    rocksdb_options_t* opt, uint64_t n);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_max_total_wal_size(rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compression_options(
    rocksdb_options_t*, int, int, int, int);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_compression_options_zstd_max_train_bytes(rocksdb_options_t*,
                                                             int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_compression_options_zstd_max_train_bytes(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_compression_options_use_zstd_dict_trainer(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_compression_options_use_zstd_dict_trainer(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_compression_options_parallel_threads(rocksdb_options_t*,
                                                         int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_compression_options_parallel_threads(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_compression_options_max_dict_buffer_bytes(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_compression_options_max_dict_buffer_bytes(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_bottommost_compression_options(rocksdb_options_t*, int, int,
                                                   int, int, unsigned char);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_bottommost_compression_options_zstd_max_train_bytes(
    rocksdb_options_t*, int, unsigned char);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_bottommost_compression_options_use_zstd_dict_trainer(
    rocksdb_options_t*, unsigned char, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_bottommost_compression_options_use_zstd_dict_trainer(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_bottommost_compression_options_max_dict_buffer_bytes(
    rocksdb_options_t*, uint64_t, unsigned char);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_prefix_extractor(
    rocksdb_options_t*, rocksdb_slicetransform_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_num_levels(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_num_levels(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_level0_file_num_compaction_trigger(rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_level0_file_num_compaction_trigger(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_level0_slowdown_writes_trigger(rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_level0_slowdown_writes_trigger(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_level0_stop_writes_trigger(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_level0_stop_writes_trigger(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_target_file_size_base(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_target_file_size_base(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_target_file_size_multiplier(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_target_file_size_multiplier(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_bytes_for_level_base(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_max_bytes_for_level_base(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_level_compaction_dynamic_level_bytes(rocksdb_options_t*,
                                                         unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_level_compaction_dynamic_level_bytes(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_max_bytes_for_level_multiplier(rocksdb_options_t*, double);
extern ROCKSDB_LIBRARY_API double
rocksdb_options_get_max_bytes_for_level_multiplier(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_max_bytes_for_level_multiplier_additional(
    rocksdb_options_t*, int* level_values, size_t num_levels);
extern ROCKSDB_LIBRARY_API void rocksdb_options_enable_statistics(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_skip_stats_update_on_db_open(rocksdb_options_t* opt,
                                                 unsigned char val);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_skip_stats_update_on_db_open(rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_skip_checking_sst_file_sizes_on_db_open(
    rocksdb_options_t* opt, unsigned char val);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_skip_checking_sst_file_sizes_on_db_open(
    rocksdb_options_t* opt);

/* Blob Options Settings */
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_enable_blob_files(
    rocksdb_options_t* opt, unsigned char val);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_enable_blob_files(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_min_blob_size(
    rocksdb_options_t* opt, uint64_t val);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_min_blob_size(rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_file_size(
    rocksdb_options_t* opt, uint64_t val);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_blob_file_size(rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_compression_type(
    rocksdb_options_t* opt, int val);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_blob_compression_type(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_enable_blob_gc(
    rocksdb_options_t* opt, unsigned char val);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_enable_blob_gc(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_gc_age_cutoff(
    rocksdb_options_t* opt, double val);
extern ROCKSDB_LIBRARY_API double rocksdb_options_get_blob_gc_age_cutoff(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_gc_force_threshold(
    rocksdb_options_t* opt, double val);
extern ROCKSDB_LIBRARY_API double rocksdb_options_get_blob_gc_force_threshold(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_blob_compaction_readahead_size(rocksdb_options_t* opt,
                                                   uint64_t val);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_blob_compaction_readahead_size(rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_file_starting_level(
    rocksdb_options_t* opt, int val);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_blob_file_starting_level(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_blob_cache(
    rocksdb_options_t* opt, rocksdb_cache_t* blob_cache);

enum {
  rocksdb_prepopulate_blob_disable = 0,
  rocksdb_prepopulate_blob_flush_only = 1
};

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_prepopulate_blob_cache(
    rocksdb_options_t* opt, int val);

extern ROCKSDB_LIBRARY_API int rocksdb_options_get_prepopulate_blob_cache(
    rocksdb_options_t* opt);

/* returns a pointer to a malloc()-ed, null terminated string */
extern ROCKSDB_LIBRARY_API char* rocksdb_options_statistics_get_string(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_write_buffer_number(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_write_buffer_number(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_min_write_buffer_number_to_merge(rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_min_write_buffer_number_to_merge(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_max_write_buffer_number_to_maintain(rocksdb_options_t*,
                                                        int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_max_write_buffer_number_to_maintain(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_max_write_buffer_size_to_maintain(rocksdb_options_t*,
                                                      int64_t);
extern ROCKSDB_LIBRARY_API int64_t
rocksdb_options_get_max_write_buffer_size_to_maintain(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_enable_pipelined_write(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_enable_pipelined_write(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_unordered_write(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_unordered_write(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_subcompactions(
    rocksdb_options_t*, uint32_t);
extern ROCKSDB_LIBRARY_API uint32_t
rocksdb_options_get_max_subcompactions(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_background_jobs(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_background_jobs(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_background_compactions(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_background_compactions(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_background_flushes(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_max_background_flushes(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_log_file_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_max_log_file_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_log_file_time_to_roll(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_log_file_time_to_roll(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_keep_log_file_num(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_keep_log_file_num(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_recycle_log_file_num(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_recycle_log_file_num(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_soft_pending_compaction_bytes_limit(rocksdb_options_t* opt,
                                                        size_t v);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_soft_pending_compaction_bytes_limit(rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_hard_pending_compaction_bytes_limit(rocksdb_options_t* opt,
                                                        size_t v);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_hard_pending_compaction_bytes_limit(rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_manifest_file_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_max_manifest_file_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_table_cache_numshardbits(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_table_cache_numshardbits(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_arena_block_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_arena_block_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_use_fsync(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_use_fsync(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_db_log_dir(
    rocksdb_options_t*, const char*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_wal_dir(rocksdb_options_t*,
                                                            const char*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_WAL_ttl_seconds(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_WAL_ttl_seconds(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_WAL_size_limit_MB(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_WAL_size_limit_MB(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_manifest_preallocation_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_manifest_preallocation_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_allow_mmap_reads(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_allow_mmap_reads(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_allow_mmap_writes(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_allow_mmap_writes(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_use_direct_reads(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_use_direct_reads(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_use_direct_io_for_flush_and_compaction(rocksdb_options_t*,
                                                           unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_use_direct_io_for_flush_and_compaction(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_is_fd_close_on_exec(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_is_fd_close_on_exec(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_stats_dump_period_sec(
    rocksdb_options_t*, unsigned int);
extern ROCKSDB_LIBRARY_API unsigned int
rocksdb_options_get_stats_dump_period_sec(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_stats_persist_period_sec(
    rocksdb_options_t*, unsigned int);
extern ROCKSDB_LIBRARY_API unsigned int
rocksdb_options_get_stats_persist_period_sec(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_advise_random_on_open(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_advise_random_on_open(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_access_hint_on_compaction_start(rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_options_get_access_hint_on_compaction_start(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_use_adaptive_mutex(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_use_adaptive_mutex(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_bytes_per_sync(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_bytes_per_sync(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_wal_bytes_per_sync(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_wal_bytes_per_sync(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_writable_file_max_buffer_size(rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_writable_file_max_buffer_size(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_allow_concurrent_memtable_write(rocksdb_options_t*,
                                                    unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_allow_concurrent_memtable_write(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_enable_write_thread_adaptive_yield(rocksdb_options_t*,
                                                       unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_enable_write_thread_adaptive_yield(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_max_sequential_skip_in_iterations(rocksdb_options_t*,
                                                      uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_max_sequential_skip_in_iterations(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_disable_auto_compactions(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_disable_auto_compactions(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_optimize_filters_for_hits(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_optimize_filters_for_hits(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_delete_obsolete_files_period_micros(rocksdb_options_t*,
                                                        uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_delete_obsolete_files_period_micros(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_prepare_for_bulk_load(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_memtable_vector_rep(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_memtable_prefix_bloom_size_ratio(rocksdb_options_t*,
                                                     double);
extern ROCKSDB_LIBRARY_API double
rocksdb_options_get_memtable_prefix_bloom_size_ratio(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_compaction_bytes(
    rocksdb_options_t*, uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_options_get_max_compaction_bytes(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_hash_skip_list_rep(
    rocksdb_options_t*, size_t, int32_t, int32_t);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_hash_link_list_rep(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_plain_table_factory(
    rocksdb_options_t*, uint32_t, int, double, size_t);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_min_level_to_compress(
    rocksdb_options_t* opt, int level);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_memtable_huge_page_size(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_memtable_huge_page_size(rocksdb_options_t*);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_max_successive_merges(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_max_successive_merges(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_bloom_locality(
    rocksdb_options_t*, uint32_t);
extern ROCKSDB_LIBRARY_API uint32_t
rocksdb_options_get_bloom_locality(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_inplace_update_support(
    rocksdb_options_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_inplace_update_support(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_inplace_update_num_locks(
    rocksdb_options_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_options_get_inplace_update_num_locks(rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_report_bg_io_stats(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_report_bg_io_stats(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_avoid_unnecessary_blocking_io(rocksdb_options_t*,
                                                  unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_options_get_avoid_unnecessary_blocking_io(rocksdb_options_t*);

extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_experimental_mempurge_threshold(rocksdb_options_t*, double);
extern ROCKSDB_LIBRARY_API double
rocksdb_options_get_experimental_mempurge_threshold(rocksdb_options_t*);

enum {
  rocksdb_tolerate_corrupted_tail_records_recovery = 0,
  rocksdb_absolute_consistency_recovery = 1,
  rocksdb_point_in_time_recovery = 2,
  rocksdb_skip_any_corrupted_records_recovery = 3
};
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_wal_recovery_mode(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_wal_recovery_mode(
    rocksdb_options_t*);

enum {
  rocksdb_no_compression = 0,
  rocksdb_snappy_compression = 1,
  rocksdb_zlib_compression = 2,
  rocksdb_bz2_compression = 3,
  rocksdb_lz4_compression = 4,
  rocksdb_lz4hc_compression = 5,
  rocksdb_xpress_compression = 6,
  rocksdb_zstd_compression = 7
};
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compression(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_compression(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_bottommost_compression(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_bottommost_compression(
    rocksdb_options_t*);

enum {
  rocksdb_level_compaction = 0,
  rocksdb_universal_compaction = 1,
  rocksdb_fifo_compaction = 2
};
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_compaction_style(
    rocksdb_options_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_compaction_style(
    rocksdb_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_universal_compaction_options(
    rocksdb_options_t*, rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_fifo_compaction_options(
    rocksdb_options_t* opt, rocksdb_fifo_compaction_options_t* fifo);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_ratelimiter(
    rocksdb_options_t* opt, rocksdb_ratelimiter_t* limiter);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_atomic_flush(
    rocksdb_options_t* opt, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_atomic_flush(
    rocksdb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_row_cache(
    rocksdb_options_t* opt, rocksdb_cache_t* cache);

extern ROCKSDB_LIBRARY_API void
rocksdb_options_add_compact_on_deletion_collector_factory(
    rocksdb_options_t*, size_t window_size, size_t num_dels_trigger);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_manual_wal_flush(
    rocksdb_options_t* opt, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_options_get_manual_wal_flush(
    rocksdb_options_t* opt);
extern ROCKSDB_LIBRARY_API void rocksdb_options_set_wal_compression(
    rocksdb_options_t* opt, int);
extern ROCKSDB_LIBRARY_API int rocksdb_options_get_wal_compression(
    rocksdb_options_t* opt);

/* RateLimiter */
extern ROCKSDB_LIBRARY_API rocksdb_ratelimiter_t* rocksdb_ratelimiter_create(
    int64_t rate_bytes_per_sec, int64_t refill_period_us, int32_t fairness);
extern ROCKSDB_LIBRARY_API void rocksdb_ratelimiter_destroy(
    rocksdb_ratelimiter_t*);

/* PerfContext */
enum {
  rocksdb_uninitialized = 0,
  rocksdb_disable = 1,
  rocksdb_enable_count = 2,
  rocksdb_enable_time_except_for_mutex = 3,
  rocksdb_enable_time = 4,
  rocksdb_out_of_bounds = 5
};

enum {
  rocksdb_user_key_comparison_count = 0,
  rocksdb_block_cache_hit_count,
  rocksdb_block_read_count,
  rocksdb_block_read_byte,
  rocksdb_block_read_time,
  rocksdb_block_checksum_time,
  rocksdb_block_decompress_time,
  rocksdb_get_read_bytes,
  rocksdb_multiget_read_bytes,
  rocksdb_iter_read_bytes,
  rocksdb_internal_key_skipped_count,
  rocksdb_internal_delete_skipped_count,
  rocksdb_internal_recent_skipped_count,
  rocksdb_internal_merge_count,
  rocksdb_get_snapshot_time,
  rocksdb_get_from_memtable_time,
  rocksdb_get_from_memtable_count,
  rocksdb_get_post_process_time,
  rocksdb_get_from_output_files_time,
  rocksdb_seek_on_memtable_time,
  rocksdb_seek_on_memtable_count,
  rocksdb_next_on_memtable_count,
  rocksdb_prev_on_memtable_count,
  rocksdb_seek_child_seek_time,
  rocksdb_seek_child_seek_count,
  rocksdb_seek_min_heap_time,
  rocksdb_seek_max_heap_time,
  rocksdb_seek_internal_seek_time,
  rocksdb_find_next_user_entry_time,
  rocksdb_write_wal_time,
  rocksdb_write_memtable_time,
  rocksdb_write_delay_time,
  rocksdb_write_pre_and_post_process_time,
  rocksdb_db_mutex_lock_nanos,
  rocksdb_db_condition_wait_nanos,
  rocksdb_merge_operator_time_nanos,
  rocksdb_read_index_block_nanos,
  rocksdb_read_filter_block_nanos,
  rocksdb_new_table_block_iter_nanos,
  rocksdb_new_table_iterator_nanos,
  rocksdb_block_seek_nanos,
  rocksdb_find_table_nanos,
  rocksdb_bloom_memtable_hit_count,
  rocksdb_bloom_memtable_miss_count,
  rocksdb_bloom_sst_hit_count,
  rocksdb_bloom_sst_miss_count,
  rocksdb_key_lock_wait_time,
  rocksdb_key_lock_wait_count,
  rocksdb_env_new_sequential_file_nanos,
  rocksdb_env_new_random_access_file_nanos,
  rocksdb_env_new_writable_file_nanos,
  rocksdb_env_reuse_writable_file_nanos,
  rocksdb_env_new_random_rw_file_nanos,
  rocksdb_env_new_directory_nanos,
  rocksdb_env_file_exists_nanos,
  rocksdb_env_get_children_nanos,
  rocksdb_env_get_children_file_attributes_nanos,
  rocksdb_env_delete_file_nanos,
  rocksdb_env_create_dir_nanos,
  rocksdb_env_create_dir_if_missing_nanos,
  rocksdb_env_delete_dir_nanos,
  rocksdb_env_get_file_size_nanos,
  rocksdb_env_get_file_modification_time_nanos,
  rocksdb_env_rename_file_nanos,
  rocksdb_env_link_file_nanos,
  rocksdb_env_lock_file_nanos,
  rocksdb_env_unlock_file_nanos,
  rocksdb_env_new_logger_nanos,
  rocksdb_number_async_seek,
  rocksdb_blob_cache_hit_count,
  rocksdb_blob_read_count,
  rocksdb_blob_read_byte,
  rocksdb_blob_read_time,
  rocksdb_blob_checksum_time,
  rocksdb_blob_decompress_time,
  rocksdb_internal_range_del_reseek_count,
  rocksdb_total_metric_count = 78
};

extern ROCKSDB_LIBRARY_API void rocksdb_set_perf_level(int);
extern ROCKSDB_LIBRARY_API rocksdb_perfcontext_t* rocksdb_perfcontext_create(
    void);
extern ROCKSDB_LIBRARY_API void rocksdb_perfcontext_reset(
    rocksdb_perfcontext_t* context);
extern ROCKSDB_LIBRARY_API char* rocksdb_perfcontext_report(
    rocksdb_perfcontext_t* context, unsigned char exclude_zero_counters);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_perfcontext_metric(rocksdb_perfcontext_t* context, int metric);
extern ROCKSDB_LIBRARY_API void rocksdb_perfcontext_destroy(
    rocksdb_perfcontext_t* context);

/* Compaction Filter */

extern ROCKSDB_LIBRARY_API rocksdb_compactionfilter_t*
rocksdb_compactionfilter_create(
    void* state, void (*destructor)(void*),
    unsigned char (*filter)(void*, int level, const char* key,
                            size_t key_length, const char* existing_value,
                            size_t value_length, char** new_value,
                            size_t* new_value_length,
                            unsigned char* value_changed),
    const char* (*name)(void*));
extern ROCKSDB_LIBRARY_API void rocksdb_compactionfilter_set_ignore_snapshots(
    rocksdb_compactionfilter_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void rocksdb_compactionfilter_destroy(
    rocksdb_compactionfilter_t*);

/* Compaction Filter Context */

extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_compactionfiltercontext_is_full_compaction(
    rocksdb_compactionfiltercontext_t* context);

extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_compactionfiltercontext_is_manual_compaction(
    rocksdb_compactionfiltercontext_t* context);

/* Compaction Filter Factory */

extern ROCKSDB_LIBRARY_API rocksdb_compactionfilterfactory_t*
rocksdb_compactionfilterfactory_create(
    void* state, void (*destructor)(void*),
    rocksdb_compactionfilter_t* (*create_compaction_filter)(
        void*, rocksdb_compactionfiltercontext_t* context),
    const char* (*name)(void*));
extern ROCKSDB_LIBRARY_API void rocksdb_compactionfilterfactory_destroy(
    rocksdb_compactionfilterfactory_t*);

/* Comparator */

extern ROCKSDB_LIBRARY_API rocksdb_comparator_t* rocksdb_comparator_create(
    void* state, void (*destructor)(void*),
    int (*compare)(void*, const char* a, size_t alen, const char* b,
                   size_t blen),
    const char* (*name)(void*));
extern ROCKSDB_LIBRARY_API void rocksdb_comparator_destroy(
    rocksdb_comparator_t*);

extern ROCKSDB_LIBRARY_API rocksdb_comparator_t*
rocksdb_comparator_with_ts_create(
    void* state, void (*destructor)(void*),
    int (*compare)(void*, const char* a, size_t alen, const char* b,
                   size_t blen),
    int (*compare_ts)(void*, const char* a_ts, size_t a_tslen, const char* b_ts,
                      size_t b_tslen),
    int (*compare_without_ts)(void*, const char* a, size_t alen,
                              unsigned char a_has_ts, const char* b,
                              size_t blen, unsigned char b_has_ts),
    const char* (*name)(void*), size_t timestamp_size);

/* Filter policy */

extern ROCKSDB_LIBRARY_API void rocksdb_filterpolicy_destroy(
    rocksdb_filterpolicy_t*);

extern ROCKSDB_LIBRARY_API rocksdb_filterpolicy_t*
rocksdb_filterpolicy_create_bloom(double bits_per_key);
extern ROCKSDB_LIBRARY_API rocksdb_filterpolicy_t*
rocksdb_filterpolicy_create_bloom_full(double bits_per_key);
extern ROCKSDB_LIBRARY_API rocksdb_filterpolicy_t*
rocksdb_filterpolicy_create_ribbon(double bloom_equivalent_bits_per_key);
extern ROCKSDB_LIBRARY_API rocksdb_filterpolicy_t*
rocksdb_filterpolicy_create_ribbon_hybrid(double bloom_equivalent_bits_per_key,
                                          int bloom_before_level);

/* Merge Operator */

extern ROCKSDB_LIBRARY_API rocksdb_mergeoperator_t*
rocksdb_mergeoperator_create(
    void* state, void (*destructor)(void*),
    char* (*full_merge)(void*, const char* key, size_t key_length,
                        const char* existing_value,
                        size_t existing_value_length,
                        const char* const* operands_list,
                        const size_t* operands_list_length, int num_operands,
                        unsigned char* success, size_t* new_value_length),
    char* (*partial_merge)(void*, const char* key, size_t key_length,
                           const char* const* operands_list,
                           const size_t* operands_list_length, int num_operands,
                           unsigned char* success, size_t* new_value_length),
    void (*delete_value)(void*, const char* value, size_t value_length),
    const char* (*name)(void*));
extern ROCKSDB_LIBRARY_API void rocksdb_mergeoperator_destroy(
    rocksdb_mergeoperator_t*);

/* Read options */

extern ROCKSDB_LIBRARY_API rocksdb_readoptions_t* rocksdb_readoptions_create(
    void);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_destroy(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_verify_checksums(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_readoptions_get_verify_checksums(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_fill_cache(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_readoptions_get_fill_cache(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_snapshot(
    rocksdb_readoptions_t*, const rocksdb_snapshot_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_iterate_upper_bound(
    rocksdb_readoptions_t*, const char* key, size_t keylen);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_iterate_lower_bound(
    rocksdb_readoptions_t*, const char* key, size_t keylen);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_read_tier(
    rocksdb_readoptions_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_readoptions_get_read_tier(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_tailing(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_readoptions_get_tailing(
    rocksdb_readoptions_t*);
// The functionality that this option controlled has been removed.
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_managed(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_readahead_size(
    rocksdb_readoptions_t*, size_t);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_readoptions_get_readahead_size(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_prefix_same_as_start(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_readoptions_get_prefix_same_as_start(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_pin_data(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_readoptions_get_pin_data(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_total_order_seek(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_readoptions_get_total_order_seek(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_readoptions_set_max_skippable_internal_keys(rocksdb_readoptions_t*,
                                                    uint64_t);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_readoptions_get_max_skippable_internal_keys(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_readoptions_set_background_purge_on_iterator_cleanup(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_readoptions_get_background_purge_on_iterator_cleanup(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_ignore_range_deletions(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_readoptions_get_ignore_range_deletions(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_deadline(
    rocksdb_readoptions_t*, uint64_t microseconds);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_readoptions_get_deadline(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_io_timeout(
    rocksdb_readoptions_t*, uint64_t microseconds);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_readoptions_get_io_timeout(rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_async_io(
    rocksdb_readoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_readoptions_get_async_io(
    rocksdb_readoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_timestamp(
    rocksdb_readoptions_t*, const char* ts, size_t tslen);
extern ROCKSDB_LIBRARY_API void rocksdb_readoptions_set_iter_start_ts(
    rocksdb_readoptions_t*, const char* ts, size_t tslen);

/* Write options */

extern ROCKSDB_LIBRARY_API rocksdb_writeoptions_t* rocksdb_writeoptions_create(
    void);
extern ROCKSDB_LIBRARY_API void rocksdb_writeoptions_destroy(
    rocksdb_writeoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writeoptions_set_sync(
    rocksdb_writeoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_writeoptions_get_sync(
    rocksdb_writeoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writeoptions_disable_WAL(
    rocksdb_writeoptions_t* opt, int disable);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_writeoptions_get_disable_WAL(
    rocksdb_writeoptions_t* opt);
extern ROCKSDB_LIBRARY_API void
rocksdb_writeoptions_set_ignore_missing_column_families(rocksdb_writeoptions_t*,
                                                        unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_writeoptions_get_ignore_missing_column_families(
    rocksdb_writeoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writeoptions_set_no_slowdown(
    rocksdb_writeoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_writeoptions_get_no_slowdown(
    rocksdb_writeoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_writeoptions_set_low_pri(
    rocksdb_writeoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_writeoptions_get_low_pri(
    rocksdb_writeoptions_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_writeoptions_set_memtable_insert_hint_per_batch(rocksdb_writeoptions_t*,
                                                        unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_writeoptions_get_memtable_insert_hint_per_batch(
    rocksdb_writeoptions_t*);

/* Compact range options */

extern ROCKSDB_LIBRARY_API rocksdb_compactoptions_t*
rocksdb_compactoptions_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_compactoptions_destroy(
    rocksdb_compactoptions_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_compactoptions_set_exclusive_manual_compaction(
    rocksdb_compactoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_compactoptions_get_exclusive_manual_compaction(
    rocksdb_compactoptions_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_compactoptions_set_bottommost_level_compaction(
    rocksdb_compactoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_compactoptions_get_bottommost_level_compaction(
    rocksdb_compactoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_compactoptions_set_change_level(
    rocksdb_compactoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char
rocksdb_compactoptions_get_change_level(rocksdb_compactoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_compactoptions_set_target_level(
    rocksdb_compactoptions_t*, int);
extern ROCKSDB_LIBRARY_API int rocksdb_compactoptions_get_target_level(
    rocksdb_compactoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_compactoptions_set_full_history_ts_low(
    rocksdb_compactoptions_t*, char* ts, size_t tslen);

/* Flush options */

extern ROCKSDB_LIBRARY_API rocksdb_flushoptions_t* rocksdb_flushoptions_create(
    void);
extern ROCKSDB_LIBRARY_API void rocksdb_flushoptions_destroy(
    rocksdb_flushoptions_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_flushoptions_set_wait(
    rocksdb_flushoptions_t*, unsigned char);
extern ROCKSDB_LIBRARY_API unsigned char rocksdb_flushoptions_get_wait(
    rocksdb_flushoptions_t*);

/* Memory allocator */

extern ROCKSDB_LIBRARY_API rocksdb_memory_allocator_t*
rocksdb_jemalloc_nodump_allocator_create(char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_memory_allocator_destroy(
    rocksdb_memory_allocator_t*);

/* Cache */

extern ROCKSDB_LIBRARY_API rocksdb_lru_cache_options_t*
rocksdb_lru_cache_options_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_lru_cache_options_destroy(
    rocksdb_lru_cache_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_lru_cache_options_set_capacity(
    rocksdb_lru_cache_options_t*, size_t);
extern ROCKSDB_LIBRARY_API void rocksdb_lru_cache_options_set_num_shard_bits(
    rocksdb_lru_cache_options_t*, int);
extern ROCKSDB_LIBRARY_API void rocksdb_lru_cache_options_set_memory_allocator(
    rocksdb_lru_cache_options_t*, rocksdb_memory_allocator_t*);

extern ROCKSDB_LIBRARY_API rocksdb_cache_t* rocksdb_cache_create_lru(
    size_t capacity);
extern ROCKSDB_LIBRARY_API rocksdb_cache_t*
rocksdb_cache_create_lru_with_strict_capacity_limit(size_t capacity);
extern ROCKSDB_LIBRARY_API rocksdb_cache_t* rocksdb_cache_create_lru_opts(
    rocksdb_lru_cache_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_cache_destroy(rocksdb_cache_t* cache);
extern ROCKSDB_LIBRARY_API void rocksdb_cache_disown_data(
    rocksdb_cache_t* cache);
extern ROCKSDB_LIBRARY_API void rocksdb_cache_set_capacity(
    rocksdb_cache_t* cache, size_t capacity);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_cache_get_capacity(rocksdb_cache_t* cache);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_cache_get_usage(rocksdb_cache_t* cache);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_cache_get_pinned_usage(rocksdb_cache_t* cache);

/* DBPath */

extern ROCKSDB_LIBRARY_API rocksdb_dbpath_t* rocksdb_dbpath_create(
    const char* path, uint64_t target_size);
extern ROCKSDB_LIBRARY_API void rocksdb_dbpath_destroy(rocksdb_dbpath_t*);

/* Env */

extern ROCKSDB_LIBRARY_API rocksdb_env_t* rocksdb_create_default_env(void);
extern ROCKSDB_LIBRARY_API rocksdb_env_t* rocksdb_create_mem_env(void);
extern ROCKSDB_LIBRARY_API void rocksdb_env_set_background_threads(
    rocksdb_env_t* env, int n);
extern ROCKSDB_LIBRARY_API int rocksdb_env_get_background_threads(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void
rocksdb_env_set_high_priority_background_threads(rocksdb_env_t* env, int n);
extern ROCKSDB_LIBRARY_API int rocksdb_env_get_high_priority_background_threads(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void rocksdb_env_set_low_priority_background_threads(
    rocksdb_env_t* env, int n);
extern ROCKSDB_LIBRARY_API int rocksdb_env_get_low_priority_background_threads(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void
rocksdb_env_set_bottom_priority_background_threads(rocksdb_env_t* env, int n);
extern ROCKSDB_LIBRARY_API int
rocksdb_env_get_bottom_priority_background_threads(rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void rocksdb_env_join_all_threads(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void rocksdb_env_lower_thread_pool_io_priority(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void
rocksdb_env_lower_high_priority_thread_pool_io_priority(rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void rocksdb_env_lower_thread_pool_cpu_priority(
    rocksdb_env_t* env);
extern ROCKSDB_LIBRARY_API void
rocksdb_env_lower_high_priority_thread_pool_cpu_priority(rocksdb_env_t* env);

extern ROCKSDB_LIBRARY_API void rocksdb_env_destroy(rocksdb_env_t*);

extern ROCKSDB_LIBRARY_API rocksdb_envoptions_t* rocksdb_envoptions_create(
    void);
extern ROCKSDB_LIBRARY_API void rocksdb_envoptions_destroy(
    rocksdb_envoptions_t* opt);
extern ROCKSDB_LIBRARY_API void rocksdb_create_dir_if_missing(
    rocksdb_env_t* env, const char* path, char** errptr);

/* SstFile */

extern ROCKSDB_LIBRARY_API rocksdb_sstfilewriter_t*
rocksdb_sstfilewriter_create(const rocksdb_envoptions_t* env,
                             const rocksdb_options_t* io_options);
extern ROCKSDB_LIBRARY_API rocksdb_sstfilewriter_t*
rocksdb_sstfilewriter_create_with_comparator(
    const rocksdb_envoptions_t* env, const rocksdb_options_t* io_options,
    const rocksdb_comparator_t* comparator);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_open(
    rocksdb_sstfilewriter_t* writer, const char* name, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_add(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* val, size_t vallen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_put(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* val, size_t vallen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_put_with_ts(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* ts, size_t tslen, const char* val, size_t vallen,
    char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_merge(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* val, size_t vallen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_delete(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_delete_with_ts(
    rocksdb_sstfilewriter_t* writer, const char* key, size_t keylen,
    const char* ts, size_t tslen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_delete_range(
    rocksdb_sstfilewriter_t* writer, const char* begin_key, size_t begin_keylen,
    const char* end_key, size_t end_keylen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_finish(
    rocksdb_sstfilewriter_t* writer, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_file_size(
    rocksdb_sstfilewriter_t* writer, uint64_t* file_size);
extern ROCKSDB_LIBRARY_API void rocksdb_sstfilewriter_destroy(
    rocksdb_sstfilewriter_t* writer);
extern ROCKSDB_LIBRARY_API rocksdb_ingestexternalfileoptions_t*
rocksdb_ingestexternalfileoptions_create(void);
extern ROCKSDB_LIBRARY_API void
rocksdb_ingestexternalfileoptions_set_move_files(
    rocksdb_ingestexternalfileoptions_t* opt, unsigned char move_files);
extern ROCKSDB_LIBRARY_API void
rocksdb_ingestexternalfileoptions_set_snapshot_consistency(
    rocksdb_ingestexternalfileoptions_t* opt,
    unsigned char snapshot_consistency);
extern ROCKSDB_LIBRARY_API void
rocksdb_ingestexternalfileoptions_set_allow_global_seqno(
    rocksdb_ingestexternalfileoptions_t* opt, unsigned char allow_global_seqno);
extern ROCKSDB_LIBRARY_API void
rocksdb_ingestexternalfileoptions_set_allow_blocking_flush(
    rocksdb_ingestexternalfileoptions_t* opt,
    unsigned char allow_blocking_flush);
extern ROCKSDB_LIBRARY_API void
rocksdb_ingestexternalfileoptions_set_ingest_behind(
    rocksdb_ingestexternalfileoptions_t* opt, unsigned char ingest_behind);
extern ROCKSDB_LIBRARY_API void rocksdb_ingestexternalfileoptions_destroy(
    rocksdb_ingestexternalfileoptions_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_ingest_external_file(
    rocksdb_t* db, const char* const* file_list, const size_t list_len,
    const rocksdb_ingestexternalfileoptions_t* opt, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_ingest_external_file_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* handle,
    const char* const* file_list, const size_t list_len,
    const rocksdb_ingestexternalfileoptions_t* opt, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_try_catch_up_with_primary(
    rocksdb_t* db, char** errptr);

/* SliceTransform */

extern ROCKSDB_LIBRARY_API rocksdb_slicetransform_t*
rocksdb_slicetransform_create(
    void* state, void (*destructor)(void*),
    char* (*transform)(void*, const char* key, size_t length,
                       size_t* dst_length),
    unsigned char (*in_domain)(void*, const char* key, size_t length),
    unsigned char (*in_range)(void*, const char* key, size_t length),
    const char* (*name)(void*));
extern ROCKSDB_LIBRARY_API rocksdb_slicetransform_t*
    rocksdb_slicetransform_create_fixed_prefix(size_t);
extern ROCKSDB_LIBRARY_API rocksdb_slicetransform_t*
rocksdb_slicetransform_create_noop(void);
extern ROCKSDB_LIBRARY_API void rocksdb_slicetransform_destroy(
    rocksdb_slicetransform_t*);

/* Universal Compaction options */

enum {
  rocksdb_similar_size_compaction_stop_style = 0,
  rocksdb_total_size_compaction_stop_style = 1
};

extern ROCKSDB_LIBRARY_API rocksdb_universal_compaction_options_t*
rocksdb_universal_compaction_options_create(void);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_size_ratio(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_size_ratio(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_min_merge_width(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_min_merge_width(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_max_merge_width(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_max_merge_width(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_max_size_amplification_percent(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_max_size_amplification_percent(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_compression_size_percent(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_compression_size_percent(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void
rocksdb_universal_compaction_options_set_stop_style(
    rocksdb_universal_compaction_options_t*, int);
extern ROCKSDB_LIBRARY_API int
rocksdb_universal_compaction_options_get_stop_style(
    rocksdb_universal_compaction_options_t*);
extern ROCKSDB_LIBRARY_API void rocksdb_universal_compaction_options_destroy(
    rocksdb_universal_compaction_options_t*);

extern ROCKSDB_LIBRARY_API rocksdb_fifo_compaction_options_t*
rocksdb_fifo_compaction_options_create(void);
extern ROCKSDB_LIBRARY_API void
rocksdb_fifo_compaction_options_set_max_table_files_size(
    rocksdb_fifo_compaction_options_t* fifo_opts, uint64_t size);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_fifo_compaction_options_get_max_table_files_size(
    rocksdb_fifo_compaction_options_t* fifo_opts);
extern ROCKSDB_LIBRARY_API void rocksdb_fifo_compaction_options_destroy(
    rocksdb_fifo_compaction_options_t* fifo_opts);

extern ROCKSDB_LIBRARY_API int rocksdb_livefiles_count(
    const rocksdb_livefiles_t*);
extern ROCKSDB_LIBRARY_API const char* rocksdb_livefiles_column_family_name(
    const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API const char* rocksdb_livefiles_name(
    const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API int rocksdb_livefiles_level(
    const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API size_t
rocksdb_livefiles_size(const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API const char* rocksdb_livefiles_smallestkey(
    const rocksdb_livefiles_t*, int index, size_t* size);
extern ROCKSDB_LIBRARY_API const char* rocksdb_livefiles_largestkey(
    const rocksdb_livefiles_t*, int index, size_t* size);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_livefiles_entries(const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_livefiles_deletions(const rocksdb_livefiles_t*, int index);
extern ROCKSDB_LIBRARY_API void rocksdb_livefiles_destroy(
    const rocksdb_livefiles_t*);

/* Utility Helpers */

extern ROCKSDB_LIBRARY_API void rocksdb_get_options_from_string(
    const rocksdb_options_t* base_options, const char* opts_str,
    rocksdb_options_t* new_options, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_file_in_range(
    rocksdb_t* db, const char* start_key, size_t start_key_len,
    const char* limit_key, size_t limit_key_len, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_delete_file_in_range_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family,
    const char* start_key, size_t start_key_len, const char* limit_key,
    size_t limit_key_len, char** errptr);

/* MetaData */

extern ROCKSDB_LIBRARY_API rocksdb_column_family_metadata_t*
rocksdb_get_column_family_metadata(rocksdb_t* db);

/**
 * Returns the rocksdb_column_family_metadata_t of the specified
 * column family.
 *
 * Note that the caller is responsible to release the returned memory
 * using rocksdb_column_family_metadata_destroy.
 */
extern ROCKSDB_LIBRARY_API rocksdb_column_family_metadata_t*
rocksdb_get_column_family_metadata_cf(
    rocksdb_t* db, rocksdb_column_family_handle_t* column_family);

extern ROCKSDB_LIBRARY_API void rocksdb_column_family_metadata_destroy(
    rocksdb_column_family_metadata_t* cf_meta);

extern ROCKSDB_LIBRARY_API uint64_t rocksdb_column_family_metadata_get_size(
    rocksdb_column_family_metadata_t* cf_meta);

extern ROCKSDB_LIBRARY_API size_t rocksdb_column_family_metadata_get_file_count(
    rocksdb_column_family_metadata_t* cf_meta);

extern ROCKSDB_LIBRARY_API char* rocksdb_column_family_metadata_get_name(
    rocksdb_column_family_metadata_t* cf_meta);

extern ROCKSDB_LIBRARY_API size_t
rocksdb_column_family_metadata_get_level_count(
    rocksdb_column_family_metadata_t* cf_meta);

/**
 * Returns the rocksdb_level_metadata_t of the ith level from the specified
 * column family metadata.
 *
 * If the specified i is greater than or equal to the number of levels
 * in the specified column family, then NULL will be returned.
 *
 * Note that the caller is responsible to release the returned memory
 * using rocksdb_level_metadata_destroy before releasing its parent
 * rocksdb_column_family_metadata_t.
 */
extern ROCKSDB_LIBRARY_API rocksdb_level_metadata_t*
rocksdb_column_family_metadata_get_level_metadata(
    rocksdb_column_family_metadata_t* cf_meta, size_t i);

/**
 * Releases the specified rocksdb_level_metadata_t.
 *
 * Note that the specified rocksdb_level_metadata_t must be released
 * before the release of its parent rocksdb_column_family_metadata_t.
 */
extern ROCKSDB_LIBRARY_API void rocksdb_level_metadata_destroy(
    rocksdb_level_metadata_t* level_meta);

extern ROCKSDB_LIBRARY_API int rocksdb_level_metadata_get_level(
    rocksdb_level_metadata_t* level_meta);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_level_metadata_get_size(rocksdb_level_metadata_t* level_meta);

extern ROCKSDB_LIBRARY_API size_t
rocksdb_level_metadata_get_file_count(rocksdb_level_metadata_t* level_meta);

/**
 * Returns the sst_file_metadata_t of the ith file from the specified level
 * metadata.
 *
 * If the specified i is greater than or equal to the number of files
 * in the specified level, then NULL will be returned.
 *
 * Note that the caller is responsible to release the returned memory
 * using rocksdb_sst_file_metadata_destroy before releasing its
 * parent rocksdb_level_metadata_t.
 */
extern ROCKSDB_LIBRARY_API rocksdb_sst_file_metadata_t*
rocksdb_level_metadata_get_sst_file_metadata(
    rocksdb_level_metadata_t* level_meta, size_t i);

/**
 * Releases the specified rocksdb_sst_file_metadata_t.
 *
 * Note that the specified rocksdb_sst_file_metadata_t must be released
 * before the release of its parent rocksdb_level_metadata_t.
 */
extern ROCKSDB_LIBRARY_API void rocksdb_sst_file_metadata_destroy(
    rocksdb_sst_file_metadata_t* file_meta);

extern ROCKSDB_LIBRARY_API char*
rocksdb_sst_file_metadata_get_relative_filename(
    rocksdb_sst_file_metadata_t* file_meta);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_sst_file_metadata_get_size(rocksdb_sst_file_metadata_t* file_meta);

/**
 * Returns the smallest key of the specified sst file.
 * The caller is responsible for releasing the returned memory.
 *
 * @param file_meta the metadata of an SST file to obtain its smallest key.
 * @param len the out value which will contain the length of the returned key
 *     after the function call.
 */
extern ROCKSDB_LIBRARY_API char* rocksdb_sst_file_metadata_get_smallestkey(
    rocksdb_sst_file_metadata_t* file_meta, size_t* len);

/**
 * Returns the smallest key of the specified sst file.
 * The caller is responsible for releasing the returned memory.
 *
 * @param file_meta the metadata of an SST file to obtain its smallest key.
 * @param len the out value which will contain the length of the returned key
 *     after the function call.
 */
extern ROCKSDB_LIBRARY_API char* rocksdb_sst_file_metadata_get_largestkey(
    rocksdb_sst_file_metadata_t* file_meta, size_t* len);

/* Transactions */

extern ROCKSDB_LIBRARY_API rocksdb_column_family_handle_t*
rocksdb_transactiondb_create_column_family(
    rocksdb_transactiondb_t* txn_db,
    const rocksdb_options_t* column_family_options,
    const char* column_family_name, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_transactiondb_t* rocksdb_transactiondb_open(
    const rocksdb_options_t* options,
    const rocksdb_transactiondb_options_t* txn_db_options, const char* name,
    char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_transactiondb_t*
rocksdb_transactiondb_open_column_families(
    const rocksdb_options_t* options,
    const rocksdb_transactiondb_options_t* txn_db_options, const char* name,
    int num_column_families, const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, char** errptr);

extern ROCKSDB_LIBRARY_API const rocksdb_snapshot_t*
rocksdb_transactiondb_create_snapshot(rocksdb_transactiondb_t* txn_db);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_release_snapshot(
    rocksdb_transactiondb_t* txn_db, const rocksdb_snapshot_t* snapshot);

extern ROCKSDB_LIBRARY_API char* rocksdb_transactiondb_property_value(
    rocksdb_transactiondb_t* db, const char* propname);

extern ROCKSDB_LIBRARY_API int rocksdb_transactiondb_property_int(
    rocksdb_transactiondb_t* db, const char* propname, uint64_t* out_val);

extern ROCKSDB_LIBRARY_API rocksdb_transaction_t* rocksdb_transaction_begin(
    rocksdb_transactiondb_t* txn_db,
    const rocksdb_writeoptions_t* write_options,
    const rocksdb_transaction_options_t* txn_options,
    rocksdb_transaction_t* old_txn);

extern ROCKSDB_LIBRARY_API rocksdb_transaction_t**
rocksdb_transactiondb_get_prepared_transactions(rocksdb_transactiondb_t* txn_db,
                                                size_t* cnt);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_set_name(
    rocksdb_transaction_t* txn, const char* name, size_t name_len,
    char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_transaction_get_name(
    rocksdb_transaction_t* txn, size_t* name_len);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_prepare(
    rocksdb_transaction_t* txn, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_commit(
    rocksdb_transaction_t* txn, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_rollback(
    rocksdb_transaction_t* txn, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_set_savepoint(
    rocksdb_transaction_t* txn);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_rollback_to_savepoint(
    rocksdb_transaction_t* txn, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_destroy(
    rocksdb_transaction_t* txn);

extern ROCKSDB_LIBRARY_API rocksdb_writebatch_wi_t*
rocksdb_transaction_get_writebatch_wi(rocksdb_transaction_t* txn);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_rebuild_from_writebatch(
    rocksdb_transaction_t* txn, rocksdb_writebatch_t* writebatch,
    char** errptr);

// This rocksdb_writebatch_wi_t should be freed with rocksdb_free
extern ROCKSDB_LIBRARY_API void rocksdb_transaction_rebuild_from_writebatch_wi(
    rocksdb_transaction_t* txn, rocksdb_writebatch_wi_t* wi, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_set_commit_timestamp(
    rocksdb_transaction_t* txn, uint64_t commit_timestamp);

extern ROCKSDB_LIBRARY_API void
rocksdb_transaction_set_read_timestamp_for_validation(
    rocksdb_transaction_t* txn, uint64_t read_timestamp);

// This snapshot should be freed using rocksdb_free
extern ROCKSDB_LIBRARY_API const rocksdb_snapshot_t*
rocksdb_transaction_get_snapshot(rocksdb_transaction_t* txn);

extern ROCKSDB_LIBRARY_API char* rocksdb_transaction_get(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    const char* key, size_t klen, size_t* vlen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transaction_get_pinned(rocksdb_transaction_t* txn,
                               const rocksdb_readoptions_t* options,
                               const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_transaction_get_cf(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key, size_t klen,
    size_t* vlen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transaction_get_pinned_cf(rocksdb_transaction_t* txn,
                                  const rocksdb_readoptions_t* options,
                                  rocksdb_column_family_handle_t* column_family,
                                  const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_transaction_get_for_update(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    const char* key, size_t klen, size_t* vlen, unsigned char exclusive,
    char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transaction_get_pinned_for_update(rocksdb_transaction_t* txn,
                                          const rocksdb_readoptions_t* options,
                                          const char* key, size_t klen,
                                          unsigned char exclusive,
                                          char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_transaction_get_for_update_cf(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key, size_t klen,
    size_t* vlen, unsigned char exclusive, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transaction_get_pinned_for_update_cf(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key, size_t klen,
    unsigned char exclusive, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_multi_get(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_multi_get_cf(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    const rocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API char* rocksdb_transactiondb_get(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    const char* key, size_t klen, size_t* vlen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transactiondb_get_pinned(rocksdb_transactiondb_t* txn_db,
                                 const rocksdb_readoptions_t* options,
                                 const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API char* rocksdb_transactiondb_get_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, size_t* vallen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t*
rocksdb_transactiondb_get_pinned_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_multi_get(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_multi_get_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    const rocksdb_column_family_handle_t* const* column_families,
    size_t num_keys, const char* const* keys_list,
    const size_t* keys_list_sizes, char** values_list,
    size_t* values_list_sizes, char** errs);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_put(
    rocksdb_transaction_t* txn, const char* key, size_t klen, const char* val,
    size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_put_cf(
    rocksdb_transaction_t* txn, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_put(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    const char* key, size_t klen, const char* val, size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_put_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, const char* val, size_t vallen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_write(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    rocksdb_writebatch_t* batch, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_merge(
    rocksdb_transaction_t* txn, const char* key, size_t klen, const char* val,
    size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_merge_cf(
    rocksdb_transaction_t* txn, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, const char* val, size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_merge(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    const char* key, size_t klen, const char* val, size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_merge_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key, size_t klen,
    const char* val, size_t vlen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_delete(
    rocksdb_transaction_t* txn, const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_delete_cf(
    rocksdb_transaction_t* txn, rocksdb_column_family_handle_t* column_family,
    const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_delete(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    const char* key, size_t klen, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_delete_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_writeoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_transaction_create_iterator(rocksdb_transaction_t* txn,
                                    const rocksdb_readoptions_t* options);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_transaction_create_iterator_cf(
    rocksdb_transaction_t* txn, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_transactiondb_create_iterator(rocksdb_transactiondb_t* txn_db,
                                      const rocksdb_readoptions_t* options);

extern ROCKSDB_LIBRARY_API rocksdb_iterator_t*
rocksdb_transactiondb_create_iterator_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_close(
    rocksdb_transactiondb_t* txn_db);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_flush(
    rocksdb_transactiondb_t* txn_db, const rocksdb_flushoptions_t* options,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_flush_cf(
    rocksdb_transactiondb_t* txn_db, const rocksdb_flushoptions_t* options,
    rocksdb_column_family_handle_t* column_family, char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_flush_wal(
    rocksdb_transactiondb_t* txn_db, unsigned char sync, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_checkpoint_t*
rocksdb_transactiondb_checkpoint_object_create(rocksdb_transactiondb_t* txn_db,
                                               char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_optimistictransactiondb_t*
rocksdb_optimistictransactiondb_open(const rocksdb_options_t* options,
                                     const char* name, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_optimistictransactiondb_t*
rocksdb_optimistictransactiondb_open_column_families(
    const rocksdb_options_t* options, const char* name, int num_column_families,
    const char* const* column_family_names,
    const rocksdb_options_t* const* column_family_options,
    rocksdb_column_family_handle_t** column_family_handles, char** errptr);

extern ROCKSDB_LIBRARY_API rocksdb_t*
rocksdb_optimistictransactiondb_get_base_db(
    rocksdb_optimistictransactiondb_t* otxn_db);

extern ROCKSDB_LIBRARY_API void rocksdb_optimistictransactiondb_close_base_db(
    rocksdb_t* base_db);

extern ROCKSDB_LIBRARY_API rocksdb_transaction_t*
rocksdb_optimistictransaction_begin(
    rocksdb_optimistictransactiondb_t* otxn_db,
    const rocksdb_writeoptions_t* write_options,
    const rocksdb_optimistictransaction_options_t* otxn_options,
    rocksdb_transaction_t* old_txn);

extern ROCKSDB_LIBRARY_API void rocksdb_optimistictransactiondb_write(
    rocksdb_optimistictransactiondb_t* otxn_db,
    const rocksdb_writeoptions_t* options, rocksdb_writebatch_t* batch,
    char** errptr);

extern ROCKSDB_LIBRARY_API void rocksdb_optimistictransactiondb_close(
    rocksdb_optimistictransactiondb_t* otxn_db);

extern ROCKSDB_LIBRARY_API rocksdb_checkpoint_t*
rocksdb_optimistictransactiondb_checkpoint_object_create(
    rocksdb_optimistictransactiondb_t* otxn_db, char** errptr);

/* Transaction Options */

extern ROCKSDB_LIBRARY_API rocksdb_transactiondb_options_t*
rocksdb_transactiondb_options_create(void);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_options_destroy(
    rocksdb_transactiondb_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_options_set_max_num_locks(
    rocksdb_transactiondb_options_t* opt, int64_t max_num_locks);

extern ROCKSDB_LIBRARY_API void rocksdb_transactiondb_options_set_num_stripes(
    rocksdb_transactiondb_options_t* opt, size_t num_stripes);

extern ROCKSDB_LIBRARY_API void
rocksdb_transactiondb_options_set_transaction_lock_timeout(
    rocksdb_transactiondb_options_t* opt, int64_t txn_lock_timeout);

extern ROCKSDB_LIBRARY_API void
rocksdb_transactiondb_options_set_default_lock_timeout(
    rocksdb_transactiondb_options_t* opt, int64_t default_lock_timeout);

extern ROCKSDB_LIBRARY_API rocksdb_transaction_options_t*
rocksdb_transaction_options_create(void);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_destroy(
    rocksdb_transaction_options_t* opt);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_set_set_snapshot(
    rocksdb_transaction_options_t* opt, unsigned char v);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_set_deadlock_detect(
    rocksdb_transaction_options_t* opt, unsigned char v);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_set_lock_timeout(
    rocksdb_transaction_options_t* opt, int64_t lock_timeout);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_set_expiration(
    rocksdb_transaction_options_t* opt, int64_t expiration);

extern ROCKSDB_LIBRARY_API void
rocksdb_transaction_options_set_deadlock_detect_depth(
    rocksdb_transaction_options_t* opt, int64_t depth);

extern ROCKSDB_LIBRARY_API void
rocksdb_transaction_options_set_max_write_batch_size(
    rocksdb_transaction_options_t* opt, size_t size);

extern ROCKSDB_LIBRARY_API void rocksdb_transaction_options_set_skip_prepare(
    rocksdb_transaction_options_t* opt, unsigned char v);

extern ROCKSDB_LIBRARY_API rocksdb_optimistictransaction_options_t*
rocksdb_optimistictransaction_options_create(void);

extern ROCKSDB_LIBRARY_API void rocksdb_optimistictransaction_options_destroy(
    rocksdb_optimistictransaction_options_t* opt);

extern ROCKSDB_LIBRARY_API void
rocksdb_optimistictransaction_options_set_set_snapshot(
    rocksdb_optimistictransaction_options_t* opt, unsigned char v);

extern ROCKSDB_LIBRARY_API char* rocksdb_optimistictransactiondb_property_value(
    rocksdb_optimistictransactiondb_t* db, const char* propname);

extern ROCKSDB_LIBRARY_API int rocksdb_optimistictransactiondb_property_int(
    rocksdb_optimistictransactiondb_t* db, const char* propname,
    uint64_t* out_val);

// referring to convention (3), this should be used by client
// to free memory that was malloc()ed
extern ROCKSDB_LIBRARY_API void rocksdb_free(void* ptr);

extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t* rocksdb_get_pinned(
    rocksdb_t* db, const rocksdb_readoptions_t* options, const char* key,
    size_t keylen, char** errptr);
extern ROCKSDB_LIBRARY_API rocksdb_pinnableslice_t* rocksdb_get_pinned_cf(
    rocksdb_t* db, const rocksdb_readoptions_t* options,
    rocksdb_column_family_handle_t* column_family, const char* key,
    size_t keylen, char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_pinnableslice_destroy(
    rocksdb_pinnableslice_t* v);
extern ROCKSDB_LIBRARY_API const char* rocksdb_pinnableslice_value(
    const rocksdb_pinnableslice_t* t, size_t* vlen);

extern ROCKSDB_LIBRARY_API rocksdb_memory_consumers_t*
rocksdb_memory_consumers_create(void);
extern ROCKSDB_LIBRARY_API void rocksdb_memory_consumers_add_db(
    rocksdb_memory_consumers_t* consumers, rocksdb_t* db);
extern ROCKSDB_LIBRARY_API void rocksdb_memory_consumers_add_cache(
    rocksdb_memory_consumers_t* consumers, rocksdb_cache_t* cache);
extern ROCKSDB_LIBRARY_API void rocksdb_memory_consumers_destroy(
    rocksdb_memory_consumers_t* consumers);
extern ROCKSDB_LIBRARY_API rocksdb_memory_usage_t*
rocksdb_approximate_memory_usage_create(rocksdb_memory_consumers_t* consumers,
                                        char** errptr);
extern ROCKSDB_LIBRARY_API void rocksdb_approximate_memory_usage_destroy(
    rocksdb_memory_usage_t* usage);

extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_approximate_memory_usage_get_mem_table_total(
    rocksdb_memory_usage_t* memory_usage);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_approximate_memory_usage_get_mem_table_unflushed(
    rocksdb_memory_usage_t* memory_usage);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_approximate_memory_usage_get_mem_table_readers_total(
    rocksdb_memory_usage_t* memory_usage);
extern ROCKSDB_LIBRARY_API uint64_t
rocksdb_approximate_memory_usage_get_cache_total(
    rocksdb_memory_usage_t* memory_usage);

extern ROCKSDB_LIBRARY_API void rocksdb_options_set_dump_malloc_stats(
    rocksdb_options_t*, unsigned char);

extern ROCKSDB_LIBRARY_API void
rocksdb_options_set_memtable_whole_key_filtering(rocksdb_options_t*,
                                                 unsigned char);

extern ROCKSDB_LIBRARY_API void rocksdb_cancel_all_background_work(
    rocksdb_t* db, unsigned char wait);

extern ROCKSDB_LIBRARY_API void rocksdb_disable_manual_compaction(
    rocksdb_t* db);

extern ROCKSDB_LIBRARY_API void rocksdb_enable_manual_compaction(rocksdb_t* db);

#ifdef __cplusplus
} /* end extern "C" */
#endif
