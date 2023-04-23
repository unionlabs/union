package descriptor

import (
	"fmt"
	"path"
	"path/filepath"
	"strings"

	"github.com/golang/glog"
	descriptor "github.com/golang/protobuf/protoc-gen-go/descriptor"
	plugin "github.com/golang/protobuf/protoc-gen-go/plugin"
	"google.golang.org/genproto/googleapis/api/annotations"
)

// Registry is a registry of information extracted from plugin.CodeGeneratorRequest.
type Registry struct {
	// msgs is a mapping from fully-qualified message name to descriptor
	msgs map[string]*Message

	// enums is a mapping from fully-qualified enum name to descriptor
	enums map[string]*Enum

	// files is a mapping from file path to descriptor
	files map[string]*File

	// prefix is a prefix to be inserted to golang package paths generated from proto package names.
	prefix string

	// importPath is used as the package if no input files declare go_package. If it contains slashes, everything up to the rightmost slash is ignored.
	importPath string

	// pkgMap is a user-specified mapping from file path to proto package.
	pkgMap map[string]string

	// pkgAliases is a mapping from package aliases to package paths in go which are already taken.
	pkgAliases map[string]string

	// allowDeleteBody permits http delete methods to have a body
	allowDeleteBody bool

	// externalHttpRules is a mapping from fully qualified service method names to additional HttpRules applicable besides the ones found in annotations.
	externalHTTPRules map[string][]*annotations.HttpRule

	// allowMerge generation one swagger file out of multiple protos
	allowMerge bool

	// mergeFileName target swagger file name after merge
	mergeFileName string

	// allowRepeatedFieldsInBody permits repeated field in body field path of `google.api.http` annotation option
	allowRepeatedFieldsInBody bool

	// includePackageInTags controls whether the package name defined in the `package` directive
	// in the proto file can be prepended to the gRPC service name in the `Tags` field of every operation.
	includePackageInTags bool

	// repeatedPathParamSeparator specifies how path parameter repeated fields are separated
	repeatedPathParamSeparator repeatedFieldSeparator

	// useJSONNamesForFields if true json tag name is used for generating fields in swagger definitions,
	// otherwise the original proto name is used. It's helpful for synchronizing the swagger definition
	// with grpc-gateway response, if it uses json tags for marshaling.
	useJSONNamesForFields bool

	// useFQNForSwaggerName if true swagger names will use the full qualified name (FQN) from proto definition,
	// and generate a dot-separated swagger name concatenating all elements from the proto FQN.
	// If false, the default behavior is to concat the last 2 elements of the FQN if they are unique, otherwise concat
	// all the elements of the FQN without any separator
	useFQNForSwaggerName bool

	// allowColonFinalSegments determines whether colons are permitted
	// in the final segment of a path.
	allowColonFinalSegments bool

	// useGoTemplate determines whether you want to use GO templates
	// in your protofile comments
	useGoTemplate bool

	// enumsAsInts render enum as integer, as opposed to string
	enumsAsInts bool

	// disableDefaultErrors disables the generation of the default error types.
	// This is useful for users who have defined custom error handling.
	disableDefaultErrors bool

	// simpleOperationIDs removes the service prefix from the generated
	// operationIDs. This risks generating duplicate operationIDs.
	simpleOperationIDs bool

	// warnOnUnboundMethods causes the registry to emit warning logs if an RPC method
	// has no HttpRule annotation.
	warnOnUnboundMethods bool

	// generateUnboundMethods causes the registry to generate proxy methods even for
	// RPC methods that have no HttpRule annotation.
	generateUnboundMethods bool

	// omitPackageDoc, if false, causes a package comment to be included in the generated code.
	omitPackageDoc bool
}

type repeatedFieldSeparator struct {
	name string
	sep  rune
}

// NewRegistry returns a new Registry.
func NewRegistry() *Registry {
	return &Registry{
		msgs:              make(map[string]*Message),
		enums:             make(map[string]*Enum),
		files:             make(map[string]*File),
		pkgMap:            make(map[string]string),
		pkgAliases:        make(map[string]string),
		externalHTTPRules: make(map[string][]*annotations.HttpRule),
		repeatedPathParamSeparator: repeatedFieldSeparator{
			name: "csv",
			sep:  ',',
		},
	}
}

