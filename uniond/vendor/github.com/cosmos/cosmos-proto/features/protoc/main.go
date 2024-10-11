// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package protoc is used to generate protoc-gen-go files
package protoc

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/features/protoc/genid"
	"github.com/cosmos/cosmos-proto/generator"
	"go/ast"
	"go/parser"
	"go/token"
	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/proto"
	"math"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"

	pref "google.golang.org/protobuf/reflect/protoreflect"

	"github.com/cosmos/cosmos-proto/features/protoc/version"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/runtime/protoimpl"

	"google.golang.org/protobuf/types/descriptorpb"
	"google.golang.org/protobuf/types/pluginpb"
)

// SupportedFeatures reports the set of supported protobuf language features.
var SupportedFeatures = uint64(pluginpb.CodeGeneratorResponse_FEATURE_PROTO3_OPTIONAL)

// GenerateVersionMarkers specifies whether to generate version markers.
var GenerateVersionMarkers = true

// Standard library dependencies.
const (
	base64Package  = protogen.GoImportPath("encoding/base64")
	mathPackage    = protogen.GoImportPath("math")
	reflectPackage = protogen.GoImportPath("reflect")
	sortPackage    = protogen.GoImportPath("sort")
	stringsPackage = protogen.GoImportPath("strings")
	syncPackage    = protogen.GoImportPath("sync")
	timePackage    = protogen.GoImportPath("time")
	utf8Package    = protogen.GoImportPath("unicode/utf8")
)

// Protobuf library dependencies.
//
// These are declared as an interface type so that they can be more easily
// patched to support unique build environments that impose restrictions
// on the dependencies of generated source code.
var (
	protoPackage         goImportPath = protogen.GoImportPath("google.golang.org/protobuf/proto")
	protoimplPackage     goImportPath = protogen.GoImportPath("google.golang.org/protobuf/runtime/protoimpl")
	protojsonPackage     goImportPath = protogen.GoImportPath("google.golang.org/protobuf/encoding/protojson")
	protoreflectPackage  goImportPath = protogen.GoImportPath("google.golang.org/protobuf/reflect/protoreflect")
	protoregistryPackage goImportPath = protogen.GoImportPath("google.golang.org/protobuf/reflect/protoregistry")
)

// GenerateFile generates the contents of a .pb.go file.
func GenerateFile(gen *protogen.Plugin, file *protogen.File, g *generator.GeneratedFile) *generator.GeneratedFile {
	// filename := file.GeneratedFilenamePrefix + ".pb.go"
	// g := gen.NewGeneratedFile(filename, file.GoImportPath)
	f := newFileInfo(file)

	genStandaloneComments(g, f, int32(genid.FileDescriptorProto_Syntax_field_number))
	genGeneratedHeader(gen, g, f)
	genStandaloneComments(g, f, int32(genid.FileDescriptorProto_Package_field_number))

	// Emit a static check that enforces a minimum version of the proto package.
	if GenerateVersionMarkers {
		g.P("const (")
		g.P("// Verify that this generated code is sufficiently up-to-date.")
		g.P("_ = ", protoimplPackage.Ident("EnforceVersion"), "(", protoimpl.GenVersion, " - ", protoimplPackage.Ident("MinVersion"), ")")
		g.P("// Verify that runtime/protoimpl is sufficiently up-to-date.")
		g.P("_ = ", protoimplPackage.Ident("EnforceVersion"), "(", protoimplPackage.Ident("MaxVersion"), " - ", protoimpl.GenVersion, ")")
		g.P(")")
		g.P()
	}

	for i, imps := 0, f.Desc.Imports(); i < imps.Len(); i++ {
		genImport(gen, g, f, imps.Get(i))
	}
	for _, enum := range f.allEnums {
		genEnum(g, f, enum)
	}
	for _, message := range f.allMessages {
		genMessage(g, f, message)
	}
	genExtensions(g, f)

	genReflectFileDescriptor(gen, g, f)

	return g
}

func fileVarName(f *protogen.File, suffix string) string {
	prefix := f.GoDescriptorIdent.GoName
	_, n := utf8.DecodeRuneInString(prefix)
	prefix = strings.ToLower(prefix[:n]) + prefix[n:]
	return prefix + "_" + suffix
}

func rawDescVarName(f *fileInfo) string {
	return fileVarName(f.File, "rawDesc")
}
func goTypesVarName(f *fileInfo) string {
	return fileVarName(f.File, "goTypes")
}
func depIdxsVarName(f *fileInfo) string {
	return fileVarName(f.File, "depIdxs")
}
func enumTypesVarName(f *fileInfo) string {
	return fileVarName(f.File, "enumTypes")
}
func messageTypesVarName(f *fileInfo) string {
	return fileVarName(f.File, "msgTypes")
}
func extensionTypesVarName(f *fileInfo) string {
	return fileVarName(f.File, "extTypes")
}
func initFuncName(f *protogen.File) string {
	return fileVarName(f, "init")
}

func genFileDescriptor(gen *protogen.Plugin, g *generator.GeneratedFile, f *fileInfo) {
	descProto := proto.Clone(f.Proto).(*descriptorpb.FileDescriptorProto)
	descProto.SourceCodeInfo = nil // drop source code information

	b, err := proto.MarshalOptions{AllowPartial: true, Deterministic: true}.Marshal(descProto)
	if err != nil {
		gen.Error(err)
		return
	}

	g.P("var ", rawDescVarName(f), " = []byte{")
	for len(b) > 0 {
		n := 16
		if n > len(b) {
			n = len(b)
		}

		s := ""
		for _, c := range b[:n] {
			s += fmt.Sprintf("0x%02x,", c)
		}
		g.P(s)

		b = b[n:]
	}
	g.P("}")
	g.P()

	if f.needRawDesc {
		onceVar := rawDescVarName(f) + "Once"
		dataVar := rawDescVarName(f) + "Data"
		g.P("var (")
		g.P(onceVar, " ", syncPackage.Ident("Once"))
		g.P(dataVar, " = ", rawDescVarName(f))
		g.P(")")
		g.P()

		g.P("func ", rawDescVarName(f), "GZIP() []byte {")
		g.P(onceVar, ".Do(func() {")
		g.P(dataVar, " = ", protoimplPackage.Ident("X"), ".CompressGZIP(", dataVar, ")")
		g.P("})")
		g.P("return ", dataVar)
		g.P("}")
		g.P()
	}
}

func genReflectFileDescriptor(gen *protogen.Plugin, g *generator.GeneratedFile, f *fileInfo) {
	g.P("var ", f.GoDescriptorIdent, " ", protoreflectPackage.Ident("FileDescriptor"))
	g.P()

	genFileDescriptor(gen, g, f)
	if len(f.allEnums) > 0 {
		g.P("var ", enumTypesVarName(f), " = make([]", protoimplPackage.Ident("EnumInfo"), ",", len(f.allEnums), ")")
	}
	if len(f.allMessages) > 0 {
		g.P("var ", messageTypesVarName(f), " = make([]", protoimplPackage.Ident("MessageInfo"), ",", len(f.allMessages), ")")
	}

	// Generate a unique list of Go types for all declarations and dependencies,
	// and the associated index into the type list for all dependencies.
	var goTypes []string
	var depIdxs []string
	seen := map[protoreflect.FullName]int{}
	genDep := func(name protoreflect.FullName, depSource string) {
		if depSource != "" {
			line := fmt.Sprintf("%d, // %d: %s -> %s", seen[name], len(depIdxs), depSource, name)
			depIdxs = append(depIdxs, line)
		}
	}
	genEnum := func(e *protogen.Enum, depSource string) {
		if e != nil {
			name := e.Desc.FullName()
			if _, ok := seen[name]; !ok {
				line := fmt.Sprintf("(%s)(0), // %d: %s", g.QualifiedGoIdent(e.GoIdent), len(goTypes), name)
				goTypes = append(goTypes, line)
				seen[name] = len(seen)
			}
			if depSource != "" {
				genDep(name, depSource)
			}
		}
	}
	genMessage := func(m *protogen.Message, depSource string) {
		if m != nil {
			name := m.Desc.FullName()
			if _, ok := seen[name]; !ok {
				line := fmt.Sprintf("(*%s)(nil), // %d: %s", g.QualifiedGoIdent(m.GoIdent), len(goTypes), name)
				if m.Desc.IsMapEntry() {
					// Map entry messages have no associated Go type.
					line = fmt.Sprintf("nil, // %d: %s", len(goTypes), name)
				}
				goTypes = append(goTypes, line)
				seen[name] = len(seen)
			}
			if depSource != "" {
				genDep(name, depSource)
			}
		}
	}

	// This ordering is significant.
	// See filetype.TypeBuilder.DependencyIndexes.
	type offsetEntry struct {
		start int
		name  string
	}
	var depOffsets []offsetEntry
	for _, enum := range f.allEnums {
		genEnum(enum.Enum, "")
	}
	for _, message := range f.allMessages {
		genMessage(message.Message, "")
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), "field type_name"})
	for _, message := range f.allMessages {
		for _, field := range message.Fields {
			if field.Desc.IsWeak() {
				continue
			}
			source := string(field.Desc.FullName())
			genEnum(field.Enum, source+":type_name")
			genMessage(field.Message, source+":type_name")
		}
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), "extension extendee"})
	for _, extension := range f.allExtensions {
		source := string(extension.Desc.FullName())
		genMessage(extension.Extendee, source+":extendee")
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), "extension type_name"})
	for _, extension := range f.allExtensions {
		source := string(extension.Desc.FullName())
		genEnum(extension.Enum, source+":type_name")
		genMessage(extension.Message, source+":type_name")
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), "method input_type"})
	for _, service := range f.Services {
		for _, method := range service.Methods {
			source := string(method.Desc.FullName())
			genMessage(method.Input, source+":input_type")
		}
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), "method output_type"})
	for _, service := range f.Services {
		for _, method := range service.Methods {
			source := string(method.Desc.FullName())
			genMessage(method.Output, source+":output_type")
		}
	}
	depOffsets = append(depOffsets, offsetEntry{len(depIdxs), ""})
	for i := len(depOffsets) - 2; i >= 0; i-- {
		curr, next := depOffsets[i], depOffsets[i+1]
		depIdxs = append(depIdxs, fmt.Sprintf("%d, // [%d:%d] is the sub-list for %s",
			curr.start, curr.start, next.start, curr.name))
	}
	if len(depIdxs) > math.MaxInt32 {
		panic("too many dependencies") // sanity check
	}

	g.P("var ", goTypesVarName(f), " = []interface{}{")
	for _, s := range goTypes {
		g.P(s)
	}
	g.P("}")

	g.P("var ", depIdxsVarName(f), " = []int32{")
	for _, s := range depIdxs {
		g.P(s)
	}
	g.P("}")

	g.P("func init() { ", initFuncName(f.File), "() }")

	g.P("func ", initFuncName(f.File), "() {")
	g.P("if ", f.GoDescriptorIdent, " != nil {")
	g.P("return")
	g.P("}")

	// Ensure that initialization functions for different files in the same Go
	// package run in the correct order: Call the init funcs for every .proto file
	// imported by this one that is in the same Go package.
	for i, imps := 0, f.Desc.Imports(); i < imps.Len(); i++ {
		impFile := gen.FilesByPath[imps.Get(i).Path()]
		if impFile.GoImportPath != f.GoImportPath {
			continue
		}
		g.P(initFuncName(impFile), "()")
	}

	if len(f.allMessages) > 0 {
		// Populate MessageInfo.Exporters.
		g.P("if !", protoimplPackage.Ident("UnsafeEnabled"), " {")
		for _, message := range f.allMessages {
			if sf := f.allMessageFieldsByPtr[message]; len(sf.unexported) > 0 {
				idx := f.allMessagesByPtr[message]
				typesVar := messageTypesVarName(f)

				g.P(typesVar, "[", idx, "].Exporter = func(v interface{}, i int) interface{} {")
				g.P("switch v := v.(*", message.GoIdent, "); i {")
				for i := 0; i < sf.count; i++ {
					if name := sf.unexported[i]; name != "" {
						g.P("case ", i, ": return &v.", name)
					}
				}
				g.P("default: return nil")
				g.P("}")
				g.P("}")
			}
		}
		g.P("}")

		// Populate MessageInfo.OneofWrappers.
		for _, message := range f.allMessages {
			if len(message.Oneofs) > 0 {
				idx := f.allMessagesByPtr[message]
				typesVar := messageTypesVarName(f)

				// Associate the wrapper types by directly passing them to the MessageInfo.
				g.P(typesVar, "[", idx, "].OneofWrappers = []interface{} {")
				for _, oneof := range message.Oneofs {
					if !oneof.Desc.IsSynthetic() {
						for _, field := range oneof.Fields {
							g.P("(*", field.GoIdent, ")(nil),")
						}
					}
				}
				g.P("}")
			}
		}
	}

	g.P("type x struct{}")
	g.P("out := ", protoimplPackage.Ident("TypeBuilder"), "{")
	g.P("File: ", protoimplPackage.Ident("DescBuilder"), "{")
	g.P("GoPackagePath: ", reflectPackage.Ident("TypeOf"), "(x{}).PkgPath(),")
	g.P("RawDescriptor: ", rawDescVarName(f), ",")
	g.P("NumEnums: ", len(f.allEnums), ",")
	g.P("NumMessages: ", len(f.allMessages), ",")
	g.P("NumExtensions: ", len(f.allExtensions), ",")
	g.P("NumServices: ", len(f.Services), ",")
	g.P("},")
	g.P("GoTypes: ", goTypesVarName(f), ",")
	g.P("DependencyIndexes: ", depIdxsVarName(f), ",")
	if len(f.allEnums) > 0 {
		g.P("EnumInfos: ", enumTypesVarName(f), ",")
	}
	if len(f.allMessages) > 0 {
		g.P("MessageInfos: ", messageTypesVarName(f), ",")
	}
	if len(f.allExtensions) > 0 {
		g.P("ExtensionInfos: ", extensionTypesVarName(f), ",")
	}
	g.P("}.Build()")
	g.P(f.GoDescriptorIdent, " = out.File")

	// Set inputs to nil to allow GC to reclaim resources.
	g.P(rawDescVarName(f), " = nil")
	g.P(goTypesVarName(f), " = nil")
	g.P(depIdxsVarName(f), " = nil")
	g.P("}")
}

