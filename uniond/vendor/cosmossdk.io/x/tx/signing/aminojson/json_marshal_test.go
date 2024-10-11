package aminojson_test

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"reflect"
	"testing"
	"time"

	"github.com/cosmos/cosmos-proto/rapidproto"
	"github.com/stretchr/testify/require"
	"github.com/tendermint/go-amino"
	"google.golang.org/protobuf/encoding/protojson"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/durationpb"
	"gotest.tools/v3/assert"
	"pgregory.net/rapid"

	"cosmossdk.io/x/tx/signing/aminojson"
	"cosmossdk.io/x/tx/signing/aminojson/internal/aminojsonpb"
	"cosmossdk.io/x/tx/signing/aminojson/internal/testpb"
)

func marshalLegacy(msg proto.Message) ([]byte, error) {
	cdc := amino.NewCodec()
	cdc.RegisterConcrete(&testpb.ABitOfEverything{}, "ABitOfEverything", nil)
	cdc.RegisterConcrete(&testpb.NestedMessage{}, "NestedMessage", nil)
	return cdc.MarshalJSON(msg)
}

func TestAminoJSON_EdgeCases(t *testing.T) {
	cdc := amino.NewCodec()
	cdc.RegisterConcrete(&testpb.ABitOfEverything{}, "ABitOfEverything", nil)
	cdc.RegisterConcrete(&testpb.NestedMessage{}, "NestedMessage", nil)
	aj := aminojson.NewEncoder(aminojson.EncoderOptions{})

	cases := map[string]struct {
		msg       proto.Message
		shouldErr bool
	}{
		"empty":      {msg: &testpb.ABitOfEverything{}},
		"single map": {msg: &testpb.WithAMap{StrMap: map[string]string{"foo": "bar"}}, shouldErr: true},
	}
	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			bz, err := aj.Marshal(tc.msg)

			if tc.shouldErr {
				require.Error(t, err)
				return
			}

			require.NoError(t, err)

			rv := reflect.New(reflect.TypeOf(tc.msg).Elem()).Elem()
			msg2 := rv.Addr().Interface().(proto.Message)

			legacyBz, err := cdc.MarshalJSON(tc.msg)
			assert.NilError(t, err)

			require.Equal(t, string(legacyBz), string(bz))

			goProtoJSON, err := protojson.Marshal(tc.msg)
			assert.NilError(t, err)
			err = cdc.UnmarshalJSON(bz, msg2)
			assert.NilError(t, err, "unmarshal failed: %s vs %s", legacyBz, goProtoJSON)
		})
	}
}

func TestAminoJSON(t *testing.T) {
	cdc := amino.NewCodec()
	cdc.RegisterConcrete(&testpb.ABitOfEverything{}, "ABitOfEverything", nil)
	cdc.RegisterConcrete(&testpb.NestedMessage{}, "NestedMessage", nil)

	msg := &testpb.ABitOfEverything{
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

	unsortedBz, err := aminojson.NewEncoder(aminojson.EncoderOptions{DoNotSortFields: true}).Marshal(msg)
	assert.NilError(t, err)
	legacyBz, err := cdc.MarshalJSON(msg)
	assert.NilError(t, err)
	require.Equal(t, string(legacyBz), string(unsortedBz))

	// Now ensure that the default encoder behavior sorts fields and that they match
	// as we'd have them from encoding/json.Marshal.
	// Please see https://github.com/cosmos/cosmos-sdk/issues/2350
	encodedDefaultBz, err := aminojson.NewEncoder(aminojson.EncoderOptions{}).Marshal(msg)
	assert.NilError(t, err)

	// Ensure that it is NOT equal to the legacy JSON but that it is equal to the sorted JSON.
	require.NotEqual(t, string(legacyBz), string(encodedDefaultBz))

	// Now ensure that the legacy's sortedJSON is as the aminojson.Encoder would produce.
	// This proves that we can eliminate the use of sdk.*SortJSON(encoderBz)
	sortedBz := naiveSortedJSON(t, unsortedBz)
	require.Equal(t, string(sortedBz), string(encodedDefaultBz))
}

func naiveSortedJSON(tb testing.TB, jsonToSort []byte) []byte {
	tb.Helper()
	var c interface{}
	err := json.Unmarshal(jsonToSort, &c)
	assert.NilError(tb, err)
	sortedBz, err := json.Marshal(c)
	assert.NilError(tb, err)
	return sortedBz
}

func TestRapid(t *testing.T) {
	gen := rapidproto.MessageGenerator(&testpb.ABitOfEverything{}, rapidproto.GeneratorOptions{})
	rapid.Check(t, func(t *rapid.T) {
		msg := gen.Draw(t, "msg")
		bz, err := aminojson.NewEncoder(aminojson.EncoderOptions{DoNotSortFields: true}).Marshal(msg)
		assert.NilError(t, err)
		checkInvariants(t, msg, bz)
	})
}

func checkInvariants(t *rapid.T, message proto.Message, marshaledBytes []byte) {
	checkLegacyParity(t, message, marshaledBytes)
	checkRoundTrip(t, message, marshaledBytes)
}

func checkLegacyParity(t *rapid.T, message proto.Message, marshaledBytes []byte) {
	legacyBz, err := marshalLegacy(message)
	assert.NilError(t, err)
	require.Equal(t, string(legacyBz), string(marshaledBytes), "%s vs legacy: %s", string(marshaledBytes),
		string(legacyBz))
}

func checkRoundTrip(t *rapid.T, message proto.Message, marshaledBytes []byte) {
	cdc := amino.NewCodec()
	cdc.RegisterConcrete(&testpb.ABitOfEverything{}, "ABitOfEverything", nil)
	cdc.RegisterConcrete(&testpb.NestedMessage{}, "NestedMessage", nil)

	message2 := message.ProtoReflect().New().Interface()
	goProtoJSON, err := cdc.MarshalJSON(message)
	assert.NilError(t, err)
	err = cdc.UnmarshalJSON(marshaledBytes, message2)
	assert.NilError(t, err, "%s vs %s", string(marshaledBytes), string(goProtoJSON))
}

func TestDynamicPb(t *testing.T) {
	msg := &aminojsonpb.AminoSignFee{}
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{})

	desc, err := protoregistry.GlobalFiles.FindDescriptorByName(proto.MessageName(msg))
	require.NoError(t, err)
	dynamicMsgType := dynamicpb.NewMessageType(desc.(protoreflect.MessageDescriptor))
	dynamicMsg := dynamicMsgType.New().Interface()

	bz, err := encoder.Marshal(msg)
	require.NoError(t, err)
	dynamicBz, err := encoder.Marshal(dynamicMsg)
	require.NoError(t, err)
	require.Equal(t, string(bz), string(dynamicBz))
}

