package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/features/fastreflection/copied"
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
)

const (
	protoreflectPkg = protogen.GoImportPath("google.golang.org/protobuf/reflect/protoreflect")
	protoifacePkg   = protogen.GoImportPath("google.golang.org/protobuf/runtime/protoiface")
	protoimplPkg    = protogen.GoImportPath("google.golang.org/protobuf/runtime/protoimpl")
	protoPkg        = protogen.GoImportPath("google.golang.org/protobuf/proto")

	sortPkg     = protogen.GoImportPath("sort")
	fmtPkg      = protogen.GoImportPath("fmt")
	mathPackage = protogen.GoImportPath("math")

	runtimePackage = protogen.GoImportPath("github.com/cosmos/cosmos-proto/runtime")
)

func GenProtoMessage(f *protogen.File, g *generator.GeneratedFile, message *protogen.Message) {
	genMessage(f, g, message)
	// check for message declarations within a message declaration
	for _, nested := range message.Messages {
		// map entries are defines as messages, but we don't want to generate those.
		if nested.Desc.IsMapEntry() {
			continue
		}
		GenProtoMessage(f, g, nested)
	}
}

func genMessage(f *protogen.File, g *generator.GeneratedFile, message *protogen.Message) {
	gen := newGenerator(f, g, message)
	gen.generateExtraTypes()
	gen.generateReflectionType()
	gen.genMessageType()
	gen.genDescriptor()
	gen.genType()
	gen.genNew()
	gen.genInterface()
	gen.genRange()
	gen.genHas()
	gen.genClear()
	gen.genGet()
	gen.genSet()
	gen.gentMutable()
	gen.genNewField()
	gen.genWhichOneof()
	gen.genGetUnknown()
	gen.genSetUnknown()
	gen.genIsValid()
	gen.genProtoMethods()
}

func fastReflectionTypeName(message *protogen.Message) string {
	return fmt.Sprintf("fastReflection_%s", message.GoIdent.GoName)
}

// generateExtraTypes generates the protoreflect.List and protoreflect.Map types required.
func (g *fastGenerator) generateExtraTypes() {
	for _, field := range g.message.Fields {
		switch {
		case field.Desc.IsMap():
			g.generateMapType(field)
		case field.Desc.IsList():
			g.generateListType(field)
		}
	}

	// generate descriptors
	(&descGen{
		GeneratedFile: g.GeneratedFile,
		file:          g.file,
		message:       g.message,
	}).generate()
}

// generateMapType generates the fastReflectionFeature reflection protoreflect.Map type
// related to the provided protogen.Field.
func (g *fastGenerator) generateMapType(field *protogen.Field) {
	(&mapGen{
		GeneratedFile: g.GeneratedFile,
		field:         field,
	}).generate()
}

// generateListType generates the fastReflectionFeature reflection protoreflect.List type
// related to the provided protogen.Field.
func (g *fastGenerator) generateListType(field *protogen.Field) {
	(&listGen{
		GeneratedFile: g.GeneratedFile,
		field:         field,
	}).generate()
}

func (g *fastGenerator) genMessageType() {
	(&messageTypeGen{
		typeName:        g.typeName,
		GeneratedFile:   g.GeneratedFile,
		message:         g.message,
		file:            g.file,
		messageTypeName: "",
	}).generate()
}

