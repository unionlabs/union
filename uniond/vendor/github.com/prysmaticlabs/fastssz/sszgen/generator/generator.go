package generator

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"go/ast"
	"go/format"
	"go/parser"
	"go/token"
	"io/ioutil"
	"os"
	"path/filepath"
	"reflect"
	"sort"
	"strconv"
	"strings"
	"text/template"
)

const bytesPerLengthOffset = 4

// The SSZ code generation works in three steps:
// 1. Parse the Go input with the go/parser library to generate an AST representation.
// 2. Convert the AST into an Internal Representation (IR) to describe the structs and fields
// using the Value object.
// 3. Use the IR to print the encoding functions

func Encode(source string, targets []string, output string, includePaths []string, excludeTypeNames map[string]bool) error {
	files, err := parseInput(source) // 1.
	if err != nil {
		return err
	}

	// parse all the include paths as well
	include := map[string]*ast.File{}
	for _, i := range includePaths {
		files, err := parseInput(i)
		if err != nil {
			return err
		}
		for k, v := range files {
			include[k] = v
		}
	}

	// read package
	var packName string
	for _, file := range files {
		packName = file.Name.Name
	}

	e := &env{
		include:          include,
		source:           source,
		files:            files,
		objs:             map[string]*Value{},
		packName:         packName,
		targets:          targets,
		excludeTypeNames: excludeTypeNames,
	}

	if err := e.generateIR(); err != nil { // 2.
		return err
	}

	// 3.
	var out map[string]string
	if output == "" {
		out, err = e.generateEncodings()
	} else {
		// output to a specific path
		out, err = e.generateOutputEncodings(output)
	}
	if err != nil {
		panic(err)
	}
	if out == nil {
		// empty output
		panic("No files to generate")
	}

	for name, str := range out {
		output := []byte(str)

		output, err = format.Source(output)
		if err != nil {
			return err
		}
		if err := ioutil.WriteFile(name, output, 0644); err != nil {
			return err
		}
	}
	return nil
}

func isDir(path string) (bool, error) {
	fileInfo, err := os.Stat(path)
	if err != nil {
		return false, err
	}
	return fileInfo.IsDir(), nil
}

func parseInput(source string) (map[string]*ast.File, error) {
	files := map[string]*ast.File{}

	ok, err := isDir(source)
	if err != nil {
		return nil, err
	}
	if ok {
		// dir
		astFiles, err := parser.ParseDir(token.NewFileSet(), source, nil, parser.AllErrors)
		if err != nil {
			return nil, err
		}
		for _, v := range astFiles {
			if strings.HasSuffix(v.Name, "_test") || v.Name == "ignore" {
				continue
			}
			files = v.Files
		}
	} else {
		// single file
		astfile, err := parser.ParseFile(token.NewFileSet(), source, nil, parser.AllErrors)
		if err != nil {
			return nil, err
		}
		files[source] = astfile
	}
	return files, nil
}

// Value is a type that represents a Go field or struct and his
// correspondent SSZ type.
type Value struct {
	// name of the variable this value represents
	name string
	// name of the Go object this value represents
	obj string
	// auxiliary int number
	s uint64
	// type of the value
	t Type
	// array of values for a container
	o []*Value
	// type of item for an array
	e *Value
	// array is fixed size. important for codegen to know so that code can be generated to interop with slices
	c bool
	// another auxiliary int number
	m uint64
	// ref is the external reference if the struct is imported
	// from another package
	ref string
	// new determines if the value is a pointer
	noPtr bool
	// isFixed allows us to explicitly mark fixed at parse time
	fixed bool
}

func (v *Value) isListElem() bool {
	return strings.HasSuffix(v.name, "]")
}

func (v *Value) objRef() string {
	// global reference of the object including the package if the reference
	// is from an external package
	if v.ref == "" {
		return v.obj
	}
	return v.ref + "." + v.obj
}

func (v *Value) copy() *Value {
	vv := new(Value)
	*vv = *v
	vv.o = make([]*Value, len(v.o))
	for indx := range v.o {
		vv.o[indx] = v.o[indx].copy()
	}
	if v.e != nil {
		vv.e = v.e.copy()
	}
	return vv
}

// Type is a SSZ type
type Type int

