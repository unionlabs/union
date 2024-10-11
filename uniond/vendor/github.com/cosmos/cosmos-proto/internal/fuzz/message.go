package fuzz

import (
	"fmt"
	"math"

	"google.golang.org/protobuf/reflect/protoreflect"
	"pgregory.net/rapid"
)

const (
	MaxDepthDefault   = 2
	MaxListLength     = 50
	MaxBytesArraySize = 100
)

func Message(t *rapid.T, typ protoreflect.MessageType) protoreflect.Message {
	g := &generator{
		typ:            typ,
		m:              typ.New(),
		t:              t,
		pickedOneofs:   map[protoreflect.FullName]protoreflect.FullName{},
		fillAll:        true,
		invalidValue:   false,
		depth:          0,
		maxDepth:       MaxDepthDefault,
		maxListLength:  MaxListLength,
		maxMapLength:   MaxListLength,
		maxBytesLength: MaxBytesArraySize,
	}

	g.generate()
	return g.m
}

type generator struct {
	typ protoreflect.MessageType
	m   protoreflect.Message
	t   *rapid.T

	pickedOneofs map[protoreflect.FullName]protoreflect.FullName // maps oneof fullname to picked field descriptor full name

	fillAll      bool
	invalidValue bool

	depth    int
	maxDepth int

	maxListLength  int
	maxMapLength   int
	maxBytesLength int
}

func (g *generator) generate() {
	if g.depth == g.maxDepth {
		g.m = g.typ.New()
		return
	}

	// pick oneofs
	g.decideOneofs()

	for i := 0; i < g.typ.Descriptor().Fields().Len(); i++ {
		fd := g.typ.Descriptor().Fields().Get(i)

		switch g.fillAll {
		case true:
			g.field(fd)
		default:
			genField := rapid.Bool().Draw(g.t, fmt.Sprintf("skip field generation: %s", fd.FullName()))
			if !genField {
				continue
			}

			g.field(fd)
		}
	}
}

// field fill the message with a random value
func (g *generator) field(fd protoreflect.FieldDescriptor) {
	// check if field is part of a oneof and if it is check if it was the picked one
	if isOneof(fd) && !g.chosenOneof(fd) {
		return
	}
	// check if we can set an invalid value
	if g.invalidValue && rapid.Bool().Draw(g.t, fmt.Sprintf("generate invalid value for field %s", fd.FullName())) {
		g.m.Set(fd, protoreflect.Value{})
	}

	switch {
	case fd.IsList():
		g.list(fd)
	case fd.IsMap():
		g.mapp(fd)
	default:
		g.value(fd)
	}
}

func isOneof(fd protoreflect.FieldDescriptor) bool {
	return fd.ContainingOneof() != nil
}

func (g *generator) list(fd protoreflect.FieldDescriptor) {
	list := g.m.NewField(fd).List()
	length := rapid.IntRange(0, g.maxListLength).Draw(g.t, fmt.Sprintf("list length for %s", fd.FullName()))

	for i := 0; i < length; i++ {
		switch fd.Kind() {
		case protoreflect.MessageKind:
			gen := g.embeddedMessage(list.NewElement().Message().Type())
			list.Append(protoreflect.ValueOfMessage(gen))
		default:
			list.Append(g.valueFor(fd))
		}
	}

	g.m.Set(fd, protoreflect.ValueOfList(list))
}

func (g *generator) mapp(fd protoreflect.FieldDescriptor) {
	keyDesc := fd.MapKey()
	valueDesc := fd.MapValue()

	mapValue := g.m.NewField(fd).Map()

	length := rapid.IntRange(0, g.maxMapLength).Draw(g.t, "map length for "+string(fd.FullName()))

	for i := 0; i < length; i++ {
		keyValue := protoreflect.MapKey(g.valueFor(keyDesc))
		var valueValue protoreflect.Value

		switch valueDesc.Kind() {
		case protoreflect.MessageKind:
			gen := g.embeddedMessage(mapValue.NewValue().Message().Type())
			valueValue = protoreflect.ValueOfMessage(gen)
		default:
			valueValue = g.valueFor(valueDesc)
		}
		mapValue.Set(keyValue, valueValue)
	}

	g.m.Set(fd, protoreflect.ValueOfMap(mapValue))
}

