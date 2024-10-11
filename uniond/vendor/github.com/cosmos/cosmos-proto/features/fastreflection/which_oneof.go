package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
)

type whichOneofGen struct {
	*generator.GeneratedFile
	typeName string
	message  *protogen.Message
}

func (g *whichOneofGen) generate() {
	g.genComment()
	g.genFunc()
}

func (g *whichOneofGen) genComment() {
	g.P("// WhichOneof reports which field within the oneof is populated,")
	g.P("// returning nil if none are populated.")
	g.P("// It panics if the oneof descriptor does not belong to this message.")
}

func (g *whichOneofGen) genFunc() {
	g.P("func (x *", g.typeName, ") WhichOneof(d ", protoreflectPkg.Ident("OneofDescriptor"), ") ", protoreflectPkg.Ident("FieldDescriptor"), " {")
	g.P("switch d.FullName() {")
	for _, oneof := range g.message.Oneofs {
		g.P("case \"", oneof.Desc.FullName(), "\": ")
		g.genOneof(oneof)
	}
	g.P("default: ")
	g.P("panic(", fmtPkg.Ident("Errorf"), "(\"%s is not a oneof field in ", g.message.Desc.FullName(), "\", d.FullName()))")
	g.P("}")

	// this part is unreachable
	g.P("panic(\"unreachable\")")
	g.P("}")
	g.P()
}

func (g *whichOneofGen) genOneof(oneof *protogen.Oneof) {
	// if none is populated then return nil
	g.P("if x.", oneof.GoName, " == nil {")
	g.P("return nil")
	g.P("}")
	// switch the type
	g.P("switch x.", oneof.GoName, ".(type) {")
	for _, field := range oneof.Fields {
		g.P("case *", g.QualifiedGoIdent(field.GoIdent), ":")
		g.P("return x.Descriptor().Fields().ByName(\"", field.Desc.Name(), "\")")
	}
	g.P("}")
}
