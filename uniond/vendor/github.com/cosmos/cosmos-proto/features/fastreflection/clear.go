package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

// fields are not populated if
// if scalar: value != zero value
// if msg value != nil
// if list len(list) != 0
// if map len(map) != 0
// if oneof: oneof != nil (if oneof is scalar do we need to check it??)
// if bytes: len(bytes) != 0
type clearGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *clearGen) genComments() {
	g.P("// Clear clears the field such that a subsequent Has call reports false.")
	g.P("//")
	g.P("// Clearing an extension field clears both the extension type and value")
	g.P("// associated with the given field number.")
	g.P("//")
	g.P("// Clear is a mutating operation and unsafe for concurrent use.")
}

func (g *clearGen) generate() {
	g.genComments()
	g.P("func (x *", g.typeName, ") Clear(fd ", protoreflectPkg.Ident("FieldDescriptor"), ") {")
	g.P("switch fd.FullName() {")
	for _, field := range g.message.Fields {
		g.genField(field)
	}
	g.P("default:")
	g.P("if fd.IsExtension() {")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"proto3 declared messages do not support extensions: ", g.message.Desc.FullName(), "\"))")
	g.P("}")
	g.P("panic(fmt.Errorf(\"message ", g.message.Desc.FullName(), " does not contain field %s\", fd.FullName()))")
	g.P("}")
	g.P("}")
}

func (g *clearGen) genField(field *protogen.Field) {
	g.P("case \"", field.Desc.FullName(), "\":")
	if field.Desc.HasPresence() || field.Desc.IsList() || field.Desc.IsMap() || field.Desc.Kind() == protoreflect.BytesKind {
		g.genNullable(field)
		return
	}

	g.P("x.", field.GoName, " = ", zeroValueForField(nil, field))
}

func (g *clearGen) genNullable(field *protogen.Field) {
	switch {
	case field.Desc.ContainingOneof() != nil:
		g.P("x.", field.Oneof.GoName, " = nil")
	case field.Desc.IsMap(), field.Desc.IsList(), field.Desc.Kind() == protoreflect.BytesKind:
		g.P("x.", field.GoName, " = nil")
	case field.Desc.Kind() == protoreflect.MessageKind:
		g.P(" x.", field.GoName, " = nil")
	default:
		panic("unknown case")
	}
}
