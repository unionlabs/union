package flag

import (
	"github.com/spf13/pflag"
	"google.golang.org/protobuf/reflect/protoreflect"
)

func bindSimpleFlag(flagSet *pflag.FlagSet, kind protoreflect.Kind, name, shorthand, usage string) HasValue {
	switch kind {
	case protoreflect.StringKind:
		val := flagSet.StringP(name, shorthand, "", usage)
		return newSimpleValue(val, protoreflect.ValueOfString)
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		val := flagSet.Uint32P(name, shorthand, 0, usage)
		return newSimpleValue(val, protoreflect.ValueOfUint32)
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		val := flagSet.Uint64P(name, shorthand, 0, usage)
		return newSimpleValue(val, protoreflect.ValueOfUint64)
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		val := flagSet.Int32P(name, shorthand, 0, usage)
		return newSimpleValue(val, protoreflect.ValueOfInt32)
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		val := flagSet.Int64P(name, shorthand, 0, usage)
		return newSimpleValue(val, protoreflect.ValueOfInt64)
	case protoreflect.BoolKind:
		val := flagSet.BoolP(name, shorthand, false, usage)
		return newSimpleValue(val, protoreflect.ValueOfBool)
	default:
		return nil
	}
}

type simpleValue[T any] struct {
	val                 *T
	toProtoreflectValue func(T) protoreflect.Value
}

func newSimpleValue[T any](val *T, toProtoreflectValue func(T) protoreflect.Value) HasValue {
	return simpleValue[T]{val: val, toProtoreflectValue: toProtoreflectValue}
}

func (v simpleValue[T]) Get(protoreflect.Value) (protoreflect.Value, error) {
	return v.toProtoreflectValue(*v.val), nil
}
