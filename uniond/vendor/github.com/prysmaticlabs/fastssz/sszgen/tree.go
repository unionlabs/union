package main

import (
	"fmt"
	"strings"
)

// getTree creates a function that SSZ hashes the structs,
func (e *env) getTree(name string, v *Value) string {
	tmpl := `// GetTree returns tree-backing for the {{.name}} object
	func (:: *{{.name}}) GetTreeWithWrapper(w *ssz.Wrapper) (err error) {
		{{.getTree}}
		return nil
	}

	func (:: *{{.name}}) GetTree() (*ssz.Node, error) {
		w := &ssz.Wrapper{}
		if err := ::.GetTreeWithWrapper(w); err != nil {
			return nil, err
		}
		return w.Node(), nil
	}`

	data := map[string]interface{}{
		"name":    name,
		"getTree": v.getTreeContainer(true),
	}
	str := execTmpl(tmpl, data)
	return appendObjSignature(str, v)
}

func (v *Value) getTrees(isList bool, elem Type) string {
	if elem != TypeUint {
		panic("unimplemented")
	}

	var merkleize string
	subLeavesTmpl := `subLeaves := ssz.LeavesFromUint64(::.{{.name}})`
	subLeaves := execTmpl(subLeavesTmpl, map[string]interface{}{
		"name": v.name,
	})

	if isList {
		tmpl := `numItems := len(::.{{.name}})
		tmp, err = ssz.TreeFromNodesWithMixin(subLeaves, numItems, int(ssz.CalculateLimit({{.listSize}}, uint64(numItems), {{.elemSize}})))
		if err != nil {
			return nil, err
		}`

		merkleize = execTmpl(tmpl, map[string]interface{}{
			"name":     v.name,
			"listSize": v.s,
			"elemSize": 8,
		})
	} else {
		merkleize = "tmp = ssz.TreeFromNodes(subLeaves)"
	}

	tmpl := `{
		{{.subLeaves}}
		{{.merkleize}}
	}`
	return execTmpl(tmpl, map[string]interface{}{
		"subLeaves": subLeaves,
		"merkleize": merkleize,
	})
}

func (v *Value) getTree() string {
	switch v.t {
	case TypeContainer, TypeReference:
		return v.getTreeContainer(false)

	case TypeBytes:
		// There are only fixed []byte
		name := v.name
		if v.c {
			name += "[:]"
		}

		tmpl := `{{.validate}}w.AddBytes(::.{{.name}})`
		return execTmpl(tmpl, map[string]interface{}{
			"validate": v.validate(),
			"name":     name,
			"size":     v.s,
		})

	case TypeUint:
		var name string
		if v.ref != "" || v.obj != "" {
			// alias to Uint64
			name = fmt.Sprintf("uint64(::.%s)", v.name)
		} else {
			name = "::." + v.name
		}

		method := uintVToName(v)
		return fmt.Sprintf("w.Add%s(%s)", method, name)

	case TypeBitList:
		panic("unimplemented")

	case TypeBool:
		return fmt.Sprintf("tmp = ssz.LeafFromBool(::.%s)", v.name)

	case TypeVector:
		return v.getTrees(false, v.e.t)

	case TypeList:
		if v.e.isFixed() {
			if v.e.t == TypeUint || v.e.t == TypeBytes {
				return v.getTrees(true, v.e.t)
			}
		}
		tmpl := `{
			subIdx := w.Indx()
			num := len(::.{{.name}})
			if num > {{.num}} {
				err = ssz.ErrIncorrectListSize
				return err
			}
			for i := 0; i < num; i++ {
				n, err := ::.{{.name}}[i].GetTree()
				if err != nil {
					return err
				}
				w.AddNode(n)
			}
			w.CommitWithMixin(subIdx, num, {{.num}})
		}`
		return execTmpl(tmpl, map[string]interface{}{
			"name": v.name,
			"num":  v.m,
		})

	default:
		panic(fmt.Errorf("hash not implemented for type %s", v.t.String()))
	}
}

func (v *Value) getTreeContainer(start bool) string {
	if !start {
		return fmt.Sprintf("if err := ::.%s.GetTreeWithWrapper(w); err != nil {\n return err\n}", v.name)
	}

	numLeaves := nextPowerOfTwo(uint64(len(v.o)))
	out := []string{}
	for indx, i := range v.o {
		str := fmt.Sprintf("// Field (%d) '%s'\n%s\n", indx, i.name, i.getTree())
		out = append(out, str)
	}

	// Empty leaves
	emptyLeaves := ""
	if numLeaves-uint(len(v.o)) > 0 {
		emptyLeaves = fmt.Sprintf("for i := 0; i < %d; i++ {\nw.AddEmpty()\n}", numLeaves-uint(len(v.o)))
	}

	tmpl := `indx := w.Indx()

	{{.fields}}
	{{.emptyLeaves}}
	
	w.Commit(indx)`

	return execTmpl(tmpl, map[string]interface{}{
		"numLeaves":   numLeaves,
		"fields":      strings.Join(out, "\n"),
		"emptyLeaves": emptyLeaves,
	})
}

func nextPowerOfTwo(v uint64) uint {
	v--
	v |= v >> 1
	v |= v >> 2
	v |= v >> 4
	v |= v >> 8
	v |= v >> 16
	v++
	return uint(v)
}
