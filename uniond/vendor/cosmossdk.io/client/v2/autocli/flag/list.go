package flag

import (
	"context"
	"fmt"

	"github.com/spf13/pflag"
	"google.golang.org/protobuf/reflect/protoreflect"
)

func bindSimpleListFlag(flagSet *pflag.FlagSet, kind protoreflect.Kind, name, shorthand, usage string) HasValue {
	switch kind {
	case protoreflect.StringKind:
		val := flagSet.StringSliceP(name, shorthand, nil, usage)
		return newListValue(val, protoreflect.ValueOfString)
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind,
		protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		val := flagSet.UintSliceP(name, shorthand, nil, usage)
		return newListValue(val, func(x uint) protoreflect.Value { return protoreflect.ValueOfUint64(uint64(x)) })
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		val := flagSet.Int32SliceP(name, shorthand, nil, usage)
		return newListValue(val, protoreflect.ValueOfInt32)
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		val := flagSet.Int64SliceP(name, shorthand, nil, usage)
		return newListValue(val, protoreflect.ValueOfInt64)
	case protoreflect.BoolKind:
		val := flagSet.BoolSliceP(name, shorthand, nil, usage)
		return newListValue(val, protoreflect.ValueOfBool)
	default:
		return nil
	}
}

type listValue[T any] struct {
	array               *[]T
	toProtoreflectValue func(T) protoreflect.Value
}

func newListValue[T any](array *[]T, toProtoreflectValue func(T) protoreflect.Value) listValue[T] {
	return listValue[T]{array: array, toProtoreflectValue: toProtoreflectValue}
}

func (v listValue[T]) Get(mutable protoreflect.Value) (protoreflect.Value, error) {
	list := mutable.List()
	for _, x := range *v.array {
		list.Append(v.toProtoreflectValue(x))
	}
	return mutable, nil
}

type compositeListType struct {
	simpleType Type
}

func (t compositeListType) NewValue(ctx context.Context, opts *Builder) Value {
	return &compositeListValue{
		simpleType: t.simpleType,
		values:     nil,
		ctx:        ctx,
		opts:       opts,
	}
}

func (t compositeListType) DefaultValue() string {
	return ""
}

type compositeListValue struct {
	simpleType Type
	values     []protoreflect.Value
	ctx        context.Context
	opts       *Builder
}

func (c *compositeListValue) Get(mutable protoreflect.Value) (protoreflect.Value, error) {
	list := mutable.List()
	for _, value := range c.values {
		list.Append(value)
	}
	return mutable, nil
}

func (c *compositeListValue) String() string {
	if len(c.values) == 0 {
		return ""
	}

	return fmt.Sprintf("%+v", c.values)
}

func (c *compositeListValue) Set(val string) error {
	simpleVal := c.simpleType.NewValue(c.ctx, c.opts)
	err := simpleVal.Set(val)
	if err != nil {
		return err
	}
	v, err := simpleVal.Get(protoreflect.Value{})
	if err != nil {
		return err
	}
	c.values = append(c.values, v)
	return nil
}

func (c *compositeListValue) Type() string {
	return fmt.Sprintf("%s (repeated)", c.simpleType.NewValue(c.ctx, c.opts).Type())
}
