package witness

import (
	"errors"
	"math/big"
	"reflect"

	"github.com/consensys/gnark-crypto/ecc"
	fr_bls12377 "github.com/consensys/gnark-crypto/ecc/bls12-377/fr"
	fr_bls12381 "github.com/consensys/gnark-crypto/ecc/bls12-381/fr"
	fr_bls24315 "github.com/consensys/gnark-crypto/ecc/bls24-315/fr"
	fr_bls24317 "github.com/consensys/gnark-crypto/ecc/bls24-317/fr"
	fr_bn254 "github.com/consensys/gnark-crypto/ecc/bn254/fr"
	fr_bw6633 "github.com/consensys/gnark-crypto/ecc/bw6-633/fr"
	fr_bw6761 "github.com/consensys/gnark-crypto/ecc/bw6-761/fr"
	"github.com/consensys/gnark/internal/tinyfield"
	"github.com/consensys/gnark/internal/utils"
)

func newVector(field *big.Int, size int) (any, error) {
	curveID := utils.FieldToCurve(field)
	switch curveID {
	case ecc.BN254:
		return make(fr_bn254.Vector, size), nil
	case ecc.BLS12_377:
		return make(fr_bls12377.Vector, size), nil
	case ecc.BLS12_381:
		return make(fr_bls12381.Vector, size), nil
	case ecc.BW6_761:
		return make(fr_bw6761.Vector, size), nil
	case ecc.BLS24_317:
		return make(fr_bls24317.Vector, size), nil
	case ecc.BLS24_315:
		return make(fr_bls24315.Vector, size), nil
	case ecc.BW6_633:
		return make(fr_bw6633.Vector, size), nil
	default:
		if field.Cmp(tinyfield.Modulus()) == 0 {
			return make(tinyfield.Vector, size), nil
		} else {
			return nil, errors.New("unsupported modulus")
		}
	}
}

func newFrom(from any, n int) (any, error) {
	switch wt := from.(type) {
	case fr_bn254.Vector:
		a := make(fr_bn254.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bls12377.Vector:
		a := make(fr_bls12377.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bls12381.Vector:
		a := make(fr_bls12381.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bw6761.Vector:
		a := make(fr_bw6761.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bls24317.Vector:
		a := make(fr_bls24317.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bls24315.Vector:
		a := make(fr_bls24315.Vector, n)
		copy(a, wt)
		return a, nil
	case fr_bw6633.Vector:
		a := make(fr_bw6633.Vector, n)
		copy(a, wt)
		return a, nil
	case tinyfield.Vector:
		a := make(tinyfield.Vector, n)
		copy(a, wt)
		return a, nil
	default:
		return nil, errors.New("unsupported modulus")
	}
}

func leafType(v any) reflect.Type {
	switch v.(type) {
	case fr_bn254.Vector:
		return reflect.TypeOf(fr_bn254.Element{})
	case fr_bls12377.Vector:
		return reflect.TypeOf(fr_bls12377.Element{})
	case fr_bls12381.Vector:
		return reflect.TypeOf(fr_bls12381.Element{})
	case fr_bw6761.Vector:
		return reflect.TypeOf(fr_bw6761.Element{})
	case fr_bls24317.Vector:
		return reflect.TypeOf(fr_bls24317.Element{})
	case fr_bls24315.Vector:
		return reflect.TypeOf(fr_bls24315.Element{})
	case fr_bw6633.Vector:
		return reflect.TypeOf(fr_bw6633.Element{})
	case tinyfield.Vector:
		return reflect.TypeOf(tinyfield.Element{})
	default:
		panic("invalid input")
	}
}

func set(v any, index int, value any) error {
	switch pv := v.(type) {
	case fr_bn254.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bls12377.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bls12381.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bw6761.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bls24317.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bls24315.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case fr_bw6633.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	case tinyfield.Vector:
		if index >= len(pv) {
			return errors.New("out of bounds")
		}
		_, err := pv[index].SetInterface(value)
		return err
	default:
		panic("invalid input")
	}
}

func iterate(v any) chan any {
	chValues := make(chan any)
	switch pv := v.(type) {
	case fr_bn254.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bls12377.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bls12381.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bw6761.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bls24317.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bls24315.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case fr_bw6633.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	case tinyfield.Vector:
		go func() {
			for i := 0; i < len(pv); i++ {
				chValues <- &(pv)[i]
			}
			close(chValues)
		}()
	default:
		panic("invalid input")
	}
	return chValues
}

func resize(v any, n int) any {
	switch v.(type) {
	case fr_bn254.Vector:
		return make(fr_bn254.Vector, n)
	case fr_bls12377.Vector:
		return make(fr_bls12377.Vector, n)
	case fr_bls12381.Vector:
		return make(fr_bls12381.Vector, n)
	case fr_bw6761.Vector:
		return make(fr_bw6761.Vector, n)
	case fr_bls24317.Vector:
		return make(fr_bls24317.Vector, n)
	case fr_bls24315.Vector:
		return make(fr_bls24315.Vector, n)
	case fr_bw6633.Vector:
		return make(fr_bw6633.Vector, n)
	case tinyfield.Vector:
		return make(tinyfield.Vector, n)
	default:
		panic("invalid input")
	}
}
