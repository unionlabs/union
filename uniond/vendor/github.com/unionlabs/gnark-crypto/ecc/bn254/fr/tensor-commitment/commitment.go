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

package tensorcommitment

import (
	"bytes"
	"errors"
	"hash"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/fft"
	"github.com/consensys/gnark-crypto/internal/parallel"
)

var (
	ErrWrongSize           = errors.New("polynomial is too large")
	ErrNotSquare           = errors.New("the size of the polynomial must be a square")
	ErrProofFailedHash     = errors.New("hash of one of the columns is wrong")
	ErrProofFailedEncoding = errors.New("inconsistency with the code word")
	ErrProofFailedOob      = errors.New("the entry is out of bound")
	ErrMaxNbColumns        = errors.New("the state is full")
	ErrCommitmentNotDone   = errors.New("the proof cannot be built before the computation of the digest")
)

// commitment (TODO Merkle tree for that...)
// The i-th entry is the hash of the i-th columns of P,
// where P is written as a matrix √(m) x √(m)
// (m = len(P)), and the ij-th entry of M is p[m*j + i].
type Digest [][]byte

// Proof that a commitment is correct
// cf https://eprint.iacr.org/2021/1043.pdf page 10
type Proof struct {

	// list of entries of ̂{u} to query (see https://eprint.iacr.org/2021/1043.pdf for notations)
	EntryList []int

	// columns on against which the linear combination is checked
	// (the i-th entry is the EntryList[i]-th column)
	Columns [][]fr.Element

	// Linear combination of the rows of the polynomial P written as a square matrix
	LinearCombination []fr.Element

	// small domain, to retrieve the canonical form of the linear combination
	Domain *fft.Domain

	// root of unity of the big domain
	Generator fr.Element
}

// TcParams stores the public parameters of the tensor commitment
type TcParams struct {
	// NbColumns number of columns of the matrix storing the polynomials. The total size of
	// the polynomials which are committed is NbColumns x NbRows.
	// The Number of columns is a power of 2, it corresponds to the original size of the codewords
	// of the Reed Solomon code.
	NbColumns int

	// NbRows number of rows of the matrix storing the polynomials. If a polynomial p is appended
	// whose size if not 0 mod NbRows, it is padded as p' so that len(p')=0 mod NbRows.
	NbRows int

	// Domains[1] used for the Reed Solomon encoding
	Domains [2]*fft.Domain

	// Rho⁻¹, rate of the RS code ( > 1)
	Rho int

	// Function that returns a fresh hasher. The returned hash function is used for hashing the
	// columns. We use this and not directly a hasher for threadsafety hasher. Indeed, if different
	// thread share the same hasher, they will end up mixing hash inputs that should remain separate.
	MakeHash func() hash.Hash
}

// TensorCommitment stores the data to use a tensor commitment
type TensorCommitment struct {
	// The public parameters of the tensor commitment
	params *TcParams

	// State contains the polynomials that have been appended so far.
	// when we append a polynomial p, it is stored in the state like this:
	// state[i][j] = p[j*nbRows + i]:
	// p[0] 		| p[nbRows] 	| p[2*nbRows] 	...
	// p[1] 		| p[nbRows+1]	| p[2*nbRows+1]
	// p[2] 		| p[nbRows+2]	| p[2*nbRows+2]
	// ..
	// p[nbRows-1] 	| p[2*nbRows-1]	| p[3*nbRows-1] ..
	State [][]fr.Element

	// same content as state, but the polynomials are displayed as a matrix
	// and the rows are encoded.
	// encodedState = encodeRows(M_0 || .. || M_n)
	// where M_i is the i-th polynomial laid out as a matrix, that is
	// M_i_jk = p_i[i*m+j] where m = \sqrt(len(p)).
	EncodedState [][]fr.Element

	// boolean telling if the commitment has already been done.
	// The method BuildProof cannot be called before Commit(),
	// because it would allow to build a proof before giving the commitment
	// to a verifier, making the workflow not secure.
	isCommitted bool

	// number of columns which have already been hashed (atomic)
	NbColumnsHashed int

	// counts the number of time `Append` was called (atomic).
	NbAppendsSoFar int
}