func (g *fastGenerator) generateReflectionType() {
	// gen interface assertion
	g.P("var _ ", protoreflectPkg.Ident("Message"), " = (*", g.typeName, ")(nil)")
	g.P()
	// gen type
	g.P("type ", g.typeName, " ", g.message.GoIdent.GoName)
	// gen msg implementation
	g.P("func (x *", g.message.GoIdent.GoName, ") ProtoReflect() ", protoreflectPkg.Ident("Message"), "{")
	g.P("return (*", g.typeName, ")(x)")
	g.P("}")
	g.P()

	// gen slowreflection
	f := copied.NewFileInfo(g.file)
	idx := func() int {
		var id int
		var found bool
		for mInfo, index := range f.AllMessagesByPtr {
			if mInfo.Message.Desc.FullName() == g.message.Desc.FullName() {
				id = index
				found = true
			}
		}
		if !found {
			panic("not found")
		}
		return id
	}()
	typesVar := copied.MessageTypesVarName(f)

	// ProtoReflect method.
	g.P("func (x *", g.message.GoIdent, ") slowProtoReflect() ", protoreflectPkg.Ident("Message"), " {")
	g.P("mi := &", typesVar, "[", idx, "]")
	g.P("if ", protoimplPkg.Ident("UnsafeEnabled"), " && x != nil {")
	g.P("ms := ", protoimplPkg.Ident("X"), ".MessageStateOf(", protoimplPkg.Ident("Pointer"), "(x))")
	g.P("if ms.LoadMessageInfo() == nil {")
	g.P("ms.StoreMessageInfo(mi)")
	g.P("}")
	g.P("return ms")
	g.P("}")
	g.P("return mi.MessageOf(x)")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genDescriptor() {
	g.P("// Descriptor returns message descriptor, which contains only the protobuf")
	g.P("// type information for the message.")
	g.P("func (x *", g.typeName, ") Descriptor() ", protoreflectPkg.Ident("MessageDescriptor"), " {")
	g.P("return ", messageDescriptorName(g.message))
	g.P("}")
	g.P()
}

func (g *fastGenerator) genType() {
	g.P("// Type returns the message type, which encapsulates both Go and protobuf")
	g.P("// type information. If the Go type information is not needed,")
	g.P("// it is recommended that the message descriptor be used instead.")
	g.P("func (x *", g.typeName, ") Type() ", protoreflectPkg.Ident("MessageType"), " {")
	g.P("return ", messageTypeNameVar(g.message))
	g.P("}")
	g.P()
}

func (g *fastGenerator) genNew() {
	g.P("// New returns a newly allocated and mutable empty message.")
	g.P("func (x *", g.typeName, ") New() ", protoreflectPkg.Ident("Message"), " {")
	g.P("return new(", g.typeName, ")")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genInterface() {
	g.P("// Interface unwraps the message reflection interface and")
	g.P("// returns the underlying ProtoMessage interface.")
	g.P("func (x *", g.typeName, ") Interface() ", protoreflectPkg.Ident("ProtoMessage"), " {")
	g.P("return (*", g.message.GoIdent, ")(x)")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genRange() {
	(&rangeGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
	g.P()
}

func (g *fastGenerator) genHas() {
	(&hasGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
}

func (g *fastGenerator) genClear() {
	(&clearGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()

	g.P()
}

func (g *fastGenerator) genSet() {
	(&setGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
}

func (g *fastGenerator) gentMutable() {
	(&mutableGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
}

func (g *fastGenerator) genNewField() {
	(&newFieldGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
	g.P()
}

func (g *fastGenerator) genWhichOneof() {
	(&whichOneofGen{
		GeneratedFile: g.GeneratedFile,
		typeName:      g.typeName,
		message:       g.message,
	}).generate()
}

func (g *fastGenerator) genGetUnknown() {
	g.P("// GetUnknown retrieves the entire list of unknown fields.")
	g.P("// The caller may only mutate the contents of the RawFields")
	g.P("// if the mutated bytes are stored back into the message with SetUnknown.")
	g.P("func (x *", g.typeName, ") GetUnknown() ", protoreflectPkg.Ident("RawFields"), " {")
	g.P("return x.unknownFields")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genSetUnknown() {
	g.P("// SetUnknown stores an entire list of unknown fields.")
	g.P("// The raw fields must be syntactically valid according to the wire format.")
	g.P("// An implementation may panic if this is not the case.")
	g.P("// Once stored, the caller must not mutate the content of the RawFields.")
	g.P("// An empty RawFields may be passed to clear the fields.")
	g.P("//")
	g.P("// SetUnknown is a mutating operation and unsafe for concurrent use.")
	g.P("func (x *", g.typeName, ") SetUnknown(fields ", protoreflectPkg.Ident("RawFields"), ") {")
	g.P("x.unknownFields = fields")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genIsValid() {
	g.P("// IsValid reports whether the message is valid.")
	g.P("//")
	g.P("// An invalid message is an empty, read-only value.")
	g.P("//")
	g.P("// An invalid message often corresponds to a nil pointer of the concrete")
	g.P("// message type, but the details are implementation dependent.")
	g.P("// Validity is not part of the protobuf data model, and may not")
	g.P("// be preserved in marshaling or other operations.")

	g.P("func (x *", g.typeName, ") IsValid() bool {")
	g.P("return x != nil")
	g.P("}")
	g.P()
}

func (g *fastGenerator) genProtoMethods() {
	g.P("// ProtoMethods returns optional fastReflectionFeature-path implementations of various operations.")
	g.P("// This method may return nil.")
	g.P("//")
	g.P("// The returned methods type is identical to")
	g.P(`// "google.golang.org/protobuf/runtime/protoiface".Methods.`)
	g.P("// Consult the protoiface package documentation for details.")
	g.P("func (x *", g.typeName, ") ProtoMethods() *", protoifacePkg.Ident("Methods"), " {")

	g.genSizeMethod()
	g.genMarshalMethod()
	g.genUnmarshalMethod()

	g.P("return &", protoifacePkg.Ident("Methods"), "{ ")
	g.P("NoUnkeyedLiterals: struct{}{},")
	g.P("Flags: ", protoifacePkg.Ident("SupportMarshalDeterministic"), "|", protoifacePkg.Ident("SupportUnmarshalDiscardUnknown"), ",")
	g.P("Size: size,")
	g.P("Marshal: marshal,")
	g.P("Unmarshal: unmarshal,")
	g.P("Merge: nil,")
	g.P("CheckInitialized: nil,")
	g.P("}")
	g.P("}")
}