// Load loads definitions of services, methods, messages, enumerations and fields from "req".
func (r *Registry) Load(req *plugin.CodeGeneratorRequest) error {
	for _, file := range req.GetProtoFile() {
		r.loadFile(file)
	}

	var targetPkg string
	for _, name := range req.FileToGenerate {
		target := r.files[name]
		if target == nil {
			return fmt.Errorf("no such file: %s", name)
		}
		name := r.packageIdentityName(target.FileDescriptorProto)
		if targetPkg == "" {
			targetPkg = name
		} else {
			if targetPkg != name {
				return fmt.Errorf("inconsistent package names: %s %s", targetPkg, name)
			}
		}

		if err := r.loadServices(target); err != nil {
			return err
		}
	}
	return nil
}

// loadFile loads messages, enumerations and fields from "file".
// It does not loads services and methods in "file".  You need to call
// loadServices after loadFiles is called for all files to load services and methods.
func (r *Registry) loadFile(file *descriptor.FileDescriptorProto) {
	pkg := GoPackage{
		Path: r.goPackagePath(file),
		Name: r.defaultGoPackageName(file),
	}
	if err := r.ReserveGoPackageAlias(pkg.Name, pkg.Path); err != nil {
		for i := 0; ; i++ {
			alias := fmt.Sprintf("%s_%d", pkg.Name, i)
			if err := r.ReserveGoPackageAlias(alias, pkg.Path); err == nil {
				pkg.Alias = alias
				break
			}
		}
	}
	f := &File{
		FileDescriptorProto: file,
		GoPkg:               pkg,
	}

	r.files[file.GetName()] = f
	r.registerMsg(f, nil, file.GetMessageType())
	r.registerEnum(f, nil, file.GetEnumType())
}

func (r *Registry) registerMsg(file *File, outerPath []string, msgs []*descriptor.DescriptorProto) {
	for i, md := range msgs {
		m := &Message{
			File:            file,
			Outers:          outerPath,
			DescriptorProto: md,
			Index:           i,
		}
		for _, fd := range md.GetField() {
			m.Fields = append(m.Fields, &Field{
				Message:              m,
				FieldDescriptorProto: fd,
			})
		}
		file.Messages = append(file.Messages, m)
		r.msgs[m.FQMN()] = m
		glog.V(1).Infof("register name: %s", m.FQMN())

		var outers []string
		outers = append(outers, outerPath...)
		outers = append(outers, m.GetName())
		r.registerMsg(file, outers, m.GetNestedType())
		r.registerEnum(file, outers, m.GetEnumType())
	}
}

func (r *Registry) registerEnum(file *File, outerPath []string, enums []*descriptor.EnumDescriptorProto) {
	for i, ed := range enums {
		e := &Enum{
			File:                file,
			Outers:              outerPath,
			EnumDescriptorProto: ed,
			Index:               i,
		}
		file.Enums = append(file.Enums, e)
		r.enums[e.FQEN()] = e
		glog.V(1).Infof("register enum name: %s", e.FQEN())
	}
}

// LookupMsg looks up a message type by "name".
// It tries to resolve "name" from "location" if "name" is a relative message name.
func (r *Registry) LookupMsg(location, name string) (*Message, error) {
	glog.V(1).Infof("lookup %s from %s", name, location)
	if strings.HasPrefix(name, ".") {
		m, ok := r.msgs[name]
		if !ok {
			return nil, fmt.Errorf("no message found: %s", name)
		}
		return m, nil
	}

	if !strings.HasPrefix(location, ".") {
		location = fmt.Sprintf(".%s", location)
	}
	components := strings.Split(location, ".")
	for len(components) > 0 {
		fqmn := strings.Join(append(components, name), ".")
		if m, ok := r.msgs[fqmn]; ok {
			return m, nil
		}
		components = components[:len(components)-1]
	}
	return nil, fmt.Errorf("no message found: %s", name)
}