// NewTensorCommitment returns a new TensorCommitment
// * ρ rate of the code ( > 1)
// * size size of the polynomial to be committed. The size of the commitment is
// then ρ * √(m) where m² = size
func NewTCParams(codeRate, NbColumns, NbRows int, makeHash func() hash.Hash) (*TcParams, error) {
	var res TcParams

	// domain[0]: domain to perform the FFT^-1, of size capacity * sqrt
	// domain[1]: domain to perform FFT, of size rho * capacity * sqrt
	res.Domains[0] = fft.NewDomain(uint64(NbColumns))
	res.Domains[1] = fft.NewDomain(uint64(codeRate * NbColumns))

	// size of the matrix
	res.NbColumns = int(res.Domains[0].Cardinality)
	res.NbRows = NbRows

	// rate
	res.Rho = codeRate

	// Hash function
	res.MakeHash = makeHash

	return &res, nil
}

// Initializes an instance of tensor commitment that we can use start
// appending value into it
func NewTensorCommitment(params *TcParams) *TensorCommitment {
	var res TensorCommitment

	// create the state. It's the matrix containing the polynomials, the ij-th
	// entry of the matrix is state[i][j]. The polynomials are split and stacked
	// columns per column.
	res.State = make([][]fr.Element, params.NbRows)
	for i := 0; i < params.NbRows; i++ {
		res.State[i] = make([]fr.Element, params.NbColumns)
	}

	// nothing has been committed...
	res.isCommitted = false
	res.params = params
	return &res
}

// Append appends p to the state.
// when we append a polynomial p, it is stored in the state like this:
// state[i][j] = p[j*nbRows + i]:
// p[0] 		| p[nbRows] 	| p[2*nbRows] 	...
// p[1] 		| p[nbRows+1]	| p[2*nbRows+1]
// p[2] 		| p[nbRows+2]	| p[2*nbRows+2]
// ..
// p[nbRows-1] 	| p[2*nbRows-1]	| p[3*nbRows-1] ..
// If p doesn't fill a full submatrix it is padded with zeroes.
func (tc *TensorCommitment) Append(ps ...[]fr.Element) ([][]byte, error) {

	nbColumnsTakenByPs := make([]int, len(ps))
	totalNumberOfColumnsTakenByPs := 0
	// Short-hand to avoid writing `tc.params.NbRows` all over the places
	numRows := tc.params.NbRows

	/*
		Precomputes the number of columns that will be taken by each colums
	*/
	for iPol, p := range ps {
		// check if there is some room for p
		nbColumnsTakenByP := len(p) / numRows
		// Note, Alex. Really, if you want to not handle the padding and just
		// panic whenever you receive "incomplete" columns this would be fine.
		if len(p)%numRows != 0 {
			// If the division has a remainder. Add an extra column
			// Implicitly, it will be padded
			nbColumnsTakenByP += 1
		}

		nbColumnsTakenByPs[iPol] = nbColumnsTakenByP
		totalNumberOfColumnsTakenByPs += nbColumnsTakenByP
	}

	// Position at which we need to start inserting columns in the state
	currentColumnToFill := int(tc.NbColumnsHashed)

	// Check that we are not inserting more columns that we can handle
	if currentColumnToFill+totalNumberOfColumnsTakenByPs > tc.params.NbColumns {
		return nil, ErrMaxNbColumns
	}

	// Update the internal state variables to keep track of how many poly
	// have been appended so far and how many columns.
	tc.NbAppendsSoFar += len(ps)
	tc.NbColumnsHashed += totalNumberOfColumnsTakenByPs

	backupCurrentColumnToFill := currentColumnToFill

	// put p in the state
	for iPol, p := range ps {

		pIsPadded := false
		if len(p)%numRows != 0 {
			pIsPadded = true
		}

		// Number of column taken by P, ignoring the last one if it is padded
		nbFullColumnsTakenByP := nbColumnsTakenByPs[iPol]
		if pIsPadded {
			nbFullColumnsTakenByP--
		}

		// Insert the "full columns" in the state
		for i := 0; i < nbFullColumnsTakenByP; i++ {
			for j := 0; j < numRows; j++ {
				tc.State[j][currentColumnToFill+i] = p[i*numRows+j]
			}
		}

		// Insert the padded column in the state if any
		currentColumnToFill += nbFullColumnsTakenByP
		if pIsPadded {
			offsetP := len(p) - len(p)%numRows
			for j := offsetP; j < len(p); j++ {
				tc.State[j-offsetP][currentColumnToFill] = p[j]
			}
			currentColumnToFill += 1
		}
	}

	// Preallocate the result, and as well a buffer for the columns to hash
	res := make([][]byte, totalNumberOfColumnsTakenByPs)

	parallel.Execute(totalNumberOfColumnsTakenByPs, func(start, stop int) {
		hasher := tc.params.MakeHash()
		for i := start; i < stop; i++ {
			hasher.Reset()
			for j := 0; j < tc.params.NbRows; j++ {
				hasher.Write(tc.State[j][i+backupCurrentColumnToFill].Marshal())
			}
			res[i] = hasher.Sum(nil)
		}
	})

	return res, nil
}

