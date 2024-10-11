package ics23

import (
	"bytes"
	"encoding/hex"
	"testing"
)

func TestLeafOp(t *testing.T) {
	cases := LeafOpTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			res, err := tc.Op.Apply(tc.Key, tc.Value)
			// short-circuit with error case
			if tc.IsErr && err == nil {
				t.Fatal("expected error, but got none")
			}
			if !tc.IsErr && err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(res, tc.Expected) {
				t.Errorf("bad result: %s vs %s", toHex(res), toHex(tc.Expected))
			}
		})
	}
}

func TestInnerOp(t *testing.T) {
	cases := InnerOpTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			res, err := tc.Op.Apply(tc.Child)
			if tc.IsErr && err == nil {
				t.Fatal("Expected error, but got none")
			}
			if !tc.IsErr && err != nil {
				t.Fatal(err)
			}
			if !bytes.Equal(res, tc.Expected) {
				t.Errorf("Bad result: %s vs %s", toHex(res), toHex(tc.Expected))
			}
		})
	}
}

func TestDoHash(t *testing.T) {
	cases := DoHashTestData(t)

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			res, err := doHash(tc.HashOp, []byte(tc.Preimage))
			if err != nil {
				t.Fatal(err)
			}
			hexRes := hex.EncodeToString(res)
			if hexRes != tc.ExpectedHash {
				t.Fatalf("Expected %s got %s", tc.ExpectedHash, hexRes)
			}
		})
	}
}