// LookupEnum looks up a enum type by "name".
// It tries to resolve "name" from "location" if "name" is a relative enum name.
func (r *Registry) LookupEnum(location, name string) (*Enum, error) {
	glog.V(1).Infof("lookup enum %s from %s", name, location)
	if strings.HasPrefix(name, ".") {
		e, ok := r.enums[name]
		if !ok {
			return nil, fmt.Errorf("no enum found: %s", name)
		}
		return e, nil
	}

	if !strings.HasPrefix(location, ".") {
		location = fmt.Sprintf(".%s", location)
	}
	components := strings.Split(location, ".")
	for len(components) > 0 {
		fqen := strings.Join(append(components, name), ".")
		if e, ok := r.enums[fqen]; ok {
			return e, nil
		}
		components = components[:len(components)-1]
	}
	return nil, fmt.Errorf("no enum found: %s", name)
}

// LookupFile looks up a file by name.
func (r *Registry) LookupFile(name string) (*File, error) {
	f, ok := r.files[name]
	if !ok {
		return nil, fmt.Errorf("no such file given: %s", name)
	}
	return f, nil
}

// LookupExternalHTTPRules looks up external http rules by fully qualified service method name
func (r *Registry) LookupExternalHTTPRules(qualifiedMethodName string) []*annotations.HttpRule {
	return r.externalHTTPRules[qualifiedMethodName]
}

// AddExternalHTTPRule adds an external http rule for the given fully qualified service method name
func (r *Registry) AddExternalHTTPRule(qualifiedMethodName string, rule *annotations.HttpRule) {
	r.externalHTTPRules[qualifiedMethodName] = append(r.externalHTTPRules[qualifiedMethodName], rule)
}

// UnboundExternalHTTPRules returns the list of External HTTPRules
// which does not have a matching method in the registry
func (r *Registry) UnboundExternalHTTPRules() []string {
	allServiceMethods := make(map[string]struct{})
	for _, f := range r.files {
		for _, s := range f.GetService() {
			svc := &Service{File: f, ServiceDescriptorProto: s}
			for _, m := range s.GetMethod() {
				method := &Method{Service: svc, MethodDescriptorProto: m}
				allServiceMethods[method.FQMN()] = struct{}{}
			}
		}
	}

	var missingMethods []string
	for httpRuleMethod := range r.externalHTTPRules {
		if _, ok := allServiceMethods[httpRuleMethod]; !ok {
			missingMethods = append(missingMethods, httpRuleMethod)
		}
	}
	return missingMethods
}

// AddPkgMap adds a mapping from a .proto file to proto package name.
func (r *Registry) AddPkgMap(file, protoPkg string) {
	r.pkgMap[file] = protoPkg
}

// SetPrefix registers the prefix to be added to go package paths generated from proto package names.
func (r *Registry) SetPrefix(prefix string) {
	r.prefix = prefix
}

// SetImportPath registers the importPath which is used as the package if no
// input files declare go_package. If it contains slashes, everything up to the
// rightmost slash is ignored.
func (r *Registry) SetImportPath(importPath string) {
	r.importPath = importPath
}

// ReserveGoPackageAlias reserves the unique alias of go package.
// If succeeded, the alias will be never used for other packages in generated go files.
// If failed, the alias is already taken by another package, so you need to use another
// alias for the package in your go files.
func (r *Registry) ReserveGoPackageAlias(alias, pkgpath string) error {
	if taken, ok := r.pkgAliases[alias]; ok {
		if taken == pkgpath {
			return nil
		}
		return fmt.Errorf("package name %s is already taken. Use another alias", alias)
	}
	r.pkgAliases[alias] = pkgpath
	return nil
}

// goPackagePath returns the go package path which go files generated from "f" should have.
// It respects the mapping registered by AddPkgMap if exists. Or use go_package as import path
// if it includes a slash,  Otherwide, it generates a path from the file name of "f".
func (r *Registry) goPackagePath(f *descriptor.FileDescriptorProto) string {
	name := f.GetName()
	if pkg, ok := r.pkgMap[name]; ok {
		return path.Join(r.prefix, pkg)
	}

	gopkg := f.Options.GetGoPackage()
	idx := strings.LastIndex(gopkg, "/")
	if idx >= 0 {
		if sc := strings.LastIndex(gopkg, ";"); sc > 0 {
			gopkg = gopkg[:sc+1-1]
		}
		return gopkg
	}

	return path.Join(r.prefix, path.Dir(name))
}

