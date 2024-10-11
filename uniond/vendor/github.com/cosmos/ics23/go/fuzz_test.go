package ics23

import (
	"bytes"
	"encoding/json"
	"os"
	"path/filepath"
	"testing"
)

func FuzzExistenceProofCalculate(f *testing.F) {
	if testing.Short() {
		f.Skip("in -short mode")
	}

	seedJSON, err := os.ReadFile(filepath.Join("..", "testdata", "TestExistenceProofData.json"))
	if err != nil {
		f.Fatal(err)
	}

	// 2. Isolate the individual JSON per case which eases with fuzz case mutation.
	existenceWholeSeedJSON := make(map[string]json.RawMessage)
	if err := json.Unmarshal(seedJSON, &existenceWholeSeedJSON); err != nil {
		f.Fatal(err)
	}

	// 3. Add the seeds:
	for _, epJSON := range existenceWholeSeedJSON {
		f.Add([]byte(epJSON))
	}

	// 4. Now run the fuzzer.
	f.Fuzz(func(t *testing.T, fJSON []byte) {
		ep := new(ExistenceProof)
		if err := json.Unmarshal(fJSON, ep); err != nil {
			return
		}

		// Now let's try this seemingly well formed ExistenceProof.
		_, _ = ep.Calculate()
	})
}

func FuzzExistenceProofCheckAgainstSpec(f *testing.F) {
	if testing.Short() {
		f.Skip("in -short mode")
	}

	seedDataMap := CheckAgainstSpecTestData(f)
	for _, seed := range seedDataMap {
		if seed.Err != "" {
			// Erroneous data, skip it.
			continue
		}
		if blob, err := json.Marshal(seed); err == nil {
			f.Add(blob)
		}
	}

	// 2. Now run the fuzzer.
	f.Fuzz(func(t *testing.T, fJSON []byte) {
		pvs := new(CheckAgainstSpecTestStruct)
		if err := json.Unmarshal(fJSON, pvs); err != nil {
			return
		}
		if pvs.Proof == nil {
			// No need checking from a nil ExistenceProof.
			return
		}

		ep, spec := pvs.Proof, pvs.Spec
		_ = ep.CheckAgainstSpec(spec)
	})
}

var batchVectorDataSeeds []*BatchVectorData

func init() {
	svtdL := VectorsTestData()
	bsL := make([]*BatchVectorData, 0, len(svtdL))
	for _, tc := range svtdL {
		proof, ref := LoadFile(new(testing.T), tc.Dir, tc.Filename)
		// Test Calculate method and skip if it produces invalid values.
		if _, err := proof.Calculate(); err != nil {
			continue
		}

		bsL = append(bsL, &BatchVectorData{
			Spec:  tc.Spec,
			Ref:   ref,
			Proof: proof,
		})
	}
	batchVectorDataSeeds = bsL
}

func FuzzVerifyNonMembership(f *testing.F) {
	if testing.Short() {
		f.Skip("in -short mode")
	}

	// 1. Add some seeds.
	for _, batchVec := range batchVectorDataSeeds {
		blob, err := json.Marshal(batchVec)
		if err != nil {
			f.Fatal(err)
		}
		f.Add(blob)
	}

	// 2. Now run the fuzzer.
	f.Fuzz(func(t *testing.T, inputJSON []byte) {
		var bv BatchVectorData
		if err := json.Unmarshal(inputJSON, &bv); err != nil {
			return
		}
		if bv.Ref == nil || bv.Proof == nil || bv.Ref.RootHash == nil {
			return
		}
		// Otherwise now run VerifyNonMembership.
		_ = VerifyNonMembership(bv.Spec, bv.Ref.RootHash, bv.Proof, bv.Ref.Key)
	})
}

// verifyJSON is necessary so we already have sources of Proof, Ref and Spec
// that'll be mutated by the fuzzer to get much closer to values for greater coverage.
type verifyJSON struct {
	Proof *CommitmentProof
	Ref   *RefData
	Spec  *ProofSpec
}

func FuzzVerifyMembership(f *testing.F) {
	seeds := VectorsTestData()

	// VerifyMembership requires:
	//  Spec, ExistenceProof, CommitmentRootref.
	for _, seed := range seeds {
		proof, ref := LoadFile(f, seed.Dir, seed.Filename)
		root, err := proof.Calculate()
		if err != nil {
			continue
		}
		if !bytes.Equal(ref.RootHash, root) {
			continue
		}

		if ref.Value == nil {
			continue
		}

		// Now serialize this value as a seed.
		// The use of already calculated values is necessary
		// for the fuzzers to have a basis of much better coverage
		// generating values.
		blob, err := json.Marshal(&verifyJSON{Proof: proof, Ref: ref, Spec: seed.Spec})
		if err == nil {
			f.Add(blob)
		}
	}

	// 2. Now let's run the fuzzer.
	f.Fuzz(func(t *testing.T, input []byte) {
		var con verifyJSON
		if err := json.Unmarshal(input, &con); err != nil {
			return
		}
		spec, ref, proof := con.Spec, con.Ref, con.Proof
		if ref == nil {
			return
		}
		_ = VerifyMembership(spec, ref.RootHash, proof, ref.Key, ref.Value)
	})
}

func FuzzCombineProofs(f *testing.F) {
	// 1. Load in the CommitmentProofs
	baseDirs := []string{"iavl", "tendermint", "smt"}
	filenames := []string{
		"exist_left.json",
		"exist_right.json",
		"exist_middle.json",
		"nonexist_left.json",
		"nonexist_right.json",
		"nonexist_middle.json",
	}

	for _, baseDir := range baseDirs {
		dir := filepath.Join("..", "testdata", baseDir)
		for _, filename := range filenames {
			proofs, _ := LoadFile(new(testing.T), dir, filename)
			blob, err := json.Marshal(proofs)
			if err != nil {
				f.Fatal(err)
			}
			f.Add(blob)
		}
	}

	// 2. Now let's run the fuzzer.
	f.Fuzz(func(t *testing.T, proofsJSON []byte) {
		var proofs []*CommitmentProof
		if err := json.Unmarshal(proofsJSON, &proofs); err != nil {
			return
		}
		_, _ = CombineProofs(proofs)
	})
}
