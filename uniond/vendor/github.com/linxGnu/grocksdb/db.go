package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import (
	"fmt"
	"unsafe"
)

// ErrColumnFamilyMustMatch indicates number of column family names and options must match.
var ErrColumnFamilyMustMatch = fmt.Errorf("must provide the same number of column family names and options")

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
func OpenDb(opts *Options, name string) (db *DB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	_db := C.rocksdb_open(opts.c, cName, &cErr)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
	}

	C.free(unsafe.Pointer(cName))
	return
}

// OpenDbWithTTL opens a database with TTL support with the specified options.
func OpenDbWithTTL(opts *Options, name string, ttl int) (db *DB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	_db := C.rocksdb_open_with_ttl(opts.c, cName, C.int(ttl), &cErr)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
	}

	C.free(unsafe.Pointer(cName))
	return
}

// OpenDbForReadOnly opens a database with the specified options for readonly usage.
func OpenDbForReadOnly(opts *Options, name string, errorIfWalFileExists bool) (db *DB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	_db := C.rocksdb_open_for_read_only(opts.c, cName, boolToChar(errorIfWalFileExists), &cErr)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
	}

	C.free(unsafe.Pointer(cName))
	return
}

// OpenDbAsSecondary creates a secondary instance that
// can dynamically tail the MANIFEST of a primary that must have already been
// created. User can call TryCatchUpWithPrimary to make the secondary
// instance catch up with primary (WAL tailing is NOT supported now) whenever
// the user feels necessary. Column families created by the primary after the
// secondary instance starts are currently ignored by the secondary instance.
// Column families opened by secondary and dropped by the primary will be
// dropped by secondary as well. However the user of the secondary instance
// can still access the data of such dropped column family as long as they
// do not destroy the corresponding column family handle.
// WAL tailing is not supported at present, but will arrive soon.
func OpenDbAsSecondary(opts *Options, name, secondaryPath string) (db *DB, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
		cPath = C.CString(secondaryPath)
	)

	_db := C.rocksdb_open_as_secondary(opts.c, cName, cPath, &cErr)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
	}

	C.free(unsafe.Pointer(cName))
	C.free(unsafe.Pointer(cPath))
	return
}

