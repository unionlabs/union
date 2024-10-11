package textual_test

import (
	"bytes"
	"context"
	"encoding/json"
	"os"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

var intValues = []protoreflect.Value{
	protoreflect.ValueOfString("1000"),
	protoreflect.ValueOfString("99900"),
	protoreflect.ValueOfString("9999999"),
	protoreflect.ValueOfString("999999999999"),
	protoreflect.ValueOfString("9999999999999999999"),
	protoreflect.ValueOfString("100000000000000000000000000000000000000000000000000000000"),
	protoreflect.ValueOfString("77777777777777777777777777777777700"),
	protoreflect.ValueOfString("-77777777777777777777777777777777700"),
	protoreflect.ValueOfString("77777777777777777777777777777777700"),
}

func BenchmarkIntValueRendererFormat(b *testing.B) {
	ctx := context.Background()
	ivr := textual.NewIntValueRenderer(fieldDescriptorFromName("UINT64"))
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		for _, value := range intValues {
			if _, err := ivr.Format(ctx, value); err != nil {
				b.Fatal(err)
			}
		}
	}
}

var decimalValues = []protoreflect.Value{
	protoreflect.ValueOfString("10.00"),
	protoreflect.ValueOfString("999.00"),
	protoreflect.ValueOfString("999.9999"),
	protoreflect.ValueOfString("99999999.9999"),
	protoreflect.ValueOfString("9999999999999999999"),
	protoreflect.ValueOfString("1000000000000000000000000000000000000000000000000000000.00"),
	protoreflect.ValueOfString("77777777777.777777777777777777777700"),
	protoreflect.ValueOfString("-77777777777.777777777777777777777700"),
	protoreflect.ValueOfString("777777777777777777777777.77777777700"),
}

func BenchmarkDecimalValueRendererFormat(b *testing.B) {
	ctx := context.Background()
	dvr := textual.NewDecValueRenderer()
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		for _, value := range decimalValues {
			if _, err := dvr.Format(ctx, value); err != nil {
				b.Fatal(err)
			}
		}
	}
}

var byteValues = []protoreflect.Value{
	protoreflect.ValueOfBytes(bytes.Repeat([]byte("abc"), 1<<20)),
	protoreflect.ValueOfBytes([]byte("999.00")),
	protoreflect.ValueOfBytes([]byte("999.9999")),
	protoreflect.ValueOfBytes([]byte("99999999.9999")),
	protoreflect.ValueOfBytes([]byte("9999999999999999999")),
	protoreflect.ValueOfBytes([]byte("1000000000000000000000000000000000000000000000000000000.00")),
	protoreflect.ValueOfBytes([]byte("77777777777.777777777777777777777700")),
	protoreflect.ValueOfBytes([]byte("-77777777777.777777777777777777777700")),
	protoreflect.ValueOfBytes([]byte("777777777777777777777777.77777777700")),
}

func BenchmarkBytesValueRendererFormat(b *testing.B) {
	ctx := context.Background()
	bvr := textual.NewBytesValueRenderer()
	b.ResetTimer()
	b.ReportAllocs()

	for i := 0; i < b.N; i++ {
		for _, value := range byteValues {
			if _, err := bvr.Format(ctx, value); err != nil {
				b.Fatal(err)
			}
		}
	}
}

var sink any

func BenchmarkMessageValueRenderer_parseRepeated(b *testing.B) {
	ctx := context.Background()
	raw, err := os.ReadFile("./internal/testdata/repeated.json")
	require.NoError(b, err)

	type rendScreens struct {
		rend    textual.ValueRenderer
		screens []textual.Screen
	}

	var rsL []*rendScreens

	var testCases []repeatedJSONTest
	err = json.Unmarshal(raw, &testCases)
	require.NoError(b, err)

	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: mockCoinMetadataQuerier})
	for _, tc := range testCases {
		rend := textual.NewMessageValueRenderer(tr, (&testpb.Qux{}).ProtoReflect().Descriptor())
		require.NoError(b, err)

		screens, err := rend.Format(ctx, protoreflect.ValueOf(tc.Proto.ProtoReflect()))
		require.NoError(b, err)
		require.Equal(b, tc.Screens, screens)

		rsL = append(rsL, &rendScreens{
			rend:    rend,
			screens: screens,
		})
	}

	b.ReportAllocs()
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		for _, rs := range rsL {
			sink, _ = rs.rend.Parse(ctx, rs.screens)
		}
	}

	if sink == nil {
		b.Fatal("Benchmark did not run!")
	}
	// Reset the sink for reuse.
	sink = nil
}
