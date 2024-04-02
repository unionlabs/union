package main

import (
	// "encoding/hex"
	"crypto/sha256"
	"encoding/base64"
	"fmt"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	exMimc "github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
	"github.com/consensys/gnark/std/hash/mimc"
	// "github.com/consensys/gnark/std/math/uints"
)

const MaxLength = 12
const InputLen = 6
const MaxPathDepth = 16
const MaxPrefixCount = 3
const MaxSuffixCount = 2

// type Circuit struct {
// 	Root     frontend.Variable `gnark:",public"`
// 	InputLen frontend.Variable
// 	Inputs   [MaxLength]frontend.Variable
// }

type Inner struct {
	// 16 or 48
	Prefix    [MaxPrefixCount]frontend.Variable
	PrefixLen frontend.Variable
	// 0 or 32
	Suffix    [MaxSuffixCount]frontend.Variable
	SuffixLen frontend.Variable
}

type Ics23Circuit struct {
	Proof ExistenceProof
	Root  frontend.Variable `gnark:",public"`
}

type ExistenceProof struct {
	HashedKey   [2]frontend.Variable `gnark:",public"`
	HashedValue [2]frontend.Variable `gnark:",public"`
	Path        [MaxPathDepth]Inner  `gnark:",public"`
	PathLen     frontend.Variable    `gnark:",public"`
	LeafPrefix  frontend.Variable    `gnark:",public"`
}

type NonExistCircuit struct {
	Left  ExistenceProof
	Right ExistenceProof
	Root  frontend.Variable
}

func (proof *ExistenceProof) Verify(api frontend.API, root frontend.Variable) frontend.Variable {
	hFunc, _ := mimc.NewMiMC(api)
	hFunc.Write(proof.LeafPrefix, proof.HashedKey[0], proof.HashedKey[1], proof.HashedValue[0], proof.HashedValue[1])
	calcRoot := hFunc.Sum()
	// api.Println(111)
	// api.Println(calcRoot)
	// api.Println(0)

	for i := 0; i < MaxPathDepth; i++ {
		hFunc.Reset()
		// prefix = 48, suffix = 0
		rootBin := api.ToBinary(calcRoot, 256)
		rootMSB := api.FromBinary(rootBin[128:256]...)
		rootLSB := api.FromBinary(rootBin[0:128]...)
		hFunc.Write(proof.Path[i].Prefix[0], proof.Path[i].Prefix[1], proof.Path[i].Prefix[2], rootMSB, rootLSB)
		withPrefix := hFunc.Sum()
		hFunc.Reset()
		// prefix = 16, suffix = 32
		hFunc.Write(proof.Path[i].Prefix[0], rootMSB, rootLSB, proof.Path[i].Suffix[0], proof.Path[i].Suffix[1])
		withSuffix := hFunc.Sum()

		shouldSet := api.IsZero(api.Sub(api.Cmp(i, proof.PathLen), -1))
		tmpRoot := api.Select(api.IsZero(api.Sub(proof.Path[i].PrefixLen, 3)), withPrefix, withSuffix)
		calcRoot = api.Select(shouldSet, tmpRoot, calcRoot)
		// api.Println(calcRoot)
	}

	api.Println(root, calcRoot)
	return api.IsZero(api.Sub(root, calcRoot))
}

func (circuit *NonExistCircuit) Define(api frontend.API) error {
	leftVerify := circuit.Left.Verify(api, circuit.Root)
	rightVerify := circuit.Right.Verify(api, circuit.Root)

	api.Println(leftVerify, rightVerify)

	leftExists := api.Sub(1, api.IsZero(circuit.Left.PathLen))
	rightExists := api.Sub(1, api.IsZero(circuit.Right.PathLen))

	// At least one of right or left should exist
	api.AssertIsEqual(1, api.Or(leftExists, rightExists))

	leftCheck := api.Select(leftExists, leftVerify, 1)
	rightCheck := api.Select(rightExists, rightVerify, 1)

	api.AssertIsEqual(1, api.And(leftCheck, rightCheck))

	return nil
}

func (circuit *Ics23Circuit) Define(api frontend.API) error {
	valid := circuit.Proof.Verify(api, circuit.Root)
	api.AssertIsEqual(valid, 1)

	return nil
}

