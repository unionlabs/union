package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type rangeGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message

	processedOneofs map[string]struct{}
}

func (g *rangeGen) generate() {
	g.processedOneofs = map[string]struct{}{}

	g.genComment()
	g.P("func (x *", g.typeName, ") Range(f func(", protoreflectPkg.Ident("FieldDescriptor"), ", ", protoreflectPkg.Ident("Value"), ") bool) {")
	for _, field := range g.message.Fields {
		g.genField(field)
	}
	g.P("}")
}

func (g *rangeGen) genComment() {
	g.P("// Range iterates over every populated field in an undefined order,")
	g.P("// calling f for each field descriptor and value encountered.")
	g.P("// Range returns immediately if f returns false.")
	g.P("// While iterating, mutating operations may only be performed")
	g.P("// on the current field descriptor.")
}

func (g *rangeGen) genField(field *protogen.Field) {
	if field.Oneof != nil {
		g.genOneof(field)
		return
	}

	switch {
	case field.Desc.IsMap():
		g.P("if len(x.", field.GoName, ") != 0 {")
		g.P("value := ", protoreflectPkg.Ident("ValueOfMap"), "(&", mapTypeName(field), "{m: &x.", field.GoName, "})")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	case field.Desc.IsList():
		g.P("if len(x.", field.GoName, ") != 0 {")
		g.P("value := ", protoreflectPkg.Ident("ValueOfList"), "(&", listTypeName(field), "{list: &x.", field.GoName, "})")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	case field.Desc.Kind() == protoreflect.MessageKind:
		g.P("if x.", field.GoName, " != nil {")
		g.P("value := ", protoreflectPkg.Ident("ValueOfMessage"), "(x.", field.GoName, ".ProtoReflect())")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	case field.Desc.Kind() == protoreflect.BytesKind:
		g.P("if len(x.", field.GoName, ") != 0 {")
		g.P("value := ", protoreflectPkg.Ident("ValueOfBytes"), "(x.", field.GoName, ")")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	case field.Desc.Kind() == protoreflect.DoubleKind:
		g.P("if x.", field.GoName, " != ", zeroValueForField(g.GeneratedFile, field), " || ", mathPkg.Ident("Signbit"), "(x.", field.GoName, ")", "{")
		g.P("value := ", kindToValueConstructor(field.Desc.Kind()), "(x.", field.GoName, ")")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	case field.Desc.Kind() == protoreflect.FloatKind:
		g.P("if x.", field.GoName, " != ", zeroValueForField(g.GeneratedFile, field), " || ", mathPkg.Ident("Signbit"), "(float64(x.", field.GoName, "))", "{")
		g.P("value := ", kindToValueConstructor(field.Desc.Kind()), "(x.", field.GoName, ")")
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	default:
		g.P("if x.", field.GoName, " != ", zeroValueForField(g.GeneratedFile, field), "{")
		switch {
		case field.Desc.Kind() == protoreflect.EnumKind:
			g.P("value := ", kindToValueConstructor(field.Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(x.", field.GoName, ")) ")
		default:
			g.P("value := ", kindToValueConstructor(field.Desc.Kind()), "(x.", field.GoName, ")")
		}
		g.P("if !f(", fieldDescriptorName(field), ", value) {")
		g.P("return")
		g.P("}")
		g.P("}")
	}
}

func (g *rangeGen) genOneof(field *protogen.Field) {
	// we check if it was processed or not
	if _, ok := g.processedOneofs[field.Oneof.GoIdent.String()]; ok {
		return
	}
	// we process it only if its != nil
	g.P("if x.", field.Oneof.GoName, " != nil {")
	g.P("switch o := x.", field.Oneof.GoName, ".(type) {")
	for _, oneofField := range field.Oneof.Fields {
		g.P("case *", g.QualifiedGoIdent(oneofField.GoIdent), ":")
		g.P("v := ", "o.", oneofField.GoName)
		switch oneofField.Desc.Kind() {
		case protoreflect.MessageKind:
			g.P("value := ", kindToValueConstructor(oneofField.Desc.Kind()), "(v.ProtoReflect())")
		case protoreflect.EnumKind:
			g.P("value :=", kindToValueConstructor(oneofField.Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(v))")
		default:
			g.P("value := ", kindToValueConstructor(oneofField.Desc.Kind()), "(v)")

		}
		g.P("if !f(", fieldDescriptorName(oneofField), ", value) {")
		g.P("return")
		g.P("}")

	}
	g.P("}")
	g.P("}")
	// add this as processed oneof
	g.processedOneofs[field.Oneof.GoIdent.String()] = struct{}{}
}
