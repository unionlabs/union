package main

import (
	"encoding/json"
	"fmt"
	fiatshamir "github.com/consensys/gnark-crypto/fiat-shamir"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational/polynomial"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational/sumcheck"
	"github.com/consensys/gnark-crypto/internal/generator/test_vector_utils/small_rational/test_vector_utils"
	"hash"
	"math/bits"
	"os"
	"path/filepath"
)

func runMultilin(testCaseInfo *TestCaseInfo) error {

	var poly polynomial.MultiLin
	if v, err := test_vector_utils.SliceToElementSlice(testCaseInfo.Values); err == nil {
		poly = v
	} else {
		return err
	}

	var hsh hash.Hash
	var err error
	if hsh, err = test_vector_utils.HashFromDescription(testCaseInfo.Hash); err != nil {
		return err
	}

	proof, err := sumcheck.Prove(
		&singleMultilinClaim{poly}, fiatshamir.WithHash(hsh))
	if err != nil {
		return err
	}
	testCaseInfo.Proof = toPrintableProof(proof)

	// Verification
	if v, _err := test_vector_utils.SliceToElementSlice(testCaseInfo.Values); _err == nil {
		poly = v
	} else {
		return _err
	}
	var claimedSum small_rational.SmallRational
	if _, err = claimedSum.SetInterface(testCaseInfo.ClaimedSum); err != nil {
		return err
	}

	if err = sumcheck.Verify(singleMultilinLazyClaim{g: poly, claimedSum: claimedSum}, proof, fiatshamir.WithHash(hsh)); err != nil {
		return fmt.Errorf("proof rejected: %v", err)
	}

	proof.PartialSumPolys[0][0].Add(&proof.PartialSumPolys[0][0], test_vector_utils.ToElement(1))
	if err = sumcheck.Verify(singleMultilinLazyClaim{g: poly, claimedSum: claimedSum}, proof, fiatshamir.WithHash(hsh)); err == nil {
		return fmt.Errorf("bad proof accepted")
	}
	return nil
}

func run(testCaseInfo *TestCaseInfo) error {
	switch testCaseInfo.Type {
	case "multilin":
		return runMultilin(testCaseInfo)
	default:
		return fmt.Errorf("type \"%s\" unrecognized", testCaseInfo.Type)
	}
}

func runAll(relPath string) error {
	var filename string
	var err error
	if filename, err = filepath.Abs(relPath); err != nil {
		return err
	}

	var bytes []byte

	if bytes, err = os.ReadFile(filename); err != nil {
		return err
	}

	var testCasesInfo TestCasesInfo
	if err = json.Unmarshal(bytes, &testCasesInfo); err != nil {
		return err
	}

	failed := false
	for name, testCase := range testCasesInfo {
		if err = run(testCase); err != nil {
			fmt.Println(name, ":", err)
			failed = true
		}
	}

	if failed {
		return fmt.Errorf("test case failed")
	}

	if bytes, err = json.MarshalIndent(testCasesInfo, "", "\t"); err != nil {
		return err
	}

	return os.WriteFile(filename, bytes, 0)
}

func main() {
	if err := runAll("sumcheck/test_vectors/vectors.json"); err != nil {
		fmt.Println(err)
		os.Exit(-1)
	}
}

type TestCasesInfo map[string]*TestCaseInfo

type TestCaseInfo struct {
	Type        string                            `json:"type"`
	Hash        test_vector_utils.HashDescription `json:"hash"`
	Values      []interface{}                     `json:"values"`
	Description string                            `json:"description"`
	Proof       PrintableProof                    `json:"proof"`
	ClaimedSum  interface{}                       `json:"claimedSum"`
}

type PrintableProof struct {
	PartialSumPolys [][]interface{} `json:"partialSumPolys"`
	FinalEvalProof  interface{}     `json:"finalEvalProof"`
}

func toPrintableProof(proof sumcheck.Proof) (printable PrintableProof) {
	if proof.FinalEvalProof != nil {
		panic("null expected")
	}
	printable.FinalEvalProof = struct{}{}
	printable.PartialSumPolys = test_vector_utils.ElementSliceSliceToInterfaceSliceSlice(proof.PartialSumPolys)
	return
}

type singleMultilinClaim struct {
	g polynomial.MultiLin
}

func (c singleMultilinClaim) ProveFinalEval([]small_rational.SmallRational) interface{} {
	return nil // verifier can compute the final eval itself
}

func (c singleMultilinClaim) VarsNum() int {
	return bits.TrailingZeros(uint(len(c.g)))
}

func (c singleMultilinClaim) ClaimsNum() int {
	return 1
}

func sumForX1One(g polynomial.MultiLin) polynomial.Polynomial {
	sum := g[len(g)/2]
	for i := len(g)/2 + 1; i < len(g); i++ {
		sum.Add(&sum, &g[i])
	}
	return []small_rational.SmallRational{sum}
}

func (c singleMultilinClaim) Combine(small_rational.SmallRational) polynomial.Polynomial {
	return sumForX1One(c.g)
}

func (c *singleMultilinClaim) Next(r small_rational.SmallRational) polynomial.Polynomial {
	c.g.Fold(r)
	return sumForX1One(c.g)
}

type singleMultilinLazyClaim struct {
	g          polynomial.MultiLin
	claimedSum small_rational.SmallRational
}

func (c singleMultilinLazyClaim) VerifyFinalEval(r []small_rational.SmallRational, _ small_rational.SmallRational, purportedValue small_rational.SmallRational, _ interface{}) error {
	val := c.g.Evaluate(r, nil)
	if val.Equal(&purportedValue) {
		return nil
	}
	return fmt.Errorf("mismatch")
}

func (c singleMultilinLazyClaim) CombinedSum(small_rational.SmallRational) small_rational.SmallRational {
	return c.claimedSum
}

func (c singleMultilinLazyClaim) Degree(int) int {
	return 1
}

func (c singleMultilinLazyClaim) ClaimsNum() int {
	return 1
}

func (c singleMultilinLazyClaim) VarsNum() int {
	return bits.TrailingZeros(uint(len(c.g)))
}
