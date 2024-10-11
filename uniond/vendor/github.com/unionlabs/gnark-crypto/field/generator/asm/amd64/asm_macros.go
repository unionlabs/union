// Copyright 2020 ConsenSys Software Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package amd64

import (
	"bytes"
	"fmt"
	"text/template"

	"github.com/consensys/bavard/amd64"
)

// LabelRegisters write comment with friendler name to registers
func (f *FFAmd64) LabelRegisters(name string, r ...amd64.Register) {
	switch len(r) {
	case 0:
		return
	case 1:
		f.Comment(fmt.Sprintf("%s -> %s", name, string(r[0])))
	default:
		for i := 0; i < len(r); i++ {
			f.Comment(fmt.Sprintf("%s[%d] -> %s", name, i, string(r[i])))
		}
	}
	// f.WriteLn("")
}

func (f *FFAmd64) ReduceElement(t, scratch []amd64.Register) {
	if len(t) != len(scratch) {
		panic("invalid call")
	}

	const tmplReduce = `// reduce element({{- range $i, $a := .A}}{{$a}}{{- if ne $.Last $i}},{{ end}}{{- end}}) using temp registers ({{- range $i, $b := .B}}{{$b}}{{- if ne $.Last $i}},{{ end}}{{- end}})
	REDUCE({{- range $i, $a := .A}}{{$a}},{{- end}}
		{{- range $i, $b := .B}}{{$b}}{{- if ne $.Last $i}},{{ end}}{{- end}})`

	var buf bytes.Buffer
	err := template.Must(template.New("").
		Parse(tmplReduce)).Execute(&buf, struct {
		A, B []amd64.Register
		Last int
	}{t, scratch, len(scratch) - 1})

	if err != nil {
		panic(err)
	}

	f.WriteLn(buf.String())
	f.WriteLn("")
}

// TODO @gbotrel: figure out if interleaving MOVQ and SUBQ or CMOVQ and MOVQ instructions makes sense
const tmplDefines = `

// modulus q
{{- range $i, $w := .Q}}
DATA q<>+{{mul $i 8}}(SB)/8, {{imm $w}}
{{- end}}
GLOBL q<>(SB), (RODATA+NOPTR), ${{mul 8 $.NbWords}}

// qInv0 q'[0]
DATA qInv0<>(SB)/8, {{$qinv0 := index .QInverse 0}}{{imm $qinv0}}
GLOBL qInv0<>(SB), (RODATA+NOPTR), $8

#define REDUCE(	{{- range $i := .NbWordsIndexesFull}}ra{{$i}},{{- end}}
				{{- range $i := .NbWordsIndexesFull}}rb{{$i}}{{- if ne $.NbWordsLastIndex $i}},{{- end}}{{- end}}) \
	MOVQ ra0, rb0;  \
	SUBQ    q<>(SB), ra0; \
	{{- range $i := .NbWordsIndexesNoZero}}
	MOVQ ra{{$i}}, rb{{$i}};  \
	SBBQ  q<>+{{mul $i 8}}(SB), ra{{$i}}; \
	{{- end}}
	{{- range $i := .NbWordsIndexesFull}}
	CMOVQCS rb{{$i}}, ra{{$i}};  \
	{{- end}}


`

func (f *FFAmd64) GenerateDefines() {
	tmpl := template.Must(template.New("").
		Funcs(helpers()).
		Parse(tmplDefines))

	// execute template
	var buf bytes.Buffer
	if err := tmpl.Execute(&buf, f); err != nil {
		panic(err)
	}

	f.WriteLn(buf.String())
}

func (f *FFAmd64) Mov(i1, i2 interface{}, offsets ...int) {
	var o1, o2 int
	if len(offsets) >= 1 {
		o1 = offsets[0]
		if len(offsets) >= 2 {
			o2 = offsets[1]
		}
	}
	switch c1 := i1.(type) {
	case []uint64:
		switch c2 := i2.(type) {
		default:
			panic("unsupported")
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				f.MOVQ(c1[i+o1], c2[i+o2])
			}
		}
	case amd64.Register:
		switch c2 := i2.(type) {
		case amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				f.MOVQ(c1.At(i+o1), c2.At(i+o2))
			}
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				f.MOVQ(c1.At(i+o1), c2[i+o2])
			}
		default:
			panic("unsupported")
		}
	case []amd64.Register:
		switch c2 := i2.(type) {
		case amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				f.MOVQ(c1[i+o1], c2.At(i+o2))
			}
		case []amd64.Register:
			// f.copyElement(c1[o1:], c2[o2:])
			for i := 0; i < f.NbWords; i++ {
				f.MOVQ(c1[i+o1], c2[i+o2])
			}
		default:
			panic("unsupported")
		}
	default:
		panic("unsupported")
	}

}

func (f *FFAmd64) Add(i1, i2 interface{}, offsets ...int) {
	var o1, o2 int
	if len(offsets) >= 1 {
		o1 = offsets[0]
		if len(offsets) >= 2 {
			o2 = offsets[1]
		}
	}
	switch c1 := i1.(type) {

	case amd64.Register:
		switch c2 := i2.(type) {
		default:
			panic("unsupported")
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				if i == 0 {
					f.ADDQ(c1.At(i+o1), c2[i+o2])
				} else {
					f.ADCQ(c1.At(i+o1), c2[i+o2])
				}
			}
		}
	case []amd64.Register:
		switch c2 := i2.(type) {
		default:
			panic("unsupported")
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				if i == 0 {
					f.ADDQ(c1[i+o1], c2[i+o2])
				} else {
					f.ADCQ(c1[i+o1], c2[i+o2])
				}
			}
		}
	default:
		panic("unsupported")
	}
}

func (f *FFAmd64) Sub(i1, i2 interface{}, offsets ...int) {
	var o1, o2 int
	if len(offsets) >= 1 {
		o1 = offsets[0]
		if len(offsets) >= 2 {
			o2 = offsets[1]
		}
	}
	switch c1 := i1.(type) {

	case amd64.Register:
		switch c2 := i2.(type) {
		default:
			panic("unsupported")
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				if i == 0 {
					f.SUBQ(c1.At(i+o1), c2[i+o2])
				} else {
					f.SBBQ(c1.At(i+o1), c2[i+o2])
				}
			}
		}
	case []amd64.Register:
		switch c2 := i2.(type) {
		default:
			panic("unsupported")
		case []amd64.Register:
			for i := 0; i < f.NbWords; i++ {
				if i == 0 {
					f.SUBQ(c1[i+o1], c2[i+o2])
				} else {
					f.SBBQ(c1[i+o1], c2[i+o2])
				}
			}
		}
	default:
		panic("unsupported")
	}
}

// Template helpers (txt/template)
func helpers() template.FuncMap {
	// functions used in template
	return template.FuncMap{
		"mul": mul,
		"imm": imm,
		"sub": sub,
	}
}

func sub(a, b int) int {
	return a - b
}

func mul(a, b int) int {
	return a * b
}

func imm(t uint64) string {
	switch t {
	case 0:
		return "$0"
	case 1:
		return "$1"
	default:
		return fmt.Sprintf("$%#016x", t)
	}
}