const (
	// TypeUndefined is a sentinel zero value to make initialization problems detectable
	TypeUndefined Type = iota
	// TypeUint is a SSZ int type
	TypeUint
	// TypeBool is a SSZ bool type
	TypeBool
	// TypeBytes is a SSZ fixed or dynamic bytes type
	TypeBytes
	// TypeBitList is a SSZ bitlist
	TypeBitList
	// TypeVector is a SSZ vector
	TypeVector
	// TypeList is a SSZ list
	TypeList
	// TypeContainer is a SSZ container
	TypeContainer
	// TypeReference is a SSZ reference
	TypeReference
)

func (t Type) String() string {
	switch t {
	case TypeUndefined:
		return "undefined"
	case TypeUint:
		return "uint"
	case TypeBool:
		return "bool"
	case TypeBytes:
		return "bytes"
	case TypeBitList:
		return "bitlist"
	case TypeVector:
		return "vector"
	case TypeList:
		return "list"
	case TypeContainer:
		return "container"
	case TypeReference:
		return "reference"
	default:
		panic("not found")
	}
}

type env struct {
	source string
	// map of the include path for cross package reference
	include map[string]*ast.File
	// map of files with their Go AST format
	files map[string]*ast.File
	// name of the package
	packName string
	// array of structs with their Go AST format
	raw []*astStruct
	// map of structs with their IR format
	objs map[string]*Value
	// map of files with their structs in order
	order map[string][]string
	// target structures to encode
	targets []string
	// imports in all the parsed packages
	imports []*astImport
	// excludeTypeNames is a map of type names to leave out of output
	excludeTypeNames map[string]bool
}

const encodingPrefix = "_encoding.go"

func (e *env) generateOutputEncodings(output string) (map[string]string, error) {
	out := map[string]string{}

	keys := make([]string, 0, len(e.order))
	for k := range e.order {
		keys = append(keys, k)
	}
	sort.Strings(keys)

	orders := []string{}
	for _, k := range keys {
		orders = append(orders, e.order[k]...)
	}

	res, ok, err := e.print(orders)
	if err != nil {
		return nil, err
	}
	if !ok {
		return nil, nil
	}
	out[output] = res
	return out, nil
}

func (e *env) generateEncodings() (map[string]string, error) {
	outs := map[string]string{}

	for name, order := range e.order {
		// remove .go prefix and replace if with our own
		ext := filepath.Ext(name)
		name = strings.TrimSuffix(name, ext)
		name += encodingPrefix

		vvv, ok, err := e.print(order)
		if err != nil {
			return nil, err
		}
		if ok {
			outs[name] = vvv
		}
	}
	return outs, nil
}

func (e *env) hashSource() (string, error) {
	content := ""
	for _, f := range e.files {
		var buf bytes.Buffer
		if err := format.Node(&buf, token.NewFileSet(), f); err != nil {
			return "", err
		}
		content += buf.String()
	}

	hash := sha256.Sum256([]byte(content))
	return hex.EncodeToString(hash[:]), nil
}

func (e *env) print(order []string) (string, bool, error) {
	hash, err := e.hashSource()
	if err != nil {
		return "", false, fmt.Errorf("failed to hash files: %v", err)
	}

	tmpl := `// Code generated by fastssz. DO NOT EDIT.
	// Hash: {{.hash}}
	package {{.package}}

	import (
		ssz "github.com/ferranbt/fastssz" {{ if .imports }}{{ range $value := .imports }}
			{{ $value }} {{ end }}
		{{ end }}
	)

	{{ range .objs }}
		{{ .Marshal }}
		{{ .Unmarshal }}
		{{ .Size }}
		{{ .HashTreeRoot }}
		{{ .GetTree }}
	{{ end }}
	`

	data := map[string]interface{}{
		"package": e.packName,
		"hash":    hash,
	}

	type Obj struct {
		Size, Marshal, Unmarshal, HashTreeRoot, GetTree string
	}

	objs := []*Obj{}
	imports := []string{}

	// Print the objects in the order in which they appear on the file.
	for _, name := range order {
		if exclude := e.excludeTypeNames[name]; exclude {
			continue
		}
		obj, ok := e.objs[name]
		if !ok {
			continue
		}

		// detect the imports required to unmarshal this objects
		refs := detectImports(obj)
		imports = appendWithoutRepeated(imports, refs)

		if obj.isFixed() && isBasicType(obj) {
			// we have an alias of a basic type (uint, bool). These objects
			// will be encoded/decoded inside their parent container and do not
			// require the sszgen functions.
			continue
		}
		objs = append(objs, &Obj{
			HashTreeRoot: e.hashTreeRoot(name, obj),
			GetTree:      e.getTree(name, obj),
			Marshal:      e.marshal(name, obj),
			Unmarshal:    e.unmarshal(name, obj),
			Size:         e.size(name, obj),
		})
	}
	if len(objs) == 0 {
		// No valid objects found for this file
		return "", false, nil
	}
	data["objs"] = objs

	// insert any required imports
	importsStr, err := e.buildImports(imports)
	if err != nil {
		return "", false, err
	}
	if len(importsStr) != 0 {
		data["imports"] = importsStr
	}

	return execTmpl(tmpl, data), true, nil
}

