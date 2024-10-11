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
	"hash"
	"math/big"
	"math/bits"
	"strconv"
	"testing"

	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/sis"
	"github.com/stretchr/testify/require"
)

type DummyHash uint

func (d DummyHash) Write(p []byte) (n int, err error) {
	return 0, nil
}

func (d DummyHash) Sum(b []byte) []byte {
	return b
}

func (d DummyHash) Reset() {}

func (d DummyHash) Size() int {
	return 0
}

func (d DummyHash) BlockSize() int {
	return 0
}

func DummyHashMaker() hash.Hash {
	var res DummyHash
	return &res
}

func TestAppend(t *testing.T) {
	if bits.UintSize == 32 {
		t.Skip("skipping this test in 32bit.")
	}

	assert := require.New(t)

	// tensor commitment
	const (
		rho       = 4
		nbRows    = 10
		nbColumns = 16
	)
	params, err := NewTCParams(rho, nbColumns, nbRows, DummyHashMaker)
	assert.NoError(err)

	tc := NewTensorCommitment(params)

	{
		// random Polynomial of size nbRows
		p := make([]fr.Element, nbRows)
		for i := 0; i < nbRows; i++ {
			p[i].SetRandom()
		}
		_, err := tc.Append(p)
		assert.NoError(err)

		// check if p corresponds to the first column of the state
		for i := 0; i < nbRows; i++ {
			assert.True(tc.State[i][0].Equal(&p[i]), "a column is not filled correctly")
		}

	}

	// after a first polynomial has been filled
	{
		// random Polynomial of size nbRows
		p := make([]fr.Element, nbRows)
		for i := 0; i < nbRows; i++ {
			p[i].SetRandom()
		}
		_, err := tc.Append(p)
		assert.NoError(err)

		// check if p corresponds to the second column of the state
		for i := 0; i < nbRows; i++ {
			assert.True(tc.State[i][1].Equal(&p[i]), "a column is not filled correctly")
		}
	}

	// polynomial whose size is not a multiple of nbRows
	{
		// random Polynomial of size nbRows
		offset := 4
		p := make([]fr.Element, nbRows+offset)
		for i := 0; i < nbRows+offset; i++ {
			p[i].SetRandom()
		}
		_, err := tc.Append(p)
		assert.NoError(err)

		// check if p corresponds to the first column of the state
		for i := 0; i < nbRows; i++ {
			assert.True(tc.State[i][2].Equal(&p[i]), "a column is not filled correctly")
		}
		for i := 0; i < offset; i++ {
			assert.True(tc.State[i][3].Equal(&p[i+nbRows]), "a column is not filled correctly")
		}
	}

	// same to see if the last column was correctly offset
	{
		// random Polynomial of size nbRows
		offset := 4
		p := make([]fr.Element, nbRows+offset)
		for i := 0; i < nbRows+offset; i++ {
			p[i].SetRandom()
		}
		_, err := tc.Append(p)
		assert.NoError(err)

		// check if p corresponds to the first column of the state
		for i := 0; i < nbRows; i++ {
			assert.True(tc.State[i][4].Equal(&p[i]), "a column is not filled correctly")
		}
		for i := 0; i < offset; i++ {
			assert.True(tc.State[i][5].Equal(&p[i+nbRows]), "a column is not filled correctly")
		}
	}

}

func TestLinearCombination(t *testing.T) {

	rho := 4
	nbRows := 8
	nbColumns := 8
	params, err := NewTCParams(rho, nbColumns, nbRows, DummyHashMaker)
	if err != nil {
		t.Fatal(err)
	}
	tc := NewTensorCommitment(params)

	// build a random polynomial
	p := make([]fr.Element, nbRows*nbColumns)
	for i := 0; i < 64; i++ {
		p[i].SetRandom()
	}

	// we select all the entries for the test
	entryList := make([]int, rho*nbColumns)
	for i := 0; i < rho*nbColumns; i++ {
		entryList[i] = i
	}

	// append p and commit (otherwise the proof cannot be built)
	tc.Append(p)
	_, err = tc.Commit()
	if err != nil {
		t.Fatal(err)
	}

	// at each trial, it's the i-th line which is selected
	for i := 0; i < nbRows; i++ {

		// used for the random linear combination.
		// it will act as a selector for the test: it selects the i-th
		// row of p, when p is written as a matrix M_ij, where M_ij=p[i*m+j].
		// The i-th entry of l is 1, the others are 0.
		l := make([]fr.Element, nbRows)
		l[i].SetInt64(1)

		proof, err := tc.BuildProofAtOnceForTest(l, entryList)
		if err != nil {
			t.Fatal(err)
		}

		// the i-th line of p is the one that is supposed to be selected
		// (corresponding to the linear combination)
		expected := make([]fr.Element, nbColumns)
		for j := 0; j < nbColumns; j++ {
			expected[j].Set(&p[j*nbRows+i])
		}

		for j := 0; j < nbColumns; j++ {
			if !expected[j].Equal(&proof.LinearCombination[j]) {
				t.Fatal("expected linear combination is incorrect")
			}
		}

	}
}

