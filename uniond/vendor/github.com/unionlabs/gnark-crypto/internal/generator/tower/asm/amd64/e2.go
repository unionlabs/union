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
	"io"

	"github.com/consensys/bavard"
	ramd64 "github.com/consensys/bavard/amd64"
	"github.com/consensys/gnark-crypto/field/generator/asm/amd64"
	field "github.com/consensys/gnark-crypto/field/generator/config"
	"github.com/consensys/gnark-crypto/internal/generator/config"
)

// Fq2Amd64 ...
type Fq2Amd64 struct {
	*amd64.FFAmd64
	config config.Curve
	w      io.Writer
	F      *field.FieldConfig
}

// NewFq2Amd64 ...
func NewFq2Amd64(w io.Writer, F *field.FieldConfig, config config.Curve) *Fq2Amd64 {
	return &Fq2Amd64{
		amd64.NewFFAmd64(w, F),
		config,
		w,
		F,
	}
}

// Generate ...
func (fq2 *Fq2Amd64) Generate(forceADXCheck bool) error {
	fq2.WriteLn(bavard.Apache2Header("ConsenSys Software Inc.", 2020))

	fq2.WriteLn("#include \"textflag.h\"")
	fq2.WriteLn("#include \"funcdata.h\"")

	fq2.GenerateDefines()
	if fq2.config.Equal(config.BN254) {
		fq2.generateMulDefine()
	}

	fq2.generateAddE2()
	fq2.generateDoubleE2()
	fq2.generateSubE2()
	fq2.generateNegE2()

	if fq2.config.Equal(config.BN254) {
		fq2.generateMulByNonResidueE2BN254()
		fq2.generateMulE2BN254(forceADXCheck)
		fq2.generateSquareE2BN254(forceADXCheck)
	} else if fq2.config.Equal(config.BLS12_381) {
		fq2.generateMulByNonResidueE2BLS381()
		fq2.generateSquareE2BLS381(forceADXCheck)
		fq2.generateMulE2BLS381(forceADXCheck)
	}

	return nil
}

func (fq2 *Fq2Amd64) generateAddE2() {
	registers := fq2.FnHeader("addE2", 0, 24)

	// registers
	x := registers.Pop()
	y := registers.Pop()
	r := registers.Pop()
	t := registers.PopN(fq2.NbWords)

	fq2.MOVQ("x+8(FP)", x)

	// move t = x
	fq2.Mov(x, t)

	fq2.MOVQ("y+16(FP)", y)

	// t = t + y = x + y
	fq2.Add(y, t)

	// reduce
	fq2.Reduce(&registers, t)
	fq2.MOVQ("res+0(FP)", r)
	fq2.Mov(t, r)

	// move x+offset(fq2.NbWords) into t
	fq2.Mov(x, t, fq2.NbWords)

	// add y+offset(fq2.NbWords) into t
	fq2.Add(y, t, fq2.NbWords)

	// reduce t into r with offset fq2.NbWords
	fq2.Reduce(&registers, t)
	fq2.Mov(t, r, 0, fq2.NbWords)

	fq2.RET()

}

func (fq2 *Fq2Amd64) generateDoubleE2() {
	// func header
	registers := fq2.FnHeader("doubleE2", 0, 16)

	// registers
	x := registers.Pop()
	r := registers.Pop()
	t := registers.PopN(fq2.NbWords)

	fq2.MOVQ("res+0(FP)", r)
	fq2.MOVQ("x+8(FP)", x)

	fq2.Mov(x, t)
	fq2.Add(t, t)
	fq2.Reduce(&registers, t)
	fq2.Mov(t, r)
	fq2.Mov(x, t, fq2.NbWords)
	fq2.Add(t, t)
	fq2.Reduce(&registers, t)
	fq2.Mov(t, r, 0, fq2.NbWords)

	fq2.RET()
}

