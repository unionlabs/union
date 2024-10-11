package types_test

import (
	"errors"
	"runtime"
	"testing"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

type errOnMarshal struct {
	testdata.Dog
}

var _ proto.Message = (*errOnMarshal)(nil)

var errAlways = errors.New("always erroring")

func (eom *errOnMarshal) XXX_Marshal(b []byte, deterministic bool) ([]byte, error) {
	return nil, errAlways
}

var eom = &errOnMarshal{}

// Ensure that returning an error doesn't suddenly allocate and waste bytes.
// See https://github.com/cosmos/cosmos-sdk/issues/8537
func TestNewAnyWithCustomTypeURLWithErrorNoAllocation(t *testing.T) {
	// This tests continues to fail inconsistently.
	//
	// Example: https://github.com/cosmos/cosmos-sdk/pull/9246/checks?check_run_id=2643313958#step:6:118
	// Ref: https://github.com/cosmos/cosmos-sdk/issues/9010
	t.SkipNow()

	// make sure we're not in the middle of a GC.
	runtime.GC()

	var ms1, ms2 runtime.MemStats
	runtime.ReadMemStats(&ms1)
	any, err := types.NewAnyWithValue(eom)
	runtime.ReadMemStats(&ms2)
	// Ensure that no fresh allocation was made.
	if diff := ms2.HeapAlloc - ms1.HeapAlloc; diff > 0 {
		t.Errorf("Unexpected allocation of %d bytes", diff)
	}
	if err == nil {
		t.Fatal("err wasn't returned")
	}
	if any != nil {
		t.Fatalf("Unexpectedly got a non-nil Any value: %v", any)
	}
}

var sink interface{}

func BenchmarkNewAnyWithCustomTypeURLWithErrorReturned(b *testing.B) {
	b.ResetTimer()
	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		any, err := types.NewAnyWithValue(eom)
		if err == nil {
			b.Fatal("err wasn't returned")
		}
		if any != nil {
			b.Fatalf("Unexpectedly got a non-nil Any value: %v", any)
		}
		sink = any
	}
	if sink == nil {
		b.Fatal("benchmark didn't run")
	}
	sink = (interface{})(nil)
}
