package proofs

import (
	"errors"
	"sort"

	ics23 "github.com/cosmos/ics23/go"

	sdkmaps "cosmossdk.io/store/internal/maps"
)

var (
	ErrEmptyKey       = errors.New("key is empty")
	ErrEmptyKeyInData = errors.New("data contains empty key")
)

/*
CreateMembershipProof will produce a CommitmentProof that the given key (and queries value) exists in the map.
If the key doesn't exist in the tree, this will return an error.
*/
func CreateMembershipProof(data map[string][]byte, key []byte) (*ics23.CommitmentProof, error) {
	if len(key) == 0 {
		return nil, ErrEmptyKey
	}
	exist, err := createExistenceProof(data, key)
	if err != nil {
		return nil, err
	}
	proof := &ics23.CommitmentProof{
		Proof: &ics23.CommitmentProof_Exist{
			Exist: exist,
		},
	}
	return proof, nil
}

/*
CreateNonMembershipProof will produce a CommitmentProof that the given key doesn't exist in the map.
If the key exists in the tree, this will return an error.
*/
func CreateNonMembershipProof(data map[string][]byte, key []byte) (*ics23.CommitmentProof, error) {
	if len(key) == 0 {
		return nil, ErrEmptyKey
	}
	// ensure this key is not in the store
	if _, ok := data[string(key)]; ok {
		return nil, errors.New("cannot create non-membership proof if key is in map")
	}

	keys := SortedKeys(data)
	rightidx := sort.SearchStrings(keys, string(key))

	var err error
	nonexist := &ics23.NonExistenceProof{
		Key: key,
	}

	// include left proof unless key is left of entire map
	if rightidx >= 1 {
		leftkey := keys[rightidx-1]
		nonexist.Left, err = createExistenceProof(data, []byte(leftkey))
		if err != nil {
			return nil, err
		}
	}

	// include right proof unless key is right of entire map
	if rightidx < len(keys) {
		rightkey := keys[rightidx]
		nonexist.Right, err = createExistenceProof(data, []byte(rightkey))
		if err != nil {
			return nil, err
		}

	}

	proof := &ics23.CommitmentProof{
		Proof: &ics23.CommitmentProof_Nonexist{
			Nonexist: nonexist,
		},
	}
	return proof, nil
}

func createExistenceProof(data map[string][]byte, key []byte) (*ics23.ExistenceProof, error) {
	for k := range data {
		if k == "" {
			return nil, ErrEmptyKeyInData
		}
	}
	value, ok := data[string(key)]
	if !ok {
		return nil, errors.New("cannot make existence proof if key is not in map")
	}

	_, proofs, _ := sdkmaps.ProofsFromMap(data)
	proof := proofs[string(key)]
	if proof == nil {
		return nil, errors.New("returned no proof for key")
	}

	return ConvertExistenceProof(proof, key, value)
}