func isBasicType(v *Value) bool {
	return v.t == TypeUint || v.t == TypeBool || v.t == TypeBytes
}

func (e *env) buildImports(imports []string) ([]string, error) {
	res := []string{}
	for _, i := range imports {
		imp := e.findImport(i)
		if imp != "" {
			res = append(res, imp)
		}
	}
	return res, nil
}

func (e *env) findImport(name string) string {
	for _, i := range e.imports {
		if i.match(name) {
			return i.getFullName()
		}
	}
	return ""
}

func appendWithoutRepeated(s []string, i []string) []string {
	for _, j := range i {
		if !contains(j, s) {
			s = append(s, j)
		}
	}
	return s
}

func detectImports(v *Value) []string {
	// for sure v is a container
	// check if any of the fields in the container has an import
	refs := []string{}
	for _, i := range v.o {
		var ref string
		switch i.t {
		case TypeReference:
			if !i.noPtr {
				// it is not a typed reference
				ref = i.ref
			}
		case TypeContainer:
			ref = i.ref
		case TypeList, TypeVector:
			ref = i.e.ref
		default:
			ref = i.ref
		}
		if ref != "" {
			refs = append(refs, ref)
		}
	}
	return refs
}

// All the generated functions use the '::' string to represent the pointer receiver
// of the struct method (i.e 'm' in func(m *Method) XX()) for convenience.
// This function replaces the '::' string with a valid one that corresponds
// to the first letter of the method in lower case.
func appendObjSignature(str string, v *Value) string {
	signatures := map[string]string{
		// pointer receiver
		"::": strings.ToLower(string(v.name[0])),
		// full name of the struct
		"--": v.name,
	}
	for k, v := range signatures {
		str = strings.Replace(str, k, v, -1)
	}
	return str
}

type astStruct struct {
	name     string
	obj      *ast.StructType
	packName string
	typ      ast.Expr
	implFunc bool
	isRef    bool
}

func (a *astStruct) isAlias() bool {
	return a.typ != nil
}

type astResult struct {
	objs     []*astStruct
	funcs    []string
	packName string
}

func decodeASTStruct(file *ast.File) *astResult {
	packName := file.Name.String()

	res := &astResult{
		objs:     []*astStruct{},
		funcs:    []string{},
		packName: packName,
	}

	funcRefs := map[string]int{}
	for _, dec := range file.Decls {
		if genDecl, ok := dec.(*ast.GenDecl); ok {
			for _, spec := range genDecl.Specs {
				if typeSpec, ok := spec.(*ast.TypeSpec); ok {
					obj := &astStruct{
						name:     typeSpec.Name.Name,
						packName: packName,
					}
					structType, ok := typeSpec.Type.(*ast.StructType)
					if ok {
						// type is a struct
						obj.obj = structType
					} else {
						if _, ok := typeSpec.Type.(*ast.InterfaceType); !ok {
							// type is an alias (skip interfaces)
							obj.typ = typeSpec.Type
						}
					}
					if obj.obj != nil || obj.typ != nil {
						res.objs = append(res.objs, obj)
					}
				}
			}
		}
		if funcDecl, ok := dec.(*ast.FuncDecl); ok {
			if funcDecl.Recv == nil {
				continue
			}
			if expr, ok := funcDecl.Recv.List[0].Type.(*ast.StarExpr); ok {
				// only allow pointer functions
				if i, ok := expr.X.(*ast.Ident); ok {
					objName := i.Name
					if ok := isFuncDecl(funcDecl); ok {
						funcRefs[objName]++
					}
				}
			}
		}
	}
	for name, count := range funcRefs {
		if count == 4 {
			// it implements all the interface functions
			res.funcs = append(res.funcs, name)
		}
	}
	return res
}