func (fq2 *Fq2Amd64) generateNegE2() {
	registers := fq2.FnHeader("negE2", 0, 16)

	nonZeroA := fq2.NewLabel()
	nonZeroB := fq2.NewLabel()
	B := fq2.NewLabel()

	// registers
	x := registers.Pop()
	r := registers.Pop()
	q := registers.Pop()
	t := registers.PopN(fq2.NbWords)

	fq2.MOVQ("res+0(FP)", r)
	fq2.MOVQ("x+8(FP)", x)

	// t = x
	fq2.Mov(x, t)

	// x = t[0] | ... | t[n]
	fq2.MOVQ(t[0], x)
	for i := 1; i < fq2.NbWords; i++ {
		fq2.ORQ(t[i], x)
	}

	fq2.TESTQ(x, x)

	// if x != 0, we jump to nonzero label
	fq2.JNE(nonZeroA)

	// if x == 0, we set the result to zero and continue
	for i := 0; i < fq2.NbWords; i++ {
		fq2.MOVQ(x, r.At(i))
	}
	fq2.JMP(B)

	fq2.LABEL(nonZeroA)

	// z = x - q
	for i := 0; i < fq2.NbWords; i++ {
		fq2.MOVQ(fq2.Q[i], q)
		if i == 0 {
			fq2.SUBQ(t[i], q)
		} else {
			fq2.SBBQ(t[i], q)
		}
		fq2.MOVQ(q, r.At(i))
	}

	fq2.LABEL(B)
	fq2.MOVQ("x+8(FP)", x)
	fq2.Mov(x, t, fq2.NbWords)

	// x = t[0] | ... | t[n]
	fq2.MOVQ(t[0], x)
	for i := 1; i < fq2.NbWords; i++ {
		fq2.ORQ(t[i], x)
	}

	fq2.TESTQ(x, x)

	// if x != 0, we jump to nonzero label
	fq2.JNE(nonZeroB)

	// if x == 0, we set the result to zero and return
	for i := 0; i < fq2.NbWords; i++ {
		fq2.MOVQ(x, r.At(i+fq2.NbWords))
	}
	fq2.RET()

	fq2.LABEL(nonZeroB)

	// z = x - q
	for i := 0; i < fq2.NbWords; i++ {
		fq2.MOVQ(fq2.Q[i], q)
		if i == 0 {
			fq2.SUBQ(t[i], q)
		} else {
			fq2.SBBQ(t[i], q)
		}
		fq2.MOVQ(q, r.At(i+fq2.NbWords))
	}

	fq2.RET()

}

func (fq2 *Fq2Amd64) generateSubE2() {
	registers := fq2.FnHeader("subE2", 0, 24)

	// registers
	t := registers.PopN(fq2.NbWords)
	xy := registers.Pop()

	zero := registers.Pop()
	fq2.XORQ(zero, zero)

	fq2.MOVQ("x+8(FP)", xy)
	fq2.Mov(xy, t)

	// z = x - y mod q
	// move t = x
	fq2.MOVQ("y+16(FP)", xy)
	fq2.Sub(xy, t)
	fq2.MOVQ("x+8(FP)", xy)

	fq2.modReduceAfterSub(&registers, zero, t)

	r := registers.Pop()
	fq2.MOVQ("res+0(FP)", r)
	fq2.Mov(t, r)
	registers.Push(r)

	fq2.Mov(xy, t, fq2.NbWords)

	// z = x - y mod q
	// move t = x
	fq2.MOVQ("y+16(FP)", xy)
	fq2.Sub(xy, t, fq2.NbWords)

	fq2.modReduceAfterSub(&registers, zero, t)

	r = xy
	fq2.MOVQ("res+0(FP)", r)

	fq2.Mov(t, r, 0, fq2.NbWords)

	fq2.RET()

}

func (fq2 *Fq2Amd64) modReduceAfterSub(registers *ramd64.Registers, zero ramd64.Register, t []ramd64.Register) {
	q := registers.PopN(fq2.NbWords)
	fq2.modReduceAfterSubScratch(zero, t, q)
	registers.Push(q...)
}

func (fq2 *Fq2Amd64) modReduceAfterSubScratch(zero ramd64.Register, t, scratch []ramd64.Register) {
	fq2.Mov(fq2.Q, scratch)
	for i := 0; i < fq2.NbWords; i++ {
		fq2.CMOVQCC(zero, scratch[i])
	}
	// add registers (q or 0) to t, and set to result
	fq2.Add(scratch, t)
}