func (g *generator) value(fd protoreflect.FieldDescriptor) {
	var value protoreflect.Value
	switch fd.Kind() {
	case protoreflect.MessageKind:
		msg := g.embeddedMessage(g.m.NewField(fd).Message().Type())
		value = protoreflect.ValueOfMessage(msg)
	default:
		value = g.valueFor(fd)
	}

	g.m.Set(fd, value)
}

// valueFor generates a random protoreflect.Value which is not of protoreflect.MessageKind
func (g *generator) valueFor(fd protoreflect.FieldDescriptor) protoreflect.Value {
	switch fd.Kind() {
	// bool kind
	case protoreflect.BoolKind:
		value := rapid.Bool().Draw(g.t, label(fd))
		return protoreflect.ValueOfBool(value)
	// int32 kinds
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		value := rapid.Int32().Draw(g.t, label(fd))
		return protoreflect.ValueOfInt32(value)
	// int64 kinds
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		value := rapid.Int64().Draw(g.t, label(fd))
		return protoreflect.ValueOfInt64(value)
	// uint32 kinds
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		value := rapid.Uint32().Draw(g.t, label(fd))
		return protoreflect.ValueOfUint32(value)
	// uint64 kinds
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		value := rapid.Uint64().Draw(g.t, label(fd))
		return protoreflect.ValueOfUint64(value)
	// float32 kind
	case protoreflect.FloatKind:
		value := rapid.Float32Max(math.MaxFloat32).Draw(g.t, label(fd))
		return protoreflect.ValueOfFloat32(value)
	// float64 kind
	case protoreflect.DoubleKind:
		value := rapid.Float64().Draw(g.t, label(fd))
		return protoreflect.ValueOfFloat64(value)
	// string kind
	case protoreflect.StringKind:
		value := rapid.String().Draw(g.t, label(fd))
		return protoreflect.ValueOfString(value)
	// bytes kind
	case protoreflect.BytesKind:
		value := randomBytes(g.t, fd)
		return protoreflect.ValueOfBytes(value)
	// enum kind
	case protoreflect.EnumKind:
		enumIndex := rapid.IntRange(0, fd.Enum().Values().Len()-1).Draw(g.t, "random enum index for "+string(fd.FullName()))
		enum := fd.Enum().Values().Get(enumIndex)
		return protoreflect.ValueOfEnum(enum.Number())
	default:
		panic(fmt.Errorf("cannot handle: %s", fd.Kind()))
	}
}

// embeddedMessage returns a generator for a message which is contained within the current message
// it is needed mainly to avoid endless cycles on recursive messages
func (g *generator) embeddedMessage(typ protoreflect.MessageType) protoreflect.Message {
	gen := &generator{
		typ:            typ,
		m:              typ.New(),
		t:              g.t,
		fillAll:        g.fillAll,
		invalidValue:   g.invalidValue,
		depth:          g.depth + 1,
		maxDepth:       g.maxDepth,
		maxListLength:  g.maxListLength,
		maxMapLength:   g.maxMapLength,
		maxBytesLength: g.maxBytesLength,
		pickedOneofs:   map[protoreflect.FullName]protoreflect.FullName{},
	}

	gen.generate()
	return gen.m
}

// decideOneofs picks the one protoreflect.FieldDescriptor for each oneof
func (g *generator) decideOneofs() {
	md := g.typ.Descriptor()
	for i := 0; i < md.Oneofs().Len(); i++ {
		oneof := md.Oneofs().Get(i)
		index := rapid.IntRange(0, oneof.Fields().Len()-1).Draw(g.t, "deciding oneof field for: "+string(oneof.FullName()))
		decidedFd := oneof.Fields().Get(index)
		g.pickedOneofs[oneof.FullName()] = decidedFd.FullName()
	}
}

func (g *generator) chosenOneof(fd protoreflect.FieldDescriptor) bool {
	chosenFdName := g.pickedOneofs[fd.ContainingOneof().FullName()]

	return chosenFdName == fd.FullName()
}

func label(fd protoreflect.FieldDescriptor) string {
	return fmt.Sprintf("value for %s", fd.FullName())
}

func randomBytes(t *rapid.T, fd protoreflect.FieldDescriptor) []byte {
	size := rapid.IntRange(0, MaxBytesArraySize).Draw(t, "bytes slice size for %s"+string(fd.FullName()))
	return rapid.SliceOfN(rapid.Byte(), 0, size).Draw(t, "bytes slice for %s"+string(fd.FullName()))
}
