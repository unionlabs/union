package unknownproto_test

import (
	"sync"
	"testing"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/codec/unknownproto"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

var n1BBlob []byte

func init() {
	n1B := &testdata.Nested1B{
		Id:  1,
		Age: 99,
		Nested: &testdata.Nested2B{
			Id:    2,
			Route: "Wintery route",
			Fee:   99,
			Nested: &testdata.Nested3B{
				Id:   3,
				Name: "3A this one that one there those oens",
				Age:  4588,
				B4: []*testdata.Nested4B{
					{
						Id:   4,
						Age:  88,
						Name: "Nested4B",
					},
				},
			},
		},
	}

	var err error
	n1BBlob, err = proto.Marshal(n1B)
	if err != nil {
		panic(err)
	}
}

func BenchmarkRejectUnknownFields_serial(b *testing.B) {
	benchmarkRejectUnknownFields(b, false)
}

func BenchmarkRejectUnknownFields_parallel(b *testing.B) {
	benchmarkRejectUnknownFields(b, true)
}

func benchmarkRejectUnknownFields(b *testing.B, parallel bool) {
	b.Helper()
	b.ReportAllocs()

	if !parallel {
		b.ResetTimer()
		for i := 0; i < b.N; i++ {
			n1A := new(testdata.Nested1A)
			if err := unknownproto.RejectUnknownFieldsStrict(n1BBlob, n1A, unknownproto.DefaultAnyResolver{}); err == nil {
				b.Fatal("expected an error")
			}
			b.SetBytes(int64(len(n1BBlob)))
		}
	} else {
		var mu sync.Mutex
		b.ResetTimer()
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				// To simulate the conditions of multiple transactions being processed in parallel.
				n1A := new(testdata.Nested1A)
				if err := unknownproto.RejectUnknownFieldsStrict(n1BBlob, n1A, unknownproto.DefaultAnyResolver{}); err == nil {
					b.Fatal("expected an error")
				}
				mu.Lock()
				b.SetBytes(int64(len(n1BBlob)))
				mu.Unlock()
			}
		})
	}
}

func BenchmarkProtoUnmarshal_serial(b *testing.B) {
	benchmarkProtoUnmarshal(b, false)
}

func BenchmarkProtoUnmarshal_parallel(b *testing.B) {
	benchmarkProtoUnmarshal(b, true)
}

func benchmarkProtoUnmarshal(b *testing.B, parallel bool) {
	b.Helper()
	b.ReportAllocs()

	if !parallel {
		for i := 0; i < b.N; i++ {
			n1A := new(testdata.Nested1A)
			if err := proto.Unmarshal(n1BBlob, n1A); err == nil {
				b.Fatal("expected an error")
			}
			b.SetBytes(int64(len(n1BBlob)))
		}
	} else {
		var mu sync.Mutex
		b.RunParallel(func(pb *testing.PB) {
			for pb.Next() {
				n1A := new(testdata.Nested1A)
				if err := proto.Unmarshal(n1BBlob, n1A); err == nil {
					b.Fatal("expected an error")
				}
				mu.Lock()
				b.SetBytes(int64(len(n1BBlob)))
				mu.Unlock()
			}
		})
	}
}
