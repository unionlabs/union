package testpb

import (
	"testing"

	"github.com/stretchr/testify/require"
	"google.golang.org/protobuf/reflect/protoreflect"
)

func TestGet_NoMap_NoList_NoOneof(t *testing.T) {
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
		ONEOF:     nil,
		LIST_ENUM: nil,
	}

	cases := map[string]struct {
		fieldName protoreflect.Name
		expected  interface{}
	}{
		"enum": {
			fieldName: "enum",
			expected:  msg.Enum,
		},

		"bool": {
			fieldName: "some_boolean",
			expected:  msg.SomeBoolean,
		},

		"int32": {
			fieldName: "INT32",
			expected:  msg.INT32,
		},

		"sint32": {
			fieldName: "SINT32",
			expected:  msg.SINT32,
		},

		"uint32": {
			fieldName: "UINT32",
			expected:  msg.UINT32,
		},

		"int64": {
			fieldName: "INT64",
			expected:  msg.INT64,
		},

		"sint64": {
			fieldName: "SING64",
			expected:  msg.SING64,
		},

		"uint64": {
			fieldName: "UINT64",
			expected:  msg.UINT64,
		},

		"sfixed32": {
			fieldName: "SFIXED32",
			expected:  msg.SFIXED32,
		},

		"float": {
			fieldName: "FLOAT",
			expected:  msg.FLOAT,
		},

		"double": {
			fieldName: "DOUBLE",
			expected:  msg.DOUBLE,
		},

		"bytes": {
			fieldName: "BYTES",
			expected:  msg.BYTES,
		},

		"string": {
			fieldName: "STRING",
			expected:  msg.STRING,
		},

		"sfixed64": {
			fieldName: "SFIXED64",
			expected:  msg.SFIXED64,
		},

		"fixed32": {
			fieldName: "FIXED32",
			expected:  msg.FIXED32,
		},

		"message": {
			fieldName: "MESSAGE",
			expected:  msg.MESSAGE,
		},
	}

	for name, tc := range cases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			fd := msg.ProtoReflect().Descriptor().Fields().ByName(tc.fieldName)

			v := msg.ProtoReflect().Get(fd)

			// validity
			require.True(t, v.IsValid(), "field must be valid")

			// value casting
			require.NotPanics(t, func() {
				switch fd.Kind() {
				case protoreflect.BoolKind:
					v.Bool()
				case protoreflect.EnumKind:
					v.Enum()
				case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind, protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
					v.Int()
				case protoreflect.Uint32Kind, protoreflect.Fixed32Kind, protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
					v.Uint()
				case protoreflect.FloatKind, protoreflect.DoubleKind:
					v.Float()
				case protoreflect.StringKind:
					_ = v.String()
				case protoreflect.BytesKind:
					v.Bytes()
				case protoreflect.MessageKind, protoreflect.GroupKind:
					v.Message()
				}
			})

			// assignment and equality
			var concreteValue interface{}

			switch fd.Kind() {
			case protoreflect.BoolKind:
				concreteValue = v.Bool()
			case protoreflect.EnumKind:
				concreteValue = (Enumeration)(v.Enum())
			case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
				concreteValue = (int32)(v.Int())
			case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
				concreteValue = (uint32)(v.Uint())
			case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
				concreteValue = (int64)(v.Int())
			case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
				concreteValue = (uint64)(v.Uint())
			case protoreflect.FloatKind:
				concreteValue = (float32)(v.Float())
			case protoreflect.DoubleKind:
				concreteValue = (float64)(v.Float())
			case protoreflect.StringKind:
				concreteValue = v.String()
			case protoreflect.BytesKind:
				concreteValue = v.Bytes()
			case protoreflect.MessageKind, protoreflect.GroupKind:
				concreteValue = v.Message().Interface().(*B)
			}

			require.Equal(t, tc.expected, concreteValue)
		})
	}
}

func TestGetPanics(t *testing.T) {
	msg := &A{}

	t.Run("unknown field", func(t *testing.T) {
		fd := (&B{}).ProtoReflect().Descriptor().Fields().ByName("X")
		require.Panics(t, func() {
			msg.ProtoReflect().Get(fd)
		})
	})
}

func TestGetList(t *testing.T) {
	fd := (&A{}).ProtoReflect().Descriptor().Fields().ByName("LIST")

	t.Run("mutability", func(t *testing.T) {
		msg := &A{LIST: []*B{
			{
				X: "1",
			},
		}}

		v := msg.ProtoReflect().Get(fd).List()

		require.True(t, v.IsValid())

		// we append a variable
		toAppend := &B{
			X: "2",
		}
		v.Append(protoreflect.ValueOfMessage(toAppend.ProtoReflect()))

		// assert that we find it inside A
		require.Len(t, msg.LIST, 2)

		require.Equal(t, toAppend, msg.LIST[1])
	})

	t.Run("invalidity", func(t *testing.T) {
		t.Run("nil", func(t *testing.T) {
			msg := &A{}

			v := msg.ProtoReflect().Get(fd).List()

			require.False(t, v.IsValid())

		})

		t.Run("empty", func(t *testing.T) {
			msg := &A{LIST: []*B{}}

			v := msg.ProtoReflect().Get(fd).List()

			require.False(t, v.IsValid())
		})

		t.Run("invalidity panics", func(t *testing.T) {
			msg := &A{}

			v := msg.ProtoReflect().Get(fd).List()

			require.Panics(t, func() {
				v.Set(0, protoreflect.ValueOfMessage((&B{}).ProtoReflect()))
			})

			require.Panics(t, func() {
				v.Append(protoreflect.ValueOfMessage((&B{}).ProtoReflect()))
			})

			require.Panics(t, func() {
				v.AppendMutable()
			})

			require.Panics(t, func() {
				v.Truncate(1)
			})

			require.Panics(t, func() {
				v.Get(0)
			})
		})

		t.Run("invalidty no panics", func(t *testing.T) {
			msg := &A{}

			v := msg.ProtoReflect().Get(fd).List()

			require.NotPanics(t, func() {
				v.NewElement()
			})

			require.NotPanics(t, func() {
				v.Len()
			})
		})
	})
}

