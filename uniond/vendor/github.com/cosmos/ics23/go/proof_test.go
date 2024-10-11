package ics23

import (
	"bytes"
	"testing"
)

func TestExistenceProof(t *testing.T) {
	cases := ExistenceProofTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			res, err := tc.Proof.Calculate()
			// short-circuit with error case
			if tc.IsErr {
				if err == nil {
					t.Fatal("Expected error, but got none")
				}
			} else {
				if err != nil {
					t.Fatal(err)
				}
			}
			if !bytes.Equal(res, tc.Expected) {
				t.Errorf("Bad result: %s vs %s", toHex(res), toHex(tc.Expected))
			}
		})
	}
}

func TestCheckLeaf(t *testing.T) {
	cases := CheckLeafTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.Leaf.CheckAgainstSpec(&ProofSpec{LeafSpec: tc.Spec})
			if tc.IsErr && err == nil {
				t.Fatal("Expected error, but got nil")
			} else if !tc.IsErr && err != nil {
				t.Fatalf("Unexpected error: %v", err)
			}
		})
	}
}

func TestCheckAgainstSpec(t *testing.T) {
	cases := CheckAgainstSpecTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			err := tc.Proof.CheckAgainstSpec(tc.Spec)
			if tc.Err == "" && err != nil {
				t.Fatalf("Unexpected error: %v", err)
			} else if tc.Err != "" && tc.Err != err.Error() {
				t.Fatalf("Expected error: %s, got %s", tc.Err, err.Error())
			}
		})
	}
}

func TestEmptyBranch(t *testing.T) {
	cases := EmptyBranchTestData(t)

	for _, tc := range cases {
		t.Run("case", func(t *testing.T) {
			if err := tc.Op.CheckAgainstSpec(tc.Spec, 1); err != nil {
				t.Errorf("Invalid InnerOp: %v", err)
			}
			if leftBranchesAreEmpty(tc.Spec.InnerSpec, tc.Op) != tc.IsLeft {
				t.Errorf("Expected leftBranchesAreEmpty to be %t but it wasn't", tc.IsLeft)
			}
			if rightBranchesAreEmpty(tc.Spec.InnerSpec, tc.Op) != tc.IsRight {
				t.Errorf("Expected rightBranchesAreEmpty to be %t but it wasn't", tc.IsRight)
			}
		})
	}
}
