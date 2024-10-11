package proofs

import (
	"errors"
	"testing"

	ics23 "github.com/cosmos/ics23/go"
	"github.com/stretchr/testify/assert"
)

func TestCreateMembership(t *testing.T) {
	cases := map[string]struct {
		size int
		loc  Where
	}{
		"small left":   {size: 100, loc: Left},
		"small middle": {size: 100, loc: Middle},
		"small right":  {size: 100, loc: Right},
		"big left":     {size: 5431, loc: Left},
		"big middle":   {size: 5431, loc: Middle},
		"big right":    {size: 5431, loc: Right},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			data := BuildMap(tc.size)
			allkeys := SortedKeys(data)
			key := GetKey(allkeys, tc.loc)
			nonKey := GetNonKey(allkeys, tc.loc)

			// error if the key does not exist
			proof, err := CreateMembershipProof(data, []byte(nonKey))
			assert.EqualError(t, err, "cannot make existence proof if key is not in map")
			assert.Nil(t, proof)

			val := data[key]
			proof, err = CreateMembershipProof(data, []byte(key))
			if err != nil {
				t.Fatalf("Creating Proof: %+v", err)
			}
			if proof.GetExist() == nil {
				t.Fatal("Unexpected proof format")
			}

			root := CalcRoot(data)
			err = proof.GetExist().Verify(ics23.TendermintSpec, root, []byte(key), val)
			if err != nil {
				t.Fatalf("Verifying Proof: %+v", err)
			}

			valid := ics23.VerifyMembership(ics23.TendermintSpec, root, proof, []byte(key), val)
			if !valid {
				t.Fatalf("Membership Proof Invalid")
			}
		})
	}
}

func TestCreateNonMembership(t *testing.T) {
	cases := map[string]struct {
		size int
		loc  Where
	}{
		"small left":   {size: 100, loc: Left},
		"small middle": {size: 100, loc: Middle},
		"small right":  {size: 100, loc: Right},
		"big left":     {size: 5431, loc: Left},
		"big middle":   {size: 5431, loc: Middle},
		"big right":    {size: 5431, loc: Right},
	}

	for name, tc := range cases {
		t.Run(name, func(t *testing.T) {
			data := BuildMap(tc.size)
			allkeys := SortedKeys(data)
			nonKey := GetNonKey(allkeys, tc.loc)
			key := GetKey(allkeys, tc.loc)

			// error if the key exists
			proof, err := CreateNonMembershipProof(data, []byte(key))
			assert.EqualError(t, err, "cannot create non-membership proof if key is in map")
			assert.Nil(t, proof)

			proof, err = CreateNonMembershipProof(data, []byte(nonKey))
			if err != nil {
				t.Fatalf("Creating Proof: %+v", err)
			}
			if proof.GetNonexist() == nil {
				t.Fatal("Unexpected proof format")
			}

			root := CalcRoot(data)
			err = proof.GetNonexist().Verify(ics23.TendermintSpec, root, []byte(nonKey))
			if err != nil {
				t.Fatalf("Verifying Proof: %+v", err)
			}

			valid := ics23.VerifyNonMembership(ics23.TendermintSpec, root, proof, []byte(nonKey))
			if !valid {
				t.Fatalf("Non Membership Proof Invalid")
			}
		})
	}
}

func TestInvalidKey(t *testing.T) {
	tests := []struct {
		name string
		f    func(data map[string][]byte, key []byte) (*ics23.CommitmentProof, error)
		data map[string][]byte
		key  []byte
		err  error
	}{
		{"CreateMembershipProof empty key", CreateMembershipProof, map[string][]byte{"": nil}, []byte(""), ErrEmptyKey},
		{"CreateMembershipProof empty key in data", CreateMembershipProof, map[string][]byte{"": nil, " ": nil}, []byte(" "), ErrEmptyKeyInData},
		{"CreateNonMembershipProof empty key", CreateNonMembershipProof, map[string][]byte{" ": nil}, []byte(""), ErrEmptyKey},
		{"CreateNonMembershipProof empty key in data", CreateNonMembershipProof, map[string][]byte{"": nil}, []byte(" "), ErrEmptyKeyInData},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			_, err := tc.f(tc.data, tc.key)
			assert.True(t, errors.Is(err, tc.err))
		})
	}
}