// Commit to p. The commitment procedure is the following:
// * Encode the rows of the state to get M'
// * Hash the columns of M'
func (tc *TensorCommitment) Commit() (Digest, error) {

	// we encode the rows of p using Reed Solomon
	// encodedState[i][:] = i-th line of M. It is of size domain[1].Cardinality
	tc.EncodedState = make([][]fr.Element, tc.params.NbRows)
	for i := 0; i < tc.params.NbRows; i++ { // we fill encodedState line by line
		tc.EncodedState[i] = make([]fr.Element, tc.params.Domains[1].Cardinality) // size = NbRows*rho*capacity
		for j := 0; j < tc.params.NbColumns; j++ {                                // for each polynomial
			tc.EncodedState[i][j].Set(&tc.State[i][j])
		}
		tc.params.Domains[0].FFTInverse(tc.EncodedState[i][:tc.params.Domains[0].Cardinality], fft.DIF)
		fft.BitReverse(tc.EncodedState[i][:tc.params.Domains[0].Cardinality])
		tc.params.Domains[1].FFT(tc.EncodedState[i], fft.DIF)
		fft.BitReverse(tc.EncodedState[i])
	}

	// now we hash each columns of _p
	res := make([][]byte, tc.params.Domains[1].Cardinality)

	parallel.Execute(int(tc.params.Domains[1].Cardinality), func(start, stop int) {
		hasher := tc.params.MakeHash()
		for i := start; i < stop; i++ {
			hasher.Reset()
			for j := 0; j < tc.params.NbRows; j++ {
				hasher.Write(tc.EncodedState[j][i].Marshal())
			}
			res[i] = hasher.Sum(nil)
		}
	})

	// records that the commitment has been built
	tc.isCommitted = true

	return res, nil

}

// BuildProofAtOnceForTest builds a proof to be tested against a previous commitment of a list of
// polynomials.
// * l the random linear coefficients used for the linear combination of size NbRows
// * entryList list of columns to hash
// l and entryList are supposed to be precomputed using Fiat Shamir
//
// The proof is the linear combination (using l) of the encoded rows of p written
// as a matrix. Only the entries contained in entryList are kept.
func (tc *TensorCommitment) BuildProofAtOnceForTest(l []fr.Element, entryList []int) (Proof, error) {
	linComb, err := tc.ProverComputeLinComb(l)
	if err != nil {
		return Proof{}, err
	}

	openedColumns, err := tc.ProverOpenColumns(entryList)
	if err != nil {
		return Proof{}, err
	}

	return BuildProof(tc.params, linComb, entryList, openedColumns), nil
}

// func printVector(v []fr.Element) {
// 	fmt.Printf("[")
// 	for i := 0; i < len(v); i++ {
// 		fmt.Printf("%s,", v[i].String())
// 	}
// 	fmt.Printf("]\n")
// }

// BuildProof builds a proof to be tested against a previous commitment of a list of
// polynomials.
// * l the random linear coefficients used for the linear combination of size NbRows
// * entryList list of columns to hash
// l and entryList are supposed to be precomputed using Fiat Shamir
//
// The proof is the linear combination (using l) of the encoded rows of p written
// as a matrix. Only the entries contained in entryList are kept.
func (tc *TensorCommitment) ProverComputeLinComb(l []fr.Element) ([]fr.Element, error) {

	// check that the digest has been computed
	if !tc.isCommitted {
		return []fr.Element{}, ErrCommitmentNotDone
	}

	// since the digest has been computed, the encodedState is already stored.
	// We use it to build the proof, without recomputing the ffts.

	// linear combination of the rows of the state
	linComb := make([]fr.Element, tc.params.NbColumns)
	for i := 0; i < tc.params.NbColumns; i++ {
		var tmp fr.Element
		for j := 0; j < tc.params.NbRows; j++ {
			tmp.Mul(&tc.State[j][i], &l[j])
			linComb[i].Add(&linComb[i], &tmp)
		}
	}

	return linComb, nil
}