// OpenDbColumnFamilies opens a database with the specified column families.
func OpenDbColumnFamilies(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
) (db *DB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	_db := C.rocksdb_open_column_families(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// OpenDbColumnFamiliesWithTTL opens a database with the specified column families along with their ttls.
//
// BEHAVIOUR:
// TTL is accepted in seconds
// (int32_t)Timestamp(creation) is suffixed to values in Put internally
// Expired TTL values deleted in compaction only:(Timestamp+ttl<time_now)
// Get/Iterator may return expired entries(compaction not run on them yet)
// Different TTL may be used during different Opens
// Example:
// Open1 at t=0 with ttl=4 and insert k1,k2, close at t=2
// Open2 at t=3 with ttl=5. Now k1,k2 should be deleted at t>=5
// read_only=true opens in the usual read-only mode. Compactions will not be
// triggered(neither manual nor automatic), so no expired entries removed
//
// CONSTRAINTS:
// Not specifying/passing or non-positive TTL behaves like TTL = infinity
func OpenDbColumnFamiliesWithTTL(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
	ttls []C.int,
) (db *DB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	_db := C.rocksdb_open_column_families_with_ttl(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&ttls[0],
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// OpenDbForReadOnlyColumnFamilies opens a database with the specified column
// families in read only mode.
func OpenDbForReadOnlyColumnFamilies(
	opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
	errorIfWalFileExists bool,
) (db *DB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	_db := C.rocksdb_open_for_read_only_column_families(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		boolToChar(errorIfWalFileExists),
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// OpenDbAsSecondaryColumnFamilies opens database as secondary instance with column families.
// You can open a subset of column families in secondary mode.
// The `opts` specify the database specific options.
// The `name` argument specifies the name of the primary db that you have used
// to open the primary instance.
// The `secondaryPath` argument points to a directory where the secondary
// instance stores its info log.
// The `column_families` arguments specifieds a list of column families to open.
// If any of the column families does not exist, the function returns non-OK
// status.
func OpenDbAsSecondaryColumnFamilies(
	opts *Options,
	name string,
	secondaryPath string,
	cfNames []string,
	cfOpts []*Options,
) (db *DB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cPath := C.CString(secondaryPath)

	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	var cErr *C.char
	_db := C.rocksdb_open_as_secondary_column_families(
		opts.c,
		cName,
		cPath,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cPath))
	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// OpenDbAndTrimHistory opens DB and trim data newer than specified timestamp.
// The trim_ts specified the user-defined timestamp trim bound.
// This API should only be used at timestamp enabled column families recovery.
// If some input column families do not support timestamp, nothing will
// be happened to them. The data with timestamp > trim_ts
// will be removed after this API returns successfully.
func OpenDbAndTrimHistory(opts *Options,
	name string,
	cfNames []string,
	cfOpts []*Options,
	trimTimestamp []byte,
) (db *DB, cfHandles []*ColumnFamilyHandle, err error) {
	numColumnFamilies := len(cfNames)
	if numColumnFamilies != len(cfOpts) {
		err = ErrColumnFamilyMustMatch
		return
	}

	cName := C.CString(name)
	cNames := make([]*C.char, numColumnFamilies)
	for i, s := range cfNames {
		cNames[i] = C.CString(s)
	}

	cOpts := make([]*C.rocksdb_options_t, numColumnFamilies)
	for i, o := range cfOpts {
		cOpts[i] = o.c
	}

	cHandles := make([]*C.rocksdb_column_family_handle_t, numColumnFamilies)

	cTs := refGoBytes(trimTimestamp)

	var cErr *C.char
	_db := C.rocksdb_open_and_trim_history(
		opts.c,
		cName,
		C.int(numColumnFamilies),
		&cNames[0],
		&cOpts[0],
		&cHandles[0],
		cTs,
		C.size_t(len(trimTimestamp)),
		&cErr,
	)
	if err = fromCError(cErr); err == nil {
		db = &DB{
			name: name,
			c:    _db,
			opts: opts,
		}
		cfHandles = make([]*ColumnFamilyHandle, numColumnFamilies)
		for i, c := range cHandles {
			cfHandles[i] = newNativeColumnFamilyHandle(c)
		}
	}

	C.free(unsafe.Pointer(cName))
	for _, s := range cNames {
		C.free(unsafe.Pointer(s))
	}
	return
}

// ListColumnFamilies lists the names of the column families in the DB.
func ListColumnFamilies(opts *Options, name string) (names []string, err error) {
	var (
		cErr  *C.char
		cLen  C.size_t
		cName = C.CString(name)
	)

	cNames := C.rocksdb_list_column_families(opts.c, cName, &cLen, &cErr)
	if err = fromCError(cErr); err == nil {
		namesLen := int(cLen)

		names = make([]string, namesLen)
		// The maximum capacity of the following two slices is limited to (2^29)-1 to remain compatible
		// with 32-bit platforms. The size of a `*C.char` (a pointer) is 4 Byte on a 32-bit system
		// and (2^29)*4 == math.MaxInt32 + 1. -- See issue golang/go#13656
		cNamesArr := (*[(1 << 29) - 1]*C.char)(unsafe.Pointer(cNames))[:namesLen:namesLen]
		for i, n := range cNamesArr {
			names[i] = C.GoString(n)
		}
	}

	C.rocksdb_list_column_families_destroy(cNames, cLen)
	C.free(unsafe.Pointer(cName))
	return
}

// Name returns the name of the database.
func (db *DB) Name() string {
	return db.name
}

// KeyMayExists the value is only allocated (using malloc) and returned if it is found and
// value_found isn't NULL. In that case the user is responsible for freeing it.
func (db *DB) KeyMayExists(opts *ReadOptions, key []byte, timestamp string) (slice *Slice) {
	t := []byte(timestamp)

	var (
		cValue     *C.char
		cValLen    C.size_t
		cKey               = refGoBytes(key)
		cFound     C.uchar = 0
		cTimestamp         = refGoBytes(t)
	)

	C.rocksdb_key_may_exist(db.c, opts.c,
		cKey, C.size_t(len(key)),
		&cValue, &cValLen,
		cTimestamp, C.size_t(len(t)),
		&cFound)

	if charToBool(cFound) {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// KeyMayExistsCF the value is only allocated (using malloc) and returned if it is found and
// value_found isn't NULL. In that case the user is responsible for freeing it.
func (db *DB) KeyMayExistsCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte, timestamp string) (slice *Slice) {
	t := []byte(timestamp)

	var (
		cValue     *C.char
		cValLen    C.size_t
		cKey               = refGoBytes(key)
		cFound     C.uchar = 0
		cTimestamp         = refGoBytes(t)
	)

	C.rocksdb_key_may_exist_cf(db.c, opts.c,
		cf.c,
		cKey, C.size_t(len(key)),
		&cValue, &cValLen,
		cTimestamp, C.size_t(len(t)),
		&cFound)

	if charToBool(cFound) {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// Get returns the data associated with the key from the database.
func (db *DB) Get(opts *ReadOptions, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetWithTS returns the data and timestamp associated with the key from the database.
func (db *DB) GetWithTS(opts *ReadOptions, key []byte) (value, timestamp *Slice, err error) {
	var (
		cErr    *C.char
		cTs     *C.char
		cValLen C.size_t
		cTsLen  C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get_with_ts(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cTs, &cTsLen, &cErr)
	if err = fromCError(cErr); err == nil {
		value = NewSlice(cValue, cValLen)
		timestamp = NewSlice(cTs, cTsLen)
	}

	return
}

// GetBytes is like Get but returns a copy of the data.
func (db *DB) GetBytes(opts *ReadOptions, key []byte) (data []byte, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		if cValue == nil {
			return nil, nil
		}

		data = C.GoBytes(unsafe.Pointer(cValue), C.int(cValLen))
		C.rocksdb_free(unsafe.Pointer(cValue))
	}

	return
}

// GetBytesWithTS is like Get but returns a copy of the data and timestamp.
func (db *DB) GetBytesWithTS(opts *ReadOptions, key []byte) (data, timestamp []byte, err error) {
	var (
		cErr    *C.char
		cTs     *C.char
		cValLen C.size_t
		cTsLen  C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get_with_ts(db.c, opts.c, cKey, C.size_t(len(key)), &cValLen, &cTs, &cTsLen, &cErr)
	if err = fromCError(cErr); err == nil {
		if cValue == nil {
			return nil, nil, nil
		}

		data = C.GoBytes(unsafe.Pointer(cValue), C.int(cValLen))
		timestamp = C.GoBytes(unsafe.Pointer(cTs), C.int(cTsLen))
		C.rocksdb_free(unsafe.Pointer(cValue))
		C.rocksdb_free(unsafe.Pointer(cTs))
	}

	return
}

// GetCF returns the data associated with the key from the database and column family.
func (db *DB) GetCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (slice *Slice, err error) {
	var (
		cErr    *C.char
		cValLen C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cValue, cValLen)
	}

	return
}

// GetCFWithTS returns the data and timestamp associated with the key from the database and column family.
func (db *DB) GetCFWithTS(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (value, timestamp *Slice, err error) {
	var (
		cErr    *C.char
		cTs     *C.char
		cValLen C.size_t
		cTsLen  C.size_t
		cKey    = refGoBytes(key)
	)

	cValue := C.rocksdb_get_cf_with_ts(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cValLen, &cTs, &cTsLen, &cErr)
	if err = fromCError(cErr); err == nil {
		value = NewSlice(cValue, cValLen)
		timestamp = NewSlice(cTs, cTsLen)
	}

	return
}

// GetPinned returns the data associated with the key from the database.
func (db *DB) GetPinned(opts *ReadOptions, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_get_pinned(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// GetPinnedCF returns the data associated with the key from the database, specific column family.
func (db *DB) GetPinnedCF(opts *ReadOptions, cf *ColumnFamilyHandle, key []byte) (handle *PinnableSliceHandle, err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	cHandle := C.rocksdb_get_pinned_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativePinnableSliceHandle(cHandle)
	}

	return
}

// MultiGet returns the data associated with the passed keys from the database
func (db *DB) MultiGet(opts *ReadOptions, keys ...[]byte) (Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

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
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	cKeys.Destroy()
	return slices, nil
}

// MultiGetWithTS returns the data and timestamps associated with the passed keys from the database
func (db *DB) MultiGetWithTS(opts *ReadOptions, keys ...[]byte) (Slices, Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))

	timestamps := make(charsSlice, len(keys))
	timestampSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_multi_get_with_ts(
		db.c,
		opts.c,
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		timestamps.c(),
		timestampSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	values := make(Slices, len(keys))
	for i, val := range vals {
		values[i] = NewSlice(val, valSizes[i])
	}

	times := make(Slices, len(keys))
	for i, ts := range timestamps {
		times[i] = NewSlice(ts, timestampSizes[i])
	}

	cKeys.Destroy()
	return values, times, nil
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
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

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
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	slices := make(Slices, len(keys))
	for i, val := range vals {
		slices[i] = NewSlice(val, valSizes[i])
	}

	cKeys.Destroy()
	return slices, nil
}

// MultiGetCFWithTS returns the data and timestamp associated with the passed keys from the column family
func (db *DB) MultiGetCFWithTS(opts *ReadOptions, cf *ColumnFamilyHandle, keys ...[]byte) (Slices, Slices, error) {
	cfs := make(ColumnFamilyHandles, len(keys))
	for i := 0; i < len(keys); i++ {
		cfs[i] = cf
	}
	return db.MultiGetMultiCFWithTS(opts, cfs, keys)
}

// MultiGetMultiCFWithTS returns the data and timestamp associated with the passed keys and
// column families.
func (db *DB) MultiGetMultiCFWithTS(opts *ReadOptions, cfs ColumnFamilyHandles, keys [][]byte) (Slices, Slices, error) {
	// will destroy `cKeys` before return
	cKeys, cKeySizes := byteSlicesToCSlices(keys)

	vals := make(charsSlice, len(keys))
	valSizes := make(sizeTSlice, len(keys))
	timestamps := make(charsSlice, len(keys))
	timestampSizes := make(sizeTSlice, len(keys))
	rocksErrs := make(charsSlice, len(keys))

	C.rocksdb_multi_get_cf_with_ts(
		db.c,
		opts.c,
		cfs.toCSlice().c(),
		C.size_t(len(keys)),
		cKeys.c(),
		cKeySizes.c(),
		vals.c(),
		valSizes.c(),
		timestamps.c(),
		timestampSizes.c(),
		rocksErrs.c(),
	)

	var errs []error

	for i, rocksErr := range rocksErrs {
		if err := fromCError(rocksErr); err != nil {
			errs = append(errs, fmt.Errorf("getting %q failed: %v", string(keys[i]), err.Error()))
		}
	}

	if len(errs) > 0 {
		cKeys.Destroy()
		return nil, nil, fmt.Errorf("failed to get %d keys, first error: %v", len(errs), errs[0])
	}

	values := make(Slices, len(keys))
	for i, val := range vals {
		values[i] = NewSlice(val, valSizes[i])
	}

	times := make(Slices, len(keys))
	for i, ts := range timestamps {
		times[i] = NewSlice(ts, timestampSizes[i])
	}

	cKeys.Destroy()
	return values, times, nil
}

// Put writes data associated with a key to the database.
func (db *DB) Put(opts *WriteOptions, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_put(db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// PutWithTS writes data associated with a key and timestamp to the database.
func (db *DB) PutWithTS(opts *WriteOptions, key, ts, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
		cTs    = refGoBytes(ts)
	)

	C.rocksdb_put_with_ts(db.c, opts.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// PutCF writes data associated with a key to the database and column family.
func (db *DB) PutCF(opts *WriteOptions, cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_put_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// PutCFWithTS writes data associated with a key and timestamp to the database and column family.
func (db *DB) PutCFWithTS(opts *WriteOptions, cf *ColumnFamilyHandle, key, ts, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
		cTs    = refGoBytes(ts)
	)

	C.rocksdb_put_cf_with_ts(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// Delete removes the data associated with the key from the database.
func (db *DB) Delete(opts *WriteOptions, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_delete(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteCF removes the data associated with the key from the database and column family.
func (db *DB) DeleteCF(opts *WriteOptions, cf *ColumnFamilyHandle, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_delete_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteWithTS removes the data associated with the key and timestamp from the database.
func (db *DB) DeleteWithTS(opts *WriteOptions, key, ts []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
		cTs  = refGoBytes(ts)
	)

	C.rocksdb_delete_with_ts(db.c, opts.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteCFWithTS removes the data associated with the key and timestamp from the database and column family.
func (db *DB) DeleteCFWithTS(opts *WriteOptions, cf *ColumnFamilyHandle, key, ts []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
		cTs  = refGoBytes(ts)
	)

	C.rocksdb_delete_cf_with_ts(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), &cErr)
	err = fromCError(cErr)

	return
}

// SingleDeleteWithTS removes the data associated with the key and timestamp from the database.
func (db *DB) SingleDeleteWithTS(opts *WriteOptions, key, ts []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
		cTs  = refGoBytes(ts)
	)

	C.rocksdb_singledelete_with_ts(db.c, opts.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), &cErr)
	err = fromCError(cErr)

	return
}

// SingleDeleteCFWithTS removes the data associated with the key and timestamp from the database and column family.
func (db *DB) SingleDeleteCFWithTS(opts *WriteOptions, cf *ColumnFamilyHandle, key, ts []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
		cTs  = refGoBytes(ts)
	)

	C.rocksdb_singledelete_cf_with_ts(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cTs, C.size_t(len(ts)), &cErr)
	err = fromCError(cErr)

	return
}

// DeleteRangeCF deletes keys that are between [startKey, endKey)
func (db *DB) DeleteRangeCF(opts *WriteOptions, cf *ColumnFamilyHandle, startKey, endKey []byte) (err error) {
	var (
		cErr      *C.char
		cStartKey = refGoBytes(startKey)
		cEndKey   = refGoBytes(endKey)
	)

	C.rocksdb_delete_range_cf(db.c, opts.c, cf.c, cStartKey, C.size_t(len(startKey)), cEndKey, C.size_t(len(endKey)), &cErr)
	err = fromCError(cErr)

	return
}

// SingleDelete removes the database entry for "key". Requires that the key exists
// and was not overwritten. Returns OK on success, and a non-OK status
// on error.  It is not an error if "key" did not exist in the database.
//
// If a key is overwritten (by calling Put() multiple times), then the result
// of calling SingleDelete() on this key is undefined.  SingleDelete() only
// behaves correctly if there has been only one Put() for this key since the
// previous call to SingleDelete() for this key.
//
// This feature is currently an experimental performance optimization
// for a very specific workload.  It is up to the caller to ensure that
// SingleDelete is only used for a key that is not deleted using Delete() or
// written using Merge().  Mixing SingleDelete operations with Deletes and
// Merges can result in undefined behavior.
//
// Note: consider setting options.sync = true.
func (db *DB) SingleDelete(opts *WriteOptions, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_singledelete(db.c, opts.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// SingleDeleteCF removes the database entry for "key". Requires that the key exists
// and was not overwritten. Returns OK on success, and a non-OK status
// on error.  It is not an error if "key" did not exist in the database.
//
// If a key is overwritten (by calling Put() multiple times), then the result
// of calling SingleDelete() on this key is undefined.  SingleDelete() only
// behaves correctly if there has been only one Put() for this key since the
// previous call to SingleDelete() for this key.
//
// This feature is currently an experimental performance optimization
// for a very specific workload.  It is up to the caller to ensure that
// SingleDelete is only used for a key that is not deleted using Delete() or
// written using Merge().  Mixing SingleDelete operations with Deletes and
// Merges can result in undefined behavior.
//
// Note: consider setting options.sync = true.
func (db *DB) SingleDeleteCF(opts *WriteOptions, cf *ColumnFamilyHandle, key []byte) (err error) {
	var (
		cErr *C.char
		cKey = refGoBytes(key)
	)

	C.rocksdb_singledelete_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), &cErr)
	err = fromCError(cErr)

	return
}

// Merge merges the data associated with the key with the actual data in the database.
func (db *DB) Merge(opts *WriteOptions, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_merge(db.c, opts.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// MergeCF merges the data associated with the key with the actual data in the
// database and column family.
func (db *DB) MergeCF(opts *WriteOptions, cf *ColumnFamilyHandle, key, value []byte) (err error) {
	var (
		cErr   *C.char
		cKey   = refGoBytes(key)
		cValue = refGoBytes(value)
	)

	C.rocksdb_merge_cf(db.c, opts.c, cf.c, cKey, C.size_t(len(key)), cValue, C.size_t(len(value)), &cErr)
	err = fromCError(cErr)

	return
}

// Write a batch to the database.
func (db *DB) Write(opts *WriteOptions, batch *WriteBatch) (err error) {
	var cErr *C.char

	C.rocksdb_write(db.c, opts.c, batch.c, &cErr)
	err = fromCError(cErr)

	return
}

// WriteWI writes a batch wi to the database.
func (db *DB) WriteWI(opts *WriteOptions, batch *WriteBatchWI) (err error) {
	var cErr *C.char

	C.rocksdb_write_writebatch_wi(db.c, opts.c, batch.c, &cErr)
	err = fromCError(cErr)

	return
}

// NewIterator returns an Iterator over the the database that uses the
// ReadOptions given.
func (db *DB) NewIterator(opts *ReadOptions) *Iterator {
	cIter := C.rocksdb_create_iterator(db.c, opts.c)
	return newNativeIterator(cIter)
}

// NewIteratorCF returns an Iterator over the the database and column family
// that uses the ReadOptions given.
func (db *DB) NewIteratorCF(opts *ReadOptions, cf *ColumnFamilyHandle) *Iterator {
	cIter := C.rocksdb_create_iterator_cf(db.c, opts.c, cf.c)
	return newNativeIterator(cIter)
}

// NewIterators returns iterators from a consistent database state across multiple
// column families. Iterators are heap allocated and need to be deleted
// before the db is deleted
func (db *DB) NewIterators(opts *ReadOptions, cfs []*ColumnFamilyHandle) (iters []*Iterator, err error) {
	if n := len(cfs); n > 0 {
		_cfs := make([]*C.rocksdb_column_family_handle_t, n)
		for i := range _cfs {
			_cfs[i] = cfs[i].c
		}
		_iters := make([]*C.rocksdb_iterator_t, n)

		var cErr *C.char
		C.rocksdb_create_iterators(db.c, opts.c, &_cfs[0], &_iters[0], C.size_t(n), &cErr)
		if err = fromCError(cErr); err == nil {
			iters = make([]*Iterator, n)
			for i := range iters {
				iters[i] = newNativeIterator(_iters[i])
			}
		}
	}
	return
}

// GetUpdatesSince if the sequence number is non existent, it returns an iterator
// at the first available seq_no after the requested seq_no.
//
// Must set WAL_ttl_seconds or WAL_size_limit_MB to large values to
// use this api, else the WAL files will get
// cleared aggressively and the iterator might keep getting invalid before
// an update is read.
//
// Note: this API is not yet consistent with WritePrepared transactions.
// Sets iter to an iterator that is positioned at a write-batch containing
// seq_number.
func (db *DB) GetUpdatesSince(seqNumber uint64) (iter *WalIterator, err error) {
	var cErr *C.char

	cIter := C.rocksdb_get_updates_since(db.c, C.uint64_t(seqNumber), nil, &cErr)
	if err = fromCError(cErr); err == nil {
		iter = newNativeWalIterator(unsafe.Pointer(cIter))
	}

	return
}

// GetLatestSequenceNumber returns sequence number of the most recent transaction.
func (db *DB) GetLatestSequenceNumber() uint64 {
	return uint64(C.rocksdb_get_latest_sequence_number(db.c))
}

// NewSnapshot creates a new snapshot of the database.
func (db *DB) NewSnapshot() *Snapshot {
	cSnap := C.rocksdb_create_snapshot(db.c)
	return newNativeSnapshot(cSnap)
}

// ReleaseSnapshot releases the snapshot and its resources.
func (db *DB) ReleaseSnapshot(snapshot *Snapshot) {
	C.rocksdb_release_snapshot(db.c, snapshot.c)
	snapshot.c = nil
}

// GetProperty returns the value of a database property.
func (db *DB) GetProperty(propName string) (value string) {
	cprop := C.CString(propName)
	cValue := C.rocksdb_property_value(db.c, cprop)

	value = C.GoString(cValue)

	C.rocksdb_free(unsafe.Pointer(cValue))
	C.free(unsafe.Pointer(cprop))
	return
}

// GetIntProperty similar to `GetProperty`, but only works for a subset of properties whose
// return value is an integer. Return the value by integer.
func (db *DB) GetIntProperty(propName string) (value uint64, success bool) {
	cProp := C.CString(propName)
	success = C.rocksdb_property_int(db.c, cProp, (*C.uint64_t)(&value)) == 0
	C.free(unsafe.Pointer(cProp))
	return
}

// GetIntPropertyCF similar to `GetProperty`, but only works for a subset of properties whose
// return value is an integer. Return the value by integer.
func (db *DB) GetIntPropertyCF(propName string, cf *ColumnFamilyHandle) (value uint64, success bool) {
	cProp := C.CString(propName)
	success = C.rocksdb_property_int_cf(db.c, cf.c, cProp, (*C.uint64_t)(&value)) == 0
	C.free(unsafe.Pointer(cProp))
	return
}

// GetPropertyCF returns the value of a database property.
func (db *DB) GetPropertyCF(propName string, cf *ColumnFamilyHandle) (value string) {
	cProp := C.CString(propName)
	cValue := C.rocksdb_property_value_cf(db.c, cf.c, cProp)

	value = C.GoString(cValue)

	C.rocksdb_free(unsafe.Pointer(cValue))
	C.free(unsafe.Pointer(cProp))
	return
}

// CreateColumnFamily create a new column family.
func (db *DB) CreateColumnFamily(opts *Options, name string) (handle *ColumnFamilyHandle, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	cHandle := C.rocksdb_create_column_family(db.c, opts.c, cName, &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativeColumnFamilyHandle(cHandle)
	}

	C.free(unsafe.Pointer(cName))
	return
}

// CreateColumnFamilies creates new column families.
func (db *DB) CreateColumnFamilies(opts *Options, names []string) (handles []*ColumnFamilyHandle, err error) {
	if len(names) == 0 {
		return nil, nil
	}

	var cErr *C.char

	n := len(names)
	cNames := make([]*C.char, 0, n)
	cSizes := make([]C.size_t, 0, n)
	for i := range names {
		cNames = append(cNames, C.CString(names[i]))
		cSizes = append(cSizes, C.size_t(len(names[i])))
	}

	cHandles := C.rocksdb_create_column_families(db.c, opts.c, C.int(n), &cNames[0], &cSizes[0], &cErr)
	if err = fromCError(cErr); err == nil {
		tmp := unsafe.Slice(cHandles, n)

		handles = make([]*ColumnFamilyHandle, 0, n)
		for i := range tmp {
			handles = append(handles, newNativeColumnFamilyHandle(tmp[i]))
		}
	}

	for i := range cNames {
		C.free(unsafe.Pointer(cNames[i]))
	}

	return
}

// CreateColumnFamilyWithTTL create a new column family along with its ttl.
//
// BEHAVIOUR:
// TTL is accepted in seconds
// (int32_t)Timestamp(creation) is suffixed to values in Put internally
// Expired TTL values deleted in compaction only:(Timestamp+ttl<time_now)
// Get/Iterator may return expired entries(compaction not run on them yet)
// Different TTL may be used during different Opens
// Example:
// Open1 at t=0 with ttl=4 and insert k1,k2, close at t=2
// Open2 at t=3 with ttl=5. Now k1,k2 should be deleted at t>=5
// read_only=true opens in the usual read-only mode. Compactions will not be
// triggered(neither manual nor automatic), so no expired entries removed
//
// CONSTRAINTS:
// Not specifying/passing or non-positive TTL behaves like TTL = infinity
func (db *DB) CreateColumnFamilyWithTTL(opts *Options, name string, ttl int) (handle *ColumnFamilyHandle, err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	cHandle := C.rocksdb_create_column_family_with_ttl(db.c, opts.c, cName, C.int(ttl), &cErr)
	if err = fromCError(cErr); err == nil {
		handle = newNativeColumnFamilyHandle(cHandle)
	}

	C.free(unsafe.Pointer(cName))
	return
}

// DropColumnFamily drops a column family.
func (db *DB) DropColumnFamily(c *ColumnFamilyHandle) (err error) {
	var cErr *C.char

	C.rocksdb_drop_column_family(db.c, c.c, &cErr)
	err = fromCError(cErr)

	return
}

// GetApproximateSizes returns the approximate number of bytes of file system
// space used by one or more key ranges.
//
// The keys counted will begin at Range.Start and end on the key before
// Range.Limit.
func (db *DB) GetApproximateSizes(ranges []Range) ([]uint64, error) {
	sizes := make([]uint64, len(ranges))
	if len(ranges) == 0 {
		return sizes, nil
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

	var cErr *C.char
	C.rocksdb_approximate_sizes(
		db.c,
		C.int(len(ranges)),
		&cStarts[0],
		&cStartLens[0],
		&cLimits[0],
		&cLimitLens[0],
		(*C.uint64_t)(&sizes[0]),
		&cErr,
	)

	// parse error
	err := fromCError(cErr)

	// free before return
	for i := range ranges {
		C.free(unsafe.Pointer(cStarts[i]))
		C.free(unsafe.Pointer(cLimits[i]))
	}

	return sizes, err
}

// GetApproximateSizesCF returns the approximate number of bytes of file system
// space used by one or more key ranges in the column family.
//
// The keys counted will begin at Range.Start and end on the key before
// Range.Limit.
func (db *DB) GetApproximateSizesCF(cf *ColumnFamilyHandle, ranges []Range) ([]uint64, error) {
	sizes := make([]uint64, len(ranges))
	if len(ranges) == 0 {
		return sizes, nil
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

	var cErr *C.char
	C.rocksdb_approximate_sizes_cf(
		db.c,
		cf.c,
		C.int(len(ranges)),
		&cStarts[0],
		&cStartLens[0],
		&cLimits[0],
		&cLimitLens[0],
		(*C.uint64_t)(&sizes[0]),
		&cErr,
	)

	// parse error
	err := fromCError(cErr)

	// free before return
	for i := range ranges {
		C.free(unsafe.Pointer(cStarts[i]))
		C.free(unsafe.Pointer(cLimits[i]))
	}

	return sizes, err
}

// SetOptions dynamically changes options through the SetOptions API.
func (db *DB) SetOptions(keys, values []string) (err error) {
	numKeys := len(keys)
	if numKeys == 0 {
		return nil
	}

	cKeys := make([]*C.char, numKeys)
	cValues := make([]*C.char, numKeys)
	for i := range keys {
		cKeys[i] = C.CString(keys[i])
		cValues[i] = C.CString(values[i])
	}

	var cErr *C.char

	C.rocksdb_set_options(
		db.c,
		C.int(numKeys),
		&cKeys[0],
		&cValues[0],
		&cErr,
	)
	err = fromCError(cErr)

	// free before return
	for i := range cKeys {
		C.free(unsafe.Pointer(cKeys[i]))
		C.free(unsafe.Pointer(cValues[i]))
	}

	return
}

// SetOptionsCF dynamically changes options through the SetOptions API for specific Column Family.
func (db *DB) SetOptionsCF(cf *ColumnFamilyHandle, keys, values []string) (err error) {
	numKeys := len(keys)
	if numKeys == 0 {
		return nil
	}

	cKeys := make([]*C.char, numKeys)
	cValues := make([]*C.char, numKeys)
	for i := range keys {
		cKeys[i] = C.CString(keys[i])
		cValues[i] = C.CString(values[i])
	}

	var cErr *C.char

	C.rocksdb_set_options_cf(
		db.c,
		cf.c,
		C.int(numKeys),
		&cKeys[0],
		&cValues[0],
		&cErr,
	)
	err = fromCError(cErr)

	// free before return
	for i := range cKeys {
		C.free(unsafe.Pointer(cKeys[i]))
		C.free(unsafe.Pointer(cValues[i]))
	}

	return
}

// LiveFileMetadata is a metadata which is associated with each SST file.
type LiveFileMetadata struct {
	Name             string
	ColumnFamilyName string
	Level            int
	Size             int64
	SmallestKey      []byte
	LargestKey       []byte
	Entries          uint64 // number of entries
	Deletions        uint64 // number of deletions
}

// GetLiveFilesMetaData returns a list of all table files with their
// level, start key and end key.
func (db *DB) GetLiveFilesMetaData() []LiveFileMetadata {
	lf := C.rocksdb_livefiles(db.c)

	count := C.rocksdb_livefiles_count(lf)
	liveFiles := make([]LiveFileMetadata, int(count))
	for i := C.int(0); i < count; i++ {
		var liveFile LiveFileMetadata
		liveFile.Name = C.GoString(C.rocksdb_livefiles_name(lf, i))
		liveFile.ColumnFamilyName = C.GoString(C.rocksdb_livefiles_column_family_name(lf, i))
		liveFile.Level = int(C.rocksdb_livefiles_level(lf, i))
		liveFile.Size = int64(C.rocksdb_livefiles_size(lf, i))

		var cSize C.size_t
		key := C.rocksdb_livefiles_smallestkey(lf, i, &cSize)
		liveFile.SmallestKey = C.GoBytes(unsafe.Pointer(key), C.int(cSize))

		key = C.rocksdb_livefiles_largestkey(lf, i, &cSize)
		liveFile.LargestKey = C.GoBytes(unsafe.Pointer(key), C.int(cSize))

		liveFile.Entries = uint64(C.rocksdb_livefiles_entries(lf, i))

		liveFile.Deletions = uint64(C.rocksdb_livefiles_deletions(lf, i))

		liveFiles[int(i)] = liveFile
	}

	C.rocksdb_livefiles_destroy(lf)
	return liveFiles
}

// CompactRange runs a manual compaction on the Range of keys given. This is
// not likely to be needed for typical usage.
func (db *DB) CompactRange(r Range) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)
	C.rocksdb_compact_range(db.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// CompactRangeCF runs a manual compaction on the Range of keys given on the
// given column family. This is not likely to be needed for typical usage.
func (db *DB) CompactRangeCF(cf *ColumnFamilyHandle, r Range) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)
	C.rocksdb_compact_range_cf(db.c, cf.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// CompactRangeOpt runs a manual compaction on the Range of keys given with provided options. This is
// not likely to be needed for typical usage.
func (db *DB) CompactRangeOpt(r Range, opt *CompactRangeOptions) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)
	C.rocksdb_compact_range_opt(db.c, opt.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// CompactRangeCFOpt runs a manual compaction on the Range of keys given on the
// given column family with provided options. This is not likely to be needed for typical usage.
func (db *DB) CompactRangeCFOpt(cf *ColumnFamilyHandle, r Range, opt *CompactRangeOptions) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)
	C.rocksdb_compact_range_cf_opt(db.c, cf.c, opt.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)))
}

// SuggestCompactRange only for leveled compaction.
func (db *DB) SuggestCompactRange(r Range) (err error) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)

	var cErr *C.char
	C.rocksdb_suggest_compact_range(db.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)), &cErr)
	err = fromCError(cErr)

	return
}

// SuggestCompactRangeCF only for leveled compaction.
func (db *DB) SuggestCompactRangeCF(cf *ColumnFamilyHandle, r Range) (err error) {
	cStart := refGoBytes(r.Start)
	cLimit := refGoBytes(r.Limit)

	var cErr *C.char
	C.rocksdb_suggest_compact_range_cf(db.c, cf.c, cStart, C.size_t(len(r.Start)), cLimit, C.size_t(len(r.Limit)), &cErr)
	err = fromCError(cErr)

	return
}

// Flush triggers a manual flush for the database.
func (db *DB) Flush(opts *FlushOptions) (err error) {
	var cErr *C.char

	C.rocksdb_flush(db.c, opts.c, &cErr)
	err = fromCError(cErr)

	return
}

// FlushCF triggers a manual flush for the database on specific column family.
func (db *DB) FlushCF(cf *ColumnFamilyHandle, opts *FlushOptions) (err error) {
	var cErr *C.char

	C.rocksdb_flush_cf(db.c, opts.c, cf.c, &cErr)
	err = fromCError(cErr)

	return
}

// FlushCFs triggers a manual flush for the database on specific column families.
func (db *DB) FlushCFs(cfs []*ColumnFamilyHandle, opts *FlushOptions) (err error) {
	if n := len(cfs); n > 0 {
		_cfs := make([]*C.rocksdb_column_family_handle_t, n)
		for i := range _cfs {
			_cfs[i] = cfs[i].c
		}

		var cErr *C.char
		C.rocksdb_flush_cfs(db.c, opts.c, &_cfs[0], C.int(n), &cErr)
		err = fromCError(cErr)
	}
	return
}

// FlushWAL flushes the WAL memory buffer to the file. If sync is true, it calls SyncWAL
// afterwards.
func (db *DB) FlushWAL(sync bool) (err error) {
	var cErr *C.char

	C.rocksdb_flush_wal(db.c, boolToChar(sync), &cErr)
	err = fromCError(cErr)

	return
}

// DisableFileDeletions disables file deletions and should be used when backup the database.
func (db *DB) DisableFileDeletions() (err error) {
	var cErr *C.char

	C.rocksdb_disable_file_deletions(db.c, &cErr)
	err = fromCError(cErr)

	return
}

// EnableFileDeletions enables file deletions for the database.
func (db *DB) EnableFileDeletions() (err error) {
	var cErr *C.char

	C.rocksdb_enable_file_deletions(db.c, &cErr)
	err = fromCError(cErr)

	return
}

// DeleteFile deletes the file name from the db directory and update the internal state to
// reflect that. Supports deletion of sst and log files only. 'name' must be
// path relative to the db directory. eg. 000001.sst, /archive/000003.log.
func (db *DB) DeleteFile(name string) {
	cName := C.CString(name)

	C.rocksdb_delete_file(db.c, cName)

	C.free(unsafe.Pointer(cName))
}

// DeleteFileInRange deletes SST files that contain keys between the Range, [r.Start, r.Limit]
func (db *DB) DeleteFileInRange(r Range) (err error) {
	cStartKey := refGoBytes(r.Start)
	cLimitKey := refGoBytes(r.Limit)

	var cErr *C.char

	C.rocksdb_delete_file_in_range(
		db.c,
		cStartKey, C.size_t(len(r.Start)),
		cLimitKey, C.size_t(len(r.Limit)),
		&cErr,
	)
	err = fromCError(cErr)

	return
}

// DeleteFileInRangeCF deletes SST files that contain keys between the Range, [r.Start, r.Limit], and
// belong to a given column family
func (db *DB) DeleteFileInRangeCF(cf *ColumnFamilyHandle, r Range) (err error) {
	cStartKey := refGoBytes(r.Start)
	cLimitKey := refGoBytes(r.Limit)

	var cErr *C.char

	C.rocksdb_delete_file_in_range_cf(
		db.c,
		cf.c,
		cStartKey, C.size_t(len(r.Start)),
		cLimitKey, C.size_t(len(r.Limit)),
		&cErr,
	)
	err = fromCError(cErr)

	return
}

// IncreaseFullHistoryTsLow increases the full_history_ts of column family. The new ts_low value should
// be newer than current full_history_ts value.
// If another thread updates full_history_ts_low concurrently to a higher
// timestamp than the requested ts_low, a try again error will be returned.
func (db *DB) IncreaseFullHistoryTsLow(handle *ColumnFamilyHandle, ts []byte) (err error) {
	var cErr *C.char

	cTs := refGoBytes(ts)
	C.rocksdb_increase_full_history_ts_low(db.c, handle.c, cTs, C.size_t(len(ts)), &cErr)

	err = fromCError(cErr)
	return
}

// GetFullHistoryTsLow returns current full_history_ts value.
func (db *DB) GetFullHistoryTsLow(handle *ColumnFamilyHandle) (slice *Slice, err error) {
	var (
		cErr   *C.char
		cTsLen C.size_t
	)

	cTs := C.rocksdb_get_full_history_ts_low(db.c, handle.c, &cTsLen, &cErr)
	if err = fromCError(cErr); err == nil {
		slice = NewSlice(cTs, cTsLen)
	}

	err = fromCError(cErr)
	return
}

// IngestExternalFile loads a list of external SST files.
func (db *DB) IngestExternalFile(filePaths []string, opts *IngestExternalFileOptions) (err error) {
	cFilePaths := make([]*C.char, len(filePaths))
	for i, s := range filePaths {
		cFilePaths[i] = C.CString(s)
	}

	var cErr *C.char

	C.rocksdb_ingest_external_file(
		db.c,
		&cFilePaths[0],
		C.size_t(len(filePaths)),
		opts.c,
		&cErr,
	)
	err = fromCError(cErr)

	// free before return
	for _, s := range cFilePaths {
		C.free(unsafe.Pointer(s))
	}

	return
}

// IngestExternalFileCF loads a list of external SST files for a column family.
func (db *DB) IngestExternalFileCF(handle *ColumnFamilyHandle, filePaths []string, opts *IngestExternalFileOptions) (err error) {
	cFilePaths := make([]*C.char, len(filePaths))
	for i, s := range filePaths {
		cFilePaths[i] = C.CString(s)
	}

	var cErr *C.char

	C.rocksdb_ingest_external_file_cf(
		db.c,
		handle.c,
		&cFilePaths[0],
		C.size_t(len(filePaths)),
		opts.c,
		&cErr,
	)
	err = fromCError(cErr)

	// free before return
	for _, s := range cFilePaths {
		C.free(unsafe.Pointer(s))
	}

	return
}

// NewCheckpoint creates a new Checkpoint for this db.
func (db *DB) NewCheckpoint() (cp *Checkpoint, err error) {
	var cErr *C.char
	cCheckpoint := C.rocksdb_checkpoint_object_create(
		db.c, &cErr,
	)
	if err = fromCError(cErr); err == nil {
		cp = newNativeCheckpoint(cCheckpoint)
	}
	return
}

// TryCatchUpWithPrimary to make the secondary
// instance catch up with primary (WAL tailing is NOT supported now) whenever
// the user feels necessary. Column families created by the primary after the
// secondary instance starts are currently ignored by the secondary instance.
// Column families opened by secondary and dropped by the primary will be
// dropped by secondary as well. However the user of the secondary instance
// can still access the data of such dropped column family as long as they
// do not destroy the corresponding column family handle.
// WAL tailing is not supported at present, but will arrive soon.
func (db *DB) TryCatchUpWithPrimary() (err error) {
	var cErr *C.char
	C.rocksdb_try_catch_up_with_primary(db.c, &cErr)
	err = fromCError(cErr)
	return
}

// CancelAllBackgroundWork requests stopping background work, if wait is true wait until it's done
func (db *DB) CancelAllBackgroundWork(wait bool) {
	C.rocksdb_cancel_all_background_work(db.c, boolToChar(wait))
}

// EnableManualCompaction enables manual compaction.
func (db *DB) EnableManualCompaction() {
	C.rocksdb_enable_manual_compaction(db.c)
}

// DisableManualCompaction disables manual compaction.
func (db *DB) DisableManualCompaction() {
	C.rocksdb_disable_manual_compaction(db.c)
}

// GetColumnFamilyMetadata returns the metadata of the default column family.
func (db *DB) GetColumnFamilyMetadata() (m *ColumnFamilyMetadata) {
	if c := C.rocksdb_get_column_family_metadata(db.c); c != nil {
		m = newColumnFamilyMetadata(c)
	}
	return
}

// GetColumnFamilyMetadataCF returns the metadata of the specified column family.
func (db *DB) GetColumnFamilyMetadataCF(cf *ColumnFamilyHandle) (m *ColumnFamilyMetadata) {
	if c := C.rocksdb_get_column_family_metadata_cf(db.c, cf.c); c != nil {
		m = newColumnFamilyMetadata(c)
	}
	return
}

// WaitForCompact waits for all flush and compactions jobs to finish. Jobs to wait
// include the unscheduled (queued, but not scheduled yet). If the db is shutting down,
// Status::ShutdownInProgress will be returned.
//
// NOTE: This may also never return if there's sufficient ongoing writes that
// keeps flush and compaction going without stopping. The user would have to
// cease all the writes to DB to make this eventually return in a stable
// state. The user may also use timeout option in WaitForCompactOptions to
// make this stop waiting and return when timeout expires.
func (db *DB) WaitForCompact(opts *WaitForCompactOptions) (err error) {
	var cErr *C.char

	C.rocksdb_wait_for_compact(db.c, opts.p, &cErr)
	err = fromCError(cErr)

	return
}

// Close the database.
func (db *DB) Close() {
	C.rocksdb_close(db.c)
	db.c = nil
}

// DestroyDb removes a database entirely, removing everything from the
// filesystem.
func DestroyDb(name string, opts *Options) (err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	C.rocksdb_destroy_db(opts.c, cName, &cErr)
	err = fromCError(cErr)

	C.free(unsafe.Pointer(cName))
	return
}

// RepairDb repairs a database.
func RepairDb(name string, opts *Options) (err error) {
	var (
		cErr  *C.char
		cName = C.CString(name)
	)

	C.rocksdb_repair_db(opts.c, cName, &cErr)
	err = fromCError(cErr)

	C.free(unsafe.Pointer(cName))
	return
}
