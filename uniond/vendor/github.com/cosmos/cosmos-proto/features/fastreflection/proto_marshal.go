package fastreflection

import (
	"fmt"
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/encoding/protowire"
	"google.golang.org/protobuf/reflect/protoreflect"
	"sort"
	"strconv"
	"strings"
)

type counter int

func (cnt *counter) Next() string {
	*cnt++
	return cnt.Current()
}

func (cnt *counter) Current() string {
	return strconv.Itoa(int(*cnt))
}

func (g *fastGenerator) genMarshalMethod() {

	var numGen counter
	// MARSHAL METHOD
	g.P(`marshal := func(input `, protoifacePkg.Ident("MarshalInput"), `) (`, protoifacePkg.Ident("MarshalOutput"), `, error) {`)

	// setup
	g.P(`x := input.Message.Interface().(*`, g.message.GoIdent, `)`)
	g.P("if x == nil {")
	g.P(`return `, protoifacePkg.Ident("MarshalOutput"), `{`)
	g.P(`		NoUnkeyedLiterals: input.NoUnkeyedLiterals,`)
	g.P(`		Buf: input.Buf,`)
	g.P("}, nil")
	g.P("}")

	// core
	g.P("options := ", runtimePackage.Ident("MarshalInputToOptions"), "(input)")
	g.P("_ = options")
	g.P("size := options.Size(x)")
	g.P(`dAtA := make([]byte, size)`)

	// from here we need to do what MarshalToSizedBuffer was doing
	g.P("i := len(dAtA)")
	g.P("_ = i")
	g.P("var l int")
	g.P("_ = l")
	g.P("if x.unknownFields != nil {")
	g.P("i -= len(x.unknownFields)")
	g.P("copy(dAtA[i:], x.unknownFields)")
	g.P("}")

	// oneofs MUST be marshalled first!
	oneofs := make(map[string]struct{})
	for i := len(g.message.Oneofs) - 1; i >= 0; i-- {
		field := g.message.Oneofs[i]
		fieldname := field.GoName
		if _, ok := oneofs[fieldname]; !ok {
			oneofs[fieldname] = struct{}{}
			g.P("switch x := x.", fieldname, ".(type) {")
			for _, ooField := range field.Fields {
				g.P("case *", ooField.GoIdent, ": ")
				g.marshalField(true, &numGen, ooField, true)
			}
			g.P("}")
		}
	}

	// here we deep copy the message.Fields slice, since it can corrupt the order of fields
	// causing issues for plugins that depend on the order
	messageFields := make([]*protogen.Field, len(g.message.Fields))
	for i := range g.message.Fields {
		messageFields[i] = g.message.Fields[i]
	}
	sort.Slice(messageFields, func(i, j int) bool {
		return messageFields[i].Desc.Number() < messageFields[j].Desc.Number()
	})

	// then we do everything else
	for i := len(messageFields) - 1; i >= 0; i-- {
		field := messageFields[i]
		isOneof := field.Oneof != nil && !field.Oneof.Desc.IsSynthetic()
		if !isOneof {
			g.marshalField(true, &numGen, field, false)
		}
	}

	g.P("if input.Buf != nil {")
	g.P(`input.Buf = append(input.Buf, dAtA...)`)
	g.P("} else {")
	g.P("input.Buf = dAtA")
	g.P("}")
	g.P(`return `, protoifacePkg.Ident("MarshalOutput"), `{`)
	g.P(`		NoUnkeyedLiterals: input.NoUnkeyedLiterals,`)
	g.P(`		Buf: input.Buf,`)
	g.P("}, nil")
	g.P("}")
}

