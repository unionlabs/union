package constraint

import (
	"math/big"

	"github.com/consensys/gnark/constraint/solver"
)

const CommitmentDst = "bsb22-commitment"

type Commitment struct {
	Committed              []int // sorted list of id's of committed variables in groth16. in plonk, list of indexes of constraints defining committed values
	NbPrivateCommitted     int
	HintID                 solver.HintID // TODO @gbotrel we probably don't need that here
	CommitmentIndex        int           // in groth16, CommitmentIndex is the wire index. in plonk, it's the constraint defining it
	CommittedAndCommitment []int         // sorted list of id's of committed variables AND the commitment itself
}

func (i *Commitment) NbPublicCommitted() int {
	return i.NbCommitted() - i.NbPrivateCommitted
}

func (i *Commitment) NbCommitted() int {
	return len(i.Committed)
}

func (i *Commitment) Is() bool {
	return len(i.Committed) != 0
}

// NewCommitment initialize a Commitment object
//   - committed are the sorted wireID to commit to (without duplicate)
//   - nbPublicCommited is the number of public inputs among the commited wireIDs
func NewCommitment(committed []int, nbPublicCommitted int) Commitment {
	return Commitment{
		Committed:          committed,
		NbPrivateCommitted: len(committed) - nbPublicCommitted,
	}
}

func (i *Commitment) SerializeCommitment(privateCommitment []byte, publicCommitted []*big.Int, fieldByteLen int) []byte {

	res := make([]byte, len(privateCommitment)+len(publicCommitted)*fieldByteLen)
	copy(res, privateCommitment)

	offset := len(privateCommitment)
	for j, inJ := range publicCommitted {
		offset += j * fieldByteLen
		inJ.FillBytes(res[offset : offset+fieldByteLen])
	}

	return res
}

// PrivateToPublic returns indexes of variables which are private to the constraint system, but public to Groth16. That is, private committed variables and the commitment itself
func (i *Commitment) PrivateToPublic() []int {
	return i.CommittedAndCommitment[i.NbPublicCommitted():]
}

func (i *Commitment) PrivateCommitted() []int {
	return i.Committed[i.NbPublicCommitted():]
}
