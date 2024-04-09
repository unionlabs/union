package main

import (
	"encoding/hex"
	"fmt"
	"github.com/herumi/bls-eth-go-binary/bls"
)

func sample1() {
	fmt.Printf("sample1\n")
	var sec bls.SecretKey
	sec.SetByCSPRNG()
	msg := []byte("abc")
	pub := sec.GetPublicKey()
	sig := sec.SignByte(msg)
	fmt.Printf("verify=%v\n", sig.VerifyByte(pub, msg))
}

func sample2() {
	fmt.Printf("sample2\n")
	var sec bls.SecretKey
	sec.SetByCSPRNG()
	fmt.Printf("sec:%s\n", sec.SerializeToHexStr())
	pub := sec.GetPublicKey()
	fmt.Printf("1.pub:%s\n", pub.SerializeToHexStr())
	fmt.Printf("1.pub x=%x\n", pub)
	var P *bls.G1 = bls.CastFromPublicKey(pub)
	bls.G1Normalize(P, P)
	fmt.Printf("2.pub:%s\n", pub.SerializeToHexStr())
	fmt.Printf("2.pub x=%x\n", pub)
	fmt.Printf("P.X=%x\n", P.X.Serialize())
	fmt.Printf("P.Y=%x\n", P.Y.Serialize())
	fmt.Printf("P.Z=%x\n", P.Z.Serialize())
}

func sample3() {
	fmt.Printf("sample3\n")
	var sec bls.SecretKey
	b := make([]byte, 64)
	for i := 0; i < len(b); i++ {
		b[i] = 0xff
	}
	err := sec.SetLittleEndianMod(b)
	if err != nil {
		fmt.Printf("err")
		return
	}
	fmt.Printf("sec=%x\n", sec.Serialize())
}

func sample4() {
	fmt.Printf("sample4\n")
	var sec bls.SecretKey
	secByte, _ := hex.DecodeString("4aac41b5cb665b93e031faa751944b1f14d77cb17322403cba8df1d6e4541a4d")
	sec.Deserialize(secByte)
	msg := []byte("message to be signed.")
	fmt.Printf("sec:%x\n", sec.Serialize())
	pub := sec.GetPublicKey()
	fmt.Printf("pub:%x\n", pub.Serialize())
	sig := sec.SignByte(msg)
	fmt.Printf("sig:%x\n", sig.Serialize())
}

func main() {
	bls.Init(bls.BLS12_381)
	bls.SetETHmode(bls.EthModeDraft07)
	sample1()
	sample2()
	sample3()
	sample4()
}
