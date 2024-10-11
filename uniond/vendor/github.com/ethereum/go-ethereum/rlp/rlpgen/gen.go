// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package main

import (
	"bytes"
	"fmt"
	"go/format"
	"go/types"
	"sort"

	"github.com/ethereum/go-ethereum/rlp/internal/rlpstruct"
)

// buildContext keeps the data needed for make*Op.
type buildContext struct {
	topType *types.Named // the type we're creating methods for

	encoderIface *types.Interface
	decoderIface *types.Interface
	rawValueType *types.Named

	typeToStructCache map[types.Type]*rlpstruct.Type
}

func newBuildContext(packageRLP *types.Package) *buildContext {
	enc := packageRLP.Scope().Lookup("Encoder").Type().Underlying()
	dec := packageRLP.Scope().Lookup("Decoder").Type().Underlying()
	rawv := packageRLP.Scope().Lookup("RawValue").Type()
	return &buildContext{
		typeToStructCache: make(map[types.Type]*rlpstruct.Type),
		encoderIface:      enc.(*types.Interface),
		decoderIface:      dec.(*types.Interface),
		rawValueType:      rawv.(*types.Named),
	}
}

func (bctx *buildContext) isEncoder(typ types.Type) bool {
	return types.Implements(typ, bctx.encoderIface)
}

func (bctx *buildContext) isDecoder(typ types.Type) bool {
	return types.Implements(typ, bctx.decoderIface)
}

// typeToStructType converts typ to rlpstruct.Type.
func (bctx *buildContext) typeToStructType(typ types.Type) *rlpstruct.Type {
	if prev := bctx.typeToStructCache[typ]; prev != nil {
		return prev // short-circuit for recursive types.
	}

	// Resolve named types to their underlying type, but keep the name.
	name := types.TypeString(typ, nil)
	for {
		utype := typ.Underlying()
		if utype == typ {
			break
		}
		typ = utype
	}

	// Create the type and store it in cache.
	t := &rlpstruct.Type{
		Name:      name,
		Kind:      typeReflectKind(typ),
		IsEncoder: bctx.isEncoder(typ),
		IsDecoder: bctx.isDecoder(typ),
	}
	bctx.typeToStructCache[typ] = t

	// Assign element type.
	switch typ.(type) {
	case *types.Array, *types.Slice, *types.Pointer:
		etype := typ.(interface{ Elem() types.Type }).Elem()
		t.Elem = bctx.typeToStructType(etype)
	}
	return t
}

// genContext is passed to the gen* methods of op when generating
// the output code. It tracks packages to be imported by the output
// file and assigns unique names of temporary variables.
type genContext struct {
	inPackage   *types.Package
	imports     map[string]struct{}
	tempCounter int
}

func newGenContext(inPackage *types.Package) *genContext {
	return &genContext{
		inPackage: inPackage,
		imports:   make(map[string]struct{}),
	}
}

func (ctx *genContext) temp() string {
	v := fmt.Sprintf("_tmp%d", ctx.tempCounter)
	ctx.tempCounter++
	return v
}

func (ctx *genContext) resetTemp() {
	ctx.tempCounter = 0
}

func (ctx *genContext) addImport(path string) {
	if path == ctx.inPackage.Path() {
		return // avoid importing the package that we're generating in.
	}
	// TODO: renaming?
	ctx.imports[path] = struct{}{}
}

// importsList returns all packages that need to be imported.
func (ctx *genContext) importsList() []string {
	imp := make([]string, 0, len(ctx.imports))
	for k := range ctx.imports {
		imp = append(imp, k)
	}
	sort.Strings(imp)
	return imp
}

// qualify is the types.Qualifier used for printing types.
func (ctx *genContext) qualify(pkg *types.Package) string {
	if pkg.Path() == ctx.inPackage.Path() {
		return ""
	}
	ctx.addImport(pkg.Path())
	// TODO: renaming?
	return pkg.Name()
}

