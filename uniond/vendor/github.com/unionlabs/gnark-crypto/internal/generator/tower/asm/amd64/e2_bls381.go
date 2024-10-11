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
	"github.com/consensys/bavard/amd64"
)

func (fq2 *Fq2Amd64) generateMulByNonResidueE2BLS381() {
	// // MulByNonResidue multiplies a E2 by (1,1)
	// func (z *E2) MulByNonResidue(x *E2) *E2 {
	// 	var a fp.Element
	// 	a.Sub(&x.A0, &x.A1)
	// 	z.A1.Add(&x.A0, &x.A1)
	// 	z.A0.Set(&a)
	// 	return z
	// }
	registers := fq2.FnHeader("mulNonResE2", 0, 16)

	a := registers.PopN(fq2.NbWords)
	b := registers.PopN(fq2.NbWords)
	x := registers.Pop()
	tr := registers.Pop() // zero or r
	fq2.XORQ(tr, tr)      // set to zero

	fq2.MOVQ("x+8(FP)", x)
	fq2.Mov(x, a) // a = a0

	// a = x.A0 - x.A1
	fq2.Sub(x, a, fq2.NbWords)
	fq2.modReduceAfterSubScratch(tr, a, b)
	// b = x.A0 + x.A1
	fq2.Mov(x, b, fq2.NbWords) // b = a1
	fq2.Add(x, b)

	fq2.MOVQ("res+0(FP)", tr)
	fq2.Mov(a, tr)
	fq2.ReduceElement(b, a)
	fq2.Mov(b, tr, 0, fq2.NbWords)

	fq2.RET()
}

