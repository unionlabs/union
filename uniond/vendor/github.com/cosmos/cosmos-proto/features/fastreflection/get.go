package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type getGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *getGen) generate() {
	g.genComment()
	g.P("func (x *", g.typeName, ") Get(descriptor ", protoreflectPkg.Ident("FieldDescriptor"), ") ", protoreflectPkg.Ident("Value"), " {")
	g.P("switch descriptor.FullName() {")
	// implement the fastReflectionFeature Get function
	for _, field := range g.message.Fields {
		g.P("case \"", field.Desc.FullName(), "\":")
		g.genFieldGetter(field)
	}
	// insert default case which panics
	g.P("default:")
	g.genDefaultCase()
	g.P("}")
	g.P("}")
	g.P()
}

func (g *getGen) genComment() {
	g.P("// Get retrieves the value for a field.")
	g.P("//")
	g.P("// For unpopulated scalars, it returns the default value, where")
	g.P("// the default value of a bytes scalar is guaranteed to be a copy.")
	g.P("// For unpopulated composite types, it returns an empty, read-only view")
	g.P("// of the value; to obtain a mutable reference, use Mutable.")
}

func (g *getGen) genFieldGetter(field *protogen.Field) {
	if field.Oneof != nil {
		g.genOneofGetter(field)
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
	g.P("value := ", fieldRef)
	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfBool"), "(value)")
	case protoreflect.EnumKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfEnum"), "((", protoreflectPkg.Ident("EnumNumber"), ")", "(value)", ")")
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		g.P("return ", protoreflectPkg.Ident("ValueOfInt32"), "(value)")
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		g.P("return ", protoreflectPkg.Ident("ValueOfUint32"), "(value)")
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		g.P("return ", protoreflectPkg.Ident("ValueOfInt64"), "(value)")
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		g.P("return ", protoreflectPkg.Ident("ValueOfUint64"), "(value)")
	case protoreflect.FloatKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfFloat32"), "(value)")
	case protoreflect.DoubleKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfFloat64"), "(value)")
	case protoreflect.StringKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfString"), "(value)")
	case protoreflect.BytesKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfBytes"), "(value)")
	case protoreflect.MessageKind, protoreflect.GroupKind:
		g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(value.ProtoReflect())")
	}
}

func (g *getGen) genOneofGetter(fd *protogen.Field) {
	// handle the case in which the oneof field is not set
	g.P("if x.", fd.Oneof.GoName, " == nil {")
	switch fd.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "((*", g.QualifiedGoIdent(fd.Message.GoIdent), ")(nil).ProtoReflect())")
	default:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "(", zeroValueForField(g.GeneratedFile, fd), ")")
	}
	// handle the case in which oneof field is set and it matches our sub-onefield type
	g.P("} else if v, ok := x.", fd.Oneof.GoName, ".(*", fd.GoIdent, "); ok {")
	oneofTypeContainerFieldName := fd.GoName // field containing the oneof value
	switch fd.Desc.Kind() {
	case protoreflect.MessageKind: // it can be mutable
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "(v.", oneofTypeContainerFieldName, ".ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(v.", oneofTypeContainerFieldName, "))")
	default:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "(v.", oneofTypeContainerFieldName, ")")
	}
	// handle the case in which the oneof field is set but it does not match our field type
	g.P("} else {")
	switch fd.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "((*", g.QualifiedGoIdent(fd.Message.GoIdent), ")(nil).ProtoReflect())")
	default:
		g.P("return ", kindToValueConstructor(fd.Desc.Kind()), "(", zeroValueForField(g.GeneratedFile, fd), ")")
	}
	g.P("}")
}

// genDefaultCase generates the default case for field descriptor
func (g *getGen) genDefaultCase() {
	g.P("if descriptor.IsExtension() {")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"proto3 declared messages do not support extensions: ", g.message.Desc.FullName(), "\"))")
	g.P("}")
	g.P("panic(fmt.Errorf(\"message ", g.message.Desc.FullName(), " does not contain field %s\", descriptor.FullName()))")
}

// genMap generates the protoreflect.Message.Get for map types
func (g *getGen) genMap(field *protogen.Field) {
	// gen invalid case
	g.P("if len(x.", field.GoName, ") == 0 {")
	g.P("return ", protoreflectPkg.Ident("ValueOfMap"), "(&", mapTypeName(field), "{})")
	g.P("}")
	// gen valid case
	g.P("mapValue := &", mapTypeName(field), "{m: &x.", field.GoName, "}")
	g.P("return ", protoreflectPkg.Ident("ValueOfMap"), "(mapValue)")
}

func (g *getGen) genList(field *protogen.Field) {
	// gen invalid case
	g.P("if len(x.", field.GoName, ") == 0 {")
	g.P("return ", protoreflectPkg.Ident("ValueOfList"), "(&", listTypeName(field), "{})")
	g.P("}")
	// gen valid case
	g.P("listValue := &", listTypeName(field), "{list: &x.", field.GoName, "}")
	g.P("return ", protoreflectPkg.Ident("ValueOfList"), "(listValue)")
}

// genGet generates the implementation for protoreflect.Message.Get
func (g *fastGenerator) genGet() {
	(&getGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
}
