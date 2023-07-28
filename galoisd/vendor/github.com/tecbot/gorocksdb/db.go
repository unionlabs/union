package gorocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"
import (
	"errors"
	"fmt"
	"unsafe"
)

// Range is a range of keys in the database. GetApproximateSizes calls with it
// begin at the key Start and end right before the key Limit.
type Range struct {
	Start []byte
	Limit []byte
}

// DB is a reusable handle to a RocksDB database on disk, created by Open.
type DB struct {
	c    *C.rocksdb_t
	name string
	opts *Options
}

// OpenDb opens a database with the specified options.
func OpenDb(opts *Options, name string) (*DB, error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	db := C.rocksdb_open(opts.c, cName, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return &DB{
		name: name,
		c:    db,
		opts: opts,
	}, nil
}

// OpenDbWithTTL opens a database with TTL support with the specified options.
func OpenDbWithTTL(opts *Options, name string, ttl int) (*DB, error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	db := C.rocksdb_open_with_ttl(opts.c, cName, C.int(ttl), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return &DB{
		name: name,
		c:    db,
		opts: opts,
	}, nil
}

// OpenDbForReadOnly opens a database with the specified options for readonly usage.
func OpenDbForReadOnly(opts *Options, name string, errorIfLogFileExist bool) (*DB, error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	db := C.rocksdb_open_for_read_only(opts.c, cName, boolToChar(errorIfLogFileExist), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return &DB{
		name: name,
		c:    db,
		opts: opts,
	}, nil
}

// OpenDbColumnFamilies opens a database with the specified column families.
func OpenDbColumnFamilies(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
) (*DB, []*ColumnFamilyHandle, error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		return nil, nil, errors.New("must provide the same number of column family names and options")
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}
	defer func() {
		for _, s := range cNames {
			C.free(unsafe.Pointer(s))
		}
	}()

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	db := C.rocksdb_open_column_families(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&cErr,
	)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, nil, errors.New(C.GoString(cErr))
	}

	cfHandles := make([]*ColumnFamilyHandle, numColumnFamilies)
	for i, c := range cHandles {
		cfHandles[i] = NewNativeColumnFamilyHandle(c)
	}

	return &DB{
		name: name,
		c:    db,
		opts: opts,
	}, cfHandles, nil
}

// OpenDbForReadOnlyColumnFamilies opens a database with the specified column
// families in read only mode.
func OpenDbForReadOnlyColumnFamilies(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
	errorIfLogFileExist bool,
) (*DB, []*ColumnFamilyHandle, error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		return nil, nil, errors.New("must provide the same number of column family names and options")
	}

	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))

	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}
	defer func() {
		for _, s := range cNames {
			C.free(unsafe.Pointer(s))
		}
	}()

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	db := C.rocksdb_open_for_read_only_column_families(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		boolToChar(errorIfLogFileExist),
		&cErr,
	)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, nil, errors.New(C.GoString(cErr))
	}

	cfHandles := make([]*ColumnFamilyHandle, numColumnFamilies)
	for i, c := range cHandles {
		cfHandles[i] = NewNativeColumnFamilyHandle(c)
	}

	return &DB{
		name: name,
		c:    db,
		opts: opts,
	}, cfHandles, nil
}