func isSpecificFunc(funcDecl *ast.FuncDecl, in, out []string) bool {
	check := func(types *ast.FieldList, args []string) bool {
		list := types.List
		if len(list) != len(args) {
			return false
		}

		for i := 0; i < len(list); i++ {
			typ := list[i].Type
			arg := args[i]

			var buf bytes.Buffer
			fset := token.NewFileSet()
			if err := format.Node(&buf, fset, typ); err != nil {
				panic(err)
			}
			if buf.String() != arg {
				return false
			}
		}

		return true
	}
	if !check(funcDecl.Type.Params, in) {
		return false
	}
	if !check(funcDecl.Type.Results, out) {
		return false
	}
	return true
}

func isFuncDecl(funcDecl *ast.FuncDecl) bool {
	name := funcDecl.Name.Name
	if name == "SizeSSZ" {
		return isSpecificFunc(funcDecl, []string{}, []string{"int"})
	}
	if name == "MarshalSSZTo" {
		return isSpecificFunc(funcDecl, []string{"[]byte"}, []string{"[]byte", "error"})
	}
	if name == "UnmarshalSSZ" {
		return isSpecificFunc(funcDecl, []string{"[]byte"}, []string{"error"})
	}
	if name == "HashTreeRootWith" {
		return isSpecificFunc(funcDecl, []string{"ssz.HashWalker"}, []string{"error"})
	}
	return false
}

type astImport struct {
	alias string
	path  string
}

func (a *astImport) getFullName() string {
	if a.alias != "" {
		return fmt.Sprintf("%s \"%s\"", a.alias, a.path)
	}
	return fmt.Sprintf("\"%s\"", a.path)
}

func (a *astImport) match(name string) bool {
	if a.alias != "" {
		return a.alias == name
	}
	return filepath.Base(a.path) == name
}

func trimQuotes(a string) string {
	return strings.Trim(a, "\"")
}

func decodeASTImports(file *ast.File) []*astImport {
	imports := []*astImport{}
	for _, i := range file.Imports {
		var alias string
		if i.Name != nil {
			if i.Name.Name == "_" {
				continue
			}
			alias = i.Name.Name
		}
		path := trimQuotes(i.Path.Value)
		imports = append(imports, &astImport{
			alias: alias,
			path:  path,
		})
	}
	return imports
}

func (e *env) getRawItemByName(name string) (*astStruct, bool) {
	for _, item := range e.raw {
		if item.name == name {
			return item, true
		}
	}
	return nil, false
}

func (e *env) addRawItem(i *astStruct) {
	e.raw = append(e.raw, i)
}