// genStandaloneComments prints all leading comments for a FileDescriptorProto
// location identified by the field number n.
func genStandaloneComments(g *generator.GeneratedFile, f *fileInfo, n int32) {
	loc := f.Desc.SourceLocations().ByPath(protoreflect.SourcePath{n})
	for _, s := range loc.LeadingDetachedComments {
		g.P(protogen.Comments(s))
		g.P()
	}
	if s := loc.LeadingComments; s != "" {
		g.P(protogen.Comments(s))
		g.P()
	}
}

func genGeneratedHeader(gen *protogen.Plugin, g *generator.GeneratedFile, f *fileInfo) {
	g.P("// Code generated by protoc-gen-go. DO NOT EDIT.")

	if GenerateVersionMarkers {
		g.P("// versions:")
		protocGenGoVersion := version.String()
		protocVersion := "(unknown)"
		if v := gen.Request.GetCompilerVersion(); v != nil {
			protocVersion = fmt.Sprintf("v%v.%v.%v", v.GetMajor(), v.GetMinor(), v.GetPatch())
			if s := v.GetSuffix(); s != "" {
				protocVersion += "-" + s
			}
		}
		g.P("// \tprotoc-gen-go ", protocGenGoVersion)
		g.P("// \tprotoc        ", protocVersion)
	}

	if f.Proto.GetOptions().GetDeprecated() {
		g.P("// ", f.Desc.Path(), " is a deprecated file.")
	} else {
		g.P("// source: ", f.Desc.Path())
	}
	g.P()
}

func genImport(gen *protogen.Plugin, g *generator.GeneratedFile, f *fileInfo, imp protoreflect.FileImport) {
	impFile, ok := gen.FilesByPath[imp.Path()]
	if !ok {
		return
	}
	if impFile.GoImportPath == f.GoImportPath {
		// Don't generate imports or aliases for types in the same Go package.
		return
	}
	// Generate imports for all non-weak dependencies, even if they are not
	// referenced, because other code and tools depend on having the
	// full transitive closure of protocol buffer types in the binary.
	if !imp.IsWeak {
		g.Import(impFile.GoImportPath)
	}
	if !imp.IsPublic {
		return
	}

	// Generate public imports by generating the imported file, parsing it,
	// and extracting every symbol that should receive a forwarding declaration.
	impGen := GenerateFile(gen, impFile, g)
	impGen.Skip()
	b, err := impGen.Content()
	if err != nil {
		gen.Error(err)
		return
	}
	fset := token.NewFileSet()
	astFile, err := parser.ParseFile(fset, "", b, parser.ParseComments)
	if err != nil {
		gen.Error(err)
		return
	}
	genForward := func(tok token.Token, name string, expr ast.Expr) {
		// Don't import unexported symbols.
		r, _ := utf8.DecodeRuneInString(name)
		if !unicode.IsUpper(r) {
			return
		}
		// Don't import the FileDescriptor.
		if name == impFile.GoDescriptorIdent.GoName {
			return
		}
		// Don't import decls referencing a symbol defined in another package.
		// i.e., don't import decls which are themselves public imports:
		//
		//	type T = somepackage.T
		if _, ok := expr.(*ast.SelectorExpr); ok {
			return
		}
		g.P(tok, " ", name, " = ", impFile.GoImportPath.Ident(name))
	}
	g.P("// Symbols defined in public import of ", imp.Path(), ".")
	g.P()
	for _, decl := range astFile.Decls {
		switch decl := decl.(type) {
		case *ast.GenDecl:
			for _, spec := range decl.Specs {
				switch spec := spec.(type) {
				case *ast.TypeSpec:
					genForward(decl.Tok, spec.Name.Name, spec.Type)
				case *ast.ValueSpec:
					for i, name := range spec.Names {
						var expr ast.Expr
						if i < len(spec.Values) {
							expr = spec.Values[i]
						}
						genForward(decl.Tok, name.Name, expr)
					}
				case *ast.ImportSpec:
				default:
					panic(fmt.Sprintf("can't generate forward for spec type %T", spec))
				}
			}
		}
	}
	g.P()
}

func genEnumReflectMethods(g *generator.GeneratedFile, f *fileInfo, e *enumInfo) {
	idx := f.allEnumsByPtr[e]
	typesVar := enumTypesVarName(f)

	// Descriptor method.
	g.P("func (", e.GoIdent, ") Descriptor() ", protoreflectPackage.Ident("EnumDescriptor"), " {")
	g.P("return ", typesVar, "[", idx, "].Descriptor()")
	g.P("}")
	g.P()

	// Type method.
	g.P("func (", e.GoIdent, ") Type() ", protoreflectPackage.Ident("EnumType"), " {")
	g.P("return &", typesVar, "[", idx, "]")
	g.P("}")
	g.P()

	// Number method.
	g.P("func (x ", e.GoIdent, ") Number() ", protoreflectPackage.Ident("EnumNumber"), " {")
	g.P("return ", protoreflectPackage.Ident("EnumNumber"), "(x)")
	g.P("}")
	g.P()
}

func genEnum(g *generator.GeneratedFile, f *fileInfo, e *enumInfo) {
	// Enum type declaration.
	g.Annotate(e.GoIdent.GoName, e.Location)
	leadingComments := appendDeprecationSuffix(e.Comments.Leading,
		e.Desc.Options().(*descriptorpb.EnumOptions).GetDeprecated())
	g.P(leadingComments,
		"type ", e.GoIdent, " int32")

	// Enum value constants.
	g.P("const (")
	for _, value := range e.Values {
		g.Annotate(value.GoIdent.GoName, value.Location)
		leadingComments := appendDeprecationSuffix(value.Comments.Leading,
			value.Desc.Options().(*descriptorpb.EnumValueOptions).GetDeprecated())
		g.P(leadingComments,
			value.GoIdent, " ", e.GoIdent, " = ", value.Desc.Number(),
			trailingComment(value.Comments.Trailing))
	}
	g.P(")")
	g.P()

	// Enum value maps.
	g.P("// Enum value maps for ", e.GoIdent, ".")
	g.P("var (")
	g.P(e.GoIdent.GoName+"_name", " = map[int32]string{")
	for _, value := range e.Values {
		duplicate := ""
		if value.Desc != e.Desc.Values().ByNumber(value.Desc.Number()) {
			duplicate = "// Duplicate value: "
		}
		g.P(duplicate, value.Desc.Number(), ": ", strconv.Quote(string(value.Desc.Name())), ",")
	}
	g.P("}")
	g.P(e.GoIdent.GoName+"_value", " = map[string]int32{")
	for _, value := range e.Values {
		g.P(strconv.Quote(string(value.Desc.Name())), ": ", value.Desc.Number(), ",")
	}
	g.P("}")
	g.P(")")
	g.P()

	// Enum method.
	//
	// NOTE: A pointer value is needed to represent presence in proto2.
	// Since a proto2 message can reference a proto3 enum, it is useful to
	// always generate this method (even on proto3 enums) to support that case.
	g.P("func (x ", e.GoIdent, ") Enum() *", e.GoIdent, " {")
	g.P("p := new(", e.GoIdent, ")")
	g.P("*p = x")
	g.P("return p")
	g.P("}")
	g.P()

	// String method.
	g.P("func (x ", e.GoIdent, ") String() string {")
	g.P("return ", protoimplPackage.Ident("X"), ".EnumStringOf(x.Descriptor(), ", protoreflectPackage.Ident("EnumNumber"), "(x))")
	g.P("}")
	g.P()

	genEnumReflectMethods(g, f, e)

	// UnmarshalJSON method.
	if e.genJSONMethod && e.Desc.Syntax() == protoreflect.Proto2 {
		g.P("// Deprecated: Do not use.")
		g.P("func (x *", e.GoIdent, ") UnmarshalJSON(b []byte) error {")
		g.P("num, err := ", protoimplPackage.Ident("X"), ".UnmarshalJSONEnum(x.Descriptor(), b)")
		g.P("if err != nil {")
		g.P("return err")
		g.P("}")
		g.P("*x = ", e.GoIdent, "(num)")
		g.P("return nil")
		g.P("}")
		g.P()
	}

	// EnumDescriptor method.
	if e.genRawDescMethod {
		var indexes []string
		for i := 1; i < len(e.Location.Path); i += 2 {
			indexes = append(indexes, strconv.Itoa(int(e.Location.Path[i])))
		}
		g.P("// Deprecated: Use ", e.GoIdent, ".Descriptor instead.")
		g.P("func (", e.GoIdent, ") EnumDescriptor() ([]byte, []int) {")
		g.P("return ", rawDescVarName(f), "GZIP(), []int{", strings.Join(indexes, ","), "}")
		g.P("}")
		g.P()
		f.needRawDesc = true
	}
}

