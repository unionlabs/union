package types

import (
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"
)

type Dog struct {
	Name string `protobuf:"bytes,1,opt,name=name,proto3" json:"name,omitempty"`
}

func (d Dog) Greet() string { return d.Name }

// We implement a minimal proto.Message interface
func (d *Dog) Reset()                  { d.Name = "" }
func (d *Dog) String() string          { return d.Name }
func (d *Dog) ProtoMessage()           {}
func (d *Dog) XXX_MessageName() string { return "tests/dog" }

type Animal interface {
	Greet() string
}

var (
	_ Animal        = (*Dog)(nil)
	_ proto.Message = (*Dog)(nil)
)

func TestAnyPackUnpack(t *testing.T) {
	registry := NewInterfaceRegistry()
	registry.RegisterInterface("Animal", (*Animal)(nil))
	registry.RegisterImplementations(
		(*Animal)(nil),
		&Dog{},
	)

	spot := &Dog{Name: "Spot"}
	var animal Animal

	// with cache
	any, err := NewAnyWithValue(spot)
	require.NoError(t, err)
	require.Equal(t, spot, any.GetCachedValue())
	err = registry.UnpackAny(any, &animal)
	require.NoError(t, err)
	require.Equal(t, spot, animal)

	// without cache
	any.ResetCachedValue()
	err = registry.UnpackAny(any, &animal)
	require.NoError(t, err)
	require.Equal(t, spot, animal)
}

func TestString(t *testing.T) {
	require := require.New(t)
	spot := &Dog{Name: "Spot"}
	any, err := NewAnyWithValue(spot)
	require.NoError(err)

	require.Equal("&Any{TypeUrl:/tests/dog,Value:[10 4 83 112 111 116],XXX_unrecognized:[]}", any.String())
	require.Equal(`&Any{TypeUrl: "/tests/dog",
  Value: []byte{0xa, 0x4, 0x53, 0x70, 0x6f, 0x74}
}`, any.GoString())
}
