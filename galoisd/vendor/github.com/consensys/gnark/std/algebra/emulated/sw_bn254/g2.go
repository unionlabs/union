package sw_bn254

import (
	"math/big"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/algebra/emulated/fields_bn254"
	"github.com/consensys/gnark/std/math/emulated"
)

type G2 struct {
	*fields_bn254.Ext2
	w    *emulated.Element[emulated.BN254Fp]
	u, v *fields_bn254.E2
}

type G2Affine struct {
	X, Y fields_bn254.E2
}

func NewG2(api frontend.API) *G2 {
	w := emulated.ValueOf[emulated.BN254Fp]("21888242871839275220042445260109153167277707414472061641714758635765020556616")
	u := fields_bn254.E2{
		A0: emulated.ValueOf[emulated.BN254Fp]("21575463638280843010398324269430826099269044274347216827212613867836435027261"),
		A1: emulated.ValueOf[emulated.BN254Fp]("10307601595873709700152284273816112264069230130616436755625194854815875713954"),
	}
	v := fields_bn254.E2{
		A0: emulated.ValueOf[emulated.BN254Fp]("2821565182194536844548159561693502659359617185244120367078079554186484126554"),
		A1: emulated.ValueOf[emulated.BN254Fp]("3505843767911556378687030309984248845540243509899259641013678093033130930403"),
	}
	return &G2{
		Ext2: fields_bn254.NewExt2(api),
		w:    &w,
		u:    &u,
		v:    &v,
	}
}

func NewG2Affine(v bn254.G2Affine) G2Affine {
	return G2Affine{
		X: fields_bn254.E2{
			A0: emulated.ValueOf[emulated.BN254Fp](v.X.A0),
			A1: emulated.ValueOf[emulated.BN254Fp](v.X.A1),
		},
		Y: fields_bn254.E2{
			A0: emulated.ValueOf[emulated.BN254Fp](v.Y.A0),
			A1: emulated.ValueOf[emulated.BN254Fp](v.Y.A1),
		},
	}
}

func (g2 *G2) phi(q *G2Affine) *G2Affine {
	x := g2.Ext2.MulByElement(&q.X, g2.w)

	return &G2Affine{
		X: *x,
		Y: q.Y,
	}
}

func (g2 *G2) psi(q *G2Affine) *G2Affine {
	x := g2.Ext2.Conjugate(&q.X)
	x = g2.Ext2.Mul(x, g2.u)
	y := g2.Ext2.Conjugate(&q.Y)
	y = g2.Ext2.Mul(y, g2.v)

	return &G2Affine{
		X: *x,
		Y: *y,
	}
}

func (g2 *G2) scalarMulBySeed(q *G2Affine) *G2Affine {
	z := g2.double(q)
	t0 := g2.add(q, z)
	t2 := g2.add(q, t0)
	t1 := g2.add(z, t2)
	z = g2.doubleAndAdd(t1, t0)
	t0 = g2.add(t0, z)
	t2 = g2.add(t2, t0)
	t1 = g2.add(t1, t2)
	t0 = g2.add(t0, t1)
	t1 = g2.add(t1, t0)
	t0 = g2.add(t0, t1)
	t2 = g2.add(t2, t0)
	t1 = g2.doubleAndAdd(t2, t1)
	t2 = g2.add(t2, t1)
	z = g2.add(z, t2)
	t2 = g2.add(t2, z)
	z = g2.doubleAndAdd(t2, z)
	t0 = g2.add(t0, z)
	t1 = g2.add(t1, t0)
	t3 := g2.double(t1)
	t3 = g2.doubleAndAdd(t3, t1)
	t2 = g2.add(t2, t3)
	t1 = g2.add(t1, t2)
	t2 = g2.add(t2, t1)
	t2 = g2.doubleN(t2, 16)
	t1 = g2.doubleAndAdd(t2, t1)
	t1 = g2.doubleN(t1, 13)
	t0 = g2.doubleAndAdd(t1, t0)
	t0 = g2.doubleN(t0, 15)
	z = g2.doubleAndAdd(t0, z)

	return z
}