func genMessage(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	if m.Desc.IsMapEntry() {
		return
	}

	// Message type declaration.
	g.Annotate(m.GoIdent.GoName, m.Location)
	leadingComments := appendDeprecationSuffix(m.Comments.Leading,
		m.Desc.Options().(*descriptorpb.MessageOptions).GetDeprecated())
	g.P(leadingComments,
		"type ", m.GoIdent, " struct {")
	genMessageFields(g, f, m)
	g.P("}")
	g.P()

	genMessageKnownFunctions(g, f, m)
	genMessageDefaultDecls(g, f, m)
	genMessageMethods(g, f, m)
	genMessageOneofWrapperTypes(g, f, m)
}

func genMessageKnownFunctions(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	switch m.Desc.FullName() {
	case genid.Any_message_fullname:
		g.P("// New marshals src into a new Any instance.")
		g.P("func New(src ", protoPackage.Ident("Message"), ") (*Any, error) {")
		g.P("	dst := new(Any)")
		g.P("	if err := dst.MarshalFrom(src); err != nil {")
		g.P("		return nil, err")
		g.P("	}")
		g.P("	return dst, nil")
		g.P("}")
		g.P()

		g.P("// MarshalFrom marshals src into dst as the underlying message")
		g.P("// using the provided marshal options.")
		g.P("//")
		g.P("// If no options are specified, call dst.MarshalFrom instead.")
		g.P("func MarshalFrom(dst *Any, src ", protoPackage.Ident("Message"), ", opts ", protoPackage.Ident("MarshalOptions"), ") error {")
		g.P("	const urlPrefix = \"type.googleapis.com/\"")
		g.P("	if src == nil {")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"invalid nil source message\")")
		g.P("	}")
		g.P("	b, err := opts.Marshal(src)")
		g.P("	if err != nil {")
		g.P("		return err")
		g.P("	}")
		g.P("	dst.TypeUrl = urlPrefix + string(src.ProtoReflect().Descriptor().FullName())")
		g.P("	dst.Value = b")
		g.P("	return nil")
		g.P("}")
		g.P()

		g.P("// UnmarshalTo unmarshals the underlying message from src into dst")
		g.P("// using the provided unmarshal options.")
		g.P("// It reports an error if dst is not of the right message type.")
		g.P("//")
		g.P("// If no options are specified, call src.UnmarshalTo instead.")
		g.P("func UnmarshalTo(src *Any, dst ", protoPackage.Ident("Message"), ", opts ", protoPackage.Ident("UnmarshalOptions"), ") error {")
		g.P("	if src == nil {")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"invalid nil source message\")")
		g.P("	}")
		g.P("	if !src.MessageIs(dst) {")
		g.P("		got := dst.ProtoReflect().Descriptor().FullName()")
		g.P("		want := src.MessageName()")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"mismatched message type: got %q, want %q\", got, want)")
		g.P("	}")
		g.P("	return opts.Unmarshal(src.GetValue(), dst)")
		g.P("}")
		g.P()

		g.P("// UnmarshalNew unmarshals the underlying message from src into dst,")
		g.P("// which is newly created message using a type resolved from the type URL.")
		g.P("// The message type is resolved according to opt.Resolver,")
		g.P("// which should implement protoregistry.MessageTypeResolver.")
		g.P("// It reports an error if the underlying message type could not be resolved.")
		g.P("//")
		g.P("// If no options are specified, call src.UnmarshalNew instead.")
		g.P("func UnmarshalNew(src *Any, opts ", protoPackage.Ident("UnmarshalOptions"), ") (dst ", protoPackage.Ident("Message"), ", err error) {")
		g.P("	if src.GetTypeUrl() == \"\" {")
		g.P("		return nil, ", protoimplPackage.Ident("X"), ".NewError(\"invalid empty type URL\")")
		g.P("	}")
		g.P("	if opts.Resolver == nil {")
		g.P("		opts.Resolver = ", protoregistryPackage.Ident("GlobalTypes"))
		g.P("	}")
		g.P("	r, ok := opts.Resolver.(", protoregistryPackage.Ident("MessageTypeResolver"), ")")
		g.P("	if !ok {")
		g.P("		return nil, ", protoregistryPackage.Ident("NotFound"))
		g.P("	}")
		g.P("	mt, err := r.FindMessageByURL(src.GetTypeUrl())")
		g.P("	if err != nil {")
		g.P("		if err == ", protoregistryPackage.Ident("NotFound"), " {")
		g.P("			return nil, err")
		g.P("		}")
		g.P("		return nil, ", protoimplPackage.Ident("X"), ".NewError(\"could not resolve %q: %v\", src.GetTypeUrl(), err)")
		g.P("	}")
		g.P("	dst = mt.New().Interface()")
		g.P("	return dst, opts.Unmarshal(src.GetValue(), dst)")
		g.P("}")
		g.P()

		g.P("// MessageIs reports whether the underlying message is of the same type as m.")
		g.P("func (x *Any) MessageIs(m ", protoPackage.Ident("Message"), ") bool {")
		g.P("	if m == nil {")
		g.P("		return false")
		g.P("	}")
		g.P("	url := x.GetTypeUrl()")
		g.P("	name := string(m.ProtoReflect().Descriptor().FullName())")
		g.P("	if !", stringsPackage.Ident("HasSuffix"), "(url, name) {")
		g.P("		return false")
		g.P("	}")
		g.P("	return len(url) == len(name) || url[len(url)-len(name)-1] == '/'")
		g.P("}")
		g.P()

		g.P("// MessageName reports the full name of the underlying message,")
		g.P("// returning an empty string if invalid.")
		g.P("func (x *Any) MessageName() ", protoreflectPackage.Ident("FullName"), " {")
		g.P("	url := x.GetTypeUrl()")
		g.P("	name := ", protoreflectPackage.Ident("FullName"), "(url)")
		g.P("	if i := ", stringsPackage.Ident("LastIndexByte"), "(url, '/'); i >= 0 {")
		g.P("		name = name[i+len(\"/\"):]")
		g.P("	}")
		g.P("	if !name.IsValid() {")
		g.P("		return \"\"")
		g.P("	}")
		g.P("	return name")
		g.P("}")
		g.P()

		g.P("// MarshalFrom marshals m into x as the underlying message.")
		g.P("func (x *Any) MarshalFrom(m ", protoPackage.Ident("Message"), ") error {")
		g.P("	return MarshalFrom(x, m, ", protoPackage.Ident("MarshalOptions"), "{})")
		g.P("}")
		g.P()

		g.P("// UnmarshalTo unmarshals the contents of the underlying message of x into m.")
		g.P("// It resets m before performing the unmarshal operation.")
		g.P("// It reports an error if m is not of the right message type.")
		g.P("func (x *Any) UnmarshalTo(m ", protoPackage.Ident("Message"), ") error {")
		g.P("	return UnmarshalTo(x, m, ", protoPackage.Ident("UnmarshalOptions"), "{})")
		g.P("}")
		g.P()

		g.P("// UnmarshalNew unmarshals the contents of the underlying message of x into")
		g.P("// a newly allocated message of the specified type.")
		g.P("// It reports an error if the underlying message type could not be resolved.")
		g.P("func (x *Any) UnmarshalNew() (", protoPackage.Ident("Message"), ", error) {")
		g.P("	return UnmarshalNew(x, ", protoPackage.Ident("UnmarshalOptions"), "{})")
		g.P("}")
		g.P()

	case genid.Timestamp_message_fullname:
		g.P("// Now constructs a new Timestamp from the current time.")
		g.P("func Now() *Timestamp {")
		g.P("	return New(", timePackage.Ident("Now"), "())")
		g.P("}")
		g.P()

		g.P("// New constructs a new Timestamp from the provided time.Time.")
		g.P("func New(t ", timePackage.Ident("Time"), ") *Timestamp {")
		g.P("	return &Timestamp{Seconds: int64(t.Unix()), Nanos: int32(t.Nanosecond())}")
		g.P("}")
		g.P()

		g.P("// AsTime converts x to a time.Time.")
		g.P("func (x *Timestamp) AsTime() ", timePackage.Ident("Time"), " {")
		g.P("	return ", timePackage.Ident("Unix"), "(int64(x.GetSeconds()), int64(x.GetNanos())).UTC()")
		g.P("}")
		g.P()

		g.P("// IsValid reports whether the timestamp is valid.")
		g.P("// It is equivalent to CheckValid == nil.")
		g.P("func (x *Timestamp) IsValid() bool {")
		g.P("	return x.check() == 0")
		g.P("}")
		g.P()

		g.P("// CheckValid returns an error if the timestamp is invalid.")
		g.P("// In particular, it checks whether the value represents a date that is")
		g.P("// in the range of 0001-01-01T00:00:00Z to 9999-12-31T23:59:59Z inclusive.")
		g.P("// An error is reported for a nil Timestamp.")
		g.P("func (x *Timestamp) CheckValid() error {")
		g.P("	switch x.check() {")
		g.P("	case invalidNil:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"invalid nil Timestamp\")")
		g.P("	case invalidUnderflow:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"timestamp (%v) before 0001-01-01\", x)")
		g.P("	case invalidOverflow:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"timestamp (%v) after 9999-12-31\", x)")
		g.P("	case invalidNanos:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"timestamp (%v) has out-of-range nanos\", x)")
		g.P("	default:")
		g.P("		return nil")
		g.P("	}")
		g.P("}")
		g.P()

		g.P("const (")
		g.P("	_ = iota")
		g.P("	invalidNil")
		g.P("	invalidUnderflow")
		g.P("	invalidOverflow")
		g.P("	invalidNanos")
		g.P(")")
		g.P()

		g.P("func (x *Timestamp) check() uint {")
		g.P("	const minTimestamp = -62135596800  // Seconds between 1970-01-01T00:00:00Z and 0001-01-01T00:00:00Z, inclusive")
		g.P("	const maxTimestamp = +253402300799 // Seconds between 1970-01-01T00:00:00Z and 9999-12-31T23:59:59Z, inclusive")
		g.P("	secs := x.GetSeconds()")
		g.P("	nanos := x.GetNanos()")
		g.P("	switch {")
		g.P("	case x == nil:")
		g.P("		return invalidNil")
		g.P("	case secs < minTimestamp:")
		g.P("		return invalidUnderflow")
		g.P("	case secs > maxTimestamp:")
		g.P("		return invalidOverflow")
		g.P("	case nanos < 0 || nanos >= 1e9:")
		g.P("		return invalidNanos")
		g.P("	default:")
		g.P("		return 0")
		g.P("	}")
		g.P("}")
		g.P()

	case genid.Duration_message_fullname:
		g.P("// New constructs a new Duration from the provided time.Duration.")
		g.P("func New(d ", timePackage.Ident("Duration"), ") *Duration {")
		g.P("	nanos := d.Nanoseconds()")
		g.P("	secs := nanos / 1e9")
		g.P("	nanos -= secs * 1e9")
		g.P("	return &Duration{Seconds: int64(secs), Nanos: int32(nanos)}")
		g.P("}")
		g.P()

		g.P("// AsDuration converts x to a time.Duration,")
		g.P("// returning the closest duration value in the event of overflow.")
		g.P("func (x *Duration) AsDuration() ", timePackage.Ident("Duration"), " {")
		g.P("	secs := x.GetSeconds()")
		g.P("	nanos := x.GetNanos()")
		g.P("	d := ", timePackage.Ident("Duration"), "(secs) * ", timePackage.Ident("Second"))
		g.P("	overflow := d/", timePackage.Ident("Second"), " != ", timePackage.Ident("Duration"), "(secs)")
		g.P("	d += ", timePackage.Ident("Duration"), "(nanos) * ", timePackage.Ident("Nanosecond"))
		g.P("	overflow = overflow || (secs < 0 && nanos < 0 && d > 0)")
		g.P("	overflow = overflow || (secs > 0 && nanos > 0 && d < 0)")
		g.P("	if overflow {")
		g.P("		switch {")
		g.P("		case secs < 0:")
		g.P("			return ", timePackage.Ident("Duration"), "(", mathPackage.Ident("MinInt64"), ")")
		g.P("		case secs > 0:")
		g.P("			return ", timePackage.Ident("Duration"), "(", mathPackage.Ident("MaxInt64"), ")")
		g.P("		}")
		g.P("	}")
		g.P("	return d")
		g.P("}")
		g.P()

		g.P("// IsValid reports whether the duration is valid.")
		g.P("// It is equivalent to CheckValid == nil.")
		g.P("func (x *Duration) IsValid() bool {")
		g.P("	return x.check() == 0")
		g.P("}")
		g.P()

		g.P("// CheckValid returns an error if the duration is invalid.")
		g.P("// In particular, it checks whether the value is within the range of")
		g.P("// -10000 years to +10000 years inclusive.")
		g.P("// An error is reported for a nil Duration.")
		g.P("func (x *Duration) CheckValid() error {")
		g.P("	switch x.check() {")
		g.P("	case invalidNil:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"invalid nil Duration\")")
		g.P("	case invalidUnderflow:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"duration (%v) exceeds -10000 years\", x)")
		g.P("	case invalidOverflow:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"duration (%v) exceeds +10000 years\", x)")
		g.P("	case invalidNanosRange:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"duration (%v) has out-of-range nanos\", x)")
		g.P("	case invalidNanosSign:")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"duration (%v) has seconds and nanos with different signs\", x)")
		g.P("	default:")
		g.P("		return nil")
		g.P("	}")
		g.P("}")
		g.P()

		g.P("const (")
		g.P("	_ = iota")
		g.P("	invalidNil")
		g.P("	invalidUnderflow")
		g.P("	invalidOverflow")
		g.P("	invalidNanosRange")
		g.P("	invalidNanosSign")
		g.P(")")
		g.P()

		g.P("func (x *Duration) check() uint {")
		g.P("	const absDuration = 315576000000 // 10000yr * 365.25day/yr * 24hr/day * 60min/hr * 60sec/min")
		g.P("	secs := x.GetSeconds()")
		g.P("	nanos := x.GetNanos()")
		g.P("	switch {")
		g.P("	case x == nil:")
		g.P("		return invalidNil")
		g.P("	case secs < -absDuration:")
		g.P("		return invalidUnderflow")
		g.P("	case secs > +absDuration:")
		g.P("		return invalidOverflow")
		g.P("	case nanos <= -1e9 || nanos >= +1e9:")
		g.P("		return invalidNanosRange")
		g.P("	case (secs > 0 && nanos < 0) || (secs < 0 && nanos > 0):")
		g.P("		return invalidNanosSign")
		g.P("	default:")
		g.P("		return 0")
		g.P("	}")
		g.P("}")
		g.P()

	case genid.Struct_message_fullname:
		g.P("// NewStruct constructs a Struct from a general-purpose Go map.")
		g.P("// The map keys must be valid UTF-8.")
		g.P("// The map values are converted using NewValue.")
		g.P("func NewStruct(v map[string]interface{}) (*Struct, error) {")
		g.P("	x := &Struct{Fields: make(map[string]*Value, len(v))}")
		g.P("	for k, v := range v {")
		g.P("		if !", utf8Package.Ident("ValidString"), "(k) {")
		g.P("			return nil, ", protoimplPackage.Ident("X"), ".NewError(\"invalid UTF-8 in string: %q\", k)")
		g.P("		}")
		g.P("		var err error")
		g.P("		x.Fields[k], err = NewValue(v)")
		g.P("		if err != nil {")
		g.P("			return nil, err")
		g.P("		}")
		g.P("	}")
		g.P("	return x, nil")
		g.P("}")
		g.P()

		g.P("// AsMap converts x to a general-purpose Go map.")
		g.P("// The map values are converted by calling Value.AsInterface.")
		g.P("func (x *Struct) AsMap() map[string]interface{} {")
		g.P("	vs := make(map[string]interface{})")
		g.P("	for k, v := range x.GetFields() {")
		g.P("		vs[k] = v.AsInterface()")
		g.P("	}")
		g.P("	return vs")
		g.P("}")
		g.P()

		g.P("func (x *Struct) MarshalJSON() ([]byte, error) {")
		g.P("	return ", protojsonPackage.Ident("Marshal"), "(x)")
		g.P("}")
		g.P()

		g.P("func (x *Struct) UnmarshalJSON(b []byte) error {")
		g.P("	return ", protojsonPackage.Ident("Unmarshal"), "(b, x)")
		g.P("}")
		g.P()

	case genid.ListValue_message_fullname:
		g.P("// NewList constructs a ListValue from a general-purpose Go slice.")
		g.P("// The slice elements are converted using NewValue.")
		g.P("func NewList(v []interface{}) (*ListValue, error) {")
		g.P("	x := &ListValue{Values: make([]*Value, len(v))}")
		g.P("	for i, v := range v {")
		g.P("		var err error")
		g.P("		x.Values[i], err = NewValue(v)")
		g.P("		if err != nil {")
		g.P("			return nil, err")
		g.P("		}")
		g.P("	}")
		g.P("	return x, nil")
		g.P("}")
		g.P()

		g.P("// AsSlice converts x to a general-purpose Go slice.")
		g.P("// The slice elements are converted by calling Value.AsInterface.")
		g.P("func (x *ListValue) AsSlice() []interface{} {")
		g.P("	vs := make([]interface{}, len(x.GetValues()))")
		g.P("	for i, v := range x.GetValues() {")
		g.P("		vs[i] = v.AsInterface()")
		g.P("	}")
		g.P("	return vs")
		g.P("}")
		g.P()

		g.P("func (x *ListValue) MarshalJSON() ([]byte, error) {")
		g.P("	return ", protojsonPackage.Ident("Marshal"), "(x)")
		g.P("}")
		g.P()

		g.P("func (x *ListValue) UnmarshalJSON(b []byte) error {")
		g.P("	return ", protojsonPackage.Ident("Unmarshal"), "(b, x)")
		g.P("}")
		g.P()

	case genid.Value_message_fullname:
		g.P("// NewValue constructs a Value from a general-purpose Go interface.")
		g.P("//")
		g.P("//	╔════════════════════════╤════════════════════════════════════════════╗")
		g.P("//	║ Go type                │ Conversion                                 ║")
		g.P("//	╠════════════════════════╪════════════════════════════════════════════╣")
		g.P("//	║ nil                    │ stored as NullValue                        ║")
		g.P("//	║ bool                   │ stored as BoolValue                        ║")
		g.P("//	║ int, int32, int64      │ stored as NumberValue                      ║")
		g.P("//	║ uint, uint32, uint64   │ stored as NumberValue                      ║")
		g.P("//	║ float32, float64       │ stored as NumberValue                      ║")
		g.P("//	║ string                 │ stored as StringValue; must be valid UTF-8 ║")
		g.P("//	║ []byte                 │ stored as StringValue; base64-encoded      ║")
		g.P("//	║ map[string]interface{} │ stored as StructValue                      ║")
		g.P("//	║ []interface{}          │ stored as ListValue                        ║")
		g.P("//	╚════════════════════════╧════════════════════════════════════════════╝")
		g.P("//")
		g.P("// When converting an int64 or uint64 to a NumberValue, numeric precision loss")
		g.P("// is possible since they are stored as a float64.")
		g.P("func NewValue(v interface{}) (*Value, error) {")
		g.P("	switch v := v.(type) {")
		g.P("	case nil:")
		g.P("		return NewNullValue(), nil")
		g.P("	case bool:")
		g.P("		return NewBoolValue(v), nil")
		g.P("	case int:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case int32:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case int64:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case uint:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case uint32:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case uint64:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case float32:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case float64:")
		g.P("		return NewNumberValue(float64(v)), nil")
		g.P("	case string:")
		g.P("		if !", utf8Package.Ident("ValidString"), "(v) {")
		g.P("			return nil, ", protoimplPackage.Ident("X"), ".NewError(\"invalid UTF-8 in string: %q\", v)")
		g.P("		}")
		g.P("		return NewStringValue(v), nil")
		g.P("	case []byte:")
		g.P("		s := ", base64Package.Ident("StdEncoding"), ".EncodeToString(v)")
		g.P("		return NewStringValue(s), nil")
		g.P("	case map[string]interface{}:")
		g.P("		v2, err := NewStruct(v)")
		g.P("		if err != nil {")
		g.P("			return nil, err")
		g.P("		}")
		g.P("		return NewStructValue(v2), nil")
		g.P("	case []interface{}:")
		g.P("		v2, err := NewList(v)")
		g.P("		if err != nil {")
		g.P("			return nil, err")
		g.P("		}")
		g.P("		return NewListValue(v2), nil")
		g.P("	default:")
		g.P("		return nil, ", protoimplPackage.Ident("X"), ".NewError(\"invalid type: %T\", v)")
		g.P("	}")
		g.P("}")
		g.P()

		g.P("// NewNullValue constructs a new null Value.")
		g.P("func NewNullValue() *Value {")
		g.P("	return &Value{Kind: &Value_NullValue{NullValue: NullValue_NULL_VALUE}}")
		g.P("}")
		g.P()

		g.P("// NewBoolValue constructs a new boolean Value.")
		g.P("func NewBoolValue(v bool) *Value {")
		g.P("	return &Value{Kind: &Value_BoolValue{BoolValue: v}}")
		g.P("}")
		g.P()

		g.P("// NewNumberValue constructs a new number Value.")
		g.P("func NewNumberValue(v float64) *Value {")
		g.P("	return &Value{Kind: &Value_NumberValue{NumberValue: v}}")
		g.P("}")
		g.P()

		g.P("// NewStringValue constructs a new string Value.")
		g.P("func NewStringValue(v string) *Value {")
		g.P("	return &Value{Kind: &Value_StringValue{StringValue: v}}")
		g.P("}")
		g.P()

		g.P("// NewStructValue constructs a new struct Value.")
		g.P("func NewStructValue(v *Struct) *Value {")
		g.P("	return &Value{Kind: &Value_StructValue{StructValue: v}}")
		g.P("}")
		g.P()

		g.P("// NewListValue constructs a new list Value.")
		g.P("func NewListValue(v *ListValue) *Value {")
		g.P("	return &Value{Kind: &Value_ListValue{ListValue: v}}")
		g.P("}")
		g.P()

		g.P("// AsInterface converts x to a general-purpose Go interface.")
		g.P("//")
		g.P("// Calling Value.MarshalJSON and \"encoding/json\".Marshal on this output produce")
		g.P("// semantically equivalent JSON (assuming no errors occur).")
		g.P("//")
		g.P("// Floating-point values (i.e., \"NaN\", \"Infinity\", and \"-Infinity\") are")
		g.P("// converted as strings to remain compatible with MarshalJSON.")
		g.P("func (x *Value) AsInterface() interface{} {")
		g.P("	switch v := x.GetKind().(type) {")
		g.P("	case *Value_NumberValue:")
		g.P("		if v != nil {")
		g.P("			switch {")
		g.P("			case ", mathPackage.Ident("IsNaN"), "(v.NumberValue):")
		g.P("				return \"NaN\"")
		g.P("			case ", mathPackage.Ident("IsInf"), "(v.NumberValue, +1):")
		g.P("				return \"Infinity\"")
		g.P("			case ", mathPackage.Ident("IsInf"), "(v.NumberValue, -1):")
		g.P("				return \"-Infinity\"")
		g.P("			default:")
		g.P("				return v.NumberValue")
		g.P("			}")
		g.P("		}")
		g.P("	case *Value_StringValue:")
		g.P("		if v != nil {")
		g.P("			return v.StringValue")
		g.P("		}")
		g.P("	case *Value_BoolValue:")
		g.P("		if v != nil {")
		g.P("			return v.BoolValue")
		g.P("		}")
		g.P("	case *Value_StructValue:")
		g.P("		if v != nil {")
		g.P("			return v.StructValue.AsMap()")
		g.P("		}")
		g.P("	case *Value_ListValue:")
		g.P("		if v != nil {")
		g.P("			return v.ListValue.AsSlice()")
		g.P("		}")
		g.P("	}")
		g.P("	return nil")
		g.P("}")
		g.P()

		g.P("func (x *Value) MarshalJSON() ([]byte, error) {")
		g.P("	return ", protojsonPackage.Ident("Marshal"), "(x)")
		g.P("}")
		g.P()

		g.P("func (x *Value) UnmarshalJSON(b []byte) error {")
		g.P("	return ", protojsonPackage.Ident("Unmarshal"), "(b, x)")
		g.P("}")
		g.P()

	case genid.FieldMask_message_fullname:
		g.P("// New constructs a field mask from a list of paths and verifies that")
		g.P("// each one is valid according to the specified message type.")
		g.P("func New(m ", protoPackage.Ident("Message"), ", paths ...string) (*FieldMask, error) {")
		g.P("	x := new(FieldMask)")
		g.P("	return x, x.Append(m, paths...)")
		g.P("}")
		g.P()

		g.P("// Union returns the union of all the paths in the input field masks.")
		g.P("func Union(mx *FieldMask, my *FieldMask, ms ...*FieldMask) *FieldMask {")
		g.P("	var out []string")
		g.P("	out = append(out, mx.GetPaths()...)")
		g.P("	out = append(out, my.GetPaths()...)")
		g.P("	for _, m := range ms {")
		g.P("		out = append(out, m.GetPaths()...)")
		g.P("	}")
		g.P("	return &FieldMask{Paths: normalizePaths(out)}")
		g.P("}")
		g.P()

		g.P("// Intersect returns the intersection of all the paths in the input field masks.")
		g.P("func Intersect(mx *FieldMask, my *FieldMask, ms ...*FieldMask) *FieldMask {")
		g.P("	var ss1, ss2 []string // reused buffers for performance")
		g.P("	intersect := func(out, in []string) []string {")
		g.P("		ss1 = normalizePaths(append(ss1[:0], in...))")
		g.P("		ss2 = normalizePaths(append(ss2[:0], out...))")
		g.P("		out = out[:0]")
		g.P("		for i1, i2 := 0, 0; i1 < len(ss1) && i2 < len(ss2); {")
		g.P("			switch s1, s2 := ss1[i1], ss2[i2]; {")
		g.P("			case hasPathPrefix(s1, s2):")
		g.P("				out = append(out, s1)")
		g.P("				i1++")
		g.P("			case hasPathPrefix(s2, s1):")
		g.P("				out = append(out, s2)")
		g.P("				i2++")
		g.P("			case lessPath(s1, s2):")
		g.P("				i1++")
		g.P("			case lessPath(s2, s1):")
		g.P("				i2++")
		g.P("			}")
		g.P("		}")
		g.P("		return out")
		g.P("	}")
		g.P()
		g.P("	out := Union(mx, my, ms...).GetPaths()")
		g.P("	out = intersect(out, mx.GetPaths())")
		g.P("	out = intersect(out, my.GetPaths())")
		g.P("	for _, m := range ms {")
		g.P("		out = intersect(out, m.GetPaths())")
		g.P("	}")
		g.P("	return &FieldMask{Paths: normalizePaths(out)}")
		g.P("}")
		g.P()

		g.P("// IsValid reports whether all the paths are syntactically valid and")
		g.P("// refer to known fields in the specified message type.")
		g.P("// It reports false for a nil FieldMask.")
		g.P("func (x *FieldMask) IsValid(m ", protoPackage.Ident("Message"), ") bool {")
		g.P("	paths := x.GetPaths()")
		g.P("	return x != nil && numValidPaths(m, paths) == len(paths)")
		g.P("}")
		g.P()

		g.P("// Append appends a list of paths to the mask and verifies that each one")
		g.P("// is valid according to the specified message type.")
		g.P("// An invalid path is not appended and breaks insertion of subsequent paths.")
		g.P("func (x *FieldMask) Append(m ", protoPackage.Ident("Message"), ", paths ...string) error {")
		g.P("	numValid := numValidPaths(m, paths)")
		g.P("	x.Paths = append(x.Paths, paths[:numValid]...)")
		g.P("	paths = paths[numValid:]")
		g.P("	if len(paths) > 0 {")
		g.P("		name := m.ProtoReflect().Descriptor().FullName()")
		g.P("		return ", protoimplPackage.Ident("X"), ".NewError(\"invalid path %q for message %q\", paths[0], name)")
		g.P("	}")
		g.P("	return nil")
		g.P("}")
		g.P()

		g.P("func numValidPaths(m ", protoPackage.Ident("Message"), ", paths []string) int {")
		g.P("	md0 := m.ProtoReflect().Descriptor()")
		g.P("	for i, path := range paths {")
		g.P("		md := md0")
		g.P("		if !rangeFields(path, func(field string) bool {")
		g.P("			// Search the field within the message.")
		g.P("			if md == nil {")
		g.P("				return false // not within a message")
		g.P("			}")
		g.P("			fd := md.Fields().ByName(", protoreflectPackage.Ident("Name"), "(field))")
		g.P("			// The real field name of a group is the message name.")
		g.P("			if fd == nil {")
		g.P("				gd := md.Fields().ByName(", protoreflectPackage.Ident("Name"), "(", stringsPackage.Ident("ToLower"), "(field)))")
		g.P("				if gd != nil && gd.Kind() == ", protoreflectPackage.Ident("GroupKind"), " && string(gd.Message().Name()) == field {")
		g.P("					fd = gd")
		g.P("				}")
		g.P("			} else if fd.Kind() == ", protoreflectPackage.Ident("GroupKind"), " && string(fd.Message().Name()) != field {")
		g.P("				fd = nil")
		g.P("			}")
		g.P("			if fd == nil {")
		g.P("				return false // message has does not have this field")
		g.P("			}")
		g.P()
		g.P("			// Identify the next message to search within.")
		g.P("			md = fd.Message() // may be nil")
		g.P()
		g.P("			// Repeated fields are only allowed at the last postion.")
		g.P("			if fd.IsList() || fd.IsMap() {")
		g.P("				md = nil")
		g.P("			}")
		g.P()
		g.P("			return true")
		g.P("		}) {")
		g.P("			return i")
		g.P("		}")
		g.P("	}")
		g.P("	return len(paths)")
		g.P("}")
		g.P()

		g.P("// Normalize converts the mask to its canonical form where all paths are sorted")
		g.P("// and redundant paths are removed.")
		g.P("func (x *FieldMask) Normalize() {")
		g.P("	x.Paths = normalizePaths(x.Paths)")
		g.P("}")
		g.P()
		g.P("func normalizePaths(paths []string) []string {")
		g.P("	", sortPackage.Ident("Slice"), "(paths, func(i, j int) bool {")
		g.P("		return lessPath(paths[i], paths[j])")
		g.P("	})")
		g.P()
		g.P("	// Elide any path that is a prefix match on the previous.")
		g.P("	out := paths[:0]")
		g.P("	for _, path := range paths {")
		g.P("		if len(out) > 0 && hasPathPrefix(path, out[len(out)-1]) {")
		g.P("			continue")
		g.P("		}")
		g.P("		out = append(out, path)")
		g.P("	}")
		g.P("	return out")
		g.P("}")
		g.P()

		g.P("// hasPathPrefix is like strings.HasPrefix, but further checks for either")
		g.P("// an exact matche or that the prefix is delimited by a dot.")
		g.P("func hasPathPrefix(path, prefix string) bool {")
		g.P("	return ", stringsPackage.Ident("HasPrefix"), "(path, prefix) && (len(path) == len(prefix) || path[len(prefix)] == '.')")
		g.P("}")
		g.P()

		g.P("// lessPath is a lexicographical comparison where dot is specially treated")
		g.P("// as the smallest symbol.")
		g.P("func lessPath(x, y string) bool {")
		g.P("	for i := 0; i < len(x) && i < len(y); i++ {")
		g.P("		if x[i] != y[i] {")
		g.P("			return (x[i] - '.') < (y[i] - '.')")
		g.P("		}")
		g.P("	}")
		g.P("	return len(x) < len(y)")
		g.P("}")
		g.P()

		g.P("// rangeFields is like strings.Split(path, \".\"), but avoids allocations by")
		g.P("// iterating over each field in place and calling a iterator function.")
		g.P("func rangeFields(path string, f func(field string) bool) bool {")
		g.P("	for {")
		g.P("		var field string")
		g.P("		if i := ", stringsPackage.Ident("IndexByte"), "(path, '.'); i >= 0 {")
		g.P("			field, path = path[:i], path[i:]")
		g.P("		} else {")
		g.P("			field, path = path, \"\"")
		g.P("		}")
		g.P()
		g.P("		if !f(field) {")
		g.P("			return false")
		g.P("		}")
		g.P()
		g.P("		if len(path) == 0 {")
		g.P("			return true")
		g.P("		}")
		g.P("		path = ", stringsPackage.Ident("TrimPrefix"), "(path, \".\")")
		g.P("	}")
		g.P("}")
		g.P()

	case genid.BoolValue_message_fullname,
		genid.Int32Value_message_fullname,
		genid.Int64Value_message_fullname,
		genid.UInt32Value_message_fullname,
		genid.UInt64Value_message_fullname,
		genid.FloatValue_message_fullname,
		genid.DoubleValue_message_fullname,
		genid.StringValue_message_fullname,
		genid.BytesValue_message_fullname:
		funcName := strings.TrimSuffix(m.GoIdent.GoName, "Value")
		typeName := strings.ToLower(funcName)
		switch typeName {
		case "float":
			typeName = "float32"
		case "double":
			typeName = "float64"
		case "bytes":
			typeName = "[]byte"
		}

		g.P("// ", funcName, " stores v in a new ", m.GoIdent, " and returns a pointer to it.")
		g.P("func ", funcName, "(v ", typeName, ") *", m.GoIdent, " {")
		g.P("	return &", m.GoIdent, "{Value: v}")
		g.P("}")
		g.P()
	}
}