func prepareNonCircuit() NonExistCircuit {
	root, _ := base64.StdEncoding.DecodeString("JCFH7nrxPNhiE94yFdzetxc/QpQBCglrm9xaAQNuZk8=")
	leftKey, _ := base64.StdEncoding.DecodeString("MefHpQ==")
	leftValue, _ := base64.StdEncoding.DecodeString("dmFsdWVfZm9yX2tleTox58el")

	leftPrefix1, _ := base64.StdEncoding.DecodeString("AQAAAAIAAAAAAAAAAQAAAA==")
	leftSuffix1, _ := base64.StdEncoding.DecodeString("ICAASxmCyYmLsLBUxVKnGFFP4Zg/io+LXwOrM9dLQJ8=")

	leftPrefix2, _ := base64.StdEncoding.DecodeString("AgAAAAQAAAAAAAAAAQAAABvS5qQ3pVHdypAkpftBpY8PlebsHKkhoiyvhuL4vQU6")

	leftPrefix3, _ := base64.StdEncoding.DecodeString("AwAAAAYAAAAAAAAAAQAAAA==")
	leftSuffix3, _ := base64.StdEncoding.DecodeString("Cfh9LYmC+DP8lnq6FQutR2LVT+LZNN72Nyvvjcob0ac=")

	leftPrefix4, _ := base64.StdEncoding.DecodeString("BAAAAAwAAAAAAAAAAQAAAB/muEIecXsMSMIl7A5+w5sxFUK2U+ihh2YtumazFXF0")

	leftPrefix5, _ := base64.StdEncoding.DecodeString("BQAAABcAAAAAAAAAAQAAACpYHxfiAgUcCtgsYqrJgZzxxDdB8MmWZw0xSoByOfRr")

	leftPrefix6, _ := base64.StdEncoding.DecodeString("BgAAAC0AAAAAAAAAAQAAAA==")
	leftSuffix6, _ := base64.StdEncoding.DecodeString("CBI5ujv5+EFSG7LsB0hNWYKkY7u+RcJXC3keSyDytho=")

	leftPrefix7, _ := base64.StdEncoding.DecodeString("CAAAAGQAAAAAAAAAAQAAAA==")
	leftSuffix7, _ := base64.StdEncoding.DecodeString("BLJDfYgn3CDoFdWiibKO69zGte8B4wljCb/xLaHj2fg=")

	rightKey, _ := base64.StdEncoding.DecodeString("M3+68Q==")
	rightValue, _ := base64.StdEncoding.DecodeString("dmFsdWVfZm9yX2tleTozf7rx")

	rightPrefix1, _ := base64.StdEncoding.DecodeString("AQAAAAIAAAAAAAAAAQAAACxZGjWW5A8bzns8z45NMw55dca/GeFcr9/ptbXSZSD9")

	rightPrefix2, _ := base64.StdEncoding.DecodeString("AgAAAAQAAAAAAAAAAQAAABvS5qQ3pVHdypAkpftBpY8PlebsHKkhoiyvhuL4vQU6")

	rightPrefix3, _ := base64.StdEncoding.DecodeString("AwAAAAYAAAAAAAAAAQAAAA==")
	rightSuffix3, _ := base64.StdEncoding.DecodeString("Cfh9LYmC+DP8lnq6FQutR2LVT+LZNN72Nyvvjcob0ac=")

	rightPrefix4, _ := base64.StdEncoding.DecodeString("BAAAAAwAAAAAAAAAAQAAAB/muEIecXsMSMIl7A5+w5sxFUK2U+ihh2YtumazFXF0")

	rightPrefix5, _ := base64.StdEncoding.DecodeString("BQAAABcAAAAAAAAAAQAAACpYHxfiAgUcCtgsYqrJgZzxxDdB8MmWZw0xSoByOfRr")

	rightPrefix6, _ := base64.StdEncoding.DecodeString("BgAAAC0AAAAAAAAAAQAAAA==")
	rightSuffix6, _ := base64.StdEncoding.DecodeString("CBI5ujv5+EFSG7LsB0hNWYKkY7u+RcJXC3keSyDytho=")

	rightPrefix7, _ := base64.StdEncoding.DecodeString("CAAAAGQAAAAAAAAAAQAAAA==")
	rightSuffix7, _ := base64.StdEncoding.DecodeString("BLJDfYgn3CDoFdWiibKO69zGte8B4wljCb/xLaHj2fg=")

	leftHashedKey := sha256.Sum256(leftKey)
	leftHashedValue := sha256.Sum256(leftValue)

	rightHashedKey := sha256.Sum256(rightKey)
	rightHashedValue := sha256.Sum256(rightValue)

	leftLeaf, _ := base64.StdEncoding.DecodeString("AAAAAAEAAAAAAAAAAQAAAA==")
	rightLeaf, _ := base64.StdEncoding.DecodeString("AAAAAAEAAAAAAAAAAQAAAA==")

	return NonExistCircuit{
		Left: ExistenceProof{
			HashedKey:   [2]frontend.Variable{leftHashedKey[:16], leftHashedKey[16:]},
			HashedValue: [2]frontend.Variable{leftHashedValue[:16], leftHashedValue[16:]},
			Path: [16]Inner{
				{
					Prefix:    [3]frontend.Variable{leftPrefix1, 0, 0},
					Suffix:    [2]frontend.Variable{leftSuffix1[:16], leftSuffix1[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix2[:16], leftPrefix2[16:32], leftPrefix2[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix3, 0, 0},
					Suffix:    [2]frontend.Variable{leftSuffix3[:16], leftSuffix3[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix4[:16], leftPrefix4[16:32], leftPrefix4[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix5[:16], leftPrefix5[16:32], leftPrefix5[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix6, 0, 0},
					Suffix:    [2]frontend.Variable{leftSuffix6[:16], leftSuffix6[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{leftPrefix7, 0, 0},
					Suffix:    [2]frontend.Variable{leftSuffix7[:16], leftSuffix7[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
			},
			PathLen:    7,
			LeafPrefix: leftLeaf,
		},
		Right: ExistenceProof{
			HashedKey:   [2]frontend.Variable{rightHashedKey[:16], rightHashedKey[16:]},
			HashedValue: [2]frontend.Variable{rightHashedValue[:16], rightHashedValue[16:]},
			Path: [16]Inner{
				{
					Prefix:    [3]frontend.Variable{rightPrefix1[:16], rightPrefix1[16:32], rightPrefix1[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix2[:16], rightPrefix2[16:32], rightPrefix2[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix3, 0, 0},
					Suffix:    [2]frontend.Variable{rightSuffix3[:16], rightSuffix3[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix4[:16], rightPrefix4[16:32], rightPrefix4[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix5[:16], rightPrefix5[16:32], rightPrefix5[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix6, 0, 0},
					Suffix:    [2]frontend.Variable{rightSuffix6[:16], rightSuffix6[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{rightPrefix7, 0, 0},
					Suffix:    [2]frontend.Variable{rightSuffix7[:16], rightSuffix7[16:32]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
			},
			PathLen:    7,
			LeafPrefix: rightLeaf,
		},
		Root: root,
	}
}

func prepareCircuit() Ics23Circuit {
	var testElem fr.Element
	testElem.SetBytes([]byte{41, 212, 239, 17, 31, 143, 172, 99, 202, 16, 252, 171, 115, 124, 255, 151, 39, 229, 12, 89, 91, 86, 178, 219, 24, 102, 213, 135, 157, 214, 63, 247})
	fmt.Println("testElem:", testElem.Text(10))

	h := exMimc.NewMiMC()
	var lPrefixElem fr.Element
	lPrefixElem.SetBytes([]byte{0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0})
	lPrefixBytes := lPrefixElem.Bytes()
	fmt.Println("lprefix      :", []byte{0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0})
	fmt.Println("lprefix field:", lPrefixBytes)
	h.Write(lPrefixBytes[:])

	lKeyFullBytes := []byte{238, 93, 223, 3, 191, 173, 139, 231, 195, 184, 18, 19, 172, 40, 194, 199, 52, 55, 66, 191, 135, 198, 5, 231, 10, 161, 54, 150, 169, 21, 107, 160}
	lValFullBytes := []byte{98, 84, 223, 160, 41, 97, 110, 216, 36, 91, 97, 58, 63, 52, 158, 244, 46, 3, 28, 194, 136, 157, 132, 128, 79, 98, 47, 91, 98, 48, 79, 55}

	var lKey fr.Element
	lKey.SetBytes(lKeyFullBytes[:16])
	lKeyBytes := lKey.Bytes()
	fmt.Println("lkeybytes:", lKeyBytes)
	h.Write(lKeyBytes[:])
	lKey.SetBytes(lKeyFullBytes[16:])
	lKeyBytes = lKey.Bytes()
	h.Write(lKeyBytes[:])

	var lVal fr.Element
	lVal.SetBytes(lValFullBytes[:16])
	lValBytes := lVal.Bytes()
	h.Write(lValBytes[:])
	lVal.SetBytes(lValFullBytes[16:])
	lValBytes = lVal.Bytes()
	h.Write(lValBytes[:])

	res := h.Sum(nil)
	var resElem fr.Element
	resElem.SetBytes(res)
	fmt.Println("res:", resElem.Text(10))
	fmt.Println("res:", res)

	h.Reset()
	var bytesBlock [32]byte
	copy(bytesBlock[16:32], []byte{0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0})
	h.Write(bytesBlock[:])
	copy(bytesBlock[16:32], lKeyFullBytes[:16])
	h.Write(bytesBlock[:])
	copy(bytesBlock[16:32], lKeyFullBytes[16:])
	h.Write(bytesBlock[:])
	copy(bytesBlock[16:32], lValFullBytes[:16])
	h.Write(bytesBlock[:])
	copy(bytesBlock[16:32], lValFullBytes[16:])
	h.Write(bytesBlock[:])
	fmt.Println("res res:", h.Sum(nil))
	fmt.Println(res)

	root, _ := base64.StdEncoding.DecodeString("KQOQdm/b1gd8yBK/SCDbkllUOE6zhxMheFfC+Kc2JEw=")
	leafPrefix, _ := base64.StdEncoding.DecodeString("AAAAAAEAAAAAAAAAAQAAAA==")

	inner1Prefix, _ := base64.StdEncoding.DecodeString("AQAAAAIAAAAAAAAAAQAAACKaEjb78wORWfgHtIXA8yt6wCQ1FAGebRHOXCy6x9BG")
	// inner1Suffix, _ := base64.StdEncoding.DecodeString("")

	inner2Prefix, _ := base64.StdEncoding.DecodeString("AwAAAAUAAAAAAAAAAQAAAA==")
	inner2Suffix, _ := base64.StdEncoding.DecodeString("BvtQqTuf9nDMp25gaFFAMp0qe9X96yiNJJq0iGHreBA=")

	inner3Prefix, _ := base64.StdEncoding.DecodeString("BAAAAAgAAAAAAAAAAQAAAB2baAUi3zePo69ERKkvs7xoa4Gw2Nw7dHldGgInun0g")
	// inner3Suffix, _ := base64.StdEncoding.DecodeString("CA8sfzISfuhW/25vFI8hKIw9hY1cbG31wQw/pxx7v/I=")

	inner4Prefix, _ := base64.StdEncoding.DecodeString("BQAAABIAAAAAAAAAAQAAAA==")
	inner4Suffix, _ := base64.StdEncoding.DecodeString("DNGx6Gl097ld8FT3IiDfW2vz6U5oQG9xjPUaBAjWri4=")

	inner5Prefix, _ := base64.StdEncoding.DecodeString("BgAAAB8AAAAAAAAAAQAAAA==")
	inner5Suffix, _ := base64.StdEncoding.DecodeString("DS+2mXD/MkkKCSi9maoi7kLTSoN6YfiPM34wCC9kbEg=")

	inner6Prefix, _ := base64.StdEncoding.DecodeString("BwAAAD0AAAAAAAAAAQAAABRuvdWX5HoZtTUIPP5fABn2ups8aHWnFbb2jDtGdTnZ")
	// inner6Suffix, _ := base64.StdEncoding.DecodeString("DTlJ8IcfYERP7tqkuCxCTDwcZ0SaY8WxIYV1yf67R7w=")

	inner7Prefix, _ := base64.StdEncoding.DecodeString("CAAAAGAAAAAAAAAAAQAAABLHy5hp4MfCb+0269wsvDe5CyQpyoaKi+cxMH76avWq")

	inner8Prefix, _ := base64.StdEncoding.DecodeString("CQAAANgAAAAAAAAAAQAAAA==")
	inner8Suffix, _ := base64.StdEncoding.DecodeString("JHQzfcvXo/EVSmkCxo2UOsu8P6gSnizjY8VjUOX+sSs=")

	inner9Prefix, _ := base64.StdEncoding.DecodeString("CgAAALABAAAAAAAAAQAAAA==")
	inner9Suffix, _ := base64.StdEncoding.DecodeString("LtNPQ5nyEMt7b3xOMMJgHvgqFnw7ohlWlOI0YbzGpHY=")

	inner10Prefix, _ := base64.StdEncoding.DecodeString("CwAAAM8CAAAAAAAAAQAAAA==")
	inner10Suffix, _ := base64.StdEncoding.DecodeString("Bd3Vjm/n9wuSSVDlQCoc0mPZKCn6T3iRAy9jGKstwvE=")

	inner11Prefix, _ := base64.StdEncoding.DecodeString("DAAAALoEAAAAAAAAAQAAAA==")
	inner11Suffix, _ := base64.StdEncoding.DecodeString("D4ldykY/blzmxbmAQswMwhs2rftsr4IHoACkogcT6Dk=")

	inner12Prefix, _ := base64.StdEncoding.DecodeString("DgAAAIIKAAAAAAAAAQAAAA==")
	inner12Suffix, _ := base64.StdEncoding.DecodeString("DudggFXz38dSg5O64xaKU4vtTIyLCIGEcSUC5jNPf1c=")

	inner13Prefix, _ := base64.StdEncoding.DecodeString("DwAAADcVAAAAAAAAAQAAAAznPdfw275BTwWyu0F9FybW7TH8DZHdQB1vv3sEz5EL")

	key, _ := base64.StdEncoding.DecodeString("g1AAhQ==")
	value, _ := base64.StdEncoding.DecodeString("dmFsdWVfZm9yX2tleTqDUACF")

	hashedKey := sha256.Sum256(key)
	hashedValue := sha256.Sum256(value)

	return Ics23Circuit{
		Proof: ExistenceProof{
			HashedKey: [2]frontend.Variable{
				hashedKey[:16], hashedKey[16:],
			},
			HashedValue: [2]frontend.Variable{
				hashedValue[:16], hashedValue[16:],
			},
			Path: [MaxPathDepth]Inner{
				{
					Prefix:    [3]frontend.Variable{inner1Prefix[:16], inner1Prefix[16:32], inner1Prefix[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{inner2Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner2Suffix[:16], inner2Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix: [3]frontend.Variable{inner3Prefix[0:16], inner3Prefix[16:32], inner3Prefix[32:48]},
					// Suffix:    [2]frontend.Variable{inner3Suffix[:16], inner3Suffix[16:]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{inner4Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner4Suffix[:16], inner4Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner5Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner5Suffix[:16], inner5Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix: [3]frontend.Variable{inner6Prefix[0:16], inner6Prefix[16:32], inner6Prefix[32:48]},
					// Suffix:    [2]frontend.Variable{inner6Suffix[:16], inner6Suffix[16:]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{inner7Prefix[0:16], inner7Prefix[16:32], inner7Prefix[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{inner8Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner8Suffix[:16], inner8Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner9Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner9Suffix[:16], inner9Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner10Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner10Suffix[:16], inner10Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner11Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner11Suffix[:16], inner11Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner12Prefix, 0, 0},
					Suffix:    [2]frontend.Variable{inner12Suffix[:16], inner12Suffix[16:]},
					PrefixLen: 1,
					SuffixLen: 2,
				},
				{
					Prefix:    [3]frontend.Variable{inner13Prefix[0:16], inner13Prefix[16:32], inner13Prefix[32:48]},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 3,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
				{
					Prefix:    [3]frontend.Variable{0, 0, 0},
					Suffix:    [2]frontend.Variable{0, 0},
					PrefixLen: 0,
					SuffixLen: 0,
				},
			},
			PathLen:    13,
			LeafPrefix: leafPrefix,
		},
		Root: root,
	}
}

func main() {
	// compiles our circuit into a R1CS
	var circuit Ics23Circuit
	ccs, _ := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &circuit)

	// groth16 zkSNARK: Setup
	pk, vk, _ := groth16.Setup(ccs)

	// witness definition
	assignment := prepareCircuit()
	witness, _ := frontend.NewWitness(&assignment, ecc.BN254.ScalarField())
	publicWitness, _ := witness.Public()

	// groth16: Prove & Verify
	proof, _ := groth16.Prove(ccs, pk, witness)
	groth16.Verify(proof, vk, publicWitness)

	var nonCircuit NonExistCircuit
	ccs, _ = frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, &nonCircuit)

	// groth16 zkSNARK: Setup
	pk, vk, _ = groth16.Setup(ccs)

	// witness definition
	assignment2 := prepareNonCircuit()
	witness, _ = frontend.NewWitness(&assignment2, ecc.BN254.ScalarField())
	publicWitness, _ = witness.Public()

	// groth16: Prove & Verify
	proof, _ = groth16.Prove(ccs, pk, witness)
	groth16.Verify(proof, vk, publicWitness)
}
