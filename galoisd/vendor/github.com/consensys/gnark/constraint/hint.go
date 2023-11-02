package constraint

import (
	"github.com/consensys/gnark/constraint/solver"
)

// HintMapping mark a list of output variables to be computed using provided hint and inputs.
type HintMapping struct {
	HintID      solver.HintID      // Hint function id
	Inputs      []LinearExpression // Terms to inject in the hint function
	OutputRange struct {           // IDs of wires the hint outputs map to
		Start, End uint32
	}
}