type op interface {
	// genWrite creates the encoder. The generated code should write v,
	// which is any Go expression, to the rlp.EncoderBuffer 'w'.
	genWrite(ctx *genContext, v string) string

	// genDecode creates the decoder. The generated code should read
	// a value from the rlp.Stream 'dec' and store it to dst.
	genDecode(ctx *genContext) (string, string)
}

// basicOp handles basic types bool, uint*, string.
type basicOp struct {
	typ           types.Type
	writeMethod   string     // calle write the value
	writeArgType  types.Type // parameter type of writeMethod
	decMethod     string
	decResultType types.Type // return type of decMethod
	decUseBitSize bool       // if true, result bit size is appended to decMethod
}

func (*buildContext) makeBasicOp(typ *types.Basic) (op, error) {
	op := basicOp{typ: typ}
	kind := typ.Kind()
	switch {
	case kind == types.Bool:
		op.writeMethod = "WriteBool"
		op.writeArgType = types.Typ[types.Bool]
		op.decMethod = "Bool"
		op.decResultType = types.Typ[types.Bool]
	case kind >= types.Uint8 && kind <= types.Uint64:
		op.writeMethod = "WriteUint64"
		op.writeArgType = types.Typ[types.Uint64]
		op.decMethod = "Uint"
		op.decResultType = typ
		op.decUseBitSize = true
	case kind == types.String:
		op.writeMethod = "WriteString"
		op.writeArgType = types.Typ[types.String]
		op.decMethod = "String"
		op.decResultType = types.Typ[types.String]
	default:
		return nil, fmt.Errorf("unhandled basic type: %v", typ)
	}
	return op, nil
}

func (*buildContext) makeByteSliceOp(typ *types.Slice) op {
	if !isByte(typ.Elem()) {
		panic("non-byte slice type in makeByteSliceOp")
	}
	bslice := types.NewSlice(types.Typ[types.Uint8])
	return basicOp{
		typ:           typ,
		writeMethod:   "WriteBytes",
		writeArgType:  bslice,
		decMethod:     "Bytes",
		decResultType: bslice,
	}
}

func (bctx *buildContext) makeRawValueOp() op {
	bslice := types.NewSlice(types.Typ[types.Uint8])
	return basicOp{
		typ:           bctx.rawValueType,
		writeMethod:   "Write",
		writeArgType:  bslice,
		decMethod:     "Raw",
		decResultType: bslice,
	}
}

func (op basicOp) writeNeedsConversion() bool {
	return !types.AssignableTo(op.typ, op.writeArgType)
}

func (op basicOp) decodeNeedsConversion() bool {
	return !types.AssignableTo(op.decResultType, op.typ)
}

func (op basicOp) genWrite(ctx *genContext, v string) string {
	if op.writeNeedsConversion() {
		v = fmt.Sprintf("%s(%s)", op.writeArgType, v)
	}
	return fmt.Sprintf("w.%s(%s)\n", op.writeMethod, v)
}

func (op basicOp) genDecode(ctx *genContext) (string, string) {
	var (
		resultV = ctx.temp()
		result  = resultV
		method  = op.decMethod
	)
	if op.decUseBitSize {
		// Note: For now, this only works for platform-independent integer
		// sizes. makeBasicOp forbids the platform-dependent types.
		var sizes types.StdSizes
		method = fmt.Sprintf("%s%d", op.decMethod, sizes.Sizeof(op.typ)*8)
	}

	// Call the decoder method.
	var b bytes.Buffer
	fmt.Fprintf(&b, "%s, err := dec.%s()\n", resultV, method)
	fmt.Fprintf(&b, "if err != nil { return err }\n")
	if op.decodeNeedsConversion() {
		conv := ctx.temp()
		fmt.Fprintf(&b, "%s := %s(%s)\n", conv, types.TypeString(op.typ, ctx.qualify), resultV)
		result = conv
	}
	return result, b.String()
}

// byteArrayOp handles [...]byte.
type byteArrayOp struct {
	typ  types.Type
	name types.Type // name != typ for named byte array types (e.g. common.Address)
}