func (g2 G2) add(p, q *G2Affine) *G2Affine {
	// compute λ = (q.y-p.y)/(q.x-p.x)
	qypy := g2.Ext2.Sub(&q.Y, &p.Y)
	qxpx := g2.Ext2.Sub(&q.X, &p.X)
	λ := g2.Ext2.DivUnchecked(qypy, qxpx)

	// xr = λ²-p.x-q.x
	λλ := g2.Ext2.Square(λ)
	qxpx = g2.Ext2.Add(&p.X, &q.X)
	xr := g2.Ext2.Sub(λλ, qxpx)

	// p.y = λ(p.x-r.x) - p.y
	pxrx := g2.Ext2.Sub(&p.X, xr)
	λpxrx := g2.Ext2.Mul(λ, pxrx)
	yr := g2.Ext2.Sub(λpxrx, &p.Y)

	return &G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func (g2 G2) neg(p *G2Affine) *G2Affine {
	xr := &p.X
	yr := g2.Ext2.Neg(&p.Y)
	return &G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func (g2 G2) sub(p, q *G2Affine) *G2Affine {
	qNeg := g2.neg(q)
	return g2.add(p, qNeg)
}

func (g2 *G2) double(p *G2Affine) *G2Affine {
	// compute λ = (3p.x²)/2*p.y
	xx3a := g2.Square(&p.X)
	xx3a = g2.MulByConstElement(xx3a, big.NewInt(3))
	y2 := g2.Double(&p.Y)
	λ := g2.DivUnchecked(xx3a, y2)

	// xr = λ²-2p.x
	x2 := g2.Double(&p.X)
	λλ := g2.Square(λ)
	xr := g2.Sub(λλ, x2)

	// yr = λ(p-xr) - p.y
	pxrx := g2.Sub(&p.X, xr)
	λpxrx := g2.Mul(λ, pxrx)
	yr := g2.Sub(λpxrx, &p.Y)

	return &G2Affine{
		X: *xr,
		Y: *yr,
	}
}

func (g2 *G2) doubleN(p *G2Affine, n int) *G2Affine {
	pn := p
	for s := 0; s < n; s++ {
		pn = g2.double(pn)
	}
	return pn
}

func (g2 G2) doubleAndAdd(p, q *G2Affine) *G2Affine {

	// compute λ1 = (q.y-p.y)/(q.x-p.x)
	yqyp := g2.Ext2.Sub(&q.Y, &p.Y)
	xqxp := g2.Ext2.Sub(&q.X, &p.X)
	λ1 := g2.Ext2.DivUnchecked(yqyp, xqxp)

	// compute x2 = λ1²-p.x-q.x
	λ1λ1 := g2.Ext2.Square(λ1)
	xqxp = g2.Ext2.Add(&p.X, &q.X)
	x2 := g2.Ext2.Sub(λ1λ1, xqxp)

	// ommit y2 computation
	// compute λ2 = -λ1-2*p.y/(x2-p.x)
	ypyp := g2.Ext2.Add(&p.Y, &p.Y)
	x2xp := g2.Ext2.Sub(x2, &p.X)
	λ2 := g2.Ext2.DivUnchecked(ypyp, x2xp)
	λ2 = g2.Ext2.Add(λ1, λ2)
	λ2 = g2.Ext2.Neg(λ2)

	// compute x3 =λ2²-p.x-x3
	λ2λ2 := g2.Ext2.Square(λ2)
	x3 := g2.Ext2.Sub(λ2λ2, &p.X)
	x3 = g2.Ext2.Sub(x3, x2)

	// compute y3 = λ2*(p.x - x3)-p.y
	y3 := g2.Ext2.Sub(&p.X, x3)
	y3 = g2.Ext2.Mul(λ2, y3)
	y3 = g2.Ext2.Sub(y3, &p.Y)

	return &G2Affine{
		X: *x3,
		Y: *y3,
	}
}

// AssertIsEqual asserts that p and q are the same point.
func (g2 *G2) AssertIsEqual(p, q *G2Affine) {
	g2.Ext2.AssertIsEqual(&p.X, &q.X)
	g2.Ext2.AssertIsEqual(&p.Y, &q.Y)
}
