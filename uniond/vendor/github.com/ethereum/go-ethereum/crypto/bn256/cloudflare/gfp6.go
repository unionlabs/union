package bn256

// For details of the algorithms used, see "Multiplication and Squaring on
// Pairing-Friendly Fields, Devegili et al.
// http://eprint.iacr.org/2006/471.pdf.

// gfP6 implements the field of size p⁶ as a cubic extension of gfP2 where τ³=ξ
// and ξ=i+9.
type gfP6 struct {
	x, y, z gfP2 // value is xτ² + yτ + z
}

func (e *gfP6) String() string {
	return "(" + e.x.String() + ", " + e.y.String() + ", " + e.z.String() + ")"
}

func (e *gfP6) Set(a *gfP6) *gfP6 {
	e.x.Set(&a.x)
	e.y.Set(&a.y)
	e.z.Set(&a.z)
	return e
}

func (e *gfP6) SetZero() *gfP6 {
	e.x.SetZero()
	e.y.SetZero()
	e.z.SetZero()
	return e
}

func (e *gfP6) SetOne() *gfP6 {
	e.x.SetZero()
	e.y.SetZero()
	e.z.SetOne()
	return e
}

func (e *gfP6) IsZero() bool {
	return e.x.IsZero() && e.y.IsZero() && e.z.IsZero()
}

func (e *gfP6) IsOne() bool {
	return e.x.IsZero() && e.y.IsZero() && e.z.IsOne()
}

func (e *gfP6) Neg(a *gfP6) *gfP6 {
	e.x.Neg(&a.x)
	e.y.Neg(&a.y)
	e.z.Neg(&a.z)
	return e
}

func (e *gfP6) Frobenius(a *gfP6) *gfP6 {
	e.x.Conjugate(&a.x)
	e.y.Conjugate(&a.y)
	e.z.Conjugate(&a.z)

	e.x.Mul(&e.x, xiTo2PMinus2Over3)
	e.y.Mul(&e.y, xiToPMinus1Over3)
	return e
}

// FrobeniusP2 computes (xτ²+yτ+z)^(p²) = xτ^(2p²) + yτ^(p²) + z
func (e *gfP6) FrobeniusP2(a *gfP6) *gfP6 {
	// τ^(2p²) = τ²τ^(2p²-2) = τ²ξ^((2p²-2)/3)
	e.x.MulScalar(&a.x, xiTo2PSquaredMinus2Over3)
	// τ^(p²) = ττ^(p²-1) = τξ^((p²-1)/3)
	e.y.MulScalar(&a.y, xiToPSquaredMinus1Over3)
	e.z.Set(&a.z)
	return e
}

func (e *gfP6) FrobeniusP4(a *gfP6) *gfP6 {
	e.x.MulScalar(&a.x, xiToPSquaredMinus1Over3)
	e.y.MulScalar(&a.y, xiTo2PSquaredMinus2Over3)
	e.z.Set(&a.z)
	return e
}

func (e *gfP6) Add(a, b *gfP6) *gfP6 {
	e.x.Add(&a.x, &b.x)
	e.y.Add(&a.y, &b.y)
	e.z.Add(&a.z, &b.z)
	return e
}

func (e *gfP6) Sub(a, b *gfP6) *gfP6 {
	e.x.Sub(&a.x, &b.x)
	e.y.Sub(&a.y, &b.y)
	e.z.Sub(&a.z, &b.z)
	return e
}

func (e *gfP6) Mul(a, b *gfP6) *gfP6 {
	// "Multiplication and Squaring on Pairing-Friendly Fields"
	// Section 4, Karatsuba method.
	// http://eprint.iacr.org/2006/471.pdf
	v0 := (&gfP2{}).Mul(&a.z, &b.z)
	v1 := (&gfP2{}).Mul(&a.y, &b.y)
	v2 := (&gfP2{}).Mul(&a.x, &b.x)

	t0 := (&gfP2{}).Add(&a.x, &a.y)
	t1 := (&gfP2{}).Add(&b.x, &b.y)
	tz := (&gfP2{}).Mul(t0, t1)
	tz.Sub(tz, v1).Sub(tz, v2).MulXi(tz).Add(tz, v0)

	t0.Add(&a.y, &a.z)
	t1.Add(&b.y, &b.z)
	ty := (&gfP2{}).Mul(t0, t1)
	t0.MulXi(v2)
	ty.Sub(ty, v0).Sub(ty, v1).Add(ty, t0)

	t0.Add(&a.x, &a.z)
	t1.Add(&b.x, &b.z)
	tx := (&gfP2{}).Mul(t0, t1)
	tx.Sub(tx, v0).Add(tx, v1).Sub(tx, v2)

	e.x.Set(tx)
	e.y.Set(ty)
	e.z.Set(tz)
	return e
}