func (bctx *buildContext) makeByteArrayOp(name *types.Named, typ *types.Array) byteArrayOp {
	nt := types.Type(name)
	if name == nil {
		nt = typ
	}
	return byteArrayOp{typ, nt}
}

func (op byteArrayOp) genWrite(ctx *genContext, v string) string {
	return fmt.Sprintf("w.WriteBytes(%s[:])\n", v)
}

func (op byteArrayOp) genDecode(ctx *genContext) (string, string) {
	var resultV = ctx.temp()

	var b bytes.Buffer
	fmt.Fprintf(&b, "var %s %s\n", resultV, types.TypeString(op.name, ctx.qualify))
	fmt.Fprintf(&b, "if err := dec.ReadBytes(%s[:]); err != nil { return err }\n", resultV)
	return resultV, b.String()
}

// bigIntOp handles big.Int.
// This exists because big.Int has it's own decoder operation on rlp.Stream,
// but the decode method returns *big.Int, so it needs to be dereferenced.
type bigIntOp struct {
	pointer bool
}

func (op bigIntOp) genWrite(ctx *genContext, v string) string {
	var b bytes.Buffer

	fmt.Fprintf(&b, "if %s.Sign() == -1 {\n", v)
	fmt.Fprintf(&b, "  return rlp.ErrNegativeBigInt\n")
	fmt.Fprintf(&b, "}\n")
	dst := v
	if !op.pointer {
		dst = "&" + v
	}
	fmt.Fprintf(&b, "w.WriteBigInt(%s)\n", dst)

	// Wrap with nil check.
	if op.pointer {
		code := b.String()
		b.Reset()
		fmt.Fprintf(&b, "if %s == nil {\n", v)
		fmt.Fprintf(&b, "  w.Write(rlp.EmptyString)")
		fmt.Fprintf(&b, "} else {\n")
		fmt.Fprint(&b, code)
		fmt.Fprintf(&b, "}\n")
	}

	return b.String()
}

func (op bigIntOp) genDecode(ctx *genContext) (string, string) {
	var resultV = ctx.temp()

	var b bytes.Buffer
	fmt.Fprintf(&b, "%s, err := dec.BigInt()\n", resultV)
	fmt.Fprintf(&b, "if err != nil { return err }\n")

	result := resultV
	if !op.pointer {
		result = "(*" + resultV + ")"
	}
	return result, b.String()
}

// uint256Op handles "github.com/holiman/uint256".Int
type uint256Op struct {
	pointer bool
}

func (op uint256Op) genWrite(ctx *genContext, v string) string {
	var b bytes.Buffer

	dst := v
	if !op.pointer {
		dst = "&" + v
	}
	fmt.Fprintf(&b, "w.WriteUint256(%s)\n", dst)

	// Wrap with nil check.
	if op.pointer {
		code := b.String()
		b.Reset()
		fmt.Fprintf(&b, "if %s == nil {\n", v)
		fmt.Fprintf(&b, "  w.Write(rlp.EmptyString)")
		fmt.Fprintf(&b, "} else {\n")
		fmt.Fprint(&b, code)
		fmt.Fprintf(&b, "}\n")
	}

	return b.String()
}

func (op uint256Op) genDecode(ctx *genContext) (string, string) {
	ctx.addImport("github.com/holiman/uint256")

	var b bytes.Buffer
	resultV := ctx.temp()
	fmt.Fprintf(&b, "var %s uint256.Int\n", resultV)
	fmt.Fprintf(&b, "if err := dec.ReadUint256(&%s); err != nil { return err }\n", resultV)

	result := resultV
	if op.pointer {
		result = "&" + resultV
	}
	return result, b.String()
}

// encoderDecoderOp handles rlp.Encoder and rlp.Decoder.
// In order to be used with this, the type must implement both interfaces.
// This restriction may be lifted in the future by creating separate ops for
// encoding and decoding.
type encoderDecoderOp struct {
	typ types.Type
}