// Test the verification of a correct proof using a mock hash
func TestCommitmentDummyHash(t *testing.T) {

	var rho, nbColumns, nbRows int
	rho = 4
	nbColumns = 8
	nbRows = 8

	var h DummyHash
	params, err := NewTCParams(rho, nbColumns, nbRows, DummyHashMaker)
	if err != nil {
		t.Fatal(err)
	}
	tc := NewTensorCommitment(params)

	// random polynomial
	p := make([]fr.Element, nbRows*nbColumns)
	for i := 0; i < nbRows*nbColumns; i++ {
		p[i].SetRandom()
	}

	// coefficients for the linear combination
	l := make([]fr.Element, nbRows)
	for i := 0; i < nbRows; i++ {
		l[i].SetRandom()
	}

	// we select all the entries for the test
	entryList := make([]int, rho*nbColumns)
	for i := 0; i < rho*nbColumns; i++ {
		entryList[i] = i
	}

	// compute the digest...
	_, err = tc.Append(p)
	if err != nil {
		t.Fatal(err)
	}
	digest, err := tc.Commit()
	if err != nil {
		t.Fatal(err)
	}

	// build the proof...
	proof, err := tc.BuildProofAtOnceForTest(l, entryList)
	if err != nil {
		t.Fatal(err)
	}

	// verify that the proof is correct
	err = Verify(proof, digest, l, h)
	if err != nil {
		t.Fatal(err)
	}

}

// Test the opening using a dummy hash
func TestOpeningDummyHash(t *testing.T) {

	var rho, nbColumns, nbRows int
	rho = 4
	nbColumns = 8
	nbRows = 8

	params, err := NewTCParams(rho, nbColumns, nbRows, DummyHashMaker)
	if err != nil {
		t.Fatal(err)
	}
	tc := NewTensorCommitment(params)

	// random polynomial
	p := make([]fr.Element, nbColumns*nbRows)
	for i := 0; i < nbColumns*nbRows; i++ {
		p[i].SetRandom()
	}

	// the coefficients are (1,x,x^2,..,x^{n-1}) where x is the point
	// at which the opening is done
	var xm, x fr.Element
	x.SetRandom()
	hi := make([]fr.Element, nbColumns) // stores [1,x^{nbRows},..,x^{nbRows*nbColumns^-1}]
	lo := make([]fr.Element, nbRows)    // stores [1,x,..,x^{nbRows-1}]
	lo[0].SetInt64(1)
	hi[0].SetInt64(1)
	xm.Exp(x, big.NewInt(int64(nbRows)))
	for i := 1; i < nbColumns; i++ {
		lo[i].Mul(&lo[i-1], &x)
		hi[i].Mul(&hi[i-1], &xm)
	}

	// create the digest before computing the proof
	_, err = tc.Append(p)
	if err != nil {
		t.Fatal(err)
	}
	_, err = tc.Commit()
	if err != nil {
		t.Fatal(err)
	}

	// build the proof
	entryList := make([]int, rho*nbColumns)
	for i := 0; i < rho*nbColumns; i++ {
		entryList[i] = i
	}
	proof, err := tc.BuildProofAtOnceForTest(lo, entryList)
	if err != nil {
		t.Fatal(err)
	}

	// finish the evaluation by computing
	// [linearCombination] * [hi]^t
	var eval, tmp fr.Element
	for i := 0; i < nbColumns; i++ {
		tmp.Mul(&proof.LinearCombination[i], &hi[i])
		eval.Add(&eval, &tmp)
	}

	// compute the real evaluation of p at x manually
	var expectedEval fr.Element
	for i := 0; i < nbRows*nbColumns; i++ {
		expectedEval.Mul(&expectedEval, &x)
		expectedEval.Add(&expectedEval, &p[len(p)-i-1])
	}

	// the results coincide
	if !expectedEval.Equal(&eval) {
		t.Fatal("p(x) != [ lo ] x M x [ hi ]^t")
	}

}

