package api

// Check https://akrennmair.github.io/golang-cgo-slides/ to learn
// how this embedded C code works.

/*
#include "bindings.h"

// All C function types in struct fields will be represented as a *[0]byte in Go and
// we don't get any type safety on the signature. To express this fact in type conversions,
// we create a single function pointer type here.
// The only thing this is used for is casting between unsafe.Pointer and *[0]byte in Go.
// See also https://github.com/golang/go/issues/19835
typedef void (*any_function_t)();

// forward declarations (db)
GoError cGet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cSet_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, U8SliceView val, UnmanagedVector *errOut);
GoError cDelete_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView key, UnmanagedVector *errOut);
GoError cScan_cgo(db_t *ptr, gas_meter_t *gas_meter, uint64_t *used_gas, U8SliceView start, U8SliceView end, int32_t order, GoIter *out, UnmanagedVector *errOut);
// iterator
GoError cNext_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *val, UnmanagedVector *errOut);
GoError cNextKey_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *key, UnmanagedVector *errOut);
GoError cNextValue_cgo(IteratorReference *ref, gas_meter_t *gas_meter, uint64_t *used_gas, UnmanagedVector *val, UnmanagedVector *errOut);
// api
GoError cHumanizeAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cCanonicalizeAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *dest, UnmanagedVector *errOut, uint64_t *used_gas);
GoError cValidateAddress_cgo(api_t *ptr, U8SliceView src, UnmanagedVector *errOut, uint64_t *used_gas);
// and querier
GoError cQueryExternal_cgo(querier_t *ptr, uint64_t gas_limit, uint64_t *used_gas, U8SliceView request, UnmanagedVector *result, UnmanagedVector *errOut);


*/
import "C"

import (
	"encoding/json"
	"fmt"
	"log"
	"reflect"
	"runtime/debug"
	"unsafe"

	"github.com/CosmWasm/wasmvm/v2/types"
)

// Note: we have to include all exports in the same file (at least since they both import bindings.h),
// or get odd cgo build errors about duplicate definitions

func recoverPanic(ret *C.GoError) {
	if rec := recover(); rec != nil {
		// This is used to handle ErrorOutOfGas panics.
		//
		// What we do here is something that should not be done in the first place.
		// "A panic typically means something went unexpectedly wrong. Mostly we use it to fail fast
		// on errors that shouldn’t occur during normal operation, or that we aren’t prepared to
		// handle gracefully." says https://gobyexample.com/panic.
		// And 'Ask yourself "when this happens, should the application immediately crash?" If yes,
		// use a panic; otherwise, use an error.' says this popular answer on SO: https://stackoverflow.com/a/44505268.
		// Oh, and "If you're already worrying about discriminating different kinds of panics, you've lost sight of the ball."
		// (Rob Pike) from https://eli.thegreenplace.net/2018/on-the-uses-and-misuses-of-panics-in-go/
		//
		// We don't want to import Cosmos SDK and also cannot use interfaces to detect these
		// error types (as they have no methods). So, let's just rely on the descriptive names.
		name := reflect.TypeOf(rec).Name()
		switch name {
		// These three types are "thrown" (which is not a thing in Go 🙃) in panics from the gas module
		// (https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/store/types/gas.go):
		// 1. ErrorOutOfGas
		// 2. ErrorGasOverflow
		// 3. ErrorNegativeGasConsumed
		//
		// In the baseapp, ErrorOutOfGas gets special treatment:
		// - https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/baseapp/baseapp.go#L607
		// - https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/baseapp/recovery.go#L50-L60
		// This turns the panic into a regular error with a helpful error message.
		//
		// The other two gas related panic types indicate programming errors and are handled along
		// with all other errors in https://github.com/cosmos/cosmos-sdk/blob/v0.45.4/baseapp/recovery.go#L66-L77.
		case "ErrorOutOfGas":
			// TODO: figure out how to pass the text in its `Descriptor` field through all the FFI
			*ret = C.GoError_OutOfGas
		default:
			log.Printf("Panic in Go callback: %#v\n", rec)
			debug.PrintStack()
			*ret = C.GoError_Panic
		}
	}
}

/****** DB ********/

