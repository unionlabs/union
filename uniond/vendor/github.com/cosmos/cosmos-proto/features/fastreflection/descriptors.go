package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/reflect/protoreflect"

	"github.com/cosmos/cosmos-proto/features/fastreflection/copied"
	"google.golang.org/protobuf/compiler/protogen"
)

type descGen struct {
	*generator.GeneratedFile
	file    *protogen.File
	message *protogen.Message
}

func (g *descGen) generate() {
	// init is going to initialize descriptor information
	g.P("var (")
	g.P(messageDescriptorName(g.message), " ", protoreflectPkg.Ident("MessageDescriptor"))
	for _, field := range g.message.Fields {
		g.P(fieldDescriptorName(field), " ", protoreflectPkg.Ident("FieldDescriptor"))
	}
	g.P(")")
	g.P("func init() {")
	g.P(copied.InitFunctionName(g.file), "()")
	// we need to consider nested messages. so we try to recreate the hierarchy
	// ex messageA contains messageB contains messageC
	args := []interface{}{messageDescriptorName(g.message), " = ", g.file.GoDescriptorIdent.GoName}
	for _, md := range findParents(g.message.Desc) {
		args = append(args, ".Messages().ByName(\"", md.Name(), "\")")
	}
	g.P(args...)
	for _, field := range g.message.Fields {
		g.P(fieldDescriptorName(field), " = ", messageDescriptorName(g.message), ".Fields().ByName(\"", field.Desc.Name(), "\")")
	}
	g.P("}")
}

func fieldDescriptorName(field *protogen.Field) string {
	return fmt.Sprintf("fd_%s_%s", field.Parent.GoIdent.GoName, field.Desc.Name())
}

func findParents(message protoreflect.MessageDescriptor) []protoreflect.MessageDescriptor {
	parent, ok := message.Parent().(protoreflect.MessageDescriptor)
	if !ok {
		return []protoreflect.MessageDescriptor{message}
	}

	return append(findParents(parent), message)
}
