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

// Expt set z to x^t in E12 and return z
func (z *E12) Expt(x *E12) *E12 {
	// const tAbsVal uint64 = 9586122913090633729
	// tAbsVal in binary: 1000010100001000110000000000000000000000000000000000000000000001
	// drop the low 46 bits (all 0 except the least significant bit): 100001010000100011 = 136227
	// Shortest addition chains can be found at https://wwwhomes.uni-bielefeld.de/achim/addition_chain.html

	var result, x33 E12

	// a shortest addition chain for 136227
	result.Set(x)
	result.nSquare(5)
	result.Mul(&result, x)
	x33.Set(&result)
	result.nSquare(7)
	result.Mul(&result, &x33)
	result.nSquare(4)
	result.Mul(&result, x)
	result.CyclotomicSquare(&result)
	result.Mul(&result, x)

	// the remaining 46 bits
	result.nSquareCompressed(46)
	result.DecompressKarabina(&result)
	result.Mul(&result, x)

	z.Set(&result)
	return z
}

// MulBy034 multiplication by sparse element (c0,0,0,c3,c4,0)
func (z *E12) MulBy034(c0, c3, c4 *E2) *E12 {

	var a, b, d E6

	a.MulByE2(&z.C0, c0)

	b.Set(&z.C1)
	b.MulBy01(c3, c4)

	c0.Add(c0, c3)
	d.Add(&z.C0, &z.C1)
	d.MulBy01(c0, c4)

	z.C1.Add(&a, &b).Neg(&z.C1).Add(&z.C1, &d)
	z.C0.MulByNonResidue(&b).Add(&z.C0, &a)

	return z
}

// Mul034By034 multiplication of sparse element (c0,0,0,c3,c4,0) by sparse element (d0,0,0,d3,d4,0)
func Mul034By034(d0, d3, d4, c0, c3, c4 *E2) [5]E2 {
	var z00, tmp, x0, x3, x4, x04, x03, x34 E2
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

	return [5]E2{z00, x3, x34, x03, x04}
}

// MulBy01234 multiplies z by an E12 sparse element of the form (x0, x1, x2, x3, x4, 0)
func (z *E12) MulBy01234(x *[5]E2) *E12 {
	var c1, a, b, c, z0, z1 E6
	c0 := &E6{B0: x[0], B1: x[1], B2: x[2]}
	c1.B0 = x[3]
	c1.B1 = x[4]
	a.Add(&z.C0, &z.C1)
	b.Add(c0, &c1)
	a.Mul(&a, &b)
	b.Mul(&z.C0, c0)
	c.Set(&z.C1).MulBy01(&x[3], &x[4])
	z1.Sub(&a, &b)
	z1.Sub(&z1, &c)
	z0.MulByNonResidue(&c)
	z0.Add(&z0, &b)

	z.C0 = z0
	z.C1 = z1

	return z
}
