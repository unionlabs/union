package types

import (
	"fmt"
	"reflect"
	"runtime/debug"

	"github.com/cosmos/gogoproto/jsonpb"
	"github.com/cosmos/gogoproto/proto"
	amino "github.com/tendermint/go-amino"
)

type anyCompat struct {
	aminoBz []byte
	jsonBz  []byte
	err     error
}

var Debug = true

func anyCompatError(errType string, x interface{}) error {
	if Debug {
		debug.PrintStack()
	}
	return fmt.Errorf(
		"%s marshaling error for %+v, this is likely because "+
			"amino is being used directly (instead of codec.LegacyAmino which is preferred) "+
			"or UnpackInterfacesMessage is not defined for some type which contains "+
			"a protobuf Any either directly or via one of its members. To see a "+
			"stacktrace of where the error is coming from, set the var Debug = true "+
			"in codec/types/compat.go",
		errType, x,
	)
}

func (any Any) MarshalAmino() ([]byte, error) {
	ac := any.compat
	if ac == nil {
		return nil, anyCompatError("amino binary marshal", any)
	}
	return ac.aminoBz, ac.err
}

func (any *Any) UnmarshalAmino(bz []byte) error {
	any.compat = &anyCompat{
		aminoBz: bz,
		err:     nil,
	}
	return nil
}

func (any *Any) MarshalJSON() ([]byte, error) {
	ac := any.compat
	if ac == nil {
		return nil, anyCompatError("JSON marshal", any)
	}
	return ac.jsonBz, ac.err
}

func (any *Any) UnmarshalJSON(bz []byte) error {
	any.compat = &anyCompat{
		jsonBz: bz,
		err:    nil,
	}
	return nil
}

// AminoUnpacker is an AnyUnpacker provided for backwards compatibility with
// amino for the binary un-marshaling phase
type AminoUnpacker struct {
	Cdc *amino.Codec
}

var _ AnyUnpacker = AminoUnpacker{}

func (a AminoUnpacker) UnpackAny(any *Any, iface interface{}) error {
	ac := any.compat
	if ac == nil {
		return anyCompatError("amino binary unmarshal", reflect.TypeOf(iface))
	}
	err := a.Cdc.UnmarshalBinaryBare(ac.aminoBz, iface)
	if err != nil {
		return err
	}
	val := reflect.ValueOf(iface).Elem().Interface()
	err = UnpackInterfaces(val, a)
	if err != nil {
		return err
	}
	if m, ok := val.(proto.Message); ok {
		if err = any.pack(m); err != nil {
			return err
		}
	} else {
		any.cachedValue = val
	}

	// this is necessary for tests that use reflect.DeepEqual and compare
	// proto vs amino marshaled values
	any.compat = nil

	return nil
}

// AminoUnpacker is an AnyUnpacker provided for backwards compatibility with
// amino for the binary marshaling phase
type AminoPacker struct {
	Cdc *amino.Codec
}

var _ AnyUnpacker = AminoPacker{}

func (a AminoPacker) UnpackAny(any *Any, _ interface{}) error {
	err := UnpackInterfaces(any.cachedValue, a)
	if err != nil {
		return err
	}
	bz, err := a.Cdc.MarshalBinaryBare(any.cachedValue)
	any.compat = &anyCompat{
		aminoBz: bz,
		err:     err,
	}
	return err
}

// AminoUnpacker is an AnyUnpacker provided for backwards compatibility with
// amino for the JSON marshaling phase
type AminoJSONUnpacker struct {
	Cdc *amino.Codec
}

var _ AnyUnpacker = AminoJSONUnpacker{}

func (a AminoJSONUnpacker) UnpackAny(any *Any, iface interface{}) error {
	ac := any.compat
	if ac == nil {
		return anyCompatError("JSON unmarshal", reflect.TypeOf(iface))
	}
	err := a.Cdc.UnmarshalJSON(ac.jsonBz, iface)
	if err != nil {
		return err
	}
	val := reflect.ValueOf(iface).Elem().Interface()
	err = UnpackInterfaces(val, a)
	if err != nil {
		return err
	}
	if m, ok := val.(proto.Message); ok {
		if err = any.pack(m); err != nil {
			return err
		}
	} else {
		any.cachedValue = val
	}

	// this is necessary for tests that use reflect.DeepEqual and compare
	// proto vs amino marshaled values
	any.compat = nil

	return nil
}

// AminoUnpacker is an AnyUnpacker provided for backwards compatibility with
// amino for the JSON un-marshaling phase
type AminoJSONPacker struct {
	Cdc *amino.Codec
}

var _ AnyUnpacker = AminoJSONPacker{}

func (a AminoJSONPacker) UnpackAny(any *Any, _ interface{}) error {
	err := UnpackInterfaces(any.cachedValue, a)
	if err != nil {
		return err
	}
	bz, err := a.Cdc.MarshalJSON(any.cachedValue)
	any.compat = &anyCompat{
		jsonBz: bz,
		err:    err,
	}
	return err
}

// ProtoJSONPacker is an AnyUnpacker provided for compatibility with jsonpb
type ProtoJSONPacker struct {
	JSONPBMarshaler *jsonpb.Marshaler
}

var _ AnyUnpacker = ProtoJSONPacker{}

func (a ProtoJSONPacker) UnpackAny(any *Any, _ interface{}) error {
	if any == nil {
		return nil
	}

	if any.cachedValue != nil {
		err := UnpackInterfaces(any.cachedValue, a)
		if err != nil {
			return err
		}
	}

	bz, err := a.JSONPBMarshaler.MarshalToString(any)
	any.compat = &anyCompat{
		jsonBz: []byte(bz),
		err:    err,
	}

	return err
}
