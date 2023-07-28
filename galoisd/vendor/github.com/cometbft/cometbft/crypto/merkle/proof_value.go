package merkle

import (
	"bytes"
	"fmt"

	"github.com/cometbft/cometbft/crypto/tmhash"
	cmtcrypto "github.com/cometbft/cometbft/proto/tendermint/crypto"
)

const ProofOpValue = "simple:v"

// ValueOp takes a key and a single value as argument and
// produces the root hash.  The corresponding tree structure is
// the SimpleMap tree.  SimpleMap takes a Hasher, and currently
// CometBFT uses tmhash.  SimpleValueOp should support
// the hash function as used in tmhash.  TODO support
// additional hash functions here as options/args to this
// operator.
//
// If the produced root hash matches the expected hash, the
// proof is good.
type ValueOp struct {
	// Encoded in ProofOp.Key.
	key []byte

	// To encode in ProofOp.Data
	Proof *Proof `json:"proof"`
}

var _ ProofOperator = ValueOp{}

func NewValueOp(key []byte, proof *Proof) ValueOp {
	return ValueOp{
		key:   key,
		Proof: proof,
	}
}

func ValueOpDecoder(pop cmtcrypto.ProofOp) (ProofOperator, error) {
	if pop.Type != ProofOpValue {
		return nil, fmt.Errorf("unexpected ProofOp.Type; got %v, want %v", pop.Type, ProofOpValue)
	}
	var pbop cmtcrypto.ValueOp // a bit strange as we'll discard this, but it works.
	err := pbop.Unmarshal(pop.Data)
	if err != nil {
		return nil, fmt.Errorf("decoding ProofOp.Data into ValueOp: %w", err)
	}

	sp, err := ProofFromProto(pbop.Proof)
	if err != nil {
		return nil, err
	}
	return NewValueOp(pop.Key, sp), nil
}

func (op ValueOp) ProofOp() cmtcrypto.ProofOp {
	pbval := cmtcrypto.ValueOp{
		Key:   op.key,
		Proof: op.Proof.ToProto(),
	}
	bz, err := pbval.Marshal()
	if err != nil {
		panic(err)
	}
	return cmtcrypto.ProofOp{
		Type: ProofOpValue,
		Key:  op.key,
		Data: bz,
	}
}

func (op ValueOp) String() string {
	return fmt.Sprintf("ValueOp{%v}", op.GetKey())
}

func (op ValueOp) Run(args [][]byte) ([][]byte, error) {
	if len(args) != 1 {
		return nil, fmt.Errorf("expected 1 arg, got %v", len(args))
	}
	value := args[0]
	hasher := tmhash.New()
	hasher.Write(value)
	vhash := hasher.Sum(nil)

	bz := new(bytes.Buffer)
	// Wrap <op.Key, vhash> to hash the KVPair.
	encodeByteSlice(bz, op.key) //nolint: errcheck // does not error
	encodeByteSlice(bz, vhash)  //nolint: errcheck // does not error
	kvhash := leafHash(bz.Bytes())

	if !bytes.Equal(kvhash, op.Proof.LeafHash) {
		return nil, fmt.Errorf("leaf hash mismatch: want %X got %X", op.Proof.LeafHash, kvhash)
	}

	rootHash, err := op.Proof.computeRootHash()
	if err != nil {
		return nil, err
	}
	return [][]byte{
		rootHash,
	}, nil
}

func (op ValueOp) GetKey() []byte {
	return op.key
}
