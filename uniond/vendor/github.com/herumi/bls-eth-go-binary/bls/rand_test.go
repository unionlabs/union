package bls

import (
	"crypto/rand"
	"fmt"
	"testing"
)

func genSec(t *testing.T) {
	var sec SecretKey
	for i := 0; i < 10; i++ {
		sec.SetByCSPRNG()
		fmt.Printf("i=%v sec=%v\n", i, sec)
		if sec.IsZero() {
			t.Fatal("err")
		}
	}
}

func TestRand(t *testing.T) {
	Init(BLS12_381)
	SetETHmode(EthModeDraft07)
	fmt.Printf("default\n")
	genSec(t)
	fmt.Printf("rand.Reader\n")
	SetRandFunc(rand.Reader)
	genSec(t)
	SetRandFunc(nil)
}
