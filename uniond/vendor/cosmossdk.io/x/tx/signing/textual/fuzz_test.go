package textual_test

import (
	"bytes"
	"context"
	"encoding/json"
	"os"
	"regexp"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/testing/protocmp"
	tspb "google.golang.org/protobuf/types/known/timestamppb"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/x/tx/internal/testpb"
	"cosmossdk.io/x/tx/signing/textual"
)

func FuzzIntValueRendererParse(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}

	// 1. Firstly add some seeds
	f.Add("10.11")
	f.Add("-10.11")
	f.Add("0.999999")
	f.Add(".999999")
	f.Add("1'000.999999")
	f.Add("1'000'111")
	f.Add("340'282'366'920'938'463'463'374'607'431'768'211'455")

	// 2. Next setup and run the fuzzer.
	ivr := textual.NewIntValueRenderer(fieldDescriptorFromName("UINT64"))
	ctx := context.Background()
	f.Fuzz(func(t *testing.T, input string) {
		_, _ = ivr.Parse(ctx, []textual.Screen{{Content: input}})
	})
}

func FuzzTimestampValueRendererParse(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}

	// 1. Firstly add some seed valid content.
	f.Add("2006-01-02T15:04:05Z")
	f.Add("1970-01-01T00:00:00.00000001Z")
	f.Add("2022-07-14T11:22:20.983Z")
	f.Add("1969-12-31T23:59:59Z")

	// 2. Now fuzz it.
	tvr := textual.NewTimestampValueRenderer()
	ctx := context.Background()
	f.Fuzz(func(t *testing.T, input string) {
		_, _ = tvr.Parse(ctx, []textual.Screen{{Content: input}})
	})
}

func FuzzTimestampJSONParseToParseRoundTrip(f *testing.F) {
	// 1. Use the seeds from testdata and mutate them.
	seed, err := os.ReadFile("./internal/testdata/timestamp.json")
	if err != nil {
		f.Fatal(err)
	}
	f.Add(seed)

	f.Fuzz(func(t *testing.T, input []byte) {
		var testCases []timestampJSONTest
		if err := json.Unmarshal(input, &testCases); err != nil {
			return
		}

		for _, tc := range testCases {
			rend := textual.NewTimestampValueRenderer()

			// If it successfully JSON unmarshals let's test it out.
			var screens []textual.Screen
			var err error

			if tc.Proto != nil {
				screens, err = rend.Format(context.Background(), protoreflect.ValueOf(tc.Proto.ProtoReflect()))
				if err != nil {
					continue
				}
			}

			val, err := rend.Parse(context.Background(), screens)
			if err != nil {
				continue
			}

			msg := val.Message().Interface()
			gotTs, ok := msg.(*tspb.Timestamp)
			if !ok {
				t.Fatalf("Wrong type for timestamp: %T", msg)
			}
			// Please avoid using proto.Equal to compare timestamps given they aren't
			// in standardized form and will produce false positives for example given input:
			//  []byte(`[{"proto":{"nanos":1000000000}}]`)
			// Per issue: https://github.com/cosmos/cosmos-sdk/issues/15761
			if !gotTs.AsTime().Equal(tc.Proto.AsTime()) {
				t.Fatalf("Roundtrip mismatch\n\tGot:  %#v\n\tWant: %#v", gotTs, tc.Proto)
			}
		}
	})
}

func FuzzBytesValueRendererParse(f *testing.F) {
	// 1. Generate some seeds from testdata.
	seed, err := os.ReadFile("./internal/testdata/bytes.json")
	if err != nil {
		f.Fatal(err)
	}
	f.Add(seed)

	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	if err != nil {
		f.Fatal(err)
	}

	ctx := context.Background()

	f.Fuzz(func(t *testing.T, input []byte) {
		var testCases []bytesTest
		if err := json.Unmarshal(input, &testCases); err != nil {
			return
		}

		for _, tc := range testCases {
			rend, err := tr.GetFieldValueRenderer(fieldDescriptorFromName("BYTES"))
			if err != nil {
				t.Fatal(err)
			}

			screens, err := rend.Format(ctx, protoreflect.ValueOfBytes(tc.base64))
			if err != nil {
				t.Fatal(err)
			}
			if g, w := len(screens), 1; g != w {
				t.Fatalf("Mismatch screen count: got=%d, want=%d", g, w)
			}

			// Round trip and test.
			val, err := rend.Parse(ctx, screens)
			if err != nil {
				t.Fatal(err)
			}
			if g, w := len(tc.base64), 35; g > w {
				if len(val.Bytes()) != 0 {
					t.Fatalf("val.Bytes() != 0:\n\tGot:  % x", val.Bytes())
				}
			} else if !bytes.Equal(tc.base64, val.Bytes()) {
				t.Fatalf("val.Bytes() mismatch:\n\tGot:  % x\n\tWant: % x", val.Bytes(), tc.base64)
			}
		}
	})
}

