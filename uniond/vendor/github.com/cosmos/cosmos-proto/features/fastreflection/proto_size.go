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

var kindToGoType = map[protoreflect.Kind]string{
	protoreflect.BoolKind:     "bool",
	protoreflect.EnumKind:     "Enumeration",
	protoreflect.Int32Kind:    "int32",
	protoreflect.Sint32Kind:   "int32",
	protoreflect.Uint32Kind:   "uint32",
	protoreflect.Int64Kind:    "int64",
	protoreflect.Sint64Kind:   "int64",
	protoreflect.Uint64Kind:   "uint64",
	protoreflect.Sfixed32Kind: "int32",
	protoreflect.Fixed32Kind:  "uint32",
	protoreflect.FloatKind:    "float32",
	protoreflect.Sfixed64Kind: "int64",
	protoreflect.Fixed64Kind:  "uint64",
	protoreflect.DoubleKind:   "float64",
	protoreflect.StringKind:   "string",
	protoreflect.BytesKind:    "byte",
}

func (g *fastGenerator) genSizeMethod() {

	g.P(`size := func(input `, protoifacePkg.Ident("SizeInput"), ") ", protoifacePkg.Ident("SizeOutput"), " {")
	g.P("x := input.Message.Interface().(*", g.message.GoIdent, ")")
	g.P(`if x == nil {`)
	g.P(`return `, protoifacePkg.Ident("SizeOutput"), "{ ")
	g.P("NoUnkeyedLiterals: input.NoUnkeyedLiterals,")
	g.P("Size: 0,")
	g.P("}")
	g.P(`}`)
	g.P("options := ", runtimePackage.Ident("SizeInputToOptions"), "(input)")
	g.P("_ = options")
	g.P(`var n int`)
	g.P(`var l int`)
	g.P(`_ = l`)
	oneofs := make(map[string]struct{})
	for _, field := range g.message.Fields {
		oneof := field.Oneof != nil && !field.Oneof.Desc.IsSynthetic()
		if !oneof {
			g.field(true, field, false)
		} else {
			fieldName := field.Oneof.GoName
			if _, ok := oneofs[fieldName]; !ok {
				oneofs[fieldName] = struct{}{}
				g.P("switch x := x.", fieldName, ".(type) {")
				for _, ooField := range field.Oneof.Fields {

					g.P("case *", ooField.GoIdent, ": ")
					g.P("if x == nil {")
					g.P("break")
					g.P("}")
					g.field(true, ooField, true)
				}
				g.P("}")
			}
		}
	}

	// last thing to do
	g.P(`if x.unknownFields != nil {`)
	g.P(`n+=len(x.unknownFields)`)
	g.P(`}`)
	g.P(`return `, protoifacePkg.Ident("SizeOutput"), "{ ")
	g.P("NoUnkeyedLiterals: input.NoUnkeyedLiterals,")
	g.P("Size: n,")
	g.P("}")
	g.P(`}`)
	g.P()
}

