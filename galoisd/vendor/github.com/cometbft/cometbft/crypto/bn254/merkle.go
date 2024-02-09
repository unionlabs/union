package bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	"math/big"
)

type MerkleLeaf struct {
	VotingPower int64
	ShiftedX    fr.Element
	ShiftedY    fr.Element
	MsbX        uint8
	MsbY        uint8
}

// Union whitepaper: construct arguments of (11) H_pre
//
func NewMerkleLeaf(pubKey bn254.G1Affine, votingPower int64) (MerkleLeaf, error) {
	x := pubKey.X.BigInt(new(big.Int))
	y := pubKey.Y.BigInt(new(big.Int))
	msbX := x.Bit(253)
	msbY := y.Bit(253)
	var frX, frY fr.Element
	x.SetBit(x, 253, 0)
	var padded [32]byte
	x.FillBytes(padded[:])
	err := frX.SetBytesCanonical(padded[:])
	if err != nil {
		return MerkleLeaf{}, err
	}
	y.SetBit(y, 253, 0)
	y.FillBytes(padded[:])
	err = frY.SetBytesCanonical(padded[:])
	if err != nil {
		return MerkleLeaf{}, err
	}
	return MerkleLeaf{
		VotingPower: votingPower,
		ShiftedX:    frX,
		ShiftedY:    frY,
		MsbX:        uint8(msbX),
		MsbY:        uint8(msbY),
	}, nil
}

// Union whitepaper: (11) H_pre
//
func (l MerkleLeaf) Hash() ([]byte, error) {
	frXBytes := l.ShiftedX.Bytes()
	frYBytes := l.ShiftedY.Bytes()
	mimc := mimc.NewMiMC()
	_, err := mimc.Write(frXBytes[:])
	if err != nil {
		return nil, err
	}
	_, err = mimc.Write(frYBytes[:])
	if err != nil {
		return nil, err
	}
	var padded [32]byte
	big.NewInt(int64(l.MsbX)).FillBytes(padded[:])
	_, err = mimc.Write(padded[:])
	if err != nil {
		return nil, err
	}
	big.NewInt(int64(l.MsbY)).FillBytes(padded[:])
	_, err = mimc.Write(padded[:])
	if err != nil {
		return nil, err
	}
	var powerBytes big.Int
	powerBytes.SetUint64(uint64(l.VotingPower))
	powerBytes.FillBytes(padded[:])
	_, err = mimc.Write(padded[:])
	if err != nil {
		return nil, err
	}
	return mimc.Sum(nil), nil
}