func TestGetMap(t *testing.T) {
	fd := (&A{}).ProtoReflect().Descriptor().Fields().ByName("MAP")

	t.Run("mutability", func(t *testing.T) {
		msg := &A{MAP: map[string]*B{
			"1": &B{X: "a"},
		}}

		mv := msg.ProtoReflect().Get(fd).Map()

		key := "2"
		value := &B{X: "b"}

		mv.Set(protoreflect.MapKey(protoreflect.ValueOfString(key)), protoreflect.ValueOfMessage(value.ProtoReflect()))

		require.Len(t, msg.MAP, 2)

		require.Equal(t, value, msg.MAP[key])
	})

	t.Run("invalidity", func(t *testing.T) {
		t.Run("nil", func(t *testing.T) {
			msg := new(A)
			require.False(t, msg.ProtoReflect().Get(fd).Map().IsValid())
		})

		t.Run("empty", func(t *testing.T) {
			msg := &A{MAP: map[string]*B{}}
			require.False(t, msg.ProtoReflect().Get(fd).Map().IsValid())
		})

		t.Run("invalidity panics", func(t *testing.T) {
			msg := &A{MAP: map[string]*B{}}

			mv := msg.ProtoReflect().Get(fd).Map()

			require.Panics(t, func() {
				mv.Mutable(protoreflect.MapKey(protoreflect.ValueOfString("something")))
			})

			require.Panics(t, func() {
				mv.Set(protoreflect.MapKey(protoreflect.ValueOfString("something")), protoreflect.ValueOfMessage((&B{}).ProtoReflect()))
			})
		})

		t.Run("invalidty no panics", func(t *testing.T) {
			msg := &A{MAP: map[string]*B{}}

			mv := msg.ProtoReflect().Get(fd).Map()

			require.NotPanics(t, func() {
				v := mv.Get(protoreflect.MapKey(protoreflect.ValueOfString("idk")))
				require.False(t, v.IsValid())
			})

			require.NotPanics(t, func() {
				mv.Len()
			})

			require.NotPanics(t, func() {
				mv.NewValue()
			})

			require.NotPanics(t, func() {
				mv.Clear(protoreflect.MapKey(protoreflect.ValueOfString("xd")))
			})

			require.NotPanics(t, func() {
				require.False(t, mv.Has(protoreflect.MapKey(protoreflect.ValueOfString("xd"))))
			})

			require.NotPanics(t, func() {
				ex := false
				mv.Range(func(key protoreflect.MapKey, value protoreflect.Value) bool {
					ex = true
					return true
				})

				require.False(t, ex)
			})

		})
	})
}

func TestGetMessage(t *testing.T) {
	fd := (&A{}).ProtoReflect().Descriptor().Fields().ByName("MESSAGE")
	t.Run("valid", func(t *testing.T) {
		msg := &A{MESSAGE: &B{}}
		require.True(t, msg.ProtoReflect().Get(fd).Message().IsValid())
	})

	t.Run("invalid", func(t *testing.T) {
		msg := new(A)
		require.False(t, msg.ProtoReflect().Get(fd).Message().IsValid())
	})
}

func TestGetOneof(t *testing.T) {
	fdMsg := (&A{}).ProtoReflect().Descriptor().Fields().ByName("ONEOF_B")
	fdString := (&A{}).ProtoReflect().Descriptor().Fields().ByName("ONEOF_STRING")
	t.Run("nil message", func(t *testing.T) {
		msg := &A{}
		value := msg.ProtoReflect().Get(fdMsg)

		require.True(t, value.IsValid())
		require.False(t, value.Message().IsValid())
	})

	t.Run("empty string", func(t *testing.T) {
		msg := &A{}
		value := msg.ProtoReflect().Get(fdString)

		require.True(t, value.IsValid())
		require.Equal(t, "", value.String())
	})

	t.Run("existing message", func(t *testing.T) {
		msg := &A{ONEOF: &A_ONEOF_B{ONEOF_B: &B{}}}
		mv := msg.ProtoReflect().Get(fdMsg).Message()
		sv := msg.ProtoReflect().Get(fdString).String()

		require.True(t, mv.IsValid())
		require.Empty(t, sv)

		require.Equal(t, msg.GetONEOF_B(), mv.Interface())
	})
}

func TestGetoneof(t *testing.T) {
	x := &A{}
	oneOf1 := x.ProtoReflect().Descriptor().Fields().ByName("ONEOF_STRING")
	oneOf2 := x.ProtoReflect().Descriptor().Fields().ByName("ONEOF_B")
	t.Logf("%s", oneOf1)

	v := x.ProtoReflect().Get(oneOf1)
	t.Logf("%s", v.Interface()) // empty string

	v = x.ProtoReflect().Get(oneOf2)
	t.Logf("%v", v.Message().IsValid()) // nil object, is valid
}
