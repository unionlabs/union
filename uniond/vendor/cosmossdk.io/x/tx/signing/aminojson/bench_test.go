package aminojson_test

import (
	"testing"

	"cosmossdk.io/x/tx/signing/aminojson"
	"cosmossdk.io/x/tx/signing/aminojson/internal/testpb"
)

var sink any

var msg = &testpb.ABitOfEverything{
	Message: &testpb.NestedMessage{
		Foo: "test",
		Bar: 0, // this is the default value and should be omitted from output
	},
	Enum:     testpb.AnEnum_ONE,
	Repeated: []int32{3, -7, 2, 6, 4},
	Str:      `abcxyz"foo"def`,
	Bool:     true,
	Bytes:    []byte{0, 1, 2, 3},
	I32:      -15,
	F32:      1001,
	U32:      1200,
	Si32:     -376,
	Sf32:     -1000,
	I64:      14578294827584932,
	F64:      9572348124213523654,
	U64:      4759492485,
	Si64:     -59268425823934,
	Sf64:     -659101379604211154,
}

func BenchmarkAminoJSONNaiveSort(b *testing.B) {
	benchmarkAminoJSON(b, true)
}

func BenchmarkAminoJSONDefaultSort(b *testing.B) {
	benchmarkAminoJSON(b, false)
}

func benchmarkAminoJSON(b *testing.B, addNaiveSort bool) {
	b.Helper()
	enc := aminojson.NewEncoder(aminojson.EncoderOptions{DoNotSortFields: addNaiveSort})
	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		sink = runAminoJSON(b, enc, addNaiveSort)
	}
	if sink == nil {
		b.Fatal("Benchmark was not run")
	}
	sink = nil
}

func runAminoJSON(b *testing.B, enc aminojson.Encoder, addNaiveSort bool) []byte {
	b.Helper()
	bz, err := enc.Marshal(msg)
	if err != nil {
		b.Fatal(err)
	}

	if addNaiveSort {
		return naiveSortedJSON(b, bz)
	}
	return bz
}
