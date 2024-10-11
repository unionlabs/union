package proofs

import (
	"bytes"
	"fmt"
	"testing"
)

func TestLeafOp(t *testing.T) {
	proof := GenerateRangeProof(20, Middle)

	converted, err := ConvertExistenceProof(proof.Proof, proof.Key, proof.Value)
	if err != nil {
		t.Fatal(err)
	}

	leaf := converted.GetLeaf()
	if leaf == nil {
		t.Fatalf("Missing leaf node")
	}

	hash, err := leaf.Apply(converted.Key, converted.Value)
	if err != nil {
		t.Fatal(err)
	}

	if !bytes.Equal(hash, proof.Proof.LeafHash) {
		t.Errorf("Calculated: %X\nExpected:   %X", hash, proof.Proof.LeafHash)
	}
}

func TestBuildPath(t *testing.T) {
	cases := map[string]struct {
		idx      int64
		total    int64
		expected []bool
	}{
		"pair left": {
			idx:      0,
			total:    2,
			expected: []bool{true},
		},
		"pair right": {
			idx:      1,
			total:    2,
			expected: []bool{false},
		},
		"power of 2": {
			idx:      3,
			total:    8,
			expected: []bool{false, false, true},
		},
		"size of 7 right most": {
			idx:      6,
			total:    7,
			expected: []bool{false, false},
		},
		"size of 6 right-left (from top)": {
			idx:      4,
			total:    6,
			expected: []bool{true, false},
		},
		"size of 6 left-right-left (from top)": {
			idx:      2,
			total:    7,
			expected: []bool{true, false, true},
		},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			path := buildPath(tc.idx, tc.total)
			if len(path) != len(tc.expected) {
				t.Fatalf("Got %v\nExpected %v", path, tc.expected)
			}
			for i := range path {
				if path[i] != tc.expected[i] {
					t.Fatalf("Differ at %d\nGot %v\nExpected %v", i, path, tc.expected)
				}
			}
		})
	}
}

func TestConvertProof(t *testing.T) {
	for i := 0; i < 100; i++ {
		t.Run(fmt.Sprintf("Run %d", i), func(t *testing.T) {
			proof := GenerateRangeProof(57, Left)

			converted, err := ConvertExistenceProof(proof.Proof, proof.Key, proof.Value)
			if err != nil {
				t.Fatal(err)
			}

			calc, err := converted.Calculate()
			if err != nil {
				t.Fatal(err)
			}

			if !bytes.Equal(calc, proof.RootHash) {
				t.Errorf("Calculated: %X\nExpected:   %X", calc, proof.RootHash)
			}
		})
	}
}
