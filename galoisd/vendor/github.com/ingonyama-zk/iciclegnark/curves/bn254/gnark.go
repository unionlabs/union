package bn254

import (
	"encoding/binary"
	"errors"
	"fmt"
	"unsafe"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	curve "github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr"
	goicicle "github.com/ingonyama-zk/icicle/goicicle"
	icicle "github.com/ingonyama-zk/icicle/goicicle/curves/bn254"
)

type OnDeviceData struct {
	P    unsafe.Pointer
	Size int
}

func INttOnDevice(scalars_d, twiddles_d, cosetPowers_d unsafe.Pointer, size, sizeBytes int, isCoset bool) unsafe.Pointer {
	icicle.ReverseScalars(scalars_d, size)

	scalarsInterp := icicle.Interpolate(scalars_d, twiddles_d, cosetPowers_d, size, isCoset)

	return scalarsInterp
}

func NttOnDevice(scalars_out, scalars_d, twiddles_d, coset_powers_d unsafe.Pointer, size, twid_size, size_bytes int, isCoset bool) {
	res := icicle.Evaluate(scalars_out, scalars_d, twiddles_d, coset_powers_d, size, twid_size, isCoset)

	if res != 0 {
		fmt.Print("Issue evaluating")
	}

	icicle.ReverseScalars(scalars_out, size)
}

func MsmOnDevice(scalars_d, points_d unsafe.Pointer, count int, convert bool) (curve.G1Jac, unsafe.Pointer, error) {
	out_d, _ := goicicle.CudaMalloc(96)

	res := icicle.Commit(out_d, scalars_d, points_d, count, 10)
	if res != 0 {
		return curve.G1Jac{}, nil, fmt.Errorf("Failed to commit %d", res)
	}

	if convert {
		outHost := make([]icicle.G1ProjectivePoint, 1)
		goicicle.CudaMemCpyDtoH[icicle.G1ProjectivePoint](outHost, out_d, 96)

		return *G1ProjectivePointToGnarkJac(&outHost[0]), nil, nil
	}

	return curve.G1Jac{}, out_d, nil
}

func MsmG2OnDevice(scalars_d, points_d unsafe.Pointer, count int, convert bool) (curve.G2Jac, unsafe.Pointer, error) {
	out_d, _ := goicicle.CudaMalloc(192)

	res := icicle.CommitG2(out_d, scalars_d, points_d, count, 10)
	if res != 0 {
		return curve.G2Jac{}, nil, fmt.Errorf("Failed to commit %d", res)
	}

	if convert {
		outHost := make([]icicle.G2Point, 1)
		goicicle.CudaMemCpyDtoH[icicle.G2Point](outHost, out_d, 192)
		return *G2PointToGnarkJac(&outHost[0]), nil, nil
	}

	return curve.G2Jac{}, out_d, nil
}

func PolyOps(a_d, b_d, c_d, den_d unsafe.Pointer, size int) {
	ret := icicle.VecScalarMulMod(a_d, b_d, size)

	if ret != 0 {
		fmt.Print("Vector mult a*b issue")
	}
	ret = icicle.VecScalarSub(a_d, c_d, size)

	if ret != 0 {
		fmt.Print("Vector sub issue")
	}
	ret = icicle.VecScalarMulMod(a_d, den_d, size)

	if ret != 0 {
		fmt.Print("Vector mult a*den issue")
	}
}

func MontConvOnDevice(scalars_d unsafe.Pointer, size int, is_into bool) {
	if is_into {
		icicle.ToMontgomery(scalars_d, size)
	} else {
		icicle.FromMontgomery(scalars_d, size)
	}
}

func CopyToDevice(scalars []fr.Element, bytes int, copyDone chan unsafe.Pointer) {
	devicePtr, _ := goicicle.CudaMalloc(bytes)
	goicicle.CudaMemCpyHtoD[fr.Element](devicePtr, scalars, bytes)
	MontConvOnDevice(devicePtr, len(scalars), false)

	copyDone <- devicePtr
}

// g2

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

func ToGnarkFp(f *icicle.G2Element) *fp.Element {
	fb := f.ToBytesLe()
	var b32 [32]byte
	copy(b32[:], fb[:32])

	v, e := ElementWithOutConvertingToMontgomery(&b32) // cuda returns montgomery format
	//v2, e := fp.LittleEndian.Element(&b32) // TODO: revert back to this once cuda code is fixed.

	if e != nil {
		panic(fmt.Sprintf("unable to create convert point %v got error %v", f, e))
	}

	return &v
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

func BatchConvertFromG2AffineThreads(elements []bn254.G2Affine, routines int) []icicle.G2PointAffine {
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

/*
TODO: the following functions are due to a bug in the cuda code,
these fucntions should be deleted once cuda MsmG2 returns non montgomery format
*/
const (
	q0 uint64 = 4332616871279656263
	q1 uint64 = 10917124144477883021
	q2 uint64 = 13281191951274694749
	q3 uint64 = 3486998266802970665
)

func smallerThanModulus(z fp.Element) bool {
	return (z[3] < q3 || (z[3] == q3 && (z[2] < q2 || (z[2] == q2 && (z[1] < q1 || (z[1] == q1 && (z[0] < q0)))))))
}

func ElementWithOutConvertingToMontgomery(b *[32]byte) (fp.Element, error) {
	var z fp.Element
	z[0] = binary.LittleEndian.Uint64((*b)[0:8])
	z[1] = binary.LittleEndian.Uint64((*b)[8:16])
	z[2] = binary.LittleEndian.Uint64((*b)[16:24])
	z[3] = binary.LittleEndian.Uint64((*b)[24:32])

	if !smallerThanModulus(z) {
		return fp.Element{}, errors.New("invalid fp.Element encoding")
	}

	return z, nil
}
