package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"

	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type listGen struct {
	*generator.GeneratedFile
	field *protogen.Field

	typeName string
}

func (g *listGen) generate() {
	g.typeName = listTypeName(g.field)

	g.genAssertions()
	g.genType()
	g.genLen()
	g.genGet()
	g.genSet()
	g.genAppend()
	g.genAppendMutable()
	g.genTruncate()
	g.genNewElement()
	g.genIsValid()
}

// genAssertions generates protoreflect.List type assertions
func (g *listGen) genAssertions() {
	// type assertion
	g.P("var _ ", protoreflectPkg.Ident("List"), " = (*", g.typeName, ")(nil)")
}

// genType generates the list type
func (g *listGen) genType() {
	g.P("type ", g.typeName, " struct {")
	g.P("list *[]", getGoType(g.GeneratedFile, g.field))
	g.P("}")
	g.P()
}

// genLen generates the implementation for protoreflect.List.Len
func (g *listGen) genLen() {
	g.P("func (x *", g.typeName, ") Len() int {")
	g.P("if x.list == nil {")
	g.P("return 0")
	g.P("}")
	g.P("return len(*x.list)")
	g.P("}")
	g.P()
}

// genGet generates the implementation for protoreflect.List.Get
func (g *listGen) genGet() {
	g.P("func (x *", g.typeName, ") Get(i int) ", protoreflectPkg.Ident("Value"), " {")
	constructor := kindToValueConstructor(g.field.Desc.Kind())
	switch g.field.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", constructor, "((*x.list)[i].ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("return ", constructor, "((", protoreflectPkg.Ident("EnumNumber"), ")((*x.list)[i]))")
	default:
		g.P("return ", constructor, "((*x.list)[i])")
	}
	g.P("}")
	g.P()
}

// genSet generates the implementation for protoreflect.List.Set
func (g *listGen) genSet() {
	// Set
	g.P("func (x *", g.typeName, ") Set(i int, value ", protoreflectPkg.Ident("Value"), ") {")
	concreteValueName := genPrefValueToGoValue(g.GeneratedFile, g.field, "value", "concreteValue")
	g.P("(*x.list)[i] = ", concreteValueName)
	g.P("}")
	g.P()
}

// genAppend generates the protoreflect.List.Append implementation
func (g *listGen) genAppend() {
	g.P("func (x *", g.typeName, ") Append(value ", protoreflectPkg.Ident("Value"), ") {")
	concreteValueName := genPrefValueToGoValue(g.GeneratedFile, g.field, "value", "concreteValue")
	g.P("*x.list = append(*x.list, ", concreteValueName, ")")
	g.P("}")
	g.P()
}

// genAppendMutable generates the protoreflect.List.AppendMutable implementation
func (g *listGen) genAppendMutable() {
	g.P("func (x *", g.typeName, ") AppendMutable() ", protoreflectPkg.Ident("Value"), " {")
	switch g.field.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("v := new(", g.QualifiedGoIdent(g.field.Message.GoIdent), ")")
		g.P("*x.list = append(*x.list, v)")
		g.P("return ", protoreflectPkg.Ident("ValueOfMessage"), "(v.ProtoReflect())")
	default:
		panicMsg := fmt.Sprintf("AppendMutable can not be called on message %s at list field %s as it is not of Message kind", g.field.Parent.GoIdent.GoName, g.field.GoName)
		g.P("panic(", fmtPkg.Ident("Errorf"), "(\"", panicMsg, "\"))")
	}
	g.P("}")
	g.P()
}

// genTruncate generates the protoreflect.List.Truncate implementation
func (g *listGen) genTruncate() {
	g.P("func (x *", g.typeName, ") Truncate(n int)", "{")

	switch g.field.Desc.Kind() {
	case protoreflect.MessageKind: // zero message kinds to avoid keeping data alive
		g.P("for i := n; i < len(*x.list); i++ {")
		g.P("(*x.list)[i] = nil")
		g.P("}")
	}
	g.P("*x.list = (*x.list)[:n]") // truncate
	g.P("}")
	g.P()
}

// genNewElement generates the protoreflect.List.NewElement implementation
func (g *listGen) genNewElement() {
	g.P("func (x *", g.typeName, ") NewElement() ", protoreflectPkg.Ident("Value"), "{")
	switch g.field.Desc.Kind() {
	case protoreflect.BytesKind:
		g.P("var v []byte")
	default:
		zeroValue := zeroValueForField(g.GeneratedFile, g.field)
		g.P("v := ", zeroValue)
	}
	switch g.field.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P("return ", kindToValueConstructor(g.field.Desc.Kind()), "(v.ProtoReflect())")
	case protoreflect.EnumKind:
		g.P("return ", kindToValueConstructor(g.field.Desc.Kind()), "((", protoreflectPkg.Ident("EnumNumber"), ")(v))")
	default:
		g.P("return ", kindToValueConstructor(g.field.Desc.Kind()), "(v)")
	}
	g.P("}")
	g.P()
}

