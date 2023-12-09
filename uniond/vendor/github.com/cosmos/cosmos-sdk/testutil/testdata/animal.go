package testdata

// nolint

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	"github.com/cosmos/cosmos-sdk/codec/types"
)

type Animal interface {
	proto.Message

	Greet() string
}

type Cartoon interface {
	proto.Message

	Identify() string
}

func (c *Cat) Greet() string {
	return fmt.Sprintf("Meow, my name is %s", c.Moniker)
}

func (c *Bird) Identify() string {
	return "This is Tweety."
}

func (d Dog) Greet() string {
	return fmt.Sprintf("Roof, my name is %s", d.Name)
}

var _ types.UnpackInterfacesMessage = HasAnimal{}

func (m HasAnimal) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	var animal Animal
	return unpacker.UnpackAny(m.Animal, &animal)
}

type HasAnimalI interface {
	TheAnimal() Animal
}

var _ HasAnimalI = &HasAnimal{}

func (m HasAnimal) TheAnimal() Animal {
	return m.Animal.GetCachedValue().(Animal)
}

type HasHasAnimalI interface {
	TheHasAnimal() HasAnimalI
}

var _ HasHasAnimalI = &HasHasAnimal{}

func (m HasHasAnimal) TheHasAnimal() HasAnimalI {
	return m.HasAnimal.GetCachedValue().(HasAnimalI)
}

var _ types.UnpackInterfacesMessage = HasHasAnimal{}

func (m HasHasAnimal) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	var animal HasAnimalI
	return unpacker.UnpackAny(m.HasAnimal, &animal)
}

type HasHasHasAnimalI interface {
	TheHasHasAnimal() HasHasAnimalI
}

var _ HasHasAnimalI = &HasHasAnimal{}

func (m HasHasHasAnimal) TheHasHasAnimal() HasHasAnimalI {
	return m.HasHasAnimal.GetCachedValue().(HasHasAnimalI)
}

var _ types.UnpackInterfacesMessage = HasHasHasAnimal{}

func (m HasHasHasAnimal) UnpackInterfaces(unpacker types.AnyUnpacker) error {
	var animal HasHasAnimalI
	return unpacker.UnpackAny(m.HasHasAnimal, &animal)
}