func (tc *TensorCommitment) ProverOpenColumns(entryList []int) ([][]fr.Element, error) {

	// check that the digest has been computed
	if !tc.isCommitted {
		return [][]fr.Element{}, ErrCommitmentNotDone
	}

	// columns of the state whose rows have been encoded, written as a matrix,
	// corresponding to the indices in entryList (we will select the columns
	// entryList[0], entryList[1], etc.
	openedColumns := make([][]fr.Element, len(entryList))
	for i := 0; i < len(entryList); i++ { // for each column (corresponding to an elmt in entryList)
		openedColumns[i] = make([]fr.Element, tc.params.NbRows)
		for j := 0; j < tc.params.NbRows; j++ {
			openedColumns[i][j] = tc.EncodedState[j][entryList[i]]
		}
	}

	return openedColumns, nil
}

/*
Reconstruct the proof from the prover's outputs
*/
func BuildProof(params *TcParams, linComb []fr.Element, entryList []int, openedCols [][]fr.Element) Proof {

	var res Proof

	// small domain to express the linear combination in canonical form
	res.Domain = params.Domains[0]

	// generator g of the biggest domain, used to evaluate the canonical form of
	// the linear combination at some powers of g.
	res.Generator.Set(&params.Domains[1].Generator)

	res.Columns = openedCols
	res.EntryList = entryList
	res.LinearCombination = linComb

	return res
}

// evalAtPower returns p(x**n) where p is interpreted as a polynomial
// p[0] + p[1]X + .. p[len(p)-1]xˡᵉⁿ⁽ᵖ⁾⁻¹
func evalAtPower(p []fr.Element, x fr.Element, n int) fr.Element {

	var xexp fr.Element
	xexp.Exp(x, big.NewInt(int64(n)))

	var res fr.Element
	for i := 0; i < len(p); i++ {
		res.Mul(&res, &xexp)
		res.Add(&p[len(p)-1-i], &res)
	}

	return res

}

// Verify a proof that digest is the hash of a  polynomial given a proof
// proof: contains the linear combination of the non-encoded rows + the
// digest: hash of the polynomial
// l: random coefficients for the linear combination, chosen by the verifier
// h: hash function that is used for hashing the columns of the polynomial
// TODO make this function private and add a Verify function that derives
// the randomness using Fiat Shamir
//
// Note (alex), A more convenient API would be to expose two functions,
// one that does FS for you and what that let you do it for yourself. And likewise
// for the prover.
func Verify(proof Proof, digest Digest, l []fr.Element, h hash.Hash) error {

	// for each entry in the list -> it corresponds to the sampling
	// set on which we probabilistically check that
	// Encoded(linear_combination) = linear_combination(encoded)
	for i := 0; i < len(proof.EntryList); i++ {

		// check that the hash of the columns correspond to what's in the digest
		h.Reset()
		for j := 0; j < len(proof.Columns[i]); j++ {
			h.Write(proof.Columns[i][j].Marshal())
		}
		s := h.Sum(nil)
		if !bytes.Equal(s, digest[proof.EntryList[i]]) {
			return ErrProofFailedHash
		}

		if proof.EntryList[i] >= len(digest) {
			return ErrProofFailedOob
		}

		// linear combination of the i-th column, whose entries
		// are the entryList[i]-th entries of the encoded lines
		// of p
		var linCombEncoded, tmp fr.Element
		for j := 0; j < len(proof.Columns[i]); j++ {

			// linear combination of the encoded rows at column i
			tmp.Mul(&proof.Columns[i][j], &l[j])
			linCombEncoded.Add(&linCombEncoded, &tmp)
		}

		// entry i of the encoded linear combination
		var encodedLinComb fr.Element
		linCombCanonical := make([]fr.Element, proof.Domain.Cardinality)
		copy(linCombCanonical, proof.LinearCombination)
		proof.Domain.FFTInverse(linCombCanonical, fft.DIF)
		fft.BitReverse(linCombCanonical)
		encodedLinComb = evalAtPower(linCombCanonical, proof.Generator, proof.EntryList[i])

		// compare both values
		if !encodedLinComb.Equal(&linCombEncoded) {
			return ErrProofFailedEncoding

		}
	}

	return nil

}
