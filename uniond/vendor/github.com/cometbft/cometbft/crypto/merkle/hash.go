package merkle

import (
	"hash"
	"math/big"

	"github.com/cometbft/cometbft/crypto/tmhash"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
)

// TODO: make these have a large predefined capacity.
var (
	leafPrefix  = []byte{0}
	innerPrefix = []byte{1}
)

// returns tmhash(<empty>).
func emptyHash() []byte {
	return tmhash.Sum([]byte{})
}

// returns tmhash(0x00 || leaf).
func leafHash(leaf []byte) []byte {
	return tmhash.Sum(append(leafPrefix, leaf...))
}

// returns tmhash(0x00 || leaf).
func leafHashOpt(s hash.Hash, leaf []byte) []byte {
	s.Reset()
	s.Write(leafPrefix)
	s.Write(leaf)
	return s.Sum(nil)
}

// returns tmhash(0x01 || left || right).
func innerHash(left []byte, right []byte) []byte {
	return tmhash.SumMany(innerPrefix, left, right)
}

func innerHashOpt(s hash.Hash, left []byte, right []byte) []byte {
	s.Reset()
	s.Write(innerPrefix)
	s.Write(left)
	s.Write(right)
	return s.Sum(nil)
}

// returns mimc(<empty>)
func emptyMimcHash() []byte {
	return mimc.NewMiMC().Sum(nil)
}

// returns mimc(0x00, leaf)
func leafMimcHash(leaf []byte) []byte {
	hash := mimc.NewMiMC()
	var prefix big.Int
	prefix.SetBit(&prefix, 0, uint(leafPrefix[0]))
	var paddedPrefix [32]byte
	prefix.FillBytes(paddedPrefix[:])
	_, err := hash.Write(paddedPrefix[:])
	if err != nil {
		panic(err)
	}
	_, err = hash.Write(leaf)
	if err != nil {
		panic(err)
	}
	return hash.Sum(nil)
}

// returns mimc(0x01, left, right)
func innerMimcHash(left []byte, right []byte) []byte {
	hash := mimc.NewMiMC()
	var prefix big.Int
	prefix.SetBit(&prefix, 0, uint(innerPrefix[0]))
	var paddedPrefix [32]byte
	prefix.FillBytes(paddedPrefix[:])
	_, err := hash.Write(paddedPrefix[:])
	if err != nil {
		panic(err)
	}
	_, err = hash.Write(left)
	if err != nil {
		panic(err)
	}
	_, err = hash.Write(right)
	if err != nil {
		panic(err)
	}
	return hash.Sum(nil)
}
