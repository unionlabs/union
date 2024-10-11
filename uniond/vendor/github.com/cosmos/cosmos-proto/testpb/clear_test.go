package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/dynamicpb"
	"google.golang.org/protobuf/types/known/anypb"
)

func TestClear(t *testing.T) {
	t.Run("clear all", func(t *testing.T) {
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

		for i := 0; i < md_A.Fields().Len(); i++ {
			fd := md_A.Fields().Get(i)

			m.ProtoReflect().Clear(fd)
			dyn.Clear(fd)

			require.Equal(t, dyn.Has(fd), m.ProtoReflect().Has(fd), fd.FullName())
		}
	})

	t.Run("unknown field descriptor", func(t *testing.T) {
		m := &A{}

		require.Panics(t, func() {
			m.ProtoReflect().Clear((&anypb.Any{}).ProtoReflect().Descriptor().Fields().Get(0))
		})
	})
}
