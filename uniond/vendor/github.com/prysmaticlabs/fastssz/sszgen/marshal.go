package main

import (
	"fmt"
	"strings"
)

// marshal creates a function that encodes the structs in SSZ format. It creates two functions:
// 1. MarshalTo(dst []byte) marshals the content to the target array.
// 2. Marshal() marshals the content to a newly created array.
func (e *env) marshal(name string, v *Value) string {
	tmpl := `// MarshalSSZ ssz marshals the {{.name}} object
	func (:: *{{.name}}) MarshalSSZ() ([]byte, error) {
		return ssz.MarshalSSZ(::)
	}

	// MarshalSSZTo ssz marshals the {{.name}} object to a target array	
	func (:: *{{.name}}) MarshalSSZTo(buf []byte) (dst []byte, err error) {
		dst = buf
		{{.offset}}
		{{.marshal}}
		return
	}`

	data := map[string]interface{}{
		"name":    name,
		"marshal": v.marshalContainer(true),
		"offset":  "",
	}
	if !v.isFixed() {
		// offset is the position where the offset starts
		data["offset"] = fmt.Sprintf("offset := int(%d)\n", v.fixedSize())
	}
	str := execTmpl(tmpl, data)
	return appendObjSignature(str, v)
}

func (v *Value) marshal() string {
	switch v.t {
	case TypeContainer, TypeReference:
		return v.marshalContainer(false)

	case TypeBytes:
		name := v.name
		if v.c {
			name += "[:]"
		}
		tmpl := `{{.validate}}dst = append(dst, ::.{{.name}}...)`

		return execTmpl(tmpl, map[string]interface{}{
			"validate": v.validate(),
			"name":     name,
		})

	case TypeUint:
		var name string
		if v.ref != "" || v.obj != "" {
			// alias to Uint64
			name = fmt.Sprintf("uint64(::.%s)", v.name)
		} else {
			name = "::." + v.name
		}
		return fmt.Sprintf("dst = ssz.Marshal%s(dst, %s)", uintVToName(v), name)

	case TypeBitList:
		return fmt.Sprintf("%sdst = append(dst, ::.%s...)", v.validate(), v.name)

	case TypeBool:
		return fmt.Sprintf("dst = ssz.MarshalBool(dst, ::.%s)", v.name)

	case TypeVector:
		if v.e.isFixed() {
			return v.marshalVector()
		}
		fallthrough

	case TypeList:
		return v.marshalList()

	default:
		panic(fmt.Errorf("marshal not implemented for type %s", v.t.String()))
	}
}

func (v *Value) marshalList() string {
	v.e.name = v.name + "[ii]"

	// bound check
	str := v.validate()

	if v.e.isFixed() {
		tmpl := `for ii := 0; ii < len(::.{{.name}}); ii++ {
			{{.dynamic}}
		}`
		str += execTmpl(tmpl, map[string]interface{}{
			"name":    v.name,
			"dynamic": v.e.marshal(),
		})
		return str
	}

	// encode a list of dynamic objects:
	// 1. write offsets for each
	// 2. marshal each element

	tmpl := `{
		offset = 4 * len(::.{{.name}})
		for ii := 0; ii < len(::.{{.name}}); ii++ {
			dst = ssz.WriteOffset(dst, offset)
			{{.size}}
		}
	}
	for ii := 0; ii < len(::.{{.name}}); ii++ {
		{{.marshal}}
	}`

	str += execTmpl(tmpl, map[string]interface{}{
		"name":    v.name,
		"size":    v.e.size("offset"),
		"marshal": v.e.marshal(),
	})
	return str
}

func (v *Value) marshalVector() (str string) {
	v.e.name = fmt.Sprintf("%s[ii]", v.name)

	tmpl := `{{.validate}}for ii := 0; ii < {{.size}}; ii++ {
		{{.marshal}}
	}`
	return execTmpl(tmpl, map[string]interface{}{
		"validate": v.validate(),
		"name":     v.name,
		"size":     v.s,
		"marshal":  v.e.marshal(),
	})
}

func (v *Value) marshalContainer(start bool) string {
	if !start {
		tmpl := `{{ if .check }}if ::.{{.name}} == nil {
			::.{{.name}} = new({{.obj}})
		}
		{{ end }}if dst, err = ::.{{.name}}.MarshalSSZTo(dst); err != nil {
			return
		}`
		// validate only for fixed structs
		check := v.isFixed()
		if v.isListElem() {
			check = false
		}
		if v.noPtr {
			check = false
		}
		return execTmpl(tmpl, map[string]interface{}{
			"name":  v.name,
			"obj":   v.objRef(),
			"check": check,
		})
	}

	offset := v.fixedSize()
	out := []string{}

	for indx, i := range v.o {
		var str string
		if i.isFixed() {
			// write the content
			str = fmt.Sprintf("// Field (%d) '%s'\n%s\n", indx, i.name, i.marshal())
		} else {
			// write the offset
			str = fmt.Sprintf("// Offset (%d) '%s'\ndst = ssz.WriteOffset(dst, offset)\n%s\n", indx, i.name, i.size("offset"))
			offset += i.fixedSize()
		}
		out = append(out, str)
	}

	// write the dynamic parts
	for indx, i := range v.o {
		if !i.isFixed() {
			out = append(out, fmt.Sprintf("// Field (%d) '%s'\n%s\n", indx, i.name, i.marshal()))
		}
	}
	return strings.Join(out, "\n")
}
