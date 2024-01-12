package cs

import (
	"math/big"

	"github.com/consensys/gnark/constraint"
)

// CoeffTable helps build a constraint system but need not be serialized after compilation
type CoeffTable struct {
	// Coefficients in the constraints
	Coeffs         []big.Int      // list of unique coefficients.
	CoeffsIDsLarge map[string]int // map to check existence of a coefficient (key = coeff.Bytes())
	CoeffsIDsInt64 map[int64]int  // map to check existence of a coefficient (key = int64 value)
}

func NewCoeffTable() CoeffTable {
	st := CoeffTable{
		Coeffs:         make([]big.Int, 4),
		CoeffsIDsLarge: make(map[string]int),
		CoeffsIDsInt64: make(map[int64]int, 4),
	}

	st.Coeffs[constraint.CoeffIdZero].SetInt64(0)
	st.Coeffs[constraint.CoeffIdOne].SetInt64(1)
	st.Coeffs[constraint.CoeffIdTwo].SetInt64(2)
	st.Coeffs[constraint.CoeffIdMinusOne].SetInt64(-1)
	st.CoeffsIDsInt64[0] = constraint.CoeffIdZero
	st.CoeffsIDsInt64[1] = constraint.CoeffIdOne
	st.CoeffsIDsInt64[2] = constraint.CoeffIdTwo
	st.CoeffsIDsInt64[-1] = constraint.CoeffIdMinusOne

	return st
}

// CoeffID tries to fetch the entry where b is if it exists, otherwise appends b to
// the list of Coeffs and returns the corresponding entry
func (t *CoeffTable) CoeffID(v *big.Int) int {

	// if the coeff is a int64 we have a fast path.
	if v.IsInt64() {
		return t.coeffID64(v.Int64())
	}

	// GobEncode is 3x faster than b.Text(16). Slightly slower than Bytes, but Bytes return the same
	// thing for -x and x .
	bKey, _ := v.GobEncode()
	key := string(bKey)

	// if the coeff is already stored, fetch its ID from the cs.CoeffsIDs map
	if idx, ok := t.CoeffsIDsLarge[key]; ok {
		return idx
	}

	// else add it in the cs.Coeffs map and update the cs.CoeffsIDs map
	var bCopy big.Int
	bCopy.Set(v)
	resID := len(t.Coeffs)
	t.Coeffs = append(t.Coeffs, bCopy)
	t.CoeffsIDsLarge[key] = resID
	return resID
}

func (t *CoeffTable) coeffID64(v int64) int {
	if resID, ok := t.CoeffsIDsInt64[v]; ok {
		return resID
	} else {
		var bCopy big.Int
		bCopy.SetInt64(v)
		resID := len(t.Coeffs)
		t.Coeffs = append(t.Coeffs, bCopy)
		t.CoeffsIDsInt64[v] = resID
		return resID
	}
}
