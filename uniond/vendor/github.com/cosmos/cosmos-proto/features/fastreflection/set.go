package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type setGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *setGen) generate() {
	g.genComment()
	g.P("func (x *", g.typeName, ") Set(fd ", protoreflectPkg.Ident("FieldDescriptor"), ", value ", protoreflectPkg.Ident("Value"), ") {")
	g.P("switch fd.FullName() {")
	for _, field := range g.message.Fields {
		g.P("case \"", field.Desc.FullName(), "\":")
		g.genField(field)
	}
	g.P("default:")
	g.genDefaultCase()
	g.P("}")
	g.P("}")
	g.P()
}

func (g *setGen) genComment() {
	g.P("// Set stores the value for a field.")
	g.P("//")
	g.P("// For a field belonging to a oneof, it implicitly clears any other field")
	g.P("// that may be currently set within the same oneof.")
	g.P("// For extension fields, it implicitly stores the provided ExtensionType.")
	g.P("// When setting a composite type, it is unspecified whether the stored value")
	g.P("// aliases the source's memory in any way. If the composite value is an")
	g.P("// empty, read-only value, then it panics.")
	g.P("//")
	g.P("// Set is a mutating operation and unsafe for concurrent use.")
}

func (g *setGen) genField(field *protogen.Field) {
	if field.Oneof != nil {
		g.genOneof(field)
		return
	}

	switch {
	case field.Desc.IsMap():
		g.genMap(field)
		return
	case field.Desc.IsList():
		g.genList(field)
		return
	}

	fieldRef := "x." + field.GoName

	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		g.P(fieldRef, " = value.Bool()")
	case protoreflect.EnumKind:
		g.P(fieldRef, " = (", g.QualifiedGoIdent(field.Enum.GoIdent), ")(value.Enum())")
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		g.P(fieldRef, " = int32(value.Int())")
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		g.P(fieldRef, " = uint32(value.Uint())")
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		g.P(fieldRef, " = value.Int()")
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		g.P(fieldRef, " = value.Uint()")
	case protoreflect.FloatKind:
		g.P(fieldRef, " = float32(value.Float())")
	case protoreflect.DoubleKind:
		g.P(fieldRef, " = value.Float()")
	case protoreflect.StringKind:
		g.P(fieldRef, " = value.Interface().(string)")
	case protoreflect.BytesKind:
		g.P(fieldRef, " = value.Bytes()")
	case protoreflect.MessageKind, protoreflect.GroupKind:
		g.P(fieldRef, " = value.Message().Interface().(*", g.QualifiedGoIdent(field.Message.GoIdent), ")")
	}

}

// genDefaultCase generates the default case for field descriptor
func (g *setGen) genDefaultCase() {
	g.P("if fd.IsExtension() {")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"proto3 declared messages do not support extensions: ", g.message.Desc.FullName(), "\"))")
	g.P("}")
	g.P("panic(fmt.Errorf(\"message ", g.message.Desc.FullName(), " does not contain field %s\", fd.FullName()))")
}

func (g *setGen) genOneof(field *protogen.Field) {
	g.genOneofValueUnwrapper(field)
	g.P("x.", field.Oneof.GoName, " = &", g.QualifiedGoIdent(field.GoIdent), "{", field.GoName, ": cv", "}")
}

// genMap generates the implementation of set for map types.
// for implementation details look at genList
func (g *setGen) genMap(field *protogen.Field) {
	g.P("mv := value.Map()")
	g.P("cmv := mv.(*", mapTypeName(field), ")")
	g.P("x.", field.GoName, " = *cmv.m")
}

// genList generates the implementation of set for list types.
// NOTE: in the original protoreflect implementation, using a List implementation
// which does not match the expected impl.listReflect type panics.
// To respect this behaviour, we use the fastpath list casting to our
// list implementation. This is to respect mutability guarantees.
// After we set a list the value can still be mutated.
func (g *setGen) genList(field *protogen.Field) {
	g.P("lv := value.List()")
	g.P("clv := lv.(*", listTypeName(field), ")")
	g.P("x.", field.GoName, " = *clv.list")
}

func (g *setGen) genOneofValueUnwrapper(field *protogen.Field) {
	fieldRef := "cv  := "
	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		g.P(fieldRef, " value.Bool()")
	case protoreflect.EnumKind:
		g.P(fieldRef, " (", g.QualifiedGoIdent(field.Enum.GoIdent), ")(value.Enum())")
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		g.P(fieldRef, " int32(value.Int())")
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		g.P(fieldRef, " uint32(value.Uint())")
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		g.P(fieldRef, "  value.Int()")
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		g.P(fieldRef, "  value.Uint()")
	case protoreflect.FloatKind:
		g.P(fieldRef, "  float32(value.Float())")
	case protoreflect.DoubleKind:
		g.P(fieldRef, "  value.Float()")
	case protoreflect.StringKind:
		g.P(fieldRef, "  value.Interface().(string)")
	case protoreflect.BytesKind:
		g.P(fieldRef, "  value.Bytes()")
	case protoreflect.MessageKind, protoreflect.GroupKind:
		g.P(fieldRef, "  value.Message().Interface().(*", g.QualifiedGoIdent(field.Message.GoIdent), ")")
	}
}