// Check the commitments are correctly formed when appending a polynomial
func TestAppendSis(t *testing.T) {
	if bits.UintSize == 32 {
		t.Skip("skipping this test in 32bit.")
	}
	const (
		rho          = 4
		nbColumns    = 8
		nbRows       = 8
		logTwoDegree = 1
		logTwoBound  = 4
	)

	assert := require.New(t)

	// keySize := 256
	hMaker, err := sis.NewRingSISMaker(5, logTwoDegree, logTwoBound, 8)
	assert.NoError(err)

	params, err := NewTCParams(rho, nbColumns, nbRows, hMaker)
	assert.NoError(err)

	tc := NewTensorCommitment(params)

	// random polynomial (that does not fill the full matrix)
	offset := 4
	p := make([]fr.Element, nbRows*nbColumns-offset)
	for i := 0; i < nbRows*nbColumns-offset; i++ {
		p[i].SetRandom()
	}

	s, err := tc.Append(p)
	assert.NoError(err)

	assert.Equal(nbColumns, len(s))

	// check the hashes of the columns
	h := hMaker()
	for i := 0; i < nbColumns-1; i++ {
		h.Reset()
		for j := 0; j < nbRows; j++ {
			h.Write(p[i*nbRows+j].Marshal())
		}
		_s := h.Sum(nil)
		assert.True(bytes.Equal(_s, s[i]), "error hash column when appending a polynomial for column", i)
	}

	// last column
	h.Reset()
	for i := (nbColumns - 1) * nbRows; i < nbColumns*nbRows-offset; i++ {
		h.Write(p[i].Marshal())
	}
	var tmp fr.Element
	for i := nbColumns*nbRows - offset; i < nbColumns*nbRows; i++ {
		h.Write(tmp.Marshal())
	}
	_s := h.Sum(nil)
	assert.True(bytes.Equal(_s, s[nbColumns-1]), "error hash column when appending a polynomial")
}

// Test the verification of a correct proof using SIS as hash
func TestCommitmentSis(t *testing.T) {
	if bits.UintSize == 32 {
		t.Skip("skipping this test in 32bit.")
	}
	var rho, nbColumns, nbRows int
	rho = 4
	nbColumns = 8
	nbRows = 8

	logTwoDegree := 1
	logTwoBound := 4
	hMaker, err := sis.NewRingSISMaker(5, logTwoDegree, logTwoBound, 8)
	if err != nil {
		t.Fatal(err)
	}

	params, err := NewTCParams(rho, nbColumns, nbRows, hMaker)
	if err != nil {
		t.Fatal(err)
	}
	tc := NewTensorCommitment(params)

	// random polynomial
	p := make([]fr.Element, nbRows*nbColumns)
	for i := 0; i < nbRows*nbColumns; i++ {
		p[i].SetRandom()
	}

	// coefficients for the linear combination
	l := make([]fr.Element, nbRows)
	for i := 0; i < nbRows; i++ {
		l[i].SetRandom()
	}

	// compute the digest...
	_, err = tc.Append(p)
	if err != nil {
		t.Fatal(err)
	}
	digest, err := tc.Commit()
	if err != nil {
		t.Fatal(err)
	}

	// test 1: we select all the entries
	{
		entryList := make([]int, rho*nbColumns)
		for i := 0; i < rho*nbColumns; i++ {
			entryList[i] = i
		}

		// build the proof...
		proof, err := tc.BuildProofAtOnceForTest(l, entryList)
		if err != nil {
			t.Fatal(err)
		}

		// verify that the proof is correct
		err = Verify(proof, digest, l, hMaker())
		if err != nil {
			t.Fatal(err)
		}
	}
	// test 2: we select a subset of the entries
	{

		entryList := make([]int, 2)
		entryList[0] = 1
		entryList[1] = 4

		// build the proof...
		proof, err := tc.BuildProofAtOnceForTest(l, entryList)
		if err != nil {
			t.Fatal(err)
		}

		// verify that the proof is correct
		err = Verify(proof, digest, l, hMaker())
		if err != nil {
			t.Fatal(err)
		}
	}
}

// benches
func BenchmarkTensorCommitment(b *testing.B) {

	// prepare the tensor commitment
	logTwoDegree := 4
	logTwoBound := 4
	rho := 4

	for i := 0; i < 6; i++ {

		nbColumns := (1 << (3 + i))
		nbRows := nbColumns

		h, _ := sis.NewRingSISMaker(5, logTwoDegree, logTwoBound, nbRows)
		params, _ := NewTCParams(rho, nbColumns, nbRows, h)
		tc := NewTensorCommitment(params)

		// random polynomial
		p := make([]fr.Element, nbRows*nbColumns)
		for i := 0; i < nbRows*nbColumns; i++ {
			p[i].SetRandom()
		}

		// run the benchmark
		b.Run("size poly"+strconv.Itoa(nbRows*nbColumns), func(b *testing.B) {
			b.ResetTimer()
			for i := 0; i < b.N; i++ {
				tc.Append(p)
				tc.Commit()
			}
		})

	}

}
