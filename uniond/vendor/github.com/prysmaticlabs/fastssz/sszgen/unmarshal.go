package main

import (
	"fmt"
	"strconv"
	"strings"
)

// unmarshal creates a function that decodes the structs with the input byte in SSZ format.
func (e *env) unmarshal(name string, v *Value) string {
	tmpl := `// UnmarshalSSZ ssz unmarshals the {{.name}} object
	func (:: *{{.name}}) UnmarshalSSZ(buf []byte) error {
		var err error
		{{.unmarshal}}
		return err
	}`

	str := execTmpl(tmpl, map[string]interface{}{
		"name":      name,
		"unmarshal": v.umarshalContainer(true, "buf"),
	})

	return appendObjSignature(str, v)
}

func (v *Value) unmarshal(dst string) string {
	// we use dst as the input buffer where the SSZ data to decode the value is.
	switch v.t {
	case TypeContainer, TypeReference:
		return v.umarshalContainer(false, dst)

	case TypeBytes:
		if v.c {
			return fmt.Sprintf("copy(::.%s[:], %s)", v.name, dst)
		}
		validate := ""
		if !v.isFixed() {
			// dynamic bytes, we need to validate the size of the buffer
			validate = fmt.Sprintf("if len(%s) > %d { return ssz.ErrBytesLength }\n", dst, v.m)
		}
		// both fixed and dynamic are decoded equally
		tmpl := `{{.validate}}if cap(::.{{.name}}) == 0 {
			::.{{.name}} = make([]byte, 0, len({{.dst}}))
		}
		::.{{.name}} = append(::.{{.name}}, {{.dst}}...)`
		return execTmpl(tmpl, map[string]interface{}{
			"validate": validate,
			"name":     v.name,
			"dst":      dst,
			"size":     v.m,
		})

	case TypeUint:
		if v.ref != "" {
			// alias, we need to cast the value
			return fmt.Sprintf("::.%s = %s.%s(ssz.Unmarshall%s(%s))", v.name, v.ref, v.obj, uintVToName(v), dst)
		}
		if v.obj != "" {
			// alias to a type on the same package
			return fmt.Sprintf("::.%s = %s(ssz.Unmarshall%s(%s))", v.name, v.obj, uintVToName(v), dst)
		}
		return fmt.Sprintf("::.%s = ssz.Unmarshall%s(%s)", v.name, uintVToName(v), dst)

	case TypeBitList:
		tmpl := `if err = ssz.ValidateBitlist({{.dst}}, {{.size}}); err != nil {
			return err
		}
		if cap(::.{{.name}}) == 0 {
			::.{{.name}} = make([]byte, 0, len({{.dst}}))
		}
		::.{{.name}} = append(::.{{.name}}, {{.dst}}...)`
		return execTmpl(tmpl, map[string]interface{}{
			"name": v.name,
			"dst":  dst,
			"size": v.m,
		})

	case TypeVector:
		if v.e.isFixed() {
			dst = fmt.Sprintf("%s[ii*%d: (ii+1)*%d]", dst, v.e.fixedSize(), v.e.fixedSize())

			tmpl := `{{.create}}
			for ii := 0; ii < {{.size}}; ii++ {
				{{.unmarshal}}
			}`
			return execTmpl(tmpl, map[string]interface{}{
				"create":    v.createSlice(false),
				"size":      v.s,
				"unmarshal": v.e.unmarshal(dst),
			})
		}
		fallthrough

	case TypeList:
		return v.unmarshalList()

	case TypeBool:
		return fmt.Sprintf("::.%s = ssz.UnmarshalBool(%s)", v.name, dst)

	default:
		panic(fmt.Errorf("unmarshal not implemented for type %d", v.t))
	}
}

func (v *Value) unmarshalList() string {
	if v.e.isFixed() {
		dst := fmt.Sprintf("buf[ii*%d: (ii+1)*%d]", v.e.fixedSize(), v.e.fixedSize())

		tmpl := `num, err := ssz.DivideInt2(len(buf), {{.size}}, {{.max}})
		if err != nil {
			return err
		}
		{{.create}}
		for ii := 0; ii < num; ii++ {
			{{.unmarshal}}
		}`
		return execTmpl(tmpl, map[string]interface{}{
			"size":      v.e.fixedSize(),
			"max":       v.s,
			"create":    v.createSlice(true),
			"unmarshal": v.e.unmarshal(dst),
		})
	}

	if v.t == TypeVector {
		panic("it cannot happen")
	}

	// Decode list with a dynamic element. 'ssz.DecodeDynamicLength' ensures
	// that the number of elements do not surpass the 'ssz-max' tag.

	tmpl := `num, err := ssz.DecodeDynamicLength(buf, {{.max}})
	if err != nil {
		return err
	}
	{{.create}}
	err = ssz.UnmarshalDynamic(buf, num, func(indx int, buf []byte) (err error) {
		{{.unmarshal}}
		return nil
	})
	if err != nil {
		return err
	}`

	v.e.name = v.name + "[indx]"

	data := map[string]interface{}{
		"max":      v.s,
		"create":    v.createSlice(true),
		"unmarshal": v.e.unmarshal("buf"),
	}
	return execTmpl(tmpl, data)
}