func (e *env) generateIR() error {
	e.raw = []*astStruct{}
	e.order = map[string][]string{}
	e.imports = []*astImport{}

	checkObjByPackage := func(packName, name string) (*astStruct, bool) {
		for _, item := range e.raw {
			if item.name == name && item.packName == packName {
				return item, true
			}
		}
		return nil, false
	}

	// we want to make sure we only include one reference for each struct name
	// among the source and include paths.
	addStructs := func(res *astResult, isRef bool) error {
		for _, i := range res.objs {
			if _, ok := checkObjByPackage(i.packName, i.name); ok {
				return fmt.Errorf("two structs share the same name %s", i.name)
			}
			i.isRef = isRef
			e.addRawItem(i)
		}
		return nil
	}

	checkImplFunc := func(res *astResult) error {
		// include all the functions that implement the interfaces
		for _, name := range res.funcs {
			v, ok := checkObjByPackage(res.packName, name)
			if !ok {
				return fmt.Errorf("cannot find %s struct", name)
			}
			v.implFunc = true
		}
		return nil
	}

	// add the imports to the environment, we want to make sure that we always import
	// the package with the same name and alias which is easier to logic with.
	addImports := func(imports []*astImport) error {
		for _, i := range imports {
			// check if we already have this import before
			found := false
			for _, j := range e.imports {
				if j.path == i.path {
					found = true
					if i.alias != j.alias {
						return fmt.Errorf("the same package is imported twice by different files of path %s and %s with different aliases: %s and %s", j.path, i.path, j.alias, i.alias)
					}
				}
			}
			if !found {
				e.imports = append(e.imports, i)
			}
		}
		return nil
	}

	// decode all the imports from the input files
	for _, file := range e.files {
		if err := addImports(decodeASTImports(file)); err != nil {
			return err
		}
	}

	astResults := []*astResult{}

	// decode the structs from the input path
	for name, file := range e.files {
		res := decodeASTStruct(file)
		if err := addStructs(res, false); err != nil {
			return err
		}

		astResults = append(astResults, res)

		// keep the ordering in which the structs appear so that we always generate them in
		// the same predictable order
		structOrdering := []string{}
		for _, i := range res.objs {
			structOrdering = append(structOrdering, i.name)
		}
		e.order[name] = structOrdering
	}

	// decode the structs from the include path but ONLY include them on 'raw' not in 'order'.
	// If the structs are in raw they can be used as a reference at compilation time and since they are
	// not in 'order' they cannot be used to marshal/unmarshal encodings
	for _, file := range e.include {
		res := decodeASTStruct(file)
		if err := addStructs(res, true); err != nil {
			return err
		}

		astResults = append(astResults, res)
	}

	for _, res := range astResults {
		if err := checkImplFunc(res); err != nil {
			return err
		}
	}

	for _, obj := range e.raw {
		// If the user does not want to generate a struct we should skip it right away
		if e.excludeTypeNames[obj.name] {
			continue
		}

		name := obj.name

		var valid bool
		if e.targets == nil || len(e.targets) == 0 {
			valid = true
		} else {
			valid = contains(name, e.targets)
		}
		if valid {
			if obj.isRef {
				// do not process imported elements
				continue
			}
			if _, err := e.encodeItem(name, ""); err != nil {
				return err
			}
		}
	}
	return nil
}

func contains(i string, j []string) bool {
	for _, a := range j {
		if a == i {
			return true
		}
	}
	return false
}

func (e *env) encodeItem(name, tags string) (*Value, error) {
	v, ok := e.objs[name]
	if !ok {
		var err error
		raw, ok := e.getRawItemByName(name)
		if !ok {
			return nil, fmt.Errorf("could not find struct with name '%s'", name)
		}
		if raw.implFunc {
			size, _ := getTagsInt(tags, "ssz-size")
			v = &Value{t: TypeReference, s: size, noPtr: raw.obj == nil}
		} else if raw.obj != nil {
			v, err = e.parseASTStructType(name, raw.obj)
		} else {
			v, err = e.parseASTFieldType(name, tags, raw.typ)
		}
		if err != nil {
			return nil, fmt.Errorf("failed to encode %s: %v", name, err)
		}
		v.name = name
		v.obj = name

		if !raw.isAlias() {
			// alias objects have to be recreated every time and cannot be reused
			// since they only define the type
			e.objs[name] = v
		}
	}
	return v.copy(), nil
}

// parse the Go AST struct
func (e *env) parseASTStructType(name string, typ *ast.StructType) (*Value, error) {
	v := &Value{
		name: name,
		t:    TypeContainer,
		o:    []*Value{},
	}

	for _, f := range typ.Fields.List {
		if len(f.Names) != 1 {
			continue
		}
		name := f.Names[0].Name
		if !isExportedField(name) {
			continue
		}
		if strings.HasPrefix(name, "XXX_") {
			// skip protobuf methods
			continue
		}
		var tags string
		if f.Tag != nil {
			tags = f.Tag.Value
		}

		elem, err := e.parseASTFieldType(name, tags, f.Type)
		if err != nil {
			return nil, err
		}
		if elem == nil {
			continue
		}
		elem.name = name
		v.o = append(v.o, elem)
	}

	return v, nil
}

