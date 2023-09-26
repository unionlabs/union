package constraint

import (
	"github.com/consensys/gnark/constraint/solver"
)

type BlueprintGenericHint struct{}

func (b *BlueprintGenericHint) DecompressHint(h *HintMapping, inst Instruction) {
	// ignore first call data == nbInputs
	h.HintID = solver.HintID(inst.Calldata[1])
	lenInputs := int(inst.Calldata[2])
	if cap(h.Inputs) >= lenInputs {
		h.Inputs = h.Inputs[:lenInputs]
	} else {
		h.Inputs = make([]LinearExpression, lenInputs)
	}

	j := 3
	for i := 0; i < lenInputs; i++ {
		n := int(inst.Calldata[j]) // len of linear expr
		j++
		if cap(h.Inputs[i]) >= n {
			h.Inputs[i] = h.Inputs[i][:0]
		} else {
			h.Inputs[i] = make(LinearExpression, 0, n)
		}
		for k := 0; k < n; k++ {
			h.Inputs[i] = append(h.Inputs[i], Term{CID: inst.Calldata[j], VID: inst.Calldata[j+1]})
			j += 2
		}
	}
	h.OutputRange.Start = inst.Calldata[j]
	h.OutputRange.End = inst.Calldata[j+1]
}

func (b *BlueprintGenericHint) CompressHint(h HintMapping, to *[]uint32) {
	nbInputs := 1 // storing nb inputs
	nbInputs++    // hintID
	nbInputs++    // len(h.Inputs)
	for i := 0; i < len(h.Inputs); i++ {
		nbInputs++ // len of h.Inputs[i]
		nbInputs += len(h.Inputs[i]) * 2
	}

	nbInputs += 2 // output range start / end

	(*to) = append((*to), uint32(nbInputs))
	(*to) = append((*to), uint32(h.HintID))
	(*to) = append((*to), uint32(len(h.Inputs)))

	for _, l := range h.Inputs {
		(*to) = append((*to), uint32(len(l)))
		for _, t := range l {
			(*to) = append((*to), uint32(t.CoeffID()), uint32(t.WireID()))
		}
	}

	(*to) = append((*to), h.OutputRange.Start)
	(*to) = append((*to), h.OutputRange.End)
}

func (b *BlueprintGenericHint) CalldataSize() int {
	return -1
}
func (b *BlueprintGenericHint) NbConstraints() int {
	return 0
}

func (b *BlueprintGenericHint) NbOutputs(inst Instruction) int {
	return 0
}

func (b *BlueprintGenericHint) WireWalker(inst Instruction) func(cb func(wire uint32)) {
	return func(cb func(wire uint32)) {
		lenInputs := int(inst.Calldata[2])
		j := 3
		for i := 0; i < lenInputs; i++ {
			n := int(inst.Calldata[j]) // len of linear expr
			j++

			for k := 0; k < n; k++ {
				t := Term{CID: inst.Calldata[j], VID: inst.Calldata[j+1]}
				if !t.IsConstant() {
					cb(t.VID)
				}
				j += 2
			}
		}
		for k := inst.Calldata[j]; k < inst.Calldata[j+1]; k++ {
			cb(k)
		}
	}
}
