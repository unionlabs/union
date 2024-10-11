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
	"html/template"
	"io"
	"strings"

	"github.com/consensys/bavard/amd64"
	gamd64 "github.com/consensys/gnark-crypto/field/generator/asm/amd64"
)

func (fq2 *Fq2Amd64) generateMulByNonResidueE2BN254() {
	// 	var a, b fp.Element
	// 	a.Double(&x.A0).Double(&a).Double(&a).fq2.Add(&a, &x.A0).fq2.Sub(&a, &x.A1)
	// 	b.Double(&x.A1).Double(&b).Double(&b).fq2.Add(&b, &x.A1).fq2.Add(&b, &x.A0)
	// 	z.A0.Set(&a)
	// 	z.A1.Set(&b)
	registers := fq2.FnHeader("mulNonResE2", 0, 16)

	a := registers.PopN(fq2.NbWords)
	b := registers.PopN(fq2.NbWords)
	x := registers.Pop()

	fq2.MOVQ("x+8(FP)", x)
	fq2.Mov(x, a) // a = a0

	fq2.Add(a, a)
	fq2.Reduce(&registers, a)

	fq2.Add(a, a)
	fq2.Reduce(&registers, a)

	fq2.Add(a, a)
	fq2.Reduce(&registers, a)

	fq2.Add(x, a)
	fq2.Reduce(&registers, a)

	fq2.Mov(x, b, fq2.NbWords) // b = a1
	zero := registers.Pop()
	fq2.XORQ(zero, zero)
	fq2.Sub(b, a)
	fq2.modReduceAfterSub(&registers, zero, a)
	registers.Push(zero)

	fq2.Add(b, b)
	fq2.Reduce(&registers, b)

	fq2.Add(b, b)
	fq2.Reduce(&registers, b)

	fq2.Add(b, b)
	fq2.Reduce(&registers, b)

	fq2.Add(x, b, fq2.NbWords)
	fq2.Reduce(&registers, b)
	fq2.Add(x, b)
	fq2.Reduce(&registers, b)

	fq2.MOVQ("res+0(FP)", x)
	fq2.Mov(a, x)
	fq2.Mov(b, x, 0, fq2.NbWords)

	fq2.RET()
}