// GetAllFQMNs returns a list of all FQMNs
func (r *Registry) GetAllFQMNs() []string {
	var keys []string
	for k := range r.msgs {
		keys = append(keys, k)
	}
	return keys
}

// GetAllFQENs returns a list of all FQENs
func (r *Registry) GetAllFQENs() []string {
	var keys []string
	for k := range r.enums {
		keys = append(keys, k)
	}
	return keys
}

// SetAllowDeleteBody controls whether http delete methods may have a
// body or fail loading if encountered.
func (r *Registry) SetAllowDeleteBody(allow bool) {
	r.allowDeleteBody = allow
}

// SetAllowMerge controls whether generation one swagger file out of multiple protos
func (r *Registry) SetAllowMerge(allow bool) {
	r.allowMerge = allow
}

// IsAllowMerge whether generation one swagger file out of multiple protos
func (r *Registry) IsAllowMerge() bool {
	return r.allowMerge
}

// SetMergeFileName controls the target swagger file name out of multiple protos
func (r *Registry) SetMergeFileName(mergeFileName string) {
	r.mergeFileName = mergeFileName
}

// SetAllowRepeatedFieldsInBody controls whether repeated field can be used
// in `body` and `response_body` (`google.api.http` annotation option) field path or not
func (r *Registry) SetAllowRepeatedFieldsInBody(allow bool) {
	r.allowRepeatedFieldsInBody = allow
}

// IsAllowRepeatedFieldsInBody checks if repeated field can be used
// in `body` and `response_body` (`google.api.http` annotation option) field path or not
func (r *Registry) IsAllowRepeatedFieldsInBody() bool {
	return r.allowRepeatedFieldsInBody
}

// SetIncludePackageInTags controls whether the package name defined in the `package` directive
// in the proto file can be prepended to the gRPC service name in the `Tags` field of every operation.
func (r *Registry) SetIncludePackageInTags(allow bool) {
	r.includePackageInTags = allow
}

// IsIncludePackageInTags checks whether the package name defined in the `package` directive
// in the proto file can be prepended to the gRPC service name in the `Tags` field of every operation.
func (r *Registry) IsIncludePackageInTags() bool {
	return r.includePackageInTags
}

// GetRepeatedPathParamSeparator returns a rune spcifying how
// path parameter repeated fields are separated.
func (r *Registry) GetRepeatedPathParamSeparator() rune {
	return r.repeatedPathParamSeparator.sep
}

// GetRepeatedPathParamSeparatorName returns the name path parameter repeated
// fields repeatedFieldSeparator. I.e. 'csv', 'pipe', 'ssv' or 'tsv'
func (r *Registry) GetRepeatedPathParamSeparatorName() string {
	return r.repeatedPathParamSeparator.name
}

// SetRepeatedPathParamSeparator sets how path parameter repeated fields are
// separated. Allowed names are 'csv', 'pipe', 'ssv' and 'tsv'.
func (r *Registry) SetRepeatedPathParamSeparator(name string) error {
	var sep rune
	switch name {
	case "csv":
		sep = ','
	case "pipes":
		sep = '|'
	case "ssv":
		sep = ' '
	case "tsv":
		sep = '\t'
	default:
		return fmt.Errorf("unknown repeated path parameter separator: %s", name)
	}
	r.repeatedPathParamSeparator = repeatedFieldSeparator{
		name: name,
		sep:  sep,
	}
	return nil
}

// SetUseJSONNamesForFields sets useJSONNamesForFields
func (r *Registry) SetUseJSONNamesForFields(use bool) {
	r.useJSONNamesForFields = use
}

// GetUseJSONNamesForFields returns useJSONNamesForFields
func (r *Registry) GetUseJSONNamesForFields() bool {
	return r.useJSONNamesForFields
}

// SetUseFQNForSwaggerName sets useFQNForSwaggerName
func (r *Registry) SetUseFQNForSwaggerName(use bool) {
	r.useFQNForSwaggerName = use
}

// GetAllowColonFinalSegments returns allowColonFinalSegments
func (r *Registry) GetAllowColonFinalSegments() bool {
	return r.allowColonFinalSegments
}

// SetAllowColonFinalSegments sets allowColonFinalSegments
func (r *Registry) SetAllowColonFinalSegments(use bool) {
	r.allowColonFinalSegments = use
}