func (g *fastGenerator) marshalField(proto3 bool, numGen *counter, field *protogen.Field, oneof bool) {
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
	switch field.Desc.Kind() {
	case protoreflect.DoubleKind:
		if packed {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`f`, numGen.Next(), ` := `, g.Ident("math", "Float64bits"), `(float64(`, val, `))`)
			g.encodeFixed64("f", numGen.Current())
			g.P(`}`)
			g.encodeVarint(`len(x.`, fieldname, `) * 8`)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`f`, numGen.Next(), ` := `, g.Ident("math", "Float64bits"), `(float64(`, val, `))`)
			g.encodeFixed64("f", numGen.Current())
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeFixed64(g.Ident("math", "Float64bits"), `(float64(*x.`+fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 || `, mathPackage.Ident("Signbit"), `(x.`, fieldname, `) {`)
			}
			g.encodeFixed64(g.Ident("math", "Float64bits"), `(float64(x.`, fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeFixed64(g.Ident("math", "Float64bits"), `(float64(x.`+fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.FloatKind:
		if packed {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`f`, numGen.Next(), ` := `, g.Ident("math", "Float32bits"), `(float32(`, val, `))`)
			g.encodeFixed32("f" + numGen.Current())
			g.P(`}`)
			g.encodeVarint(`len(x.`, fieldname, `) * 4`)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`f`, numGen.Next(), ` := `, g.Ident("math", "Float32bits"), `(float32(`, val, `))`)
			g.encodeFixed32("f" + numGen.Current())
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeFixed32(g.Ident("math", "Float32bits"), `(float32(*x.`+fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 || `, mathPackage.Ident("Signbit"), `(float64(x.`, fieldname, `)) {`)
			}
			g.encodeFixed32(g.Ident("math", "Float32bits"), `(float32(x.`+fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeFixed32(g.Ident("math", "Float32bits"), `(float32(x.`+fieldname, `))`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Int32Kind, protoreflect.Uint32Kind, protoreflect.EnumKind:
		if packed {
			jvar := "j" + numGen.Next()
			total := "pksize" + numGen.Next()

			g.P(`var `, total, ` int`)
			g.P(`for _, num := range x.`, fieldname, ` {`)
			g.P(total, ` += `, runtimePackage.Ident("Sov"), `(uint64(num))`)
			g.P(`}`)

			g.P(`i -= `, total)
			g.P(jvar, `:= i`)

			switch field.Desc.Kind() {
			case protoreflect.Int64Kind, protoreflect.Int32Kind, protoreflect.EnumKind:
				g.P(`for _, num1 := range x.`, fieldname, ` {`)
				g.P(`num := uint64(num1)`)
			default:
				g.P(`for _, num := range x.`, fieldname, ` {`)
			}
			g.P(`for num >= 1<<7 {`)
			g.P(`dAtA[`, jvar, `] = uint8(uint64(num)&0x7f|0x80)`)
			g.P(`num >>= 7`)
			g.P(jvar, `++`)
			g.P(`}`)
			g.P(`dAtA[`, jvar, `] = uint8(num)`)
			g.P(jvar, `++`)
			g.P(`}`)

			g.encodeVarint(total)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.encodeVarint(val)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeVarint(`*x.`, fieldname)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.encodeVarint(`x.`, fieldname)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeVarint(`x.`, fieldname)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
		if packed {
			val := g.reverseListRange(`x.`, fieldname)
			g.encodeFixed64(val)
			g.P(`}`)
			g.encodeVarint(`len(x.`, fieldname, `) * 8`)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.encodeFixed64(val)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeFixed64("*x.", fieldname)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.encodeFixed64("x.", fieldname)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeFixed64("x.", fieldname)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
		if packed {
			val := g.reverseListRange(`x.`, fieldname)
			g.encodeFixed32(val)
			g.P(`}`)
			g.encodeVarint(`len(x.`, fieldname, `) * 4`)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.encodeFixed32(val)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeFixed32("*x." + fieldname)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.encodeFixed32("x." + fieldname)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeFixed32("x." + fieldname)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.BoolKind:
		if packed {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`i--`)
			g.P(`if `, val, ` {`)
			g.P(`dAtA[i] = 1`)
			g.P(`} else {`)
			g.P(`dAtA[i] = 0`)
			g.P(`}`)
			g.P(`}`)
			g.encodeVarint(`len(x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`i--`)
			g.P(`if `, val, ` {`)
			g.P(`dAtA[i] = 1`)
			g.P(`} else {`)
			g.P(`dAtA[i] = 0`)
			g.P(`}`)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.P(`i--`)
			g.P(`if *x.`, fieldname, ` {`)
			g.P(`dAtA[i] = 1`)
			g.P(`} else {`)
			g.P(`dAtA[i] = 0`)
			g.P(`}`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` {`)
			}
			g.P(`i--`)
			g.P(`if x.`, fieldname, ` {`)
			g.P(`dAtA[i] = 1`)
			g.P(`} else {`)
			g.P(`dAtA[i] = 0`)
			g.P(`}`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`i--`)
			g.P(`if x.`, fieldname, ` {`)
			g.P(`dAtA[i] = 1`)
			g.P(`} else {`)
			g.P(`dAtA[i] = 0`)
			g.P(`}`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.StringKind:
		if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`i -= len(`, val, `)`)
			g.P(`copy(dAtA[i:], `, val, `)`)
			g.encodeVarint(`len(`, val, `)`)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.P(`i -= len(*x.`, fieldname, `)`)
			g.P(`copy(dAtA[i:], *x.`, fieldname, `)`)
			g.encodeVarint(`len(*x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if len(x.`, fieldname, `) > 0 {`)
			}
			g.P(`i -= len(x.`, fieldname, `)`)
			g.P(`copy(dAtA[i:], x.`, fieldname, `)`)
			g.encodeVarint(`len(x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`i -= len(x.`, fieldname, `)`)
			g.P(`copy(dAtA[i:], x.`, fieldname, `)`)
			g.encodeVarint(`len(x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.GroupKind:
		panic(fmt.Errorf("marshaler does not support group %v", fieldname))
	case protoreflect.MessageKind:
		if field.Desc.IsMap() {
			goTypK, _ := g.FieldGoType(field.Message.Fields[0])
			goTypV, ptr := g.FieldGoType(field.Message.Fields[1])
			if ptr {
				goTypV = "*" + goTypV
			}
			keyKind := field.Message.Fields[0].Desc.Kind()
			valKind := field.Message.Fields[1].Desc.Kind()

			_, ok := kindToGoType[keyKind]
			if !ok {
				panic(fmt.Sprintf("pulsar does not support %s types as map keys", field.Desc.MapKey().Kind().String()))
			}

			g.P("MaRsHaLmAp := func(k ", goTypK, ", v ", goTypV, ") (", protoifacePkg.Ident("MarshalOutput"), ", error) {")
			g.P(`baseI := i`)
			accessor := `v`
			g.mapField(field.Message.Fields[1], accessor)
			g.encodeKey(2, generator.ProtoWireType(valKind))

			g.mapField(field.Message.Fields[0], "k")
			g.encodeKey(1, generator.ProtoWireType(keyKind))
			g.encodeVarint(`baseI - i`)
			g.encodeKey(fieldNumber, wireType)
			g.P("return ", protoifacePkg.Ident("MarshalOutput"), "{}, nil")
			g.P("}")

			var val string
			g.P("if options.Deterministic {")
			keysName := `keysFor` + fieldname
			g.P(keysName, ` := make([]`, goTypK, `, 0, len(x.`, fieldname, `))`)
			g.P(`for k := range x.`, fieldname, ` {`)
			g.P(keysName, ` = append(`, keysName, `, `, goTypK, `(k))`)
			g.P(`}`)
			g.P(g.Ident("sort", "Slice"), `(`, keysName, `, func(i, j int) bool {`)
			switch keyKind {
			case protoreflect.BoolKind:
				g.P("return !", keysName, "[i] && ", keysName, "[j]")
			default:
				g.P(`return `, keysName, `[i] < `, keysName, `[j]`)
			}
			g.P(`})`)
			val = g.reverseListRange(keysName)
			g.P(`v := x.`, fieldname, `[`, goTypK, `(`, val, `)]`)
			g.P("out, err := MaRsHaLmAp(", val, ", v)")
			g.P("if err != nil {")
			g.P("return out, err")
			g.P("}")
			g.P("}")
			g.P("} else {")

			g.P(`for k := range x.`, fieldname, ` {`)
			val = "k"
			g.P(`v := x.`, fieldname, `[`, val, `]`)
			g.P("out, err := MaRsHaLmAp(k,v)")
			g.P("if err != nil {")
			g.P("return out, err")
			g.P("}")
			g.P("}")
			g.P("}")
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.marshalBackward(val, true, field.Message)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else {
			g.marshalBackward(`x.`+fieldname, true, field.Message)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.BytesKind:
		if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`i -= len(`, val, `)`)
			g.P(`copy(dAtA[i:], `, val, `)`)
			g.encodeVarint(`len(`, val, `)`)
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if proto3 {
			if !oneof {
				g.P(`if len(x.`, fieldname, `) > 0 {`)
			}
			g.P(`i -= len(x.`, fieldname, `)`)
			g.P(`copy(dAtA[i:], x.`, fieldname, `)`)
			g.encodeVarint(`len(x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.P(`i -= len(x.`, fieldname, `)`)
			g.P(`copy(dAtA[i:], x.`, fieldname, `)`)
			g.encodeVarint(`len(x.`, fieldname, `)`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.Sint32Kind:
		if packed {
			jvar := "j" + numGen.Next()
			total := "pksize" + numGen.Next()

			g.P(`var `, total, ` int`)
			g.P(`for _, num := range x.`, fieldname, ` {`)
			g.P(total, ` += `, runtimePackage.Ident("Soz"), `(uint64(num))`)
			g.P(`}`)
			g.P(`i -= `, total)
			g.P(jvar, `:= i`)

			g.P(`for _, num := range x.`, fieldname, ` {`)
			xvar := "x" + numGen.Next()
			g.P(xvar, ` := (uint32(num) << 1) ^ uint32((num >> 31))`)
			g.P(`for `, xvar, ` >= 1<<7 {`)
			g.P(`dAtA[`, jvar, `] = uint8(uint64(`, xvar, `)&0x7f|0x80)`)
			g.P(jvar, `++`)
			g.P(xvar, ` >>= 7`)
			g.P(`}`)
			g.P(`dAtA[`, jvar, `] = uint8(`, xvar, `)`)
			g.P(jvar, `++`)
			g.P(`}`)

			g.encodeVarint(total)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`x`, numGen.Next(), ` := (uint32(`, val, `) << 1) ^ uint32((`, val, ` >> 31))`)
			g.encodeVarint(`x`, numGen.Current())
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeVarint(`(uint32(*x.`, fieldname, `) << 1) ^ uint32((*x.`, fieldname, ` >> 31))`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.encodeVarint(`(uint32(x.`, fieldname, `) << 1) ^ uint32((x.`, fieldname, ` >> 31))`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeVarint(`(uint32(x.`, fieldname, `) << 1) ^ uint32((x.`, fieldname, ` >> 31))`)
			g.encodeKey(fieldNumber, wireType)
		}
	case protoreflect.Sint64Kind:
		if packed {
			jvar := "j" + numGen.Next()
			total := "pksize" + numGen.Next()

			g.P(`var `, total, ` int`)
			g.P(`for _, num := range x.`, fieldname, ` {`)
			g.P(total, ` += `, runtimePackage.Ident("Soz"), `(uint64(num))`)
			g.P(`}`)
			g.P(`i -= `, total)
			g.P(jvar, `:= i`)

			g.P(`for _, num := range x.`, fieldname, ` {`)
			xvar := "x" + numGen.Next()
			g.P(xvar, ` := (uint64(num) << 1) ^ uint64((num >> 63))`)
			g.P(`for `, xvar, ` >= 1<<7 {`)
			g.P(`dAtA[`, jvar, `] = uint8(uint64(`, xvar, `)&0x7f|0x80)`)
			g.P(jvar, `++`)
			g.P(xvar, ` >>= 7`)
			g.P(`}`)
			g.P(`dAtA[`, jvar, `] = uint8(`, xvar, `)`)
			g.P(jvar, `++`)
			g.P(`}`)

			g.encodeVarint(total)
			g.encodeKey(fieldNumber, wireType)
		} else if repeated {
			val := g.reverseListRange(`x.`, fieldname)
			g.P(`x`, numGen.Next(), ` := (uint64(`, val, `) << 1) ^ uint64((`, val, ` >> 63))`)
			g.encodeVarint("x" + numGen.Current())
			g.encodeKey(fieldNumber, wireType)
			g.P(`}`)
		} else if nullable {
			g.encodeVarint(`(uint64(*x.`, fieldname, `) << 1) ^ uint64((*x.`, fieldname, ` >> 63))`)
			g.encodeKey(fieldNumber, wireType)
		} else if proto3 {
			if !oneof {
				g.P(`if x.`, fieldname, ` != 0 {`)
			}
			g.encodeVarint(`(uint64(x.`, fieldname, `) << 1) ^ uint64((x.`, fieldname, ` >> 63))`)
			g.encodeKey(fieldNumber, wireType)
			if !oneof {
				g.P(`}`)
			}
		} else {
			g.encodeVarint(`(uint64(x.`, fieldname, `) << 1) ^ uint64((x.`, fieldname, ` >> 63))`)
			g.encodeKey(fieldNumber, wireType)
		}
	default:
		panic("not implemented")
	}
	if (repeated || nullable) && !oneof {
		g.P(`}`)
	}
}

func (g *fastGenerator) marshalBackward(varName string, varInt bool, message *protogen.Message) {
	g.P(`encoded, err := `, "options.Marshal(", varName, ")")
	g.P(`if err != nil {`)
	g.P(`return `, protoifacePkg.Ident("MarshalOutput"), " {")
	g.P("NoUnkeyedLiterals: input.NoUnkeyedLiterals,")
	g.P("Buf: input.Buf,")
	g.P("}, err")
	g.P(`}`)
	g.P(`i -= len(encoded)`)
	g.P(`copy(dAtA[i:], encoded)`)
	if varInt {
		g.encodeVarint(`len(encoded)`)
	}
}

func (g *fastGenerator) reverseListRange(expression ...string) string {
	exp := strings.Join(expression, "")
	g.P(`for iNdEx := len(`, exp, `) - 1; iNdEx >= 0; iNdEx-- {`)
	return exp + `[iNdEx]`
}

func (g *fastGenerator) encodeFixed64(varName ...string) {
	g.P(`i -= 8`)
	g.P(g.Ident("encoding/binary", "LittleEndian"), `.PutUint64(dAtA[i:], uint64(`, strings.Join(varName, ""), `))`)
}

func (g *fastGenerator) encodeFixed32(varName ...string) {
	g.P(`i -= 4`)
	g.P(g.Ident("encoding/binary", "LittleEndian"), `.PutUint32(dAtA[i:], uint32(`, strings.Join(varName, ""), `))`)
}

func (g *fastGenerator) encodeVarint(varName ...string) {
	g.P(`i = `, runtimePackage.Ident("EncodeVarint"), `(dAtA, i, uint64(`, strings.Join(varName, ""), `))`)
}

func (g *fastGenerator) encodeKey(fieldNumber protoreflect.FieldNumber, wireType protowire.Type) {
	x := uint32(fieldNumber)<<3 | uint32(wireType)
	i := 0
	keybuf := make([]byte, 0)
	for i = 0; x > 127; i++ {
		keybuf = append(keybuf, 0x80|uint8(x&0x7F))
		x >>= 7
	}
	keybuf = append(keybuf, uint8(x))
	for i = len(keybuf) - 1; i >= 0; i-- {
		g.P(`i--`)
		g.P(`dAtA[i] = `, fmt.Sprintf("%#v", keybuf[i]))
	}
}

func (g *fastGenerator) mapField(kvField *protogen.Field, varName string) {
	switch kvField.Desc.Kind() {
	case protoreflect.DoubleKind:
		g.encodeFixed64(g.Ident("math", "Float64bits"), `(float64(`, varName, `))`)
	case protoreflect.FloatKind:
		g.encodeFixed32(g.Ident("math", "Float32bits"), `(float32(`, varName, `))`)
	case protoreflect.Int64Kind, protoreflect.Uint64Kind, protoreflect.Int32Kind, protoreflect.Uint32Kind, protoreflect.EnumKind:
		g.encodeVarint(varName)
	case protoreflect.Fixed64Kind, protoreflect.Sfixed64Kind:
		g.encodeFixed64(varName)
	case protoreflect.Fixed32Kind, protoreflect.Sfixed32Kind:
		g.encodeFixed32(varName)
	case protoreflect.BoolKind:
		g.P(`i--`)
		g.P(`if `, varName, ` {`)
		g.P(`dAtA[i] = 1`)
		g.P(`} else {`)
		g.P(`dAtA[i] = 0`)
		g.P(`}`)
	case protoreflect.StringKind, protoreflect.BytesKind:
		g.P(`i -= len(`, varName, `)`)
		g.P(`copy(dAtA[i:], `, varName, `)`)
		g.encodeVarint(`len(`, varName, `)`)
	case protoreflect.Sint32Kind:
		g.encodeVarint(`(uint32(`, varName, `) << 1) ^ uint32((`, varName, ` >> 31))`)
	case protoreflect.Sint64Kind:
		g.encodeVarint(`(uint64(`, varName, `) << 1) ^ uint64((`, varName, ` >> 63))`)
	case protoreflect.MessageKind:
		g.marshalBackward(varName, true, kvField.Message)
	}
}
