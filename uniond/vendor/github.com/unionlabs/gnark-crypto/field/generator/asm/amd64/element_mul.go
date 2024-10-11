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
	"fmt"

	"github.com/consensys/bavard/amd64"
)

// MulADX uses AX, DX and BP
// sets x * y into t, without modular reduction
// x() will have more accesses than y()
// (caller should store x in registers, if possible)
// if no (tmp) register is available, this uses one PUSH/POP on the stack in the hot loop.
func (f *FFAmd64) MulADX(registers *amd64.Registers, x, y func(int) string, t []amd64.Register) []amd64.Register {
	// registers
	var tr amd64.Register // temporary register
	A := amd64.BP

	hasFreeRegister := registers.Available() > 0
	if hasFreeRegister {
		tr = registers.Pop()
	} else {
		tr = A
	}

	f.LabelRegisters("A", A)
	f.LabelRegisters("t", t...)

	for i := 0; i < f.NbWords; i++ {
		f.Comment("clear the flags")
		f.XORQ(amd64.AX, amd64.AX)

		f.MOVQ(y(i), amd64.DX)

		// for j=0 to N-1
		//    (A,t[j])  := t[j] + x[j]*y[i] + A
		if i == 0 {
			for j := 0; j < f.NbWords; j++ {
				f.Comment(fmt.Sprintf("(A,t[%[1]d])  := x[%[1]d]*y[%[2]d] + A", j, i))

				if j == 0 && f.NbWords == 1 {
					f.MULXQ(x(j), t[j], A)
				} else if j == 0 {
					f.MULXQ(x(j), t[j], t[j+1])
				} else {
					highBits := A
					if j != f.NbWordsLastIndex {
						highBits = t[j+1]
					}
					f.MULXQ(x(j), amd64.AX, highBits)
					f.ADOXQ(amd64.AX, t[j])
				}
			}
		} else {
			for j := 0; j < f.NbWords; j++ {
				f.Comment(fmt.Sprintf("(A,t[%[1]d])  := t[%[1]d] + x[%[1]d]*y[%[2]d] + A", j, i))

				if j != 0 {
					f.ADCXQ(A, t[j])
				}
				f.MULXQ(x(j), amd64.AX, A)
				f.ADOXQ(amd64.AX, t[j])
			}
		}

		f.Comment("A += carries from ADCXQ and ADOXQ")
		f.MOVQ(0, amd64.AX)
		if i != 0 {
			f.ADCXQ(amd64.AX, A)
		}
		f.ADOXQ(amd64.AX, A)

		if !hasFreeRegister {
			f.PUSHQ(A)
		}

		// m := t[0]*q'[0] mod W
		f.Comment("m := t[0]*q'[0] mod W")
		m := amd64.DX
		// f.MOVQ(t[0], m)
		// f.MULXQ(f.qInv0(), m, amd64.AX)
		f.MOVQ(f.qInv0(), m)
		f.IMULQ(t[0], m)

		// clear the carry flags
		f.Comment("clear the flags")
		f.XORQ(amd64.AX, amd64.AX)

		// C,_ := t[0] + m*q[0]
		f.Comment("C,_ := t[0] + m*q[0]")

		f.MULXQ(f.qAt(0), amd64.AX, tr)
		f.ADCXQ(t[0], amd64.AX)
		f.MOVQ(tr, t[0])

		if !hasFreeRegister {
			f.POPQ(A)
		}
		// for j=1 to N-1
		//    (C,t[j-1]) := t[j] + m*q[j] + C
		for j := 1; j < f.NbWords; j++ {
			f.Comment(fmt.Sprintf("(C,t[%[1]d]) := t[%[2]d] + m*q[%[2]d] + C", j-1, j))
			f.ADCXQ(t[j], t[j-1])
			f.MULXQ(f.qAt(j), amd64.AX, t[j])
			f.ADOXQ(amd64.AX, t[j-1])
		}

		f.Comment(fmt.Sprintf("t[%d] = C + A", f.NbWordsLastIndex))
		f.MOVQ(0, amd64.AX)
		f.ADCXQ(amd64.AX, t[f.NbWordsLastIndex])
		f.ADOXQ(A, t[f.NbWordsLastIndex])

	}

	if hasFreeRegister {
		registers.Push(tr)
	}

	return t
}

