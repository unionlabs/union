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

// Package amd64 contains syntactic sugar to generate amd64 assembly code
package amd64

import (
	"fmt"
	"io"
	"strings"

	"github.com/consensys/bavard"

	"github.com/consensys/bavard/amd64"
	"github.com/consensys/gnark-crypto/field/generator/config"
)

const SmallModulus = 6

func NewFFAmd64(w io.Writer, F *config.FieldConfig) *FFAmd64 {
	return &FFAmd64{F, amd64.NewAmd64(w), 0, 0}
}

type FFAmd64 struct {
	*config.FieldConfig
	*amd64.Amd64
	nbElementsOnStack int
	maxOnStack        int
}

func (f *FFAmd64) StackSize(maxNbRegistersNeeded, nbRegistersReserved, minStackSize int) int {
	got := amd64.NbRegisters - nbRegistersReserved
	r := got - maxNbRegistersNeeded
	if r >= 0 {
		return minStackSize
	}
	r *= -8
	return max(r, minStackSize)
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}

func (f *FFAmd64) AssertCleanStack(reservedStackSize, minStackSize int) {
	if f.nbElementsOnStack != 0 {
		panic("missing f.Push stack elements")
	}
	if reservedStackSize < minStackSize {
		panic("invalid minStackSize or reservedStackSize")
	}
	usedStackSize := f.maxOnStack * 8
	if usedStackSize > reservedStackSize {
		panic("using more stack size than reserved")
	} else if max(usedStackSize, minStackSize) < reservedStackSize {
		// this panic is for dev purposes as this may be by design for aligment
		panic("reserved more stack size than needed")
	}

	f.maxOnStack = 0
}

func (f *FFAmd64) Push(registers *amd64.Registers, rIn ...amd64.Register) {
	for _, r := range rIn {
		if strings.HasPrefix(string(r), "s") {
			// it's on the stack, decrease the offset
			f.nbElementsOnStack--
			continue
		}
		registers.Push(r)
	}
}

func (f *FFAmd64) Pop(registers *amd64.Registers, forceStack ...bool) amd64.Register {
	if registers.Available() >= 1 && !(len(forceStack) > 0 && forceStack[0]) {
		return registers.Pop()
	}
	r := amd64.Register(fmt.Sprintf("s%d-%d(SP)", f.nbElementsOnStack, 8+f.nbElementsOnStack*8))
	f.nbElementsOnStack++
	if f.nbElementsOnStack > f.maxOnStack {
		f.maxOnStack = f.nbElementsOnStack
	}
	return r
}

func (f *FFAmd64) PopN(registers *amd64.Registers, forceStack ...bool) []amd64.Register {
	if len(forceStack) > 0 && forceStack[0] {
		nbStack := f.NbWords
		var u []amd64.Register

		for i := f.nbElementsOnStack; i < nbStack+f.nbElementsOnStack; i++ {
			u = append(u, amd64.Register(fmt.Sprintf("s%d-%d(SP)", i, 8+i*8)))
		}
		f.nbElementsOnStack += nbStack
		if f.nbElementsOnStack > f.maxOnStack {
			f.maxOnStack = f.nbElementsOnStack
		}
		return u
	}
	if registers.Available() >= f.NbWords {
		return registers.PopN(f.NbWords)
	}
	nbStack := f.NbWords - registers.Available()
	u := registers.PopN(registers.Available())

	for i := f.nbElementsOnStack; i < nbStack+f.nbElementsOnStack; i++ {
		u = append(u, amd64.Register(fmt.Sprintf("s%d-%d(SP)", i, 8+i*8)))
	}
	f.nbElementsOnStack += nbStack
	if f.nbElementsOnStack > f.maxOnStack {
		f.maxOnStack = f.nbElementsOnStack
	}
	return u
}

func (f *FFAmd64) qAt(index int) string {
	return fmt.Sprintf("q<>+%d(SB)", index*8)
}

func (f *FFAmd64) qInv0() string {
	return "qInv0<>(SB)"
}

// Generate generates assembly code for the base field provided to goff
// see internal/templates/ops*
func Generate(w io.Writer, F *config.FieldConfig) error {
	f := NewFFAmd64(w, F)
	f.WriteLn(bavard.Apache2Header("ConsenSys Software Inc.", 2020))

	f.WriteLn("#include \"textflag.h\"")
	f.WriteLn("#include \"funcdata.h\"")
	f.WriteLn("")

	f.GenerateDefines()

	// reduce
	f.generateReduce()

	// mul by constants
	f.generateMulBy3()
	f.generateMulBy5()
	f.generateMulBy13()

	// fft butterflies
	f.generateButterfly()

	return nil
}

func GenerateMul(w io.Writer, F *config.FieldConfig) error {
	f := NewFFAmd64(w, F)
	f.WriteLn(bavard.Apache2Header("ConsenSys Software Inc.", 2020))

	f.WriteLn("#include \"textflag.h\"")
	f.WriteLn("#include \"funcdata.h\"")
	f.WriteLn("")
	f.GenerateDefines()

	// mul
	f.generateMul(false)

	// from mont
	f.generateFromMont(false)

	return nil
}

func GenerateMulADX(w io.Writer, F *config.FieldConfig) error {
	f := NewFFAmd64(w, F)
	f.WriteLn(bavard.Apache2Header("ConsenSys Software Inc.", 2020))

	f.WriteLn("#include \"textflag.h\"")
	f.WriteLn("#include \"funcdata.h\"")
	f.WriteLn("")
	f.GenerateDefines()

	// mul
	f.generateMul(true)

	// from mont
	f.generateFromMont(true)

	return nil
}
