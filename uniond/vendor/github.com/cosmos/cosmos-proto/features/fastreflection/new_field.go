package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type newFieldGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *newFieldGen) generate() {
	g.genComment()
	g.P("func (x *", g.typeName, ") NewField(fd ", protoreflectPkg.Ident("FieldDescriptor"), ") ", protoreflectPkg.Ident("Value"), " {")
	g.P("switch fd.FullName() {")
	for _, field := range g.message.Fields {
		g.P("case \"", field.Desc.FullName(), "\":")
		g.genField(field)
	}
	g.P("default: ")
	g.P("if fd.IsExtension() {")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"proto3 declared messages do not support extensions: ", g.message.Desc.FullName(), "\"))")
	g.P("}")
	g.P("panic(fmt.Errorf(\"message ", g.message.Desc.FullName(), " does not contain field %s\", fd.FullName()))")
	g.P("}")
	g.P("}")
}

func (g *newFieldGen) genComment() {
	g.P("// NewField returns a new value that is assignable to the field")
	g.P("// for the given descriptor. For scalars, this returns the default value.")
	g.P("// For lists, maps, and messages, this returns a new, empty, mutable value.")
}

func (g *newFieldGen) genField(field *protogen.Field) {
	switch {
	case field.Desc.IsMap(), field.Desc.IsList(), field.Desc.Kind() == protoreflect.MessageKind:
		g.genMutable(field)
	default:
		g.P("return ", kindToValueConstructor(field.Desc.Kind()), "(", zeroValueForField(g.GeneratedFile, field), ")")
	}
}

func (g *newFieldGen) genMutable(field *protogen.Field) {
	switch {
	case field.Oneof != nil:
		g.genOneof(field)
	case field.Desc.IsMap():
		g.P("m := make(map[", getGoType(g.GeneratedFile, field.Message.Fields[0]), "]", getGoType(g.GeneratedFile, field.Message.Fields[1]), ")")
		g.P("return ", protoreflectPkg.Ident("ValueOfMap"), "(&", mapTypeName(field), "{m: &m})")
	case field.Desc.IsList():
		g.P("list := []", getGoType(g.GeneratedFile, field), "{}")
		g.P("return ", protoreflectPkg.Ident("ValueOfList"), "(&", listTypeName(field), "{list: &list})")
	case field.Desc.Kind() == protoreflect.MessageKind:
		g.P("m := new(", g.QualifiedGoIdent(field.Message.GoIdent), ")")
		g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(m.ProtoReflect())")
	default:
		panic("unreachable")
	}
}

func (g *newFieldGen) genOneof(field *protogen.Field) {
	if field.Desc.Kind() != protoreflect.MessageKind {
		panic("newfield oneof fastGenerator should be applied only to mutable message types")
	}
	g.P("value := &", g.QualifiedGoIdent(field.Message.GoIdent), "{}")
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(value.ProtoReflect())")
}
