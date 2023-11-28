package flag

import (
	"context"
	"fmt"
	"strings"

	"google.golang.org/protobuf/reflect/protoreflect"

	basev1beta1 "cosmossdk.io/api/cosmos/base/v1beta1"
	"cosmossdk.io/core/coins"
)

type coinType struct{}

type coinValue struct {
	value *basev1beta1.Coin
}

func (c coinType) NewValue(context.Context, *Builder) Value {
	return &coinValue{}
}

func (c coinType) DefaultValue() string {
	stringCoin, _ := coins.FormatCoins([]*basev1beta1.Coin{}, nil)
	return stringCoin
}

func (c *coinValue) Get(protoreflect.Value) (protoreflect.Value, error) {
	if c.value == nil {
		return protoreflect.Value{}, nil
	}
	return protoreflect.ValueOfMessage(c.value.ProtoReflect()), nil
}

func (c *coinValue) String() string {
	return c.value.String()
}

func (c *coinValue) Set(stringValue string) error {
	if strings.Contains(stringValue, ",") {
		return fmt.Errorf("coin flag must be a single coin, specific multiple coins with multiple flags or spaces")
	}

	coin, err := coins.ParseCoin(stringValue)
	if err != nil {
		return err
	}
	c.value = coin
	return nil
}

func (c *coinValue) Type() string {
	return "cosmos.base.v1beta1.Coin"
}