func FuzzMessageValueRendererParse(f *testing.F) {
	if testing.Short() {
		f.Skip()
	}

	// 1. Use the seeds from testdata and mutate them.
	seed, err := os.ReadFile("./internal/testdata/message.json")
	if err != nil {
		f.Fatal(err)
	}
	f.Add(seed)

	ctx := context.Background()
	tr, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: EmptyCoinMetadataQuerier})
	if err != nil {
		f.Fatalf("Failed to create SignModeHandler: %v", err)
	}

	f.Fuzz(func(t *testing.T, input []byte) {
		var testCases []messageJSONTest
		if err := json.Unmarshal(input, &testCases); err != nil {
			return
		}

		for _, tc := range testCases {
			rend := textual.NewMessageValueRenderer(tr, (&testpb.Foo{}).ProtoReflect().Descriptor())

			var screens []textual.Screen
			var err error

			if tc.Proto != nil {
				screens, err = rend.Format(ctx, protoreflect.ValueOf(tc.Proto.ProtoReflect()))
				if err != nil {
					continue
				}
			}

			val, err := rend.Parse(ctx, screens)
			if err != nil {
				continue
			}

			msg := val.Message().Interface()
			gotMsg, ok := msg.(*testpb.Foo)
			if !ok {
				t.Fatalf("Wrong type for Foo: %T", msg)
			}
			diff := cmp.Diff(gotMsg, tc.Proto, protocmp.Transform())
			if diff != "" {
				t.Fatalf("Roundtrip mismatch\n\tGot:  %#v\n\tWant: %#v", gotMsg, tc.Proto)
			}
		}
	})
}

// Copied from types/coin.go but pasted in here so as to avoid any imports
// of that package as has been mandated by team decisions.
var (
	reCoinDenom  = regexp.MustCompile(`[a-zA-Z][a-zA-Z0-9/:._-]{2,127}`)
	reCoinAmount = regexp.MustCompile(`[[:digit:]]+(?:\.[[:digit:]]+)?|\.[[:digit:]]+`)
)

func FuzzCoinsJSONTestcases(f *testing.F) {
	f.Skip() // https://github.com/cosmos/cosmos-sdk/pull/16521#issuecomment-1614507574

	// Generate some seeds.
	seed, err := os.ReadFile("./internal/testdata/coins.json")
	if err != nil {
		f.Fatal(err)
	}
	f.Add(seed)

	txt, err := textual.NewSignModeHandler(textual.SignModeOptions{CoinMetadataQuerier: mockCoinMetadataQuerier})
	if err != nil {
		f.Fatal(err)
	}
	rend, err := txt.GetFieldValueRenderer(fieldDescriptorFromName("COINS"))
	if err != nil {
		f.Fatal(err)
	}
	vrr := rend.(textual.RepeatedValueRenderer)

	f.Fuzz(func(t *testing.T, input []byte) {
		var testCases []coinsJSONTest
		if err := json.Unmarshal(input, &testCases); err != nil {
			return
		}

		for _, tc := range testCases {
			if tc.Proto == nil {
				continue
			}

			// Create a context.Context containing all coins metadata, to simulate
			// that they are in state.
			ctx := context.Background()
			for _, v := range tc.Metadata {
				ctx = addMetadataToContext(ctx, v)
			}

			listValue := NewGenericList(tc.Proto)
			screens, err := vrr.FormatRepeated(ctx, protoreflect.ValueOf(listValue))
			if err != nil {
				cpt := tc.Proto[0]
				likeEmpty := err.Error() == "cannot format empty string" || err.Error() == "decimal string cannot be empty"
				if likeEmpty && (!reCoinDenom.MatchString(cpt.Denom) || cpt.Amount == "") {
					return
				}
				if !reCoinDenom.MatchString(cpt.Denom) {
					return
				}
				if !reCoinAmount.MatchString(cpt.Amount) {
					return
				}
				t.Fatalf("%v\n%q\n%#v => %t", err, tc.Text, cpt, cpt.Amount == "")
			}

			if g, w := len(screens), 1; g != w {
				t.Fatalf("Screens mismatch: got=%d want=%d", g, w)
			}

			wantContent := tc.Text
			if wantContent == "" {
				wantContent = "zero"
			}
			if false {
				if g, w := screens[0].Content, wantContent; g != w {
					t.Fatalf("Content mismatch:\n\tGot:  %s\n\tWant: %s", g, w)
				}
			}

			// Round trip.
			parsedValue := NewGenericList([]*basev1beta1.Coin{})
			if err := vrr.ParseRepeated(ctx, screens, parsedValue); err != nil {
				return
			}
			checkCoinsEqual(t, listValue, parsedValue)
		}
	})
}
