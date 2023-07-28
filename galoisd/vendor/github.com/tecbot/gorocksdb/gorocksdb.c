#include "gorocksdb.h"
#include "_cgo_export.h"

/* Base */

void gorocksdb_destruct_handler(void* state) { }

/* Comparator */

rocksdb_comparator_t* gorocksdb_comparator_create(uintptr_t idx) {
    return rocksdb_comparator_create(
        (void*)idx,
        gorocksdb_destruct_handler,
        (int (*)(void*, const char*, size_t, const char*, size_t))(gorocksdb_comparator_compare),
        (const char *(*)(void*))(gorocksdb_comparator_name));
}

/* CompactionFilter */

rocksdb_compactionfilter_t* gorocksdb_compactionfilter_create(uintptr_t idx) {
    return rocksdb_compactionfilter_create(
        (void*)idx,
        gorocksdb_destruct_handler,
        (unsigned char (*)(void*, int, const char*, size_t, const char*, size_t, char**, size_t*, unsigned char*))(gorocksdb_compactionfilter_filter),
        (const char *(*)(void*))(gorocksdb_compactionfilter_name));
}

/* Filter Policy */

rocksdb_filterpolicy_t* gorocksdb_filterpolicy_create(uintptr_t idx) {
    return rocksdb_filterpolicy_create(
        (void*)idx,
        gorocksdb_destruct_handler,
        (char* (*)(void*, const char* const*, const size_t*, int, size_t*))(gorocksdb_filterpolicy_create_filter),
        (unsigned char (*)(void*, const char*, size_t, const char*, size_t))(gorocksdb_filterpolicy_key_may_match),
        gorocksdb_filterpolicy_delete_filter,
        (const char *(*)(void*))(gorocksdb_filterpolicy_name));
}

void gorocksdb_filterpolicy_delete_filter(void* state, const char* v, size_t s) {
    free((char*)v);
}

/* Merge Operator */

rocksdb_mergeoperator_t* gorocksdb_mergeoperator_create(uintptr_t idx) {
    return rocksdb_mergeoperator_create(
        (void*)idx,
        gorocksdb_destruct_handler,
        (char* (*)(void*, const char*, size_t, const char*, size_t, const char* const*, const size_t*, int, unsigned char*, size_t*))(gorocksdb_mergeoperator_full_merge),
        (char* (*)(void*, const char*, size_t, const char* const*, const size_t*, int, unsigned char*, size_t*))(gorocksdb_mergeoperator_partial_merge_multi),
        gorocksdb_mergeoperator_delete_value,
        (const char* (*)(void*))(gorocksdb_mergeoperator_name));
}

void gorocksdb_mergeoperator_delete_value(void* id, const char* v, size_t s) {
    free((char*)v);
}

/* Slice Transform */

rocksdb_slicetransform_t* gorocksdb_slicetransform_create(uintptr_t idx) {
    return rocksdb_slicetransform_create(
    	(void*)idx,
    	gorocksdb_destruct_handler,
    	(char* (*)(void*, const char*, size_t, size_t*))(gorocksdb_slicetransform_transform),
    	(unsigned char (*)(void*, const char*, size_t))(gorocksdb_slicetransform_in_domain),
    	(unsigned char (*)(void*, const char*, size_t))(gorocksdb_slicetransform_in_range),
    	(const char* (*)(void*))(gorocksdb_slicetransform_name));
}
