package bn254

import (
	"fmt"
	"unsafe"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	icicle "github.com/ingonyama-zk/icicle/goicicle/curves/bn254"
	goicicle "github.com/ingonyama-zk/icicle/goicicle"
)

func CopyPointsToDevice(points []bn254.G1Affine, pointsBytes int, copyDone chan unsafe.Pointer) {
	if pointsBytes == 0 {
		copyDone <- nil
	} else {
		devicePtr, _ := goicicle.CudaMalloc(pointsBytes)
		iciclePoints := BatchConvertFromG1Affine(points)
		goicicle.CudaMemCpyHtoD[icicle.G1PointAffine](devicePtr, iciclePoints, pointsBytes)
		
		copyDone <- devicePtr
	}
}

func CopyG2PointsToDevice(points []bn254.G2Affine, pointsBytes int, copyDone chan unsafe.Pointer) {
	if pointsBytes == 0 {
		copyDone <- nil
	} else {
		devicePtr, _ := goicicle.CudaMalloc(pointsBytes)
		iciclePoints := BatchConvertFromG2Affine(points)
		goicicle.CudaMemCpyHtoD[icicle.G2PointAffine](devicePtr, iciclePoints, pointsBytes)
		
		copyDone <- devicePtr
	}
}


func ScalarToGnarkFr(f *icicle.G1ScalarField) *fr.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := fr.LittleEndian.Element(&b32)

	if e != nil {
		panic(fmt.Sprintf("unable to create convert point %v got error %v", f, e))
	}

	return &v
}

func ScalarToGnarkFp(f *icicle.G1ScalarField) *fp.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := fp.LittleEndian.Element(&b32)

	if e != nil {
		panic(fmt.Sprintf("unable to create convert point %v got error %v", f, e))
	}

	return &v
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

func BatchConvertFromFrGnark[T icicle.G1BaseField | icicle.G1ScalarField](elements []fr.Element) []T {
	var newElements []T
	for _, e := range elements {
		converted := NewFieldFromFrGnark[T](e)
		newElements = append(newElements, *converted)
	}

	return newElements
}

func BatchConvertFromFrGnarkThreaded[T icicle.G1BaseField | icicle.G1ScalarField](elements []fr.Element, routines int) []T {
	var newElements []T

	if routines > 1 && routines <= len(elements) {
		channels := make([]chan []T, routines)
		for i := 0; i < routines; i++ {
			channels[i] = make(chan []T, 1)
		}

		convert := func(elements []fr.Element, chanIndex int) {
			var convertedElements []T
			for _, e := range elements {
				converted := NewFieldFromFrGnark[T](e)
				convertedElements = append(convertedElements, *converted)
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
			converted := NewFieldFromFrGnark[T](e)
			newElements = append(newElements, *converted)
		}
	}

	return newElements
}

func BatchConvertG1BaseFieldToFrGnark(elements []icicle.G1BaseField) []fr.Element {
	var newElements []fr.Element
	for _, e := range elements {
		converted := BaseFieldToGnarkFr(&e)
		newElements = append(newElements, *converted)
	}

	return newElements
}

func BatchConvertG1ScalarFieldToFrGnark(elements []icicle.G1ScalarField) []fr.Element {
	var newElements []fr.Element
	for _, e := range elements {
		converted := ScalarToGnarkFr(&e)
		newElements = append(newElements, *converted)
	}

	return newElements
}

func BatchConvertG1BaseFieldToFrGnarkThreaded(elements []icicle.G1BaseField, routines int) []fr.Element {
	var newElements []fr.Element

	if routines > 1 {
		channels := make([]chan []fr.Element, routines)
		for i := 0; i < routines; i++ {
			channels[i] = make(chan []fr.Element, 1)
		}

		convert := func(elements []icicle.G1BaseField, chanIndex int) {
			var convertedElements []fr.Element
			for _, e := range elements {
				converted := BaseFieldToGnarkFr(&e)
				convertedElements = append(convertedElements, *converted)
			}

			channels[chanIndex] <- convertedElements
		}

		batchLen := len(elements) / routines
		for i := 0; i < routines; i++ {
			elemsToConv := elements[batchLen*i : batchLen*(i+1)]
			go convert(elemsToConv, i)
		}

		for i := 0; i < routines; i++ {
			newElements = append(newElements, <-channels[i]...)
		}
	} else {
		for _, e := range elements {
			converted := BaseFieldToGnarkFr(&e)
			newElements = append(newElements, *converted)
		}
	}

	return newElements
}

func BatchConvertG1ScalarFieldToFrGnarkThreaded(elements []icicle.G1ScalarField, routines int) []fr.Element {
	var newElements []fr.Element

	if routines > 1 {
		channels := make([]chan []fr.Element, routines)
		for i := 0; i < routines; i++ {
			channels[i] = make(chan []fr.Element, 1)
		}

		convert := func(elements []icicle.G1ScalarField, chanIndex int) {
			var convertedElements []fr.Element
			for _, e := range elements {
				converted := ScalarToGnarkFr(&e)
				convertedElements = append(convertedElements, *converted)
			}

			channels[chanIndex] <- convertedElements
		}

		batchLen := len(elements) / routines
		for i := 0; i < routines; i++ {
			elemsToConv := elements[batchLen*i : batchLen*(i+1)]
			go convert(elemsToConv, i)
		}

		for i := 0; i < routines; i++ {
			newElements = append(newElements, <-channels[i]...)
		}
	} else {
		for _, e := range elements {
			converted := ScalarToGnarkFr(&e)
			newElements = append(newElements, *converted)
		}
	}

	return newElements
}

func BatchConvertFromG1Affine(elements []bn254.G1Affine) []icicle.G1PointAffine {
	var newElements []icicle.G1PointAffine
	for _, e := range elements {
		var newElement icicle.G1ProjectivePoint
		FromG1AffineGnark(&e, &newElement)

		newElements = append(newElements, *newElement.StripZ())
	}
	return newElements
}

func NewFieldFromFrGnark[T icicle.G1BaseField | icicle.G1ScalarField](element fr.Element) *T {
	s := icicle.ConvertUint64ArrToUint32Arr(element.Bits()) // get non-montgomry

	return &T{s}
}

func NewFieldFromFpGnark[T icicle.G1BaseField | icicle.G1ScalarField](element fp.Element) *T {
	s := icicle.ConvertUint64ArrToUint32Arr(element.Bits()) // get non-montgomry

	return &T{s}
}

func BaseFieldToGnarkFr(f *icicle.G1BaseField) *fr.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := fr.LittleEndian.Element(&b32)

	if e != nil {
		panic(fmt.Sprintf("unable to create convert point %v got error %v", f, e))
	}

	return &v
}

func BaseFieldToGnarkFp(f *icicle.G1BaseField) *fp.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := fp.LittleEndian.Element(&b32)

	if e != nil {
		panic(fmt.Sprintf("unable to create convert point %v got error %v", f, e))
	}

	return &v
}