func (fq2 *Fq2Amd64) generateSquareE2BN254(forceCheck bool) {

	const argSize = 16
	minStackSize := 0
	if forceCheck {
		minStackSize = argSize
	}

	stackSize := fq2.StackSize(fq2.NbWords*3, 2, minStackSize)
	registers := fq2.FnHeader("squareAdxE2", stackSize, argSize, amd64.DX, amd64.AX)
	defer fq2.AssertCleanStack(stackSize, minStackSize)
	fq2.WriteLn("NO_LOCAL_POINTERS")

	fq2.WriteLn(`
	// z.A0 = (x.A0 + x.A1) * (x.A0 - x.A1)
	// z.A1 = 2 * x.A0 * x.A1
	`)

	// check ADX instruction support
	lblNoAdx := fq2.NewLabel()
	if forceCheck {
		fq2.CMPB("·supportAdx(SB)", 1)
		fq2.JNE(lblNoAdx)
	}

	// used in the mul operation
	op1 := registers.PopN(fq2.NbWords)
	op2 := registers.PopN(fq2.NbWords)
	res := registers.PopN(fq2.NbWords)

	ax := amd64.AX
	dx := amd64.DX
	// b = a0 * a1 * 2

	fq2.Comment("2 * x.A0 * x.A1")
	fq2.MOVQ("x+8(FP)", ax)

	fq2.LabelRegisters("x.A0", op2...)
	fq2.Mov(ax, op2)

	fq2.LabelRegisters("2 * x.A1", op1...)
	fq2.Mov(ax, op1, fq2.NbWords)
	fq2.Add(op1, op1) // op1, no reduce

	fq2.mulElement()
	fq2.ReduceElement(res, op1)

	fq2.MOVQ("x+8(FP)", ax)

	fq2.LabelRegisters("x.A1", op1...)
	fq2.Mov(ax, op1, fq2.NbWords)

	fq2.MOVQ("res+0(FP)", dx)
	fq2.Mov(res, dx, 0, fq2.NbWords)
	fq2.Mov(op1, res)

	// a = a0 + a1
	fq2.Comment("Add(&x.A0, &x.A1)")
	fq2.Add(op2, op1)

	zero := amd64.BP
	fq2.XORQ(zero, zero)

	// b = a0 - a1
	fq2.Comment("Sub(&x.A0, &x.A1)")
	fq2.Sub(res, op2)
	fq2.modReduceAfterSubScratch(zero, op2, res) // using res as scratch registers

	// a = a * b
	fq2.mulElement()
	fq2.ReduceElement(res, op1)

	fq2.MOVQ("res+0(FP)", ax)
	fq2.Mov(res, ax)

	// result.a0 = a
	fq2.RET()

	// No adx
	if forceCheck {
		fq2.LABEL(lblNoAdx)
		fq2.MOVQ("res+0(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "(SP)")
		fq2.MOVQ("x+8(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "8(SP)")
		fq2.WriteLn("CALL ·squareGenericE2(SB)")
		fq2.RET()
	}

}

func (fq2 *Fq2Amd64) generateMulE2BN254(forceCheck bool) {
	const argSize = 24
	minStackSize := 0
	if forceCheck {
		minStackSize = argSize
	}
	stackSize := fq2.StackSize(fq2.NbWords*5, 2, minStackSize)
	registers := fq2.FnHeader("mulAdxE2", stackSize, argSize, amd64.DX, amd64.AX)
	defer fq2.AssertCleanStack(stackSize, minStackSize)

	fq2.WriteLn("NO_LOCAL_POINTERS")
	fq2.WriteLn(`
	// var a, b, c fp.Element
	// a.Add(&x.A0, &x.A1)
	// b.Add(&y.A0, &y.A1)
	// a.Mul(&a, &b)
	// b.Mul(&x.A0, &y.A0)
	// c.Mul(&x.A1, &y.A1)
	// z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	// z.A0.Sub(&b, &c)
	`)
	lblNoAdx := fq2.NewLabel()

	if forceCheck {
		fq2.CMPB("·supportAdx(SB)", 1)
		fq2.JNE(lblNoAdx)
	}

	// used in the mul operation
	op1 := registers.PopN(fq2.NbWords)
	op2 := registers.PopN(fq2.NbWords)
	res := registers.PopN(fq2.NbWords)

	ax := amd64.AX
	dx := amd64.DX

	aStack := fq2.PopN(&registers, true)
	cStack := fq2.PopN(&registers, true)

	fq2.MOVQ("x+8(FP)", ax)
	fq2.MOVQ("y+16(FP)", dx)

	// c = x.A1 * y.A1
	fq2.Mov(ax, op1, fq2.NbWords)
	fq2.Mov(dx, op2, fq2.NbWords)

	fq2.mulElement()
	fq2.ReduceElement(res, op2)
	// res = x.A1 * y.A1
	// pushing on stack for later use.
	fq2.Mov(res, cStack)

	fq2.MOVQ("x+8(FP)", ax)
	fq2.MOVQ("y+16(FP)", dx)

	// a = x.a0 + x.a1
	fq2.Add(ax, op1)

	// b = y.a0 + y.a1
	fq2.Mov(dx, op2)
	fq2.Add(dx, op2, fq2.NbWords)
	// --> note, we don't reduce, as this is used as input to the mul which accept input of size D-1/2 -1
	// TODO @gbotrel prove the upper bound / lower bound case for the no_carry mul

	// a = 	a * b = (x.a0 + x.a1) *  (y.a0 + y.a1)
	fq2.mulElement()
	fq2.ReduceElement(res, op2)

	// moving result to the stack.
	fq2.Mov(res, aStack)

	// b = x.A0 * y.AO
	fq2.MOVQ("x+8(FP)", ax)
	fq2.MOVQ("y+16(FP)", dx)

	fq2.Mov(ax, op1)
	fq2.Mov(dx, op2)

	fq2.mulElement()
	fq2.ReduceElement(res, op2)

	zero := dx
	fq2.XORQ(zero, zero)

	// a = a - b -c
	fq2.Mov(aStack, op1)
	fq2.Sub(res, op1) // a -= b
	fq2.modReduceAfterSubScratch(zero, op1, op2)

	fq2.Sub(cStack, op1) // a -= c
	fq2.modReduceAfterSubScratch(zero, op1, op2)

	fq2.MOVQ("res+0(FP)", ax)
	fq2.Mov(op1, ax, 0, fq2.NbWords)

	// b = b - c
	fq2.Mov(cStack, op2)
	fq2.Sub(op2, res) // b -= c
	fq2.modReduceAfterSubScratch(zero, res, op1)

	fq2.Mov(res, ax)

	fq2.RET()

	// No adx
	if forceCheck {
		fq2.LABEL(lblNoAdx)
		fq2.MOVQ("res+0(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "(SP)")
		fq2.MOVQ("x+8(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "8(SP)")
		fq2.MOVQ("y+16(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "16(SP)")
		fq2.WriteLn("CALL ·mulGenericE2(SB)")
		fq2.RET()
	}
	fq2.Push(&registers, aStack...)
	fq2.Push(&registers, cStack...)

}

func (fq2 *Fq2Amd64) generateMulDefine() {
	r := amd64.NewRegisters()
	r.Remove(amd64.DX)
	r.Remove(amd64.AX)
	op1 := r.PopN(fq2.NbWords)
	op2 := r.PopN(fq2.NbWords)
	res := r.PopN(fq2.NbWords)
	xat := func(i int) string {
		return string(op1[i])
	}
	yat := func(i int) string {
		return string(op2[i])
	}

	wd := writerDefine{fq2.w}
	tw := gamd64.NewFFAmd64(&wd, fq2.F)

	_, _ = io.WriteString(fq2.w, "// this code is generated and identical to fp.Mul(...)\n")
	_, _ = io.WriteString(fq2.w, "#define MUL() \\ \n")
	tw.MulADX(&r, xat, yat, res)
}

func (fq2 *Fq2Amd64) mulElement() {
	r := amd64.NewRegisters()
	r.Remove(amd64.DX)
	r.Remove(amd64.AX)
	op1 := r.PopN(fq2.NbWords)
	op2 := r.PopN(fq2.NbWords)
	res := r.PopN(fq2.NbWords)
	const tmplMul = `// mul ({{- range $i, $a := .A}}{{$a}}{{- if ne $.Last $i}},{{ end}}{{- end}}) with ({{- range $i, $b := .B}}{{$b}}{{- if ne $.Last $i}},{{ end}}{{- end}}) into ({{- range $i, $c := .C}}{{$c}}{{- if ne $.Last $i}},{{ end}}{{- end}})
	MUL()`

	var buf bytes.Buffer
	err := template.Must(template.New("").
		Parse(tmplMul)).Execute(&buf, struct {
		A, B, C []amd64.Register
		Last    int
	}{op1, op2, res, len(op1) - 1})

	if err != nil {
		panic(err)
	}

	fq2.WriteLn(buf.String())
	fq2.WriteLn("")
}

type writerDefine struct {
	w io.Writer
}

func (w *writerDefine) Write(p []byte) (n int, err error) {
	line := string(p)
	line = strings.TrimSpace(line)
	if strings.HasPrefix(line, "//") {
		return // drop comments
	}
	line = string(p)
	line = strings.ReplaceAll(line, "\n", "; \\ \n")
	return io.WriteString(w.w, line)
}
