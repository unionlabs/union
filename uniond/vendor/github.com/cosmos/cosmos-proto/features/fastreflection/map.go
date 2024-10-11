package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"

	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type mapGen struct {
	*generator.GeneratedFile

	field    *protogen.Field // TODO(fdmylja): maybe we could split this field into 2 fields one for key and one for value for the sake of being more readable
	typeName string
}

func (g *mapGen) generate() {
	g.typeName = mapTypeName(g.field)

	g.genAssertions()
	g.genType()
	g.genLen()
	g.genRange()
	g.genHas()
	g.genClear()
	g.genGet()
	g.genSet()
	g.genMutable()
	g.genNewValue()
	g.genIsValid()
}

// genAssertions generates protoreflect.Map type assertions
func (g *mapGen) genAssertions() {
	g.P("var _ ", protoreflectPkg.Ident("Map"), " = (*", g.typeName, ")(nil)")
}

// genType generates the type definition for the protoreflect.Map implementer
func (g *mapGen) genType() {
	g.P("type ", g.typeName, " struct {")
	g.P("m *map[", getGoType(g.GeneratedFile, g.field.Message.Fields[0]), "]", getGoType(g.GeneratedFile, g.field.Message.Fields[1]))
	g.P("}")
	g.P()
}

// genLen generates the implementation of protoreflect.Map.Len
func (g *mapGen) genLen() {
	g.P("func (x *", g.typeName, ") Len() int {")
	// invalid map
	g.P("if x.m == nil {")
	g.P("return 0")
	g.P("}")
	// valid map
	g.P("return len(*x.m)")
	g.P("}")
	g.P()
}

// genRange generates the implementation for protoreflect.Map.Range
func (g *mapGen) genRange() {
	g.P("func (x *", g.typeName, ") Range(f func(", protoreflectPkg.Ident("MapKey"), ", ", protoreflectPkg.Ident("Value"), ") bool) {")
	// invalid map
	g.P("if x.m == nil {")
	g.P("return")
	g.P("}")
	// valid map
	g.P("for k, v := range *x.m {")
	g.P("mapKey := (", protoreflectPkg.Ident("MapKey"), ")(", kindToValueConstructor(g.field.Message.Fields[0].Desc.Kind()), "(k))")
	switch g.field.Message.Fields[1].Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("mapValue := ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "(v.ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("mapValue := ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "(v.Number())")
	default:
		g.P("mapValue := ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "(v)")
	}
	g.P("if !f(mapKey, mapValue) {")
	g.P("break")
	g.P("}")
	g.P("}")
	g.P("}")
	g.P()
}

// genHas generates the implementation for protoreflect.Map.Has
func (g *mapGen) genHas() {
	g.P("func (x *", g.typeName, ") Has(key ", protoreflectPkg.Ident("MapKey"), ") bool {")
	// invalid map
	g.P("if x.m == nil {")
	g.P("return false")
	g.P("}")
	// valid map
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[0], "key", "concreteValue")
	g.P("_, ok := (*x.m)[concreteValue]")
	g.P("return ok")
	g.P("}")
	g.P()
}

func (g *mapGen) genClear() {
	g.P("func (x *", g.typeName, ") Clear(key ", protoreflectPkg.Ident("MapKey"), ") {")
	// invalid map
	g.P("if x.m == nil {")
	g.P("return")
	g.P("}")
	// valid map
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[0], "key", "concreteKey")
	g.P("delete(*x.m, concreteKey)")
	g.P("}")
	g.P()
}

func (g *mapGen) genGet() {
	g.P("func (x *", g.typeName, ") Get(key ", protoreflectPkg.Ident("MapKey"), ") ", protoreflectPkg.Ident("Value"), "{")
	g.P("if x.m == nil {")
	g.P("return ", protoreflectPkg.Ident("Value"), "{}")
	g.P("}")
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[0], "key", "concreteKey")
	g.P("v, ok := (*x.m)[concreteKey]")
	g.P("if !ok {")
	g.P("return ", protoreflectPkg.Ident("Value"), "{}")
	g.P("}")
	switch g.field.Message.Fields[1].Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "(v.ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("return ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(v))")
	default:
		g.P("return ", kindToValueConstructor(g.field.Message.Fields[1].Desc.Kind()), "(v)")
	}
	g.P("}")
	g.P()
}

func (g *mapGen) genSet() {
	g.P("func (x *", g.typeName, ") Set(key ", protoreflectPkg.Ident("MapKey"), ", value ", protoreflectPkg.Ident("Value"), ") {")
	g.P("if !key.IsValid() || !value.IsValid() {")
	g.P("panic(\"invalid key or value provided\")")
	g.P("}")
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[0], "key", "concreteKey")
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[1], "value", "concreteValue")
	g.P("(*x.m)[concreteKey] = concreteValue")
	g.P("}")
	g.P()
}

func (g *mapGen) genMutable() {
	// if it's not a message value type, we construct a panic function
	g.P("func (x *", g.typeName, ") Mutable(key ", protoreflectPkg.Ident("MapKey"), ") ", protoreflectPkg.Ident("Value"), " {")
	if g.field.Message.Fields[1].Desc.Kind() != protoreflect.MessageKind {
		panicMsg := "should not call Mutable on protoreflect.Map whose value is not of type protoreflect.Message"
		g.P("panic(\"", panicMsg, "\")")
		g.P("}")
		g.P()
		return
	}
	// generate mutable message logic
	genPrefValueToGoValue(g.GeneratedFile, g.field.Message.Fields[0], "key", "concreteKey")
	g.P("v, ok := (*x.m)[concreteKey]")
	g.P("if ok {")
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(v.ProtoReflect())")
	g.P("}")
	g.P("newValue := new(", g.QualifiedGoIdent(g.field.Message.Fields[1].Message.GoIdent), ")")
	g.P("(*x.m)[concreteKey] = newValue")
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(newValue.ProtoReflect())")
	g.P("}")
	g.P()
}

func (g *mapGen) genNewValue() {
	g.P("func (x *", g.typeName, ") NewValue() ", protoreflectPkg.Ident("Value"), " {")
	valueField := g.field.Message.Fields[1]
	switch {
	case valueField.Desc.Kind() == protoreflect.BytesKind:
		g.P("var v []byte")
	default:
		g.P("v := ", zeroValueForField(g.GeneratedFile, valueField))
	}
	switch valueField.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", kindToValueConstructor(valueField.Desc.Kind()), "(v.ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("return ", kindToValueConstructor(valueField.Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(v))")
	default:
		g.P("return ", kindToValueConstructor(valueField.Desc.Kind()), "(v)")
	}
	g.P("}")
	g.P()
}

func (g *mapGen) genIsValid() {
	g.P("func (x *", g.typeName, ") IsValid() bool {")
	g.P("return x.m != nil")
	g.P("}")
	g.P()
}

func mapTypeName(field *protogen.Field) string {
	return fmt.Sprintf("_%s_%d_map", field.Parent.GoIdent.GoName, field.Desc.Number())
}