// ListColumnFamilies lists the names of the column families in the DB.
func ListColumnFamilies(opts *Options, name string) ([]string, error) {
	var (
		cErr  *C.char
		cLen  C.size_t
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	cNames := C.rocksdb_list_column_families(opts.c, cName, &cLen, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	namesLen := int(cLen)
	names := make([]string, namesLen)
	// The maximum capacity of the following two slices is limited to (2^29)-1 to remain compatible
	// with 32-bit platforms. The size of a `*C.char` (a pointer) is 4 Byte on a 32-bit system
	// and (2^29)*4 == math.MaxInt32 + 1. -- See issue golang/go#13656
	cNamesArr := (*[(1 << 29) - 1]*C.char)(unsafe.Pointer(cNames))[:namesLen:namesLen]
	for i, n := range cNamesArr {
		names[i] = C.GoString(n)
	}
	C.rocksdb_list_column_families_destroy(cNames, cLen)
	return names, nil
}

// UnsafeGetDB returns the underlying c rocksdb instance.
func (db *DB) UnsafeGetDB() unsafe.Pointer {
	return unsafe.Pointer(db.c)
}

// Name returns the name of the database.
func (db *DB) Name() string {
	return db.name
}

// Get returns the data associated with the key from the database.
func (db *DB) Get(opts *ReadOptions, key []byte) (*Slice, error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = byteToChar(key)
	)
	cValue := C.rocksdb_get(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return NewSlice(cValue, cValLen), nil
}

// GetBytes is like Get but returns a copy of the data.
func (db *DB) GetBytes(opts *ReadOptions, key []byte) ([]byte, error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = byteToChar(key)
	)
	cValue := C.rocksdb_get(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	if cValue == nil {
		return nil, nil
	}
	defer C.rocksdb_free(unsafe.Pointer(cValue))
	return C.GoBytes(unsafe.Pointer(cValue), C.int(cValLen)), nil
}

// GetCF returns the data associated with the key from the database and column family.
func (db *DB) GetCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (*Slice, error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = byteToChar(key)
	)
	cValue := C.rocksdb_get_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return NewSlice(cValue, cValLen), nil
}

// GetPinned returns the data associated with the key from the database.
func (db *DB) GetPinned(opts *ReadOptions, key []byte) (*PinnableSliceHandle, error) {
	var (
		cErr *C.char
		cKey = byteToChar(key)
	)
	cHandle := C.rocksdb_get_pinned(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return NewNativePinnableSliceHandle(cHandle), nil
}

// MultiGet returns the data associated with the passed keys from the database
func (db *DB) MultiGet(opts *ReadOptions, keys ...[]byte) (Slices, error) {
	cKeys, cKeySizes := byteSlicesToCSlices(keys)
	defer cKeys.Destroy()
	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_multi_get(
		db.c,
		opts.c,
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if rocksErr != nil {
			defer C.rocksdb_free(unsafe.Pointer(rocksErr))
			err := fmt.Errorf("getting %q failed: %v", string(keys[i]), C.GoString(rocksErr))
			errs = append(errs, err)
		}
	}

	if len(errs) > 0 {
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	return slices, nil
}

// MultiGetCF returns the data associated with the passed keys from the column family
func (db *DB) MultiGetCF(opts *ReadOptions, cf *ColumnFamilyHandle, keys ...[]byte) (Slices, error) {
	cfs := make(ColumnFamilyHandles, len(keys))
	for i := 0; i < len(keys); i++ {
		cfs[i] = cf
	}
	return db.MultiGetCFMultiCF(opts, cfs, keys)
}

// MultiGetCFMultiCF returns the data associated with the passed keys and
// column families.
func (db *DB) MultiGetCFMultiCF(opts *ReadOptions, cfs ColumnFamilyHandles, keys [][]byte) (Slices, error) {
	cKeys, cKeySizes := byteSlicesToCSlices(keys)
	defer cKeys.Destroy()
	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_multi_get_cf(
		db.c,
		opts.c,
		cfs.toCSlice().c(),
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if rocksErr != nil {
			defer C.rocksdb_free(unsafe.Pointer(rocksErr))
			err := fmt.Errorf("getting %q failed: %v", string(keys[i]), C.GoString(rocksErr))
			errs = append(errs, err)
		}
	}

	if len(errs) > 0 {
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	return slices, nil
}

// Put writes data associated with a key to the database.
func (db *DB) Put(opts *WriteOptions, key, value []byte) error {
	var (
		cErr   *C.char
		cKey   = byteToChar(key)
		cValue = byteToChar(value)
	)
	C.rocksdb_put(db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// PutCF writes data associated with a key to the database and column family.
func (db *DB) PutCF(opts *WriteOptions, cf *ColumnFamilyHandle, key, value []byte) error {
	var (
		cErr   *C.char
		cKey   = byteToChar(key)
		cValue = byteToChar(value)
	)
	C.rocksdb_put_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// Delete removes the data associated with the key from the database.
func (db *DB) Delete(opts *WriteOptions, key []byte) error {
	var (
		cErr *C.char
		cKey = byteToChar(key)
	)
	C.rocksdb_delete(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// DeleteCF removes the data associated with the key from the database and column family.
func (db *DB) DeleteCF(opts *WriteOptions, cf *ColumnFamilyHandle, key []byte) error {
	var (
		cErr *C.char
		cKey = byteToChar(key)
	)
	C.rocksdb_delete_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// Merge merges the data associated with the key with the actual data in the database.
func (db *DB) Merge(opts *WriteOptions, key []byte, value []byte) error {
	var (
		cErr   *C.char
		cKey   = byteToChar(key)
		cValue = byteToChar(value)
	)
	C.rocksdb_merge(db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// MergeCF merges the data associated with the key with the actual data in the
// database and column family.
func (db *DB) MergeCF(opts *WriteOptions, cf *ColumnFamilyHandle, key []byte, value []byte) error {
	var (
		cErr   *C.char
		cKey   = byteToChar(key)
		cValue = byteToChar(value)
	)
	C.rocksdb_merge_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// Write writes a WriteBatch to the database
func (db *DB) Write(opts *WriteOptions, batch *WriteBatch) error {
	var cErr *C.char
	C.rocksdb_write(db.c, opts.c, batch.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// NewIterator returns an Iterator over the the database that uses the
// ReadOptions given.
func (db *DB) NewIterator(opts *ReadOptions) *Iterator {
	cIter := C.rocksdb_create_iterator(db.c, opts.c)
	return NewNativeIterator(unsafe.Pointer(cIter))
}

// NewIteratorCF returns an Iterator over the the database and column family
// that uses the ReadOptions given.
func (db *DB) NewIteratorCF(opts *ReadOptions, cf *ColumnFamilyHandle) *Iterator {
	cIter := C.rocksdb_create_iterator_cf(db.c, opts.c, cf.c)
	return NewNativeIterator(unsafe.Pointer(cIter))
}

func (db *DB) GetUpdatesSince(seqNumber uint64) (*WalIterator, error) {
	var cErr *C.char
	cIter := C.rocksdb_get_updates_since(db.c, C.uint64_t(seqNumber), nil, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return NewNativeWalIterator(unsafe.Pointer(cIter)), nil
}

func (db *DB) GetLatestSequenceNumber() uint64 {
	return uint64(C.rocksdb_get_latest_sequence_number(db.c))
}

// NewSnapshot creates a new snapshot of the database.
func (db *DB) NewSnapshot() *Snapshot {
	cSnap := C.rocksdb_create_snapshot(db.c)
	return NewNativeSnapshot(cSnap)
}

// ReleaseSnapshot releases the snapshot and its resources.
func (db *DB) ReleaseSnapshot(snapshot *Snapshot) {
	C.rocksdb_release_snapshot(db.c, snapshot.c)
	snapshot.c = nil
}

// GetProperty returns the value of a database property.
func (db *DB) GetProperty(propName string) string {
	cprop := C.CString(propName)
	defer C.free(unsafe.Pointer(cprop))
	cValue := C.rocksdb_property_value(db.c, cprop)
	defer C.rocksdb_free(unsafe.Pointer(cValue))
	return C.GoString(cValue)
}

// GetPropertyCF returns the value of a database property.
func (db *DB) GetPropertyCF(propName string, cf *ColumnFamilyHandle) string {
	cProp := C.CString(propName)
	defer C.free(unsafe.Pointer(cProp))
	cValue := C.rocksdb_property_value_cf(db.c, cf.c, cProp)
	defer C.rocksdb_free(unsafe.Pointer(cValue))
	return C.GoString(cValue)
}

// CreateColumnFamily create a new column family.
func (db *DB) CreateColumnFamily(opts *Options, name string) (*ColumnFamilyHandle, error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	cHandle := C.rocksdb_create_column_family(db.c, opts.c, cName, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}
	return NewNativeColumnFamilyHandle(cHandle), nil
}

// DropColumnFamily drops a column family.
func (db *DB) DropColumnFamily(c *ColumnFamilyHandle) error {
	var cErr *C.char
	C.rocksdb_drop_column_family(db.c, c.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// GetApproximateSizes returns the approximate number of bytes of file system
// space used by one or more key ranges.
//
// The keys counted will begin at Range.Start and end on the key before
// Range.Limit.
func (db *DB) GetApproximateSizes(ranges []Range) []uint64 {
	sizes := make([]uint64, len(ranges))
	if len(ranges) == 0 {
		return sizes
	}

	cStarts := make([]*C.char, len(ranges))
	cLimits := make([]*C.char, len(ranges))
	cStartLens := make([]C.size_t, len(ranges))
	cLimitLens := make([]C.size_t, len(ranges))
	for i, r := range ranges {
		cStarts[i] = (*C.char)(C.CBytes(r.Start))
		cStartLens[i] = C.size_t(len(r.Start))
		cLimits[i] = (*C.char)(C.CBytes(r.Limit))
		cLimitLens[i] = C.size_t(len(r.Limit))
	}

	defer func() {
		for i := range ranges {
			C.free(unsafe.Pointer(cStarts[i]))
			C.free(unsafe.Pointer(cLimits[i]))
		}
	}()

	C.rocksdb_approximate_sizes(
		db.c,
		C.int(len(ranges)),
		&cStarts[0],
		&cStartLens[0],
		&cLimits[0],
		&cLimitLens[0],
		(*C.uint64_t)(&sizes[0]))

	return sizes
}

// GetApproximateSizesCF returns the approximate number of bytes of file system
// space used by one or more key ranges in the column family.
//
// The keys counted will begin at Range.Start and end on the key before
// Range.Limit.
func (db *DB) GetApproximateSizesCF(cf *ColumnFamilyHandle, ranges []Range) []uint64 {
	sizes := make([]uint64, len(ranges))
	if len(ranges) == 0 {
		return sizes
	}

	cStarts := make([]*C.char, len(ranges))
	cLimits := make([]*C.char, len(ranges))
	cStartLens := make([]C.size_t, len(ranges))
	cLimitLens := make([]C.size_t, len(ranges))
	for i, r := range ranges {
		cStarts[i] = (*C.char)(C.CBytes(r.Start))
		cStartLens[i] = C.size_t(len(r.Start))
		cLimits[i] = (*C.char)(C.CBytes(r.Limit))
		cLimitLens[i] = C.size_t(len(r.Limit))
	}

	defer func() {
		for i := range ranges {
			C.free(unsafe.Pointer(cStarts[i]))
			C.free(unsafe.Pointer(cLimits[i]))
		}
	}()

	C.rocksdb_approximate_sizes_cf(
		db.c,
		cf.c,
		C.int(len(ranges)),
		&cStarts[0],
		&cStartLens[0],
		&cLimits[0],
		&cLimitLens[0],
		(*C.uint64_t)(&sizes[0]))

	return sizes
}

// SetOptions dynamically changes options through the SetOptions API.
func (db *DB) SetOptions(keys, values []string) error {
	num_keys := len(keys)

	if num_keys == 0 {
		return nil
	}

	cKeys := make([]*C.char, num_keys)
	cValues := make([]*C.char, num_keys)
	for i := range keys {
		cKeys[i] = C.CString(keys[i])
		cValues[i] = C.CString(values[i])
	}

	var cErr *C.char

	C.rocksdb_set_options(
		db.c,
		C.int(num_keys),
		&cKeys[0],
		&cValues[0],
		&cErr,
	)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// LiveFileMetadata is a metadata which is associated with each SST file.
type LiveFileMetadata struct {
	Name        string
	Level       int
	Size        int64
	SmallestKey []byte
	LargestKey  []byte
}

// GetLiveFilesMetaData returns a list of all table files with their
// level, start key and end key.
func (db *DB) GetLiveFilesMetaData() []LiveFileMetadata {
	lf := C.rocksdb_livefiles(db.c)
	defer C.rocksdb_livefiles_destroy(lf)

	count := C.rocksdb_livefiles_count(lf)
	liveFiles := make([]LiveFileMetadata, int(count))
	for i := C.int(0); i < count; i++ {
		var liveFile LiveFileMetadata
		liveFile.Name = C.GoString(C.rocksdb_livefiles_name(lf, i))
		liveFile.Level = int(C.rocksdb_livefiles_level(lf, i))
		liveFile.Size = int64(C.rocksdb_livefiles_size(lf, i))

		var cSize C.size_t
		key := C.rocksdb_livefiles_smallestkey(lf, i, &cSize)
		liveFile.SmallestKey = C.GoBytes(unsafe.Pointer(key), C.int(cSize))

		key = C.rocksdb_livefiles_largestkey(lf, i, &cSize)
		liveFile.LargestKey = C.GoBytes(unsafe.Pointer(key), C.int(cSize))
		liveFiles[int(i)] = liveFile
	}
	return liveFiles
}

// CompactRange runs a manual compaction on the Range of keys given. This is
// not likely to be needed for typical usage.
func (db *DB) CompactRange(r Range) {
	cStart := byteToChar(r.Start)
	cLimit := byteToChar(r.Limit)
	C.rocksdb_compact_range(db.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// CompactRangeCF runs a manual compaction on the Range of keys given on the
// given column family. This is not likely to be needed for typical usage.
func (db *DB) CompactRangeCF(cf *ColumnFamilyHandle, r Range) {
	cStart := byteToChar(r.Start)
	cLimit := byteToChar(r.Limit)
	C.rocksdb_compact_range_cf(db.c, cf.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// Flush triggers a manuel flush for the database.
func (db *DB) Flush(opts *FlushOptions) error {
	var cErr *C.char
	C.rocksdb_flush(db.c, opts.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// DisableFileDeletions disables file deletions and should be used when backup the database.
func (db *DB) DisableFileDeletions() error {
	var cErr *C.char
	C.rocksdb_disable_file_deletions(db.c, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// EnableFileDeletions enables file deletions for the database.
func (db *DB) EnableFileDeletions(force bool) error {
	var cErr *C.char
	C.rocksdb_enable_file_deletions(db.c, boolToChar(force), &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// DeleteFile deletes the file name from the db directory and update the internal state to
// reflect that. Supports deletion of sst and log files only. 'name' must be
// path relative to the db directory. eg. 000001.sst, /archive/000003.log.
func (db *DB) DeleteFile(name string) {
	cName := C.CString(name)
	defer C.free(unsafe.Pointer(cName))
	C.rocksdb_delete_file(db.c, cName)
}

// DeleteFileInRange deletes SST files that contain keys between the Range, [r.Start, r.Limit]
func (db *DB) DeleteFileInRange(r Range) error {
	cStartKey := byteToChar(r.Start)
	cLimitKey := byteToChar(r.Limit)

	var cErr *C.char

	C.rocksdb_delete_file_in_range(
		db.c,
		cStartKey, C.size_t(len(r.Start)),
		cLimitKey, C.size_t(len(r.Limit)),
		&cErr,
	)

	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// DeleteFileInRangeCF deletes SST files that contain keys between the Range, [r.Start, r.Limit], and
// belong to a given column family
func (db *DB) DeleteFileInRangeCF(cf *ColumnFamilyHandle, r Range) error {
	cStartKey := byteToChar(r.Start)
	cLimitKey := byteToChar(r.Limit)

	var cErr *C.char

	C.rocksdb_delete_file_in_range_cf(
		db.c,
		cf.c,
		cStartKey, C.size_t(len(r.Start)),
		cLimitKey, C.size_t(len(r.Limit)),
		&cErr,
	)

	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// IngestExternalFile loads a list of external SST files.
func (db *DB) IngestExternalFile(filePaths []string, opts *IngestExternalFileOptions) error {
	cFilePaths := make([]*C.char, len(filePaths))
	for i, s := range filePaths {
		cFilePaths[i] = C.CString(s)
	}
	defer func() {
		for _, s := range cFilePaths {
			C.free(unsafe.Pointer(s))
		}
	}()

	var cErr *C.char

	C.rocksdb_ingest_external_file(
		db.c,
		&cFilePaths[0],
		C.size_t(len(filePaths)),
		opts.c,
		&cErr,
	)

	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// IngestExternalFileCF loads a list of external SST files for a column family.
func (db *DB) IngestExternalFileCF(handle *ColumnFamilyHandle, filePaths []string, opts *IngestExternalFileOptions) error {
	cFilePaths := make([]*C.char, len(filePaths))
	for i, s := range filePaths {
		cFilePaths[i] = C.CString(s)
	}
	defer func() {
		for _, s := range cFilePaths {
			C.free(unsafe.Pointer(s))
		}
	}()

	var cErr *C.char

	C.rocksdb_ingest_external_file_cf(
		db.c,
		handle.c,
		&cFilePaths[0],
		C.size_t(len(filePaths)),
		opts.c,
		&cErr,
	)

	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// NewCheckpoint creates a new Checkpoint for this db.
func (db *DB) NewCheckpoint() (*Checkpoint, error) {
	var (
		cErr *C.char
	)
	cCheckpoint := C.rocksdb_checkpoint_object_create(
		db.c, &cErr,
	)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return nil, errors.New(C.GoString(cErr))
	}

	return NewNativeCheckpoint(cCheckpoint), nil
}

// Close closes the database.
func (db *DB) Close() {
	C.rocksdb_close(db.c)
}

// DestroyDb removes a database entirely, removing everything from the
// filesystem.
func DestroyDb(name string, opts *Options) error {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	C.rocksdb_destroy_db(opts.c, cName, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}

// RepairDb repairs a database.
func RepairDb(name string, opts *Options) error {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)
	defer C.free(unsafe.Pointer(cName))
	C.rocksdb_repair_db(opts.c, cName, &cErr)
	if cErr != nil {
		defer C.rocksdb_free(unsafe.Pointer(cErr))
		return errors.New(C.GoString(cErr))
	}
	return nil
}