var db_vtable = C.DbVtable{
	read_db:   C.any_function_t(C.cGet_cgo),
	write_db:  C.any_function_t(C.cSet_cgo),
	remove_db: C.any_function_t(C.cDelete_cgo),
	scan_db:   C.any_function_t(C.cScan_cgo),
}

type DBState struct {
	Store types.KVStore
	// CallID is used to lookup the proper frame for iterators associated with this contract call (iterator.go)
	CallID uint64
}

// use this to create C.Db in two steps, so the pointer lives as long as the calling stack
//
//	state := buildDBState(kv, callID)
//	db := buildDB(&state, &gasMeter)
//	// then pass db into some FFI function
func buildDBState(kv types.KVStore, callID uint64) DBState {
	return DBState{
		Store:  kv,
		CallID: callID,
	}
}

// contract: original pointer/struct referenced must live longer than C.Db struct
// since this is only used internally, we can verify the code that this is the case
func buildDB(state *DBState, gm *types.GasMeter) C.Db {
	return C.Db{
		gas_meter: (*C.gas_meter_t)(unsafe.Pointer(gm)),
		state:     (*C.db_t)(unsafe.Pointer(state)),
		vtable:    db_vtable,
	}
}

var iterator_vtable = C.IteratorVtable{
	next:       C.any_function_t(C.cNext_cgo),
	next_key:   C.any_function_t(C.cNextKey_cgo),
	next_value: C.any_function_t(C.cNextValue_cgo),
}

// An iterator including referenced objects is 117 bytes large (calculated using https://github.com/DmitriyVTitov/size).
// We limit the number of iterators per contract call ID here in order limit memory usage to 32768*117 = ~3.8 MB as a safety measure.
// In any reasonable contract, gas limits should hit sooner than that though.
const frameLenLimit = 32768

// contract: original pointer/struct referenced must live longer than C.Db struct
// since this is only used internally, we can verify the code that this is the case
func buildIterator(callID uint64, it types.Iterator) (C.IteratorReference, error) {
	iteratorID, err := storeIterator(callID, it, frameLenLimit)
	if err != nil {
		return C.IteratorReference{}, err
	}
	return C.IteratorReference{
		call_id:     cu64(callID),
		iterator_id: cu64(iteratorID),
	}, nil
}

//export cGet
func cGet(ptr *C.db_t, gasMeter *C.gas_meter_t, usedGas *cu64, key C.U8SliceView, val *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || gasMeter == nil || usedGas == nil || val == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	// errOut is unused and we don't check `is_none` because of https://github.com/CosmWasm/wasmvm/issues/536
	if !(*val).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	kv := *(*types.KVStore)(unsafe.Pointer(ptr))
	k := copyU8Slice(key)

	gasBefore := gm.GasConsumed()
	v := kv.Get(k)
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	// v will equal nil when the key is missing
	// https://github.com/cosmos/cosmos-sdk/blob/1083fa948e347135861f88e07ec76b0314296832/store/types/store.go#L174
	*val = newUnmanagedVector(v)

	return C.GoError_None
}

//export cSet
func cSet(ptr *C.db_t, gasMeter *C.gas_meter_t, usedGas *cu64, key C.U8SliceView, val C.U8SliceView, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || gasMeter == nil || usedGas == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	// errOut is unused and we don't check `is_none` because of https://github.com/CosmWasm/wasmvm/issues/536

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	kv := *(*types.KVStore)(unsafe.Pointer(ptr))
	k := copyU8Slice(key)
	v := copyU8Slice(val)

	gasBefore := gm.GasConsumed()
	kv.Set(k, v)
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	return C.GoError_None
}

//export cDelete
func cDelete(ptr *C.db_t, gasMeter *C.gas_meter_t, usedGas *cu64, key C.U8SliceView, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || gasMeter == nil || usedGas == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	// errOut is unused and we don't check `is_none` because of https://github.com/CosmWasm/wasmvm/issues/536

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	kv := *(*types.KVStore)(unsafe.Pointer(ptr))
	k := copyU8Slice(key)

	gasBefore := gm.GasConsumed()
	kv.Delete(k)
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	return C.GoError_None
}