func (g *listGen) genIsValid() {
	g.P("func (x *", g.typeName, ") IsValid() bool {")
	g.P("return x.list != nil") // if we generate this type it's always valid, it either comes from Mutable path, or Get path in which the list is valid.
	g.P("}")
	g.P()
}

func listTypeName(field *protogen.Field) string {
	return fmt.Sprintf("_%s_%d_list", field.Parent.GoIdent.GoName, field.Desc.Number())
}

func getGoType(g *generator.GeneratedFile, field *protogen.Field) (goType string) {
	if field.Desc.IsWeak() {
		return "struct{}"
	}

	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		goType = "bool"
	case protoreflect.EnumKind:
		goType = g.QualifiedGoIdent(field.Enum.GoIdent)
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		goType = "int32"
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		goType = "uint32"
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		goType = "int64"
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		goType = "uint64"
	case protoreflect.FloatKind:
		goType = "float32"
	case protoreflect.DoubleKind:
		goType = "float64"
	case protoreflect.StringKind:
		goType = "string"
	case protoreflect.BytesKind:
		goType = "[]byte"
	case protoreflect.MessageKind, protoreflect.GroupKind:
		goType = "*" + g.QualifiedGoIdent(field.Message.GoIdent)
	}
	return goType
}

func kindToValueConstructor(kind protoreflect.Kind) protogen.GoIdent {
	switch kind {
	case protoreflect.BoolKind:
		return protoreflectPkg.Ident("ValueOfBool")
	case protoreflect.EnumKind:
		return protoreflectPkg.Ident("ValueOfEnum")
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		return protoreflectPkg.Ident("ValueOfInt32")
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		return protoreflectPkg.Ident("ValueOfUint32")
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		return protoreflectPkg.Ident("ValueOfInt64")
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		return protoreflectPkg.Ident("ValueOfUint64")
	case protoreflect.FloatKind:
		return protoreflectPkg.Ident("ValueOfFloat32")
	case protoreflect.DoubleKind:
		return protoreflectPkg.Ident("ValueOfFloat64")
	case protoreflect.StringKind:
		return protoreflectPkg.Ident("ValueOfString")
	case protoreflect.BytesKind:
		return protoreflectPkg.Ident("ValueOfBytes")
	case protoreflect.MessageKind, protoreflect.GroupKind:
		return protoreflectPkg.Ident("ValueOfMessage")
	default:
		panic("should not reach here")
	}
}

// valueUnwrapper provides the function to call on value
// in order to get the concrete underlying type
func valueUnwrapper(kind protoreflect.Kind) string {
	switch kind {
	case protoreflect.BoolKind:
		return "Bool"
	case protoreflect.EnumKind:
		return "Enum"
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		return "Int"
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		return "Uint"
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		return "Int"
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		return "Uint"
	case protoreflect.FloatKind:
		return "Float"
	case protoreflect.DoubleKind:
		return "Float"
	case protoreflect.StringKind:
		return "String"
	case protoreflect.BytesKind:
		return "Bytes"
	case protoreflect.MessageKind, protoreflect.GroupKind:
		return "Message"
	default:
		panic("should not reach here")
	}
}

func genPrefValueToGoValue(g *generator.GeneratedFile, field *protogen.Field, inputName string, outputName string) string {

	unwrapperFunc := valueUnwrapper(field.Desc.Kind())
	unwrapperVar := fmt.Sprintf("%sUnwrapped", inputName)
	g.P(unwrapperVar, " := ", inputName, ".", unwrapperFunc, "()")
	switch field.Desc.Kind() {
	case protoreflect.MessageKind:
		g.P(outputName, " := ", unwrapperVar, ".Interface().(*", g.QualifiedGoIdent(field.Message.GoIdent), ")")
	case protoreflect.EnumKind:
		g.P(outputName, " := (", g.QualifiedGoIdent(field.Enum.GoIdent), ")(", unwrapperVar, ")")
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		g.P(outputName, " := (int32)(", unwrapperVar, ")")
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		g.P(outputName, " := (uint32)(", unwrapperVar, ")")
	case protoreflect.FloatKind:
		g.P(outputName, " := (float32)(", unwrapperVar, ")")
	default:
		g.P(outputName, " := ", unwrapperVar)
	}

	return outputName
}

func zeroValueForField(g *generator.GeneratedFile, field *protogen.Field) string {
	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		return "false"
	case protoreflect.EnumKind:
		return fmt.Sprintf("%d", field.Enum.Desc.Values().Get(0).Number())
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		return "int32(0)"
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		return "uint32(0)"
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		return "int64(0)"
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		return "uint64(0)"
	case protoreflect.FloatKind:
		return "float32(0)"
	case protoreflect.DoubleKind:
		return "float64(0)"
	case protoreflect.StringKind:
		return "\"\""
	case protoreflect.BytesKind:
		return "nil"
	case protoreflect.MessageKind, protoreflect.GroupKind:
		return fmt.Sprintf("new(%s)", g.QualifiedGoIdent(field.Message.GoIdent))
	default:
		panic("should not reach")
	}
}