func (op encoderDecoderOp) genWrite(ctx *genContext, v string) string {
	return fmt.Sprintf("if err := %s.EncodeRLP(w); err != nil { return err }\n", v)
}

func (op encoderDecoderOp) genDecode(ctx *genContext) (string, string) {
	// DecodeRLP must have pointer receiver, and this is verified in makeOp.
	etyp := op.typ.(*types.Pointer).Elem()
	var resultV = ctx.temp()

	var b bytes.Buffer
	fmt.Fprintf(&b, "%s := new(%s)\n", resultV, types.TypeString(etyp, ctx.qualify))
	fmt.Fprintf(&b, "if err := %s.DecodeRLP(dec); err != nil { return err }\n", resultV)
	return resultV, b.String()
}

// ptrOp handles pointer types.
type ptrOp struct {
	elemTyp  types.Type
	elem     op
	nilOK    bool
	nilValue rlpstruct.NilKind
}

func (bctx *buildContext) makePtrOp(elemTyp types.Type, tags rlpstruct.Tags) (op, error) {
	elemOp, err := bctx.makeOp(nil, elemTyp, rlpstruct.Tags{})
	if err != nil {
		return nil, err
	}
	op := ptrOp{elemTyp: elemTyp, elem: elemOp}

	// Determine nil value.
	if tags.NilOK {
		op.nilOK = true
		op.nilValue = tags.NilKind
	} else {
		styp := bctx.typeToStructType(elemTyp)
		op.nilValue = styp.DefaultNilValue()
	}
	return op, nil
}

func (op ptrOp) genWrite(ctx *genContext, v string) string {
	// Note: in writer functions, accesses to v are read-only, i.e. v is any Go
	// expression. To make all accesses work through the pointer, we substitute
	// v with (*v). This is required for most accesses including `v`, `call(v)`,
	// and `v[index]` on slices.
	//
	// For `v.field` and `v[:]` on arrays, the dereference operation is not required.
	var vv string
	_, isStruct := op.elem.(structOp)
	_, isByteArray := op.elem.(byteArrayOp)
	if isStruct || isByteArray {
		vv = v
	} else {
		vv = fmt.Sprintf("(*%s)", v)
	}

	var b bytes.Buffer
	fmt.Fprintf(&b, "if %s == nil {\n", v)
	fmt.Fprintf(&b, "  w.Write([]byte{0x%X})\n", op.nilValue)
	fmt.Fprintf(&b, "} else {\n")
	fmt.Fprintf(&b, "  %s", op.elem.genWrite(ctx, vv))
	fmt.Fprintf(&b, "}\n")
	return b.String()
}

func (op ptrOp) genDecode(ctx *genContext) (string, string) {
	result, code := op.elem.genDecode(ctx)
	if !op.nilOK {
		// If nil pointers are not allowed, we can just decode the element.
		return "&" + result, code
	}

	// nil is allowed, so check the kind and size first.
	// If size is zero and kind matches the nilKind of the type,
	// the value decodes as a nil pointer.
	var (
		resultV  = ctx.temp()
		kindV    = ctx.temp()
		sizeV    = ctx.temp()
		wantKind string
	)
	if op.nilValue == rlpstruct.NilKindList {
		wantKind = "rlp.List"
	} else {
		wantKind = "rlp.String"
	}
	var b bytes.Buffer
	fmt.Fprintf(&b, "var %s %s\n", resultV, types.TypeString(types.NewPointer(op.elemTyp), ctx.qualify))
	fmt.Fprintf(&b, "if %s, %s, err := dec.Kind(); err != nil {\n", kindV, sizeV)
	fmt.Fprintf(&b, "  return err\n")
	fmt.Fprintf(&b, "} else if %s != 0 || %s != %s {\n", sizeV, kindV, wantKind)
	fmt.Fprint(&b, code)
	fmt.Fprintf(&b, "  %s = &%s\n", resultV, result)
	fmt.Fprintf(&b, "}\n")
	return resultV, b.String()
}