func genMessageFields(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	sf := f.allMessageFieldsByPtr[m]
	genMessageInternalFields(g, f, m, sf)
	for _, field := range m.Fields {
		genMessageField(g, f, m, field, sf)
	}
}

func genMessageInternalFields(g *generator.GeneratedFile, f *fileInfo, m *messageInfo, sf *structFields) {
	g.P(genid.State_goname, " ", protoimplPackage.Ident("MessageState"))
	sf.append(genid.State_goname)
	g.P(genid.SizeCache_goname, " ", protoimplPackage.Ident("SizeCache"))
	sf.append(genid.SizeCache_goname)
	if m.hasWeak {
		g.P(genid.WeakFields_goname, " ", protoimplPackage.Ident("WeakFields"))
		sf.append(genid.WeakFields_goname)
	}
	g.P(genid.UnknownFields_goname, " ", protoimplPackage.Ident("UnknownFields"))
	sf.append(genid.UnknownFields_goname)
	if m.Desc.ExtensionRanges().Len() > 0 {
		g.P(genid.ExtensionFields_goname, " ", protoimplPackage.Ident("ExtensionFields"))
		sf.append(genid.ExtensionFields_goname)
	}
	if sf.count > 0 {
		g.P()
	}
}

func genMessageField(g *generator.GeneratedFile, f *fileInfo, m *messageInfo, field *protogen.Field, sf *structFields) {
	if oneof := field.Oneof; oneof != nil && !oneof.Desc.IsSynthetic() {
		// It would be a bit simpler to iterate over the oneofs below,
		// but generating the field here keeps the contents of the Go
		// struct in the same order as the contents of the source
		// .proto file.
		if oneof.Fields[0] != field {
			return // only generate for first appearance
		}

		tags := structTags{
			{"protobuf_oneof", string(oneof.Desc.Name())},
		}
		if m.isTracked {
			tags = append(tags, gotrackTags...)
		}

		g.Annotate(m.GoIdent.GoName+"."+oneof.GoName, oneof.Location)
		leadingComments := oneof.Comments.Leading
		if leadingComments != "" {
			leadingComments += "\n"
		}
		ss := []string{fmt.Sprintf(" Types that are assignable to %s:\n", oneof.GoName)}
		for _, field := range oneof.Fields {
			ss = append(ss, "\t*"+field.GoIdent.GoName+"\n")
		}
		leadingComments += protogen.Comments(strings.Join(ss, ""))
		g.P(leadingComments,
			oneof.GoName, " ", oneofInterfaceName(oneof), tags)
		sf.append(oneof.GoName)
		return
	}
	goType, pointer := fieldGoType(g, f, field)
	if pointer {
		goType = "*" + goType
	}
	tags := structTags{
		{"protobuf", fieldProtobufTagValue(field)},
		{"json", fieldJSONTagValue(field)},
	}
	if field.Desc.IsMap() {
		key := field.Message.Fields[0]
		val := field.Message.Fields[1]
		tags = append(tags, structTags{
			{"protobuf_key", fieldProtobufTagValue(key)},
			{"protobuf_val", fieldProtobufTagValue(val)},
		}...)
	}
	if m.isTracked {
		tags = append(tags, gotrackTags...)
	}

	name := field.GoName
	if field.Desc.IsWeak() {
		name = genid.WeakFieldPrefix_goname + name
	}
	g.Annotate(m.GoIdent.GoName+"."+name, field.Location)
	leadingComments := appendDeprecationSuffix(field.Comments.Leading,
		field.Desc.Options().(*descriptorpb.FieldOptions).GetDeprecated())
	g.P(leadingComments,
		name, " ", goType, tags,
		trailingComment(field.Comments.Trailing))
	sf.append(field.GoName)
}

