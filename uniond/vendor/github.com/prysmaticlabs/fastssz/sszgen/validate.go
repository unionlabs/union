package main

func (v *Value) validate() string {
	switch v.t {
	case TypeBitList, TypeBytes:
		// this is a fixed-length array, not a slice, so it's size is a constant we don't need to check
		if v.c {
			return ""
		}
		// for fixed size collections, we need to ensure the size is an exact match
		cmp := "!="
		// for variable size values, we want to ensure it doesn't exceed max size bound
		if !v.isFixed() {
			cmp = ">"
		}

		tmpl := `if size := len(::.{{.name}}); size {{.cmp}} {{.size}} {
			err = ssz.ErrBytesLengthFn("--.{{.name}}", size, {{.size}})
			return
		}
		`
		return execTmpl(tmpl, map[string]interface{}{
			"cmp":  cmp,
			"name": v.name,
			"size": v.s,
		})

	case TypeVector:
		// this is a fixed-length array, not a slice, so it's size is a constant we don't need to check
		if v.c {
			return ""
		}
		// We only have vectors for [][]byte roots
		tmpl := `if size := len(::.{{.name}}); size != {{.size}} {
			err = ssz.ErrVectorLengthFn("--.{{.name}}", size, {{.size}})
			return
		}
		`
		return execTmpl(tmpl, map[string]interface{}{
			"name": v.name,
			"size": v.s,
		})

	case TypeList:
		tmpl := `if size := len(::.{{.name}}); size > {{.size}} {
			err = ssz.ErrListTooBigFn("--.{{.name}}", size, {{.size}})
			return
		}
		`
		return execTmpl(tmpl, map[string]interface{}{
			"name": v.name,
			"size": v.s,
		})

	default:
		return ""
	}
}
