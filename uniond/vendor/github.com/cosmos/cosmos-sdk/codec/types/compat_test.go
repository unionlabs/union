package types_test

import (
	"testing"

	"github.com/stretchr/testify/suite"
	amino "github.com/tendermint/go-amino"

	"github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
)

type TypeWithInterface struct {
	Animal testdata.Animal `json:"animal"`
	X      int64           `json:"x,omitempty"`
}

type Suite struct {
	suite.Suite
	cdc  *amino.Codec
	a    TypeWithInterface
	b    testdata.HasAnimal
	spot *testdata.Dog
}

func (s *Suite) SetupTest() {
	s.cdc = amino.NewCodec()
	s.cdc.RegisterInterface((*testdata.Animal)(nil), nil)
	s.cdc.RegisterConcrete(&testdata.Dog{}, "testdata/Dog", nil)

	s.spot = &testdata.Dog{Size_: "small", Name: "Spot"}
	s.a = TypeWithInterface{Animal: s.spot}

	any, err := types.NewAnyWithValue(s.spot)
	s.Require().NoError(err)
	s.b = testdata.HasAnimal{Animal: any}
}

func (s *Suite) TestAminoBinary() {
	bz, err := s.cdc.MarshalBinaryBare(s.a)
	s.Require().NoError(err)

	// expect plain amino marshal to fail
	_, err = s.cdc.MarshalBinaryBare(s.b)
	s.Require().Error(err)

	// expect unpack interfaces before amino marshal to succeed
	err = types.UnpackInterfaces(s.b, types.AminoPacker{Cdc: s.cdc})
	s.Require().NoError(err)
	bz2, err := s.cdc.MarshalBinaryBare(s.b)
	s.Require().NoError(err)
	s.Require().Equal(bz, bz2)

	var c testdata.HasAnimal
	err = s.cdc.UnmarshalBinaryBare(bz, &c)
	s.Require().NoError(err)
	err = types.UnpackInterfaces(c, types.AminoUnpacker{Cdc: s.cdc})
	s.Require().NoError(err)
	s.Require().Equal(s.spot, c.Animal.GetCachedValue())
}

func (s *Suite) TestAminoJSON() {
	bz, err := s.cdc.MarshalJSON(s.a)
	s.Require().NoError(err)

	// expect plain amino marshal to fail
	_, err = s.cdc.MarshalJSON(s.b)
	s.Require().Error(err)

	// expect unpack interfaces before amino marshal to succeed
	err = types.UnpackInterfaces(s.b, types.AminoJSONPacker{Cdc: s.cdc})
	s.Require().NoError(err)
	bz2, err := s.cdc.MarshalJSON(s.b)
	s.Require().NoError(err)
	s.Require().Equal(string(bz), string(bz2))

	var c testdata.HasAnimal
	err = s.cdc.UnmarshalJSON(bz, &c)
	s.Require().NoError(err)
	err = types.UnpackInterfaces(c, types.AminoJSONUnpacker{Cdc: s.cdc})
	s.Require().NoError(err)
	s.Require().Equal(s.spot, c.Animal.GetCachedValue())
}

func (s *Suite) TestNested() {
	s.cdc.RegisterInterface((*testdata.HasAnimalI)(nil), nil)
	s.cdc.RegisterInterface((*testdata.HasHasAnimalI)(nil), nil)
	s.cdc.RegisterConcrete(&testdata.HasAnimal{}, "testdata/HasAnimal", nil)
	s.cdc.RegisterConcrete(&testdata.HasHasAnimal{}, "testdata/HasHasAnimal", nil)
	s.cdc.RegisterConcrete(&testdata.HasHasHasAnimal{}, "testdata/HasHasHasAnimal", nil)

	any, err := types.NewAnyWithValue(&s.b)
	s.Require().NoError(err)
	hha := testdata.HasHasAnimal{HasAnimal: any}
	any2, err := types.NewAnyWithValue(&hha)
	s.Require().NoError(err)
	hhha := testdata.HasHasHasAnimal{HasHasAnimal: any2}

	// marshal
	err = types.UnpackInterfaces(hhha, types.AminoPacker{Cdc: s.cdc})
	s.Require().NoError(err)
	bz, err := s.cdc.MarshalBinaryBare(hhha)
	s.Require().NoError(err)

	// unmarshal
	var hhha2 testdata.HasHasHasAnimal
	err = s.cdc.UnmarshalBinaryBare(bz, &hhha2)
	s.Require().NoError(err)
	err = types.UnpackInterfaces(hhha2, types.AminoUnpacker{Cdc: s.cdc})
	s.Require().NoError(err)

	s.Require().Equal(s.spot, hhha2.TheHasHasAnimal().TheHasAnimal().TheAnimal())

	// json marshal
	err = types.UnpackInterfaces(hhha, types.AminoJSONPacker{Cdc: s.cdc})
	s.Require().NoError(err)
	jsonBz, err := s.cdc.MarshalJSON(hhha)
	s.Require().NoError(err)

	// json unmarshal
	var hhha3 testdata.HasHasHasAnimal
	err = s.cdc.UnmarshalJSON(jsonBz, &hhha3)
	s.Require().NoError(err)
	err = types.UnpackInterfaces(hhha3, types.AminoJSONUnpacker{Cdc: s.cdc})
	s.Require().NoError(err)

	s.Require().Equal(s.spot, hhha3.TheHasHasAnimal().TheHasAnimal().TheAnimal())
}

func TestSuite(t *testing.T) {
	suite.Run(t, &Suite{})
}
