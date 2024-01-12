package constraint

import (
	"math/big"
)

const CommitmentDst = "bsb22-commitment"

type Groth16Commitment struct {
	PublicAndCommitmentCommitted []int // PublicAndCommitmentCommitted sorted list of id's of public and commitment committed wires
	PrivateCommitted             []int // PrivateCommitted sorted list of id's of private/internal committed wires
	CommitmentIndex              int   // CommitmentIndex the wire index of the commitment
	NbPublicCommitted            int
}

type PlonkCommitment struct {
	Committed       []int // sorted list of id's of committed variables in groth16. in plonk, list of indexes of constraints defining committed values
	CommitmentIndex int   // CommitmentIndex index of the constraint defining the commitment
}

type Commitment interface{}
type Commitments interface{ CommitmentIndexes() []int }

type Groth16Commitments []Groth16Commitment
type PlonkCommitments []PlonkCommitment

func (c Groth16Commitments) CommitmentIndexes() []int {
	commitmentWires := make([]int, len(c))
	for i := range c {
		commitmentWires[i] = c[i].CommitmentIndex
	}
	return commitmentWires
}

func (c PlonkCommitments) CommitmentIndexes() []int {
	commitmentWires := make([]int, len(c))
	for i := range c {
		commitmentWires[i] = c[i].CommitmentIndex
	}
	return commitmentWires
}

func (c Groth16Commitments) GetPrivateCommitted() [][]int {
	res := make([][]int, len(c))
	for i := range c {
		res[i] = c[i].PrivateCommitted
	}
	return res
}

// GetPublicAndCommitmentCommitted returns the list of public and commitment committed wires
// if committedTranslationList is not nil, commitment indexes are translated into their relative positions on the list plus the offset
func (c Groth16Commitments) GetPublicAndCommitmentCommitted(committedTranslationList []int, offset int) [][]int {
	res := make([][]int, len(c))
	for i := range c {
		res[i] = make([]int, len(c[i].PublicAndCommitmentCommitted))
		copy(res[i], c[i].GetPublicCommitted())
		translatedCommitmentCommitted := res[i][c[i].NbPublicCommitted:]
		commitmentCommitted := c[i].GetCommitmentCommitted()
		// convert commitment indexes to verifier understandable ones
		if committedTranslationList == nil {
			copy(translatedCommitmentCommitted, commitmentCommitted)
		} else {
			k := 0
			for j := range translatedCommitmentCommitted {
				for committedTranslationList[k] != commitmentCommitted[j] {
					k++
				} // find it in the translation list
				translatedCommitmentCommitted[j] = k + offset
			}
		}
	}
	return res
}

func SerializeCommitment(privateCommitment []byte, publicCommitted []*big.Int, fieldByteLen int) []byte {

	res := make([]byte, len(privateCommitment)+len(publicCommitted)*fieldByteLen)
	copy(res, privateCommitment)

	offset := len(privateCommitment)
	for _, inJ := range publicCommitted {
		inJ.FillBytes(res[offset : offset+fieldByteLen])
		offset += fieldByteLen
	}

	return res
}

func NewCommitments(t SystemType) Commitments {
	switch t {
	case SystemR1CS:
		return Groth16Commitments{}
	case SystemSparseR1CS:
		return PlonkCommitments{}
	}
	panic("unknown cs type")
}

func (c Groth16Commitment) GetPublicCommitted() []int {
	return c.PublicAndCommitmentCommitted[:c.NbPublicCommitted]
}

func (c Groth16Commitment) GetCommitmentCommitted() []int {
	return c.PublicAndCommitmentCommitted[c.NbPublicCommitted:]
}