func TestMarshalDuration(t *testing.T) {
	msg := &testpb.Duration{
		Duration: &durationpb.Duration{Seconds: 1},
	}
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{})

	bz, err := encoder.Marshal(msg)
	require.NoError(t, err)
	require.Equal(t, `{"duration":"1000000000"}`, string(bz))

	// define a custom marshaler for duration
	encoder.DefineTypeEncoding("google.protobuf.Duration", func(_ *aminojson.Encoder, msg protoreflect.Message, w io.Writer) error {
		var secondsName protoreflect.Name = "seconds"

		fields := msg.Descriptor().Fields()
		secondsField := fields.ByName(secondsName)
		if secondsField == nil {
			return errors.New("expected seconds field")
		}
		seconds := msg.Get(secondsField).Int()

		_, err = fmt.Fprint(w, "\"", (time.Duration(seconds) * time.Second).String(), "\"")
		return err
	})
	bz, err = encoder.Marshal(msg)
	require.NoError(t, err)
	require.Equal(t, `{"duration":"1s"}`, string(bz))
}

func TestWithAJson(t *testing.T) {
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{})

	// list
	msg := &testpb.WithAJson{
		Field1: []byte(`[{"name":"child1"}]`),
	}
	bz, err := encoder.Marshal(msg)
	require.NoError(t, err)
	require.Equal(t, `{"field1":[{"name":"child1"}]}`, string(bz))

	// string
	msg = &testpb.WithAJson{
		Field1: []byte(`"hello again"`),
	}
	bz, err = encoder.Marshal(msg)
	require.NoError(t, err)
	require.Equal(t, `{"field1":"hello again"}`, string(bz))

	// object
	msg = &testpb.WithAJson{
		Field1: []byte(`{"deeper":{"nesting":1}}`),
	}
	bz, err = encoder.Marshal(msg)
	require.NoError(t, err)
	require.Equal(t, `{"field1":{"deeper":{"nesting":1}}}`, string(bz))
}

func TestIndent(t *testing.T) {
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{Indent: "	"})

	msg := &testpb.ABitOfEverything{
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

	bz, err := encoder.Marshal(msg)
	require.NoError(t, err)
	fmt.Println(string(bz))
	require.Equal(t, `{
	"type": "ABitOfEverything",
	"value": {
		"bool": true,
		"bytes": "AAECAw==",
		"enum": 1,
		"f32": 1001,
		"f64": "9572348124213523654",
		"i32": -15,
		"i64": "14578294827584932",
		"message": {
			"foo": "test"
		},
		"repeated": [
			3,
			-7,
			2,
			6,
			4
		],
		"sf32": -1000,
		"sf64": "-659101379604211154",
		"si32": -376,
		"si64": "-59268425823934",
		"str": "abcxyz\"foo\"def",
		"u32": 1200,
		"u64": "4759492485"
	}
}`, string(bz))
}

func TestEnumAsString(t *testing.T) {
	encoder := aminojson.NewEncoder(aminojson.EncoderOptions{Indent: "	", EnumAsString: true})

	msg := &testpb.ABitOfEverything{
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

	bz, err := encoder.Marshal(msg)
	require.NoError(t, err)
	fmt.Println(string(bz))
	require.Equal(t, `{
	"type": "ABitOfEverything",
	"value": {
		"bool": true,
		"bytes": "AAECAw==",
		"enum": "ONE",
		"f32": 1001,
		"f64": "9572348124213523654",
		"i32": -15,
		"i64": "14578294827584932",
		"message": {
			"foo": "test"
		},
		"repeated": [
			3,
			-7,
			2,
			6,
			4
		],
		"sf32": -1000,
		"sf64": "-659101379604211154",
		"si32": -376,
		"si64": "-59268425823934",
		"str": "abcxyz\"foo\"def",
		"u32": 1200,
		"u64": "4759492485"
	}
}`, string(bz))
}