// genMessageDefaultDecls generates consts and vars holding the default
// values of fields.
func genMessageDefaultDecls(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	var consts, vars []string
	for _, field := range m.Fields {
		if !field.Desc.HasDefault() {
			continue
		}
		name := "Default_" + m.GoIdent.GoName + "_" + field.GoName
		goType, _ := fieldGoType(g, f, field)
		defVal := field.Desc.Default()
		switch field.Desc.Kind() {
		case protoreflect.StringKind:
			consts = append(consts, fmt.Sprintf("%s = %s(%q)", name, goType, defVal.String()))
		case protoreflect.BytesKind:
			vars = append(vars, fmt.Sprintf("%s = %s(%q)", name, goType, defVal.Bytes()))
		case protoreflect.EnumKind:
			idx := field.Desc.DefaultEnumValue().Index()
			val := field.Enum.Values[idx]
			if val.GoIdent.GoImportPath == f.GoImportPath {
				consts = append(consts, fmt.Sprintf("%s = %s", name, g.QualifiedGoIdent(val.GoIdent)))
			} else {
				// If the enum value is declared in a different Go package,
				// reference it by number since the name may not be correct.
				// See https://github.com/golang/protobuf/issues/513.
				consts = append(consts, fmt.Sprintf("%s = %s(%d) // %s",
					name, g.QualifiedGoIdent(field.Enum.GoIdent), val.Desc.Number(), g.QualifiedGoIdent(val.GoIdent)))
			}
		case protoreflect.FloatKind, protoreflect.DoubleKind:
			if f := defVal.Float(); math.IsNaN(f) || math.IsInf(f, 0) {
				var fn, arg string
				switch f := defVal.Float(); {
				case math.IsInf(f, -1):
					fn, arg = g.QualifiedGoIdent(mathPackage.Ident("Inf")), "-1"
				case math.IsInf(f, +1):
					fn, arg = g.QualifiedGoIdent(mathPackage.Ident("Inf")), "+1"
				case math.IsNaN(f):
					fn, arg = g.QualifiedGoIdent(mathPackage.Ident("NaN")), ""
				}
				vars = append(vars, fmt.Sprintf("%s = %s(%s(%s))", name, goType, fn, arg))
			} else {
				consts = append(consts, fmt.Sprintf("%s = %s(%v)", name, goType, f))
			}
		default:
			consts = append(consts, fmt.Sprintf("%s = %s(%v)", name, goType, defVal.Interface()))
		}
	}
	if len(consts) > 0 {
		g.P("// Default values for ", m.GoIdent, " fields.")
		g.P("const (")
		for _, s := range consts {
			g.P(s)
		}
		g.P(")")
	}
	if len(vars) > 0 {
		g.P("// Default values for ", m.GoIdent, " fields.")
		g.P("var (")
		for _, s := range vars {
			g.P(s)
		}
		g.P(")")
	}
	g.P()
}