func (f *FFAmd64) generateMul(forceADX bool) {
	f.Comment("mul(res, x, y *Element)")

	const argSize = 3 * 8
	minStackSize := argSize
	if forceADX {
		minStackSize = 0
	}
	stackSize := f.StackSize(f.NbWords*2, 2, minStackSize)
	reserved := []amd64.Register{amd64.DX, amd64.AX}
	if f.NbWords <= 5 {
		// when dynamic linking, R15 is clobbered by a global variable access
		// this is a temporary workaround --> don't use R15 when we can avoid it.
		// see https://github.com/ConsenSys/gnark-crypto/issues/113
		reserved = append(reserved, amd64.R15)
	}
	registers := f.FnHeader("mul", stackSize, argSize, reserved...)
	defer f.AssertCleanStack(stackSize, minStackSize)

	f.WriteLn(fmt.Sprintf(`
	// the algorithm is described in the %s.Mul declaration (.go)
	// however, to benefit from the ADCX and ADOX carry chains
	// we split the inner loops in 2:
	// for i=0 to N-1
	// 		for j=0 to N-1
	// 		    (A,t[j])  := t[j] + x[j]*y[i] + A
	// 		m := t[0]*q'[0] mod W
	// 		C,_ := t[0] + m*q[0]
	// 		for j=1 to N-1
	// 		    (C,t[j-1]) := t[j] + m*q[j] + C
	// 		t[N-1] = C + A
	`, f.ElementName))
	if stackSize > 0 {
		f.WriteLn("NO_LOCAL_POINTERS")
	}

	noAdx := f.NewLabel()

	if !forceADX {
		// check ADX instruction support
		f.CMPB("·supportAdx(SB)", 1)
		f.JNE(noAdx)
	}

	{
		// we need to access x and y words, per index
		var xat, yat func(int) string
		var gc func()

		// we need NbWords registers for t, plus optionally one for tmp register in mulADX if we want to avoid PUSH/POP
		nbRegisters := registers.Available()
		if nbRegisters < f.NbWords {
			panic("not enough registers, not supported.")
		}

		t := registers.PopN(f.NbWords)
		nbRegisters = registers.Available()
		switch nbRegisters {
		case 0:
			// y is access through use of AX/DX
			yat = func(i int) string {
				y := amd64.AX
				f.MOVQ("y+16(FP)", y)
				return y.At(i)
			}

			// we move x on the stack.
			f.MOVQ("x+8(FP)", amd64.AX)
			_x := f.PopN(&registers, true)
			f.LabelRegisters("x", _x...)
			f.Mov(amd64.AX, t)
			f.Mov(t, _x)
			xat = func(i int) string {
				return string(_x[i])
			}
			gc = func() {
				f.Push(&registers, _x...)
			}
		case 1:
			// y is access through use of AX/DX
			yat = func(i int) string {
				y := amd64.AX
				f.MOVQ("y+16(FP)", y)
				return y.At(i)
			}
			// x uses the register
			x := registers.Pop()
			f.MOVQ("x+8(FP)", x)
			xat = func(i int) string {
				return x.At(i)
			}
			gc = func() {
				registers.Push(x)
			}
		case 2, 3:
			// x, y uses registers
			x := registers.Pop()
			y := registers.Pop()

			f.MOVQ("x+8(FP)", x)
			f.MOVQ("y+16(FP)", y)

			xat = func(i int) string {
				return x.At(i)
			}

			yat = func(i int) string {
				return y.At(i)
			}
			gc = func() {
				registers.Push(x, y)
			}
		default:
			// we have a least 4 registers.
			// 1 for tmp.
			nbRegisters--
			// 1 for y
			nbRegisters--
			var y amd64.Register

			if nbRegisters >= f.NbWords {
				// we store x fully in registers
				x := registers.Pop()
				f.MOVQ("x+8(FP)", x)
				_x := registers.PopN(f.NbWords)
				f.LabelRegisters("x", _x...)
				f.Mov(x, _x)

				xat = func(i int) string {
					return string(_x[i])
				}
				registers.Push(x)
				gc = func() {
					registers.Push(y)
					registers.Push(_x...)
				}
			} else {
				// we take at least 1 register for x addr
				nbRegisters--
				x := registers.Pop()
				y = registers.Pop() // temporary lock 1 for y
				f.MOVQ("x+8(FP)", x)

				// and use the rest for x0...xn
				_x := registers.PopN(nbRegisters)
				f.LabelRegisters("x", _x...)
				for i := 0; i < len(_x); i++ {
					f.MOVQ(x.At(i), _x[i])
				}
				xat = func(i int) string {
					if i < len(_x) {
						return string(_x[i])
					}
					return x.At(i)
				}
				registers.Push(y)

				gc = func() {
					registers.Push(x, y)
					registers.Push(_x...)
				}

			}
			y = registers.Pop()

			f.MOVQ("y+16(FP)", y)
			yat = func(i int) string {
				return y.At(i)
			}

		}

		f.MulADX(&registers, xat, yat, t)
		gc()

		// ---------------------------------------------------------------------------------------------
		// reduce
		f.Reduce(&registers, t)

		f.MOVQ("res+0(FP)", amd64.AX)
		f.Mov(t, amd64.AX)
		f.RET()
	}

	// ---------------------------------------------------------------------------------------------
	// no MULX, ADX instructions
	if !forceADX {
		f.LABEL(noAdx)

		f.MOVQ("res+0(FP)", amd64.AX)
		f.MOVQ(amd64.AX, "(SP)")
		f.MOVQ("x+8(FP)", amd64.AX)
		f.MOVQ(amd64.AX, "8(SP)")
		f.MOVQ("y+16(FP)", amd64.AX)
		f.MOVQ(amd64.AX, "16(SP)")
		f.WriteLn("CALL ·_mulGeneric(SB)")
		f.RET()

	}
}
