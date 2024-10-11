package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"

	"google.golang.org/protobuf/compiler/protogen"
)

func messageTypeName(message *protogen.Message) string {
	return fmt.Sprintf("%s_messageType", fastReflectionTypeName(message))
}

func messageTypeNameVar(message *protogen.Message) string {
	return fmt.Sprintf("_%s", messageTypeName(message))
}

type messageTypeGen struct {
	typeName string
	*generator.GeneratedFile
	message *protogen.Message

	messageTypeName string
	file            *protogen.File
}

func (g *messageTypeGen) generate() {
	g.messageTypeName = fmt.Sprintf("%s_messageType", g.typeName)

	g.P("var ", messageTypeNameVar(g.message), " ", g.messageTypeName)

	g.P("var _ ", protoreflectPkg.Ident("MessageType"), " = ", g.messageTypeName, "{}")
	g.P("type ", g.messageTypeName, " struct {}")

	g.P("func (x ", g.messageTypeName, ") Zero() ", protoreflectPkg.Ident("Message"), "{")
	g.P("return (*", g.typeName, ")(nil)")
	g.P("}")

	g.P("func (x ", g.messageTypeName, ") New() ", protoreflectPkg.Ident("Message"), "{")
	g.P("return new(", g.typeName, ")")
	g.P("}")

	g.P("func (x ", g.messageTypeName, ") Descriptor() ", protoreflectPkg.Ident("MessageDescriptor"), " {")
	g.P("return ", messageDescriptorName(g.message))
	g.P("}")
}

func messageDescriptorName(message *protogen.Message) string {
	return fmt.Sprintf("md_%s", message.GoIdent.GoName)
}