// GetUseFQNForSwaggerName returns useFQNForSwaggerName
func (r *Registry) GetUseFQNForSwaggerName() bool {
	return r.useFQNForSwaggerName
}

// GetMergeFileName return the target merge swagger file name
func (r *Registry) GetMergeFileName() string {
	return r.mergeFileName
}

// SetUseGoTemplate sets useGoTemplate
func (r *Registry) SetUseGoTemplate(use bool) {
	r.useGoTemplate = use
}

// GetUseGoTemplate returns useGoTemplate
func (r *Registry) GetUseGoTemplate() bool {
	return r.useGoTemplate
}

// SetEnumsAsInts set enumsAsInts
func (r *Registry) SetEnumsAsInts(enumsAsInts bool) {
	r.enumsAsInts = enumsAsInts
}

// GetEnumsAsInts returns enumsAsInts
func (r *Registry) GetEnumsAsInts() bool {
	return r.enumsAsInts
}

// SetDisableDefaultErrors sets disableDefaultErrors
func (r *Registry) SetDisableDefaultErrors(use bool) {
	r.disableDefaultErrors = use
}

// GetDisableDefaultErrors returns disableDefaultErrors
func (r *Registry) GetDisableDefaultErrors() bool {
	return r.disableDefaultErrors
}

// SetSimpleOperationIDs sets simpleOperationIDs
func (r *Registry) SetSimpleOperationIDs(use bool) {
	r.simpleOperationIDs = use
}

// GetSimpleOperationIDs returns simpleOperationIDs
func (r *Registry) GetSimpleOperationIDs() bool {
	return r.simpleOperationIDs
}

// SetWarnOnUnboundMethods sets warnOnUnboundMethods
func (r *Registry) SetWarnOnUnboundMethods(warn bool) {
	r.warnOnUnboundMethods = warn
}

// SetGenerateUnboundMethods sets generateUnboundMethods
func (r *Registry) SetGenerateUnboundMethods(generate bool) {
	r.generateUnboundMethods = generate
}

// SetOmitPackageDoc controls whether the generated code contains a package comment (if set to false, it will contain one)
func (r *Registry) SetOmitPackageDoc(omit bool) {
	r.omitPackageDoc = omit
}

// GetOmitPackageDoc returns whether a package comment will be omitted from the generated code
func (r *Registry) GetOmitPackageDoc() bool {
	return r.omitPackageDoc
}

// sanitizePackageName replaces unallowed character in package name
// with allowed character.
func sanitizePackageName(pkgName string) string {
	pkgName = strings.Replace(pkgName, ".", "_", -1)
	pkgName = strings.Replace(pkgName, "-", "_", -1)
	return pkgName
}

// defaultGoPackageName returns the default go package name to be used for go files generated from "f".
// You might need to use an unique alias for the package when you import it.  Use ReserveGoPackageAlias to get a unique alias.
func (r *Registry) defaultGoPackageName(f *descriptor.FileDescriptorProto) string {
	name := r.packageIdentityName(f)
	return sanitizePackageName(name)
}

// packageIdentityName returns the identity of packages.
// protoc-gen-grpc-gateway rejects CodeGenerationRequests which contains more than one packages
// as protoc-gen-go does.
func (r *Registry) packageIdentityName(f *descriptor.FileDescriptorProto) string {
	if f.Options != nil && f.Options.GoPackage != nil {
		gopkg := f.Options.GetGoPackage()
		idx := strings.LastIndex(gopkg, "/")
		if idx < 0 {
			gopkg = gopkg[idx+1:]
		}

		gopkg = gopkg[idx+1:]
		// package name is overrided with the string after the
		// ';' character
		sc := strings.IndexByte(gopkg, ';')
		if sc < 0 {
			return sanitizePackageName(gopkg)

		}
		return sanitizePackageName(gopkg[sc+1:])
	}
	if p := r.importPath; len(p) != 0 {
		if i := strings.LastIndex(p, "/"); i >= 0 {
			p = p[i+1:]
		}
		return p
	}

	if f.Package == nil {
		base := filepath.Base(f.GetName())
		ext := filepath.Ext(base)
		return strings.TrimSuffix(base, ext)
	}
	return f.GetPackage()
}