func genMessageMethods(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	genMessageBaseMethods(g, f, m)
	genMessageGetterMethods(g, f, m)
	genMessageSetterMethods(g, f, m)
}

func genMessageBaseMethods(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	// Reset method.
	g.P("func (x *", m.GoIdent, ") Reset() {")
	g.P("*x = ", m.GoIdent, "{}")
	g.P("if ", protoimplPackage.Ident("UnsafeEnabled"), " {")
	g.P("mi := &", messageTypesVarName(f), "[", f.allMessagesByPtr[m], "]")
	g.P("ms := ", protoimplPackage.Ident("X"), ".MessageStateOf(", protoimplPackage.Ident("Pointer"), "(x))")
	g.P("ms.StoreMessageInfo(mi)")
	g.P("}")
	g.P("}")
	g.P()

	// String method.
	g.P("func (x *", m.GoIdent, ") String() string {")
	g.P("return ", protoimplPackage.Ident("X"), ".MessageStringOf(x)")
	g.P("}")
	g.P()

	// ProtoMessage method.
	g.P("func (*", m.GoIdent, ") ProtoMessage() {}")
	g.P()

	// ProtoReflect method.
	genMessageReflectMethods(g, f, m)

	// Descriptor method.
	if m.genRawDescMethod {
		var indexes []string
		for i := 1; i < len(m.Location.Path); i += 2 {
			indexes = append(indexes, strconv.Itoa(int(m.Location.Path[i])))
		}
		g.P("// Deprecated: Use ", m.GoIdent, ".ProtoReflect.Descriptor instead.")
		g.P("func (*", m.GoIdent, ") Descriptor() ([]byte, []int) {")
		g.P("return ", rawDescVarName(f), "GZIP(), []int{", strings.Join(indexes, ","), "}")
		g.P("}")
		g.P()
		f.needRawDesc = true
	}
}

func genMessageGetterMethods(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	for _, field := range m.Fields {
		genNoInterfacePragma(g, m.isTracked)

		// Getter for parent oneof.
		if oneof := field.Oneof; oneof != nil && oneof.Fields[0] == field && !oneof.Desc.IsSynthetic() {
			g.Annotate(m.GoIdent.GoName+".Get"+oneof.GoName, oneof.Location)
			g.P("func (x *", m.GoIdent.GoName, ") Get", oneof.GoName, "() ", oneofInterfaceName(oneof), " {")
			g.P("if x != nil {")
			g.P("return x.", oneof.GoName)
			g.P("}")
			g.P("return nil")
			g.P("}")
			g.P()
		}

		// Getter for message field.
		goType, pointer := fieldGoType(g, f, field)
		defaultValue := fieldDefaultValue(g, f, m, field)
		g.Annotate(m.GoIdent.GoName+".Get"+field.GoName, field.Location)
		leadingComments := appendDeprecationSuffix("",
			field.Desc.Options().(*descriptorpb.FieldOptions).GetDeprecated())
		switch {
		case field.Desc.IsWeak():
			g.P(leadingComments, "func (x *", m.GoIdent, ") Get", field.GoName, "() ", protoPackage.Ident("Message"), "{")
			g.P("var w ", protoimplPackage.Ident("WeakFields"))
			g.P("if x != nil {")
			g.P("w = x.", genid.WeakFields_goname)
			if m.isTracked {
				g.P("_ = x.", genid.WeakFieldPrefix_goname+field.GoName)
			}
			g.P("}")
			g.P("return ", protoimplPackage.Ident("X"), ".GetWeak(w, ", field.Desc.Number(), ", ", strconv.Quote(string(field.Message.Desc.FullName())), ")")
			g.P("}")
		case field.Oneof != nil && !field.Oneof.Desc.IsSynthetic():
			g.P(leadingComments, "func (x *", m.GoIdent, ") Get", field.GoName, "() ", goType, " {")
			g.P("if x, ok := x.Get", field.Oneof.GoName, "().(*", field.GoIdent, "); ok {")
			g.P("return x.", field.GoName)
			g.P("}")
			g.P("return ", defaultValue)
			g.P("}")
		default:
			g.P(leadingComments, "func (x *", m.GoIdent, ") Get", field.GoName, "() ", goType, " {")
			if !field.Desc.HasPresence() || defaultValue == "nil" {
				g.P("if x != nil {")
			} else {
				g.P("if x != nil && x.", field.GoName, " != nil {")
			}
			star := ""
			if pointer {
				star = "*"
			}
			g.P("return ", star, " x.", field.GoName)
			g.P("}")
			g.P("return ", defaultValue)
			g.P("}")
		}
		g.P()
	}
}

