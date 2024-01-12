package fptower

func (z *E24) nSquareCompressed(n int) {
	for i := 0; i < n; i++ {
		z.CyclotomicSquareCompressed(z)
	}
}

// Expt set z to x^t in E24 and return z (t is the seed of the curve)
// -2**32+2**30+2**22-2**20+1
func (z *E24) Expt(x *E24) *E24 {

	var result, x20, x22, x30, x32 E24
	result.Set(x)

	result.nSquareCompressed(20)
	x20.Conjugate(&result)
	result.nSquareCompressed(2)
	x22.Set(&result)
	result.nSquareCompressed(8)
	x30.Set(&result)

	batch := BatchDecompressKarabina([]E24{x20, x22, x30})

	x32.CyclotomicSquare(&batch[2]).
		CyclotomicSquare(&x32).
		Conjugate(&x32)

	z.Mul(x, &batch[0]).
		Mul(z, &batch[1]).
		Mul(z, &batch[2]).
		Mul(z, &x32)

	return z
}

// MulBy034 multiplication by sparse element (c0,0,0,c3,c4,0)
func (z *E24) MulBy034(c0, c3, c4 *E4) *E24 {

	var a, b, d E12

	a.MulByE2(&z.D0, c0)

	b.Set(&z.D1)
	b.MulBy01(c3, c4)

	var d0 E4
	d0.Add(c0, c3)
	d.Add(&z.D0, &z.D1)
	d.MulBy01(&d0, c4)

	z.D1.Add(&a, &b).Neg(&z.D1).Add(&z.D1, &d)
	z.D0.MulByNonResidue(&b).Add(&z.D0, &a)

	return z
}

// MulBy34 multiplication by sparse element (1,0,0,c3,c4,0)
func (z *E24) MulBy34(c3, c4 *E4) *E24 {

	var a, b, d E12

	a.Set(&z.D0)

	b.Set(&z.D1)
	b.MulBy01(c3, c4)

	var d0 E4
	d0.SetOne().Add(&d0, c3)
	d.Add(&z.D0, &z.D1)
	d.MulBy01(&d0, c4)

	z.D1.Add(&a, &b).Neg(&z.D1).Add(&z.D1, &d)
	z.D0.MulByNonResidue(&b).Add(&z.D0, &a)

	return z
}

// Mul034By034 multiplication of sparse element (c0,0,0,c3,c4,0) by sparse element (d0,0,0,d3,d4,0)
func Mul034By034(d0, d3, d4, c0, c3, c4 *E4) [5]E4 {
	var z00, tmp, x0, x3, x4, x04, x03, x34 E4
	x0.Mul(c0, d0)
	x3.Mul(c3, d3)
	x4.Mul(c4, d4)
	tmp.Add(c0, c4)
	x04.Add(d0, d4).
		Mul(&x04, &tmp).
		Sub(&x04, &x0).
		Sub(&x04, &x4)
	tmp.Add(c0, c3)
	x03.Add(d0, d3).
		Mul(&x03, &tmp).
		Sub(&x03, &x0).
		Sub(&x03, &x3)
	tmp.Add(c3, c4)
	x34.Add(d3, d4).
		Mul(&x34, &tmp).
		Sub(&x34, &x3).
		Sub(&x34, &x4)

	z00.MulByNonResidue(&x4).
		Add(&z00, &x0)

	return [5]E4{z00, x3, x34, x03, x04}
}

// Mul34By34 multiplication of sparse element (1,0,0,c3,c4,0) by sparse element (1,0,0,d3,d4,0)
func Mul34By34(d3, d4, c3, c4 *E4) [5]E4 {
	var z00, tmp, x0, x3, x4, x04, x03, x34 E4
	x3.Mul(c3, d3)
	x4.Mul(c4, d4)
	x04.Add(c4, d4)
	x03.Add(c3, d3)
	tmp.Add(c3, c4)
	x34.Add(d3, d4).
		Mul(&x34, &tmp).
		Sub(&x34, &x3).
		Sub(&x34, &x4)

	x0.SetOne()
	z00.MulByNonResidue(&x4).
		Add(&z00, &x0)

	return [5]E4{z00, x3, x34, x03, x04}
}

// MulBy01234 multiplies z by an E12 sparse element of the form (x0, x1, x2, x3, x4, 0)
func (z *E24) MulBy01234(x *[5]E4) *E24 {
	var c1, a, b, c, z0, z1 E12
	c0 := &E12{C0: x[0], C1: x[1], C2: x[2]}
	c1.C0 = x[3]
	c1.C1 = x[4]
	a.Add(&z.D0, &z.D1)
	b.Add(c0, &c1)
	a.Mul(&a, &b)
	b.Mul(&z.D0, c0)
	c.Set(&z.D1).MulBy01(&x[3], &x[4])
	z1.Sub(&a, &b)
	z1.Sub(&z1, &c)
	z0.MulByNonResidue(&c)
	z0.Add(&z0, &b)

	z.D0 = z0
	z.D1 = z1

	return z
}
