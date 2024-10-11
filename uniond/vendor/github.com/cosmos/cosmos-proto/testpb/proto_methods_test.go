package testpb

import (
	"fmt"
	"math"
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/runtime/protoiface"
	"google.golang.org/protobuf/runtime/protoimpl"
	"google.golang.org/protobuf/types/dynamicpb"
	"pgregory.net/rapid"
)

func TestProtoMethods(t *testing.T) {
	t.Run("testSize", rapid.MakeCheck(testSize))
	t.Run("testMarshal", rapid.MakeCheck(testMarshal))
	t.Run("testUnmarshal", rapid.MakeCheck(testUnmarshal))
}

func testSize(t *rapid.T) {
	slowMsg := getRapidMsg(t)
	fastMsg := slowMsg.ProtoReflect()
	dyn := dynamicpb.NewMessage(md_A)
	populateDynamicMsg(dyn, fastMsg)
	methods := fastMsg.ProtoMethods()

	result := methods.Size(protoiface.SizeInput{Message: fastMsg})
	expected := proto.Size(dyn)

	require.Equal(t, expected, result.Size)
}

func testMarshal(t *rapid.T) {
	msg := getRapidMsg(t)
	fastMsg := msg.ProtoReflect()
	dyn := dynamicpb.NewMessage(md_A)
	populateDynamicMsg(dyn, fastMsg)

	result, err := proto.MarshalOptions{Deterministic: true}.Marshal(fastMsg.Interface())
	require.NoError(t, err)

	canonical, err := proto.MarshalOptions{Deterministic: true}.Marshal(dyn)
	require.NoError(t, err)

	require.Equal(t, canonical, result)
}

func testUnmarshal(t *rapid.T) {
	a := getRapidMsg(t)
	fastMsg := a.ProtoReflect()
	dyn := dynamicpb.NewMessage(md_A)
	populateDynamicMsg(dyn, fastMsg)
	bz, err := proto.MarshalOptions{Deterministic: true}.Marshal(dyn)
	require.NoError(t, err)

	aa := A{}
	fastaa := aa.ProtoReflect()
	err = proto.UnmarshalOptions{
		NoUnkeyedLiterals: struct{}{},
		Merge:             false,
		AllowPartial:      false,
		DiscardUnknown:    false,
		Resolver:          nil,
	}.Unmarshal(bz, fastaa.Interface())
	require.NoError(t, err)

	require.True(t, proto.Equal(fastMsg.Interface(), fastaa.Interface()), fmt.Sprintf("left: %+v\nright:%+v", fastMsg, fastaa))
}

func TestNegativeZero(t *testing.T) {

	testCases := []struct {
		name  string
		value float64
	}{
		{
			name:  "negative 0",
			value: math.Copysign(0, -1),
		},
		{
			name:  "negative float",
			value: -0.420,
		},
		{
			name:  "regular zero",
			value: 0,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			a := A{}
			a.DOUBLE = tc.value

			dyn := dynamicpb.NewMessage(md_A)
			dyn.Set(fd_A_DOUBLE, protoreflect.ValueOfFloat64(tc.value))

			bz, err := proto.MarshalOptions{Deterministic: true}.Marshal(dyn)
			require.NoError(t, err)

			bz2, err := proto.Marshal(a.ProtoReflect().Interface())
			require.NoError(t, err)

			require.Equal(t, bz, bz2)
		})
	}
}

func populateDynamicMsg(dyn *dynamicpb.Message, msg protoreflect.Message) {
	msg.Range(func(descriptor protoreflect.FieldDescriptor, value protoreflect.Value) bool {
		if descriptor.IsMap() {
			dynMap := dyn.Mutable(descriptor).Map()
			underlying := value.Map()
			underlying.Range(func(key protoreflect.MapKey, value protoreflect.Value) bool {
				dynMap.Set(key, value)
				return true
			})
			dyn.Set(fd_A_MAP, protoreflect.ValueOfMap(dynMap))
		} else if descriptor.IsList() {
			dynList := dyn.Mutable(descriptor).List()
			underlying := value.List()
			for i := 0; i < underlying.Len(); i++ {
				dynList.Append(underlying.Get(i))
			}
			dyn.Set(descriptor, protoreflect.ValueOfList(dynList))
		} else {
			dyn.Set(descriptor, value)
		}
		return true
	})
}

func getRapidMsg(t *rapid.T) A {
	return A{
		Enum:        Enumeration(rapid.IntRange(0, 1).Draw(t, "enum")),
		SomeBoolean: rapid.Bool().Draw(t, "SomeBool"),
		INT32:       rapid.Int32().Draw(t, "INT32"),
		SINT32:      rapid.Int32().Draw(t, "SINT32"),
		UINT32:      rapid.Uint32().Draw(t, "UINT32"),
		INT64:       rapid.Int64().Draw(t, "INT64"),
		SING64:      rapid.Int64().Draw(t, "SING64"),
		UINT64:      rapid.Uint64().Draw(t, "UINT64"),
		SFIXED32:    rapid.Int32().Draw(t, "SFIXED32"),
		FIXED32:     rapid.Uint32().Draw(t, "FIXED32"),
		FLOAT:       rapid.Float32().Draw(t, "FLOAT"),
		SFIXED64:    rapid.Int64().Draw(t, "SFIXED64"),
		FIXED64:     rapid.Uint64().Draw(t, "FIXED64"),
		DOUBLE:      rapid.Float64().Draw(t, "DOUBLE"),
		STRING:      rapid.String().Draw(t, "STRING"),
		BYTES:       rapid.SliceOf(rapid.Byte()).Draw(t, "byte slice"),
		MESSAGE:     genMessageB.Draw(t, "MESSAGE"),
		LIST:        rapid.SliceOf(genMessageB).Draw(t, "LIST"),
		ONEOF:       genOneOf.Draw(t, "one of"),
		MAP:         rapid.MapOf(rapid.String(), genMessageB).Draw(t, "map[string]*B"),
		LIST_ENUM:   rapid.SliceOf(genEnumSlice).Draw(t, "slice enum"),
	}
}

var genEnumSlice = rapid.Custom(func(t *rapid.T) Enumeration {
	n := rapid.Int32Range(0, 1).Draw(t, "int32")
	return Enumeration(n)
})

var genOneOf = rapid.Custom(func(t *rapid.T) isA_ONEOF {
	oneof := rapid.OneOf(genOneOfB, genOneOfString).Draw(t, "oneof")
	return oneof
})

var genOneOfB = rapid.Custom(func(t *rapid.T) isA_ONEOF {
	return &A_ONEOF_B{ONEOF_B: genMessageB.Draw(t, "message B in one of")}
})

var genOneOfString = rapid.Custom(func(t *rapid.T) isA_ONEOF {
	return &A_ONEOF_STRING{ONEOF_STRING: rapid.StringN(1, -1, -1).Draw(t, "string in one of")}
})

var genMessageB = rapid.Custom(func(t *rapid.T) *B {
	msg := B{
		state:         protoimpl.MessageState{},
		sizeCache:     0,
		unknownFields: nil,
		X:             rapid.String().Draw(t, "X"),
	}
	return &msg
})
