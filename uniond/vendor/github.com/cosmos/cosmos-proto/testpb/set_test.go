package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"
)

func TestSet(t *testing.T) {
	t.Run("extension panics", func(t *testing.T) {
		// TODO(fdymylja): mock extensions fd
	})

	t.Run("set scalar types and message types", func(t *testing.T) {
		msg := &A{
			Enum:        Enumeration_Two,
			SomeBoolean: true,
			INT32:       1,
			SINT32:      2,
			UINT32:      3,
			INT64:       4,
			SING64:      5,
			UINT64:      6,
			SFIXED32:    7,
			FIXED32:     8,
			FLOAT:       9,
			SFIXED64:    10,
			FIXED64:     11,
			DOUBLE:      12,
			STRING:      "a string",
			BYTES:       []byte("test bytes"),
			MESSAGE: &B{
				X: "something else",
			},

			ONEOF: &A_ONEOF_STRING{ONEOF_STRING: "test"},
		}

		m := &A{}

		msg.slowProtoReflect().Range(func(fd protoreflect.FieldDescriptor, v protoreflect.Value) bool {
			m.ProtoReflect().Set(fd, v) // set the field

			gotValue := m.ProtoReflect().Get(fd) // get the field which was set

			require.Equal(t, v, gotValue) // and assert it was equal to the provided field of the default proto impl

			return true
		})
	})

	// NOTE(fdymylja): mutability of composite types is not tested
	// because it is not part of the specification.

	t.Run("set list type", func(t *testing.T) {
		m := &A{}
		fd := m.ProtoReflect().Descriptor().Fields().ByName("LIST")
		v := m.ProtoReflect().NewField(fd).List() // no mutability with NewField

		el1 := &B{X: "1"}
		el2 := &B{X: "2"}
		v.Append(protoreflect.ValueOfMessage(el1.ProtoReflect()))
		v.Append(protoreflect.ValueOfMessage(el2.ProtoReflect()))

		m.ProtoReflect().Set(fd, protoreflect.ValueOfList(v))

		require.Len(t, m.LIST, 2)
		require.Equal(t, m.LIST[0], el1)
		require.Equal(t, m.LIST[1], el2)
	})

	t.Run("set map type", func(t *testing.T) {
		m := &A{}
		fd := m.ProtoReflect().Descriptor().Fields().ByName("MAP")
		v := m.ProtoReflect().NewField(fd).Map()

		mk1, mk2 := "1", "2"
		mv1, mv2 := &B{X: "1"}, &B{X: "2"}

		v.Set((protoreflect.MapKey)(protoreflect.ValueOfString(mk1)), protoreflect.ValueOfMessage(mv1.ProtoReflect()))
		v.Set((protoreflect.MapKey)(protoreflect.ValueOfString(mk2)), protoreflect.ValueOfMessage(mv2.ProtoReflect()))

		m.ProtoReflect().Set(fd, protoreflect.ValueOfMap(v))

		require.Len(t, m.MAP, 2)
		require.Equal(t, m.MAP[mk1], mv1)
		require.Equal(t, m.MAP[mk2], mv2)
	})
}
