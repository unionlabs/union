package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/dynamicpb"
)

func TestRange(t *testing.T) {
	t.Run("empty", func(t *testing.T) {
		dyn := dynamicpb.NewMessage(md_A)
		msg := (&A{}).ProtoReflect()

		dynV := map[protoreflect.FieldDescriptor]protoreflect.Value{}
		msgV := map[protoreflect.FieldDescriptor]protoreflect.Value{}

		dyn.Range(func(descriptor protoreflect.FieldDescriptor, value protoreflect.Value) bool {
			dynV[descriptor] = value
			return true
		})

		msg.Range(func(descriptor protoreflect.FieldDescriptor, value protoreflect.Value) bool {
			msgV[descriptor] = protoreflect.Value{}
			return true
		})

		require.Equal(t, len(dynV), len(msgV))
		require.Equal(t, len(msgV), 0)
	})

	t.Run("all fields filled", func(t *testing.T) {
		// create a message which has all fields set
		// and copy it to dynamicpb message
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
			MAP:       map[string]*B{"item": {X: "inside_map_item"}},
			LIST:      []*B{{X: "part of list"}},
			ONEOF:     &A_ONEOF_B{ONEOF_B: &B{X: "1"}},
			LIST_ENUM: []Enumeration{Enumeration_One, Enumeration_One},
		}

		dyn := dynamicpb.NewMessage(msg.ProtoReflect().Descriptor())

		b, err := proto.MarshalOptions{}.Marshal(msg)
		require.NoError(t, err)
		require.NoError(t, proto.Unmarshal(b, dyn))

		dynV := map[protoreflect.FieldDescriptor]protoreflect.Value{}
		msgV := map[protoreflect.FieldDescriptor]protoreflect.Value{}

		dyn.Range(func(descriptor protoreflect.FieldDescriptor, value protoreflect.Value) bool {
			dynV[descriptor] = value
			return true
		})

		msg.ProtoReflect().Range(func(descriptor protoreflect.FieldDescriptor, value protoreflect.Value) bool {
			msgV[descriptor] = value
			return true
		})

		require.Equal(t, len(dynV), len(msgV))

		// assert field equality
		for field, dynValue := range dynV {
			msgValue, exists := msgV[field]
			require.True(t, exists, "field ", field.FullName(), "not found")

			valueEquality(t, field, dynValue, msgValue)
		}
	})
}

func valueEquality(t *testing.T, field protoreflect.FieldDescriptor, v1, v2 protoreflect.Value) {
	if !v1.IsValid() {
		require.False(t, v2.IsValid())
		return
	}

	switch {
	case field.IsList():
		list1 := v1.List()
		list2 := v2.List()

		if !list1.IsValid() {
			require.False(t, list2.IsValid())
			return
		}

		require.Equal(t, list1.Len(), list2.Len())

		for i := 0; i < list1.Len(); i++ {
			elem1 := list1.Get(i)
			elem2 := list2.Get(i)

			// note: we cannot call again valueEquality otherwise we end up in a loop
			switch {
			case field.Kind() == protoreflect.MessageKind:
				require.True(t, proto.Equal(elem1.Message().Interface(), elem2.Message().Interface()))
			default:
				require.Equal(t, elem1.Interface(), elem2.Interface())
			}
		}
	case field.IsMap():
		map1 := v1.Map()
		map2 := v2.Map()

		if !map1.IsValid() {
			require.False(t, map2.IsValid())
			return
		}

		require.Equal(t, map1.Len(), map2.Len())

		map1.Range(func(key protoreflect.MapKey, map1Value protoreflect.Value) bool {
			// assert map2 has the key from map1
			map2Value := map2.Get(key)
			require.True(t, map2Value.IsValid(), "key not found", key)

			// assert the values are equal
			valueEquality(t, field.MapValue(), map1Value, map2Value)
			return true
		})
	case field.Kind() == protoreflect.MessageKind:
		if !v1.Message().IsValid() {
			require.False(t, v2.Message().IsValid())
			return
		}
		require.True(t, proto.Equal(v1.Message().Interface(), v2.Message().Interface()))
	default:
		require.Equal(t, v1.Interface(), v2.Interface())
	}
}
