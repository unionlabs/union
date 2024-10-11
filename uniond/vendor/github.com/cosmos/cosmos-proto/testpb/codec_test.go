package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
)

func TestMarshal(t *testing.T) {
	msg := &A{
		Enum:        Enumeration_Two,
		SomeBoolean: true,
		INT32:       2,
		SINT32:      3,
		UINT32:      4,
		INT64:       5,
		SING64:      6,
		UINT64:      7,
		SFIXED32:    8,
		FIXED32:     9,
		FLOAT:       10.1,
		SFIXED64:    11,
		FIXED64:     12,
		DOUBLE:      13,
		STRING:      "fourteen",
		BYTES:       []byte("fifteen"),
		MESSAGE:     &B{X: "something"},
		MAP:         map[string]*B{"a": &B{X: "aa"}},
		LIST:        []*B{{X: "list"}},
		ONEOF:       &A_ONEOF_B{ONEOF_B: &B{X: "ONEOF"}},
		LIST_ENUM:   []Enumeration{Enumeration_One},
	}

	dynA := dynamicpb.NewMessage(md_A)
	// dynB := dynamicpb.NewMessage(md_B)

	dynA.Set(fd_A_enum, protoreflect.ValueOfEnum(protoreflect.EnumNumber(msg.Enum)))
	dynA.Set(fd_A_some_boolean, protoreflect.ValueOfBool(msg.SomeBoolean))
	dynA.Set(fd_A_INT32, protoreflect.ValueOfInt32(msg.INT32))
	dynA.Set(fd_A_SINT32, protoreflect.ValueOfInt32(msg.SINT32))
	dynA.Set(fd_A_UINT32, protoreflect.ValueOfUint32(msg.UINT32))
	dynA.Set(fd_A_INT64, protoreflect.ValueOfInt64(msg.INT64))
	dynA.Set(fd_A_SING64, protoreflect.ValueOfInt64(msg.SING64))
	dynA.Set(fd_A_UINT64, protoreflect.ValueOfUint64(msg.UINT64))
	dynA.Set(fd_A_SFIXED32, protoreflect.ValueOfInt32(msg.SFIXED32))
	dynA.Set(fd_A_FIXED32, protoreflect.ValueOfUint32(msg.FIXED32))
	dynA.Set(fd_A_FLOAT, protoreflect.ValueOfFloat32(msg.FLOAT))
	dynA.Set(fd_A_SFIXED64, protoreflect.ValueOfInt64(msg.SFIXED64))
	dynA.Set(fd_A_FIXED64, protoreflect.ValueOfUint64(msg.FIXED64))
	dynA.Set(fd_A_DOUBLE, protoreflect.ValueOfFloat64(msg.DOUBLE))
	dynA.Set(fd_A_STRING, protoreflect.ValueOfString(msg.STRING))
	dynA.Set(fd_A_BYTES, protoreflect.ValueOfBytes(msg.BYTES))
	dynA.Set(fd_A_MESSAGE, protoreflect.ValueOfMessage(msg.MESSAGE.ProtoReflect()))
	dynMap := dynA.Mutable(fd_A_MAP).Map()
	dynMap.Set(protoreflect.MapKey(protoreflect.ValueOfString("a")), protoreflect.ValueOfMessage(msg.MAP["a"].ProtoReflect()))
	dynA.Mutable(fd_A_LIST).List().AppendMutable().Message().Set(fd_B_x, protoreflect.ValueOfString(msg.LIST[0].X))
	dynA.Set(fd_A_ONEOF_B, protoreflect.ValueOfMessage(msg.ONEOF.(*A_ONEOF_B).ONEOF_B.ProtoReflect()))
	dynA.Mutable(fd_A_LIST_ENUM).List().Append(protoreflect.ValueOfEnum((protoreflect.EnumNumber)(Enumeration_One)))

	got, err := proto.MarshalOptions{Deterministic: true}.Marshal(msg)
	require.NoError(t, err)

	expected, err := proto.MarshalOptions{Deterministic: true}.Marshal(dynA)
	require.NoError(t, err)

	require.Equal(t, expected, got)
}
