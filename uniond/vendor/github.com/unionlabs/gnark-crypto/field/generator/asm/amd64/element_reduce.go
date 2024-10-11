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

func (f *FFAmd64) generateReduce() {
	stackSize := f.StackSize(1+f.NbWords*2, 0, 0)
	registers := f.FnHeader("reduce", stackSize, 8)
	defer f.AssertCleanStack(stackSize, 0)

	// registers
	r := registers.Pop()
	t := registers.PopN(f.NbWords)

	f.MOVQ("res+0(FP)", r)
	f.Mov(r, t)
	f.Reduce(&registers, t)
	f.Mov(t, r)
	f.RET()
}

// Reduce scratch can be on the stack or a set of registers.
func (f *FFAmd64) Reduce(registers *amd64.Registers, t []amd64.Register) {
	scratch := f.PopN(registers)
	f.ReduceElement(t, scratch)
	f.Push(registers, scratch...)
}