func (fq2 *Fq2Amd64) generateSquareE2BLS381(forceCheck bool) {
	// // Square sets z to the E2-product of x,x returns z
	// func (z *E2) Square(x *E2) *E2 {
	// 	// adapted from algo 22 https://eprint.iacr.org/2010/354.pdf
	// 	var a, b fp.Element
	// 	a.Add(&x.A0, &x.A1)
	// 	b.Sub(&x.A0, &x.A1)
	// 	a.Mul(&a, &b)
	// 	b.Mul(&x.A0, &x.A1).Double(&b)
	// 	z.A0.Set(&a)
	// 	z.A1.Set(&b)
	// 	return z
	// }
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

	noAdx := fq2.NewLabel()
	if forceCheck {
		// check ADX instruction support
		fq2.CMPB("·supportAdx(SB)", 1)
		fq2.JNE(noAdx)
	}

	// used in the mul operation
	op1 := registers.PopN(fq2.NbWords)
	res := registers.PopN(fq2.NbWords)

	xat := func(i int) string {
		return string(op1[i])
	}

	ax := amd64.AX
	dx := amd64.DX

	// b = a0 * a1 * 2

	fq2.Comment("2 * x.A0 * x.A1")
	fq2.MOVQ("x+8(FP)", ax)

	fq2.LabelRegisters("2 * x.A1", op1...)
	fq2.Mov(ax, op1, fq2.NbWords)
	fq2.Add(op1, op1) // op1, no reduce

	fq2.MulADX(&registers, xat, func(i int) string {
		fq2.MOVQ("x+8(FP)", dx)
		return dx.At(i)
	}, res)
	fq2.ReduceElement(res, op1)

	fq2.MOVQ("x+8(FP)", ax)

	fq2.LabelRegisters("x.A1", op1...)
	fq2.Mov(ax, op1, fq2.NbWords)

	fq2.MOVQ("res+0(FP)", dx)
	fq2.Mov(res, dx, 0, fq2.NbWords)
	fq2.Mov(op1, res)

	// op1 and res both contains x.A1 at this point
	// res+0(FP) (z.A1) must not be referenced.

	// a = a0 + a1
	fq2.Comment("Add(&x.A0, &x.A1)")
	fq2.Add(ax, op1)
	//--> must save on stack
	a0a1 := fq2.PopN(&registers, true)
	fq2.Mov(op1, a0a1)

	zero := amd64.BP
	fq2.XORQ(zero, zero)

	// b = a0 - a1
	fq2.Comment("Sub(&x.A0, &x.A1)")
	fq2.Mov(ax, op1)
	fq2.Sub(res, op1)
	fq2.modReduceAfterSubScratch(zero, op1, res) // using res as scratch registers

	// a = a * b
	fq2.MulADX(&registers, xat, func(i int) string { return string(a0a1[i]) }, res)
	fq2.ReduceElement(res, op1)

	fq2.MOVQ("res+0(FP)", ax)
	fq2.Mov(res, ax)

	// result.a0 = a
	fq2.RET()

	// No adx
	if forceCheck {
		fq2.LABEL(noAdx)
		fq2.MOVQ("res+0(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "(SP)")
		fq2.MOVQ("x+8(FP)", amd64.AX)
		fq2.MOVQ(amd64.AX, "8(SP)")
		fq2.WriteLn("CALL ·squareGenericE2(SB)")
		fq2.RET()
	}

	fq2.Push(&registers, a0a1...)
}

func (fq2 *Fq2Amd64) generateMulE2BLS381(forceCheck bool) {
	// var a, b, c fp.Element
	// a.Add(&x.A0, &x.A1)
	// b.Add(&y.A0, &y.A1)
	// a.Mul(&a, &b)
	// b.Mul(&x.A0, &y.A0)
	// c.Mul(&x.A1, &y.A1)
	// z.A1.Sub(&a, &b).Sub(&z.A1, &c)
	// z.A0.Sub(&b, &c)

	// we need a bit of stack space to store the results of the xA0yA0 and xA1yA1 multiplications
	const argSize = 24
	minStackSize := 0
	if forceCheck {
		minStackSize = argSize
	}
	stackSize := fq2.StackSize(fq2.NbWords*4, 2, minStackSize)
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
	// check ADX instruction support
	if forceCheck {
		fq2.CMPB("·supportAdx(SB)", 1)
		fq2.JNE(lblNoAdx)
	}

	// used in the mul operation
	op1 := registers.PopN(fq2.NbWords)
	res := registers.PopN(fq2.NbWords)

	xat := func(i int) string {
		return string(op1[i])
	}

	ax := amd64.AX
	dx := amd64.DX

	aStack := fq2.PopN(&registers, true)
	cStack := fq2.PopN(&registers, true)

	fq2.MOVQ("x+8(FP)", ax)

	// c = x.A1 * y.A1
	fq2.Mov(ax, op1, fq2.NbWords)
	fq2.MulADX(&registers, xat, func(i int) string {
		fq2.MOVQ("y+16(FP)", dx)
		return dx.At(i + fq2.NbWords)
	}, res)
	fq2.ReduceElement(res, op1)
	// res = x.A1 * y.A1
	// pushing on stack for later use.
	fq2.Mov(res, cStack)

	fq2.MOVQ("x+8(FP)", ax)
	fq2.MOVQ("y+16(FP)", dx)

	// a = x.a0 + x.a1
	fq2.Mov(ax, op1, fq2.NbWords)
	fq2.Add(ax, op1)
	fq2.Mov(op1, aStack)

	// b = y.a0 + y.a1
	fq2.Mov(dx, op1)
	fq2.Add(dx, op1, fq2.NbWords)
	// --> note, we don't reduce, as this is used as input to the mul which accept input of size D-1/2 -1

	// a = 	a * b = (x.a0 + x.a1) *  (y.a0 + y.a1)
	fq2.MulADX(&registers, xat, func(i int) string {
		return string(aStack[i])
	}, res)
	fq2.ReduceElement(res, op1)

	// moving result to the stack.
	fq2.Mov(res, aStack)

	// b = x.A0 * y.AO
	fq2.MOVQ("x+8(FP)", ax)

	fq2.Mov(ax, op1)

	fq2.MulADX(&registers, xat, func(i int) string {
		fq2.MOVQ("y+16(FP)", dx)
		return dx.At(i)
	}, res)
	fq2.ReduceElement(res, op1)

	zero := dx
	fq2.XORQ(zero, zero)

	// a = a - b -c
	fq2.Mov(aStack, op1)
	fq2.Sub(res, op1) // a -= b
	fq2.Mov(res, aStack)
	fq2.modReduceAfterSubScratch(zero, op1, res)

	fq2.Sub(cStack, op1) // a -= c
	fq2.modReduceAfterSubScratch(zero, op1, res)

	fq2.MOVQ("z+0(FP)", ax)
	fq2.Mov(op1, ax, 0, fq2.NbWords)

	// b = b - c
	fq2.Mov(aStack, res)
	fq2.Sub(cStack, res) // b -= c
	fq2.modReduceAfterSubScratch(zero, res, op1)

	fq2.Mov(res, ax)

	fq2.RET()

	// No adx
	if forceCheck {
		fq2.LABEL(lblNoAdx)
		fq2.MOVQ("z+0(FP)", amd64.AX)
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
