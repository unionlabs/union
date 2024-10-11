package bls12381

import (
	"bytes"
	"crypto/rand"
	"math/big"
	"testing"
)

func TestFieldElementValidation(t *testing.T) {
	zero := new(fe).zero()
	if !zero.isValid() {
		t.Fatal("zero must be valid")
	}
	one := new(fe).one()
	if !one.isValid() {
		t.Fatal("one must be valid")
	}
	if modulus.isValid() {
		t.Fatal("modulus must be invalid")
	}
	n := modulus.big()
	n.Add(n, big.NewInt(1))
	if new(fe).setBig(n).isValid() {
		t.Fatal("number greater than modulus must be invalid")
	}
}

func TestFieldElementEquality(t *testing.T) {
	// fe
	zero := new(fe).zero()
	if !zero.equal(zero) {
		t.Fatal("0 == 0")
	}
	one := new(fe).one()
	if !one.equal(one) {
		t.Fatal("1 == 1")
	}
	a, _ := new(fe).rand(rand.Reader)
	if !a.equal(a) {
		t.Fatal("a == a")
	}
	b := new(fe)
	add(b, a, one)
	if a.equal(b) {
		t.Fatal("a != a + 1")
	}
	// fe2
	zero2 := new(fe2).zero()
	if !zero2.equal(zero2) {
		t.Fatal("0 == 0")
	}
	one2 := new(fe2).one()
	if !one2.equal(one2) {
		t.Fatal("1 == 1")
	}
	a2, _ := new(fe2).rand(rand.Reader)
	if !a2.equal(a2) {
		t.Fatal("a == a")
	}
	b2 := new(fe2)
	fp2 := newFp2()
	fp2.add(b2, a2, one2)
	if a2.equal(b2) {
		t.Fatal("a != a + 1")
	}
	// fe6
	zero6 := new(fe6).zero()
	if !zero6.equal(zero6) {
		t.Fatal("0 == 0")
	}
	one6 := new(fe6).one()
	if !one6.equal(one6) {
		t.Fatal("1 == 1")
	}
	a6, _ := new(fe6).rand(rand.Reader)
	if !a6.equal(a6) {
		t.Fatal("a == a")
	}
	b6 := new(fe6)
	fp6 := newFp6(fp2)
	fp6.add(b6, a6, one6)
	if a6.equal(b6) {
		t.Fatal("a != a + 1")
	}
	// fe12
	zero12 := new(fe12).zero()
	if !zero12.equal(zero12) {
		t.Fatal("0 == 0")
	}
	one12 := new(fe12).one()
	if !one12.equal(one12) {
		t.Fatal("1 == 1")
	}
	a12, _ := new(fe12).rand(rand.Reader)
	if !a12.equal(a12) {
		t.Fatal("a == a")
	}
	b12 := new(fe12)
	fp12 := newFp12(fp6)
	fp12.add(b12, a12, one12)
	if a12.equal(b12) {
		t.Fatal("a != a + 1")
	}
}

func TestFieldElementHelpers(t *testing.T) {
	// fe
	zero := new(fe).zero()
	if !zero.isZero() {
		t.Fatal("'zero' is not zero")
	}
	one := new(fe).one()
	if !one.isOne() {
		t.Fatal("'one' is not one")
	}
	odd := new(fe).setBig(big.NewInt(1))
	if !odd.isOdd() {
		t.Fatal("1 must be odd")
	}
	if odd.isEven() {
		t.Fatal("1 must not be even")
	}
	even := new(fe).setBig(big.NewInt(2))
	if !even.isEven() {
		t.Fatal("2 must be even")
	}
	if even.isOdd() {
		t.Fatal("2 must not be odd")
	}
	// fe2
	zero2 := new(fe2).zero()
	if !zero2.isZero() {
		t.Fatal("'zero' is not zero, 2")
	}
	one2 := new(fe2).one()
	if !one2.isOne() {
		t.Fatal("'one' is not one, 2")
	}
	// fe6
	zero6 := new(fe6).zero()
	if !zero6.isZero() {
		t.Fatal("'zero' is not zero, 6")
	}
	one6 := new(fe6).one()
	if !one6.isOne() {
		t.Fatal("'one' is not one, 6")
	}
	// fe12
	zero12 := new(fe12).zero()
	if !zero12.isZero() {
		t.Fatal("'zero' is not zero, 12")
	}
	one12 := new(fe12).one()
	if !one12.isOne() {
		t.Fatal("'one' is not one, 12")
	}
}

func TestFieldElementSerialization(t *testing.T) {
	t.Run("zero", func(t *testing.T) {
		in := make([]byte, 48)
		fe := new(fe).setBytes(in)
		if !fe.isZero() {
			t.Fatal("bad serialization")
		}
		if !bytes.Equal(in, fe.bytes()) {
			t.Fatal("bad serialization")
		}
	})
	t.Run("bytes", func(t *testing.T) {
		for i := 0; i < fuz; i++ {
			a, _ := new(fe).rand(rand.Reader)
			b := new(fe).setBytes(a.bytes())
			if !a.equal(b) {
				t.Fatal("bad serialization")
			}
		}
	})
	t.Run("big", func(t *testing.T) {
		for i := 0; i < fuz; i++ {
			a, _ := new(fe).rand(rand.Reader)
			b := new(fe).setBig(a.big())
			if !a.equal(b) {
				t.Fatal("bad encoding or decoding")
			}
		}
	})
	t.Run("string", func(t *testing.T) {
		for i := 0; i < fuz; i++ {
			a, _ := new(fe).rand(rand.Reader)
			b, err := new(fe).setString(a.string())
			if err != nil {
				t.Fatal(err)
			}
			if !a.equal(b) {
				t.Fatal("bad encoding or decoding")
			}
		}
	})
}

func TestFieldElementByteInputs(t *testing.T) {
	zero := new(fe).zero()
	in := make([]byte, 0)
	a := new(fe).setBytes(in)
	if !a.equal(zero) {
		t.Fatal("bad serialization")
	}
	in = make([]byte, 48)
	a = new(fe).setBytes(in)
	if !a.equal(zero) {
		t.Fatal("bad serialization")
	}
	in = make([]byte, 64)
	a = new(fe).setBytes(in)
	if !a.equal(zero) {
		t.Fatal("bad serialization")
	}
	in = make([]byte, 49)
	in[47] = 1
	normalOne := &fe{1, 0, 0, 0, 0, 0}
	a = new(fe).setBytes(in)
	if !a.equal(normalOne) {
		t.Fatal("bad serialization")
	}
}

func TestFieldElementCopy(t *testing.T) {
	a, _ := new(fe).rand(rand.Reader)
	b := new(fe).set(a)
	if !a.equal(b) {
		t.Fatal("bad copy, 1")
	}
	a2, _ := new(fe2).rand(rand.Reader)
	b2 := new(fe2).set(a2)
	if !a2.equal(b2) {
		t.Fatal("bad copy, 2")
	}
	a6, _ := new(fe6).rand(rand.Reader)
	b6 := new(fe6).set(a6)
	if !a6.equal(b6) {
		t.Fatal("bad copy, 6")
	}
	a12, _ := new(fe12).rand(rand.Reader)
	b12 := new(fe12).set(a12)
	if !a12.equal(b12) {
		t.Fatal("bad copy, 12")
	}
}