// structOp handles struct types.
type structOp struct {
	named          *types.Named
	typ            *types.Struct
	fields         []*structField
	optionalFields []*structField
}

type structField struct {
	name string
	typ  types.Type
	elem op
}

func (bctx *buildContext) makeStructOp(named *types.Named, typ *types.Struct) (op, error) {
	// Convert fields to []rlpstruct.Field.
	var allStructFields []rlpstruct.Field
	for i := 0; i < typ.NumFields(); i++ {
		f := typ.Field(i)
		allStructFields = append(allStructFields, rlpstruct.Field{
			Name:     f.Name(),
			Exported: f.Exported(),
			Index:    i,
			Tag:      typ.Tag(i),
			Type:     *bctx.typeToStructType(f.Type()),
		})
	}

	// Filter/validate fields.
	fields, tags, err := rlpstruct.ProcessFields(allStructFields)
	if err != nil {
		return nil, err
	}

	// Create field ops.
	var op = structOp{named: named, typ: typ}
	for i, field := range fields {
		// Advanced struct tags are not supported yet.
		tag := tags[i]
		if err := checkUnsupportedTags(field.Name, tag); err != nil {
			return nil, err
		}
		typ := typ.Field(field.Index).Type()
		elem, err := bctx.makeOp(nil, typ, tags[i])
		if err != nil {
			return nil, fmt.Errorf("field %s: %v", field.Name, err)
		}
		f := &structField{name: field.Name, typ: typ, elem: elem}
		if tag.Optional {
			op.optionalFields = append(op.optionalFields, f)
		} else {
			op.fields = append(op.fields, f)
		}
	}
	return op, nil
}

func checkUnsupportedTags(field string, tag rlpstruct.Tags) error {
	if tag.Tail {
		return fmt.Errorf(`field %s has unsupported struct tag "tail"`, field)
	}
	return nil
}

func (op structOp) genWrite(ctx *genContext, v string) string {
	var b bytes.Buffer
	var listMarker = ctx.temp()
	fmt.Fprintf(&b, "%s := w.List()\n", listMarker)
	for _, field := range op.fields {
		selector := v + "." + field.name
		fmt.Fprint(&b, field.elem.genWrite(ctx, selector))
	}
	op.writeOptionalFields(&b, ctx, v)
	fmt.Fprintf(&b, "w.ListEnd(%s)\n", listMarker)
	return b.String()
}

func (op structOp) writeOptionalFields(b *bytes.Buffer, ctx *genContext, v string) {
	if len(op.optionalFields) == 0 {
		return
	}
	// First check zero-ness of all optional fields.
	var zeroV = make([]string, len(op.optionalFields))
	for i, field := range op.optionalFields {
		selector := v + "." + field.name
		zeroV[i] = ctx.temp()
		fmt.Fprintf(b, "%s := %s\n", zeroV[i], nonZeroCheck(selector, field.typ, ctx.qualify))
	}
	// Now write the fields.
	for i, field := range op.optionalFields {
		selector := v + "." + field.name
		cond := ""
		for j := i; j < len(op.optionalFields); j++ {
			if j > i {
				cond += " || "
			}
			cond += zeroV[j]
		}
		fmt.Fprintf(b, "if %s {\n", cond)
		fmt.Fprint(b, field.elem.genWrite(ctx, selector))
		fmt.Fprintf(b, "}\n")
	}
}

