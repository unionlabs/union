package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

const (
	mathPkg = protogen.GoImportPath("math")
)

// fields are not populated if
// if scalar: value != zero value
// if msg value != nil
// if list len(list) != 0
// if map len(map) != 0
// if oneof: oneof != nil (if oneof is scalar do we need to check it??)
// if bytes: len(bytes) != 0
type hasGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *hasGen) genComments() {
	g.P("// Has reports whether a field is populated.")
	g.P("//")
	g.P("// Some fields have the property of nullability where it is possible to")
	g.P("// distinguish between the default value of a field and whether the field")
	g.P("// was explicitly populated with the default value. Singular message fields,")
	g.P("// member fields of a oneof, and proto2 scalar fields are nullable. Such")
	g.P("// fields are populated only if explicitly set.")
	g.P("//")
	g.P("// In other cases (aside from the nullable cases above),")
	g.P("// a proto3 scalar field is populated if it contains a non-zero value, and")
	g.P("// a repeated field is populated if it is non-empty.")
}

func (g *hasGen) generate() {
	g.genComments()
	g.P("func (x *", g.typeName, ") Has(fd ", protoreflectPkg.Ident("FieldDescriptor"), ") bool {")
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

func (g *hasGen) genField(field *protogen.Field) {
	g.P("case \"", field.Desc.FullName(), "\":")
	if field.Desc.HasPresence() || field.Desc.IsList() || field.Desc.IsMap() || field.Desc.Kind() == protoreflect.BytesKind {
		g.genNullable(field)
		return
	}

	switch field.Desc.Kind() {
	case protoreflect.FloatKind:
		g.P("return x.", field.GoName, " != ", zeroValueForField(nil, field), " || ", mathPkg.Ident("Signbit"), "(float64(x.", field.GoName, "))")
	case protoreflect.DoubleKind:
		g.P("return x.", field.GoName, " != ", zeroValueForField(nil, field), " || ", mathPkg.Ident("Signbit"), "(x.", field.GoName, ")")
	default:
		g.P("return x.", field.GoName, " != ", zeroValueForField(nil, field))
	}

}

func (g *hasGen) genNullable(field *protogen.Field) {
	switch {
	case field.Desc.ContainingOneof() != nil:
		// case oneof is nil
		g.P("if x.", field.Oneof.GoName, " == nil {")
		g.P("return false")
		// if oneof is not nil we need to try cast it to the concrete type
		// and if it succeeds then it means the message has the field
		g.P("} else if _, ok := x.", field.Oneof.GoName, ".(*", field.GoIdent, "); ok {")
		g.P("return true")
		g.P("} else { ")
		g.P("return false")
		g.P("}")
	case field.Desc.IsMap(), field.Desc.IsList(), field.Desc.Kind() == protoreflect.BytesKind:
		g.P("return len(x.", field.GoName, ") != 0")
	case field.Desc.Kind() == protoreflect.MessageKind:
		g.P("return x.", field.GoName, " != nil")
	default:
		panic("unknown case")
	}
}