// parse the Go AST field
func (e *env) parseASTFieldType(name, tags string, expr ast.Expr) (*Value, error) {
	if tag, ok := getTags(tags, "ssz"); ok && tag == "-" {
		// omit value
		return nil, nil
	}

	switch obj := expr.(type) {
	case *ast.StarExpr:
		// *Struct
		switch elem := obj.X.(type) {
		case *ast.Ident:
			// reference to a local package
			return e.encodeItem(elem.Name, tags)

		case *ast.SelectorExpr:
			// reference of the external package
			ref := elem.X.(*ast.Ident).Name
			// reference to a struct from another package
			v, err := e.encodeItem(elem.Sel.Name, tags)
			if err != nil {
				return nil, err
			}
			v.ref = ref
			return v, nil

		default:
			return nil, fmt.Errorf("cannot handle %s", elem)
		}

	case *ast.ArrayType:
		dims, err := extractSSZDimensions(tags)
		if err != nil {
			return nil, err
		}

		collectionExpr := obj
		outer := &Value{}
		collection := outer
		for _, dim := range dims {
			if dim.IsVector() {
				collection.t = TypeVector
				collection.s = uint64(dim.VectorLen())
			}
			if dim.IsList() {
				collection.t = TypeList
				collection.m = uint64(dim.ListLen())
				collection.s = uint64(dim.ListLen())
			}

			// If we're looking at a fixed-size array, attempt to grab the parsed size value. from go/ast
			// Ellipsis node for [...]T array types, nil for slice types
			// so when a `[]byte` expression is parsed, Len will be nil:
			var astSize *uint64
			// if .Len is nil, this is a slice, not a fixed length array
			if collectionExpr.Len != nil {
				arrayLen, ok := collectionExpr.Len.(*ast.BasicLit)
				if !ok {
					return nil, fmt.Errorf("failed to parse field %s. byte array definition not understood by go/ast", name)
				}
				a, err := strconv.ParseUint(arrayLen.Value, 0, 64)
				if err != nil {
					return nil, fmt.Errorf("could not parse array length for field %s", name)
				}
				astSize = &a
			}
			if astSize != nil {
				collection.c = true
			}
			if astSize != nil {
				if collection.t != TypeVector {
					return nil, fmt.Errorf("unexpected type for fixed size array, name=%s, type=%s", name, collection.t.String())
				}
				if collection.s != *astSize {
					return nil, fmt.Errorf("unexpected mismatch between ssz-size and array fixed size, name=%s, ssz-size=%d, fixed=%d", name, collection.s, *astSize)
				}
			}

			switch eeType := collectionExpr.Elt.(type) {
			case *ast.ArrayType:
				// we expect there to a subsequent dimension when the element type is an ArrayType
				// so we update the expression and value container in preparation for the next iteration
				collectionExpr = eeType
				collection.e = &Value{}
				collection = collection.e
				continue
			case *ast.Ident:
				// this condition is preserving the special nesting of byte,
				// because byte has special handling in the code generator templates.
				if eeType.Name == "byte" {
					// note that we are overwriting the list/vector types and replacing them with TypeBytes
					// TypeBytes can either be a list or vector (determined by looking at the isFixed result)
					collection.t = TypeBytes
					if dim.IsVector() {
						// this is how we differentiate byte lists from byte vectors, rather than the usual approach
						// of nesting a Value for the element within the .e attribute
						collection.fixed = true
					}
					if dim.IsBitlist() {
						collection.t = TypeBitList
					}
					continue
				} else {
					// anything else should recurse to the basic *ast.Ident case defined just below this ArrayType case
					element, err := e.parseASTFieldType(name, tags, eeType)
					if err != nil {
						return nil, err
					}
					collection.e = element
				}
			default:
				element, err := e.parseASTFieldType(name, tags, eeType)
				if err != nil {
					return nil, err
				}
				collection.e = element
			}
		}
		return outer, nil
	case *ast.Ident:
		// basic type
		var v *Value
		switch obj.Name {
		case "uint64":
			v = &Value{t: TypeUint, s: 8}
		case "uint32":
			v = &Value{t: TypeUint, s: 4}
		case "uint16":
			v = &Value{t: TypeUint, s: 2}
		case "uint8":
			v = &Value{t: TypeUint, s: 1}
		case "bool":
			v = &Value{t: TypeBool, s: 1}
		default:
			// try to resolve as an alias
			vv, err := e.encodeItem(obj.Name, tags)
			if err != nil {
				return nil, fmt.Errorf("failed to encode %s: %v", obj.Name, err)
			}
			return vv, nil
		}
		return v, nil

	case *ast.SelectorExpr:
		name := obj.X.(*ast.Ident).Name
		sel := obj.Sel.Name

		if sel == "Bitlist" {
			// go-bitfield/Bitlist
			maxSize, ok := getTagsInt(tags, "ssz-max")
			if !ok {
				return nil, fmt.Errorf("bitlist %s does not have ssz-max tag", name)
			}
			return &Value{t: TypeBitList, m: maxSize, s: maxSize}, nil
		} else if strings.HasPrefix(sel, "Bitvector") {
			// go-bitfield/Bitvector, fixed bytes
			dims, err := extractSSZDimensions(tags)
			if err != nil {
				return nil, fmt.Errorf("failed to parse ssz-size tag for bitvector %s, err=%s", name, err)
			}
			if len(dims) < 1 {
				return nil, fmt.Errorf("did not find any ssz tags for the bitvector named %s", name)
			}
			tailDim := dims[len(dims)-1] // get last value in case this value is nested within a List/Vector
			if !tailDim.IsVector() {
				return nil, fmt.Errorf("bitvector tag parse failed (no ssz-size for last dim) %s, err=%s", name, err)
			}
			return &Value{t: TypeBytes, fixed: true, s: uint64(tailDim.VectorLen())}, nil
		}
		// external reference
		vv, err := e.encodeItem(sel, tags)
		if err != nil {
			return nil, fmt.Errorf("failed to encode %s: %v", sel, err)
		}
		vv.ref = name
		vv.noPtr = true
		return vv, nil

	default:
		panic(fmt.Errorf("ast type '%s' not expected", reflect.TypeOf(expr)))
	}
}