func genMessageSetterMethods(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	for _, field := range m.Fields {
		if !field.Desc.IsWeak() {
			continue
		}

		genNoInterfacePragma(g, m.isTracked)

		g.Annotate(m.GoIdent.GoName+".Set"+field.GoName, field.Location)
		leadingComments := appendDeprecationSuffix("",
			field.Desc.Options().(*descriptorpb.FieldOptions).GetDeprecated())
		g.P(leadingComments, "func (x *", m.GoIdent, ") Set", field.GoName, "(v ", protoPackage.Ident("Message"), ") {")
		g.P("var w *", protoimplPackage.Ident("WeakFields"))
		g.P("if x != nil {")
		g.P("w = &x.", genid.WeakFields_goname)
		if m.isTracked {
			g.P("_ = x.", genid.WeakFieldPrefix_goname+field.GoName)
		}
		g.P("}")
		g.P(protoimplPackage.Ident("X"), ".SetWeak(w, ", field.Desc.Number(), ", ", strconv.Quote(string(field.Message.Desc.FullName())), ", v)")
		g.P("}")
		g.P()
	}
}

// fieldGoType returns the Go type used for a field.
//
// If it returns pointer=true, the struct field is a pointer to the type.
func fieldGoType(g *generator.GeneratedFile, f *fileInfo, field *protogen.Field) (goType string, pointer bool) {
	if field.Desc.IsWeak() {
		return "struct{}", false
	}

	pointer = field.Desc.HasPresence()
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
		pointer = false // rely on nullability of slices for presence
	case protoreflect.MessageKind, protoreflect.GroupKind:
		goType = "*" + g.QualifiedGoIdent(field.Message.GoIdent)
		pointer = false // pointer captured as part of the type
	}
	switch {
	case field.Desc.IsList():
		return "[]" + goType, false
	case field.Desc.IsMap():
		keyType, _ := fieldGoType(g, f, field.Message.Fields[0])
		valType, _ := fieldGoType(g, f, field.Message.Fields[1])
		return fmt.Sprintf("map[%v]%v", keyType, valType), false
	}
	return goType, pointer
}

func fieldProtobufTagValue(field *protogen.Field) string {
	var enumName string
	if field.Desc.Kind() == protoreflect.EnumKind {
		enumName = protoimpl.X.LegacyEnumName(field.Enum.Desc)
	}
	return TagMarshal(field.Desc, enumName)
}

func fieldDefaultValue(g *generator.GeneratedFile, f *fileInfo, m *messageInfo, field *protogen.Field) string {
	if field.Desc.IsList() {
		return "nil"
	}
	if field.Desc.HasDefault() {
		defVarName := "Default_" + m.GoIdent.GoName + "_" + field.GoName
		if field.Desc.Kind() == protoreflect.BytesKind {
			return "append([]byte(nil), " + defVarName + "...)"
		}
		return defVarName
	}
	switch field.Desc.Kind() {
	case protoreflect.BoolKind:
		return "false"
	case protoreflect.StringKind:
		return `""`
	case protoreflect.MessageKind, protoreflect.GroupKind, protoreflect.BytesKind:
		return "nil"
	case protoreflect.EnumKind:
		val := field.Enum.Values[0]
		if val.GoIdent.GoImportPath == f.GoImportPath {
			return g.QualifiedGoIdent(val.GoIdent)
		} else {
			// If the enum value is declared in a different Go package,
			// reference it by number since the name may not be correct.
			// See https://github.com/golang/protobuf/issues/513.
			return g.QualifiedGoIdent(field.Enum.GoIdent) + "(" + strconv.FormatInt(int64(val.Desc.Number()), 10) + ")"
		}
	default:
		return "0"
	}
}

func fieldJSONTagValue(field *protogen.Field) string {
	return string(field.Desc.Name()) + ",omitempty"
}

func genExtensions(g *generator.GeneratedFile, f *fileInfo) {
	if len(f.allExtensions) == 0 {
		return
	}

	g.P("var ", extensionTypesVarName(f), " = []", protoimplPackage.Ident("ExtensionInfo"), "{")
	for _, x := range f.allExtensions {
		g.P("{")
		g.P("ExtendedType: (*", x.Extendee.GoIdent, ")(nil),")
		goType, pointer := fieldGoType(g, f, x.Extension)
		if pointer {
			goType = "*" + goType
		}
		g.P("ExtensionType: (", goType, ")(nil),")
		g.P("Field: ", x.Desc.Number(), ",")
		g.P("Name: ", strconv.Quote(string(x.Desc.FullName())), ",")
		g.P("Tag: ", strconv.Quote(fieldProtobufTagValue(x.Extension)), ",")
		g.P("Filename: ", strconv.Quote(f.Desc.Path()), ",")
		g.P("},")
	}
	g.P("}")
	g.P()

	// Group extensions by the target message.
	var orderedTargets []protogen.GoIdent
	allExtensionsByTarget := make(map[protogen.GoIdent][]*extensionInfo)
	allExtensionsByPtr := make(map[*extensionInfo]int)
	for i, x := range f.allExtensions {
		target := x.Extendee.GoIdent
		if len(allExtensionsByTarget[target]) == 0 {
			orderedTargets = append(orderedTargets, target)
		}
		allExtensionsByTarget[target] = append(allExtensionsByTarget[target], x)
		allExtensionsByPtr[x] = i
	}
	for _, target := range orderedTargets {
		g.P("// Extension fields to ", target, ".")
		g.P("var (")
		for _, x := range allExtensionsByTarget[target] {
			xd := x.Desc
			typeName := xd.Kind().String()
			switch xd.Kind() {
			case protoreflect.EnumKind:
				typeName = string(xd.Enum().FullName())
			case protoreflect.MessageKind, protoreflect.GroupKind:
				typeName = string(xd.Message().FullName())
			}
			fieldName := string(xd.Name())

			leadingComments := x.Comments.Leading
			if leadingComments != "" {
				leadingComments += "\n"
			}
			leadingComments += protogen.Comments(fmt.Sprintf(" %v %v %v = %v;\n",
				xd.Cardinality(), typeName, fieldName, xd.Number()))
			leadingComments = appendDeprecationSuffix(leadingComments,
				x.Desc.Options().(*descriptorpb.FieldOptions).GetDeprecated())
			g.P(leadingComments,
				"E_", x.GoIdent, " = &", extensionTypesVarName(f), "[", allExtensionsByPtr[x], "]",
				trailingComment(x.Comments.Trailing))
		}
		g.P(")")
		g.P()
	}
}

// genMessageOneofWrapperTypes generates the oneof wrapper types and
// associates the types with the parent message type.
func genMessageOneofWrapperTypes(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	for _, oneof := range m.Oneofs {
		if oneof.Desc.IsSynthetic() {
			continue
		}
		ifName := oneofInterfaceName(oneof)
		g.P("type ", ifName, " interface {")
		g.P(ifName, "()")
		g.P("}")
		g.P()
		for _, field := range oneof.Fields {
			g.Annotate(field.GoIdent.GoName, field.Location)
			g.Annotate(field.GoIdent.GoName+"."+field.GoName, field.Location)
			g.P("type ", field.GoIdent, " struct {")
			goType, _ := fieldGoType(g, f, field)
			tags := structTags{
				{"protobuf", fieldProtobufTagValue(field)},
			}
			if m.isTracked {
				tags = append(tags, gotrackTags...)
			}
			leadingComments := appendDeprecationSuffix(field.Comments.Leading,
				field.Desc.Options().(*descriptorpb.FieldOptions).GetDeprecated())
			g.P(leadingComments,
				field.GoName, " ", goType, tags,
				trailingComment(field.Comments.Trailing))
			g.P("}")
			g.P()
		}
		for _, field := range oneof.Fields {
			g.P("func (*", field.GoIdent, ") ", ifName, "() {}")
			g.P()
		}
	}
}

// oneofInterfaceName returns the name of the interface type implemented by
// the oneof field value types.
func oneofInterfaceName(oneof *protogen.Oneof) string {
	return "is" + oneof.GoIdent.GoName
}

// genNoInterfacePragma generates a standalone "nointerface" pragma to
// decorate methods with field-tracking support.
func genNoInterfacePragma(g *generator.GeneratedFile, tracked bool) {
	if tracked {
		g.P("//go:nointerface")
		g.P()
	}
}

var gotrackTags = structTags{{"go", "track"}}

// structTags is a data structure for build idiomatic Go struct tags.
// Each [2]string is a key-value pair, where value is the unescaped string.
//
// Example: structTags{{"key", "value"}}.String() -> `key:"value"`
type structTags [][2]string

func (tags structTags) String() string {
	if len(tags) == 0 {
		return ""
	}
	var ss []string
	for _, tag := range tags {
		// NOTE: When quoting the value, we need to make sure the backtick
		// character does not appear. Convert all cases to the escaped hex form.
		key := tag[0]
		val := strings.Replace(strconv.Quote(tag[1]), "`", `\x60`, -1)
		ss = append(ss, fmt.Sprintf("%s:%s", key, val))
	}
	return "`" + strings.Join(ss, " ") + "`"
}

// appendDeprecationSuffix optionally appends a deprecation notice as a suffix.
func appendDeprecationSuffix(prefix protogen.Comments, deprecated bool) protogen.Comments {
	if !deprecated {
		return prefix
	}
	if prefix != "" {
		prefix += "\n"
	}
	return prefix + " Deprecated: Do not use.\n"
}

// trailingComment is like protogen.Comments, but lacks a trailing newline.
type trailingComment protogen.Comments

func (c trailingComment) String() string {
	s := strings.TrimSuffix(protogen.Comments(c).String(), "\n")
	if strings.Contains(s, "\n") {
		// We don't support multi-lined trailing comments as it is unclear
		// how to best render them in the generated code.
		return ""
	}
	return s
}

