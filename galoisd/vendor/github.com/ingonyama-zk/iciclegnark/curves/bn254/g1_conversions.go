package bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	icicle "github.com/ingonyama-zk/icicle/goicicle/curves/bn254"
)

func BatchConvertFromG1Affine(elements []bn254.G1Affine) []icicle.G1PointAffine {
	var newElements []icicle.G1PointAffine
	for _, e := range elements {
		var newElement icicle.G1ProjectivePoint
		FromG1AffineGnark(&e, &newElement)

		newElements = append(newElements, *newElement.StripZ())
	}
	return newElements
}

func ProjectiveToGnarkAffine(p *icicle.G1ProjectivePoint) *bn254.G1Affine {
	px := BaseFieldToGnarkFp(&p.X)
	py := BaseFieldToGnarkFp(&p.Y)
	pz := BaseFieldToGnarkFp(&p.Z)

	zInv := new(fp.Element)
	x := new(fp.Element)
	y := new(fp.Element)

	zInv.Inverse(pz)

	x.Mul(px, zInv)
	y.Mul(py, zInv)

	return &bn254.G1Affine{X: *x, Y: *y}
}

func G1ProjectivePointToGnarkJac(p *icicle.G1ProjectivePoint) *bn254.G1Jac {
	var p1 bn254.G1Jac
	p1.FromAffine(ProjectiveToGnarkAffine(p))

	return &p1
}

func FromG1AffineGnark(gnark *bn254.G1Affine, p *icicle.G1ProjectivePoint) *icicle.G1ProjectivePoint {
	var z icicle.G1BaseField
	z.SetOne()

	p.X = *NewFieldFromFpGnark[icicle.G1BaseField](gnark.X)
	p.Y = *NewFieldFromFpGnark[icicle.G1BaseField](gnark.Y)
	p.Z = z

	return p
}

func G1ProjectivePointFromJacGnark(p *icicle.G1ProjectivePoint, gnark *bn254.G1Jac) *icicle.G1ProjectivePoint {
	var pointAffine bn254.G1Affine
	pointAffine.FromJacobian(gnark)

	var z icicle.G1BaseField
	z.SetOne()

	p.X = *NewFieldFromFpGnark[icicle.G1BaseField](pointAffine.X)
	p.Y = *NewFieldFromFpGnark[icicle.G1BaseField](pointAffine.Y)
	p.Z = z

	return p
}

func AffineToGnarkAffine(p *icicle.G1PointAffine) *bn254.G1Affine {
	return ProjectiveToGnarkAffine(p.ToProjective())
}