func (op structOp) genDecode(ctx *genContext) (string, string) {
	// Get the string representation of the type.
	// Here, named types are handled separately because the output
	// would contain a copy of the struct definition otherwise.
	var typeName string
	if op.named != nil {
		typeName = types.TypeString(op.named, ctx.qualify)
	} else {
		typeName = types.TypeString(op.typ, ctx.qualify)
	}

	// Create struct object.
	var resultV = ctx.temp()
	var b bytes.Buffer
	fmt.Fprintf(&b, "var %s %s\n", resultV, typeName)

	// Decode fields.
	fmt.Fprintf(&b, "{\n")
	fmt.Fprintf(&b, "if _, err := dec.List(); err != nil { return err }\n")
	for _, field := range op.fields {
		result, code := field.elem.genDecode(ctx)
		fmt.Fprintf(&b, "// %s:\n", field.name)
		fmt.Fprint(&b, code)
		fmt.Fprintf(&b, "%s.%s = %s\n", resultV, field.name, result)
	}
	op.decodeOptionalFields(&b, ctx, resultV)
	fmt.Fprintf(&b, "if err := dec.ListEnd(); err != nil { return err }\n")
	fmt.Fprintf(&b, "}\n")
	return resultV, b.String()
}

func (op structOp) decodeOptionalFields(b *bytes.Buffer, ctx *genContext, resultV string) {
	var suffix bytes.Buffer
	for _, field := range op.optionalFields {
		result, code := field.elem.genDecode(ctx)
		fmt.Fprintf(b, "// %s:\n", field.name)
		fmt.Fprintf(b, "if dec.MoreDataInList() {\n")
		fmt.Fprint(b, code)
		fmt.Fprintf(b, "%s.%s = %s\n", resultV, field.name, result)
		fmt.Fprintf(&suffix, "}\n")
	}
	suffix.WriteTo(b)
}

// sliceOp handles slice types.
type sliceOp struct {
	typ    *types.Slice
	elemOp op
}

func (bctx *buildContext) makeSliceOp(typ *types.Slice) (op, error) {
	elemOp, err := bctx.makeOp(nil, typ.Elem(), rlpstruct.Tags{})
	if err != nil {
		return nil, err
	}
	return sliceOp{typ: typ, elemOp: elemOp}, nil
}

func (op sliceOp) genWrite(ctx *genContext, v string) string {
	var (
		listMarker = ctx.temp() // holds return value of w.List()
		iterElemV  = ctx.temp() // iteration variable
		elemCode   = op.elemOp.genWrite(ctx, iterElemV)
	)

	var b bytes.Buffer
	fmt.Fprintf(&b, "%s := w.List()\n", listMarker)
	fmt.Fprintf(&b, "for _, %s := range %s {\n", iterElemV, v)
	fmt.Fprint(&b, elemCode)
	fmt.Fprintf(&b, "}\n")
	fmt.Fprintf(&b, "w.ListEnd(%s)\n", listMarker)
	return b.String()
}

func (op sliceOp) genDecode(ctx *genContext) (string, string) {
	var sliceV = ctx.temp() // holds the output slice
	elemResult, elemCode := op.elemOp.genDecode(ctx)

	var b bytes.Buffer
	fmt.Fprintf(&b, "var %s %s\n", sliceV, types.TypeString(op.typ, ctx.qualify))
	fmt.Fprintf(&b, "if _, err := dec.List(); err != nil { return err }\n")
	fmt.Fprintf(&b, "for dec.MoreDataInList() {\n")
	fmt.Fprintf(&b, "  %s", elemCode)
	fmt.Fprintf(&b, "  %s = append(%s, %s)\n", sliceV, sliceV, elemResult)
	fmt.Fprintf(&b, "}\n")
	fmt.Fprintf(&b, "if err := dec.ListEnd(); err != nil { return err }\n")
	return sliceV, b.String()
}

