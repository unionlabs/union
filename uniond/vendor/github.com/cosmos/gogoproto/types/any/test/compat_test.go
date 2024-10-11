package test

import (
	"bytes"
	"testing"

	types "github.com/cosmos/gogoproto/types/any"
	"github.com/google/go-cmp/cmp"
	amino "github.com/tendermint/go-amino"
)

type TypeWithInterface struct {
	Animal Animal `json:"animal"`
	X      int64  `json:"x,omitempty"`
}

type testFixture struct {
	cdc  *amino.Codec
	a    TypeWithInterface
	b    HasAnimal
	spot *Dog
}

func newTestFixture(t *testing.T) *testFixture {
	t.Helper()

	cdc := amino.NewCodec()
	cdc.RegisterInterface((*Animal)(nil), nil)
	cdc.RegisterConcrete(&Dog{}, "test/Dog", nil)

	spot := &Dog{Size_: "small", Name: "Spot"}
	a := TypeWithInterface{Animal: spot}

	any, err := types.NewAnyWithCacheWithValue(spot)
	if err != nil {
		t.Fatal(err)
	}

	b := HasAnimal{Animal: any}

	return &testFixture{
		cdc:  cdc,
		a:    a,
		b:    b,
		spot: spot,
	}
}

func TestAminoBinary(t *testing.T) {
	s := newTestFixture(t)

	bz, err := s.cdc.MarshalBinaryBare(s.a)
	if err != nil {
		t.Fatal(err)
	}

	// expect plain amino marshal to fail
	_, err = s.cdc.MarshalBinaryBare(s.b)
	if err == nil {
		t.Fatal("expected error")
	}

	// expect unpack interfaces before amino marshal to succeed
	err = types.UnpackInterfaces(s.b, types.AminoPacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}
	bz2, err := s.cdc.MarshalBinaryBare(s.b)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(bz, bz2) {
		t.Fatalf("expected %X, got %X", bz, bz2)
	}

	var c HasAnimal
	err = s.cdc.UnmarshalBinaryBare(bz, &c)
	if err != nil {
		t.Fatal(err)
	}
	err = types.UnpackInterfaces(c, types.AminoUnpacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}

	if result := cmp.Diff(s.spot, c.Animal.GetCachedValue()); result != "" {
		t.Fatalf("expected %v, got %v: %v", s.spot, c.Animal.GetCachedValue(), result)
	}
}

func TestAminoJSON(t *testing.T) {
	s := newTestFixture(t)

	bz, err := s.cdc.MarshalJSON(s.a)
	if err != nil {
		t.Fatal(err)
	}

	// expect plain amino marshal to fail
	_, err = s.cdc.MarshalJSON(s.b)
	if err == nil {
		t.Fatal("expected error")
	}
	// expect unpack interfaces before amino marshal to succeed
	err = types.UnpackInterfaces(s.b, types.AminoJSONPacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}
	bz2, err := s.cdc.MarshalJSON(s.b)
	if err != nil {
		t.Fatal(err)
	}
	if !bytes.Equal(bz, bz2) {
		t.Fatalf("expected %X, got %X", bz, bz2)
	}

	var c HasAnimal
	err = s.cdc.UnmarshalJSON(bz, &c)
	if err != nil {
		t.Fatal(err)
	}
	err = types.UnpackInterfaces(c, types.AminoJSONUnpacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}

	if result := cmp.Diff(s.spot, c.Animal.GetCachedValue()); result != "" {
		t.Fatalf("expected %v, got %v: %v", s.spot, c.Animal.GetCachedValue(), result)
	}
}

func TestNested(t *testing.T) {
	s := newTestFixture(t)

	s.cdc.RegisterInterface((*HasAnimalI)(nil), nil)
	s.cdc.RegisterInterface((*HasHasAnimalI)(nil), nil)
	s.cdc.RegisterConcrete(&HasAnimal{}, "test/HasAnimal", nil)
	s.cdc.RegisterConcrete(&HasHasAnimal{}, "test/HasHasAnimal", nil)
	s.cdc.RegisterConcrete(&HasHasHasAnimal{}, "test/HasHasHasAnimal", nil)

	any, err := types.NewAnyWithCacheWithValue(&s.b)
	if err != nil {
		t.Fatal(err)
	}
	hha := HasHasAnimal{HasAnimal: any}
	any2, err := types.NewAnyWithCacheWithValue(&hha)
	if err != nil {
		t.Fatal(err)
	}
	hhha := HasHasHasAnimal{HasHasAnimal: any2}

	// marshal
	err = types.UnpackInterfaces(hhha, types.AminoPacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}
	bz, err := s.cdc.MarshalBinaryBare(hhha)
	if err != nil {
		t.Fatal(err)
	}

	// unmarshal
	var hhha2 HasHasHasAnimal
	err = s.cdc.UnmarshalBinaryBare(bz, &hhha2)
	if err != nil {
		t.Fatal(err)
	}
	err = types.UnpackInterfaces(hhha2, types.AminoUnpacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}

	if result := cmp.Diff(hhha2.TheHasHasAnimal().TheHasAnimal().TheAnimal(), s.spot); result != "" {
		t.Fatalf("expected %v, got %v: %v", s.spot, hhha2.TheHasHasAnimal().TheHasAnimal().TheAnimal(), result)
	}

	// json marshal
	err = types.UnpackInterfaces(hhha, types.AminoJSONPacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}
	jsonBz, err := s.cdc.MarshalJSON(hhha)
	if err != nil {
		t.Fatal(err)
	}

	// json unmarshal
	var hhha3 HasHasHasAnimal
	err = s.cdc.UnmarshalJSON(jsonBz, &hhha3)
	if err != nil {
		t.Fatal(err)
	}
	err = types.UnpackInterfaces(hhha3, types.AminoJSONUnpacker{Cdc: s.cdc})
	if err != nil {
		t.Fatal(err)
	}

	if result := cmp.Diff(hhha3.TheHasHasAnimal().TheHasAnimal().TheAnimal(), s.spot); result != "" {
		t.Fatalf("expected %v, got %v: %v", s.spot, hhha3.TheHasHasAnimal().TheHasAnimal().TheAnimal(), result)
	}
}
