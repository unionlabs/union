// Package bn256 implements a particular bilinear group at the 128-bit security
// level.
//
// Bilinear groups are the basis of many of the new cryptographic protocols that
// have been proposed over the past decade. They consist of a triplet of groups
// (G₁, G₂ and GT) such that there exists a function e(g₁ˣ,g₂ʸ)=gTˣʸ (where gₓ
// is a generator of the respective group). That function is called a pairing
// function.
//
// This package specifically implements the Optimal Ate pairing over a 256-bit
// Barreto-Naehrig curve as described in
// http://cryptojedi.org/papers/dclxvi-20100714.pdf. Its output is not
// compatible with the implementation described in that paper, as different
// parameters are chosen.
//
// (This package previously claimed to operate at a 128-bit security level.
// However, recent improvements in attacks mean that is no longer true. See
// https://moderncrypto.org/mail-archive/curves/2016/000740.html.)
package bn256

import (
	"crypto/rand"
	"errors"
	"io"
	"math/big"
)

func randomK(r io.Reader) (k *big.Int, err error) {
	for {
		k, err = rand.Int(r, Order)
		if err != nil || k.Sign() > 0 {
			return
		}
	}
}

// G1 is an abstract cyclic group. The zero value is suitable for use as the
// output of an operation, but cannot be used as an input.
type G1 struct {
	p *curvePoint
}

// RandomG1 returns x and g₁ˣ where x is a random, non-zero number read from r.
func RandomG1(r io.Reader) (*big.Int, *G1, error) {
	k, err := randomK(r)
	if err != nil {
		return nil, nil, err
	}

	return k, new(G1).ScalarBaseMult(k), nil
}

func (g *G1) String() string {
	return "bn256.G1" + g.p.String()
}

// ScalarBaseMult sets e to g*k where g is the generator of the group and then
// returns e.
func (e *G1) ScalarBaseMult(k *big.Int) *G1 {
	if e.p == nil {
		e.p = &curvePoint{}
	}
	e.p.Mul(curveGen, k)
	return e
}

// ScalarMult sets e to a*k and then returns e.
func (e *G1) ScalarMult(a *G1, k *big.Int) *G1 {
	if e.p == nil {
		e.p = &curvePoint{}
	}
	e.p.Mul(a.p, k)
	return e
}

// Add sets e to a+b and then returns e.
func (e *G1) Add(a, b *G1) *G1 {
	if e.p == nil {
		e.p = &curvePoint{}
	}
	e.p.Add(a.p, b.p)
	return e
}

// Neg sets e to -a and then returns e.
func (e *G1) Neg(a *G1) *G1 {
	if e.p == nil {
		e.p = &curvePoint{}
	}
	e.p.Neg(a.p)
	return e
}

// Set sets e to a and then returns e.
func (e *G1) Set(a *G1) *G1 {
	if e.p == nil {
		e.p = &curvePoint{}
	}
	e.p.Set(a.p)
	return e
}

// Marshal converts e to a byte slice.
func (e *G1) Marshal() []byte {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8

	if e.p == nil {
		e.p = &curvePoint{}
	}

	e.p.MakeAffine()
	ret := make([]byte, numBytes*2)
	if e.p.IsInfinity() {
		return ret
	}
	temp := &gfP{}

	montDecode(temp, &e.p.x)
	temp.Marshal(ret)
	montDecode(temp, &e.p.y)
	temp.Marshal(ret[numBytes:])

	return ret
}

// Unmarshal sets e to the result of converting the output of Marshal back into
// a group element and then returns e.
func (e *G1) Unmarshal(m []byte) ([]byte, error) {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8
	if len(m) < 2*numBytes {
		return nil, errors.New("bn256: not enough data")
	}
	// Unmarshal the points and check their caps
	if e.p == nil {
		e.p = &curvePoint{}
	} else {
		e.p.x, e.p.y = gfP{0}, gfP{0}
	}
	var err error
	if err = e.p.x.Unmarshal(m); err != nil {
		return nil, err
	}
	if err = e.p.y.Unmarshal(m[numBytes:]); err != nil {
		return nil, err
	}
	// Encode into Montgomery form and ensure it's on the curve
	montEncode(&e.p.x, &e.p.x)
	montEncode(&e.p.y, &e.p.y)

	zero := gfP{0}
	if e.p.x == zero && e.p.y == zero {
		// This is the point at infinity.
		e.p.y = *newGFp(1)
		e.p.z = gfP{0}
		e.p.t = gfP{0}
	} else {
		e.p.z = *newGFp(1)
		e.p.t = *newGFp(1)

		if !e.p.IsOnCurve() {
			return nil, errors.New("bn256: malformed point")
		}
	}
	return m[2*numBytes:], nil
}

