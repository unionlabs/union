package api

/*
#include "bindings.h"
#include <stdio.h>

// imports (db)
GoError cSet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, UnmanagedVector *errOut);
GoError cGet(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cDelete(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *errOut);
GoError cScan(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, UnmanagedVector *errOut);
// imports (iterator)
GoError cNext(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cNextKey(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *errOut);
GoError cNextValue(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *value, UnmanagedVector *errOut);
// imports (api)
GoError cHumanizeAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cCanonicalizeAddress(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cValidateAddress(api_t *ptr, U8SliceView src, UnmanagedVector *errOut, uint64_t *used_gas);
// imports (querier)
GoError cQueryExternal(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, UnmanagedVector *result, UnmanagedVector *errOut);

// Gateway functions (db)
GoError cGet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cGet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoError cSet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, UnmanagedVector *errOut) {
	return cSet(ptr, gas_meter, used_gas, key, val, errOut);
}
GoError cDelete_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *errOut) {
	return cDelete(ptr, gas_meter, used_gas, key, errOut);
}
GoError cScan_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, UnmanagedVector *errOut) {
	return cScan(ptr, gas_meter, used_gas, start, end, order, out, errOut);
}

// Gateway functions (iterator)
GoError cNext_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cNext(ref, gas_meter, used_gas, key, val, errOut);
}
GoError cNextKey_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *errOut) {
	return cNextKey(ref, gas_meter, used_gas, key, errOut);
}
GoError cNextValue_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *val, UnmanagedVector *errOut) {
	return cNextValue(ref, gas_meter, used_gas, val, errOut);
}

// Gateway functions (api)
GoError cCanonicalizeAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas) {
    return cCanonicalizeAddress(ptr, src, dest, errOut, used_gas);
}
GoError cHumanizeAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas) {
    return cHumanizeAddress(ptr, src, dest, errOut, used_gas);
}
GoError cValidateAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *errOut, uint64_t *used_gas) {
    return cValidateAddress(ptr, src, errOut, used_gas);
}

// Gateway functions (querier)
GoError cQueryExternal_cgo(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, UnmanagedVector *result, UnmanagedVector *errOut) {
    return cQueryExternal(ptr, gas_limit, used_gas, request, result, errOut);
}
*/
import "C"

// We need these gateway functions to allow calling back to a go function from the c code.
// At least I didn't discover a cleaner way.
// Also, this needs to be in a different file than `callbacks.go`, as we cannot create functions
// in the same file that has //export directives. Only import header types
