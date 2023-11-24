package grocksdb

// #include "rocksdb/c.h"
import "C"

// IngestExternalFileOptions represents available options when ingesting external files.
type IngestExternalFileOptions struct {
	c *C.rocksdb_ingestexternalfileoptions_t
}

// NewDefaultIngestExternalFileOptions creates a default IngestExternalFileOptions object.
func NewDefaultIngestExternalFileOptions() *IngestExternalFileOptions {
	return newNativeIngestExternalFileOptions(C.rocksdb_ingestexternalfileoptions_create())
}

// NewNativeIngestExternalFileOptions creates a IngestExternalFileOptions object.
func newNativeIngestExternalFileOptions(c *C.rocksdb_ingestexternalfileoptions_t) *IngestExternalFileOptions {
	return &IngestExternalFileOptions{c: c}
}

// SetMoveFiles specifies if it should move the files instead of copying them.
// Default to false.
func (opts *IngestExternalFileOptions) SetMoveFiles(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_move_files(opts.c, boolToChar(flag))
}

// SetSnapshotConsistency if specifies the consistency.
// If set to false, an ingested file key could appear in existing snapshots that were created before the
// file was ingested.
// Default to true.
func (opts *IngestExternalFileOptions) SetSnapshotConsistency(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_snapshot_consistency(opts.c, boolToChar(flag))
}

// SetAllowGlobalSeqNo sets allow_global_seqno. If set to false,IngestExternalFile() will fail if the file key
// range overlaps with existing keys or tombstones in the DB.
// Default true.
func (opts *IngestExternalFileOptions) SetAllowGlobalSeqNo(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_allow_global_seqno(opts.c, boolToChar(flag))
}

// SetAllowBlockingFlush sets allow_blocking_flush. If set to false and the file key range overlaps with
// the memtable key range (memtable flush required), IngestExternalFile will fail.
// Default to true.
func (opts *IngestExternalFileOptions) SetAllowBlockingFlush(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_allow_blocking_flush(opts.c, boolToChar(flag))
}

// SetIngestionBehind sets ingest_behind
// Set to true if you would like duplicate keys in the file being ingested
// to be skipped rather than overwriting existing data under that key.
// Usecase: back-fill of some historical data in the database without
// over-writing existing newer version of data.
// This option could only be used if the DB has been running
// with allow_ingest_behind=true since the dawn of time.
// All files will be ingested at the bottommost level with seqno=0.
func (opts *IngestExternalFileOptions) SetIngestionBehind(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_ingest_behind(opts.c, boolToChar(flag))
}

// SetFailIfNotBottommostLevel sets to TRUE if user wants file to be ingested to the bottommost level. An
// error of Status::TryAgain() will be returned if a file cannot fit in the bottommost level when calling
// DB::IngestExternalFile()/DB::IngestExternalFiles().
//
// The user should clear the bottommost level in the overlapping range before re-attempt.
// Ingest_behind takes precedence over fail_if_not_bottommost_level.
func (opts *IngestExternalFileOptions) SetFailIfNotBottommostLevel(flag bool) {
	C.rocksdb_ingestexternalfileoptions_set_fail_if_not_bottommost_level(opts.c, boolToChar(flag))
}

// Destroy deallocates the IngestExternalFileOptions object.
func (opts *IngestExternalFileOptions) Destroy() {
	C.rocksdb_ingestexternalfileoptions_destroy(opts.c)
	opts.c = nil
}