func (g *fastGenerator) field(proto3 bool, field *protogen.Field, oneof bool) {
	fieldname := field.GoName
	nullable := field.Message != nil || (field.Oneof != nil && field.Oneof.Desc.IsSynthetic())
	repeated := field.Desc.Cardinality() == protoreflect.Repeated
	if repeated && !oneof {
		g.P(`if len(x.`, fieldname, `) > 0 {`)
	} else if nullable && !oneof {
		g.P(`if x.`, fieldname, ` != nil {`)
	}

	packed := field.Desc.IsPacked()
	wireType := generator.ProtoWireType(field.Desc.Kind())
	fieldNumber := field.Desc.Number()
	if packed {
		wireType = protowire.BytesType
	}
	key := generator.KeySize(fieldNumber, wireType)
	switch field.Desc.Kind() {
	case protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
		if packed {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(len(x.`, fieldname, `)*8))`, `+len(x.`, fieldname, `)*8`)
		} else if repeated {
			g.P(`n+=`, strconv.Itoa(key+8), `*len(x.`, fieldname, `)`)
		} else if proto3 && !nullable {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key+8))
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key+8))
		}
	case protoreflect.DoubleKind:
		if packed {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(len(x.`, fieldname, `)*8))`, `+len(x.`, fieldname, `)*8`)
		} else if repeated {
			g.P(`n+=`, strconv.Itoa(key+8), `*len(x.`, fieldname, `)`)
		} else if proto3 && !nullable {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 || `, mathPackage.Ident("Signbit"), `(x.`, fieldname, `) {`)
			}
			g.P(`n+=`, strconv.Itoa(key+8))
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key+8))
		}
	case protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
		if packed {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(len(x.`, fieldname, `)*4))`, `+len(x.`, fieldname, `)*4`)
		} else if repeated {
			g.P(`n+=`, strconv.Itoa(key+4), `*len(x.`, fieldname, `)`)
		} else if proto3 && !nullable {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key+4))
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key+4))
		}
	case protoreflect.FloatKind:
		if packed {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(len(x.`, fieldname, `)*4))`, `+len(x.`, fieldname, `)*4`)
		} else if repeated {
			g.P(`n+=`, strconv.Itoa(key+4), `*len(x.`, fieldname, `)`)
		} else if proto3 && !nullable {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 || `, mathPackage.Ident("Signbit"), `(float64(x.`, fieldname, `)) {`)
			}
			g.P(`n+=`, strconv.Itoa(key+4))
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key+4))
		}
	case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Uint32Kind, protoreflect.EnumKind, protoreflect.Int32Kind:
		if packed {
			g.P(`l = 0`)
			g.P(`for _, e := range x.`, fieldname, ` {`)
			g.P(`l+=`, runtimePackage.Ident("Sov"), `(uint64(e))`)
			g.P(`}`)
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(l))+l`)
		} else if repeated {
			g.P(`for _, e := range x.`, fieldname, ` {`)
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(e))`)
			g.P(`}`)
		} else if nullable {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(*x.`, fieldname, `))`)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(x.`, fieldname, `))`)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(x.`, fieldname, `))`)
		}
	case protoreflect.BoolKind:
		if packed {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(len(x.`, fieldname, `)))`, `+len(x.`, fieldname, `)*1`)
		} else if repeated {
			g.P(`n+=`, strconv.Itoa(key+1), `*len(x.`, fieldname, `)`)
		} else if proto3 && !nullable {
			if !oneof {
				g.P(`if x.`, fieldname, ` {`)
			}
			g.P(`n+=`, strconv.Itoa(key+1))
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`n+=`, strconv.Itoa(key+1))
		}
	case protoreflect.StringKind:
		if repeated {
			g.P(`for _, s := range x.`, fieldname, ` { `)
			g.P(`l = len(s)`)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
			g.P(`}`)
		} else if nullable {
			g.P(`l=len(*x.`, fieldname, `)`)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
		} else if proto3 {
			g.P(`l=len(x.`, fieldname, `)`)
			if !oneof {
				g.P(`if l > 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`l=len(x.`, fieldname, `)`)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
		}
	case protoreflect.GroupKind:
		panic(fmt.Errorf("size does not support group %v", fieldname))
	case protoreflect.MessageKind:
		if field.Desc.IsMap() {
			fieldKeySize := generator.KeySize(field.Desc.Number(), generator.ProtoWireType(field.Desc.Kind()))
			goTypeK, _ := g.FieldGoType(field.Message.Fields[0])
			goTypeV, ptr := g.FieldGoType(field.Message.Fields[1])
			if ptr {
				goTypeV = "*" + goTypeV
			}
			keyKeySize := generator.KeySize(1, generator.ProtoWireType(field.Message.Fields[0].Desc.Kind()))
			valueKeySize := generator.KeySize(2, generator.ProtoWireType(field.Message.Fields[1].Desc.Kind()))

			sum := []string{strconv.Itoa(keyKeySize)}
			g.P("SiZeMaP := func(k ", goTypeK, ", v ", goTypeV, ") {")
			switch field.Desc.MapKey().Kind() {
			case protoreflect.DoubleKind, protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
				sum = append(sum, `8`)
			case protoreflect.FloatKind, protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
				sum = append(sum, `4`)
			case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Uint32Kind, protoreflect.EnumKind, protoreflect.Int32Kind:
				sum = append(sum, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Sov")), `(uint64(k))`))
			case protoreflect.BoolKind:
				sum = append(sum, `1`)
			case protoreflect.StringKind, protoreflect.BytesKind:
				sum = append(sum, `len(k)`, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Sov")), `(uint64(len(k)))`))
			case protoreflect.Sint32Kind, protoreflect.Sint64Kind:
				sum = append(sum, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Soz")), `(uint64(k))`))
			}

			switch field.Desc.MapValue().Kind() {
			case protoreflect.DoubleKind, protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, strconv.Itoa(8))
			case protoreflect.FloatKind, protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, strconv.Itoa(4))
			case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Uint32Kind, protoreflect.EnumKind, protoreflect.Int32Kind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Sov")), `(uint64(v))`))
			case protoreflect.BoolKind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, `1`)
			case protoreflect.StringKind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, `len(v)`, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Sov")), `(uint64(len(v)))`))
			case protoreflect.BytesKind:
				g.P(`l = `, strconv.Itoa(valueKeySize), ` + len(v)+`, runtimePackage.Ident("Sov"), `(uint64(len(v)))`)
				sum = append(sum, `l`)
			case protoreflect.Sint32Kind, protoreflect.Sint64Kind:
				sum = append(sum, strconv.Itoa(valueKeySize))
				sum = append(sum, fmt.Sprintf("%s%s", g.QualifiedGoIdent(runtimePackage.Ident("Soz")), `(uint64(v))`))
			case protoreflect.MessageKind:
				g.P(`l := 0`)
				g.P(`if v != nil {`)
				g.messageSize("v", field.Message.Fields[1].Message)
				g.P(`}`)
				g.P(`l += `, strconv.Itoa(valueKeySize), `+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
				sum = append(sum, `l`)
			}
			g.P(`mapEntrySize := `, strings.Join(sum, "+"))
			g.P(`n+=mapEntrySize+`, fieldKeySize, `+`, runtimePackage.Ident("Sov"), `(uint64(mapEntrySize))`)
			g.P("}")
			// first we have to sort the key
			typ, ok := kindToGoType[field.Desc.MapKey().Kind()]
			if !ok {
				panic(fmt.Sprintf("pulsar does not support %s types as map keys", field.Desc.MapKey().Kind().String()))
			}
			g.P("if options.Deterministic {")
			g.P("sortme := make([]", typ, ", 0, len(x.", field.GoName, "))")
			g.P("for k := range x.", fieldname, " {")
			g.P("sortme = append(sortme, k)")
			g.P("}")
			switch field.Desc.MapKey().Kind() {
			case protoreflect.StringKind:
				g.P(sortPkg.Ident("Strings"), "(sortme)")
			default:
				g.P(sortPkg.Ident("Slice"), "(sortme, func(i, j int) bool {")
				switch field.Desc.MapKey().Kind() {
				case protoreflect.BoolKind:
					g.P("return !sortme[i] && sortme[j]")
				default:
					g.P("return sortme[i] < sortme[j]")
				}
				g.P("})")

			}

			g.P(`for _, k := range sortme {`)
			g.P("v := x.", fieldname, "[k]")
			g.P("SiZeMaP(k,v)")
			g.P(`}`)
			g.P("} else {")
			g.P("for k,v := range x.", fieldname, " {")
			g.P("SiZeMaP(k,v)")
			g.P("}")
			g.P("}")
		} else if field.Desc.IsList() {
			g.P(`for _, e := range x.`, fieldname, ` { `)
			g.messageSize("e", field.Message)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
			g.P(`}`)
		} else {
			g.messageSize("x."+fieldname, field.Message)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
		}
	case protoreflect.BytesKind:
		if repeated {
			g.P(`for _, b := range x.`, fieldname, ` { `)
			g.P(`l = len(b)`)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
			g.P(`}`)
		} else if proto3 {
			g.P(`l=len(x.`, fieldname, `)`)
			if !oneof {
				g.P(`if l > 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`l=len(x.`, fieldname, `)`)
			g.P(`n+=`, strconv.Itoa(key), `+l+`, runtimePackage.Ident("Sov"), `(uint64(l))`)
		}
	case protoreflect.Sint32Kind, protoreflect.Sint64Kind:
		if packed {
			g.P(`l = 0`)
			g.P(`for _, e := range x.`, fieldname, ` {`)
			g.P(`l+=`, runtimePackage.Ident("Soz"), `(uint64(e))`)
			g.P(`}`)
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Sov"), `(uint64(l))+l`)
		} else if repeated {
			g.P(`for _, e := range x.`, fieldname, ` {`)
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Soz"), `(uint64(e))`)
			g.P(`}`)
		} else if nullable {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Soz"), `(uint64(*x.`, fieldname, `))`)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Soz"), `(uint64(x.`, fieldname, `))`)
			g.P(`}`)
		} else {
			g.P(`n+=`, strconv.Itoa(key), `+`, runtimePackage.Ident("Soz"), `(uint64(x.`, fieldname, `))`)
		}
	default:
		panic("not implemented")
	}
	if (repeated || nullable) && !oneof {
		g.P(`}`)
	}
}

func (g *fastGenerator) messageSize(varName string, message *protogen.Message) {
	g.P(`l = options.Size(`, varName, `)`)
}