func isExportedField(str string) bool {
	return str[0] <= 90
}

// getTagsInt returns tags of the format 'ssz-size:"32"'
func getTagsInt(str string, field string) (uint64, bool) {
	numStr, ok := getTags(str, field)
	if !ok {
		return 0, false
	}
	num, err := strconv.Atoi(numStr)
	if err != nil {
		return 0, false
	}
	return uint64(num), true
}

// getTags returns the tags from a given field
func getTags(str string, field string) (string, bool) {
	str = strings.Trim(str, "`")

	for _, tag := range strings.Split(str, " ") {
		if !strings.Contains(tag, ":") {
			return "", false
		}
		spl := strings.Split(tag, ":")
		if len(spl) != 2 {
			return "", false
		}

		tagName, vals := spl[0], spl[1]
		if !strings.HasPrefix(vals, "\"") || !strings.HasSuffix(vals, "\"") {
			return "", false
		}
		if tagName != field {
			continue
		}

		vals = strings.Trim(vals, "\"")
		return vals, true
	}
	return "", false
}

func (v *Value) isFixed() bool {
	switch v.t {
	// fixed size primitive types
	case TypeUint, TypeBool:
		return true
	// dynamic collection types
	case TypeList, TypeBitList:
		return false
	case TypeVector:
		if v.e.t == TypeUndefined {
			fmt.Printf("%s", v.name)
		}
		return v.e.isFixed()
	case TypeBytes:
		// we set this flag for all fixed size values of TypeBytes
		// based on the presence of a corresponding ssz-size tag
		if v.fixed {
			return true
		}
		// critical that we set this correctly since the zero-value is false
		return false
	case TypeContainer:
		for _, f := range v.o {
			if f.t == TypeUndefined {
				fmt.Printf("%s %s", v.name, f.name)
			}
			// if any contained value is not fixed, it is not fixed
			if !f.isFixed() {
				return false
			}
		}
		// if all values within the container are fixed, it is fixed
		return true
	case TypeReference:
		if v.s != 0 {
			return true
		}
		return false
	default:
		// TypeUndefined should be the only type to fallthrough to this case
		// TypeUndefined always means there is a fatal error in the parsing logic
		panic(fmt.Errorf("is fixed not implemented for type %s named %s", v.t.String(), v.name))
	}
}

func execTmpl(tpl string, input interface{}) string {
	tmpl, err := template.New("tmpl").Parse(tpl)
	if err != nil {
		panic(err)
	}
	buf := new(bytes.Buffer)
	if err = tmpl.Execute(buf, input); err != nil {
		panic(err)
	}
	return buf.String()
}

func uintVToName(v *Value) string {
	if v.t != TypeUint {
		panic("not expected")
	}
	switch v.s {
	case 8:
		return "Uint64"
	case 4:
		return "Uint32"
	case 2:
		return "Uint16"
	case 1:
		return "Uint8"
	default:
		panic(fmt.Sprintf("unknown uint size, %d bytes. field name=%s", v.s, v.name))
	}
}
