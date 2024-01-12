package bn254

import (
	"fmt"
	"math"
	"unsafe"

	"github.com/consensys/gnark-crypto/ecc/bn254"
	"github.com/consensys/gnark-crypto/ecc/bn254/fp"
	goicicle "github.com/ingonyama-zk/icicle/goicicle"
	icicle "github.com/ingonyama-zk/icicle/goicicle/curves/bn254"
)

type OnDeviceData struct {
	P    unsafe.Pointer
	Size int
}

func INttOnDevice(scalars_d, twiddles_d, cosetPowers_d unsafe.Pointer, size, sizeBytes int, isCoset bool) unsafe.Pointer {
	ReverseScalars(scalars_d, size)

	scalarsInterp := icicle.Interpolate(scalars_d, twiddles_d, cosetPowers_d, size, isCoset)

	return scalarsInterp
}

func NttOnDevice(scalars_out, scalars_d, twiddles_d, coset_powers_d unsafe.Pointer, size, twid_size, size_bytes int, isCoset bool) {
	res := icicle.Evaluate(scalars_out, scalars_d, twiddles_d, coset_powers_d, size, twid_size, isCoset)

	if res != 0 {
		fmt.Print("Issue evaluating")
	}

	ReverseScalars(scalars_out, size)
}

func MsmOnDevice(scalars_d, points_d unsafe.Pointer, count int, convert bool) (bn254.G1Jac, unsafe.Pointer, error) {
	pointBytes := fp.Bytes * 3  // 3 Elements because of 3 coordinates
	out_d, _ := goicicle.CudaMalloc(pointBytes)

	icicle.Commit(out_d, scalars_d, points_d, count, 10)

	if convert {
		outHost := make([]icicle.G1ProjectivePoint, 1)
		goicicle.CudaMemCpyDtoH[icicle.G1ProjectivePoint](outHost, out_d, pointBytes)

		return *G1ProjectivePointToGnarkJac(&outHost[0]), nil, nil
	}

	return bn254.G1Jac{}, out_d, nil
}

func MsmG2OnDevice(scalars_d, points_d unsafe.Pointer, count int, convert bool) (bn254.G2Jac, unsafe.Pointer, error) {
	pointBytes := fp.Bytes * 6  // 6 Elements because of 3 coordinates each with real and imaginary elements
	out_d, _ := goicicle.CudaMalloc(pointBytes)

	icicle.CommitG2(out_d, scalars_d, points_d, count, 10)

	if convert {
		outHost := make([]icicle.G2Point, 1)
		goicicle.CudaMemCpyDtoH[icicle.G2Point](outHost, out_d, pointBytes)
		return *G2PointToGnarkJac(&outHost[0]), nil, nil
	}

	return bn254.G2Jac{}, out_d, nil
}

func GenerateTwiddleFactors(size int, inverse bool) (unsafe.Pointer, error) {
	om_selector := int(math.Log(float64(size)) / math.Log(2))
	return icicle.GenerateTwiddles(size, om_selector, inverse)
}

func ReverseScalars(ptr unsafe.Pointer, size int) error {
	if success, err := icicle.ReverseScalars(ptr, size); success != 0 {
		return err
	}
	
	return nil
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