// TagMarshal encodes the protoreflect.FieldDescriptor as a tag.
//
// The enumName must be provided if the kind is an enum.
// Historically, the formulation of the enum "name" was the proto package
// dot-concatenated with the generated Go identifier for the enum type.
// Depending on the context on how Marshal is called, there are different ways
// through which that information is determined. As such it is the caller's
// responsibility to provide a function to obtain that information.
func TagMarshal(fd pref.FieldDescriptor, enumName string) string {
	var tag []string
	switch fd.Kind() {
	case pref.BoolKind, pref.EnumKind, pref.Int32Kind, pref.Uint32Kind, pref.Int64Kind, pref.Uint64Kind:
		tag = append(tag, "varint")
	case pref.Sint32Kind:
		tag = append(tag, "zigzag32")
	case pref.Sint64Kind:
		tag = append(tag, "zigzag64")
	case pref.Sfixed32Kind, pref.Fixed32Kind, pref.FloatKind:
		tag = append(tag, "fixed32")
	case pref.Sfixed64Kind, pref.Fixed64Kind, pref.DoubleKind:
		tag = append(tag, "fixed64")
	case pref.StringKind, pref.BytesKind, pref.MessageKind:
		tag = append(tag, "bytes")
	case pref.GroupKind:
		tag = append(tag, "group")
	}
	tag = append(tag, strconv.Itoa(int(fd.Number())))
	switch fd.Cardinality() {
	case pref.Optional:
		tag = append(tag, "opt")
	case pref.Required:
		tag = append(tag, "req")
	case pref.Repeated:
		tag = append(tag, "rep")
	}
	if fd.IsPacked() {
		tag = append(tag, "packed")
	}
	name := string(fd.Name())
	if fd.Kind() == pref.GroupKind {
		// The name of the FieldDescriptor for a group field is
		// lowercased. To find the original capitalization, we
		// look in the field's MessageType.
		name = string(fd.Message().Name())
	}
	tag = append(tag, "name="+name)
	if jsonName := fd.JSONName(); jsonName != "" && jsonName != name && !fd.IsExtension() {
		// NOTE: The jsonName != name condition is suspect, but it preserve
		// the exact same semantics from the previous generator.
		tag = append(tag, "json="+jsonName)
	}
	if fd.IsWeak() {
		tag = append(tag, "weak="+string(fd.Message().FullName()))
	}
	// The previous implementation does not tag extension fields as proto3,
	// even when the field is defined in a proto3 file. Match that behavior
	// for consistency.
	if fd.Syntax() == pref.Proto3 && !fd.IsExtension() {
		tag = append(tag, "proto3")
	}
	if fd.Kind() == pref.EnumKind && enumName != "" {
		tag = append(tag, "enum="+enumName)
	}
	if fd.ContainingOneof() != nil {
		tag = append(tag, "oneof")
	}
	// This must appear last in the tag, since commas in strings aren't escaped.
	if fd.HasDefault() {
		def, _ := DefValMarshal(fd.Default(), fd.DefaultEnumValue(), fd.Kind(), 2) // 2 = defval.GoTag
		tag = append(tag, "def="+def)
	}
	return strings.Join(tag, ",")
}

func genMessageReflectMethods(g *generator.GeneratedFile, f *fileInfo, m *messageInfo) {
	// This is being handled by the fast reflection feature.

	//idx := f.allMessagesByPtr[m]
	//typesVar := messageTypesVarName(f)

	// ProtoReflect method.
	//g.P("func (x *", m.GoIdent, ") ProtoReflect() ", protoreflectPackage.Ident("Message"), " {")
	//g.P("mi := &", typesVar, "[", idx, "]")
	//g.P("if ", protoimplPackage.Ident("UnsafeEnabled"), " && x != nil {")
	//g.P("ms := ", protoimplPackage.Ident("X"), ".MessageStateOf(", protoimplPackage.Ident("Pointer"), "(x))")
	//g.P("if ms.LoadMessageInfo() == nil {")
	//g.P("ms.StoreMessageInfo(mi)")
	//g.P("}")
	//g.P("return ms")
	//g.P("}")
	//g.P("return mi.MessageOf(x)")
	//g.P("}")
	//g.P()
}

// DefValMarshal serializes v as the default string according to the given kind k.
// When specifying the Descriptor format for an enum kind, the associated
// enum value descriptor must be provided.
func DefValMarshal(v pref.Value, ev pref.EnumValueDescriptor, k pref.Kind, f Format) (string, error) {
	switch k {
	case pref.BoolKind:
		if f == GoTag {
			if v.Bool() {
				return "1", nil
			} else {
				return "0", nil
			}
		} else {
			if v.Bool() {
				return "true", nil
			} else {
				return "false", nil
			}
		}
	case pref.EnumKind:
		if f == GoTag {
			return strconv.FormatInt(int64(v.Enum()), 10), nil
		} else {
			return string(ev.Name()), nil
		}
	case pref.Int32Kind, pref.Sint32Kind, pref.Sfixed32Kind, pref.Int64Kind, pref.Sint64Kind, pref.Sfixed64Kind:
		return strconv.FormatInt(v.Int(), 10), nil
	case pref.Uint32Kind, pref.Fixed32Kind, pref.Uint64Kind, pref.Fixed64Kind:
		return strconv.FormatUint(v.Uint(), 10), nil
	case pref.FloatKind, pref.DoubleKind:
		f := v.Float()
		switch {
		case math.IsInf(f, -1):
			return "-inf", nil
		case math.IsInf(f, +1):
			return "inf", nil
		case math.IsNaN(f):
			return "nan", nil
		default:
			if k == pref.FloatKind {
				return strconv.FormatFloat(f, 'g', -1, 32), nil
			} else {
				return strconv.FormatFloat(f, 'g', -1, 64), nil
			}
		}
	case pref.StringKind:
		// String values are serialized as is without any escaping.
		return v.String(), nil
	case pref.BytesKind:
		if s, ok := marshalBytes(v.Bytes()); ok {
			return s, nil
		}
	}
	return "", fmt.Errorf("could not format value for %v: %v", k, v)
}

type Format int

const (
	_ Format = iota

	// Descriptor uses the serialization format that protoc uses with the
	// google.protobuf.FieldDescriptorProto.default_value field.
	Descriptor

	// GoTag uses the historical serialization format in Go struct field tags.
	GoTag
)

// marshalBytes serializes bytes by using C escaping.
// To match the exact output of protoc, this is identical to the
// CEscape function in strutil.cc of the protoc source code.
func marshalBytes(b []byte) (string, bool) {
	var s []byte
	for _, c := range b {
		switch c {
		case '\n':
			s = append(s, `\n`...)
		case '\r':
			s = append(s, `\r`...)
		case '\t':
			s = append(s, `\t`...)
		case '"':
			s = append(s, `\"`...)
		case '\'':
			s = append(s, `\'`...)
		case '\\':
			s = append(s, `\\`...)
		default:
			if printableASCII := c >= 0x20 && c <= 0x7e; printableASCII {
				s = append(s, c)
			} else {
				s = append(s, fmt.Sprintf(`\%03o`, c)...)
			}
		}
	}
	return string(s), true
}

type goImportPath interface {
	String() string
	Ident(string) protogen.GoIdent
}

type enumInfo struct {
	*protogen.Enum

	genJSONMethod    bool
	genRawDescMethod bool
}

func newEnumInfo(f *fileInfo, enum *protogen.Enum) *enumInfo {
	e := &enumInfo{Enum: enum}
	e.genJSONMethod = true
	e.genRawDescMethod = true
	return e
}

type messageInfo struct {
	*protogen.Message

	genRawDescMethod  bool
	genExtRangeMethod bool

	isTracked bool
	hasWeak   bool
}

// isTrackedMessage reports whether field tracking is enabled on the message.
func isTrackedMessage(m *messageInfo) (tracked bool) {
	const trackFieldUse_fieldNumber = 37383685

	// Decode the option from unknown fields to avoid a dependency on the
	// annotation proto from protoc-gen-go.
	b := m.Desc.Options().(*descriptorpb.MessageOptions).ProtoReflect().GetUnknown()
	for len(b) > 0 {
		num, typ, n := protowire.ConsumeTag(b)
		b = b[n:]
		if num == trackFieldUse_fieldNumber && typ == protowire.VarintType {
			v, _ := protowire.ConsumeVarint(b)
			tracked = protowire.DecodeBool(v)
		}
		m := protowire.ConsumeFieldValue(num, typ, b)
		b = b[m:]
	}
	return tracked
}

func newMessageInfo(f *fileInfo, message *protogen.Message) *messageInfo {
	m := &messageInfo{Message: message}
	m.genRawDescMethod = true
	m.genExtRangeMethod = true
	m.isTracked = isTrackedMessage(m)
	for _, field := range m.Fields {
		m.hasWeak = m.hasWeak || field.Desc.IsWeak()
	}
	return m
}

type extensionInfo struct {
	*protogen.Extension
}

func newExtensionInfo(f *fileInfo, extension *protogen.Extension) *extensionInfo {
	x := &extensionInfo{Extension: extension}
	return x
}

type structFields struct {
	count      int
	unexported map[int]string
}

func (sf *structFields) append(name string) {
	if r, _ := utf8.DecodeRuneInString(name); !unicode.IsUpper(r) {
		if sf.unexported == nil {
			sf.unexported = make(map[int]string)
		}
		sf.unexported[sf.count] = name
	}
	sf.count++
}

type fileInfo struct {
	*protogen.File

	allEnums      []*enumInfo
	allMessages   []*messageInfo
	allExtensions []*extensionInfo

	allEnumsByPtr         map[*enumInfo]int    // value is index into allEnums
	allMessagesByPtr      map[*messageInfo]int // value is index into allMessages
	allMessageFieldsByPtr map[*messageInfo]*structFields

	// needRawDesc specifies whether the generator should emit logic to provide
	// the legacy raw descriptor in GZIP'd form.
	// This is updated by enum and message generation logic as necessary,
	// and checked at the end of file generation.
	needRawDesc bool
}

func newFileInfo(file *protogen.File) *fileInfo {
	f := &fileInfo{File: file}

	// Collect all enums, messages, and extensions in "flattened ordering".
	// See filetype.TypeBuilder.
	var walkMessages func([]*protogen.Message, func(*protogen.Message))
	walkMessages = func(messages []*protogen.Message, f func(*protogen.Message)) {
		for _, m := range messages {
			f(m)
			walkMessages(m.Messages, f)
		}
	}
	initEnumInfos := func(enums []*protogen.Enum) {
		for _, enum := range enums {
			f.allEnums = append(f.allEnums, newEnumInfo(f, enum))
		}
	}
	initMessageInfos := func(messages []*protogen.Message) {
		for _, message := range messages {
			f.allMessages = append(f.allMessages, newMessageInfo(f, message))
		}
	}
	initExtensionInfos := func(extensions []*protogen.Extension) {
		for _, extension := range extensions {
			f.allExtensions = append(f.allExtensions, newExtensionInfo(f, extension))
		}
	}
	initEnumInfos(f.Enums)
	initMessageInfos(f.Messages)
	initExtensionInfos(f.Extensions)
	walkMessages(f.Messages, func(m *protogen.Message) {
		initEnumInfos(m.Enums)
		initMessageInfos(m.Messages)
		initExtensionInfos(m.Extensions)
	})

	// Derive a reverse mapping of enum and message pointers to their index
	// in allEnums and allMessages.
	if len(f.allEnums) > 0 {
		f.allEnumsByPtr = make(map[*enumInfo]int)
		for i, e := range f.allEnums {
			f.allEnumsByPtr[e] = i
		}
	}
	if len(f.allMessages) > 0 {
		f.allMessagesByPtr = make(map[*messageInfo]int)
		f.allMessageFieldsByPtr = make(map[*messageInfo]*structFields)
		for i, m := range f.allMessages {
			f.allMessagesByPtr[m] = i
			f.allMessageFieldsByPtr[m] = new(structFields)
		}
	}

	return f
}