//export cScan
func cScan(ptr *C.db_t, gasMeter *C.gas_meter_t, usedGas *cu64, start C.U8SliceView, end C.U8SliceView, order ci32, out *C.GoIter, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || gasMeter == nil || usedGas == nil || out == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	state := (*DBState)(unsafe.Pointer(ptr))
	kv := state.Store
	s := copyU8Slice(start)
	e := copyU8Slice(end)

	var iter types.Iterator
	gasBefore := gm.GasConsumed()
	switch order {
	case 1: // Ascending
		iter = kv.Iterator(s, e)
	case 2: // Descending
		iter = kv.ReverseIterator(s, e)
	default:
		return C.GoError_BadArgument
	}
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	iteratorRef, err := buildIterator(state.CallID, iter)
	if err != nil {
		// store the actual error message in the return buffer
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_User
	}

	*out = C.GoIter{
		gas_meter: gasMeter,
		reference: iteratorRef,
		vtable:    iterator_vtable,
	}

	return C.GoError_None
}

//export cNext
func cNext(ref C.IteratorReference, gasMeter *C.gas_meter_t, usedGas *cu64, key *C.UnmanagedVector, val *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	// typical usage of iterator
	// 	for ; itr.Valid(); itr.Next() {
	// 		k, v := itr.Key(); itr.Value()
	// 		...
	// 	}

	defer recoverPanic(&ret)
	if ref.call_id == 0 || gasMeter == nil || usedGas == nil || key == nil || val == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	// errOut is unused and we don't check `is_none` because of https://github.com/CosmWasm/wasmvm/issues/536
	if !(*key).is_none || !(*val).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	iter := retrieveIterator(uint64(ref.call_id), uint64(ref.iterator_id))
	if iter == nil {
		panic("Unable to retrieve iterator.")
	}
	if !iter.Valid() {
		// end of iterator, return as no-op, nil key is considered end
		return C.GoError_None
	}

	gasBefore := gm.GasConsumed()
	// call Next at the end, upon creation we have first data loaded
	k := iter.Key()
	v := iter.Value()
	// check iter.Error() ????
	iter.Next()
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	*key = newUnmanagedVector(k)
	*val = newUnmanagedVector(v)
	return C.GoError_None
}

//export cNextKey
func cNextKey(ref C.IteratorReference, gasMeter *C.gas_meter_t, usedGas *cu64, key *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	return nextPart(ref, gasMeter, usedGas, key, errOut, func(iter types.Iterator) []byte { return iter.Key() })
}

//export cNextValue
func cNextValue(ref C.IteratorReference, gasMeter *C.gas_meter_t, usedGas *cu64, value *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	return nextPart(ref, gasMeter, usedGas, value, errOut, func(iter types.Iterator) []byte { return iter.Value() })
}

// nextPart is a helper function that contains the shared code for key- and value-only iteration.
func nextPart(ref C.IteratorReference, gasMeter *C.gas_meter_t, usedGas *cu64, output *C.UnmanagedVector, errOut *C.UnmanagedVector, valFn func(types.Iterator) []byte) (ret C.GoError) {
	// typical usage of iterator
	// 	for ; itr.Valid(); itr.Next() {
	// 		k, v := itr.Key(); itr.Value()
	// 		...
	// 	}

	defer recoverPanic(&ret)
	if ref.call_id == 0 || gasMeter == nil || usedGas == nil || output == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	// errOut is unused and we don't check `is_none` because of https://github.com/CosmWasm/wasmvm/issues/536
	if !(*output).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	gm := *(*types.GasMeter)(unsafe.Pointer(gasMeter))
	iter := retrieveIterator(uint64(ref.call_id), uint64(ref.iterator_id))
	if iter == nil {
		panic("Unable to retrieve iterator.")
	}
	if !iter.Valid() {
		// end of iterator, return as no-op, nil `output` is considered end
		return C.GoError_None
	}

	gasBefore := gm.GasConsumed()
	// call Next at the end, upon creation we have first data loaded
	out := valFn(iter)
	// check iter.Error() ????
	iter.Next()
	gasAfter := gm.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	*output = newUnmanagedVector(out)
	return C.GoError_None
}

var api_vtable = C.GoApiVtable{
	humanize_address:     C.any_function_t(C.cHumanizeAddress_cgo),
	canonicalize_address: C.any_function_t(C.cCanonicalizeAddress_cgo),
	validate_address:     C.any_function_t(C.cValidateAddress_cgo),
}

