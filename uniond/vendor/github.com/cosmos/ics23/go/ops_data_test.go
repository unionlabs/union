package ics23

import (
	"encoding/hex"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

type LeafOpTestStruct struct {
	Op       *LeafOp
	Key      []byte
	Value    []byte
	IsErr    bool
	Expected []byte
}

func LeafOpTestData(t *testing.T) map[string]LeafOpTestStruct {
	t.Helper()
	fname := filepath.Join("..", "testdata", "TestLeafOpData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		t.Fatal(err)
	}
	var cases map[string]LeafOpTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		t.Fatal(err)
	}
	return cases
}

type InnerOpTestStruct struct {
	Op       *InnerOp
	Child    []byte
	IsErr    bool
	Expected []byte
}

func InnerOpTestData(t *testing.T) map[string]InnerOpTestStruct {
	t.Helper()
	fname := filepath.Join("..", "testdata", "TestInnerOpData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		t.Fatal(err)
	}
	var cases map[string]InnerOpTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		t.Fatal(err)
	}
	return cases
}

type DoHashTestStruct struct {
	HashOp       HashOp
	Preimage     string
	ExpectedHash string
}

func DoHashTestData(t *testing.T) map[string]DoHashTestStruct {
	t.Helper()
	fname := filepath.Join("..", "testdata", "TestDoHashData.json")
	ffile, err := os.Open(fname)
	if err != nil {
		t.Fatal(err)
	}
	var cases map[string]DoHashTestStruct
	jsonDecoder := json.NewDecoder(ffile)
	err = jsonDecoder.Decode(&cases)
	if err != nil {
		t.Fatal(err)
	}
	return cases
}

func toHex(data []byte) string {
	return hex.EncodeToString(data)
}