// G2 is an abstract cyclic group. The zero value is suitable for use as the
// output of an operation, but cannot be used as an input.
type G2 struct {
	p *twistPoint
}

// RandomG2 returns x and g₂ˣ where x is a random, non-zero number read from r.
func RandomG2(r io.Reader) (*big.Int, *G2, error) {
	k, err := randomK(r)
	if err != nil {
		return nil, nil, err
	}

	return k, new(G2).ScalarBaseMult(k), nil
}

func (e *G2) String() string {
	return "bn256.G2" + e.p.String()
}

// ScalarBaseMult sets e to g*k where g is the generator of the group and then
// returns out.
func (e *G2) ScalarBaseMult(k *big.Int) *G2 {
	if e.p == nil {
		e.p = &twistPoint{}
	}
	e.p.Mul(twistGen, k)
	return e
}

// ScalarMult sets e to a*k and then returns e.
func (e *G2) ScalarMult(a *G2, k *big.Int) *G2 {
	if e.p == nil {
		e.p = &twistPoint{}
	}
	e.p.Mul(a.p, k)
	return e
}

// Add sets e to a+b and then returns e.
func (e *G2) Add(a, b *G2) *G2 {
	if e.p == nil {
		e.p = &twistPoint{}
	}
	e.p.Add(a.p, b.p)
	return e
}

// Neg sets e to -a and then returns e.
func (e *G2) Neg(a *G2) *G2 {
	if e.p == nil {
		e.p = &twistPoint{}
	}
	e.p.Neg(a.p)
	return e
}

// Set sets e to a and then returns e.
func (e *G2) Set(a *G2) *G2 {
	if e.p == nil {
		e.p = &twistPoint{}
	}
	e.p.Set(a.p)
	return e
}

// Marshal converts e into a byte slice.
func (e *G2) Marshal() []byte {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8

	if e.p == nil {
		e.p = &twistPoint{}
	}

	e.p.MakeAffine()
	ret := make([]byte, numBytes*4)
	if e.p.IsInfinity() {
		return ret
	}
	temp := &gfP{}

	montDecode(temp, &e.p.x.x)
	temp.Marshal(ret)
	montDecode(temp, &e.p.x.y)
	temp.Marshal(ret[numBytes:])
	montDecode(temp, &e.p.y.x)
	temp.Marshal(ret[2*numBytes:])
	montDecode(temp, &e.p.y.y)
	temp.Marshal(ret[3*numBytes:])

	return ret
}

