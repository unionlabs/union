package grocksdb

// #include "rocksdb/c.h"
import "C"
import "unsafe"

// ColumnFamilyMetadata contains metadata info of column family.
type ColumnFamilyMetadata struct {
	size       uint64
	fileCount  int
	name       string
	levelMetas []LevelMetadata
}

func newColumnFamilyMetadata(c *C.rocksdb_column_family_metadata_t) *ColumnFamilyMetadata {
	return &ColumnFamilyMetadata{
		size:       uint64(C.rocksdb_column_family_metadata_get_size(c)),
		fileCount:  int(C.rocksdb_column_family_metadata_get_file_count(c)),
		name:       C.GoString(C.rocksdb_column_family_metadata_get_name(c)),
		levelMetas: levelMetas(c),
	}
}

// GetSize returns size of this column family in bytes, which is equal to the sum of
// the file size of its "levels".
func (cm *ColumnFamilyMetadata) Size() uint64 {
	return cm.size
}

// FileCount returns number of files in this column family.
func (cm *ColumnFamilyMetadata) FileCount() int {
	return cm.fileCount
}

// Name returns name of this column family.
func (cm *ColumnFamilyMetadata) Name() string {
	return cm.name
}

// LevelMetas returns metadata(s) of each level.
func (cm *ColumnFamilyMetadata) LevelMetas() []LevelMetadata {
	return cm.levelMetas
}

// LevelMetadata represents the metadata that describes a level.
type LevelMetadata struct {
	level    int
	size     uint64
	sstMetas []SstMetadata
}

func levelMetas(c *C.rocksdb_column_family_metadata_t) []LevelMetadata {
	n := int(C.rocksdb_column_family_metadata_get_level_count(c))

	metas := make([]LevelMetadata, n)
	for i := range metas {
		lm := C.rocksdb_column_family_metadata_get_level_metadata(c, C.size_t(i))
		metas[i].level = int(C.rocksdb_level_metadata_get_level(lm))
		metas[i].size = uint64(C.rocksdb_level_metadata_get_size(lm))
		metas[i].sstMetas = sstMetas(lm)
	}

	C.rocksdb_column_family_metadata_destroy(c)

	return metas
}

// Level returns level value.
func (l *LevelMetadata) Level() int {
	return l.level
}

// Size returns the sum of the file size in this level.
func (l *LevelMetadata) Size() uint64 {
	return l.size
}

// SstMetas returns metadata(s) of sst-file(s) in this level.
func (l *LevelMetadata) SstMetas() []SstMetadata {
	return l.sstMetas
}

// SstMetadata represents metadata of sst file.
type SstMetadata struct {
	relativeFileName string
	directory        string
	size             uint64
	smallestKey      []byte
	largestKey       []byte
}

func sstMetas(c *C.rocksdb_level_metadata_t) []SstMetadata {
	n := int(C.rocksdb_level_metadata_get_file_count(c))

	metas := make([]SstMetadata, n)
	for i := range metas {
		sm := C.rocksdb_level_metadata_get_sst_file_metadata(c, C.size_t(i))
		metas[i].relativeFileName = C.GoString(C.rocksdb_sst_file_metadata_get_relative_filename(sm))
		metas[i].directory = C.GoString(C.rocksdb_sst_file_metadata_get_directory(sm))
		metas[i].size = uint64(C.rocksdb_sst_file_metadata_get_size(sm))

		var ln C.size_t
		sk := C.rocksdb_sst_file_metadata_get_smallestkey(sm, &ln)
		metas[i].smallestKey = C.GoBytes(unsafe.Pointer(sk), C.int(ln))
		C.rocksdb_free(unsafe.Pointer(sk))

		sk = C.rocksdb_sst_file_metadata_get_largestkey(sm, &ln)
		metas[i].largestKey = C.GoBytes(unsafe.Pointer(sk), C.int(ln))
		C.rocksdb_free(unsafe.Pointer(sk))

		C.rocksdb_sst_file_metadata_destroy(sm)
	}

	C.rocksdb_level_metadata_destroy(c)

	return metas
}

// RelativeFileName returns relative file name.
func (s *SstMetadata) RelativeFileName() string {
	return s.relativeFileName
}

// Size returns size of this sst file.
func (s *SstMetadata) Size() uint64 {
	return s.size
}

// SmallestKey returns smallest-key in this sst file.
func (s *SstMetadata) SmallestKey() []byte {
	return s.smallestKey
}

// LargestKey returns largest-key in this sst file.
func (s *SstMetadata) LargestKey() []byte {
	return s.largestKey
}
