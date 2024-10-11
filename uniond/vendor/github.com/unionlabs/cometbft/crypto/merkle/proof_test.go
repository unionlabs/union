package merkle

import (
	"bytes"
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	cmtcrypto "github.com/cometbft/cometbft/api/cometbft/crypto/v1"
	"github.com/cometbft/cometbft/crypto/tmhash"
)

const ProofOpDomino = "test:domino"

// Expects given input, produces given output.
// Like the game dominos.
type DominoOp struct {
	key    string // unexported, may be empty
	Input  string
	Output string
}

func NewDominoOp(key, input, output string) DominoOp {
	return DominoOp{
		key:    key,
		Input:  input,
		Output: output,
	}
}

func (dop DominoOp) ProofOp() cmtcrypto.ProofOp {
	dopb := cmtcrypto.DominoOp{
		Key:    dop.key,
		Input:  dop.Input,
		Output: dop.Output,
	}
	bz, err := dopb.Marshal()
	if err != nil {
		panic(err)
	}

	return cmtcrypto.ProofOp{
		Type: ProofOpDomino,
		Key:  []byte(dop.key),
		Data: bz,
	}
}

func (dop DominoOp) Run(input [][]byte) (output [][]byte, err error) {
	if len(input) != 1 {
		return nil, errors.New("expected input of length 1")
	}
	if string(input[0]) != dop.Input {
		return nil, fmt.Errorf("expected input %v, got %v",
			dop.Input, string(input[0]))
	}
	return [][]byte{[]byte(dop.Output)}, nil
}

func (dop DominoOp) GetKey() []byte {
	return []byte(dop.key)
}

// ----------------------------------------

func TestProofOperators(t *testing.T) {
	var err error

	// ProofRuntime setup
	// TODO test this somehow.

	// ProofOperators setup
	op1 := NewDominoOp("KEY1", "INPUT1", "INPUT2")
	op2 := NewDominoOp("KEY2", "INPUT2", "INPUT3")
	op3 := NewDominoOp("", "INPUT3", "INPUT4")
	op4 := NewDominoOp("KEY4", "INPUT4", "OUTPUT4")

	// Good
	popz := ProofOperators([]ProofOperator{op1, op2, op3, op4})
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.NoError(t, err)
	err = popz.VerifyValue(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", bz("INPUT1"))
	require.NoError(t, err)

	// BAD INPUT
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1_WRONG")})
	require.Error(t, err)
	err = popz.VerifyValue(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", bz("INPUT1_WRONG"))
	require.Error(t, err)

	// BAD KEY 1
	err = popz.Verify(bz("OUTPUT4"), "/KEY3/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD KEY 2
	err = popz.Verify(bz("OUTPUT4"), "KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD KEY 3
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1/", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD KEY 4
	err = popz.Verify(bz("OUTPUT4"), "//KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD KEY 5
	err = popz.Verify(bz("OUTPUT4"), "/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD OUTPUT 1
	err = popz.Verify(bz("OUTPUT4_WRONG"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD OUTPUT 2
	err = popz.Verify(bz(""), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD POPZ 1
	popz = []ProofOperator{op1, op2, op4}
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD POPZ 2
	popz = []ProofOperator{op4, op3, op2, op1}
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)

	// BAD POPZ 3
	popz = []ProofOperator{}
	err = popz.Verify(bz("OUTPUT4"), "/KEY4/KEY2/KEY1", [][]byte{bz("INPUT1")})
	require.Error(t, err)
}

func bz(s string) []byte {
	return []byte(s)
}

func TestProofValidateBasic(t *testing.T) {
	testCases := []struct {
		testName      string
		malleateProof func(*Proof)
		errStr        string
	}{
		{"Good", func(_ *Proof) {}, ""},
		{"Negative Total", func(sp *Proof) { sp.Total = -1 }, "negative proof total"},
		{"Negative Index", func(sp *Proof) { sp.Index = -1 }, "negative proof index"},
		{
			"Invalid LeafHash", func(sp *Proof) { sp.LeafHash = make([]byte, 10) },
			"leaf length 10, want 32",
		},
		{
			"Too many Aunts", func(sp *Proof) { sp.Aunts = make([][]byte, MaxAunts+1) },
			"maximum aunts length, 100, exceeded",
		},
		{
			"Invalid Aunt", func(sp *Proof) { sp.Aunts[0] = make([]byte, 10) },
			"aunt#0 hash length 10, want 32",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.testName, func(t *testing.T) {
			_, proofs := ProofsFromByteSlices([][]byte{
				[]byte("apple"),
				[]byte("watermelon"),
				[]byte("kiwi"),
			})
			tc.malleateProof(proofs[0])
			err := proofs[0].ValidateBasic()
			if tc.errStr != "" {
				assert.Contains(t, err.Error(), tc.errStr)
			}
		})
	}
}

func TestVoteProtobuf(t *testing.T) {
	_, proofs := ProofsFromByteSlices([][]byte{
		[]byte("apple"),
		[]byte("watermelon"),
		[]byte("kiwi"),
	})

	testCases := []struct {
		testName string
		v1       *Proof
		expPass  bool
	}{
		{"empty proof", &Proof{}, false},
		{"failure nil", nil, false},
		{"success", proofs[0], true},
	}
	for _, tc := range testCases {
		pb := tc.v1.ToProto()

		v, err := ProofFromProto(pb)
		if tc.expPass {
			require.NoError(t, err)
			require.Equal(t, tc.v1, v, tc.testName)
		} else {
			require.Error(t, err)
		}
	}
}

// TestVsa2022_100 verifies https://blog.verichains.io/p/vsa-2022-100-tendermint-forging-membership-proof
func TestVsa2022_100(t *testing.T) {
	// a fake key-value pair and its hash
	key := []byte{0x13}
	value := []byte{0x37}
	vhash := tmhash.Sum(value)
	bz := new(bytes.Buffer)
	_ = encodeByteSlice(bz, key)
	_ = encodeByteSlice(bz, vhash)
	kvhash := tmhash.Sum(append([]byte{0}, bz.Bytes()...))

	// the malicious `op`
	op := NewValueOp(
		key,
		&Proof{LeafHash: kvhash},
	)

	// the nil root
	var root []byte

	require.Error(t, ProofOperators{op}.Verify(root, "/"+string(key), [][]byte{value}))
}
