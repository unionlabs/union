package flag

import (
	"context"
	"time"

	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/types/known/durationpb"
)

type durationType struct{}

func (d durationType) NewValue(context.Context, *Builder) Value {
	return &durationValue{}
}

func (d durationType) DefaultValue() string {
	return ""
}

type durationValue struct {
	value *durationpb.Duration
}

func (d durationValue) Get(protoreflect.Value) (protoreflect.Value, error) {
	if d.value == nil {
		return protoreflect.Value{}, nil
	}
	return protoreflect.ValueOfMessage(d.value.ProtoReflect()), nil
}

func (d durationValue) String() string {
	if d.value == nil {
		return ""
	}
	return d.value.AsDuration().String()
}

func (d *durationValue) Set(s string) error {
	dur, err := time.ParseDuration(s)
	if err != nil {
		return err
	}

	d.value = durationpb.New(dur)
	return nil
}

func (d durationValue) Type() string {
	return "duration"
}
