package bandersnatch

import (
	"math"
	"math/big"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
)

// phi endomorphism sqrt(-2) \in O(-8)
// (x,y,z)->\lambda*(x,y,z) s.t. \lamba^2 = -2 mod Order
func (p *PointProj) phi(p1 *PointProj) *PointProj {

	initOnce.Do(initCurveParams)

	var zz, yy, xy, f, g, h fr.Element
	zz.Square(&p1.Z)
	yy.Square(&p1.Y)
	xy.Mul(&p1.X, &p1.Y)
	f.Sub(&zz, &yy).Mul(&f, &curveParams.endo[1])
	zz.Mul(&zz, &curveParams.endo[0])
	g.Add(&yy, &zz).Mul(&g, &curveParams.endo[0])
	h.Sub(&yy, &zz)

	p.X.Mul(&f, &h)
	p.Y.Mul(&g, &xy)
	p.Z.Mul(&h, &xy)

	return p
}

// ScalarMultiplication scalar multiplication (GLV) of a point
// p1 in projective coordinates with a scalar in big.Int
func (p *PointProj) scalarMulGLV(p1 *PointProj, scalar *big.Int) *PointProj {

	initOnce.Do(initCurveParams)

	var table [15]PointProj
	var zero big.Int
	var res PointProj
	var k1, k2 fr.Element

	res.setInfinity()

	// table[b3b2b1b0-1] = b3b2*phi(p1) + b1b0*p1
	table[0].Set(p1)
	table[3].phi(p1)

	// split the scalar, modifies +-p1, phi(p1) accordingly
	k := ecc.SplitScalar(scalar, &curveParams.glvBasis)

	if k[0].Cmp(&zero) == -1 {
		k[0].Neg(&k[0])
		table[0].Neg(&table[0])
	}
	if k[1].Cmp(&zero) == -1 {
		k[1].Neg(&k[1])
		table[3].Neg(&table[3])
	}

	// precompute table (2 bits sliding window)
	// table[b3b2b1b0-1] = b3b2*phi(p1) + b1b0*p1 if b3b2b1b0 != 0
	table[1].Double(&table[0])
	table[2].Set(&table[1]).Add(&table[2], &table[0])
	table[4].Set(&table[3]).Add(&table[4], &table[0])
	table[5].Set(&table[3]).Add(&table[5], &table[1])
	table[6].Set(&table[3]).Add(&table[6], &table[2])
	table[7].Double(&table[3])
	table[8].Set(&table[7]).Add(&table[8], &table[0])
	table[9].Set(&table[7]).Add(&table[9], &table[1])
	table[10].Set(&table[7]).Add(&table[10], &table[2])
	table[11].Set(&table[7]).Add(&table[11], &table[3])
	table[12].Set(&table[11]).Add(&table[12], &table[0])
	table[13].Set(&table[11]).Add(&table[13], &table[1])
	table[14].Set(&table[11]).Add(&table[14], &table[2])

	// bounds on the lattice base vectors guarantee that k1, k2 are len(r)/2 bits long max
	k1 = k1.SetBigInt(&k[0]).Bits()
	k2 = k2.SetBigInt(&k[1]).Bits()

	// loop starts from len(k1)/2 due to the bounds
	// fr.Limbs == Order.limbs
	for i := int(math.Ceil(fr.Limbs/2. - 1)); i >= 0; i-- {
		mask := uint64(3) << 62
		for j := 0; j < 32; j++ {
			res.Double(&res).Double(&res)
			b1 := (k1[i] & mask) >> (62 - 2*j)
			b2 := (k2[i] & mask) >> (62 - 2*j)
			if b1|b2 != 0 {
				scalar := (b2<<2 | b1)
				res.Add(&res, &table[scalar-1])
			}
			mask = mask >> 2
		}
	}

	p.Set(&res)
	return p
}

// phi endomorphism sqrt(-2) \in O(-8)
// (x,y,z)->\lambda*(x,y,z) s.t. \lamba^2 = -2 mod Order
func (p *PointExtended) phi(p1 *PointExtended) *PointExtended {
	initOnce.Do(initCurveParams)

	var zz, yy, xy, f, g, h fr.Element
	zz.Square(&p1.Z)
	yy.Square(&p1.Y)
	xy.Mul(&p1.X, &p1.Y)
	f.Sub(&zz, &yy).Mul(&f, &curveParams.endo[1])
	zz.Mul(&zz, &curveParams.endo[0])
	g.Add(&yy, &zz).Mul(&g, &curveParams.endo[0])
	h.Sub(&yy, &zz)

	p.X.Mul(&f, &h)
	p.Y.Mul(&g, &xy)
	p.Z.Mul(&h, &xy)
	p.T.Mul(&f, &g)

	return p
}

// ScalarMultiplication scalar multiplication (GLV) of a point
// p1 in projective coordinates with a scalar in big.Int
func (p *PointExtended) scalarMulGLV(p1 *PointExtended, scalar *big.Int) *PointExtended {
	initOnce.Do(initCurveParams)

	var table [15]PointExtended
	var zero big.Int
	var res PointExtended
	var k1, k2 fr.Element

	res.setInfinity()

	// table[b3b2b1b0-1] = b3b2*phi(p1) + b1b0*p1
	table[0].Set(p1)
	table[3].phi(p1)

	// split the scalar, modifies +-p1, phi(p1) accordingly
	k := ecc.SplitScalar(scalar, &curveParams.glvBasis)

	if k[0].Cmp(&zero) == -1 {
		k[0].Neg(&k[0])
		table[0].Neg(&table[0])
	}
	if k[1].Cmp(&zero) == -1 {
		k[1].Neg(&k[1])
		table[3].Neg(&table[3])
	}

	// precompute table (2 bits sliding window)
	// table[b3b2b1b0-1] = b3b2*phi(p1) + b1b0*p1 if b3b2b1b0 != 0
	table[1].Double(&table[0])
	table[2].Set(&table[1]).Add(&table[2], &table[0])
	table[4].Set(&table[3]).Add(&table[4], &table[0])
	table[5].Set(&table[3]).Add(&table[5], &table[1])
	table[6].Set(&table[3]).Add(&table[6], &table[2])
	table[7].Double(&table[3])
	table[8].Set(&table[7]).Add(&table[8], &table[0])
	table[9].Set(&table[7]).Add(&table[9], &table[1])
	table[10].Set(&table[7]).Add(&table[10], &table[2])
	table[11].Set(&table[7]).Add(&table[11], &table[3])
	table[12].Set(&table[11]).Add(&table[12], &table[0])
	table[13].Set(&table[11]).Add(&table[13], &table[1])
	table[14].Set(&table[11]).Add(&table[14], &table[2])

	// bounds on the lattice base vectors guarantee that k1, k2 are len(r)/2 bits long max
	k1 = k1.SetBigInt(&k[0]).Bits()
	k2 = k2.SetBigInt(&k[1]).Bits()

	// loop starts from len(k1)/2 due to the bounds
	// fr.Limbs == Order.limbs
	for i := int(math.Ceil(fr.Limbs/2. - 1)); i >= 0; i-- {
		mask := uint64(3) << 62
		for j := 0; j < 32; j++ {
			res.Double(&res).Double(&res)
			b1 := (k1[i] & mask) >> (62 - 2*j)
			b2 := (k2[i] & mask) >> (62 - 2*j)
			if b1|b2 != 0 {
				scalar := (b2<<2 | b1)
				res.Add(&res, &table[scalar-1])
			}
			mask = mask >> 2
		}
	}

	p.Set(&res)
	return p
}
