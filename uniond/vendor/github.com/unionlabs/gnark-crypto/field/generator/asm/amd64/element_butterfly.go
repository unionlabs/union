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

// Butterfly sets
// a = a + b
// b = a - b
//
//	func Butterfly(a, b *{{.ElementName}}) {
//		t := *a
//		a.Add(a, b)
//		b.Sub(&t, b)
//	}
func (f *FFAmd64) generateButterfly() {
	f.Comment("Butterfly(a, b *Element) sets a = a + b; b = a - b")

	nbRegisters := f.NbWords*3 + 2
	if f.NbWords > 6 {
		nbRegisters = 2*f.NbWords + 1
	}
	stackSize := f.StackSize(nbRegisters, 0, 0)
	registers := f.FnHeader("Butterfly", stackSize, 16)
	defer f.AssertCleanStack(stackSize, 0)

	if f.NbWords <= 6 {
		// registers
		a := f.Pop(&registers)
		b := f.Pop(&registers)
		t0 := f.PopN(&registers)
		t1 := f.PopN(&registers)
		q := f.PopN(&registers)

		// t = a
		f.MOVQ("a+0(FP)", a)
		f.Mov(a, t0)
		f.Mov(t0, t1)
		f.XORQ(a, a) // set a to zero for later reduction

		f.MOVQ("b+8(FP)", b)
		f.Add(b, t0) // t0 = a + b
		f.Sub(b, t1) // t1 = a - b

		// reduce t1
		if f.NbWords >= 5 {
			// q is on the stack, can't use for CMOVQCC
			f.Mov(t0, q) // save t0
			f.Mov(f.Q, t0)
			for i := 0; i < f.NbWords; i++ {
				f.CMOVQCC(a, t0[i])
			}
			// add registers (q or 0) to t, and set to result
			f.Add(t0, t1)
			f.Mov(q, t0) // restore t0
		} else {
			f.Mov(f.Q, q)
			for i := 0; i < f.NbWords; i++ {
				f.CMOVQCC(a, q[i])
			}
			// add registers (q or 0) to t, and set to result
			f.Add(q, t1)
		}

		f.Push(&registers, q...)

		// save t1
		f.Mov(t1, b)

		// reduce t0
		f.ReduceElement(t0, t1)

		// save t0
		f.MOVQ("a+0(FP)", a)
		f.Mov(t0, a)

		f.RET()

		f.Push(&registers, t0...)
		f.Push(&registers, t1...)
		f.Push(&registers, a, b)
	} else {
		// registers
		r := f.Pop(&registers)
		t0 := f.PopN(&registers)
		t1 := f.PopN(&registers)

		// t = a
		f.MOVQ("b+8(FP)", r)
		f.Mov(r, t0)

		f.MOVQ("a+0(FP)", r)
		f.Add(r, t0)  // t0 = a + b
		f.Mov(t0, t1) // save t1 = t0
		f.Mov(r, t0)  // t0 = a
		f.MOVQ("b+8(FP)", r)
		f.Sub(r, t0) // t0 = a - b

		// reduce t0
		noReduce := f.NewLabel()
		f.JCC(noReduce)
		q := r
		f.MOVQ(f.Q[0], q)
		f.ADDQ(q, t0[0])
		for i := 1; i < f.NbWords; i++ {
			f.MOVQ(f.Q[i], q)
			f.ADCQ(q, t0[i])
		}
		f.LABEL(noReduce)

		// save t1
		f.MOVQ("b+8(FP)", r)
		f.Mov(t0, r)

		// reduce t0
		f.Mov(t1, t0)
		f.ReduceElement(t0, t1)

		// save t0
		f.MOVQ("a+0(FP)", r)
		f.Mov(t0, r)

		f.RET()

		f.Push(&registers, t0...)
		f.Push(&registers, t1...)
		f.Push(&registers, r)
	}

}
