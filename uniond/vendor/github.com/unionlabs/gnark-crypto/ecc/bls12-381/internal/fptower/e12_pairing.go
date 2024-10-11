package fptower

func (z *E12) nSquare(n int) {
	for i := 0; i < n; i++ {
		z.CyclotomicSquare(z)
	}
}

func (z *E12) nSquareCompressed(n int) {
	for i := 0; i < n; i++ {
		z.CyclotomicSquareCompressed(z)
	}
}

// ExptHalf set z to x^(t/2) in E12 and return z
// const t/2 uint64 = 7566188111470821376 // negative
func (z *E12) ExptHalf(x *E12) *E12 {
	var result E12
	var t [2]E12
	result.Set(x)
	result.nSquareCompressed(15)
	t[0].Set(&result)
	result.nSquareCompressed(32)
	t[1].Set(&result)
	batch := BatchDecompressKarabina([]E12{t[0], t[1]})
	result.Mul(&batch[0], &batch[1])
	batch[1].nSquare(9)
	result.Mul(&result, &batch[1])
	batch[1].nSquare(3)
	result.Mul(&result, &batch[1])
	batch[1].nSquare(2)
	result.Mul(&result, &batch[1])
	batch[1].CyclotomicSquare(&batch[1])
	result.Mul(&result, &batch[1])
	return z.Conjugate(&result) // because tAbsVal is negative
}

// Expt set z to xᵗ in E12 and return z
// const t uint64 = 15132376222941642752 // negative
func (z *E12) Expt(x *E12) *E12 {
	var result E12
	result.ExptHalf(x)
	return z.CyclotomicSquare(&result)
}

// MulBy014 multiplication by sparse element (c0, c1, 0, 0, c4)
func (z *E12) MulBy014(c0, c1, c4 *E2) *E12 {

	var a, b E6
	var d E2

	a.Set(&z.C0)
	a.MulBy01(c0, c1)

	b.Set(&z.C1)
	b.MulBy1(c4)
	d.Add(c1, c4)

	z.C1.Add(&z.C1, &z.C0)
	z.C1.MulBy01(c0, &d)
	z.C1.Sub(&z.C1, &a)
	z.C1.Sub(&z.C1, &b)
	z.C0.MulByNonResidue(&b)
	z.C0.Add(&z.C0, &a)

	return z
}

// Mul014By014 multiplication of sparse element (c0,c1,0,0,c4,0) by sparse element (d0,d1,0,0,d4,0)
func Mul014By014(d0, d1, d4, c0, c1, c4 *E2) [5]E2 {
	var z00, tmp, x0, x1, x4, x04, x01, x14 E2
	x0.Mul(c0, d0)
	x1.Mul(c1, d1)
	x4.Mul(c4, d4)
	tmp.Add(c0, c4)
	x04.Add(d0, d4).
		Mul(&x04, &tmp).
		Sub(&x04, &x0).
		Sub(&x04, &x4)
	tmp.Add(c0, c1)
	x01.Add(d0, d1).
		Mul(&x01, &tmp).
		Sub(&x01, &x0).
		Sub(&x01, &x1)
	tmp.Add(c1, c4)
	x14.Add(d1, d4).
		Mul(&x14, &tmp).
		Sub(&x14, &x1).
		Sub(&x14, &x4)

	z00.MulByNonResidue(&x4).
		Add(&z00, &x0)

	return [5]E2{z00, x01, x1, x04, x14}
}

// MulBy01245 multiplies z by an E12 sparse element of the form (x0, x1, x2, 0, x4, x5)
func (z *E12) MulBy01245(x *[5]E2) *E12 {
	var c1, a, b, c, z0, z1 E6
	c0 := &E6{B0: x[0], B1: x[1], B2: x[2]}
	c1.B1 = x[3]
	c1.B2 = x[4]
	a.Add(&z.C0, &z.C1)
	b.Add(c0, &c1)
	a.Mul(&a, &b)
	b.Mul(&z.C0, c0)
	c.Set(&z.C1).MulBy12(&x[3], &x[4])
	z1.Sub(&a, &b)
	z1.Sub(&z1, &c)
	z0.MulByNonResidue(&c)
	z0.Add(&z0, &b)

	z.C0 = z0
	z.C1 = z1

	return z
}