func (v *Value) umarshalContainer(start bool, dst string) (str string) {
	if !start {
		tmpl := `{{ if .check }}if ::.{{.name}} == nil {
			::.{{.name}} = new({{.obj}})
		}
		{{ end }}if err = ::.{{.name}}.UnmarshalSSZ({{.dst}}); err != nil {
			return err
		}`
		check := true
		if v.noPtr {
			check = false
		}
		return execTmpl(tmpl, map[string]interface{}{
			"name":  v.name,
			"obj":   v.objRef(),
			"dst":   dst,
			"check": check,
		})
	}

	var offsets []string
	offsetsMatch := map[string]string{}

	for indx, i := range v.o {
		if !i.isFixed() {
			name := "o" + strconv.Itoa(indx)
			if len(offsets) != 0 {
				offsetsMatch[name] = offsets[len(offsets)-1]
			}
			offsets = append(offsets, name)
		}
	}

	// safe check for the size. Two cases:
	// 1. Struct is fixed: The size of the input buffer must be the same as the struct.
	// 2. Struct is dynamic. The size of the input buffer must be higher than the fixed part of the struct.

	var cmp string
	if v.isFixed() {
		cmp = "!="
	} else {
		cmp = "<"
	}

	// If the struct is dynamic we create a set of offset variables that will be readed later.

	tmpl := `size := uint64(len(buf))
	if size {{.cmp}} {{.size}} {
		return ssz.ErrSize
	}
	{{if .offsets}}
		tail := buf
		var {{.offsets}} uint64
	{{end}}
	`

	str += execTmpl(tmpl, map[string]interface{}{
		"cmp":     cmp,
		"size":    v.fixedSize(),
		"offsets": strings.Join(offsets, ", "),
	})

	var o0 uint64

	// Marshal the fixed part and offsets

	// used for bounds checking of variable length offsets.
	// for the first offset, use the size of the fixed-length data
	// as the minimum boundary. subsequent offsets will replace this
	// value with the name of the previous offset variable.
	firstOffsetCheck := fmt.Sprintf("%d", v.fixedSize())
	outs := []string{}
	for indx, i := range v.o {

		// How much it increases on every item
		var incr uint64
		if i.isFixed() {
			incr = i.fixedSize()
		} else {
			incr = bytesPerLengthOffset
		}

		dst = fmt.Sprintf("%s[%d:%d]", "buf", o0, o0+incr)
		o0 += incr

		var res string
		if i.isFixed() {
			res = fmt.Sprintf("// Field (%d) '%s'\n%s\n\n", indx, i.name, i.unmarshal(dst))

		} else {
			// read the offset
			offset := "o" + strconv.Itoa(indx)

			data := map[string]interface{}{
				"indx":   indx,
				"name":   i.name,
				"offset": offset,
				"dst":    dst,
				"firstOffsetCheck": firstOffsetCheck,
			}

			// We need to do two validations for the offset:
			// 1. The offset is lower than the total size of the input buffer
			// 2. The offset i needs to be higher than the offset i-1 (Only if the offset is not the first).

			if prev, ok := offsetsMatch[offset]; ok {
				data["more"] = fmt.Sprintf(" || %s > %s", prev, offset)
			} else {
				data["more"] = ""
			}

			tmpl := `// Offset ({{.indx}}) '{{.name}}'
			if {{.offset}} = ssz.ReadOffset({{.dst}}); {{.offset}} > size {{.more}} {
				return ssz.ErrOffset
			}
			{{ if .firstOffsetCheck }}
			if {{.offset}} < {{.firstOffsetCheck}} {
				return ssz.ErrInvalidVariableOffset
			}
			{{ end }}
			`
			res = execTmpl(tmpl, data)
			firstOffsetCheck = ""
		}
		outs = append(outs, res)
	}

	// Marshal the dynamic parts

	c := 0

	for indx, i := range v.o {
		if !i.isFixed() {
			from := offsets[c]
			var to string
			if c == len(offsets)-1 {
				to = ""
			} else {
				to = offsets[c+1]
			}
			tmpl := `// Field ({{.indx}}) '{{.name}}'
			{
				buf = tail[{{.from}}:{{.to}}]
				{{.unmarshal}}
			}`
			res := execTmpl(tmpl, map[string]interface{}{
				"indx":      indx,
				"name":      i.name,
				"from":      from,
				"to":        to,
				"unmarshal": i.unmarshal("buf"),
			})
			outs = append(outs, res)
			c++
		}
	}

	str += strings.Join(outs, "\n\n")
	return
}

// createItem is used to initialize slices of objects
func (v *Value) createSlice(useNumVariable bool) string {
	if v.t != TypeVector && v.t != TypeList {
		panic("BUG: create item is only intended to be used with vectors and lists")
	}

	size := strconv.Itoa(int(v.s))
	// when useNumVariable is specified, we assume there is a 'num' variable generated beforehand with the expected size.
	if useNumVariable {
		size = "num"
	}

	switch v.e.t {
	case TypeUint:
		// []int uses the Extend functions in the fastssz package
		return fmt.Sprintf("::.%s = ssz.Extend%s(::.%s, %s)", v.name, uintVToName(v.e), v.name, size)

	case TypeContainer:
		// []*(ref.)Struct{}
		return fmt.Sprintf("::.%s = make([]*%s, %s)", v.name, v.e.objRef(), size)

	case TypeBytes:
		// [][]byte
		if v.c {
			return ""
		}
		if v.e.c {
			return fmt.Sprintf("::.%s = make([][%d]byte, %s)", v.name, v.e.s, size)
		}
		return fmt.Sprintf("::.%s = make([][]byte, %s)", v.name, size)

	default:
		panic(fmt.Sprintf("create not implemented for type %s", v.e.t.String()))
	}
}
