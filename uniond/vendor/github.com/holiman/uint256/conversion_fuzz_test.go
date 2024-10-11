package uint256

import (
	"fmt"
	"math/big"
	"testing"
)

func testSetFromDecForFuzzing(tc string) error {
	a := new(Int).SetAllOne()
	err := a.SetFromDecimal(tc)
	// If input is negative, we should eror
	if len(tc) > 0 && tc[0] == '-' {
		if err == nil {
			return fmt.Errorf("want error on negative input")
		}
		return nil
	}
	// Need to compare with big.Int
	bigA, ok := big.NewInt(0).SetString(tc, 10)
	if !ok {
		if err == nil {
			return fmt.Errorf("want error")
		}
		return nil // both agree that input is bad
	}
	if bigA.BitLen() > 256 {
		if err == nil {
			return fmt.Errorf("want error (bitlen > 256)")
		}
		return nil
	}
	want := bigA.String()
	have := a.Dec()
	if want != have {
		return fmt.Errorf("want %v, have %v", want, have)
	}
	if _, err := a.Value(); err != nil {
		return fmt.Errorf("fail to Value() %s, got err %s", tc, err)
	}
	return nil
}

func FuzzSetString(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		if len(data) > 512 {
			return
		}
		if err := testSetFromDecForFuzzing(string(data)); err != nil {
			t.Fatal(err)
		}
	})
}
