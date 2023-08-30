//go:build !windows
// +build !windows

package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

import "unsafe"

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
