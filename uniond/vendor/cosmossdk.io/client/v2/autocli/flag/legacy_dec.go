package flag

import (
	"context"

	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/math"
)

type decType struct{}

func (a decType) NewValue(_ *context.Context, _ *Builder) Value {
	return &decValue{}
}

func (a decType) DefaultValue() string {
	return "0"
}

type decValue struct {
	value string
}

func (a decValue) Get(protoreflect.Value) (protoreflect.Value, error) {
	return protoreflect.ValueOf(a.value), nil
}

func (a decValue) String() string {
	return a.value
}

func (a *decValue) Set(s string) error {
	dec, err := math.LegacyNewDecFromStr(s)
	if err != nil {
		return err
	}

	// we need to convert from float representation to non-float representation using default precision
	// 0.5 -> 500000000000000000
	a.value = dec.BigInt().String()

	return nil
}

func (a decValue) Type() string {
	return "cosmos.Dec"
}
