package flag

import (
	"context"
	"errors"
	"fmt"

	"google.golang.org/protobuf/reflect/protoreflect"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type pubkeyType struct{}

func (a pubkeyType) NewValue(_ *context.Context, _ *Builder) Value {
	return &pubkeyValue{}
}

func (a pubkeyType) DefaultValue() string {
	return ""
}

type pubkeyValue struct {
	value *types.Any
}

func (a pubkeyValue) Get(protoreflect.Value) (protoreflect.Value, error) {
	return protoreflect.ValueOf(a.value), nil
}

func (a pubkeyValue) String() string {
	return a.value.String()
}

func (a *pubkeyValue) Set(s string) error {
	registry := types.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)
	cdc := codec.NewProtoCodec(registry)

	var pk cryptotypes.PubKey
	err := cdc.UnmarshalInterfaceJSON([]byte(s), &pk)
	if err != nil {
		return fmt.Errorf("input isn't a pubkey: %w", err)
	}

	any, err := types.NewAnyWithValue(pk)
	if err != nil {
		return errors.New("error converting to any type")
	}

	a.value = any

	return nil
}

func (a pubkeyValue) Type() string {
	return "pubkey"
}
