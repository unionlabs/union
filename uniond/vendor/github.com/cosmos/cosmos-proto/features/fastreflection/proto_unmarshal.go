package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/reflect/protoreflect"
	"strconv"
	"strings"
)

func (g *fastGenerator) genUnmarshalMethod() {

	required := g.message.Desc.RequiredNumbers()

	// UNMARSHAL METHOD
	g.P(`unmarshal := func(input `, protoifacePkg.Ident("UnmarshalInput"), `) (`, protoifacePkg.Ident("UnmarshalOutput"), `, error) {`)
	g.P(`x := input.Message.Interface().(*`, g.message.GoIdent, `)`)
	g.P(`if x == nil {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), ` {`)
	g.P("NoUnkeyedLiterals: input.NoUnkeyedLiterals,")
	g.P("Flags:               input.Flags,")
	g.P("}, nil")
	g.P("}")
	g.P("options := ", runtimePackage.Ident("UnmarshalInputToOptions"), "(input)")
	g.P("_ = options")
	g.P("dAtA := input.Buf")
	// body
	if required.Len() > 0 {
		g.P(`var hasFields [`, strconv.Itoa(1+(required.Len()-1)/64), `]uint64`)
	}
	g.P(`l := len(dAtA)`)
	g.P(`iNdEx := 0`)
	g.P(`for iNdEx < l {`)
	g.P(`preIndex := iNdEx`)
	g.P(`var wire uint64`)
	g.decodeVarint("wire", "uint64")
	g.P(`fieldNum := int32(wire >> 3)`)
	g.P(`wireType := int(wire & 0x7)`)
	g.P(`if wireType == `, strconv.Itoa(int(protowire.EndGroupType)), ` {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("fmt", "Errorf"), `("proto: `, g.message.GoIdent.GoName, `: wiretype end group for non-group")`)
	g.P(`}`)
	g.P(`if fieldNum <= 0 {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("fmt", "Errorf"), `("proto: `, g.message.GoIdent.GoName, `: illegal tag %d (wire type %d)", fieldNum, wire)`)
	g.P(`}`)
	g.P(`switch fieldNum {`)
	for _, field := range g.message.Fields {
		g.unmarshalField(field, g.message, true, required)
	}
	g.P(`default:`)
	if len(g.message.Extensions) > 0 {
		c := []string{}
		eranges := g.message.Desc.ExtensionRanges()
		for e := 0; e < eranges.Len(); e++ {
			erange := eranges.Get(e)
			c = append(c, `((fieldNum >= `, strconv.Itoa(int(erange[0])), ") && (fieldNum < ", strconv.Itoa(int(erange[1])), `))`)
		}
		g.P(`if `, strings.Join(c, "||"), `{`)
		g.P(`var sizeOfWire int`)
		g.P(`for {`)
		g.P(`sizeOfWire++`)
		g.P(`wire >>= 7`)
		g.P(`if wire == 0 {`)
		g.P(`break`)
		g.P(`}`)
		g.P(`}`)
		g.P(`iNdEx-=sizeOfWire`)
		g.P(`skippy, err := `, runtimePackage.Ident("Skip"), `(dAtA[iNdEx:])`)
		g.P(`if err != nil {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", `err`)
		g.P(`}`)
		g.P(`if (skippy < 0) || (iNdEx + skippy) < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if (iNdEx + skippy) > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		g.P(g.Ident(generator.ProtoPkg, "AppendExtension"), `(m, int32(fieldNum), dAtA[iNdEx:iNdEx+skippy])`)
		g.P(`iNdEx += skippy`)
		g.P(`} else {`)
	}
	g.P(`iNdEx=preIndex`)
	g.P(`skippy, err := `, runtimePackage.Ident("Skip"), `(dAtA[iNdEx:])`)
	g.P(`if err != nil {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", `err`)
	g.P(`}`)
	g.P(`if (skippy < 0) || (iNdEx + skippy) < 0 {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
	g.P(`}`)
	g.P(`if (iNdEx + skippy) > l {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
	g.P(`}`)
	g.P("if !options.DiscardUnknown {")
	g.P(`x.unknownFields = append(x.unknownFields, dAtA[iNdEx:iNdEx+skippy]...)`)
	g.P("}")
	g.P(`iNdEx += skippy`)
	if len(g.message.Extensions) > 0 {
		g.P(`}`)
	}
	g.P(`}`)
	g.P(`}`)

	for _, field := range g.message.Fields {
		if field.Desc.Cardinality() != protoreflect.Required {
			continue
		}
		var fieldBit int
		for fieldBit = 0; fieldBit < required.Len(); fieldBit++ {
			if required.Get(fieldBit) == field.Desc.Number() {
				break
			}
		}
		if fieldBit == required.Len() {
			panic("missing required field")
		}
		g.P(`if hasFields[`, strconv.Itoa(fieldBit/64), `] & uint64(`, fmt.Sprintf("0x%08x", uint64(1)<<(fieldBit%64)), `) == 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", `new(`, g.Ident(generator.ProtoPkg, "RequiredNotSetError"), `)`)
		g.P(`}`)
	}
	g.P()
	g.P(`if iNdEx > l {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags}, ", g.Ident("io", `ErrUnexpectedEOF`))
	g.P(`}`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags}, ", `nil`)
	g.P(`}`)
}

func (g *fastGenerator) decodeVarint(varName string, typName string) {
	g.P(`for shift := uint(0); ; shift += 7 {`)
	g.P(`if shift >= 64 {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags}, ", runtimePackage.Ident("ErrIntOverflow"))
	g.P(`}`)
	g.P(`if iNdEx >= l {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags}, ", g.Ident("io", `ErrUnexpectedEOF`))
	g.P(`}`)
	g.P(`b := dAtA[iNdEx]`)
	g.P(`iNdEx++`)
	g.P(varName, ` |= `, typName, `(b&0x7F) << shift`)
	g.P(`if b < 0x80 {`)
	g.P(`break`)
	g.P(`}`)
	g.P(`}`)
}

func (g *fastGenerator) unmarshalField(field *protogen.Field, message *protogen.Message, proto3 bool, required protoreflect.FieldNumbers) {
	fieldname := field.GoName
	errFieldname := fieldname
	if field.Oneof != nil && !field.Oneof.Desc.IsSynthetic() {
		fieldname = field.Oneof.GoName
	}

	g.P(`case `, strconv.Itoa(int(field.Desc.Number())), `:`)
	wireType := generator.ProtoWireType(field.Desc.Kind())
	if field.Desc.IsList() && wireType != protowire.BytesType {
		g.P(`if wireType == `, strconv.Itoa(int(wireType)), `{`)
		g.fieldItem(field, fieldname, message, false)
		g.P(`} else if wireType == `, strconv.Itoa(int(protowire.BytesType)), `{`)
		g.P(`var packedLen int`)
		g.decodeVarint("packedLen", "int")
		g.P(`if packedLen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postIndex := iNdEx + packedLen`)
		g.P(`if postIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", "ErrUnexpectedEOF"))
		g.P(`}`)

		g.P(`var elementCount int`)
		switch field.Desc.Kind() {
		case protoreflect.DoubleKind, protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
			g.P(`elementCount = packedLen/`, 8)
		case protoreflect.FloatKind, protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
			g.P(`elementCount = packedLen/`, 4)
		case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Int32Kind, protoreflect.Uint32Kind, protoreflect.Sint32Kind, protoreflect.Sint64Kind:
			g.P(`var count int`)
			g.P(`for _, integer := range dAtA[iNdEx:postIndex] {`)
			g.P(`if integer < 128 {`)
			g.P(`count++`)
			g.P(`}`)
			g.P(`}`)
			g.P(`elementCount = count`)
		case protoreflect.BoolKind:
			g.P(`elementCount = packedLen`)
		}

		g.P(`if elementCount != 0 && len(x.`, fieldname, `) == 0 {`)

		fieldtyp, _ := g.FieldGoType(field)
		g.P(`x.`, fieldname, ` = make(`, fieldtyp, `, 0, elementCount)`)
		g.P(`}`)

		g.P(`for iNdEx < postIndex {`)
		g.fieldItem(field, fieldname, message, false)
		g.P(`}`)
		g.P(`} else {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("fmt", "Errorf"), `("proto: wrong wireType = %d for field `, errFieldname, `", wireType)`)
		g.P(`}`)
	} else {
		g.P(`if wireType != `, strconv.Itoa(int(wireType)), `{`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("fmt", "Errorf"), `("proto: wrong wireType = %d for field `, errFieldname, `", wireType)`)
		g.P(`}`)
		g.fieldItem(field, fieldname, message, proto3)
	}

	if field.Desc.Cardinality() == protoreflect.Required {
		var fieldBit int
		for fieldBit = 0; fieldBit < required.Len(); fieldBit++ {
			if required.Get(fieldBit) == field.Desc.Number() {
				break
			}
		}
		if fieldBit == required.Len() {
			panic("missing required field")
		}
		g.P(`hasFields[`, strconv.Itoa(fieldBit/64), `] |= uint64(`, fmt.Sprintf("0x%08x", uint64(1)<<(fieldBit%64)), `)`)
	}
}

func (g *fastGenerator) fieldItem(field *protogen.Field, fieldname string, message *protogen.Message, proto3 bool) {
	repeated := field.Desc.Cardinality() == protoreflect.Repeated
	typ := g.noStarOrSliceType(field)
	oneof := field.Oneof != nil && !field.Oneof.Desc.IsSynthetic()
	nullable := field.Oneof != nil && field.Oneof.Desc.IsSynthetic()

	switch field.Desc.Kind() {
	case protoreflect.DoubleKind:
		g.P(`var v uint64`)
		g.decodeFixed64("v", "uint64")
		if oneof {
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{`, typ, "(", g.Ident("math", `Float64frombits`), `(v))}`)
		} else if repeated {
			g.P(`v2 := `, typ, "(", g.Ident("math", "Float64frombits"), `(v))`)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v2)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = `, typ, "(", g.Ident("math", "Float64frombits"), `(v))`)
		} else {
			g.P(`v2 := `, typ, "(", g.Ident("math", "Float64frombits"), `(v))`)
			g.P(`x.`, fieldname, ` = &v2`)
		}
	case protoreflect.FloatKind:
		g.P(`var v uint32`)
		g.decodeFixed32("v", "uint32")
		if oneof {
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{`, typ, "(", g.Ident("math", "Float32frombits"), `(v))}`)
		} else if repeated {
			g.P(`v2 := `, typ, "(", g.Ident("math", "Float32frombits"), `(v))`)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v2)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = `, typ, "(", g.Ident("math", "Float32frombits"), `(v))`)
		} else {
			g.P(`v2 := `, typ, "(", g.Ident("math", "Float32frombits"), `(v))`)
			g.P(`x.`, fieldname, ` = &v2`)
		}
	case protoreflect.Int64Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeVarint("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Uint64Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeVarint("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Int32Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeVarint("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Fixed64Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeFixed64("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Fixed32Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeFixed32("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.BoolKind:
		g.P(`var v int`)
		g.decodeVarint("v", "int")
		if oneof {
			g.P(`b := `, typ, `(v != 0)`)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{b}`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, `, typ, `(v != 0))`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = `, typ, `(v != 0)`)
		} else {
			g.P(`b := `, typ, `(v != 0)`)
			g.P(`x.`, fieldname, ` = &b`)
		}
	case protoreflect.StringKind:
		g.P(`var stringLen uint64`)
		g.decodeVarint("stringLen", "uint64")
		g.P(`intStringLen := int(stringLen)`)
		g.P(`if intStringLen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postIndex := iNdEx + intStringLen`)
		g.P(`if postIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		if oneof {
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{`, typ, `(dAtA[iNdEx:postIndex])}`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, `, typ, `(dAtA[iNdEx:postIndex]))`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = `, typ, `(dAtA[iNdEx:postIndex])`)
		} else {
			g.P(`s := `, typ, `(dAtA[iNdEx:postIndex])`)
			g.P(`x.`, fieldname, ` = &s`)
		}
		g.P(`iNdEx = postIndex`)
	case protoreflect.GroupKind:
		panic(fmt.Errorf("unmarshaler does not support group %v", fieldname))
	case protoreflect.MessageKind:
		g.P(`var msglen int`)
		g.decodeVarint("msglen", "int")
		g.P(`if msglen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postIndex := iNdEx + msglen`)
		g.P(`if postIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		if oneof {
			buf := `dAtA[iNdEx:postIndex]`
			msgname := g.noStarOrSliceType(field)
			g.P(`v := &`, msgname, `{}`)
			g.decodeMessage("v", buf, field.Message)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)

		} else if field.Desc.IsMap() {
			goTyp, _ := g.FieldGoType(field)
			goTypK, _ := g.FieldGoType(field.Message.Fields[0])
			goTypV, _ := g.FieldGoType(field.Message.Fields[1])

			g.P(`if x.`, fieldname, ` == nil {`)
			g.P(`x.`, fieldname, ` = make(`, goTyp, `)`)
			g.P(`}`)

			g.P("var mapkey ", goTypK)
			g.P("var mapvalue ", goTypV)
			g.P(`for iNdEx < postIndex {`)

			g.P(`entryPreIndex := iNdEx`)
			g.P(`var wire uint64`)
			g.decodeVarint("wire", "uint64")
			g.P(`fieldNum := int32(wire >> 3)`)

			g.P(`if fieldNum == 1 {`)
			g.unmarshalMapField("mapkey", field.Message.Fields[0])
			g.P(`} else if fieldNum == 2 {`)
			g.unmarshalMapField("mapvalue", field.Message.Fields[1])
			g.P(`} else {`)
			g.P(`iNdEx = entryPreIndex`)
			g.P(`skippy, err := `, runtimePackage.Ident("Skip"), `(dAtA[iNdEx:])`)
			g.P(`if err != nil {`)
			g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", `err`)
			g.P(`}`)
			g.P(`if (skippy < 0) || (iNdEx + skippy) < 0 {`)
			g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
			g.P(`}`)
			g.P(`if (iNdEx + skippy) > postIndex {`)
			g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
			g.P(`}`)
			g.P(`iNdEx += skippy`)
			g.P(`}`)
			g.P(`}`)
			g.P(`x.`, fieldname, `[mapkey] = mapvalue`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, &`, field.Message.GoIdent, `{})`)

			varname := fmt.Sprintf("x.%s[len(x.%s) - 1]", fieldname, fieldname)
			buf := `dAtA[iNdEx:postIndex]`
			g.decodeMessage(varname, buf, field.Message)
		} else {
			g.P(`if x.`, fieldname, ` == nil {`)
			g.P(`x.`, fieldname, ` = &`, field.Message.GoIdent, `{}`)
			g.P(`}`)
			g.decodeMessage("x."+fieldname, "dAtA[iNdEx:postIndex]", field.Message)
		}
		g.P(`iNdEx = postIndex`)

	case protoreflect.BytesKind:
		g.P(`var byteLen int`)
		g.decodeVarint("byteLen", "int")
		g.P(`if byteLen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postIndex := iNdEx + byteLen`)
		g.P(`if postIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		if oneof {
			g.P(`v := make([]byte, postIndex-iNdEx)`)
			g.P(`copy(v, dAtA[iNdEx:postIndex])`)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, make([]byte, postIndex-iNdEx))`)
			g.P(`copy(x.`, fieldname, `[len(x.`, fieldname, `)-1], dAtA[iNdEx:postIndex])`)
		} else {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `[:0] , dAtA[iNdEx:postIndex]...)`)
			g.P(`if x.`, fieldname, ` == nil {`)
			g.P(`x.`, fieldname, ` = []byte{}`)
			g.P(`}`)
		}
		g.P(`iNdEx = postIndex`)
	case protoreflect.Uint32Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeVarint("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.EnumKind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeVarint("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeVarint("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Sfixed32Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeFixed32("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeFixed32("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Sfixed64Kind:
		if oneof {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = 0`)
			g.decodeFixed64("x."+fieldname, typ)
		} else {
			g.P(`var v `, typ)
			g.decodeFixed64("v", typ)
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Sint32Kind:
		g.P(`var v `, typ)
		g.decodeVarint("v", typ)
		g.P(`v = `, typ, `((uint32(v) >> 1) ^ uint32(((v&1)<<31)>>31))`)
		if oneof {
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{v}`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, v)`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = v`)
		} else {
			g.P(`x.`, fieldname, ` = &v`)
		}
	case protoreflect.Sint64Kind:
		g.P(`var v uint64`)
		g.decodeVarint("v", "uint64")
		g.P(`v = (v >> 1) ^ uint64((int64(v&1)<<63)>>63)`)
		if oneof {
			g.P(`x.`, fieldname, ` = &`, field.GoIdent, `{`, typ, `(v)}`)
		} else if repeated {
			g.P(`x.`, fieldname, ` = append(x.`, fieldname, `, `, typ, `(v))`)
		} else if proto3 && !nullable {
			g.P(`x.`, fieldname, ` = `, typ, `(v)`)
		} else {
			g.P(`v2 := `, typ, `(v)`)
			g.P(`x.`, fieldname, ` = &v2`)
		}
	default:
		panic("not implemented")
	}
}

func (g *fastGenerator) noStarOrSliceType(field *protogen.Field) string {
	typ, _ := g.FieldGoType(field)
	if typ[0] == '[' && typ[1] == ']' {
		typ = typ[2:]
	}
	if typ[0] == '*' {
		typ = typ[1:]
	}
	return typ
}

func (g *fastGenerator) decodeFixed64(varName string, typeName string) {
	g.P(`if (iNdEx+8) > l {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
	g.P(`}`)
	g.P(varName, ` = `, typeName, `(`, g.Ident("encoding/binary", "LittleEndian"), `.Uint64(dAtA[iNdEx:]))`)
	g.P(`iNdEx += 8`)
}

func (g *fastGenerator) decodeFixed32(varName string, typeName string) {
	g.P(`if (iNdEx+4) > l {`)
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
	g.P(`}`)
	g.P(varName, ` = `, typeName, `(`, g.Ident("encoding/binary", "LittleEndian"), `.Uint32(dAtA[iNdEx:]))`)
	g.P(`iNdEx += 4`)
}

func (g *fastGenerator) decodeMessage(varName, buf string, message *protogen.Message) {

	g.P("if err := options.Unmarshal(", buf, ", ", varName, "); err != nil {")
	g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", `err`)
	g.P(`}`)

}

func (g *fastGenerator) unmarshalMapField(varName string, field *protogen.Field) {
	switch field.Desc.Kind() {
	case protoreflect.DoubleKind:
		g.P(`var `, varName, `temp uint64`)
		g.decodeFixed64(varName+"temp", "uint64")
		g.P(varName, ` = `, g.Ident("math", "Float64frombits"), `(`, varName, `temp)`)
	case protoreflect.FloatKind:
		g.P(`var `, varName, `temp uint32`)
		g.decodeFixed32(varName+"temp", "uint32")
		g.P(varName, ` = `, g.Ident("math", "Float32frombits"), `(`, varName, `temp)`)
	case protoreflect.Int64Kind:
		g.decodeVarint(varName, "int64")
	case protoreflect.Uint64Kind:
		g.decodeVarint(varName, "uint64")
	case protoreflect.Int32Kind:
		g.decodeVarint(varName, "int32")
	case protoreflect.Fixed64Kind:
		g.decodeFixed64(varName, "uint64")
	case protoreflect.Fixed32Kind:
		g.decodeFixed32(varName, "uint32")
	case protoreflect.BoolKind:
		g.P(`var `, varName, `temp int`)
		g.decodeVarint(varName+"temp", "int")
		g.P(varName, ` = bool(`, varName, `temp != 0)`)
	case protoreflect.StringKind:
		g.P(`var stringLen`, varName, ` uint64`)
		g.decodeVarint("stringLen"+varName, "uint64")
		g.P(`intStringLen`, varName, ` := int(stringLen`, varName, `)`)
		g.P(`if intStringLen`, varName, ` < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postStringIndex`, varName, ` := iNdEx + intStringLen`, varName)
		g.P(`if postStringIndex`, varName, ` < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postStringIndex`, varName, ` > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		g.P(varName, ` = `, "string", `(dAtA[iNdEx:postStringIndex`, varName, `])`)
		g.P(`iNdEx = postStringIndex`, varName)
	case protoreflect.MessageKind:
		g.P(`var mapmsglen int`)
		g.decodeVarint("mapmsglen", "int")
		g.P(`if mapmsglen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postmsgIndex := iNdEx + mapmsglen`)
		g.P(`if postmsgIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postmsgIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		buf := `dAtA[iNdEx:postmsgIndex]`
		g.P(varName, ` = &`, g.noStarOrSliceType(field), `{}`)
		g.decodeMessage(varName, buf, field.Message)
		g.P(`iNdEx = postmsgIndex`)
	case protoreflect.BytesKind:
		g.P(`var mapbyteLen uint64`)
		g.decodeVarint("mapbyteLen", "uint64")
		g.P(`intMapbyteLen := int(mapbyteLen)`)
		g.P(`if intMapbyteLen < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`postbytesIndex := iNdEx + intMapbyteLen`)
		g.P(`if postbytesIndex < 0 {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", runtimePackage.Ident("ErrInvalidLength"))
		g.P(`}`)
		g.P(`if postbytesIndex > l {`)
		g.P(`return `, protoifacePkg.Ident("UnmarshalOutput"), "{NoUnkeyedLiterals: input.NoUnkeyedLiterals, Flags: input.Flags},", g.Ident("io", `ErrUnexpectedEOF`))
		g.P(`}`)
		g.P(varName, ` = make([]byte, mapbyteLen)`)
		g.P(`copy(`, varName, `, dAtA[iNdEx:postbytesIndex])`)
		g.P(`iNdEx = postbytesIndex`)
	case protoreflect.Uint32Kind:
		g.decodeVarint(varName, "uint32")
	case protoreflect.EnumKind:
		goTypV, _ := g.FieldGoType(field)
		g.decodeVarint(varName, goTypV)
	case protoreflect.Sfixed32Kind:
		g.decodeFixed32(varName, "int32")
	case protoreflect.Sfixed64Kind:
		g.decodeFixed64(varName, "int64")
	case protoreflect.Sint32Kind:
		g.P(`var `, varName, `temp int32`)
		g.decodeVarint(varName+"temp", "int32")
		g.P(varName, `temp = int32((uint32(`, varName, `temp) >> 1) ^ uint32(((`, varName, `temp&1)<<31)>>31))`)
		g.P(varName, ` = int32(`, varName, `temp)`)
	case protoreflect.Sint64Kind:
		g.P(`var `, varName, `temp uint64`)
		g.decodeVarint(varName+"temp", "uint64")
		g.P(varName, `temp = (`, varName, `temp >> 1) ^ uint64((int64(`, varName, `temp&1)<<63)>>63)`)
		g.P(varName, ` = int64(`, varName, `temp)`)
	}
}
