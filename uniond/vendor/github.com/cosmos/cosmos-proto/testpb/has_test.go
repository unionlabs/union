package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

func TestHas(t *testing.T) {
	t.Run("has all set", func(t *testing.T) {
		m := &A{
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
			MAP:       map[string]*B{"item": {X: "inside_map_item"}},
			LIST:      []*B{{X: "part of list"}},
			ONEOF:     &A_ONEOF_B{ONEOF_B: &B{X: "1"}},
			LIST_ENUM: []Enumeration{Enumeration_One, Enumeration_One},
		}

		dyn := dynamicpb.NewMessage(m.ProtoReflect().Descriptor())

		b, err := proto.MarshalOptions{}.Marshal(m)
		require.NoError(t, err)
		require.NoError(t, proto.Unmarshal(b, dyn))

		for i := 0; i < dyn.Descriptor().Fields().Len(); i++ {
			fd := dyn.Descriptor().Fields().Get(i)

			require.Equal(t, dyn.Has(fd), m.ProtoReflect().Has(fd), fd.FullName())
		}
	})

	t.Run("has all unset", func(t *testing.T) {
		m := &A{}
		dyn := dynamicpb.NewMessage(m.ProtoReflect().Descriptor())

		for i := 0; i < dyn.Descriptor().Fields().Len(); i++ {
			fd := dyn.Descriptor().Fields().Get(i)

			require.Equal(t, dyn.Has(fd), m.ProtoReflect().Has(fd), fd.FullName())
		}
	})

	t.Run("oneof field is set but the value is the default one", func(t *testing.T) {
		dyn := dynamicpb.NewMessage(md_A)
		dyn.Set(md_A.Fields().ByName("ONEOF_STRING"), protoreflect.ValueOfString(""))
		require.True(t, dyn.Has(md_A.Fields().ByName("ONEOF_STRING")))

		m := &A{ONEOF: &A_ONEOF_STRING{ONEOF_STRING: ""}}
		require.True(t, m.ProtoReflect().Has(md_A.Fields().ByName("ONEOF_STRING")))
	})

	t.Run("nil bytes", func(t *testing.T) {
		fd := (&A{}).ProtoReflect().Descriptor().Fields().ByName("BYTES")
		m := &A{BYTES: nil}
		require.False(t, m.slowProtoReflect().Has(fd))
		require.False(t, m.ProtoReflect().Has(fd))
	})

	t.Run("0 len bytes", func(t *testing.T) {
		fd := (&A{}).ProtoReflect().Descriptor().Fields().ByName("BYTES")
		m := &A{BYTES: []byte{}}

		require.False(t, m.slowProtoReflect().Has(fd))
		require.False(t, m.ProtoReflect().Has(fd))
	})

	t.Run("invalid fd", func(t *testing.T) {
		invalidFd := (&anypb.Any{}).ProtoReflect().Descriptor().Fields().ByName("value")

		require.Panics(t, func() {
			(&A{}).ProtoReflect().Has(invalidFd)
		})
	})
}
