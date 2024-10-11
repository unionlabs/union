package arm64

import (
	"fmt"
	"math/bits"
)

type Register uint

// Registers 1 means available to use
type Registers uint32
const registersCount = 28

func (r Register) Name() string {
	return fmt.Sprintf("R%d", r)
}

func (r Register) At(wordOffset int) string {
	return fmt.Sprintf("%d(R%d)", wordOffset*8, uint(r))
}

func (r Registers) Available() int {
	return bits.OnesCount32(uint32(r))
}

func (r *Registers) Pop() Register {

	s := uint32(*r)
	var step int
	for i := 0; i < registersCount; i+=step {
		step = bits.TrailingZeros32(s)
		if step == 0 {
			*r &= ^(1 << i)
			return Register(i)
		}
		s = s >> step
	}

	panic("no registers available")
}

func (r *Registers) PopN(n int) []Register {
	toReturn := make([]Register, n)
	for i := 0; i < n; i++ {
		toReturn[i] = r.Pop()
	}
	return toReturn
}

func (r *Registers) Push(rIn ...Register) {
	// ensure register is in our original list, and no duplicate
	for _, register := range rIn {

		if uint(register) >= registersCount {
			panic("warning: unknown register")
		}

		if *r & 1 << uint(register) == 1 {
			panic("duplicate register, already present.")
		}

		*r |= 1 << uint(register)
	}
}

func (r *Registers) Remove(registers ...Register) {
	for _, register := range registers {
		*r &= ^( 1 << uint(register))
	}
}

func NewRegisters() Registers {
	// R18 is reserved
	return (1 << registersCount - 1) &
		^(1 << 18)
}