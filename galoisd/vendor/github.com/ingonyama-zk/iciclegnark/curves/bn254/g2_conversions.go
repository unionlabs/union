package bn254

import (
	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	icicle "github.com/ingonyama-zk/icicle/goicicle/curves/bn254"
	"fmt"
)

func ToGnarkFp(f *icicle.G2Element) *fp.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := fp.LittleEndian.Element(&b32)

	if e != nil {
		panic(fmt.Sprintf("unable to convert point %v; got error %v", f, e))
	}

	return &v
}

func ToGnarkE2(f *icicle.ExtentionField) bn254.E2 {
	return bn254.E2{
		A0: *ToGnarkFp(&f.A0),
		A1: *ToGnarkFp(&f.A1),
	}
}

func G2PointToGnarkJac(p *icicle.G2Point) *bn254.G2Jac {
	x := ToGnarkE2(&p.X)
	y := ToGnarkE2(&p.Y)
	z := ToGnarkE2(&p.Z)
	var zSquared bn254.E2
	zSquared.Mul(&z, &z)

	var X bn254.E2
	X.Mul(&x, &z)

	var Y bn254.E2
	Y.Mul(&y, &zSquared)

	after := bn254.G2Jac{
		X: X,
		Y: Y,
		Z: z,
	}

	return &after
}

func G2AffineFromGnarkAffine(gnark *bn254.G2Affine, g *icicle.G2PointAffine) *icicle.G2PointAffine {
	g.X.A0 = gnark.X.A0.Bits()
	g.X.A1 = gnark.X.A1.Bits()
	g.Y.A0 = gnark.Y.A0.Bits()
	g.Y.A1 = gnark.Y.A1.Bits()

	return g
}

func G2PointAffineFromGnarkJac(gnark *bn254.G2Jac, g *icicle.G2PointAffine) *icicle.G2PointAffine {
	var pointAffine bn254.G2Affine
	pointAffine.FromJacobian(gnark)

	g.X.A0 = pointAffine.X.A0.Bits()
	g.X.A1 = pointAffine.X.A1.Bits()
	g.Y.A0 = pointAffine.Y.A0.Bits()
	g.Y.A1 = pointAffine.Y.A1.Bits()

	return g
}

func BatchConvertFromG2Affine(elements []bn254.G2Affine) []icicle.G2PointAffine {
	var newElements []icicle.G2PointAffine
	for _, gg2Affine := range elements {
		var newElement icicle.G2PointAffine
		G2AffineFromGnarkAffine(&gg2Affine, &newElement)

		newElements = append(newElements, newElement)
	}
	return newElements
}

func BatchConvertFromG2AffineThreaded(elements []bn254.G2Affine, routines int) []icicle.G2PointAffine {
	var newElements []icicle.G2PointAffine

	if routines > 1 && routines <= len(elements) {
		channels := make([]chan []icicle.G2PointAffine, routines)
		for i := 0; i < routines; i++ {
			channels[i] = make(chan []icicle.G2PointAffine, 1)
		}

		convert := func(elements []bn254.G2Affine, chanIndex int) {
			var convertedElements []icicle.G2PointAffine
			for _, e := range elements {
				var converted icicle.G2PointAffine
				G2AffineFromGnarkAffine(&e, &converted)
				convertedElements = append(convertedElements, converted)
			}

			channels[chanIndex] <- convertedElements
		}

		batchLen := len(elements) / routines
		for i := 0; i < routines; i++ {
			start := batchLen * i
			end := batchLen * (i + 1)
			elemsToConv := elements[start:end]
			if i == routines-1 {
				elemsToConv = elements[start:]
			}
			go convert(elemsToConv, i)
		}

		for i := 0; i < routines; i++ {
			newElements = append(newElements, <-channels[i]...)
		}
	} else {
		for _, e := range elements {
			var converted icicle.G2PointAffine
			G2AffineFromGnarkAffine(&e, &converted)
			newElements = append(newElements, converted)
		}
	}

	return newElements
}
