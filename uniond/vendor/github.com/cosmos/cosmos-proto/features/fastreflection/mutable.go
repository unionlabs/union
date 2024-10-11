package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type mutableGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *mutableGen) generate() {
	g.genComment()
	g.P("func (x *", g.typeName, ") Mutable(fd ", protoreflectPkg.Ident("FieldDescriptor"), ") ", protoreflectPkg.Ident("Value"), " {")
	g.P("switch fd.FullName()  {")
	// we first output all the fields that are mutable
	for _, field := range g.message.Fields {
		if !mutable(field) {
			continue
		}
		g.P("case \"", field.Desc.FullName(), "\":")
		g.genField(field)
	}
	// then we parse those that are not mutable
	for _, field := range g.message.Fields {
		if mutable(field) {
			continue
		}
		g.P("case \"", field.Desc.FullName(), "\":")
		g.P("panic(", fmtPkg.Ident("Errorf"), "(\"field ", field.Desc.Name(), " of message ", g.message.Desc.FullName(), " is not mutable\"))")
	}
	// then the default case
	g.P("default:")
	g.P("if fd.IsExtension() {")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"proto3 declared messages do not support extensions: ", g.message.Desc.FullName(), "\"))")
	g.P("}")
	g.P("panic(fmt.Errorf(\"message ", g.message.Desc.FullName(), " does not contain field %s\", fd.FullName()))")
	g.P("}")
	g.P("}")
}

func mutable(field *protogen.Field) bool {
	switch {
	case field.Desc.IsMap():
		return true
	case field.Desc.IsList():
		return true
	case field.Desc.Kind() == protoreflect.MessageKind:
		return true
	default:
		return false
	}
}

func (g *mutableGen) genComment() {
	g.P("// Mutable returns a mutable reference to a composite type.")
	g.P("//")
	g.P("// If the field is unpopulated, it may allocate a composite value.")
	g.P("// For a field belonging to a oneof, it implicitly clears any other field")
	g.P("// that may be currently set within the same oneof.")
	g.P("// For extension fields, it implicitly stores the provided ExtensionType")
	g.P("// if not already stored.")
	g.P("// It panics if the field does not contain a composite type.")
	g.P("//")
	g.P("// Mutable is a mutating operation and unsafe for concurrent use.")
}

func (g *mutableGen) genField(field *protogen.Field) {
	if field.Oneof != nil {
		g.genOneof(field)
		return
	}

	switch {
	case field.Desc.IsMap():
		// if map is invalid then we make it valid
		g.P("if x.", field.GoName, " == nil {")
		g.P("x.", field.GoName, " = make(map[", getGoType(g.GeneratedFile, field.Message.Fields[0]), "]", getGoType(g.GeneratedFile, field.Message.Fields[1]), ")")
		g.P("}")
		// return value of map
		g.P("value := &", mapTypeName(field), "{m: &x.", field.GoName, "}")
		g.P("return ", protoreflectPkg.Ident("ValueOfMap"), "(value)")
	case field.Desc.IsList():
		g.P("if x.", field.GoName, " == nil {")
		g.P("x.", field.GoName, " = []", getGoType(g.GeneratedFile, field), "{}")
		g.P("}")
		g.P("value := &", listTypeName(field), "{list: &x.", field.GoName, "}")
		g.P("return ", protoreflectPkg.Ident("ValueOfList"), "(value)")
	case field.Desc.Kind() == protoreflect.MessageKind:
		g.P("if x.", field.GoName, " == nil {")
		g.P("x.", field.GoName, " = new(", g.QualifiedGoIdent(field.Message.GoIdent), ")")
		g.P("}")
		g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(x.", field.GoName, ".ProtoReflect())")
	default:
		panic("unreachable")
	}
}

func (g *mutableGen) genOneof(field *protogen.Field) {
	// if the oneof is nil then we just create a new instance of the object and return it
	g.P("if x.", field.Oneof.GoName, " == nil {")
	g.P("value := &", g.QualifiedGoIdent(field.Message.GoIdent), "{}")
	g.P("oneofValue := &", g.QualifiedGoIdent(field.GoIdent), "{", field.GoName, ": value}")
	g.P("x.", field.Oneof.GoName, " = oneofValue")
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(value.ProtoReflect())")
	g.P("}")
	// if the oneof is not nil,
	g.P("switch m := x.", field.Oneof.GoName, ".(type) {")
	// we check if the type matches the oneof type of the field
	g.P("case *", g.QualifiedGoIdent(field.GoIdent), ":")
	// if it does we return it
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(m.", field.GoName, ".ProtoReflect())")
	// otherwise we reset the field with the new instance
	g.P("default:")
	g.P("value := &", g.QualifiedGoIdent(field.Message.GoIdent), "{}")
	g.P("oneofValue := &", g.QualifiedGoIdent(field.GoIdent), "{", field.GoName, ": value}")
	g.P("x.", field.Oneof.GoName, " = oneofValue")
	g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(value.ProtoReflect())")
	g.P("}")

}