// contract: original pointer/struct referenced must live longer than C.GoApi struct
// since this is only used internally, we can verify the code that this is the case
func buildAPI(api *types.GoAPI) C.GoApi {
	return C.GoApi{
		state:  (*C.api_t)(unsafe.Pointer(api)),
		vtable: api_vtable,
	}
}

//export cHumanizeAddress
func cHumanizeAddress(ptr *C.api_t, src C.U8SliceView, dest *C.UnmanagedVector, errOut *C.UnmanagedVector, used_gas *cu64) (ret C.GoError) {
	defer recoverPanic(&ret)

	if dest == nil || errOut == nil {
		return C.GoError_BadArgument
	}
	if !(*dest).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	api := (*types.GoAPI)(unsafe.Pointer(ptr))
	s := copyU8Slice(src)

	h, cost, err := api.HumanizeAddress(s)
	*used_gas = cu64(cost)
	if err != nil {
		// store the actual error message in the return buffer
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_User
	}
	if len(h) == 0 {
		panic(fmt.Sprintf("`api.HumanizeAddress()` returned an empty string for %q", s))
	}
	*dest = newUnmanagedVector([]byte(h))
	return C.GoError_None
}

//export cCanonicalizeAddress
func cCanonicalizeAddress(ptr *C.api_t, src C.U8SliceView, dest *C.UnmanagedVector, errOut *C.UnmanagedVector, used_gas *cu64) (ret C.GoError) {
	defer recoverPanic(&ret)

	if dest == nil || errOut == nil {
		return C.GoError_BadArgument
	}
	if !(*dest).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	api := (*types.GoAPI)(unsafe.Pointer(ptr))
	s := string(copyU8Slice(src))
	c, cost, err := api.CanonicalizeAddress(s)
	*used_gas = cu64(cost)
	if err != nil {
		// store the actual error message in the return buffer
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_User
	}
	if len(c) == 0 {
		panic(fmt.Sprintf("`api.CanonicalizeAddress()` returned an empty string for %q", s))
	}
	*dest = newUnmanagedVector(c)
	return C.GoError_None
}

//export cValidateAddress
func cValidateAddress(ptr *C.api_t, src C.U8SliceView, errOut *C.UnmanagedVector, used_gas *cu64) (ret C.GoError) {
	defer recoverPanic(&ret)

	if errOut == nil {
		return C.GoError_BadArgument
	}
	if !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	api := (*types.GoAPI)(unsafe.Pointer(ptr))
	s := string(copyU8Slice(src))
	cost, err := api.ValidateAddress(s)

	*used_gas = cu64(cost)
	if err != nil {
		// store the actual error message in the return buffer
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_User
	}
	return C.GoError_None
}

/****** Go Querier ********/

var querier_vtable = C.QuerierVtable{
	query_external: C.any_function_t(C.cQueryExternal_cgo),
}

// contract: original pointer/struct referenced must live longer than C.GoQuerier struct
// since this is only used internally, we can verify the code that this is the case
func buildQuerier(q *Querier) C.GoQuerier {
	return C.GoQuerier{
		state:  (*C.querier_t)(unsafe.Pointer(q)),
		vtable: querier_vtable,
	}
}

//export cQueryExternal
func cQueryExternal(ptr *C.querier_t, gasLimit cu64, usedGas *cu64, request C.U8SliceView, result *C.UnmanagedVector, errOut *C.UnmanagedVector) (ret C.GoError) {
	defer recoverPanic(&ret)

	if ptr == nil || usedGas == nil || result == nil || errOut == nil {
		// we received an invalid pointer
		return C.GoError_BadArgument
	}
	if !(*result).is_none || !(*errOut).is_none {
		panic("Got a non-none UnmanagedVector we're about to override. This is a bug because someone has to drop the old one.")
	}

	// query the data
	querier := *(*Querier)(unsafe.Pointer(ptr))
	req := copyU8Slice(request)

	gasBefore := querier.GasConsumed()
	res := types.RustQuery(querier, req, uint64(gasLimit))
	gasAfter := querier.GasConsumed()
	*usedGas = (cu64)(gasAfter - gasBefore)

	// serialize the response
	bz, err := json.Marshal(res)
	if err != nil {
		*errOut = newUnmanagedVector([]byte(err.Error()))
		return C.GoError_CannotSerialize
	}
	*result = newUnmanagedVector(bz)
	return C.GoError_None
}
