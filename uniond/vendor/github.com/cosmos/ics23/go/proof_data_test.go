package ics23

import (
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

type ExistenceProofTestStruct struct {
	Proof    *ExistenceProof
	IsErr    bool
	Expected []byte
}

func ExistenceProofTestData(tb testing.TB) map[string]ExistenceProofTestStruct {
	tb.Helper()
	fname := filepath.Join("..", "testdata", "TestExistenceProofData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		tb.Fatal(err)
	}
	var cases map[string]ExistenceProofTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		tb.Fatal(err)
	}
	return cases
}

type CheckLeafTestStruct struct {
	Leaf  *LeafOp
	Spec  *LeafOp
	IsErr bool
}

func CheckLeafTestData(tb testing.TB) map[string]CheckLeafTestStruct {
	tb.Helper()
	fname := filepath.Join("..", "testdata", "TestCheckLeafData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		tb.Fatal(err)
	}
	var cases map[string]CheckLeafTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		tb.Fatal(err)
	}
	return cases
}

type CheckAgainstSpecTestStruct struct {
	Proof *ExistenceProof
	Spec  *ProofSpec
	Err   string
}

func CheckAgainstSpecTestData(tb testing.TB) map[string]CheckAgainstSpecTestStruct {
	tb.Helper()
	fname := filepath.Join("..", "testdata", "TestCheckAgainstSpecData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		tb.Fatal(err)
	}
	var cases map[string]CheckAgainstSpecTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		tb.Fatal(err)
	}

	return cases
}

type EmptyBranchTestStruct struct {
	Op      *InnerOp
	Spec    *ProofSpec
	IsLeft  bool
	IsRight bool
}

func EmptyBranchTestData(tb testing.TB) []EmptyBranchTestStruct {
	tb.Helper()
	fname := filepath.Join("..", "testdata", "TestEmptyBranchData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		tb.Fatal(err)
	}
	var cases []EmptyBranchTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		tb.Fatal(err)
	}
	return cases
}