func (e *gfP6) MulScalar(a *gfP6, b *gfP2) *gfP6 {
	e.x.Mul(&a.x, b)
	e.y.Mul(&a.y, b)
	e.z.Mul(&a.z, b)
	return e
}

func (e *gfP6) MulGFP(a *gfP6, b *gfP) *gfP6 {
	e.x.MulScalar(&a.x, b)
	e.y.MulScalar(&a.y, b)
	e.z.MulScalar(&a.z, b)
	return e
}

// MulTau computes τ·(aτ²+bτ+c) = bτ²+cτ+aξ
func (e *gfP6) MulTau(a *gfP6) *gfP6 {
	tz := (&gfP2{}).MulXi(&a.x)
	ty := (&gfP2{}).Set(&a.y)

	e.y.Set(&a.z)
	e.x.Set(ty)
	e.z.Set(tz)
	return e
}

func (e *gfP6) Square(a *gfP6) *gfP6 {
	v0 := (&gfP2{}).Square(&a.z)
	v1 := (&gfP2{}).Square(&a.y)
	v2 := (&gfP2{}).Square(&a.x)

	c0 := (&gfP2{}).Add(&a.x, &a.y)
	c0.Square(c0).Sub(c0, v1).Sub(c0, v2).MulXi(c0).Add(c0, v0)

	c1 := (&gfP2{}).Add(&a.y, &a.z)
	c1.Square(c1).Sub(c1, v0).Sub(c1, v1)
	xiV2 := (&gfP2{}).MulXi(v2)
	c1.Add(c1, xiV2)

	c2 := (&gfP2{}).Add(&a.x, &a.z)
	c2.Square(c2).Sub(c2, v0).Add(c2, v1).Sub(c2, v2)

	e.x.Set(c2)
	e.y.Set(c1)
	e.z.Set(c0)
	return e
}

func (e *gfP6) Invert(a *gfP6) *gfP6 {
	// See "Implementing cryptographic pairings", M. Scott, section 3.2.
	// ftp://136.206.11.249/pub/crypto/pairings.pdf

	// Here we can give a short explanation of how it works: let j be a cubic root of
	// unity in GF(p²) so that 1+j+j²=0.
	// Then (xτ² + yτ + z)(xj²τ² + yjτ + z)(xjτ² + yj²τ + z)
	// = (xτ² + yτ + z)(Cτ²+Bτ+A)
	// = (x³ξ²+y³ξ+z³-3ξxyz) = F is an element of the base field (the norm).
	//
	// On the other hand (xj²τ² + yjτ + z)(xjτ² + yj²τ + z)
	// = τ²(y²-ξxz) + τ(ξx²-yz) + (z²-ξxy)
	//
	// So that's why A = (z²-ξxy), B = (ξx²-yz), C = (y²-ξxz)
	t1 := (&gfP2{}).Mul(&a.x, &a.y)
	t1.MulXi(t1)

	A := (&gfP2{}).Square(&a.z)
	A.Sub(A, t1)

	B := (&gfP2{}).Square(&a.x)
	B.MulXi(B)
	t1.Mul(&a.y, &a.z)
	B.Sub(B, t1)

	C := (&gfP2{}).Square(&a.y)
	t1.Mul(&a.x, &a.z)
	C.Sub(C, t1)

	F := (&gfP2{}).Mul(C, &a.y)
	F.MulXi(F)
	t1.Mul(A, &a.z)
	F.Add(F, t1)
	t1.Mul(B, &a.x).MulXi(t1)
	F.Add(F, t1)

	F.Invert(F)

	e.x.Mul(C, F)
	e.y.Mul(B, F)
	e.z.Mul(A, F)
	return e
}