// Unmarshal sets e to the result of converting the output of Marshal back into
// a group element and then returns e.
func (e *G2) Unmarshal(m []byte) ([]byte, error) {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8
	if len(m) < 4*numBytes {
		return nil, errors.New("bn256: not enough data")
	}
	// Unmarshal the points and check their caps
	if e.p == nil {
		e.p = &twistPoint{}
	}
	var err error
	if err = e.p.x.x.Unmarshal(m); err != nil {
		return nil, err
	}
	if err = e.p.x.y.Unmarshal(m[numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.x.Unmarshal(m[2*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.y.Unmarshal(m[3*numBytes:]); err != nil {
		return nil, err
	}
	// Encode into Montgomery form and ensure it's on the curve
	montEncode(&e.p.x.x, &e.p.x.x)
	montEncode(&e.p.x.y, &e.p.x.y)
	montEncode(&e.p.y.x, &e.p.y.x)
	montEncode(&e.p.y.y, &e.p.y.y)

	if e.p.x.IsZero() && e.p.y.IsZero() {
		// This is the point at infinity.
		e.p.y.SetOne()
		e.p.z.SetZero()
		e.p.t.SetZero()
	} else {
		e.p.z.SetOne()
		e.p.t.SetOne()

		if !e.p.IsOnCurve() {
			return nil, errors.New("bn256: malformed point")
		}
	}
	return m[4*numBytes:], nil
}

// GT is an abstract cyclic group. The zero value is suitable for use as the
// output of an operation, but cannot be used as an input.
type GT struct {
	p *gfP12
}

// Pair calculates an Optimal Ate pairing.
func Pair(g1 *G1, g2 *G2) *GT {
	return &GT{optimalAte(g2.p, g1.p)}
}

// PairingCheck calculates the Optimal Ate pairing for a set of points.
func PairingCheck(a []*G1, b []*G2) bool {
	acc := new(gfP12)
	acc.SetOne()

	for i := 0; i < len(a); i++ {
		if a[i].p.IsInfinity() || b[i].p.IsInfinity() {
			continue
		}
		acc.Mul(acc, miller(b[i].p, a[i].p))
	}
	return finalExponentiation(acc).IsOne()
}

// Miller applies Miller's algorithm, which is a bilinear function from the
// source groups to F_p^12. Miller(g1, g2).Finalize() is equivalent to Pair(g1,
// g2).
func Miller(g1 *G1, g2 *G2) *GT {
	return &GT{miller(g2.p, g1.p)}
}

func (g *GT) String() string {
	return "bn256.GT" + g.p.String()
}

// ScalarMult sets e to a*k and then returns e.
func (e *GT) ScalarMult(a *GT, k *big.Int) *GT {
	if e.p == nil {
		e.p = &gfP12{}
	}
	e.p.Exp(a.p, k)
	return e
}

// Add sets e to a+b and then returns e.
func (e *GT) Add(a, b *GT) *GT {
	if e.p == nil {
		e.p = &gfP12{}
	}
	e.p.Mul(a.p, b.p)
	return e
}

// Neg sets e to -a and then returns e.
func (e *GT) Neg(a *GT) *GT {
	if e.p == nil {
		e.p = &gfP12{}
	}
	e.p.Conjugate(a.p)
	return e
}

// Set sets e to a and then returns e.
func (e *GT) Set(a *GT) *GT {
	if e.p == nil {
		e.p = &gfP12{}
	}
	e.p.Set(a.p)
	return e
}

// Finalize is a linear function from F_p^12 to GT.
func (e *GT) Finalize() *GT {
	ret := finalExponentiation(e.p)
	e.p.Set(ret)
	return e
}

// Marshal converts e into a byte slice.
func (e *GT) Marshal() []byte {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8

	if e.p == nil {
		e.p = &gfP12{}
		e.p.SetOne()
	}

	ret := make([]byte, numBytes*12)
	temp := &gfP{}

	montDecode(temp, &e.p.x.x.x)
	temp.Marshal(ret)
	montDecode(temp, &e.p.x.x.y)
	temp.Marshal(ret[numBytes:])
	montDecode(temp, &e.p.x.y.x)
	temp.Marshal(ret[2*numBytes:])
	montDecode(temp, &e.p.x.y.y)
	temp.Marshal(ret[3*numBytes:])
	montDecode(temp, &e.p.x.z.x)
	temp.Marshal(ret[4*numBytes:])
	montDecode(temp, &e.p.x.z.y)
	temp.Marshal(ret[5*numBytes:])
	montDecode(temp, &e.p.y.x.x)
	temp.Marshal(ret[6*numBytes:])
	montDecode(temp, &e.p.y.x.y)
	temp.Marshal(ret[7*numBytes:])
	montDecode(temp, &e.p.y.y.x)
	temp.Marshal(ret[8*numBytes:])
	montDecode(temp, &e.p.y.y.y)
	temp.Marshal(ret[9*numBytes:])
	montDecode(temp, &e.p.y.z.x)
	temp.Marshal(ret[10*numBytes:])
	montDecode(temp, &e.p.y.z.y)
	temp.Marshal(ret[11*numBytes:])

	return ret
}

// Unmarshal sets e to the result of converting the output of Marshal back into
// a group element and then returns e.
func (e *GT) Unmarshal(m []byte) ([]byte, error) {
	// Each value is a 256-bit number.
	const numBytes = 256 / 8

	if len(m) < 12*numBytes {
		return nil, errors.New("bn256: not enough data")
	}

	if e.p == nil {
		e.p = &gfP12{}
	}

	var err error
	if err = e.p.x.x.x.Unmarshal(m); err != nil {
		return nil, err
	}
	if err = e.p.x.x.y.Unmarshal(m[numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.x.y.x.Unmarshal(m[2*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.x.y.y.Unmarshal(m[3*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.x.z.x.Unmarshal(m[4*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.x.z.y.Unmarshal(m[5*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.x.x.Unmarshal(m[6*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.x.y.Unmarshal(m[7*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.y.x.Unmarshal(m[8*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.y.y.Unmarshal(m[9*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.z.x.Unmarshal(m[10*numBytes:]); err != nil {
		return nil, err
	}
	if err = e.p.y.z.y.Unmarshal(m[11*numBytes:]); err != nil {
		return nil, err
	}
	montEncode(&e.p.x.x.x, &e.p.x.x.x)
	montEncode(&e.p.x.x.y, &e.p.x.x.y)
	montEncode(&e.p.x.y.x, &e.p.x.y.x)
	montEncode(&e.p.x.y.y, &e.p.x.y.y)
	montEncode(&e.p.x.z.x, &e.p.x.z.x)
	montEncode(&e.p.x.z.y, &e.p.x.z.y)
	montEncode(&e.p.y.x.x, &e.p.y.x.x)
	montEncode(&e.p.y.x.y, &e.p.y.x.y)
	montEncode(&e.p.y.y.x, &e.p.y.y.x)
	montEncode(&e.p.y.y.y, &e.p.y.y.y)
	montEncode(&e.p.y.z.x, &e.p.y.z.x)
	montEncode(&e.p.y.z.y, &e.p.y.z.y)

	return m[12*numBytes:], nil
}