func (bctx *buildContext) makeOp(name *types.Named, typ types.Type, tags rlpstruct.Tags) (op, error) {
	switch typ := typ.(type) {
	case *types.Named:
		if isBigInt(typ) {
			return bigIntOp{}, nil
		}
		if isUint256(typ) {
			return uint256Op{}, nil
		}
		if typ == bctx.rawValueType {
			return bctx.makeRawValueOp(), nil
		}
		if bctx.isDecoder(typ) {
			return nil, fmt.Errorf("type %v implements rlp.Decoder with non-pointer receiver", typ)
		}
		// TODO: same check for encoder?
		return bctx.makeOp(typ, typ.Underlying(), tags)
	case *types.Pointer:
		if isBigInt(typ.Elem()) {
			return bigIntOp{pointer: true}, nil
		}
		if isUint256(typ.Elem()) {
			return uint256Op{pointer: true}, nil
		}
		// Encoder/Decoder interfaces.
		if bctx.isEncoder(typ) {
			if bctx.isDecoder(typ) {
				return encoderDecoderOp{typ}, nil
			}
			return nil, fmt.Errorf("type %v implements rlp.Encoder but not rlp.Decoder", typ)
		}
		if bctx.isDecoder(typ) {
			return nil, fmt.Errorf("type %v implements rlp.Decoder but not rlp.Encoder", typ)
		}
		// Default pointer handling.
		return bctx.makePtrOp(typ.Elem(), tags)
	case *types.Basic:
		return bctx.makeBasicOp(typ)
	case *types.Struct:
		return bctx.makeStructOp(name, typ)
	case *types.Slice:
		etyp := typ.Elem()
		if isByte(etyp) && !bctx.isEncoder(etyp) {
			return bctx.makeByteSliceOp(typ), nil
		}
		return bctx.makeSliceOp(typ)
	case *types.Array:
		etyp := typ.Elem()
		if isByte(etyp) && !bctx.isEncoder(etyp) {
			return bctx.makeByteArrayOp(name, typ), nil
		}
		return nil, fmt.Errorf("unhandled array type: %v", typ)
	default:
		return nil, fmt.Errorf("unhandled type: %v", typ)
	}
}

// generateDecoder generates the DecodeRLP method on 'typ'.
func generateDecoder(ctx *genContext, typ string, op op) []byte {
	ctx.resetTemp()
	ctx.addImport(pathOfPackageRLP)

	result, code := op.genDecode(ctx)
	var b bytes.Buffer
	fmt.Fprintf(&b, "func (obj *%s) DecodeRLP(dec *rlp.Stream) error {\n", typ)
	fmt.Fprint(&b, code)
	fmt.Fprintf(&b, "  *obj = %s\n", result)
	fmt.Fprintf(&b, "  return nil\n")
	fmt.Fprintf(&b, "}\n")
	return b.Bytes()
}

// generateEncoder generates the EncodeRLP method on 'typ'.
func generateEncoder(ctx *genContext, typ string, op op) []byte {
	ctx.resetTemp()
	ctx.addImport("io")
	ctx.addImport(pathOfPackageRLP)

	var b bytes.Buffer
	fmt.Fprintf(&b, "func (obj *%s) EncodeRLP(_w io.Writer) error {\n", typ)
	fmt.Fprintf(&b, "  w := rlp.NewEncoderBuffer(_w)\n")
	fmt.Fprint(&b, op.genWrite(ctx, "obj"))
	fmt.Fprintf(&b, "  return w.Flush()\n")
	fmt.Fprintf(&b, "}\n")
	return b.Bytes()
}

func (bctx *buildContext) generate(typ *types.Named, encoder, decoder bool) ([]byte, error) {
	bctx.topType = typ

	pkg := typ.Obj().Pkg()
	op, err := bctx.makeOp(nil, typ, rlpstruct.Tags{})
	if err != nil {
		return nil, err
	}

	var (
		ctx       = newGenContext(pkg)
		encSource []byte
		decSource []byte
	)
	if encoder {
		encSource = generateEncoder(ctx, typ.Obj().Name(), op)
	}
	if decoder {
		decSource = generateDecoder(ctx, typ.Obj().Name(), op)
	}

	var b bytes.Buffer
	fmt.Fprintf(&b, "package %s\n\n", pkg.Name())
	for _, imp := range ctx.importsList() {
		fmt.Fprintf(&b, "import %q\n", imp)
	}
	if encoder {
		fmt.Fprintln(&b)
		b.Write(encSource)
	}
	if decoder {
		fmt.Fprintln(&b)
		b.Write(decSource)
	}

	source := b.Bytes()
	// fmt.Println(string(source))
	return format.Source(source)
}
